mod attempt;
mod diagnostics;
mod export;
mod progress;
mod schema;

use std::collections::{BTreeMap, BTreeSet};

use crate::input::BundleSession;
use crate::{
    context::assemble_context, context::collect_command_observations, context::evidence_from_row,
    context::extract_verification_commands, context::focusable_directive_rows,
    context::CommandObservation, context::ContextPack, context::ObjectiveSummary,
    inference::infer_delegation_context, inference::infer_task_frame,
    inference::ChildWorkVisibility, inference::DelegationContext, inference::DelegationTopology,
    scoring::DriftStateHint, scoring::ScoredDrift,
};
use agent_session_compactor::{CompactionKind, CompactionRow, RowRef, UserMessageRole};
use attempt::{
    build_command_attempts, build_verification_attempts, CommandAttempt, VerificationAttempt,
};
use camino::Utf8PathBuf;
use progress::build_session_progress;

pub use export::{
    export_checkpoints, summarize_checkpoint_diagnostics, CheckpointDiagnosticStats,
    ConfidenceDistribution, ExportError, ExportResult,
};
pub use schema::{
    Checkpoint, CheckpointBoundary, CheckpointDiagnostics, Confidence, DriftClass, DriftScore,
    DriftState, EvidenceRef, ObjectiveClass, ObjectiveConstraint, ObjectiveConstraintKind,
    ObjectiveEvidenceSpan, ObjectiveIntent, ObjectiveRole, ObjectiveSectionKind,
    ObjectiveSourceKind, ObjectiveTarget, ObjectiveTargetKind, ObjectiveUnknown, ProgressDimension,
    ProgressSignal, ProgressSignalCode, ProgressStatus, RequestedDeliverable,
    RequestedDeliverableKind, SessionArchetype, SessionArchetypeLabel, SessionProgress,
    SignalPolarity, SignalStrength, StructuredObjective, SuccessCondition, TaskFrame,
    TurnActivityMix, TurnContext, TurnExecutionMode,
};

