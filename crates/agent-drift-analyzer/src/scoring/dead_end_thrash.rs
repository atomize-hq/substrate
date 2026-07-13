use std::collections::BTreeSet;

use crate::checkpoint::{
    CheckpointAnalysis, Confidence, DriftClass, DriftScore, DriftState, EvidenceRef,
    ProgressDimension, ProgressSignal, ProgressSignalCode, RepeatedCommandLoop,
    RepeatedFailureLoop, SessionProgress,
};
use crate::scoring::{DriftStateHint, ScoredDrift};

const CHURN_WITH_PROGRESS_REASON_PREFIX: &str = "churn with progress evidence:";
const HISTORICAL_REPEATED_VERIFICATION_REASON_PREFIX: &str =
    "historical repeated verification evidence:";
const HISTORICAL_REPEATED_FAILURE_REASON_PREFIX: &str = "historical repeated failure evidence:";
const STALL_WITHOUT_FRONTIER_MOVEMENT_REASON_PREFIX: &str =
    "stall without frontier movement evidence:";

pub(crate) fn score_dead_end_thrash(
    analysis: &CheckpointAnalysis,
    session_progress: &SessionProgress,
) -> ScoredDrift {
    let has_history = !analysis.repetition.repeated_verification_loops.is_empty()
        || !analysis.repetition.repeated_failure_loops.is_empty();
    let would_flag_active = has_history
        && (analysis.recovery.active_repeated_failure
            || analysis.recovery.active_repeated_verification);

    if would_flag_active && frontier_advanced_in_interval(session_progress) {
        let mut evidence = churn_with_progress_evidence(analysis, session_progress);
        dedupe_evidence(&mut evidence);

        return ScoredDrift::new(
            DriftScore {
                class: DriftClass::DeadEndThrash,
                state: DriftState::Cleared,
                raw_score: 20,
                confidence: score_confidence(analysis, session_progress),
                flagged: false,
                evidence,
            },
            DriftStateHint::HistoricalContext,
        );
    }

    if would_flag_active {
        let mut evidence = stall_without_frontier_movement_evidence(analysis, session_progress);
        dedupe_evidence(&mut evidence);

        return ScoredDrift::new(
            DriftScore {
                class: DriftClass::DeadEndThrash,
                state: DriftState::Cleared,
                raw_score: decisive_stall_raw_score(analysis),
                confidence: score_confidence(analysis, session_progress),
                flagged: true,
                evidence,
            },
            DriftStateHint::HistoricalContext,
        );
    }

    let raw_score = if has_history { 20 } else { 0 };
    let flagged = false;
    let mut evidence = historical_thrash_evidence(analysis);
    dedupe_evidence(&mut evidence);

    ScoredDrift::new(
        DriftScore {
            class: DriftClass::DeadEndThrash,
            state: DriftState::Cleared,
            raw_score,
            confidence: score_confidence(analysis, session_progress),
            flagged,
            evidence,
        },
        if has_history {
            DriftStateHint::HistoricalContext
        } else {
            DriftStateHint::None
        },
    )
}

