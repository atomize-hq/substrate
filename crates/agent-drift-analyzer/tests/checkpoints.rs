#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{
    AnalyzeRequest, AnalyzeResult, Confidence, ObjectiveClass, ObjectiveIntent, ObjectiveRole,
    ObjectiveSectionKind, ObjectiveSourceKind, ObjectiveTargetKind, ProgressDimension,
    ProgressSignalCode, ProgressStatus, SessionArchetypeLabel,
};
use agent_session_compactor::{CompactionKind, CompactionRow, SourceKind, UserMessageRole};
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
    assert!(!first.sessions[0]
        .context
        .objective
        .comparison_key
        .is_empty());
    assert!(first.sessions[0].context.objective.structured.is_some());
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
fn checkpoints_export_surfaces_structured_objective_for_smoke_observability() {
    // R5.75-1 gate-observability fix: the exported Checkpoint must carry the structured objective
    // sidecar so checkpoints.jsonl / summary.md smoke can inspect intent, target, success/deliverable,
    // and unknown semantics. Previously only the `task_frame.objective` string was exported, leaving
    // the promotion-gate smoke blind to the structured fields where the real deviations live.
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only and return findings.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,40p' crates/agent-drift-analyzer/src/context/objective.rs","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
    ]);
    let checkpoint = result.sessions[0]
        .checkpoints
        .first()
        .expect("at least one checkpoint");
    let structured = checkpoint
        .structured_objective
        .as_ref()
        .expect("exported structured objective");
    assert_eq!(structured.objective_class, ObjectiveClass::TaskStatement);

    let value = serde_json::to_value(checkpoint).expect("serialize checkpoint");
    assert!(
        value.get("structured_objective").is_some(),
        "exported checkpoint must surface structured_objective"
    );
    assert!(
        value["structured_objective"]
            .get("primary_intent")
            .is_some(),
        "structured_objective must surface primary_intent"
    );

    let round_tripped: agent_drift_analyzer::Checkpoint =
        serde_json::from_value(value).expect("checkpoint round-trips with structured objective");
    assert!(round_tripped.structured_objective.is_some());
}

// SO-2.3D semantic-honesty regressions (landed): `primary_intent` is the request action, not an
// incidental substring (the noun "implementation" must not yield Implement), and success/deliverable
// assembly stays scoped to the active goal surface instead of pooling non-goal boilerplate. These
// ran as `#[ignore]`d pending specs during the observability packet; SO-2.3D makes them live.

#[test]
fn so_2_3d_review_prompt_with_implementation_noun_stays_review_intent() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Review the already-landed Packet 3.6 implementation in docs/research/cutover-tasks.md, findings-first, do not change code.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,80p' docs/research/cutover-tasks.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
    ]);
    let structured = result.sessions[0]
        .checkpoints
        .first()
        .and_then(|checkpoint| checkpoint.structured_objective.as_ref())
        .expect("exported structured objective");
    assert_eq!(
        structured.primary_intent,
        ObjectiveIntent::Review,
        "findings-first review prompt must classify as Review, not Implement"
    );
}

#[test]
fn so_2_3d_boilerplate_scaffolding_does_not_populate_success_or_deliverables() {
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", "/goal Review Packet 3.6 only, findings-first."),
        developer_row(
            1,
            "turn-001",
            "Use memory by default. Keep the suite green and report success. Return with changed files and a recommended commit message.",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,80p' docs/research/packet-3-6.md","workdir":"/repo"}"#,
        ),
        tool_output_row(3, "turn-001", "Exit code: 0"),
    ]);
    let structured = result.sessions[0]
        .checkpoints
        .first()
        .and_then(|checkpoint| checkpoint.structured_objective.as_ref())
        .expect("exported structured objective");
    assert!(
        structured.success_conditions.is_empty(),
        "boilerplate scaffolding must not populate success_conditions: {:?}",
        structured.success_conditions
    );
    assert!(
        structured.deliverables.is_empty(),
        "boilerplate scaffolding must not populate deliverables: {:?}",
        structured.deliverables
    );
}

