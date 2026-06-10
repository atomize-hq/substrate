mod attempt;
mod diagnostics;
mod export;
mod schema;

use std::collections::{BTreeMap, BTreeSet};

use crate::input::BundleSession;
use crate::{
    context::assemble_context, context::collect_command_observations,
    context::focusable_directive_rows, context::CommandObservation, context::ContextPack,
    inference::infer_delegation_context, inference::infer_task_frame,
    inference::ChildWorkVisibility, inference::DelegationContext, inference::DelegationTopology,
    scoring::DriftStateHint, scoring::ScoredDrift,
};
use agent_session_compactor::{CompactionKind, CompactionRow, RowRef, UserMessageRole};
use attempt::{
    build_command_attempts, build_verification_attempts, CommandAttempt, VerificationAttempt,
};
use camino::Utf8PathBuf;

pub use export::{
    export_checkpoints, summarize_checkpoint_diagnostics, CheckpointDiagnosticStats,
    ConfidenceDistribution, ExportError, ExportResult,
};
pub use schema::{
    Checkpoint, CheckpointBoundary, CheckpointDiagnostics, Confidence, DriftClass, DriftScore,
    DriftState, EvidenceRef, ProgressDimension, ProgressSignal, ProgressSignalCode, ProgressStatus,
    SessionArchetype, SessionArchetypeLabel, SessionProgress, SignalPolarity, SignalStrength,
    TaskFrame, TurnActivityMix, TurnContext, TurnExecutionMode,
};

