#![allow(unused_crate_dependencies)]

mod support;

use support::{analyze_clean_recovery_bundle, analyze_sample_bundle};

#[test]
fn dead_end_thrash_stays_active_when_verification_interval_remains_out_of_scope() {
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

    assert!(second.raw_score >= 60);
    assert!(second.flagged);
    assert!(second
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));
    assert!(second
        .evidence
        .iter()
        .any(|item| item.reason.starts_with("repeated verification command:")));
}

#[test]
fn dead_end_thrash_clears_after_one_clean_in_scope_verification_interval() {
    let result = analyze_clean_recovery_bundle();
    let checkpoints = &result.sessions[0].checkpoints;
    let recovered = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("recovered dead end thrash score");

    assert_eq!(recovered.raw_score, 20);
    assert!(!recovered.flagged);
    assert!(recovered.evidence.iter().any(|item| item
        .reason
        .starts_with("historical repeated failure evidence:")));
    assert!(recovered.evidence.iter().any(|item| item
        .reason
        .starts_with("historical repeated verification evidence:")));
}
