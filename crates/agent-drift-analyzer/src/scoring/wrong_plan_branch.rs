use crate::checkpoint::{CheckpointAnalysis, Confidence, DriftClass, DriftScore, DriftState};
use crate::context::command_has_unresolved_paths;
use crate::input::{
    path_can_establish_wrong_plan_scope, rooted_path_is_equal_or_descendant,
    trusted_repository_root,
};
use crate::scoring::{DriftStateHint, ScoredDrift};

pub(crate) fn score_wrong_plan_branch(analysis: &CheckpointAnalysis) -> ScoredDrift {
    let context = &analysis.current.context;
    let trusted_roots = context
        .truth_artifacts
        .iter()
        .filter(|artifact| artifact.source == "trusted_repository_root")
        .filter_map(|artifact| trusted_repository_root(&artifact.path))
        .collect::<Vec<_>>();
    let mut expected = context
        .truth_artifacts
        .iter()
        .filter(|artifact| {
            !matches!(
                artifact.source.as_str(),
                "control_directive_literal" | "trusted_repository_root"
            )
        })
        .filter(|artifact| path_can_establish_wrong_plan_scope(&artifact.path, &trusted_roots))
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    expected.sort();
    expected.dedup();
    let mut out_of_scope = Vec::new();
    for command in &analysis.interval.command_observations {
        if !command.write_like && !command.verification_like {
            continue;
        }
        if command_has_unresolved_paths(command) {
            out_of_scope.extend(command.evidence.clone());
            continue;
        }
        if expected.is_empty() || command.paths.is_empty() {
            continue;
        }
        let matches_scope = command.paths.iter().all(|path| {
            expected.iter().any(|expected_path| {
                rooted_path_is_equal_or_descendant(path, expected_path, &trusted_roots)
                    == Some(true)
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
    ScoredDrift::new(
        DriftScore {
            class: DriftClass::WrongPlanBranch,
            state: DriftState::Cleared,
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
        },
        DriftStateHint::None,
    )
}
