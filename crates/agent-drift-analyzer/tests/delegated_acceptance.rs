#![allow(unused_crate_dependencies)]

mod support;

use std::collections::BTreeSet;
use std::fs;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, Checkpoint, ChildWorkVisibility, Confidence,
    DelegationTopology, DriftClass, DriftState, ProgressDimension,
};
use agent_session_compactor::{
    BundleManifest, CompactionKind, CompactionRow, DelegationEvidenceRef, DelegationLink,
    DelegationLinkState, SourceKind, UserMessageRole,
};
use camino::Utf8PathBuf;
use serde::Deserialize;
use support::BundleFixture;

const FIXTURE_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/delegated_acceptance"
);
const MATRIX_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/delegated_acceptance/matrix.json"
);
const SPEC_CASE_IDS: [&str; 10] = [
    "one_parent_one_verified_child",
    "one_parent_multiple_verified_children",
    "one_verified_child_plus_one_missing_child",
    "parent_spawn_result_without_child_file",
    "child_metadata_without_parent_result",
    "conflicting_parent_child_ids",
    "nested_depth_greater_than_one_residue",
    "ordinary_single_agent_control",
    "parent_wait_loop_while_child_advances",
    "child_stalls_while_parent_orchestration_clean",
];

#[derive(Debug, Deserialize)]
struct AcceptanceMatrix {
    cases: Vec<AcceptanceCase>,
}

#[derive(Debug, Deserialize)]
struct AcceptanceCase {
    id: String,
    sessions: Vec<SessionFixture>,
    links: Vec<LinkFixture>,
    expected: Vec<ExpectedTrajectory>,
}

