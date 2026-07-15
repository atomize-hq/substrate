#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, Checkpoint, Confidence, DelegationTopology, DriftClass,
    DriftScore, DriftState, ProgressDimension, ProgressSignalCode, ProgressStatus,
    SessionArchetype, SessionArchetypeLabel, SessionProgress,
};
use agent_session_compactor::{
    BundleManifest, CompactionKind, CompactionRow, DelegationEvidenceRef, DelegationLink,
    DelegationLinkState, SourceKind, UserMessageRole,
};
use camino::{Utf8Path, Utf8PathBuf};
use support::BundleFixture;
use tempfile::TempDir;

const LINKED_PROGRESS_PARENT: &str = "r7-3-1-parent";
const LINKED_PROGRESS_CHILD: &str = "r7-3-1-child";

const NATIVE_PROGRESS_ACCEPTANCE_CASE_IDS: [&str; 9] = [
    "019e899c-453f-71f2-a99d-155848c7b081",
    "019e8b42-42bd-7b10-baae-3265edb65f4b",
    "019e940c-a91b-7fe0-a967-b0bdd595b581",
    "019eb311-c7ce-7f50-ae13-b51a5b5461c3",
    "019eb970-3543-7ab1-a5d6-2a62c00c7185",
    "019f1ecb-b93a-7570-8d8d-9ce4e711880b",
    "real-closeout-conservative-019e767c-ord3",
    "real-implementation-advancing-019e894a-ord6",
    "real-reopen-regressing-019e894a-ord7",
];

const ADAPTED_EXTERNAL_PROGRESS_CASE_IDS: [&str; 3] = [
    "adapted-sparse-readable-f47b81f39f2495dd",
    "adapted-zero-verifier-097d97e914ca220f",
    "adapted-parent-visible-da59436e63915185",
];

const SYNTHETIC_PROGRESS_ACCEPTANCE_CASE_IDS: [&str; 4] = [
    "synthetic-implementation-advancing",
    "synthetic-parent-visible-opaque",
    "synthetic-zero-verifier-anti-flap",
    "synthetic-planning-advancing",
];

const PROGRESS_ACCEPTANCE_CASE_IDS: [&str; 16] = [
    "019e899c-453f-71f2-a99d-155848c7b081",
    "019e8b42-42bd-7b10-baae-3265edb65f4b",
    "019e940c-a91b-7fe0-a967-b0bdd595b581",
    "019eb311-c7ce-7f50-ae13-b51a5b5461c3",
    "019eb970-3543-7ab1-a5d6-2a62c00c7185",
    "019f1ecb-b93a-7570-8d8d-9ce4e711880b",
    "real-closeout-conservative-019e767c-ord3",
    "real-implementation-advancing-019e894a-ord6",
    "real-reopen-regressing-019e894a-ord7",
    "adapted-sparse-readable-f47b81f39f2495dd",
    "adapted-zero-verifier-097d97e914ca220f",
    "adapted-parent-visible-da59436e63915185",
    "synthetic-implementation-advancing",
    "synthetic-parent-visible-opaque",
    "synthetic-zero-verifier-anti-flap",
    "synthetic-planning-advancing",
];

const PROGRESS_ACCEPTANCE_EXCLUDED_CASES: [(&str, &str); 3] = [
    (
        "019e93f8-a5e9-7490-ac1a-955b74c92ad0",
        "delegated live bundle did not deterministically surface parent_visible_orchestration",
    ),
    (
        "019e9406-6736-79a2-946b-8a603e557422",
        "delegated live bundle did not deterministically surface parent_visible_orchestration",
    ),
    (
        "019e9401-9d69-7190-a43e-9ee3be08b369",
        "review-only bundle duplicated covered planning/closeout semantics without adding a new core dimension",
    ),
];

