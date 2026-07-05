use std::collections::BTreeSet;

use crate::checkpoint::{
    CheckpointAnalysis, Confidence, DriftClass, DriftScore, DriftState, EvidenceRef,
    ObjectiveClass, ObjectiveConstraint, ObjectiveConstraintKind, ObjectiveEvidenceSpan,
    ObjectiveTarget, StructuredObjective,
};
use crate::context::ObjectiveSummary;
use crate::inference::{ChildWorkVisibility, DelegationContext, DelegationTopology};
use crate::scoring::{DriftStateHint, ScoredDrift};

const CURRENT_GOAL_REASON_PREFIX: &str = "semantic goal drift current goal:";
const KICKOFF_ANCHOR_REASON_PREFIX: &str = "semantic goal drift kickoff anchor:";
const ROLLING_CURRENT_REASON_PREFIX: &str = "rolling semantic goal drift current goal:";
const ROLLING_PREVIOUS_REASON_PREFIX: &str = "rolling semantic goal drift previous goal:";
const DRIFT_RAW_SCORE: u8 = 80;

pub(crate) fn score_semantic_goal_drift(
    analysis: &CheckpointAnalysis,
    kickoff_anchor: Option<&StructuredObjective>,
) -> ScoredDrift {
    let Some(current_goal) = eligible_current_goal(&analysis.current.context.objective) else {
        return no_claim(Confidence::Low);
    };
    let anchor_goal = eligible_anchor_goal(kickoff_anchor);
    let previous_goal = eligible_previous_goal(analysis);
    let mut kickoff_drift =
        anchor_goal.is_some_and(|anchor| semantic_goal_diverged(&current_goal, anchor));
    let mut rolling_drift = previous_goal
        .as_ref()
        .is_some_and(|previous| rolling_goal_diverged(&current_goal, previous));

    // R6-3.5 delegation guardrail (bounded; full parent/child semantics stay in R7). On an OPAQUE
    // delegating-parent trace the analyzer sees only parent orchestration, so apparent objective
    // churn may be child work it cannot observe. Require a stable parent-visible *target anchor* on
    // both compared goals before claiming drift; otherwise fall through to limited-evidence
    // no-claim. Partial / MixedOrAmbiguous visibility is NOT hard-suppressed (codex review §5) — it
    // relies on the shared stable-term backstop already applied in eligibility.
    if is_opaque_delegated_parent(&analysis.delegation) {
        let current_anchor = has_stable_target_anchor(current_goal.structured);
        kickoff_drift =
            kickoff_drift && current_anchor && anchor_goal.is_some_and(has_stable_target_anchor);
        rolling_drift = rolling_drift
            && current_anchor
            && previous_goal
                .as_ref()
                .is_some_and(|previous| has_stable_target_anchor(previous.structured));
    }

    if analysis.sanctioned_replan {
        return no_claim(current_goal.structured.confidence);
    }
    if !(kickoff_drift || rolling_drift) {
        let confidence = if anchor_goal.is_some() || previous_goal.is_some() {
            current_goal.structured.confidence
        } else {
            Confidence::Low
        };
        return no_claim(confidence);
    }

    let mut evidence = Vec::new();
    if kickoff_drift {
        evidence.extend(goal_evidence(
            current_goal.summary.evidence.first(),
            current_goal.description(),
            CURRENT_GOAL_REASON_PREFIX,
        ));
        if let Some(anchor_goal) = anchor_goal {
            evidence.extend(structured_goal_evidence(
                anchor_goal,
                structured_goal_description(anchor_goal),
                KICKOFF_ANCHOR_REASON_PREFIX,
            ));
        }
    }
    if rolling_drift {
        // De-dup the shared current-goal line across families. When kickoff drift already
        // surfaced the current goal, a second `rolling ... current goal:` line names the same
        // goal and adds nothing; worse, under the default `max_evidence_lines` cap (3) that
        // redundant line pushes the informative `rolling ... previous goal:` line — the one
        // naming what the goal lurched away from — out of the rendered evidence. Only surface
        // the rolling current-goal line when kickoff did not already name the current goal.
        if !kickoff_drift {
            evidence.extend(goal_evidence(
                current_goal.summary.evidence.first(),
                current_goal.description(),
                ROLLING_CURRENT_REASON_PREFIX,
            ));
        }
        if let Some(previous_goal) = previous_goal.as_ref() {
            evidence.extend(goal_evidence(
                previous_goal.summary.evidence.first(),
                previous_goal.description(),
                ROLLING_PREVIOUS_REASON_PREFIX,
            ));
        }
    }
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
        self.specific_terms
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("|")
    }
}

