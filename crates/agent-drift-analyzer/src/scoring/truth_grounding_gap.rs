use std::collections::BTreeSet;

use crate::checkpoint::{
    CheckpointAnalysis, Confidence, DriftClass, DriftScore, DriftState, EvidenceRef,
};
use crate::context::CommandObservation;
use crate::input::{extract_directive_path_hints, truth_paths_equal, truth_paths_overlap};
use crate::scoring::{DriftStateHint, ScoredDrift};

const HISTORICAL_TRUTH_GROUNDING_GAP_REASON_PREFIX: &str = "historical truth-grounding gap:";

#[derive(Debug, Default)]
pub(crate) struct TruthGroundingProvenance {
    grounded_paths: BTreeSet<String>,
}

pub(crate) fn score_truth_grounding_gap(
    analysis: &CheckpointAnalysis,
    previous_truth_grounding_gap: Option<&DriftScore>,
    provenance: &mut TruthGroundingProvenance,
) -> ScoredDrift {
    let truth_paths = declared_truth_paths(analysis);
    provenance.grounded_paths.retain(|grounded| {
        truth_paths
            .iter()
            .any(|truth| truth_paths_equal(grounded, truth))
    });
    let mut grounded_reads = Vec::<EvidenceRef>::new();
    let mut ungrounded_actions = Vec::<EvidenceRef>::new();

    for command in &analysis.interval.command_observations {
        let matching_paths = matching_truth_paths(command, &truth_paths);
        if command.write_like || command.verification_like {
            let grounded = if matching_paths.is_empty() {
                truth_paths
                    .iter()
                    .all(|path| path_is_grounded(provenance, path))
            } else {
                matching_paths
                    .iter()
                    .all(|path| path_is_grounded(provenance, path))
            };
            if !grounded {
                ungrounded_actions.extend(command.evidence.clone());
            }
        }
        if command.read_like && !matching_paths.is_empty() {
            provenance
                .grounded_paths
                .extend(matching_paths.into_iter().cloned());
            grounded_reads.extend(command.evidence.clone());
        }
    }

    let historical_evidence = previous_truth_grounding_gap
        .map(historical_truth_grounding_gap_evidence)
        .unwrap_or_default();
    let active_gap = !truth_paths.is_empty() && !ungrounded_actions.is_empty();
    let raw_score = if truth_paths.is_empty() {
        0
    } else if active_gap {
        80
    } else if !historical_evidence.is_empty() || !grounded_reads.is_empty() {
        20
    } else {
        0
    };

    let historical_context = !historical_evidence.is_empty();
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

    ScoredDrift::new(
        DriftScore {
            class: DriftClass::TruthGroundingGap,
            state: DriftState::Cleared,
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
        },
        if historical_context {
            DriftStateHint::HistoricalContext
        } else {
            DriftStateHint::None
        },
    )
}

fn declared_truth_paths(analysis: &CheckpointAnalysis) -> BTreeSet<String> {
    let task_frame = &analysis.current.task_frame;
    let extracted_paths = extract_directive_path_hints(&task_frame.objective);
    let objective_paths = task_frame
        .truth_artifacts
        .iter()
        .filter(|path| {
            extracted_paths
                .iter()
                .any(|extracted| truth_paths_equal(extracted, path))
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    if extracted_paths.is_empty() {
        task_frame.truth_artifacts.iter().cloned().collect()
    } else {
        objective_paths
    }
}

fn matching_truth_paths<'a>(
    command: &CommandObservation,
    truth_paths: &'a BTreeSet<String>,
) -> Vec<&'a String> {
    truth_paths
        .iter()
        .filter(|truth| {
            command
                .paths
                .iter()
                .any(|path| paths_share_identity(path, truth))
        })
        .collect()
}

fn paths_share_identity(left: &str, right: &str) -> bool {
    truth_paths_overlap(left, right)
}

fn path_is_grounded(provenance: &TruthGroundingProvenance, path: &str) -> bool {
    provenance
        .grounded_paths
        .iter()
        .any(|grounded| truth_paths_equal(grounded, path))
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

fn is_historical_truth_grounding_gap_reason(reason: &str) -> bool {
    reason.starts_with(HISTORICAL_TRUTH_GROUNDING_GAP_REASON_PREFIX)
}