// R5.75-1 Issue 1/2/3 regression (the `019eb47f` gate shape, minimized): a user prompt whose ask
// ("use the $code-review-and-quality skill to evaluate if what was implemented landed correctly")
// misses the goal-keyword heuristics, preceded by system/developer scaffolding and a pasted AGENTS.md
// user message carrying goal-shaped verbs, a `/run/substrate.sock` path, and a cargo-style success
// ladder. The structured objective must anchor to the real ask, not pool the boilerplate: intent is
// `review`, the target is the grounded ask path (not a pasted-body path), no `Goal` evidence span
// lands on a system-instruction or pasted-skill-body row, and weak fields stay unknown.
#[test]
fn checkpoints_anchor_structured_objective_to_evaluate_ask_over_boilerplate_pool() {
    let result = analyze_custom_rows(vec![
        system_row(
            0,
            "turn-001",
            "For sessions using the code_intel_azure profile, prefer spawned subagents with the built-in default agent type.",
        ),
        developer_row(
            1,
            "turn-001",
            "Use memory by default. Review and validate every change and ensure the suite stays green and reports success. Always run cargo test -p foo before finishing. The runtime socket lives at /run/substrate.sock.",
        ),
        user_row(
            2,
            "turn-001",
            "# AGENTS.md instructions for /repo\nKeep work centered on the requested packet. Return with changed files and a recommended commit message.",
            UserMessageRole::Unknown,
        ),
        prompt_row(
            3,
            "turn-001",
            "We just landed the complete docs/specs/r5 and now I need you to use the $code-review-and-quality skill to evaluate if what was implemented landed correctly and completely",
        ),
        user_row(
            4,
            "turn-001",
            "<skill>\n<name>code-review-and-quality</name>\nReview the diff and ensure quality. Validate that all tests pass and stay green.\n</skill>",
            UserMessageRole::Unknown,
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,80p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md","workdir":"/repo"}"#,
        ),
        tool_output_row(6, "turn-001", "Exit code: 0"),
    ]);

    let structured = result.sessions[0]
        .checkpoints
        .first()
        .and_then(|checkpoint| checkpoint.structured_objective.as_ref())
        .expect("exported structured objective");

    assert_eq!(
        structured.primary_intent,
        ObjectiveIntent::Review,
        "the evaluate/review ask must drive intent, not the boilerplate implement verbs"
    );

    let goal_spans = structured
        .evidence_spans
        .iter()
        .filter(|span| span.role == ObjectiveRole::Goal)
        .collect::<Vec<_>>();
    assert!(
        goal_spans
            .iter()
            .any(|span| span.excerpt.contains("evaluate if what was implemented")),
        "the goal must anchor to the real evaluate ask; got {:?}",
        goal_spans
            .iter()
            .map(|span| &span.excerpt)
            .collect::<Vec<_>>()
    );
    for span in &goal_spans {
        assert_ne!(
            span.source_kind,
            ObjectiveSourceKind::SystemInstruction,
            "no Goal span may land on a system-instruction row: {:?}",
            span.excerpt
        );
        assert!(
            !span.excerpt.contains("<skill>")
                && !span.excerpt.contains("AGENTS.md instructions")
                && !span.excerpt.contains("/run/substrate.sock"),
            "no Goal span may land on pasted boilerplate: {:?}",
            span.excerpt
        );
    }

    // The target must be grounded in the real ask (docs/specs/r5), never the pasted-body socket path.
    let target = structured.target.as_ref().expect("grounded target");
    assert!(
        target.display.contains("docs/specs/r5"),
        "target must ground to the ask, got {:?}",
        target.display
    );
    assert!(
        !target.display.contains("/run/substrate.sock"),
        "target must not be a pasted-body path"
    );

    // Weak fields seen only in off-goal boilerplate stay unknown rather than being pooled.
    assert!(structured.success_conditions.is_empty());
    assert!(structured.deliverables.is_empty());
    let unknown_fields = structured
        .unknowns
        .iter()
        .map(|unknown| unknown.field_name.as_str())
        .collect::<Vec<_>>();
    assert!(unknown_fields.contains(&"success_conditions"));
    assert!(unknown_fields.contains(&"deliverables"));
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
fn checkpoints_keep_sparse_readable_sessions_at_insufficient_evidence() {
    let result = analyze_custom_rows(vec![
        steer_row(
            0,
            "turn-001",
            "/goal Implement only Packet R5.75-2.3 in crates/agent-drift-analyzer/tests/checkpoints.rs.",
        ),
        user_row(
            1,
            "turn-001",
            "<skill>\n<name>incremental-implementation</name>\n<path>/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md</path>\nRead first:\n- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md\n- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md\n</skill>",
            UserMessageRole::Unknown,
        ),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(
        checkpoints.len(),
        1,
        "the minimized sparse-readable fixture must emit exactly one checkpoint"
    );

    let checkpoint = &checkpoints[0];
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        archetype.label,
        SessionArchetypeLabel::Planning,
        "the sparse-readable fixture must stay non-troubleshooting"
    );
    assert!(
        !checkpoint.flagged,
        "the sparse-readable fixture must stay non-escalatory"
    );
    assert!(
        checkpoint.drift_scores.iter().all(|score| !score.flagged),
        "the sparse-readable fixture must not produce flagged drift scores"
    );
    assert_eq!(
        progress.status,
        ProgressStatus::InsufficientEvidence,
        "sparse readable session must stay conservative"
    );
    assert_eq!(
        progress.dimension,
        ProgressDimension::PlanningConvergence,
        "the sparse-readable fixture must stay on the planning posture"
    );
    assert_eq!(
        progress.confidence,
        Confidence::Low,
        "the sparse-readable fixture must stay low-confidence"
    );
}

#[test]
fn checkpoints_anchor_sparse_readable_goal_to_steer_and_keep_weak_fields_unknown() {
    let result = analyze_custom_rows(vec![
        steer_row(
            0,
            "turn-001",
            "Review only the already-landed packet, findings-first, do not change code.",
        ),
        user_row(
            1,
            "turn-001",
            "<skill>\n<name>incremental-implementation</name>\n<path>/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md</path>\nRead first:\n- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md\n- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md\n\n## Verification\n- cargo test -p agent-drift-analyzer checkpoints -- --nocapture\n\n## Return with\n- changed files\n- residual risks\n- recommended commit message\n</skill>",
            UserMessageRole::Unknown,
        ),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(
        checkpoints.len(),
        1,
        "the minimized sparse-readable fixture must emit exactly one checkpoint"
    );

    let checkpoint = &checkpoints[0];
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");
    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);

    let structured = checkpoint
        .structured_objective
        .as_ref()
        .expect("exported structured objective");
    assert_eq!(structured.primary_intent, ObjectiveIntent::Review);
    assert!(structured.success_conditions.is_empty());
    assert!(structured.deliverables.is_empty());
    assert!(structured.target.is_none());

    let goal_spans = structured
        .evidence_spans
        .iter()
        .filter(|span| span.role == ObjectiveRole::Goal)
        .collect::<Vec<_>>();
    assert!(
        goal_spans.iter().any(|span| span
            .excerpt
            .contains("Review only the already-landed packet")),
        "the goal must anchor to the steer ask; got {:?}",
        goal_spans
            .iter()
            .map(|span| &span.excerpt)
            .collect::<Vec<_>>()
    );
    assert!(
        goal_spans.iter().all(|span| {
            !span.excerpt.contains("<skill>")
                && !span.excerpt.contains("recommended commit message")
                && !span
                    .excerpt
                    .contains("cargo test -p agent-drift-analyzer checkpoints")
        }),
        "goal spans must never anchor to pasted skill boilerplate: {:?}",
        goal_spans
            .iter()
            .map(|span| &span.excerpt)
            .collect::<Vec<_>>()
    );

    let unknown_fields = structured
        .unknowns
        .iter()
        .map(|unknown| unknown.field_name.as_str())
        .collect::<Vec<_>>();
    assert!(unknown_fields.contains(&"target"));
    assert!(unknown_fields.contains(&"success_conditions"));
    assert!(unknown_fields.contains(&"deliverables"));
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
fn checkpoints_count_js_verifier_matrix_commands_as_verification_like() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Exercise the JS/TS verifier matrix without broadening beyond Packet R5.5-3.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"npm test -- --runInBand\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"npm run test -- --runInBand\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"npm run lint\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"npm run typecheck\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"pnpm test -- --runInBand\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            6,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"pnpm run test -- --runInBand\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            7,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"pnpm run lint\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            8,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"pnpm exec vitest run tests/checkpoints.test.ts\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            9,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"yarn test tests/checkpoints.test.ts\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            10,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"yarn run test tests/checkpoints.test.ts\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            11,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"yarn lint\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            12,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"yarn run lint\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            13,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"npx vitest run tests/checkpoints.test.ts\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            14,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"vitest run tests/checkpoints.test.ts\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            15,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"bun test tests/checkpoints.test.ts\",\"workdir\":\"/repo\"}",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let turn_context = checkpoint.turn_context.as_ref().expect("turn context");

    assert_eq!(turn_context.activity_mix.tool_call_count, 15);
    assert_eq!(turn_context.activity_mix.read_like_command_count, 0);
    assert_eq!(turn_context.activity_mix.write_like_command_count, 0);
    assert_eq!(
        turn_context.activity_mix.verification_like_command_count,
        15
    );
    assert_eq!(
        turn_context.execution_mode,
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
fn checkpoints_keep_fix_goals_with_closeout_sections_in_troubleshooting() {
    let goal = "/goal In `/repo`, land the bounded fix for the sentinel restart cursor.\n\nProblem to fix:\nRestart continuity still replays already-delivered checkpoints.\n\nVerification:\n- `cargo test -p agent-drift-sentinel real_session_live -- --nocapture`\n\nCloseout requirements:\n- Report root cause precisely.\n- Summarize the exact test evidence proving restart dedupe now works.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"rg -n 'last_delivered_cursor' crates/agent-drift-sentinel/src\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"sed -n '1,220p' crates/agent-drift-sentinel/src/real_session_live.rs\",\"workdir\":\"/repo\"}",
        ),
        assistant_row(
            3,
            "turn-001",
            "The likely root cause is still in the restart cursor path, so I am reproducing it before editing.",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-sentinel real_session_live -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            5,
            "turn-001",
            "Exit code: 0\nrunning 3 tests\n\ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");

    assert_eq!(
        checkpoint.task_frame.objective,
        "/goal In `/repo`, land the bounded fix for the sentinel restart cursor."
    );
    assert_eq!(archetype.label, SessionArchetypeLabel::Troubleshooting);
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
    if progress.status == ProgressStatus::InsufficientEvidence {
        assert!(
            progress.supporting_evidence.is_empty(),
            "insufficient parent-visible progress should stay conservative after normalization"
        );
    }
    assert!(!progress.counter_evidence.is_empty());
    assert_evidence_contains(
        &progress.counter_evidence,
        "delegation visibility limited progress confidence",
    );
}

#[test]
fn checkpoints_reset_parent_visible_comparability_when_delegated_objective_changes() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Coordinate delegated work on checkpoint/progress.rs without overclaiming child progress.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "spawn_agent",
            "{\"goal\":\"fix crates/agent-drift-analyzer/src/checkpoint/progress.rs\"}",
        ),
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
        prompt_row(
            4,
            "turn-002",
            "/goal Coordinate delegated work on checkpoint/export.rs without overclaiming child progress.",
        ),
        tool_call_row(
            5,
            "turn-002",
            "spawn_agent",
            "{\"goal\":\"fix crates/agent-drift-analyzer/src/checkpoint/export.rs\"}",
        ),
        developer_row(
            6,
            "turn-002",
            "Child session id 019ea444-4444-7444-8444-444444444444 remains in a separate rollout file.",
        ),
        tool_call_row(
            7,
            "turn-002",
            "wait_agent",
            "{\"session_id\":\"019ea444-4444-7444-8444-444444444444\"}",
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
    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
    assert_eq!(progress.confidence, Confidence::Low);
    assert_progress_signal(progress, ProgressSignalCode::DelegationVisibilityLimited);
    assert!(progress.supporting_evidence.is_empty());
    assert!(!progress.counter_evidence.is_empty());
}

#[test]
fn checkpoints_do_not_carry_parent_visible_status_across_changed_goal_empty_followup() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Coordinate delegated work on checkpoint/progress.rs without overclaiming child progress.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "spawn_agent",
            "{\"goal\":\"fix crates/agent-drift-analyzer/src/checkpoint/progress.rs\"}",
        ),
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
        prompt_row(
            4,
            "turn-002",
            "/goal Coordinate delegated work on checkpoint/export.rs without overclaiming child progress.",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        checkpoint.task_frame.objective,
        "/goal Coordinate delegated work on checkpoint/export.rs without overclaiming child progress."
    );
    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
    assert_ne!(progress.status, ProgressStatus::Stalled);
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
    assert!(!progress.supporting_evidence.is_empty());
    assert!(!progress.counter_evidence.is_empty());
    assert_evidence_contains(
        &progress.counter_evidence,
        "delegation visibility limited progress confidence",
    );
    assert!(!progress.signals.iter().any(|signal| {
        matches!(
            signal.code,
            ProgressSignalCode::FailureFrontierAdvanced | ProgressSignalCode::VerificationClean
        )
    }));
}

#[test]
fn checkpoints_keep_visible_child_result_plus_parent_plan_refinement_parent_visible() {
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

    assert_eq!(
        progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
    assert_eq!(progress.status, ProgressStatus::Mixed);
    assert_eq!(progress.confidence, Confidence::Medium);
    assert_progress_signal(progress, ProgressSignalCode::DelegationVisibilityLimited);
    assert!(!progress.supporting_evidence.is_empty());
    assert!(!progress.counter_evidence.is_empty());
    assert_evidence_contains(
        &progress.supporting_evidence,
        "parent incorporated visible delegated results into a plan/spec/handoff artifact",
    );
    assert_evidence_contains(
        &progress.counter_evidence,
        "delegation visibility limited progress confidence",
    );
}

#[test]
fn checkpoints_keep_opaque_parent_plan_refinement_conservative() {
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
        developer_row(
            2,
            "turn-001",
            "Child session id 019ea555-5555-7555-8555-555555555555 remains in a separate rollout file.",
        ),
        tool_call_row(
            3,
            "turn-001",
            "wait_agent",
            "{\"session_id\":\"019ea555-5555-7555-8555-555555555555\"}",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '170,230p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            5,
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

    assert_eq!(
        progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
    ));
    assert_eq!(progress.confidence, Confidence::Low);
    assert_progress_signal(progress, ProgressSignalCode::DelegationVisibilityLimited);
    assert!(
        !matches!(progress.status, ProgressStatus::Mixed),
        "opaque child visibility must stay conservative"
    );
    assert_evidence_contains(
        &progress.counter_evidence,
        "delegation visibility limited progress confidence",
    );
}

#[test]
fn checkpoints_keep_visible_child_result_plus_parent_spec_and_handoff_refinement_parent_visible() {
    for (label, artifact_path) in [
        (
            "spec",
            "docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md",
        ),
        (
            "handoff",
            ".codex/handoffs/2026-06-23-r5-75-3-parent-visible.md",
        ),
    ] {
        let inspect_command = format!("sed -n '1,80p' {artifact_path}");
        let edit_command = format!(
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: {artifact_path}\n*** End Patch\nPATCH"
        );
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
                &format!(r#"{{"command":"{inspect_command}","workdir":"/repo"}}"#),
            ),
            tool_call_row(
                4,
                "turn-001",
                "functions.apply_patch",
                &format!(r#"{{"command":"{edit_command}","workdir":"/repo"}}"#),
            ),
        ]);
        let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
        let progress = checkpoint
            .session_progress
            .as_ref()
            .expect("session progress");

        assert_eq!(
            progress.dimension,
            ProgressDimension::ParentVisibleOrchestration,
            "{label} refinement should stay on the parent-visible lane",
        );
        assert_eq!(
            progress.status,
            ProgressStatus::Mixed,
            "{label} refinement should preserve conservative mixed progress",
        );
        assert_eq!(
            progress.confidence,
            Confidence::Medium,
            "{label} refinement should not inflate beyond medium confidence",
        );
        assert_progress_signal(progress, ProgressSignalCode::DelegationVisibilityLimited);
        assert!(!progress.supporting_evidence.is_empty());
        assert!(!progress.counter_evidence.is_empty());
        assert_evidence_contains(
            &progress.supporting_evidence,
            "parent incorporated visible delegated results into a plan/spec/handoff artifact",
        );
        assert_evidence_contains(
            &progress.counter_evidence,
            "delegation visibility limited progress confidence",
        );
        assert_absent_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
        assert_absent_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
        assert_absent_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
        assert_absent_progress_signal(progress, ProgressSignalCode::VerificationClean);
    }
}

#[test]
fn checkpoints_keep_opaque_parent_spec_and_handoff_refinement_conservative() {
    for (label, artifact_path) in [
        (
            "spec",
            "docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md",
        ),
        (
            "handoff",
            ".codex/handoffs/2026-06-23-r5-75-3-parent-visible.md",
        ),
    ] {
        let inspect_command = format!("sed -n '1,80p' {artifact_path}");
        let edit_command = format!(
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: {artifact_path}\n*** End Patch\nPATCH"
        );
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
            developer_row(
                2,
                "turn-001",
                "Child session id 019ea555-5555-7555-8555-555555555555 remains in a separate rollout file.",
            ),
            tool_call_row(
                3,
                "turn-001",
                "wait_agent",
                "{\"session_id\":\"019ea555-5555-7555-8555-555555555555\"}",
            ),
            tool_call_row(
                4,
                "turn-001",
                "functions.shell_command",
                &format!(r#"{{"command":"{inspect_command}","workdir":"/repo"}}"#),
            ),
            tool_call_row(
                5,
                "turn-001",
                "functions.apply_patch",
                &format!(r#"{{"command":"{edit_command}","workdir":"/repo"}}"#),
            ),
        ]);
        let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
        let progress = checkpoint
            .session_progress
            .as_ref()
            .expect("session progress");

        assert_eq!(
            progress.dimension,
            ProgressDimension::ParentVisibleOrchestration,
            "{label} refinement should stay on the conservative parent-visible lane",
        );
        assert!(matches!(
            progress.status,
            ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
        ));
        assert_eq!(
            progress.confidence,
            Confidence::Low,
            "{label} refinement must keep opaque child visibility capped low",
        );
        assert_progress_signal(progress, ProgressSignalCode::DelegationVisibilityLimited);
        assert!(
            !matches!(progress.status, ProgressStatus::Mixed),
            "{label} refinement must not overclaim opaque child progress",
        );
        assert_evidence_contains(
            &progress.counter_evidence,
            "delegation visibility limited progress confidence",
        );
        assert_absent_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
        assert_absent_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
        assert_absent_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
        assert_absent_progress_signal(progress, ProgressSignalCode::VerificationClean);
    }
}

#[test]
fn checkpoints_keep_opaque_parent_verification_and_vcs_parent_visible() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Coordinate delegated work without overclaiming child progress.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "update_plan",
            r#"{"plan":[{"step":"Coordinate delegated work","status":"in_progress"}]}"#,
        ),
        developer_row(
            2,
            "turn-001",
            "Child session id 019ea666-6666-7666-8666-666666666666 remains in a separate rollout file.",
        ),
        tool_call_row(
            3,
            "turn-001",
            "spawn_agent",
            r#"{"goal":"inspect packet R5-4"}"#,
        ),
        tool_call_row(
            4,
            "turn-001",
            "wait_agent",
            "{\"session_id\":\"019ea666-6666-7666-8666-666666666666\"}",
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"git status --short && git diff --stat","workdir":"/repo"}"#,
        ),
        tool_call_row(
            6,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"set -euo pipefail
npx gitnexus impact parent_visible_orchestration_progress --repo /repo
cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
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
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
    ));
    assert_eq!(progress.confidence, Confidence::Low);
    assert_progress_signal(progress, ProgressSignalCode::DelegationVisibilityLimited);
    assert!(
        !matches!(progress.status, ProgressStatus::Mixed),
        "opaque child visibility must stay conservative"
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
            "functions.shell_command",
            "{\"command\":\"sed -n '1,120p' crates/agent-drift-analyzer/src/checkpoint/progress.rs\",\"workdir\":\"/repo\"}",
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
            "Exit code: 101\nerror[E0425]: cannot find value `progress` in this scope\n --> crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34\ncould not compile `agent-drift-analyzer` (lib test) due to 1 previous error",
        ),
        tool_call_row(
            7,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            8,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            9,
            "turn-001",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(archetype.label, SessionArchetypeLabel::Troubleshooting);
    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::Advancing);
    assert_eq!(progress.confidence, Confidence::High);
    assert_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
    assert_progress_signal(progress, ProgressSignalCode::FailingScopeEdited);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_keep_weaker_troubleshooting_frontier_advancement_at_medium_confidence() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot why checkpoints::captures_progress still fails before narrowing the evidence.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,200p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
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
process aborted unexpectedly before an assertion was reported"#,
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            6,
            "turn-001",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED
