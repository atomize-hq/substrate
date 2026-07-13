#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{AnalyzeRequest, Confidence, DriftClass, DriftState};
use agent_session_compactor::{
    CompactionKind, CompactionRow, DedupeGroup, RowRef, SourceKind, UserMessageRole,
};
use camino::Utf8PathBuf;
use support::{analyze_sample_bundle, read_checkpoints, BundleFixture};

#[test]
fn truth_grounding_gap_flags_verification_without_truth_reads() {
    let result = analyze_sample_bundle();
    let score = result.sessions[0]
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.drift_scores.iter())
        .find(|score| score.class == DriftClass::TruthGroundingGap && score.flagged)
        .expect("truth grounding gap score");

    assert!(score.raw_score >= 60);
    assert!(score.flagged);
    assert_eq!(score.state, DriftState::Active);
}

#[test]
fn truth_grounding_gap_keeps_no_action_planning_clear() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Research the analyzer plan using docs/specs/agent-drift-analyzer-v0.4-spec.md before deciding what to implement.",
        ),
        row(
            1,
            CompactionKind::AssistantMessage,
            "I will compare the declared authority with the current planning assumptions and report the research result before proposing any action.",
        ),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze no-action planning bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints.last().expect("no-action planning checkpoint");
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("truth grounding gap score");

    assert_eq!(
        (
            score.raw_score,
            score.confidence,
            score.state,
            score.flagged,
        ),
        (0, Confidence::Medium, DriftState::Cleared, false),
    );
    assert!(!score.evidence.is_empty());
    assert!(score
        .evidence
        .iter()
        .all(|evidence| evidence.reason.starts_with("truth artifact hint:")));
}

#[test]
fn truth_grounding_gap_flags_successful_verification_without_truth_reads() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer --test truth_grounding_gap -- --nocapture`.",
        ),
        tool_row(
            1,
            "cargo test -p agent-drift-analyzer --test truth_grounding_gap -- --nocapture",
        ),
        row(
            2,
            CompactionKind::ToolOutput,
            "Exit code: 0\nrunning 1 test\ntest result: ok. 1 passed; 0 failed",
        ),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze successful ungrounded verification bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints
        .last()
        .expect("successful ungrounded verification checkpoint");
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("truth grounding gap score");

    assert_eq!(
        (
            score.raw_score,
            score.confidence,
            score.state,
            score.flagged,
        ),
        (80, Confidence::High, DriftState::Active, true),
    );
    assert!(score.evidence.iter().any(|evidence| {
        evidence.reason == "truth artifact hint: docs/specs/agent-drift-analyzer-v0.4-spec.md"
    }));
    assert!(score
        .evidence
        .iter()
        .any(|evidence| evidence.reason == "command family: cargo"));
}

#[test]
fn truth_grounding_gap_is_event_order_invariant_across_turn_shapes() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md.",
        ),
        row(
            1,
            CompactionKind::AssistantMessage,
            "I will preserve the declared truth while preparing the requested update.",
        ),
        row(
            2,
            CompactionKind::AssistantMessage,
            "I will keep the action order fixed across the session.",
        ),
        tool_row(3, "pwd"),
        row(4, CompactionKind::ToolOutput, "/repo"),
        row(
            5,
            CompactionKind::UserMessage,
            "/goal Continue updating crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer --test truth_grounding_gap -- --nocapture`.",
        ),
        row(
            6,
            CompactionKind::AssistantMessage,
            "I will continue with the same declared truth and event order.",
        ),
        row(
            7,
            CompactionKind::AssistantMessage,
            "I will preserve the same verification action before reporting its result.",
        ),
        row(
            8,
            CompactionKind::AssistantMessage,
            "I am verifying the requested source change now.",
        ),
        tool_row(9, "pwd"),
        row(10, CompactionKind::ToolOutput, "/repo"),
        tool_row(
            11,
            "cargo test -p agent-drift-analyzer --test truth_grounding_gap -- --nocapture",
        ),
        row(
            12,
            CompactionKind::ToolOutput,
            "Exit code: 0\nrunning 1 test\ntest result: ok. 1 passed; 0 failed",
        ),
    ];
    let long_autonomous_rows = rows.clone();
    let mut many_short_conversational_rows = rows;
    for row in &mut many_short_conversational_rows[5..] {
        row.turn_id = Some("turn-002".to_string());
    }

    let analyze = |rows: Vec<CompactionRow>, shape: &str| {
        let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
        let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
            input_dir: fixture.input_dir.clone(),
            output_dir: fixture.output_dir.clone(),
        })
        .unwrap_or_else(|error| panic!("analyze {shape} truth-grounding bundle: {error}"));
        read_checkpoints(&result.checkpoints_path)
    };

    let long_checkpoints = analyze(long_autonomous_rows, "long-autonomous");
    let short_checkpoints = analyze(many_short_conversational_rows, "many-short-conversational");

    assert_eq!(long_checkpoints.len(), 2);
    assert_eq!(short_checkpoints.len(), 2);
    assert!(short_checkpoints.iter().all(|checkpoint| {
        checkpoint.turn_context.as_ref().is_some_and(|context| {
            context.execution_mode == agent_drift_analyzer::TurnExecutionMode::Conversational
        })
    }));
    let long_checkpoint = long_checkpoints.last().expect("long-autonomous checkpoint");
    let short_checkpoint = short_checkpoints
        .last()
        .expect("many-short-conversational checkpoint");
    assert_eq!(
        long_checkpoint
            .turn_context
            .as_ref()
            .expect("long-autonomous turn context")
            .execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Autonomous,
    );
    assert_eq!(
        short_checkpoint
            .turn_context
            .as_ref()
            .expect("many-short-conversational turn context")
            .execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Conversational,
    );

    let long_score = long_checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("long-autonomous truth grounding gap score");
    let short_score = short_checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("many-short-conversational truth grounding gap score");
    for score in [long_score, short_score] {
        assert_eq!(
            (
                score.raw_score,
                score.confidence,
                score.state,
                score.flagged,
            ),
            (80, Confidence::High, DriftState::Active, true),
        );
    }

    let long_reasons = long_score
        .evidence
        .iter()
        .map(|evidence| evidence.reason.as_str())
        .collect::<Vec<_>>();
    let short_reasons = short_score
        .evidence
        .iter()
        .map(|evidence| evidence.reason.as_str())
        .collect::<Vec<_>>();
    assert_eq!(long_reasons, short_reasons);
    assert!(long_reasons.iter().any(|reason| {
        *reason == "truth artifact hint: docs/specs/agent-drift-analyzer-v0.4-spec.md"
    }));
    assert!(long_reasons
        .iter()
        .any(|reason| *reason == "command family: cargo"));
}

