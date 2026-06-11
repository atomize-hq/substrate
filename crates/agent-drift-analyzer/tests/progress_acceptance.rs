#![allow(unused_crate_dependencies)]

use std::fs;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, Confidence, ProgressDimension, ProgressSignalCode,
    ProgressStatus, SessionArchetypeLabel,
};
use camino::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;

const PROGRESS_ACCEPTANCE_CASE_IDS: [&str; 6] = [
    "019e899c-453f-71f2-a99d-155848c7b081",
    "019e8b42-42bd-7b10-baae-3265edb65f4b",
    "019e940c-a91b-7fe0-a967-b0bdd595b581",
    "synthetic-implementation-advancing",
    "synthetic-parent-visible-opaque",
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
    SyntheticBundleShaped,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
struct SelectedCheckpointExpected {
    ordinal: usize,
    archetype: SessionArchetypeLabel,
    dimension: ProgressDimension,
    status: ProgressStatus,
    confidence_min: Confidence,
    confidence_max: Confidence,
    #[serde(default)]
    required_signal_codes: Vec<ProgressSignalCode>,
    supporting_evidence_min: usize,
    counter_evidence_min: usize,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
struct ProgressAcceptanceExpected {
    case_id: String,
    fixture_kind: FixtureKind,
    source_artifact: String,
    notes: String,
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
    let mut synthetic_bundle_shaped_count = 0usize;
    for case_id in PROGRESS_ACCEPTANCE_CASE_IDS {
        let case = ProgressAcceptanceFixture::load(case_id);
        assert_eq!(case.expected.case_id, case_id);
        assert_eq!(
            sorted_entry_names(&fixture_root.join(case_id)),
            expected_case_entries,
            "progress acceptance case {case_id} must contain exactly the committed Packet R5-7 fixture contract files"
        );
        if case.expected.fixture_kind == FixtureKind::AnnotatedRealRollout {
            annotated_real_rollout_count += 1;
        } else {
            synthetic_bundle_shaped_count += 1;
        }
    }

    assert_eq!(
        annotated_real_rollout_count, 3,
        "Packet R5-7 must keep the locked corpus shape of exactly 3 annotated real-rollout cases"
    );
    assert_eq!(
        synthetic_bundle_shaped_count, 3,
        "Packet R5-7 must keep the locked corpus shape of exactly 3 synthetic bundle-shaped support cases"
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

fn assert_progress_case(case_id: &str) {
    let case = ProgressAcceptanceFixture::load(case_id);
    let result = case.analyze();
    let session = result
        .sessions
        .iter()
        .find(|session| session.session_id == case.expected.case_id)
        .unwrap_or_else(|| panic!("expected session {} in {case_id}", case.expected.case_id));
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
