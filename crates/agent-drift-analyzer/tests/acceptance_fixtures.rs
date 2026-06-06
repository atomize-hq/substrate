#![allow(unused_crate_dependencies)]

mod support;

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
