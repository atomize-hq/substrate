mod export;
mod schema;

use std::collections::{BTreeMap, BTreeSet};

use crate::input::BundleSession;
use crate::{
    context::assemble_context, context::collect_command_observations, context::CommandObservation,
    context::ContextPack, context::focusable_directive_rows, inference::infer_task_frame,
    scoring::DriftStateHint, scoring::ScoredDrift,
};
use agent_session_compactor::{CompactionKind, CompactionRow, RowRef, UserMessageRole};
use camino::Utf8PathBuf;

pub use export::{
    export_checkpoints, summarize_checkpoint_diagnostics, CheckpointDiagnosticStats,
    ConfidenceDistribution, ExportError, ExportResult,
};
pub use schema::{
    Checkpoint, CheckpointBoundary, CheckpointDiagnostics, Confidence, DriftClass, DriftScore,
    DriftState, EvidenceRef, TaskFrame, TurnActivityMix, TurnContext, TurnExecutionMode,
};

const MAX_ROWS_PER_CHECKPOINT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointAnalysis {
    pub session_id: String,
    pub ordinal: usize,
    pub current: CheckpointSlice,
    pub previous: Option<CheckpointSlice>,
    pub interval: IntervalSlice,
    pub turn_context: TurnContext,
    pub repetition: RepetitionSlice,
    pub task_frame_delta: TaskFrameDelta,
    pub recovery: RecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointSlice {
    pub window: BundleSession,
    pub context: ContextPack,
    pub task_frame: TaskFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntervalSlice {
    pub archival_rows: Vec<CompactionRow>,
    pub compact_rows: Vec<CompactionRow>,
    pub command_observations: Vec<CommandObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepetitionSlice {
    pub compact_rows: Vec<CompactionRow>,
    pub repeated_verification_loops: Vec<RepeatedCommandLoop>,
    pub repeated_failure_loops: Vec<RepeatedFailureLoop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepeatedCommandLoop {
    pub raw_command: String,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepeatedFailureLoop {
    pub text_hash_hex: String,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutcomeEvidenceKind {
    Failure,
    Neutral,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TaskFrameDelta {
    pub task_frame_transitioned: bool,
    pub working_set_changed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RecoveryState {
    pub interval_verification_command_count: usize,
    pub clean_verification_interval: bool,
    pub recovered_from_thrash: bool,
    pub active_repeated_verification: bool,
    pub active_repeated_failure: bool,
}

pub(crate) fn checkpoint_analyses(session: &BundleSession) -> Vec<CheckpointAnalysis> {
    let mut analyses = Vec::new();
    let mut previous = None;
    let prompts_observed_in_session = prompts_observed_in_session(session);
    let mut checkpoints_in_turn = BTreeMap::<TurnSliceIdentity, usize>::new();

    for (index, window) in checkpoint_windows(session).into_iter().enumerate() {
        let context = assemble_context(&window);
        let task_frame = infer_task_frame(&context);
        let current = CheckpointSlice {
            window,
            context,
            task_frame,
        };
        let interval = interval_slice(previous.as_ref(), &current);
        let turn_context = turn_context(
            session,
            &current,
            &interval,
            prompts_observed_in_session,
            &mut checkpoints_in_turn,
        );
        let repetition = repetition_slice(&current);
        let task_frame_delta = task_frame_delta(previous.as_ref(), &current);
        let recovery = recovery_state(&current, &interval, &repetition);

        analyses.push(CheckpointAnalysis {
            session_id: current.window.session_id.clone(),
            ordinal: index + 1,
            current: current.clone(),
            previous: previous.clone(),
            interval,
            turn_context,
            repetition,
            task_frame_delta,
            recovery,
        });

        previous = Some(current);
    }

    analyses
}

pub(crate) fn build_session_checkpoint_from_analysis(
    analysis: &CheckpointAnalysis,
    task_frame: &TaskFrame,
    drift_scores: Vec<DriftScore>,
) -> Checkpoint {
    build_session_checkpoint_from_analysis_with_ordinal(
        analysis,
        analysis.ordinal,
        task_frame,
        drift_scores,
    )
}

pub(crate) fn assign_drift_states(
    drift_scores: Vec<ScoredDrift>,
    previous_drift_scores: Option<&[DriftScore]>,
) -> Vec<DriftScore> {
    drift_scores
        .into_iter()
        .map(|scored| DriftScore {
            state: drift_state_for_score(&scored.score, scored.state_hint, previous_drift_scores),
            ..scored.score
        })
        .collect()
}

fn build_session_checkpoint_from_analysis_with_ordinal(
    analysis: &CheckpointAnalysis,
    ordinal: usize,
    task_frame: &TaskFrame,
    drift_scores: Vec<DriftScore>,
) -> Checkpoint {
    let boundary = checkpoint_boundary(&analysis.current.window);
    let diagnostics = checkpoint_diagnostics_from_analysis(analysis, task_frame, &drift_scores);
    let expected_next_step = expected_next_step(task_frame);
    Checkpoint {
        schema_version: "v0.4".to_string(),
        session_id: analysis.session_id.clone(),
        checkpoint_id: format!("{}:{ordinal:04}", analysis.session_id),
        ordinal,
        boundary,
        turn_context: Some(analysis.turn_context.clone()),
        diagnostics,
        task_frame: task_frame.clone(),
        flagged: drift_scores.iter().any(|score| score.flagged),
        drift_scores,
        expected_next_step,
    }
}

fn drift_state_for_score(
    score: &DriftScore,
    state_hint: DriftStateHint,
    previous_drift_scores: Option<&[DriftScore]>,
) -> DriftState {
    if score.flagged {
        return DriftState::Active;
    }

    if !matches!(state_hint, DriftStateHint::HistoricalContext) {
        return DriftState::Cleared;
    }

    match previous_state_for_class(previous_drift_scores, score.class) {
        Some(DriftState::Active) => DriftState::Recovered,
        _ => DriftState::HistoricalOnly,
    }
}

fn previous_state_for_class(
    previous_drift_scores: Option<&[DriftScore]>,
    class: DriftClass,
) -> Option<DriftState> {
    previous_drift_scores?
        .iter()
        .find(|score| score.class == class)
        .map(|score| score.state)
}

pub fn build_session_checkpoint(
    session: &BundleSession,
    ordinal: usize,
    task_frame: &TaskFrame,
    drift_scores: Vec<DriftScore>,
) -> Checkpoint {
    let analyses = checkpoint_analyses(session);
    let analysis = analyses
        .get(ordinal.checked_sub(1).unwrap_or(usize::MAX))
        .or_else(|| analyses.last())
        .unwrap_or_else(|| {
            panic!(
                "session {} must contain at least one checkpoint window",
                session.session_id
            )
        });
    build_session_checkpoint_from_analysis_with_ordinal(analysis, ordinal, task_frame, drift_scores)
}

fn checkpoint_diagnostics_from_analysis(
    analysis: &CheckpointAnalysis,
    task_frame: &TaskFrame,
    drift_scores: &[DriftScore],
) -> CheckpointDiagnostics {
    CheckpointDiagnostics {
        task_frame_transitioned: analysis.task_frame_delta.task_frame_transitioned,
        working_set_changed: analysis.task_frame_delta.working_set_changed,
        interval_command_count: analysis.interval.command_observations.len(),
        interval_verification_command_count: analysis.recovery.interval_verification_command_count,
        evidence_item_count: evidence_item_count(task_frame, drift_scores),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TurnSliceIdentity {
    turn_id: String,
    start: (Utf8PathBuf, usize, usize),
}

struct CurrentTurnSlice<'a> {
    rows: &'a [CompactionRow],
    turn_id: Option<&'a str>,
    turn_ordinal: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TurnActivityAnalysis {
    mix: TurnActivityMix,
    command_primary_counts: CommandPrimaryCounts,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CommandPrimaryCounts {
    read_like: usize,
    write_like: usize,
    verification_like: usize,
    other: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandPrimaryKind {
    ReadLike,
    WriteLike,
    VerificationLike,
    Other,
}

fn turn_context(
    session: &BundleSession,
    current: &CheckpointSlice,
    interval: &IntervalSlice,
    prompts_observed_in_session: usize,
    checkpoints_in_turn: &mut BTreeMap<TurnSliceIdentity, usize>,
) -> TurnContext {
    let turn_slice = current_turn_slice(session, current, interval);
    let checkpoints_in_turn = match turn_slice_identity(&turn_slice) {
        Some(identity) => {
            let count = checkpoints_in_turn.entry(identity).or_default();
            *count += 1;
            *count
        }
        None => 1,
    };
    let activity = turn_activity_analysis(turn_slice.rows);
    let execution_mode = turn_execution_mode(&turn_slice, checkpoints_in_turn, &activity);

    TurnContext {
        turn_id: turn_slice.turn_id.map(str::to_owned),
        turn_ordinal: turn_slice.turn_ordinal,
        rows_since_turn_start: turn_slice.rows.len(),
        seconds_since_turn_start: seconds_since_turn_start(turn_slice.rows),
        checkpoints_in_turn,
        prompts_observed_in_session,
        execution_mode,
        activity_mix: activity.mix,
    }
}

fn turn_activity_analysis(rows: &[CompactionRow]) -> TurnActivityAnalysis {
    let command_observations = collect_command_observations(rows);
    let mut command_primary_counts = CommandPrimaryCounts::default();
    for command in &command_observations {
        match primary_command_kind(command) {
            CommandPrimaryKind::ReadLike => command_primary_counts.read_like += 1,
            CommandPrimaryKind::WriteLike => command_primary_counts.write_like += 1,
            CommandPrimaryKind::VerificationLike => {
                command_primary_counts.verification_like += 1;
            }
            CommandPrimaryKind::Other => command_primary_counts.other += 1,
        }
    }

    TurnActivityAnalysis {
        mix: TurnActivityMix {
            directive_row_count: focusable_directive_rows(rows).count(),
            assistant_message_count: rows
                .iter()
                .filter(|row| row.kind == CompactionKind::AssistantMessage)
                .count(),
            tool_call_count: rows
                .iter()
                .filter(|row| row.kind == CompactionKind::ToolCall)
                .count(),
            read_like_command_count: command_observations
                .iter()
                .filter(|command| command.read_like)
                .count(),
            write_like_command_count: command_observations
                .iter()
                .filter(|command| command.write_like)
                .count(),
            verification_like_command_count: command_observations
                .iter()
                .filter(|command| command.verification_like)
                .count(),
            tool_output_count: rows
                .iter()
                .filter(|row| row.kind == CompactionKind::ToolOutput)
                .count(),
        },
        command_primary_counts,
    }
}

fn primary_command_kind(command: &CommandObservation) -> CommandPrimaryKind {
    if command.verification_like {
        CommandPrimaryKind::VerificationLike
    } else if command.write_like {
        CommandPrimaryKind::WriteLike
    } else if command.read_like {
        CommandPrimaryKind::ReadLike
    } else {
        CommandPrimaryKind::Other
    }
}

fn turn_execution_mode(
    turn_slice: &CurrentTurnSlice<'_>,
    checkpoints_in_turn: usize,
    activity: &TurnActivityAnalysis,
) -> TurnExecutionMode {
    let directive_and_assistant_rows =
        activity.mix.directive_row_count + activity.mix.assistant_message_count;
    if activity.mix.tool_call_count == 0
        || (checkpoints_in_turn <= 1 && directive_and_assistant_rows > activity.mix.tool_call_count)
    {
        return TurnExecutionMode::Conversational;
    }

    if activity.mix.tool_call_count > 0 && checkpoints_in_turn > 1 && turn_slice.turn_id.is_some() {
        return TurnExecutionMode::Autonomous;
    }

    if verification_like_is_strict_plurality(&activity.command_primary_counts) {
        return TurnExecutionMode::VerificationHeavy;
    }

    TurnExecutionMode::Mixed
}

fn verification_like_is_strict_plurality(counts: &CommandPrimaryCounts) -> bool {
    counts.verification_like > 0
        && counts.verification_like > counts.read_like
        && counts.verification_like > counts.write_like
        && counts.verification_like > counts.other
}

fn current_turn_slice<'a>(
    session: &'a BundleSession,
    current: &'a CheckpointSlice,
    interval: &'a IntervalSlice,
) -> CurrentTurnSlice<'a> {
    let rows = preferred_rows(&current.window.archival_rows, &current.window.compact_rows);
    let Some(turn_id) = rows.iter().rev().find_map(|row| row.turn_id.as_deref()) else {
        return CurrentTurnSlice {
            rows: preferred_rows(&interval.archival_rows, &interval.compact_rows),
            turn_id: None,
            turn_ordinal: 0,
        };
    };

    let start_index = rows
        .iter()
        .rposition(|row| matches!(row.turn_id.as_deref(), Some(id) if id != turn_id))
        .map_or(0, |index| index + 1);

    CurrentTurnSlice {
        rows: &rows[start_index..],
        turn_id: Some(turn_id),
        turn_ordinal: turn_ordinal(session, turn_id),
    }
}

fn turn_slice_identity(turn_slice: &CurrentTurnSlice<'_>) -> Option<TurnSliceIdentity> {
    Some(TurnSliceIdentity {
        turn_id: turn_slice.turn_id?.to_string(),
        start: row_key(turn_slice.rows.first()?),
    })
}

fn turn_ordinal(session: &BundleSession, current_turn_id: &str) -> usize {
    let mut seen = BTreeSet::new();
    let mut ordinal = 0;

    for row in preferred_rows(&session.archival_rows, &session.compact_rows) {
        let Some(turn_id) = row.turn_id.as_deref() else {
            continue;
        };
        if seen.insert(turn_id) {
            ordinal += 1;
        }
        if turn_id == current_turn_id {
            return ordinal;
        }
    }

    0
}

fn seconds_since_turn_start(rows: &[CompactionRow]) -> Option<i64> {
    let start = rows.iter().find_map(|row| row.timestamp)?;
    let end = rows.last().and_then(|row| row.timestamp)?;
    Some((end - start).whole_seconds().max(0))
}

fn prompts_observed_in_session(session: &BundleSession) -> usize {
    session
        .compact_rows
        .iter()
        .filter(|row| row.kind == CompactionKind::UserMessage)
        .filter(|row| !is_synthetic_user_message(row))
        .filter(|row| {
            matches!(
                row.user_message_role.unwrap_or(UserMessageRole::Unknown),
                UserMessageRole::Prompt
            )
        })
        .count()
}

fn preferred_rows<'a>(
    archival_rows: &'a [CompactionRow],
    compact_rows: &'a [CompactionRow],
) -> &'a [CompactionRow] {
    if archival_rows.is_empty() {
        compact_rows
    } else {
        archival_rows
    }
}

fn interval_slice(previous: Option<&CheckpointSlice>, current: &CheckpointSlice) -> IntervalSlice {
    let archival_start = previous.map_or(0, |slice| slice.window.archival_rows.len());
    let interval_start = previous.map_or(0, |slice| slice.window.compact_rows.len());
    let archival_rows = current.window.archival_rows[archival_start..].to_vec();
    let compact_rows = current.window.compact_rows[interval_start..].to_vec();
    let command_observations = collect_command_observations(&compact_rows);

    IntervalSlice {
        archival_rows,
        compact_rows,
        command_observations,
    }
}

fn repetition_slice(current: &CheckpointSlice) -> RepetitionSlice {
    let mut repeated_commands = BTreeMap::<String, Vec<EvidenceRef>>::new();
    let archival_commands = collect_command_observations(&current.window.archival_rows);
    for command in archival_commands
        .iter()
        .filter(|command| command.verification_like)
    {
        repeated_commands
            .entry(command.raw_command.clone())
            .or_default()
            .extend(command.evidence.clone());
    }

    let mut repeated_failures = BTreeMap::<String, Vec<EvidenceRef>>::new();
    for row in current
        .window
        .archival_rows
        .iter()
        .filter(|row| is_failure_row(row))
    {
        repeated_failures
            .entry(row.text_hash_hex.clone())
            .or_default()
            .push(EvidenceRef {
                row: RowRef::from_row(row),
                reason: "repeated failure evidence".to_string(),
            });
    }

    RepetitionSlice {
        compact_rows: current.window.compact_rows.clone(),
        repeated_verification_loops: repeated_commands
            .into_iter()
            .filter_map(|(raw_command, evidence)| {
                (evidence.len() >= 3).then_some(RepeatedCommandLoop {
                    raw_command,
                    evidence,
                })
            })
            .collect(),
        repeated_failure_loops: repeated_failures
            .into_iter()
            .filter_map(|(text_hash_hex, evidence)| {
                (evidence.len() >= 2).then_some(RepeatedFailureLoop {
                    text_hash_hex,
                    evidence,
                })
            })
            .collect(),
    }
}

fn task_frame_delta(
    previous: Option<&CheckpointSlice>,
    current: &CheckpointSlice,
) -> TaskFrameDelta {
    let Some(previous) = previous else {
        return TaskFrameDelta::default();
    };

    TaskFrameDelta {
        task_frame_transitioned: task_frame_identity(&current.task_frame)
            != task_frame_identity(&previous.task_frame),
        working_set_changed: working_set_identity(&current.task_frame)
            != working_set_identity(&previous.task_frame),
    }
}

fn recovery_state(
    current: &CheckpointSlice,
    interval: &IntervalSlice,
    repetition: &RepetitionSlice,
) -> RecoveryState {
    let interval_verification_command_count = interval
        .command_observations
        .iter()
        .filter(|command| command.verification_like)
        .count();
    let preserves_out_of_scope = preserves_out_of_scope_thrash(current, interval);
    let failure_touches_interval =
        failure_loops_touch_interval(&repetition.repeated_failure_loops, interval);
    let clean_verification_interval = interval_verification_command_count > 0
        && !preserves_out_of_scope
        && !failure_touches_interval;

    RecoveryState {
        interval_verification_command_count,
        clean_verification_interval,
        recovered_from_thrash: clean_verification_interval
            && (!repetition.repeated_verification_loops.is_empty()
                || !repetition.repeated_failure_loops.is_empty()),
        active_repeated_verification: interval_verification_command_count > 0
            && !repetition.repeated_verification_loops.is_empty()
            && preserves_out_of_scope,
        active_repeated_failure: failure_touches_interval,
    }
}

fn preserves_out_of_scope_thrash(current: &CheckpointSlice, interval: &IntervalSlice) -> bool {
    let mut expected = current.task_frame.truth_artifacts.clone();
    expected.extend(
        current
            .context
            .working_set_paths
            .iter()
            .filter(|path| {
                path.source != "observed_command"
                    || path
                        .evidence
                        .iter()
                        .any(|evidence| !interval_contains_evidence(interval, evidence))
            })
            .map(|path| path.path.clone()),
    );
    expected.sort();
    expected.dedup();

    interval.command_observations.iter().any(|command| {
        if command.paths.is_empty() || (!command.write_like && !command.verification_like) {
            return false;
        }

        !command.paths.iter().all(|path| {
            expected.iter().any(|expected_path| {
                path == expected_path
                    || path.starts_with(expected_path)
                    || expected_path.starts_with(path)
            })
        })
    })
}

fn failure_loops_touch_interval(loops: &[RepeatedFailureLoop], interval: &IntervalSlice) -> bool {
    loops.iter().any(|failure_loop| {
        failure_loop
            .evidence
            .iter()
            .any(|evidence| interval_contains_evidence(interval, evidence))
    })
}

fn interval_contains_evidence(interval: &IntervalSlice, evidence: &EvidenceRef) -> bool {
    let interval_row_keys = interval
        .archival_rows
        .iter()
        .map(row_key)
        .collect::<BTreeSet<_>>();
    interval_row_keys.contains(&row_ref_key(&evidence.row))
}

pub fn checkpoint_windows(session: &BundleSession) -> Vec<BundleSession> {
    let Some(last_index) = session.compact_rows.len().checked_sub(1) else {
        if session.archival_rows.is_empty() {
            return Vec::new();
        }
        return vec![session.clone()];
    };

    let mut phase_end_indices = checkpoint_end_indices(&session.compact_rows);
    if phase_end_indices.last().copied() != Some(last_index) {
        phase_end_indices.push(last_index);
    }

    let mut windows = Vec::with_capacity(phase_end_indices.len());
    for end_index in phase_end_indices {
        let compact_rows = session.compact_rows[..=end_index].to_vec();
        let end_row = compact_rows
            .last()
            .expect("checkpoint window must contain at least one compact row");
        let end_key = row_key(end_row);
        let archival_rows = session
            .archival_rows
            .iter()
            .take_while(|row| row_key(row) <= end_key)
            .cloned()
            .collect::<Vec<_>>();
        windows.push(BundleSession {
            session_id: session.session_id.clone(),
            archival_rows,
            compact_rows,
        });
    }

    windows
}

fn checkpoint_boundary(session: &BundleSession) -> CheckpointBoundary {
    let start_row = session
        .archival_rows
        .first()
        .or_else(|| session.compact_rows.first())
        .expect("session must contain rows");
    let end_row = session
        .archival_rows
        .last()
        .or_else(|| session.compact_rows.last())
        .expect("session must contain rows");
    CheckpointBoundary {
        start: RowRef::from_row(start_row),
        end: RowRef::from_row(end_row),
    }
}

fn expected_next_step(task_frame: &TaskFrame) -> String {
    task_frame
        .verification_commands
        .first()
        .cloned()
        .unwrap_or_else(|| "continue on the current task frame".to_string())
}

fn task_frame_identity(task_frame: &TaskFrame) -> String {
    serde_json::to_string(&(
        &task_frame.objective,
        &task_frame.truth_artifacts,
        &task_frame.working_set_paths,
        &task_frame.tools,
        &task_frame.command_families,
        &task_frame.verification_commands,
    ))
    .expect("task frame identity should serialize")
}

fn working_set_identity(task_frame: &TaskFrame) -> BTreeSet<&str> {
    task_frame
        .working_set_paths
        .iter()
        .map(String::as_str)
        .collect()
}

fn evidence_item_count(task_frame: &TaskFrame, drift_scores: &[DriftScore]) -> usize {
    let mut seen = BTreeSet::new();
    for evidence in task_frame
        .supporting_evidence
        .iter()
        .chain(task_frame.counter_evidence.iter())
        .chain(drift_scores.iter().flat_map(|score| score.evidence.iter()))
    {
        seen.insert((
            evidence.row.source_file.clone(),
            evidence.row.event_index,
            evidence.row.row_ordinal,
            evidence.reason.clone(),
        ));
    }
    seen.len()
}

fn checkpoint_end_indices(rows: &[CompactionRow]) -> Vec<usize> {
    if rows.is_empty() {
        return Vec::new();
    }

    let mut phase_ends = Vec::new();
    let mut saw_activity = false;
    let mut saw_objective = false;
    for (index, row) in rows.iter().enumerate() {
        if objective_row(row) {
            saw_objective = true;
        }
        if index > 0 && row_starts_new_phase(row) && saw_activity && saw_objective {
            phase_ends.push(index - 1);
            saw_activity = false;
        }
        if row_is_activity(row) {
            saw_activity = true;
        }
    }

    let mut normalized = Vec::new();
    let mut last_end = None;
    for end_index in phase_ends
        .into_iter()
        .chain(std::iter::once(rows.len() - 1))
    {
        let start_index = last_end.map_or(0, |end| end + 1);
        if end_index >= start_index + MAX_ROWS_PER_CHECKPOINT {
            let mut chunk_end = start_index + MAX_ROWS_PER_CHECKPOINT - 1;
            while chunk_end < end_index {
                normalized.push(chunk_end);
                chunk_end += MAX_ROWS_PER_CHECKPOINT;
            }
        }
        if normalized.last().copied() != Some(end_index) {
            normalized.push(end_index);
        }
        last_end = Some(end_index);
    }

    normalized
}

fn row_starts_new_phase(row: &CompactionRow) -> bool {
    matches!(
        row.kind,
        CompactionKind::UserMessage
            | CompactionKind::AssistantMessage
            | CompactionKind::DeveloperMessage
            | CompactionKind::SystemMessage
    ) && row_text_is_focusable(row)
}

fn objective_row(row: &CompactionRow) -> bool {
    matches!(
        row.kind,
        CompactionKind::UserMessage | CompactionKind::DeveloperMessage
    ) && row_text_is_focusable(row)
}

fn row_is_activity(row: &CompactionRow) -> bool {
    matches!(
        row.kind,
        CompactionKind::ToolCall
            | CompactionKind::ToolOutput
            | CompactionKind::Error
            | CompactionKind::Reasoning
            | CompactionKind::Unknown
    )
}

fn row_text_is_focusable(row: &CompactionRow) -> bool {
    row.text.len() <= 2_000
        && !row.text.trim().is_empty()
        && !row.text.contains("AGENTS.md instructions")
        && !row.text.contains("<skill>")
        && !row.text.contains("Available skills")
        && row.text != "[encrypted_reasoning]"
}

fn is_synthetic_user_message(row: &CompactionRow) -> bool {
    row.text.contains("AGENTS.md instructions")
        || row.text.contains("<skill>")
        || row.text.contains("Available skills")
}

fn row_key(row: &CompactionRow) -> (Utf8PathBuf, usize, usize) {
    (row.source_file.clone(), row.event_index, row.row_ordinal)
}

fn row_ref_key(row: &RowRef) -> (Utf8PathBuf, usize, usize) {
    (row.source_file.clone(), row.event_index, row.row_ordinal)
}

fn classify_outcome_evidence(row: &CompactionRow) -> OutcomeEvidenceKind {
    if row.text.trim().is_empty() {
        return OutcomeEvidenceKind::Neutral;
    }

    match row.kind {
        CompactionKind::Error => OutcomeEvidenceKind::Failure,
        CompactionKind::ToolOutput if tool_output_is_unambiguous_failure(&row.text) => {
            OutcomeEvidenceKind::Failure
        }
        _ => OutcomeEvidenceKind::Neutral,
    }
}

fn tool_output_is_unambiguous_failure(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("error:"))
        || trimmed
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("Exit code: "))
            .and_then(|code| code.trim().parse::<i32>().ok())
            .is_some_and(|code| code != 0)
}

fn is_failure_row(row: &CompactionRow) -> bool {
    matches!(classify_outcome_evidence(row), OutcomeEvidenceKind::Failure)
}

#[cfg(test)]
mod tests {
    use super::tool_output_is_unambiguous_failure;

    #[test]
    fn tool_output_unambiguous_failure_subset_stays_tiny_and_deterministic() {
        for text in [
            "error: failed to compile analyzer",
            "Exit code: 1",
            "Exit code: 101\nWall time: 1.2 seconds\nOutput:\n",
            "Exit code: 128",
        ] {
            assert!(tool_output_is_unambiguous_failure(text), "{text}");
        }

        for text in [
            "",
            "Exit code: 0",
            "Plan updated",
            "function_call_output: wrote analyzer patch",
        ] {
            assert!(!tool_output_is_unambiguous_failure(text), "{text}");
        }
    }
}