fn eligible_current_goal(summary: &ObjectiveSummary) -> Option<EligibleCurrentGoal<'_>> {
    let structured = summary.structured.as_ref()?;
    if !structured_matches_current_goal_bar(structured) || summary.comparison_key.trim().is_empty()
    {
        return None;
    }
    // R6-3.5 backstop: a semantically eligible goal must carry at least one stable, non-junk
    // *distinguishing* term (from the structured target or the summary comparison_key) that is not
    // merely a constraint. This keeps the strict posture meaningful — a checkpoint resting on a
    // junk-only target (the F2 finding: 8/156 eligible checkpoints) can no longer fire, while a goal
    // whose distinguishing term lives only in its comparison_key stays eligible. Constraint terms
    // (platform/scope boundaries) alone never satisfy the requirement.
    if !has_stable_distinguishing_term(structured, Some(summary)) {
        return None;
    }

    Some(EligibleCurrentGoal {
        summary,
        structured,
        specific_terms: goal_specific_terms(structured, Some(summary)),
    })
}

fn eligible_anchor_goal(anchor: Option<&StructuredObjective>) -> Option<&StructuredObjective> {
    anchor.filter(|structured| {
        structured_matches_anchor_bar(structured)
            && has_stable_distinguishing_term(structured, None)
    })
}

/// True iff the goal contributes at least one stable, non-junk term (from its target or
/// comparison_key) that is not merely a constraint. Applied symmetrically to current, previous, and
/// anchor goals; the junk filter itself lives in the shared `push_goal_term` -> `is_stable_goal_term`.
fn has_stable_distinguishing_term(
    structured: &StructuredObjective,
    summary: Option<&ObjectiveSummary>,
) -> bool {
    let mut terms = goal_specific_terms(structured, summary);
    let mut constraint_terms = BTreeSet::new();
    for constraint in &structured.constraints {
        collect_constraint_terms(&mut constraint_terms, constraint);
    }
    terms.retain(|term| !constraint_terms.contains(term));
    !terms.is_empty()
}

/// A checkpoint whose parent delegates work to a child it cannot observe (`DelegatingParent` +
/// `Opaque`). The scorer only sees parent-side orchestration here, so it must not read parent-side
/// objective churn as drift without a concrete, parent-visible target anchor on both sides.
fn is_opaque_delegated_parent(delegation: &DelegationContext) -> bool {
    matches!(
        delegation.topology,
        Some(DelegationTopology::DelegatingParent)
    ) && matches!(
        delegation.child_work_visibility,
        Some(ChildWorkVisibility::Opaque)
    )
}

/// Stricter than `has_stable_distinguishing_term`: requires the *structured target* itself (not a
/// comparison_key-only term) to contribute a stable, non-junk term.
fn has_stable_target_anchor(structured: &StructuredObjective) -> bool {
    structured.target.as_ref().is_some_and(|target| {
        let mut terms = BTreeSet::new();
        collect_target_terms(&mut terms, target);
        !terms.is_empty()
    })
}

fn eligible_previous_goal(analysis: &CheckpointAnalysis) -> Option<EligibleCurrentGoal<'_>> {
    eligible_current_goal(&analysis.previous.as_ref()?.context.objective)
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