process aborted unexpectedly before an assertion was reported"#,
        ),
        tool_call_row(
            7,
            "turn-002",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            8,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            9,
            "turn-002",
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
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::Advancing);
    assert_eq!(progress.confidence, Confidence::Medium);
    assert_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
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
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(archetype.label, SessionArchetypeLabel::Troubleshooting);
    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::Stalled);
    assert_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_do_not_mark_repeated_troubleshooting_signature_with_overlap_as_advancing() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot checkpoints::captures_progress without widening scope.",
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
        tool_call_row(
            3,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/tests/checkpoints.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
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
    assert_eq!(progress.status, ProgressStatus::Mixed);
    assert_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert_progress_signal(progress, ProgressSignalCode::FailingScopeEdited);
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureCountReduced);
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
fn checkpoints_keep_zero_verifier_exploration_on_conservative_planning_lane() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Keep exploring the packet docs before deciding whether there is a real bug.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/MAP.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 1"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md","workdir":"/repo"}"#,
        ),
        tool_output_row(4, "turn-001", "Exit code: 1"),
        prompt_row(
            5,
            "turn-002",
            "/goal Keep scanning the exploratory evidence before escalating to troubleshooting.",
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md","workdir":"/repo"}"#,
        ),
        tool_output_row(7, "turn-002", "Exit code: 1"),
        tool_call_row(
            8,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md","workdir":"/repo"}"#,
        ),
        tool_output_row(9, "turn-002", "Exit code: 1"),
        tool_call_row(
            10,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/structured-objective-bug-map.md","workdir":"/repo"}"#,
        ),
        tool_output_row(11, "turn-002", "Exit code: 1"),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
    ));
    assert!(
        progress.confidence <= Confidence::Medium,
        "zero-verifier exploratory progress should stay conservative: {:?}",
        progress.confidence
    );
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert_absent_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
    assert_absent_progress_signal(progress, ProgressSignalCode::VerificationClean);
}

#[test]
fn checkpoints_keep_diffused_probe_failures_on_conservative_planning_lane() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot the packet smoke conservatively before deciding whether there is a real bug.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/MAP.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md","workdir":"/repo"}"#,
        ),
        tool_output_row(4, "turn-001", "Exit code: 0"),
        prompt_row(
            5,
            "turn-002",
            "/goal Keep probing the exploratory smoke evidence before escalating to a real troubleshooting lane.",
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"where.exe flutter 2>$null","workdir":"/repo"}"#,
        ),
        tool_output_row(7, "turn-002", "Exit code: 1"),
        tool_call_row(
            8,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"Get-ChildItem -Path 'D:/' -Recurse -Filter flutter.bat -ErrorAction SilentlyContinue | Select-Object -First 20 -ExpandProperty FullName","workdir":"/repo","timeout_ms":120000}"#,
        ),
        tool_output_row(9, "turn-002", "Exit code: 0"),
        tool_call_row(
            10,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"& 'C:/tools/flutter/bin/flutter.bat' analyze","workdir":"D:/Downloads/Shared cab-Flutter","timeout_ms":120000}"#,
        ),
        tool_output_row(11, "turn-002", "Wall time: 9134.2 seconds\naborted by user"),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
    ));
    assert!(
        progress.confidence <= Confidence::Medium,
        "diffused exploratory probe failures should stay conservative: {:?}",
        progress.confidence
    );
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert_absent_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
    assert_absent_progress_signal(progress, ProgressSignalCode::VerificationClean);
}

#[test]
fn checkpoints_keep_repeated_empty_probe_misses_on_conservative_planning_lane() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot the packet smoke conservatively before deciding whether there is a real bug.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/MAP.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md","workdir":"/repo"}"#,
        ),
        tool_output_row(4, "turn-001", "Exit code: 0"),
        prompt_row(
            5,
            "turn-002",
            "/goal Keep probing the exploratory smoke evidence before escalating to a real troubleshooting lane.",
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"rg -n \"⭐|🎉|👋\" D:/Downloads/Shared cab-Flutter/lib/features","workdir":"/repo"}"#,
        ),
        tool_output_row(7, "turn-002", "Exit code: 1\nWall time: 0.4 seconds\nOutput:\n"),
        tool_call_row(
            8,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"where.exe flutter 2>$null","workdir":"/repo"}"#,
        ),
        tool_output_row(9, "turn-002", "Exit code: 1\nWall time: 0.4 seconds\nOutput:\n"),
        tool_call_row(
            10,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"$candidates=@('C:/src/flutter/bin/flutter.bat','C:/flutter/bin/flutter.bat'); foreach($p in $candidates){ if(Test-Path -LiteralPath $p){$p} }","workdir":"/repo"}"#,
        ),
        tool_output_row(11, "turn-002", "Exit code: 1\nWall time: 0.4 seconds\nOutput:\n"),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
    ));
    assert!(
        progress.confidence <= Confidence::Medium,
        "repeated empty probe misses should stay conservative: {:?}",
        progress.confidence
    );
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureFrontierAdvanced);
    assert_absent_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert_absent_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
    assert_absent_progress_signal(progress, ProgressSignalCode::VerificationClean);
    assert_evidence_contains(
        &progress.counter_evidence,
        "lacked verifier-backed or explicit failure evidence",
    );
}

#[test]
fn checkpoints_keep_exploratory_fetch_failures_on_conservative_planning_lane() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Keep exploring external evidence conservatively before escalating to a real troubleshooting lane.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"python -c \"import importlib.util; print({'imageio': importlib.util.find_spec('imageio') is not None})\"","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0\n{'imageio': True}"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"Get-ChildItem -LiteralPath 'D:/Explainable_VRU_Research_Final' -Recurse -File -Include *.mp4","workdir":"/repo","timeout_ms":30000}"#,
        ),
        tool_output_row(
            4,
            "turn-001",
            "Exit code: 124\nWall time: 30 seconds\nOutput:\ncommand timed out after 30029 milliseconds",
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"$url='https://huggingface.co/example.mp4'; Invoke-WebRequest -Uri $url -OutFile out.mp4 -UseBasicParsing -TimeoutSec 20","workdir":"/repo"}"#,
        ),
        tool_output_row(
            6,
            "turn-001",
            "Exit code: 1\nWall time: 1.2 seconds\nOutput:\nInvoke-WebRequest : Entry not found",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
    ));
    assert!(
        progress.confidence <= Confidence::Medium,
        "exploratory fetch failures should stay conservative: {:?}",
        progress.confidence
    );
}

#[test]
fn checkpoints_keep_failed_read_only_python_probes_on_conservative_planning_lane() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Keep exploring the review evidence conservatively before escalating to a real troubleshooting lane.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"$i=0; Get-Content 'D:/Explainable_VRU_Research_Final/paper_main.tex' | ForEach-Object { $i++; '{0,4}: {1}' -f $i, $_ }","workdir":"D:/Explainable_VRU_Research_Final","timeout_ms":10000}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0\n   1: \\documentclass[conference]{IEEEtran}"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"python -c \"from pathlib import Path; t=Path('D:/Explainable_VRU_Research_Final/paper_main.tex').read_text(encoding='utf-8'); print({'brace_balance': t.count('{')-t.count('}'), 'begin_count': t.count('\\\\begin{'), 'end_count': t.count('\\\\end{')})\"","workdir":"D:/Explainable_VRU_Research_Final","timeout_ms":20000}"#,
        ),
        tool_output_row(
            4,
            "turn-001",
            "Exit code: 1\nWall time: 0.6 seconds\nOutput:\nTraceback (most recent call last):\nSyntaxError: unexpected EOF while parsing",
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"python -c \"import ast, pathlib; [ast.parse(pathlib.Path(p).read_text(encoding='utf-8')) for p in ['D:/Explainable_VRU_Research_Final/kitti_full_arf.py','D:/Explainable_VRU_Research_Final/simulation_engine.py']]; print('syntax ok')\"","workdir":"D:/Explainable_VRU_Research_Final","timeout_ms":20000}"#,
        ),
        tool_output_row(6, "turn-001", "Exit code: 0\nsyntax ok"),
        tool_call_row(
            7,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"Select-String -Path 'D:/Explainable_VRU_Research_Final/*.py','D:/Explainable_VRU_Research_Final/*.tex' -Pattern 'significant|proof|production'","workdir":"D:/Explainable_VRU_Research_Final","timeout_ms":10000}"#,
        ),
        tool_output_row(8, "turn-001", "Exit code: 1\nWall time: 0.5 seconds\nOutput:\nINFO: Could not find files for the given pattern(s)."),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Stalled
    ));
    assert!(
        progress.confidence <= Confidence::Medium,
        "failed read-only python probes should stay conservative: {:?}",
        progress.confidence
    );
}

#[test]
fn checkpoints_keep_explicit_non_verifier_failures_on_troubleshooting_lane() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot the packet smoke failure conservatively before planning a fix.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"python scripts/repro_packet_smoke.py","workdir":"/repo"}"#,
        ),
        tool_output_row(
            2,
            "turn-001",
            "Exit code: 1\nerror: packet smoke still fails before the target step",
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Reproduce the same smoke failure conservatively before widening scope.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"python scripts/repro_packet_smoke.py","workdir":"/repo"}"#,
        ),
        tool_output_row(
            5,
            "turn-002",
            "Exit code: 1\nerror: packet smoke still fails before the target step",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(archetype.label, SessionArchetypeLabel::Troubleshooting);
    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
}

