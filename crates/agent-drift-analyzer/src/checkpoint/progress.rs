use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use agent_session_compactor::RowRef;

use crate::checkpoint::{ChildWorkVisibility, DelegationTopology};

use super::attempt::{
    verification_target_from_command, AttemptOutcome, CommandAttempt, CommandAttemptRole,
    ExerciseState, VerificationAttempt, VerificationScope, VerifierKind,
};
use super::diagnostics::{
    classify_attempt_scope_edit_overlap, classify_edit_overlap, match_diagnostic_signatures,
    DiagnosticMatchKind, DiagnosticSignature, EditOverlapStrength, FailureClass,
};
use super::{
    CheckpointAnalysis, Confidence, EvidenceRef, ProgressDimension, ProgressSignal,
    ProgressSignalCode, ProgressStatus, SessionArchetype, SessionArchetypeLabel, SessionProgress,
    SignalPolarity, SignalStrength,
};

const MAX_PROGRESS_EVIDENCE_ITEMS: usize = 8;
const DELEGATION_LIMITING_CONFIDENCE_REASON: &str =
    "delegation visibility limited progress confidence";
const ZERO_VERIFIER_ANTI_FLAP_REASON: &str =
    "diffused exploratory activity lacked verifier-backed or explicit failure evidence";

pub(crate) fn build_session_progress(
    analysis: &CheckpointAnalysis,
    session_archetype: &SessionArchetype,
) -> SessionProgress {
    let mut progress = if let Some(progress) =
        parent_visible_orchestration_progress(analysis, session_archetype.label)
    {
        progress
    } else {
        match session_archetype.label {
            SessionArchetypeLabel::Troubleshooting => assess_troubleshooting_progress(
                analysis,
                default_dimension(session_archetype.label),
                session_archetype.label,
            ),
            SessionArchetypeLabel::Planning => {
                assess_planning_progress(analysis, default_dimension(session_archetype.label))
            }
            SessionArchetypeLabel::AutonomousImplementation => assess_implementation_progress(
                analysis,
                default_dimension(session_archetype.label),
                session_archetype.label,
            ),
            SessionArchetypeLabel::VerificationCloseout => assess_closeout_progress(
                analysis,
                default_dimension(session_archetype.label),
                session_archetype.label,
            ),
        }
    };

    if session_archetype.label == SessionArchetypeLabel::Troubleshooting
        && progress.dimension == ProgressDimension::TroubleshootingFrontier
        && progress.status == ProgressStatus::InsufficientEvidence
        && verifier_failure_evidence(analysis).is_empty()
        && source_edits(analysis).is_empty()
        && zero_verifier_exploratory_interval(analysis)
        && explicit_failure_evidence(analysis).is_empty()
    {
        let planning_progress =
            assess_planning_progress(analysis, ProgressDimension::PlanningConvergence);
        if !matches!(
            planning_progress.status,
            ProgressStatus::Advancing | ProgressStatus::Mixed
        ) {
            progress = annotate_zero_verifier_fallback(analysis, planning_progress);
        }
    }

    progress = apply_delegation_caps(analysis, progress);
    finalize_progress(progress)
}

fn default_dimension(label: SessionArchetypeLabel) -> ProgressDimension {
    match label {
        SessionArchetypeLabel::Troubleshooting => ProgressDimension::TroubleshootingFrontier,
        SessionArchetypeLabel::Planning => ProgressDimension::PlanningConvergence,
        SessionArchetypeLabel::AutonomousImplementation => {
            ProgressDimension::ImplementationVerificationWall
        }
        SessionArchetypeLabel::VerificationCloseout => {
            ProgressDimension::VerificationCloseoutNarrowing
        }
    }
}

fn explicit_failure_evidence(analysis: &CheckpointAnalysis) -> Vec<EvidenceRef> {
    let repeated_failure_row_keys = analysis
        .repetition
        .repeated_failure_loops
        .iter()
        .flat_map(|failure_loop| failure_loop.evidence.iter())
        .map(evidence_row_key)
        .collect::<BTreeSet<_>>();
    let compact_rows_by_key = analysis
        .interval
        .compact_rows
        .iter()
        .map(|row| {
            (
                (row.source_file.clone(), row.event_index, row.row_ordinal),
                row,
            )
        })
        .collect::<BTreeMap<_, _>>();

    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| {
            attempt.outcome == AttemptOutcome::Failed
                && !matches!(
                    attempt.role,
                    CommandAttemptRole::Read
                        | CommandAttemptRole::VcsInspection
                        | CommandAttemptRole::Orchestration
                )
        })
        .flat_map(|attempt| {
            let repeated = attempt.output_rows.iter().any(|row| {
                repeated_failure_row_keys.contains(&(
                    row.source_file.clone(),
                    row.event_index,
                    row.row_ordinal,
                ))
            });
            if repeated {
                if touches_only_unrelated_paths(analysis, attempt)
                    || attempt_is_nondiagnostic_probe_failure(attempt, &compact_rows_by_key)
                {
                    return Vec::new();
                }
                return attempt_evidence(
                    attempt,
                    "decisive repeated failure evidence kept the interval on the troubleshooting frontier",
                );
            }
            if !touches_only_unrelated_paths(analysis, attempt)
                && !attempt_is_nondiagnostic_probe_failure(attempt, &compact_rows_by_key)
            {
                return attempt_evidence(
                    attempt,
                    "explicit failure evidence kept the interval on the troubleshooting frontier",
                );
            }
            Vec::new()
        })
        .collect()
}

fn verifier_failure_evidence(analysis: &CheckpointAnalysis) -> Vec<EvidenceRef> {
    let compact_rows_by_key = analysis
        .interval
        .compact_rows
        .iter()
        .map(|row| {
            (
                (row.source_file.clone(), row.event_index, row.row_ordinal),
                row,
            )
        })
        .collect::<BTreeMap<_, _>>();

    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| {
            matches!(
                attempt.role,
                CommandAttemptRole::Compile
                    | CommandAttemptRole::Test
                    | CommandAttemptRole::Lint
                    | CommandAttemptRole::FormatCheck
                    | CommandAttemptRole::Build
                    | CommandAttemptRole::Replay
            )
        })
        .flat_map(|attempt| match attempt.outcome {
            AttemptOutcome::Failed
                if !attempt_is_nondiagnostic_probe_failure(attempt, &compact_rows_by_key) =>
            {
                attempt_evidence(
                    attempt,
                    "verification failure evidence kept the interval on the troubleshooting frontier",
                )
            }
            AttemptOutcome::Failed => Vec::new(),
            AttemptOutcome::Unknown
                if !attempt_has_output_matching(attempt, &compact_rows_by_key, |text| {
                    text.contains("aborted by user")
                }) =>
            {
                attempt_evidence(
                    attempt,
                    "unresolved verifier outcome kept the interval on the troubleshooting frontier",
                )
            }
            AttemptOutcome::Clean | AttemptOutcome::Unknown => Vec::new(),
        })
        .collect()
}

fn attempt_is_nondiagnostic_probe_failure(
    attempt: &CommandAttempt,
    compact_rows_by_key: &BTreeMap<
        (camino::Utf8PathBuf, usize, usize),
        &agent_session_compactor::CompactionRow,
    >,
) -> bool {
    attempt_has_output_matching(attempt, compact_rows_by_key, |text| {
        let body = text
            .split_once("Output:\n")
            .map(|(_, body)| body)
            .unwrap_or(text);
        let body = body.trim();
        is_probe_discovery_attempt(attempt)
            || body.is_empty()
            || body.contains("Could not find files for the given pattern(s).")
            || body.contains("command timed out after")
            || body.contains("Entry not found")
            || (attempt.raw_command.contains("compileall")
                && (body.contains("PermissionError") || body.contains("Access is denied")))
    })
}

fn is_probe_discovery_attempt(attempt: &CommandAttempt) -> bool {
    matches!(
        attempt.role,
        CommandAttemptRole::Read | CommandAttemptRole::VcsInspection
    ) || matches!(
        attempt.family.as_str(),
        "rg" | "grep"
            | "Select-String"
            | "where.exe"
            | "Get-ChildItem"
            | "Get-Command"
            | "Test-Path"
            | "Invoke-WebRequest"
    ) || is_read_only_python_probe(attempt)
        || (attempt.family == "git" && attempt.raw_command.contains(" status"))
}

fn is_read_only_python_probe(attempt: &CommandAttempt) -> bool {
    let family = attempt.family.to_ascii_lowercase();
    let raw_command = attempt.raw_command.to_ascii_lowercase();
    if !(family == "python"
        || family == "python3"
        || family.ends_with("python.exe")
        || family.ends_with("/python")
        || family.ends_with("\\python")
        || family.ends_with("/python.exe")
        || family.ends_with("\\python.exe"))
    {
        return false;
    }

    if !(raw_command.contains(" -c ")
        || raw_command.contains(" -c\"")
        || raw_command.contains(" -c'")
        || raw_command.contains("<<'py'")
        || raw_command.contains("<<\"py\"")
        || raw_command.contains("<<py"))
    {
        return false;
    }

    let read_only_markers = [
        ".read_text(",
        "ast.parse(",
        "importlib.util.find_spec(",
        "brace_balance",
        "begin_count",
        "end_count",
    ];
    let mutating_markers = [
        ".write_text(",
        ".write_bytes(",
        ".mkdir(",
        "makedirs(",
        "json.dump(",
        "to_csv(",
        "savefig(",
        "shutil.copy",
        "shutil.move",
        "rename(",
        "unlink(",
        "rmtree(",
    ];

    read_only_markers
        .iter()
        .any(|marker| raw_command.contains(marker))
        && !mutating_markers
            .iter()
            .any(|marker| raw_command.contains(marker))
}

fn attempt_has_output_matching(
    attempt: &CommandAttempt,
    compact_rows_by_key: &BTreeMap<
        (camino::Utf8PathBuf, usize, usize),
        &agent_session_compactor::CompactionRow,
    >,
    predicate: impl Fn(&str) -> bool,
) -> bool {
    attempt.output_rows.iter().any(|row| {
        compact_rows_by_key
            .get(&(row.source_file.clone(), row.event_index, row.row_ordinal))
            .map(|compact_row| predicate(&compact_row.text))
            .unwrap_or(false)
    })
}

fn annotate_zero_verifier_fallback(
    analysis: &CheckpointAnalysis,
    mut planning_progress: SessionProgress,
) -> SessionProgress {
    let fallback_evidence = merge_evidence(vec![
        analysis
            .interval
            .command_attempts
            .iter()
            .flat_map(|attempt| attempt_evidence(attempt, ZERO_VERIFIER_ANTI_FLAP_REASON))
            .collect(),
        task_frame_evidence(analysis, ZERO_VERIFIER_ANTI_FLAP_REASON),
        previous_task_frame_evidence(analysis, ZERO_VERIFIER_ANTI_FLAP_REASON),
    ]);
    if fallback_evidence.is_empty() {
        return planning_progress;
    }

    if planning_progress.status == ProgressStatus::InsufficientEvidence
        && planning_progress.signals.is_empty()
        && planning_progress.counter_evidence.is_empty()
    {
        if !working_set_is_diffused(analysis) {
            return insufficient_progress(
                planning_progress.dimension,
                None,
                Some(fallback_evidence),
            );
        }
        return insufficient_progress(
            planning_progress.dimension,
            Some(progress_signal(
                ProgressSignalCode::WorkingSetDiffused,
                SignalPolarity::Limiting,
                SignalStrength::Moderate,
                "diffused exploratory activity stayed conservative because verifier-backed or explicit failure evidence was absent",
                None,
                Some(set_preview(&working_set(
                    &analysis.current.task_frame.working_set_paths,
                ))),
                fallback_evidence.clone(),
            )),
            Some(fallback_evidence),
        );
    }

    planning_progress.counter_evidence.extend(fallback_evidence);
    planning_progress
}

fn parent_visible_orchestration_progress(
    analysis: &CheckpointAnalysis,
    archetype_label: SessionArchetypeLabel,
) -> Option<SessionProgress> {
    let topology = analysis.delegation.topology?;
    let visibility = effective_child_work_visibility(analysis);
    if !matches!(
        topology,
        DelegationTopology::DelegatingParent | DelegationTopology::MixedOrAmbiguous
    ) {
        return None;
    }

    let orchestration_attempts = parent_visible_orchestration_attempts(analysis);
    let synthesis_attempts = parent_visible_synthesis_attempts(analysis);
    let visible_child_surface = has_visible_child_surface(analysis);
    let prior_parent_visible = previous_checkpoint_was_comparable_parent_visible(analysis);
    let conservative_stall_evidence =
        parent_visible_conservative_stall_evidence(analysis, visibility);
    let comparable_parent_visible_followup =
        analysis.interval.command_attempts.is_empty() && prior_parent_visible;
    let parent_visible_synthesis_case = visible_child_surface
        && !synthesis_attempts.is_empty()
        && source_edits(analysis).is_empty();
    if !parent_visible_synthesis_case
        && !comparable_parent_visible_followup
        && ((orchestration_attempts.is_empty() && synthesis_attempts.is_empty())
            || !has_parent_visible_orchestration_evidence(analysis, visibility))
    {
        return None;
    }

    let child_opaque_limiting_evidence = analysis
        .delegation
        .counter_evidence
        .iter()
        .map(|evidence| EvidenceRef {
            row: evidence.row.clone(),
            reason: "child-opaque delegation limited direct child progress claims".to_string(),
        })
        .collect::<Vec<_>>();
    let limiting_evidence = merge_evidence(vec![
        orchestration_attempts
            .iter()
            .flat_map(|attempt| attempt_evidence(*attempt, "parent-visible orchestration row"))
            .collect(),
        parent_visible_synthesis_evidence(&synthesis_attempts),
        delegation_supporting_evidence(
            analysis,
            "only parent-visible orchestration evidence was available for progress assessment",
        ),
        child_opaque_limiting_evidence.clone(),
    ]);
    let limiting_signal = progress_signal(
        ProgressSignalCode::DelegationVisibilityLimited,
        SignalPolarity::Limiting,
        SignalStrength::Strong,
        delegation_limiting_summary(topology, visibility),
        Some(format!("{:?}", archetype_label).to_ascii_lowercase()),
        Some("parent_visible_orchestration".to_string()),
        limiting_evidence,
    );

    let parent_synthesis = has_parent_visible_synthesis(
        visibility,
        visible_child_surface,
        &orchestration_attempts,
        &synthesis_attempts,
    );
    let status = if parent_synthesis {
        ProgressStatus::Mixed
    } else if prior_parent_visible || !conservative_stall_evidence.is_empty() {
        ProgressStatus::Stalled
    } else {
        ProgressStatus::InsufficientEvidence
    };

    let confidence = match visibility {
        ChildWorkVisibility::Opaque => Confidence::Low,
        ChildWorkVisibility::Linked => Confidence::Medium,
        ChildWorkVisibility::Partial => Confidence::Medium,
        ChildWorkVisibility::None => Confidence::Low,
    };

    let progress = if status == ProgressStatus::InsufficientEvidence {
        insufficient_progress(
            ProgressDimension::ParentVisibleOrchestration,
            Some(limiting_signal),
            Some(child_opaque_limiting_evidence),
        )
    } else {
        progress_from_signals(
            status,
            ProgressDimension::ParentVisibleOrchestration,
            confidence,
            vec![limiting_signal],
            merge_evidence(vec![
                child_opaque_limiting_evidence,
                conservative_stall_evidence,
            ]),
        )
    };

    Some(progress)
}

