use std::collections::BTreeSet;

use crate::checkpoint::{
    CheckpointAnalysis, Confidence, DriftClass, DriftScore, DriftState, EvidenceRef,
    ObjectiveClass, ObjectiveConstraint, ObjectiveConstraintKind, ObjectiveEvidenceSpan,
    ObjectiveTarget, StructuredObjective,
};
use crate::context::ObjectiveSummary;
use crate::scoring::{DriftStateHint, ScoredDrift};

const CURRENT_GOAL_REASON_PREFIX: &str = "semantic goal drift current goal:";
const KICKOFF_ANCHOR_REASON_PREFIX: &str = "semantic goal drift kickoff anchor:";
const DRIFT_RAW_SCORE: u8 = 80;

pub(crate) fn score_semantic_goal_drift(
    analysis: &CheckpointAnalysis,
    kickoff_anchor: Option<&StructuredObjective>,
) -> ScoredDrift {
    let Some(current_goal) = eligible_current_goal(&analysis.current.context.objective) else {
        return no_claim(Confidence::Low);
    };
    let Some(anchor_goal) = eligible_anchor_goal(kickoff_anchor) else {
        return no_claim(Confidence::Low);
    };

    if analysis.sanctioned_replan || !semantic_goal_diverged(&current_goal, anchor_goal) {
        return no_claim(current_goal.structured.confidence);
    }

    let mut evidence = goal_evidence(
        current_goal.summary.evidence.first(),
        current_goal.description(),
        CURRENT_GOAL_REASON_PREFIX,
    );
    evidence.extend(structured_goal_evidence(
        anchor_goal,
        structured_goal_description(anchor_goal),
        KICKOFF_ANCHOR_REASON_PREFIX,
    ));
    dedupe_evidence(&mut evidence);

    ScoredDrift::new(
        DriftScore {
            class: DriftClass::SemanticGoalDrift,
            state: DriftState::Cleared,
            raw_score: DRIFT_RAW_SCORE,
            confidence: Confidence::High,
            flagged: true,
            evidence,
        },
        DriftStateHint::None,
    )
}

struct EligibleCurrentGoal<'a> {
    summary: &'a ObjectiveSummary,
    structured: &'a StructuredObjective,
    specific_terms: BTreeSet<String>,
}

impl EligibleCurrentGoal<'_> {
    fn description(&self) -> String {
        if !self.summary.comparison_key.trim().is_empty() {
            return self.summary.comparison_key.clone();
        }
        self.specific_terms.iter().cloned().collect::<Vec<_>>().join("|")
    }
}

fn eligible_current_goal(summary: &ObjectiveSummary) -> Option<EligibleCurrentGoal<'_>> {
    let structured = summary.structured.as_ref()?;
    if !structured_matches_current_goal_bar(structured) || summary.comparison_key.trim().is_empty() {
        return None;
    }

    Some(EligibleCurrentGoal {
        summary,
        structured,
        specific_terms: goal_specific_terms(structured, Some(summary)),
    })
}

fn eligible_anchor_goal(anchor: Option<&StructuredObjective>) -> Option<&StructuredObjective> {
    anchor.filter(|structured| structured_matches_anchor_bar(structured))
}

fn structured_matches_current_goal_bar(structured: &StructuredObjective) -> bool {
    structured_matches_confident_task_statement(structured, Confidence::Medium)
}