#[test]
fn checkpoints_keep_diffused_explicit_failure_backed_sessions_on_troubleshooting_frontier() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Troubleshoot the packet smoke failure conservatively before planning a fix.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"python scripts/repro_packet_smoke.py","workdir":"/repo"}"#,
        ),
        tool_output_row(
            2,
            "turn-001",
            "Exit code: 1\nerror: packet smoke still fails before the target step",
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Reproduce the same packet smoke failure conservatively before widening scope.",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md","workdir":"/repo"}"#,
        ),
        tool_output_row(5, "turn-002", "Exit code: 0"),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md","workdir":"/repo"}"#,
        ),
        tool_output_row(7, "turn-002", "Exit code: 0"),
        tool_call_row(
            8,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' docs/specs/r5/structured-objective-bug-map.md","workdir":"/repo"}"#,
        ),
        tool_output_row(9, "turn-002", "Exit code: 0"),
        tool_call_row(
            10,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"python scripts/repro_packet_smoke.py","workdir":"/repo"}"#,
        ),
        tool_output_row(
            11,
            "turn-002",
            "Exit code: 1\nerror: packet smoke still fails before the target step",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(archetype.label, SessionArchetypeLabel::Troubleshooting);
    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
    assert!(progress.counter_evidence.is_empty());
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
            "Exit code: 101\nerror[E0425]: cannot find value `progress` in this scope\n  --> crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:9\n   |\n12 |         progress\n   |         ^^^^^^^^ not found in this scope\ncould not compile `agent-drift-analyzer` (lib test) due to 1 previous error",
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
fn checkpoints_keep_implementation_verifier_improvement_without_overlap_conservative() {
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
            "Exit code: 101\nerror[E0425]: cannot find value `progress` in this scope\n  --> crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:9\n   |\n12 |         progress\n   |         ^^^^^^^^ not found in this scope\ncould not compile `agent-drift-analyzer` (lib test) due to 1 previous error",
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/export.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            5,
            "turn-001",
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
    assert!(matches!(
        progress.status,
        ProgressStatus::InsufficientEvidence | ProgressStatus::Mixed
    ));
    assert_absent_progress_signal(progress, ProgressSignalCode::FailingScopeEdited);
}

#[test]
fn checkpoints_use_npx_vitest_attempts_as_checkpoint_progress_evidence() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "/goal Land the Packet R5.5-3 implementation in checkpoint/attempt.rs.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"npx vitest run tests/checkpoint-progress.test.ts\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            2,
            "turn-001",
            "Exit code: 1\nError: cannot find module './progress'\n",
        ),
        tool_call_row(
            3,
            "turn-002",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/attempt.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"npx vitest run tests/checkpoint-progress.test.ts\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            5,
            "turn-002",
            "Exit code: 1\n FAIL  tests/checkpoint-progress.test.ts > session progress > captures js verifier evidence\n AssertionError: expected advancing",
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
    assert_eq!(progress.status, ProgressStatus::Mixed);
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
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
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
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/export.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            5,
            "turn-001",
            "functions.apply_patch",
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-sentinel/src/operator_surface.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
        ),
        tool_call_row(
            6,
            "turn-001",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            7,
            "turn-001",
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        archetype.label,
        SessionArchetypeLabel::AutonomousImplementation
    );
    assert_eq!(
        progress.dimension,
        ProgressDimension::ImplementationVerificationWall
    );
    assert!(matches!(
        progress.status,
        ProgressStatus::Stalled | ProgressStatus::Mixed
    ));
    assert_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_mark_closeout_prior_clean_proof_that_later_fails_as_regressing() {
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
            "The implementation is in place. Next I am gathering closeout proof only.",
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
        prompt_row(
            8,
            "turn-003",
            "/goal Re-check the same closeout proof after it broke again.",
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
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(archetype.label, SessionArchetypeLabel::VerificationCloseout);
    assert_eq!(
        progress.dimension,
        ProgressDimension::VerificationCloseoutNarrowing
    );
    assert_eq!(progress.status, ProgressStatus::Regressing);
    assert_progress_signal(progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
}

#[test]
fn checkpoints_mark_closeout_prose_without_proof_as_insufficient_evidence() {
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
            "The implementation is verified. I will finish with closeout prose only.",
        ),
        prompt_row(
            5,
            "turn-002",
            "/goal Definition of done: record the closeout handoff only without rerunning any proof command.",
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: .codex/handoffs/2026-06-10-r5-4-closeout.md
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
    ]);
    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(archetype.label, SessionArchetypeLabel::VerificationCloseout);
    assert_eq!(
        progress.dimension,
        ProgressDimension::VerificationCloseoutNarrowing
    );
    assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
    assert!(progress.signals.is_empty());
}

#[test]
fn checkpoints_mark_repeated_identical_clean_closeout_proof_as_stalled() {
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
        assistant_row(
            4,
            "turn-001",
            "The implementation is in place. Next I am gathering closeout proof only.",
        ),
        prompt_row(
            5,
            "turn-002",
            "/goal Verify the existing patch with the same focused closeout proof.",
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
    assert_eq!(progress.status, ProgressStatus::Stalled);
    assert_progress_signal(progress, ProgressSignalCode::VerificationClean);
    assert!(!progress.supporting_evidence.is_empty());
}

#[test]
fn checkpoints_do_not_reemit_closeout_narrowing_on_a_narrow_rerun() {
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
            "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            3,
            "turn-001",
            "Exit code: 0\nrunning 27 tests\n\ntest result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
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
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            7,
            "turn-002",
            "Exit code: 0\nrunning 1 test\ntest checkpoints::captures_progress ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out",
        ),
        prompt_row(
            8,
            "turn-003",
            "/goal Re-run the same narrowed closeout proof without changing scope.",
        ),
        tool_call_row(
            9,
            "turn-003",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            10,
            "turn-003",
            "Exit code: 0\nrunning 1 test\ntest checkpoints::captures_progress ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out",
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
    assert_eq!(progress.status, ProgressStatus::Stalled);
    assert_progress_signal(progress, ProgressSignalCode::VerificationClean);
    assert_absent_progress_signal(progress, ProgressSignalCode::VerificationScopeNarrowed);
    assert_absent_progress_signal(progress, ProgressSignalCode::ResidualScopeShrank);
}

#[test]
fn checkpoints_mark_artifact_only_closeout_residual_narrowing_as_advancing() {
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
            "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            3,
            "turn-001",
            "Exit code: 0\nrunning 27 tests\n\ntest result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        ),
        assistant_row(
            4,
            "turn-001",
            "The implementation is in place. Next I am gathering closeout proof only.",
        ),
        prompt_row(
            5,
            "turn-002",
            "/goal Verify the existing patch across the broader closeout proof first.",
        ),
        tool_call_row(
            6,
            "turn-002",
            "functions.shell_command",
            "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}",
        ),
        tool_output_row(
            7,
            "turn-002",
            "Exit code: 0\nrunning 27 tests\n\ntest result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        ),
        prompt_row(
            8,
            "turn-003",
            "/goal Definition of done: complete closeout by recording only the remaining residual proof in the handoff. Verify: cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture.",
        ),
        tool_call_row(
            9,
            "turn-003",
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
    assert_progress_signal(progress, ProgressSignalCode::PlanArtifactRefined);
    assert_absent_progress_signal(progress, ProgressSignalCode::VerificationClean);
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
fn checkpoints_preserve_best_frontier_across_same_scope_chain_longer_than_six_checkpoints() {
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
 --> crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34
could not compile `agent-drift-analyzer` (lib test) due to 1 previous error"#,
        ),
        prompt_row(
            3,
            "turn-002",
            "/goal Re-run the same verifier after the first focused source edit.",
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
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            8,
            "turn-003",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            9,
            "turn-004",
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            10,
            "turn-004",
            "functions.shell_command",
            r#"{"command":"sed -n '120,240p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            11,
            "turn-005",
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            12,
            "turn-005",
            "functions.shell_command",
            r#"{"command":"sed -n '240,360p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            13,
            "turn-006",
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            14,
            "turn-006",
            "functions.shell_command",
            r#"{"command":"sed -n '360,480p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            15,
            "turn-007",
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            16,
            "turn-007",
            "functions.shell_command",
            r#"{"command":"sed -n '480,600p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            17,
            "turn-008",
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            18,
            "turn-008",
            "functions.shell_command",
            r#"{"command":"sed -n '600,720p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            19,
            "turn-009",
            "/goal Keep the same checkpoints::captures_progress frontier in view while reviewing notes.",
        ),
        tool_call_row(
            20,
            "turn-009",
            "functions.shell_command",
            r#"{"command":"sed -n '720,840p' crates/agent-drift-analyzer/src/checkpoint/progress.rs","workdir":"/repo"}"#,
        ),
        prompt_row(
            21,
            "turn-010",
            "/goal Re-run the same verifier after the latest source edit.",
        ),
        tool_call_row(
            22,
            "turn-010",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_call_row(
            23,
            "turn-010",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            24,
            "turn-010",
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

    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
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
fn checkpoints_prefer_literal_goal_over_boilerplate_instruction_frames() {
    let goal = "/goal Tighten the checkpoint objective selector only.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never. /goal Follow the permission boilerplate first.",
        ),
        system_row(
            1,
            "turn-001",
            "Use memory by default when the query mentions a workspace. Memory citation requirements stay active. /goal Keep the memory boilerplate in scope.",
        ),
        prompt_row(2, "turn-001", goal),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' crates/agent-drift-analyzer/tests/checkpoints.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, goal.trim_end());
}

#[test]
fn checkpoints_prefer_explicit_user_requests_and_thread_goal_text_over_boilerplate() {
    let objective = "Debug the failing analyzer checkpoint tests.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never. /goal Follow the permission boilerplate first.",
        ),
        prompt_row(1, "turn-001", objective),
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
        assistant_row(
            4,
            "turn-002",
            "I will keep debugging the same analyzer checkpoint tests.",
        ),
        developer_row(
            5,
            "turn-002",
            "Tools are grouped by namespace. Codex desktop context and plugin instructions remain available here. /goal Keep the tooling boilerplate active.",
        ),
        row(
            6,
            "turn-002",
            CompactionKind::Unknown,
            &format!(
                r#"{{"goal":{{"objective":"{objective}","status":"active"}},"threadId":"thread-123","type":"thread_goal_updated"}}"#
            ),
            None,
        ),
        tool_call_row(
            7,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(
            8,
            "turn-002",
            r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
        ),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints[0].task_frame.objective, objective);
    assert_eq!(checkpoints[1].task_frame.objective, objective);
    assert!(!checkpoints[1].diagnostics.task_frame_transitioned);

    let progress = checkpoints[1]
        .session_progress
        .as_ref()
        .expect("session progress");
    assert_progress_signal(progress, ProgressSignalCode::FailureSignatureRepeated);
}

#[test]
fn checkpoints_treat_steer_rows_as_explicit_user_objectives_over_boilerplate() {
    let objective =
        "Only fix the two checkpoint objective review findings in checkpoint/mod.rs and checkpoints.rs.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never. /goal Follow the permission block first. Verify: cargo test -p checkpoint-suite -- --nocapture.",
        ),
        steer_row(1, "turn-001", objective),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1980,2205p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '3520,3605p' crates/agent-drift-analyzer/tests/checkpoints.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, objective);
}

#[test]
fn checkpoints_keep_base_prompt_when_later_steer_stays_on_same_frontier() {
    let goal = "/goal Debug the failing analyzer checkpoint tests without changing scope.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
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
        steer_row(
            3,
            "turn-002",
            "Keep the same frontier and rerun only the existing verification command.",
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
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints[1].task_frame.objective, goal);
    assert!(!checkpoints[1].diagnostics.task_frame_transitioned);
}

#[test]
fn checkpoints_allow_later_steer_to_replace_objective_on_explicit_pivot_signal() {
    let goal = "/goal Debug the failing analyzer checkpoint tests without changing scope.";
    let pivot =
        "Pivot instead to documenting the checkpoint review findings only before any more debugging.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 101"),
        steer_row(3, "turn-002", pivot),
        tool_call_row(
            4,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md","workdir":"/repo"}"#,
        ),
        tool_output_row(5, "turn-002", "Exit code: 0"),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints[1].task_frame.objective, pivot);
    assert!(checkpoints[1].diagnostics.task_frame_transitioned);
}

