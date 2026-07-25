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
fn wrong_plan_branch_control_invocations_and_plain_absolute_paths_are_distinct() {
    let cases = [
        (
            "goal-chain",
            "/goal Update src/foo and then run /spec.",
            Vec::<&str>::new(),
            vec!["src/foo"],
        ),
        (
            "control-option-continuation",
            "/review\n--path /repo",
            Vec::<&str>::new(),
            vec!["/repo"],
        ),
        (
            "plain-prose-posix",
            "The trusted repository root is /repo for this task.",
            vec!["/repo"],
            Vec::<&str>::new(),
        ),
        (
            "standalone-posix",
            "/repo",
            vec!["/repo"],
            Vec::<&str>::new(),
        ),
        (
            "option-positional-posix",
            "--path /repo",
            vec!["/repo"],
            Vec::<&str>::new(),
        ),
        (
            "option-equals-posix",
            "--path=/repo",
            vec!["/repo"],
            Vec::<&str>::new(),
        ),
    ];

    for (case, directive, expected_authority, expected_control) in cases {
        let result = analyze_rows(vec![
            row(0, CompactionKind::UserMessage, directive),
            typed_tool_row(1, "functions.write_file", r#"{"path":"src/foobar.rs"}"#),
        ]);
        let context = &result.sessions[0].context;
        let authority = context
            .truth_artifacts
            .iter()
            .filter(|artifact| artifact.source != "control_directive_literal")
            .map(|artifact| artifact.path.as_str())
            .collect::<Vec<_>>();
        let controls = context
            .truth_artifacts
            .iter()
            .filter(|artifact| artifact.source == "control_directive_literal")
            .map(|artifact| artifact.path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(authority, expected_authority, "{case}");
        assert_eq!(controls, expected_control, "{case}");

        let score = final_wrong_plan_score(&result);
        assert_eq!(
            (score.raw_score, score.confidence, score.flagged),
            (0, Confidence::Low, false),
            "{case}",
        );
    }
}

#[test]
fn wrong_plan_branch_rejects_raw_parent_components_before_authority_normalization() {
    let cases = [
        "Authorized scope: scope/../sibling.",
        "Authorized scope: scope/../..",
        "Authorized scope: scope/..",
        "Authorized scope: scope/../../.",
        r"Authorized scope: scope\..\sibling.",
        "/review\n--path scope/../sibling",
        "/review\n--path=scope/../..",
        "/review\n--path scope/..",
        "/review\n--path=scope\\..\\sibling",
        "--path scope/../sibling",
        "--path=scope/..",
    ];

    for directive in cases {
        let result = analyze_rows(vec![
            row(0, CompactionKind::UserMessage, directive),
            typed_tool_row(
                1,
                "functions.write_file",
                r#"{"path":"scope/sibling/file.rs"}"#,
            ),
        ]);
        assert!(
            result.sessions[0].context.truth_artifacts.is_empty(),
            "{directive}",
        );
        assert!(
            result.sessions[0]
                .context
                .working_set_paths
                .iter()
                .all(|path| path.source == "observed_command"),
            "{directive}",
        );
        let score = final_wrong_plan_score(&result);
        assert_eq!(
            (score.raw_score, score.confidence, score.flagged),
            (0, Confidence::Low, false),
            "{directive}",
        );
    }
}

#[test]
fn truth_grounding_gap_preserves_typed_absolute_identity_and_lifecycle_provenance() {
    let cases = [
        (
            "posix",
            "/repo",
            "/repo/docs/specs/absolute-truth.md",
            "/repo/docs/specs",
            "/repo/docs/specs/absolute-truth.md",
        ),
        (
            "windows-drive",
            r"C:\repo",
            r"C:\repo\docs\specs\absolute-truth.md",
            r"C:\repo\docs\specs",
            "c:/repo/docs/specs/absolute-truth.md",
        ),
    ];

    for (case, root, truth_path, nested_cwd, expected_truth_path) in cases {
        let rows = vec![
            row(
                0,
                CompactionKind::UserMessage,
                &format!(
                    "/goal Update the absolute truth artifact {truth_path} only after grounding it.\nTrusted repository root: {root}."
                ),
            ),
            typed_tool_row(
                1,
                "functions.write_file",
                &format!(r#"{{"path":{truth_path:?},"cwd":{root:?}}}"#),
            ),
            row(
                2,
                CompactionKind::AssistantMessage,
                "I need to re-ground on the absolute truth artifact before acting again.",
            ),
            typed_tool_row(
                3,
                "functions.read_file",
                &format!(r#"{{"path":"absolute-truth.md","cwd":{nested_cwd:?}}}"#),
            ),
            row(
                4,
                CompactionKind::AssistantMessage,
                "The absolute truth artifact is grounded; I am moving to the next checkpoint.",
            ),
            typed_tool_row(
                5,
                "functions.edit_file",
                &format!(r#"{{"path":"absolute-truth.md","cwd":{nested_cwd:?}}}"#),
            ),
        ];
        let result = analyze_rows(rows);
        assert_eq!(
            result.sessions[0]
                .context
                .command_observations
                .iter()
                .map(|command| command.paths.clone())
                .collect::<Vec<_>>(),
            vec![
                vec![expected_truth_path.to_string()],
                vec![expected_truth_path.to_string()],
                vec![expected_truth_path.to_string()],
            ],
            "{case}",
        );

        let truth_artifact = result.sessions[0]
            .context
            .truth_artifacts
            .iter()
            .find(|artifact| artifact.path == expected_truth_path)
            .expect("absolute truth artifact");
        assert!(
            truth_artifact.evidence.iter().any(|evidence| {
                evidence.row.event_index == 0
                    && evidence.reason == format!("truth artifact hint: {expected_truth_path}")
            }),
            "{case}",
        );

        let checkpoints = &result.sessions[0].checkpoints;
        assert_eq!(checkpoints.len(), 2, "{case}");
        assert!(
            checkpoints.iter().all(|checkpoint| checkpoint
                .task_frame
                .truth_artifacts
                .iter()
                .any(|path| path == expected_truth_path)),
            "{case}",
        );

        let truth_scores = checkpoints
            .iter()
            .map(|checkpoint| {
                checkpoint
                    .drift_scores
                    .iter()
                    .find(|score| score.class == DriftClass::TruthGroundingGap)
                    .expect("truth grounding gap score")
            })
            .collect::<Vec<_>>();
        assert!(
            truth_scores
                .iter()
                .all(|score| score.evidence.iter().any(|evidence| {
                    evidence.row.event_index == 0
                        && evidence.reason == format!("truth artifact hint: {expected_truth_path}")
                })),
            "{case}",
        );
        assert_eq!(
            (
                truth_scores[0].raw_score,
                truth_scores[0].confidence,
                truth_scores[0].state,
                truth_scores[0].flagged,
            ),
            (80, Confidence::High, DriftState::Active, true),
            "{case}",
        );
        assert_eq!(
            (
                truth_scores[1].raw_score,
                truth_scores[1].confidence,
                truth_scores[1].state,
                truth_scores[1].flagged,
            ),
            (20, Confidence::Medium, DriftState::Recovered, false),
            "{case}",
        );
        assert!(
            truth_scores[1].evidence.iter().any(|evidence| evidence
                .reason
                .starts_with("historical truth-grounding gap:")),
            "{case}",
        );

        for checkpoint in checkpoints {
            let wrong_plan = checkpoint
                .drift_scores
                .iter()
                .find(|score| score.class == DriftClass::WrongPlanBranch)
                .expect("wrong plan branch score");
            assert_eq!(wrong_plan.confidence, Confidence::Low, "{case}");
            assert!(!wrong_plan.flagged, "{case}");
        }
    }
}

#[test]
fn wrong_plan_branch_resolves_typed_paths_only_through_trusted_roots() {
    let cases = [
        (
            "nested-workdir",
            Some("/repo"),
            "bar.rs",
            Some(("workdir", "/repo/src/foo")),
            vec!["/repo/src/foo/bar.rs".to_string()],
        ),
        (
            "nested-cwd",
            Some("/repo"),
            "bar.rs",
            Some(("cwd", "/repo/src/foo")),
            vec!["/repo/src/foo/bar.rs".to_string()],
        ),
        (
            "absolute-in-root",
            Some("/repo"),
            "/repo/src/foo/bar.rs",
            Some(("workdir", "/repo/src/foo")),
            vec!["/repo/src/foo/bar.rs".to_string()],
        ),
        (
            "nested-dot-components",
            Some("/repo"),
            "tmp/../bar.rs",
            Some(("workdir", "/repo/src/foo")),
            vec!["/repo/src/foo/bar.rs".to_string()],
        ),
        (
            "windows-nested-workdir",
            Some(r"C:\repo"),
            "bar.rs",
            Some(("workdir", r"C:\repo\src\foo")),
            vec!["c:/repo/src/foo/bar.rs".to_string()],
        ),
        (
            "external-workdir-relative-path",
            Some("/repo"),
            "src/foo/bar.rs",
            Some(("workdir", "/tmp")),
            Vec::new(),
        ),
        (
            "external-matching-scope",
            Some("/repo"),
            "/tmp/src/foo/bar.rs",
            Some(("workdir", "/tmp")),
            Vec::new(),
        ),
        (
            "inside-path-external-workdir",
            Some("/repo"),
            "/repo/src/foo/bar.rs",
            Some(("workdir", "/tmp")),
            Vec::new(),
        ),
        (
            "mismatched-drive",
            Some(r"C:\repo"),
            r"D:\repo\src\foo\bar.rs",
            Some(("workdir", r"C:\repo")),
            Vec::new(),
        ),
        (
            "typed-unc",
            Some(r"C:\repo"),
            r"\\server\share\src\foo\bar.rs",
            Some(("workdir", r"C:\repo")),
            Vec::new(),
        ),
        (
            "typed-rooted-backslash",
            Some("/repo"),
            r"\repo\src\foo\bar.rs",
            Some(("workdir", "/repo")),
            Vec::new(),
        ),
        (
            "untrusted-workdir-cannot-anchor-relative-path",
            None,
            "src/foo/bar.rs",
            Some(("workdir", "/repo")),
            Vec::new(),
        ),
        (
            "repo-relative-without-cwd",
            None,
            "src/foo/bar.rs",
            None,
            vec!["src/foo/bar.rs".to_string()],
        ),
        (
            "traversal-escape",
            Some("/repo"),
            "../../../outside.rs",
            Some(("workdir", "/repo/src/foo")),
            Vec::new(),
        ),
        (
            "path-escape-and-reenter",
            Some("/repo"),
            "../../../repo/secret.rs",
            Some(("workdir", "/repo/src/foo")),
            Vec::new(),
        ),
        (
            "absolute-path-escape-and-reenter",
            Some("/repo"),
            "/repo/../repo/src/foo/bar.rs",
            Some(("workdir", "/repo")),
            Vec::new(),
        ),
        (
            "workdir-escape-and-reenter",
            Some("/repo"),
            "bar.rs",
            Some(("workdir", "/repo/src/../../repo/src/foo")),
            Vec::new(),
        ),
    ];

    for (case, trusted_root, path, cwd, expected_paths) in cases {
        let authority = trusted_root.map_or_else(
            || "Authorized filesystem scope: src/foo.".to_string(),
            |root| {
                format!("Trusted repository root: {root}. Authorized filesystem scope: src/foo.")
            },
        );
        let payload = match cwd {
            None => format!(r#"{{"path":{path:?}}}"#),
            Some(("workdir", cwd)) => {
                format!(r#"{{"path":{path:?},"workdir":{cwd:?}}}"#)
            }
            Some(("cwd", cwd)) => format!(r#"{{"path":{path:?},"cwd":{cwd:?}}}"#),
            Some((field, _)) => panic!("unexpected typed location field: {field}"),
        };
        let result = analyze_rows(vec![
            row(
                0,
                CompactionKind::UserMessage,
                "Implement the requested change.",
            ),
            row(1, CompactionKind::SystemMessage, &authority),
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
fn wrong_plan_branch_projects_absolute_authority_through_its_trusted_root() {
    let cases = [
        (
            "posix-descendant",
            "/repo",
            "/repo/src/foo",
            "/repo/src/foo/bar.rs",
            false,
        ),
        (
            "posix-prefix-collision",
            "/repo",
            "/repo/src/foo",
            "/repo/src/foobar.rs",
            true,
        ),
        (
            "windows-descendant",
            r"C:\repo",
            r"C:\repo\src\foo",
            r"C:\repo\src\foo\bar.rs",
            false,
        ),
        (
            "windows-prefix-collision",
            r"C:\repo",
            r"C:\repo\src\foo",
            r"C:\repo\src\foobar.rs",
            true,
        ),
    ];

    for (case, root, authority, path, flagged) in cases {
        let result = analyze_rows(vec![
            row(
                0,
                CompactionKind::UserMessage,
                "Implement the requested change.",
            ),
            row(
                1,
                CompactionKind::SystemMessage,
                &format!(
                    "Trusted repository root: {root}. Authorized filesystem scope: {authority}."
                ),
            ),
            typed_tool_row(
                2,
                "functions.write_file",
                &format!(r#"{{"path":{path:?},"cwd":{root:?}}}"#),
            ),
        ]);
        let score = final_wrong_plan_score(&result);
        assert_eq!(score.flagged, flagged, "{case}");
        assert_eq!(score.raw_score, if flagged { 60 } else { 0 }, "{case}");
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
                "Trusted repository root: /repo. Authorized filesystem scope: src/foo.",
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
            "Trusted repository root: /repo. Authorized filesystem scope: ./.",
        ),
        typed_tool_row(
            2,
            "functions.write_file",
            r#"{"path":"src/anything.rs","workdir":"/repo"}"#,
        ),
    ]);
    assert_eq!(
        root.sessions[0].context.command_observations[0].paths,
        vec!["/repo/src/anything.rs".to_string()]
    );
    assert!(!final_wrong_plan_score(&root).flagged);
}

#[test]
fn wrong_plan_branch_normalizes_backslash_relative_authority_paths() {
    let cases = [
        ("authority-exact", r"src\foo", "src/foo", "src/foo", false),
        (
            "authority-descendant",
            r"src\foo",
            "src/foo",
            "src/foo/bar.rs",
            false,
        ),
        (
            "authority-sibling",
            r"src\foo",
            "src/foo",
            "src/bar.rs",
            true,
        ),
        (
            "authority-prefix-collision",
            r"src\foo",
            "src/foo",
            "src/foobar.rs",
            true,
        ),
        (
            "authority-dot-and-repeated-separators",
            r"src\.\foo\\",
            "src/foo",
            "src/foo/bar.rs",
            false,
        ),
        ("authority-root", r".\.", ".", "src/anything.rs", false),
    ];

    for (case, authority, normalized_authority, action_path, flagged) in cases {
        let payload = format!(r#"{{"path":{action_path:?},"workdir":"/repo"}}"#);
        let result = analyze_rows(vec![
            row(
                0,
                CompactionKind::UserMessage,
                "Implement the requested change.",
            ),
            row(
                1,
                CompactionKind::SystemMessage,
                &format!(
                    "Trusted repository root: /repo. Authorized filesystem scope: {authority}"
                ),
            ),
            typed_tool_row(2, "functions.write_file", &payload),
        ]);
        let authoritative = result.sessions[0]
            .context
            .truth_artifacts
            .iter()
            .find(|artifact| artifact.path == normalized_authority)
            .expect("normalized relative authority");
        assert_ne!(authoritative.source, "control_directive_literal", "{case}");
        assert!(!authoritative.evidence.is_empty(), "{case}");

        let score = final_wrong_plan_score(&result);
        assert_eq!(
            (
                score.raw_score,
                score.confidence,
                score.state,
                score.flagged,
            ),
            (
                if flagged { 60 } else { 0 },
                Confidence::Medium,
                if flagged {
                    DriftState::Active
                } else {
                    DriftState::Cleared
                },
                flagged,
            ),
            "{case}",
        );
    }
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
