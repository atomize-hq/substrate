#![allow(unused_crate_dependencies)]

mod support;

use support::analyze_sample_bundle;

#[test]
fn dead_end_thrash_uses_archival_repetition() {
    let result = analyze_sample_bundle();
    let score = result.sessions[0]
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.drift_scores.iter())
        .find(|score| {
            score.class == agent_drift_analyzer::DriftClass::DeadEndThrash && score.flagged
        })
        .expect("dead end thrash score");

    assert!(score.raw_score >= 60);
    assert!(score.flagged);
    assert!(score.evidence.len() >= 3);
}
