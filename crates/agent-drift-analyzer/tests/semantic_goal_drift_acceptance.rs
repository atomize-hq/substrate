#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, Checkpoint, DriftClass, DriftScore, DriftState,
    StructuredObjective,
};
use agent_session_compactor::{CompactionKind, CompactionRow, SourceKind, UserMessageRole};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use support::BundleFixture;

const SEMANTIC_GOAL_DRIFT_ACCEPTANCE_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/semantic_goal_drift_acceptance"
);

const SEMANTIC_GOAL_DRIFT_ACCEPTANCE_CASE_IDS: [&str; 16] = [
    "synthetic-kickoff-anchor-unauthorized-pivot",
    "synthetic-kickoff-generic-spec-plan-tasks-pivot",
    "synthetic-kickoff-hyphen-collision-pivot",
    "synthetic-kickoff-line-suffix-same-file",
    "synthetic-kickoff-objective-vs-objective-rs-pivot",
    "synthetic-rolling-artifact-family-progression",
    "synthetic-kickoff-narrowing-into-anchored-subtree",
    "synthetic-rolling-directory-to-file-narrowing",
    "synthetic-rolling-file-to-containing-directory-broadening",
    "synthetic-rolling-map-review-to-map-verify-progression",
    "synthetic-rolling-mid-session-pivot",
    "synthetic-rolling-plan-code-role-shift",
    "synthetic-rolling-review-verify-role-shift",
    "synthetic-rolling-doc-bundle-broadening",
    "synthetic-rolling-sibling-stem-pivot",
    "synthetic-rolling-work-item-family-progression",
];

#[derive(Debug, Deserialize)]
struct AcceptanceFixtureRaw {
    case_id: String,
    session_id: String,
    source_artifact: String,
    notes: String,
    rows: Vec<AcceptanceFixtureRawRow>,
}

