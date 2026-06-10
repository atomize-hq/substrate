#![allow(unused_crate_dependencies)]

use std::fs;

use agent_drift_analyzer::{
    checkpoint::CheckpointDiagnostics, Checkpoint, CheckpointBoundary, Confidence, DriftState,
    ProgressDimension, ProgressStatus, SessionArchetype, SessionArchetypeLabel, SessionProgress,
    TaskFrame, TurnActivityMix, TurnContext, TurnExecutionMode,
};
use agent_drift_sentinel::input::{load_replay_bundle, InputError};
use agent_session_compactor::RowRef;
use camino::Utf8Path;
use tempfile::TempDir;

struct ReplayFixture {
    _temp_dir: TempDir,
    checkpoint_dir: camino::Utf8PathBuf,
}

impl ReplayFixture {
    fn from_checkpoints(checkpoints: Vec<Checkpoint>, summary: &str) -> Self {
        let temp_dir = TempDir::new().expect("temp dir");
        let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
        let checkpoint_dir = root.join("checkpoint");
        fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");
        write_jsonl(checkpoint_dir.join("checkpoints.jsonl"), &checkpoints);
        fs::write(checkpoint_dir.join("summary.md"), summary).expect("write summary");
        Self {
            _temp_dir: temp_dir,
            checkpoint_dir,
        }
    }
}

fn sample_summary() -> &'static str {
    "# Agent Drift Analyzer Summary\n\nSessions analyzed: `2`\nFlagged checkpoints: `3`\n"
}