fn structured_matches_anchor_bar(structured: &StructuredObjective) -> bool {
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

// KNOWN LIMITATION (accepted v1 debt, not a bug to silently "fix" here — see
// docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md Resolved Decision 7 and
// docs/specs/r6/MAP.md item 5): this is a binary disjoint-set check, not a graduated distance.
// A legitimate narrowing (e.g. a target path narrowing from a crate root to one file inside it)
// reads as fully disjoint and gets flagged; any single shared constraint term (PlatformBoundary/
// ScopeBoundary) masks real drift. Revisiting this as a weighted/graduated distance is deferred
// to a later R6 iteration — do not assume this already grades partial overlap.
fn semantic_goal_diverged(
    current_goal: &EligibleCurrentGoal<'_>,
    anchor_goal: &StructuredObjective,
) -> bool {
    let anchor_terms = goal_specific_terms(anchor_goal, None);
    if current_goal.specific_terms.is_empty() || anchor_terms.is_empty() {
        return false;
    }

    current_goal.specific_terms.is_disjoint(&anchor_terms)
}

fn goal_specific_terms(
    structured: &StructuredObjective,
    summary: Option<&ObjectiveSummary>,
) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();

    if let Some(summary) = summary {
        for segment in summary.comparison_key.split('|') {
            push_goal_term(&mut terms, segment);
        }
    }

    if let Some(target) = structured.target.as_ref() {
        collect_target_terms(&mut terms, target);
    }

    for constraint in &structured.constraints {
        collect_constraint_terms(&mut terms, constraint);
    }
    terms
}

fn collect_target_terms(terms: &mut BTreeSet<String>, target: &ObjectiveTarget) {
    push_goal_term(terms, &target.display);
    for value in target
        .paths
        .iter()
        .chain(target.symbols.iter())
        .chain(target.named_artifacts.iter())
        .chain(target.workspace_refs.iter())
    {
        push_goal_term(terms, value);
    }
}

fn collect_constraint_terms(terms: &mut BTreeSet<String>, constraint: &ObjectiveConstraint) {
    if matches!(
        constraint.constraint_kind,
        ObjectiveConstraintKind::PlatformBoundary | ObjectiveConstraintKind::ScopeBoundary
    ) {
        push_goal_term(terms, &constraint.display);
    }
}

fn push_goal_term(terms: &mut BTreeSet<String>, raw: &str) {
    let normalized = normalize_goal_term(raw);
    if normalized.is_empty() || is_generic_goal_term(&normalized) {
        return;
    }
    terms.insert(normalized);
}

