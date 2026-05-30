mod dead_end_thrash;
mod ignoring_repo_truth;
mod wrong_plan_branch;

use crate::checkpoint::{DriftClass, DriftScore, TaskFrame};
use crate::context::ContextPack;
use crate::input::BundleSession;

pub use dead_end_thrash::score_dead_end_thrash;
pub use ignoring_repo_truth::score_ignoring_repo_truth;
pub use wrong_plan_branch::score_wrong_plan_branch;

pub fn score_session(
    session: &BundleSession,
    context: &ContextPack,
    task_frame: &TaskFrame,
) -> Vec<DriftScore> {
    let mut scores = vec![
        score_wrong_plan_branch(context, task_frame),
        score_ignoring_repo_truth(context, task_frame),
        score_dead_end_thrash(session, context),
    ];
    scores.sort_by_key(|score| match score.class {
        DriftClass::WrongPlanBranch => 0,
        DriftClass::IgnoringRepoTruth => 1,
        DriftClass::DeadEndThrash => 2,
    });
    scores
}
