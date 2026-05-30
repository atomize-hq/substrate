use crate::checkpoint::{Confidence, DriftClass, DriftScore, EvidenceRef, TaskFrame};
use crate::context::ContextPack;

pub fn score_ignoring_repo_truth(context: &ContextPack, task_frame: &TaskFrame) -> DriftScore {
    let truth_paths = task_frame.truth_artifacts.iter().collect::<Vec<_>>();
    let mut truth_reads = Vec::<EvidenceRef>::new();
    let mut acting_without_truth = Vec::<EvidenceRef>::new();
    let first_action_index = context
        .command_observations
        .iter()
        .filter(|command| command.write_like || command.verification_like)
        .filter_map(first_event_index)
        .min();

    for command in &context.command_observations {
        let touches_truth = command.paths.iter().any(|path| {
            truth_paths
                .iter()
                .any(|truth| path == *truth || path.starts_with(*truth) || truth.starts_with(path))
        });
        let before_first_action = first_action_index
            .zip(first_event_index(command))
            .map(|(first_action, current)| current < first_action)
            .unwrap_or(true);
        if touches_truth && command.read_like && before_first_action {
            truth_reads.extend(command.evidence.clone());
        } else if (command.write_like || command.verification_like) && !touches_truth {
            acting_without_truth.extend(command.evidence.clone());
        }
    }

    let raw_score = if truth_paths.is_empty() {
        0
    } else if truth_reads.is_empty() && !acting_without_truth.is_empty() {
        80
    } else if truth_reads.is_empty() {
        60
    } else {
        20
    };

    let mut evidence = Vec::new();
    evidence.extend(
        context
            .truth_artifacts
            .iter()
            .flat_map(|artifact| artifact.evidence.clone()),
    );
    if truth_reads.is_empty() {
        evidence.extend(acting_without_truth);
    } else {
        evidence.extend(truth_reads.clone());
    }

    DriftScore {
        class: DriftClass::IgnoringRepoTruth,
        raw_score,
        confidence: if truth_paths.is_empty() {
            Confidence::Low
        } else if truth_reads.is_empty() {
            Confidence::High
        } else {
            Confidence::Medium
        },
        flagged: raw_score >= 60,
        evidence,
    }
}

fn first_event_index(command: &crate::context::CommandObservation) -> Option<usize> {
    command
        .evidence
        .first()
        .map(|evidence| evidence.row.event_index)
}