#[derive(Debug, Deserialize)]
struct AcceptanceFixtureRawRow {
    turn_id: String,
    kind: CompactionKind,
    #[serde(default)]
    user_message_role: Option<UserMessageRole>,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AcceptanceExpectedScore {
    state: DriftState,
    flagged: bool,
    raw_score: u8,
}

#[derive(Debug, Deserialize)]
struct AcceptanceFixtureExpected {
    case_id: String,
    session_id: String,
    #[serde(default)]
    kickoff_target_display: Option<String>,
    final_target_display: String,
    #[serde(default)]
    final_target_paths: Vec<String>,
    #[serde(default)]
    penultimate_target_display: Option<String>,
    #[serde(default)]
    penultimate_semantic_goal_drift_flagged: Option<bool>,
    final_semantic_goal_drift: AcceptanceExpectedScore,
    required_reason_prefixes: Vec<String>,
    #[serde(default)]
    forbidden_reason_prefixes: Vec<String>,
}

#[test]
fn semantic_goal_drift_acceptance_corpus_stays_bounded_and_bundle_shaped() {
    let fixture_root = std::path::Path::new(SEMANTIC_GOAL_DRIFT_ACCEPTANCE_ROOT);
    let mut expected_root_entries = SEMANTIC_GOAL_DRIFT_ACCEPTANCE_CASE_IDS
        .iter()
        .map(|case_id| (*case_id).to_owned())
        .collect::<Vec<_>>();
    expected_root_entries.push("README.md".to_owned());
    expected_root_entries.sort();
    assert_eq!(sorted_entry_names(fixture_root), expected_root_entries);

    for case_id in SEMANTIC_GOAL_DRIFT_ACCEPTANCE_CASE_IDS {
        let case_dir = fixture_root.join(case_id);
        assert_eq!(
            sorted_entry_names(&case_dir),
            vec!["expected.json".to_owned(), "raw.json".to_owned()],
            "semantic-goal-drift acceptance case {case_id} must keep the committed raw/expected contract only"
        );

        let raw = load_raw_fixture(case_id);
        let expected = load_expected_fixture(case_id);
        assert_eq!(raw.case_id, case_id);
        assert_eq!(expected.case_id, case_id);
        assert!(
            !raw.source_artifact.trim().is_empty(),
            "acceptance case {case_id} must document its source artifact"
        );
        assert!(
            !raw.notes.trim().is_empty(),
            "acceptance case {case_id} must document its packet-local notes"
        );
        assert!(
            raw.rows.len() >= 4,
            "acceptance case {case_id} must provide enough rows to build kickoff and pivot checkpoints"
        );
        if expected.penultimate_target_display.is_some() {
            assert!(
                raw.rows.len() >= 7,
                "acceptance case {case_id} must provide enough rows to build a mid-session rolling pivot"
            );
        }
    }
}

#[test]
fn semantic_goal_drift_acceptance_fixture_runs_through_live_analyzer_checkpoint_path() {
    for case_id in SEMANTIC_GOAL_DRIFT_ACCEPTANCE_CASE_IDS {
        let raw = load_raw_fixture(case_id);
        let expected = load_expected_fixture(case_id);
        let fixture = BundleFixture::from_compact_rows(rows_from_raw_fixture(case_id, &raw));
        let result = analyze_bundle(&AnalyzeRequest {
            input_dir: fixture.input_dir.clone(),
            output_dir: fixture.output_dir.clone(),
        })
        .expect("analyze semantic goal drift acceptance case");

        let session = result
            .sessions
            .iter()
            .find(|session| session.session_id == expected.session_id)
            .expect("acceptance session");
        assert!(
            session.checkpoints.len() >= 2,
            "acceptance case {case_id} must emit a kickoff checkpoint and a later pivot checkpoint"
        );

        let kickoff_checkpoint = session.checkpoints.first().expect("kickoff checkpoint");
        if let Some(expected_kickoff_target) = expected.kickoff_target_display.as_deref() {
            assert_eq!(
                checkpoint_target_display(kickoff_checkpoint),
                expected_kickoff_target,
                "kickoff checkpoint must preserve the original anchored target"
            );
            assert_checkpoint_has_full_goal_eligibility(
                kickoff_checkpoint,
                "kickoff checkpoint",
                case_id,
            );
        }
        assert!(
            !semantic_goal_drift_score(kickoff_checkpoint).flagged,
            "kickoff checkpoint must not flag semantic_goal_drift before the objective pivots"
        );

        if let Some(expected_penultimate_target) = expected.penultimate_target_display.as_deref() {
            let penultimate_checkpoint = session
                .checkpoints
                .get(session.checkpoints.len().saturating_sub(2))
                .expect("penultimate checkpoint");
            assert_eq!(
                checkpoint_target_display(penultimate_checkpoint),
                expected_penultimate_target,
                "penultimate checkpoint must preserve the pre-pivot rolling anchor"
            );
            assert_checkpoint_has_full_goal_eligibility(
                penultimate_checkpoint,
                "penultimate checkpoint",
                case_id,
            );
            if let Some(expected_flagged) = expected.penultimate_semantic_goal_drift_flagged {
                assert_eq!(
                    semantic_goal_drift_score(penultimate_checkpoint).flagged,
                    expected_flagged,
                    "penultimate checkpoint semantic_goal_drift flagged bit drifted for {case_id}"
                );
            }
        }

        let final_checkpoint = session.checkpoints.last().expect("final checkpoint");
        assert_eq!(
            checkpoint_target_display(final_checkpoint),
            expected.final_target_display,
            "final checkpoint must expose the pivoted structured target through the live objective path"
        );
        if !expected.final_target_paths.is_empty() {
            let structured = checkpoint_structured_objective(final_checkpoint);
            let target = structured
                .target
                .as_ref()
                .expect("final checkpoint structured target");
            assert_eq!(
                target.paths, expected.final_target_paths,
                "final checkpoint must preserve the expected concrete target paths for {case_id}"
            );
        }
        assert_checkpoint_has_full_goal_eligibility(final_checkpoint, "final checkpoint", case_id);

        let final_semantic_goal_drift = semantic_goal_drift_score(final_checkpoint);
        assert_eq!(
            final_semantic_goal_drift.state,
            expected.final_semantic_goal_drift.state
        );
        assert_eq!(
            final_semantic_goal_drift.flagged,
            expected.final_semantic_goal_drift.flagged
        );
        assert_eq!(
            final_semantic_goal_drift.raw_score,
            expected.final_semantic_goal_drift.raw_score
        );
        for prefix in &expected.required_reason_prefixes {
            assert!(
                final_semantic_goal_drift
                    .evidence
                    .iter()
                    .any(|item| item.reason.starts_with(prefix)),
                "acceptance case {case_id} must preserve evidence prefix `{prefix}` on the live analyzer path"
            );
        }
        for prefix in &expected.forbidden_reason_prefixes {
            assert!(
                final_semantic_goal_drift
                    .evidence
                    .iter()
                    .all(|item| !item.reason.starts_with(prefix)),
                "acceptance case {case_id} must not pick up forbidden evidence prefix `{prefix}`"
            );
        }
    }
}

fn assert_checkpoint_has_full_goal_eligibility(
    checkpoint: &Checkpoint,
    label: &str,
    case_id: &str,
) {
    let structured = checkpoint_structured_objective(checkpoint);
    assert!(
        structured.unknowns.is_empty(),
        "{label} for {case_id} must clear the full semantic_goal_drift eligibility bar: {:?}",
        structured.unknowns
    );
    let target = structured
        .target
        .as_ref()
        .unwrap_or_else(|| panic!("{label} for {case_id} missing structured target"));
    assert!(
        !target.evidence.is_empty(),
        "{label} for {case_id} must retain non-empty target evidence"
    );
}

fn load_raw_fixture(case_id: &str) -> AcceptanceFixtureRaw {
    read_json(&Utf8PathBuf::from(format!(
        "{SEMANTIC_GOAL_DRIFT_ACCEPTANCE_ROOT}/{case_id}/raw.json"
    )))
}

fn load_expected_fixture(case_id: &str) -> AcceptanceFixtureExpected {
    read_json(&Utf8PathBuf::from(format!(
        "{SEMANTIC_GOAL_DRIFT_ACCEPTANCE_ROOT}/{case_id}/expected.json"
    )))
}

fn rows_from_raw_fixture(case_id: &str, raw: &AcceptanceFixtureRaw) -> Vec<CompactionRow> {
    let source_file = Utf8PathBuf::from(format!(
        "/fixtures/semantic_goal_drift_acceptance/{case_id}.jsonl"
    ));
    raw.rows
        .iter()
        .enumerate()
        .map(|(index, row)| CompactionRow {
            source_file: source_file.clone(),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some(raw.session_id.clone()),
            turn_id: Some(row.turn_id.clone()),
            event_index: index,
            line_number: index + 1,
            row_ordinal: 0,
            timestamp: None,
            kind: row.kind,
            user_message_role: row.user_message_role,
            dedupe_identity: None,
            text: row.text.clone(),
            canonical_text: row.text.clone(),
            text_hash_hex: format!("hash-{case_id}-{index}"),
        })
        .collect()
}

fn checkpoint_target_display(checkpoint: &Checkpoint) -> String {
    checkpoint_structured_objective(checkpoint)
        .target
        .as_ref()
        .map(|target| target.display.clone())
        .unwrap_or_else(|| {
            panic!(
                "checkpoint {} missing structured target",
                checkpoint.checkpoint_id
            )
        })
}

fn semantic_goal_drift_score(checkpoint: &Checkpoint) -> &DriftScore {
    checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::SemanticGoalDrift)
        .unwrap_or_else(|| {
            panic!(
                "checkpoint {} missing semantic_goal_drift score",
                checkpoint.checkpoint_id
            )
        })
}

fn checkpoint_structured_objective(checkpoint: &Checkpoint) -> &StructuredObjective {
    checkpoint.structured_objective.as_ref().unwrap_or_else(|| {
        panic!(
            "checkpoint {} missing structured objective",
            checkpoint.checkpoint_id
        )
    })
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Utf8Path) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read json artifact"))
        .expect("parse json artifact")
}

fn sorted_entry_names(path: &std::path::Path) -> Vec<String> {
    let mut entries = fs::read_dir(path)
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