// KNOWN LIMITATION (accepted debt, partially retired by the R6-3.X.2 containment first cut — see
// docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md Resolved Decision 7,
// docs/specs/r6/MAP.md item 5, and the R6-3 TASKS ledger `R6-3.X.2`): divergence is a binary
// set comparison, not a graduated distance. The R6-3.X.2 first cut absorbs one relation beyond
// exact term equality: structural path/symbol containment (`goals_in_structural_containment`), so a
// goal narrowing from a crate/directory root to one file inside it — or broadening back out — no
// longer reads as a pivot. Containment is computed on the RAW structured-target strings
// (`target.display` / `paths` / `symbols` / …), split only on real structural separators
// (`/`, `\`, `::`), NOT on the normalized `specific_terms` set. That matters: `normalize_goal_term`
// flattens `/`, `-`, and `.` all to `_`, so a normalized-prefix test would treat `docs/specs/r6-map`
// and `docs/specs/r6/map.md` as ancestor/descendant and silently drop a real pivot (codex review
// finding). Splitting the raw path on structural separators only keeps `-`/`.` inside a segment, so
// `r6-map` and `r6` stay distinct and the pivot still fires.
//
// Everything else stays accepted v1 debt: sibling artifacts and shared family stems
// (`audit-trio.report.json` -> `audit-trio.model-selection/…report.json`), doc progression,
// plan->code->plan work cycles, and dotted work-item narrowing (`R6-3` -> `R6-3.5`, no structural
// separator) still read as fully disjoint; any single shared constraint term (PlatformBoundary/
// ScopeBoundary) masks real drift. Containment also only applies when both goals expose a concrete
// structured target — comparison_key-only goals fall through to the disjoint check.
//
// Known residual OVER-fire, deliberately NOT closed here (codex re-review §P2): a bare
// `ObjectiveTargetKind::CrateOrPackage` target (`agent-drift-analyzer`) is not matched as an
// ancestor of a path form of the same crate (`crates/agent-drift-analyzer/src/…`), because the bare
// name is not a structural path prefix of the path. So `Review the agent-drift-analyzer crate` ->
// `Update crates/agent-drift-analyzer/src/…` still fires. This is left as open debt rather than
// widened here: matching a bare crate name against an interior path segment needs the
// package-root convention (which lives in `context/objective.rs`, not the scorer) and a loose
// "segment appears anywhere" rule would reintroduce exactly the false negatives §P1 just fixed.
// Over-firing is the conservative direction (it never masks drift), so it waits for the graduated
// metric. The full weighted/graduated distance, and threading the anchor's own comparison_key so
// both sides extract symmetrically, remain deferred; do not assume this already grades partial
// overlap.
fn semantic_goal_diverged(
    current_goal: &EligibleCurrentGoal<'_>,
    anchor_goal: &StructuredObjective,
) -> bool {
    let anchor_terms = goal_specific_terms(anchor_goal, None);
    goal_sets_diverged(
        &current_goal.specific_terms,
        current_goal.structured,
        &anchor_terms,
        anchor_goal,
    )
}

fn rolling_goal_diverged(
    current_goal: &EligibleCurrentGoal<'_>,
    previous_goal: &EligibleCurrentGoal<'_>,
) -> bool {
    goal_sets_diverged(
        &current_goal.specific_terms,
        current_goal.structured,
        &previous_goal.specific_terms,
        previous_goal.structured,
    )
}

/// Two goals diverge when their term sets are fully disjoint AND their structured targets are not in
/// a structural path/symbol containment relationship. The disjoint check is the original binary rule;
/// the containment carve-out (computed on raw target strings, never normalized terms) rescues a
/// legitimate narrowing/broadening within one namespace so it does not read as drift.
fn goal_sets_diverged(
    current_terms: &BTreeSet<String>,
    current_structured: &StructuredObjective,
    other_terms: &BTreeSet<String>,
    other_structured: &StructuredObjective,
) -> bool {
    if current_terms.is_empty() || other_terms.is_empty() {
        return false;
    }
    if !current_terms.is_disjoint(other_terms) {
        return false;
    }
    !goals_in_structural_containment(current_structured, other_structured)
}

/// True iff either structured target is a structural ancestor of the other. Operates on the raw
/// target anchor strings (path/symbol form preserved) so `-`/`.` punctuation inside a segment cannot
/// forge a false boundary.
fn goals_in_structural_containment(a: &StructuredObjective, b: &StructuredObjective) -> bool {
    let a_anchors = target_anchor_strings(a);
    let b_anchors = target_anchor_strings(b);
    a_anchors.iter().any(|a_anchor| {
        b_anchors.iter().any(|b_anchor| {
            structural_path_ancestor_or_equal(a_anchor, b_anchor)
                || structural_path_ancestor_or_equal(b_anchor, a_anchor)
        })
    })
}

/// Raw specific strings for the structured target (display plus every concrete anchor list), with
/// original separators intact. Empty when the goal has no concrete target.
fn target_anchor_strings(structured: &StructuredObjective) -> Vec<&str> {
    let Some(target) = structured.target.as_ref() else {
        return Vec::new();
    };
    let mut anchors = vec![target.display.as_str()];
    for value in target
        .paths
        .iter()
        .chain(target.symbols.iter())
        .chain(target.named_artifacts.iter())
        .chain(target.workspace_refs.iter())
    {
        anchors.push(value.as_str());
    }
    anchors
}

