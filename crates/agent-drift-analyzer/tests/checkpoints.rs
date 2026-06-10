#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{
    AnalyzeRequest, AnalyzeResult, Confidence, ProgressDimension, ProgressSignalCode,
    ProgressStatus, SessionArchetypeLabel,
};
use agent_session_compactor::{
    CompactionKind, CompactionRow, DedupeGroup, RowRef, SourceKind, UserMessageRole,
};
use camino::Utf8PathBuf;
use serde_json::Value;
use support::{analyze_sample_bundle, load_sample_bundle, BundleFixture};
use time::macros::datetime;

#[test]
fn checkpoints_are_deterministic_and_session_scoped() {
    let first = analyze_sample_bundle();
    let second = analyze_sample_bundle();

    assert_eq!(
        first.sessions[0].checkpoints,
        second.sessions[0].checkpoints
    );
    let checkpoints = &first.sessions[0].checkpoints;
    assert_eq!(checkpoints.len(), 2);
    assert_eq!(checkpoints[0].session_id, "session-alpha");
    assert_eq!(checkpoints[0].schema_version, "v0.6");
    assert!(checkpoints[0].session_archetype.is_some());
    assert!(checkpoints[0].session_progress.is_some());
    assert_eq!(checkpoints[0].ordinal, 1);
    let first_turn = checkpoints[0]
        .turn_context
        .as_ref()
        .expect("first turn context");
    assert_eq!(first_turn.turn_id.as_deref(), Some("turn-001"));
    assert_eq!(first_turn.turn_ordinal, 1);
    assert_eq!(first_turn.rows_since_turn_start, 9);
    assert_eq!(first_turn.seconds_since_turn_start, None);
    assert_eq!(first_turn.checkpoints_in_turn, 1);
    assert_eq!(first_turn.prompts_observed_in_session, 1);
    assert_eq!(checkpoints[1].ordinal, 2);
    assert_eq!(checkpoints[1].schema_version, "v0.6");
    assert!(checkpoints[1].session_archetype.is_some());
    assert!(checkpoints[1].session_progress.is_some());
    let second_turn = checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");
    assert_eq!(second_turn.turn_id.as_deref(), Some("turn-001"));
    assert_eq!(second_turn.turn_ordinal, 1);
    assert_eq!(second_turn.rows_since_turn_start, 13);
    assert_eq!(second_turn.seconds_since_turn_start, None);
    assert_eq!(second_turn.checkpoints_in_turn, 2);
    assert_eq!(second_turn.prompts_observed_in_session, 1);
    assert_eq!(checkpoints[0].boundary.end.event_index, 8);
    assert_eq!(checkpoints[1].boundary.end.event_index, 12);
    assert!(checkpoints[0].boundary.end.event_index < checkpoints[1].boundary.end.event_index);
}

#[test]
fn checkpoints_keep_legacy_session_progress_loads_but_fail_closed_for_v0_6() {
    let checkpoint = analyze_sample_bundle().sessions[0].checkpoints[0].clone();

    for schema_version in ["v0.2", "v0.3", "v0.4"] {
        let mut legacy_json = serde_json::to_value(&checkpoint).expect("serialize checkpoint");
        legacy_json["schema_version"] = Value::String(schema_version.to_string());
        legacy_json
            .as_object_mut()
            .expect("checkpoint object")
            .remove("session_progress");
        legacy_json
            .as_object_mut()
            .expect("checkpoint object")
            .remove("session_archetype");

        let parsed: agent_drift_analyzer::Checkpoint =
            serde_json::from_value(legacy_json).expect("legacy checkpoint stays loadable");
        assert_eq!(parsed.schema_version, schema_version);
        assert!(parsed.session_archetype.is_none());
        assert!(parsed.session_progress.is_none());
    }

    let mut v0_5_without_progress =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint");
    v0_5_without_progress["schema_version"] = Value::String("v0.5".to_string());
    v0_5_without_progress
        .as_object_mut()
        .expect("checkpoint object")
        .remove("session_progress");
    let parsed_v0_5: agent_drift_analyzer::Checkpoint =
        serde_json::from_value(v0_5_without_progress)
            .expect("v0.5 stays loadable without progress");
    assert_eq!(parsed_v0_5.schema_version, "v0.5");
    assert!(parsed_v0_5.session_archetype.is_some());
    assert!(parsed_v0_5.session_progress.is_none());

    let mut missing_v0_6_progress =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint");
    missing_v0_6_progress
        .as_object_mut()
        .expect("checkpoint object")
        .remove("session_progress");
    let err = serde_json::from_value::<agent_drift_analyzer::Checkpoint>(missing_v0_6_progress)
        .expect_err("v0.6 checkpoint without session_progress must fail");
    let message = err.to_string();
    assert!(message.contains("v0.6"));
    assert!(message.contains("session_progress"));

    let mut missing_v0_6_archetype =
        serde_json::to_value(&checkpoint).expect("serialize checkpoint");
    missing_v0_6_archetype
        .as_object_mut()
        .expect("checkpoint object")
        .remove("session_archetype");
    let err = serde_json::from_value::<agent_drift_analyzer::Checkpoint>(missing_v0_6_archetype)
        .expect_err("v0.6 checkpoint without session_archetype must fail");
    let message = err.to_string();
    assert!(message.contains("v0.6"));
    assert!(message.contains("session_archetype"));

    let round_tripped: agent_drift_analyzer::Checkpoint =
        serde_json::from_value(serde_json::to_value(&checkpoint).expect("serialize checkpoint"))
            .expect("v0.6 checkpoint with progress should round-trip");
    assert_eq!(round_tripped, checkpoint);
}

