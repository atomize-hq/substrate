use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use agent_session_compactor::RowRef;

use crate::inference::{ChildWorkVisibility, DelegationTopology};

use super::attempt::{
    AttemptOutcome, CommandAttempt, CommandAttemptRole, ExerciseState, VerificationAttempt,
    VerificationScope, VerifierKind,
};
use super::diagnostics::{
    classify_edit_overlap, match_diagnostic_signatures, DiagnosticMatchKind, DiagnosticSignature,
    EditOverlapStrength, FailureClass,
};
use super::{
    CheckpointAnalysis, Confidence, EvidenceRef, ProgressDimension, ProgressSignal,
    ProgressSignalCode, ProgressStatus, SessionArchetype, SessionArchetypeLabel, SessionProgress,
    SignalPolarity, SignalStrength,
};

const MAX_PROGRESS_EVIDENCE_ITEMS: usize = 8;
const MAX_COMPARABLE_CHECKPOINT_CHAIN: usize = 6;

pub(crate) fn build_session_progress(
    analysis: &CheckpointAnalysis,
    session_archetype: &SessionArchetype,
) -> SessionProgress {
    if let Some(progress) = parent_visible_orchestration_progress(analysis, session_archetype.label)
    {
        return progress;
    }

    let mut progress = match session_archetype.label {
        SessionArchetypeLabel::Troubleshooting => {
            assess_troubleshooting_progress(
                analysis,
                default_dimension(session_archetype.label),
                session_archetype.label,
            )
        }
        SessionArchetypeLabel::Planning => {
            assess_planning_progress(analysis, default_dimension(session_archetype.label))
        }
        SessionArchetypeLabel::AutonomousImplementation => {
            assess_implementation_progress(
                analysis,
                default_dimension(session_archetype.label),
                session_archetype.label,
            )
        }
        SessionArchetypeLabel::VerificationCloseout => {
            assess_closeout_progress(
                analysis,
                default_dimension(session_archetype.label),
                session_archetype.label,
            )
        }
    };

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
    let parent_visible_synthesis_case =
        visible_child_surface && !synthesis_attempts.is_empty() && source_edits(analysis).is_empty();
    if !parent_visible_synthesis_case
        && ((orchestration_attempts.is_empty() && synthesis_attempts.is_empty())
            || !has_parent_visible_orchestration_evidence(analysis, visibility))
    {
        return None;
    }

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
        analysis
            .delegation
            .counter_evidence
            .iter()
            .map(|evidence| EvidenceRef {
                row: evidence.row.clone(),
                reason: "child-opaque delegation limited direct child progress claims".to_string(),
            })
            .collect(),
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

    let parent_synthesis =
        has_parent_visible_synthesis(&orchestration_attempts, &synthesis_attempts);
    let prior_parent_visible = previous_checkpoint_was_parent_visible(analysis);
    let status = if parent_synthesis {
        ProgressStatus::Mixed
    } else if prior_parent_visible {
        ProgressStatus::Stalled
    } else {
        ProgressStatus::InsufficientEvidence
    };

    let confidence = match visibility {
        ChildWorkVisibility::Opaque => Confidence::Low,
        ChildWorkVisibility::Partial => Confidence::Medium,
        ChildWorkVisibility::None => Confidence::Low,
    };

    Some(SessionProgress {
        status,
        dimension: ProgressDimension::ParentVisibleOrchestration,
        confidence,
        supporting_evidence: if status == ProgressStatus::InsufficientEvidence {
            Vec::new()
        } else {
            limiting_signal.evidence.clone()
        },
        counter_evidence: Vec::new(),
        signals: vec![limiting_signal],
    })
}

