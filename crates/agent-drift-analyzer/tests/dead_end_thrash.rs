#![allow(unused_crate_dependencies)]

mod support;

use support::analyze_sample_bundle;

#[test]
fn dead_end_thrash_clears_after_one_clean_verification_interval() {
    let result = analyze_sample_bundle();
    let checkpoints = &result.sessions[0].checkpoints;
    let first = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("first dead end thrash score");
    let second = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("second dead end thrash score");

    assert!(first.raw_score >= 60);
    assert!(first.flagged);
    assert!(first.evidence.len() >= 3);
    assert!(first
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));

    assert_eq!(second.raw_score, 20);
    assert!(!second.flagged);
    assert!(second
        .evidence
        .iter()
        .any(|item| item
            .reason
            .starts_with("historical repeated failure evidence:")));
    assert!(second
        .evidence
        .iter()
        .any(|item| item
            .reason
            .starts_with("historical repeated verification evidence:")));
}
