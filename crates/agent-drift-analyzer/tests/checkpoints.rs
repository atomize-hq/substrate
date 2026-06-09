#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{AnalyzeRequest, AnalyzeResult, Confidence, SessionArchetypeLabel};
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
    assert_eq!(checkpoints[0].schema_version, "v0.5");
    assert!(checkpoints[0].session_archetype.is_some());
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
    assert_eq!(checkpoints[1].schema_version, "v0.5");
    assert!(checkpoints[1].session_archetype.is_some());
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
fn checkpoints_keep_legacy_session_archetype_loads_but_fail_closed_for_v0_5() {
    let checkpoint = analyze_sample_bundle().sessions[0].checkpoints[0].clone();

    for schema_version in ["v0.2", "v0.3", "v0.4"] {
        let mut legacy_json = serde_json::to_value(&checkpoint).expect("serialize checkpoint");
        legacy_json["schema_version"] = Value::String(schema_version.to_string());
        legacy_json
            .as_object_mut()
            .expect("checkpoint object")
            .remove("session_archetype");

        let parsed: agent_drift_analyzer::Checkpoint =
            serde_json::from_value(legacy_json).expect("legacy checkpoint stays loadable");
        assert_eq!(parsed.schema_version, schema_version);
        assert!(parsed.session_archetype.is_none());
    }

    let mut missing_v0_5 = serde_json::to_value(&checkpoint).expect("serialize checkpoint");
    missing_v0_5
        .as_object_mut()
        .expect("checkpoint object")
        .remove("session_archetype");
    let err = serde_json::from_value::<agent_drift_analyzer::Checkpoint>(missing_v0_5)
        .expect_err("v0.5 checkpoint without session_archetype must fail");
    let message = err.to_string();
    assert!(message.contains("v0.5"));
    assert!(message.contains("session_archetype"));
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
