#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{DriftClass, EvidenceRef};
use agent_drift_sentinel::{
    load_live_fixture,
    operator_surface::{present_checkpoint, present_checkpoint_with_previous, CheckpointPosture},
    scheduler::ReplayScheduler,
    verify_live_checkpoint_compatibility, DecisionReason, LiveInputError, SchedulerPolicy,
    TriggerClass, WarningDisposition, WarningPolicy,
};
use camino::Utf8Path;
use tempfile::TempDir;

fn schema_checkpoint(
    schema_version: &str,
    session_id: &str,
    ordinal: usize,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
) -> agent_drift_analyzer::Checkpoint {
    let mut checkpoint =
        support::checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = schema_version.to_string();
    checkpoint
}

#[test]
fn live_checkpoint_compatibility_accepts_v0_3_checkpoint() {
    let checkpoint = schema_checkpoint(
        "v0.3",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    let compatibility =
        verify_live_checkpoint_compatibility(&checkpoint).expect("checkpoint is live-compatible");
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());
    let decision = scheduler.observe(
        compatibility.cursor.clone(),
        TriggerClass::CheckpointReady,
        compatibility.flagged,
        Some(&compatibility.warning_fingerprint),
    );
    let presentation = present_checkpoint(
        &checkpoint,
        TriggerClass::CheckpointReady,
        &decision,
        &WarningPolicy::default(),
    );

    assert!(decision.evaluate);
    assert_eq!(decision.reason, DecisionReason::InitialCheckpoint);
    assert_eq!(compatibility.max_flagged_score, Some(88));
    assert!(matches!(
        presentation.disposition,
        WarningDisposition::Visible
    ));
    assert!(presentation.headline.contains("session-alpha:0001"));
    assert_eq!(
        presentation.expected_next_step,
        "re-read the implementation plan"
    );
}

#[test]
fn live_checkpoint_compatibility_retains_v0_2_support() {
    let checkpoint = schema_checkpoint(
        "v0.2",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );

    let compatibility =
        verify_live_checkpoint_compatibility(&checkpoint).expect("checkpoint is live-compatible");

    assert_eq!(compatibility.cursor.ordinal, 1);
    assert_eq!(compatibility.max_flagged_score, Some(88));
    assert!(compatibility.flagged);
}

#[test]
fn live_checkpoint_compatibility_loads_v0_2_fixture_without_state() {
    let temp_dir = TempDir::new().expect("temp dir");
    let fixture_path = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("live-checkpoints.jsonl");
    let checkpoint = schema_checkpoint(
        "v0.2",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    let mut checkpoint_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    checkpoint_json["drift_scores"][0]
        .as_object_mut()
        .expect("drift score object")
        .remove("state");
    let record = serde_json::json!({
        "event_type": "checkpoint_ready",
        "emission_ordinal": 1,
        "checkpoint": checkpoint_json,
    });
    fs::write(
        fixture_path.as_std_path(),
        format!("{}\n", serde_json::to_string(&record).expect("record json")),
    )
    .expect("write live fixture");

    let events = load_live_fixture(&fixture_path).expect("load legacy live fixture");

    assert_eq!(events.len(), 1);
    let loaded_checkpoint = events[0]
        .checkpoint
        .as_ref()
        .expect("checkpoint-ready payload");
    assert_eq!(loaded_checkpoint.schema_version, "v0.2");
    assert_eq!(loaded_checkpoint.checkpoint_id, "session-alpha:0001");
}

#[test]
fn live_checkpoint_compatibility_rejects_v0_3_fixture_missing_state() {
    let temp_dir = TempDir::new().expect("temp dir");
    let fixture_path = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("live-checkpoints.jsonl");
    let checkpoint = schema_checkpoint(
        "v0.3",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    let mut checkpoint_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    checkpoint_json["drift_scores"][0]
        .as_object_mut()
        .expect("drift score object")
        .remove("state");
    let record = serde_json::json!({
        "event_type": "checkpoint_ready",
        "emission_ordinal": 1,
        "checkpoint": checkpoint_json,
    });
    fs::write(
        fixture_path.as_std_path(),
        format!("{}\n", serde_json::to_string(&record).expect("record json")),
    )
    .expect("write live fixture");

    let error =
        load_live_fixture(&fixture_path).expect_err("v0.3 fixtures missing state must fail");

    assert!(matches!(
        error,
        LiveInputError::FixtureContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.3" && field == "checkpoint.drift_scores[0].state"
    ));
    assert!(error
        .to_string()
        .contains("missing checkpoint.drift_scores[0].state"));
}

#[test]
fn live_checkpoint_compatibility_surfaces_analyzer_contract_gaps_explicitly() {
    let mut checkpoint = schema_checkpoint(
        "v0.3",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.task_frame.objective.clear();

    let error = verify_live_checkpoint_compatibility(&checkpoint)
        .expect_err("missing objective must be reported as an analyzer contract gap");

    assert!(matches!(
        error,
        LiveInputError::CompatibilityGap {
            field: "task_frame.objective",
            ..
        }
    ));
}

#[test]
fn live_checkpoint_compatibility_supports_recovered_posture_without_schema_widening() {
    let previous = checkpoint_with_drift(
        "v0.2",
        "session-posture",
        1,
        DriftClass::TruthGroundingGap,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-posture:1"],
    );
    let current = checkpoint_with_drift(
        "v0.2",
        "session-posture",
        2,
        DriftClass::TruthGroundingGap,
        20,
        false,
        "continue on the current task frame",
        &["historical truth-grounding gap: flagged score for session-posture:1"],
    );

    let previous_compatibility =
        verify_live_checkpoint_compatibility(&previous).expect("previous checkpoint compatible");
    let current_compatibility =
        verify_live_checkpoint_compatibility(&current).expect("current checkpoint compatible");
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());
    let decision = scheduler.observe(
        current_compatibility.cursor.clone(),
        TriggerClass::CheckpointReady,
        current_compatibility.flagged,
        Some(&current_compatibility.warning_fingerprint),
    );
    let presentation = present_checkpoint_with_previous(
        &current,
        Some(&previous),
        TriggerClass::CheckpointReady,
        &decision,
        &WarningPolicy::default(),
    );

    assert_eq!(previous_compatibility.cursor.ordinal, 1);
    assert_eq!(current_compatibility.cursor.ordinal, 2);
    assert_eq!(presentation.posture, Some(CheckpointPosture::Recovered));
    assert!(presentation
        .evidence_lines
        .iter()
        .any(|line| line
            .contains("historical truth-grounding gap: flagged score for session-posture:1")));
}

fn checkpoint_with_drift(
    schema_version: &str,
    session_id: &str,
    ordinal: usize,
    class: DriftClass,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
    evidence_reasons: &[&str],
) -> agent_drift_analyzer::Checkpoint {
    let mut checkpoint =
        support::checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = schema_version.to_string();
    checkpoint.flagged = flagged;
    checkpoint.drift_scores[0].class = class;
    checkpoint.drift_scores[0].raw_score = raw_score;
    checkpoint.drift_scores[0].flagged = flagged;
    checkpoint.drift_scores[0].evidence = evidence_reasons
        .iter()
        .map(|reason| EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: (*reason).to_string(),
        })
        .collect();
    checkpoint.diagnostics.evidence_item_count = checkpoint.drift_scores[0].evidence.len();
    checkpoint
}