const MAX_ROWS_PER_CHECKPOINT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointAnalysis {
    pub session_id: String,
    pub ordinal: usize,
    pub current: CheckpointSlice,
    pub previous: Option<CheckpointSlice>,
    pub delegation: DelegationContext,
    pub interval: IntervalSlice,
    pub turn_context: TurnContext,
    pub repetition: RepetitionSlice,
    pub task_frame_delta: TaskFrameDelta,
    pub recovery: RecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointSlice {
    pub window: BundleSession,
    pub context: ContextPack,
    pub task_frame: TaskFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntervalSlice {
    pub archival_rows: Vec<CompactionRow>,
    pub compact_rows: Vec<CompactionRow>,
    pub command_observations: Vec<CommandObservation>,
    pub command_attempts: Vec<CommandAttempt>,
    pub verification_attempts: Vec<VerificationAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepetitionSlice {
    pub compact_rows: Vec<CompactionRow>,
    pub repeated_verification_loops: Vec<RepeatedCommandLoop>,
    pub repeated_failure_loops: Vec<RepeatedFailureLoop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepeatedCommandLoop {
    pub raw_command: String,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepeatedFailureLoop {
    pub text_hash_hex: String,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutcomeEvidenceKind {
    Failure,
    Neutral,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TaskFrameDelta {
    pub task_frame_transitioned: bool,
    pub working_set_changed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RecoveryState {
    pub interval_verification_command_count: usize,
    pub clean_verification_interval: bool,
    pub recovered_from_thrash: bool,
    pub active_repeated_verification: bool,
    pub active_repeated_failure: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct IntentEvidenceProfile {
    exploration_like: EvidenceBucket,
    implementation_like: EvidenceBucket,
    verification_like: EvidenceBucket,
    orchestration_like: EvidenceBucket,
    exploration_command_count: usize,
    source_write_command_count: usize,
    test_or_golden_write_command_count: usize,
    docs_or_spec_write_command_count: usize,
    verification_command_count: usize,
    orchestration_command_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EvidenceBucket {
    raw_score: i32,
    evidence: Vec<EvidenceRef>,
    counter_evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
enum EvidenceStrength {
    #[default]
    None,
    Weak,
    Moderate,
    Strong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandRole {
    Exploration,
    Implementation,
    Verification,
    Orchestration,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FileRole {
    Source,
    TestOrGolden,
    DocsOrSpec,
    ConfigOrBuild,
    GeneratedArtifact,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArchetypeCandidate {
    label: SessionArchetypeLabel,
    score: i32,
    supporting_evidence: Vec<EvidenceRef>,
    counter_evidence: Vec<EvidenceRef>,
}

const ENABLE_KICKOFF_PRIORS: bool = false;
const MAX_ARCHETYPE_EVIDENCE_ITEMS: usize = 8;

impl EvidenceBucket {
    fn add(&mut self, score: i32, evidence: Vec<EvidenceRef>) {
        if score <= 0 || evidence.is_empty() {
            return;
        }

        self.raw_score += score;
        self.evidence.extend(evidence);
    }

    fn add_counter(&mut self, evidence: Vec<EvidenceRef>) {
        if evidence.is_empty() {
            return;
        }

        self.counter_evidence.extend(evidence);
    }

    fn strength(&self) -> EvidenceStrength {
        match self.raw_score {
            i32::MIN..=0 => EvidenceStrength::None,
            1..=2 => EvidenceStrength::Weak,
            3..=5 => EvidenceStrength::Moderate,
            _ => EvidenceStrength::Strong,
        }
    }
}

pub(crate) fn checkpoint_analyses(session: &BundleSession) -> Vec<CheckpointAnalysis> {
    let mut analyses = Vec::new();
    let mut previous = None;
    let prompts_observed_in_session = prompts_observed_in_session(session);
    let mut checkpoints_in_turn = BTreeMap::<TurnSliceIdentity, usize>::new();

    for (index, window) in checkpoint_windows(session).into_iter().enumerate() {
        let context = assemble_context(&window);
        let task_frame = infer_task_frame(&context);
        let delegation =
            classify_checkpoint_delegation(infer_delegation_context(&window, &context));
        let current = CheckpointSlice {
            window,
            context,
            task_frame,
        };
        let interval = interval_slice(previous.as_ref(), &current);
        let turn_context = turn_context(
            session,
            &current,
            &interval,
            prompts_observed_in_session,
            &mut checkpoints_in_turn,
        );
        let repetition = repetition_slice(&current);
        let task_frame_delta = task_frame_delta(previous.as_ref(), &current);
        let recovery = recovery_state(&current, &interval, &repetition);

        analyses.push(CheckpointAnalysis {
            session_id: current.window.session_id.clone(),
            ordinal: index + 1,
            current: current.clone(),
            previous: previous.clone(),
            delegation,
            interval,
            turn_context,
            repetition,
            task_frame_delta,
            recovery,
        });

        previous = Some(current);
    }

    analyses
}

fn classify_checkpoint_delegation(mut delegation: DelegationContext) -> DelegationContext {
    let topology = derive_delegation_topology(&delegation);
    let child_work_visibility = derive_child_work_visibility(&delegation, topology);
    let confidence = derive_delegation_confidence(&delegation, topology, child_work_visibility);

    delegation.topology = Some(topology);
    delegation.child_work_visibility = Some(child_work_visibility);
    delegation.confidence = Some(confidence);
    delegation
}

fn derive_delegation_topology(delegation: &DelegationContext) -> DelegationTopology {
    if delegation.markers.is_empty() {
        return DelegationTopology::SingleAgent;
    }

    if delegation.markers.iter().any(|marker| {
        matches!(
            marker.as_str(),
            "spawn_agent" | "wait_agent" | "close_agent"
        )
    }) {
        return DelegationTopology::DelegatingParent;
    }

    DelegationTopology::MixedOrAmbiguous
}

fn derive_child_work_visibility(
    delegation: &DelegationContext,
    topology: DelegationTopology,
) -> ChildWorkVisibility {
    if matches!(topology, DelegationTopology::SingleAgent) {
        return ChildWorkVisibility::None;
    }

    if delegation_supports_partial_child_visibility(delegation) {
        return ChildWorkVisibility::Partial;
    }

    ChildWorkVisibility::Opaque
}

fn delegation_supports_partial_child_visibility(delegation: &DelegationContext) -> bool {
    delegation.supporting_evidence.iter().any(|evidence| {
        evidence
            .reason
            .contains("delegation child rollout surface links child/subagent work")
    })
}

fn derive_delegation_confidence(
    delegation: &DelegationContext,
    topology: DelegationTopology,
    child_work_visibility: ChildWorkVisibility,
) -> Confidence {
    match topology {
        DelegationTopology::SingleAgent => Confidence::High,
        DelegationTopology::DelegatingParent => match child_work_visibility {
            ChildWorkVisibility::Partial => Confidence::Medium,
            ChildWorkVisibility::Opaque => {
                if delegation.counter_evidence.is_empty() {
                    Confidence::Medium
                } else {
                    Confidence::Low
                }
            }
            ChildWorkVisibility::None => Confidence::Low,
        },
        DelegationTopology::DelegatedChild | DelegationTopology::MixedOrAmbiguous => {
            Confidence::Low
        }
    }
}

pub(crate) fn build_session_checkpoint_from_analysis(
    analysis: &CheckpointAnalysis,
    task_frame: &TaskFrame,
    drift_scores: Vec<DriftScore>,
) -> Checkpoint {
    build_session_checkpoint_from_analysis_with_ordinal(
        analysis,
        analysis.ordinal,
        task_frame,
        drift_scores,
    )
}

pub(crate) fn assign_drift_states(
    drift_scores: Vec<ScoredDrift>,
    previous_drift_scores: Option<&[DriftScore]>,
) -> Vec<DriftScore> {
    drift_scores
        .into_iter()
        .map(|scored| DriftScore {
            state: drift_state_for_score(&scored.score, scored.state_hint, previous_drift_scores),
            ..scored.score
        })
        .collect()
}

fn build_session_checkpoint_from_analysis_with_ordinal(
    analysis: &CheckpointAnalysis,
    ordinal: usize,
    task_frame: &TaskFrame,
    drift_scores: Vec<DriftScore>,
) -> Checkpoint {
    let boundary = checkpoint_boundary(&analysis.current.window);
    let diagnostics = checkpoint_diagnostics_from_analysis(analysis, task_frame, &drift_scores);
    let expected_next_step = expected_next_step(task_frame);
    let session_archetype = build_session_archetype(analysis);
    let session_progress = build_conservative_session_progress(&session_archetype);
    Checkpoint {
        schema_version: "v0.6".to_string(),
        session_id: analysis.session_id.clone(),
        checkpoint_id: format!("{}:{ordinal:04}", analysis.session_id),
        ordinal,
        boundary,
        turn_context: Some(analysis.turn_context.clone()),
        diagnostics,
        task_frame: task_frame.clone(),
        session_archetype: Some(session_archetype),
        session_progress: Some(session_progress),
        flagged: drift_scores.iter().any(|score| score.flagged),
        drift_scores,
        expected_next_step,
    }
}

fn build_conservative_session_progress(session_archetype: &SessionArchetype) -> SessionProgress {
    SessionProgress {
        status: ProgressStatus::InsufficientEvidence,
        dimension: progress_dimension_for_archetype(session_archetype.label),
        confidence: Confidence::Low,
        signals: Vec::new(),
        supporting_evidence: Vec::new(),
        counter_evidence: Vec::new(),
    }
}

fn progress_dimension_for_archetype(label: SessionArchetypeLabel) -> ProgressDimension {
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

fn build_session_archetype(analysis: &CheckpointAnalysis) -> SessionArchetype {
    let intent = build_intent_evidence_profile(analysis);
    let mut candidates = aggregate_session_archetype(analysis, &intent);
    candidates.sort_by(|left, right| {
        right.score.cmp(&left.score).then_with(|| {
            archetype_label_sort_key(left.label).cmp(&archetype_label_sort_key(right.label))
        })
    });

    let winner = candidates
        .first()
        .cloned()
        .unwrap_or_else(|| ArchetypeCandidate {
            label: SessionArchetypeLabel::Planning,
            score: 0,
            supporting_evidence: Vec::new(),
            counter_evidence: Vec::new(),
        });
    let runner_up_score = candidates.get(1).map_or(0, |candidate| candidate.score);
    let confidence = archetype_confidence(analysis, &intent, &winner, runner_up_score);
    let supporting_evidence = dedupe_and_limit_evidence(combine_evidence(vec![
        winner.supporting_evidence,
        delegation_supporting_archetype_evidence(analysis),
    ]));
    let counter_evidence = dedupe_and_limit_evidence(combine_evidence(vec![
        winner.counter_evidence,
        delegation_counter_archetype_evidence(analysis),
    ]));

    SessionArchetype {
        label: winner.label,
        confidence,
        supporting_evidence,
        counter_evidence,
    }
}

fn build_intent_evidence_profile(analysis: &CheckpointAnalysis) -> IntentEvidenceProfile {
    let mut profile = IntentEvidenceProfile::default();

    for command in &analysis.interval.command_observations {
        let role = classify_command_role(command);
        let file_roles = command_file_roles(command);
        let source_scope = file_roles.contains(&FileRole::Source);
        let test_scope = file_roles.contains(&FileRole::TestOrGolden);
        let docs_scope = file_roles.contains(&FileRole::DocsOrSpec);
        let config_scope = file_roles.contains(&FileRole::ConfigOrBuild);
        let docs_only = docs_scope && !source_scope && !test_scope && !config_scope;
        let test_only = test_scope && !source_scope;

        match role {
            CommandRole::Exploration => {
                profile.exploration_command_count += 1;
                profile.exploration_like.add(
                    2,
                    command_evidence(
                        command,
                        "inspection-style command widened the visible search space",
                    ),
                );
            }
            CommandRole::Implementation => {
                if source_scope {
                    profile.source_write_command_count += 1;
                    profile.implementation_like.add(
                        3,
                        command_evidence(
                            command,
                            "source edit command strengthened implementation-like evidence",
                        ),
                    );
                } else if test_only {
                    profile.test_or_golden_write_command_count += 1;
                    profile.verification_like.add(
                        2,
                        command_evidence(
                            command,
                            "test-or-golden-only edit behaved more like verification support than direct source implementation",
                        ),
                    );
                    profile.implementation_like.add_counter(command_evidence(
                        command,
                        "test-or-golden-only edit did not directly prove source implementation",
                    ));
                } else if docs_only {
                    profile.docs_or_spec_write_command_count += 1;
                    profile.exploration_like.add(
                        1,
                        command_evidence(
                            command,
                            "docs/spec-only edit behaved like scope-shaping rather than direct source implementation",
                        ),
                    );
                    profile.orchestration_like.add(
                        1,
                        command_evidence(
                            command,
                            "docs/spec-only edit behaved like planning-orchestration context",
                        ),
                    );
                    profile.implementation_like.add_counter(command_evidence(
                        command,
                        "docs/spec-only edit did not directly prove source implementation",
                    ));
                } else if config_scope {
                    profile.implementation_like.add(
                        2,
                        command_evidence(
                            command,
                            "config/build edit supported implementation-like work on the active scope",
                        ),
                    );
                    profile.verification_like.add(
                        1,
                        command_evidence(
                            command,
                            "config/build edit also supported build-and-verify preparation",
                        ),
                    );
                } else {
                    profile.implementation_like.add(
                        1,
                        command_evidence(
                            command,
                            "write-like command weakly supported implementation-like work",
                        ),
                    );
                }
            }
            CommandRole::Verification => {
                profile.verification_command_count += 1;
                if docs_only {
                    profile.exploration_like.add(
                        1,
                        command_evidence(
                            command,
                            "docs/spec-scoped verification command supported planning-style inspection more than closeout proof",
                        ),
                    );
                    profile.verification_like.add_counter(command_evidence(
                        command,
                        "docs/spec-scoped verification command was too broad to act as closeout proof by itself",
                    ));
                } else {
                    profile.verification_like.add(
                        2,
                        command_evidence(
                            command,
                            "verification command strengthened verification-like evidence",
                        ),
                    );
                    if test_only {
                        profile.verification_like.add(
                            1,
                            command_evidence(
                                command,
                                "test-scoped verification command concentrated on proof-oriented scope",
                            ),
                        );
                    }
                    if source_scope {
                        profile.implementation_like.add(
                            1,
                            command_evidence(
                                command,
                                "local verification against source scope also supported implementation follow-through",
                            ),
                        );
                    }
                }
            }
            CommandRole::Orchestration => {
                profile.orchestration_command_count += 1;
                profile.orchestration_like.add(
                    2,
                    command_evidence(
                        command,
                        "delegation-orchestration command strengthened orchestration-like evidence",
                    ),
                );
            }
            CommandRole::Neutral => {}
        }
    }

    if matches!(
        analysis.turn_context.execution_mode,
        TurnExecutionMode::Autonomous
    ) {
        profile.implementation_like.add(
            1,
            task_frame_evidence(
                &analysis.current.task_frame,
                "autonomous turn execution mode aligned with implementation-like work",
            ),
        );
    }

    if matches!(
        analysis.turn_context.execution_mode,
        TurnExecutionMode::VerificationHeavy
    ) {
        profile.verification_like.add(
            1,
            interval_verification_evidence(
                analysis,
                "verification-heavy turn execution mode aligned with proof-oriented work",
            ),
        );
    }

    if analysis.turn_context.activity_mix.directive_row_count
        > analysis.turn_context.activity_mix.tool_call_count
    {
        profile.orchestration_like.add(
            1,
            task_frame_evidence(
                &analysis.current.task_frame,
                "directive-heavy turn shape aligned with planning-orchestration work",
            ),
        );
    }

    if analysis.task_frame_delta.task_frame_transitioned {
        profile.exploration_like.add(
            1,
            task_frame_evidence(
                &analysis.current.task_frame,
                "task-frame transition signaled active scope exploration",
            ),
        );
        profile.implementation_like.add_counter(task_frame_evidence(
            &analysis.current.task_frame,
            "task-frame transition weakened claims of a settled implementation cadence",
        ));
    }

    if analysis.current.task_frame.working_set_paths.len() > 4 {
        profile.exploration_like.add(
            1,
            task_frame_evidence(
                &analysis.current.task_frame,
                "broad working set supported exploration-like behavior",
            ),
        );
    }

    if profile.source_write_command_count > 0 && !analysis.task_frame_delta.working_set_changed {
        profile.implementation_like.add(
            1,
            task_frame_evidence(
                &analysis.current.task_frame,
                "stable working set plus source edits supported concentrated implementation",
            ),
        );
    }

    if analysis.recovery.clean_verification_interval {
        profile.verification_like.add(
            2,
            interval_verification_evidence(
                analysis,
                "clean recent verification interval strengthened proof-oriented evidence",
            ),
        );
    }

    if analysis.recovery.recovered_from_thrash {
        profile.verification_like.add(
            1,
            interval_verification_evidence(
                analysis,
                "recent recovery after repeated verification loops supported proof-oriented work",
            ),
        );
    }

    if analysis.recovery.active_repeated_verification || analysis.recovery.active_repeated_failure {
        profile.verification_like.add(
            1,
            repeated_failure_evidence(
                analysis,
                "repeated failing verification kept the checkpoint in verification-driven diagnosis",
            ),
        );
        profile.exploration_like.add(
            1,
            repeated_failure_evidence(
                analysis,
                "repeated failing verification forced renewed inspection and diagnosis",
            ),
        );
    }

    if !matches!(
        analysis.delegation.topology,
        Some(DelegationTopology::SingleAgent) | None
    ) {
        profile.orchestration_like.add(
            1,
            delegation_evidence(
                &analysis.delegation,
                "delegation topology kept orchestration evidence in the visible parent prefix",
            ),
        );
    }

    if ENABLE_KICKOFF_PRIORS {
        // Packet R4-2 keeps kickoff priors disabled by default so behavior remains the primary
        // source of evidence until behavior-only regressions stabilize.
    }

    profile
}

fn aggregate_session_archetype(
    analysis: &CheckpointAnalysis,
    intent: &IntentEvidenceProfile,
) -> Vec<ArchetypeCandidate> {
    let failure_pressure = i32::from(analysis.recovery.active_repeated_verification)
        + i32::from(analysis.recovery.active_repeated_failure)
        + i32::from(!analysis.repetition.repeated_failure_loops.is_empty());
    let clean_verification_bonus = i32::from(analysis.recovery.clean_verification_interval) * 2;
    let docs_heavy =
        intent.docs_or_spec_write_command_count > 0 && intent.source_write_command_count == 0;
    let source_writes = intent.source_write_command_count as i32;
    let test_only_writes = i32::from(
        intent.test_or_golden_write_command_count > 0 && intent.source_write_command_count == 0,
    );

    let planning_score = (intent.exploration_like.raw_score + intent.orchestration_like.raw_score)
        + i32::from(docs_heavy) * 2
        + i32::from(analysis.task_frame_delta.task_frame_transitioned)
        + i32::from(intent.verification_command_count == 0)
        - source_writes.saturating_mul(2)
        - clean_verification_bonus / 2;

    let implementation_score = intent.implementation_like.raw_score
        + source_writes
        + i32::from(intent.verification_command_count > 0)
        + i32::from(matches!(
            analysis.turn_context.execution_mode,
            TurnExecutionMode::Autonomous
        ))
        - failure_pressure
        - i32::from(docs_heavy);
    let no_source_write_closeout_bonus = i32::from(
        intent.source_write_command_count == 0
            && (intent.verification_like.raw_score > 0
                || analysis.recovery.clean_verification_interval),
    ) * 2;

    let verification_closeout_score = intent.verification_like.raw_score
        + clean_verification_bonus
        + no_source_write_closeout_bonus
        + test_only_writes
        + i32::from(matches!(
            analysis.turn_context.execution_mode,
            TurnExecutionMode::VerificationHeavy
        ))
        - failure_pressure.saturating_mul(2)
        - source_writes;

    let troubleshooting_score = intent.verification_like.raw_score
        + intent.exploration_like.raw_score
        + failure_pressure.saturating_mul(2)
        + i32::from(!analysis.repetition.repeated_verification_loops.is_empty())
        - clean_verification_bonus
        - i32::from(intent.implementation_like.raw_score >= intent.verification_like.raw_score + 2);
    let troubleshooting_score =
        if failure_pressure == 0 && analysis.repetition.repeated_verification_loops.is_empty() {
            troubleshooting_score - 2
        } else {
            troubleshooting_score
        };

    vec![
        archetype_candidate(
            SessionArchetypeLabel::Planning,
            planning_score,
            combine_evidence(vec![
                intent.exploration_like.evidence.clone(),
                intent.orchestration_like.evidence.clone(),
            ]),
            vec![
                intent.implementation_like.evidence.clone(),
                intent.verification_like.evidence.clone(),
            ],
            vec![
                intent.implementation_like.counter_evidence.clone(),
                intent.verification_like.counter_evidence.clone(),
            ],
        ),
        archetype_candidate(
            SessionArchetypeLabel::AutonomousImplementation,
            implementation_score,
            intent.implementation_like.evidence.clone(),
            vec![
                intent.exploration_like.evidence.clone(),
                intent.verification_like.evidence.clone(),
                intent.verification_like.counter_evidence.clone(),
            ],
            vec![intent.implementation_like.counter_evidence.clone()],
        ),
        archetype_candidate(
            SessionArchetypeLabel::VerificationCloseout,
            verification_closeout_score,
            intent.verification_like.evidence.clone(),
            vec![
                intent.implementation_like.evidence.clone(),
                intent.exploration_like.evidence.clone(),
            ],
            vec![intent.verification_like.counter_evidence.clone()],
        ),
        archetype_candidate(
            SessionArchetypeLabel::Troubleshooting,
            troubleshooting_score,
            combine_evidence(vec![
                intent.verification_like.evidence.clone(),
                intent.exploration_like.evidence.clone(),
                repeated_failure_evidence(
                    analysis,
                    "failing verification and diagnosis remained active in the recent prefix",
                ),
            ]),
            vec![
                intent.implementation_like.evidence.clone(),
                if analysis.recovery.clean_verification_interval {
                    intent.verification_like.evidence.clone()
                } else {
                    Vec::new()
                },
            ],
            vec![
                intent.verification_like.counter_evidence.clone(),
                intent.exploration_like.counter_evidence.clone(),
            ],
        ),
    ]
}

fn archetype_candidate(
    label: SessionArchetypeLabel,
    score: i32,
    supporting_evidence: Vec<EvidenceRef>,
    competing_evidence: Vec<Vec<EvidenceRef>>,
    bucket_counter_evidence: Vec<Vec<EvidenceRef>>,
) -> ArchetypeCandidate {
    ArchetypeCandidate {
        label,
        score: score.max(0),
        supporting_evidence: dedupe_and_limit_evidence(supporting_evidence),
        counter_evidence: dedupe_and_limit_evidence(combine_evidence(
            competing_evidence
                .into_iter()
                .chain(bucket_counter_evidence)
                .collect(),
        )),
    }
}

fn archetype_confidence(
    analysis: &CheckpointAnalysis,
    intent: &IntentEvidenceProfile,
    winner: &ArchetypeCandidate,
    runner_up_score: i32,
) -> Confidence {
    let margin = winner.score - runner_up_score;
    let mixed_implementation_and_verification = intent.implementation_like.strength()
        >= EvidenceStrength::Moderate
        && intent.verification_like.strength() >= EvidenceStrength::Moderate
        && intent.source_write_command_count == 0
        && margin <= 2;
    let mixed_exploration_and_verification = intent.exploration_like.strength()
        >= EvidenceStrength::Moderate
        && intent.verification_like.strength() >= EvidenceStrength::Moderate
        && !analysis.recovery.clean_verification_interval
        && margin <= 2;
    let ambiguous =
        margin <= 1 || mixed_implementation_and_verification || mixed_exploration_and_verification;

    let delegated_opaque = matches!(
        (
            analysis.delegation.topology,
            analysis.delegation.child_work_visibility
        ),
        (
            Some(DelegationTopology::DelegatingParent),
            Some(ChildWorkVisibility::Opaque)
        ) | (
            Some(DelegationTopology::MixedOrAmbiguous),
            Some(ChildWorkVisibility::Opaque)
        )
    );

    if delegated_opaque {
        return Confidence::Low;
    }

    if winner.score >= 8 && margin >= 3 && !ambiguous {
        return Confidence::High;
    }

    if winner.score >= 4 && margin >= 1 && !ambiguous {
        return Confidence::Medium;
    }

    Confidence::Low
}

fn archetype_label_sort_key(label: SessionArchetypeLabel) -> u8 {
    match label {
        SessionArchetypeLabel::Troubleshooting => 0,
        SessionArchetypeLabel::Planning => 1,
        SessionArchetypeLabel::AutonomousImplementation => 2,
        SessionArchetypeLabel::VerificationCloseout => 3,
    }
}

fn classify_command_role(command: &CommandObservation) -> CommandRole {
    let family = command.family.as_str();
    let raw_command = command.raw_command.to_ascii_lowercase();
    let tool_name = command.tool_name.as_str();
    let tokens = normalized_command_tokens(&command.raw_command);

    if matches!(
        tool_name,
        "spawn_agent" | "wait_agent" | "close_agent" | "multi_agent_v1"
    ) {
        return CommandRole::Orchestration;
    }

    if matches!(family, "apply_patch" | "mkdir" | "mv" | "cp")
        || raw_command.contains("*** begin patch")
    {
        return CommandRole::Implementation;
    }

    if let Some(role) = classify_known_tool_family_role(command) {
        return role;
    }

    if matches!(
        family,
        "cat" | "sed" | "rg" | "ls" | "find" | "head" | "tail" | "jq"
    ) {
        return CommandRole::Exploration;
    }

    if matches!(
        family,
        "spawn_agent" | "wait_agent" | "close_agent" | "multi_agent_v1"
    ) || tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "spawn_agent" | "wait_agent" | "close_agent" | "multi_agent_v1"
        )
    }) {
        return CommandRole::Orchestration;
    }

    if requires_shallow_subcommand_parse(family) {
        return CommandRole::Neutral;
    }

    if command.verification_like || is_verification_command_family(family) {
        return CommandRole::Verification;
    }

    if command.read_like {
        return CommandRole::Exploration;
    }

    if command.write_like {
        return CommandRole::Implementation;
    }

    CommandRole::Neutral
}

fn classify_known_tool_family_role(command: &CommandObservation) -> Option<CommandRole> {
    let family = command.family.as_str();
    let tokens = normalized_command_tokens(&command.raw_command);
    let args = tokens_after_family(&tokens, family)?;

    match family {
        "cargo" => role_from_subcommand(next_positional_token(
            args,
            &[
                "-p",
                "--package",
                "--manifest-path",
                "--config",
                "-Z",
                "--target",
                "--target-dir",
                "--message-format",
                "--color",
                "--jobs",
            ],
        )?),
        "npm" | "pnpm" => npm_like_role(args),
        "git" => role_from_git_subcommand(next_positional_token(
            args,
            &[
                "-C",
                "-c",
                "--git-dir",
                "--work-tree",
                "--namespace",
                "--exec-path",
                "--config-env",
            ],
        )?),
        _ => None,
    }
}

fn requires_shallow_subcommand_parse(family: &str) -> bool {
    matches!(family, "cargo" | "npm" | "pnpm" | "git")
}

fn is_verification_command_family(family: &str) -> bool {
    matches!(
        family,
        "pytest"
            | "vitest"
            | "jest"
            | "ruff"
            | "clippy"
            | "lint"
            | "fmt"
            | "check"
            | "doctor"
            | "replay"
            | "test"
            | "build"
    )
}

fn normalized_command_tokens(command: &str) -> Vec<String> {
    command
        .split(['\n', ';', '|', '&'])
        .next()
        .unwrap_or(command)
        .split_whitespace()
        .map(|token| {
            token
                .trim_matches(|ch: char| matches!(ch, '(' | ')' | '"' | '\''))
                .to_ascii_lowercase()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn tokens_after_family<'a>(tokens: &'a [String], family: &str) -> Option<&'a [String]> {
    let family_index = tokens
        .iter()
        .position(|token| token == family && !token.contains('=') && !token.is_empty())?;
    Some(&tokens[family_index + 1..])
}

fn next_positional_token<'a>(
    tokens: &'a [String],
    options_with_values: &[&str],
) -> Option<&'a str> {
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index].as_str();
        if token == "--" {
            return tokens.get(index + 1).map(String::as_str);
        }
        if token.starts_with('-') {
            index += 1;
            if !token.contains('=') && options_with_values.contains(&token) {
                index += 1;
            }
            continue;
        }
        return Some(token);
    }
    None
}

fn npm_like_role(tokens: &[String]) -> Option<CommandRole> {
    let subcommand = next_positional_token(
        tokens,
        &[
            "-C",
            "--prefix",
            "--dir",
            "-w",
            "--workspace",
            "--filter",
            "-F",
        ],
    )?;
    if subcommand == "run" {
        let run_index = tokens.iter().position(|token| token == "run")?;
        return role_from_subcommand(next_positional_token(&tokens[run_index + 1..], &[])?);
    }
    role_from_subcommand(subcommand)
}

fn role_from_subcommand(subcommand: &str) -> Option<CommandRole> {
    if matches!(
        subcommand,
        "test" | "check" | "clippy" | "fmt" | "format" | "build" | "lint"
    ) {
        return Some(CommandRole::Verification);
    }

    if matches!(
        subcommand,
        "add" | "remove" | "rm" | "update" | "install" | "i" | "ci" | "up"
    ) {
        return Some(CommandRole::Implementation);
    }

    None
}

fn role_from_git_subcommand(subcommand: &str) -> Option<CommandRole> {
    if matches!(subcommand, "status" | "diff" | "show" | "log") {
        return Some(CommandRole::Exploration);
    }

    None
}

fn command_file_roles(command: &CommandObservation) -> BTreeSet<FileRole> {
    command
        .paths
        .iter()
        .map(|path| file_role(path))
        .collect::<BTreeSet<_>>()
}

fn file_role(path: &str) -> FileRole {
    let lower = path.to_ascii_lowercase();
    if lower.contains("/tests/")
        || lower.contains("/fixtures/")
        || lower.contains("golden")
        || lower.ends_with(".snap")
        || lower.ends_with(".golden")
    {
        return FileRole::TestOrGolden;
    }

    if lower.starts_with("docs/")
        || lower.contains("/docs/")
        || lower.contains("/specs/")
        || lower.ends_with(".md")
    {
        return FileRole::DocsOrSpec;
    }

    if lower.ends_with("cargo.toml")
        || lower.ends_with("cargo.lock")
        || lower.ends_with("package.json")
        || lower.ends_with("pnpm-lock.yaml")
        || lower.ends_with("package-lock.json")
        || lower.ends_with(".toml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.contains(".github/")
        || lower.contains("/scripts/")
    {
        return FileRole::ConfigOrBuild;
    }

    if lower.starts_with("target/")
        || lower.contains("/target/")
        || lower.starts_with("dist/")
        || lower.contains("/generated/")
        || lower.contains("/artifacts/")
    {
        return FileRole::GeneratedArtifact;
    }

    if lower.ends_with(".rs")
        || lower.ends_with(".ts")
        || lower.ends_with(".tsx")
        || lower.ends_with(".js")
        || lower.ends_with(".jsx")
        || lower.ends_with(".py")
        || lower.contains("/src/")
        || lower.contains("/crates/")
    {
        return FileRole::Source;
    }

    FileRole::Unknown
}

fn command_evidence(command: &CommandObservation, reason: &str) -> Vec<EvidenceRef> {
    command
        .evidence
        .iter()
        .map(|evidence| with_reason(evidence, reason))
        .collect()
}

fn task_frame_evidence(task_frame: &TaskFrame, reason: &str) -> Vec<EvidenceRef> {
    task_frame
        .supporting_evidence
        .iter()
        .take(2)
        .map(|evidence| with_reason(evidence, reason))
        .collect()
}

fn interval_verification_evidence(analysis: &CheckpointAnalysis, reason: &str) -> Vec<EvidenceRef> {
    analysis
        .interval
        .command_observations
        .iter()
        .filter(|command| classify_command_role(command) == CommandRole::Verification)
        .flat_map(|command| command.evidence.iter())
        .take(2)
        .map(|evidence| with_reason(evidence, reason))
        .collect()
}

fn repeated_failure_evidence(analysis: &CheckpointAnalysis, reason: &str) -> Vec<EvidenceRef> {
    analysis
        .repetition
        .repeated_failure_loops
        .iter()
        .flat_map(|failure_loop| failure_loop.evidence.iter())
        .chain(
            analysis
                .repetition
                .repeated_verification_loops
                .iter()
                .flat_map(|loop_evidence| loop_evidence.evidence.iter()),
        )
        .take(2)
        .map(|evidence| with_reason(evidence, reason))
        .collect()
}

fn delegation_evidence(delegation: &DelegationContext, reason: &str) -> Vec<EvidenceRef> {
    delegation
        .supporting_evidence
        .iter()
        .take(2)
        .map(|evidence| with_reason(evidence, reason))
        .collect()
}

fn with_reason(evidence: &EvidenceRef, reason: &str) -> EvidenceRef {
    EvidenceRef {
        row: evidence.row.clone(),
        reason: reason.to_string(),
    }
}

fn combine_evidence(groups: Vec<Vec<EvidenceRef>>) -> Vec<EvidenceRef> {
    groups.into_iter().flatten().collect()
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
    deduped.truncate(MAX_ARCHETYPE_EVIDENCE_ITEMS);
    deduped
}

fn delegation_supporting_archetype_evidence(analysis: &CheckpointAnalysis) -> Vec<EvidenceRef> {
    if matches!(
        analysis.delegation.topology,
        Some(DelegationTopology::SingleAgent) | None
    ) {
        return Vec::new();
    }

    delegation_evidence(
        &analysis.delegation,
        "visible delegation topology informed the checkpoint-local archetype decision",
    )
}

fn delegation_counter_archetype_evidence(analysis: &CheckpointAnalysis) -> Vec<EvidenceRef> {
    let mut counter = analysis
        .delegation
        .counter_evidence
        .iter()
        .map(|evidence| {
            with_reason(
                evidence,
                "child-opaque delegation limited direct confidence in parent-visible archetype semantics",
            )
        })
        .collect::<Vec<_>>();

    if matches!(
        (
            analysis.delegation.topology,
            analysis.delegation.child_work_visibility
        ),
        (
            Some(DelegationTopology::DelegatingParent),
            Some(ChildWorkVisibility::Opaque)
        ) | (
            Some(DelegationTopology::MixedOrAmbiguous),
            Some(ChildWorkVisibility::Opaque)
        )
    ) {
        counter.extend(delegation_evidence(
            &analysis.delegation,
            "delegating-parent plus child-opaque visibility capped checkpoint-local archetype certainty",
        ));
    }

    counter
}

fn drift_state_for_score(
    score: &DriftScore,
    state_hint: DriftStateHint,
    previous_drift_scores: Option<&[DriftScore]>,
) -> DriftState {
    if score.flagged {
        return DriftState::Active;
    }

    if !matches!(state_hint, DriftStateHint::HistoricalContext) {
        return DriftState::Cleared;
    }

    match previous_state_for_class(previous_drift_scores, score.class) {
        Some(DriftState::Active) => DriftState::Recovered,
        _ => DriftState::HistoricalOnly,
    }
}

fn previous_state_for_class(
    previous_drift_scores: Option<&[DriftScore]>,
    class: DriftClass,
) -> Option<DriftState> {
    previous_drift_scores?
        .iter()
        .find(|score| score.class == class)
        .map(|score| score.state)
}

pub fn build_session_checkpoint(
    session: &BundleSession,
    ordinal: usize,
    task_frame: &TaskFrame,
    drift_scores: Vec<DriftScore>,
) -> Checkpoint {
    let analyses = checkpoint_analyses(session);
    let analysis = analyses
        .get(ordinal.checked_sub(1).unwrap_or(usize::MAX))
        .or_else(|| analyses.last())
        .unwrap_or_else(|| {
            panic!(
                "session {} must contain at least one checkpoint window",
                session.session_id
            )
        });
    build_session_checkpoint_from_analysis_with_ordinal(analysis, ordinal, task_frame, drift_scores)
}

fn checkpoint_diagnostics_from_analysis(
    analysis: &CheckpointAnalysis,
    task_frame: &TaskFrame,
    drift_scores: &[DriftScore],
) -> CheckpointDiagnostics {
    CheckpointDiagnostics {
        task_frame_transitioned: analysis.task_frame_delta.task_frame_transitioned,
        working_set_changed: analysis.task_frame_delta.working_set_changed,
        interval_command_count: analysis.interval.command_observations.len(),
        interval_verification_command_count: analysis.recovery.interval_verification_command_count,
        evidence_item_count: evidence_item_count(task_frame, drift_scores),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TurnSliceIdentity {
    turn_id: String,
    start: (Utf8PathBuf, usize, usize),
}

struct CurrentTurnSlice<'a> {
    rows: &'a [CompactionRow],
    turn_id: Option<&'a str>,
    turn_ordinal: usize,
}

fn turn_context(
    session: &BundleSession,
    current: &CheckpointSlice,
    interval: &IntervalSlice,
    prompts_observed_in_session: usize,
    checkpoints_in_turn: &mut BTreeMap<TurnSliceIdentity, usize>,
) -> TurnContext {
    let turn_slice = current_turn_slice(session, current, interval);
    let checkpoints_in_turn = match turn_slice_identity(&turn_slice) {
        Some(identity) => {
            let count = checkpoints_in_turn.entry(identity).or_default();
            *count += 1;
            *count
        }
        None => 1,
    };
    let activity_mix = turn_activity_mix(turn_slice.rows);
    let execution_mode = turn_execution_mode(&turn_slice, checkpoints_in_turn, &activity_mix);

    TurnContext {
        turn_id: turn_slice.turn_id.map(str::to_owned),
        turn_ordinal: turn_slice.turn_ordinal,
        rows_since_turn_start: turn_slice.rows.len(),
        seconds_since_turn_start: seconds_since_turn_start(turn_slice.rows),
        checkpoints_in_turn,
        prompts_observed_in_session,
        execution_mode,
        activity_mix,
    }
}

fn turn_activity_mix(rows: &[CompactionRow]) -> TurnActivityMix {
    let command_observations = collect_command_observations(rows);
    let command_roles = command_observations
        .iter()
        .map(classify_command_role)
        .collect::<Vec<_>>();
    TurnActivityMix {
        directive_row_count: focusable_directive_rows(rows).count(),
        assistant_message_count: rows
            .iter()
            .filter(|row| row.kind == CompactionKind::AssistantMessage)
            .count(),
        tool_call_count: rows
            .iter()
            .filter(|row| row.kind == CompactionKind::ToolCall)
            .count(),
        read_like_command_count: command_roles
            .iter()
            .filter(|role| matches!(role, CommandRole::Exploration))
            .count(),
        write_like_command_count: command_roles
            .iter()
            .filter(|role| matches!(role, CommandRole::Implementation))
            .count(),
        verification_like_command_count: command_roles
            .iter()
            .filter(|role| matches!(role, CommandRole::Verification))
            .count(),
        tool_output_count: rows
            .iter()
            .filter(|row| row.kind == CompactionKind::ToolOutput)
            .count(),
    }
}

fn turn_execution_mode(
    turn_slice: &CurrentTurnSlice<'_>,
    checkpoints_in_turn: usize,
    activity_mix: &TurnActivityMix,
) -> TurnExecutionMode {
    let directive_and_assistant_rows =
        activity_mix.directive_row_count + activity_mix.assistant_message_count;
    let conversational = activity_mix.tool_call_count == 0
        || (checkpoints_in_turn <= 1
            && directive_and_assistant_rows > activity_mix.tool_call_count);
    let autonomous =
        activity_mix.tool_call_count > 0 && checkpoints_in_turn > 1 && turn_slice.turn_id.is_some();
    let verification_heavy = verification_like_is_strict_plurality(activity_mix);

    match [
        conversational.then_some(TurnExecutionMode::Conversational),
        autonomous.then_some(TurnExecutionMode::Autonomous),
        verification_heavy.then_some(TurnExecutionMode::VerificationHeavy),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .as_slice()
    {
        [mode] => *mode,
        _ => TurnExecutionMode::Mixed,
    }
}

fn verification_like_is_strict_plurality(mix: &TurnActivityMix) -> bool {
    let non_verification_like_command_count = mix
        .tool_call_count
        .saturating_sub(mix.verification_like_command_count);
    mix.verification_like_command_count > 0
        && mix.verification_like_command_count > mix.read_like_command_count
        && mix.verification_like_command_count > mix.write_like_command_count
        && mix.verification_like_command_count > non_verification_like_command_count
}

fn current_turn_slice<'a>(
    session: &'a BundleSession,
    current: &'a CheckpointSlice,
    interval: &'a IntervalSlice,
) -> CurrentTurnSlice<'a> {
    let rows = preferred_rows(&current.window.archival_rows, &current.window.compact_rows);
    let Some(turn_id) = rows.iter().rev().find_map(|row| row.turn_id.as_deref()) else {
        return CurrentTurnSlice {
            rows: preferred_rows(&interval.archival_rows, &interval.compact_rows),
            turn_id: None,
            turn_ordinal: 0,
        };
    };

    let start_index = rows
        .iter()
        .rposition(|row| matches!(row.turn_id.as_deref(), Some(id) if id != turn_id))
        .map_or(0, |index| index + 1);

    CurrentTurnSlice {
        rows: &rows[start_index..],
        turn_id: Some(turn_id),
        turn_ordinal: turn_ordinal(session, turn_id),
    }
}

fn turn_slice_identity(turn_slice: &CurrentTurnSlice<'_>) -> Option<TurnSliceIdentity> {
    Some(TurnSliceIdentity {
        turn_id: turn_slice.turn_id?.to_string(),
        start: row_key(turn_slice.rows.first()?),
    })
}

fn turn_ordinal(session: &BundleSession, current_turn_id: &str) -> usize {
    let mut seen = BTreeSet::new();
    let mut ordinal = 0;

    for row in preferred_rows(&session.archival_rows, &session.compact_rows) {
        let Some(turn_id) = row.turn_id.as_deref() else {
            continue;
        };
        if seen.insert(turn_id) {
            ordinal += 1;
        }
        if turn_id == current_turn_id {
            return ordinal;
        }
    }

    0
}

fn seconds_since_turn_start(rows: &[CompactionRow]) -> Option<i64> {
    let start = rows.iter().find_map(|row| row.timestamp)?;
    let end = rows.last().and_then(|row| row.timestamp)?;
    Some((end - start).whole_seconds().max(0))
}

fn prompts_observed_in_session(session: &BundleSession) -> usize {
    session
        .compact_rows
        .iter()
        .filter(|row| row.kind == CompactionKind::UserMessage)
        .filter(|row| !is_synthetic_user_message(row))
        .filter(|row| {
            matches!(
                row.user_message_role.unwrap_or(UserMessageRole::Unknown),
                UserMessageRole::Prompt
            )
        })
        .count()
}

fn preferred_rows<'a>(
    archival_rows: &'a [CompactionRow],
    compact_rows: &'a [CompactionRow],
) -> &'a [CompactionRow] {
    if archival_rows.is_empty() {
        compact_rows
    } else {
        archival_rows
    }
}

fn interval_slice(previous: Option<&CheckpointSlice>, current: &CheckpointSlice) -> IntervalSlice {
    let archival_start = previous.map_or(0, |slice| slice.window.archival_rows.len());
    let interval_start = previous.map_or(0, |slice| slice.window.compact_rows.len());
    let archival_rows = current.window.archival_rows[archival_start..].to_vec();
    let compact_rows = current.window.compact_rows[interval_start..].to_vec();
    let command_observations = collect_command_observations(&compact_rows);
    let command_attempts = build_command_attempts(&compact_rows, &command_observations);
    let verification_attempts = build_verification_attempts(&command_attempts, &compact_rows);

    IntervalSlice {
        archival_rows,
        compact_rows,
        command_observations,
        command_attempts,
        verification_attempts,
    }
}

fn repetition_slice(current: &CheckpointSlice) -> RepetitionSlice {
    let mut repeated_commands = BTreeMap::<String, Vec<EvidenceRef>>::new();
    let archival_commands = collect_command_observations(&current.window.archival_rows);
    for command in archival_commands
        .iter()
        .filter(|command| command.verification_like)
    {
        repeated_commands
            .entry(command.raw_command.clone())
            .or_default()
            .extend(command.evidence.clone());
    }

    let mut repeated_failures = BTreeMap::<String, Vec<EvidenceRef>>::new();
    for row in current
        .window
        .archival_rows
        .iter()
        .filter(|row| is_failure_row(row))
    {
        repeated_failures
            .entry(row.text_hash_hex.clone())
            .or_default()
            .push(EvidenceRef {
                row: RowRef::from_row(row),
                reason: "repeated failure evidence".to_string(),
            });
    }

    RepetitionSlice {
        compact_rows: current.window.compact_rows.clone(),
        repeated_verification_loops: repeated_commands
            .into_iter()
            .filter_map(|(raw_command, evidence)| {
                (evidence.len() >= 3).then_some(RepeatedCommandLoop {
                    raw_command,
                    evidence,
                })
            })
            .collect(),
        repeated_failure_loops: repeated_failures
            .into_iter()
            .filter_map(|(text_hash_hex, evidence)| {
                (evidence.len() >= 2).then_some(RepeatedFailureLoop {
                    text_hash_hex,
                    evidence,
                })
            })
            .collect(),
    }
}

fn task_frame_delta(
    previous: Option<&CheckpointSlice>,
    current: &CheckpointSlice,
) -> TaskFrameDelta {
    let Some(previous) = previous else {
        return TaskFrameDelta::default();
    };

    TaskFrameDelta {
        task_frame_transitioned: task_frame_identity(&current.task_frame)
            != task_frame_identity(&previous.task_frame),
        working_set_changed: working_set_identity(&current.task_frame)
            != working_set_identity(&previous.task_frame),
    }
}

fn recovery_state(
    current: &CheckpointSlice,
    interval: &IntervalSlice,
    repetition: &RepetitionSlice,
) -> RecoveryState {
    let interval_verification_command_count = interval
        .command_observations
        .iter()
        .filter(|command| command.verification_like)
        .count();
    let preserves_out_of_scope = preserves_out_of_scope_thrash(current, interval);
    let failure_touches_interval =
        failure_loops_touch_interval(&repetition.repeated_failure_loops, interval);
    let clean_verification_interval = interval_verification_command_count > 0
        && !preserves_out_of_scope
        && !failure_touches_interval;

    RecoveryState {
        interval_verification_command_count,
        clean_verification_interval,
        recovered_from_thrash: clean_verification_interval
            && (!repetition.repeated_verification_loops.is_empty()
                || !repetition.repeated_failure_loops.is_empty()),
        active_repeated_verification: interval_verification_command_count > 0
            && !repetition.repeated_verification_loops.is_empty()
            && preserves_out_of_scope,
        active_repeated_failure: failure_touches_interval,
    }
}

fn preserves_out_of_scope_thrash(current: &CheckpointSlice, interval: &IntervalSlice) -> bool {
    let mut expected = current.task_frame.truth_artifacts.clone();
    expected.extend(
        current
            .context
            .working_set_paths
            .iter()
            .filter(|path| {
                path.source != "observed_command"
                    || path
                        .evidence
                        .iter()
                        .any(|evidence| !interval_contains_evidence(interval, evidence))
            })
            .map(|path| path.path.clone()),
    );
    expected.sort();
    expected.dedup();

    interval.command_observations.iter().any(|command| {
        if command.paths.is_empty() || (!command.write_like && !command.verification_like) {
            return false;
        }

        !command.paths.iter().all(|path| {
            expected.iter().any(|expected_path| {
                path == expected_path
                    || path.starts_with(expected_path)
                    || expected_path.starts_with(path)
            })
        })
    })
}

fn failure_loops_touch_interval(loops: &[RepeatedFailureLoop], interval: &IntervalSlice) -> bool {
    loops.iter().any(|failure_loop| {
        failure_loop
            .evidence
            .iter()
            .any(|evidence| interval_contains_evidence(interval, evidence))
    })
}

fn interval_contains_evidence(interval: &IntervalSlice, evidence: &EvidenceRef) -> bool {
    let interval_row_keys = interval
        .archival_rows
        .iter()
        .map(row_key)
        .collect::<BTreeSet<_>>();
    interval_row_keys.contains(&row_ref_key(&evidence.row))
}

pub fn checkpoint_windows(session: &BundleSession) -> Vec<BundleSession> {
    let Some(last_index) = session.compact_rows.len().checked_sub(1) else {
        if session.archival_rows.is_empty() {
            return Vec::new();
        }
        return vec![session.clone()];
    };

    let mut phase_end_indices = checkpoint_end_indices(&session.compact_rows);
    if phase_end_indices.last().copied() != Some(last_index) {
        phase_end_indices.push(last_index);
    }

    let mut windows = Vec::with_capacity(phase_end_indices.len());
    for end_index in phase_end_indices {
        let compact_rows = session.compact_rows[..=end_index].to_vec();
        let end_row = compact_rows
            .last()
            .expect("checkpoint window must contain at least one compact row");
        let end_key = row_key(end_row);
        let archival_rows = session
            .archival_rows
            .iter()
            .take_while(|row| row_key(row) <= end_key)
            .cloned()
            .collect::<Vec<_>>();
        windows.push(BundleSession {
            session_id: session.session_id.clone(),
            archival_rows,
            compact_rows,
        });
    }

    windows
}

fn checkpoint_boundary(session: &BundleSession) -> CheckpointBoundary {
    let start_row = session
        .archival_rows
        .first()
        .or_else(|| session.compact_rows.first())
        .expect("session must contain rows");
    let end_row = session
        .archival_rows
        .last()
        .or_else(|| session.compact_rows.last())
        .expect("session must contain rows");
    CheckpointBoundary {
        start: RowRef::from_row(start_row),
        end: RowRef::from_row(end_row),
    }
}

fn expected_next_step(task_frame: &TaskFrame) -> String {
    task_frame
        .verification_commands
        .first()
        .cloned()
        .unwrap_or_else(|| "continue on the current task frame".to_string())
}

fn task_frame_identity(task_frame: &TaskFrame) -> String {
    serde_json::to_string(&(
        &task_frame.objective,
        &task_frame.truth_artifacts,
        &task_frame.working_set_paths,
        &task_frame.tools,
        &task_frame.command_families,
        &task_frame.verification_commands,
    ))
    .expect("task frame identity should serialize")
}

fn working_set_identity(task_frame: &TaskFrame) -> BTreeSet<&str> {
    task_frame
        .working_set_paths
        .iter()
        .map(String::as_str)
        .collect()
}

fn evidence_item_count(task_frame: &TaskFrame, drift_scores: &[DriftScore]) -> usize {
    let mut seen = BTreeSet::new();
    for evidence in task_frame
        .supporting_evidence
        .iter()
        .chain(task_frame.counter_evidence.iter())
        .chain(drift_scores.iter().flat_map(|score| score.evidence.iter()))
    {
        seen.insert((
            evidence.row.source_file.clone(),
            evidence.row.event_index,
            evidence.row.row_ordinal,
            evidence.reason.clone(),
        ));
    }
    seen.len()
}

fn checkpoint_end_indices(rows: &[CompactionRow]) -> Vec<usize> {
    if rows.is_empty() {
        return Vec::new();
    }

    let mut phase_ends = Vec::new();
    let mut saw_activity = false;
    let mut saw_objective = false;
    for (index, row) in rows.iter().enumerate() {
        if objective_row(row) {
            saw_objective = true;
        }
        if index > 0 && row_starts_new_phase(row) && saw_activity && saw_objective {
            phase_ends.push(index - 1);
            saw_activity = false;
        }
        if row_is_activity(row) {
            saw_activity = true;
        }
    }

    let mut normalized = Vec::new();
    let mut last_end = None;
    for end_index in phase_ends
        .into_iter()
        .chain(std::iter::once(rows.len() - 1))
    {
        let start_index = last_end.map_or(0, |end| end + 1);
        if end_index >= start_index + MAX_ROWS_PER_CHECKPOINT {
            let mut chunk_end = start_index + MAX_ROWS_PER_CHECKPOINT - 1;
            while chunk_end < end_index {
                normalized.push(chunk_end);
                chunk_end += MAX_ROWS_PER_CHECKPOINT;
            }
        }
        if normalized.last().copied() != Some(end_index) {
            normalized.push(end_index);
        }
        last_end = Some(end_index);
    }

    normalized
}

fn row_starts_new_phase(row: &CompactionRow) -> bool {
    matches!(
        row.kind,
        CompactionKind::UserMessage
            | CompactionKind::AssistantMessage
            | CompactionKind::DeveloperMessage
            | CompactionKind::SystemMessage
    ) && row_text_is_focusable(row)
}

fn objective_row(row: &CompactionRow) -> bool {
    matches!(
        row.kind,
        CompactionKind::UserMessage | CompactionKind::DeveloperMessage
    ) && row_text_is_focusable(row)
}

fn row_is_activity(row: &CompactionRow) -> bool {
    matches!(
        row.kind,
        CompactionKind::ToolCall
            | CompactionKind::ToolOutput
            | CompactionKind::Error
            | CompactionKind::Reasoning
            | CompactionKind::Unknown
    )
}

fn row_text_is_focusable(row: &CompactionRow) -> bool {
    row.text.len() <= 2_000
        && !row.text.trim().is_empty()
        && !row.text.contains("AGENTS.md instructions")
        && !row.text.contains("<skill>")
        && !row.text.contains("Available skills")
        && row.text != "[encrypted_reasoning]"
}

fn is_synthetic_user_message(row: &CompactionRow) -> bool {
    row.text.contains("AGENTS.md instructions")
        || row.text.contains("<skill>")
        || row.text.contains("Available skills")
}

fn row_key(row: &CompactionRow) -> (Utf8PathBuf, usize, usize) {
    (row.source_file.clone(), row.event_index, row.row_ordinal)
}

fn row_ref_key(row: &RowRef) -> (Utf8PathBuf, usize, usize) {
    (row.source_file.clone(), row.event_index, row.row_ordinal)
}

fn classify_outcome_evidence(row: &CompactionRow) -> OutcomeEvidenceKind {
    if row.text.trim().is_empty() {
        return OutcomeEvidenceKind::Neutral;
    }

    match row.kind {
        CompactionKind::Error => OutcomeEvidenceKind::Failure,
        CompactionKind::ToolOutput if tool_output_is_unambiguous_failure(&row.text) => {
            OutcomeEvidenceKind::Failure
        }
        _ => OutcomeEvidenceKind::Neutral,
    }
}

fn tool_output_is_unambiguous_failure(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("error:"))
        || trimmed
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("Exit code: "))
            .and_then(|code| code.trim().parse::<i32>().ok())
            .is_some_and(|code| code != 0)
}

fn is_failure_row(row: &CompactionRow) -> bool {
    matches!(classify_outcome_evidence(row), OutcomeEvidenceKind::Failure)
}

#[cfg(test)]
mod tests {
    use crate::context::CommandObservation;
    use agent_session_compactor::{CompactionKind, CompactionRow, SourceKind};
    use camino::Utf8PathBuf;

    use crate::input::BundleSession;

    use super::{
        checkpoint_analyses, classify_command_role, tool_output_is_unambiguous_failure,
        CommandRole, EvidenceRef,
    };

    #[test]
    fn tool_output_unambiguous_failure_subset_stays_tiny_and_deterministic() {
        for text in [
            "error: failed to compile analyzer",
            "Exit code: 1",
            "Exit code: 101\nWall time: 1.2 seconds\nOutput:\n",
            "Exit code: 128",
        ] {
            assert!(tool_output_is_unambiguous_failure(text), "{text}");
        }

        for text in [
            "",
            "Exit code: 0",
            "Plan updated",
            "function_call_output: wrote analyzer patch",
        ] {
            assert!(!tool_output_is_unambiguous_failure(text), "{text}");
        }
    }

    #[test]
    fn checkpoints_wire_delegation_from_checkpoint_analysis_without_public_api() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![
                row(
                    0,
                    CompactionKind::UserMessage,
                    "/goal Inspect delegation wiring only.",
                ),
                tool_call(1, "spawn_agent", "{\"agent_type\":\"worker\"}"),
            ],
            compact_rows: vec![
                row(
                    0,
                    CompactionKind::UserMessage,
                    "/goal Inspect delegation wiring only.",
                ),
                tool_call(
                    2,
                    "functions.shell_command",
                    "{\"command\":\"printf 'child rollout ' && sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };

        let analyses = checkpoint_analyses(&session);
        assert_eq!(analyses.len(), 1);

        let delegation = &analyses[0].delegation;
        assert_eq!(
            delegation.topology,
            Some(crate::inference::DelegationTopology::DelegatingParent)
        );
        assert_eq!(
            delegation.child_work_visibility,
            Some(crate::inference::ChildWorkVisibility::Partial)
        );
        assert_eq!(
            delegation.confidence,
            Some(crate::checkpoint::Confidence::Medium)
        );
        assert_eq!(delegation.markers, vec!["spawn_agent".to_string()]);
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence
                .reason
                .contains("child rollout surface links child/subagent work")));
        assert!(delegation.counter_evidence.is_empty());
    }

    #[test]
    fn checkpoints_degrade_generic_delegation_markers_to_ambiguous_and_opaque() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![
                row(
                    0,
                    CompactionKind::UserMessage,
                    "/goal Inspect delegation wiring only.",
                ),
                tool_call(1, "multi_agent_v1", "{\"mode\":\"delegated\"}"),
                row(
                    2,
                    CompactionKind::DeveloperMessage,
                    "Child session id 019ea222-2222-7222-8222-222222222222 remains in separate rollout /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-30-00-019ea222-2222-7222-8222-222222222222.jsonl",
                ),
            ],
            compact_rows: vec![
                row(
                    0,
                    CompactionKind::UserMessage,
                    "/goal Inspect delegation wiring only.",
                ),
                tool_call(1, "multi_agent_v1", "{\"mode\":\"delegated\"}"),
                row(
                    2,
                    CompactionKind::DeveloperMessage,
                    "Child session id 019ea222-2222-7222-8222-222222222222 remains in separate rollout /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-30-00-019ea222-2222-7222-8222-222222222222.jsonl",
                ),
            ],
        };

        let analyses = checkpoint_analyses(&session);
        assert_eq!(analyses.len(), 2);

        let delegation = &analyses[1].delegation;
        assert_eq!(
            delegation.topology,
            Some(crate::inference::DelegationTopology::MixedOrAmbiguous)
        );
        assert_eq!(
            delegation.child_work_visibility,
            Some(crate::inference::ChildWorkVisibility::Opaque)
        );
        assert_eq!(
            delegation.confidence,
            Some(crate::checkpoint::Confidence::Low)
        );
        assert_eq!(delegation.markers, vec!["multi_agent_v1".to_string()]);
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence
                .reason
                .contains("delegation directive surface references separate child rollout")));
    }

    #[test]
    fn checkpoints_populate_internal_interval_attempts_from_checkpoint_analysis() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![
                row(
                    0,
                    CompactionKind::UserMessage,
                    "/goal Verify attempt wiring.",
                ),
                tool_call(
                    1,
                    "functions.shell_command",
                    "cargo test -p agent-drift-analyzer checkpoints",
                ),
                row(
                    2,
                    CompactionKind::ToolOutput,
                    "Exit code: 0\nOutput:\nrunning 1 test",
                ),
            ],
            compact_rows: vec![
                row(
                    0,
                    CompactionKind::UserMessage,
                    "/goal Verify attempt wiring.",
                ),
                tool_call(
                    1,
                    "functions.shell_command",
                    "cargo test -p agent-drift-analyzer checkpoints",
                ),
                row(
                    2,
                    CompactionKind::ToolOutput,
                    "Exit code: 0\nOutput:\nrunning 1 test",
                ),
            ],
        };

        let analyses = checkpoint_analyses(&session);
        assert_eq!(analyses.len(), 1);

        let interval = &analyses[0].interval;
        assert_eq!(interval.command_attempts.len(), 1);
        assert_eq!(
            interval.command_attempts[0].role,
            super::attempt::CommandAttemptRole::Test
        );
        assert_eq!(
            interval.command_attempts[0].outcome,
            super::attempt::AttemptOutcome::Clean
        );
        assert_eq!(interval.verification_attempts.len(), 1);
        assert_eq!(
            interval.verification_attempts[0].exercise_state,
            super::attempt::ExerciseState::TargetExercised
        );
        assert_eq!(
            interval.verification_attempts[0].target_scope.tests,
            vec!["checkpoints".to_string()]
        );
    }

    #[test]
    fn classify_command_role_keeps_reads_of_test_and_checkpoint_paths_as_exploration() {
        for command in [
            command_observation(
                "sed",
                "sed -n '1,120p' crates/agent-drift-analyzer/tests/checkpoints.rs",
            ),
            command_observation(
                "rg",
                "rg -n 'session_archetype' crates/agent-drift-analyzer/src/checkpoint/mod.rs",
            ),
            command_observation("sed", "sed -n '1,120p' docs/REPLAY.md"),
            command_observation(
                "rg",
                "rg -n 'spawn_agent' docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md",
            ),
        ] {
            assert_eq!(classify_command_role(&command), CommandRole::Exploration);
        }
    }

    #[test]
    fn classify_command_role_uses_cargo_subcommands_instead_of_family() {
        assert_eq!(
            classify_command_role(&command_observation(
                "cargo",
                "cargo add serde --features derive"
            )),
            CommandRole::Implementation
        );
        assert_eq!(
            classify_command_role(&command_observation(
                "cargo",
                "cargo --locked test -p agent-drift-analyzer checkpoints -- --nocapture"
            )),
            CommandRole::Verification
        );
        assert_eq!(
            classify_command_role(&command_observation("cargo", "cargo fmt --all -- --check")),
            CommandRole::Verification
        );
    }

    #[test]
    fn classify_command_role_uses_npm_and_pnpm_subcommands_instead_of_family() {
        assert_eq!(
            classify_command_role(&command_observation("npm", "npm install typescript")),
            CommandRole::Implementation
        );
        assert_eq!(
            classify_command_role(&command_observation("npm", "npm --prefix web run build")),
            CommandRole::Verification
        );
        assert_eq!(
            classify_command_role(&command_observation("pnpm", "pnpm add zod")),
            CommandRole::Implementation
        );
        assert_eq!(
            classify_command_role(&command_observation("pnpm", "pnpm run fmt")),
            CommandRole::Verification
        );
    }

    #[test]
    fn classify_command_role_limits_git_family_to_inspect_verbs() {
        assert_eq!(
            classify_command_role(&command_observation(
                "git",
                "git -C crates/shell diff --stat"
            )),
            CommandRole::Exploration
        );
        assert_eq!(
            classify_command_role(&command_observation("git", "git status --short")),
            CommandRole::Exploration
        );
        assert_eq!(
            classify_command_role(&command_observation("git", "git show HEAD:docs/REPLAY.md")),
            CommandRole::Exploration
        );
        assert_eq!(
            classify_command_role(&command_observation(
                "git",
                "git add crates/shell/src/lib.rs"
            )),
            CommandRole::Neutral
        );
    }

    #[test]
    fn classify_command_role_keeps_unknown_shallow_parse_families_neutral() {
        for command in [
            command_observation_with_flags(
                "cargo",
                "cargo metadata --format-version 1",
                false,
                true,
                true,
            ),
            command_observation_with_flags(
                "npm",
                "npm exec playwright --version",
                false,
                false,
                true,
            ),
            command_observation_with_flags(
                "pnpm",
                "pnpm dlx tsx scripts/report.ts",
                false,
                false,
                true,
            ),
            command_observation_with_flags(
                "git",
                "git checkout feature/r4-2-review-fix",
                true,
                false,
                false,
            ),
        ] {
            assert_eq!(classify_command_role(&command), CommandRole::Neutral);
        }
    }

    fn row(event_index: usize, kind: CompactionKind, text: &str) -> CompactionRow {
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session-alpha".to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index,
            line_number: event_index + 1,
            row_ordinal: event_index,
            timestamp: None,
            kind,
            user_message_role: None,
            dedupe_identity: None,
            text: text.to_string(),
            canonical_text: text.to_string(),
            text_hash_hex: format!("{kind:?}-{event_index}"),
        }
    }

    fn tool_call(event_index: usize, tool_name: &str, text: &str) -> CompactionRow {
        let mut row = row(event_index, CompactionKind::ToolCall, text);
        row.dedupe_identity = Some(format!(
            "{{\"call_id\":\"call-{event_index}\",\"name\":\"{tool_name}\",\"type\":\"function_call\"}}"
        ));
        row
    }

    fn command_observation(family: &str, raw_command: &str) -> CommandObservation {
        command_observation_with_flags(
            family,
            raw_command,
            matches!(family, "sed" | "rg"),
            false,
            false,
        )
    }

    fn command_observation_with_flags(
        family: &str,
        raw_command: &str,
        read_like: bool,
        write_like: bool,
        verification_like: bool,
    ) -> CommandObservation {
        CommandObservation {
            family: family.to_string(),
            raw_command: raw_command.to_string(),
            tool_name: "functions.shell_command".to_string(),
            paths: Vec::new(),
            read_like,
            write_like,
            verification_like,
            evidence: vec![EvidenceRef {
                row: agent_session_compactor::RowRef {
                    source_file: Utf8PathBuf::from("/tmp/rollout.jsonl"),
                    event_index: 1,
                    row_ordinal: 0,
                },
                reason: format!("command family: {family}"),
            }],
        }
    }
}