const PROGRESS_FIXTURE_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/progress_acceptance"
);

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum FixtureKind {
    AnnotatedRealRollout,
    AnnotatedAdaptedExternal,
    SyntheticBundleShaped,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ChildVisibility {
    NotApplicable,
    Opaque,
    Visible,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FixtureScreening {
    delegated: bool,
    child_visibility: ChildVisibility,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SelectedCheckpointExpected {
    ordinal: usize,
    archetype: SessionArchetypeLabel,
    dimension: ProgressDimension,
    status: ProgressStatus,
    confidence_min: Confidence,
    confidence_max: Confidence,
    required_signal_codes: Vec<ProgressSignalCode>,
    forbidden_signal_codes: Vec<ProgressSignalCode>,
    supporting_evidence_min: usize,
    counter_evidence_min: usize,
    decisive_evidence: Vec<String>,
    counter_evidence: Vec<String>,
    why_not_other_dimensions: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProgressAcceptanceExpected {
    case_id: String,
    fixture_kind: FixtureKind,
    #[serde(default)]
    source_rollout_id: Option<String>,
    source_artifact: String,
    notes: String,
    #[serde(default)]
    screening: Option<FixtureScreening>,
    selected_checkpoint: SelectedCheckpointExpected,
}

struct ProgressAcceptanceFixture {
    _temp_dir: TempDir,
    input_dir: Utf8PathBuf,
    output_dir: Utf8PathBuf,
    expected: ProgressAcceptanceExpected,
}

impl ProgressAcceptanceFixture {
    fn load(case_id: &str) -> Self {
        let input_dir = progress_fixture_dir(case_id);
        let expected = read_json(input_dir.join("expected.json").as_ref());
        let temp_dir = TempDir::new().expect("temp dir");
        let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
        let output_dir = root.join("output");
        fs::create_dir_all(&output_dir).expect("create output dir");

        Self {
            _temp_dir: temp_dir,
            input_dir,
            output_dir,
            expected,
        }
    }

    fn request(&self) -> AnalyzeRequest {
        AnalyzeRequest {
            input_dir: self.input_dir.clone(),
            output_dir: self.output_dir.clone(),
        }
    }

    fn analyze(&self) -> agent_drift_analyzer::AnalyzeResult {
        analyze_bundle(&self.request()).expect("analyze progress acceptance case")
    }
}

#[test]
fn progress_acceptance_corpus_stays_bounded_and_contains_real_rollout_proof() {
    let fixture_root = std::path::Path::new(PROGRESS_FIXTURE_ROOT);
    let mut expected_root_entries = PROGRESS_ACCEPTANCE_CASE_IDS
        .iter()
        .map(|case_id| (*case_id).to_owned())
        .collect::<Vec<_>>();
    expected_root_entries.push("README.md".to_owned());
    expected_root_entries.sort();
    assert_eq!(
        sorted_entry_names(fixture_root),
        expected_root_entries,
        "Packet R5-7 progress corpus must stay bounded to README.md plus the approved case directories"
    );

    let expected_case_entries = [
        "dedupe-audit.jsonl",
        "expected.json",
        "manifest.json",
        "rows.archival.jsonl",
        "rows.compact.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    let mut annotated_real_rollout_count = 0usize;
    let mut annotated_adapted_external_count = 0usize;
    let mut synthetic_bundle_shaped_count = 0usize;
    for case_id in PROGRESS_ACCEPTANCE_CASE_IDS {
        let case = ProgressAcceptanceFixture::load(case_id);
        assert_eq!(case.expected.case_id, case_id);
        assert_eq!(
            sorted_entry_names(&fixture_root.join(case_id)),
            expected_case_entries,
            "progress acceptance case {case_id} must contain exactly the committed Packet R5-7 fixture contract files"
        );
        match case.expected.fixture_kind {
            FixtureKind::AnnotatedRealRollout => {
                annotated_real_rollout_count += 1;
                let expected_json: serde_json::Value =
                    read_json(case.input_dir.join("expected.json").as_ref());
                assert!(
                    case.expected.source_rollout_id.is_some(),
                    "annotated real-rollout case {case_id} must record source_rollout_id"
                );
                let screening = case.expected.screening.as_ref().unwrap_or_else(|| {
                    panic!("annotated real-rollout case {case_id} must record screening metadata")
                });
                assert!(
                    !screening.delegated || screening.child_visibility != ChildVisibility::NotApplicable,
                    "delegated real-rollout case {case_id} must not use not_applicable child_visibility"
                );
                assert_selected_checkpoint_has_array_field(
                    &expected_json,
                    case_id,
                    "required_signal_codes",
                );
                assert_selected_checkpoint_has_array_field(
                    &expected_json,
                    case_id,
                    "forbidden_signal_codes",
                );
                assert_selected_checkpoint_has_array_field(
                    &expected_json,
                    case_id,
                    "counter_evidence",
                );
                assert!(
                    !case
                        .expected
                        .selected_checkpoint
                        .decisive_evidence
                        .is_empty(),
                    "annotated real-rollout case {case_id} must record decisive_evidence"
                );
                assert!(
                    !case
                        .expected
                        .selected_checkpoint
                        .why_not_other_dimensions
                        .is_empty(),
                    "annotated real-rollout case {case_id} must explain why alternative dimensions were not chosen"
                );
            }
            FixtureKind::AnnotatedAdaptedExternal => {
                annotated_adapted_external_count += 1;
            }
            FixtureKind::SyntheticBundleShaped => {
                synthetic_bundle_shaped_count += 1;
            }
        }
    }

    assert_eq!(
        annotated_real_rollout_count,
        NATIVE_PROGRESS_ACCEPTANCE_CASE_IDS.len(),
        "Packet R5-7 progress corpus must keep the committed shape of exactly {} annotated real-rollout cases",
        NATIVE_PROGRESS_ACCEPTANCE_CASE_IDS.len()
    );
    assert!(
        annotated_real_rollout_count >= 1,
        "Packet R5-7 progress corpus must keep at least one annotated real-rollout case as the primary semantic anchor"
    );
    assert_eq!(
        annotated_adapted_external_count,
        ADAPTED_EXTERNAL_PROGRESS_CASE_IDS.len(),
        "Packet R5.75-5 adapted external progress lane must keep the committed shape of exactly {} secondary robustness cases",
        ADAPTED_EXTERNAL_PROGRESS_CASE_IDS.len()
    );
    assert_eq!(
        synthetic_bundle_shaped_count,
        SYNTHETIC_PROGRESS_ACCEPTANCE_CASE_IDS.len(),
        "Packet R5-7 progress corpus must keep the committed shape of exactly {} synthetic bundle-shaped support cases",
        SYNTHETIC_PROGRESS_ACCEPTANCE_CASE_IDS.len()
    );

    for (excluded_case_id, reason) in PROGRESS_ACCEPTANCE_EXCLUDED_CASES {
        let excluded_dir = fixture_root.join(excluded_case_id);
        assert!(
            !excluded_dir.exists(),
            "excluded progress case {excluded_case_id} must stay out of the bounded semantic corpus: {reason}"
        );
    }
}

#[test]
fn progress_acceptance_cases_match_expected_progress_contract() {
    for case_id in PROGRESS_ACCEPTANCE_CASE_IDS {
        assert_progress_case(case_id);
    }
}

#[test]
fn linked_parent_and_child_keep_trajectory_local_progress() {
    let fixture = linked_progress_fixture();
    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze linked progress fixture");

    let parent = final_checkpoint_for_session(&result, LINKED_PROGRESS_PARENT);
    let child = final_checkpoint_for_session(&result, LINKED_PROGRESS_CHILD);
    let child_archetype = child
        .session_archetype
        .as_ref()
        .expect("linked child archetype");
    let parent_progress = parent
        .session_progress
        .as_ref()
        .expect("linked parent progress");
    let child_progress = child
        .session_progress
        .as_ref()
        .expect("linked child progress");

    assert_eq!(
        (
            parent.delegation.topology,
            parent.delegation.child_session_ids.as_slice(),
            parent_progress.dimension,
            parent_progress.status,
        ),
        (
            DelegationTopology::DelegatingParent,
            [LINKED_PROGRESS_CHILD.to_string()].as_slice(),
            ProgressDimension::ParentVisibleOrchestration,
            ProgressStatus::InsufficientEvidence,
        ),
        "the parent must reference the child while retaining parent-visible progress"
    );
    assert_eq!(
        (
            child.delegation.topology,
            child.delegation.parent_session_id.as_deref(),
            child.delegation.child_session_ids.as_slice(),
            child_archetype.label,
            child_progress.dimension,
            child_progress.status,
        ),
        (
            DelegationTopology::DelegatedChild,
            Some(LINKED_PROGRESS_PARENT),
            [].as_slice(),
            SessionArchetypeLabel::VerificationCloseout,
            ProgressDimension::VerificationCloseoutNarrowing,
            ProgressStatus::Mixed,
        ),
        "the child must retain its own fail/edit/clean archetype and progress"
    );
    assert_checkpoint_evidence_stays_session_local(parent, LINKED_PROGRESS_PARENT);
    assert_checkpoint_evidence_stays_session_local(child, LINKED_PROGRESS_CHILD);
}

fn linked_progress_fixture() -> BundleFixture {
    let rows = linked_parent_rows()
        .into_iter()
        .chain(linked_child_rows())
        .collect();
    let fixture = BundleFixture::from_compact_rows(rows);
    let manifest_path = fixture.input_dir.join("manifest.json");
    let mut manifest: BundleManifest = read_json(manifest_path.as_ref());
    manifest.delegation_links = vec![DelegationLink {
        parent_session_id: LINKED_PROGRESS_PARENT.to_string(),
        child_session_id: LINKED_PROGRESS_CHILD.to_string(),
        child_origin_parent_session_id: Some(LINKED_PROGRESS_PARENT.to_string()),
        depth: Some(1),
        state: DelegationLinkState::Verified,
        parent_evidence: vec![delegation_evidence(LINKED_PROGRESS_PARENT, 1)],
        child_evidence: vec![delegation_evidence(LINKED_PROGRESS_CHILD, 0)],
    }];
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize linked progress manifest"),
    )
    .expect("write linked progress manifest");
    fixture
}

fn linked_parent_rows() -> Vec<CompactionRow> {
    vec![
        progress_row(
            LINKED_PROGRESS_PARENT,
            0,
            CompactionKind::UserMessage,
            "/goal Delegate the bounded child implementation and coordinate its result.",
            None,
        ),
        progress_row(
            LINKED_PROGRESS_PARENT,
            1,
            CompactionKind::ToolCall,
            r#"{"task_name":"r7_3_1_child","message":"Implement and verify the bounded child target."}"#,
            Some("spawn_agent"),
        ),
    ]
}

fn linked_child_rows() -> Vec<CompactionRow> {
    vec![
        progress_row(
            LINKED_PROGRESS_CHILD,
            0,
            CompactionKind::UserMessage,
            "/goal Implement only crates/child-target/src/lib.rs and verify child_target.",
            None,
        ),
        progress_row(
            LINKED_PROGRESS_CHILD,
            1,
            CompactionKind::ToolCall,
            r#"{"command":"cargo test -p child-target child_target -- --exact","workdir":"/repo"}"#,
            Some("functions.shell_command"),
        ),
        progress_row(
            LINKED_PROGRESS_CHILD,
            2,
            CompactionKind::ToolOutput,
            "Exit code: 101\nrunning 1 test\ntest child_target ... FAILED\n\nfailures:\n    child_target\n\nthread 'child_target' panicked at crates/child-target/src/lib.rs:10:5:\nassertion failed: child target\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            None,
        ),
        progress_row(
            LINKED_PROGRESS_CHILD,
            3,
            CompactionKind::ToolCall,
            r#"{"command":"apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/child-target/src/lib.rs\n*** End Patch\nPATCH","workdir":"/repo"}"#,
            Some("functions.apply_patch"),
        ),
        progress_row(
            LINKED_PROGRESS_CHILD,
            4,
            CompactionKind::ToolCall,
            r#"{"command":"cargo test -p child-target child_target -- --exact","workdir":"/repo"}"#,
            Some("functions.shell_command"),
        ),
        progress_row(
            LINKED_PROGRESS_CHILD,
            5,
            CompactionKind::ToolOutput,
            "Exit code: 0\nrunning 1 test\ntest child_target ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
            None,
        ),
    ]
}

fn progress_row(
    session_id: &str,
    event_index: usize,
    kind: CompactionKind,
    text: &str,
    tool_name: Option<&str>,
) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from(format!("/tmp/{session_id}/rollout.jsonl")),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some(session_id.to_string()),
        turn_id: Some("turn-001".to_string()),
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
        source_file: Utf8PathBuf::from(format!("/tmp/{session_id}/rollout.jsonl")),
        line_number: event_index + 1,
        event_index,
    }
}

fn final_checkpoint_for_session<'a>(
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

fn assert_checkpoint_evidence_stays_session_local(checkpoint: &Checkpoint, session_id: &str) {
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
        "checkpoint boundary must remain on {session_id}"
    );
    assert!(
        evidence
            .into_iter()
            .all(|item| item.row.source_file.as_str().contains(session_id)),
        "archetype/progress evidence must remain on {session_id}"
    );
}

#[test]
fn progress_acceptance_r5_75_witnesses_preserve_r6_1_1_frontier_boundary() {
    let zero_verifier_case =
        ProgressAcceptanceFixture::load("adapted-zero-verifier-097d97e914ca220f");
    let zero_verifier_result = zero_verifier_case.analyze();
    let zero_verifier_session = zero_verifier_result
        .sessions
        .iter()
        .find(|session| session.session_id == "097d97e914ca220f")
        .expect("expected adapted zero-verifier witness session");
    let zero_verifier_checkpoint = zero_verifier_session
        .checkpoints
        .iter()
        .find(|checkpoint| {
            checkpoint.ordinal == zero_verifier_case.expected.selected_checkpoint.ordinal
        })
        .expect("expected adapted zero-verifier witness checkpoint");
    let zero_verifier_archetype = zero_verifier_checkpoint
        .session_archetype
        .as_ref()
        .expect("zero-verifier witness archetype");
    let zero_verifier_progress = zero_verifier_checkpoint
        .session_progress
        .as_ref()
        .expect("zero-verifier witness progress");
    assert_eq!(
        zero_verifier_archetype.label,
        SessionArchetypeLabel::Planning
    );
    assert_eq!(
        zero_verifier_progress.dimension,
        ProgressDimension::PlanningConvergence
    );
    assert_eq!(zero_verifier_progress.status, ProgressStatus::Stalled);
    assert!(
        !zero_verifier_progress
            .signals
            .iter()
            .any(|signal| signal.code == ProgressSignalCode::FailureFrontierAdvanced),
        "R5.75-4 zero-verifier witness must stay outside troubleshooting-frontier advancement"
    );
    assert!(
        !zero_verifier_progress
            .signals
            .iter()
            .any(|signal| signal.code == ProgressSignalCode::VerificationClean),
        "R5.75-4 zero-verifier witness must not regain verification_clean"
    );
    assert!(
        !semantic_goal_drift_score(zero_verifier_checkpoint).flagged,
        "R5.75-4 zero-verifier witness must stay unflagged while its planning boundary stays intact"
    );
    assert_eq!(
        semantic_goal_drift_score(zero_verifier_checkpoint).state,
        DriftState::Cleared
    );
    assert_eq!(
        semantic_goal_drift_score(zero_verifier_checkpoint).raw_score,
        0
    );

    let adapted_parent_visible_case =
        ProgressAcceptanceFixture::load("adapted-parent-visible-da59436e63915185");
    let adapted_parent_visible_result = adapted_parent_visible_case.analyze();
    let adapted_parent_visible_session = adapted_parent_visible_result
        .sessions
        .iter()
        .find(|session| session.session_id == "da59436e63915185")
        .expect("expected adapted parent-visible witness session");
    let adapted_parent_visible_checkpoint = adapted_parent_visible_session
        .checkpoints
        .iter()
        .find(|checkpoint| {
            checkpoint.ordinal
                == adapted_parent_visible_case
                    .expected
                    .selected_checkpoint
                    .ordinal
        })
        .expect("expected adapted parent-visible witness checkpoint");
    let adapted_parent_visible_progress = adapted_parent_visible_checkpoint
        .session_progress
        .as_ref()
        .expect("adapted parent-visible witness progress");
    assert_eq!(
        adapted_parent_visible_progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
    assert_eq!(
        adapted_parent_visible_progress.status,
        ProgressStatus::Stalled
    );
    assert!(
        !adapted_parent_visible_progress
            .signals
            .iter()
            .any(|signal| signal.code == ProgressSignalCode::FailureFrontierAdvanced),
        "adapted delegated guardrail witness must stay outside troubleshooting-frontier advancement"
    );
    assert!(
        !adapted_parent_visible_progress
            .signals
            .iter()
            .any(|signal| signal.code == ProgressSignalCode::VerificationClean),
        "adapted delegated guardrail witness must not regain verification_clean"
    );
    assert!(
        !semantic_goal_drift_score(adapted_parent_visible_checkpoint).flagged,
        "combined R5.75-3/R5.75-4 delegated witness must stay unflagged"
    );
    assert_eq!(
        semantic_goal_drift_score(adapted_parent_visible_checkpoint).state,
        DriftState::Cleared
    );
    assert_eq!(
        semantic_goal_drift_score(adapted_parent_visible_checkpoint).raw_score,
        0
    );

    let native_parent_visible_case =
        ProgressAcceptanceFixture::load("019eb970-3543-7ab1-a5d6-2a62c00c7185");
    let native_parent_visible_result = native_parent_visible_case.analyze();
    let native_parent_visible_session = native_parent_visible_result
        .sessions
        .iter()
        .find(|session| session.session_id == "019eb970-3543-7ab1-a5d6-2a62c00c7185")
        .expect("expected native parent-visible witness session");
    let native_parent_visible_checkpoint = native_parent_visible_session
        .checkpoints
        .iter()
        .find(|checkpoint| {
            checkpoint.ordinal
                == native_parent_visible_case
                    .expected
                    .selected_checkpoint
                    .ordinal
        })
        .expect("expected native parent-visible witness checkpoint");
    let native_parent_visible_progress = native_parent_visible_checkpoint
        .session_progress
        .as_ref()
        .expect("native parent-visible witness progress");
    assert_eq!(
        native_parent_visible_progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
    assert_eq!(native_parent_visible_progress.status, ProgressStatus::Mixed);
    assert!(
        !semantic_goal_drift_score(native_parent_visible_checkpoint).flagged,
        "native delegated parent-visible witness must stay unflagged"
    );
    assert_eq!(
        semantic_goal_drift_score(native_parent_visible_checkpoint).state,
        DriftState::Cleared
    );
    assert_eq!(
        semantic_goal_drift_score(native_parent_visible_checkpoint).raw_score,
        0
    );
}

fn semantic_goal_drift_score(checkpoint: &Checkpoint) -> &DriftScore {
    checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::SemanticGoalDrift)
        .expect("semantic_goal_drift score")
}

fn assert_progress_case(case_id: &str) {
    let case = ProgressAcceptanceFixture::load(case_id);
    let expected_session_id = case
        .expected
        .source_rollout_id
        .as_deref()
        .unwrap_or(case.expected.case_id.as_str());
    let result = case.analyze();
    let session = result
        .sessions
        .iter()
        .find(|session| session.session_id == expected_session_id)
        .unwrap_or_else(|| panic!("expected session {expected_session_id} in {case_id}"));
    let manifest: serde_json::Value = read_json(case.input_dir.join("manifest.json").as_ref());
    if let Some(expected_source_rollout_id) = case.expected.source_rollout_id.as_deref() {
        let manifest_session_ids = manifest["session_ids"]
            .as_array()
            .unwrap_or_else(|| panic!("manifest session_ids must be an array for {case_id}"))
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>();
        assert!(
            manifest_session_ids.contains(&expected_source_rollout_id),
            "manifest session_ids {:?} must include source_rollout_id {} for {case_id}",
            manifest_session_ids,
            expected_source_rollout_id
        );
    }
    let checkpoint = session
        .checkpoints
        .iter()
        .find(|checkpoint| checkpoint.ordinal == case.expected.selected_checkpoint.ordinal)
        .unwrap_or_else(|| {
            panic!(
                "expected checkpoint ordinal {} for {case_id}",
                case.expected.selected_checkpoint.ordinal
            )
        });
    let archetype = checkpoint
        .session_archetype
        .as_ref()
        .expect("session archetype");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("session progress");

    assert_eq!(
        archetype.label, case.expected.selected_checkpoint.archetype,
        "unexpected archetype for {case_id} checkpoint {}",
        checkpoint.ordinal
    );
    assert_eq!(
        progress.dimension, case.expected.selected_checkpoint.dimension,
        "unexpected progress dimension for {case_id} checkpoint {}",
        checkpoint.ordinal
    );
    assert_eq!(
        progress.status, case.expected.selected_checkpoint.status,
        "unexpected progress status for {case_id} checkpoint {}",
        checkpoint.ordinal
    );

    let actual_confidence = confidence_rank(progress.confidence);
    let minimum_confidence = confidence_rank(case.expected.selected_checkpoint.confidence_min);
    let maximum_confidence = confidence_rank(case.expected.selected_checkpoint.confidence_max);
    assert!(
        actual_confidence >= minimum_confidence && actual_confidence <= maximum_confidence,
        "unexpected confidence for {case_id} checkpoint {}: {:?} not in [{:?}, {:?}]",
        checkpoint.ordinal,
        progress.confidence,
        case.expected.selected_checkpoint.confidence_min,
        case.expected.selected_checkpoint.confidence_max
    );

    for required_signal in &case.expected.selected_checkpoint.required_signal_codes {
        assert!(
            progress
                .signals
                .iter()
                .any(|signal| &signal.code == required_signal),
            "expected progress signal {:?} for {case_id} checkpoint {}; got {:?}",
            required_signal,
            checkpoint.ordinal,
            progress
                .signals
                .iter()
                .map(|signal| signal.code)
                .collect::<Vec<_>>()
        );
    }
    for forbidden_signal in &case.expected.selected_checkpoint.forbidden_signal_codes {
        assert!(
            !progress
                .signals
                .iter()
                .any(|signal| &signal.code == forbidden_signal),
            "unexpected forbidden progress signal {:?} for {case_id} checkpoint {}; got {:?}",
            forbidden_signal,
            checkpoint.ordinal,
            progress
                .signals
                .iter()
                .map(|signal| signal.code)
                .collect::<Vec<_>>()
        );
    }

    assert!(
        progress.supporting_evidence.len()
            >= case.expected.selected_checkpoint.supporting_evidence_min,
        "expected at least {} supporting evidence refs for {case_id} checkpoint {}, got {}",
        case.expected.selected_checkpoint.supporting_evidence_min,
        checkpoint.ordinal,
        progress.supporting_evidence.len()
    );
    assert!(
        progress.counter_evidence.len() >= case.expected.selected_checkpoint.counter_evidence_min,
        "expected at least {} counter evidence refs for {case_id} checkpoint {}, got {}",
        case.expected.selected_checkpoint.counter_evidence_min,
        checkpoint.ordinal,
        progress.counter_evidence.len()
    );

    if case.expected.fixture_kind == FixtureKind::AnnotatedRealRollout {
        assert_selected_checkpoint_narrative_alignment(
            case_id,
            archetype,
            progress,
            &case.expected.selected_checkpoint,
        );
    }
}

fn confidence_rank(confidence: Confidence) -> u8 {
    match confidence {
        Confidence::Low => 0,
        Confidence::Medium => 1,
        Confidence::High => 2,
    }
}

fn progress_fixture_dir(case_id: &str) -> Utf8PathBuf {
    let input_dir = Utf8PathBuf::from(PROGRESS_FIXTURE_ROOT).join(case_id);
    assert!(
        input_dir.is_dir(),
        "missing progress acceptance fixture {case_id}; Packet R5-7 tests must not fall back to target/ or ~/.codex"
    );
    input_dir
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Utf8Path) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read json artifact"))
        .expect("parse json artifact")
}

fn sorted_entry_names(path: &std::path::Path) -> Vec<String> {
    let mut entries = std::fs::read_dir(path)
        .unwrap_or_else(|error| panic!("read fixture directory {}: {error}", path.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| {
                    panic!(
                        "read fixture directory entry in {}: {error}",
                        path.display()
                    )
                })
                .file_name()
                .into_string()
                .unwrap_or_else(|name| {
                    panic!(
                        "fixture directory entry {} must be valid UTF-8",
                        std::path::Path::new(&name).display()
                    )
                })
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn assert_selected_checkpoint_has_array_field(
    expected_json: &serde_json::Value,
    case_id: &str,
    field_name: &str,
) {
    let selected_checkpoint = expected_json.get("selected_checkpoint").unwrap_or_else(|| {
        panic!("expected selected_checkpoint object in expected.json for {case_id}")
    });
    let field_value = selected_checkpoint.get(field_name).unwrap_or_else(|| {
        panic!(
            "annotated real-rollout case {case_id} must carry selected_checkpoint.{field_name} explicitly in expected.json"
        )
    });
    assert!(
        field_value.is_array(),
        "annotated real-rollout case {case_id} must encode selected_checkpoint.{field_name} as an array in expected.json"
    );
}

fn assert_selected_checkpoint_narrative_alignment(
    case_id: &str,
    archetype: &SessionArchetype,
    progress: &SessionProgress,
    expected: &SelectedCheckpointExpected,
) {
    match case_id {
        "real-closeout-conservative-019e767c-ord3" => {
            let decisive_text = normalized_join(&expected.decisive_evidence);
            let counter_text = normalized_join(&expected.counter_evidence);
            let why_not_text = normalized_join(&expected.why_not_other_dimensions);
            let actual_facts = normalized_progress_facts(archetype, progress);

            assert_text_contains_all(
                &decisive_text,
                &["verification closeout", "closeout"],
                case_id,
                "selected_checkpoint.decisive_evidence",
            );
            assert_text_contains_all(
                &actual_facts,
                &["verification closeout", "verification closeout narrowing"],
                case_id,
                "selected checkpoint facts",
            );
            assert!(
                progress.signals.is_empty()
                    && progress.supporting_evidence.is_empty()
                    && progress.counter_evidence.is_empty(),
                "expected {case_id} selected checkpoint to stay sparse so the conservative closeout narrative remains honest"
            );
            assert_text_contains_all(
                &counter_text,
                &["absence", "proof", "narrowing", "reopened"],
                case_id,
                "selected_checkpoint.counter_evidence",
            );
            assert_text_contains_all(
                &actual_facts,
                &["insufficient evidence"],
                case_id,
                "selected checkpoint facts",
            );
            assert_text_contains_all(
                &why_not_text,
                &[
                    "implementation verification wall",
                    "troubleshooting frontier",
                ],
                case_id,
                "selected_checkpoint.why_not_other_dimensions",
            );
            assert_eq!(
                progress.dimension,
                ProgressDimension::VerificationCloseoutNarrowing,
                "{case_id} narrative alignment expects a verification_closeout_narrowing checkpoint"
            );
        }
        "real-implementation-advancing-019e894a-ord6" => {
            let decisive_text = normalized_join(&expected.decisive_evidence);
            let why_not_text = normalized_join(&expected.why_not_other_dimensions);
            let actual_facts = normalized_progress_facts(archetype, progress);

            assert_text_contains_all(
                &actual_facts,
                &[
                    "earlier failing implementation verifier",
                    "later clean implementation verifier",
                    "intervening edit overlapped the active implementation scope before verification moved",
                ],
                case_id,
                "selected checkpoint facts",
            );
            assert_text_contains_all(
                &decisive_text,
                &[
                    "earlier failing implementation verifier",
                    "later clean implementation verifier",
                    "overlapping edit",
                ],
                case_id,
                "selected_checkpoint.decisive_evidence",
            );
            assert_text_contains_all(
                &why_not_text,
                &[
                    "verification closeout narrowing",
                    "troubleshooting frontier",
                ],
                case_id,
                "selected_checkpoint.why_not_other_dimensions",
            );
            assert_eq!(
                progress.dimension,
                ProgressDimension::ImplementationVerificationWall,
                "{case_id} narrative alignment expects an implementation_verification_wall checkpoint"
            );
        }
        "real-reopen-regressing-019e894a-ord7" => {
            let decisive_text = normalized_join(&expected.decisive_evidence);
            let counter_text = normalized_join(&expected.counter_evidence);
            let why_not_text = normalized_join(&expected.why_not_other_dimensions);
            let actual_facts = normalized_progress_facts(archetype, progress);

            assert_text_contains_all(
                &actual_facts,
                &[
                    "previously clean scope broken",
                    "earlier clean verification attempt",
                    "later failing verification attempt",
                ],
                case_id,
                "selected checkpoint facts",
            );
            assert_text_contains_all(
                &decisive_text,
                &["earlier", "clean", "later failing verification attempt"],
                case_id,
                "selected_checkpoint.decisive_evidence",
            );
            assert_text_contains_all(
                &counter_text,
                &[
                    "never established a closeout checkpoint first",
                    "previously clean verifier broke again",
                ],
                case_id,
                "selected_checkpoint.counter_evidence",
            );
            assert_text_contains_all(
                &why_not_text,
                &[
                    "verification closeout",
                    "implementation verification wall",
                    "never established a verification closeout checkpoint before the failing re verification attempt",
                ],
                case_id,
                "selected_checkpoint.why_not_other_dimensions",
            );
            assert_eq!(
                progress.dimension,
                ProgressDimension::TroubleshootingFrontier,
                "{case_id} narrative alignment expects a troubleshooting_frontier checkpoint"
            );
        }
        _ => {}
    }
}

fn normalized_progress_facts(archetype: &SessionArchetype, progress: &SessionProgress) -> String {
    let mut facts = vec![
        normalize_text(&format!("{:?}", archetype.label)),
        normalize_text(&format!("{:?}", progress.dimension)),
        normalize_text(&format!("{:?}", progress.status)),
    ];
    facts.extend(progress.signals.iter().flat_map(|signal| {
        [
            normalize_text(&format!("{:?}", signal.code)),
            normalize_text(&signal.summary),
        ]
    }));
    facts.extend(
        progress
            .supporting_evidence
            .iter()
            .map(|evidence| normalize_text(&evidence.reason)),
    );
    facts.extend(
        progress
            .counter_evidence
            .iter()
            .map(|evidence| normalize_text(&evidence.reason)),
    );
    facts.join(" ")
}

fn normalized_join(items: &[String]) -> String {
    normalize_text(&items.join(" "))
}

fn normalize_text(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut previous_was_lower_or_digit = false;
    for ch in text.chars() {
        if ch.is_ascii_uppercase() {
            if previous_was_lower_or_digit {
                normalized.push(' ');
            }
            normalized.push(ch.to_ascii_lowercase());
            previous_was_lower_or_digit = false;
        } else if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            normalized.push(ch);
            previous_was_lower_or_digit = true;
        } else {
            normalized.push(' ');
            previous_was_lower_or_digit = false;
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn assert_text_contains_all(haystack: &str, needles: &[&str], case_id: &str, field_name: &str) {
    for needle in needles {
        let normalized_needle = normalize_text(needle);
        assert!(
            haystack.contains(&normalized_needle),
            "{case_id} {field_name} must contain stable keyword/substrings for {:?}; got {:?}",
            needle,
            haystack
        );
    }
}