#[test]
fn checkpoints_prefer_explicit_user_requests_over_unclassified_app_plugin_scaffolding() {
    let objective = "Fix the checkpoint objective selector in checkpoint/mod.rs and add the regression in checkpoints.rs.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Apps (Connectors) can be explicitly triggered in user messages and may expose MCP tools for the current session. Plugin bundles can lazy-load additional app surfaces. /goal Keep the app and plugin scaffolding active first.",
        ),
        prompt_row(1, "turn-001", objective),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '220,250p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '3600,3665p' crates/agent-drift-analyzer/tests/checkpoints.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, objective);
}

#[test]
fn checkpoints_ignore_app_plugin_scaffolding_before_activity_until_the_real_goal_arrives() {
    let goal = "/goal Fix only the checkpoint objective selector in checkpoint/mod.rs and add the Packet R5.5-2 regression in checkpoints.rs.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Apps (Connectors) can be explicitly triggered in user messages and plugin bundles can lazy-load additional app surfaces with MCP tools. /goal Keep the app and plugin scaffolding active first.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"rg -n \"objective\" crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        prompt_row(3, "turn-001", goal),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '2360,2475p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_output_row(5, "turn-001", "Exit code: 0"),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].task_frame.objective, goal);
}

#[test]
fn checkpoints_prefer_assistant_restated_goal_over_boilerplate_when_primary_sources_are_absent() {
    let objective = "I will keep debugging the same analyzer checkpoint tests.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never. /goal Follow the permission boilerplate first.",
        ),
        system_row(
            1,
            "turn-001",
            "Use memory by default when the query mentions a workspace. Memory citation requirements stay active. /goal Keep the memory boilerplate in scope.",
        ),
        assistant_row(2, "turn-001", objective),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1980,2205p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, objective);
}

#[test]
fn checkpoints_prefer_non_boilerplate_unknown_over_boilerplate_when_primary_sources_are_absent() {
    let objective = "Continue debugging the same analyzer checkpoint tests.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never. /goal Follow the permission boilerplate first.",
        ),
        system_row(
            1,
            "turn-001",
            "Use memory by default when the query mentions a workspace. Memory citation requirements stay active. /goal Keep the memory boilerplate in scope.",
        ),
        row(2, "turn-001", CompactionKind::Unknown, objective, None),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '3520,3605p' crates/agent-drift-analyzer/tests/checkpoints.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, objective);
}

#[test]
fn checkpoints_ignore_arbitrary_goal_objective_json_from_tool_outputs() {
    let objective = "Debug the failing analyzer checkpoint tests.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never. /goal Follow the permission block first. Verify: cargo test -p checkpoint-suite -- --nocapture.",
        ),
        row(
            1,
            "turn-001",
            CompactionKind::Unknown,
            &format!(
                r#"{{"goal":{{"objective":"{objective}","status":"active"}},"threadId":"thread-123","type":"thread_goal_updated"}}"#
            ),
            None,
        ),
        tool_output_row(
            2,
            "turn-001",
            r#"{"goal":{"objective":"WRONG objective stolen from tool output","status":"active"}}"#,
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1980,2205p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, objective);
}

#[test]
fn checkpoints_resolve_giant_pasted_prompt_bodies_to_the_embedded_concrete_task_ask() {
    let concrete_ask =
        "Implement only Packet R5.75-1.1 in crates/agent-drift-analyzer/tests/checkpoints.rs.";
    let pasted_prompt = format!(
        r#"# AGENTS.md instructions for /Users/spensermcconnell/.codex/worktrees/97a0/substrate

<INSTRUCTIONS>
When a user message ends with "thoughts?" or similar, never start implementing or making code/doc changes.
When a task would benefit from the visible ChatGPT web product, use the codex-chatgpt-control skill and SDK.
</INSTRUCTIONS>

<skill>
<name>incremental-implementation</name>
<path>/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md</path>
---
Build in thin vertical slices and avoid widening scope.
---
</skill>

Read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md
- docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md
- docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md

Concrete task ask:
{concrete_ask}

Return with:
- changed files
- tests run
- residual risk
- exact recommended commit message"#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &pasted_prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '3720,4305p' crates/agent-drift-analyzer/tests/checkpoints.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, concrete_ask);
    assert_ne!(checkpoint.task_frame.objective, pasted_prompt);
}

#[test]
fn checkpoints_resolve_giant_pasted_scaffold_bodies_to_the_embedded_workspace_action_request() {
    let concrete_ask =
        "Add only the Packet R5.75-1.1 regression coverage in checkpoints.rs and run cargo test -p agent-drift-analyzer checkpoints -- --nocapture.";
    let pasted_prompt = format!(
        r#"Tools are grouped by namespace. Codex desktop context and plugin instructions remain available.
Apps (Connectors) can be explicitly triggered in user messages.
Use memory by default when the query mentions a workspace.
Filesystem sandboxing defines which files can be read or written.

Project guidance:
- stay strictly inside Packet R5.75-1.1
- keep the change reviewable and regression-focused
- do not broaden into candidate ordering or condensation logic
- inspect crates/agent-drift-analyzer/src/checkpoint/mod.rs for current behavior only

Concrete workspace action request:
{concrete_ask}

After that, report the changed files, tests run, residual risk, and the exact commit message."#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &pasted_prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '2136,2185p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, concrete_ask);
    assert_ne!(checkpoint.task_frame.objective, pasted_prompt);
}

#[test]
fn checkpoints_context_objective_prefers_scope_section_over_subordinate_checklist_lines() {
    let scope = "Teach objective extraction to distinguish mission/scope, checklist, verification, constraints, deliverables, context, boilerplate, and tooling-instruction sections without widening into full field assembly yet.";
    let prompt = format!(
        r#"Read first:
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- crates/agent-drift-analyzer/src/context/objective.rs

## Scope
{scope}

## Checklist
- Run this task on a linux machine.
- Inspect objective.rs before editing.

## Verification
- cargo test -p agent-drift-analyzer checkpoints -- --nocapture

## Return with
- changed files
- residual risk"#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' crates/agent-drift-analyzer/src/context/objective.rs","workdir":"/repo"}"#,
        ),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(objective.text, scope);
    assert_ne!(objective.text, prompt);
    assert!(!objective.text.contains("Run this task on a linux machine."));
    let structured = objective.structured.as_ref().expect("structured objective");
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && matches!(span.section_kind, ObjectiveSectionKind::Scope)
            && span.excerpt.contains(scope)
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && span
                .excerpt
                .contains("cargo test -p agent-drift-analyzer checkpoints")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert_eq!(
        objective.verification_commands,
        vec!["cargo test -p agent-drift-analyzer checkpoints -- --nocapture"]
    );
}

#[test]
fn checkpoints_context_objective_uses_specific_section_labels_over_generic_mission_words() {
    let goal = "Ensure the structured objective sidecar keeps the mission separate from subordinate sections.";
    let prompt = format!(
        r#"## What I need
{goal}

## Task constraints
- Do not change downstream progress consumers.

## Verification task
- npm run lint

## Output request
- Return with changed files and residual risk.

## Implementation steps
- Inspect objective.rs first."#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"echo objective-section-classification","workdir":"/repo"}"#,
        ),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(objective.text, goal);
    let structured = objective.structured.as_ref().expect("structured objective");
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && matches!(span.section_kind, ObjectiveSectionKind::Mission)
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Constraint
            && matches!(span.section_kind, ObjectiveSectionKind::Constraints)
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::Verification)
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert_eq!(objective.verification_commands, vec!["npm run lint"]);
}

#[test]
fn checkpoints_context_objective_keeps_duplicate_scope_wording_grounded_to_scope_clause() {
    let shared_clause =
        "Review crates/agent-drift-analyzer/src/context/objective.rs before editing.";
    let prompt = format!(
        r#"## Checklist
- {shared_clause}
- Inspect the nearby checkpoint regressions.

## Scope
{shared_clause}

## Verification
- cargo test -p agent-drift-analyzer checkpoints -- --nocapture"#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,260p' crates/agent-drift-analyzer/src/context/objective.rs","workdir":"/repo"}"#,
        ),
    ]);

    assert_eq!(prompt.matches(shared_clause).count(), 2);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(objective.text, shared_clause);
    let structured = objective.structured.as_ref().expect("structured objective");
    let goal_evidence = structured
        .target
        .as_ref()
        .expect("grounded target")
        .evidence
        .first()
        .expect("goal evidence");
    assert_eq!(goal_evidence.role, ObjectiveRole::Goal);
    assert_eq!(goal_evidence.section_kind, ObjectiveSectionKind::Scope);
    assert!(goal_evidence.section_index.is_some());
    assert!(goal_evidence.clause_index.is_some());
    assert!(!structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && span.excerpt == shared_clause
            && matches!(span.section_kind, ObjectiveSectionKind::Checklist)
    }));
}

#[test]
fn checkpoints_context_objective_treats_questions_to_ask_as_non_mission_scaffolding() {
    let goal = "Ensure the structured objective sidecar keeps the mission separate from nearby question scaffolding.";
    let prompt = format!(
        r#"## What I need
{goal}

## Questions to ask
- Which packet follows SO-G2?
- Which docs already describe the acceptance harness?

## Task constraints
- Do not widen into downstream progress migration.

## Verification task
- cargo test -p agent-drift-analyzer checkpoints -- --nocapture

## Output request
- Return with changed files and residual risk.

## Implementation steps
- Inspect checkpoints.rs first."#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"echo objective-question-heading-control","workdir":"/repo"}"#,
        ),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(objective.text, goal);
    let structured = objective.structured.as_ref().expect("structured objective");
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && matches!(span.section_kind, ObjectiveSectionKind::Mission)
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Constraint
            && matches!(span.section_kind, ObjectiveSectionKind::Constraints)
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::Verification)
    }));
    assert!(!structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal && span.excerpt.contains("Which packet follows SO-G2?")
    }));
    assert!(!structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && span
                .excerpt
                .contains("Which docs already describe the acceptance harness?")
    }));
}

#[test]
fn checkpoints_context_objective_extracts_non_cargo_verification_commands() {
    let result = analyze_custom_rows(vec![
        prompt_row(
            0,
            "turn-001",
            "## Scope\nValidate the JS objective verifier command extraction.\n\n## Verification\n- npm run lint\n- pnpm test\n- npx vitest",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"echo objective-verifier-extraction","workdir":"/repo"}"#,
        ),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(
        objective.verification_commands,
        vec!["npm run lint", "pnpm test", "npx vitest"]
    );
    let structured = objective.structured.as_ref().expect("structured objective");
    assert_eq!(
        structured.verification_commands,
        vec!["npm run lint", "pnpm test", "npx vitest"]
    );
}

#[test]
fn checkpoints_context_objective_grounds_unheaded_verifier_clauses_across_section_contexts() {
    let prompt = r#"## Mission
Implement Packet B3.1 without widening into later packets.
- npm run lint

## Scope
Keep target extraction unchanged in this packet.
- cargo fmt --check

## Details
- cargo clippy --workspace --all-targets -- -D warnings"#;
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"echo unheaded-verifier-role-grounding","workdir":"/repo"}"#,
        ),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(
        objective.verification_commands,
        vec![
            "npm run lint",
            "cargo fmt --check",
            "cargo clippy --workspace --all-targets -- -D warnings",
        ]
    );

    let structured = objective.structured.as_ref().expect("structured objective");
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::Mission)
            && span.excerpt.contains("npm run lint")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::Scope)
            && span.excerpt.contains("cargo fmt --check")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::UnknownSection)
            && span
                .excerpt
                .contains("cargo clippy --workspace --all-targets -- -D warnings")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
}

