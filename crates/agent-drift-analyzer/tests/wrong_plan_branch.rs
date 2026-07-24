#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{AnalyzeRequest, Confidence, DriftClass, DriftState};
use agent_session_compactor::{
    CompactionKind, CompactionRow, DedupeGroup, RowRef, SourceKind, UserMessageRole,
};
use camino::Utf8PathBuf;
use support::{analyze_sample_bundle, read_checkpoints, BundleFixture};

#[test]
fn wrong_plan_branch_scores_out_of_scope_changes() {
    let result = analyze_sample_bundle();
    let score = result.sessions[0]
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.drift_scores.iter())
        .find(|score| {
            score.class == agent_drift_analyzer::DriftClass::WrongPlanBranch && score.flagged
        })
        .expect("wrong plan branch score");

    assert!(score.raw_score >= 60);
    assert!(score.flagged);
}

#[test]
fn wrong_plan_branch_ignores_read_only_out_of_scope_exploration() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Authorized scope: crates/agent-drift-analyzer/src/lib.rs and docs/specs/agent-drift-analyzer-v0.4-spec.md.",
        ),
        tool_row(2, "sed -n '1,120p' docs/specs/unrelated-plan.md"),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze read-only out-of-scope exploration bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints
        .last()
        .expect("read-only out-of-scope exploration checkpoint");
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::WrongPlanBranch)
        .expect("wrong plan branch score");

    assert_eq!(
        (
            score.raw_score,
            score.confidence,
            score.state,
            score.flagged,
        ),
        (0, Confidence::Medium, DriftState::Cleared, false),
    );
    assert!(score.evidence.is_empty());
}

#[test]
fn wrong_plan_branch_accepts_write_under_sanctioned_replan_scope() {
    let mut replan = row(
        2,
        CompactionKind::UserMessage,
        "Replan: update crates/agent-drift-analyzer/tests/wrong_plan_branch.rs using docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-spec.md.",
    );
    replan.user_message_role = Some(UserMessageRole::Steer);
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md.",
        ),
        tool_row(
            1,
            "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md",
        ),
        replan,
        tool_row(
            3,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/tests/wrong_plan_branch.rs\n*** End Patch\nPATCH",
        ),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sanctioned replan bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints.last().expect("sanctioned replan checkpoint");
    assert!(checkpoint
        .task_frame
        .truth_artifacts
        .iter()
        .any(|path| path == "crates/agent-drift-analyzer/tests/wrong_plan_branch.rs"));
    assert!(checkpoint
        .task_frame
        .working_set_paths
        .iter()
        .any(|path| path == "crates/agent-drift-analyzer/tests/wrong_plan_branch.rs"));
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::WrongPlanBranch)
        .expect("wrong plan branch score");

    assert_eq!(
        (
            score.raw_score,
            score.confidence,
            score.state,
            score.flagged,
        ),
        (0, Confidence::Medium, DriftState::Cleared, false),
    );
    assert!(score.evidence.is_empty());
}

#[test]
fn wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action() {
    let mut spawn = row(
        1,
        CompactionKind::ToolCall,
        "{\"goal\":\"implement the delegated wrong-plan-branch acceptance control\"}",
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
            "/goal Coordinate delegated work using docs/specs/agent-drift-analyzer-v0.4-spec.md and crates/agent-drift-analyzer/src/lib.rs without overclaiming child progress.",
        ),
        spawn,
        row(
            2,
            CompactionKind::SystemMessage,
            "Authorized parent scope remains docs/specs/agent-drift-analyzer-v0.4-spec.md and crates/agent-drift-analyzer/src/lib.rs. Child session id 019ea333-3333-7333-8333-333333333333 remains in a separate rollout file.",
        ),
        wait,
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze opaque-parent wrong-plan-branch bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints.last().expect("opaque-parent checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("opaque-parent session progress");
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::WrongPlanBranch)
        .expect("wrong plan branch score");

    assert_eq!(
        progress.dimension,
        agent_drift_analyzer::ProgressDimension::ParentVisibleOrchestration,
    );
    assert!(!checkpoint.task_frame.truth_artifacts.is_empty());
    assert!(!checkpoint.task_frame.working_set_paths.is_empty());
    assert_eq!(
        (
            score.raw_score,
            score.confidence,
            score.state,
            score.flagged,
        ),
        (0, Confidence::Medium, DriftState::Cleared, false),
    );
    assert!(score.evidence.is_empty());
}

