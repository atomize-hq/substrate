#![allow(unused_crate_dependencies)]

mod support;

use support::analyze_sample_bundle;

#[test]
fn ignoring_repo_truth_flags_verification_without_truth_reads() {
    let result = analyze_sample_bundle();
    let checkpoint = &result.sessions[0].checkpoints[0];
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::IgnoringRepoTruth)
        .expect("ignoring repo truth score");

    assert!(score.raw_score >= 60);
    assert!(score.flagged);
}
