mod export;
mod schema;

use std::collections::BTreeSet;

use crate::input::BundleSession;
use crate::{
    context::assemble_context, context::collect_command_observations, inference::infer_task_frame,
};
use agent_session_compactor::{CompactionKind, CompactionRow, RowRef};
use camino::Utf8PathBuf;

pub use export::{
    export_checkpoints, summarize_checkpoint_diagnostics, CheckpointDiagnosticStats,
    ConfidenceDistribution, ExportError, ExportResult,
};
pub use schema::{
    Checkpoint, CheckpointBoundary, CheckpointDiagnostics, Confidence, DriftClass, DriftScore,
    EvidenceRef, TaskFrame,
};

const MAX_ROWS_PER_CHECKPOINT: usize = 64;

pub fn build_session_checkpoint(
    session: &BundleSession,
    ordinal: usize,
    task_frame: &TaskFrame,
    drift_scores: Vec<DriftScore>,
) -> Checkpoint {
    let boundary = checkpoint_boundary(session);
    let diagnostics = checkpoint_diagnostics(session, ordinal, task_frame, &drift_scores);
    let expected_next_step = expected_next_step(task_frame);
    Checkpoint {
        schema_version: "v0.2".to_string(),
        session_id: session.session_id.clone(),
        checkpoint_id: format!("{}:{ordinal:04}", session.session_id),
        ordinal,
        boundary,
        diagnostics,
        task_frame: task_frame.clone(),
        flagged: drift_scores.iter().any(|score| score.flagged),
        drift_scores,
        expected_next_step,
    }
}

fn checkpoint_diagnostics(
    session: &BundleSession,
    ordinal: usize,
    task_frame: &TaskFrame,
    drift_scores: &[DriftScore],
) -> CheckpointDiagnostics {
    let prior_window = previous_checkpoint_window(session, ordinal);
    let (task_frame_transitioned, working_set_changed, interval_rows) =
        if let Some(previous_window) = prior_window.as_ref() {
            let previous_context = assemble_context(previous_window);
            let previous_task_frame = infer_task_frame(&previous_context);
            let interval_start = previous_window.compact_rows.len();
            (
                task_frame_identity(task_frame) != task_frame_identity(&previous_task_frame),
                working_set_identity(task_frame) != working_set_identity(&previous_task_frame),
                &session.compact_rows[interval_start..],
            )
        } else {
            (false, false, session.compact_rows.as_slice())
        };
    let interval_commands = collect_command_observations(interval_rows);

    CheckpointDiagnostics {
        task_frame_transitioned,
        working_set_changed,
        interval_command_count: interval_commands.len(),
        interval_verification_command_count: interval_commands
            .iter()
            .filter(|command| command.verification_like)
            .count(),
        evidence_item_count: evidence_item_count(task_frame, drift_scores),
    }
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

fn previous_checkpoint_window(session: &BundleSession, ordinal: usize) -> Option<BundleSession> {
    (ordinal > 1).then(|| {
        checkpoint_windows(session)
            .into_iter()
            .nth(ordinal.saturating_sub(2))
    })?
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

fn row_key(row: &CompactionRow) -> (Utf8PathBuf, usize, usize) {
    (row.source_file.clone(), row.event_index, row.row_ordinal)
}