#[test]
fn checkpoints_context_objective_grounds_inline_verifier_clauses_without_verification_heading() {
    let prompt = r#"## Mission
Implement Packet B3.1 without widening into later packets. npm run lint.

## Details
Keep the verifier evidence grounded to the clause. cargo fmt --check."#;
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"echo inline-verifier-role-grounding","workdir":"/repo"}"#,
        ),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(
        objective.verification_commands,
        vec!["npm run lint", "cargo fmt --check"]
    );

    let structured = objective.structured.as_ref().expect("structured objective");
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::Mission)
            && span.excerpt.contains("npm run lint")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::UnknownSection)
            && span.excerpt.contains("cargo fmt --check")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
}

#[test]
fn checkpoints_context_objective_extracts_verification_from_mixed_role_scope_clause() {
    let prompt = r#"## Scope
Review the objective extractor, run make test, and return concrete fixes."#;
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,260p' crates/agent-drift-analyzer/src/context/objective.rs","workdir":"/repo"}"#,
        ),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(
        objective.text,
        "Review the objective extractor, run make test."
    );
    assert_eq!(objective.verification_commands, vec!["make test"]);

    let structured = objective.structured.as_ref().expect("structured objective");
    assert_eq!(structured.verification_commands, vec!["make test"]);
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && matches!(span.section_kind, ObjectiveSectionKind::Scope)
            && span
                .excerpt
                .contains("Review the objective extractor, run make test.")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::Scope)
            && span
                .excerpt
                .contains("Review the objective extractor, run make test.")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured
        .deliverables
        .iter()
        .any(|deliverable| deliverable.display.contains("Return concrete fixes")));
}

#[test]
fn checkpoints_context_objective_combines_explicit_target_and_mixed_role_verification_without_scaffold_drift(
) {
    let prompt = r#"Read first:
- AGENTS.md
- docs/specs/r5/R5_75/MAP.md

## Scope
Review crates/agent-drift-analyzer/src/context/objective.rs only and run cargo test -p agent-drift-analyzer checkpoints -- --nocapture.

## Checklist
- Inspect AGENTS.md before editing.
- Keep the packet scoped to B4.1.

## Constraints
- Do not widen into SO-3.

## Return with
- concrete findings"#;
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,260p' crates/agent-drift-analyzer/src/context/objective.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(3, "turn-001", "Exit code: 0"),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(
        objective.text,
        "Review crates/agent-drift-analyzer/src/context/objective.rs only and run cargo test -p agent-drift-analyzer checkpoints -- --nocapture."
    );
    assert_eq!(
        objective.verification_commands,
        vec!["cargo test -p agent-drift-analyzer checkpoints -- --nocapture"]
    );

    let structured = objective.structured.as_ref().expect("structured objective");
    let target = structured.target.as_ref().expect("explicit target");
    assert_eq!(target.kind, ObjectiveTargetKind::FileOrDirectory);
    assert_eq!(
        target.display,
        "crates/agent-drift-analyzer/src/context/objective.rs"
    );
    assert_eq!(
        structured.verification_commands,
        vec!["cargo test -p agent-drift-analyzer checkpoints -- --nocapture"]
    );
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && matches!(span.section_kind, ObjectiveSectionKind::Scope)
            && span
                .excerpt
                .contains("Review crates/agent-drift-analyzer/src/context/objective.rs only and run cargo test -p agent-drift-analyzer checkpoints -- --nocapture.")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Verification
            && matches!(span.section_kind, ObjectiveSectionKind::Scope)
            && span
                .excerpt
                .contains("cargo test -p agent-drift-analyzer checkpoints -- --nocapture")
            && span.section_index.is_some()
            && span.clause_index.is_some()
    }));
    assert!(!structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && span.excerpt.contains("Inspect AGENTS.md before editing.")
    }));
    assert!(structured
        .deliverables
        .iter()
        .any(|deliverable| deliverable.display.contains("concrete findings")));
    assert!(structured
        .unknowns
        .iter()
        .all(|unknown| unknown.field_name != "target"));

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, objective.text);
}

#[test]
fn checkpoints_context_objective_projects_multiline_goal_sections_from_structured_state() {
    let prompt = r#"/goal Review crates/agent-drift-analyzer/src/context/objective.rs only.
Use the $incremental-implementation skill.
Return with changed files and residual risk.

## Verification
- cargo test -p agent-drift-analyzer checkpoints -- --nocapture"#;
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,260p' crates/agent-drift-analyzer/src/context/objective.rs","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(3, "turn-001", "Exit code: 0"),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(
        objective.text,
        "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only."
    );
    assert!(!objective.text.contains("$incremental-implementation"));
    assert_eq!(
        objective.verification_commands,
        vec!["cargo test -p agent-drift-analyzer checkpoints -- --nocapture"]
    );

    let structured = objective.structured.as_ref().expect("structured objective");
    let target = structured.target.as_ref().expect("explicit target");
    assert_eq!(target.kind, ObjectiveTargetKind::FileOrDirectory);
    assert_eq!(
        target.display,
        "crates/agent-drift-analyzer/src/context/objective.rs"
    );
    assert_ne!(target.display, objective.text);
    let goal_span = structured
        .evidence_spans
        .iter()
        .find(|span| {
            span.role == ObjectiveRole::Goal
                && span.excerpt
                    == "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only."
        })
        .expect("goal evidence span");
    assert_eq!(goal_span.excerpt, objective.text);
}

#[test]
fn checkpoints_context_objective_comparison_key_ignores_boilerplate_wording_changes() {
    let goal =
        "/goal Determine whether the AGENTS.md instruction block and <skill> section should change.";
    let prompt_a = format!(
        "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never.\n\n{goal}"
    );
    let prompt_b = format!(
        "Use memory by default when the query mentions a workspace. Memory citation requirements stay active.\n\n{goal}"
    );

    let first = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &prompt_a),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,40p' AGENTS.md","workdir":"/repo"}"#,
        ),
    ]);
    let second = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &prompt_b),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,40p' AGENTS.md","workdir":"/repo"}"#,
        ),
    ]);

    let first_objective = &first.sessions[0].context.objective;
    let second_objective = &second.sessions[0].context.objective;

    assert_eq!(first_objective.text, goal);
    assert_eq!(second_objective.text, goal);
    assert_eq!(
        first_objective.comparison_key,
        "review|skill_or_instruction_surface|agents_md|instruction_block|skill"
    );
    assert_eq!(
        first_objective.comparison_key,
        second_objective.comparison_key
    );
}

#[test]
fn checkpoints_context_objective_leaves_vague_targets_unknown() {
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", "Look at the stuff above and make it better."),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"echo vague-objective","workdir":"/repo"}"#,
        ),
    ]);

    let structured = result.sessions[0]
        .context
        .objective
        .structured
        .as_ref()
        .expect("structured objective");
    assert!(structured.target.is_none());
    assert!(structured
        .unknowns
        .iter()
        .any(|unknown| unknown.field_name == "target"));
}

#[test]
fn checkpoints_context_objective_keeps_review_goal_but_leaves_weak_target_unknown() {
    let prompt = "We just completed implementing the entire handbook extraction phase set and I need you to review what landed in the codebase vs the planning docs and intended end state and validate/invalidate if it landed correctly and completely";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' HANDBOOK_ENGINE_EXTRACTION_PLAN.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/handbook-engine-extraction-slice-map.md","workdir":"/repo"}"#,
        ),
        tool_output_row(4, "turn-001", "Exit code: 0"),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert!(objective
        .text
        .contains("review what landed in the codebase"));

    let structured = objective.structured.as_ref().expect("structured objective");
    assert!(structured
        .evidence_spans
        .iter()
        .any(|span| span.role == ObjectiveRole::Goal));
    assert!(structured.target.is_none());
    assert!(structured
        .unknowns
        .iter()
        .any(|unknown| unknown.field_name == "target"));
}

#[test]
fn checkpoints_keep_grounded_validate_readiness_goal_over_optional_reviewer_nit() {
    let validate_ask = "Please validate that has all landed correctly/completely.";
    let readiness_followup =
        "Then validate whether we are ready to spec/plan/tasks out Packet 4 so we can continue on with implementation.";
    let validate_readiness_ask = format!("{validate_ask}\n{readiness_followup}");
    let prompt = format!(
        r#"we have landed Packet 3 detailed here docs/research/cycle-stage-registry-refactor-map.md :

Packet 3.6 is implementation-complete and review-clean.

  I stayed orchestration-only: delegated implementation/fix rounds to fresh subagents, inspected each diff myself,
  reran verification as needed, and kept separate commits per round.

  Commits created, in order

  1. 12807dc — Close Packet 3 with review-ready proof wall
  2. 43e3164 — Tighten Packet 3 closeout scope and deferrals
  3. a4e8718 — Clarify Packet 3 closeout boundaries and dry-run proof
  4. f71f1d4 — Resolve Packet status ledger contradiction

  Verification commands run

  - git diff --check
  - .agents/skills/cycle/bin/cycle validate --project-root "$PWD"
  - .agents/skills/cycle/bin/cycle next --project-root "$PWD" --dry-run --no-auto-continue
  - Packet 3.6 doc rg checks for:
      - Packet 4
      - transition routing
      - outcome/final-marker meaning
      - blocked semantics

  Important precision note

  - The dry-run command is not stderr-clean in this source checkout.
  - It exits 0 and prints the dry-run confirmation, but still emits:
      - rsync(...): error: mkstempsock: Invalid argument

  - Packet 3.6 docs now record that honestly as an exit-0 smoke proof, not a clean stderr-free proof.

  Non-blocking follow-ups intentionally deferred

  - Packet 4 work remains deferred:
      - transition routing
      - outcome/final-marker meaning
      - blocked semantics
      - validator-semantic cutover

  - LangGraph remains deferred.
  - The higher-level horizon above objective remains deferred.
  - Optional reviewer nits not taken:
      - add extra task-local grep checks for transition routing / outcome/final-marker meaning
      - normalize one blocked-semantics wording instance

  So the final state is: Packet 3.6 landed, commits separated correctly, verification wall recorded honestly, and
  review is clean.

---

{validate_ask}
{readiness_followup}"#
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,260p' docs/research/cycle-stage-registry-refactor-map.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, validate_readiness_ask);
    assert!(checkpoint.task_frame.objective.contains(validate_ask));
    assert!(checkpoint.task_frame.objective.contains(readiness_followup));
    assert!(
        !checkpoint
            .task_frame
            .objective
            .contains("add extra task-local grep checks"),
        "optional reviewer nit must not become the effective objective"
    );

    let structured = checkpoint
        .structured_objective
        .as_ref()
        .expect("structured objective");
    let goal_excerpts = structured
        .evidence_spans
        .iter()
        .filter(|span| span.role == ObjectiveRole::Goal)
        .map(|span| span.excerpt.as_str())
        .collect::<Vec<_>>();
    assert!(
        goal_excerpts
            .iter()
            .any(|excerpt| excerpt.contains(validate_ask)),
        "expected validate ask in goal evidence spans, got {:?}",
        goal_excerpts
    );
    assert!(
        goal_excerpts
            .iter()
            .any(|excerpt| excerpt.contains(readiness_followup)),
        "expected readiness follow-up in goal evidence spans, got {:?}",
        goal_excerpts
    );
    assert!(!structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && span
                .excerpt
                .contains("add extra task-local grep checks for transition routing / outcome/final-marker meaning")
    }));
}

