# R6-3.X.2B SPEC — Graduated / Weighted Semantic Distance For Semantic Goal Drift

Status: OPEN (planning lock 2026-07-06). Packet-scoped to analyzer-local semantic-goal-drift scoring,
acceptance coverage, corpus tooling strata, and routing docs.

Assumptions carried into this spec:
1. `R6-3.5` objective-target hygiene is already landed and remains the extraction baseline.
2. The `R6-3.X.2` containment first cut is already landed and must stay behaviorally intact.
3. The next blocker is precision-first relation grading over **stable target anchors**, not broader eligibility.
4. This packet is allowed to change analyzer-local scorer logic and analyzer-local validation/docs only.
5. Sentinel, compactor, `TaskFrame`, `working_set`, `progress`, and full delegated-subagent semantics stay out of scope.

Authority / cross-references:
- `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
- `docs/specs/r6/MAP.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
- `docs/specs/r6/R6-3.5/agent-drift-analyzer-objective-target-hygiene-{spec,plan,tasks}.md`
- `docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-{spec,plan,tasks}.md`
- `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`
- `docs/specs/design-arch/DESIGN-r5-structured-objective-{architecture,migration-and-integration,classifier-taxonomy,evaluation-and-annotation,map}.md`
- Live scorer/tests/tooling:
  - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - `scripts/dev/drift-batch-scan/**`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

## Objective

Design the next bounded semantic-goal-drift packet after target hygiene and containment: a deterministic,
analyzer-local **graduated / weighted relation assessment** that classifies goal-target relationships more
precisely than exact match / raw containment / disjoint.

Success means the scorer can:
- absorb legitimate progression that is currently still term-disjoint,
- keep true unrelated pivots firing,
- explain the decision with structured relation evidence,
- stay local to the analyzer scorer and its tests/docs/tooling,
- keep `R6-3.X.3` eligibility loosening deferred unless this packet's post-landing evidence says otherwise.

## Commands

Read-only planning commands used for this packet:

```bash
sed -n '1,220p' docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md
sed -n '1,260p' docs/specs/r6/MAP.md
sed -n '324,424p' docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
sed -n '1,620p' crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
sed -n '1,220p' crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs
find scripts/dev/drift-batch-scan -maxdepth 1 -type f | sort
```

Implementation verification commands this packet must carry forward:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo build -p agent-session-compactor -p agent-drift-analyzer
python3 scripts/dev/drift-batch-scan/sample_sessions.py --out /tmp/r6_3_x_2b_selected.jsonl
python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2b_selected.jsonl --batch-dir /tmp/r6_3_x_2b_batch
python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2b_batch/checkpoints
python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2b_batch/checkpoints
```

## Project Structure

Primary packet surfaces:
- `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - relation taxonomy
  - stable-anchor normalization helpers
  - weighted relation assessment
  - drift/no-claim routing based on relation outcome
- `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - curated acceptance allowlist
  - positive/negative control assertions
  - relation-proof expectations where acceptance-visible
- `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - bounded positive and negative corpus additions
- `scripts/dev/drift-batch-scan/{README.md,tabulate.py,inspect_targets.py,run_batch.py}`
  - 110-session rerun reporting
  - validation strata summaries when inferable from current export/tooling
- `docs/specs/r6/**`
  - findings, map, and task-ledger routing updates