#[test]
fn checkpoints_emit_conservative_session_progress_placeholders() {
    let checkpoint = &analyze_sample_bundle().sessions[0].checkpoints[0];
    let session_archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let session_progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    let expected_dimension = match session_archetype.label {
        SessionArchetypeLabel::Troubleshooting => ProgressDimension::TroubleshootingFrontier,
        SessionArchetypeLabel::Planning => ProgressDimension::PlanningConvergence,
        SessionArchetypeLabel::AutonomousImplementation => {
            ProgressDimension::ImplementationVerificationWall
        }
        SessionArchetypeLabel::VerificationCloseout => {
            ProgressDimension::VerificationCloseoutNarrowing
        }
    };

    assert_eq!(
        session_progress.status,
        ProgressStatus::InsufficientEvidence
    );
    assert_eq!(session_progress.dimension, expected_dimension);
    assert_eq!(session_progress.confidence, Confidence::Low);
    assert!(session_progress.signals.is_empty());
    assert!(session_progress.supporting_evidence.is_empty());
    assert!(session_progress.counter_evidence.is_empty());
}

#[test]
fn checkpoints_compute_turn_timing_from_turn_slice_boundaries() {
    let mut bundle = load_sample_bundle();
    for row in &mut bundle.archival_rows {
        row.timestamp = Some(
            datetime!(2026-05-29 12:00:00 UTC)
                + time::Duration::seconds(row.event_index as i64 * 60),
        );
    }
    for row in &mut bundle.compact_rows {
        row.timestamp = Some(
            datetime!(2026-05-29 12:00:00 UTC)
                + time::Duration::seconds(row.event_index as i64 * 60),
        );
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle with timestamps");
    let checkpoints = &result.sessions[0].checkpoints;

    assert_eq!(
        checkpoints[0]
            .turn_context
            .as_ref()
            .and_then(|turn| turn.seconds_since_turn_start),
        Some(480)
    );
    assert_eq!(
        checkpoints[1]
            .turn_context
            .as_ref()
            .and_then(|turn| turn.seconds_since_turn_start),
        Some(720)
    );
}

#[test]
fn checkpoints_start_a_new_turn_when_the_boundary_moves_to_a_new_turn_id() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
        .filter(|row| row.event_index >= 9)
    {
        row.turn_id = Some("turn-002".to_string());
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle with two turns");
    let checkpoints = &result.sessions[0].checkpoints;
    let second_turn = checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(second_turn.turn_id.as_deref(), Some("turn-002"));
    assert_eq!(second_turn.turn_ordinal, 2);
    assert_eq!(second_turn.rows_since_turn_start, 4);
    assert_eq!(second_turn.checkpoints_in_turn, 1);
    assert_eq!(second_turn.prompts_observed_in_session, 1);
}

#[test]
fn checkpoints_compute_turn_activity_mix_and_execution_modes() {
    let result = analyze_sample_bundle();
    let checkpoints = &result.sessions[0].checkpoints;
    let first_turn = checkpoints[0]
        .turn_context
        .as_ref()
        .expect("first turn context");
    let second_turn = checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(first_turn.activity_mix.directive_row_count, 2);
    assert_eq!(first_turn.activity_mix.assistant_message_count, 0);
    assert_eq!(first_turn.activity_mix.tool_call_count, 5);
    assert_eq!(first_turn.activity_mix.read_like_command_count, 2);
    assert_eq!(first_turn.activity_mix.write_like_command_count, 0);
    assert_eq!(first_turn.activity_mix.verification_like_command_count, 3);
    assert_eq!(first_turn.activity_mix.tool_output_count, 2);
    assert_eq!(
        first_turn.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::VerificationHeavy
    );

    assert_eq!(second_turn.activity_mix.directive_row_count, 2);
    assert_eq!(second_turn.activity_mix.assistant_message_count, 1);
    assert_eq!(second_turn.activity_mix.tool_call_count, 8);
    assert_eq!(second_turn.activity_mix.read_like_command_count, 2);
    assert_eq!(second_turn.activity_mix.write_like_command_count, 2);
    assert_eq!(second_turn.activity_mix.verification_like_command_count, 4);
    assert_eq!(second_turn.activity_mix.tool_output_count, 2);
    assert_eq!(
        second_turn.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Autonomous
    );
}

#[test]
fn checkpoints_recognize_multiple_checkpoints_in_one_long_autonomous_turn() {
    let result = analyze_sample_bundle();
    let checkpoints = &result.sessions[0].checkpoints;

    assert_eq!(checkpoints.len(), 2);
    assert_eq!(checkpoints[0].session_id, "session-alpha");
    assert_eq!(checkpoints[1].session_id, "session-alpha");
    assert_eq!(checkpoints[0].ordinal, 1);
    assert_eq!(checkpoints[1].ordinal, 2);

    let first_turn = checkpoints[0]
        .turn_context
        .as_ref()
        .expect("first turn context");
    let second_turn = checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(first_turn.turn_id.as_deref(), Some("turn-001"));
    assert_eq!(second_turn.turn_id.as_deref(), Some("turn-001"));
    assert_eq!(first_turn.turn_ordinal, 1);
    assert_eq!(second_turn.turn_ordinal, 1);
    assert_eq!(first_turn.checkpoints_in_turn, 1);
    assert_eq!(second_turn.checkpoints_in_turn, 2);
    assert!(
        first_turn.rows_since_turn_start < second_turn.rows_since_turn_start,
        "later checkpoints in one long turn should accumulate more rows"
    );
    assert_eq!(
        second_turn.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Autonomous
    );
}

#[test]
fn checkpoints_mark_cargo_heavy_write_turns_as_mixed() {
    let result = analyze_sample_bundle();
    let first_turn = result.sessions[0].checkpoints[0]
        .turn_context
        .as_ref()
        .expect("first turn context");

    assert_eq!(first_turn.activity_mix.tool_call_count, 5);
    assert_eq!(first_turn.activity_mix.write_like_command_count, 0);
    assert_eq!(first_turn.activity_mix.verification_like_command_count, 3);
    assert_eq!(
        first_turn.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::VerificationHeavy
    );
}

#[test]
fn checkpoints_bias_same_turn_verification_plurality_overlap_to_mixed() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
    {
        match row.event_index {
            2 | 8 => {
                row.text =
                    "{\"command\":\"sed -n '1,40p' crates/agent-drift-analyzer/src/lib.rs\",\"workdir\":\"/repo\"}"
                        .to_string();
            }
            3 | 4 | 5 | 10 | 11 | 12 => {
                row.text =
                    "{\"command\":\"npm test -- --runInBand\",\"workdir\":\"/repo\"}".to_string();
            }
            _ => {}
        }
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze overlapping mode bundle");
    let second_turn = result.sessions[0].checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(second_turn.checkpoints_in_turn, 2);
    assert_eq!(second_turn.activity_mix.tool_call_count, 8);
    assert_eq!(second_turn.activity_mix.read_like_command_count, 2);
    assert_eq!(second_turn.activity_mix.write_like_command_count, 0);
    assert_eq!(second_turn.activity_mix.verification_like_command_count, 6);
    assert_eq!(
        second_turn.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Mixed
    );
}

#[test]
fn checkpoints_mark_single_checkpoint_verification_plurality_turns_as_verification_heavy() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
        .filter(|row| row.event_index >= 9)
    {
        row.turn_id = Some("turn-002".to_string());
        match row.event_index {
            10 => {
                row.text =
                    "{\"command\":\"npm test -- --runInBand\",\"workdir\":\"/repo\"}".to_string();
            }
            11 => {
                row.text = "{\"command\":\"pnpm lint\",\"workdir\":\"/repo\"}".to_string();
            }
            12 => {
                row.text =
                    "{\"command\":\"pnpm test -- --runInBand\",\"workdir\":\"/repo\"}".to_string();
            }
            _ => {}
        }
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze verification-heavy turn bundle");
    let second_turn = result.sessions[0].checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(second_turn.checkpoints_in_turn, 1);
    assert_eq!(second_turn.activity_mix.directive_row_count, 0);
    assert_eq!(second_turn.activity_mix.assistant_message_count, 1);
    assert_eq!(second_turn.activity_mix.tool_call_count, 3);
    assert_eq!(second_turn.activity_mix.read_like_command_count, 0);
    assert_eq!(second_turn.activity_mix.write_like_command_count, 0);
    assert_eq!(second_turn.activity_mix.verification_like_command_count, 3);
    assert_eq!(
        second_turn.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::VerificationHeavy
    );
}

#[test]
fn checkpoints_mark_tool_free_short_turns_as_conversational() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
        .filter(|row| row.event_index >= 9)
    {
        row.turn_id = Some("turn-002".to_string());
        match row.event_index {
            10 => {
                row.kind = agent_session_compactor::CompactionKind::AssistantMessage;
                row.text = "I am summarizing the next patch step.".to_string();
                row.dedupe_identity = None;
            }
            11 => {
                row.kind = agent_session_compactor::CompactionKind::DeveloperMessage;
                row.text = "Stay inside Packet R3-4 only.".to_string();
                row.dedupe_identity = None;
            }
            12 => {
                row.kind = agent_session_compactor::CompactionKind::AssistantMessage;
                row.text = "Waiting for the next instruction.".to_string();
                row.dedupe_identity = None;
            }
            _ => {}
        }
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze conversational turn bundle");
    let second_turn = result.sessions[0].checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(second_turn.activity_mix.directive_row_count, 1);
    assert_eq!(second_turn.activity_mix.assistant_message_count, 3);
    assert_eq!(second_turn.activity_mix.tool_call_count, 0);
    assert_eq!(second_turn.activity_mix.tool_output_count, 0);
    assert_eq!(
        second_turn.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Conversational
    );
}

#[test]
fn checkpoints_bias_ambiguous_turns_to_mixed() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
        .filter(|row| row.event_index >= 9)
    {
        row.turn_id = Some("turn-002".to_string());
        match row.event_index {
            10 => {
                row.text =
                    "{\"command\":\"sed -n '1,40p' crates/agent-drift-analyzer/src/lib.rs\",\"workdir\":\"/repo\"}"
                        .to_string();
                row.dedupe_identity = Some(
                    "{\"call_id\":\"call-9\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"
                        .to_string(),
                );
            }
            11 => {
                row.text = "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}".to_string();
                row.dedupe_identity = Some(
                    "{\"call_id\":\"call-10\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"
                        .to_string(),
                );
            }
            12 => {
                row.kind = agent_session_compactor::CompactionKind::ToolOutput;
                row.text = "patched lib.rs".to_string();
                row.dedupe_identity = None;
            }
            _ => {}
        }
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze ambiguous turn bundle");
    let second_turn = result.sessions[0].checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(second_turn.activity_mix.directive_row_count, 0);
    assert_eq!(second_turn.activity_mix.assistant_message_count, 1);
    assert_eq!(second_turn.activity_mix.tool_call_count, 2);
    assert_eq!(second_turn.activity_mix.read_like_command_count, 1);
    assert_eq!(second_turn.activity_mix.write_like_command_count, 1);
    assert_eq!(second_turn.activity_mix.verification_like_command_count, 0);
    assert_eq!(second_turn.activity_mix.tool_output_count, 1);
    assert_eq!(
        second_turn.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Mixed
    );
}

#[test]
fn checkpoints_degrade_conservatively_when_no_turn_id_is_available() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
    {
        row.turn_id = None;
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle without turn ids");
    let checkpoints = &result.sessions[0].checkpoints;
    let first_turn = checkpoints[0]
        .turn_context
        .as_ref()
        .expect("first turn context");
    let second_turn = checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(first_turn.turn_id, None);
    assert_eq!(first_turn.turn_ordinal, 0);
    assert_eq!(first_turn.rows_since_turn_start, 9);
    assert_eq!(first_turn.checkpoints_in_turn, 1);
    assert_eq!(first_turn.prompts_observed_in_session, 1);

    assert_eq!(second_turn.turn_id, None);
    assert_eq!(second_turn.turn_ordinal, 0);
    assert_eq!(second_turn.rows_since_turn_start, 4);
    assert_eq!(second_turn.checkpoints_in_turn, 1);
    assert_eq!(second_turn.prompts_observed_in_session, 1);
}

#[test]
fn checkpoints_render_single_agent_delegation_summary_as_none() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let summary = fs::read_to_string(&result.summary_path).expect("summary");

    assert!(summary.contains(
        "  delegation: `topology=single_agent visibility=none confidence=high markers=none support[none] counter[none]`"
    ));
}