fn sample_checkpoints() -> Vec<Checkpoint> {
    vec![
        checkpoint("session-alpha", 1, 82, true, "align plan to repo truth"),
        checkpoint(
            "session-alpha",
            2,
            40,
            true,
            "re-read the task doc before editing",
        ),
        checkpoint(
            "session-alpha",
            3,
            86,
            true,
            "re-read the task doc before editing",
        ),
        checkpoint(
            "session-beta",
            1,
            0,
            false,
            "continue on the current task frame",
        ),
    ]
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
            supporting_evidence: vec![agent_drift_analyzer::EvidenceRef {
                row: row.clone(),
                reason: "objective row".to_string(),
            }],
            counter_evidence: Vec::new(),
        },
        session_archetype: None,
        session_progress: None,
        drift_scores: vec![agent_drift_analyzer::DriftScore {
            class: agent_drift_analyzer::DriftClass::WrongPlanBranch,
            state: if flagged {
                DriftState::Active
            } else {
                DriftState::Cleared
            },
            raw_score,
            confidence: Confidence::Medium,
            flagged,
            evidence: if flagged {
                vec![agent_drift_analyzer::EvidenceRef {
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

fn write_jsonl<T: serde::Serialize>(path: camino::Utf8PathBuf, items: &[T]) {
    let body = items
        .iter()
        .map(|item| serde_json::to_string(item).expect("json"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{body}\n")).expect("write jsonl");
}

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

fn sample_turn_context(turn_ordinal: usize) -> TurnContext {
    TurnContext {
        turn_id: Some(format!("turn-{turn_ordinal}")),
        turn_ordinal,
        rows_since_turn_start: 4,
        seconds_since_turn_start: Some(12),
        checkpoints_in_turn: 2,
        prompts_observed_in_session: turn_ordinal,
        execution_mode: TurnExecutionMode::Autonomous,
        activity_mix: TurnActivityMix {
            directive_row_count: 1,
            assistant_message_count: 1,
            tool_call_count: 1,
            read_like_command_count: 1,
            write_like_command_count: 1,
            verification_like_command_count: 0,
            tool_output_count: 1,
        },
    }
}

fn sample_session_archetype(checkpoint: &Checkpoint) -> SessionArchetype {
    SessionArchetype {
        label: SessionArchetypeLabel::Planning,
        confidence: Confidence::Medium,
        supporting_evidence: vec![agent_drift_analyzer::EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: "session archetype evidence".to_string(),
        }],
        counter_evidence: Vec::new(),
    }
}

fn sample_session_progress() -> SessionProgress {
    SessionProgress {
        status: ProgressStatus::InsufficientEvidence,
        dimension: ProgressDimension::PlanningConvergence,
        confidence: Confidence::Low,
        signals: Vec::new(),
        supporting_evidence: Vec::new(),
        counter_evidence: Vec::new(),
    }
}

#[test]
fn replay_input_loads_and_sorts_v0_3_checkpoints() {
    let fixture = ReplayFixture::from_checkpoints(
        vec![
            schema_checkpoint("v0.3", "session-beta", 2, 0, false, "continue"),
            schema_checkpoint("v0.3", "session-alpha", 3, 65, true, "repair"),
            schema_checkpoint("v0.3", "session-alpha", 1, 85, true, "repair"),
        ],
        sample_summary(),
    );

    let bundle = load_replay_bundle(&fixture.checkpoint_dir).expect("load replay bundle");

    assert_eq!(bundle.schema_version, "v0.3");
    assert_eq!(bundle.checkpoints.len(), 3);
    assert_eq!(bundle.checkpoints[0].checkpoint_id, "session-alpha:0001");
    assert_eq!(bundle.checkpoints[1].checkpoint_id, "session-alpha:0003");
    assert_eq!(bundle.checkpoints[2].checkpoint_id, "session-beta:0002");
}

#[test]
fn replay_input_loads_and_sorts_v0_4_checkpoints() {
    let mut session_beta = schema_checkpoint("v0.4", "session-beta", 2, 0, false, "continue");
    session_beta.turn_context = Some(sample_turn_context(2));
    let mut session_alpha_late = schema_checkpoint("v0.4", "session-alpha", 3, 65, true, "repair");
    session_alpha_late.turn_context = Some(sample_turn_context(3));
    let mut session_alpha_early = schema_checkpoint("v0.4", "session-alpha", 1, 85, true, "repair");
    session_alpha_early.turn_context = Some(sample_turn_context(1));

    let fixture = ReplayFixture::from_checkpoints(
        vec![session_beta, session_alpha_late, session_alpha_early],
        sample_summary(),
    );

    let bundle = load_replay_bundle(&fixture.checkpoint_dir).expect("load replay bundle");

    assert_eq!(bundle.schema_version, "v0.4");
    assert_eq!(bundle.checkpoints.len(), 3);
    assert_eq!(bundle.checkpoints[0].checkpoint_id, "session-alpha:0001");
    assert_eq!(bundle.checkpoints[1].checkpoint_id, "session-alpha:0003");
    assert_eq!(bundle.checkpoints[2].checkpoint_id, "session-beta:0002");
    assert_eq!(
        bundle.checkpoints[0]
            .turn_context
            .as_ref()
            .expect("v0.4 turn context")
            .execution_mode,
        TurnExecutionMode::Autonomous
    );
}

#[test]
fn replay_input_loads_and_sorts_v0_5_checkpoints() {
    let mut session_beta = schema_checkpoint("v0.5", "session-beta", 2, 0, false, "continue");
    session_beta.turn_context = Some(sample_turn_context(2));
    session_beta.session_archetype = Some(sample_session_archetype(&session_beta));

    let mut session_alpha_late = schema_checkpoint("v0.5", "session-alpha", 3, 65, true, "repair");
    session_alpha_late.turn_context = Some(sample_turn_context(3));
    session_alpha_late.session_archetype = Some(sample_session_archetype(&session_alpha_late));

    let mut session_alpha_early = schema_checkpoint("v0.5", "session-alpha", 1, 85, true, "repair");
    session_alpha_early.turn_context = Some(sample_turn_context(1));
    session_alpha_early.session_archetype = Some(sample_session_archetype(&session_alpha_early));

    let fixture = ReplayFixture::from_checkpoints(
        vec![session_beta, session_alpha_late, session_alpha_early],
        sample_summary(),
    );

    let bundle = load_replay_bundle(&fixture.checkpoint_dir).expect("load replay bundle");

    assert_eq!(bundle.schema_version, "v0.5");
    assert_eq!(bundle.checkpoints.len(), 3);
    assert_eq!(bundle.checkpoints[0].checkpoint_id, "session-alpha:0001");
    assert_eq!(bundle.checkpoints[1].checkpoint_id, "session-alpha:0003");
    assert_eq!(bundle.checkpoints[2].checkpoint_id, "session-beta:0002");
    assert_eq!(
        bundle.checkpoints[0]
            .session_archetype
            .as_ref()
            .expect("v0.5 session archetype")
            .label,
        SessionArchetypeLabel::Planning
    );
}

#[test]
fn replay_input_loads_and_sorts_v0_6_checkpoints() {
    let mut session_beta = schema_checkpoint("v0.6", "session-beta", 2, 0, false, "continue");
    session_beta.turn_context = Some(sample_turn_context(2));
    session_beta.session_archetype = Some(sample_session_archetype(&session_beta));
    session_beta.session_progress = Some(sample_session_progress());

    let mut session_alpha_late = schema_checkpoint("v0.6", "session-alpha", 3, 65, true, "repair");
    session_alpha_late.turn_context = Some(sample_turn_context(3));
    session_alpha_late.session_archetype = Some(sample_session_archetype(&session_alpha_late));
    session_alpha_late.session_progress = Some(sample_session_progress());

    let mut session_alpha_early = schema_checkpoint("v0.6", "session-alpha", 1, 85, true, "repair");
    session_alpha_early.turn_context = Some(sample_turn_context(1));
    session_alpha_early.session_archetype = Some(sample_session_archetype(&session_alpha_early));
    session_alpha_early.session_progress = Some(sample_session_progress());

    let fixture = ReplayFixture::from_checkpoints(
        vec![session_beta, session_alpha_late, session_alpha_early],
        sample_summary(),
    );

    let bundle = load_replay_bundle(&fixture.checkpoint_dir).expect("load replay bundle");

    assert_eq!(bundle.schema_version, "v0.6");
    assert_eq!(bundle.checkpoints.len(), 3);
    assert_eq!(bundle.checkpoints[0].checkpoint_id, "session-alpha:0001");
    assert_eq!(bundle.checkpoints[1].checkpoint_id, "session-alpha:0003");
    assert_eq!(bundle.checkpoints[2].checkpoint_id, "session-beta:0002");
    assert_eq!(
        bundle.checkpoints[0]
            .session_progress
            .as_ref()
            .expect("v0.6 session progress")
            .dimension,
        ProgressDimension::PlanningConvergence
    );
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
    fs::write(checkpoint_dir.join("summary.md"), sample_summary()).expect("write summary");

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
    fs::write(checkpoint_dir.join("summary.md"), sample_summary()).expect("write summary");

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
fn replay_input_rejects_v0_4_checkpoints_missing_turn_context() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
    let checkpoint_dir = root.join("checkpoint");
    fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");

    let mut checkpoint = schema_checkpoint("v0.4", "session-alpha", 1, 85, true, "repair");
    checkpoint.turn_context = Some(sample_turn_context(1));
    let mut malformed_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    malformed_json
        .as_object_mut()
        .expect("checkpoint object")
        .remove("turn_context");
    let malformed_line =
        serde_json::to_string(&malformed_json).expect("serialize malformed checkpoint");
    fs::write(
        checkpoint_dir.join("checkpoints.jsonl"),
        format!("{malformed_line}\n"),
    )
    .expect("write checkpoints");
    fs::write(checkpoint_dir.join("summary.md"), sample_summary()).expect("write summary");

    let error = load_replay_bundle(&checkpoint_dir)
        .expect_err("v0.4 checkpoints missing turn context must fail closed");

    assert!(matches!(
        error,
        InputError::ContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.4" && field == "turn_context"
    ));
    assert!(error.to_string().contains("missing turn_context"));
}

#[test]
fn replay_input_rejects_v0_5_checkpoints_missing_turn_context() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
    let checkpoint_dir = root.join("checkpoint");
    fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");

    let mut checkpoint = schema_checkpoint("v0.5", "session-alpha", 1, 85, true, "repair");
    checkpoint.turn_context = Some(sample_turn_context(1));
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));
    let mut malformed_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    malformed_json
        .as_object_mut()
        .expect("checkpoint object")
        .remove("turn_context");
    let malformed_line =
        serde_json::to_string(&malformed_json).expect("serialize malformed checkpoint");
    fs::write(
        checkpoint_dir.join("checkpoints.jsonl"),
        format!("{malformed_line}\n"),
    )
    .expect("write checkpoints");
    fs::write(checkpoint_dir.join("summary.md"), sample_summary()).expect("write summary");

    let error = load_replay_bundle(&checkpoint_dir)
        .expect_err("v0.5 checkpoints missing turn context must fail closed");

    assert!(matches!(
        error,
        InputError::ContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.5" && field == "turn_context"
    ));
    assert!(error.to_string().contains("missing turn_context"));
}

#[test]
fn replay_input_rejects_v0_5_checkpoints_missing_session_archetype() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
    let checkpoint_dir = root.join("checkpoint");
    fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");

    let mut checkpoint = schema_checkpoint("v0.5", "session-alpha", 1, 85, true, "repair");
    checkpoint.turn_context = Some(sample_turn_context(1));
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));
    let mut malformed_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    malformed_json
        .as_object_mut()
        .expect("checkpoint object")
        .remove("session_archetype");
    let malformed_line =
        serde_json::to_string(&malformed_json).expect("serialize malformed checkpoint");
    fs::write(
        checkpoint_dir.join("checkpoints.jsonl"),
        format!("{malformed_line}\n"),
    )
    .expect("write checkpoints");
    fs::write(checkpoint_dir.join("summary.md"), sample_summary()).expect("write summary");

    let error = load_replay_bundle(&checkpoint_dir)
        .expect_err("v0.5 checkpoints missing session archetype must fail closed");

    assert!(matches!(
        error,
        InputError::ContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.5" && field == "session_archetype"
    ));
    assert!(error.to_string().contains("missing session_archetype"));
}

