#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::DriftState;
use agent_drift_sentinel::input::{load_replay_bundle, InputError};
use camino::Utf8Path;
use support::{checkpoint, ReplayFixture};
use tempfile::TempDir;

fn schema_checkpoint(
    schema_version: &str,
    session_id: &str,
    ordinal: usize,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
) -> agent_drift_analyzer::Checkpoint {
    let mut checkpoint = checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = schema_version.to_string();
    checkpoint
}

#[test]
fn replay_input_loads_and_sorts_v0_3_checkpoints() {
    let fixture = ReplayFixture::from_checkpoints(
        vec![
            schema_checkpoint("v0.3", "session-beta", 2, 0, false, "continue"),
            schema_checkpoint("v0.3", "session-alpha", 3, 65, true, "repair"),
            schema_checkpoint("v0.3", "session-alpha", 1, 85, true, "repair"),
        ],
        support::sample_summary(),
    );

    let bundle = load_replay_bundle(&fixture.checkpoint_dir).expect("load replay bundle");

    assert_eq!(bundle.schema_version, "v0.3");
    assert_eq!(bundle.checkpoints.len(), 3);
    assert_eq!(bundle.checkpoints[0].checkpoint_id, "session-alpha:0001");
    assert_eq!(bundle.checkpoints[1].checkpoint_id, "session-alpha:0003");
    assert_eq!(bundle.checkpoints[2].checkpoint_id, "session-beta:0002");
}

#[test]
fn replay_input_retains_v0_2_compatibility() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
    let checkpoint_dir = root.join("checkpoint");
    fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");

    let checkpoint = schema_checkpoint("v0.2", "session-alpha", 1, 85, true, "repair");
    let mut legacy_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    legacy_json["drift_scores"][0]
        .as_object_mut()
        .expect("drift score object")
        .remove("state");
    let legacy_line = serde_json::to_string(&legacy_json).expect("serialize legacy checkpoint");
    fs::write(
        checkpoint_dir.join("checkpoints.jsonl"),
        format!("{legacy_line}\n"),
    )
    .expect("write checkpoints");
    fs::write(checkpoint_dir.join("summary.md"), support::sample_summary()).expect("write summary");

    let bundle = load_replay_bundle(&checkpoint_dir).expect("load replay bundle");

    assert_eq!(bundle.schema_version, "v0.2");
    assert_eq!(bundle.checkpoints.len(), 1);
    assert_eq!(bundle.checkpoints[0].checkpoint_id, "session-alpha:0001");
}

#[test]
fn replay_input_rejects_v0_3_checkpoints_missing_state() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
    let checkpoint_dir = root.join("checkpoint");
    fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");

    let checkpoint = schema_checkpoint("v0.3", "session-alpha", 1, 85, true, "repair");
    let mut malformed_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    malformed_json["drift_scores"][0]
        .as_object_mut()
        .expect("drift score object")
        .remove("state");
    let malformed_line =
        serde_json::to_string(&malformed_json).expect("serialize malformed checkpoint");
    fs::write(
        checkpoint_dir.join("checkpoints.jsonl"),
        format!("{malformed_line}\n"),
    )
    .expect("write checkpoints");
    fs::write(checkpoint_dir.join("summary.md"), support::sample_summary()).expect("write summary");

    let error = load_replay_bundle(&checkpoint_dir)
        .expect_err("v0.3 checkpoints missing state must fail closed");

    assert!(matches!(
        error,
        InputError::ContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.3" && field == "drift_scores[0].state"
    ));
    assert!(error.to_string().contains("missing drift_scores[0].state"));
}

#[test]
fn replay_input_preserves_explicit_v0_3_drift_state() {
    let mut checkpoint = schema_checkpoint("v0.3", "session-alpha", 1, 20, false, "continue");
    checkpoint.drift_scores[0].state = DriftState::HistoricalOnly;
    checkpoint.drift_scores[0].evidence = vec![agent_drift_analyzer::EvidenceRef {
        row: checkpoint.boundary.start.clone(),
        reason: "explicit analyzer historical state".to_string(),
    }];

    let fixture =
        ReplayFixture::from_checkpoints(vec![checkpoint.clone()], support::sample_summary());

    let bundle = load_replay_bundle(&fixture.checkpoint_dir).expect("load replay bundle");

    assert_eq!(bundle.schema_version, "v0.3");
    assert_eq!(
        bundle.checkpoints[0].drift_scores[0].state,
        DriftState::HistoricalOnly
    );
    assert_eq!(
        bundle.checkpoints[0].drift_scores[0].evidence[0].reason,
        "explicit analyzer historical state"
    );
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
        schema_checkpoint("v0.2", "session-alpha", 1, 65, true, "re-read the task doc");
    checkpoint.drift_scores[0].class = agent_drift_analyzer::DriftClass::TruthGroundingGap;
    let legacy_line = serde_json::to_string(&checkpoint)
        .expect("serialize checkpoint")
        .replace("truth_grounding_gap", "ignoring_repo_truth");
    fs::write(
        checkpoint_dir.join("checkpoints.jsonl"),
        format!("{legacy_line}\n"),
    )
    .expect("write checkpoints");
    fs::write(checkpoint_dir.join("summary.md"), support::sample_summary()).expect("write summary");

    let bundle = load_replay_bundle(&checkpoint_dir).expect("load replay bundle");

    assert_eq!(
        bundle.checkpoints[0].drift_scores[0].class,
        agent_drift_analyzer::DriftClass::TruthGroundingGap
    );
}