const MAX_ROWS_PER_CHECKPOINT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointAnalysis {
    pub session_id: String,
    pub ordinal: usize,
    pub current: CheckpointSlice,
    pub previous: Option<CheckpointSlice>,
    pub sanctioned_replan: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectiveSource {
    LiteralGoalCommand,
    ExplicitUserRequest,
    ThreadGoalText,
    AssistantRestatedGoal,
    NonBoilerplateUnknown,
    NonBoilerplateDirective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoilerplateClass {
    PermissionBlock,
    SkillsBlock,
    AgentInstructionBlock,
    MemoryOrProfileBlock,
    ToolingCapabilityBlock,
    SafetyOrPolicyBlock,
    GenericScaffold,
}

#[derive(Debug, Clone)]
struct ObjectiveCandidate<'a> {
    row: &'a CompactionRow,
    text: String,
    source: ObjectiveSource,
    boilerplate_class: Option<BoilerplateClass>,
    priority: u8,
    role_priority: u8,
    pivot_priority: u8,
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
        let mut context = assemble_context(&window);
        let narrowed_objective = narrowed_objective_summary(&window.compact_rows);
        if let (Some(anchor_text), Some(narrowed)) = (
            grounded_structured_goal_anchor_text(&window.compact_rows, &context.objective),
            narrowed_objective.as_ref(),
        ) {
            if should_prefer_grounded_goal_anchor(&anchor_text, &narrowed.text) {
                context.objective.text = anchor_text;
            } else if context.objective.structured.is_some() {
                context.objective = context.objective.with_compatibility_display_from(narrowed);
            } else {
                context.objective = narrowed.clone();
            }
        } else if let Some(objective) = narrowed_objective {
            if context.objective.structured.is_some() {
                context.objective = context
                    .objective
                    .with_compatibility_display_from(&objective);
            } else {
                context.objective = objective;
            }
        }
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
        let sanctioned_replan = checkpoint_has_sanctioned_replan(&interval.compact_rows);
        let repetition = repetition_slice(&current);
        let task_frame_delta = task_frame_delta(previous.as_ref(), &current);
        let recovery = recovery_state(&current, &interval, &repetition);

        analyses.push(CheckpointAnalysis {
            session_id: current.window.session_id.clone(),
            ordinal: index + 1,
            current: current.clone(),
            previous: previous.clone(),
            sanctioned_replan,
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

pub(crate) fn session_kickoff_structured_goal_anchor(
    analyses: &[CheckpointAnalysis],
) -> Option<(usize, StructuredObjective)> {
    analyses.iter().find_map(|analysis| {
        let structured = analysis.current.context.objective.structured.as_ref()?;
        structured_matches_kickoff_anchor_bar(structured)
            .then(|| (analysis.ordinal, structured.clone()))
    })
}

/// The anchor is captured from whichever checkpoint first establishes it, so checkpoints
/// ordinally before that point must not be scored against a goal the analyzer had not yet
/// recognized — otherwise a still-undiscovered anchor gets applied retroactively.
pub(crate) fn kickoff_anchor_for_ordinal(
    anchor: Option<&(usize, StructuredObjective)>,
    ordinal: usize,
) -> Option<&StructuredObjective> {
    anchor
        .and_then(|(anchor_ordinal, structured)| (ordinal >= *anchor_ordinal).then_some(structured))
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

fn checkpoint_has_sanctioned_replan(rows: &[CompactionRow]) -> bool {
    rows.iter()
        .any(objective_candidate_is_explicit_replan_pivot)
}

fn structured_matches_kickoff_anchor_bar(structured: &StructuredObjective) -> bool {
    structured_matches_confident_task_statement(structured, Confidence::High)
}

fn structured_matches_confident_task_statement(
    structured: &StructuredObjective,
    minimum_confidence: Confidence,
) -> bool {
    structured.objective_class == ObjectiveClass::TaskStatement
        && structured.confidence >= minimum_confidence
        && structured.unknowns.is_empty()
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

pub(crate) fn build_scoring_session_progress(analysis: &CheckpointAnalysis) -> SessionProgress {
    let session_archetype = build_session_archetype(analysis);
    build_session_progress(analysis, &session_archetype)
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
    let session_progress = build_session_progress(analysis, &session_archetype);
    Checkpoint {
        schema_version: "v0.7".to_string(),
        session_id: analysis.session_id.clone(),
        checkpoint_id: format!("{}:{ordinal:04}", analysis.session_id),
        ordinal,
        boundary,
        turn_context: Some(analysis.turn_context.clone()),
        diagnostics,
        task_frame: task_frame.clone(),
        structured_objective: analysis.current.context.objective.structured.clone(),
        session_archetype: Some(session_archetype),
        session_progress: Some(session_progress),
        flagged: drift_scores.iter().any(|score| score.flagged),
        drift_scores,
        expected_next_step,
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
        let closeout_artifact_scope = docs_only
            && command
                .paths
                .iter()
                .any(|path| is_closeout_artifact_path(path));

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
                    if closeout_artifact_scope
                        && (objective_mentions_closeout_phase(
                            &analysis.current.context.objective.text,
                        ) || closeout_artifact_preserves_closeout_context(analysis))
                    {
                        profile.verification_like.add(
                            4,
                            command_evidence(
                                command,
                                "closeout artifact edit preserved proof-oriented closeout context even without a fresh proof command",
                            ),
                        );
                        profile.exploration_like.add_counter(command_evidence(
                            command,
                            "closeout artifact edit followed a prior clean proof path instead of open-ended planning",
                        ));
                        profile.orchestration_like.add_counter(command_evidence(
                            command,
                            "closeout artifact edit followed a prior clean proof path instead of planning-orchestration setup",
                        ));
                    } else {
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
                    }
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
    let closeout_ready =
        objective_mentions_closeout_phase(&analysis.current.context.objective.text)
            || prior_source_write_history(analysis)
            || closeout_artifact_preserves_closeout_context(analysis);
    let explicit_no_code_review =
        objective_is_explicit_no_code_review(&analysis.current.context.objective.text)
            && intent.source_write_command_count == 0
            && intent.test_or_golden_write_command_count == 0;
    let verification_diagnosis_without_closeout = !closeout_ready
        && objective_mentions_repair_work(&analysis.current.context.objective.text)
        && !explicit_no_code_review
        && intent.source_write_command_count == 0
        && intent.verification_command_count > 0
        && intent.exploration_like.raw_score > 0;

    let planning_score = (intent.exploration_like.raw_score + intent.orchestration_like.raw_score)
        + i32::from(docs_heavy) * 2
        + i32::from(explicit_no_code_review) * 3
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
                || analysis.recovery.clean_verification_interval)
            && closeout_ready,
    ) * 2;

    let verification_closeout_score = intent.verification_like.raw_score
        + clean_verification_bonus
        + no_source_write_closeout_bonus
        + test_only_writes
        + i32::from(matches!(
            analysis.turn_context.execution_mode,
            TurnExecutionMode::VerificationHeavy
        ))
        - i32::from(!closeout_ready)
        - i32::from(explicit_no_code_review) * 2
        - i32::from(verification_diagnosis_without_closeout) * 6
        - failure_pressure.saturating_mul(2)
        - source_writes;

    let troubleshooting_score = intent.verification_like.raw_score
        + intent.exploration_like.raw_score
        + failure_pressure.saturating_mul(2)
        + i32::from(!analysis.repetition.repeated_verification_loops.is_empty())
        + i32::from(verification_diagnosis_without_closeout) * 4
        - i32::from(explicit_no_code_review) * 3
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
        "npm" | "pnpm" | "yarn" | "bun" | "npx" => npm_like_role(args),
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
    matches!(
        family,
        "cargo" | "npm" | "pnpm" | "yarn" | "bun" | "npx" | "git"
    )
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
    if matches!(subcommand, "run" | "exec" | "dlx") {
        let subcommand_index = tokens.iter().position(|token| token == subcommand)?;
        return role_from_subcommand(next_positional_token(&tokens[subcommand_index + 1..], &[])?);
    }
    role_from_subcommand(subcommand)
}

fn role_from_subcommand(subcommand: &str) -> Option<CommandRole> {
    if matches!(
        subcommand,
        "test"
            | "check"
            | "clippy"
            | "fmt"
            | "format"
            | "build"
            | "lint"
            | "typecheck"
            | "vitest"
            | "jest"
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
    let mut phase_start_index = 0usize;
    let mut saw_activity = false;
    let mut saw_objective = false;
    for (index, row) in rows.iter().enumerate() {
        let objective_candidate = objective_candidate(row);
        let objective_gate_row =
            checkpoint_phase_boundary_objective_row(rows, index, row, objective_candidate.as_ref());
        let phase_boundary_objective =
            row_starts_new_phase(&rows[phase_start_index..index], row) || objective_gate_row;
        if index > 0 && phase_boundary_objective && saw_activity && saw_objective {
            phase_ends.push(index - 1);
            saw_activity = false;
            phase_start_index = index;
        }
        if matches!(
            objective_candidate
                .as_ref()
                .map(|candidate| candidate.source),
            Some(ObjectiveSource::ExplicitUserRequest)
        ) && !row_text_is_focusable(row)
        {
            saw_objective = false;
        } else if objective_gate_row {
            saw_objective = true;
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

fn row_starts_new_phase(prior_rows: &[CompactionRow], row: &CompactionRow) -> bool {
    preserves_user_requested_boilerplate_target(row)
        || match row.kind {
            CompactionKind::AssistantMessage => {
                assistant_phase_boundary_text(row).is_some()
                    && assistant_phase_boundary_allowed(prior_rows, row)
            }
            CompactionKind::UserMessage
            | CompactionKind::DeveloperMessage
            | CompactionKind::SystemMessage => row_text_is_focusable(row),
            _ => false,
        }
}

fn assistant_phase_boundary_allowed(prior_rows: &[CompactionRow], row: &CompactionRow) -> bool {
    if assistant_truth_grounding_phase_boundary_text(row) {
        return true;
    }

    let commands = collect_command_observations(prior_rows);
    if commands
        .iter()
        .any(|command| command.write_like || command.verification_like)
    {
        return true;
    }

    prior_rows.iter().any(|row| {
        matches!(row.kind, CompactionKind::Error)
            || matches!(row.kind, CompactionKind::ToolOutput)
                && tool_output_is_unambiguous_failure(&row.text)
    }) || commands.iter().filter(|command| command.read_like).count() >= 4
        && assistant_read_heavy_phase_boundary_text(row)
}

fn assistant_read_heavy_phase_boundary_text(row: &CompactionRow) -> bool {
    let Some(text) = assistant_phase_boundary_text(row) else {
        return false;
    };

    let lower = text.to_ascii_lowercase();
    lower.contains("next i’m ")
        || lower.contains("next i'm ")
        || lower.contains("next i am ")
        || lower.contains("next i will ")
        || lower.contains("remaining work is")
}

fn assistant_truth_grounding_phase_boundary_text(row: &CompactionRow) -> bool {
    let Some(text) = assistant_phase_boundary_text(row) else {
        return false;
    };

    let lower = text.to_ascii_lowercase();
    lower.contains("re-ground on the spec")
        || lower.contains("grounding is back in place")
        || lower.contains("spec is grounded")
        || lower.contains("moving to the next checkpoint")
        || lower.contains("historical context only")
}

fn assistant_phase_boundary_text(row: &CompactionRow) -> Option<String> {
    assistant_restated_goal_text(row).or_else(|| {
        if !matches!(row.kind, CompactionKind::AssistantMessage) || !row_text_is_focusable(row) {
            return None;
        }

        let text = row.text.trim();
        let lower = text.to_ascii_lowercase();
        (lower.contains("next i’m ")
            || lower.contains("next i'm ")
            || lower.contains("next i am ")
            || lower.contains("next i will ")
            || lower.starts_with("i’ve narrowed the scope")
            || lower.starts_with("i've narrowed the scope")
            || lower.contains("remaining work is")
            || lower.contains("re-ground on the spec")
            || lower.contains("grounding is back in place")
            || lower.contains("spec is grounded")
            || lower.contains("moving to the next checkpoint")
            || lower.contains("verification pass")
            || lower.contains("stood out during verification")
            || lower.contains("historical context")
            || lower.contains("completed successfully"))
        .then(|| text.to_string())
    })
}

fn checkpoint_phase_boundary_objective_row(
    rows: &[CompactionRow],
    index: usize,
    row: &CompactionRow,
    objective_candidate: Option<&ObjectiveCandidate<'_>>,
) -> bool {
    let Some(candidate) = objective_candidate else {
        return false;
    };

    match candidate.source {
        ObjectiveSource::LiteralGoalCommand => true,
        ObjectiveSource::ThreadGoalText => thread_goal_boundary_objective_row(rows, index, row),
        ObjectiveSource::ExplicitUserRequest
        | ObjectiveSource::AssistantRestatedGoal
        | ObjectiveSource::NonBoilerplateUnknown => row_text_is_focusable(row),
        ObjectiveSource::NonBoilerplateDirective => {
            candidate.boilerplate_class.is_none() && row_text_is_focusable(row)
        }
    }
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

fn grounded_structured_goal_anchor_text(
    rows: &[CompactionRow],
    objective: &ObjectiveSummary,
) -> Option<String> {
    let structured = objective.structured.as_ref()?;
    if structured.objective_class != ObjectiveClass::TaskStatement
        || structured
            .unknowns
            .iter()
            .any(|unknown| unknown.field_name == "primary_goal")
    {
        return None;
    }

    let anchor_span = structured
        .evidence_spans
        .iter()
        .filter(|span| span.role == ObjectiveRole::Goal)
        .filter(|span| {
            matches!(
                span.source_kind,
                ObjectiveSourceKind::ThreadGoal | ObjectiveSourceKind::UserPrompt
            )
        })
        .max_by_key(|span| grounded_goal_anchor_score(span))?;

    if grounded_goal_anchor_score(anchor_span) < 700 {
        return None;
    }

    let anchor_text = rows
        .iter()
        .find(|row| {
            row.event_index == anchor_span.row.event_index
                && row.row_ordinal == anchor_span.row.row_ordinal
                && row.source_file == anchor_span.row.source_file
        })
        .and_then(|row| paragraph_containing_excerpt(&row.text, &anchor_span.excerpt))
        .map(|paragraph| normalized_objective_text(&paragraph))
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| anchor_span.excerpt.trim().to_string());

    (!anchor_text.is_empty()).then_some(anchor_text)
}

fn should_prefer_grounded_goal_anchor(anchor_text: &str, narrowed_text: &str) -> bool {
    anchor_text_looks_grounded_goal(anchor_text)
        && narrowed_objective_looks_subordinate(narrowed_text)
}

fn anchor_text_looks_grounded_goal(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    lowered.starts_with("/goal ")
        || [
            "please validate",
            "confirm/deny",
            "determine whether",
            "evaluate if",
            "need you to",
            "want you to",
            "findings-first",
            "review ",
            "validate ",
        ]
        .iter()
        .any(|needle| lowered.contains(needle))
}

fn narrowed_objective_looks_subordinate(text: &str) -> bool {
    let lowered = text.trim().to_ascii_lowercase();
    if lowered.is_empty() {
        return false;
    }

    if [
        "review is clean",
        "implementation-complete",
        "future work only",
        "after that, report the changed files",
        "report the changed files",
        "return with changed files",
        "optional reviewer nits",
        "non-blocking follow-ups",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
    {
        return true;
    }

    ["add ", "normalize ", "tighten ", "wire ", "update ", "fix "]
        .iter()
        .any(|needle| lowered.starts_with(needle))
}

fn grounded_goal_anchor_score(span: &ObjectiveEvidenceSpan) -> i32 {
    let mut score = match span.source_kind {
        ObjectiveSourceKind::ThreadGoal => 900,
        ObjectiveSourceKind::UserPrompt => 600,
        ObjectiveSourceKind::AssistantContext => 100,
        ObjectiveSourceKind::SystemInstruction
        | ObjectiveSourceKind::ToolOutput
        | ObjectiveSourceKind::UnknownSource => -500,
    };

    score += match span.confidence {
        Confidence::High => 250,
        Confidence::Medium => 125,
        Confidence::Low => 0,
    };

    score += match span.section_kind {
        ObjectiveSectionKind::Scope | ObjectiveSectionKind::Mission => 200,
        ObjectiveSectionKind::Checklist | ObjectiveSectionKind::Verification => -400,
        ObjectiveSectionKind::Boilerplate | ObjectiveSectionKind::ToolingInstructions => -500,
        ObjectiveSectionKind::Constraints
        | ObjectiveSectionKind::Deliverables
        | ObjectiveSectionKind::Context
        | ObjectiveSectionKind::UnknownSection => 0,
    };

    let lowered = span.excerpt.to_ascii_lowercase();
    if lowered.starts_with("/goal ") {
        score += 700;
    }
    if [
        "please validate",
        "confirm/deny",
        "determine whether",
        "evaluate if",
        "review whether",
        "validate whether",
        "are we ready",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
    {
        score += 700;
    }
    if ["need you to", "want you to", "please ", "findings-first"]
        .iter()
        .any(|needle| lowered.contains(needle))
    {
        score += 250;
    }
    if [
        "implementation-complete",
        "review-clean",
        "review is clean",
        "orchestration-only",
        "optional reviewer nits",
        "not taken",
        "future work",
        "deferred",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
    {
        score -= 500;
    }
    if lowered.contains("cargo ")
        || lowered.contains(" --")
        || lowered.contains("\"$pwd\"")
        || lowered.starts_with(".agents/")
    {
        score -= 700;
    }
    if ["add ", "normalize ", "tighten ", "wire ", "update ", "fix "]
        .iter()
        .any(|needle| lowered.starts_with(needle))
    {
        score -= 350;
    }
    if lowered.ends_with("is clean.") || lowered.ends_with("review-clean.") {
        score -= 350;
    }

    score + span.clause_index.unwrap_or_default() as i32
}

fn paragraph_containing_excerpt(text: &str, excerpt: &str) -> Option<String> {
    let excerpt = excerpt.trim();
    if excerpt.is_empty() {
        return None;
    }

    text.split("\n\n")
        .map(str::trim)
        .find(|paragraph| paragraph.contains(excerpt))
        .map(|paragraph| paragraph.to_string())
}

fn narrowed_objective_summary(rows: &[CompactionRow]) -> Option<ObjectiveSummary> {
    let candidate = rows
        .iter()
        .filter_map(objective_candidate)
        .max_by(|left, right| objective_sort_key(left).cmp(&objective_sort_key(right)))?;
    let objective_text = normalized_objective_text(&candidate.text);

    let reason = match candidate.source {
        ObjectiveSource::LiteralGoalCommand => "literal /goal objective row",
        ObjectiveSource::ExplicitUserRequest => "explicit user objective row",
        ObjectiveSource::ThreadGoalText => "thread goal objective row",
        ObjectiveSource::AssistantRestatedGoal => "assistant restated goal row",
        ObjectiveSource::NonBoilerplateUnknown => "non-boilerplate unknown objective row",
        ObjectiveSource::NonBoilerplateDirective => match candidate.boilerplate_class {
            Some(BoilerplateClass::PermissionBlock) => "permission objective row",
            Some(BoilerplateClass::SkillsBlock) => "skills objective row",
            Some(BoilerplateClass::AgentInstructionBlock) => "instruction objective row",
            Some(BoilerplateClass::MemoryOrProfileBlock) => "memory/profile objective row",
            Some(BoilerplateClass::ToolingCapabilityBlock) => "tooling capability objective row",
            Some(BoilerplateClass::SafetyOrPolicyBlock) => "safety/policy objective row",
            Some(BoilerplateClass::GenericScaffold) => "scaffold objective row",
            None => "non-boilerplate directive objective row",
        },
    };

    Some(ObjectiveSummary::compatibility(
        objective_text,
        extract_verification_commands(&candidate.text),
        vec![evidence_from_row(candidate.row, reason)],
    ))
}

fn objective_sort_key(
    candidate: &ObjectiveCandidate<'_>,
) -> (u8, u8, u8, u8, usize, std::cmp::Reverse<usize>) {
    let normalized = normalized_objective_text(&candidate.text);
    (
        candidate.pivot_priority,
        candidate.priority,
        objective_text_specificity_priority(&candidate.text, &normalized),
        candidate.role_priority,
        candidate.row.event_index,
        std::cmp::Reverse(normalized.len()),
    )
}

fn objective_text_specificity_priority(original: &str, normalized: &str) -> u8 {
    if normalized.trim_start().starts_with("/goal") {
        3
    } else if extract_labeled_concrete_objective_text(original).is_some() {
        2
    } else if objective_line_looks_concrete(normalized) {
        1
    } else {
        0
    }
}

fn normalized_objective_text(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some(concrete) = extract_labeled_concrete_objective_text(trimmed) {
        return concrete;
    }

    if let Some(goal_line) = extract_embedded_goal_line(trimmed) {
        return goal_line;
    }

    if let Some(clause) = extract_inline_concrete_objective_clause(trimmed) {
        return clause;
    }

    if !text.contains('\n') {
        return text.to_string();
    }

    if contains_embedded_goal_line(trimmed) {
        let first_paragraph = trimmed.split("\n\n").next().unwrap_or(trimmed).trim();
        return if first_paragraph.is_empty() {
            trimmed.to_string()
        } else {
            first_paragraph.to_string()
        };
    }

    if let Some(imperative_line) = extract_concrete_imperative_objective_line(trimmed) {
        return imperative_line;
    }

    if !text.contains("\n\n") {
        return trimmed.to_string();
    }

    let first_paragraph = trimmed.split("\n\n").next().unwrap_or(trimmed).trim();
    if first_paragraph.is_empty() {
        trimmed.to_string()
    } else {
        first_paragraph.to_string()
    }
}

fn extract_inline_concrete_objective_clause(text: &str) -> Option<String> {
    const MARKERS: &[&str] = &[
        "now i need you to ",
        "now i want you to ",
        "i need you to ",
        "i want you to ",
        "need you to ",
        "want you to ",
    ];

    let lower = text.to_ascii_lowercase();
    for marker in MARKERS {
        let Some(start) = lower.find(marker) else {
            continue;
        };

        let clause = &text[start + marker.len()..];
        if let Some(candidate) = cleaned_inline_objective_clause(clause) {
            return Some(candidate);
        }
    }

    None
}

fn cleaned_inline_objective_clause(clause: &str) -> Option<String> {
    let mut trimmed = clause
        .trim()
        .trim_start_matches([':', '-', '—', '–'])
        .trim_start();

    loop {
        let next = trimmed
            .strip_prefix("actually ")
            .or_else(|| trimmed.strip_prefix("just "))
            .or_else(|| trimmed.strip_prefix("please "))
            .or_else(|| trimmed.strip_prefix("kindly "))
            .or_else(|| trimmed.strip_prefix("then "))
            .or_else(|| trimmed.strip_prefix("first "));
        let Some(next) = next else {
            break;
        };
        trimmed = next.trim_start();
    }

    let first_segment = trimmed.split("\n\n").next().unwrap_or(trimmed).trim();
    if let Some(line) = first_segment
        .lines()
        .filter_map(objective_candidate_line_text)
        .find(|line| objective_line_looks_concrete(line))
    {
        return Some(line);
    }

    let candidate = cleaned_objective_candidate_text(first_segment)?;
    objective_line_looks_concrete(&candidate).then_some(candidate)
}

fn extract_embedded_goal_line(text: &str) -> Option<String> {
    let goal_line = text
        .lines()
        .filter_map(objective_candidate_line_text)
        .find(|line| line.starts_with("/goal"))?;

    embedded_goal_line_overrides_first_paragraph(text, &goal_line).then_some(goal_line)
}

fn contains_embedded_goal_line(text: &str) -> bool {
    text.lines()
        .filter_map(objective_candidate_line_text)
        .any(|line| line.starts_with("/goal"))
}

fn embedded_goal_line_overrides_first_paragraph(text: &str, goal_line: &str) -> bool {
    let first_paragraph = text.trim().split("\n\n").next().unwrap_or(text).trim();
    if first_paragraph == goal_line {
        return true;
    }

    let mut saw_non_goal_line = false;
    for line in first_paragraph
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if line == goal_line {
            continue;
        }

        saw_non_goal_line = true;
        let lower = line.to_ascii_lowercase();
        if boilerplate_class(line).is_none() && !objective_line_is_metadata(&lower) {
            return false;
        }
    }

    saw_non_goal_line
}

fn extract_labeled_concrete_objective_text(text: &str) -> Option<String> {
    const LABELS: &[&str] = &[
        "concrete task ask:",
        "concrete workspace action request:",
        "concrete workspace ask:",
        "concrete ask:",
        "true concrete ask:",
        "workspace action request:",
        "task ask:",
        "concrete request:",
    ];

    let lines = text.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        let Some(label) = LABELS.iter().find(|label| lower.starts_with(**label)) else {
            continue;
        };

        let inline = trimmed[label.len()..].trim();
        if let Some(candidate) = cleaned_objective_candidate_text(inline) {
            return Some(candidate);
        }

        for next in &lines[index + 1..] {
            let next = next.trim();
            if next.is_empty() {
                break;
            }

            if let Some(candidate) = cleaned_objective_candidate_text(next) {
                return Some(candidate);
            }
        }
    }

    None
}

fn extract_concrete_imperative_objective_line(text: &str) -> Option<String> {
    text.lines()
        .filter_map(objective_candidate_line_text)
        .find(|line| objective_line_looks_concrete(line))
}

fn objective_candidate_line_text(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    cleaned_objective_candidate_text(trimmed)
}

fn cleaned_objective_candidate_text(text: &str) -> Option<String> {
    let stripped = strip_markdown_list_prefix(text.trim());
    (!stripped.is_empty()).then(|| stripped.to_string())
}

fn strip_markdown_list_prefix(text: &str) -> &str {
    let trimmed = text.trim_start();
    for prefix in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest.trim_start();
        }
    }

    let digit_count = trimmed.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digit_count > 0 {
        let rest = &trimmed[digit_count..];
        if let Some(rest) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
            return rest.trim_start();
        }
    }

    trimmed
}

fn objective_line_looks_concrete(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    if objective_line_is_metadata(&lower) || line.len() > 280 {
        return false;
    }

    [
        "/goal ",
        "implement ",
        "add ",
        "fix ",
        "update ",
        "debug ",
        "investigate ",
        "analyze ",
        "inspect ",
        "compare ",
        "explain ",
        "determine ",
        "review ",
        "use ",
        "plan ",
        "land ",
        "tighten ",
        "condense ",
        "preserve ",
        "refine ",
        "run ",
        "rerun ",
        "re-run ",
        "perform ",
        "only ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn objective_line_is_metadata(lower: &str) -> bool {
    [
        "# ",
        "read first:",
        "return with:",
        "primary files:",
        "project guidance:",
        "context from prior attempt:",
        "execution rules:",
        "gitnexus requirements:",
        "bundle scope only:",
        "stop before:",
        "verify:",
        "files:",
        "acceptance:",
        "status:",
        "problem:",
        "required change:",
        "manual smoke check",
        "promotion gate",
        "after that,",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
        || lower.starts_with("<")
        || lower == "---"
}

fn objective_mentions_closeout_phase(objective: &str) -> bool {
    let lower = objective.to_ascii_lowercase();
    lower.contains("verify the existing patch")
        || lower.contains("gather proof only")
        || lower.contains("residual proof")
        || lower.contains("closeout proof")
        || lower.contains("closeout handoff")
        || lower.contains("complete closeout")
        || lower.contains("re-check the closeout")
        || lower.contains("handoff only")
        || lower.contains("record the closeout handoff")
        || lower.contains("doc-only follow-up")
}

fn objective_mentions_repair_work(objective: &str) -> bool {
    let lower = objective.to_ascii_lowercase();
    [
        "fix",
        "debug",
        "troubleshoot",
        "root cause",
        "reproduce",
        "repair",
        "patch",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn objective_is_explicit_no_code_review(objective: &str) -> bool {
    let lower = objective.to_ascii_lowercase();
    let review_skill = lower.contains("code-review-and-quality");
    (lower.contains("review") || lower.contains("findings-first") || review_skill)
        && (lower.contains("without making code changes")
            || lower.contains("do not make code changes")
            || lower.contains("no code changes")
            || review_skill)
}

fn thread_goal_boundary_objective_row(
    rows: &[CompactionRow],
    index: usize,
    row: &CompactionRow,
) -> bool {
    thread_goal_objective_text(row).is_some()
        && (index == 0 || rows[index - 1].turn_id != row.turn_id)
}

fn objective_candidate(row: &CompactionRow) -> Option<ObjectiveCandidate<'_>> {
    if row.text.trim().is_empty() {
        return None;
    }

    if let Some(text) = thread_goal_objective_text(row) {
        return Some(ObjectiveCandidate {
            row,
            text,
            source: ObjectiveSource::ThreadGoalText,
            boilerplate_class: None,
            priority: 4,
            role_priority: 0,
            pivot_priority: 0,
        });
    }

    if row_is_pure_boilerplate(row) {
        return None;
    }

    if matches!(row.kind, CompactionKind::UserMessage) && row.text.trim_start().starts_with("/goal")
    {
        return Some(ObjectiveCandidate {
            row,
            text: row.text.clone(),
            source: ObjectiveSource::LiteralGoalCommand,
            boilerplate_class: boilerplate_class(&row.text),
            priority: 6,
            role_priority: objective_candidate_role_priority(row),
            pivot_priority: objective_candidate_pivot_priority(row),
        });
    }

    if matches!(row.kind, CompactionKind::UserMessage) {
        return Some(ObjectiveCandidate {
            row,
            text: row.text.clone(),
            source: ObjectiveSource::ExplicitUserRequest,
            boilerplate_class: boilerplate_class(&row.text),
            priority: 5,
            role_priority: objective_candidate_role_priority(row),
            pivot_priority: objective_candidate_pivot_priority(row),
        });
    }

    if let Some(text) = assistant_restated_goal_text(row) {
        return Some(ObjectiveCandidate {
            row,
            text,
            source: ObjectiveSource::AssistantRestatedGoal,
            boilerplate_class: boilerplate_class(&row.text),
            priority: 3,
            role_priority: 0,
            pivot_priority: 0,
        });
    }

    if let Some(text) = non_boilerplate_unknown_objective_text(row) {
        return Some(ObjectiveCandidate {
            row,
            text,
            source: ObjectiveSource::NonBoilerplateUnknown,
            boilerplate_class: None,
            priority: 2,
            role_priority: 0,
            pivot_priority: 0,
        });
    }

    if matches!(
        row.kind,
        CompactionKind::DeveloperMessage | CompactionKind::SystemMessage
    ) {
        return Some(ObjectiveCandidate {
            row,
            text: row.text.clone(),
            source: ObjectiveSource::NonBoilerplateDirective,
            boilerplate_class: boilerplate_class(&row.text),
            priority: 1,
            role_priority: 0,
            pivot_priority: 0,
        });
    }

    None
}

fn objective_candidate_role_priority(row: &CompactionRow) -> u8 {
    match row.user_message_role.unwrap_or(UserMessageRole::Unknown) {
        UserMessageRole::Prompt => 2,
        UserMessageRole::Unknown => 1,
        UserMessageRole::Steer => 0,
    }
}

fn objective_candidate_pivot_priority(row: &CompactionRow) -> u8 {
    objective_candidate_is_explicit_replan_pivot(row).into()
}

fn objective_candidate_is_explicit_replan_pivot(row: &CompactionRow) -> bool {
    if !matches!(row.kind, CompactionKind::UserMessage)
        || row.user_message_role != Some(UserMessageRole::Steer)
    {
        return false;
    }

    // "instead"/"instead of" were dropped from this needle list: they fire on routine tool/approach
    // substitutions ("use rg instead of grep") with no objective-level pivot, which would wrongly
    // suppress a real semantic_goal_drift claim. The remaining phrases are unambiguously about the
    // objective/scope itself.
    let normalized = normalize_objective_candidate_text(&row.text);
    [
        "replan",
        "pivot",
        "new objective",
        "change objective",
        "change the objective",
        "change scope",
        "change the scope",
        "different objective",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn normalize_objective_candidate_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn is_closeout_artifact_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("handoff")
        || lower.contains("summary")
        || lower.contains("fixture")
        || lower.starts_with("docs/")
        || lower.ends_with(".md")
}

fn closeout_artifact_preserves_closeout_context(analysis: &CheckpointAnalysis) -> bool {
    interval_has_closeout_artifact_write(analysis)
        && (analysis.recovery.clean_verification_interval
            || prior_clean_verification_history(analysis))
}

fn interval_has_closeout_artifact_write(analysis: &CheckpointAnalysis) -> bool {
    analysis
        .interval
        .command_observations
        .iter()
        .any(|command| {
            command.write_like
                && command
                    .paths
                    .iter()
                    .any(|path| is_closeout_artifact_path(path))
                && command_file_roles(command).iter().all(|role| {
                    matches!(
                        role,
                        FileRole::DocsOrSpec | FileRole::TestOrGolden | FileRole::Unknown
                    )
                })
        })
}

fn prior_clean_verification_history(analysis: &CheckpointAnalysis) -> bool {
    analysis.previous.as_ref().is_some_and(|previous| {
        let commands = collect_command_observations(&previous.window.compact_rows);
        commands
            .iter()
            .any(|command| classify_command_role(command) == CommandRole::Verification)
            && !previous.window.compact_rows.iter().any(is_failure_row)
    })
}

fn prior_source_write_history(analysis: &CheckpointAnalysis) -> bool {
    analysis.previous.as_ref().is_some_and(|previous| {
        collect_command_observations(&previous.window.compact_rows)
            .iter()
            .any(|command| {
                command.write_like && command_file_roles(command).contains(&FileRole::Source)
            })
    })
}

fn assistant_restated_goal_text(row: &CompactionRow) -> Option<String> {
    if !matches!(row.kind, CompactionKind::AssistantMessage) || !row_text_is_focusable(row) {
        return None;
    }

    let text = row.text.trim();
    if text.is_empty() || serde_json::from_str::<serde_json::Value>(text).is_ok() {
        return None;
    }

    let lower = text.to_ascii_lowercase();
    let starts_like_restatement = [
        "i will ",
        "i'll ",
        "i can ",
        "i am going to ",
        "i'm going to ",
        "let me ",
        "next i'll ",
        "next, i'll ",
    ]
    .iter()
    .any(|needle| lower.starts_with(needle));
    let mentions_objective_action = [
        "debug",
        "fix",
        "implement",
        "update",
        "review",
        "analyze",
        "investigate",
        "tighten",
        "add",
        "remove",
        "rerun",
        "verify",
    ]
    .iter()
    .any(|needle| lower.contains(needle));

    (starts_like_restatement && mentions_objective_action).then(|| text.to_string())
}

fn non_boilerplate_unknown_objective_text(row: &CompactionRow) -> Option<String> {
    if !matches!(row.kind, CompactionKind::Unknown) || !row_text_is_focusable(row) {
        return None;
    }

    let text = row.text.trim();
    if text.is_empty()
        || boilerplate_class(text).is_some()
        || serde_json::from_str::<serde_json::Value>(text).is_ok()
    {
        return None;
    }

    Some(text.to_string())
}

fn thread_goal_objective_text(row: &CompactionRow) -> Option<String> {
    if !matches!(row.kind, CompactionKind::Unknown) {
        return None;
    }

    let value: serde_json::Value = serde_json::from_str(&row.text).ok()?;
    if value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        != Some("thread_goal_updated")
    {
        return None;
    }

    value
        .get("threadId")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|thread_id| !thread_id.is_empty())?;

    value
        .get("goal")
        .and_then(|goal| goal.get("objective"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|objective| !objective.is_empty())
        .map(ToOwned::to_owned)
}

fn row_is_pure_boilerplate(row: &CompactionRow) -> bool {
    (objective_text_is_pure_boilerplate(&row.text) || row_is_session_bootstrap_scaffold(row))
        && extract_embedded_goal_line(&row.text).is_none()
        && extract_labeled_concrete_objective_text(&row.text).is_none()
        && extract_inline_concrete_objective_clause(&row.text).is_none()
        && !preserves_user_requested_boilerplate_target(row)
}

fn objective_text_is_pure_boilerplate(text: &str) -> bool {
    boilerplate_class(text).is_some()
}

fn row_is_session_bootstrap_scaffold(row: &CompactionRow) -> bool {
    if !matches!(
        row.kind,
        CompactionKind::SystemMessage | CompactionKind::DeveloperMessage
    ) {
        return false;
    }

    let lower = row.text.to_ascii_lowercase();
    lower.contains("prefer spawned subagents with the built-in `default` agent type")
        || lower.contains("only use non-default built-in agent roles on this profile")
        || (lower.contains("for sessions using the `") && lower.contains("profile:"))
}

fn preserves_user_requested_boilerplate_target(row: &CompactionRow) -> bool {
    if !matches!(row.kind, CompactionKind::UserMessage) {
        return false;
    }

    user_request_targets_boilerplate_surface(&row.text)
}

fn user_request_targets_boilerplate_surface(text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.starts_with("# AGENTS.md instructions")
        || trimmed.starts_with("<skill>")
        || trimmed.starts_with("Available skills")
    {
        return false;
    }

    let lower = text.to_ascii_lowercase();
    mentions_preservable_boilerplate_target(&lower)
}

fn mentions_preservable_boilerplate_target(lower: &str) -> bool {
    let target = [
        "agents.md instructions",
        "agents.md",
        "<skill>",
        "skill block",
        "skill file",
        "available skills",
        "codex desktop context",
        "plugin instructions",
        "apps (connectors)",
        "app scaffold",
        "plugin scaffold",
        "connector scaffold",
        "mcp tools",
        "mcp server",
        "tooling capability",
        "permission block",
        "approval policy",
        "memory block",
        "safety guardrails",
        "security guardrails",
        "policy block",
        "boilerplate",
        "instruction block",
    ]
    .iter()
    .any(|needle| lower.contains(needle));

    target
}

fn boilerplate_class(text: &str) -> Option<BoilerplateClass> {
    let lower = text.to_ascii_lowercase();
    if [
        "filesystem sandboxing",
        "approval policy",
        "permission_profile",
        "sandbox_mode",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Some(BoilerplateClass::PermissionBlock)
    } else if [
        "<skill>",
        "available skills",
        "skill roots",
        "how to use skills",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Some(BoilerplateClass::SkillsBlock)
    } else if ["agents.md instructions", "# agents.md", "<instructions>"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some(BoilerplateClass::AgentInstructionBlock)
    } else if [
        "memory summary begins",
        "use memory by default",
        "memory citation requirements",
        "<oai-mem-citation>",
        "user profile",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Some(BoilerplateClass::MemoryOrProfileBlock)
    } else if [
        "tools are grouped by namespace",
        "codex desktop context",
        "workspace dependencies",
        "how to use plugins",
        "plugins are enabled",
        "plugin instructions",
        "apps (connectors)",
        "apps can be explicitly triggered",
        "plugin bundles",
        "mcp tools",
        "app surfaces",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Some(BoilerplateClass::ToolingCapabilityBlock)
    } else if [
        "safety guardrails",
        "security guardrails",
        "never start implementing or making code/doc changes",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Some(BoilerplateClass::SafetyOrPolicyBlock)
    } else if ["project-doc", "<environment_context>"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some(BoilerplateClass::GenericScaffold)
    } else {
        None
    }
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
    use agent_session_compactor::{CompactionKind, CompactionRow, SourceKind, UserMessageRole};
    use camino::Utf8PathBuf;

    use crate::input::BundleSession;

    use super::{
        assign_drift_states, checkpoint_analyses, classify_command_role,
        kickoff_anchor_for_ordinal, session_kickoff_structured_goal_anchor,
        tool_output_is_unambiguous_failure, CommandRole, Confidence, DriftClass, DriftScore,
        DriftState, DriftStateHint, EvidenceRef, ObjectiveClass, ObjectiveIntent, ScoredDrift,
        StructuredObjective,
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
    fn checkpoints_preserve_structured_objective_when_narrowing_runs() {
        let prompt = "## Scope\nValidate the structured objective sidecar.\n\n## Verification\n- cargo test -p agent-drift-analyzer checkpoints -- --nocapture";
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![
                row(0, CompactionKind::UserMessage, prompt),
                tool_call(
                    1,
                    "functions.shell_command",
                    "{\"command\":\"echo structured-objective-checkpoint\",\"workdir\":\"/repo\"}",
                ),
            ],
            compact_rows: vec![
                row(0, CompactionKind::UserMessage, prompt),
                tool_call(
                    1,
                    "functions.shell_command",
                    "{\"command\":\"echo structured-objective-checkpoint\",\"workdir\":\"/repo\"}",
                ),
            ],
        };

        let analyses = checkpoint_analyses(&session);
        assert_eq!(analyses.len(), 1);

        let objective = &analyses[0].current.context.objective;
        assert!(objective
            .text
            .contains("Validate the structured objective sidecar."));

        let structured = objective.structured.as_ref().expect("structured objective");
        assert!(structured.evidence_spans.iter().any(|span| {
            span.role == crate::checkpoint::ObjectiveRole::Goal
                && matches!(
                    span.section_kind,
                    crate::checkpoint::ObjectiveSectionKind::Scope
                )
                && span
                    .excerpt
                    .contains("Validate the structured objective sidecar.")
                && span.section_index.is_some()
                && span.clause_index.is_some()
        }));
        assert!(structured.evidence_spans.iter().any(|span| {
            span.role == crate::checkpoint::ObjectiveRole::Verification
                && span
                    .excerpt
                    .contains("cargo test -p agent-drift-analyzer checkpoints")
                && span.section_index.is_some()
                && span.clause_index.is_some()
        }));
    }

    #[test]
    fn checkpoints_capture_kickoff_anchor_once_and_mark_sanctioned_replans_from_steer_rows() {
        let kickoff = "## Scope\nValidate the kickoff anchor helper in crates/agent-drift-analyzer/src/checkpoint/mod.rs only.\n\n## Deliverables\n- Return findings.\n\n## Verification\n- cargo test -p agent-drift-analyzer checkpoints -- --nocapture";
        let replan = "Replan instead: review crates/agent-drift-analyzer/src/checkpoint/export.rs only and return findings.";
        let followup =
            "Continue by validating the checkpoint exporter findings and return the result.";
        let mut replan_row = row(2, CompactionKind::UserMessage, replan);
        replan_row.user_message_role = Some(UserMessageRole::Steer);
        let mut followup_row = row(4, CompactionKind::UserMessage, followup);
        followup_row.user_message_role = Some(UserMessageRole::Steer);

        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![
                row(0, CompactionKind::UserMessage, kickoff),
                tool_call(
                    1,
                    "functions.shell_command",
                    "{\"command\":\"echo kickoff-anchor\",\"workdir\":\"/repo\"}",
                ),
                replan_row.clone(),
                tool_call(
                    3,
                    "functions.shell_command",
                    "{\"command\":\"echo sanctioned-replan\",\"workdir\":\"/repo\"}",
                ),
                followup_row.clone(),
                tool_call(
                    5,
                    "functions.shell_command",
                    "{\"command\":\"echo later-followup\",\"workdir\":\"/repo\"}",
                ),
            ],
            compact_rows: vec![
                row(0, CompactionKind::UserMessage, kickoff),
                tool_call(
                    1,
                    "functions.shell_command",
                    "{\"command\":\"echo kickoff-anchor\",\"workdir\":\"/repo\"}",
                ),
                replan_row,
                tool_call(
                    3,
                    "functions.shell_command",
                    "{\"command\":\"echo sanctioned-replan\",\"workdir\":\"/repo\"}",
                ),
                followup_row,
                tool_call(
                    5,
                    "functions.shell_command",
                    "{\"command\":\"echo later-followup\",\"workdir\":\"/repo\"}",
                ),
            ],
        };

        let analyses = checkpoint_analyses(&session);
        assert_eq!(analyses.len(), 3);
        assert!(!analyses[0].sanctioned_replan);
        assert!(analyses[1].sanctioned_replan);
        assert!(
            !analyses[2].sanctioned_replan,
            "later non-replan checkpoints must not inherit the prior sanctioned replan"
        );

        let (anchor_ordinal, anchor) =
            session_kickoff_structured_goal_anchor(&analyses).expect("kickoff structured anchor");
        assert_eq!(anchor_ordinal, 1);
        assert_eq!(anchor.confidence, Confidence::High);
        assert!(anchor
            .evidence_spans
            .iter()
            .any(|span| span.excerpt.contains("Validate the kickoff anchor helper")));
    }

    #[test]
    fn kickoff_anchor_for_ordinal_withholds_a_not_yet_established_anchor() {
        let anchor_structured = StructuredObjective {
            objective_class: ObjectiveClass::TaskStatement,
            primary_intent: ObjectiveIntent::Implement,
            target: None,
            constraints: Vec::new(),
            success_conditions: Vec::new(),
            deliverables: Vec::new(),
            verification_commands: Vec::new(),
            evidence_spans: Vec::new(),
            confidence: Confidence::High,
            unknowns: Vec::new(),
        };
        let anchor = Some((3usize, anchor_structured));

        assert!(
            kickoff_anchor_for_ordinal(anchor.as_ref(), 1).is_none(),
            "a checkpoint ordinally before the anchor's own checkpoint must not be scored against it"
        );
        assert!(
            kickoff_anchor_for_ordinal(anchor.as_ref(), 2).is_none(),
            "still before the anchor's ordinal"
        );
        assert!(
            kickoff_anchor_for_ordinal(anchor.as_ref(), 3).is_some(),
            "the anchor's own establishing checkpoint must see it"
        );
        assert!(
            kickoff_anchor_for_ordinal(anchor.as_ref(), 4).is_some(),
            "later checkpoints must see the established anchor"
        );
        assert!(kickoff_anchor_for_ordinal(None, 1).is_none());
    }

    #[test]
    fn sanctioned_replan_does_not_fire_on_routine_instead_of_tool_choice_steer_rows() {
        let routine = "Use rg instead of grep for this search, it's faster.";
        let mut routine_row = row(1, CompactionKind::UserMessage, routine);
        routine_row.user_message_role = Some(UserMessageRole::Steer);

        let session = BundleSession {
            session_id: "session-routine-instead".to_string(),
            archival_rows: vec![
                row(
                    0,
                    CompactionKind::UserMessage,
                    "/goal Implement the search helper.",
                ),
                routine_row.clone(),
            ],
            compact_rows: vec![
                row(
                    0,
                    CompactionKind::UserMessage,
                    "/goal Implement the search helper.",
                ),
                routine_row,
            ],
        };

        let analyses = checkpoint_analyses(&session);
        assert_eq!(analyses.len(), 1);
        assert!(
            !analyses[0].sanctioned_replan,
            "a routine tool-choice steer ('instead of') must not be treated as an authorized replan"
        );
    }

    #[test]
    fn assign_drift_states_never_promotes_a_cleared_semantic_goal_drift_score_to_historical() {
        // semantic_goal_drift never emits DriftStateHint::HistoricalContext (unlike
        // truth_grounding_gap/dead_end_thrash), so a cleared score must come out Cleared even
        // right after a previously Active one on the same class, not echo as HistoricalOnly.
        let previous_active = DriftScore {
            class: DriftClass::SemanticGoalDrift,
            state: DriftState::Active,
            raw_score: 80,
            confidence: Confidence::High,
            flagged: true,
            evidence: Vec::new(),
        };
        let cleared = ScoredDrift::new(
            DriftScore {
                class: DriftClass::SemanticGoalDrift,
                state: DriftState::Cleared,
                raw_score: 0,
                confidence: Confidence::Low,
                flagged: false,
                evidence: Vec::new(),
            },
            DriftStateHint::None,
        );

        let assigned = assign_drift_states(vec![cleared], Some(&[previous_active]));

        assert_eq!(assigned.len(), 1);
        assert_eq!(assigned[0].state, DriftState::Cleared);
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