#[test]
fn replay_input_rejects_v0_6_checkpoints_missing_session_progress() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
    let checkpoint_dir = root.join("checkpoint");
    fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");

    let mut checkpoint = schema_checkpoint("v0.6", "session-alpha", 1, 85, true, "repair");
    checkpoint.turn_context = Some(sample_turn_context(1));
    checkpoint.session_archetype = Some(sample_session_archetype(&checkpoint));
    checkpoint.session_progress = Some(sample_session_progress());
    let mut malformed_json =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint to json value");
    malformed_json
        .as_object_mut()
        .expect("checkpoint object")
        .remove("session_progress");
    let malformed_line =
        serde_json::to_string(&malformed_json).expect("serialize malformed checkpoint");
    fs::write(
        checkpoint_dir.join("checkpoints.jsonl"),
        format!("{malformed_line}\n"),
    )
    .expect("write checkpoints");
    fs::write(checkpoint_dir.join("summary.md"), sample_summary()).expect("write summary");

    let error = load_replay_bundle(&checkpoint_dir)
        .expect_err("v0.6 checkpoints missing session progress must fail closed");

    assert!(matches!(
        error,
        InputError::ContractGap {
            ref schema_version,
            ref field,
            ..
        } if schema_version == "v0.6" && field == "session_progress"
    ));
    assert!(error.to_string().contains("missing session_progress"));
}

#[test]
fn replay_input_preserves_explicit_v0_3_drift_state() {
    let mut checkpoint = schema_checkpoint("v0.3", "session-alpha", 1, 20, false, "continue");
    checkpoint.drift_scores[0].state = DriftState::HistoricalOnly;
    checkpoint.drift_scores[0].evidence = vec![agent_drift_analyzer::EvidenceRef {
        row: checkpoint.boundary.start.clone(),
        reason: "explicit analyzer historical state".to_string(),
    }];

    let fixture = ReplayFixture::from_checkpoints(vec![checkpoint.clone()], sample_summary());

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
        sample_checkpoints()
            .into_iter()
            .map(|mut checkpoint| {
                checkpoint.schema_version = "v0.2".to_string();
                checkpoint
            })
            .collect(),
        sample_summary(),
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
    fs::write(checkpoint_dir.join("summary.md"), sample_summary()).expect("write summary");

    let bundle = load_replay_bundle(&checkpoint_dir).expect("load replay bundle");

    assert_eq!(
        bundle.checkpoints[0].drift_scores[0].class,
        agent_drift_analyzer::DriftClass::TruthGroundingGap
    );
}