/// `ancestor` structurally contains (or equals) `descendant`: split both on real structural
/// separators (`/`, `\`, `::`) — never `-`/`.` — and require the ancestor's segments to be a prefix
/// of the descendant's. `crates/foo` contains `crates/foo/bar.rs` and `a::b` contains `a::b::c`;
/// `docs/specs/r6-map` does NOT contain `docs/specs/r6/map.md`. Segment comparison is
/// case-SENSITIVE (codex re-review §P3): the same carve-out is applied to raw Rust symbol refs and
/// to paths on case-sensitive filesystems, where `Foo::Bar` and `foo::bar` are different items, so
/// a case-only difference must read as a real pivot, never as a benign narrowing.
fn structural_path_ancestor_or_equal(ancestor: &str, descendant: &str) -> bool {
    let ancestor_segments = structural_path_segments(ancestor);
    let descendant_segments = structural_path_segments(descendant);
    if ancestor_segments.is_empty() || ancestor_segments.len() > descendant_segments.len() {
        return false;
    }
    ancestor_segments
        .iter()
        .zip(descendant_segments.iter())
        .all(|(ancestor_segment, descendant_segment)| ancestor_segment == descendant_segment)
}

/// Split a raw target string into structural segments on `/`, `\`, and `::` only, dropping empties.
fn structural_path_segments(value: &str) -> Vec<&str> {
    value
        .split(['/', '\\'])
        .flat_map(|segment| segment.split("::"))
        .filter(|segment| !segment.is_empty())
        .collect()
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
    // R6-3.5 backstop: share the extraction taxonomy so junk (coordinate/number runs, control
    // residue, model-version tokens) can never become a distinguishing goal term, whatever ingestion
    // path produced it. Real-word constraint terms (`linux`, `windows`) pass `is_stable_goal_term`.
    if normalized.is_empty()
        || is_generic_goal_term(&normalized)
        || !crate::context::objective::is_stable_goal_term(&normalized)
    {
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
        eligible_current_goal, score_semantic_goal_drift, CURRENT_GOAL_REASON_PREFIX,
        KICKOFF_ANCHOR_REASON_PREFIX, ROLLING_CURRENT_REASON_PREFIX,
        ROLLING_PREVIOUS_REASON_PREFIX,
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
    use crate::inference::{ChildWorkVisibility, DelegationContext, DelegationTopology};
    use crate::input::BundleSession;

    fn opaque_delegating_parent() -> DelegationContext {
        DelegationContext {
            topology: Some(DelegationTopology::DelegatingParent),
            child_work_visibility: Some(ChildWorkVisibility::Opaque),
            confidence: Some(Confidence::Medium),
            markers: vec!["spawn_agent".to_string()],
            supporting_evidence: Vec::new(),
            counter_evidence: Vec::new(),
        }
    }

    #[test]
    fn semantic_goal_drift_suppressed_on_opaque_delegated_parent_without_current_target_anchor() {
        let anchor = structured_goal("crates/foo/anchor.rs", Confidence::High, Vec::new());
        // Current goal is distinguished only through its comparison_key (no structured target).
        let current =
            structured_goal_with_constraints(None, Confidence::High, Vec::new(), Vec::new());
        let mut analysis = analysis_with_summary(
            objective_summary(
                "implement|file_or_directory|docs_specs_r6_map_md",
                Some(current),
                "goal",
            ),
            false,
        );

        // Control: without delegation, comparison-key-only churn fires.
        assert!(
            score_semantic_goal_drift(&analysis, Some(&anchor))
                .score
                .flagged
        );

        // Opaque delegating parent: no parent-visible target anchor -> limited-evidence no-claim.
        analysis.delegation = opaque_delegating_parent();
        assert!(
            !score_semantic_goal_drift(&analysis, Some(&anchor))
                .score
                .flagged,
            "opaque delegated parent without a current target anchor must not fire"
        );
    }

    #[test]
    fn semantic_goal_drift_still_fires_on_opaque_delegated_parent_with_target_anchors_both_sides() {
        let anchor = structured_goal("crates/foo/anchor.rs", Confidence::High, Vec::new());
        let current = structured_goal("docs/specs/r6/map.md", Confidence::High, Vec::new());
        let mut analysis = analysis_with_summary(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "goal",
            ),
            false,
        );
        analysis.delegation = opaque_delegating_parent();

        assert!(
            score_semantic_goal_drift(&analysis, Some(&anchor))
                .score
                .flagged,
            "opaque delegated parent with stable target anchors on both sides still fires"
        );
    }

    #[test]
    fn semantic_goal_drift_not_hard_suppressed_on_partial_delegated_parent() {
        let anchor = structured_goal("crates/foo/anchor.rs", Confidence::High, Vec::new());
        let current =
            structured_goal_with_constraints(None, Confidence::High, Vec::new(), Vec::new());
        let mut analysis = analysis_with_summary(
            objective_summary(
                "implement|file_or_directory|docs_specs_r6_map_md",
                Some(current),
                "goal",
            ),
            false,
        );
        // Partial visibility relies on the stable-term backstop, not hard suppression (codex §5).
        analysis.delegation = DelegationContext {
            child_work_visibility: Some(ChildWorkVisibility::Partial),
            ..opaque_delegating_parent()
        };

        assert!(
            score_semantic_goal_drift(&analysis, Some(&anchor))
                .score
                .flagged,
            "partial delegated-parent visibility must not be hard-suppressed"
        );
    }

    #[test]
    fn semantic_goal_drift_rejects_junk_only_current_goal_eligibility() {
        // R6-3.5 backstop: a target resting entirely on log/coordinate junk (F2 finding) is not
        // a semantically eligible goal, even though it clears TaskStatement + Medium + no-unknowns.
        let junk = structured_goal("0.0.0.0:4000", Confidence::High, Vec::new());
        let summary = objective_summary(
            "implement|file_or_directory|0_0_0_0_4000",
            Some(junk),
            "goal",
        );
        assert!(
            eligible_current_goal(&summary).is_none(),
            "junk-only target must not be an eligible current goal"
        );
    }

    #[test]
    fn semantic_goal_drift_keeps_stable_target_goal_eligible() {
        let stable = structured_goal(
            "crates/agent-drift-analyzer/src/context/objective.rs",
            Confidence::High,
            Vec::new(),
        );
        let summary = objective_summary(
            "implement|file_or_directory|crates_agent_drift_analyzer_src_context_objective_rs",
            Some(stable),
            "goal",
        );
        assert!(
            eligible_current_goal(&summary).is_some(),
            "a stable repo-path target goal must remain eligible"
        );
    }

    #[test]
    fn semantic_goal_drift_does_not_fire_on_junk_only_current_goal() {
        let anchor = structured_goal(
            "docs/architecture_overview.md",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("0.0.0.0:4000", Confidence::High, Vec::new());
        let analysis = analysis_with_summary(
            objective_summary(
                "implement|file_or_directory|0_0_0_0_4000",
                Some(current),
                "goal",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            !scored.score.flagged,
            "junk-only current goal must not fire semantic goal drift"
        );
    }

    #[test]
    fn structural_path_ancestry_respects_real_separators_only() {
        use super::structural_path_ancestor_or_equal;

        // Directory / crate root contains a file inside it (either split direction).
        assert!(structural_path_ancestor_or_equal(
            "crates/agent-drift-analyzer/src/context",
            "crates/agent-drift-analyzer/src/context/objective.rs"
        ));
        // Rust symbol-path narrowing (`::` is a structural separator).
        assert!(structural_path_ancestor_or_equal(
            "foo::bar",
            "foo::bar::baz"
        ));
        // Genuinely equal paths are ancestor-or-equal.
        assert!(structural_path_ancestor_or_equal(
            "docs/specs/r6/map.md",
            "docs/specs/r6/map.md"
        ));
        // Codex re-review §P3: comparison is case-SENSITIVE. Rust symbol refs and case-sensitive
        // filesystems treat these as different items, so a case-only difference must NOT be read as
        // benign containment (it would mask a real pivot).
        assert!(!structural_path_ancestor_or_equal(
            "docs/specs/r6/MAP.md",
            "docs/specs/r6/map.md"
        ));
        assert!(!structural_path_ancestor_or_equal(
            "Foo::Bar",
            "foo::bar::baz"
        ));
        // Codex finding: `-`/`.` are intra-segment punctuation, not separators, so a hyphen
        // collision must NOT forge ancestry (`r6-map` segment != `r6` segment).
        assert!(!structural_path_ancestor_or_equal(
            "docs/specs/r6-map",
            "docs/specs/r6/map.md"
        ));
        // Same-basename with/without extension is a leaf difference, not containment.
        assert!(!structural_path_ancestor_or_equal(
            "crates/foo/objective",
            "crates/foo/objective.rs"
        ));
        // Siblings sharing a parent are not ancestors.
        assert!(!structural_path_ancestor_or_equal(
            "docs/specs/r6/MAP.md",
            "docs/specs/r6/MAPPING-guide.md"
        ));
        // Empty ancestor never contains anything.
        assert!(!structural_path_ancestor_or_equal(
            "",
            "docs/specs/r6/map.md"
        ));
    }

    #[test]
    fn semantic_goal_drift_still_flags_hyphen_collision_pivot_through_scorer() {
        // Codex review regression guard: `docs/specs/r6-map` (a file) and `docs/specs/r6/map.md`
        // (a different file) both normalize to `docs_specs_r6_map*`, so a normalized-prefix
        // containment test would silently drop this pivot. The structural check splits on real
        // separators only, so `r6-map` != `r6` and the pivot still fires.
        let anchor = structured_goal("docs/specs/r6-map", Confidence::High, Vec::new());
        let current = structured_goal("docs/specs/r6/map.md", Confidence::High, Vec::new());
        let analysis = analysis_with_summary(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            scored.score.flagged,
            "a hyphen/slash normalization collision must not suppress a real pivot"
        );
    }

    #[test]
    fn semantic_goal_drift_does_not_flag_kickoff_narrowing_into_anchored_subtree() {
        // R6-3.X.2 containment first cut: a kickoff anchor naming a crate/directory root and a
        // current goal naming one file inside it is the canonical legitimate narrowing (R6-2 SPEC
        // Resolved Decision 7); it must not read as an abandoned goal.
        let anchor = structured_goal("crates/agent-drift-analyzer", Confidence::High, Vec::new());
        let current = structured_goal(
            "crates/agent-drift-analyzer/src/context/objective.rs",
            Confidence::High,
            Vec::new(),
        );
        let analysis = analysis_with_summary(
            objective_summary(
                "implement|file_or_directory|crates_agent_drift_analyzer_src_context_objective_rs",
                Some(current),
                "goal",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            !scored.score.flagged,
            "narrowing from an anchored crate root into one of its files must not flag kickoff semantic_goal_drift"
        );
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_does_not_flag_rolling_narrowing_from_directory_to_contained_file() {
        let previous = structured_goal("docs/specs/r6", Confidence::High, Vec::new());
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "review|spec_or_design_doc|docs_specs_r6",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(
            !scored.score.flagged,
            "narrowing from a directory goal into a file it contains must not flag rolling semantic_goal_drift"
        );
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_still_flags_sibling_artifacts_sharing_a_stem() {
        // Boundary guard: containment is whole-term, never common-prefix similarity. Sibling
        // artifacts under the same tree remain a real pivot (the existing co-fire and rolling
        // fixtures pin the same semantics for docs/specs/r6 siblings).
        let previous = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let current = structured_goal(
            "docs/specs/r6/MAPPING-guide.md",
            Confidence::High,
            Vec::new(),
        );
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_mapping_guide_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(
            scored.score.flagged,
            "sibling artifacts sharing a lexical stem are not hierarchically related and must still fire"
        );
    }

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
        assert_eq!(
            scored.score.class,
            crate::checkpoint::DriftClass::SemanticGoalDrift
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
    fn semantic_goal_drift_flags_rolling_pivot_with_named_previous_and_current_evidence() {
        let anchor = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let previous = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(scored.score.flagged);
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_CURRENT_REASON_PREFIX)));
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_PREVIOUS_REASON_PREFIX)));
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(CURRENT_GOAL_REASON_PREFIX)));
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(KICKOFF_ANCHOR_REASON_PREFIX)));
    }

    #[test]
    fn semantic_goal_drift_flags_kickoff_only_when_previous_goal_matches_current_goal() {
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let previous = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(scored.score.flagged);
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
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_CURRENT_REASON_PREFIX)));
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_PREVIOUS_REASON_PREFIX)));
    }

    #[test]
    fn semantic_goal_drift_rolling_still_flags_without_confident_anchor() {
        let previous = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(scored.score.flagged);
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_CURRENT_REASON_PREFIX)));
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_PREVIOUS_REASON_PREFIX)));
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(CURRENT_GOAL_REASON_PREFIX)));
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(KICKOFF_ANCHOR_REASON_PREFIX)));
    }

    #[test]
    fn semantic_goal_drift_rolling_still_flags_with_below_high_anchor() {
        // A present-but-ineligible kickoff anchor (below the High bar) is rejected by
        // eligible_anchor_goal via its confidence filter — a different path than an absent
        // (None) anchor. It must be a kickoff-side no-claim and must NOT short-circuit the
        // rolling comparison, so rolling still fires with an ineligible anchor present.
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::Medium,
            Vec::new(),
        );
        let previous = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            scored.score.flagged,
            "an ineligible below-High anchor must not short-circuit the rolling comparison"
        );
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_CURRENT_REASON_PREFIX)));
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_PREVIOUS_REASON_PREFIX)));
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(CURRENT_GOAL_REASON_PREFIX)));
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(KICKOFF_ANCHOR_REASON_PREFIX)));
    }

    #[test]
    fn semantic_goal_drift_skips_rolling_when_previous_checkpoint_is_absent() {
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
    fn semantic_goal_drift_skips_rolling_when_previous_goal_is_present_but_unknown() {
        let anchor = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let previous = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            vec![ObjectiveUnknown {
                field_name: "target".to_string(),
                reason: "previous checkpoint goal stayed ambiguous".to_string(),
                evidence: vec![objective_span(2, "unknown previous target")],
            }],
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs",
                Some(previous),
                "previous structured goal remained unknown",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(!scored.score.flagged);
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_skips_rolling_pivots_when_sanctioned_replan_is_present() {
        let previous = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs",
                Some(previous),
                "previous structured goal",
            )),
            true,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(!scored.score.flagged);
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_shared_boundary_keeps_slow_evolution_out_of_rolling_drift() {
        let previous = structured_goal_with_constraints(
            Some("crates/agent-drift-analyzer/src/scoring/mod.rs"),
            Confidence::High,
            vec![platform_boundary_constraint("linux")],
            Vec::new(),
        );
        let current = structured_goal_with_constraints(
            Some("docs/specs/r6/MAP.md"),
            Confidence::High,
            vec![platform_boundary_constraint("linux")],
            Vec::new(),
        );
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md|linux",
                Some(current),
                "current structured goal stayed on the linux verification frontier",
            ),
            Some(objective_summary_at(
                2,
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs|linux",
                Some(previous),
                "previous structured goal on the same linux verification frontier",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(
            !scored.score.flagged,
            "shared boundary terms should keep slow evolution out of rolling semantic_goal_drift"
        );
        assert!(scored.score.evidence.is_empty());
    }

    #[test]
    fn semantic_goal_drift_cofire_dedupes_current_goal_and_surfaces_rolling_previous() {
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let previous = structured_goal(
            "docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "docs|spec_or_design_doc|docs_specs_r6_r6_2_agent_drift_analyzer_semantic_goal_drift_spec_md",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(scored.score.flagged);
        let reasons = scored
            .score
            .evidence
            .iter()
            .map(|item| item.reason.as_str())
            .collect::<Vec<_>>();
        // On co-fire the shared current-goal line is de-duped: the redundant rolling current-goal
        // line is dropped so the informative rolling previous-goal line survives the default
        // max_evidence_lines cap (3). Order stays kickoff-family (current, anchor), then the
        // rolling previous-goal line.
        assert_eq!(reasons.len(), 3);
        assert!(reasons[0].starts_with(CURRENT_GOAL_REASON_PREFIX));
        assert!(reasons[1].starts_with(KICKOFF_ANCHOR_REASON_PREFIX));
        assert!(reasons[2].starts_with(ROLLING_PREVIOUS_REASON_PREFIX));
        assert!(
            !reasons
                .iter()
                .any(|reason| reason.starts_with(ROLLING_CURRENT_REASON_PREFIX)),
            "co-fire must drop the redundant rolling current-goal line so the previous-goal line is not truncated under the default evidence cap"
        );
        assert_eq!(
            reasons
                .iter()
                .filter(|reason| reason.starts_with(CURRENT_GOAL_REASON_PREFIX))
                .count(),
            1,
            "co-fire surfaces exactly one current-goal line"
        );
    }

    #[test]
    fn semantic_goal_drift_rolling_previous_uses_summary_comparison_key_symmetrically() {
        let previous =
            structured_goal_with_constraints(None, Confidence::High, Vec::new(), Vec::new());
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_map_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "docs|spec_or_design_doc|docs_specs_r6_r6_2_agent_drift_analyzer_semantic_goal_drift_spec_md",
                Some(previous),
                "previous goal is only distinguishable through its comparison_key",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(
            scored.score.flagged,
            "rolling drift must keep extracting the previous goal's comparison_key even when its structured target is empty"
        );
        assert!(scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(ROLLING_CURRENT_REASON_PREFIX)));
        let previous_reason = scored
            .score
            .evidence
            .iter()
            .find(|item| item.reason.starts_with(ROLLING_PREVIOUS_REASON_PREFIX))
            .expect("rolling previous evidence");
        assert!(
            previous_reason.reason.contains(
                "docs|spec_or_design_doc|docs_specs_r6_r6_2_agent_drift_analyzer_semantic_goal_drift_spec_md"
            ),
            "the previous goal's distinguishing term must come from its summary comparison_key; a naive structured-only reuse would drop it"
        );
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(CURRENT_GOAL_REASON_PREFIX)));
        assert!(!scored
            .score
            .evidence
            .iter()
            .any(|item| item.reason.starts_with(KICKOFF_ANCHOR_REASON_PREFIX)));
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
        analysis_with_current_and_previous_summaries(objective, None, sanctioned_replan)
    }

    fn analysis_with_current_and_previous_summaries(
        objective: ObjectiveSummary,
        previous_objective: Option<ObjectiveSummary>,
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
            previous: previous_objective.map(|objective| CheckpointSlice {
                window: window.clone(),
                context: ContextPack {
                    session_id: "session-alpha".to_string(),
                    objective,
                    truth_artifacts: Vec::<CandidateTruthArtifact>::new(),
                    working_set_paths: Vec::<WorkingSetPath>::new(),
                    tools: Vec::<ToolObservation>::new(),
                    command_families: Vec::new(),
                    command_observations: Vec::<CommandObservation>::new(),
                    supporting_evidence: Vec::new(),
                },
                task_frame: task_frame.clone(),
            }),
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
        objective_summary_at(1, comparison_key, structured, text)
    }

    fn objective_summary_at(
        event_index: usize,
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
                row: row_ref(event_index),
                reason: "current objective evidence".to_string(),
            }],
        }
    }

    fn structured_goal(
        target_display: &str,
        confidence: Confidence,
        unknowns: Vec<ObjectiveUnknown>,
    ) -> StructuredObjective {
        structured_goal_with_constraints(Some(target_display), confidence, Vec::new(), unknowns)
    }

    fn structured_goal_with_constraints(
        target_display: Option<&str>,
        confidence: Confidence,
        constraints: Vec<crate::checkpoint::ObjectiveConstraint>,
        unknowns: Vec<ObjectiveUnknown>,
    ) -> StructuredObjective {
        StructuredObjective {
            objective_class: ObjectiveClass::TaskStatement,
            primary_intent: ObjectiveIntent::Implement,
            target: target_display.map(|target_display| ObjectiveTarget {
                display: target_display.to_string(),
                kind: ObjectiveTargetKind::FileOrDirectory,
                paths: vec![target_display.to_string()],
                symbols: Vec::new(),
                named_artifacts: Vec::new(),
                workspace_refs: Vec::new(),
                evidence: vec![objective_span(3, target_display)],
                confidence,
            }),
            constraints,
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
            verification_commands: vec![
                "cargo test -p agent-drift-analyzer -- --nocapture".to_string()
            ],
            evidence_spans: target_display
                .map(|target_display| vec![objective_span(3, target_display)])
                .unwrap_or_default(),
            confidence,
            unknowns,
        }
    }

    fn platform_boundary_constraint(display: &str) -> crate::checkpoint::ObjectiveConstraint {
        crate::checkpoint::ObjectiveConstraint {
            display: display.to_string(),
            constraint_kind: crate::checkpoint::ObjectiveConstraintKind::PlatformBoundary,
            evidence: vec![objective_span(6, display)],
            confidence: Confidence::High,
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
