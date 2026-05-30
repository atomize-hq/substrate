#![allow(unused_crate_dependencies)]

mod support;

use support::analyze_sample_bundle;

#[test]
fn wrong_plan_branch_scores_out_of_scope_changes() {
    let result = analyze_sample_bundle();
    let checkpoint = &result.sessions[0].checkpoints[0];
    let score = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::WrongPlanBranch)
        .expect("wrong plan branch score");

    assert!(score.raw_score >= 60);
    assert!(score.flagged);
}