fn parent_visible_conservative_stall_evidence(
    analysis: &CheckpointAnalysis,
    visibility: ChildWorkVisibility,
) -> Vec<EvidenceRef> {
    if visibility != ChildWorkVisibility::Opaque || !plan_artifact_edits(analysis).is_empty() {
        return Vec::new();
    }

    let read_attempts = analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| attempt.role == CommandAttemptRole::Read)
        .collect::<Vec<_>>();
    if read_attempts.len() < 2 || !working_set_is_diffused(analysis) {
        return Vec::new();
    }

    read_attempts
        .into_iter()
        .flat_map(|attempt| {
            attempt_evidence(
                attempt,
                "repeated broad planning scan without convergence artifact",
            )
        })
        .collect()
}

fn assess_troubleshooting_progress(
    analysis: &CheckpointAnalysis,
    dimension: ProgressDimension,
    archetype_label: SessionArchetypeLabel,
) -> SessionProgress {
    let current_attempts = &analysis.interval.verification_attempts;
    if current_attempts.is_empty() {
        return insufficient_progress(dimension, None, None);
    }

    let lane_results = current_attempts
        .iter()
        .enumerate()
        .filter(|(current_index, current)| {
            !current_attempts[current_index + 1..]
                .iter()
                .any(|later| attempts_are_comparable(current, later))
        })
        .map(|(current_index, current)| {
            let current_lane_window = &current_attempts[..=current_index];
            let prior_attempts =
                comparable_attempts(analysis, current_lane_window, current, archetype_label);
            (
                current.command_row.event_index,
                assess_troubleshooting_tail(analysis, dimension, current, &prior_attempts),
            )
        })
        .collect::<Vec<_>>();

    let newest_informative_tail_event = lane_results
        .iter()
        .rev()
        .find(|(_, progress)| progress.status != ProgressStatus::InsufficientEvidence)
        .map(|(tail_event, _)| *tail_event);
    let latest_preceding_edit_event = newest_informative_tail_event.and_then(|tail_event| {
        analysis
            .interval
            .command_attempts
            .iter()
            .filter(|attempt| attempt.role == CommandAttemptRole::Edit)
            .filter(|attempt| attempt.command_row.event_index < tail_event)
            .filter(|attempt| {
                attempt
                    .paths
                    .iter()
                    .any(|path| is_source_path(path) || is_test_path(path))
            })
            .map(|attempt| attempt.command_row.event_index)
            .max()
    });
    let lane_results = lane_results
        .into_iter()
        .filter(|(tail_event, _)| {
            latest_preceding_edit_event.is_none_or(|edit_event| *tail_event > edit_event)
        })
        .map(|(_, progress)| progress)
        .collect();

    aggregate_troubleshooting_lanes(dimension, lane_results)
}

fn aggregate_troubleshooting_lanes(
    dimension: ProgressDimension,
    lane_results: Vec<SessionProgress>,
) -> SessionProgress {
    if lane_results.len() == 1 {
        return lane_results
            .into_iter()
            .next()
            .expect("one troubleshooting lane");
    }
    let informative = lane_results
        .into_iter()
        .filter(|progress| progress.status != ProgressStatus::InsufficientEvidence)
        .collect::<Vec<_>>();
    if informative.is_empty() {
        return insufficient_progress(dimension, None, None);
    }
    if informative.len() == 1 {
        return informative
            .into_iter()
            .next()
            .expect("one informative lane");
    }

    let has_status = |status| informative.iter().any(|progress| progress.status == status);
    let has_advancing = has_status(ProgressStatus::Advancing);
    let has_negative =
        has_status(ProgressStatus::Stalled) || has_status(ProgressStatus::Regressing);
    let status = if has_status(ProgressStatus::Mixed) || (has_advancing && has_negative) {
        ProgressStatus::Mixed
    } else if has_status(ProgressStatus::Regressing) {
        ProgressStatus::Regressing
    } else if has_status(ProgressStatus::Stalled) {
        ProgressStatus::Stalled
    } else {
        ProgressStatus::Advancing
    };
    let mut confidence = informative
        .iter()
        .map(|progress| progress.confidence)
        .min()
        .expect("multiple informative lanes");
    if status == ProgressStatus::Mixed {
        confidence = confidence.min(Confidence::Medium);
    }

    let mut signals = Vec::new();
    let mut counter_evidence = Vec::new();
    for lane in informative {
        signals.extend(lane.signals);
        counter_evidence.extend(lane.counter_evidence);
    }
    progress_from_signals(status, dimension, confidence, signals, counter_evidence)
}

fn assess_troubleshooting_tail(
    analysis: &CheckpointAnalysis,
    dimension: ProgressDimension,
    current: &VerificationAttempt,
    prior_attempts: &[VerificationAttempt],
) -> SessionProgress {
    let best_failed = best_failed_attempt(prior_attempts);
    let best_clean = best_clean_attempt(prior_attempts);
    let latest_failed = latest_failed_attempt(prior_attempts);

    if attempt_is_clean_proof(current) {
        if let Some(previous_failed) = best_failed {
            let mut signals = vec![progress_signal(
                ProgressSignalCode::VerificationClean,
                SignalPolarity::Positive,
                SignalStrength::Strong,
                "focused verification reached a clean result after prior failure",
                Some(previous_failed.target_scope.raw.clone()),
                Some(current.target_scope.raw.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier failing verification attempt"),
                    attempt_evidence(current, "later clean verification attempt"),
                ]),
            )];
            if scope_is_broader(&current.target_scope, &previous_failed.target_scope) {
                signals.push(progress_signal(
                    ProgressSignalCode::VerificationScopeBroadened,
                    SignalPolarity::Positive,
                    SignalStrength::Moderate,
                    "clean proof broadened after focused verification passed",
                    Some(previous_failed.target_scope.raw.clone()),
                    Some(current.target_scope.raw.clone()),
                    merge_evidence(vec![
                        attempt_evidence(previous_failed, "earlier focused verification scope"),
                        attempt_evidence(current, "later broader verification scope"),
                    ]),
                ));
            }
            return progress_from_signals(
                ProgressStatus::Advancing,
                dimension,
                Confidence::High,
                signals,
                Vec::new(),
            );
        }
        return insufficient_progress(dimension, None, None);
    }

    if current.outcome == AttemptOutcome::Clean {
        return insufficient_progress(dimension, None, None);
    }

    if let Some(previous_clean) = best_clean {
        return progress_from_signals(
            ProgressStatus::Regressing,
            dimension,
            Confidence::High,
            vec![progress_signal(
                ProgressSignalCode::PreviouslyCleanScopeBroken,
                SignalPolarity::Negative,
                SignalStrength::Strong,
                "a previously clean troubleshooting scope is failing again",
                Some(previous_clean.target_scope.raw.clone()),
                Some(current.target_scope.raw.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous_clean, "earlier clean verification attempt"),
                    attempt_evidence(current, "later failing verification attempt"),
                ]),
            )],
            Vec::new(),
        );
    }

    if let Some(previous_failed) =
        best_failed.filter(|candidate| failed_attempt_frontier_cmp(candidate, current).is_gt())
    {
        return progress_from_signals(
            ProgressStatus::Regressing,
            dimension,
            Confidence::High,
            vec![progress_signal(
                ProgressSignalCode::PreviouslyCleanScopeBroken,
                SignalPolarity::Negative,
                SignalStrength::Strong,
                "the troubleshooting frontier fell back behind a previously later comparable verifier result",
                Some(verification_attempt_preview(previous_failed)),
                Some(verification_attempt_preview(current)),
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier later-stage verification attempt"),
                    attempt_evidence(current, "later regressing verification attempt"),
                ]),
            )],
            Vec::new(),
        );
    }

    if current.exercise_state == ExerciseState::BlockedBeforeTarget {
        return insufficient_progress(
            dimension,
            Some(progress_signal(
                ProgressSignalCode::TargetNotExercised,
                SignalPolarity::Limiting,
                SignalStrength::Moderate,
                "verification stayed blocked before the requested target executed",
                None,
                Some(current.target_scope.raw.clone()),
                attempt_evidence(current, "verifier did not exercise the requested target"),
            )),
            None,
        );
    }

    let Some(previous_failed) = latest_failed else {
        return insufficient_progress(dimension, None, None);
    };

    let current_signature = best_signature(current);
    let previous_signature = best_signature(previous_failed);
    let edit_overlap = classify_edit_overlap(
        previous_failed,
        current,
        &analysis.interval.command_attempts,
    );
    let mut signals = Vec::new();
    let mut positive_confidence = Confidence::Medium;

    if previous_failed.exercise_state == ExerciseState::BlockedBeforeTarget
        && current.exercise_state == ExerciseState::TargetExercised
    {
        signals.push(progress_signal(
            ProgressSignalCode::FailureFrontierAdvanced,
            SignalPolarity::Positive,
            SignalStrength::Strong,
            "verification progressed from a blocked frontier to target execution",
            Some(previous_failed.target_scope.raw.clone()),
            Some(current.target_scope.raw.clone()),
            merge_evidence(vec![
                attempt_evidence(previous_failed, "earlier blocked verification attempt"),
                attempt_evidence(current, "later target-exercising verification attempt"),
            ]),
        ));
    }

    if let (Some(previous_signature), Some(current_signature)) =
        (previous_signature, current_signature)
    {
        let match_kind = match_diagnostic_signatures(previous_signature, current_signature);
        let frontier_advanced = frontier_rank(current_signature.failure_class)
            > frontier_rank(previous_signature.failure_class);
        if frontier_advanced {
            positive_confidence = positive_confidence.max(troubleshooting_advancing_confidence(
                previous_failed,
                current,
                Some(previous_signature),
                Some(current_signature),
            ));
            signals.push(progress_signal(
                ProgressSignalCode::FailureFrontierAdvanced,
                SignalPolarity::Positive,
                SignalStrength::Strong,
                "failure frontier moved later in the troubleshooting pipeline",
                Some(previous_signature.preview.clone()),
                Some(current_signature.preview.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier failing verification attempt"),
                    attempt_evidence(current, "later failing verification attempt"),
                    edit_overlap.evidence.clone(),
                ]),
            ));
        }
        let fail_count_delta = comparable_fail_count_delta(previous_signature, current_signature);
        if matches!(fail_count_delta, Some(Ordering::Less)) {
            positive_confidence = positive_confidence.max(troubleshooting_advancing_confidence(
                previous_failed,
                current,
                Some(previous_signature),
                Some(current_signature),
            ));
            signals.push(progress_signal(
                ProgressSignalCode::FailureCountReduced,
                SignalPolarity::Positive,
                SignalStrength::Moderate,
                "the failing count dropped on the same troubleshooting scope",
                previous_signature
                    .failing_count
                    .map(|count| count.to_string()),
                current_signature
                    .failing_count
                    .map(|count| count.to_string()),
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier failing verification attempt"),
                    attempt_evidence(current, "later failing verification attempt"),
                ]),
            ));
        }
        if matches!(fail_count_delta, Some(Ordering::Greater)) {
            signals.push(progress_signal(
                ProgressSignalCode::FailureCountIncreased,
                SignalPolarity::Negative,
                SignalStrength::Moderate,
                "the failing count increased on the same troubleshooting scope",
                previous_signature
                    .failing_count
                    .map(|count| count.to_string()),
                current_signature
                    .failing_count
                    .map(|count| count.to_string()),
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier failing verification attempt"),
                    attempt_evidence(current, "later failing verification attempt"),
                ]),
            ));
        }
        let repeated_signature = matches!(
            match_kind,
            DiagnosticMatchKind::Exact | DiagnosticMatchKind::StrongFuzzy
        );
        if repeated_signature {
            signals.push(progress_signal(
                ProgressSignalCode::FailureSignatureRepeated,
                SignalPolarity::Negative,
                SignalStrength::Strong,
                if edit_overlap.strength >= EditOverlapStrength::Moderate {
                    "the same normalized failure signature repeated despite an overlapping fix edit"
                } else {
                    "the same normalized failure signature repeated without an overlapping fix edit"
                },
                Some(previous_signature.preview.clone()),
                Some(current_signature.preview.clone()),
                merge_evidence(vec![
                    attempt_evidence(
                        previous_failed,
                        "earlier repeated failing verification attempt",
                    ),
                    attempt_evidence(current, "later repeated failing verification attempt"),
                    edit_overlap.evidence.clone(),
                ]),
            ));
        }
        if repeated_signature && edit_overlap.strength == EditOverlapStrength::None {
            signals.push(progress_signal(
                ProgressSignalCode::FailingScopeUnchanged,
                SignalPolarity::Negative,
                SignalStrength::Moderate,
                "the failing troubleshooting scope stayed unchanged",
                None,
                None,
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier failing scope"),
                    attempt_evidence(current, "later unchanged failing scope"),
                ]),
            ));
        }
        if edit_overlap.strength >= EditOverlapStrength::Moderate
            && (frontier_advanced
                || repeated_signature
                || matches!(match_kind, DiagnosticMatchKind::FrontierRelated))
        {
            signals.push(progress_signal(
                ProgressSignalCode::FailingScopeEdited,
                SignalPolarity::Positive,
                SignalStrength::Moderate,
                "intervening edits overlapped the failing troubleshooting scope",
                None,
                None,
                edit_overlap.evidence.clone(),
            ));
        }
        if frontier_rank(current_signature.failure_class)
            < frontier_rank(previous_signature.failure_class)
        {
            signals.push(progress_signal(
                ProgressSignalCode::PreviouslyCleanScopeBroken,
                SignalPolarity::Negative,
                SignalStrength::Strong,
                "the troubleshooting frontier moved earlier after prior progress",
                Some(previous_signature.preview.clone()),
                Some(current_signature.preview.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier later-stage failure"),
                    attempt_evidence(current, "later earlier-stage failure"),
                ]),
            ));
        }
    }

    classify_progress_outcome(dimension, signals, positive_confidence, Confidence::Medium)
}

fn assess_planning_progress(
    analysis: &CheckpointAnalysis,
    dimension: ProgressDimension,
) -> SessionProgress {
    let read_count = analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| attempt.role == CommandAttemptRole::Read)
        .count();
    let plan_edits = plan_artifact_edits(analysis);
    let current_working_set = working_set(&analysis.current.task_frame.working_set_paths);
    let previous_working_set = analysis
        .previous
        .as_ref()
        .map(|slice| working_set(&slice.task_frame.working_set_paths))
        .unwrap_or_default();
    let narrowed = working_set_narrowed(&previous_working_set, &current_working_set)
        || truth_artifacts_narrowed(analysis)
        || (!plan_edits.is_empty() && current_working_set.len() <= 3);
    let diffused = working_set_diffused(&previous_working_set, &current_working_set);

    let mut signals = Vec::new();

    if !plan_edits.is_empty() {
        let created = plan_edits
            .iter()
            .any(|attempt| plan_artifact_created(analysis, attempt));
        signals.push(progress_signal(
            if created {
                ProgressSignalCode::PlanArtifactCreated
            } else {
                ProgressSignalCode::PlanArtifactRefined
            },
            SignalPolarity::Positive,
            SignalStrength::Moderate,
            "planning produced or refined a concrete truth artifact",
            None,
            None,
            plan_edits
                .iter()
                .flat_map(|attempt| {
                    attempt_evidence(*attempt, "plan/spec/tasks/design artifact edit")
                })
                .collect(),
        ));
    }
    if narrowed {
        signals.push(progress_signal(
            ProgressSignalCode::CandidateSetNarrowed,
            SignalPolarity::Positive,
            SignalStrength::Moderate,
            "the planning candidate set narrowed toward a concrete artifact family",
            Some(set_preview(&previous_working_set)),
            Some(set_preview(&current_working_set)),
            merge_evidence(vec![
                previous_task_frame_evidence(analysis, "earlier planning working set was broader"),
                task_frame_evidence(
                    analysis,
                    "current planning working set concentrated on fewer artifacts",
                ),
            ]),
        ));
    }
    if !current_working_set.is_empty() && current_working_set.len() <= 3 {
        signals.push(progress_signal(
            ProgressSignalCode::WorkingSetConcentrated,
            SignalPolarity::Positive,
            SignalStrength::Weak,
            "the planning working set stayed concentrated",
            None,
            Some(set_preview(&current_working_set)),
            task_frame_evidence(
                analysis,
                "planning working set remained concentrated on a small artifact family",
            ),
        ));
    }
    if diffused {
        signals.push(progress_signal(
            ProgressSignalCode::CandidateSetExpanded,
            SignalPolarity::Negative,
            SignalStrength::Moderate,
            "broad planning scans expanded the visible candidate set",
            Some(set_preview(&previous_working_set)),
            Some(set_preview(&current_working_set)),
            merge_evidence(vec![
                previous_task_frame_evidence(
                    analysis,
                    "earlier planning working set was more focused",
                ),
                task_frame_evidence(
                    analysis,
                    "planning working set expanded instead of narrowing",
                ),
            ]),
        ));
    }

    if plan_edits.is_empty() && read_count >= 2 && diffused {
        return progress_from_signals(
            ProgressStatus::Stalled,
            dimension,
            Confidence::Medium,
            signals,
            analysis
                .interval
                .command_attempts
                .iter()
                .filter(|attempt| attempt.role == CommandAttemptRole::Read)
                .flat_map(|attempt| {
                    attempt_evidence(
                        attempt,
                        "repeated broad planning scan without convergence artifact",
                    )
                })
                .collect(),
        );
    }

    if !plan_edits.is_empty() && narrowed {
        let counter = negative_signal_evidence(&signals);
        if diffused {
            return progress_from_signals(
                ProgressStatus::Mixed,
                dimension,
                Confidence::Medium,
                signals,
                counter,
            );
        }
        return progress_from_signals(
            ProgressStatus::Advancing,
            dimension,
            Confidence::Medium,
            signals,
            Vec::new(),
        );
    }

    insufficient_progress(dimension, None, None)
}

