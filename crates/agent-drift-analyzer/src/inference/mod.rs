use std::collections::BTreeSet;

use crate::checkpoint::{Confidence, EvidenceRef, TaskFrame};
use crate::context::ContextPack;

pub fn infer_task_frame(context: &ContextPack) -> TaskFrame {
    let truth_artifacts = context
        .truth_artifacts
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    let working_set_paths = context
        .working_set_paths
        .iter()
        .map(|path| path.path.clone())
        .collect::<Vec<_>>();
    let tools = context.tools.iter().map(|tool| tool.name.clone()).collect();
    let counter_evidence = infer_counter_evidence(context, &truth_artifacts);
    let confidence = infer_confidence(context, &counter_evidence);

    TaskFrame {
        objective: context.objective.text.clone(),
        confidence,
        truth_artifacts,
        working_set_paths,
        tools,
        command_families: context.command_families.clone(),
        verification_commands: context.objective.verification_commands.clone(),
        supporting_evidence: context.supporting_evidence.clone(),
        counter_evidence,
    }
}

fn infer_counter_evidence(context: &ContextPack, truth_artifacts: &[String]) -> Vec<EvidenceRef> {
    let truth_set = truth_artifacts.iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut evidence = Vec::new();
    for command in &context.command_observations {
        for path in &command.paths {
            if !truth_set.contains(path) {
                for item in &command.evidence {
                    let candidate = EvidenceRef {
                        row: item.row.clone(),
                        reason: format!(
                            "observed path outside explicit truth artifact set: {path}"
                        ),
                    };
                    let key = format!(
                        "{}:{}:{}:{}:{}",
                        candidate.row.source_file,
                        candidate.row.line_number,
                        candidate.row.event_index,
                        candidate.row.row_ordinal,
                        candidate.reason
                    );
                    if seen.insert(key) {
                        evidence.push(candidate);
                    }
                }
            }
        }
    }
    evidence
}

fn infer_confidence(context: &ContextPack, counter_evidence: &[EvidenceRef]) -> Confidence {
    let has_objective = !context.objective.text.trim().is_empty()
        && context.objective.text != "No objective row available";
    let has_truth = !context.truth_artifacts.is_empty();
    let has_working_set = !context.working_set_paths.is_empty();
    let has_commands = !context.command_observations.is_empty();

    match (
        has_objective,
        has_truth || has_working_set,
        has_commands,
        counter_evidence.len(),
    ) {
        (true, true, true, 0..=2) => Confidence::High,
        (true, true, _, _) | (true, _, true, _) => Confidence::Medium,
        _ => Confidence::Low,
    }
}