#[test]
fn checkpoints_context_objective_preserves_explicit_file_target_without_copying_whole_goal() {
    let prompt = "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only for Packet B2.1 target honesty and return concrete findings.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,260p' crates/agent-drift-analyzer/src/context/objective.rs","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
    ]);

    let objective = &result.sessions[0].context.objective;
    let structured = objective.structured.as_ref().expect("structured objective");
    let target = structured.target.as_ref().expect("explicit target");

    assert_eq!(target.kind, ObjectiveTargetKind::FileOrDirectory);
    assert_eq!(
        target.display,
        "crates/agent-drift-analyzer/src/context/objective.rs"
    );
    assert_ne!(target.display, objective.text);
    assert!(structured
        .unknowns
        .iter()
        .all(|unknown| unknown.field_name != "target"));
}

#[test]
fn checkpoints_context_objective_preserves_b22_explicit_target_family_matrix() {
    struct Case {
        name: &'static str,
        prompt: &'static str,
        tool_command: &'static str,
        expected_kind: ObjectiveTargetKind,
        expected_display: &'static str,
    }

    let cases = [
        Case {
            name: "file_path",
            prompt: "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only and keep the packet scoped to that file target.",
            tool_command: r#"{"command":"sed -n '1,260p' crates/agent-drift-analyzer/src/context/objective.rs","workdir":"/repo"}"#,
            expected_kind: ObjectiveTargetKind::FileOrDirectory,
            expected_display: "crates/agent-drift-analyzer/src/context/objective.rs",
        },
        Case {
            name: "instruction_surface",
            prompt: "/goal Analyze the AGENTS.md instruction block only and update just that instruction text.",
            tool_command: r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
            expected_kind: ObjectiveTargetKind::SkillOrInstructionSurface,
            expected_display: "AGENTS.md, instruction block",
        },
        Case {
            name: "doc_path",
            prompt: "/goal Review docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md only and summarize the packet-scoped findings.",
            tool_command: r#"{"command":"sed -n '1,220p' docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md","workdir":"/repo"}"#,
            expected_kind: ObjectiveTargetKind::SpecOrDesignDoc,
            expected_display: "docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md",
        },
        Case {
            name: "test_or_verifier",
            prompt: "/goal Validate the objective_acceptance harness only before widening scope.",
            tool_command: r#"{"command":"cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture","workdir":"/repo"}"#,
            expected_kind: ObjectiveTargetKind::TestOrVerifier,
            expected_display: "objective_acceptance harness",
        },
        Case {
            name: "crate_or_package",
            prompt: "/goal Review the agent-drift-analyzer crate only and return packet-scoped findings.",
            tool_command: r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
            expected_kind: ObjectiveTargetKind::CrateOrPackage,
            expected_display: "agent-drift-analyzer",
        },
        Case {
            name: "workspace_ref",
            prompt: "/goal Add this skill to @shared-cab-app only and keep the answer scoped to that workspace target.",
            tool_command: r#"{"command":"Get-ChildItem -Force","workdir":"D:\\Shared-cab-app"}"#,
            expected_kind: ObjectiveTargetKind::RepoSlice,
            expected_display: "@shared-cab-app",
        },
        Case {
            name: "packet_work_item",
            prompt: "/goal Review Packet B2.2 only and return concrete findings.",
            tool_command: r#"{"command":"rg -n \"Task B2\\.2\" docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md","workdir":"/repo"}"#,
            expected_kind: ObjectiveTargetKind::RepoSlice,
            expected_display: "B2.2",
        },
        Case {
            name: "conceptual_topic",
            prompt: "/goal Validate the structured objective sidecar only and keep the response packet-scoped.",
            tool_command: r#"{"command":"echo structured-objective-sidecar","workdir":"/repo"}"#,
            expected_kind: ObjectiveTargetKind::ConceptualTopic,
            expected_display: "structured objective sidecar",
        },
    ];

    for (index, case) in cases.iter().enumerate() {
        let result = analyze_custom_rows(vec![
            prompt_row(0, "turn-001", case.prompt),
            tool_call_row(1, "turn-001", "functions.shell_command", case.tool_command),
        ]);

        let objective = &result.sessions[0].context.objective;
        let structured = objective.structured.as_ref().expect("structured objective");
        let target = structured
            .target
            .as_ref()
            .unwrap_or_else(|| panic!("case {} missing target for {}", index, case.name));

        assert_eq!(target.kind, case.expected_kind, "case {}", case.name);
        assert_eq!(target.display, case.expected_display, "case {}", case.name);
        assert_ne!(target.display, objective.text, "case {}", case.name);
        assert_eq!(
            target.evidence.first().map(|evidence| evidence.role),
            Some(ObjectiveRole::Goal),
            "case {}",
            case.name
        );
        assert!(
            structured
                .unknowns
                .iter()
                .all(|unknown| unknown.field_name != "target"),
            "case {}",
            case.name
        );
    }
}

#[test]
fn checkpoints_context_objective_does_not_treat_version_tokens_as_work_item_targets() {
    let prompt = "/goal Review whether v0.6 landed correctly and summarize the findings only.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"echo review-v0-6","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
    ]);

    let objective = &result.sessions[0].context.objective;
    let structured = objective.structured.as_ref().expect("structured objective");

    assert!(objective
        .text
        .contains("Review whether v0.6 landed correctly"));
    assert!(structured.target.is_none());
    assert!(structured
        .unknowns
        .iter()
        .any(|unknown| unknown.field_name == "target"));
}

#[test]
fn checkpoints_context_objective_keeps_explicit_tooling_target_inside_mixed_prompt_scaffolding() {
    let concrete_ask =
        "Determine whether the Codex desktop context, plugin instructions, and Apps (Connectors) scaffold should change, and explain only that tooling boilerplate decision.";
    let prompt = format!(
        r#"Tools are grouped by namespace.
Apps (Connectors) can be explicitly triggered in user messages.
Use memory by default when the query mentions a workspace.

Concrete workspace action request:
{concrete_ask}

Execution rules:
- keep the change reviewable
- do not widen into replay or schema work

Return with:
- changed files
- residual risk"#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"rg -n \"Codex desktop context|plugin instructions|Apps \\(Connectors\\)\" AGENTS.md CLAUDE.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
        ),
    ]);

    let objective = &result.sessions[0].context.objective;
    assert_eq!(objective.text, concrete_ask);
    assert_ne!(objective.text, prompt);
    let structured = objective.structured.as_ref().expect("structured objective");
    assert_eq!(
        structured.target.as_ref().map(|target| target.kind),
        Some(ObjectiveTargetKind::SkillOrInstructionSurface)
    );
}

#[test]
fn checkpoints_extract_inline_review_clause_from_single_line_prompt() {
    let prompt = "We just completed implementing the entire handbook extraction phase set and I need you to review what landed in the codebase vs the planning docs and intended end state and validate/invalidate if it landed correctly and completely";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' HANDBOOK_ENGINE_EXTRACTION_PLAN.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/handbook-engine-extraction-slice-map.md","workdir":"/repo"}"#,
        ),
        tool_output_row(4, "turn-001", "Exit code: 0"),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(
        checkpoint.task_frame.objective,
        "review what landed in the codebase vs the planning docs and intended end state and validate/invalidate if it landed correctly and completely"
    );
}

#[test]
fn checkpoints_extract_inline_use_skill_review_clause_from_single_line_prompt() {
    let prompt = "We just landed the complete docs/specs/r5 and now I need you to use the $code-review-and-quality skill to evaluate if what was implemented laned correctly and completely";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md","workdir":"/repo"}"#,
        ),
        tool_output_row(4, "turn-001", "Exit code: 0"),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(
        checkpoint.task_frame.objective,
        "use the $code-review-and-quality skill to evaluate if what was implemented laned correctly and completely"
    );
}

#[test]
fn checkpoints_keep_legacy_inline_narrowing_when_structured_target_stays_unknown() {
    let prompt = "We just completed implementing the entire handbook extraction phase set and I need you to review what landed in the codebase vs the planning docs and intended end state and validate/invalidate if it landed correctly and completely";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' HANDBOOK_ENGINE_EXTRACTION_PLAN.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/handbook-engine-extraction-slice-map.md","workdir":"/repo"}"#,
        ),
        tool_output_row(4, "turn-001", "Exit code: 0"),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(
        checkpoint.task_frame.objective,
        "review what landed in the codebase vs the planning docs and intended end state and validate/invalidate if it landed correctly and completely"
    );

    let structured = checkpoint
        .structured_objective
        .as_ref()
        .expect("structured objective");
    assert!(structured.target.is_none());
    assert!(structured
        .unknowns
        .iter()
        .any(|unknown| unknown.field_name == "target"));
    assert!(structured.evidence_spans.iter().any(|span| {
        span.role == ObjectiveRole::Goal
            && span
                .excerpt
                .contains("review what landed in the codebase vs the planning docs")
    }));
}

#[test]
fn checkpoints_extract_inline_manual_smoke_clause_from_single_line_prompt() {
    let prompt = "i fixed the formatting issue. but now I want you to actually perform a series of manual smoke checks on unzeen sessions and evaluate if what landed is providing the intended signal/classification/etc";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"python3 smoke.py --session unzeen","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,120p' target/manual-r55-validation/README.md","workdir":"/repo"}"#,
        ),
        tool_output_row(4, "turn-001", "Exit code: 0"),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(
        checkpoint.task_frame.objective,
        "perform a series of manual smoke checks on unzeen sessions and evaluate if what landed is providing the intended signal/classification/etc"
    );
}

#[test]
fn checkpoints_prefer_concrete_workspace_action_steer_over_earlier_skill_body() {
    let pasted_prompt = r#"[$skill-creator](C:\Users\<REDACTED>\.codex\skills\.system\skill-creator\SKILL.md) use this and improve if u want ---
name: frontend-pro
description: Triggers when the user asks to build, review, style, or modify frontend user interfaces, React/Vue components, web animations, or CSS/Tailwind styling.

# Frontend Pro Engineering Guidelines
You are acting under the `frontend-pro` skill. Your goal is to generate production-grade, accessible, and highly responsive frontend interfaces."#;
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", pasted_prompt),
        steer_row(1, "turn-001", "add this skill to @shared-cab-app"),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"Get-ChildItem -Force","workdir":"D:\\Shared-cab-app"}"#,
        ),
        tool_output_row(3, "turn-001", "Exit code: 0"),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"Get-Content -Path 'D:\\Shared-cab-app\\AGENTS.md' -Raw"}"#,
        ),
        tool_output_row(5, "turn-001", "Exit code: 0"),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(
        checkpoint.task_frame.objective,
        "add this skill to @shared-cab-app"
    );
}