fn assess_implementation_progress(
    analysis: &CheckpointAnalysis,
    dimension: ProgressDimension,
    archetype_label: SessionArchetypeLabel,
) -> SessionProgress {
    let source_edits = source_edits(analysis);
    let test_edits = test_edits(analysis);
    let current_attempts = &analysis.interval.verification_attempts;
    let Some(current) = current_attempts.last() else {
        return insufficient_progress(dimension, None, None);
    };
    let prior_attempts = comparable_attempts(analysis, current_attempts, current, archetype_label);
    let best_failed = best_failed_attempt(&prior_attempts);
    let best_clean = best_clean_attempt(&prior_attempts);
    let latest_failed = latest_failed_attempt(&prior_attempts);
    let concentrated = working_set_is_concentrated(analysis);
    let diffused = working_set_is_diffused(analysis);

    let mut signals = Vec::new();
    let mut counter_evidence = Vec::new();
    let mut verifier_progress_with_overlap = false;

    if concentrated {
        signals.push(progress_signal(
            ProgressSignalCode::WorkingSetConcentrated,
            SignalPolarity::Positive,
            SignalStrength::Moderate,
            "source edits stayed concentrated in the active implementation working set",
            None,
            Some(set_preview(&working_set(
                &analysis.current.task_frame.working_set_paths,
            ))),
            merge_evidence(vec![
                source_edits
                    .iter()
                    .flat_map(|attempt| {
                        attempt_evidence(
                            *attempt,
                            "source edit stayed inside the active working set",
                        )
                    })
                    .collect(),
                task_frame_evidence(
                    analysis,
                    "stable working set plus source edits supported concentrated implementation",
                ),
            ]),
        ));
    }
    if diffused {
        signals.push(progress_signal(
            ProgressSignalCode::WorkingSetDiffused,
            SignalPolarity::Negative,
            SignalStrength::Moderate,
            "implementation edits diffused into unrelated working-set paths",
            None,
            Some(set_preview(&working_set(
                &analysis.current.task_frame.working_set_paths,
            ))),
            task_frame_evidence(
                analysis,
                "working set widened during implementation instead of staying concentrated",
            ),
        ));
    }

    if attempt_is_clean_proof(current) {
        if let Some(previous_failed) = latest_failed {
            let edit_overlap = classify_attempt_scope_edit_overlap(
                previous_failed,
                current,
                &analysis.interval.command_attempts,
            );
            let scope_aligned = edit_overlap.strength >= EditOverlapStrength::Moderate
                || working_set_overlaps_verifier_scope(analysis, previous_failed, current);
            signals.push(progress_signal(
                ProgressSignalCode::VerificationClean,
                SignalPolarity::Positive,
                SignalStrength::Strong,
                "the active implementation slice reached clean verification",
                Some(previous_failed.target_scope.raw.clone()),
                Some(current.target_scope.raw.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier failing implementation verifier"),
                    attempt_evidence(current, "later clean implementation verifier"),
                    edit_overlap.evidence.clone(),
                ]),
            ));
            if scope_is_broader(&current.target_scope, &previous_failed.target_scope) {
                signals.push(progress_signal(
                    ProgressSignalCode::VerificationScopeBroadened,
                    SignalPolarity::Positive,
                    SignalStrength::Moderate,
                    "proof broadened after the focused implementation verifier turned clean",
                    Some(previous_failed.target_scope.raw.clone()),
                    Some(current.target_scope.raw.clone()),
                    merge_evidence(vec![
                        attempt_evidence(
                            previous_failed,
                            "earlier focused implementation verifier",
                        ),
                        attempt_evidence(current, "later broader implementation verifier"),
                        edit_overlap.evidence.clone(),
                    ]),
                ));
            }
            verifier_progress_with_overlap = scope_aligned;
            if edit_overlap.strength >= EditOverlapStrength::Moderate {
                signals.push(progress_signal(
                    ProgressSignalCode::FailingScopeEdited,
                    SignalPolarity::Positive,
                    SignalStrength::Moderate,
                    "an overlapping implementation edit touched the failing scope",
                    None,
                    None,
                    edit_overlap.evidence.clone(),
                ));
            }
        }
        if verifier_progress_with_overlap {
            return progress_from_signals(
                ProgressStatus::Advancing,
                dimension,
                if source_edits.is_empty() {
                    Confidence::Medium
                } else {
                    Confidence::High
                },
                signals,
                Vec::new(),
            );
        }
        return insufficient_progress(dimension, None, None);
    }

    if current.outcome == AttemptOutcome::Clean {
        return insufficient_progress(dimension, None, None);
    }

    if let Some(previous_clean) = best_clean {
        return progress_from_signals(
            ProgressStatus::Regressing,
            dimension,
            Confidence::High,
            vec![progress_signal(
                ProgressSignalCode::PreviouslyCleanScopeBroken,
                SignalPolarity::Negative,
                SignalStrength::Strong,
                "a previously clean implementation verifier is failing again",
                Some(previous_clean.target_scope.raw.clone()),
                Some(current.target_scope.raw.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous_clean, "earlier clean implementation verifier"),
                    attempt_evidence(current, "later failing implementation verifier"),
                ]),
            )],
            Vec::new(),
        );
    }

    if let Some(previous_failed) =
        best_failed.filter(|candidate| failed_attempt_frontier_cmp(candidate, current).is_gt())
    {
        return progress_from_signals(
            ProgressStatus::Regressing,
            dimension,
            Confidence::High,
            vec![progress_signal(
                ProgressSignalCode::PreviouslyCleanScopeBroken,
                SignalPolarity::Negative,
                SignalStrength::Strong,
                "the implementation verifier fell back behind a previously later comparable frontier",
                Some(verification_attempt_preview(previous_failed)),
                Some(verification_attempt_preview(current)),
                merge_evidence(vec![
                    attempt_evidence(previous_failed, "earlier later-stage implementation verifier"),
                    attempt_evidence(current, "later regressing implementation verifier"),
                ]),
            )],
            Vec::new(),
        );
    }

    if let Some(previous_failed) = latest_failed {
        let current_signature = best_signature(current);
        let previous_signature = best_signature(previous_failed);
        let edit_overlap = classify_attempt_scope_edit_overlap(
            previous_failed,
            current,
            &analysis.interval.command_attempts,
        );
        let scope_aligned = edit_overlap.strength >= EditOverlapStrength::Moderate
            || working_set_overlaps_verifier_scope(analysis, previous_failed, current);

        if let (Some(previous_signature), Some(current_signature)) =
            (previous_signature, current_signature)
        {
            let match_kind = match_diagnostic_signatures(previous_signature, current_signature);
            if matches!(
                match_kind,
                DiagnosticMatchKind::Exact | DiagnosticMatchKind::StrongFuzzy
            ) {
                signals.push(progress_signal(
                    ProgressSignalCode::FailureSignatureRepeated,
                    SignalPolarity::Negative,
                    SignalStrength::Strong,
                    "the same implementation verifier failed with the same signature again",
                    Some(previous_signature.preview.clone()),
                    Some(current_signature.preview.clone()),
                    merge_evidence(vec![
                        attempt_evidence(
                            previous_failed,
                            "earlier failing implementation verifier",
                        ),
                        attempt_evidence(current, "later failing implementation verifier"),
                    ]),
                ));
                signals.push(progress_signal(
                    ProgressSignalCode::FailingScopeUnchanged,
                    SignalPolarity::Negative,
                    SignalStrength::Moderate,
                    "the failing implementation scope did not change",
                    None,
                    None,
                    merge_evidence(vec![
                        attempt_evidence(previous_failed, "earlier failing scope"),
                        attempt_evidence(current, "later unchanged failing scope"),
                    ]),
                ));
            }
            if frontier_rank(current_signature.failure_class)
                > frontier_rank(previous_signature.failure_class)
                || matches!(
                    comparable_fail_count_delta(previous_signature, current_signature),
                    Some(Ordering::Less)
                )
            {
                verifier_progress_with_overlap = scope_aligned;
                signals.push(progress_signal(
                    ProgressSignalCode::FailureFrontierAdvanced,
                    SignalPolarity::Positive,
                    SignalStrength::Moderate,
                    "verification moved later in the implementation wall",
                    Some(previous_signature.preview.clone()),
                    Some(current_signature.preview.clone()),
                    merge_evidence(vec![
                        attempt_evidence(previous_failed, "earlier implementation verifier"),
                        attempt_evidence(current, "later implementation verifier"),
                        edit_overlap.evidence.clone(),
                    ]),
                ));
            }
            if edit_overlap.strength >= EditOverlapStrength::Moderate {
                signals.push(progress_signal(
                    ProgressSignalCode::FailingScopeEdited,
                    SignalPolarity::Positive,
                    SignalStrength::Moderate,
                    "an overlapping implementation edit touched the failing scope",
                    None,
                    None,
                    edit_overlap.evidence.clone(),
                ));
            }
        }
    }

    if !test_edits.is_empty() && source_edits.is_empty() {
        counter_evidence.extend(
            test_edits
                .iter()
                .flat_map(|attempt| {
                    attempt_evidence(
                        *attempt,
                        "test-only edits did not prove source-side implementation progress",
                    )
                })
                .collect::<Vec<_>>(),
        );
    }
    if diffused {
        counter_evidence.extend(
            source_edits
                .iter()
                .filter(|attempt| touches_only_unrelated_paths(analysis, attempt))
                .flat_map(|attempt| {
                    attempt_evidence(
                        *attempt,
                        "unrelated edit activity did not overlap the failing implementation scope",
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    let positive = has_direct_verifier_progress_signal(&signals);
    let negative = has_negative_signal(&signals);
    if positive && negative {
        return progress_from_signals(
            ProgressStatus::Mixed,
            dimension,
            Confidence::Medium,
            signals,
            counter_evidence,
        );
    }
    if positive && verifier_progress_with_overlap {
        return progress_from_signals(
            ProgressStatus::Advancing,
            dimension,
            Confidence::Medium,
            signals,
            counter_evidence,
        );
    }
    if negative {
        return progress_from_signals(
            ProgressStatus::Stalled,
            dimension,
            Confidence::Medium,
            signals,
            counter_evidence,
        );
    }

    insufficient_progress(dimension, None, None)
}

fn assess_closeout_progress(
    analysis: &CheckpointAnalysis,
    dimension: ProgressDimension,
    archetype_label: SessionArchetypeLabel,
) -> SessionProgress {
    let source_edits = source_edits(analysis);
    let closeout_artifact_edits = closeout_artifact_edits(analysis);
    let current_attempts = &analysis.interval.verification_attempts;
    let current = current_attempts.last();
    let prior_attempts = current
        .map(|current| comparable_attempts(analysis, current_attempts, current, archetype_label))
        .unwrap_or_else(|| prior_closeout_attempts_in_window(analysis));
    let best_clean = best_clean_attempt(&prior_attempts);

    let mut signals = Vec::new();

    if !source_edits.is_empty() {
        signals.push(progress_signal(
            ProgressSignalCode::ResidualScopeReopened,
            SignalPolarity::Negative,
            SignalStrength::Strong,
            "new source churn reopened closeout scope",
            None,
            None,
            source_edits
                .iter()
                .flat_map(|attempt| {
                    attempt_evidence(*attempt, "new source edit reopened closeout work")
                })
                .collect(),
        ));
    }

    if let (Some(previous_clean), Some(current)) = (best_clean, current) {
        if current.outcome == AttemptOutcome::Failed {
            signals.push(progress_signal(
                ProgressSignalCode::PreviouslyCleanScopeBroken,
                SignalPolarity::Negative,
                SignalStrength::Strong,
                "a previously clean closeout proof failed again",
                Some(previous_clean.target_scope.raw.clone()),
                Some(current.target_scope.raw.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous_clean, "earlier clean closeout proof"),
                    attempt_evidence(current, "later failing closeout proof"),
                ]),
            ));
            return progress_from_signals(
                ProgressStatus::Regressing,
                dimension,
                Confidence::Medium,
                signals,
                Vec::new(),
            );
        }
    }

    let artifact_target = current
        .is_none()
        .then(|| artifact_closeout_target(analysis))
        .flatten();

    if let Some(current) = current {
        if !attempt_is_clean_proof(current) {
            if !source_edits.is_empty() {
                return progress_from_signals(
                    ProgressStatus::Mixed,
                    dimension,
                    Confidence::Medium,
                    signals,
                    Vec::new(),
                );
            }
            return insufficient_progress(dimension, None, None);
        }

        let repeated_clean = prior_attempts.iter().find(|candidate| {
            attempt_is_clean_proof(candidate) && candidate.target_scope == current.target_scope
        });
        let broader_clean = most_relevant_broader_clean_attempt(
            &prior_attempts,
            current.verifier,
            &current.target_scope,
        );

        if let Some(previous) = broader_clean.filter(|_| repeated_clean.is_none()) {
            signals.push(progress_signal(
                ProgressSignalCode::VerificationScopeNarrowed,
                SignalPolarity::Positive,
                SignalStrength::Moderate,
                "closeout proof narrowed to a smaller residual verification scope",
                Some(previous.target_scope.raw.clone()),
                Some(current.target_scope.raw.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous, "earlier broader closeout proof"),
                    attempt_evidence(current, "later narrower closeout proof"),
                ]),
            ));
            signals.push(progress_signal(
                ProgressSignalCode::ResidualScopeShrank,
                SignalPolarity::Positive,
                SignalStrength::Moderate,
                "the residual closeout scope became smaller and more explicit",
                Some(previous.target_scope.raw.clone()),
                Some(current.target_scope.raw.clone()),
                merge_evidence(vec![
                    attempt_evidence(previous, "earlier broader residual scope"),
                    attempt_evidence(current, "later narrower residual scope"),
                ]),
            ));
        }

        signals.push(progress_signal(
            ProgressSignalCode::VerificationClean,
            SignalPolarity::Positive,
            SignalStrength::Strong,
            "closeout proof completed cleanly without reopening source work",
            None,
            Some(current.target_scope.raw.clone()),
            attempt_evidence(current, "clean closeout proof command"),
        ));

        if !closeout_artifact_edits.is_empty() {
            signals.push(progress_signal(
                ProgressSignalCode::PlanArtifactRefined,
                SignalPolarity::Positive,
                SignalStrength::Weak,
                "closeout artifact was refined after proof",
                None,
                None,
                closeout_artifact_edits
                    .iter()
                    .flat_map(|attempt| {
                        attempt_evidence(
                            *attempt,
                            "summary/handoff/fixture artifact refined after proof",
                        )
                    })
                    .collect(),
            ));
        }

        if source_edits.is_empty() && repeated_clean.is_some() && closeout_artifact_edits.is_empty()
        {
            return progress_from_signals(
                ProgressStatus::Stalled,
                dimension,
                Confidence::Medium,
                signals,
                repeated_clean
                    .map(|attempt| {
                        attempt_evidence(attempt, "earlier identical clean closeout proof")
                    })
                    .unwrap_or_default(),
            );
        }

        if !source_edits.is_empty() {
            let counter = negative_signal_evidence(&signals);
            return progress_from_signals(
                ProgressStatus::Mixed,
                dimension,
                Confidence::Medium,
                signals,
                counter,
            );
        }

        return progress_from_signals(
            ProgressStatus::Advancing,
            dimension,
            if source_edits.is_empty() {
                Confidence::High
            } else {
                Confidence::Medium
            },
            signals,
            Vec::new(),
        );
    }

    let Some((artifact_verifier, artifact_scope)) = artifact_target else {
        return insufficient_progress(dimension, None, None);
    };
    let broader_clean =
        most_relevant_broader_clean_attempt(&prior_attempts, artifact_verifier, &artifact_scope);

    if let Some(previous) = broader_clean {
        signals.push(progress_signal(
            ProgressSignalCode::VerificationScopeNarrowed,
            SignalPolarity::Positive,
            SignalStrength::Moderate,
            "closeout artifact narrowed to a smaller residual verification scope",
            Some(previous.target_scope.raw.clone()),
            Some(artifact_scope.raw.clone()),
            merge_evidence(vec![
                attempt_evidence(previous, "earlier broader closeout proof"),
                closeout_artifact_edits
                    .iter()
                    .flat_map(|attempt| {
                        attempt_evidence(
                            *attempt,
                            "later closeout artifact recorded a narrower residual scope",
                        )
                    })
                    .collect(),
                task_frame_evidence(
                    analysis,
                    "closeout objective kept only the narrowed residual verification scope",
                ),
            ]),
        ));
        signals.push(progress_signal(
            ProgressSignalCode::ResidualScopeShrank,
            SignalPolarity::Positive,
            SignalStrength::Moderate,
            "the residual closeout scope became smaller and more explicit",
            Some(previous.target_scope.raw.clone()),
            Some(artifact_scope.raw.clone()),
            merge_evidence(vec![
                attempt_evidence(previous, "earlier broader residual scope"),
                closeout_artifact_edits
                    .iter()
                    .flat_map(|attempt| {
                        attempt_evidence(
                            *attempt,
                            "later handoff/checklist refined the residual scope",
                        )
                    })
                    .collect(),
                task_frame_evidence(
                    analysis,
                    "current closeout objective names only the smaller residual scope",
                ),
            ]),
        ));
    }

    if !closeout_artifact_edits.is_empty() {
        signals.push(progress_signal(
            ProgressSignalCode::PlanArtifactRefined,
            SignalPolarity::Positive,
            SignalStrength::Weak,
            "closeout artifact was refined after proof",
            None,
            None,
            closeout_artifact_edits
                .iter()
                .flat_map(|attempt| {
                    attempt_evidence(
                        *attempt,
                        "summary/handoff/fixture artifact refined after proof",
                    )
                })
                .collect(),
        ));
    }

    if !source_edits.is_empty() {
        let counter = negative_signal_evidence(&signals);
        return progress_from_signals(
            ProgressStatus::Mixed,
            dimension,
            Confidence::Medium,
            signals,
            counter,
        );
    }

    if broader_clean.is_some() && !closeout_artifact_edits.is_empty() {
        return progress_from_signals(
            ProgressStatus::Advancing,
            dimension,
            Confidence::Medium,
            signals,
            Vec::new(),
        );
    }

    insufficient_progress(dimension, None, None)
}

