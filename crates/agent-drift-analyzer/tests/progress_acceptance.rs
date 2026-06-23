#![allow(unused_crate_dependencies)]

use std::fs;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, Confidence, ProgressDimension, ProgressSignalCode,
    ProgressStatus, SessionArchetype, SessionArchetypeLabel, SessionProgress,
};
use camino::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;

const PROGRESS_ACCEPTANCE_CASE_IDS: [&str; 10] = [
    "019e899c-453f-71f2-a99d-155848c7b081",
    "019e8b42-42bd-7b10-baae-3265edb65f4b",
    "019e940c-a91b-7fe0-a967-b0bdd595b581",
    "019eb970-3543-7ab1-a5d6-2a62c00c7185",
    "real-closeout-conservative-019e767c-ord3",
    "real-implementation-advancing-019e894a-ord6",
    "real-reopen-regressing-019e894a-ord7",
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
            assert_selected_checkpoint_has_array_field(&expected_json, case_id, "counter_evidence");
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
        } else {
            synthetic_bundle_shaped_count += 1;
        }
    }

    assert_eq!(
        annotated_real_rollout_count, 7,
        "Packet R5-7 progress corpus must keep the committed shape of exactly 7 annotated real-rollout cases"
    );
    assert_eq!(
        synthetic_bundle_shaped_count, 3,
        "Packet R5-7 progress corpus must keep the committed shape of exactly 3 synthetic bundle-shaped support cases"
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
                    "earlier later stage verification attempt",
                    "later regressing verification attempt",
                ],
                case_id,
                "selected checkpoint facts",
            );
            assert_text_contains_all(
                &decisive_text,
                &["earlier", "clean", "later regressing verification attempt"],
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