#[derive(Debug, Deserialize)]
struct SessionFixture {
    id: String,
    behavior: SessionBehavior,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SessionBehavior {
    ParentClean,
    ParentWaitLoop,
    ChildMinimal,
    ChildAdvances,
    ChildStalls,
    SingleAgentClean,
}

#[derive(Debug, Deserialize)]
struct LinkFixture {
    parent_session_id: String,
    child_session_id: String,
    child_origin_parent_session_id: Option<String>,
    depth: Option<u32>,
    state: DelegationLinkState,
}

#[derive(Debug, Deserialize)]
struct ExpectedTrajectory {
    session_id: String,
    topology: DelegationTopology,
    parent_session_id: Option<String>,
    child_session_ids: Vec<String>,
    visibility: ChildWorkVisibility,
    confidence: Confidence,
    progress_dimension: Option<ProgressDimension>,
    dead_end_thrash: Option<ExpectedScore>,
}

#[derive(Debug, Deserialize)]
struct ExpectedScore {
    state: DriftState,
    flagged: bool,
}

#[test]
fn delegated_acceptance_matrix_covers_every_spec_case() {
    let matrix = load_matrix();
    let actual_case_ids = matrix
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected_case_ids = SPEC_CASE_IDS.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(
        actual_case_ids, expected_case_ids,
        "the committed matrix must cover every R7 delegated acceptance case exactly once"
    );
    assert_eq!(matrix.cases.len(), SPEC_CASE_IDS.len());

    for case in matrix.cases {
        assert_case(&case);
    }
}

#[test]
fn delegated_acceptance_fixtures_are_sanitized() {
    let readme = fs::read_to_string(format!("{FIXTURE_ROOT}/README.md"))
        .expect("read delegated acceptance fixture README");
    let matrix = fs::read_to_string(MATRIX_PATH).expect("read delegated acceptance matrix");
    let committed_fixture_text = format!("{readme}\n{matrix}");

    for forbidden in [
        "/Users/",
        ".codex/sessions",
        "rollout-2026",
        "019e93f8",
        "019e93fa",
        "agent_id\":\"019",
        "Bearer ",
    ] {
        assert!(
            !committed_fixture_text.contains(forbidden),
            "sanitized delegated acceptance fixtures must not contain {forbidden:?}"
        );
    }
}

fn assert_case(case: &AcceptanceCase) {
    let fixture_session_ids = case
        .sessions
        .iter()
        .map(|session| session.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected_session_ids = case
        .expected
        .iter()
        .map(|expected| expected.session_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected_session_ids, fixture_session_ids,
        "{} must assert every included trajectory exactly once",
        case.id
    );

    let rows = case
        .sessions
        .iter()
        .flat_map(session_rows)
        .collect::<Vec<_>>();
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
    let links = case.links.iter().map(build_link).collect::<Vec<_>>();
    let manifest_path = fixture.input_dir.join("manifest.json");
    let mut manifest: BundleManifest = serde_json::from_str(
        &fs::read_to_string(&manifest_path).expect("read delegated acceptance manifest"),
    )
    .expect("parse delegated acceptance manifest");
    manifest.delegation_links = links;
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize delegated acceptance manifest"),
    )
    .expect("write delegated acceptance manifest");

    let loaded = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .unwrap_or_else(|error| panic!("load {} delegated acceptance bundle: {error}", case.id));
    assert_eq!(
        loaded
            .manifest
            .delegation_links
            .iter()
            .map(|link| link.state)
            .collect::<Vec<_>>(),
        case.links.iter().map(|link| link.state).collect::<Vec<_>>(),
        "{} must preserve every typed input link state",
        case.id
    );
    assert_non_verified_links_fail_closed(case, &loaded);

    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .unwrap_or_else(|error| panic!("analyze {} delegated acceptance bundle: {error}", case.id));

    assert_eq!(
        result.sessions.len(),
        case.sessions.len(),
        "{} must not invent or drop a trajectory",
        case.id
    );
    for expected in &case.expected {
        let checkpoint = final_checkpoint(&result, &expected.session_id);
        assert_eq!(
            (
                checkpoint.delegation.topology,
                checkpoint.delegation.parent_session_id.as_deref(),
                checkpoint.delegation.child_session_ids.as_slice(),
                checkpoint.delegation.child_work_visibility,
                checkpoint.delegation.confidence,
            ),
            (
                expected.topology,
                expected.parent_session_id.as_deref(),
                expected.child_session_ids.as_slice(),
                expected.visibility,
                expected.confidence,
            ),
            "{} delegation context mismatch for {}",
            case.id,
            expected.session_id
        );
        assert_progress_ownership(checkpoint, &expected.session_id);
        assert_scorer_ownership(checkpoint, &expected.session_id);

        if let Some(expected_dimension) = expected.progress_dimension {
            assert_eq!(
                checkpoint
                    .session_progress
                    .as_ref()
                    .expect("session progress")
                    .dimension,
                expected_dimension,
                "{} progress dimension mismatch for {}",
                case.id,
                expected.session_id
            );
        }
        if let Some(expected_score) = &expected.dead_end_thrash {
            let score = checkpoint
                .drift_scores
                .iter()
                .find(|score| score.class == DriftClass::DeadEndThrash)
                .expect("dead_end_thrash score");
            assert_eq!(
                (score.state, score.flagged),
                (expected_score.state, expected_score.flagged),
                "{} dead_end_thrash ownership mismatch for {}",
                case.id,
                expected.session_id
            );
        }
    }
}

fn assert_non_verified_links_fail_closed(
    case: &AcceptanceCase,
    loaded: &agent_drift_analyzer::InputBundle,
) {
    for link in case
        .links
        .iter()
        .filter(|link| link.state != DelegationLinkState::Verified)
    {
        let semantic_edge_exists = loaded
            .delegation_graph
            .by_session_id
            .values()
            .flat_map(|session_links| {
                session_links
                    .parent_links
                    .iter()
                    .chain(&session_links.child_links)
            })
            .any(|semantic_link| {
                semantic_link.parent_session_id == link.parent_session_id
                    && semantic_link.child_session_id == link.child_session_id
            });
        assert!(
            !semantic_edge_exists,
            "{} non-verified {:?} link must not enter the semantic graph",
            case.id, link.state
        );
    }
}

fn assert_progress_ownership(checkpoint: &Checkpoint, session_id: &str) {
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");
    let evidence = archetype
        .supporting_evidence
        .iter()
        .chain(&archetype.counter_evidence)
        .chain(&progress.supporting_evidence)
        .chain(&progress.counter_evidence)
        .chain(progress.signals.iter().flat_map(|signal| &signal.evidence));

    assert!(
        checkpoint
            .boundary
            .start
            .source_file
            .as_str()
            .contains(session_id)
            && checkpoint
                .boundary
                .end
                .source_file
                .as_str()
                .contains(session_id),
        "checkpoint boundary must remain owned by {session_id}"
    );
    assert!(
        evidence
            .into_iter()
            .all(|item| item.row.source_file.as_str().contains(session_id)),
        "archetype/progress evidence must remain owned by {session_id}"
    );
}

fn assert_scorer_ownership(checkpoint: &Checkpoint, session_id: &str) {
    assert!(
        checkpoint
            .drift_scores
            .iter()
            .all(|score| score.evidence.iter().all(|item| item
                .row
                .source_file
                .as_str()
                .contains(session_id))),
        "every scorer evidence row must remain owned by {session_id}"
    );
}

fn load_matrix() -> AcceptanceMatrix {
    serde_json::from_str(
        &fs::read_to_string(MATRIX_PATH).expect("read delegated acceptance matrix"),
    )
    .expect("parse delegated acceptance matrix")
}

fn build_link(link: &LinkFixture) -> DelegationLink {
    let parent_evidence = (link.state != DelegationLinkState::ChildOnly)
        .then(|| delegation_evidence(&link.parent_session_id, 1))
        .into_iter()
        .collect();
    let child_evidence = (link.state != DelegationLinkState::ParentOnly)
        .then(|| delegation_evidence(&link.child_session_id, 0))
        .into_iter()
        .collect();

    DelegationLink {
        parent_session_id: link.parent_session_id.clone(),
        child_session_id: link.child_session_id.clone(),
        child_origin_parent_session_id: link.child_origin_parent_session_id.clone(),
        depth: link.depth,
        state: link.state,
        parent_evidence,
        child_evidence,
    }
}

fn session_rows(session: &SessionFixture) -> Vec<CompactionRow> {
    match session.behavior {
        SessionBehavior::ParentClean => vec![
            row(
                &session.id,
                0,
                CompactionKind::UserMessage,
                "/goal Coordinate only the bounded delegated acceptance task.",
                None,
            ),
            row(
                &session.id,
                1,
                CompactionKind::ToolCall,
                r#"{"task_name":"sanitized_child","message":"Implement and verify only the bounded child target."}"#,
                Some("spawn_agent"),
            ),
        ],
        SessionBehavior::ParentWaitLoop => vec![
            row(
                &session.id,
                0,
                CompactionKind::UserMessage,
                "/goal Coordinate only the bounded delegated acceptance task.",
                None,
            ),
            row(
                &session.id,
                1,
                CompactionKind::ToolCall,
                r#"{"task_name":"sanitized_child","message":"Implement and verify only the bounded child target."}"#,
                Some("spawn_agent"),
            ),
            row(
                &session.id,
                2,
                CompactionKind::ToolCall,
                r#"{"session_id":"sanitized-child"}"#,
                Some("wait_agent"),
            ),
            row(
                &session.id,
                3,
                CompactionKind::ToolCall,
                r#"{"session_id":"sanitized-child"}"#,
                Some("wait_agent"),
            ),
        ],
        SessionBehavior::ChildMinimal => vec![
            row(
                &session.id,
                0,
                CompactionKind::UserMessage,
                "/goal Implement only the bounded child target.",
                None,
            ),
            row(
                &session.id,
                1,
                CompactionKind::AssistantMessage,
                "The bounded child target remains session-local.",
                None,
            ),
        ],
        SessionBehavior::ChildAdvances => vec![
            row(
                &session.id,
                0,
                CompactionKind::UserMessage,
                "/goal Implement and verify only child_target.",
                None,
            ),
            tool_row(&session.id, 1),
            row(
                &session.id,
                2,
                CompactionKind::ToolOutput,
                "Exit code: 101\nrunning 1 test\ntest child_target ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed",
                None,
            ),
            row(
                &session.id,
                3,
                CompactionKind::ToolCall,
                r#"{"patch":"Update File: crates/child-target/src/lib.rs"}"#,
                Some("apply_patch"),
            ),
            tool_row(&session.id, 4),
            row(
                &session.id,
                5,
                CompactionKind::ToolOutput,
                "Exit code: 0\nrunning 1 test\ntest child_target ... ok\n\ntest result: ok. 1 passed; 0 failed",
                None,
            ),
        ],
        SessionBehavior::ChildStalls => {
            let mut first_failure = row(
                &session.id,
                2,
                CompactionKind::Error,
                "child target failed",
                None,
            );
            let mut second_failure = row(
                &session.id,
                5,
                CompactionKind::Error,
                "child target failed",
                None,
            );
            first_failure.text_hash_hex = format!("hash-{}-repeated-failure", session.id);
            second_failure.text_hash_hex = first_failure.text_hash_hex.clone();
            vec![
                row(
                    &session.id,
                    0,
                    CompactionKind::UserMessage,
                    "/goal Troubleshoot only child_target without widening scope.",
                    None,
                ),
                tool_row(&session.id, 1),
                first_failure,
                row(
                    &session.id,
                    3,
                    CompactionKind::UserMessage,
                    "/goal Re-run the same child_target verifier without widening scope.",
                    None,
                ),
                tool_row(&session.id, 4),
                second_failure,
            ]
        }
        SessionBehavior::SingleAgentClean => vec![
            row(
                &session.id,
                0,
                CompactionKind::UserMessage,
                "/goal Implement and verify only single_target.",
                None,
            ),
            row(
                &session.id,
                1,
                CompactionKind::ToolCall,
                r#"{"command":"cargo test -p single-target single_target -- --exact","workdir":"/repo"}"#,
                Some("functions.shell_command"),
            ),
            row(
                &session.id,
                2,
                CompactionKind::ToolOutput,
                "Exit code: 0\nrunning 1 test\ntest single_target ... ok\n\ntest result: ok. 1 passed; 0 failed",
                None,
            ),
        ],
    }
}

fn tool_row(session_id: &str, event_index: usize) -> CompactionRow {
    row(
        session_id,
        event_index,
        CompactionKind::ToolCall,
        r#"{"command":"cargo test -p child-target child_target -- --exact","workdir":"/repo"}"#,
        Some("functions.shell_command"),
    )
}

fn row(
    session_id: &str,
    event_index: usize,
    kind: CompactionKind,
    text: &str,
    tool_name: Option<&str>,
) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from(format!(
            "/tmp/r7-5-delegated-acceptance/{session_id}/rollout.jsonl"
        )),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some(session_id.to_string()),
        turn_id: Some("turn-sanitized".to_string()),
        event_index,
        line_number: event_index + 1,
        row_ordinal: 0,
        timestamp: None,
        kind,
        user_message_role: matches!(kind, CompactionKind::UserMessage)
            .then_some(UserMessageRole::Prompt),
        dedupe_identity: tool_name.map(|name| {
            format!(
                "{{\"call_id\":\"call-{session_id}-{event_index}\",\"name\":\"{name}\",\"type\":\"function_call\"}}"
            )
        }),
        text: text.to_string(),
        canonical_text: text.to_string(),
        text_hash_hex: format!("hash-{session_id}-{event_index}"),
    }
}

fn delegation_evidence(session_id: &str, event_index: usize) -> DelegationEvidenceRef {
    DelegationEvidenceRef {
        source_file: Utf8PathBuf::from(format!(
            "/tmp/r7-5-delegated-acceptance/{session_id}/rollout.jsonl"
        )),
        line_number: event_index + 1,
        event_index,
    }
}

fn final_checkpoint<'a>(
    result: &'a agent_drift_analyzer::AnalyzeResult,
    session_id: &str,
) -> &'a Checkpoint {
    result
        .sessions
        .iter()
        .find(|session| session.session_id == session_id)
        .and_then(|session| session.checkpoints.last())
        .unwrap_or_else(|| panic!("final checkpoint for {session_id}"))
}