#[test]
fn truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action() {
    let mut spawn = row(
        1,
        CompactionKind::ToolCall,
        "{\"goal\":\"implement the delegated truth-grounding acceptance control\"}",
    );
    spawn.dedupe_identity = Some(
        "{\"call_id\":\"call-spawn\",\"name\":\"spawn_agent\",\"type\":\"function_call\"}"
            .to_string(),
    );
    let mut wait = row(
        3,
        CompactionKind::ToolCall,
        "{\"session_id\":\"019ea333-3333-7333-8333-333333333333\"}",
    );
    wait.dedupe_identity = Some(
        "{\"call_id\":\"call-wait\",\"name\":\"wait_agent\",\"type\":\"function_call\"}"
            .to_string(),
    );
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Coordinate delegated work using docs/specs/agent-drift-analyzer-v0.4-spec.md without overclaiming child progress.",
        ),
        spawn,
        row(
            2,
            CompactionKind::SystemMessage,
            "Child session id 019ea333-3333-7333-8333-333333333333 remains in a separate rollout file.",
        ),
        wait,
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze opaque-parent truth-grounding bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints.last().expect("opaque-parent checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("opaque-parent session progress");
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("truth grounding gap score");

    assert_eq!(
        progress.dimension,
        agent_drift_analyzer::ProgressDimension::ParentVisibleOrchestration,
    );
    assert_eq!(
        (
            score.raw_score,
            score.confidence,
            score.state,
            score.flagged,
        ),
        (0, Confidence::Medium, DriftState::Cleared, false),
    );
    assert!(!score.evidence.is_empty());
    assert!(score
        .evidence
        .iter()
        .all(|evidence| evidence.reason.starts_with("truth artifact hint:")));
}

#[test]
fn truth_grounding_gap_flags_truth_path_action_before_read() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update docs/specs/agent-drift-analyzer-v0.4-spec.md using that declared truth artifact before changing behavior.",
        ),
        tool_row(
            1,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: docs/specs/agent-drift-analyzer-v0.4-spec.md\n*** End Patch\nPATCH",
        ),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze truth-path action-before-read bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints
        .last()
        .expect("truth-path action-before-read checkpoint");
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("truth grounding gap score");

    assert_eq!(
        (
            score.raw_score,
            score.confidence,
            score.state,
            score.flagged,
        ),
        (80, Confidence::High, DriftState::Active, true),
    );
    assert!(score.evidence.iter().any(|evidence| {
        evidence.reason == "truth artifact hint: docs/specs/agent-drift-analyzer-v0.4-spec.md"
    }));
    assert!(score
        .evidence
        .iter()
        .any(|evidence| evidence.reason == "command family: apply_patch"));
}