Secondary scope only if the smallest analyzer-local move needs it:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/support/mod.rs`

## Code Style

Design constraints for the implementation agent:
- extend the existing deterministic scorer style; do not introduce model calls or learned similarity.
- keep relation logic small, ordered, and auditable rather than one broad fuzzy matcher.
- preserve the existing structural-containment semantics as-is; add new relation detectors **after** that
  decision point rather than widening the containment helper.
- treat raw structured target anchors as the authority for relation grading; use summary/comparison-key
  text only as supporting evidence or fallback explanation, not as a new eligibility surface.
- prefer explicit typed helpers such as `GoalRelation`, `AnchorRelationEvidence`, and
  `WeightedRelationAssessment` over ad hoc booleans.
- keep case-sensitive symbol/path protection, multi-anchor symmetry, and junk/stable-term guards intact.

Illustrative shape only (not line-prescriptive):

```rust
struct WeightedRelationAssessment {
    relation: GoalRelation,
    score: u8,
    confidence: Confidence,
    decisive_evidence: Vec<AnchorRelationEvidence>,
    counter_evidence: Vec<AnchorRelationEvidence>,
}
```

## Testing Strategy

This packet needs four proof layers:
1. scorer-local unit coverage for relation classification precedence and boundary cases;
2. acceptance-fixture coverage for positive and negative semantic-goal-drift cases through the live analyzer
   checkpoint path;
3. corpus-tooling rerun coverage on the 110-session harness with explicit strata and residue accounting;
4. docs-routing closeout proving whether `R6-3.X.3` remains deferred.

Acceptance is not just green tests. The packet is incomplete unless the rerun and findings update explain the
remaining residue honestly.

## Boundaries

Always do:
- keep the packet analyzer-local and precision-first.
- preserve the current `unknowns.is_empty()` eligibility bar.
- preserve the current structural-containment helper semantics.
- require multi-anchor symmetry for any suppressive relation classification.
- keep every new suppressive relation paired with at least one false-negative guard.
- update findings/map/ledger with the routing decision after the 110-session rerun.

Ask first before:
- widening beyond analyzer-local scorer/tests/docs/tooling,
- changing export/schema or checkpoint serialization,
- changing sentinel rendering or compactor normalization,
- migrating `TaskFrame`, `working_set`, `progress`, or downstream consumers.

Never do in this packet:
- loosen `R6-3.X.3` eligibility as part of the weighted-distance landing,
- broaden raw containment to solve family/progression cases,
- implement full parent/child semantic support,
- treat shared constraints alone as a suppressive relation,
- claim success from synthetic fixtures without the 110-session rerun,
- overfit relation rules to Rust/Codex/spec-doc naming only.

## Live Repo Truth Reconciled

The live authority stack agrees on the following starting point:
- `R6-3.5` removed junk-target over-fire.
- `R6-3.X.2` containment first cut removed structural subtree narrowing/broadening over-fire.
- `R6-3.6` proved the detector can still fire (`5/5` positive controls) and still stay quiet on bounded
  non-pivots (`5/5` negative controls).
- the fresh corpus rerun (`110` sessions / `938` checkpoints / `7` remaining disjoint adjacent pairs /
  `0` fires) shows the next blocker is **not** eligibility loosening.
- the remaining residue is progression/family/cycle style relation debt: same artifact family,
  same work-item family, doc progression, plan→code→plan, review→fix→verify, absolute-vs-repo-relative
  subtree equivalence, crate/package naming vs package-root paths, and shared-constraint masking.

Therefore this packet should not reopen extraction or containment. It should add a bounded relation layer on
stable target anchors.

## Design

### 1. Keep the current gate and containment cut intact

The packet starts **after** current-bar eligibility and exact-match checks. It must not change:
- `structured_matches_current_goal_bar(...)`
- `structured_matches_anchor_bar(...)`
- the `unknowns.is_empty()` requirement
- `goals_in_structural_containment(...)`
- the existing case-sensitive / raw-separator / leaf-line-strip rules

New logic sits after exact-match and structural-containment suppression, and before the final
"disjoint means drift" conclusion.

### 2. Introduce a stable-anchor relation taxonomy

Add an explicit relation taxonomy for compared goals. Recommended precedence:

1. `Exact`
   - same stable anchor set after current canonicalization.
2. `StructuralContainment`
   - existing landed helper; no behavior change.
3. `RepoRelativeEquivalentAfterCwdStrip`
   - absolute-vs-repo-relative spellings collapse to the same repo-relative anchor after stripping the
     traced session cwd prefix when present.
4. `SameArtifactFamily`
   - same stable artifact family, same repo workstream, different artifact role or stage.
5. `SameWorkItemFamily`
   - same numbered or dotted work-item lineage (`R6-3` → `R6-3.5`, `Packet-4` → `Packet-4.2`, etc.).
6. `SameDocFamily`
   - same documentation family/subtree with explicit progression roles (status/risks/planning/spec/examples).
7. `PlanCodePlanCycle`
   - plan/spec/task doc ↔ code target ↔ plan/spec/task doc within the same stable workstream.
8. `ReviewFixVerifyCycle`
   - review or finding target ↔ code fix target ↔ verifier/test target within the same stable workstream.
9. `SharedConstraintOnly`
   - overlap exists only in constraint terms; this is **not** suppressive.
10. `WeakOrGenericOnly`
    - overlap exists only in generic or weak family markers; this is **not** suppressive.
11. `Unrelated`
    - stable anchors disagree with no acceptable family/cycle explanation.
12. `Unknown`
    - not enough stable anchor information to justify either suppression or a confident pivot claim.

Important design rule: only relations `Exact` through `ReviewFixVerifyCycle` may suppress drift, and only
when every compared anchor is explained by that relation family or a stronger one. `SharedConstraintOnly`,
`WeakOrGenericOnly`, and `Unknown` are never enough to suppress by themselves.

### 3. Define the stable-anchor preparation layer

Build the weighted relation only from **stable target anchors** already preserved in structured objectives:
- `paths`
- `symbols`
- `named_artifacts`
- `workspace_refs`

For each anchor, derive a small analyzer-local comparison record such as:
- raw anchor string
- anchor kind
- current canonical form under existing containment rules
- repo-relative form when `session_meta.payload.cwd` can safely strip an absolute prefix
- leaf/basename
- extension / role suffix when applicable
- work-item tokens (e.g. `R6-3.5`, `packet-4`, `plan-04`)
- family tokens with generic/weak pieces removed
- workflow role hints (`plan`, `review`, `verify`, `code`, `examples`, `status`, etc.)

Guardrails:
- derive role/family signals from anchor structure first; do not add a broad prose classifier.
- keep type-aware rules: symbol logic stays case-sensitive; path logic may use cwd stripping but not global
  case folding.
- when no stable anchors survive, preserve the existing conservative behavior and surface `Unknown` or the
  current no-claim path rather than inventing relation confidence.

### 4. Add a weighted assessment shape

The scorer should compute a bounded assessment object for kickoff and rolling comparisons:

```rust
enum GoalRelation {
    Exact,
    StructuralContainment,
    RepoRelativeEquivalentAfterCwdStrip,
    SameArtifactFamily,
    SameWorkItemFamily,
    SameDocFamily,
    PlanCodePlanCycle,
    ReviewFixVerifyCycle,
    SharedConstraintOnly,
    WeakOrGenericOnly,
    Unrelated,
    Unknown,
}
```

Recommended assessment fields:
- `relation`
- `score` (`0..=100`, where lower means more related and higher means more divergent)
- `confidence`
- `decisive_evidence` (which anchors or normalized families caused the chosen relation)
- `counter_evidence` (unmatched anchors, conflicting family hints, missing cwd, constraint-only overlap, etc.)

Recommended score bands:
- `0..=10`: exact or containment-equivalent
- `11..=25`: same target family / repo-relative equivalent
- `26..=40`: same work-item/doc/cycle progression
- `41..=64`: ambiguous / unknown / weak evidence → conservative no-claim or preserve current non-fire posture
- `65..=84`: only weak or constraint overlap; treat as divergent unless stronger relation evidence exists
- `85..=100`: unrelated pivot

The implementation agent may adjust exact numeric values, but the packet must preserve the band meanings and
must document them in code comments/tests/docs.

### 5. Multi-anchor decision rule

For goals with multiple stable anchors, relation grading stays symmetric and explanation-complete:
- every anchor on both sides must be matched to an anchor of equal-or-stronger relation for suppressive
  outcomes;
- any unmatched anchor becomes counter-evidence and can escalate the relation to `Unrelated` or `Unknown`;
- one related pair must not mask one unrelated pair.

This extends the current containment symmetry rule rather than replacing it.

### 6. Drift routing policy

After exact-match and containment checks, use the weighted assessment as follows:
- suppress drift for high-confidence `Exact` / `StructuralContainment` / `RepoRelativeEquivalentAfterCwdStrip`
  / `SameArtifactFamily` / `SameWorkItemFamily` / `SameDocFamily` / `PlanCodePlanCycle` /
  `ReviewFixVerifyCycle`.
- keep or trigger drift for high-confidence `Unrelated`, `SharedConstraintOnly`, and `WeakOrGenericOnly`.
- prefer no-claim over speculative firing for `Unknown` or low-confidence middle-band assessments.

This packet is allowed to change how the analyzer reaches the drift decision, but it must stay conservative:
false positives may be reduced; false negatives must be pinned by explicit controls before any new
suppressive relation ships.

### 7. Required positive controls

Add new positive controls that prove the weighted relation does **not** over-suppress real pivots. At minimum:
1. crate/package pivot
2. workspace-ref pivot
3. verification-target pivot
4. repo/work-item pivot
5. shared-constraint false-negative guard (same platform/scope boundary, unrelated targets, must still fire)

Each must clear the existing eligibility bar and use stable target anchors.

### 8. Required negative controls

Add new negative controls that prove the weighted relation absorbs legitimate progression. At minimum:
1. absolute-vs-repo-relative subtree progression after cwd stripping
2. same artifact family progression
3. same work-item family progression
4. plan→code→plan cycle
5. review→fix→verify cycle

These are **in addition to** the already-landed containment negatives and must not replace them.

### 9. Generalization guardrails

The scorer must not become a Rust/Codex/spec-doc memorizer. Guardrails:
- prefer anchor-kind and repo-relative structure over hand-curated filename allowlists;
- if a rule depends on a workflow role (`plan`, `review`, `verify`), derive it from a small cross-language
  role vocabulary and anchor placement, not Rust-only path fragments;
- keep an explicit `Unknown` bucket rather than forcing every case into a suppressive family;
- require at least one counter-example test for each new suppressive relation family;
- do not treat any single shared token as enough unless the relation family definition says so and the
  whole anchor set agrees.

### 10. Validation strata

The rerun and findings update must report the packet across these strata when inferable without schema changes:
- **language/repo type:** Rust, JS/TS, Python, docs-only, mixed, unknown
- **workflow type:** implementation, docs/planning, verification, review/fix, mixed, unknown
- **tooling type:** Cargo/Rust, Node/npm, Python/pytest, generic filesystem/doc, unknown
- **delegation topology:** `single_agent`, `delegated_child_visible`, `delegated_parent_opaque`, `unknown`

Best-effort inference is acceptable if each row documents its heuristic source and supports an `unknown`
bucket. If a stratum cannot be inferred from the current export/tooling without duplicating Rust logic or
widening the export schema, the packet must record that honestly.

### 11. 110-session rerun gate

The packet is not complete without a fresh 110-session corpus rerun using the committed batch harness.
Required closeout items:
- total sessions / repos / checkpoints
- eligible and target-resolved funnel counts
- same-target suppressions
- weighted-relation suppressions by family when derivable
- remaining residue pairs with hand-labeled relation explanations
- strata split from the section above
- explicit yes/no routing decision for `R6-3.X.3`

### 12. Default routing decision

The default closeout decision is:
- **`R6-3.X.3` remains deferred** if the weighted packet lands cleanly and the rerun still shows no evidence
  that the strict eligibility bar is the next dominant blocker.
- reopen `R6-3.X.3` only if, after weighted relation grading is live, the rerun or curated positive
  controls show genuine pivots that remain suppressed solely by the eligibility bar rather than by relation
  grading.

## Acceptance

This packet is complete only when all are true:
1. The scorer has an explicit relation taxonomy covering the required families.
2. The scorer emits a weighted relation assessment shape with relation, score, confidence, decisive evidence,
   and counter-evidence.
3. Structural containment behavior is preserved, not broadened.
4. Shared-constraint-only overlap no longer suppresses unrelated pivots.
5. Positive controls include crate/package, workspace-ref, verification-target, and shared-constraint guards.
6. Negative controls include absolute-vs-repo-relative subtree, same artifact family, same work-item family,
   plan→code→plan, and review→fix→verify.
7. Every new suppressive relation has at least one paired false-negative guard.
8. The design documents generalization guardrails and validation strata.
9. The 110-session corpus rerun and findings update are completed or blocked honestly.
10. The closeout docs make an explicit `R6-3.X.3` defer/reopen decision.
11. No sentinel, compactor, export/schema, `TaskFrame`, `working_set`, or `progress` change lands in this packet.

## Non-Goals

- loosening the shared eligibility bar
- changing sentinel/operator-surface behavior
- changing compactor normalization
- broadening the raw containment carve-out
- migrating `TaskFrame`, `working_set`, or `progress`
- full delegated-session/subagent semantic support
- replacing deterministic scoring with a learned similarity model

## Open Questions

1. Should the weighted score bands be implemented as fixed constants or as a small mapping table on
   `GoalRelation` plus penalties? Recommendation: mapping table plus bounded penalties for unmatched anchors.
2. Which validation-strata buckets are truly derivable from current exports without export widening?
   Recommendation: land best-effort inference with an honest `unknown` bucket rather than block the packet.
3. Should kickoff-anchor comparisons gain any summary/comparison-key support in this packet? Recommendation:
   no new kickoff-side summary threading unless a required relation family cannot be expressed on stable target
   anchors alone; keep anchor comparison-key asymmetry deferred unless the implementation proof shows it is
   necessary for one of the named residue classes.