fn apply_delegation_caps(
    analysis: &CheckpointAnalysis,
    mut progress: SessionProgress,
) -> SessionProgress {
    let Some(topology) = analysis.delegation.topology else {
        return progress;
    };
    let visibility = effective_child_work_visibility(analysis);

    if matches!(topology, DelegationTopology::SingleAgent) {
        return progress;
    }

    progress.confidence = progress.confidence.min(Confidence::Medium);
    if matches!(visibility, ChildWorkVisibility::Opaque) {
        progress.confidence = Confidence::Low;
    }

    let limiting_evidence =
        delegation_supporting_evidence(analysis, DELEGATION_LIMITING_CONFIDENCE_REASON);
    if !progress
        .signals
        .iter()
        .any(|signal| signal.code == ProgressSignalCode::DelegationVisibilityLimited)
        && !matches!(visibility, ChildWorkVisibility::None)
    {
        progress.signals.push(progress_signal(
            ProgressSignalCode::DelegationVisibilityLimited,
            SignalPolarity::Limiting,
            if matches!(visibility, ChildWorkVisibility::Opaque) {
                SignalStrength::Strong
            } else {
                SignalStrength::Moderate
            },
            delegation_limiting_summary(topology, visibility),
            None,
            None,
            limiting_evidence.clone(),
        ));
    }
    if !matches!(visibility, ChildWorkVisibility::None) {
        progress.counter_evidence.extend(limiting_evidence);
    }

    progress
}

fn finalize_progress(mut progress: SessionProgress) -> SessionProgress {
    progress.signals.sort_by(signal_sort_key);
    progress.supporting_evidence = dedupe_and_limit_evidence(progress.supporting_evidence);
    let required_delegation_counter_evidence =
        required_delegation_limiting_counter_evidence(&progress);
    progress.counter_evidence = dedupe_and_limit_counter_evidence(
        progress.counter_evidence,
        required_delegation_counter_evidence,
    );
    if progress.status != ProgressStatus::InsufficientEvidence
        && progress.supporting_evidence.is_empty()
    {
        progress.status = ProgressStatus::InsufficientEvidence;
        progress.confidence = Confidence::Low;
    }
    if progress.status == ProgressStatus::InsufficientEvidence {
        progress.confidence = Confidence::Low;
    }
    progress
}

fn classify_progress_outcome(
    dimension: ProgressDimension,
    signals: Vec<ProgressSignal>,
    positive_confidence: Confidence,
    nonpositive_confidence: Confidence,
) -> SessionProgress {
    let direct_positive = has_direct_troubleshooting_advancement_signal(&signals);
    let positive = has_positive_signal(&signals);
    let negative = has_negative_signal(&signals);
    if direct_positive {
        return progress_from_signals(
            ProgressStatus::Advancing,
            dimension,
            positive_confidence,
            signals,
            Vec::new(),
        );
    }
    if positive && negative {
        let counter = negative_signal_evidence(&signals);
        return progress_from_signals(
            ProgressStatus::Mixed,
            dimension,
            nonpositive_confidence,
            signals,
            counter,
        );
    }
    if positive {
        return progress_from_signals(
            ProgressStatus::Mixed,
            dimension,
            nonpositive_confidence,
            signals,
            Vec::new(),
        );
    }
    if signals.iter().any(|signal| {
        signal.code == ProgressSignalCode::PreviouslyCleanScopeBroken
            || signal.code == ProgressSignalCode::FailureCountIncreased
    }) {
        return progress_from_signals(
            ProgressStatus::Regressing,
            dimension,
            nonpositive_confidence,
            signals,
            Vec::new(),
        );
    }
    if negative {
        return progress_from_signals(
            ProgressStatus::Stalled,
            dimension,
            nonpositive_confidence,
            signals,
            Vec::new(),
        );
    }
    insufficient_progress(dimension, None, None)
}

fn comparable_attempts<'a>(
    analysis: &'a CheckpointAnalysis,
    current_attempts: &'a [VerificationAttempt],
    current: &'a VerificationAttempt,
    _archetype_label: SessionArchetypeLabel,
) -> Vec<VerificationAttempt> {
    let previous_attempts = prior_verification_attempts(analysis, current);
    let mut combined = Vec::new();
    combined.extend(previous_attempts);
    combined.extend(
        current_attempts
            .iter()
            .take(current_attempts.len().saturating_sub(1))
            .cloned(),
    );
    combined
        .into_iter()
        .filter(|candidate| attempts_are_comparable(candidate, current))
        .collect::<Vec<_>>()
}

fn prior_closeout_attempts_in_window(analysis: &CheckpointAnalysis) -> Vec<VerificationAttempt> {
    let Some(_) = &analysis.previous else {
        return Vec::new();
    };
    let analyses = super::checkpoint_analyses_with_typed_delegation(
        &analysis.current.window,
        analysis.typed_delegation.as_ref(),
    );
    let Some(current_index) = analyses.len().checked_sub(1) else {
        return Vec::new();
    };

    let mut collected = Vec::new();
    let mut newer = &analyses[current_index];
    let mut scanned_any = false;
    for older in analyses[..current_index].iter().rev() {
        if scanned_any && closeout_window_boundary(newer, older) {
            break;
        }

        collected.extend(older.interval.verification_attempts.clone());
        scanned_any = true;
        newer = older;
    }

    collected
}

fn prior_verification_attempts(
    analysis: &CheckpointAnalysis,
    current: &VerificationAttempt,
) -> Vec<VerificationAttempt> {
    let Some(_) = &analysis.previous else {
        return Vec::new();
    };
    let analyses = super::checkpoint_analyses_with_typed_delegation(
        &analysis.current.window,
        analysis.typed_delegation.as_ref(),
    );
    let Some(current_index) = analyses.len().checked_sub(1) else {
        return Vec::new();
    };

    let mut collected = Vec::new();
    let mut newer = &analyses[current_index];
    let mut scanned_any = false;
    for older in analyses[..current_index].iter().rev() {
        if scanned_any && comparable_window_boundary(newer, older, current) {
            break;
        }

        collected.extend(older.interval.verification_attempts.clone());
        scanned_any = true;
        newer = older;
    }

    collected
}

fn closeout_window_boundary(newer: &CheckpointAnalysis, older: &CheckpointAnalysis) -> bool {
    strong_archetype_boundary(newer, older)
        || explicit_replan_boundary(newer, older)
        || delegation_visibility_changed(newer, older)
        || objective_or_truth_artifacts_shifted(newer, older)
        || working_set_pivoted(newer, older)
}

fn comparable_window_boundary(
    newer: &CheckpointAnalysis,
    older: &CheckpointAnalysis,
    _current: &VerificationAttempt,
) -> bool {
    strong_archetype_boundary(newer, older)
        || explicit_replan_boundary(newer, older)
        || delegation_visibility_changed(newer, older)
        || objective_or_truth_artifacts_shifted(newer, older)
        || working_set_pivoted(newer, older)
}

fn strong_archetype_boundary(newer: &CheckpointAnalysis, older: &CheckpointAnalysis) -> bool {
    let newer_archetype = super::build_session_archetype(newer);
    let older_archetype = super::build_session_archetype(older);
    newer_archetype.label != older_archetype.label
        && newer_archetype.confidence == Confidence::High
        && older_archetype.confidence == Confidence::High
}

fn explicit_replan_boundary(newer: &CheckpointAnalysis, older: &CheckpointAnalysis) -> bool {
    let newer_objective = normalize_task_text(&newer.current.task_frame.objective);
    let older_objective = normalize_task_text(&older.current.task_frame.objective);
    newer_objective != older_objective
        && ["replan", "pivot", "switch", "instead"]
            .iter()
            .any(|keyword| newer_objective.contains(keyword))
}

fn delegation_visibility_changed(newer: &CheckpointAnalysis, older: &CheckpointAnalysis) -> bool {
    newer.delegation.topology != older.delegation.topology
        || newer.delegation.child_work_visibility != older.delegation.child_work_visibility
}

fn objective_or_truth_artifacts_shifted(
    newer: &CheckpointAnalysis,
    older: &CheckpointAnalysis,
) -> bool {
    let newer_truth = working_set(&newer.current.task_frame.truth_artifacts);
    let older_truth = working_set(&older.current.task_frame.truth_artifacts);
    let truth_shifted = sets_mostly_unrelated(&newer_truth, &older_truth);
    truth_shifted || material_objective_delta(newer, older)
}

fn working_set_pivoted(newer: &CheckpointAnalysis, older: &CheckpointAnalysis) -> bool {
    let newer_working = working_set(&newer.current.task_frame.working_set_paths);
    let older_working = working_set(&older.current.task_frame.working_set_paths);
    sets_mostly_unrelated(&newer_working, &older_working)
}

fn sets_mostly_unrelated(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    if left.is_empty() || right.is_empty() {
        return false;
    }

    let overlap = left.intersection(right).count();
    overlap == 0 || overlap * 2 < left.len().min(right.len())
}

fn normalize_task_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn material_objective_delta(newer: &CheckpointAnalysis, older: &CheckpointAnalysis) -> bool {
    if !newer.task_frame_delta.task_frame_transitioned {
        return false;
    }
    if objective_continues_current_line_of_work(&newer.current.task_frame.objective) {
        return false;
    }

    let newer_terms = objective_terms(&newer.current.task_frame.objective);
    let older_terms = objective_terms(&older.current.task_frame.objective);
    if newer_terms.is_empty() || older_terms.is_empty() {
        return false;
    }

    sets_mostly_unrelated(&newer_terms, &older_terms)
}

fn objective_continues_current_line_of_work(text: &str) -> bool {
    let normalized = normalize_task_text(text);
    [
        "same",
        "re-run",
        "rerun",
        "follow-up",
        "latest",
        "continue",
        "again",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn objective_terms(text: &str) -> BTreeSet<String> {
    text.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter_map(|term| {
            let normalized = term.to_ascii_lowercase();
            (normalized.len() >= 3
                && !matches!(
                    normalized.as_str(),
                    "the"
                        | "and"
                        | "for"
                        | "with"
                        | "into"
                        | "from"
                        | "that"
                        | "this"
                        | "while"
                        | "without"
                        | "same"
                        | "keep"
                        | "packet"
                        | "scope"
                        | "goal"
                ))
            .then_some(normalized)
        })
        .collect()
}

fn attempts_are_comparable(left: &VerificationAttempt, right: &VerificationAttempt) -> bool {
    if left.verifier == right.verifier && scopes_overlap(&left.target_scope, &right.target_scope) {
        return true;
    }
    if let (Some(left_signature), Some(right_signature)) =
        (best_signature(left), best_signature(right))
    {
        return !matches!(
            match_diagnostic_signatures(left_signature, right_signature),
            DiagnosticMatchKind::Unrelated | DiagnosticMatchKind::WeakRelated
        );
    }
    if attempt_is_clean_proof(left) || attempt_is_clean_proof(right) {
        return verifier_family(left.verifier) == verifier_family(right.verifier)
            && scopes_overlap(&left.target_scope, &right.target_scope);
    }
    if left.exercise_state == ExerciseState::BlockedBeforeTarget
        && right.exercise_state == ExerciseState::TargetExercised
    {
        return verifier_family(left.verifier) == verifier_family(right.verifier)
            || left.target_scope.broad
            || right.target_scope.broad;
    }
    false
}

fn attempt_matches_target(
    candidate: &VerificationAttempt,
    verifier: VerifierKind,
    scope: &VerificationScope,
) -> bool {
    if candidate.verifier == verifier && scopes_overlap(&candidate.target_scope, scope) {
        return true;
    }

    attempt_is_clean_proof(candidate)
        && verifier_family(candidate.verifier) == verifier_family(verifier)
        && scopes_overlap(&candidate.target_scope, scope)
}

fn most_relevant_broader_clean_attempt<'a>(
    prior_attempts: &'a [VerificationAttempt],
    verifier: VerifierKind,
    current_scope: &VerificationScope,
) -> Option<&'a VerificationAttempt> {
    prior_attempts
        .iter()
        .filter(|candidate| attempt_is_clean_proof(candidate))
        .filter(|candidate| attempt_matches_target(candidate, verifier, current_scope))
        .filter(|candidate| scope_is_narrower(current_scope, &candidate.target_scope))
        .min_by(|left, right| {
            scope_cardinality(&left.target_scope)
                .cmp(&scope_cardinality(&right.target_scope))
                .then_with(|| right.attempt_ordinal.cmp(&left.attempt_ordinal))
        })
}

