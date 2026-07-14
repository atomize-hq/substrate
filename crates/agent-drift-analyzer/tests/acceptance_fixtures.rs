#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, Confidence, DriftClass, DriftState, ProgressDimension,
    ProgressSignalCode, ProgressStatus,
};
use support::{load_acceptance_case, ACCEPTANCE_CASE_IDS, ACCEPTANCE_EXCLUDED_CASES};

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

fn assert_acceptance_case(case_id: &str) {
    let case = load_acceptance_case(case_id);
    let result = case.analyze();
    let final_score = case.final_dead_end_thrash(&result);

    assert_eq!(
        final_score.state, case.expected.final_dead_end_thrash.state,
        "unexpected dead_end_thrash state for {case_id}"
    );
    assert_eq!(
        final_score.flagged, case.expected.final_dead_end_thrash.flagged,
        "unexpected dead_end_thrash flagged bit for {case_id}"
    );
    assert_eq!(
        final_score.raw_score, case.expected.final_dead_end_thrash.raw_score,
        "unexpected dead_end_thrash raw_score for {case_id}"
    );
}

#[test]
fn acceptance_fixtures_corpus_stays_screened_and_excluded_sessions_stay_out() {
    let fixture_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/acceptance");
    let mut expected_root_entries = ACCEPTANCE_CASE_IDS
        .iter()
        .map(|case_id| (*case_id).to_owned())
        .collect::<Vec<_>>();
    expected_root_entries.push("README.md".to_owned());
    expected_root_entries.sort();
    assert_eq!(
        sorted_entry_names(&fixture_root),
        expected_root_entries,
        "Packet R2 acceptance corpus must contain only README.md and the four allowed case directories"
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

    for case_id in ACCEPTANCE_CASE_IDS {
        let case = load_acceptance_case(case_id);
        assert_eq!(case.expected.session_id, case_id);
        assert_eq!(
            case.expected.source_artifact,
            format!("target/hybrid-drift-evals/{case_id}-r1e/compactor")
        );
        assert_eq!(
            sorted_entry_names(&fixture_root.join(case_id)),
            expected_case_entries,
            "acceptance case {case_id} must contain exactly the Packet R2 fixture contract files"
        );
    }

    for (excluded_case_id, reason) in ACCEPTANCE_EXCLUDED_CASES {
        let excluded_dir = fixture_root.join(excluded_case_id);
        assert!(
            !excluded_dir.exists(),
            "excluded session {excluded_case_id} must stay out of the frozen acceptance corpus: {reason}"
        );
    }
}

#[test]
fn acceptance_fixtures_cleared_control_019e93fa_stays_cleared() {
    assert_acceptance_case("019e93fa-60d4-73d1-9092-014130b60e14");
}

#[test]
fn acceptance_fixtures_cleared_control_019e940c_stays_cleared() {
    assert_acceptance_case("019e940c-a91b-7fe0-a967-b0bdd595b581");
}

#[test]
fn acceptance_fixtures_cleared_control_019e943c_stays_cleared() {
    assert_acceptance_case("019e943c-668e-7a03-992b-6a98cf3055da");
}

#[test]
fn acceptance_fixtures_representative_sticky_success_tail_stays_recovered() {
    assert_acceptance_case("019e894a-86c9-71e3-b57b-e3d3285f0988");
}

#[test]
fn acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture() {
    for case_id in [
        "019e93fa-60d4-73d1-9092-014130b60e14",
        "019e940c-a91b-7fe0-a967-b0bdd595b581",
        "019e943c-668e-7a03-992b-6a98cf3055da",
    ] {
        let case = load_acceptance_case(case_id);
        let result = case.analyze();
        let final_score = case.final_dead_end_thrash(&result);

        assert_eq!(
            final_score.state,
            DriftState::Cleared,
            "frozen cleared control {case_id} must stay cleared"
        );
        assert!(
            !final_score.flagged,
            "frozen cleared control {case_id} must stay unflagged"
        );
        assert_eq!(
            final_score.raw_score, 0,
            "frozen cleared control {case_id} must stay at raw_score 0"
        );
    }

    let sticky_case = load_acceptance_case("019e894a-86c9-71e3-b57b-e3d3285f0988");
    let sticky_result = sticky_case.analyze();
    let sticky_score = sticky_case.final_dead_end_thrash(&sticky_result);

    assert_eq!(
        sticky_score.state,
        DriftState::Recovered,
        "sticky success-tail witness must stay recovered"
    );
    assert!(
        !sticky_score.flagged,
        "sticky success-tail witness must stay unflagged"
    );
    assert_eq!(
        sticky_score.raw_score, 20,
        "sticky success-tail witness must stay at raw_score 20"
    );
}

#[test]
fn acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged() {
    const CASE_ID: &str = "019e899c-453f-71f2-a99d-155848c7b081";
    const SELECTED_CHECKPOINT_ORDINAL: usize = 3;

    let fixture_dir = camino::Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/progress_acceptance")
        .join(CASE_ID);
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture_dir.join("manifest.json"))
            .expect("read advancing replay manifest"),
    )
    .expect("parse advancing replay manifest");
    assert_eq!(
        manifest["session_ids"],
        serde_json::json!([CASE_ID]),
        "trusted replay fixture must contain only the selected real rollout"
    );

    let expected: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture_dir.join("expected.json"))
            .expect("read advancing replay annotation"),
    )
    .expect("parse advancing replay annotation");
    assert_eq!(expected["fixture_kind"], "annotated_real_rollout");
    assert_eq!(expected["source_rollout_id"], CASE_ID);
    let fixture_notes = expected["notes"]
        .as_str()
        .expect("advancing replay fixture notes");
    assert!(fixture_notes.contains("CTX-R6-01"));
    assert!(fixture_notes.contains("event 171"));
    assert_eq!(
        expected["selected_checkpoint"]["ordinal"],
        SELECTED_CHECKPOINT_ORDINAL
    );

    let compact_rows = std::fs::read_to_string(fixture_dir.join("rows.compact.jsonl"))
        .expect("read advancing replay compact rows");
    let repeated_failure_output = compact_rows
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("parse compact row"))
        .find(|row| row["event_index"] == 171 && row["kind"] == "tool_output")
        .expect("trusted replay fixture must retain the repeated-failure output");
    let repeated_failure_text = repeated_failure_output["text"]
        .as_str()
        .expect("repeated-failure output text");
    assert!(
        repeated_failure_text
            .contains("real_session_live_coordinator_persists_cursor_across_restarts ... FAILED"),
        "trusted replay fixture must retain the first concrete failure"
    );
    assert!(
        repeated_failure_text.contains(
            "real_session_live_coordinator_rejects_invalid_persisted_cursor_state ... FAILED"
        ),
        "trusted replay fixture must retain the second concrete failure"
    );
    assert!(
        repeated_failure_text.contains("test result: FAILED. 3 passed; 2 failed"),
        "trusted replay fixture must retain the repeated-failure result summary"
    );

    let temp_dir = tempfile::TempDir::new().expect("temp dir");
    let output_dir = camino::Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("output");
    std::fs::create_dir_all(&output_dir).expect("create advancing replay output dir");
    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: fixture_dir,
        output_dir,
    })
    .expect("analyze trusted advancing repeated-failure fixture");
    let session = result
        .sessions
        .iter()
        .find(|session| session.session_id == CASE_ID)
        .expect("selected real-rollout session");
    let checkpoint = session
        .checkpoints
        .iter()
        .find(|checkpoint| checkpoint.ordinal == SELECTED_CHECKPOINT_ORDINAL)
        .expect("selected advancing checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("selected checkpoint progress");
    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::Advancing);
    assert!(
        progress
            .signals
            .iter()
            .any(|signal| signal.code == ProgressSignalCode::VerificationClean),
        "selected checkpoint must carry a direct troubleshooting-frontier advancement signal"
    );

    let dead_end_score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::DeadEndThrash)
        .expect("selected dead_end_thrash score");
    assert!(!dead_end_score.flagged);
    assert_ne!(dead_end_score.state, DriftState::Active);
    assert!(
        session
            .checkpoints
            .iter()
            .take_while(|candidate| candidate.ordinal < SELECTED_CHECKPOINT_ORDINAL)
            .filter_map(|candidate| {
                candidate
                    .drift_scores
                    .iter()
                    .find(|score| score.class == DriftClass::DeadEndThrash)
            })
            .all(|score| score.state != DriftState::Active),
        "HistoricalOnly is allowed only because no prior checkpoint was Active"
    );
    assert_eq!(dead_end_score.state, DriftState::HistoricalOnly);
    assert_eq!(dead_end_score.raw_score, 20);
    assert_eq!(dead_end_score.confidence, Confidence::High);
}
