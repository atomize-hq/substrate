use std::fmt::Write;

use agent_drift_analyzer::{Checkpoint, DriftClass, EvidenceRef};
use camino::Utf8Path;

use crate::input::{CheckpointCursor, ReplayCheckpointBundle};
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

pub fn render_replay_report(
    bundle: &ReplayCheckpointBundle,
    checkpoints: &[Checkpoint],
    scheduler_policy: &SchedulerPolicy,
    warning_policy: &WarningPolicy,
) -> ReplayReport {
    let mut scheduler = ReplayScheduler::new(*scheduler_policy);
    let mut visible_warnings = Vec::new();
    let mut silent_checkpoints = Vec::new();

    for checkpoint in checkpoints {
        let cursor = CheckpointCursor::from(checkpoint);
        let fingerprint = warning_fingerprint(checkpoint);
        let trigger = if checkpoint.flagged {
            TriggerClass::RepeatedFailure
        } else {
            TriggerClass::CheckpointReady
        };
        let decision = scheduler.observe(cursor, trigger, checkpoint.flagged, Some(&fingerprint));
        let presentation = present_checkpoint(checkpoint, trigger, &decision, warning_policy);
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
    let disposition = classify_checkpoint(checkpoint, decision, warning_policy);
    let flagged_scores = checkpoint
        .drift_scores
        .iter()
        .filter(|score| score.flagged)
        .collect::<Vec<_>>();
    let evidence_lines = flagged_scores
        .iter()
        .flat_map(|score| score.evidence.iter())
        .take(warning_policy.max_evidence_lines)
        .map(format_evidence_ref)
        .collect::<Vec<_>>();
    let severity = max_flagged_score(checkpoint)
        .map(severity_for_score)
        .unwrap_or("low")
        .to_string();

    CheckpointPresentation {
        checkpoint: checkpoint.clone(),
        trigger,
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

pub fn classify_checkpoint(
    checkpoint: &Checkpoint,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> WarningDisposition {
    if !decision.evaluate {
        return WarningDisposition::Silent {
            reason: "scheduler cooldown deferred replay evaluation".to_string(),
        };
    }

    if !checkpoint.flagged {
        return WarningDisposition::Silent {
            reason: "checkpoint recorded without a visible warning".to_string(),
        };
    }

    let Some(max_score) = max_flagged_score(checkpoint) else {
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

pub fn warning_fingerprint(checkpoint: &Checkpoint) -> String {
    let classes = checkpoint
        .drift_scores
        .iter()
        .filter(|score| score.flagged)
        .map(|score| drift_class_name(score.class))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}:{}:{}",
        checkpoint.session_id, classes, checkpoint.expected_next_step
    )
}

fn max_flagged_score(checkpoint: &Checkpoint) -> Option<u8> {
    checkpoint
        .drift_scores
        .iter()
        .filter(|score| score.flagged)
        .map(|score| score.raw_score)
        .max()
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
        DriftClass::IgnoringRepoTruth => "ignoring_repo_truth",
        DriftClass::DeadEndThrash => "dead_end_thrash",
    }
}

fn format_trigger(trigger: TriggerClass) -> &'static str {
    match trigger {
        TriggerClass::CheckpointReady => "checkpoint_ready",
        TriggerClass::Heartbeat => "heartbeat",
        TriggerClass::RepeatedFailure => "repeated_failure",
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
