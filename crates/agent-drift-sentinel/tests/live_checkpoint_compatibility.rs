#![allow(unused_crate_dependencies)]

use std::fs;

use agent_drift_analyzer::{
    checkpoint::CheckpointDiagnostics, Checkpoint, CheckpointBoundary, Confidence, DriftClass,
    DriftState, EvidenceRef, SessionArchetype, SessionArchetypeLabel, TaskFrame, TurnActivityMix,
    TurnContext, TurnExecutionMode,
};
use agent_drift_sentinel::{
    load_live_fixture,
    operator_surface::{present_checkpoint, present_checkpoint_with_previous, CheckpointPosture},
    scheduler::ReplayScheduler,
    verify_live_checkpoint_compatibility, DecisionReason, LiveInputError, SchedulerPolicy,
    TriggerClass, WarningDisposition, WarningPolicy,
};
use agent_session_compactor::RowRef;
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
    let mut checkpoint = checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = schema_version.to_string();
    checkpoint
}

fn checkpoint(
    session_id: &str,
    ordinal: usize,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
) -> Checkpoint {
    let row = RowRef {
        source_file: camino::Utf8PathBuf::from(format!("/tmp/{session_id}.jsonl")),
        event_index: ordinal,
        row_ordinal: 0,
    };
    let evidence_item_count = if flagged { 2 } else { 1 };
    Checkpoint {
        schema_version: "v0.2".to_string(),
        session_id: session_id.to_string(),
        checkpoint_id: format!("{session_id}:{ordinal:04}"),
        ordinal,
        boundary: CheckpointBoundary {
            start: row.clone(),
            end: row.clone(),
        },
        turn_context: None,
        diagnostics: CheckpointDiagnostics {
            task_frame_transitioned: ordinal == 1,
            working_set_changed: false,
            interval_command_count: 1,
            interval_verification_command_count: 1,
            evidence_item_count,
        },
        task_frame: TaskFrame {
            objective: format!(
                "/goal Complete replay validation for {session_id} checkpoint {ordinal}"
            ),
            confidence: Confidence::Medium,
            truth_artifacts: vec!["docs/specs/agent-drift-sentinel-v0.2-spec.md".to_string()],
            working_set_paths: vec!["crates/agent-drift-sentinel/src/lib.rs".to_string()],
            tools: vec!["functions.shell_command".to_string()],
            command_families: vec!["cargo".to_string()],
            verification_commands: vec![
                "cargo test -p agent-drift-sentinel -- --nocapture".to_string()
            ],
            supporting_evidence: vec![EvidenceRef {
                row: row.clone(),
                reason: "objective row".to_string(),
            }],
            counter_evidence: Vec::new(),
        },
        session_archetype: None,
        drift_scores: vec![agent_drift_analyzer::DriftScore {
            class: DriftClass::WrongPlanBranch,
            state: if flagged {
                DriftState::Active
            } else {
                DriftState::Cleared
            },
            raw_score,
            confidence: Confidence::Medium,
            flagged,
            evidence: if flagged {
                vec![EvidenceRef {
                    row,
                    reason: format!("flagged score for {session_id}:{ordinal}"),
                }]
            } else {
                Vec::new()
            },
        }],
        expected_next_step: expected_next_step.to_string(),
        flagged,
    }
}

fn sample_turn_context(turn_ordinal: usize) -> TurnContext {
    TurnContext {
        turn_id: Some(format!("turn-{turn_ordinal}")),
        turn_ordinal,
        rows_since_turn_start: 5,
        seconds_since_turn_start: Some(21),
        checkpoints_in_turn: 2,
        prompts_observed_in_session: turn_ordinal,
        execution_mode: TurnExecutionMode::Autonomous,
        activity_mix: TurnActivityMix {
            directive_row_count: 1,
            assistant_message_count: 1,
            tool_call_count: 1,
            read_like_command_count: 1,
            write_like_command_count: 1,
            verification_like_command_count: 1,
            tool_output_count: 1,
        },
    }
}