#[test]
fn wrong_plan_branch_makes_no_claim_for_path_action_without_authority() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "Continue the current task.",
        ),
        tool_row(
            1,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\n*** End Patch\nPATCH",
        ),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze empty-authority wrong-plan-branch bundle");
    let context = &result.sessions[0].context;
    assert!(context.truth_artifacts.is_empty());
    assert_eq!(context.working_set_paths.len(), 1);
    assert!(context
        .working_set_paths
        .iter()
        .all(|path| path.source == "observed_command"));
    assert_eq!(context.command_observations.len(), 1);
    assert!(context.command_observations[0].write_like);
    assert_eq!(
        context.command_observations[0].paths,
        vec!["crates/agent-drift-analyzer/src/lib.rs"]
    );

    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints.last().expect("empty-authority checkpoint");
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::WrongPlanBranch)
        .expect("wrong plan branch score");

    assert_eq!(
        (
            score.raw_score,
            score.confidence,
            score.state,
            score.flagged,
        ),
        (0, Confidence::Low, DriftState::Cleared, false),
    );
    assert!(score.evidence.is_empty());
}

#[test]
fn wrong_plan_branch_control_syntax_never_establishes_path_authority() {
    let cases = [
        ("/goal Update src/foo and keep changes there.", true),
        ("/goal Update /repo/src/foo and keep changes there.", true),
        ("/review --path src/foo", true),
        ("/review --path=src/foo", true),
        (
            "Continue the task; the ambiguous token /src/foo is not a typed path.",
            false,
        ),
    ];

    for (directive, expected_control_hint) in cases {
        let result = analyze_rows(vec![
            row(0, CompactionKind::UserMessage, directive),
            typed_tool_row(
                1,
                "functions.write_file",
                r#"{"path":"src/foobar.rs","workdir":"/repo"}"#,
            ),
        ]);
        if expected_control_hint {
            assert!(result.sessions[0]
                .context
                .truth_artifacts
                .iter()
                .all(|artifact| artifact.source == "control_directive_literal"));
        } else {
            assert!(result.sessions[0].context.truth_artifacts.is_empty());
        }

        let score = final_wrong_plan_score(&result);
        assert_eq!(
            (
                score.raw_score,
                score.confidence,
                score.state,
                score.flagged,
            ),
            (0, Confidence::Low, DriftState::Cleared, false),
            "{directive}",
        );
    }
}