#[test]
fn truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes() {
    let truth_path = "docs/specs/agent-drift-analyzer-v0.4-spec.md";
    let action = "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: scratch/archetype-equivalence-control.toml\n*** End Patch\nPATCH";
    let analyze = |task: &str, archetype: &str| {
        let rows = vec![
            row(
                0,
                CompactionKind::UserMessage,
                &format!("{task} using {truth_path} before changing behavior."),
            ),
            tool_row(1, action),
        ];
        let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
        let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
            input_dir: fixture.input_dir.clone(),
            output_dir: fixture.output_dir.clone(),
        })
        .unwrap_or_else(|error| panic!("analyze {archetype} truth-grounding bundle: {error}"));
        read_checkpoints(&result.checkpoints_path)
            .pop()
            .unwrap_or_else(|| panic!("{archetype} checkpoint"))
    };

    let planning = analyze(
        "Use code-review-and-quality to review the scorer context and record the findings",
        "planning/research",
    );
    let implementation = analyze("Implement the scorer context change", "implementation");

    assert_eq!(planning.task_frame.truth_artifacts, vec![truth_path]);
    assert_eq!(
        implementation.task_frame.truth_artifacts,
        planning.task_frame.truth_artifacts,
    );
    assert_eq!(
        planning
            .session_archetype
            .as_ref()
            .expect("planning/research archetype")
            .label,
        agent_drift_analyzer::SessionArchetypeLabel::Planning,
    );
    assert_eq!(
        implementation
            .session_archetype
            .as_ref()
            .expect("implementation archetype")
            .label,
        agent_drift_analyzer::SessionArchetypeLabel::AutonomousImplementation,
    );

    let planning_score = planning
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("planning/research truth grounding gap score");
    let implementation_score = implementation
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("implementation truth grounding gap score");
    for score in [planning_score, implementation_score] {
        assert_eq!(
            (
                score.raw_score,
                score.confidence,
                score.state,
                score.flagged,
            ),
            (80, Confidence::High, DriftState::Active, true),
        );
    }

    let planning_reasons = planning_score
        .evidence
        .iter()
        .map(|evidence| evidence.reason.as_str())
        .collect::<Vec<_>>();
    let implementation_reasons = implementation_score
        .evidence
        .iter()
        .map(|evidence| evidence.reason.as_str())
        .collect::<Vec<_>>();
    assert_eq!(planning_reasons, implementation_reasons);
    assert!(planning_reasons
        .contains(&"truth artifact hint: docs/specs/agent-drift-analyzer-v0.4-spec.md"));
    assert!(planning_reasons.contains(&"command family: apply_patch"));
}

#[test]
fn truth_grounding_gap_preserves_history_without_keeping_the_latest_interval_active() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer -- --nocapture`.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Read docs/specs/agent-drift-analyzer-v0.4-spec.md before acting.",
        ),
        tool_row(2, "cargo test -p agent-drift-analyzer -- --nocapture"),
        row(
            3,
            CompactionKind::AssistantMessage,
            "I need to re-ground on the spec before editing again.",
        ),
        tool_row(4, "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md"),
        tool_row(
            5,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\n*** End Patch\nPATCH",
        ),
    ];
    let mut archival_rows = rows.clone();
    let mut duplicate = archival_rows[2].clone();
    duplicate.row_ordinal = 1;
    duplicate.line_number += 100;
    archival_rows.push(duplicate.clone());
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "truth-grounding-gap-dup".to_string(),
        representative: RowRef::from_row(&rows[2]),
        duplicates: vec![RowRef::from_row(&duplicate)],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, rows, dedupe_groups);
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze custom bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    assert_eq!(checkpoints.len(), 2);
    let first = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("first truth grounding gap score");
    let second = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("second truth grounding gap score");

    assert!(first.flagged);
    assert!(!second.flagged);
    assert_eq!(first.state, DriftState::Active);
    assert_eq!(second.state, DriftState::Recovered);
    assert!(second.evidence.iter().any(|evidence| evidence
        .reason
        .starts_with("historical truth-grounding gap:")));
}

#[test]
fn truth_grounding_gap_does_not_turn_clean_grounding_into_historical_gap_evidence() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer -- --nocapture`.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Read docs/specs/agent-drift-analyzer-v0.4-spec.md before acting.",
        ),
        tool_row(2, "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md"),
        row(
            3,
            CompactionKind::AssistantMessage,
            "The spec is grounded; I am moving to the next checkpoint.",
        ),
        tool_row(4, "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md"),
        tool_row(
            5,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\n*** End Patch\nPATCH",
        ),
    ];
    let mut archival_rows = rows.clone();
    let mut duplicate = archival_rows[2].clone();
    duplicate.row_ordinal = 1;
    duplicate.line_number += 100;
    archival_rows.push(duplicate.clone());
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "truth-grounding-gap-clean-dup".to_string(),
        representative: RowRef::from_row(&rows[2]),
        duplicates: vec![RowRef::from_row(&duplicate)],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, rows, dedupe_groups);
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze custom bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    assert_eq!(checkpoints.len(), 2);
    let first = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("first truth grounding gap score");
    let second = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("second truth grounding gap score");

    assert!(!first.flagged);
    assert!(!second.flagged);
    assert_eq!(first.state, DriftState::Cleared);
    assert_eq!(second.state, DriftState::Cleared);
    assert!(first.evidence.iter().any(|evidence| !evidence
        .reason
        .starts_with("historical truth-grounding gap:")));
    assert!(!second.evidence.iter().any(|evidence| evidence
        .reason
        .starts_with("historical truth-grounding gap:")));
}

