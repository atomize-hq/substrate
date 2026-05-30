use crate::checkpoint::{Confidence, DriftClass, DriftScore, TaskFrame};
use crate::context::ContextPack;

pub fn score_wrong_plan_branch(context: &ContextPack, task_frame: &TaskFrame) -> DriftScore {
    let expected = if !task_frame.truth_artifacts.is_empty() {
        task_frame.truth_artifacts.clone()
    } else {
        task_frame.working_set_paths.clone()
    };
    let mut out_of_scope = Vec::new();
    for command in &context.command_observations {
        if command.paths.is_empty() {
            continue;
        }
        let matches_scope = command.paths.iter().all(|path| {
            expected.iter().any(|expected_path| {
                path == expected_path
                    || path.starts_with(expected_path)
                    || expected_path.starts_with(path)
            })
        });
        if !matches_scope {
            out_of_scope.extend(command.evidence.clone());
        }
    }

    let raw_score = match out_of_scope.len() {
        0 => 0,
        1 => 60,
        2 => 80,
        _ => 100,
    };
    DriftScore {
        class: DriftClass::WrongPlanBranch,
        raw_score,
        confidence: if expected.is_empty() {
            Confidence::Low
        } else if out_of_scope.len() >= 2 {
            Confidence::High
        } else {
            Confidence::Medium
        },
        flagged: raw_score >= 60,
        evidence: out_of_scope,
    }
}
