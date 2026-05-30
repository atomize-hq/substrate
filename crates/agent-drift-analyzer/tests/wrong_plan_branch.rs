#![allow(unused_crate_dependencies)]

mod support;

use support::analyze_sample_bundle;

#[test]
fn wrong_plan_branch_scores_out_of_scope_changes() {
    let result = analyze_sample_bundle();
    let score = result.sessions[0]
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.drift_scores.iter())
        .find(|score| {
            score.class == agent_drift_analyzer::DriftClass::WrongPlanBranch && score.flagged
        })
        .expect("wrong plan branch score");

    assert!(score.raw_score >= 60);
    assert!(score.flagged);
}
