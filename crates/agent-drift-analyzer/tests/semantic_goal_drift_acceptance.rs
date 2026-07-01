#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{analyze_bundle, AnalyzeRequest, Checkpoint, DriftClass, DriftScore, DriftState};
use agent_session_compactor::{CompactionKind, CompactionRow, SourceKind, UserMessageRole};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use support::BundleFixture;

const SEMANTIC_GOAL_DRIFT_ACCEPTANCE_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/semantic_goal_drift_acceptance"
);

const SEMANTIC_GOAL_DRIFT_ACCEPTANCE_CASE_IDS: [&str; 1] =
    ["synthetic-kickoff-anchor-unauthorized-pivot"];

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
    kickoff_target_display: String,
    final_target_display: String,
    final_semantic_goal_drift: AcceptanceExpectedScore,
    required_reason_prefixes: Vec<String>,
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
        assert_eq!(
            checkpoint_target_display(kickoff_checkpoint),
            expected.kickoff_target_display,
            "kickoff checkpoint must preserve the original anchored target"
        );
        assert!(
            !semantic_goal_drift_score(kickoff_checkpoint).flagged,
            "kickoff checkpoint must not flag semantic_goal_drift before the objective pivots"
        );

        let final_checkpoint = session.checkpoints.last().expect("final checkpoint");
        assert_eq!(
            checkpoint_target_display(final_checkpoint),
            expected.final_target_display,
            "final checkpoint must expose the pivoted structured target through the live objective path"
        );

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
    }
}

fn load_raw_fixture(case_id: &str) -> AcceptanceFixtureRaw {
    read_json(
        &Utf8PathBuf::from(format!(
            "{SEMANTIC_GOAL_DRIFT_ACCEPTANCE_ROOT}/{case_id}/raw.json"
        )),
    )
}

fn load_expected_fixture(case_id: &str) -> AcceptanceFixtureExpected {
    read_json(
        &Utf8PathBuf::from(format!(
            "{SEMANTIC_GOAL_DRIFT_ACCEPTANCE_ROOT}/{case_id}/expected.json"
        )),
    )
}

fn rows_from_raw_fixture(case_id: &str, raw: &AcceptanceFixtureRaw) -> Vec<CompactionRow> {
    let source_file =
        Utf8PathBuf::from(format!("/fixtures/semantic_goal_drift_acceptance/{case_id}.jsonl"));
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
    checkpoint
        .structured_objective
        .as_ref()
        .and_then(|objective| objective.target.as_ref())
        .map(|target| target.display.clone())
        .unwrap_or_else(|| panic!("checkpoint {} missing structured target", checkpoint.checkpoint_id))
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