#[test]
fn checkpoints_classify_docs_heavy_scope_shaping_as_planning() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Plan Packet R4-2 and keep the landing strictly scoped.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'Packet R4-2' docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,200p' docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let archetype = result.sessions[0].checkpoints[0]
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(archetype.label, SessionArchetypeLabel::Planning);
    assert!(matches!(
        archetype.confidence,
        Confidence::Medium | Confidence::High
    ));
    assert!(!archetype.supporting_evidence.is_empty());
    assert!(archetype
        .supporting_evidence
        .iter()
        .any(|evidence| evidence.reason.contains("planning-orchestration")));
    assert!(!archetype.counter_evidence.is_empty());
}

#[test]
fn checkpoints_classify_source_edit_plus_local_verification_as_autonomous_implementation() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Implement the checkpoint archetype classifier in crates/agent-drift-analyzer/src/checkpoint/mod.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,260p' crates/agent-drift-analyzer/src/checkpoint/mod.rs\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/mod.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/tests/checkpoints.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let archetype = result.sessions[0].checkpoints[0]
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(
        archetype.label,
        SessionArchetypeLabel::AutonomousImplementation
    );
    assert!(matches!(
        archetype.confidence,
        Confidence::Medium | Confidence::High
    ));
    assert!(!archetype.supporting_evidence.is_empty());
    assert!(!archetype.supporting_evidence.iter().any(|evidence| {
        evidence.row.event_index == 4
            && evidence
                .reason
                .contains("verification command strengthened verification-like evidence")
    }));
    assert!(archetype.counter_evidence.iter().any(|evidence| {
        evidence.row.event_index == 4
            && evidence
                .reason
                .contains("verification command strengthened verification-like evidence")
    }));
}