fn score_confidence(
    analysis: &CheckpointAnalysis,
    session_progress: &SessionProgress,
) -> Confidence {
    if !analysis.repetition.repeated_verification_loops.is_empty() {
        Confidence::High
    } else if !analysis.repetition.repeated_failure_loops.is_empty()
        || (!analysis.current.context.command_observations.is_empty()
            && session_progress.dimension != ProgressDimension::ParentVisibleOrchestration)
    {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

fn decisive_stall_raw_score(analysis: &CheckpointAnalysis) -> u8 {
    let has_failure_history = !analysis.repetition.repeated_failure_loops.is_empty();

    match (
        analysis.recovery.active_repeated_verification,
        analysis.recovery.active_repeated_failure,
        has_failure_history,
    ) {
        (true, true, _) => 70,
        (true, false, true) => 60,
        (true, false, false) => 40,
        (false, true, _) => 30,
        (false, false, _) => 0,
    }
}

fn current_thrashing_evidence(analysis: &CheckpointAnalysis) -> Vec<EvidenceRef> {
    let mut evidence = repeated_verification_evidence(
        &analysis.repetition.repeated_verification_loops,
        !analysis.recovery.active_repeated_verification,
    );
    evidence.extend(repeated_failure_evidence(
        &analysis.repetition.repeated_failure_loops,
        false,
    ));
    evidence
}

fn churn_with_progress_evidence(
    analysis: &CheckpointAnalysis,
    session_progress: &SessionProgress,
) -> Vec<EvidenceRef> {
    let mut evidence =
        direct_frontier_signal_evidence(session_progress, CHURN_WITH_PROGRESS_REASON_PREFIX);
    evidence.extend(historical_thrash_evidence(analysis));
    if evidence.is_empty() {
        evidence.extend(named_fallback_evidence(
            historical_thrash_evidence(analysis).first(),
            CHURN_WITH_PROGRESS_REASON_PREFIX,
        ));
    }
    evidence
}

fn stall_without_frontier_movement_evidence(
    analysis: &CheckpointAnalysis,
    session_progress: &SessionProgress,
) -> Vec<EvidenceRef> {
    let current_thrashing = current_thrashing_evidence(analysis);
    let mut evidence = non_advancing_frontier_signal_evidence(
        session_progress,
        STALL_WITHOUT_FRONTIER_MOVEMENT_REASON_PREFIX,
    );
    evidence.extend(named_fallback_evidence(
        current_thrashing.first(),
        STALL_WITHOUT_FRONTIER_MOVEMENT_REASON_PREFIX,
    ));
    evidence.extend(current_thrashing);
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

fn frontier_advanced_in_interval(session_progress: &SessionProgress) -> bool {
    session_progress.dimension == ProgressDimension::TroubleshootingFrontier
        && session_progress
            .signals
            .iter()
            .any(is_direct_troubleshooting_advancement_signal)
}

fn is_direct_troubleshooting_advancement_signal(signal: &ProgressSignal) -> bool {
    matches!(
        signal.code,
        ProgressSignalCode::FailureFrontierAdvanced
            | ProgressSignalCode::FailureCountReduced
            | ProgressSignalCode::VerificationClean
    )
}

fn direct_frontier_signal_evidence(
    session_progress: &SessionProgress,
    reason_prefix: &str,
) -> Vec<EvidenceRef> {
    if session_progress.dimension != ProgressDimension::TroubleshootingFrontier {
        return Vec::new();
    }

    session_progress
        .signals
        .iter()
        .filter(|signal| is_direct_troubleshooting_advancement_signal(signal))
        .flat_map(|signal| named_signal_evidence(signal, reason_prefix))
        .collect()
}

fn non_advancing_frontier_signal_evidence(
    session_progress: &SessionProgress,
    reason_prefix: &str,
) -> Vec<EvidenceRef> {
    if session_progress.dimension != ProgressDimension::TroubleshootingFrontier {
        return Vec::new();
    }

    session_progress
        .signals
        .iter()
        .filter(|signal| !is_direct_troubleshooting_advancement_signal(signal))
        .flat_map(|signal| named_signal_evidence(signal, reason_prefix))
        .collect()
}

fn named_signal_evidence(signal: &ProgressSignal, reason_prefix: &str) -> Vec<EvidenceRef> {
    signal
        .evidence
        .iter()
        .map(|evidence| EvidenceRef {
            row: evidence.row.clone(),
            reason: format!("{reason_prefix} {}", signal.summary),
        })
        .collect()
}

fn named_fallback_evidence(
    evidence: Option<&EvidenceRef>,
    reason_prefix: &str,
) -> Vec<EvidenceRef> {
    evidence
        .map(|evidence| {
            vec![EvidenceRef {
                row: evidence.row.clone(),
                reason: format!("{reason_prefix} {}", evidence.reason),
            }]
        })
        .unwrap_or_default()
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