fn normalize_goal_term(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn is_generic_goal_term(term: &str) -> bool {
    matches!(
        term,
        "implement"
            | "debug"
            | "review"
            | "research"
            | "plan"
            | "validate"
            | "docs"
            | "other_task"
            | "repo_slice"
            | "crate_or_package"
            | "file_or_directory"
            | "spec_or_design_doc"
            | "test_or_verifier"
            | "skill_or_instruction_surface"
            | "external_artifact"
            | "conceptual_topic"
            | "unknown_target"
            | "green"
    )
}

fn structured_goal_description(structured: &StructuredObjective) -> String {
    let terms = goal_specific_terms(structured, None);
    if !terms.is_empty() {
        return terms.into_iter().collect::<Vec<_>>().join("|");
    }
    format!("{:?}", structured.primary_intent).to_lowercase()
}

fn goal_evidence(
    source: Option<&EvidenceRef>,
    description: String,
    reason_prefix: &str,
) -> Vec<EvidenceRef> {
    source
        .map(|evidence| EvidenceRef {
            row: evidence.row.clone(),
            reason: format!("{reason_prefix} {description}"),
        })
        .into_iter()
        .collect()
}

fn structured_goal_evidence(
    structured: &StructuredObjective,
    description: String,
    reason_prefix: &str,
) -> Vec<EvidenceRef> {
    structured_goal_spans(structured)
        .first()
        .map(|span| EvidenceRef {
            row: span.row.clone(),
            reason: format!("{reason_prefix} {description}"),
        })
        .into_iter()
        .collect()
}

fn structured_goal_spans(structured: &StructuredObjective) -> Vec<&ObjectiveEvidenceSpan> {
    let mut spans = structured.evidence_spans.iter().collect::<Vec<_>>();
    if let Some(target) = structured.target.as_ref() {
        spans.extend(target.evidence.iter());
    }
    for constraint in &structured.constraints {
        spans.extend(constraint.evidence.iter());
    }
    for success_condition in &structured.success_conditions {
        spans.extend(success_condition.evidence.iter());
    }
    for deliverable in &structured.deliverables {
        spans.extend(deliverable.evidence.iter());
    }
    spans
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

fn no_claim(confidence: Confidence) -> ScoredDrift {
    ScoredDrift::new(
        DriftScore {
            class: DriftClass::SemanticGoalDrift,
            state: DriftState::Cleared,
            raw_score: 0,
            confidence,
            flagged: false,
            evidence: Vec::new(),
        },
        DriftStateHint::None,
    )
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;

    use super::{
        score_semantic_goal_drift, CURRENT_GOAL_REASON_PREFIX, KICKOFF_ANCHOR_REASON_PREFIX,
    };
    use crate::checkpoint::{
        CheckpointAnalysis, CheckpointSlice, Confidence, EvidenceRef, ObjectiveClass,
        ObjectiveEvidenceSpan, ObjectiveIntent, ObjectiveRole, ObjectiveSectionKind,
        ObjectiveSourceKind, ObjectiveTarget, ObjectiveTargetKind, ObjectiveUnknown,
        RequestedDeliverable, RequestedDeliverableKind, StructuredObjective, SuccessCondition,
        TaskFrame, TurnActivityMix, TurnContext, TurnExecutionMode,
    };
    use crate::context::{
        CandidateTruthArtifact, CommandObservation, ContextPack, ObjectiveSummary, ToolObservation,
        WorkingSetPath,
    };
    use crate::inference::DelegationContext;
    use crate::input::BundleSession;

    #[test]
    fn semantic_goal_drift_flags_unauthorized_pivot_with_named_anchor_and_current_evidence() {
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_summary(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "legacy bridge display should be ignored",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(scored.score.flagged);
        assert_eq!(scored.score.class, crate::checkpoint::DriftClass::SemanticGoalDrift);
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(CURRENT_GOAL_REASON_PREFIX)));
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(KICKOFF_ANCHOR_REASON_PREFIX)));
    }

    #[test]
    fn semantic_goal_drift_skips_when_current_sidecar_is_absent() {
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let analysis = analysis_with_summary(
            objective_summary(
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs",
                None,
                "legacy-only objective text",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(!scored.score.flagged);
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_skips_when_current_sidecar_is_present_but_unknown() {
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal(
            "docs/specs/r6/MAP.md",
            Confidence::High,
            vec![ObjectiveUnknown {
                field_name: "target".to_string(),
                reason: "parser could not confirm scope".to_string(),
                evidence: vec![objective_span(2, "unknown target")],
            }],
        );
        let analysis = analysis_with_summary(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "unknown structured target",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(!scored.score.flagged);
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_skips_without_confident_anchor() {
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_summary(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(!scored.score.flagged);
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_skips_sanctioned_replan_pivots() {
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_summary(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            true,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(!scored.score.flagged);
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_prefers_structured_goal_over_legacy_bridge_display_string() {
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_summary(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "crates/agent-drift-analyzer/src/scoring/mod.rs",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            scored.score.flagged,
            "structured comparison_key drift should win even when the legacy bridge display string still matches the kickoff anchor"
        );
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(CURRENT_GOAL_REASON_PREFIX)));
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(KICKOFF_ANCHOR_REASON_PREFIX)));
    }

    fn analysis_with_summary(
        objective: ObjectiveSummary,
        sanctioned_replan: bool,
    ) -> CheckpointAnalysis {
        let task_frame = TaskFrame {
            objective: "legacy bridge objective".to_string(),
            confidence: Confidence::High,
            truth_artifacts: Vec::new(),
            working_set_paths: Vec::new(),
            tools: Vec::new(),
            command_families: Vec::new(),
            verification_commands: Vec::new(),
            supporting_evidence: Vec::new(),
            counter_evidence: Vec::new(),
        };
        let context = ContextPack {
            session_id: "session-alpha".to_string(),
            objective,
            truth_artifacts: Vec::<CandidateTruthArtifact>::new(),
            working_set_paths: Vec::<WorkingSetPath>::new(),
            tools: Vec::<ToolObservation>::new(),
            command_families: Vec::new(),
            command_observations: Vec::<CommandObservation>::new(),
            supporting_evidence: Vec::new(),
        };
        let window = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: Vec::new(),
        };

        CheckpointAnalysis {
            session_id: "session-alpha".to_string(),
            ordinal: 1,
            current: CheckpointSlice {
                window: window.clone(),
                context,
                task_frame: task_frame.clone(),
            },
            previous: None,
            sanctioned_replan,
            delegation: DelegationContext {
                topology: None,
                child_work_visibility: None,
                confidence: None,
                markers: Vec::new(),
                supporting_evidence: Vec::new(),
                counter_evidence: Vec::new(),
            },
            interval: crate::checkpoint::IntervalSlice {
                archival_rows: Vec::new(),
                compact_rows: Vec::new(),
                command_observations: Vec::new(),
                command_attempts: Vec::new(),
                verification_attempts: Vec::new(),
            },
            turn_context: TurnContext {
                turn_id: Some("turn-001".to_string()),
                turn_ordinal: 1,
                rows_since_turn_start: 0,
                seconds_since_turn_start: None,
                checkpoints_in_turn: 1,
                prompts_observed_in_session: 1,
                execution_mode: TurnExecutionMode::Mixed,
                activity_mix: TurnActivityMix::default(),
            },
            repetition: crate::checkpoint::RepetitionSlice {
                compact_rows: Vec::new(),
                repeated_verification_loops: Vec::new(),
                repeated_failure_loops: Vec::new(),
            },
            task_frame_delta: crate::checkpoint::TaskFrameDelta::default(),
            recovery: crate::checkpoint::RecoveryState::default(),
        }
    }

    fn objective_summary(
        comparison_key: &str,
        structured: Option<StructuredObjective>,
        text: &str,
    ) -> ObjectiveSummary {
        ObjectiveSummary {
            text: text.to_string(),
            comparison_key: comparison_key.to_string(),
            structured,
            verification_commands: Vec::new(),
            evidence: vec![EvidenceRef {
                row: row_ref(1),
                reason: "current objective evidence".to_string(),
            }],
        }
    }

    fn structured_goal(
        target_display: &str,
        confidence: Confidence,
        unknowns: Vec<ObjectiveUnknown>,
    ) -> StructuredObjective {
        StructuredObjective {
            objective_class: ObjectiveClass::TaskStatement,
            primary_intent: ObjectiveIntent::Implement,
            target: Some(ObjectiveTarget {
                display: target_display.to_string(),
                kind: ObjectiveTargetKind::FileOrDirectory,
                paths: vec![target_display.to_string()],
                symbols: Vec::new(),
                named_artifacts: Vec::new(),
                workspace_refs: Vec::new(),
                evidence: vec![objective_span(3, target_display)],
                confidence,
            }),
            constraints: Vec::new(),
            success_conditions: vec![SuccessCondition {
                display: "cargo test -p agent-drift-analyzer -- --nocapture".to_string(),
                evidence: vec![objective_span(4, "test wall")],
                confidence,
            }],
            deliverables: vec![RequestedDeliverable {
                display: "bounded implementation change".to_string(),
                deliverable_kind: RequestedDeliverableKind::CodeChange,
                evidence: vec![objective_span(5, "deliverable")],
                confidence,
            }],
            verification_commands: vec!["cargo test -p agent-drift-analyzer -- --nocapture".to_string()],
            evidence_spans: vec![objective_span(3, target_display)],
            confidence,
            unknowns,
        }
    }

    fn objective_span(event_index: usize, excerpt: &str) -> ObjectiveEvidenceSpan {
        ObjectiveEvidenceSpan {
            row: row_ref(event_index),
            source_kind: ObjectiveSourceKind::UserPrompt,
            section_kind: ObjectiveSectionKind::Mission,
            role: ObjectiveRole::Goal,
            excerpt: excerpt.to_string(),
            section_index: Some(0),
            clause_index: Some(0),
            start_char: Some(0),
            end_char: Some(excerpt.len()),
            confidence: Confidence::High,
        }
    }

    fn row_ref(event_index: usize) -> agent_session_compactor::RowRef {
        agent_session_compactor::RowRef {
            source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
            event_index,
            row_ordinal: 0,
        }
    }

}