#[test]
fn checkpoints_keep_read_only_test_and_checkpoint_path_commands_out_of_verification() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Inspect the analyzer state before choosing a packet-scoped fix.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,120p' crates/agent-drift-analyzer/tests/checkpoints.rs\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'session_archetype' crates/agent-drift-analyzer/src/checkpoint/mod.rs\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoint = &result.sessions[0].checkpoints[0];
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(archetype.label, SessionArchetypeLabel::Planning);
    assert_eq!(
        checkpoint
            .turn_context
            .as_ref()
            .expect("turn context")
            .activity_mix
            .verification_like_command_count,
        0
    );
    assert!(matches!(
        archetype.confidence,
        Confidence::Low | Confidence::Medium
    ));
}

#[test]
fn checkpoints_bias_sparse_neutral_prefixes_to_low_confidence_planning() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Inspect the landed Packet R4-2 scope before choosing a narrow follow-up.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"echo packet-r4-2-status\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"printf packet-r4-2-next-step\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoint = &result.sessions[0].checkpoints[0];
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(archetype.label, SessionArchetypeLabel::Planning);
    assert_eq!(archetype.confidence, Confidence::Low);
    assert_eq!(
        checkpoint
            .turn_context
            .as_ref()
            .expect("turn context")
            .activity_mix
            .verification_like_command_count,
        0
    );
}

