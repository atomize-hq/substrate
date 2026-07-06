use std::cmp::Ordering;
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

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum GoalRelation {
    Exact,
    StructuralContainment,
    RepoRelativeEquivalentAfterCwdStrip,
    SameArtifactFamily,
    SameWorkItemFamily,
    SameDocFamily,
    PlanCodeRoleShift,
    ReviewFixVerifyRoleShift,
    SharedConstraintOnly,
    WeakOrGenericOnly,
    Unrelated,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AnchorRelationEvidence {
    left_anchor: String,
    right_anchor: Option<String>,
    note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WeightedRelationAssessment {
    relation: GoalRelation,
    score: u8,
    confidence: Confidence,
    decisive_evidence: Vec<AnchorRelationEvidence>,
    counter_evidence: Vec<AnchorRelationEvidence>,
}

impl WeightedRelationAssessment {
    fn suppressive(
        relation: GoalRelation,
        score: u8,
        confidence: Confidence,
        decisive_evidence: Vec<AnchorRelationEvidence>,
        counter_evidence: Vec<AnchorRelationEvidence>,
    ) -> Self {
        Self {
            relation,
            score,
            confidence,
            decisive_evidence,
            counter_evidence,
        }
    }

    fn drift(
        relation: GoalRelation,
        score: u8,
        confidence: Confidence,
        decisive_evidence: Vec<AnchorRelationEvidence>,
        counter_evidence: Vec<AnchorRelationEvidence>,
    ) -> Self {
        Self {
            relation,
            score,
            confidence,
            decisive_evidence,
            counter_evidence,
        }
    }

    fn no_claim(
        relation: GoalRelation,
        score: u8,
        confidence: Confidence,
        decisive_evidence: Vec<AnchorRelationEvidence>,
        counter_evidence: Vec<AnchorRelationEvidence>,
    ) -> Self {
        Self {
            relation,
            score,
            confidence,
            decisive_evidence,
            counter_evidence,
        }
    }

    fn claims_drift(&self) -> bool {
        matches!(
            self.relation,
            GoalRelation::SharedConstraintOnly
                | GoalRelation::WeakOrGenericOnly
                | GoalRelation::Unrelated
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AnchorKind {
    Path,
    Symbol,
    NamedArtifact,
    WorkspaceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AnchorRole {
    Plan,
    Spec,
    Tasks,
    Review,
    Findings,
    Verify,
    Code,
    GenericDoc,
    Other,
}

impl AnchorRole {
    fn is_docish(&self) -> bool {
        matches!(
            self,
            Self::Plan
                | Self::Spec
                | Self::Tasks
                | Self::Review
                | Self::Findings
                | Self::GenericDoc
        )
    }

    fn is_plan_spec_task(&self) -> bool {
        matches!(self, Self::Plan | Self::Spec | Self::Tasks)
    }

    fn is_review_or_findings(&self) -> bool {
        matches!(self, Self::Review | Self::Findings)
    }

    fn is_code_or_verify(&self) -> bool {
        matches!(self, Self::Code | Self::Verify)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreparedAnchor {
    raw: String,
    kind: AnchorKind,
    normalized: String,
    segments: Vec<String>,
    family_tokens: BTreeSet<String>,
    work_item_lineages: Vec<Vec<String>>,
    role: AnchorRole,
}

#[derive(Clone, Debug, Default)]
struct PreparedGoalSide {
    anchors: Vec<PreparedAnchor>,
    constraint_terms: BTreeSet<String>,
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
    semantic_goal_relation(
        &current_goal.specific_terms,
        current_goal.structured,
        Some(current_goal.summary),
        &goal_specific_terms(anchor_goal, None),
        anchor_goal,
        None,
    )
    .claims_drift()
}

fn rolling_goal_diverged(
    current_goal: &EligibleCurrentGoal<'_>,
    previous_goal: &EligibleCurrentGoal<'_>,
) -> bool {
    semantic_goal_relation(
        &current_goal.specific_terms,
        current_goal.structured,
        Some(current_goal.summary),
        &previous_goal.specific_terms,
        previous_goal.structured,
        Some(previous_goal.summary),
    )
    .claims_drift()
}

/// Relation-authoritative assessment over the stable target anchors. Numeric score remains explanatory
/// only; suppression vs drift routing is decided by the relation family itself.
fn semantic_goal_relation(
    current_terms: &BTreeSet<String>,
    current_structured: &StructuredObjective,
    current_summary: Option<&ObjectiveSummary>,
    other_terms: &BTreeSet<String>,
    other_structured: &StructuredObjective,
    other_summary: Option<&ObjectiveSummary>,
) -> WeightedRelationAssessment {
    let current_side = prepare_goal_side(current_structured, current_summary);
    let other_side = prepare_goal_side(other_structured, other_summary);
    let shared_terms = intersect_terms(current_terms, other_terms);
    let shared_constraint_terms =
        intersect_terms(&current_side.constraint_terms, &other_side.constraint_terms);

    if current_terms.is_empty() || other_terms.is_empty() {
        return WeightedRelationAssessment::no_claim(
            GoalRelation::Unknown,
            50,
            Confidence::Low,
            Vec::new(),
            vec![AnchorRelationEvidence {
                left_anchor: current_structured
                    .target
                    .as_ref()
                    .map(|target| target.display.clone())
                    .unwrap_or_default(),
                right_anchor: other_structured
                    .target
                    .as_ref()
                    .map(|target| target.display.clone()),
                note: "not enough stable goal terms to classify relation".to_string(),
            }],
        );
    }

    if exact_anchor_sets(current_structured, other_structured) {
        return WeightedRelationAssessment::suppressive(
            GoalRelation::Exact,
            5,
            Confidence::High,
            target_pair_evidence(
                current_structured,
                other_structured,
                "exact stable target match",
            ),
            Vec::new(),
        );
    }

    if goals_in_structural_containment(current_structured, other_structured) {
        return WeightedRelationAssessment::suppressive(
            GoalRelation::StructuralContainment,
            10,
            Confidence::High,
            target_pair_evidence(
                current_structured,
                other_structured,
                "structural containment on stable anchors",
            ),
            Vec::new(),
        );
    }

    if let Some(assessment) =
        doc_bundle_member_narrowing_relation(&current_side.anchors, &other_side.anchors)
    {
        return assessment;
    }

    if let Some(assessment) = all_anchors_match_relation(
        &current_side.anchors,
        &other_side.anchors,
        GoalRelation::SameArtifactFamily,
        same_artifact_family_anchor,
        24,
    ) {
        return assessment;
    }

    if let Some(assessment) = all_anchors_match_relation(
        &current_side.anchors,
        &other_side.anchors,
        GoalRelation::SameWorkItemFamily,
        same_work_item_family_anchor,
        30,
    ) {
        return assessment;
    }

    if let Some(assessment) = all_anchors_match_relation(
        &current_side.anchors,
        &other_side.anchors,
        GoalRelation::SameDocFamily,
        same_doc_family_anchor,
        34,
    ) {
        return assessment;
    }

    if let Some(assessment) = all_anchors_match_relation(
        &current_side.anchors,
        &other_side.anchors,
        GoalRelation::PlanCodeRoleShift,
        plan_code_role_shift_anchor,
        38,
    ) {
        return assessment;
    }

    if let Some(assessment) = all_anchors_match_relation(
        &current_side.anchors,
        &other_side.anchors,
        GoalRelation::ReviewFixVerifyRoleShift,
        review_fix_verify_role_shift_anchor,
        40,
    ) {
        return assessment;
    }

    if !shared_terms.is_empty() && shared_terms == shared_constraint_terms {
        return WeightedRelationAssessment::drift(
            GoalRelation::SharedConstraintOnly,
            72,
            Confidence::High,
            shared_term_evidence(&shared_terms, "shared constraint terms only"),
            target_pair_evidence(
                current_structured,
                other_structured,
                "anchors still disagree outside shared constraints",
            ),
        );
    }

    if !shared_terms.is_empty()
        && shared_terms
            .iter()
            .all(|term| is_weak_overlap_term(term) || shared_constraint_terms.contains(term))
    {
        return WeightedRelationAssessment::drift(
            GoalRelation::WeakOrGenericOnly,
            76,
            Confidence::Medium,
            shared_term_evidence(&shared_terms, "only weak/generic overlap remains"),
            target_pair_evidence(
                current_structured,
                other_structured,
                "no suppressive stable-anchor relation found",
            ),
        );
    }

    if current_side.anchors.is_empty() || other_side.anchors.is_empty() {
        if current_terms.is_disjoint(other_terms) {
            return WeightedRelationAssessment::drift(
                GoalRelation::Unrelated,
                DRIFT_RAW_SCORE,
                Confidence::Medium,
                Vec::new(),
                target_pair_evidence(
                    current_structured,
                    other_structured,
                    "stable terms disagree and one side lacks enough stable anchors",
                ),
            );
        }
        return WeightedRelationAssessment::no_claim(
            GoalRelation::Unknown,
            50,
            Confidence::Low,
            shared_term_evidence(
                &shared_terms,
                "some overlap remains but anchor proof is incomplete",
            ),
            Vec::new(),
        );
    }

    WeightedRelationAssessment::drift(
        GoalRelation::Unrelated,
        DRIFT_RAW_SCORE,
        Confidence::High,
        Vec::new(),
        target_pair_evidence(
            current_structured,
            other_structured,
            "stable anchors disagree with no accepted family or role-shift explanation",
        ),
    )
}

fn exact_anchor_sets(a: &StructuredObjective, b: &StructuredObjective) -> bool {
    let a_anchors = target_concrete_anchors(a)
        .into_iter()
        .map(structural_path_segments_owned)
        .collect::<Vec<_>>();
    let b_anchors = target_concrete_anchors(b)
        .into_iter()
        .map(structural_path_segments_owned)
        .collect::<Vec<_>>();
    !a_anchors.is_empty() && a_anchors == b_anchors
}

fn prepare_goal_side(
    structured: &StructuredObjective,
    summary: Option<&ObjectiveSummary>,
) -> PreparedGoalSide {
    let mut side = PreparedGoalSide::default();
    for constraint in &structured.constraints {
        collect_constraint_terms(&mut side.constraint_terms, constraint);
    }
    if let Some(target) = structured.target.as_ref() {
        for value in &target.paths {
            side.anchors.push(prepare_anchor(value, AnchorKind::Path));
        }
        for value in &target.symbols {
            side.anchors.push(prepare_anchor(value, AnchorKind::Symbol));
        }
        for value in &target.named_artifacts {
            side.anchors
                .push(prepare_anchor(value, AnchorKind::NamedArtifact));
        }
        for value in &target.workspace_refs {
            side.anchors
                .push(prepare_anchor(value, AnchorKind::WorkspaceRef));
        }
    }
    if side.anchors.is_empty() {
        if let Some(summary) = summary {
            for segment in summary.comparison_key.split('|') {
                if !segment.trim().is_empty() {
                    side.anchors
                        .push(prepare_anchor(segment, AnchorKind::NamedArtifact));
                }
            }
        }
    }
    side
}

fn prepare_anchor(raw: &str, kind: AnchorKind) -> PreparedAnchor {
    let normalized = normalize_goal_term(raw);
    let segments = structural_path_segments(raw)
        .into_iter()
        .map(normalize_goal_term)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let token_stream = normalized
        .split('_')
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    PreparedAnchor {
        raw: raw.to_string(),
        kind,
        normalized,
        segments,
        family_tokens: extract_family_tokens(&token_stream),
        work_item_lineages: extract_work_item_lineages(&token_stream),
        role: classify_anchor_role(raw, &token_stream),
    }
}

fn structural_path_segments_owned(value: &str) -> Vec<String> {
    structural_path_segments(value)
        .into_iter()
        .map(ToOwned::to_owned)
        .collect()
}

fn intersect_terms(left: &BTreeSet<String>, right: &BTreeSet<String>) -> BTreeSet<String> {
    left.intersection(right).cloned().collect()
}

fn all_anchors_match_relation<F>(
    current: &[PreparedAnchor],
    other: &[PreparedAnchor],
    relation: GoalRelation,
    predicate: F,
    score: u8,
) -> Option<WeightedRelationAssessment>
where
    F: Fn(&PreparedAnchor, &PreparedAnchor) -> Option<AnchorRelationEvidence>,
{
    if current.is_empty() || other.is_empty() {
        return None;
    }

    let current_matches = relation_matches(current, other, &predicate)?;
    let other_matches = relation_matches(other, current, &predicate)?;

    Some(WeightedRelationAssessment::suppressive(
        relation,
        score,
        Confidence::Medium,
        current_matches.into_iter().chain(other_matches).collect(),
        Vec::new(),
    ))
}

fn relation_matches<F>(
    from: &[PreparedAnchor],
    to: &[PreparedAnchor],
    predicate: &F,
) -> Option<Vec<AnchorRelationEvidence>>
where
    F: Fn(&PreparedAnchor, &PreparedAnchor) -> Option<AnchorRelationEvidence>,
{
    let mut evidence = Vec::new();
    for from_anchor in from {
        let matched = to
            .iter()
            .find_map(|to_anchor| predicate(from_anchor, to_anchor));
        let matched = matched?;
        evidence.push(matched);
    }
    Some(evidence)
}

fn same_artifact_family_anchor(
    left: &PreparedAnchor,
    right: &PreparedAnchor,
) -> Option<AnchorRelationEvidence> {
    if extension_only_leaf_variant(left, right) {
        return None;
    }
    let shared_family = shared_family_tokens(left, right);
    if shared_family.len() < 2 {
        return None;
    }
    if left.role.is_docish() && right.role.is_docish() {
        return None;
    }
    Some(AnchorRelationEvidence {
        left_anchor: left.raw.clone(),
        right_anchor: Some(right.raw.clone()),
        note: format!("same artifact-family tokens: {}", shared_family.join(",")),
    })
}

fn same_work_item_family_anchor(
    left: &PreparedAnchor,
    right: &PreparedAnchor,
) -> Option<AnchorRelationEvidence> {
    let lineage = shared_work_item_lineage(left, right)?;
    Some(AnchorRelationEvidence {
        left_anchor: left.raw.clone(),
        right_anchor: Some(right.raw.clone()),
        note: format!("same work-item lineage: {}", lineage.join(".")),
    })
}

fn doc_bundle_member_narrowing_relation(
    current: &[PreparedAnchor],
    other: &[PreparedAnchor],
) -> Option<WeightedRelationAssessment> {
    let (smaller, larger) = match current.len().cmp(&other.len()) {
        Ordering::Less => (current, other),
        Ordering::Greater => (other, current),
        Ordering::Equal => return None,
    };
    if smaller.is_empty()
        || !smaller.iter().all(|anchor| anchor.role.is_docish())
        || !larger.iter().all(|anchor| anchor.role.is_docish())
    {
        return None;
    }

    let mut evidence = Vec::new();
    for anchor in smaller {
        let matched = larger
            .iter()
            .find(|candidate| candidate.normalized == anchor.normalized)?;
        evidence.push(AnchorRelationEvidence {
            left_anchor: anchor.raw.clone(),
            right_anchor: Some(matched.raw.clone()),
            note: "doc bundle member narrowing/broadening via exact anchored doc reuse".to_string(),
        });
    }

    Some(WeightedRelationAssessment::suppressive(
        GoalRelation::SameDocFamily,
        28,
        Confidence::Medium,
        evidence,
        Vec::new(),
    ))
}

fn same_doc_family_anchor(
    left: &PreparedAnchor,
    right: &PreparedAnchor,
) -> Option<AnchorRelationEvidence> {
    if !(left.role.is_docish() && right.role.is_docish()) {
        return None;
    }
    let shared_family = shared_family_tokens(left, right);
    let shared_prefix = shared_segment_prefix_len(left, right);
    if shared_prefix < 3 {
        return None;
    }
    if shared_family.len() < 2 && shared_work_item_lineage(left, right).is_none() {
        return None;
    }
    Some(AnchorRelationEvidence {
        left_anchor: left.raw.clone(),
        right_anchor: Some(right.raw.clone()),
        note: format!(
            "same doc family under shared prefix depth {}",
            shared_prefix
        ),
    })
}

fn plan_code_role_shift_anchor(
    left: &PreparedAnchor,
    right: &PreparedAnchor,
) -> Option<AnchorRelationEvidence> {
    if !((left.role.is_plan_spec_task() && right.role.is_code_or_verify())
        || (right.role.is_plan_spec_task() && left.role.is_code_or_verify()))
    {
        return None;
    }
    if !shares_workstream(left, right) {
        return None;
    }
    Some(AnchorRelationEvidence {
        left_anchor: left.raw.clone(),
        right_anchor: Some(right.raw.clone()),
        note: "plan/spec/tasks ↔ code/verify role shift in one workstream".to_string(),
    })
}

fn review_fix_verify_role_shift_anchor(
    left: &PreparedAnchor,
    right: &PreparedAnchor,
) -> Option<AnchorRelationEvidence> {
    if !((left.role.is_review_or_findings() && right.role.is_code_or_verify())
        || (right.role.is_review_or_findings() && left.role.is_code_or_verify()))
    {
        return None;
    }
    if !shares_workstream(left, right) {
        return None;
    }
    Some(AnchorRelationEvidence {
        left_anchor: left.raw.clone(),
        right_anchor: Some(right.raw.clone()),
        note: "review/findings ↔ fix/verify role shift in one workstream".to_string(),
    })
}

fn shared_family_tokens(left: &PreparedAnchor, right: &PreparedAnchor) -> Vec<String> {
    left.family_tokens
        .intersection(&right.family_tokens)
        .cloned()
        .collect()
}

fn shares_workstream(left: &PreparedAnchor, right: &PreparedAnchor) -> bool {
    shared_family_tokens(left, right).len() >= 2 || shared_work_item_lineage(left, right).is_some()
}

fn shared_work_item_lineage(left: &PreparedAnchor, right: &PreparedAnchor) -> Option<Vec<String>> {
    let mut best: Option<Vec<String>> = None;
    for left_chain in &left.work_item_lineages {
        for right_chain in &right.work_item_lineages {
            let common = left_chain
                .iter()
                .zip(right_chain.iter())
                .take_while(|(l, r)| l == r)
                .map(|(value, _)| value.clone())
                .collect::<Vec<_>>();
            if common.len() >= 2 && best.as_ref().is_none_or(|best| common.len() > best.len()) {
                best = Some(common);
            }
        }
    }
    best
}

fn shared_segment_prefix_len(left: &PreparedAnchor, right: &PreparedAnchor) -> usize {
    left.segments
        .iter()
        .zip(right.segments.iter())
        .take_while(|(l, r)| l == r)
        .count()
}

fn extension_only_leaf_variant(left: &PreparedAnchor, right: &PreparedAnchor) -> bool {
    let Some(left_leaf) = left.segments.last() else {
        return false;
    };
    let Some(right_leaf) = right.segments.last() else {
        return false;
    };
    let left_stripped = strip_known_extension_suffix(left_leaf);
    let right_stripped = strip_known_extension_suffix(right_leaf);
    left_stripped == right_stripped
        && left_leaf != right_leaf
        && (left_leaf != left_stripped || right_leaf != right_stripped)
}

fn strip_known_extension_suffix(value: &str) -> &str {
    for suffix in [
        "_md", "_rs", "_py", "_ts", "_tsx", "_js", "_jsx", "_json", "_yaml", "_yml", "_toml",
        "_txt", "_sql", "_go", "_java", "_kt", "_c", "_cpp",
    ] {
        if let Some(stripped) = value.strip_suffix(suffix) {
            return stripped;
        }
    }
    value
}

fn extract_family_tokens(tokens: &[String]) -> BTreeSet<String> {
    tokens
        .iter()
        .filter(|token| !is_generic_family_token(token))
        .cloned()
        .collect()
}

fn extract_work_item_lineages(tokens: &[String]) -> Vec<Vec<String>> {
    let mut lineages = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if is_work_item_start_token(token) {
            let mut lineage = vec![token.clone()];
            let mut lookahead = index + 1;
            while lookahead < tokens.len() && is_work_item_continuation_token(&tokens[lookahead]) {
                lineage.push(tokens[lookahead].clone());
                lookahead += 1;
            }
            if lineage.len() >= 2 {
                lineages.push(lineage);
            }
            index = lookahead;
            continue;
        }
        index += 1;
    }
    lineages
}

fn is_work_item_start_token(token: &str) -> bool {
    matches!(token, "packet" | "slice" | "set" | "phase")
        || (token.starts_with('r')
            && token.len() > 1
            && token[1..].chars().all(|ch| ch.is_ascii_digit()))
}

fn is_work_item_continuation_token(token: &str) -> bool {
    token.chars().all(|ch| ch.is_ascii_digit())
        || (token.len() == 1 && token.chars().all(|ch| ch.is_ascii_alphabetic()))
        || (token.len() == 2
            && token.chars().next().is_some_and(|ch| ch.is_ascii_digit())
            && token
                .chars()
                .nth(1)
                .is_some_and(|ch| ch.is_ascii_alphabetic()))
}

fn classify_anchor_role(raw: &str, tokens: &[String]) -> AnchorRole {
    let normalized = normalize_goal_term(raw);
    let has = |needle: &str| normalized.contains(needle);
    if has("findings") || has("finding") {
        return AnchorRole::Findings;
    }
    if has("review") {
        return AnchorRole::Review;
    }
    if has("verify") || has("verification") || has("acceptance") || has("regression") {
        return AnchorRole::Verify;
    }
    if has("plan") {
        return AnchorRole::Plan;
    }
    if has("tasks") || has("task_ledger") {
        return AnchorRole::Tasks;
    }
    if has("spec") || has("design") {
        return AnchorRole::Spec;
    }
    if matches!(
        tokens.last().map(String::as_str),
        Some("rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "go" | "java" | "kt" | "c" | "cpp")
    ) || normalized.contains("_src_")
        || normalized.contains("crates_")
        || normalized.contains("_tests_")
    {
        return AnchorRole::Code;
    }
    if normalized.contains("docs_") || normalized.contains("readme") || normalized.ends_with("_md")
    {
        return AnchorRole::GenericDoc;
    }
    AnchorRole::Other
}

fn is_generic_family_token(token: &str) -> bool {
    matches!(
        token,
        "docs"
            | "doc"
            | "spec"
            | "specs"
            | "design"
            | "plan"
            | "task"
            | "tasks"
            | "review"
            | "verify"
            | "verification"
            | "finding"
            | "findings"
            | "fix"
            | "tests"
            | "test"
            | "acceptance"
            | "regression"
            | "readme"
            | "crates"
            | "crate"
            | "src"
            | "scripts"
            | "dev"
            | "md"
            | "rs"
            | "py"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "json"
            | "yml"
            | "yaml"
            | "toml"
            | "repo"
            | "only"
            | "file"
            | "directory"
    ) || token.chars().all(|ch| ch.is_ascii_digit())
}

fn is_weak_overlap_term(term: &str) -> bool {
    term.len() <= 3
        || term.ends_with("_md")
        || term.ends_with("_rs")
        || term.contains("spec")
        || term.contains("plan")
        || term.contains("tasks")
}

fn shared_term_evidence(terms: &BTreeSet<String>, note: &str) -> Vec<AnchorRelationEvidence> {
    terms
        .iter()
        .map(|term| AnchorRelationEvidence {
            left_anchor: term.clone(),
            right_anchor: None,
            note: note.to_string(),
        })
        .collect()
}

fn target_pair_evidence(
    left: &StructuredObjective,
    right: &StructuredObjective,
    note: &str,
) -> Vec<AnchorRelationEvidence> {
    vec![AnchorRelationEvidence {
        left_anchor: left
            .target
            .as_ref()
            .map(|target| target.display.clone())
            .unwrap_or_default(),
        right_anchor: right.target.as_ref().map(|target| target.display.clone()),
        note: note.to_string(),
    }]
}

/// True iff the two goals describe the same path/symbol region differing only in depth — a pure
/// narrowing or broadening, with no unrelated anchor on either side. A `/goal` clause can name
/// several concrete targets (all preserved on `target.paths`/`symbols`), so this is checked
/// SYMMETRICALLY: every concrete anchor of each goal must be structurally related (ancestor-or-equal,
/// either direction) to some anchor of the other. That closes both leaks codex review found — a
/// narrowing that adds an unrelated target (`docs/specs/r6` -> `docs/specs/r6/MAP.md +
/// crates/other/src/lib.rs`) and a broadening that adds one (`docs/specs/r6/MAP.md` ->
/// `docs/specs/r6 + crates/other/src/lib.rs`) — because in each case the unrelated anchor is related
/// to nothing on the other side, so the symmetric test fails and the pivot fires. A one-directional
/// "all within" test would mask the broadening case.
fn goals_in_structural_containment(a: &StructuredObjective, b: &StructuredObjective) -> bool {
    let a_anchors = target_concrete_anchors(a);
    let b_anchors = target_concrete_anchors(b);
    if a_anchors.is_empty() || b_anchors.is_empty() {
        return false;
    }
    every_anchor_related(&a_anchors, &b_anchors) && every_anchor_related(&b_anchors, &a_anchors)
}

/// Every `from` anchor is structurally related — ancestor-or-equal in either direction — to at least
/// one `to` anchor.
fn every_anchor_related(from: &[&str], to: &[&str]) -> bool {
    from.iter().all(|from_anchor| {
        to.iter().any(|to_anchor| {
            structural_path_ancestor_or_equal(from_anchor, to_anchor)
                || structural_path_ancestor_or_equal(to_anchor, from_anchor)
        })
    })
}

/// Raw concrete anchor strings for the structured target (paths / symbols / named artifacts /
/// workspace refs), with original separators intact. Excludes `target.display`, which is a derived
/// summary (e.g. a comma-joined artifact list) that every extraction path also mirrors into one of
/// these concrete lists — including it would let a synthetic join string distort the all-within test.
/// Empty when the goal has no concrete target.
fn target_concrete_anchors(structured: &StructuredObjective) -> Vec<&str> {
    let Some(target) = structured.target.as_ref() else {
        return Vec::new();
    };
    target
        .paths
        .iter()
        .chain(target.symbols.iter())
        .chain(target.named_artifacts.iter())
        .chain(target.workspace_refs.iter())
        .map(String::as_str)
        .collect()
}

/// `ancestor` structurally contains (or equals) `descendant`: split both on real structural
/// separators (`/`, `\`, `::`) — never `-`/`.` — and require the ancestor's segments to be a prefix
/// of the descendant's. `crates/foo` contains `crates/foo/bar.rs` and `a::b` contains `a::b::c`;
/// `docs/specs/r6-map` does NOT contain `docs/specs/r6/map.md`. Segment comparison is
/// case-SENSITIVE (codex re-review §P3): the same carve-out is applied to raw Rust symbol refs and
/// to paths on case-sensitive filesystems, where `Foo::Bar` and `foo::bar` are different items, so
/// a case-only difference must read as a real pivot, never as a benign narrowing. Segmentation
/// canonicalizes identity-preserving spellings (codex re-review round 4): a `./` current-dir prefix
/// and a trailing `:line`/`:line:col` reference are no-ops, so `src/lib.rs` relates to
/// `./src/lib.rs` and `exec.rs` to `exec.rs:1537` — otherwise a narrowing that only adds one of
/// those common forms would spuriously fire.
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

/// Split a raw target string into structural segments on `/`, `\`, and `::` only, dropping empty
/// and `.` (current-dir) segments, then stripping a trailing `:line` reference from the final
/// segment. `-`/`.` inside a segment are never separators; a leading-dot dotfile dir (`.github`) is
/// a real segment, only an exact `.` segment is dropped. The line strip runs on the leaf, not the
/// whole string (codex re-review round 5): on a Windows absolute path (`C:/repo/src/lib.rs:42`) a
/// whole-string strip stops at the drive-letter colon and never removes the `:42`, so the same-file
/// narrowing would still read as a pivot.
fn structural_path_segments(value: &str) -> Vec<&str> {
    let mut segments: Vec<&str> = value
        .split(['/', '\\'])
        .flat_map(|segment| segment.split("::"))
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect();
    if let Some(last) = segments.last_mut() {
        *last = strip_line_suffix(last);
    }
    segments
}

/// Strip a trailing `:line` / `:line:col` reference from a leaf segment (`exec.rs:1537` ->
/// `exec.rs`) so narrowing to a specific line stays the same file, not a pivot. Mirrors the
/// upstream `strip_line_ref` in `context/objective.rs` — including its per-leaf application, which
/// is what keeps a Windows drive colon out of reach — kept local to avoid widening that module's
/// visibility. A `:`-joined non-numeric tail (`a:b`) or a bare drive (`C:`, empty tail) is left
/// intact.
fn strip_line_suffix(value: &str) -> &str {
    match value.split_once(':') {
        Some((head, tail))
            if !head.is_empty()
                && !tail.is_empty()
                && tail.chars().all(|c| c.is_ascii_digit() || c == ':') =>
        {
            head
        }
        _ => value,
    }
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
        // Codex re-review round 4: identity-preserving spellings canonicalize. A `./` current-dir
        // prefix and a `:line`/`:line:col` suffix are no-ops, so these count as related.
        assert!(structural_path_ancestor_or_equal(
            "src/lib.rs",
            "./src/lib.rs"
        ));
        assert!(structural_path_ancestor_or_equal("exec.rs", "exec.rs:1537"));
        assert!(structural_path_ancestor_or_equal(
            "crates/foo",
            "crates/foo/exec.rs:1537:5"
        ));
        // A leading-dot dotfile dir is a real segment, not a current-dir no-op.
        assert!(structural_path_ancestor_or_equal(
            ".github/workflows",
            ".github/workflows/ci.yml"
        ));
        // A Rust symbol ref keeps its `::` tail (non-numeric), not stripped as a line ref.
        assert!(!structural_path_ancestor_or_equal("a::b", "a::c"));
        // Codex re-review round 5: the line strip runs on the leaf, not the whole string, so a
        // Windows absolute path canonicalizes too — a whole-string strip would stop at the
        // drive-letter colon and leave the `:42` in place.
        assert!(structural_path_ancestor_or_equal(
            "C:/repo/src/lib.rs",
            "C:/repo/src/lib.rs:42"
        ));
        assert!(structural_path_ancestor_or_equal(
            r"C:\repo\src",
            "C:/repo/src/lib.rs:42"
        ));
        // A bare drive segment (empty tail after the colon) is left intact, and a drive-only
        // ancestor still requires matching path segments below it.
        assert!(!structural_path_ancestor_or_equal(
            "C:/other",
            "C:/repo/src/lib.rs:42"
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
    fn semantic_goal_drift_still_flags_when_only_one_of_several_current_targets_is_nested() {
        // codex re-review §P2 (multi-anchor): the current goal narrows one target into the kickoff
        // anchor's subtree (docs/specs/r6 -> docs/specs/r6/MAP.md) but also picks up an unrelated
        // second target (crates/other/src/lib.rs). Containment must require ALL concrete anchors to
        // stay in one subtree, so the one nested pair cannot mask the unrelated work.
        let anchor = structured_goal("docs/specs/r6", Confidence::High, Vec::new());
        let current = structured_goal_with_paths(
            &["docs/specs/r6/MAP.md", "crates/other/src/lib.rs"],
            Confidence::High,
        );
        let analysis = analysis_with_summary(
            objective_summary(
                "implement|file_or_directory|docs_specs_r6_map_md|crates_other_src_lib_rs",
                Some(current),
                "goal",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            scored.score.flagged,
            "an unrelated second target must not be masked by one nested target"
        );
    }

    #[test]
    fn semantic_goal_drift_does_not_flag_line_suffix_narrowing_of_same_file() {
        // codex re-review round 4: narrowing to a specific line of the same file
        // (mod.rs -> mod.rs:42) is the same target, not a pivot. The `:line` suffix must
        // canonicalize away so structural containment (equal file) suppresses the claim.
        let anchor = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/mod.rs:42",
            Confidence::High,
            Vec::new(),
        );
        let analysis = analysis_with_summary(
            objective_summary(
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs_42",
                Some(current),
                "goal",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            !scored.score.flagged,
            "a :line suffix on the same file must canonicalize away, not read as a pivot"
        );
    }

    #[test]
    fn semantic_goal_drift_still_flags_when_broadening_adds_unrelated_target() {
        // codex re-review §P2 (broadening direction): the current goal broadens the previous file to
        // its directory (docs/specs/r6/MAP.md -> docs/specs/r6) but ALSO adds an unrelated target.
        // The old file sits under the broadened directory, so a one-directional all-within test would
        // mask it; the symmetric relatedness test must fire because crates/other relates to nothing
        // on the previous side.
        let previous = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let current = structured_goal_with_paths(
            &["docs/specs/r6", "crates/other/src/lib.rs"],
            Confidence::High,
        );
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "implement|file_or_directory|docs_specs_r6|crates_other_src_lib_rs",
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
            "broadening that also adds an unrelated target must still fire"
        );
    }

    #[test]
    fn semantic_goal_drift_suppresses_pure_broadening_to_a_containing_directory() {
        // Complement: a pure broadening (previous file -> its enclosing directory, nothing unrelated
        // added) is a legitimate scope change and must stay suppressed.
        let previous = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
        let current = structured_goal("docs/specs/r6", Confidence::High, Vec::new());
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "review|spec_or_design_doc|docs_specs_r6",
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
            !scored.score.flagged,
            "pure broadening into a containing directory is not drift"
        );
    }

    #[test]
    fn semantic_goal_drift_suppresses_when_all_current_targets_stay_in_anchor_subtree() {
        // Complement of the multi-anchor guard: when EVERY concrete target narrows into the anchor
        // subtree, it is a legitimate narrowing and must stay suppressed.
        let anchor = structured_goal("crates/agent-drift-analyzer", Confidence::High, Vec::new());
        let current = structured_goal_with_paths(
            &[
                "crates/agent-drift-analyzer/src/scoring/mod.rs",
                "crates/agent-drift-analyzer/src/context/objective.rs",
            ],
            Confidence::High,
        );
        let analysis = analysis_with_summary(
            objective_summary(
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_mod_rs|crates_agent_drift_analyzer_src_context_objective_rs",
                Some(current),
                "goal",
            ),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            !scored.score.flagged,
            "every concrete target staying inside the anchor subtree is a narrowing, not drift"
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
            scored.score.flagged,
            "shared-constraint-only overlap must no longer suppress rolling semantic_goal_drift"
        );
    }

    #[test]
    fn semantic_goal_drift_suppresses_same_work_item_family_progression() {
        let previous = structured_goal(
            "docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal(
            "docs/specs/r6/R6-3.5/agent-drift-analyzer-objective-target-hygiene-spec.md",
            Confidence::High,
            Vec::new(),
        );
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "docs|spec_or_design_doc|docs_specs_r6_r6_3_5_agent_drift_analyzer_objective_target_hygiene_spec_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "docs|spec_or_design_doc|docs_specs_r6_r6_3_agent_drift_analyzer_rolling_semantic_goal_drift_tasks_md",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(
            !scored.score.flagged,
            "same work-item family progression must not flag rolling semantic_goal_drift"
        );
    }

    #[test]
    fn semantic_goal_drift_still_flags_crate_package_pivot() {
        let anchor = structured_goal_with_target(
            "agent-drift-analyzer",
            ObjectiveTargetKind::CrateOrPackage,
            Vec::new(),
            Vec::new(),
            vec!["agent-drift-analyzer".to_string()],
            Vec::new(),
            Confidence::High,
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
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
            "a bare crate/package kickoff anchor must still fire when the work pivots to unrelated docs"
        );
    }

    #[test]
    fn semantic_goal_drift_still_flags_workspace_ref_pivot() {
        let anchor = structured_goal_with_target(
            "@shared-cab-app",
            ObjectiveTargetKind::RepoSlice,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec!["@shared-cab-app".to_string()],
            Confidence::High,
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
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
            "a workspace-ref kickoff anchor must still fire when the work pivots to unrelated docs"
        );
    }

    #[test]
    fn semantic_goal_drift_still_flags_verification_target_pivot() {
        let anchor = structured_goal_with_target(
            "objective_acceptance harness",
            ObjectiveTargetKind::TestOrVerifier,
            Vec::new(),
            Vec::new(),
            vec!["objective_acceptance harness".to_string()],
            Vec::new(),
            Confidence::High,
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
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
            "a verification-target kickoff anchor must still fire when the work pivots to unrelated docs"
        );
    }

    #[test]
    fn semantic_goal_drift_still_flags_repo_work_item_pivot() {
        let anchor = structured_goal_with_target(
            "B2.2",
            ObjectiveTargetKind::RepoSlice,
            Vec::new(),
            Vec::new(),
            vec!["B2.2".to_string()],
            Vec::new(),
            Confidence::High,
        );
        let current = structured_goal("docs/specs/r6/MAP.md", Confidence::High, Vec::new());
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
            "a repo/work-item kickoff anchor must still fire when the work pivots to unrelated docs"
        );
    }

    #[test]
    fn semantic_goal_drift_suppresses_plan_code_role_shift_with_shared_workstream() {
        let previous = structured_goal(
            "docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-plan.md",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal(
            "crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs",
            Confidence::High,
            Vec::new(),
        );
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "implement|file_or_directory|crates_agent_drift_analyzer_src_scoring_semantic_goal_drift_rs",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "plan|spec_or_design_doc|docs_specs_r6_r6_3_x_2b_agent_drift_analyzer_semantic_goal_drift_graduated_weighted_distance_plan_md",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(
            !scored.score.flagged,
            "plan-doc to code work inside one semantic-goal-drift workstream must stay suppressed"
        );
    }

    #[test]
    fn semantic_goal_drift_suppresses_review_fix_verify_role_shift_with_shared_workstream() {
        let previous = structured_goal(
            "docs/specs/r6/FINDINGS-semantic-goal-drift-validation.md",
            Confidence::High,
            Vec::new(),
        );
        let current = structured_goal(
            "crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs",
            Confidence::High,
            Vec::new(),
        );
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "verify|test_or_verifier|crates_agent_drift_analyzer_tests_semantic_goal_drift_acceptance_rs",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "review|spec_or_design_doc|docs_specs_r6_findings_semantic_goal_drift_validation_md",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(
            !scored.score.flagged,
            "review/findings to verify work inside one semantic-goal-drift workstream must stay suppressed"
        );
    }

    #[test]
    fn semantic_goal_drift_suppresses_doc_bundle_member_narrowing() {
        let mut anchor = structured_goal_with_paths(
            &[
                "architecture-overview.md",
                "README.md",
                "authentication-security.md",
                "graphql-federation.md",
                "module-development.md",
                "monitoring.md",
            ],
            Confidence::High,
        );
        anchor.primary_intent = ObjectiveIntent::Review;
        if let Some(target) = anchor.target.as_mut() {
            target.kind = ObjectiveTargetKind::SpecOrDesignDoc;
            target.display = "architecture-overview.md".to_string();
        }

        let previous = anchor.clone();
        let mut current = structured_goal("README.md", Confidence::High, Vec::new());
        current.primary_intent = ObjectiveIntent::OtherTask;
        if let Some(target) = current.target.as_mut() {
            target.kind = ObjectiveTargetKind::SpecOrDesignDoc;
        }

        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "other_task|spec_or_design_doc|readme_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "review|spec_or_design_doc|architecture_overview_md|authentication_security_md|graphql_federation_md|module_development_md|monitoring_md|readme_md",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, Some(&anchor));

        assert!(
            !scored.score.flagged,
            "narrowing from a doc bundle to an anchored member doc must stay suppressed for both kickoff and rolling comparisons"
        );
    }

    #[test]
    fn semantic_goal_drift_shared_constraint_only_overlap_still_flags_pivot() {
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
            scored.score.flagged,
            "shared-constraint-only overlap must not suppress an unrelated pivot"
        );
    }

    #[test]
    fn semantic_goal_drift_generic_spec_plan_tasks_overlap_still_flags_pivot() {
        let previous = structured_goal("docs/specs/r6/R6-1/plan.md", Confidence::High, Vec::new());
        let current = structured_goal(
            "docs/specs/design-arch/tasks.md",
            Confidence::High,
            Vec::new(),
        );
        let analysis = analysis_with_current_and_previous_summaries(
            objective_summary(
                "plan|spec_or_design_doc|docs_specs_design_arch_tasks_md",
                Some(current),
                "current structured goal",
            ),
            Some(objective_summary_at(
                2,
                "plan|spec_or_design_doc|docs_specs_r6_r6_1_plan_md",
                Some(previous),
                "previous structured goal",
            )),
            false,
        );

        let scored = score_semantic_goal_drift(&analysis, None);

        assert!(
            scored.score.flagged,
            "generic spec/plan/tasks overlap without shared lineage must still flag a pivot"
        );
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

    fn structured_goal_with_target(
        target_display: &str,
        kind: ObjectiveTargetKind,
        paths: Vec<String>,
        symbols: Vec<String>,
        named_artifacts: Vec<String>,
        workspace_refs: Vec<String>,
        confidence: Confidence,
    ) -> StructuredObjective {
        let mut goal = structured_goal_with_constraints(None, confidence, Vec::new(), Vec::new());
        goal.target = Some(ObjectiveTarget {
            display: target_display.to_string(),
            kind,
            paths,
            symbols,
            named_artifacts,
            workspace_refs,
            evidence: vec![objective_span(3, target_display)],
            confidence,
        });
        goal.evidence_spans = vec![objective_span(3, target_display)];
        goal
    }

    /// A TaskStatement goal whose structured target carries several concrete paths (multi-anchor),
    /// for the codex re-review §P2 all-within containment coverage.
    fn structured_goal_with_paths(paths: &[&str], confidence: Confidence) -> StructuredObjective {
        let mut goal = structured_goal_with_constraints(
            paths.first().copied(),
            confidence,
            Vec::new(),
            Vec::new(),
        );
        if let Some(target) = goal.target.as_mut() {
            target.paths = paths.iter().map(|path| path.to_string()).collect();
        }
        goal
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
