use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use agent_drift_analyzer::{
    Checkpoint, ChildWorkVisibility, DelegationContext, DelegationTopology, DriftClass,
    EvidenceRef, ProgressDimension, ProgressStatus, SessionArchetype, SessionArchetypeLabel,
    SessionProgress, TurnActivityMix, TurnContext, TurnExecutionMode,
};
use camino::Utf8Path;

use crate::checkpoint_interpretation::{
    interpret_checkpoint, project_checkpoint_compatibility, CheckpointCompatibilityProjectionInput,
    CheckpointInterpretation, CheckpointInterpretationInput,
};
use crate::input::{
    map_typed_contract_error, CheckpointCursor, InputError, ReplayCheckpointBundle,
};
use crate::scheduler::{
    DecisionReason, EvaluationDecision, ReplayScheduler, SchedulerPolicy, TriggerClass,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarningPolicy {
    pub minimum_visible_score: u8,
    pub max_evidence_lines: usize,
    pub max_objective_chars: usize,
}

impl Default for WarningPolicy {
    fn default() -> Self {
        Self {
            minimum_visible_score: 50,
            max_evidence_lines: 3,
            max_objective_chars: 120,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningDisposition {
    Visible,
    Silent { reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointPosture {
    Active,
    Recovered,
    HistoricalOnly,
}

impl CheckpointPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Recovered => "recovered",
            Self::HistoricalOnly => "historical-only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointDiagnosticsSummary {
    pub task_frame_transitioned: bool,
    pub working_set_changed: bool,
    pub interval_command_count: usize,
    pub interval_verification_command_count: usize,
    pub verification_density_basis_points: Option<u32>,
    pub evidence_item_count: usize,
}

impl CheckpointDiagnosticsSummary {
    pub fn from_checkpoint(checkpoint: &Checkpoint) -> Self {
        let diagnostics = &checkpoint.diagnostics;
        Self {
            task_frame_transitioned: diagnostics.task_frame_transitioned,
            working_set_changed: diagnostics.working_set_changed,
            interval_command_count: diagnostics.interval_command_count,
            interval_verification_command_count: diagnostics.interval_verification_command_count,
            verification_density_basis_points: verification_density_basis_points(
                diagnostics.interval_verification_command_count,
                diagnostics.interval_command_count,
            ),
            evidence_item_count: diagnostics.evidence_item_count,
        }
    }

    pub fn render_console_summary(&self) -> String {
        format!(
            "task_frame_transitioned={}, working_set_changed={}, verification={}/{} ({}), evidence_items={}",
            self.task_frame_transitioned,
            self.working_set_changed,
            self.interval_verification_command_count,
            self.interval_command_count,
            format_density(self.verification_density_basis_points),
            self.evidence_item_count
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointPresentation {
    pub checkpoint: Checkpoint,
    pub trigger: TriggerClass,
    pub posture: Option<CheckpointPosture>,
    pub disposition: WarningDisposition,
    pub severity: String,
    pub headline: String,
    pub objective: String,
    pub drift_summary: String,
    pub diagnostics_summary: CheckpointDiagnosticsSummary,
    pub expected_next_step: String,
    pub evidence_lines: Vec<String>,
}

impl CheckpointPresentation {
    pub fn render_console_block(&self, adjudication_note: Option<&str>) -> String {
        let mut lines = Vec::new();
        let label = match self.disposition {
            WarningDisposition::Visible => "warning",
            WarningDisposition::Silent { .. } => "checkpoint",
        };
        lines.push(format!("[{label}] {} ({})", self.headline, self.severity));
        lines.push(format!("- Objective: {}", self.objective));
        lines.push(format!("- Drift: {}", self.drift_summary));
        if let Some(posture) = self.posture {
            lines.push(format!("- Posture: {}", posture.as_str()));
        }
        if let Some(turn_context) = self.checkpoint.turn_context.as_ref() {
            lines.push(format!(
                "- Turn context: {}",
                format_turn_context(turn_context)
            ));
        }
        if let Some(session_archetype) = self.checkpoint.session_archetype.as_ref() {
            lines.push(format!(
                "- Archetype: {}",
                format_session_archetype_summary(session_archetype)
            ));
        }
        if let Some(session_progress) = self.checkpoint.session_progress.as_ref() {
            lines.push(format!(
                "- Progress: {}",
                format_session_progress_summary(session_progress)
            ));
        }
        if self.checkpoint.schema_version == "v0.8" {
            lines.push(format!(
                "- Delegation: {}",
                format_delegation_summary(&self.checkpoint.delegation)
            ));
        }
        lines.push(format!(
            "- Diagnostics: {}",
            self.diagnostics_summary.render_console_summary()
        ));
        lines.push(format!("- Expected next step: {}", self.expected_next_step));
        for evidence in &self.evidence_lines {
            lines.push(format!("- Evidence: {evidence}"));
        }
        if let WarningDisposition::Silent { reason } = &self.disposition {
            lines.push(format!("- Silent reason: {reason}"));
        }
        if let Some(note) = adjudication_note {
            lines.push(format!("- Adjudication: {note}"));
        }
        lines.join("\n")
    }
}

fn format_turn_context(turn_context: &TurnContext) -> String {
    let mut parts = vec![
        format_turn_label(turn_context),
        format!("rows={}", turn_context.rows_since_turn_start),
        format!("checkpoints={}", turn_context.checkpoints_in_turn),
        format!(
            "session-prompts={}",
            turn_context.prompts_observed_in_session
        ),
        format!(
            "mode={}",
            format_turn_execution_mode(turn_context.execution_mode)
        ),
        format!(
            "activity[{}]",
            format_turn_activity_mix(&turn_context.activity_mix)
        ),
    ];
    if let Some(seconds_since_turn_start) = turn_context.seconds_since_turn_start {
        parts.insert(2, format!("elapsed={}s", seconds_since_turn_start));
    }

    parts.join(" ")
}

fn format_turn_label(turn_context: &TurnContext) -> String {
    match (turn_context.turn_id.as_deref(), turn_context.turn_ordinal) {
        (Some(turn_id), ordinal) if ordinal > 0 => format!("{turn_id} (#{ordinal})"),
        (Some(turn_id), _) => turn_id.to_string(),
        (None, ordinal) if ordinal > 0 => format!("turn #{ordinal}"),
        (None, _) => "no turn id".to_string(),
    }
}

fn format_turn_activity_mix(activity_mix: &TurnActivityMix) -> String {
    format!(
        "dir={} asst={} tool={} read={} write={} verify={} out={}",
        activity_mix.directive_row_count,
        activity_mix.assistant_message_count,
        activity_mix.tool_call_count,
        activity_mix.read_like_command_count,
        activity_mix.write_like_command_count,
        activity_mix.verification_like_command_count,
        activity_mix.tool_output_count,
    )
}

fn format_turn_execution_mode(mode: TurnExecutionMode) -> &'static str {
    match mode {
        TurnExecutionMode::Conversational => "conversational",
        TurnExecutionMode::Autonomous => "autonomous",
        TurnExecutionMode::VerificationHeavy => "verification_heavy",
        TurnExecutionMode::Mixed => "mixed",
    }
}

fn format_session_archetype_summary(archetype: &SessionArchetype) -> String {
    format!(
        "label={} confidence={} support[{}] counter[{}]",
        format_session_archetype_label(archetype.label),
        confidence_name(archetype.confidence),
        format_archetype_evidence(&archetype.supporting_evidence),
        format_archetype_evidence(&archetype.counter_evidence)
    )
}

fn format_session_progress_summary(progress: &SessionProgress) -> String {
    format!(
        "status={} dimension={} confidence={} support[{}] counter[{}]",
        format_progress_status(progress.status),
        format_progress_dimension(progress.dimension),
        confidence_name(progress.confidence),
        format_progress_support(progress),
        format_progress_evidence(&progress.counter_evidence)
    )
}

fn format_delegation_summary(delegation: &DelegationContext) -> String {
    let parent = delegation.parent_session_id.as_deref().unwrap_or("none");
    let children = if delegation.child_session_ids.is_empty() {
        "none".to_string()
    } else {
        delegation.child_session_ids.join(",")
    };

    format!(
        "topology={} parent={parent} children=[{children}] visibility={} confidence={}",
        format_delegation_topology(delegation.topology),
        format_child_work_visibility(delegation.child_work_visibility),
        confidence_name(delegation.confidence)
    )
}

fn format_delegation_topology(topology: DelegationTopology) -> &'static str {
    match topology {
        DelegationTopology::SingleAgent => "single_agent",
        DelegationTopology::DelegatingParent => "delegating_parent",
        DelegationTopology::DelegatedChild => "delegated_child",
        DelegationTopology::MixedOrAmbiguous => "mixed_or_ambiguous",
    }
}

fn format_child_work_visibility(visibility: ChildWorkVisibility) -> &'static str {
    match visibility {
        ChildWorkVisibility::None => "none",
        ChildWorkVisibility::Linked => "linked",
        ChildWorkVisibility::Partial => "partial",
        ChildWorkVisibility::Opaque => "opaque",
    }
}

fn format_session_archetype_label(label: SessionArchetypeLabel) -> &'static str {
    match label {
        SessionArchetypeLabel::Troubleshooting => "troubleshooting",
        SessionArchetypeLabel::Planning => "planning",
        SessionArchetypeLabel::AutonomousImplementation => "autonomous_implementation",
        SessionArchetypeLabel::VerificationCloseout => "verification_closeout",
    }
}

fn format_archetype_evidence(evidence: &[EvidenceRef]) -> String {
    const DISPLAY_LIMIT: usize = 2;
    const REASON_LIMIT: usize = 64;

    if evidence.is_empty() {
        return "none".to_string();
    }

    let mut unique_reasons = Vec::<String>::new();
    for item in evidence {
        let reason = truncate(&item.reason, REASON_LIMIT);
        if unique_reasons.iter().any(|existing| existing == &reason) {
            continue;
        }
        unique_reasons.push(reason);
    }

    let remaining = unique_reasons.len().saturating_sub(DISPLAY_LIMIT);
    let mut displayed = unique_reasons
        .into_iter()
        .take(DISPLAY_LIMIT)
        .collect::<Vec<_>>();
    if remaining > 0 {
        displayed.push(format!("+{remaining} more"));
    }

    displayed.join("; ")
}

fn format_progress_status(status: ProgressStatus) -> &'static str {
    match status {
        ProgressStatus::Advancing => "advancing",
        ProgressStatus::Mixed => "mixed",
        ProgressStatus::Stalled => "stalled",
        ProgressStatus::Regressing => "regressing",
        ProgressStatus::InsufficientEvidence => "insufficient_evidence",
    }
}

fn format_progress_dimension(dimension: ProgressDimension) -> &'static str {
    match dimension {
        ProgressDimension::TroubleshootingFrontier => "troubleshooting_frontier",
        ProgressDimension::PlanningConvergence => "planning_convergence",
        ProgressDimension::ImplementationVerificationWall => "implementation_verification_wall",
        ProgressDimension::VerificationCloseoutNarrowing => "verification_closeout_narrowing",
        ProgressDimension::ParentVisibleOrchestration => "parent_visible_orchestration",
    }
}

fn format_progress_support(progress: &SessionProgress) -> String {
    const DISPLAY_LIMIT: usize = 2;
    const SUMMARY_LIMIT: usize = 64;

    if progress.signals.is_empty() {
        return format_progress_evidence(&progress.supporting_evidence);
    }

    format_compact_items(
        progress
            .signals
            .iter()
            .map(|signal| signal.summary.as_str()),
        DISPLAY_LIMIT,
        SUMMARY_LIMIT,
    )
}

fn format_progress_evidence(evidence: &[EvidenceRef]) -> String {
    format_compact_items(evidence.iter().map(|item| item.reason.as_str()), 2, 64)
}

fn format_compact_items<'a>(
    items: impl IntoIterator<Item = &'a str>,
    display_limit: usize,
    summary_limit: usize,
) -> String {
    let mut unique_items = Vec::<String>::new();
    for item in items {
        let item = truncate(item, summary_limit);
        if unique_items.iter().all(|existing| existing != &item) {
            unique_items.push(item);
        }
    }

    if unique_items.is_empty() {
        return "none".to_string();
    }

    let remaining = unique_items.len().saturating_sub(display_limit);
    let mut displayed = unique_items
        .into_iter()
        .take(display_limit)
        .collect::<Vec<_>>();
    if remaining > 0 {
        displayed.push(format!("+{remaining} more"));
    }

    displayed.join("; ")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    pub checkpoint_dir: camino::Utf8PathBuf,
    pub analyzer_summary_excerpt: Vec<String>,
    pub processed_checkpoints: usize,
    pub visible_warnings: Vec<CheckpointPresentation>,
    pub silent_checkpoints: Vec<CheckpointPresentation>,
    pub next_cursor: Option<CheckpointCursor>,
}

impl ReplayReport {
    pub fn to_console_text(&self) -> String {
        let mut body = String::new();
        let _ = writeln!(&mut body, "# Agent Drift Sentinel Replay");
        let _ = writeln!(&mut body);
        let _ = writeln!(
            &mut body,
            "Checkpoint bundle: `{}`",
            self.checkpoint_dir.as_str()
        );
        let _ = writeln!(
            &mut body,
            "Processed checkpoints: `{}`",
            self.processed_checkpoints
        );
        let _ = writeln!(
            &mut body,
            "Visible warnings: `{}`",
            self.visible_warnings.len()
        );
        let _ = writeln!(
            &mut body,
            "Silent checkpoints: `{}`",
            self.silent_checkpoints.len()
        );
        if let Some(cursor) = &self.next_cursor {
            let _ = writeln!(
                &mut body,
                "Next cursor: `{}:{}`",
                cursor.session_id, cursor.ordinal
            );
        }
        if !self.analyzer_summary_excerpt.is_empty() {
            let _ = writeln!(&mut body);
            let _ = writeln!(&mut body, "Analyzer summary:");
            for line in &self.analyzer_summary_excerpt {
                let _ = writeln!(&mut body, "- {line}");
            }
        }
        if !self.visible_warnings.is_empty() {
            let _ = writeln!(&mut body);
            let _ = writeln!(&mut body, "Visible warnings:");
            for warning in &self.visible_warnings {
                let _ = writeln!(&mut body);
                let _ = writeln!(&mut body, "{}", warning.render_console_block(None));
            }
        }
        if !self.silent_checkpoints.is_empty() {
            let _ = writeln!(&mut body);
            let _ = writeln!(&mut body, "Silent checkpoints:");
            for checkpoint in &self.silent_checkpoints {
                let _ = writeln!(&mut body);
                let _ = writeln!(&mut body, "{}", checkpoint.render_console_block(None));
            }
        }
        body.trim_end().to_string()
    }
}

pub(crate) fn try_render_replay_report(
    bundle: &ReplayCheckpointBundle,
    checkpoints: &[Checkpoint],
    scheduler_policy: &SchedulerPolicy,
    warning_policy: &WarningPolicy,
) -> Result<ReplayReport, InputError> {
    let included_checkpoint_ids = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.checkpoint_id.as_str())
        .collect::<HashSet<_>>();
    let mut previous_by_session = HashMap::new();
    let mut interpretations = Vec::new();

    for checkpoint in &bundle.checkpoints {
        let previous_same_session = previous_by_session
            .get(checkpoint.session_id.as_str())
            .copied();
        previous_by_session.insert(checkpoint.session_id.as_str(), checkpoint);
        if !included_checkpoint_ids.contains(checkpoint.checkpoint_id.as_str()) {
            continue;
        }

        interpretations.push(
            interpret_checkpoint(CheckpointInterpretationInput {
                checkpoint,
                previous_same_session,
            })
            .map_err(|error| map_typed_contract_error(bundle, error))?,
        );
    }

    let mut scheduler = ReplayScheduler::new(*scheduler_policy);
    let mut visible_warnings = Vec::new();
    let mut silent_checkpoints = Vec::new();
    for interpretation in &interpretations {
        let decision = scheduler.observe(
            interpretation.cursor.clone(),
            TriggerClass::CheckpointReady,
            interpretation.flagged,
            Some(&interpretation.warning_fingerprint),
        );
        let presentation = present_interpretation(
            interpretation,
            TriggerClass::CheckpointReady,
            &decision,
            warning_policy,
        );
        match presentation.disposition {
            WarningDisposition::Visible => visible_warnings.push(presentation),
            WarningDisposition::Silent { .. } => silent_checkpoints.push(presentation),
        }
    }

    Ok(ReplayReport {
        checkpoint_dir: bundle.checkpoint_dir.clone(),
        analyzer_summary_excerpt: bundle.summary_excerpt(4),
        processed_checkpoints: checkpoints.len(),
        visible_warnings,
        silent_checkpoints,
        next_cursor: checkpoints.last().map(CheckpointCursor::from),
    })
}