#[test]
fn checkpoints_shift_to_verification_closeout_when_proof_dominates_new_source_edits() {
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", "/goal Implement the patch and then verify it."),
        tool_call_row(
            1,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/mod.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        assistant_row(
            3,
            "turn-001",
            "The implementation is in place. Next I am gathering proof only.",
        ),
        prompt_row(
            4,
            "turn-002",
            "/goal Verify the existing patch and gather proof only.",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo fmt --all -- --check\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            7,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,120p' crates/agent-drift-analyzer/tests/checkpoints.rs\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoints = &result.sessions[0].checkpoints;
    assert!(checkpoints.len() >= 2);
    let archetype = checkpoints
        .last()
        .expect("final checkpoint")
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(archetype.label, SessionArchetypeLabel::VerificationCloseout);
    assert!(matches!(
        archetype.confidence,
        Confidence::Medium | Confidence::High
    ));
    assert!(!archetype.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_classify_repeated_failing_verification_as_troubleshooting() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Debug the failing analyzer checkpoint tests.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'session_archetype' crates/agent-drift-analyzer/src/checkpoint/mod.rs\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(3, "turn-001", "Exit code: 1"),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,220p' crates/agent-drift-analyzer/src/checkpoint/mod.rs\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(6, "turn-001", "Exit code: 1"),
        tool_call_row(
            7,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(8, "turn-001", "Exit code: 1"),
    ]);
    let archetype = result.sessions[0].checkpoints[0]
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(archetype.label, SessionArchetypeLabel::Troubleshooting);
    assert!(matches!(
        archetype.confidence,
        Confidence::Medium | Confidence::High
    ));
    assert!(!archetype.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_cap_confidence_when_parent_visible_behavior_is_child_opaque() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Coordinate a delegated implementation while keeping parent-visible work conservative.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "multi_agent_v1",
            "{\"mode\":\"delegated\"}",
        ),
        developer_row(
            2,
            "turn-001",
            "Child session id 019ea222-2222-7222-8222-222222222222 remains in separate rollout /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-30-00-019ea222-2222-7222-8222-222222222222.jsonl",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'session_archetype' docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let archetype = result.sessions[0]
        .checkpoints
        .last()
        .expect("checkpoint")
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert!(matches!(
        archetype.confidence,
        Confidence::Low | Confidence::Medium
    ));
    assert_ne!(archetype.confidence, Confidence::High);
    assert!(!archetype.counter_evidence.is_empty());
}

#[test]
fn checkpoints_keep_unknown_shallow_parse_command_families_neutral() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Inspect the repo state before deciding whether a packet-scoped fix is needed.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo metadata --format-version 1\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"npm exec playwright --version\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"pnpm dlx tsx scripts/report.ts\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"git checkout feature/r4-2-review-fix\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoint = &result.sessions[0].checkpoints[0];
    let turn_context = checkpoint.turn_context.as_ref().expect("turn context");
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(turn_context.activity_mix.read_like_command_count, 0);
    assert_eq!(turn_context.activity_mix.write_like_command_count, 0);
    assert_eq!(turn_context.activity_mix.verification_like_command_count, 0);
    assert_eq!(
        turn_context.execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Mixed
    );
    assert_eq!(archetype.label, SessionArchetypeLabel::Planning);
    assert!(matches!(
        archetype.confidence,
        Confidence::Low | Confidence::Medium
    ));
}

#[test]
fn checkpoints_degrade_mixed_cases_instead_of_overclaiming_high_confidence() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Investigate, patch, and verify the analyzer while updating the docs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'session_archetype' crates/agent-drift-analyzer/src/checkpoint/mod.rs\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/mod.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let archetype = result.sessions[0].checkpoints[0]
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert!(matches!(
        archetype.confidence,
        Confidence::Low | Confidence::Medium
    ));
}

#[test]
fn checkpoints_lock_ambiguous_mixed_fixture_as_medium_autonomous_implementation() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Investigate, patch, and verify the analyzer while updating the docs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'session_archetype' crates/agent-drift-analyzer/src/checkpoint/mod.rs\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/mod.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let archetype = result.sessions[0].checkpoints[0]
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(
        archetype.label,
        SessionArchetypeLabel::AutonomousImplementation
    );
    assert_eq!(archetype.confidence, Confidence::Medium);
    assert_evidence_contains(
        &archetype.supporting_evidence,
        "stable working set plus source edits supported concentrated implementation",
    );
    assert_evidence_contains(
        &archetype.supporting_evidence,
        "source edit command strengthened implementation-like evidence",
    );
    assert_evidence_contains(
        &archetype.counter_evidence,
        "inspection-style command widened the visible search space",
    );
    assert_evidence_contains(
        &archetype.counter_evidence,
        "verification command strengthened verification-like evidence",
    );
}

#[test]
fn checkpoints_lock_transition_from_planning_to_implementation_without_flapping() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Plan the narrow Packet R4-3 landing first.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,200p' docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'Packet R4-3' docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md\",\"workdir\":\"/repo\"}",
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Land the agreed Packet R4-3 code changes.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/export.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer export_bundle -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints.len(), 2);
    let first = checkpoints[0]
        .session_archetype
        .as_ref()
        .expect("first session archetype");
    let second = checkpoints[1]
        .session_archetype
        .as_ref()
        .expect("second session archetype");

    assert_eq!(first.label, SessionArchetypeLabel::Planning);
    assert_eq!(first.confidence, Confidence::Medium);
    assert_eq!(
        second.label,
        SessionArchetypeLabel::AutonomousImplementation
    );
    assert_eq!(second.confidence, Confidence::Medium);
    assert_ne!(first.label, second.label);
    assert_evidence_contains(
        &second.supporting_evidence,
        "source edit command strengthened implementation-like evidence",
    );
    assert_evidence_contains(
        &second.counter_evidence,
        "task-frame transition signaled active scope exploration",
    );
    assert_evidence_contains(
        &second.counter_evidence,
        "verification command strengthened verification-like evidence",
    );
}

#[test]
fn checkpoints_lock_delegated_parent_opaque_fixture_as_low_confidence_planning() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Coordinate a delegated implementation while keeping parent-visible work conservative.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "multi_agent_v1",
            "{\"mode\":\"delegated\"}",
        ),
        developer_row(
            2,
            "turn-001",
            "Child session id 019ea222-2222-7222-8222-222222222222 remains in separate rollout /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-30-00-019ea222-2222-7222-8222-222222222222.jsonl",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'session_archetype' docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let archetype = result.sessions[0]
        .checkpoints
        .last()
        .expect("checkpoint")
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(archetype.label, SessionArchetypeLabel::Planning);
    assert_eq!(archetype.confidence, Confidence::Low);
    assert_evidence_contains(
        &archetype.supporting_evidence,
        "delegation topology kept orchestration evidence in the visible parent prefix",
    );
    assert_evidence_contains(
        &archetype.supporting_evidence,
        "visible delegation topology informed the checkpoint-local archetype decision",
    );
    assert_evidence_contains(
        &archetype.counter_evidence,
        "child-opaque delegation limited direct confidence in parent-visible archetype semantics",
    );
    assert_evidence_contains(
        &archetype.counter_evidence,
        "delegating-parent plus child-opaque visibility capped checkpoint-local archetype certainty",
    );
}

#[test]
fn checkpoints_lock_pr_response_loop_as_medium_autonomous_implementation() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Address PR feedback for Packet R4-3 with a narrow patch and local proof.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,220p' crates/agent-drift-analyzer/src/checkpoint/export.rs\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/export.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/tests/export_bundle.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer export_bundle -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let archetype = result.sessions[0].checkpoints[0]
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(
        archetype.label,
        SessionArchetypeLabel::AutonomousImplementation
    );
    assert_eq!(archetype.confidence, Confidence::Medium);
    assert_evidence_contains(
        &archetype.supporting_evidence,
        "stable working set plus source edits supported concentrated implementation",
    );
    assert_evidence_contains(
        &archetype.supporting_evidence,
        "source edit command strengthened implementation-like evidence",
    );
    assert_evidence_contains(
        &archetype.counter_evidence,
        "inspection-style command widened the visible search space",
    );
    assert_evidence_contains(
        &archetype.counter_evidence,
        "verification command strengthened verification-like evidence",
    );
}

