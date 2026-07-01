mod dead_end_thrash;
mod semantic_goal_drift;
mod truth_grounding_gap;
mod wrong_plan_branch;

use crate::checkpoint::{
    build_scoring_session_progress, CheckpointAnalysis, DriftClass, DriftScore, StructuredObjective,
};

pub(crate) use truth_grounding_gap::score_truth_grounding_gap;
pub(crate) use wrong_plan_branch::score_wrong_plan_branch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DriftStateHint {
    None,
    HistoricalContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScoredDrift {
    pub score: DriftScore,
    pub state_hint: DriftStateHint,
}

impl ScoredDrift {
    pub(crate) fn new(score: DriftScore, state_hint: DriftStateHint) -> Self {
        Self { score, state_hint }
    }
}

pub(crate) fn score_session(
    analysis: &CheckpointAnalysis,
    previous_truth_grounding_gap: Option<&DriftScore>,
    kickoff_anchor: Option<&StructuredObjective>,
) -> Vec<ScoredDrift> {
    let _ = kickoff_anchor;
    let session_progress = build_scoring_session_progress(analysis);
    let mut scores = vec![
        score_wrong_plan_branch(analysis),
        score_truth_grounding_gap(analysis, previous_truth_grounding_gap),
        dead_end_thrash::score_dead_end_thrash(analysis, &session_progress),
        semantic_goal_drift::score_semantic_goal_drift(analysis, kickoff_anchor),
    ];
    scores.sort_by_key(|score| match score.score.class {
        DriftClass::WrongPlanBranch => 0,
        DriftClass::TruthGroundingGap => 1,
        DriftClass::DeadEndThrash => 2,
        DriftClass::SemanticGoalDrift => 3,
    });
    scores
}
