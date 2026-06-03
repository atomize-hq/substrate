mod dead_end_thrash;
mod truth_grounding_gap;
mod wrong_plan_branch;

use crate::checkpoint::{CheckpointAnalysis, DriftClass, DriftScore};

pub(crate) use dead_end_thrash::score_dead_end_thrash;
pub(crate) use truth_grounding_gap::{
    score_truth_grounding_gap, truth_grounding_gap_has_history,
};
pub(crate) use wrong_plan_branch::score_wrong_plan_branch;

pub(crate) fn score_session(
    analysis: &CheckpointAnalysis,
    previous_truth_grounding_gap: Option<&DriftScore>,
) -> Vec<DriftScore> {
    let mut scores = vec![
        score_wrong_plan_branch(analysis),
        score_truth_grounding_gap(analysis, previous_truth_grounding_gap),
        score_dead_end_thrash(analysis),
    ];
    scores.sort_by_key(|score| match score.class {
        DriftClass::WrongPlanBranch => 0,
        DriftClass::TruthGroundingGap => 1,
        DriftClass::DeadEndThrash => 2,
    });
    scores
}
