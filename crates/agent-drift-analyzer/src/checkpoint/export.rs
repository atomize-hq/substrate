use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;

use agent_session_compactor::{CompactionKind, CompactionRow, RowRef, UserMessageRole};
use camino::{Utf8Path, Utf8PathBuf};
use time::OffsetDateTime;

use crate::checkpoint::{Checkpoint, Confidence, DriftClass, TaskFrame};
use crate::input::BundleSession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportResult {
    pub checkpoints_path: Utf8PathBuf,
    pub summary_path: Utf8PathBuf,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfidenceDistribution {
    pub low: usize,
    pub medium: usize,
    pub high: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointDiagnosticStats {
    pub checkpoint_count: usize,
    pub flagged_checkpoint_count: usize,
    pub drift_class_flagged_counts: BTreeMap<DriftClass, usize>,
    pub task_frame_transition_count: usize,
    pub confidence_distribution: ConfidenceDistribution,
    pub working_set_change_count: usize,
    pub adjacent_checkpoint_pair_count: usize,
    pub total_evidence_item_count: usize,
}

impl CheckpointDiagnosticStats {
    pub fn flagged_checkpoint_rate(&self) -> Option<f64> {
        ratio(self.flagged_checkpoint_count, self.checkpoint_count)
    }

    pub fn drift_class_flagged_rate(&self, class: DriftClass) -> Option<f64> {
        let count = self
            .drift_class_flagged_counts
            .get(&class)
            .copied()
            .unwrap_or(0);
        ratio(count, self.checkpoint_count)
    }

    pub fn working_set_churn(&self) -> Option<f64> {
        ratio(
            self.working_set_change_count,
            self.adjacent_checkpoint_pair_count,
        )
    }

    pub fn average_evidence_items_per_checkpoint(&self) -> Option<f64> {
        average(self.total_evidence_item_count, self.checkpoint_count)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("failed to create analyzer output directory {path}: {source}")]
    CreateOutputDirectory {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write analyzer artifact {path}: {source}")]
    WriteArtifact {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to serialize analyzer artifact {path}: {source}")]
    SerializeArtifact {
        path: Utf8PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub fn export_checkpoints(
    output_dir: &Utf8Path,
    sessions: &[BundleSession],
    checkpoints: &[Checkpoint],
) -> Result<ExportResult, ExportError> {
    fs::create_dir_all(output_dir).map_err(|source| ExportError::CreateOutputDirectory {
        path: output_dir.to_owned(),
        source,
    })?;

    let checkpoints_path = output_dir.join("checkpoints.jsonl");
    let summary_path = output_dir.join("summary.md");

    let mut checkpoints_file =
        fs::File::create(&checkpoints_path).map_err(|source| ExportError::WriteArtifact {
            path: checkpoints_path.clone(),
            source,
        })?;
    let mut sorted = checkpoints.to_vec();
    sorted.sort_by(|left, right| {
        left.session_id
            .cmp(&right.session_id)
            .then_with(|| left.ordinal.cmp(&right.ordinal))
    });
    for checkpoint in &sorted {
        let line =
            serde_json::to_string(checkpoint).map_err(|source| ExportError::SerializeArtifact {
                path: checkpoints_path.clone(),
                source,
            })?;
        writeln!(checkpoints_file, "{line}").map_err(|source| ExportError::WriteArtifact {
            path: checkpoints_path.clone(),
            source,
        })?;
    }

    let summary = render_summary(sessions, &sorted);
    fs::write(&summary_path, summary).map_err(|source| ExportError::WriteArtifact {
        path: summary_path.clone(),
        source,
    })?;

    Ok(ExportResult {
        checkpoints_path,
        summary_path,
    })
}

pub fn summarize_checkpoint_diagnostics(checkpoints: &[Checkpoint]) -> CheckpointDiagnosticStats {
    let mut drift_class_flagged_counts = BTreeMap::from([
        (DriftClass::WrongPlanBranch, 0usize),
        (DriftClass::IgnoringRepoTruth, 0usize),
        (DriftClass::DeadEndThrash, 0usize),
    ]);
    let mut confidence_distribution = ConfidenceDistribution::default();

    for checkpoint in checkpoints {
        for score in checkpoint.drift_scores.iter().filter(|score| score.flagged) {
            *drift_class_flagged_counts.entry(score.class).or_default() += 1;
        }
        match checkpoint.task_frame.confidence {
            Confidence::Low => confidence_distribution.low += 1,
            Confidence::Medium => confidence_distribution.medium += 1,
            Confidence::High => confidence_distribution.high += 1,
        }
    }

    CheckpointDiagnosticStats {
        checkpoint_count: checkpoints.len(),
        flagged_checkpoint_count: checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.flagged)
            .count(),
        drift_class_flagged_counts,
        task_frame_transition_count: checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.diagnostics.task_frame_transitioned)
            .count(),
        confidence_distribution,
        working_set_change_count: checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.diagnostics.working_set_changed)
            .count(),
        adjacent_checkpoint_pair_count: checkpoints.len().saturating_sub(1),
        total_evidence_item_count: checkpoints
            .iter()
            .map(|checkpoint| checkpoint.diagnostics.evidence_item_count)
            .sum(),
    }
}

fn render_summary(sessions: &[BundleSession], checkpoints: &[Checkpoint]) -> String {
    let mut by_session = BTreeMap::<&str, Vec<&Checkpoint>>::new();
    for checkpoint in checkpoints {
        by_session
            .entry(checkpoint.session_id.as_str())
            .or_default()
            .push(checkpoint);
    }
    let session_summaries = sessions
        .iter()
        .map(|session| {
            let session_checkpoints = by_session
                .get(session.session_id.as_str())
                .cloned()
                .unwrap_or_default();
            summarize_session(session, &session_checkpoints)
        })
        .collect::<Vec<_>>();
    let overall = aggregate_summary_metrics(&session_summaries);

    let mut lines = vec![
        "# Agent Drift Analyzer Summary".to_string(),
        String::new(),
        format!("Sessions analyzed: `{}`", session_summaries.len()),
        format!("Turns observed: `{}`", overall.turns_observed),
        format!("User prompts observed: `{}`", overall.user_prompts_observed),
        format!("Checkpoints emitted: `{}`", overall.checkpoints_emitted),
        format!(
            "Checkpoints per turn: `{}`",
            format_optional_metric(overall.checkpoints_per_turn)
        ),
        format!(
            "Checkpoints per user prompt: `{}`",
            format_optional_metric(overall.checkpoints_per_user_prompt)
        ),
        format!(
            "Avg rows between checkpoints: `{}`",
            format_optional_metric(overall.avg_rows_between_checkpoints)
        ),
        format!(
            "Avg seconds between checkpoints: `{}`",
            format_optional_metric(overall.avg_seconds_between_checkpoints)
        ),
        format!("Flagged checkpoints: `{}`", overall.flagged_checkpoints),
        format!(
            "Longest flagged streak: `{}`",
            overall.longest_flagged_streak
        ),
        format!(
            "Prompt user messages: `{}`",
            overall.user_message_roles.prompt
        ),
        format!(
            "Steer user messages: `{}`",
            overall.user_message_roles.steer
        ),
        format!(
            "Unknown user messages: `{}`",
            overall.user_message_roles.unknown
        ),
        String::new(),
    ];

    for session_summary in &session_summaries {
        lines.push(format!("## {}", session_summary.session_id));
        lines.push(format!(
            "- Turns observed: `{}`",
            session_summary.metrics.turns_observed
        ));
        lines.push(format!(
            "- User prompts observed: `{}`",
            session_summary.metrics.user_prompts_observed
        ));
        lines.push(format!(
            "- Checkpoints emitted: `{}`",
            session_summary.metrics.checkpoints_emitted
        ));
        lines.push(format!(
            "- Checkpoints per turn: `{}`",
            format_optional_metric(session_summary.metrics.checkpoints_per_turn)
        ));
        lines.push(format!(
            "- Checkpoints per user prompt: `{}`",
            format_optional_metric(session_summary.metrics.checkpoints_per_user_prompt)
        ));
        lines.push(format!(
            "- Avg rows between checkpoints: `{}`",
            format_optional_metric(session_summary.metrics.avg_rows_between_checkpoints)
        ));
        lines.push(format!(
            "- Avg seconds between checkpoints: `{}`",
            format_optional_metric(session_summary.metrics.avg_seconds_between_checkpoints)
        ));
        lines.push(format!(
            "- Flagged checkpoints: `{}`",
            session_summary.metrics.flagged_checkpoints
        ));
        lines.push(format!(
            "- Longest flagged streak: `{}`",
            session_summary.metrics.longest_flagged_streak
        ));
        lines.push(format!(
            "- Distinct task frames: `{}`",
            session_summary.diagnostics.distinct_task_frames
        ));
        lines.push(format!(
            "- Truth artifacts referenced: `{}`",
            session_summary.diagnostics.truth_artifacts_referenced
        ));
        lines.push(format!(
            "- Verification commands observed: `{}`",
            session_summary.diagnostics.verification_commands_observed
        ));
        lines.push(format!(
            "- Prompt user messages: `{}`",
            session_summary.diagnostics.user_message_roles.prompt
        ));
        lines.push(format!(
            "- Steer user messages: `{}`",
            session_summary.diagnostics.user_message_roles.steer
        ));
        lines.push(format!(
            "- Unknown user messages: `{}`",
            session_summary.diagnostics.user_message_roles.unknown
        ));
        for checkpoint in &session_summary.checkpoints {
            let flagged_scores = checkpoint
                .drift_scores
                .iter()
                .filter(|score| score.flagged)
                .map(|score| format!("{:?}", score.class))
                .collect::<Vec<_>>();
            lines.push(format!(
                "- {}: flagged=`{}` next=`{}` drift=`{}`",
                checkpoint.checkpoint_id,
                if checkpoint.flagged { "yes" } else { "no" },
                checkpoint.expected_next_step,
                if flagged_scores.is_empty() {
                    "none".to_string()
                } else {
                    flagged_scores.join(", ")
                }
            ));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

fn session_turn_count(session: &BundleSession) -> usize {
    session
        .archival_rows
        .iter()
        .filter_map(|row| row.turn_id.as_deref())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

#[derive(Debug, Clone, PartialEq)]
struct SessionSummary {
    session_id: String,
    metrics: SessionSummaryMetrics,
    diagnostics: SessionDiagnostics,
    spacing: SpacingAccumulator,
    checkpoints: Vec<Checkpoint>,
}

#[derive(Debug, Clone, PartialEq)]
struct SessionSummaryMetrics {
    turns_observed: usize,
    user_prompts_observed: usize,
    checkpoints_emitted: usize,
    checkpoints_per_turn: Option<f64>,
    checkpoints_per_user_prompt: Option<f64>,
    avg_rows_between_checkpoints: Option<f64>,
    avg_seconds_between_checkpoints: Option<f64>,
    flagged_checkpoints: usize,
    longest_flagged_streak: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionDiagnostics {
    user_message_roles: UserMessageRoleCounts,
    distinct_task_frames: usize,
    truth_artifacts_referenced: usize,
    verification_commands_observed: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SpacingAccumulator {
    row_gap_sum: usize,
    row_gap_count: usize,
    second_gap_sum: i64,
    second_gap_count: usize,
}

impl SpacingAccumulator {
    fn avg_rows(self) -> Option<f64> {
        average(self.row_gap_sum, self.row_gap_count)
    }

    fn avg_seconds(self) -> Option<f64> {
        average_i64(self.second_gap_sum, self.second_gap_count)
    }
}

impl std::ops::Add for SpacingAccumulator {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row_gap_sum: self.row_gap_sum + rhs.row_gap_sum,
            row_gap_count: self.row_gap_count + rhs.row_gap_count,
            second_gap_sum: self.second_gap_sum + rhs.second_gap_sum,
            second_gap_count: self.second_gap_count + rhs.second_gap_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct OverallSummaryMetrics {
    turns_observed: usize,
    user_prompts_observed: usize,
    checkpoints_emitted: usize,
    checkpoints_per_turn: Option<f64>,
    checkpoints_per_user_prompt: Option<f64>,
    avg_rows_between_checkpoints: Option<f64>,
    avg_seconds_between_checkpoints: Option<f64>,
    flagged_checkpoints: usize,
    longest_flagged_streak: usize,
    user_message_roles: UserMessageRoleCounts,
}

fn summarize_session(session: &BundleSession, checkpoints: &[&Checkpoint]) -> SessionSummary {
    let mut sorted_checkpoints = checkpoints
        .iter()
        .map(|checkpoint| (*checkpoint).clone())
        .collect::<Vec<_>>();
    sorted_checkpoints.sort_by(|left, right| left.ordinal.cmp(&right.ordinal));

    let user_message_roles = user_message_role_counts(&session.compact_rows);
    let spacing = checkpoint_spacing(session, &sorted_checkpoints);
    let metrics = SessionSummaryMetrics {
        turns_observed: session_turn_count(session),
        user_prompts_observed: user_message_roles.prompt,
        checkpoints_emitted: sorted_checkpoints.len(),
        checkpoints_per_turn: ratio(sorted_checkpoints.len(), session_turn_count(session)),
        checkpoints_per_user_prompt: ratio(sorted_checkpoints.len(), user_message_roles.prompt),
        avg_rows_between_checkpoints: spacing.avg_rows(),
        avg_seconds_between_checkpoints: spacing.avg_seconds(),
        flagged_checkpoints: sorted_checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.flagged)
            .count(),
        longest_flagged_streak: longest_flagged_streak(&sorted_checkpoints),
    };
    let diagnostics = SessionDiagnostics {
        user_message_roles,
        distinct_task_frames: distinct_task_frame_count(&sorted_checkpoints),
        truth_artifacts_referenced: distinct_truth_artifact_count(&sorted_checkpoints),
        verification_commands_observed: distinct_verification_command_count(&sorted_checkpoints),
    };

    SessionSummary {
        session_id: session.session_id.clone(),
        metrics,
        diagnostics,
        spacing,
        checkpoints: sorted_checkpoints,
    }
}

fn aggregate_summary_metrics(sessions: &[SessionSummary]) -> OverallSummaryMetrics {
    let user_message_roles = sessions
        .iter()
        .fold(UserMessageRoleCounts::default(), |acc, session| {
            acc + session.diagnostics.user_message_roles
        });
    let turns_observed = sessions
        .iter()
        .map(|session| session.metrics.turns_observed)
        .sum::<usize>();
    let user_prompts_observed = sessions
        .iter()
        .map(|session| session.metrics.user_prompts_observed)
        .sum::<usize>();
    let checkpoints_emitted = sessions
        .iter()
        .map(|session| session.metrics.checkpoints_emitted)
        .sum::<usize>();
    let flagged_checkpoints = sessions
        .iter()
        .map(|session| session.metrics.flagged_checkpoints)
        .sum::<usize>();
    let longest_flagged_streak = sessions
        .iter()
        .map(|session| session.metrics.longest_flagged_streak)
        .max()
        .unwrap_or(0);
    let spacing = sessions
        .iter()
        .fold(SpacingAccumulator::default(), |acc, session| {
            acc + session.spacing
        });

    OverallSummaryMetrics {
        turns_observed,
        user_prompts_observed,
        checkpoints_emitted,
        checkpoints_per_turn: ratio(checkpoints_emitted, turns_observed),
        checkpoints_per_user_prompt: ratio(checkpoints_emitted, user_prompts_observed),
        avg_rows_between_checkpoints: spacing.avg_rows(),
        avg_seconds_between_checkpoints: spacing.avg_seconds(),
        flagged_checkpoints,
        longest_flagged_streak,
        user_message_roles,
    }
}

fn checkpoint_spacing(session: &BundleSession, checkpoints: &[Checkpoint]) -> SpacingAccumulator {
    let archival_positions = session
        .archival_rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row_ref_key(RowRef::from_row(row)), (index, row.timestamp)))
        .collect::<BTreeMap<_, _>>();
    let compact_positions = session
        .compact_rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row_ref_key(RowRef::from_row(row)), (index, row.timestamp)))
        .collect::<BTreeMap<_, _>>();

    let mut spacing = SpacingAccumulator::default();
    for pair in checkpoints.windows(2) {
        let Some(previous) = boundary_end_observation(
            &pair[0].boundary.end,
            &archival_positions,
            &compact_positions,
        ) else {
            continue;
        };
        let Some(current) = boundary_end_observation(
            &pair[1].boundary.end,
            &archival_positions,
            &compact_positions,
        ) else {
            continue;
        };
        if current.position >= previous.position {
            spacing.row_gap_sum += current.position - previous.position;
            spacing.row_gap_count += 1;
        }
        if let (Some(previous_timestamp), Some(current_timestamp)) =
            (previous.timestamp, current.timestamp)
        {
            let seconds = (current_timestamp - previous_timestamp).whole_seconds();
            if seconds >= 0 {
                spacing.second_gap_sum += seconds;
                spacing.second_gap_count += 1;
            }
        }
    }

    spacing
}

fn longest_flagged_streak(checkpoints: &[Checkpoint]) -> usize {
    let mut longest = 0usize;
    let mut current = 0usize;
    for checkpoint in checkpoints {
        if checkpoint.flagged {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

fn distinct_task_frame_count(checkpoints: &[Checkpoint]) -> usize {
    checkpoints
        .iter()
        .map(|checkpoint| task_frame_identity(&checkpoint.task_frame))
        .collect::<BTreeSet<_>>()
        .len()
}

fn distinct_truth_artifact_count(checkpoints: &[Checkpoint]) -> usize {
    checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.task_frame.truth_artifacts.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len()
}

fn distinct_verification_command_count(checkpoints: &[Checkpoint]) -> usize {
    checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.task_frame.verification_commands.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundaryObservation {
    position: usize,
    timestamp: Option<OffsetDateTime>,
}

fn boundary_end_observation(
    row_ref: &RowRef,
    archival_positions: &BTreeMap<RowRefKey, (usize, Option<OffsetDateTime>)>,
    compact_positions: &BTreeMap<RowRefKey, (usize, Option<OffsetDateTime>)>,
) -> Option<BoundaryObservation> {
    archival_positions
        .get(&row_ref_key(row_ref.clone()))
        .or_else(|| compact_positions.get(&row_ref_key(row_ref.clone())))
        .map(|(position, timestamp)| BoundaryObservation {
            position: *position,
            timestamp: *timestamp,
        })
}

type RowRefKey = (Utf8PathBuf, usize, usize);

fn row_ref_key(row_ref: RowRef) -> RowRefKey {
    (
        row_ref.source_file,
        row_ref.event_index,
        row_ref.row_ordinal,
    )
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then_some(numerator as f64 / denominator as f64)
}

fn average(sum: usize, count: usize) -> Option<f64> {
    (count > 0).then_some(sum as f64 / count as f64)
}

fn average_i64(sum: i64, count: usize) -> Option<f64> {
    (count > 0).then_some(sum as f64 / count as f64)
}

fn format_optional_metric(metric: Option<f64>) -> String {
    metric
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "unavailable".to_string())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct UserMessageRoleCounts {
    prompt: usize,
    steer: usize,
    unknown: usize,
}

impl std::ops::Add for UserMessageRoleCounts {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            prompt: self.prompt + rhs.prompt,
            steer: self.steer + rhs.steer,
            unknown: self.unknown + rhs.unknown,
        }
    }
}

fn user_message_role_counts(rows: &[CompactionRow]) -> UserMessageRoleCounts {
    let mut counts = UserMessageRoleCounts::default();
    for row in rows
        .iter()
        .filter(|row| row.kind == CompactionKind::UserMessage)
        .filter(|row| !is_synthetic_user_message(row))
    {
        match row.user_message_role.unwrap_or(UserMessageRole::Unknown) {
            UserMessageRole::Prompt => counts.prompt += 1,
            UserMessageRole::Steer => counts.steer += 1,
            UserMessageRole::Unknown => counts.unknown += 1,
        }
    }
    counts
}

fn is_synthetic_user_message(row: &CompactionRow) -> bool {
    row.text.contains("AGENTS.md instructions")
        || row.text.contains("<skill>")
        || row.text.contains("Available skills")
}
