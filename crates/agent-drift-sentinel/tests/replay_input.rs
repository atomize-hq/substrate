#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_sentinel::input::load_replay_bundle;
use camino::Utf8Path;
use support::{checkpoint, ReplayFixture};
use tempfile::TempDir;

fn current_schema_checkpoint(
    session_id: &str,
    ordinal: usize,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
) -> agent_drift_analyzer::Checkpoint {
    let mut checkpoint = checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = "v0.2".to_string();
    checkpoint
}

#[test]
fn replay_input_loads_and_sorts_current_schema_checkpoints() {
    let fixture = ReplayFixture::from_checkpoints(
        vec![
            current_schema_checkpoint("session-beta", 2, 0, false, "continue"),
            current_schema_checkpoint("session-alpha", 3, 65, true, "repair"),
            current_schema_checkpoint("session-alpha", 1, 85, true, "repair"),
        ],
        support::sample_summary(),
    );

    let bundle = load_replay_bundle(&fixture.checkpoint_dir).expect("load replay bundle");

    assert_eq!(bundle.schema_version, "v0.2");
    assert_eq!(bundle.checkpoints.len(), 3);
    assert_eq!(bundle.checkpoints[0].checkpoint_id, "session-alpha:0001");
    assert_eq!(bundle.checkpoints[1].checkpoint_id, "session-alpha:0003");
    assert_eq!(bundle.checkpoints[2].checkpoint_id, "session-beta:0002");
}

#[test]
fn replay_input_applies_cursor_strictly_after_session_and_ordinal() {
    let fixture = ReplayFixture::from_checkpoints(
        support::sample_checkpoints()
            .into_iter()
            .map(|mut checkpoint| {
                checkpoint.schema_version = "v0.2".to_string();
                checkpoint
            })
            .collect(),
        support::sample_summary(),
    );
    let bundle = load_replay_bundle(&fixture.checkpoint_dir).expect("load replay bundle");

    let remaining = bundle.checkpoints_after(Some(&agent_drift_sentinel::CheckpointCursor {
        session_id: "session-alpha".to_string(),
        ordinal: 2,
    }));

    assert_eq!(
        remaining
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["session-alpha:0003", "session-beta:0001"]
    );
}

#[test]
fn replay_input_accepts_legacy_ignoring_repo_truth_rows_while_mapping_to_truth_grounding_gap() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
    let checkpoint_dir = root.join("checkpoint");
    fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");

    let mut checkpoint =
        current_schema_checkpoint("session-alpha", 1, 65, true, "re-read the task doc");
    checkpoint.drift_scores[0].class = agent_drift_analyzer::DriftClass::TruthGroundingGap;
    let legacy_line = serde_json::to_string(&checkpoint)
        .expect("serialize checkpoint")
        .replace("truth_grounding_gap", "ignoring_repo_truth");
    fs::write(
        checkpoint_dir.join("checkpoints.jsonl"),
        format!("{legacy_line}\n"),
    )
    .expect("write checkpoints");
    fs::write(checkpoint_dir.join("summary.md"), support::sample_summary())
        .expect("write summary");

    let bundle = load_replay_bundle(&checkpoint_dir).expect("load replay bundle");

    assert_eq!(
        bundle.checkpoints[0].drift_scores[0].class,
        agent_drift_analyzer::DriftClass::TruthGroundingGap
    );
}