pub fn render_replay_report(
    bundle: &ReplayCheckpointBundle,
    checkpoints: &[Checkpoint],
    scheduler_policy: &SchedulerPolicy,
    warning_policy: &WarningPolicy,
) -> ReplayReport {
    let mut scheduler = ReplayScheduler::new(*scheduler_policy);
    let mut previous_by_session = HashMap::new();
    let mut visible_warnings = Vec::new();
    let mut silent_checkpoints = Vec::new();
    let included_checkpoint_ids = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.checkpoint_id.as_str())
        .collect::<HashSet<_>>();

    for checkpoint in &bundle.checkpoints {
        let previous_checkpoint = previous_by_session
            .get(checkpoint.session_id.as_str())
            .copied();
        previous_by_session.insert(checkpoint.session_id.as_str(), checkpoint);
        if !included_checkpoint_ids.contains(checkpoint.checkpoint_id.as_str()) {
            continue;
        }

        let interpretation =
            project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput {
                checkpoint,
                previous_checkpoint,
            });
        let decision = scheduler.observe(
            interpretation.cursor.clone(),
            TriggerClass::CheckpointReady,
            interpretation.flagged,
            Some(&interpretation.warning_fingerprint),
        );
        let presentation = present_interpretation(
            &interpretation,
            TriggerClass::CheckpointReady,
            &decision,
            warning_policy,
        );
        match presentation.disposition {
            WarningDisposition::Visible => visible_warnings.push(presentation),
            WarningDisposition::Silent { .. } => silent_checkpoints.push(presentation),
        }
    }

    let next_cursor = checkpoints.last().map(CheckpointCursor::from);
    ReplayReport {
        checkpoint_dir: bundle.checkpoint_dir.clone(),
        analyzer_summary_excerpt: bundle.summary_excerpt(4),
        processed_checkpoints: checkpoints.len(),
        visible_warnings,
        silent_checkpoints,
        next_cursor,
    }
}