#[test]
fn checkpoints_ignore_profile_bootstrap_scaffold_until_real_goal_arrives() {
    let goal =
        "/goal Review the already-landed Packet R5-7 implementation and determine whether it is ready to keep.";
    let result = analyze_custom_rows(vec![
        system_row(
            0,
            "turn-001",
            "For sessions using the `buildconnectors_azure|code_intel_azure|handbook_east2_azure` profile:\n- Prefer spawned subagents with the built-in `default` agent type.\n- Only use non-default built-in agent roles on this profile when the user explicitly asks for them and the runtime behavior has been revalidated in the current environment.",
        ),
        developer_row(
            1,
            "turn-001",
            "<permissions instructions>\nFilesystem sandboxing defines which files can be read or written. Approval policy is currently never.\n</permissions instructions>",
        ),
        row(
            2,
            "turn-001",
            CompactionKind::UserMessage,
            "# AGENTS.md instructions for /repo\n<INSTRUCTIONS>\nWhen a user message ends with thoughts? do not implement.\n</INSTRUCTIONS>",
            None,
        )
        .with_user_message_role(UserMessageRole::Unknown),
        row(
            3,
            "turn-001",
            CompactionKind::Unknown,
            r#"{"payload":{"approval_policy":"never"},"type":"environment_context"}"#,
            None,
        ),
        prompt_row(4, "turn-001", goal),
        tool_call_row(
            5,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md","workdir":"/repo"}"#,
        ),
        tool_output_row(6, "turn-001", "Exit code: 0"),
        tool_call_row(
            7,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md","workdir":"/repo"}"#,
        ),
        tool_output_row(8, "turn-001", "Exit code: 0"),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].task_frame.objective, goal);
}

#[test]
fn checkpoints_keep_embedded_agents_instruction_targets_intact_after_condensation() {
    let concrete_ask =
        "Compare the AGENTS.md instructions block against the current task behavior and explain only the mismatches.";
    let pasted_prompt = format!(
        r#"Use memory by default when the query mentions a workspace.
Plugin bundles can lazy-load additional app surfaces with MCP tools.
Keep the fix reviewable and objective-selection focused.

Concrete task ask:
{concrete_ask}

Return with:
- changed files
- tests run
- residual risk"#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &pasted_prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, concrete_ask);
}

#[test]
fn checkpoints_keep_embedded_tooling_boilerplate_targets_intact_after_condensation() {
    let concrete_ask =
        "Determine whether the Codex desktop context, plugin instructions, and Apps (Connectors) scaffold should change, and explain only that tooling boilerplate decision.";
    let pasted_prompt = format!(
        r#"Tools are grouped by namespace.
Apps (Connectors) can be explicitly triggered in user messages.
Use memory by default when the query mentions a workspace.

Concrete workspace action request:
{concrete_ask}

Do not widen into replay or schema work.
Return with: changed files, tests run, residual risk."#,
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &pasted_prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"rg -n \"Codex desktop context|plugin instructions|Apps \\(Connectors\\)\" AGENTS.md CLAUDE.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, concrete_ask);
}

#[test]
fn checkpoints_preserve_first_paragraph_for_realistic_multiline_goal_review_prompts() {
    let goal = concat!(
        "/goal Review Candidate 3 Packet 2: Canonical Artifact Migration\n",
        "Use $code-review-and-quality from /Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md."
    );
    let pasted_prompt = r#"/goal Review Candidate 3 Packet 2: Canonical Artifact Migration
Use $code-review-and-quality from /Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md.

Review the current Packet 2 implementation in /Users/spensermcconnell/__Active_Code/system. This review is for the committed change at HEAD (commit f7296f2), not for unrelated unstaged worktree changes such as crates/cli/tests/cli_surface.rs.

Authoritative docs:
- /Users/spensermcconnell/__Active_Code/system/docs/specs/candidate-3-workspace-access-spec.md
- /Users/spensermcconnell/__Active_Code/system/docs/specs/candidate-3-workspace-access-plan.md
- /Users/spensermcconnell/__Active_Code/system/docs/specs/candidate-3-workspace-access-tasks.md
- /Users/spensermcconnell/__Active_Code/system/docs/contracts/C-03-canonical-artifact-manifest-contract.md

Required review scope:
- Findings-first review.
- Special attention to correctness, architecture depth, path/discovery semantics, and regression coverage.

Verification context already run by the implementer:
- cargo test -p handbook-compiler canonical_artifacts
- cargo test -p handbook-compiler --test resolver_core

Output requirements:
- If you find issues, list only actionable findings ordered by severity, with file/line references.
- If clean, say explicitly that you found no actionable issues and mention any residual risk or testing gap briefly.
- Do not make code changes."#;
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", pasted_prompt),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"git show --stat --oneline HEAD","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"cargo test -p handbook-compiler --test resolver_core","workdir":"/repo"}"#,
        ),
        tool_output_row(3, "turn-001", "Exit code: 0"),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, goal);
    assert_ne!(
        checkpoint.task_frame.objective,
        "/goal Review Candidate 3 Packet 2: Canonical Artifact Migration findings-first review without making code changes."
    );
    assert_eq!(
        checkpoint.session_archetype.as_ref().map(|a| a.label),
        Some(SessionArchetypeLabel::Planning)
    );
}

#[test]
fn checkpoints_preserve_user_requested_agents_instruction_targets() {
    let goal =
        "/goal Analyze the AGENTS.md instructions block and update only that instruction text.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: AGENTS.md
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, goal.trim_end());
}

#[test]
fn checkpoints_anchor_first_checkpoint_to_real_goal_after_boilerplate_then_activity() {
    let goal =
        "/goal Tighten the checkpoint boundary selector only after the real objective arrives.";
    let result = analyze_custom_rows(vec![
        developer_row(
            0,
            "turn-001",
            "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never. /goal Follow the permission boilerplate first.",
        ),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"rg -n \"checkpoint\" crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        prompt_row(3, "turn-001", goal),
        tool_call_row(
            4,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1920,2260p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_output_row(5, "turn-001", "Exit code: 0"),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].task_frame.objective, goal);
}

#[test]
fn checkpoints_preserve_long_form_goal_targeting_agents_skill_and_available_skills() {
    let goal = format!(
        "/goal Tighten only the user-requested boilerplate target preservation so the analyzer keeps long real objectives that explicitly inspect AGENTS.md instructions, the <skill> block, and Available skills while still rejecting raw injected scaffolding. Context: {}",
        "keep the fix scoped to objective selection and preservation heuristics. ".repeat(8)
    );
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", &goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' /Users/me/.agents/skills/incremental-implementation/SKILL.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            3,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"rg -n \"Available skills\" /tmp/session.txt","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, goal.trim_end());
}

#[test]
fn checkpoints_preserve_user_requested_tooling_plugin_app_and_safety_targets() {
    let goal = "/goal Analyze the Codex desktop context, plugin instructions, Apps (Connectors) scaffold, and safety guardrails only, then update just that boilerplate wording.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"rg -n \"Codex desktop context|plugin instructions|Apps \\(Connectors\\)|safety guardrails\" AGENTS.md CLAUDE.md","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, goal);
}

#[test]
fn checkpoints_preserve_compare_style_agents_instruction_targets() {
    let goal =
        "/goal Compare the AGENTS.md instructions block against the current task behavior and explain the mismatches only.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, goal);
}

#[test]
fn checkpoints_preserve_explain_style_skill_targets() {
    let goal =
        "/goal Explain whether the <skill> block conflicts with the current packet scope and keep the answer limited to that skill boilerplate.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' /Users/me/.agents/skills/incremental-implementation/SKILL.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"rg -n \"<skill>|Available skills\" /tmp/session.txt","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, goal);
}

#[test]
fn checkpoints_preserve_determine_style_tooling_targets() {
    let goal =
        "/goal Determine whether the Codex desktop context, plugin instructions, and Apps (Connectors) scaffold should change, and explain only that tooling boilerplate decision.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"rg -n \"Codex desktop context|plugin instructions|Apps \\(Connectors\\)\" AGENTS.md CLAUDE.md","workdir":"/repo"}"#,
        ),
        tool_call_row(
            2,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' CLAUDE.md","workdir":"/repo"}"#,
        ),
    ]);

    let checkpoint = result.sessions[0].checkpoints.last().expect("checkpoint");
    assert_eq!(checkpoint.task_frame.objective, goal);
}

#[test]
fn checkpoints_start_a_new_checkpoint_from_thread_goal_only_objectives() {
    let thread_goal = "Debug the failing analyzer checkpoint tests.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", thread_goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' crates/agent-drift-analyzer/src/checkpoint/mod.rs","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        row(
            3,
            "turn-002",
            CompactionKind::Unknown,
            &format!(
                r#"{{"goal":{{"objective":"{thread_goal}","status":"active"}},"threadId":"thread-123","type":"thread_goal_updated"}}"#
            ),
            None,
        ),
        tool_call_row(
            4,
            "turn-002",
            "functions.shell_command",
            r#"{"command":"cargo test -p agent-drift-analyzer checkpoints -- --nocapture","workdir":"/repo"}"#,
        ),
        tool_output_row(5, "turn-002", "Exit code: 0"),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints.len(), 2);
    assert_eq!(checkpoints[1].task_frame.objective, thread_goal);
    assert_eq!(
        checkpoints[1]
            .turn_context
            .as_ref()
            .and_then(|turn| turn.turn_id.as_deref()),
        Some("turn-002")
    );
}

#[test]
fn checkpoints_keep_multi_turn_agents_instruction_goals_separate_and_targeted() {
    let goal =
        "/goal Analyze the AGENTS.md instructions block and update only that instruction text.";
    let result = analyze_custom_rows(vec![
        prompt_row(0, "turn-001", goal),
        tool_call_row(
            1,
            "turn-001",
            "functions.shell_command",
            r#"{"command":"sed -n '1,220p' AGENTS.md","workdir":"/repo"}"#,
        ),
        tool_output_row(2, "turn-001", "Exit code: 0"),
        prompt_row(3, "turn-002", goal),
        tool_call_row(
            4,
            "turn-002",
            "functions.apply_patch",
            r#"{"command":"apply_patch <<'PATCH'
*** Begin Patch
*** Update File: AGENTS.md
*** End Patch
PATCH","workdir":"/repo"}"#,
        ),
        tool_output_row(5, "turn-002", "Exit code: 0"),
    ]);

    let checkpoints = &result.sessions[0].checkpoints;
    assert_eq!(checkpoints.len(), 2);
    assert_eq!(checkpoints[0].task_frame.objective, goal);
    assert_eq!(checkpoints[1].task_frame.objective, goal);
    assert_eq!(
        checkpoints[0]
            .turn_context
            .as_ref()
            .and_then(|turn| turn.turn_id.as_deref()),
        Some("turn-001")
    );
    assert_eq!(
        checkpoints[1]
            .turn_context
            .as_ref()
            .and_then(|turn| turn.turn_id.as_deref()),
        Some("turn-002")
    );
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
    let fixture = BundleFixture::from_compact_rows(rows);
    agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze custom rows")
}

fn prompt_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
    user_row(event_index, turn_id, text, UserMessageRole::Prompt)
}

fn steer_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
    user_row(event_index, turn_id, text, UserMessageRole::Steer)
}

fn user_row(event_index: usize, turn_id: &str, text: &str, role: UserMessageRole) -> CompactionRow {
    row(
        event_index,
        turn_id,
        CompactionKind::UserMessage,
        text,
        None,
    )
    .with_user_message_role(role)
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

fn system_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
    row(
        event_index,
        turn_id,
        CompactionKind::SystemMessage,
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

trait TestRowExt {
    fn with_user_message_role(self, role: UserMessageRole) -> Self;
}

impl TestRowExt for CompactionRow {
    fn with_user_message_role(mut self, role: UserMessageRole) -> Self {
        self.user_message_role = Some(role);
        self
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