fn best_signature(attempt: &VerificationAttempt) -> Option<&DiagnosticSignature> {
    attempt
        .signatures
        .iter()
        .min_by(|left, right| signature_sort_key(left).cmp(&signature_sort_key(right)))
}

fn troubleshooting_advancing_confidence(
    previous: &VerificationAttempt,
    current: &VerificationAttempt,
    previous_signature: Option<&DiagnosticSignature>,
    current_signature: Option<&DiagnosticSignature>,
) -> Confidence {
    let Some(previous_signature) = previous_signature else {
        return Confidence::Medium;
    };
    let Some(current_signature) = current_signature else {
        return Confidence::Medium;
    };

    let parser_strong = previous_signature.parser_confidence == Confidence::High
        && current_signature.parser_confidence == Confidence::High;
    let target_overlap_strong = diagnostic_targets_overlap(previous_signature, current_signature)
        || focused_verifier_targets_overlap(previous, current);

    if parser_strong && target_overlap_strong {
        Confidence::High
    } else {
        Confidence::Medium
    }
}

fn diagnostic_targets_overlap(
    previous: &DiagnosticSignature,
    current: &DiagnosticSignature,
) -> bool {
    previous.target_fingerprint.is_some()
        && previous.target_fingerprint == current.target_fingerprint
        && precise_target_fingerprint(previous)
        && precise_target_fingerprint(current)
        || collection_overlap(&previous.failing_paths, &current.failing_paths)
        || collection_overlap(&previous.failing_tests, &current.failing_tests)
}

fn precise_target_fingerprint(signature: &DiagnosticSignature) -> bool {
    signature
        .target_fingerprint
        .as_deref()
        .is_some_and(|fingerprint| {
            fingerprint.contains("path:")
                || fingerprint.contains("test:")
                || fingerprint.contains("cmd:")
        })
}

fn focused_verifier_targets_overlap(
    previous: &VerificationAttempt,
    current: &VerificationAttempt,
) -> bool {
    !previous.target_scope.broad
        && !current.target_scope.broad
        && scopes_overlap(&previous.target_scope, &current.target_scope)
}

fn collection_overlap(left: &[String], right: &[String]) -> bool {
    let left = left.iter().collect::<BTreeSet<_>>();
    let right = right.iter().collect::<BTreeSet<_>>();
    !left.is_empty() && !right.is_empty() && !left.is_disjoint(&right)
}

fn signature_sort_key(signature: &DiagnosticSignature) -> (u8, bool, usize, usize, usize) {
    (
        frontier_rank(signature.failure_class),
        signature.parser_confidence == Confidence::Low,
        signature.failing_paths.len(),
        signature.failing_tests.len(),
        signature.failing_symbols.len(),
    )
}

fn comparable_fail_count_delta(
    previous: &DiagnosticSignature,
    current: &DiagnosticSignature,
) -> Option<Ordering> {
    match (previous.failing_count, current.failing_count) {
        (Some(previous), Some(current)) => Some(current.cmp(&previous)),
        _ => None,
    }
}

fn frontier_rank(class: FailureClass) -> u8 {
    match class {
        FailureClass::Unknown => 0,
        FailureClass::EnvSetup | FailureClass::DependencyResolution => 1,
        FailureClass::CompileType
        | FailureClass::BuildLink
        | FailureClass::LintStyle
        | FailureClass::FormatStyle => 2,
        FailureClass::TestDiscovery => 3,
        FailureClass::TestExecution
        | FailureClass::TimeoutOrHang
        | FailureClass::ToolCrash
        | FailureClass::ExitCodeOnly => 4,
        FailureClass::AssertionOrGolden | FailureClass::ReplayMismatch => 5,
    }
}

fn scopes_overlap(left: &VerificationScope, right: &VerificationScope) -> bool {
    if left.broad || right.broad {
        return true;
    }
    let left_paths = left.paths.iter().collect::<BTreeSet<_>>();
    let right_paths = right.paths.iter().collect::<BTreeSet<_>>();
    let left_tests = left.tests.iter().collect::<BTreeSet<_>>();
    let right_tests = right.tests.iter().collect::<BTreeSet<_>>();
    !left_paths.is_disjoint(&right_paths) || !left_tests.is_disjoint(&right_tests)
}

fn scope_is_broader(current: &VerificationScope, previous: &VerificationScope) -> bool {
    if current.broad && !previous.broad {
        return true;
    }
    scope_cardinality(current) > scope_cardinality(previous)
}

fn scope_is_narrower(current: &VerificationScope, previous: &VerificationScope) -> bool {
    if !current.broad && previous.broad {
        return true;
    }
    scope_cardinality(current) < scope_cardinality(previous)
}

fn scope_cardinality(scope: &VerificationScope) -> usize {
    if scope.broad {
        usize::MAX / 2
    } else {
        scope.paths.len() + scope.tests.len()
    }
}

fn verifier_family(verifier: VerifierKind) -> &'static str {
    match verifier {
        VerifierKind::CargoCheck
        | VerifierKind::CargoBuild
        | VerifierKind::GenericBuild
        | VerifierKind::CargoClippy
        | VerifierKind::CargoFmt => "build",
        VerifierKind::CargoTest
        | VerifierKind::NpmTest
        | VerifierKind::PnpmTest
        | VerifierKind::Pytest
        | VerifierKind::Vitest
        | VerifierKind::Jest
        | VerifierKind::BunTest
        | VerifierKind::DenoTest
        | VerifierKind::GenericTest => "test",
        VerifierKind::Replay => "replay",
    }
}

fn plan_artifact_edits(analysis: &CheckpointAnalysis) -> Vec<&CommandAttempt> {
    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| attempt.role == CommandAttemptRole::Edit)
        .filter(|attempt| attempt.paths.iter().any(|path| is_plan_artifact_path(path)))
        .collect()
}

fn closeout_artifact_edits(analysis: &CheckpointAnalysis) -> Vec<&CommandAttempt> {
    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| attempt.role == CommandAttemptRole::Edit)
        .filter(|attempt| {
            attempt
                .paths
                .iter()
                .any(|path| is_closeout_artifact_path(path))
        })
        .collect()
}

fn artifact_closeout_target(
    analysis: &CheckpointAnalysis,
) -> Option<(VerifierKind, VerificationScope)> {
    analysis
        .current
        .task_frame
        .verification_commands
        .iter()
        .find_map(|command| verification_target_from_command(command))
}

fn source_edits(analysis: &CheckpointAnalysis) -> Vec<&CommandAttempt> {
    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| attempt.role == CommandAttemptRole::Edit)
        .filter(|attempt| attempt.paths.iter().any(|path| is_source_path(path)))
        .collect()
}

fn test_edits(analysis: &CheckpointAnalysis) -> Vec<&CommandAttempt> {
    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| attempt.role == CommandAttemptRole::Edit)
        .filter(|attempt| attempt.paths.iter().any(|path| is_test_path(path)))
        .collect()
}

fn has_parent_visible_orchestration_evidence(
    analysis: &CheckpointAnalysis,
    visibility: ChildWorkVisibility,
) -> bool {
    let orchestration_attempts = parent_visible_orchestration_attempts(analysis);
    let has_evidence = !analysis.interval.command_attempts.is_empty()
        && !orchestration_attempts.is_empty()
        && source_edits(analysis).is_empty()
        && analysis.interval.command_attempts.iter().all(|attempt| {
            !is_parent_visible_disqualifying_attempt(attempt)
                || is_parent_visible_artifact_refinement(attempt)
        })
        && match visibility {
            ChildWorkVisibility::Opaque => true,
            ChildWorkVisibility::Linked => true,
            ChildWorkVisibility::Partial => true,
            ChildWorkVisibility::None => false,
        };

    has_evidence
}

fn is_parent_visible_disqualifying_attempt(attempt: &CommandAttempt) -> bool {
    matches!(
        attempt.role,
        CommandAttemptRole::Edit
            | CommandAttemptRole::DependencyMutation
            | CommandAttemptRole::FormatWrite
    ) && !is_parent_visible_artifact_refinement(attempt)
}

fn effective_child_work_visibility(analysis: &CheckpointAnalysis) -> ChildWorkVisibility {
    if has_visible_child_surface(analysis) {
        ChildWorkVisibility::Partial
    } else {
        analysis
            .delegation
            .child_work_visibility
            .unwrap_or(ChildWorkVisibility::None)
    }
}

fn has_visible_child_surface(analysis: &CheckpointAnalysis) -> bool {
    analysis
        .delegation
        .supporting_evidence
        .iter()
        .any(|evidence| {
            evidence
                .reason
                .contains("delegation child rollout surface links child/subagent work")
        })
}

fn is_parent_visible_synthesis_attempt(attempt: &CommandAttempt) -> bool {
    matches!(
        attempt.role,
        CommandAttemptRole::Compile
            | CommandAttemptRole::Test
            | CommandAttemptRole::Lint
            | CommandAttemptRole::FormatCheck
            | CommandAttemptRole::Build
            | CommandAttemptRole::Replay
    ) || (attempt.role == CommandAttemptRole::Edit
        && attempt
            .paths
            .iter()
            .any(|path| is_plan_artifact_path(path) || is_closeout_artifact_path(path)))
}

fn is_parent_visible_artifact_refinement(attempt: &CommandAttempt) -> bool {
    attempt.role == CommandAttemptRole::Edit
        && attempt
            .paths
            .iter()
            .any(|path| is_plan_artifact_path(path) || is_closeout_artifact_path(path))
}

fn parent_visible_orchestration_attempts(analysis: &CheckpointAnalysis) -> Vec<&CommandAttempt> {
    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| attempt.role == CommandAttemptRole::Orchestration)
        .collect()
}

fn parent_visible_synthesis_attempts(analysis: &CheckpointAnalysis) -> Vec<&CommandAttempt> {
    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| is_parent_visible_synthesis_attempt(attempt))
        .collect()
}

fn parent_visible_synthesis_evidence(attempts: &[&CommandAttempt]) -> Vec<EvidenceRef> {
    attempts
        .iter()
        .flat_map(|attempt| {
            let reason = match attempt.role {
                CommandAttemptRole::Edit => {
                    "parent incorporated visible delegated results into a plan/spec/handoff artifact"
                }
                CommandAttemptRole::Compile
                | CommandAttemptRole::Test
                | CommandAttemptRole::Lint
                | CommandAttemptRole::FormatCheck
                | CommandAttemptRole::Build
                | CommandAttemptRole::Replay => {
                    "parent-owned verification followed visible delegated results"
                }
                _ => "parent-visible synthesis row",
            };
            attempt_evidence(*attempt, reason)
        })
        .collect()
}

fn has_parent_visible_synthesis(
    visibility: ChildWorkVisibility,
    visible_child_surface: bool,
    orchestration_attempts: &[&CommandAttempt],
    synthesis_attempts: &[&CommandAttempt],
) -> bool {
    matches!(visibility, ChildWorkVisibility::Partial)
        && visible_child_surface
        && (!synthesis_attempts.is_empty()
            || orchestration_attempts.iter().any(|attempt| {
                matches!(attempt.tool_name.as_str(), "close_agent" | "multi_agent_v1")
            }))
}

fn previous_checkpoint_was_comparable_parent_visible(analysis: &CheckpointAnalysis) -> bool {
    let analyses = super::checkpoint_analyses_with_typed_delegation(
        &analysis.current.window,
        analysis.typed_delegation.as_ref(),
    );
    analyses
        .iter()
        .rev()
        .skip(1)
        .take_while(|previous| {
            !parent_visible_comparability_reset(analysis, previous)
                || parent_visible_followup_without_new_attempts(analysis, previous)
        })
        .find_map(|previous| {
            if has_parent_visible_orchestration_evidence(
                previous,
                effective_child_work_visibility(previous),
            ) {
                Some(true)
            } else if previous.interval.command_attempts.is_empty() {
                None
            } else {
                Some(false)
            }
        })
        .unwrap_or(false)
}

fn parent_visible_followup_without_new_attempts(
    newer: &CheckpointAnalysis,
    older: &CheckpointAnalysis,
) -> bool {
    newer.interval.command_attempts.is_empty()
        && newer.delegation.topology == older.delegation.topology
        && effective_child_work_visibility(newer) == effective_child_work_visibility(older)
        && has_visible_child_surface(newer) == has_visible_child_surface(older)
        && normalize_task_text(&newer.current.task_frame.objective)
            == normalize_task_text(&older.current.task_frame.objective)
}

fn parent_visible_comparability_reset(
    newer: &CheckpointAnalysis,
    older: &CheckpointAnalysis,
) -> bool {
    parent_visible_comparability_fingerprint(newer)
        != parent_visible_comparability_fingerprint(older)
}

type ParentVisibleComparabilityFingerprint = (
    DelegationTopology,
    ChildWorkVisibility,
    bool,
    String,
    Vec<String>,
    BTreeSet<String>,
);

fn parent_visible_comparability_fingerprint(
    analysis: &CheckpointAnalysis,
) -> Option<ParentVisibleComparabilityFingerprint> {
    Some((
        analysis.delegation.topology?,
        effective_child_work_visibility(analysis),
        has_visible_child_surface(analysis),
        normalize_task_text(&analysis.current.task_frame.objective),
        parent_visible_delegated_objective_surface(analysis),
        working_set(&analysis.current.task_frame.working_set_paths),
    ))
}

fn parent_visible_delegated_objective_surface(analysis: &CheckpointAnalysis) -> Vec<String> {
    parent_visible_orchestration_attempts(analysis)
        .iter()
        .filter(|attempt| matches!(attempt.tool_name.as_str(), "spawn_agent" | "multi_agent_v1"))
        .map(|attempt| {
            format!(
                "{}:{}",
                attempt.tool_name,
                normalize_task_text(&attempt.raw_command)
            )
        })
        .collect()
}

fn working_set(paths: &[String]) -> BTreeSet<String> {
    paths.iter().cloned().collect()
}

fn working_set_narrowed(previous: &BTreeSet<String>, current: &BTreeSet<String>) -> bool {
    !current.is_empty()
        && ((!previous.is_empty() && current.len() < previous.len() && current.is_subset(previous))
            || (previous.is_empty() && current.len() <= 3))
}

fn working_set_overlaps_verifier_scope(
    analysis: &CheckpointAnalysis,
    previous: &VerificationAttempt,
    current: &VerificationAttempt,
) -> bool {
    let current_working_set = working_set(&analysis.current.task_frame.working_set_paths);
    if current_working_set.is_empty() {
        return false;
    }

    let scope_paths = previous
        .signatures
        .iter()
        .flat_map(|signature| signature.failing_paths.iter().cloned())
        .chain(
            current
                .signatures
                .iter()
                .flat_map(|signature| signature.failing_paths.iter().cloned()),
        )
        .chain(previous.target_scope.paths.iter().cloned())
        .chain(current.target_scope.paths.iter().cloned())
        .collect::<BTreeSet<_>>();

    current_working_set.iter().any(|working_path| {
        scope_paths.iter().any(|scope_path| {
            working_path == scope_path
                || working_path.starts_with(scope_path)
                || scope_path.starts_with(working_path)
        })
    })
}