pub fn present_checkpoint(
    checkpoint: &Checkpoint,
    trigger: TriggerClass,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> CheckpointPresentation {
    present_checkpoint_with_previous(checkpoint, None, trigger, decision, warning_policy)
}

pub fn present_checkpoint_with_previous(
    checkpoint: &Checkpoint,
    previous_checkpoint: Option<&Checkpoint>,
    trigger: TriggerClass,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> CheckpointPresentation {
    let interpretation = project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput {
        checkpoint,
        previous_checkpoint,
    });
    present_interpretation(&interpretation, trigger, decision, warning_policy)
}

pub(crate) fn present_interpretation(
    interpretation: &CheckpointInterpretation,
    trigger: TriggerClass,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> CheckpointPresentation {
    let checkpoint = &interpretation.checkpoint;
    let disposition = classify_interpretation(interpretation, decision, warning_policy);
    let flagged_scores = checkpoint
        .drift_scores
        .iter()
        .filter(|score| score.flagged)
        .collect::<Vec<_>>();
    let mut evidence_lines = Vec::new();
    push_evidence_lines(
        &mut evidence_lines,
        &interpretation.evidence,
        warning_policy.max_evidence_lines,
    );
    let severity = interpretation
        .max_flagged_score
        .map(severity_for_score)
        .unwrap_or("low")
        .to_string();

    CheckpointPresentation {
        checkpoint: checkpoint.clone(),
        trigger,
        posture: interpretation.posture,
        disposition,
        severity,
        headline: format!("{} @ {}", checkpoint.checkpoint_id, format_trigger(trigger)),
        objective: truncate(
            &checkpoint.task_frame.objective,
            warning_policy.max_objective_chars,
        ),
        drift_summary: if flagged_scores.is_empty() {
            "no flagged drift classes".to_string()
        } else {
            flagged_scores
                .iter()
                .map(|score| {
                    format!(
                        "{}={} ({})",
                        drift_class_name(score.class),
                        score.raw_score,
                        confidence_name(score.confidence)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        },
        diagnostics_summary: CheckpointDiagnosticsSummary::from_checkpoint(checkpoint),
        expected_next_step: checkpoint.expected_next_step.clone(),
        evidence_lines,
    }
}

fn classify_interpretation(
    interpretation: &CheckpointInterpretation,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> WarningDisposition {
    if !decision.evaluate {
        return WarningDisposition::Silent {
            reason: "scheduler cooldown deferred replay evaluation".to_string(),
        };
    }
    if !interpretation.flagged {
        return WarningDisposition::Silent {
            reason: "checkpoint recorded without a visible warning".to_string(),
        };
    }
    let Some(max_score) = interpretation.max_flagged_score else {
        return WarningDisposition::Silent {
            reason: "checkpoint flagged without a surfaced drift score".to_string(),
        };
    };
    if max_score < warning_policy.minimum_visible_score {
        return WarningDisposition::Silent {
            reason: format!(
                "flagged checkpoint stayed below visible score threshold ({max_score} < {})",
                warning_policy.minimum_visible_score
            ),
        };
    }
    if matches!(decision.reason, DecisionReason::WarningDebounced)
        || !decision.visible_warning_allowed
    {
        return WarningDisposition::Silent {
            reason: "warning debounce suppressed a duplicate replay warning".to_string(),
        };
    }
    WarningDisposition::Visible
}

fn push_evidence_lines(
    evidence_lines: &mut Vec<String>,
    evidence_refs: &[EvidenceRef],
    max_evidence_lines: usize,
) {
    for evidence in evidence_refs {
        let formatted = format_evidence_ref(evidence);
        if !evidence_lines.contains(&formatted) {
            evidence_lines.push(formatted);
        }
        if evidence_lines.len() >= max_evidence_lines {
            return;
        }
    }
}

pub fn classify_checkpoint(
    checkpoint: &Checkpoint,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> WarningDisposition {
    let interpretation = project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput {
        checkpoint,
        previous_checkpoint: None,
    });
    classify_interpretation(&interpretation, decision, warning_policy)
}

pub fn warning_fingerprint(checkpoint: &Checkpoint) -> String {
    project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput {
        checkpoint,
        previous_checkpoint: None,
    })
    .warning_fingerprint
}

fn severity_for_score(score: u8) -> &'static str {
    match score {
        80..=u8::MAX => "high",
        50..=79 => "medium",
        _ => "low",
    }
}

fn confidence_name(confidence: agent_drift_analyzer::Confidence) -> &'static str {
    match confidence {
        agent_drift_analyzer::Confidence::Low => "low",
        agent_drift_analyzer::Confidence::Medium => "medium",
        agent_drift_analyzer::Confidence::High => "high",
    }
}

fn drift_class_name(class: DriftClass) -> &'static str {
    match class {
        DriftClass::WrongPlanBranch => "wrong_plan_branch",
        DriftClass::TruthGroundingGap => "truth_grounding_gap",
        DriftClass::DeadEndThrash => "dead_end_thrash",
        DriftClass::SemanticGoalDrift => "semantic_goal_drift",
    }
}

fn format_trigger(trigger: TriggerClass) -> &'static str {
    match trigger {
        TriggerClass::CheckpointReady => "checkpoint_ready",
        TriggerClass::Heartbeat => "heartbeat",
        TriggerClass::RepeatedFailure => "scheduler_repeated_failure_trigger",
        TriggerClass::ManualReview => "manual_review",
    }
}

fn format_evidence_ref(reference: &EvidenceRef) -> String {
    format!(
        "{}#{}:{} {}",
        file_name(&reference.row.source_file),
        reference.row.event_index,
        reference.row.row_ordinal,
        truncate(&reference.reason, 96)
    )
}

fn file_name(path: &Utf8Path) -> &str {
    path.file_name().unwrap_or(path.as_str())
}

fn truncate(text: &str, max_chars: usize) -> String {
    let truncated = text.chars().take(max_chars).collect::<String>();
    if text.chars().count() > max_chars {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn verification_density_basis_points(
    verification_command_count: usize,
    command_count: usize,
) -> Option<u32> {
    if command_count == 0 {
        return None;
    }

    Some((((verification_command_count * 10_000) + (command_count / 2)) / command_count) as u32)
}

fn format_density(density_basis_points: Option<u32>) -> String {
    match density_basis_points {
        Some(basis_points) => {
            format!("{}.{:02}%", basis_points / 100, basis_points % 100)
        }
        None => "unavailable".to_string(),
    }
}
