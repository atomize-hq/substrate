use std::collections::BTreeSet;

use crate::checkpoint::{
    CheckpointAnalysis, Confidence, DriftClass, DriftScore, DriftState, EvidenceRef,
    RepeatedCommandLoop, RepeatedFailureLoop,
};

const HISTORICAL_REPEATED_VERIFICATION_REASON_PREFIX: &str =
    "historical repeated verification evidence:";
const HISTORICAL_REPEATED_FAILURE_REASON_PREFIX: &str = "historical repeated failure evidence:";

pub(crate) fn score_dead_end_thrash(analysis: &CheckpointAnalysis) -> DriftScore {
    let has_history = !analysis.repetition.repeated_verification_loops.is_empty()
        || !analysis.repetition.repeated_failure_loops.is_empty();
    let flagged = has_history && !analysis.recovery.recovered_from_thrash;
    let raw_score = if flagged {
        active_raw_score(analysis)
    } else if has_history {
        20
    } else {
        0
    };
    let mut evidence = if flagged {
        current_thrashing_evidence(analysis)
    } else {
        historical_thrash_evidence(analysis)
    };
    dedupe_evidence(&mut evidence);

    DriftScore {
        class: DriftClass::DeadEndThrash,
        state: DriftState::Cleared,
        raw_score,
        confidence: if !analysis.repetition.repeated_verification_loops.is_empty() {
            Confidence::High
        } else if !analysis.repetition.repeated_failure_loops.is_empty()
            || !analysis.current.context.command_observations.is_empty()
        {
            Confidence::Medium
        } else {
            Confidence::Low
        },
        flagged,
        evidence,
    }
}

fn active_raw_score(analysis: &CheckpointAnalysis) -> u8 {
    ((analysis.repetition.repeated_verification_loops.len() * 40)
        + (analysis.repetition.repeated_failure_loops.len() * 30))
        .min(100) as u8
}

fn current_thrashing_evidence(analysis: &CheckpointAnalysis) -> Vec<EvidenceRef> {
    let mut evidence =
        repeated_verification_evidence(&analysis.repetition.repeated_verification_loops, false);
    evidence.extend(repeated_failure_evidence(
        &analysis.repetition.repeated_failure_loops,
        false,
    ));
    evidence
}

fn historical_thrash_evidence(analysis: &CheckpointAnalysis) -> Vec<EvidenceRef> {
    let mut evidence =
        repeated_verification_evidence(&analysis.repetition.repeated_verification_loops, true);
    evidence.extend(repeated_failure_evidence(
        &analysis.repetition.repeated_failure_loops,
        true,
    ));
    evidence
}

fn repeated_verification_evidence(
    loops: &[RepeatedCommandLoop],
    historical: bool,
) -> Vec<EvidenceRef> {
    loops.iter()
        .flat_map(|command_loop| {
            command_loop.evidence.iter().map(move |evidence| EvidenceRef {
                row: evidence.row.clone(),
                reason: if historical {
                    format!(
                        "{HISTORICAL_REPEATED_VERIFICATION_REASON_PREFIX} repeated verification command: {}",
                        command_loop.raw_command
                    )
                } else {
                    format!("repeated verification command: {}", command_loop.raw_command)
                },
            })
        })
        .collect()
}

fn repeated_failure_evidence(loops: &[RepeatedFailureLoop], historical: bool) -> Vec<EvidenceRef> {
    loops
        .iter()
        .flat_map(|failure_loop| {
            failure_loop
                .evidence
                .iter()
                .map(move |evidence| EvidenceRef {
                    row: evidence.row.clone(),
                    reason: if historical {
                        format!(
                            "{HISTORICAL_REPEATED_FAILURE_REASON_PREFIX} {}",
                            evidence.reason
                        )
                    } else {
                        evidence.reason.clone()
                    },
                })
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
