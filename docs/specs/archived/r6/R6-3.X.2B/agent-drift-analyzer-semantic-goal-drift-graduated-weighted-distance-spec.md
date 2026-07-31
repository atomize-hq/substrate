# R6-3.X.2B SPEC — Graduated / Weighted Semantic Distance For Semantic Goal Drift

Status: OPEN (planning lock 2026-07-06, tightened after review). Packet-scoped to analyzer-local
semantic-goal-drift scoring, acceptance coverage, corpus tooling strata, and routing docs.

Assumptions carried into this spec:
1. `R6-3.5` objective-target hygiene is already landed and remains the extraction baseline.
2. The `R6-3.X.2` containment first cut is already landed and must stay behaviorally intact.
3. The next blocker is precision-first relation grading over **stable target anchors**, not broader eligibility.
4. This packet is allowed to change analyzer-local scorer logic and analyzer-local validation/docs only.
5. `crates/agent-drift-analyzer/src/context/objective.rs`, sentinel, compactor, `TaskFrame`, `working_set`,
   `progress`, and full delegated-subagent semantics stay out of scope unless the user explicitly reopens them.

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
sed -n '1,320p' docs/specs/r6/MAP.md
sed -n '324,520p' docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
sed -n '1,620p' crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
sed -n '1,260p' crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs
python3 scripts/dev/drift-batch-scan/sample_sessions.py --help
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
python3 scripts/dev/drift-batch-scan/sample_sessions.py --seed 42 --out /tmp/r6_3_x_2b_selected.jsonl
python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2b_selected.jsonl --batch-dir /tmp/r6_3_x_2b_batch
python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2b_batch/checkpoints
python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2b_batch/checkpoints
```

Implementation preflight when code work begins:
- before editing shared scorer helpers such as `eligible_current_goal`, `goal_specific_terms`, relation helpers,
  or any helper reused by both kickoff and rolling paths, run GitNexus impact analysis for the touched symbol(s)
  and record the blast radius; if the index is stale, refresh it first.
- before commit, run `gitnexus detect-changes` (or the repo-qualified CLI equivalent) to confirm the change set
  stays within the intended scorer/acceptance scope.

## Project Structure

Primary packet surfaces:
- `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - relation taxonomy
  - stable-anchor normalization helpers
  - relation-authoritative drift/no-claim routing
  - explanatory assessment payloads/evidence
- `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - curated acceptance allowlist
  - positive/negative control assertions
  - prior-witness non-regression checks
- `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - bounded positive and negative corpus additions
- `scripts/dev/drift-batch-scan/{README.md,tabulate.py,inspect_targets.py,run_batch.py}`
  - seed-pinned 110-session rerun reporting
  - validation strata summaries when inferable from current export/tooling
- `docs/specs/r6/**`
  - findings, map, and task-ledger routing updates