#[test]
fn wrong_plan_branch_normalizes_typed_paths_and_rejects_unsafe_paths() {
    let cases = [
        (
            "typed-relative",
            "src/foo/./bar.rs",
            "/repo",
            vec!["src/foo/bar.rs".to_string()],
        ),
        (
            "typed-absolute",
            "/repo/src/foo/bar.rs",
            "/repo",
            vec!["src/foo/bar.rs".to_string()],
        ),
        (
            "typed-parent-component",
            "src/foo/tmp/../bar.rs",
            "/repo",
            vec!["src/foo/bar.rs".to_string()],
        ),
        (
            "typed-windows-absolute",
            r"C:\repo\src\foo\bar.rs",
            r"C:\repo",
            vec!["src/foo/bar.rs".to_string()],
        ),
        ("typed-external", "/tmp/src/foo/bar.rs", "/repo", Vec::new()),
        (
            "typed-windows-external",
            r"D:\repo\src\foo\bar.rs",
            r"C:\repo",
            Vec::new(),
        ),
        (
            "typed-traversal",
            "../../src/foo/bar.rs",
            "/repo",
            Vec::new(),
        ),
    ];

    for (case, path, workdir, expected_paths) in cases {
        let payload = format!(r#"{{"path":{path:?},"workdir":{workdir:?}}}"#);
        let result = analyze_rows(vec![
            row(
                0,
                CompactionKind::UserMessage,
                "Implement the requested change.",
            ),
            row(
                1,
                CompactionKind::SystemMessage,
                "Authorized filesystem scope: src/foo.",
            ),
            typed_tool_row(2, "functions.write_file", &payload),
        ]);
        assert_eq!(
            result.sessions[0].context.command_observations[0].paths, expected_paths,
            "{case}",
        );
        let score = final_wrong_plan_score(&result);
        assert_eq!(score.raw_score, 0, "{case}");
        assert!(!score.flagged, "{case}");
    }
}

#[test]
fn wrong_plan_branch_uses_component_aware_scope_containment() {
    let cases = [
        ("exact", "src/foo", false),
        ("descendant", "src/foo/bar.rs", false),
        ("dot-and-repeated-separators", "src/./foo//bar.rs", false),
        ("portable-separators", r"src\foo\bar.rs", false),
        ("lexical-prefix-collision", "src/foobar.rs", true),
        ("sibling", "src/bar.rs", true),
    ];

    for (case, path, flagged) in cases {
        let payload = format!(r#"{{"path":{path:?},"workdir":"/repo"}}"#);
        let result = analyze_rows(vec![
            row(
                0,
                CompactionKind::UserMessage,
                "Implement the requested change.",
            ),
            row(
                1,
                CompactionKind::SystemMessage,
                "Authorized filesystem scope: src/foo.",
            ),
            typed_tool_row(2, "functions.write_file", &payload),
        ]);
        let score = final_wrong_plan_score(&result);
        assert_eq!(score.flagged, flagged, "{case}");
        assert_eq!(score.raw_score, if flagged { 60 } else { 0 }, "{case}");
    }

    let root = analyze_rows(vec![
        row(
            0,
            CompactionKind::UserMessage,
            "Implement the requested change.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Authorized filesystem scope: ./.",
        ),
        typed_tool_row(
            2,
            "functions.write_file",
            r#"{"path":"src/anything.rs","workdir":"/repo"}"#,
        ),
    ]);
    assert_eq!(
        root.sessions[0].context.command_observations[0].paths,
        vec!["src/anything.rs".to_string()]
    );
    assert!(!final_wrong_plan_score(&root).flagged);
}

#[test]
fn wrong_plan_branch_clears_after_a_later_interval_returns_in_scope() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Stay inside crates/agent-drift-analyzer/src/lib.rs and the v0.4 spec.",
        ),
        tool_row(
            2,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Add File: crates/agent-drift-sentinel/src/offscope_notes.rs\n+rogue\n*** End Patch\nPATCH",
        ),
        row(
            3,
            CompactionKind::AssistantMessage,
            "I found the off-scope change. Next I am returning to the requested file.",
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
        canonical_text_hash_hex: "wrong-plan-branch-dup".to_string(),
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
        .find(|score| score.class == agent_drift_analyzer::DriftClass::WrongPlanBranch)
        .expect("first wrong plan branch score");
    let second = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::WrongPlanBranch)
        .expect("second wrong plan branch score");

    assert!(first.flagged);
    assert_eq!(first.raw_score, 60);
    assert_eq!(first.state, DriftState::Active);
    assert!(!second.flagged);
    assert_eq!(second.raw_score, 0);
    assert_eq!(second.state, DriftState::Cleared);
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

fn analyze_rows(rows: Vec<CompactionRow>) -> agent_drift_analyzer::AnalyzeResult {
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
    agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze path-semantics bundle")
}

fn final_wrong_plan_score(
    result: &agent_drift_analyzer::AnalyzeResult,
) -> &agent_drift_analyzer::DriftScore {
    result.sessions[0]
        .checkpoints
        .last()
        .expect("path-semantics checkpoint")
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::WrongPlanBranch)
        .expect("wrong plan branch score")
}

fn typed_tool_row(event_index: usize, tool_name: &str, payload: &str) -> CompactionRow {
    let mut row = row(event_index, CompactionKind::ToolCall, payload);
    row.dedupe_identity = Some(format!(
        r#"{{"call_id":"call-{event_index}","name":{tool_name:?},"type":"function_call"}}"#,
    ));
    row
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
