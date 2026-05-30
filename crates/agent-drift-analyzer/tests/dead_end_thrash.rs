#![allow(unused_crate_dependencies)]

mod support;

use support::analyze_sample_bundle;

#[test]
fn dead_end_thrash_uses_archival_repetition() {
    let result = analyze_sample_bundle();
    let checkpoint = &result.sessions[0].checkpoints[0];
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert!(score.raw_score >= 60);
    assert!(score.flagged);
    assert!(score.evidence.len() >= 3);
}