Secondary scope only if the smallest analyzer-local move needs it:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/support/mod.rs`

Out of scope unless the user explicitly reopens them:
- `crates/agent-drift-analyzer/src/context/objective.rs`
- `crates/agent-drift-analyzer/src/checkpoint/{progress.rs,working_set.rs,export.rs,schema.rs}`
- `crates/agent-drift-sentinel/**`
- `crates/agent-session-compactor/**`

## Code Style

Design constraints for the implementation agent:
- extend the existing deterministic scorer style; do not introduce model calls or learned similarity.
- keep the **relation taxonomy** as the only authoritative routing surface; if a numeric score remains, it is
  explanatory/audit-only and must not become the gating authority for suppress vs fire.
- keep relation logic small, ordered, and auditable rather than one broad fuzzy matcher.
- preserve the existing structural-containment semantics as-is; add new relation detectors **after** that
  decision point rather than widening the containment helper.
- treat raw structured target anchors as the authority for relation grading; use summary/comparison-key
  text only as supporting evidence or fallback explanation, not as a new eligibility or suppressive surface.
- prefer explicit typed helpers such as `GoalRelation`, `AnchorRelationEvidence`, and
  `WeightedRelationAssessment` over ad hoc booleans.
- keep case-sensitive symbol/path protection, multi-anchor symmetry, and junk/stable-term guards intact.
- do not sneak `context/objective.rs` extraction changes into this packet under relation-scoring work.

Illustrative shape only (not line-prescriptive):

```rust
struct WeightedRelationAssessment {
    relation: GoalRelation,
    score: u8, // explanatory only; relation + confidence decide routing
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
3. prior-witness non-regression across the existing `R6-1`/`R6-2`/`R6-3`/`R6-3.6` acceptance and regression surfaces;
4. corpus-tooling rerun coverage on the seed-pinned 110-session harness with explicit strata and residue accounting.

Acceptance is not just green tests. The packet is incomplete unless the rerun and findings update explain the
remaining residue honestly and the existing witnesses stay locked without rebaseline.

## Boundaries

Always do:
- keep the packet analyzer-local and precision-first.
- preserve the current `unknowns.is_empty()` eligibility bar.
- preserve the current structural-containment helper semantics.
- require multi-anchor symmetry for any suppressive relation classification.
- keep every new suppressive relation paired with at least one false-negative guard.
- baseline the rerun against the published `R6-3.6` seed-42 sample (`110` sessions / `44` repos / `938`
  checkpoints / `221` adjacent target-eligible pairs / `213` same-target suppressions / `7` remaining
  disjoint pairs / `0` fires); if the exact old manifest is unavailable, say so explicitly and compare
  directionally against those published counts.
- update findings/map/ledger with the routing decision after the 110-session rerun.

Ask first before:
- widening beyond analyzer-local scorer/tests/docs/tooling,
- editing `context/objective.rs` or any checkpoint/export/schema/progress/working-set seam,
- changing export/schema or checkpoint serialization,
- changing sentinel rendering or compactor normalization,
- implementing repo-relative cwd stripping without first proving the current scorer seam can already reach the
  needed cwd/session-root truth analyzer-locally.

Never do in this packet:
- loosen `R6-3.X.3` eligibility as part of the weighted-distance landing,
- broaden raw containment to solve family/progression cases,
- implement full parent/child semantic support,
- treat shared constraints alone as a suppressive relation,
- let `SameArtifactFamily` or `SameWorkItemFamily` suppress on sibling-stem or generic `spec/plan/tasks`
  overlap alone,
- claim success from synthetic fixtures without the 110-session rerun,
- satisfy validation-strata closeout with all-unknown reporting.

## Live Repo Truth Reconciled

The live authority stack agrees on the following starting point:
- `R6-3.5` removed junk-target over-fire.
- `R6-3.X.2` containment first cut removed structural subtree narrowing/broadening over-fire.
- `R6-3.6` proved the detector can still fire (`5/5` positive controls) and still stay quiet on bounded
  non-pivots (`5/5` negative controls).
- the fresh **seed-42** corpus rerun (`110` sessions / `44` repos / `938` checkpoints / `221` adjacent
  target-eligible pairs / `213` same-target exact-match suppressions / `7` remaining disjoint pairs / `0`
  fires) shows the next blocker is **not** eligibility loosening.
- the remaining residue is progression/family/role-shift relation debt: same artifact family,
  same work-item family, doc progression, plan-doc ↔ code role shift, review/findings ↔ fix/verify role shift,
  absolute-vs-repo-relative subtree equivalence, crate/package naming vs package-root paths, and
  shared-constraint masking.
- a live repo search over `crates/agent-drift-analyzer/src/{checkpoint,scoring}` found no scorer-local `cwd`
  or `session_meta` surface today, so repo-relative cwd stripping is **not yet proven reachable** at the
  current scorer seam.

Therefore this packet should not reopen extraction or containment, should not assume cwd-aware equivalence is
available for free, and should add only a bounded relation layer on stable target anchors.

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

Add an explicit relation taxonomy for compared goals. This taxonomy — not any numeric score band — is the
routing authority.

1. `Exact`
   - same stable anchor set after current canonicalization.
2. `StructuralContainment`
   - existing landed helper; no behavior change.
3. `RepoRelativeEquivalentAfterCwdStrip`
   - **deferred / ask-first by default.** Only eligible if the implementation first proves the live scorer seam
     can already read the needed cwd/session-root truth analyzer-locally, with no `context/objective.rs`,
     export/schema, or compactor widening. If that reachability is not proven, this relation stays out of the
     packet and is recorded as deferred rather than guessed.
4. `SameArtifactFamily`
   - same stable artifact lineage within the same workstream, with decisive evidence beyond a shared sibling stem.
   - not enough: same basename fragment alone, same extension alone, or `audit-trio.*`-style sibling overlap with
     no stronger lineage evidence.
5. `SameWorkItemFamily`
   - same numbered/dotted work-item lineage (`R6-3` ↔ `R6-3.5`, `Packet-4` ↔ `Packet-4.2`) with at least one
     matching work-item token that is more specific than generic `spec`/`plan`/`tasks` role words.
   - not enough: generic `spec_plan`, `plan`, `tasks`, or shared repo prefixes without the same lineage token.
6. `SameDocFamily`
   - same documentation subtree or explicit doc-family lineage with decisive family evidence and compatible roles.
7. `PlanCodeRoleShift`
   - a **pairwise** relation only: one compared anchor is plan/spec/task-doc shaped and the other is code/symbol/
     implementation shaped, both inside the same stable workstream.
   - this does **not** require proving a three-step temporal `plan → code → plan` loop; the current vs kickoff or
     current vs previous pair alone must justify it.
8. `ReviewFixVerifyRoleShift`
   - a **pairwise** relation only: one compared anchor is review/findings/issue/verifier shaped and the other is
     implementation or verification shaped, both inside the same stable workstream.
   - this does **not** require proving a longer review-history cycle beyond the current compared pair.
9. `SharedConstraintOnly`
   - overlap exists only in constraint terms; this is **not** suppressive.
10. `WeakOrGenericOnly`
    - overlap exists only in generic or weak markers (including generic `spec`/`plan`/`tasks` tokens,
      broad docs/code buckets, or sibling-stem crumbs); this is **not** suppressive.
11. `Unrelated`
    - stable anchors disagree with no acceptable family/role-shift explanation.
12. `Unknown`
    - not enough stable anchor information to justify either suppression or a confident pivot claim.

Important design rule: only relations `Exact` through `ReviewFixVerifyRoleShift` may suppress drift, and only
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
- repo-relative form **only if** the scorer seam proves cwd/session-root reachability without widening scope
- leaf/basename
- extension / role suffix when applicable
- work-item tokens (e.g. `R6-3.5`, `packet-4`, `plan-04`)
- family tokens with generic/weak pieces removed
- workflow role hints (`plan`, `review`, `verify`, `code`, `examples`, `status`, etc.)

Guardrails:
- derive role/family signals from anchor structure first; do not add a broad prose classifier.
- keep type-aware rules: symbol logic stays case-sensitive; path logic may use cwd stripping only if the seam
  proof exists, and may not use global case folding.
- when no stable anchors survive, preserve the existing conservative behavior and surface `Unknown` or the
  current no-claim path rather than inventing relation confidence.

### 4. Add a weighted assessment shape

The scorer may compute a bounded assessment object for kickoff and rolling comparisons, but the assessment's
`relation` remains the authoritative decision field.

```rust
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
```

Recommended assessment fields:
- `relation`
- `score` (`0..=100`, explanatory only)
- `confidence`
- `decisive_evidence` (which anchors or normalized families caused the chosen relation)
- `counter_evidence` (unmatched anchors, conflicting family hints, missing cwd proof, constraint-only overlap, etc.)

Recommended score bands, if retained at all:
- `0..=10`: exact or containment-equivalent
- `11..=25`: same target family / repo-relative equivalent
- `26..=40`: same work-item/doc/role-shift progression
- `41..=64`: ambiguous / unknown / weak evidence → conservative no-claim or preserve current non-fire posture
- `65..=84`: only weak or constraint overlap; treat as divergent unless stronger relation evidence exists
- `85..=100`: unrelated pivot

The implementation agent may adjust exact numeric values, but the packet must preserve the band meanings and
must document clearly that they are descriptive summaries of the chosen relation, not an independent routing
authority. The scorer must classify the relation first and only then attach any explanatory score/band.

### 5. Multi-anchor decision rule

For goals with multiple stable anchors, relation grading stays symmetric and explanation-complete:
- every anchor on both sides must be matched to an anchor of equal-or-stronger relation for suppressive
  outcomes;
- any unmatched anchor becomes counter-evidence and can escalate the relation to `Unrelated` or `Unknown`;
- one related pair must not mask one unrelated pair.

This extends the current containment symmetry rule rather than replacing it.

### 6. Drift routing policy

After exact-match and containment checks, route by **relation + confidence + decisive/counter evidence**, not by
raw numeric threshold:
- suppress drift for high-confidence `Exact`, `StructuralContainment`, `RepoRelativeEquivalentAfterCwdStrip`
  (only if the seam proof landed), `SameArtifactFamily`, `SameWorkItemFamily`, `SameDocFamily`,
  `PlanCodeRoleShift`, and `ReviewFixVerifyRoleShift`.
- keep or trigger drift for high-confidence `Unrelated`, `SharedConstraintOnly`, and `WeakOrGenericOnly`.
- prefer no-claim over speculative firing for `Unknown`, low-confidence assessments, or any case where the only
  overlap is generic/family residue.

This packet is allowed to change how the analyzer reaches the drift decision, but it must stay conservative:
false positives may be reduced; false negatives must be pinned by explicit controls before any new suppressive
relation ships.

### 7. Required positive controls

Add new positive controls that prove the weighted relation does **not** over-suppress real pivots. At minimum:
1. crate/package pivot
2. workspace-ref pivot
3. verification-target pivot
4. repo/work-item pivot
5. shared-constraint false-negative guard (same platform/scope boundary, unrelated targets, must still fire)
6. sibling-stem pivot guard (`SameArtifactFamily` must not swallow it)
7. generic `spec/plan/tasks` pivot guard (`SameWorkItemFamily` / `WeakOrGenericOnly` must not swallow it)

Each must clear the existing eligibility bar and use stable target anchors.

### 8. Required negative controls

Add new negative controls that prove the weighted relation absorbs legitimate progression. At minimum:
1. absolute-vs-repo-relative subtree progression after cwd stripping **only if seam reachability is proven**;
   otherwise carry this as deferred/ask-first, not silently implemented.
2. same artifact family progression
3. same work-item family progression
4. plan-doc ↔ code role shift within one workstream
5. review/findings ↔ fix/verify role shift within one workstream

These are **in addition to** the already-landed containment negatives and must not replace them.

### 9. Prior-witness non-regression wall

This packet must preserve, not rebaseline, the existing relevant surfaces:
- the `R6-3.6` bounded 10-case acceptance allowlist in
  `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
- the existing `R6-1` dead-end-thrash posture / acceptance witnesses carried forward by the `R6` map
- the existing `R6-2` kickoff-anchor true-positive witnesses
- the `R6-3.X.2` containment negatives (directory/file narrowing, file/directory broadening,
  same-file `:line` handling, same-target review→verify progression)
- the `R5.75-3` delegated-stability / `R5.75-4` zero-verifier anti-flap witnesses called forward by the
  `R6-3` ledger

Any change to those surfaces is a regression unless the packet documents and proves an intentional follow-on
seam, which this packet does not open.

### 10. Generalization guardrails

The scorer must not become a Rust/Codex/spec-doc memorizer. Guardrails:
- prefer anchor-kind and repo-relative structure over hand-curated filename allowlists;
- if a rule depends on a workflow role (`plan`, `review`, `verify`), derive it from a small cross-language
  role vocabulary and anchor placement, not Rust-only path fragments;
- keep an explicit `Unknown` bucket rather than forcing every case into a suppressive family;
- require at least one counter-example test for each new suppressive relation family;
- do not treat any single shared token as enough unless the relation family definition says so and the
  whole anchor set agrees.

### 11. Validation strata

The rerun and findings update must report the packet across these strata when inferable without schema changes:
- **language/repo type:** Rust, JS/TS, Python, docs-only, mixed, unknown
- **workflow type:** implementation, docs/planning, verification, review/fix, mixed, unknown
- **tooling type:** Cargo/Rust, Node/npm, Python/pytest, generic filesystem/doc, unknown
- **delegation topology:** `single_agent`, `delegated_child_visible`, `delegated_parent_opaque`, `unknown`

Closeout rule:
- delegation topology is mandatory because the current harness already derives it.
- any additional stratum reported as `100% unknown` does **not** count as satisfying the gate; it must be called
  out as unresolved or blocked, with the blocker explained.
- best-effort inference is acceptable only when the docs say what evidence it uses; do not hand-wave an
  all-unknown table into success.

### 12. 110-session rerun gate

The packet is not complete without a fresh 110-session corpus rerun using the committed batch harness.
Required closeout items:
- total sessions / repos / checkpoints
- explicit baseline comparison to the published `R6-3.6` seed-42 numbers
- eligible and target-resolved funnel counts
- same-target suppressions
- weighted-relation suppressions by family when derivable
- remaining residue pairs with hand-labeled relation explanations
- strata split from the section above
- explicit yes/no routing decision for `R6-3.X.3`

### 13. Default routing decision

The default closeout decision is:
- **`R6-3.X.3` remains deferred** if the weighted packet lands cleanly and the rerun still shows no evidence
  that the strict eligibility bar is the next dominant blocker.
- reopen `R6-3.X.3` only if, after weighted relation grading is live, the rerun or curated positive
  controls show genuine pivots that remain suppressed solely by the eligibility bar rather than by relation
  grading.

## Acceptance

This packet is complete only when all are true:
1. The scorer has an explicit relation taxonomy covering the required families.
2. That taxonomy — not the numeric score — is the authoritative routing surface.
3. If a numeric score remains, the code/tests/docs mark it explanatory only.
4. Structural containment behavior is preserved, not broadened.
5. `RepoRelativeEquivalentAfterCwdStrip` is either backed by an explicit scorer-seam reachability proof or is
   recorded as deferred / ask-first.
6. Shared-constraint-only overlap no longer suppresses unrelated pivots.
7. Positive controls include crate/package, workspace-ref, verification-target, shared-constraint, sibling-stem,
   and generic-spec/plan/tasks false-negative guards.
8. Negative controls include same artifact family, same work-item family, plan-doc ↔ code role shift, and
   review/findings ↔ fix/verify role shift; cwd-strip negatives land only if the seam proof exists.
9. Every new suppressive relation has at least one paired false-negative guard.
10. The prior witnesses listed above stay locked with no rebaseline.
11. The design documents generalization guardrails and validation-strata limits.
12. The 110-session corpus rerun is seed-pinned or explicitly baseline-compared against `R6-3.6`.
13. Validation strata are not claimed complete via all-unknown reporting.
14. The closeout docs make an explicit `R6-3.X.3` defer/reopen decision.
15. No sentinel, compactor, export/schema, `TaskFrame`, `working_set`, `progress`, or `context/objective.rs`
    change lands in this packet without an explicit ask-first scope change.

## Non-Goals

- loosening the shared eligibility bar
- changing sentinel/operator-surface behavior
- changing compactor normalization
- broadening the raw containment carve-out
- migrating `TaskFrame`, `working_set`, or `progress`
- editing `context/objective.rs` or objective extraction heuristics
- full delegated-session/subagent semantic support
- replacing deterministic scoring with a learned similarity model

## Open Questions

1. Should the weighted score bands be implemented as fixed constants or as a small mapping table on
   `GoalRelation` plus penalties? Recommendation: mapping table plus bounded penalties for unmatched anchors,
   while keeping relation rather than score authoritative.
2. Which validation-strata buckets are truly derivable from current exports without export widening?
   Recommendation: land delegation for sure, then add best-effort inference with honest blockers for any
   stratum that would otherwise be all-unknown.
3. Can cwd-aware repo-relative equivalence be implemented analyzer-locally at the current scorer seam?
   Recommendation: do not assume yes; prove reachability first or defer it.