#[test]
fn checkpoints_progress_falls_back_to_parent_visible_orchestration_for_opaque_parent_work() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Coordinate delegated work without overclaiming child progress.",
        ),
        tool_call_row(1, "turn-001", "spawn_agent", "{\"goal\":\"fix packet R5-4\"}"),
        developer_row(
            2,
            "turn-001",
            "Child session id 019ea333-3333-7333-8333-333333333333 remains in a separate rollout file.",
        ),
        tool_call_row(
            3,
            "turn-001",
            "wait_agent",
            "{\"session_id\":\"019ea333-3333-7333-8333-333333333333\"}",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");
    assert_eq!(
        progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled | ProgressStatus::Mixed
    ));
    assert_eq!(progress.confidence, Confidence::Low);
    assert_progress_signal(progress, ProgressSignalCode::DelegationVisibilityLimited);
}

#[test]
fn checkpoints_keep_parent_visible_synthesis_capped_under_partial_child_visibility() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Coordinate delegated findings without claiming child execution progress.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "spawn_agent",
            "{\"goal\":\"fix packet R5-4\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"printf 'child rollout ' && sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl","workdir":"/repo"}"#,
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            4,
            "turn-001",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
    assert_eq!(progress.status, ProgressStatus::Mixed);
    assert_eq!(progress.confidence, Confidence::Medium);
    assert_progress_signal(progress, ProgressSignalCode::DelegationVisibilityLimited);
    assert!(!progress.signals.iter().any(|signal| {
        matches!(
            signal.code,
            ProgressSignalCode::FailureFrontierAdvanced | ProgressSignalCode::VerificationClean
        )
    }));
}

#[test]
fn checkpoints_keep_visible_child_result_plus_parent_plan_refinement_in_planning() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Coordinate delegated findings while refining the packet plan myself.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "spawn_agent",
            r#"{"goal":"inspect packet R5-4"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"printf 'child rollout ' && sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl","workdir":"/repo"}"#,
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '170,230p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
    assert_progress_signal(progress, ProgressSignalCode::PlanArtifactCreated);
    assert_ne!(
        progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
}

#[test]
fn checkpoints_mark_troubleshooting_frontier_advancement_from_compile_to_test_failure() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot why checkpoints::captures_progress is blocked.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,200p' crates/agent-drift-analyzer/src/checkpoint/progress.rs\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            3,
            "turn-001",
            "Exit code: 101\nerror[E0425]: cannot find value `progress` in this scope\n --> crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34\ncould not compile `agent-drift-analyzer` (lib test) due to 1 previous error",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            6,
            "turn-001",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert!(matches!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
            | ProgressDimension::ImplementationVerificationWall
    ));
    assert_eq!(progress.status, ProgressStatus::Advancing);
    assert_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
    assert_progress_signal(progress, ProgressSignalCode::FailingScopeEdited);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_mark_troubleshooting_target_not_exercised_as_insufficient() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Debug the failing analyzer checkpoint tests before the target test executes.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            3,
            "turn-001",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
        prompt_row(
            4,
            "turn-002",
            "/goal Re-run the same blocked verifier while still debugging the same scope.",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '120,220p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            7,
            "turn-002",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
    assert_progress_signal(progress, ProgressSignalCode::TargetNotExercised);
}

#[test]
fn checkpoints_mark_troubleshooting_regression_when_frontier_falls_back() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot the failing checkpoint verifier without changing scope.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            2,
            "turn-001",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
        tool_call_row(
            3,
            "turn-002",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            5,
            "turn-002",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
        tool_call_row(
            6,
            "turn-003",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            7,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            8,
            "turn-003",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::Regressing);
    assert_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
}

#[test]
fn checkpoints_mark_repeated_same_troubleshooting_signature_as_stalled() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot checkpoints::captures_progress without changing scope.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            2,
            "turn-001",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Re-run the same troubleshooting verifier before widening scope.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            5,
            "turn-002",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::Stalled);
    assert_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_mark_planning_narrowing_as_advancing() {
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", "/goal Plan Packet R5-4 conservatively."),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'R5-4' docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,200p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md\",\"workdir\":\"/repo\"}",
        ),
        prompt_row(3, "turn-002", "/goal Narrow the plan to the concrete packet artifact."),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"sed -n '170,230p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
    assert_eq!(progress.status, ProgressStatus::Advancing);
    assert_progress_signal(progress, ProgressSignalCode::PlanArtifactCreated);
    assert_progress_signal(progress, ProgressSignalCode::CandidateSetNarrowed);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_mark_broad_planning_meander_as_stalled() {
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", "/goal Plan Packet R5-4 before coding."),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'progress' docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,200p' docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md\",\"workdir\":\"/repo\"}",
        ),
        prompt_row(3, "turn-002", "/goal Keep scanning before committing to an artifact."),
        tool_call_row(
            4,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,200p' docs/specs/r5/DESIGN-r5-archetype-progress-rules.md\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,200p' docs/specs/r5/DESIGN-r5-delegation-progress-guardrails.md\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
    assert_eq!(progress.status, ProgressStatus::Stalled);
    assert_progress_signal(progress, ProgressSignalCode::CandidateSetExpanded);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_mark_implementation_wall_advancement_with_concentrated_edits() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5-4 implementation in checkpoint/progress.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            2,
            "turn-001",
            "Exit code: 101\nerror[E0425]: cannot find value `progress` in this scope\ncould not compile `agent-drift-analyzer` (lib test) due to 1 previous error",
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Apply the narrow source and test patch, then rerun the same verifier.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            6,
            "turn-002",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
    );
    assert_eq!(progress.status, ProgressStatus::Advancing);
    assert_progress_signal(progress, ProgressSignalCode::WorkingSetConcentrated);
    assert_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_do_not_mark_concentrated_implementation_edit_as_advancing_without_verifier_progress()
{
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5-4 implementation in checkpoint/progress.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            3,
            "turn-001",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
    );
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
    ));
}

