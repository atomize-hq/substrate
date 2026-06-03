use std::collections::BTreeSet;

use crate::checkpoint::{CheckpointAnalysis, Confidence, DriftClass, DriftScore, EvidenceRef};
use crate::context::CommandObservation;

const HISTORICAL_TRUTH_GROUNDING_GAP_REASON_PREFIX: &str = "historical truth-grounding gap:";

pub(crate) fn score_truth_grounding_gap(
    analysis: &CheckpointAnalysis,
    previous_truth_grounding_gap: Option<&DriftScore>,
) -> DriftScore {
    let truth_paths = analysis
        .current
        .task_frame
        .truth_artifacts
        .iter()
        .collect::<Vec<_>>();
    let first_action_index = analysis
        .interval
        .command_observations
        .iter()
        .filter(|command| command.write_like || command.verification_like)
        .filter_map(first_event_index)
        .min();
    let mut grounded_reads = Vec::<EvidenceRef>::new();
    let mut ungrounded_actions = Vec::<EvidenceRef>::new();

    for command in &analysis.interval.command_observations {
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
            grounded_reads.extend(command.evidence.clone());
        } else if (command.write_like || command.verification_like) && !touches_truth {
            ungrounded_actions.extend(command.evidence.clone());
        }
    }

    let historical_evidence = previous_truth_grounding_gap
        .map(historical_truth_grounding_gap_evidence)
        .unwrap_or_default();
    let active_gap =
        !truth_paths.is_empty() && grounded_reads.is_empty() && !ungrounded_actions.is_empty();
    let raw_score = if truth_paths.is_empty() {
        0
    } else if active_gap {
        80
    } else if !historical_evidence.is_empty() || !grounded_reads.is_empty() {
        20
    } else {
        0
    };

    let mut evidence = analysis
        .current
        .context
        .truth_artifacts
        .iter()
        .flat_map(|artifact| artifact.evidence.clone())
        .collect::<Vec<_>>();
    if active_gap {
        evidence.extend(ungrounded_actions);
    } else if !grounded_reads.is_empty() {
        evidence.extend(grounded_reads);
    }
    evidence.extend(historical_evidence);
    dedupe_evidence(&mut evidence);

    DriftScore {
        class: DriftClass::TruthGroundingGap,
        raw_score,
        confidence: if truth_paths.is_empty() {
            Confidence::Low
        } else if active_gap {
            Confidence::High
        } else {
            Confidence::Medium
        },
        flagged: active_gap,
        evidence,
    }
}

pub(crate) fn truth_grounding_gap_has_history(score: &DriftScore) -> bool {
    score.flagged
        || score
            .evidence
            .iter()
            .any(|evidence| is_historical_truth_grounding_gap_reason(&evidence.reason))
}

fn historical_truth_grounding_gap_evidence(previous: &DriftScore) -> Vec<EvidenceRef> {
    previous
        .evidence
        .iter()
        .filter_map(|evidence| {
            if is_historical_truth_grounding_gap_reason(&evidence.reason) {
                Some(evidence.clone())
            } else if previous.flagged {
                Some(EvidenceRef {
                    row: evidence.row.clone(),
                    reason: format!(
                        "{HISTORICAL_TRUTH_GROUNDING_GAP_REASON_PREFIX} {}",
                        evidence.reason
                    ),
                })
            } else {
                None
            }
        })
        .collect()
}

fn dedupe_evidence(evidence: &mut Vec<EvidenceRef>) {
    let mut seen = BTreeSet::new();
    evidence.retain(|item| {
        seen.insert((
            item.row.source_file.clone(),
            item.row.event_index,
            item.row.row_ordinal,
            item.reason.clone(),
        ))
    });
}

fn first_event_index(command: &CommandObservation) -> Option<usize> {
    command
        .evidence
        .first()
        .map(|evidence| evidence.row.event_index)
}

fn is_historical_truth_grounding_gap_reason(reason: &str) -> bool {
    reason.starts_with(HISTORICAL_TRUTH_GROUNDING_GAP_REASON_PREFIX)
}
