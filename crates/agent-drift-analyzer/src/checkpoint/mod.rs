mod export;
mod schema;

use agent_session_compactor::RowRef;

use crate::input::BundleSession;

pub use export::{export_checkpoints, ExportError, ExportResult};
pub use schema::{
    Checkpoint, CheckpointBoundary, Confidence, DriftClass, DriftScore, EvidenceRef, TaskFrame,
};

pub fn build_session_checkpoints(
    session: &BundleSession,
    task_frame: &TaskFrame,
    drift_scores: Vec<DriftScore>,
) -> Vec<Checkpoint> {
    let boundary = checkpoint_boundary(session);
    let expected_next_step = expected_next_step(task_frame);
    vec![Checkpoint {
        schema_version: "v0.1".to_string(),
        session_id: session.session_id.clone(),
        checkpoint_id: format!("{}:0001", session.session_id),
        ordinal: 1,
        boundary,
        task_frame: task_frame.clone(),
        flagged: drift_scores.iter().any(|score| score.flagged),
        drift_scores,
        expected_next_step,
    }]
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