fn truth_artifacts_narrowed(analysis: &CheckpointAnalysis) -> bool {
    let previous = analysis
        .previous
        .as_ref()
        .map(|slice| working_set(&slice.task_frame.truth_artifacts))
        .unwrap_or_default();
    let current = working_set(&analysis.current.task_frame.truth_artifacts);
    working_set_narrowed(&previous, &current)
}

fn working_set_diffused(previous: &BTreeSet<String>, current: &BTreeSet<String>) -> bool {
    current.len() >= 4
        && (previous.is_empty()
            || current.len() > previous.len()
            || current.intersection(previous).count() * 2 < current.len())
}

fn working_set_is_concentrated(analysis: &CheckpointAnalysis) -> bool {
    let current = working_set(&analysis.current.task_frame.working_set_paths);
    let previous = analysis
        .previous
        .as_ref()
        .map(|slice| working_set(&slice.task_frame.working_set_paths))
        .unwrap_or_default();
    !current.is_empty() && (current.len() <= 3 || working_set_narrowed(&previous, &current))
}

fn working_set_is_diffused(analysis: &CheckpointAnalysis) -> bool {
    let current = working_set(&analysis.current.task_frame.working_set_paths);
    let previous = analysis
        .previous
        .as_ref()
        .map(|slice| working_set(&slice.task_frame.working_set_paths))
        .unwrap_or_default();
    working_set_diffused(&previous, &current)
}

fn zero_verifier_exploratory_interval(analysis: &CheckpointAnalysis) -> bool {
    if working_set_is_diffused(analysis) {
        return true;
    }

    let current = working_set(&analysis.current.task_frame.working_set_paths);
    if current.len() < 8 {
        return false;
    }

    analysis
        .interval
        .command_attempts
        .iter()
        .filter(|attempt| {
            attempt.role == CommandAttemptRole::Read || is_probe_discovery_attempt(attempt)
        })
        .count()
        >= 2
}

fn touches_only_unrelated_paths(analysis: &CheckpointAnalysis, attempt: &CommandAttempt) -> bool {
    let current_working_set = working_set(&analysis.current.task_frame.working_set_paths);
    !attempt.paths.iter().any(|path| {
        current_working_set.iter().any(|working_path| {
            path == working_path || path.starts_with(working_path) || working_path.starts_with(path)
        })
    })
}

fn task_frame_evidence(analysis: &CheckpointAnalysis, reason: &str) -> Vec<EvidenceRef> {
    analysis
        .current
        .task_frame
        .supporting_evidence
        .iter()
        .take(2)
        .map(|evidence| EvidenceRef {
            row: evidence.row.clone(),
            reason: reason.to_string(),
        })
        .collect()
}