#[test]
fn checkpoints_keep_implementation_present_to_missing_fail_count_conservative() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5-4 implementation in checkpoint/progress.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            2,
            "turn-001",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\nAssertionError: expected advancing",
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Apply the narrow Packet R5-4 patch, then rerun the same verifier.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            6,
            "turn-002",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nAssertionError: expected advancing",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
    );
    assert_eq!(progress.status, ProgressStatus::Stalled);
    assert_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
}

#[test]
fn checkpoints_keep_implementation_missing_to_present_fail_count_conservative() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5-4 implementation in checkpoint/progress.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            2,
            "turn-001",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nAssertionError: expected advancing",
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Apply the narrow Packet R5-4 patch, then rerun the same verifier.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            6,
            "turn-002",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\nAssertionError: expected advancing",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
    );
    assert_eq!(progress.status, ProgressStatus::Stalled);
    assert_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
}

#[test]
fn checkpoints_mark_repeated_implementation_failure_after_unrelated_edits_as_stalled() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5-4 implementation in checkpoint/progress.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            2,
            "turn-001",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Keep coding, but the edits drift away from the failing verifier scope.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-sentinel/src/operator_surface.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            6,
            "turn-002",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert!(matches!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
            | ProgressDimension::TroubleshootingFrontier
    ));
    assert!(matches!(
        progress.status,
        ProgressStatus::Stalled | ProgressStatus::Mixed
    ));
    assert_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_mark_implementation_regression_when_previously_clean_scope_breaks() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5-4 implementation in checkpoint/progress.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            3,
            "turn-001",
            r#"Exit code: 0
running 1 test
test checkpoints::captures_progress ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out"#,
        ),
        prompt_row(
            4,
            "turn-002",
            "/goal Re-run the same focused verifier after a follow-up source edit.",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            7,
            "turn-002",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert!(matches!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
            | ProgressDimension::TroubleshootingFrontier
    ));
    assert_eq!(progress.status, ProgressStatus::Regressing);
    assert_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
}

#[test]
fn checkpoints_mark_regression_across_a_same_scope_gap_after_clean_verification() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5-4 implementation in checkpoint/progress.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            3,
            "turn-001",
            r#"Exit code: 0
running 1 test
test checkpoints::captures_progress ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out"#,
        ),
        prompt_row(
            4,
            "turn-002",
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            6,
            "turn-003",
            "/goal Re-run the same focused verifier after the follow-up source edit.",
        ),
        tool_call_row(
            7,
            "turn-003",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            8,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            9,
            "turn-003",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
 --> crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert!(matches!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
            | ProgressDimension::TroubleshootingFrontier
    ));
    assert_eq!(progress.status, ProgressStatus::Regressing);
    assert_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
}

#[test]
fn checkpoints_mark_implementation_regression_when_frontier_falls_back() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5-4 implementation in checkpoint/progress.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            2,
            "turn-001",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Apply the narrow source patch, then rerun the same verifier.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            6,
            "turn-002",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
        prompt_row(
            7,
            "turn-003",
            "/goal Re-run the implementation verifier after the latest source edit.",
        ),
        tool_call_row(
            8,
            "turn-003",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            9,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            10,
            "turn-003",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert!(matches!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
            | ProgressDimension::TroubleshootingFrontier
    ));
    assert_eq!(progress.status, ProgressStatus::Regressing);
    assert_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
}

#[test]
fn checkpoints_mark_regression_across_a_same_scope_gap_after_a_later_failure_frontier() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot the failing checkpoint verifier without changing scope.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            3,
            "turn-001",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
        prompt_row(
            4,
            "turn-002",
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            6,
            "turn-003",
            "/goal Re-run the same verifier after the latest source edit.",
        ),
        tool_call_row(
            7,
            "turn-003",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            8,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            9,
            "turn-003",
            r#"Exit code: 101
error[E0425]: cannot find value `progress` in this scope
 --> crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert!(matches!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
            | ProgressDimension::TroubleshootingFrontier
    ));
    assert_eq!(progress.status, ProgressStatus::Regressing);
    assert_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
}

#[test]
fn checkpoints_reset_comparability_after_an_explicit_replan_and_working_set_pivot() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot checkpoints::captures_progress without changing scope.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            2,
            "turn-001",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Replan: pivot to the packet docs working set instead of the verifier frontier.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        prompt_row(
            5,
            "turn-003",
            "/goal Keep following the replan even if the old verifier still fails.",
        ),
        tool_call_row(
            6,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            7,
            "turn-003",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
    assert!(!progress.signals.iter().any(|signal| {
        signal.code == ProgressSignalCode::FailureSignatureRepeated
            || signal.code == ProgressSignalCode::PreviouslyCleanScopeBroken
    }));
}

#[test]
fn checkpoints_reset_comparability_after_a_material_objective_pivot_without_replan_keywords() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot checkpoints::captures_progress without changing scope.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            3,
            "turn-001",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
        prompt_row(
            4,
            "turn-002",
            "/goal Document the reviewer-facing progress-window rationale for packet R5-4.",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            6,
            "turn-003",
            "/goal Keep documenting the reviewer-facing progress-window rationale for packet R5-4.",
        ),
        tool_call_row(
            7,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            8,
            "turn-003",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
    assert!(!progress.signals.iter().any(|signal| {
        signal.code == ProgressSignalCode::FailureSignatureRepeated
            || signal.code == ProgressSignalCode::PreviouslyCleanScopeBroken
    }));
}

#[test]
fn checkpoints_mark_closeout_reopened_scope_as_mixed() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Implement the patch and then verify it.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            3,
            "turn-001",
            r#"Exit code: 0