#[test]
fn truth_grounding_gap_downgrades_to_historical_only_after_the_recovery_transition() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer -- --nocapture`.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Read docs/specs/agent-drift-analyzer-v0.4-spec.md before acting.",
        ),
        tool_row(2, "cargo test -p agent-drift-analyzer -- --nocapture"),
        row(
            3,
            CompactionKind::AssistantMessage,
            "I need to re-ground on the spec before editing again.",
        ),
        tool_row(4, "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md"),
        row(
            5,
            CompactionKind::AssistantMessage,
            "Grounding is back in place. I am continuing with the requested change.",
        ),
        tool_row(
            6,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\n*** End Patch\nPATCH",
        ),
        row(
            7,
            CompactionKind::AssistantMessage,
            "The recovery checkpoint already happened; this is later historical context only.",
        ),
        tool_row(
            8,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\n*** End Patch\nPATCH",
        ),
    ];
    let mut archival_rows = rows.clone();
    let mut duplicate = archival_rows[2].clone();
    duplicate.row_ordinal = 1;
    duplicate.line_number += 100;
    archival_rows.push(duplicate.clone());
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "truth-grounding-gap-historical-only-dup".to_string(),
        representative: RowRef::from_row(&rows[2]),
        duplicates: vec![RowRef::from_row(&duplicate)],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, rows, dedupe_groups);
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze historical-only custom bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    assert!(checkpoints.len() >= 3);
    let truth_gap_scores = checkpoints
        .iter()
        .map(|checkpoint| {
            checkpoint
                .drift_scores
                .iter()
                .find(|score| score.class == DriftClass::TruthGroundingGap)
                .expect("truth grounding gap score")
        })
        .collect::<Vec<_>>();
    let recovered_index = truth_gap_scores
        .iter()
        .position(|score| score.state == DriftState::Recovered)
        .expect("recovered checkpoint");
    let historical_only_index = truth_gap_scores
        .iter()
        .rposition(|score| score.state == DriftState::HistoricalOnly)
        .expect("historical-only checkpoint");
    let historical_only = truth_gap_scores[historical_only_index];

    assert_eq!(truth_gap_scores[0].state, DriftState::Active);
    assert!(historical_only_index > recovered_index);
    assert!(!historical_only.flagged);
    assert!(historical_only.evidence.iter().any(|evidence| evidence
        .reason
        .starts_with("historical truth-grounding gap:")));
}

fn row(event_index: usize, kind: CompactionKind, text: &str) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index,
        line_number: event_index + 1,
        row_ordinal: 0,
        timestamp: None,
        kind,
        user_message_role: matches!(kind, CompactionKind::UserMessage)
            .then_some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: text.to_string(),
        canonical_text: text.to_string(),
        text_hash_hex: format!("hash-{event_index}"),
    }
}

fn tool_row(event_index: usize, command: &str) -> CompactionRow {
    let payload = format!("{{\"command\":{command:?},\"workdir\":\"/repo\"}}",);
    let mut row = row(event_index, CompactionKind::ToolCall, &payload);
    row.dedupe_identity = Some(
        "{\"call_id\":\"call-1\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"
            .to_string(),
    );
    row
}