fn previous_task_frame_evidence(analysis: &CheckpointAnalysis, reason: &str) -> Vec<EvidenceRef> {
    analysis
        .previous
        .as_ref()
        .map(|slice| {
            slice
                .task_frame
                .supporting_evidence
                .iter()
                .take(2)
                .map(|evidence| EvidenceRef {
                    row: evidence.row.clone(),
                    reason: reason.to_string(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn delegation_supporting_evidence(analysis: &CheckpointAnalysis, reason: &str) -> Vec<EvidenceRef> {
    analysis
        .delegation
        .supporting_evidence
        .iter()
        .take(2)
        .map(|evidence| EvidenceRef {
            row: evidence.row.clone(),
            reason: reason.to_string(),
        })
        .collect()
}

fn attempt_evidence<T: AttemptEvidence>(attempt: &T, reason: &str) -> Vec<EvidenceRef> {
    vec![EvidenceRef {
        row: attempt.row_ref().clone(),
        reason: reason.to_string(),
    }]
}

trait AttemptEvidence {
    fn row_ref(&self) -> &RowRef;
}

impl AttemptEvidence for CommandAttempt {
    fn row_ref(&self) -> &RowRef {
        &self.command_row
    }
}

impl AttemptEvidence for VerificationAttempt {
    fn row_ref(&self) -> &RowRef {
        &self.command_row
    }
}

fn progress_signal(
    code: ProgressSignalCode,
    polarity: SignalPolarity,
    strength: SignalStrength,
    summary: impl Into<String>,
    before: Option<String>,
    after: Option<String>,
    evidence: Vec<EvidenceRef>,
) -> ProgressSignal {
    ProgressSignal {
        code,
        polarity,
        strength,
        summary: summary.into(),
        before,
        after,
        evidence: dedupe_and_limit_evidence(evidence),
    }
}

fn progress_from_signals(
    status: ProgressStatus,
    dimension: ProgressDimension,
    confidence: Confidence,
    signals: Vec<ProgressSignal>,
    counter_evidence: Vec<EvidenceRef>,
) -> SessionProgress {
    let supporting_evidence = signals
        .iter()
        .flat_map(|signal| signal.evidence.clone())
        .collect::<Vec<_>>();
    SessionProgress {
        status,
        dimension,
        confidence,
        signals,
        supporting_evidence,
        counter_evidence,
    }
}

fn insufficient_progress(
    dimension: ProgressDimension,
    signal: Option<ProgressSignal>,
    counter_evidence: Option<Vec<EvidenceRef>>,
) -> SessionProgress {
    SessionProgress {
        status: ProgressStatus::InsufficientEvidence,
        dimension,
        confidence: Confidence::Low,
        signals: signal.into_iter().collect(),
        supporting_evidence: Vec::new(),
        counter_evidence: counter_evidence.unwrap_or_default(),
    }
}

fn latest_failed_attempt(prior_attempts: &[VerificationAttempt]) -> Option<&VerificationAttempt> {
    prior_attempts
        .iter()
        .rev()
        .find(|candidate| candidate.outcome == AttemptOutcome::Failed)
}

fn best_clean_attempt(prior_attempts: &[VerificationAttempt]) -> Option<&VerificationAttempt> {
    prior_attempts
        .iter()
        .filter(|candidate| attempt_is_clean_proof(candidate))
        .max_by(|left, right| {
            scope_cardinality(&left.target_scope)
                .cmp(&scope_cardinality(&right.target_scope))
                .then_with(|| left.attempt_ordinal.cmp(&right.attempt_ordinal))
        })
}

fn best_failed_attempt(prior_attempts: &[VerificationAttempt]) -> Option<&VerificationAttempt> {
    prior_attempts
        .iter()
        .filter(|candidate| candidate.outcome == AttemptOutcome::Failed)
        .max_by(|left, right| failed_attempt_frontier_cmp(left, right))
}

fn failed_attempt_frontier_cmp(
    left: &VerificationAttempt,
    right: &VerificationAttempt,
) -> Ordering {
    let left_signature = best_signature(left);
    let right_signature = best_signature(right);

    matches!(left.exercise_state, ExerciseState::TargetExercised)
        .cmp(&matches!(
            right.exercise_state,
            ExerciseState::TargetExercised
        ))
        .then_with(|| {
            left_signature
                .map(|signature| frontier_rank(signature.failure_class))
                .unwrap_or_default()
                .cmp(
                    &right_signature
                        .map(|signature| frontier_rank(signature.failure_class))
                        .unwrap_or_default(),
                )
        })
        .then_with(|| comparable_fail_count_frontier_cmp(left_signature, right_signature))
        .then_with(|| {
            scope_cardinality(&left.target_scope).cmp(&scope_cardinality(&right.target_scope))
        })
        .then_with(|| left.attempt_ordinal.cmp(&right.attempt_ordinal))
}

fn comparable_fail_count_frontier_cmp(
    left: Option<&DiagnosticSignature>,
    right: Option<&DiagnosticSignature>,
) -> Ordering {
    match (
        left.and_then(|signature| signature.failing_count),
        right.and_then(|signature| signature.failing_count),
    ) {
        (Some(left), Some(right)) => right.cmp(&left),
        _ => Ordering::Equal,
    }
}

fn verification_attempt_preview(attempt: &VerificationAttempt) -> String {
    if attempt_is_clean_proof(attempt) {
        format!("clean {}", attempt.target_scope.raw)
    } else {
        best_signature(attempt)
            .map(|signature| signature.preview.clone())
            .unwrap_or_else(|| attempt.target_scope.raw.clone())
    }
}

fn attempt_is_clean_proof(attempt: &VerificationAttempt) -> bool {
    attempt.outcome == AttemptOutcome::Clean
        && attempt.exercise_state == ExerciseState::TargetExercised
}

fn has_direct_verifier_progress_signal(signals: &[ProgressSignal]) -> bool {
    signals.iter().any(|signal| {
        matches!(
            signal.code,
            ProgressSignalCode::FailureFrontierAdvanced | ProgressSignalCode::VerificationClean
        )
    })
}

fn has_direct_troubleshooting_advancement_signal(signals: &[ProgressSignal]) -> bool {
    signals.iter().any(|signal| {
        matches!(
            signal.code,
            ProgressSignalCode::FailureFrontierAdvanced
                | ProgressSignalCode::FailureCountReduced
                | ProgressSignalCode::VerificationClean
        )
    })
}

fn has_positive_signal(signals: &[ProgressSignal]) -> bool {
    signals
        .iter()
        .any(|signal| signal.polarity == SignalPolarity::Positive)
}

fn has_negative_signal(signals: &[ProgressSignal]) -> bool {
    signals.iter().any(|signal| {
        matches!(
            signal.polarity,
            SignalPolarity::Negative | SignalPolarity::Mixed
        )
    })
}

fn negative_signal_evidence(signals: &[ProgressSignal]) -> Vec<EvidenceRef> {
    signals
        .iter()
        .filter(|signal| {
            matches!(
                signal.polarity,
                SignalPolarity::Negative | SignalPolarity::Mixed
            )
        })
        .flat_map(|signal| signal.evidence.clone())
        .collect()
}

fn merge_evidence(groups: Vec<Vec<EvidenceRef>>) -> Vec<EvidenceRef> {
    dedupe_and_limit_evidence(groups.into_iter().flatten().collect())
}

fn dedupe_and_limit_evidence(items: Vec<EvidenceRef>) -> Vec<EvidenceRef> {
    let mut deduped = dedupe_evidence(items);
    deduped.truncate(MAX_PROGRESS_EVIDENCE_ITEMS);
    deduped
}

fn dedupe_and_limit_counter_evidence(
    items: Vec<EvidenceRef>,
    required: Option<EvidenceRef>,
) -> Vec<EvidenceRef> {
    let mut deduped = dedupe_evidence(items);
    if deduped.len() <= MAX_PROGRESS_EVIDENCE_ITEMS {
        return deduped;
    }
    let Some(required) = required else {
        deduped.truncate(MAX_PROGRESS_EVIDENCE_ITEMS);
        return deduped;
    };

    let mut limited = deduped
        .iter()
        .take(MAX_PROGRESS_EVIDENCE_ITEMS)
        .cloned()
        .collect::<Vec<_>>();
    if limited.contains(&required) {
        return limited;
    }

    limited.pop();
    limited.push(required);
    limited.sort_by(evidence_sort_key);
    limited
}

fn required_delegation_limiting_counter_evidence(
    progress: &SessionProgress,
) -> Option<EvidenceRef> {
    let limiting_row_keys = progress
        .signals
        .iter()
        .filter(|signal| signal.code == ProgressSignalCode::DelegationVisibilityLimited)
        .flat_map(|signal| signal.evidence.iter())
        .map(evidence_row_key)
        .collect::<BTreeSet<_>>();
    if limiting_row_keys.is_empty() {
        return None;
    }

    let counter_evidence = dedupe_evidence(progress.counter_evidence.clone());
    counter_evidence
        .iter()
        .find(|evidence| {
            evidence.reason == DELEGATION_LIMITING_CONFIDENCE_REASON
                && limiting_row_keys.contains(&evidence_row_key(evidence))
        })
        .cloned()
        .or_else(|| {
            counter_evidence
                .into_iter()
                .find(|evidence| limiting_row_keys.contains(&evidence_row_key(evidence)))
        })
}

fn dedupe_evidence(items: Vec<EvidenceRef>) -> Vec<EvidenceRef> {
    let mut deduped = items
        .into_iter()
        .fold(BTreeMap::new(), |mut acc, evidence| {
            acc.entry(evidence_key(&evidence)).or_insert(evidence);
            acc
        })
        .into_values()
        .collect::<Vec<_>>();
    deduped.sort_by(evidence_sort_key);
    deduped
}

fn evidence_key(evidence: &EvidenceRef) -> (camino::Utf8PathBuf, usize, usize, String) {
    (
        evidence.row.source_file.clone(),
        evidence.row.event_index,
        evidence.row.row_ordinal,
        evidence.reason.clone(),
    )
}

fn evidence_row_key(evidence: &EvidenceRef) -> (camino::Utf8PathBuf, usize, usize) {
    (
        evidence.row.source_file.clone(),
        evidence.row.event_index,
        evidence.row.row_ordinal,
    )
}

fn evidence_sort_key(left: &EvidenceRef, right: &EvidenceRef) -> Ordering {
    (
        left.row.source_file.as_str(),
        left.row.event_index,
        left.row.row_ordinal,
        left.reason.as_str(),
    )
        .cmp(&(
            right.row.source_file.as_str(),
            right.row.event_index,
            right.row.row_ordinal,
            right.reason.as_str(),
        ))
}

fn set_preview(set: &BTreeSet<String>) -> String {
    if set.is_empty() {
        return "none".to_string();
    }
    set.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
}

fn plan_artifact_created(analysis: &CheckpointAnalysis, attempt: &CommandAttempt) -> bool {
    let previous_truth_artifacts = analysis
        .previous
        .as_ref()
        .map(|slice| working_set(&slice.task_frame.truth_artifacts))
        .unwrap_or_default();
    attempt
        .paths
        .iter()
        .any(|path| !previous_truth_artifacts.contains(path))
}

fn is_plan_artifact_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("docs/specs/")
        || lower.contains("/docs/specs/")
        || lower.ends_with("-spec.md")
        || lower.ends_with("-plan.md")
        || lower.ends_with("-tasks.md")
        || lower.contains("design-")
}

fn is_closeout_artifact_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("handoff")
        || lower.contains("summary")
        || lower.contains("fixture")
        || lower.starts_with("docs/")
        || lower.ends_with(".md")
}

fn is_source_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    (lower.ends_with(".rs")
        || lower.ends_with(".ts")
        || lower.ends_with(".tsx")
        || lower.ends_with(".js")
        || lower.ends_with(".jsx")
        || lower.ends_with(".py"))
        && !is_test_path(path)
        && !lower.ends_with(".md")
}

fn is_test_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("/tests/")
        || lower.contains("/fixtures/")
        || lower.contains("golden")
        || lower.ends_with(".snap")
        || lower.ends_with(".golden")
}

fn delegation_limiting_summary(
    topology: DelegationTopology,
    visibility: ChildWorkVisibility,
) -> &'static str {
    match (topology, visibility) {
        (DelegationTopology::DelegatingParent, ChildWorkVisibility::Opaque) => {
            "delegating-parent plus child-opaque visibility prevented direct child progress claim"
        }
        (DelegationTopology::DelegatingParent, ChildWorkVisibility::Partial) => {
            "partial child visibility limited progress confidence"
        }
        (DelegationTopology::MixedOrAmbiguous, ChildWorkVisibility::Opaque) => {
            "only parent-visible orchestration evidence was available for progress assessment"
        }
        _ => "delegation visibility limited progress confidence",
    }
}

fn signal_sort_key(left: &ProgressSignal, right: &ProgressSignal) -> std::cmp::Ordering {
    (
        left.code as u8,
        left.summary.as_str(),
        left.before.as_deref().unwrap_or_default(),
        left.after.as_deref().unwrap_or_default(),
    )
        .cmp(&(
            right.code as u8,
            right.summary.as_str(),
            right.before.as_deref().unwrap_or_default(),
            right.after.as_deref().unwrap_or_default(),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::checkpoint_analyses;
    use crate::input::BundleSession;
    use agent_session_compactor::{CompactionKind, CompactionRow, UserMessageRole};
    use camino::Utf8PathBuf;

    #[test]
    fn troubleshooting_present_to_missing_fail_count_stays_conservative() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Troubleshoot checkpoints::captures_progress without changing scope.",
            ),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\nAssertionError: expected advancing",
            ),
            prompt_row(
                3,
                "turn-002",
                "/goal Re-run the same troubleshooting verifier before widening scope.",
            ),
            tool_call_row(
                4,
                "turn-002",
                "functions.shell_command",
                "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
            ),
            tool_output_row(
                5,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nAssertionError: expected advancing",
            ),
        ]);

        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_has_signal(&progress, ProgressSignalCode::FailureSignatureRepeated);
        assert_lacks_signal(&progress, ProgressSignalCode::FailureCountReduced);
        assert_lacks_signal(&progress, ProgressSignalCode::FailureCountIncreased);
        assert_lacks_signal(&progress, ProgressSignalCode::FailureFrontierAdvanced);
    }

    #[test]
    fn troubleshooting_missing_to_present_fail_count_stays_conservative() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Troubleshoot checkpoints::captures_progress without changing scope.",
            ),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nAssertionError: expected advancing",
            ),
            prompt_row(
                3,
                "turn-002",
                "/goal Re-run the same troubleshooting verifier before widening scope.",
            ),
            tool_call_row(
                4,
                "turn-002",
                "functions.shell_command",
                "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
            ),
            tool_output_row(
                5,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\nAssertionError: expected advancing",
            ),
        ]);

        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_has_signal(&progress, ProgressSignalCode::FailureSignatureRepeated);
        assert_lacks_signal(&progress, ProgressSignalCode::FailureCountReduced);
        assert_lacks_signal(&progress, ProgressSignalCode::FailureCountIncreased);
        assert_lacks_signal(&progress, ProgressSignalCode::FailureFrontierAdvanced);
    }

    #[test]
    fn checkpoints_troubleshooting_fail_count_reduction_with_overlap_still_advances() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Troubleshoot checkpoints::captures_progress without widening scope.",
            ),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 2 tests\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nthread 'checkpoints::captures_progress' panicked at crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34:\nAssertionError: expected advancing\n\ntest result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 26 filtered out",
            ),
            prompt_row(
                3,
                "turn-002",
                "/goal Re-run the same troubleshooting verifier after a focused fix edit.",
            ),
            tool_call_row(
                4,
                "turn-002",
                "functions.apply_patch",
                "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
            ),
            tool_call_row(
                5,
                "turn-002",
                "functions.shell_command",
                "{\"command\":\"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture\",\"workdir\":\"/repo\"}",
            ),
            tool_output_row(
                6,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nthread 'checkpoints::captures_progress' panicked at crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34:\nAssertionError: expected advancing\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out",
            ),
        ]);

        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Advancing);
        assert_has_signal(&progress, ProgressSignalCode::FailureSignatureRepeated);
        assert_has_signal(&progress, ProgressSignalCode::FailureCountReduced);
        assert_has_signal(&progress, ProgressSignalCode::FailingScopeEdited);
    }

    #[test]
    fn troubleshooting_concurrent_clean_sibling_does_not_erase_repeated_failed_lane() {
        let analysis = last_analysis(vec![
            prompt_row(0, "turn-001", "/goal Troubleshoot the target lane."),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test target_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest target_lane ... FAILED\n\nfailures:\n    target_lane\n\nthread 'target_lane' panicked at crates/target/src/lib.rs:10:5:\nassertion `left == right` failed: target lane repeated\n  left: 1\n right: 2\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            prompt_row(
                3,
                "turn-002",
                "/goal Re-run the target lane alongside an unrelated clean sibling.",
            ),
            tool_call_row(
                4,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test target_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                5,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest target_lane ... FAILED\n\nfailures:\n    target_lane\n\nthread 'target_lane' panicked at crates/target/src/lib.rs:10:5:\nassertion `left == right` failed: target lane repeated\n  left: 1\n right: 2\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test sibling_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                7,
                "turn-002",
                "Exit code: 0\nrunning 1 test\ntest sibling_lane ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
        ]);

        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_eq!(progress.confidence, Confidence::Medium);
        assert_has_signal(&progress, ProgressSignalCode::FailureSignatureRepeated);
        assert!(
            progress
                .supporting_evidence
                .iter()
                .all(|evidence| evidence.row.event_index != 6),
            "unrelated clean sibling must not become failure-lane evidence"
        );
    }

    #[test]
    fn troubleshooting_advancing_lane_ignores_unrelated_insufficient_sibling() {
        let analysis = last_analysis(vec![
            prompt_row(0, "turn-001", "/goal Troubleshoot the advancing target lane."),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test advancing_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest advancing_lane ... FAILED\n\nfailures:\n    advancing_lane\n\nthread 'advancing_lane' panicked at crates/advancing/src/lib.rs:20:5:\nassertion failed: expected clean target\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            prompt_row(
                3,
                "turn-002",
                "/goal Re-run the advancing lane alongside an unrelated clean sibling.",
            ),
            tool_call_row(
                4,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test advancing_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                5,
                "turn-002",
                "Exit code: 0\nrunning 1 test\ntest advancing_lane ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test unrelated_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                7,
                "turn-002",
                "Exit code: 0\nrunning 1 test\ntest unrelated_lane ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
        ]);

        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Advancing);
        assert_eq!(progress.confidence, Confidence::High);
        assert_has_signal(&progress, ProgressSignalCode::VerificationClean);
        assert!(
            progress
                .supporting_evidence
                .iter()
                .all(|evidence| evidence.row.event_index != 6),
            "unrelated insufficient sibling must not change advancing-lane evidence"
        );
    }

    #[test]
    fn troubleshooting_conflicting_informative_lanes_aggregate_mixed() {
        let analysis = last_analysis(vec![
            prompt_row(0, "turn-001", "/goal Troubleshoot two independent lanes."),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test positive_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest positive_lane ... FAILED\n\nfailures:\n    positive_lane\n\nthread 'positive_lane' panicked at crates/positive/src/lib.rs:30:5:\nassertion failed: positive lane\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test negative_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                4,
                "turn-001",
                "Exit code: 0\nrunning 1 test\ntest negative_lane ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            prompt_row(
                5,
                "turn-002",
                "/goal Re-run both independent troubleshooting lanes.",
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test positive_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                7,
                "turn-002",
                "Exit code: 0\nrunning 1 test\ntest positive_lane ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                8,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test negative_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                9,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest negative_lane ... FAILED\n\nfailures:\n    negative_lane\n\nthread 'negative_lane' panicked at crates/negative/src/lib.rs:40:5:\nassertion failed: negative lane\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
        ]);

        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Mixed);
        assert_eq!(progress.confidence, Confidence::Medium);
        assert!(progress
            .signals
            .iter()
            .any(|signal| signal.polarity == SignalPolarity::Positive));
        assert!(progress
            .signals
            .iter()
            .any(|signal| signal.polarity == SignalPolarity::Negative));
    }

    #[test]
    fn troubleshooting_regressing_dominates_only_negative_lanes() {
        let analysis = last_analysis(vec![
            prompt_row(0, "turn-001", "/goal Troubleshoot two negative lanes."),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test regressing_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 0\nrunning 1 test\ntest regressing_lane ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test stalled_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                4,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest stalled_lane ... FAILED\n\nfailures:\n    stalled_lane\n\nthread 'stalled_lane' panicked at crates/stalled/src/lib.rs:50:5:\nassertion failed: stalled lane repeated\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            prompt_row(5, "turn-002", "/goal Re-run both negative lanes."),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test regressing_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                7,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest regressing_lane ... FAILED\n\nfailures:\n    regressing_lane\n\nthread 'regressing_lane' panicked at crates/regressing/src/lib.rs:60:5:\nassertion failed: regressing lane broke\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                8,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test stalled_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                9,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest stalled_lane ... FAILED\n\nfailures:\n    stalled_lane\n\nthread 'stalled_lane' panicked at crates/stalled/src/lib.rs:50:5:\nassertion failed: stalled lane repeated\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
        ]);

        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Regressing);
        assert_eq!(progress.confidence, Confidence::Medium);
        assert_has_signal(&progress, ProgressSignalCode::PreviouslyCleanScopeBroken);
        assert_has_signal(&progress, ProgressSignalCode::FailureSignatureRepeated);
        assert!(
            progress
                .signals
                .iter()
                .all(|signal| signal.polarity != SignalPolarity::Positive),
            "regressing plus stalled must not invent positive evidence"
        );
    }

    #[test]
    fn troubleshooting_latest_verified_edit_epoch_supersedes_earlier_informative_lanes() {
        let analysis = last_analysis(vec![
            prompt_row(0, "turn-001", "/goal Verify an earlier independent lane."),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test earlier_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest earlier_lane ... FAILED\n\nfailures:\n    earlier_lane\n\nthread 'earlier_lane' panicked at crates/earlier/src/lib.rs:10:5:\nassertion failed: earlier lane\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test earlier_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                4,
                "turn-001",
                "Exit code: 0\nrunning 1 test\ntest earlier_lane ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                5,
                "turn-002",
                "functions.apply_patch",
                r#"{"command":"apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\n*** End Patch\nPATCH","workdir":"/repo"}"#,
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test terminal_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                7,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest terminal_lane ... FAILED\n\nfailures:\n    terminal_lane\n\nthread 'terminal_lane' panicked at crates/target/src/lib.rs:20:5:\nassertion failed: terminal lane repeated\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                8,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test terminal_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                9,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest terminal_lane ... FAILED\n\nfailures:\n    terminal_lane\n\nthread 'terminal_lane' panicked at crates/target/src/lib.rs:20:5:\nassertion failed: terminal lane repeated\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
        ]);
        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_eq!(progress.confidence, Confidence::Medium);
        assert_has_signal(&progress, ProgressSignalCode::FailureSignatureRepeated);
        assert_lacks_signal(&progress, ProgressSignalCode::VerificationClean);
    }

    #[test]
    fn troubleshooting_unverified_trailing_edit_does_not_erase_latest_informative_lane() {
        let analysis = last_analysis(vec![
            prompt_row(0, "turn-001", "/goal Verify an earlier independent lane."),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test earlier_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest earlier_lane ... FAILED\n\nfailures:\n    earlier_lane\n\nthread 'earlier_lane' panicked at crates/earlier/src/lib.rs:10:5:\nassertion failed: earlier lane\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test earlier_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                4,
                "turn-001",
                "Exit code: 0\nrunning 1 test\ntest earlier_lane ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                5,
                "turn-002",
                "functions.apply_patch",
                r#"{"command":"apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/tests/checkpoints.rs\n*** End Patch\nPATCH","workdir":"/repo"}"#,
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test terminal_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                7,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest terminal_lane ... FAILED\n\nfailures:\n    terminal_lane\n\nthread 'terminal_lane' panicked at crates/target/src/lib.rs:20:5:\nassertion failed: terminal lane repeated\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                8,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test terminal_lane -- --exact","workdir":"/repo"}"#,
            ),
            tool_output_row(
                9,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest terminal_lane ... FAILED\n\nfailures:\n    terminal_lane\n\nthread 'terminal_lane' panicked at crates/target/src/lib.rs:20:5:\nassertion failed: terminal lane repeated\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out",
            ),
            tool_call_row(
                10,
                "turn-002",
                "functions.apply_patch",
                r#"{"command":"apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/tests/checkpoints.rs\n*** End Patch\nPATCH","workdir":"/repo"}"#,
            ),
        ]);

        let progress = assess_troubleshooting_progress(
            &analysis,
            ProgressDimension::TroubleshootingFrontier,
            SessionArchetypeLabel::Troubleshooting,
        );

        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_eq!(progress.confidence, Confidence::Medium);
        assert_has_signal(&progress, ProgressSignalCode::FailureSignatureRepeated);
        assert_lacks_signal(&progress, ProgressSignalCode::VerificationClean);
    }

    #[test]
    fn annotate_zero_verifier_fallback_exposes_counter_evidence_for_empty_conservative_path() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Troubleshoot the packet smoke conservatively before planning a fix.",
            ),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/MAP.md","workdir":"/repo"}"#,
            ),
            tool_output_row(2, "turn-001", "Exit code: 0"),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md","workdir":"/repo"}"#,
            ),
            tool_output_row(4, "turn-001", "Exit code: 0"),
            prompt_row(
                5,
                "turn-002",
                "/goal Troubleshoot the same packet smoke conservatively while checking broader exploratory evidence before planning a fix.",
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"where.exe flutter 2>$null","workdir":"/repo"}"#,
            ),
            tool_output_row(7, "turn-002", "Exit code: 1"),
            tool_call_row(
                8,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md","workdir":"/repo"}"#,
            ),
            tool_output_row(9, "turn-002", "Exit code: 0"),
            tool_call_row(
                10,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md","workdir":"/repo"}"#,
            ),
            tool_output_row(11, "turn-002", "Exit code: 0"),
            tool_call_row(
                12,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"& 'C:/tools/flutter/bin/flutter.bat' analyze","workdir":"D:/Downloads/Shared cab-Flutter","timeout_ms":120000}"#,
            ),
            tool_output_row(13, "turn-002", "Wall time: 9134.2 seconds\naborted by user"),
        ]);

        let progress = annotate_zero_verifier_fallback(
            &analysis,
            insufficient_progress(ProgressDimension::PlanningConvergence, None, None),
        );

        assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
        assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
        assert_has_signal(&progress, ProgressSignalCode::WorkingSetDiffused);
        assert!(
            progress
                .counter_evidence
                .iter()
                .any(|evidence| evidence.reason == ZERO_VERIFIER_ANTI_FLAP_REASON),
            "expected zero-verifier anti-flap counter evidence, got {:?}",
            progress
                .counter_evidence
                .iter()
                .map(|evidence| evidence.reason.as_str())
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn build_session_progress_falls_back_when_only_clean_verifiers_back_diffused_exploration() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Troubleshoot the exploratory packet evidence conservatively before deciding whether there is a real bug.",
            ),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/MAP.md","workdir":"/repo"}"#,
            ),
            tool_output_row(2, "turn-001", "Exit code: 0"),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md","workdir":"/repo"}"#,
            ),
            tool_output_row(4, "turn-001", "Exit code: 0"),
            prompt_row(
                5,
                "turn-002",
                "/goal Keep checking broader exploratory evidence before escalating to a real troubleshooting lane.",
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"python -c \"import ast, pathlib; [ast.parse(pathlib.Path(p).read_text(encoding='utf-8')) for p in ['D:/Explainable_VRU_Research_Final/kitti_full_arf.py','D:/Explainable_VRU_Research_Final/simulation_engine.py']]; print('syntax ok')\"","workdir":"D:/Explainable_VRU_Research_Final","timeout_ms":20000}"#,
            ),
            tool_output_row(7, "turn-002", "Exit code: 0\nsyntax ok"),
            tool_call_row(
                8,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"python -c \"from pathlib import Path; t=Path('D:/Explainable_VRU_Research_Final/paper_main.tex').read_text(encoding='utf-8'); print({'brace_balance': t.count('{')-t.count('}')})\"","workdir":"D:/Explainable_VRU_Research_Final","timeout_ms":20000}"#,
            ),
            tool_output_row(9, "turn-002", "Exit code: 0\n{'brace_balance': 0}"),
        ]);

        let progress = build_session_progress(
            &analysis,
            &SessionArchetype {
                label: SessionArchetypeLabel::Troubleshooting,
                confidence: Confidence::High,
                supporting_evidence: Vec::new(),
                counter_evidence: Vec::new(),
            },
        );

        assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
        assert_eq!(progress.status, ProgressStatus::InsufficientEvidence);
        assert_has_signal(&progress, ProgressSignalCode::WorkingSetDiffused);
        assert!(
            progress
                .counter_evidence
                .iter()
                .any(|evidence| evidence.reason == ZERO_VERIFIER_ANTI_FLAP_REASON),
            "expected clean verifier probes to keep the conservative fallback evidence, got {:?}",
            progress
                .counter_evidence
                .iter()
                .map(|evidence| evidence.reason.as_str())
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn build_session_progress_keeps_aborted_verifier_probe_cases_on_conservative_planning_lane() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Troubleshoot the packet smoke conservatively before planning a fix.",
            ),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/MAP.md","workdir":"/repo"}"#,
            ),
            tool_output_row(2, "turn-001", "Exit code: 0"),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md","workdir":"/repo"}"#,
            ),
            tool_output_row(4, "turn-001", "Exit code: 0"),
            prompt_row(
                5,
                "turn-002",
                "/goal Troubleshoot the same packet smoke conservatively while checking broader exploratory evidence before planning a fix.",
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"where.exe flutter 2>$null","workdir":"/repo"}"#,
            ),
            tool_output_row(7, "turn-002", "Exit code: 1"),
            tool_call_row(
                8,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md","workdir":"/repo"}"#,
            ),
            tool_output_row(9, "turn-002", "Exit code: 0"),
            tool_call_row(
                10,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"sed -n '1,120p' docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md","workdir":"/repo"}"#,
            ),
            tool_output_row(11, "turn-002", "Exit code: 0"),
            tool_call_row(
                12,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"& 'C:/tools/flutter/bin/flutter.bat' analyze","workdir":"D:/Downloads/Shared cab-Flutter","timeout_ms":120000}"#,
            ),
            tool_output_row(13, "turn-002", "Wall time: 9134.2 seconds\naborted by user"),
        ]);

        let progress = build_session_progress(
            &analysis,
            &SessionArchetype {
                label: SessionArchetypeLabel::Troubleshooting,
                confidence: Confidence::High,
                supporting_evidence: Vec::new(),
                counter_evidence: Vec::new(),
            },
        );

        assert_eq!(progress.dimension, ProgressDimension::PlanningConvergence);
        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_has_signal(&progress, ProgressSignalCode::CandidateSetExpanded);
        assert!(
            progress
                .counter_evidence
                .iter()
                .any(|evidence| evidence.reason == ZERO_VERIFIER_ANTI_FLAP_REASON),
            "expected aborted verifier probes to keep the conservative anti-flap counter evidence, got {:?}",
            progress
                .counter_evidence
                .iter()
                .map(|evidence| evidence.reason.as_str())
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn build_session_progress_keeps_failed_verifier_evidence_on_troubleshooting_frontier() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Troubleshoot the packet smoke failure conservatively before planning a fix.",
            ),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
            ),
            tool_output_row(
                2,
                "turn-001",
                "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nAssertionError: expected advancing",
            ),
            prompt_row(
                3,
                "turn-002",
                "/goal Re-run the same troubleshooting verifier before widening scope.",
            ),
            tool_call_row(
                4,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
            ),
            tool_output_row(
                5,
                "turn-002",
                "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nAssertionError: expected advancing",
            ),
        ]);

        let progress = build_session_progress(
            &analysis,
            &SessionArchetype {
                label: SessionArchetypeLabel::Troubleshooting,
                confidence: Confidence::High,
                supporting_evidence: Vec::new(),
                counter_evidence: Vec::new(),
            },
        );

        assert_eq!(
            progress.dimension,
            ProgressDimension::TroubleshootingFrontier
        );
        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_has_signal(&progress, ProgressSignalCode::FailureSignatureRepeated);
    }

    #[test]
    fn delegation_limiting_counter_evidence_survives_finalization_pressure_for_partial_visibility()
    {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Coordinate delegated findings without claiming child execution progress.",
            ),
            identified_tool_call_row(
                1,
                "turn-001",
                "spawn_agent",
                "{\"goal\":\"fix packet R5-4\"}",
            ),
            tool_call_row(
                2,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"printf 'child rollout ' && sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl","workdir":"/repo"}"#,
            ),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture","workdir":"/repo"}"#,
            ),
            tool_output_row(
                4,
                "turn-001",
                r#"Exit code: 101
running 1 test
test checkpoints::captures_progress ... FAILED

failures:
    checkpoints::captures_progress

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
AssertionError: expected advancing"#,
            ),
        ]);
        assert_eq!(
            analysis.delegation.topology,
            Some(DelegationTopology::DelegatingParent)
        );
        assert_eq!(
            effective_child_work_visibility(&analysis),
            ChildWorkVisibility::Partial
        );

        let mut progress = parent_visible_orchestration_progress(
            &analysis,
            SessionArchetypeLabel::AutonomousImplementation,
        )
        .expect("parent-visible orchestration progress");
        let limiting_signal = progress
            .signals
            .iter()
            .find(|signal| signal.code == ProgressSignalCode::DelegationVisibilityLimited)
            .expect("preexisting delegation limiting signal");
        assert!(
            limiting_signal
                .evidence
                .iter()
                .all(|evidence| evidence.reason != DELEGATION_LIMITING_CONFIDENCE_REASON),
            "expected parent-visible path to predate promoted limiting evidence, got {:?}",
            limiting_signal
                .evidence
                .iter()
                .map(|evidence| evidence.reason.as_str())
                .collect::<Vec<_>>()
        );
        progress
            .counter_evidence
            .extend(competing_counter_evidence());

        let progress = finalize_progress(apply_delegation_caps(&analysis, progress));

        assert_eq!(progress.confidence, Confidence::Medium);
        assert_has_signal(&progress, ProgressSignalCode::DelegationVisibilityLimited);
        assert_eq!(progress.counter_evidence.len(), MAX_PROGRESS_EVIDENCE_ITEMS);
        assert!(
            progress
                .counter_evidence
                .iter()
                .any(|evidence| { evidence.reason == DELEGATION_LIMITING_CONFIDENCE_REASON }),
            "expected delegated limiting evidence to survive finalization pressure, got {:?}",
            progress
                .counter_evidence
                .iter()
                .map(|evidence| evidence.reason.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn delegation_limiting_counter_evidence_survives_finalization_pressure_for_opaque_visibility() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Coordinate delegated work without overclaiming child progress.",
            ),
            identified_tool_call_row(
                1,
                "turn-001",
                "spawn_agent",
                "{\"goal\":\"fix packet R5-4\"}",
            ),
            identified_tool_call_row(
                2,
                "turn-001",
                "wait_agent",
                "{\"session_id\":\"019ea333-3333-7333-8333-333333333333\"}",
            ),
        ]);
        assert_eq!(
            analysis.delegation.topology,
            Some(DelegationTopology::DelegatingParent)
        );
        assert_eq!(
            effective_child_work_visibility(&analysis),
            ChildWorkVisibility::Opaque
        );

        let mut progress = parent_visible_orchestration_progress(
            &analysis,
            SessionArchetypeLabel::AutonomousImplementation,
        )
        .expect("parent-visible orchestration progress");
        let limiting_signal = progress
            .signals
            .iter()
            .find(|signal| signal.code == ProgressSignalCode::DelegationVisibilityLimited)
            .expect("preexisting delegation limiting signal");
        assert!(
            limiting_signal
                .evidence
                .iter()
                .all(|evidence| evidence.reason != DELEGATION_LIMITING_CONFIDENCE_REASON),
            "expected parent-visible path to predate promoted limiting evidence, got {:?}",
            limiting_signal
                .evidence
                .iter()
                .map(|evidence| evidence.reason.as_str())
                .collect::<Vec<_>>()
        );
        progress
            .counter_evidence
            .extend(competing_counter_evidence());

        let progress = finalize_progress(apply_delegation_caps(&analysis, progress));

        assert_eq!(progress.confidence, Confidence::Low);
        assert_has_signal(&progress, ProgressSignalCode::DelegationVisibilityLimited);
        assert_eq!(progress.counter_evidence.len(), MAX_PROGRESS_EVIDENCE_ITEMS);
        assert!(
            progress
                .counter_evidence
                .iter()
                .any(|evidence| { evidence.reason == DELEGATION_LIMITING_CONFIDENCE_REASON }),
            "expected delegated limiting evidence to survive finalization pressure, got {:?}",
            progress
                .counter_evidence
                .iter()
                .map(|evidence| evidence.reason.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn parent_visible_progress_carries_across_multiple_empty_followups() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Coordinate delegated work without overclaiming child progress.",
            ),
            identified_tool_call_row(
                1,
                "turn-001",
                "spawn_agent",
                r#"{"goal":"inspect packet R5-4"}"#,
            ),
            identified_tool_call_row(
                2,
                "turn-001",
                "wait_agent",
                "{\"session_id\":\"019ea777-7777-7777-8777-777777777777\"}",
            ),
            tool_call_row(
                3,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"git status --short && git diff --stat","workdir":"/repo"}"#,
            ),
            prompt_row(
                4,
                "turn-002",
                "/goal Coordinate delegated work without overclaiming child progress.",
            ),
            prompt_row(
                5,
                "turn-003",
                "/goal Coordinate delegated work without overclaiming child progress.",
            ),
        ]);

        let progress =
            parent_visible_orchestration_progress(&analysis, SessionArchetypeLabel::Planning)
                .expect("parent-visible followup progress");

        assert_eq!(
            progress.dimension,
            ProgressDimension::ParentVisibleOrchestration
        );
        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_eq!(progress.confidence, Confidence::Low);
    }

    #[test]
    fn opaque_parent_visible_broad_scans_stay_stalled_without_child_progress() {
        let analysis = last_analysis(vec![
            prompt_row(
                0,
                "turn-001",
                "/goal Coordinate delegated work without overclaiming child progress.",
            ),
            tool_call_row(
                1,
                "turn-001",
                "functions.shell_command",
                r#"{"command":"sed -n '1,80p' docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md","workdir":"/repo"}"#,
            ),
            prompt_row(
                2,
                "turn-002",
                "/goal Coordinate delegated work without overclaiming child progress.",
            ),
            identified_tool_call_row(
                3,
                "turn-002",
                "spawn_agent",
                r#"{"goal":"inspect delegated packet R5-75"}"#,
            ),
            identified_tool_call_row(
                4,
                "turn-002",
                "wait_agent",
                r#"{"session_id":"019ea333-3333-7333-8333-333333333333"}"#,
            ),
            tool_call_row(
                5,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"sed -n '1,80p' docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-plan.md","workdir":"/repo"}"#,
            ),
            tool_call_row(
                6,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"sed -n '1,80p' docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md","workdir":"/repo"}"#,
            ),
            tool_call_row(
                7,
                "turn-002",
                "functions.shell_command",
                r#"{"command":"rg -n \"parent_visible_orchestration|delegation\" crates/agent-drift-analyzer/src/checkpoint/progress.rs crates/agent-drift-analyzer/tests/checkpoints.rs","workdir":"/repo"}"#,
            ),
        ]);

        assert_eq!(
            analysis.delegation.topology,
            Some(DelegationTopology::DelegatingParent)
        );
        assert_eq!(
            effective_child_work_visibility(&analysis),
            ChildWorkVisibility::Opaque
        );

        let progress =
            parent_visible_orchestration_progress(&analysis, SessionArchetypeLabel::Planning)
                .expect("parent-visible orchestration progress");

        assert_eq!(
            progress.dimension,
            ProgressDimension::ParentVisibleOrchestration
        );
        assert_eq!(progress.status, ProgressStatus::Stalled);
        assert_eq!(progress.confidence, Confidence::Low);
        assert_has_signal(&progress, ProgressSignalCode::DelegationVisibilityLimited);
        assert!(
            progress.counter_evidence.iter().any(|evidence| {
                evidence
                    .reason
                    .contains("repeated broad planning scan without convergence artifact")
            }),
            "expected broad-scan stall evidence in counter_evidence, got {:?}",
            progress
                .counter_evidence
                .iter()
                .map(|evidence| evidence.reason.as_str())
                .collect::<Vec<_>>(),
        );
    }

    fn last_analysis(rows: Vec<CompactionRow>) -> CheckpointAnalysis {
        let session = BundleSession {
            session_id: "session-r5-4".to_string(),
            archival_rows: rows.clone(),
            compact_rows: rows,
        };
        checkpoint_analyses(&session)
            .pop()
            .expect("checkpoint analysis")
    }

    fn prompt_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
        row(
            event_index,
            turn_id,
            CompactionKind::UserMessage,
            text,
            None,
        )
    }

    fn tool_output_row(event_index: usize, turn_id: &str, text: &str) -> CompactionRow {
        row(event_index, turn_id, CompactionKind::ToolOutput, text, None)
    }

    fn tool_call_row(
        event_index: usize,
        turn_id: &str,
        _tool_name: &str,
        text: &str,
    ) -> CompactionRow {
        row(event_index, turn_id, CompactionKind::ToolCall, text, None)
    }

    fn identified_tool_call_row(
        event_index: usize,
        turn_id: &str,
        tool_name: &str,
        text: &str,
    ) -> CompactionRow {
        row(
            event_index,
            turn_id,
            CompactionKind::ToolCall,
            text,
            Some(format!(
                "{{\"call_id\":\"call-{event_index}\",\"name\":\"{tool_name}\",\"type\":\"function_call\"}}"
            )),
        )
    }

    fn row(
        event_index: usize,
        turn_id: &str,
        kind: CompactionKind,
        text: &str,
        dedupe_identity: Option<String>,
    ) -> CompactionRow {
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/session-r5-4/rollout.jsonl"),
            source_kind: agent_session_compactor::SourceKind::CodexRolloutJsonl,
            session_id: Some("session-r5-4".to_string()),
            turn_id: Some(turn_id.to_string()),
            event_index,
            line_number: event_index + 1,
            row_ordinal: 0,
            timestamp: None,
            kind,
            user_message_role: matches!(kind, CompactionKind::UserMessage)
                .then_some(UserMessageRole::Prompt),
            dedupe_identity,
            text: text.to_string(),
            canonical_text: text.to_string(),
            text_hash_hex: format!("hash-{}", text.split_whitespace().collect::<String>()),
        }
    }

    fn assert_has_signal(progress: &SessionProgress, code: ProgressSignalCode) {
        assert!(
            progress.signals.iter().any(|signal| signal.code == code),
            "expected progress signal {:?}, got {:?}",
            code,
            progress
                .signals
                .iter()
                .map(|signal| signal.code)
                .collect::<Vec<_>>()
        );
    }

    fn assert_lacks_signal(progress: &SessionProgress, code: ProgressSignalCode) {
        assert!(
            progress.signals.iter().all(|signal| signal.code != code),
            "expected progress signal {:?} to be absent, got {:?}",
            code,
            progress
                .signals
                .iter()
                .map(|signal| signal.code)
                .collect::<Vec<_>>()
        );
    }

    fn competing_counter_evidence() -> Vec<EvidenceRef> {
        (0..=MAX_PROGRESS_EVIDENCE_ITEMS)
            .map(|index| EvidenceRef {
                row: RowRef {
                    source_file: Utf8PathBuf::from("/aaa/competing-counter-evidence.jsonl"),
                    event_index: index,
                    row_ordinal: 0,
                },
                reason: format!("competing counter evidence {index:02}"),
            })
            .collect()
    }
}