fn assess_troubleshooting_progress(
    analysis: &CheckpointAnalysis,
    dimension: ProgressDimension,
    archetype_label: SessionArchetypeLabel,
) -> SessionProgress {
    let current_attempts = &analysis.interval.verification_attempts;
    let Some(current) = current_attempts.last() else {
        return insufficient_progress(dimension, None, None);
    };
    let prior_attempts = comparable_attempts(analysis, current_attempts, current, archetype_label);
    let best_failed = best_failed_attempt(&prior_attempts);
    let best_clean = best_clean_attempt(&prior_attempts);
    let latest_failed = latest_failed_attempt(&prior_attempts);

    if current.outcome == AttemptOutcome::Clean {
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
        let frontier_advanced =
            frontier_rank(current_signature.failure_class)
                > frontier_rank(previous_signature.failure_class);
        if frontier_advanced {
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
        if fail_count(current_signature) < fail_count(previous_signature) {
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
        if fail_count(current_signature) > fail_count(previous_signature) {
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
        if matches!(
            match_kind,
            DiagnosticMatchKind::Exact | DiagnosticMatchKind::StrongFuzzy
        ) && edit_overlap.strength == EditOverlapStrength::None
        {
            signals.push(progress_signal(
                ProgressSignalCode::FailureSignatureRepeated,
                SignalPolarity::Negative,
                SignalStrength::Strong,
                "the same normalized failure signature repeated without an overlapping fix edit",
                Some(previous_signature.preview.clone()),
                Some(current_signature.preview.clone()),
                merge_evidence(vec![
                    attempt_evidence(
                        previous_failed,
                        "earlier repeated failing verification attempt",
                    ),
                    attempt_evidence(current, "later repeated failing verification attempt"),
                ]),
            ));
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
                || matches!(
                    match_kind,
                    DiagnosticMatchKind::Exact
                        | DiagnosticMatchKind::StrongFuzzy
                        | DiagnosticMatchKind::FrontierRelated
                ))
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

    classify_progress_outcome(dimension, signals, Confidence::High, Confidence::Medium)
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

    if current.outcome == AttemptOutcome::Clean {
        if let Some(previous_failed) = best_failed {
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
                    ]),
                ));
            }
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
        let edit_overlap = classify_edit_overlap(
            previous_failed,
            current,
            &analysis.interval.command_attempts,
        );

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
                || fail_count(current_signature) < fail_count(previous_signature)
            {
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
    if positive {
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
    let Some(current) = current_attempts.last() else {
        return insufficient_progress(dimension, None, None);
    };
    let prior_attempts = comparable_attempts(analysis, current_attempts, current, archetype_label);
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

    if let Some(previous_clean) = best_clean {
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

    if current.outcome != AttemptOutcome::Clean {
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

    if let Some(previous) = prior_attempts.first() {
        if scope_is_narrower(&current.target_scope, &previous.target_scope) {
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

    progress_from_signals(
        ProgressStatus::Advancing,
        dimension,
        if source_edits.is_empty() {
            Confidence::High
        } else {
            Confidence::Medium
        },
        signals,
        Vec::new(),
    )
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
            delegation_supporting_evidence(
                analysis,
                "delegation visibility limited progress confidence",
            ),
        ));
    }

    progress
}

fn finalize_progress(mut progress: SessionProgress) -> SessionProgress {
    progress.signals.sort_by(signal_sort_key);
    progress.supporting_evidence = dedupe_and_limit_evidence(progress.supporting_evidence);
    progress.counter_evidence = dedupe_and_limit_evidence(progress.counter_evidence);
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
    let positive = has_positive_signal(&signals);
    let negative = has_negative_signal(&signals);
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
            ProgressStatus::Advancing,
            dimension,
            positive_confidence,
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

fn prior_verification_attempts(
    analysis: &CheckpointAnalysis,
    current: &VerificationAttempt,
) -> Vec<VerificationAttempt> {
    let Some(_) = &analysis.previous else {
        return Vec::new();
    };
    let analyses = super::checkpoint_analyses(&analysis.current.window);
    let Some(current_index) = analyses.len().checked_sub(1) else {
        return Vec::new();
    };

    let mut collected = Vec::new();
    let mut newer = &analyses[current_index];
    let mut checkpoints_scanned = 0;
    for older in analyses[..current_index].iter().rev() {
        if checkpoints_scanned >= MAX_COMPARABLE_CHECKPOINT_CHAIN {
            break;
        }
        if checkpoints_scanned > 0 && comparable_window_boundary(newer, older, current) {
            break;
        }

        collected.extend(older.interval.verification_attempts.clone());
        checkpoints_scanned += 1;
        newer = older;
    }

    collected
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
    let objective_shifted = normalize_task_text(&newer.current.task_frame.objective)
        != normalize_task_text(&older.current.task_frame.objective);

    truth_shifted || (objective_shifted && truth_shifted)
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
    if left.outcome == AttemptOutcome::Clean || right.outcome == AttemptOutcome::Clean {
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

fn best_signature(attempt: &VerificationAttempt) -> Option<&DiagnosticSignature> {
    attempt
        .signatures
        .iter()
        .min_by(|left, right| signature_sort_key(left).cmp(&signature_sort_key(right)))
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

fn fail_count(signature: &DiagnosticSignature) -> u32 {
    signature.failing_count.unwrap_or(u32::MAX)
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
    !analysis.interval.command_attempts.is_empty()
        && source_edits(analysis).is_empty()
        && analysis.interval.command_attempts.iter().all(|attempt| {
            is_parent_visible_attempt(attempt, Some(visibility))
        })
        && match visibility {
            ChildWorkVisibility::Opaque => {
                parent_visible_synthesis_attempts(analysis).is_empty()
                    || parent_visible_orchestration_attempts(analysis)
                        .iter()
                        .any(|attempt| matches!(attempt.tool_name.as_str(), "close_agent" | "multi_agent_v1"))
            }
            ChildWorkVisibility::Partial => true,
            ChildWorkVisibility::None => false,
        }
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
    analysis.delegation.supporting_evidence.iter().any(|evidence| {
        evidence
            .reason
            .contains("delegation child rollout surface links child/subagent work")
    })
}

fn is_parent_visible_attempt(
    attempt: &CommandAttempt,
    visibility: Option<ChildWorkVisibility>,
) -> bool {
    matches!(attempt.role, CommandAttemptRole::Orchestration | CommandAttemptRole::Read)
        || matches!(visibility, Some(ChildWorkVisibility::Partial))
            && is_parent_visible_synthesis_attempt(attempt)
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
    orchestration_attempts: &[&CommandAttempt],
    synthesis_attempts: &[&CommandAttempt],
) -> bool {
    !synthesis_attempts.is_empty()
        || orchestration_attempts
            .iter()
            .any(|attempt| matches!(attempt.tool_name.as_str(), "close_agent" | "multi_agent_v1"))
}

fn previous_checkpoint_was_parent_visible(analysis: &CheckpointAnalysis) -> bool {
    let analyses = super::checkpoint_analyses(&analysis.current.window);
    analyses
        .iter()
        .rev()
        .nth(1)
        .is_some_and(|previous| {
            has_parent_visible_orchestration_evidence(
                previous,
                effective_child_work_visibility(previous),
            )
        })
}

fn working_set(paths: &[String]) -> BTreeSet<String> {
    paths.iter().cloned().collect()
}

fn working_set_narrowed(previous: &BTreeSet<String>, current: &BTreeSet<String>) -> bool {
    !current.is_empty()
        && ((!previous.is_empty() && current.len() < previous.len() && current.is_subset(previous))
            || (previous.is_empty() && current.len() <= 3))
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
        .filter(|candidate| candidate.outcome == AttemptOutcome::Clean)
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
    failed_attempt_frontier_key(left).cmp(&failed_attempt_frontier_key(right))
}

fn failed_attempt_frontier_key(attempt: &VerificationAttempt) -> (u8, u8, u32, usize, usize) {
    let signature = best_signature(attempt);
    (
        matches!(attempt.exercise_state, ExerciseState::TargetExercised) as u8,
        signature
            .map(|signature| frontier_rank(signature.failure_class))
            .unwrap_or_default(),
        signature
            .map(|signature| u32::MAX.saturating_sub(fail_count(signature)))
            .unwrap_or_default(),
        scope_cardinality(&attempt.target_scope),
        attempt.attempt_ordinal,
    )
}

fn verification_attempt_preview(attempt: &VerificationAttempt) -> String {
    if attempt.outcome == AttemptOutcome::Clean {
        format!("clean {}", attempt.target_scope.raw)
    } else {
        best_signature(attempt)
            .map(|signature| signature.preview.clone())
            .unwrap_or_else(|| attempt.target_scope.raw.clone())
    }
}

fn has_direct_verifier_progress_signal(signals: &[ProgressSignal]) -> bool {
    signals.iter().any(|signal| {
        matches!(
            signal.code,
            ProgressSignalCode::FailureFrontierAdvanced | ProgressSignalCode::VerificationClean
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
    let mut deduped = items
        .into_iter()
        .fold(BTreeMap::new(), |mut acc, evidence| {
            acc.entry((
                evidence.row.source_file.clone(),
                evidence.row.event_index,
                evidence.row.row_ordinal,
                evidence.reason.clone(),
            ))
            .or_insert(evidence);
            acc
        })
        .into_values()
        .collect::<Vec<_>>();
    deduped.sort_by(|left, right| {
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
    });
    deduped.truncate(MAX_PROGRESS_EVIDENCE_ITEMS);
    deduped
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