running 27 tests

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"#,
        ),
        assistant_row(
            4,
            "turn-001",
            "The implementation is in place. Next I am gathering proof only.",
        ),
        prompt_row(
            5,
            "turn-002",
            "/goal Verify the existing patch, narrow the residual proof, and record it.",
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            7,
            "turn-002",
            r#"Exit code: 0
running 1 test
test checkpoints::captures_progress ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out"#,
        ),
        tool_call_row(
            8,
            "turn-002",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: .codex/handoffs/2026-06-10-r5-4-closeout.md
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        prompt_row(
            9,
            "turn-003",
            "/goal Re-check the closeout after a reopened source tweak.",
        ),
        tool_call_row(
            10,
            "turn-003",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            11,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            12,
            "turn-003",
            r#"Exit code: 0
running 1 test
test checkpoints::captures_progress ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out"#,
        ),
        tool_call_row(
            13,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            14,
            "turn-003",
            r#"Exit code: 0
running 27 tests

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"#,
        ),
        tool_call_row(
            15,
            "turn-003",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: .codex/handoffs/2026-06-10-r5-4-closeout.md
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::VerificationCloseoutNarrowing
    );
    assert_eq!(progress.status, ProgressStatus::Mixed);
    assert_progress_signal(progress, ProgressSignalCode::ResidualScopeReopened);
    assert_progress_signal(progress, ProgressSignalCode::VerificationClean);
}

#[test]
fn checkpoints_mark_closeout_scope_narrowing_as_advancing() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Implement the patch and then verify it.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        assistant_row(
            3,
            "turn-001",
            "The implementation is in place. Next I am gathering proof only.",
        ),
        prompt_row(
            4,
            "turn-002",
            "/goal Verify the existing patch, narrow the residual proof, and record it.",
        ),
        tool_call_row(
            5,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            6,
            "turn-002",
            "Exit code: 0\nrunning 27 tests\n\ntest result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        ),
        tool_call_row(
            7,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            8,
            "turn-002",
            "Exit code: 0\nrunning 1 test\ntest checkpoints::captures_progress ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out",
        ),
        tool_call_row(
            9,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: .codex/handoffs/2026-06-10-r5-4-closeout.md\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        progress.dimension,
        ProgressDimension::VerificationCloseoutNarrowing
    );
    assert_eq!(progress.status, ProgressStatus::Advancing);
    assert_progress_signal(progress, ProgressSignalCode::VerificationScopeNarrowed);
    assert_progress_signal(progress, ProgressSignalCode::ResidualScopeShrank);
    assert_progress_signal(progress, ProgressSignalCode::VerificationClean);
    assert!(!progress.supporting_evidence.is_empty());
}

fn analyze_custom_rows(rows: Vec<CompactionRow>) -> AnalyzeResult {
    let tool_rows = rows
        .iter()
        .filter(|row| row.kind == CompactionKind::ToolCall)
        .take(2)
        .collect::<Vec<_>>();
    let dedupe_groups = if tool_rows.len() == 2 {
        vec![DedupeGroup {
            kind: CompactionKind::ToolCall,
            canonical_text_hash_hex: "test-dedupe".to_string(),
            representative: RowRef::from_row(tool_rows[0]),
            duplicates: vec![RowRef::from_row(tool_rows[1])],
        }]
    } else {
        Vec::new()
    };
    let fixture = BundleFixture::from_rows(rows.clone(), rows, dedupe_groups);
    agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze custom rows")
}

fn prompt_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
    row(
        event_index,
        turn_id,
        CompactionKind::UserMessage,
        text,
        None,
    )
}

fn assistant_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
    row(
        event_index,
        turn_id,
        CompactionKind::AssistantMessage,
        text,
        None,
    )
}

fn developer_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
    row(
        event_index,
        turn_id,
        CompactionKind::DeveloperMessage,
        text,
        None,
    )
}

fn tool_output_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
    row(event_index, turn_id, CompactionKind::ToolOutput, text, None)
}

fn tool_call_row(event_index: usize, turn_id: &str, tool_name: &str, text: &str) -> CompactionRow {
    row(
        event_index,
        turn_id,
        CompactionKind::ToolCall,
        text,
        Some(format!(
            "{{\"call_id\":\"call-{event_index}\",\"name\":\"{tool_name}\",\"type\":\"function_call\"}}"
        )),
    )
}

fn row(
    event_index: usize,
    turn_id: &str,
    kind: CompactionKind,
    text: &str,
    dedupe_identity: Option<String>,
) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-r4-2/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-r4-2".to_string()),
        turn_id: Some(turn_id.to_string()),
        event_index,
        line_number: event_index + 1,
        row_ordinal: 0,
        timestamp: None,
        kind,
        user_message_role: matches!(kind, CompactionKind::UserMessage)
            .then_some(UserMessageRole::Prompt),
        dedupe_identity,
        text: text.to_string(),
        canonical_text: text.to_string(),
        text_hash_hex: format!("hash-{}", text.split_whitespace().collect::<String>()),
    }
}

fn assert_evidence_contains(evidence: &[agent_drift_analyzer::EvidenceRef], fragment: &str) {
    assert!(
        evidence.iter().any(|item| item.reason.contains(fragment)),
        "expected evidence containing `{fragment}`, got {:?}",
        evidence
            .iter()
            .map(|item| item.reason.as_str())
            .collect::<Vec<_>>()
    );
}

fn assert_progress_signal(
    progress: &agent_drift_analyzer::SessionProgress,
    code: ProgressSignalCode,
) {
    assert!(
        progress.signals.iter().any(|signal| signal.code == code),
        "expected progress signal {:?}, got {:?}",
        code,
        progress
            .signals
            .iter()
            .map(|signal| signal.code)
            .collect::<Vec<_>>()
    );
}

fn assert_absent_progress_signal(
    progress: &agent_drift_analyzer::SessionProgress,
    code: ProgressSignalCode,
) {
    assert!(
        progress.signals.iter().all(|signal| signal.code != code),
        "expected progress signal {:?} to be absent, got {:?}",
        code,
        progress
            .signals
            .iter()
            .map(|signal| signal.code)
            .collect::<Vec<_>>()
    );
}