fn sample_session_archetype(checkpoint: &Checkpoint) -> SessionArchetype {
    SessionArchetype {
        label: SessionArchetypeLabel::Planning,
        confidence: Confidence::Medium,
        supporting_evidence: vec![EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: "session archetype evidence".to_string(),
        }],
        counter_evidence: Vec::new(),
    }
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
fn live_checkpoint_compatibility_accepts_v0_4_checkpoint() {
    let mut checkpoint = schema_checkpoint(
        "v0.4",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.turn_context = Some(sample_turn_context(1));

    let compatibility =
        verify_live_checkpoint_compatibility(&checkpoint).expect("checkpoint is live-compatible");

    assert_eq!(compatibility.cursor.ordinal, 1);
    assert_eq!(compatibility.max_flagged_score, Some(88));
    assert!(compatibility.flagged);
}

#[test]
fn live_checkpoint_compatibility_accepts_v0_5_checkpoint() {
    let mut checkpoint = schema_checkpoint(
        "v0.5",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.turn_context = Some(sample_turn_context(1));
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));

    let compatibility =
        verify_live_checkpoint_compatibility(&checkpoint).expect("checkpoint is live-compatible");

    assert_eq!(compatibility.cursor.ordinal, 1);
    assert_eq!(compatibility.max_flagged_score, Some(88));
    assert!(compatibility.flagged);
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
fn live_checkpoint_compatibility_loads_v0_4_fixture_with_turn_context() {
    let temp_dir = TempDir::new().expect("temp dir");
    let fixture_path = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("live-checkpoints.jsonl");
    let mut checkpoint = schema_checkpoint(
        "v0.4",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.turn_context = Some(sample_turn_context(1));
    let record = serde_json::json!({
        "event_type": "checkpoint_ready",
        "emission_ordinal": 1,
        "checkpoint": checkpoint,
    });
    fs::write(
        fixture_path.as_std_path(),
        format!("{}\n", serde_json::to_string(&record).expect("record json")),
    )
    .expect("write live fixture");

    let events = load_live_fixture(&fixture_path).expect("load v0.4 live fixture");

    assert_eq!(events.len(), 1);
    let loaded_checkpoint = events[0]
        .checkpoint
        .as_ref()
        .expect("checkpoint-ready payload");
    assert_eq!(loaded_checkpoint.schema_version, "v0.4");
    assert_eq!(
        loaded_checkpoint
            .turn_context
            .as_ref()
            .expect("v0.4 turn context")
            .turn_ordinal,
        1
    );
}

#[test]
fn live_checkpoint_compatibility_loads_v0_5_fixture_with_session_archetype() {
    let temp_dir = TempDir::new().expect("temp dir");
    let fixture_path = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("live-checkpoints.jsonl");
    let mut checkpoint = schema_checkpoint(
        "v0.5",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.turn_context = Some(sample_turn_context(1));
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));
    let record = serde_json::json!({
        "event_type": "checkpoint_ready",
        "emission_ordinal": 1,
        "checkpoint": checkpoint,
    });
    fs::write(
        fixture_path.as_std_path(),
        format!("{}\n", serde_json::to_string(&record).expect("record json")),
    )
    .expect("write live fixture");

    let events = load_live_fixture(&fixture_path).expect("load v0.5 live fixture");

    assert_eq!(events.len(), 1);
    let loaded_checkpoint = events[0]
        .checkpoint
        .as_ref()
        .expect("checkpoint-ready payload");
    assert_eq!(loaded_checkpoint.schema_version, "v0.5");
    assert_eq!(
        loaded_checkpoint
            .session_archetype
            .as_ref()
            .expect("v0.5 session archetype")
            .label,
        SessionArchetypeLabel::Planning
    );
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
fn live_checkpoint_compatibility_rejects_v0_4_fixture_missing_turn_context() {
    let temp_dir = TempDir::new().expect("temp dir");
    let fixture_path = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("live-checkpoints.jsonl");
    let mut checkpoint = schema_checkpoint(
        "v0.4",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.turn_context = Some(sample_turn_context(1));
    let mut checkpoint_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    checkpoint_json
        .as_object_mut()
        .expect("checkpoint object")
        .remove("turn_context");
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
        load_live_fixture(&fixture_path).expect_err("v0.4 fixtures missing turn context must fail");

    assert!(matches!(
        error,
        LiveInputError::FixtureContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.4" && field == "checkpoint.turn_context"
    ));
    assert!(error
        .to_string()
        .contains("missing checkpoint.turn_context"));
}

#[test]
fn live_checkpoint_compatibility_rejects_v0_5_fixture_missing_turn_context() {
    let temp_dir = TempDir::new().expect("temp dir");
    let fixture_path = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("live-checkpoints.jsonl");
    let mut checkpoint = schema_checkpoint(
        "v0.5",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.turn_context = Some(sample_turn_context(1));
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));
    let mut checkpoint_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    checkpoint_json
        .as_object_mut()
        .expect("checkpoint object")
        .remove("turn_context");
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
        load_live_fixture(&fixture_path).expect_err("v0.5 fixtures missing turn context must fail");

    assert!(matches!(
        error,
        LiveInputError::FixtureContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.5" && field == "checkpoint.turn_context"
    ));
    assert!(error
        .to_string()
        .contains("missing checkpoint.turn_context"));
}

#[test]
fn live_checkpoint_compatibility_rejects_v0_5_fixture_missing_session_archetype() {
    let temp_dir = TempDir::new().expect("temp dir");
    let fixture_path = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("live-checkpoints.jsonl");
    let mut checkpoint = schema_checkpoint(
        "v0.5",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.turn_context = Some(sample_turn_context(1));
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));
    let mut checkpoint_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    checkpoint_json
        .as_object_mut()
        .expect("checkpoint object")
        .remove("session_archetype");
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

    let error = load_live_fixture(&fixture_path)
        .expect_err("v0.5 fixtures missing session archetype must fail");

    assert!(matches!(
        error,
        LiveInputError::FixtureContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.5" && field == "checkpoint.session_archetype"
    ));
    assert!(error
        .to_string()
        .contains("missing checkpoint.session_archetype"));
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
fn live_checkpoint_compatibility_rejects_v0_4_checkpoint_without_turn_context() {
    let checkpoint = schema_checkpoint(
        "v0.4",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );

    let error = verify_live_checkpoint_compatibility(&checkpoint)
        .expect_err("v0.4 checkpoints missing turn context must fail closed");

    assert!(matches!(
        error,
        LiveInputError::CompatibilityGap {
            field: "turn_context",
            ..
        }
    ));
}

#[test]
fn live_checkpoint_compatibility_rejects_v0_5_checkpoint_without_turn_context() {
    let mut checkpoint = schema_checkpoint(
        "v0.5",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));

    let error = verify_live_checkpoint_compatibility(&checkpoint)
        .expect_err("v0.5 checkpoints missing turn context must fail closed");

    assert!(matches!(
        error,
        LiveInputError::CompatibilityGap {
            field: "turn_context",
            ..
        }
    ));
}

#[test]
fn live_checkpoint_compatibility_rejects_v0_5_checkpoint_without_session_archetype() {
    let mut checkpoint = schema_checkpoint(
        "v0.5",
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.turn_context = Some(sample_turn_context(1));

    let error = verify_live_checkpoint_compatibility(&checkpoint)
        .expect_err("v0.5 checkpoints missing session archetype must fail closed");

    assert!(matches!(
        error,
        LiveInputError::CompatibilityGap {
            field: "session_archetype",
            ..
        }
    ));
}

#[test]
fn live_checkpoint_compatibility_prefers_explicit_v0_3_state_without_previous_checkpoint() {
    let checkpoint = checkpoint_with_state(
        "session-v03",
        2,
        DriftClass::TruthGroundingGap,
        DriftState::Recovered,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer recovery evidence"],
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

    assert!(!compatibility.flagged);
    assert_eq!(presentation.posture, Some(CheckpointPosture::Recovered));
    assert!(presentation
        .evidence_lines
        .iter()
        .any(|line| line.contains("explicit analyzer recovery evidence")));
}

#[test]
fn live_checkpoint_compatibility_prefers_explicit_v0_5_state_without_previous_checkpoint() {
    let mut checkpoint = checkpoint_with_state(
        "session-v05",
        2,
        DriftClass::TruthGroundingGap,
        DriftState::Recovered,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer recovery evidence"],
    );
    checkpoint.schema_version = "v0.5".to_string();
    checkpoint.turn_context = Some(sample_turn_context(2));
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));

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

    assert!(!compatibility.flagged);
    assert_eq!(presentation.posture, Some(CheckpointPosture::Recovered));
    assert!(presentation
        .evidence_lines
        .iter()
        .any(|line| line.contains("explicit analyzer recovery evidence")));
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
    let mut checkpoint = checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
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

fn checkpoint_with_state(
    session_id: &str,
    ordinal: usize,
    class: DriftClass,
    state: DriftState,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
    evidence_reasons: &[&str],
) -> agent_drift_analyzer::Checkpoint {
    let mut checkpoint = checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = "v0.3".to_string();
    checkpoint.flagged = flagged;
    checkpoint.drift_scores[0].class = class;
    checkpoint.drift_scores[0].state = state;
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
