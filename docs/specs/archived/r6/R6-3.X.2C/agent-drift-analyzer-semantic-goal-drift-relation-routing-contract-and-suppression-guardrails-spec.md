# R6-3.X.2C SPEC — Relation Routing Contract And Suppression Guardrails For Semantic Goal Drift

Status: OPEN (created 2026-07-06 from the July 6 external review follow-up plus live `R6` repo truth).
Packet-scoped to analyzer-local semantic-goal-drift scorer routing, suppressive-family guardrails,
acceptance coverage, rerun proof, and routing-doc reconciliation.

Assumptions carried into this spec:
1. `R6-3.5` target hygiene remains landed and authoritative.
2. The `R6-3.X.2` containment first cut remains landed and must stay behaviorally intact.
3. `R6-3.X.2B` landed useful analyzer-local relation machinery, but the strongest confirmed issue is that
   the scorer still routes by relation type alone while the docs/spec describe a richer
   relation + confidence + evidence contract.
4. The remaining review findings are tracked here as routing-contract, false-negative-risk, and coverage
   follow-ups unless later rerun evidence proves a live regression.
5. This packet stays analyzer-local: scorer logic, analyzer tests/fixtures, rerun tooling/docs, and
   findings/map/ledger truth only.
6. `context/objective.rs`, sentinel, compactor, `TaskFrame`, `working_set`, `progress`, and
   export/schema seams remain out of scope unless explicitly reopened.

Authority / cross-references:
- `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
- `docs/specs/r6/MAP.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
- `docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-{spec,plan,tasks}.md`
- live scorer/tests/tooling:
  - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - `scripts/dev/drift-batch-scan/**`
- `AGENTS.md`

## Objective

Reopen the semantic-goal-drift weighted-distance closeout just enough to make the routing contract coherent,
testable, and honestly documented.

Success means this packet:
- replaces relation-only `claims_drift()` routing with an explicit drift-decision contract for scorer-local routing/audit semantics; at today's public `semantic_goal_drift` emission boundary, only `Fire` emits while both `Suppress` and `NoClaim` remain non-fire,
- keeps relation type as the primary routing authority,
- uses confidence plus decisive/counter evidence as suppressive gates for weaker relation families,
- preserves numeric score as explanatory/audit-only,
- adds explicit false-negative guards for suppressive families,
- expands doc-bundle-member coverage to both scorer-local and live-analyzer acceptance proof,
- reruns the focused wall and, by default, the seed-42 corpus before closing unless explicitly waived with
  an honest confidence-loss note,
- reopens `FINDINGS` / `MAP` / `R6-3` ledger truth so `R6-3.X.2B` is recorded as landed but not the final
  routing-contract closeout, and `R6-3.X.2C` becomes the remaining closeout packet,
- keeps `R6-3.X.3` deferred unless this packet's rerun proves otherwise.

## Commands

Read-only grounding commands used for this packet:

```bash
sed -n '1,260p' docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-spec.md
sed -n '1,260p' docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-plan.md
sed -n '1,360p' docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-tasks.md
sed -n '420,520p' docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
sed -n '1,320p' docs/specs/r6/MAP.md
sed -n '300,420p' docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md
sed -n '1,620p' crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
sed -n '1,260p' crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs
```

Implementation verification commands this packet must carry:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo build -p agent-session-compactor -p agent-drift-analyzer
python3 scripts/dev/drift-batch-scan/sample_sessions.py --seed 42 --out /tmp/r6_3_x_2c_selected.jsonl
python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2c_selected.jsonl --batch-dir /tmp/r6_3_x_2c_batch
python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2c_batch/checkpoints
python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2c_batch/checkpoints
python3 scripts/dev/drift-batch-scan/filter_junk.py --checkpoints-dir /tmp/r6_3_x_2c_batch/checkpoints
```

Implementation preflight when code work begins:
- before editing any scorer symbol, run GitNexus impact analysis on that symbol and record the blast radius;
- warn before proceeding if impact is HIGH or CRITICAL;
- run `gitnexus detect-changes` (or the repo-qualified equivalent) before commit.

## Project Structure

Primary packet surfaces:
- `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - `WeightedRelationAssessment`
  - routing contract (`claims_drift()` replacement)
  - suppressive-family gating
  - doc-bundle-member handling
  - role-classification ordering
- `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - acceptance allowlist growth
  - new positive and negative doc-bundle / family / role-classifier controls
- `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - bounded new acceptance fixtures for doc-bundle and false-negative guards
- `docs/specs/r6/**`
  - packet-local spec/plan/tasks
  - `FINDINGS`, `MAP`, and `R6-3` ledger reopen
- `scripts/dev/drift-batch-scan/**`
  - rerun only if doc wording or reporting truth needs adjustment after the scorer change

Secondary scope only if the smallest analyzer-local change needs it:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/support/mod.rs`

Out of scope unless explicitly reopened:
- `crates/agent-drift-analyzer/src/context/objective.rs`
- `crates/agent-drift-analyzer/src/checkpoint/{progress.rs,working_set.rs,export.rs,schema.rs}`
- `crates/agent-drift-sentinel/**`
- `crates/agent-session-compactor/**`

## Code Style

Implementation constraints:
- stay deterministic and analyzer-local;
- keep relation type as the **primary routing authority**;
- keep numeric score explanatory only;
- route weak suppressive families through explicit confidence/evidence gates rather than numeric thresholds;
- preserve multi-anchor symmetry and explicit counter-evidence accounting;
- keep structural-containment logic frozen; add new logic after containment, not inside it;
- prefer small explicit enums/helpers over fuzzy score-band branching;
- classify code/test/verifier anchors from path structure before substring doc/workflow hints;
- use the live `GoalRelation` / `WeightedRelationAssessment` names where possible; do not create a parallel
  relation enum or duplicate routing path unless review proves the live type cannot carry the contract;
- do not reopen extraction or checkpoint seams under the guise of routing cleanup.

Illustrative contract only:

```rust
enum DriftDecision {
    Suppress,
    Fire,
    NoClaim,
}

impl WeightedRelationAssessment {
    fn drift_decision(&self) -> DriftDecision { /* relation-authoritative */ }
}
```

## Testing Strategy

This packet needs five proof layers:
1. scorer-local routing tests proving `drift_decision()` behavior by relation family;
2. scorer-local false-negative guards for every suppressive family touched here;
3. acceptance-fixture proof through the live analyzer checkpoint path, including explicit doc-bundle cases;
4. prior-witness non-regression across the existing `R6-1` / `R6-2` / `R6-3` / `R6-3.6` / `R6-3.X.2B`
   surfaces;
5. focused rerun proof, including the seed-42 110-session corpus again by default unless explicitly waived.

Acceptance is not only green unit tests. The packet is incomplete unless the routing docs are reconciled and
the rerun truth is recorded honestly.

## Boundaries

Always do:
- preserve the current `unknowns.is_empty()` eligibility bar;
- preserve the current structural-containment carve-out;
- use relation-authoritative routing with explicit `Suppress` / `Fire` / `NoClaim` outcomes;
- require confidence plus decisive/counter evidence gates for weaker suppressive relations;
- keep numeric score explanatory only;
- add explicit doc-bundle-member positive and negative coverage;
- rerun focused scorer and acceptance walls after the routing change;
- reopen `FINDINGS`, `MAP`, and the `R6-3` ledger so the packet status is honest.

Ask first before:
- widening beyond analyzer-local scorer/tests/docs/tooling,
- changing export/schema/checkpoint/session surfaces,
- reopening extraction or eligibility logic,
- adding new relation families rather than tightening existing ones,
- implementing repo-relative cwd-strip equivalence without fresh scorer-local seam proof.

Never do in this packet:
- loosen `R6-3.X.3` eligibility,
- broaden containment to solve family/progression residue,
- migrate sentinel/compactor/export/schema surfaces,
- claim score thresholds are the routing authority,
- keep docs saying relation + confidence + evidence already route decisions unless the code truly does,
- treat the unknown-strata wording issue as fully resolved if tooling still supports only a coarse sanity
  check.

## Live Repo Truth Reconciled

The live authority stack currently says:
- `R6-3.5` target hygiene is landed.
- `R6-3.X.2` containment first cut is landed.
- `R6-3.6` positive controls and corpus revalidation are landed.
- `R6-3.X.2B` added relation taxonomy, suppressive families, batch strata reporting, and a doc-bundle-member
  scorer fix, and the current closeout docs present that packet as fully closed.
- the strongest confirmed issue from the July 6 external review is that the implementation still routes
  `semantic_goal_drift` by relation type alone even though the docs/spec describe richer routing.
- the remaining review findings are tracked as suppressive-family risk/coverage follow-ups unless rerun
  evidence proves a live regression.

Therefore this packet exists to reconcile the implementation contract and close the remaining routing gap.

## Design

### 1. Preserve the current gate, target hygiene, and containment cut

This packet starts after the current eligibility bar, exact-match checks, and structural-containment checks.
It must not change:
- `eligible_current_goal(...)`
- `eligible_anchor_goal(...)`
- `eligible_previous_goal(...)`
- `structured_matches_*_bar(...)`
- `goals_in_structural_containment(...)`
- the current case-sensitive path/symbol protections

### 2. Replace relation-only routing with an explicit decision contract

The scorer should use an explicit decision contract rather than a relation-only boolean drift check.

Required shape:
- add an explicit `DriftDecision`-style outcome (`Suppress`, `Fire`, `NoClaim`);
- replace `claims_drift()` calls in kickoff and rolling paths with that decision contract;
- keep relation family authoritative as the candidate routing class;
- keep numeric score explanatory only.

Routing contract:
- `Exact` and `StructuralContainment` suppress directly at high confidence.
- `SameArtifactFamily`, `SameWorkItemFamily`, `SameDocFamily`, `PlanCodeRoleShift`, and
  `ReviewFixVerifyRoleShift` suppress only when:
  - confidence is at least `Medium`,
  - decisive evidence is non-empty,
  - counter-evidence is empty; any non-empty material `counter_evidence` blocks weak suppression in this
    packet,
  - every compared anchor is explained by the chosen relation or a stronger one.
- `SharedConstraintOnly`, `WeakOrGenericOnly`, and `Unrelated` fire.
- `Unknown`, low-confidence relation results, or mixed/counter-evidence-heavy family cases no-claim.

Operationally:
- relation family selects the candidate routing class;
- confidence plus decisive/counter evidence determine whether a weak suppressive candidate may suppress or
  must fall back to `NoClaim`;
- numeric score never determines suppress vs fire vs no-claim.

`NoClaim` means the scorer is refusing to make a `semantic_goal_drift` assertion because the evidence is
under-supported or ambiguous. It is not positive relatedness. `NoClaim` must be distinguishable in scorer
tests, debug comments, and rerun/funnel reporting from `Suppress`. `Suppress` means "related enough to stay
quiet." `NoClaim` means "not enough evidence to make a drift claim." At today's public
`semantic_goal_drift` emission boundary, both `Suppress` and `NoClaim` are non-fire; `NoClaim` is therefore an
internal abstention/audit state unless a later packet exports it separately.

### 3. Keep the weighted apparatus, but make it coherent

`WeightedRelationAssessment` may keep:
- `relation`
- `score`
- `confidence`
- `decisive_evidence`
- `counter_evidence`

But the packet must make these semantics true in code and docs:
- relation family is the routing backbone and selects the candidate routing class;
- confidence/evidence gate weaker suppressive families;
- score is explanatory only;
- callers no longer ignore the confidence/evidence fields.

If a weak suppressive-family candidate fails its evidence gate, the implementation must first ask whether the
relation was over-classified. Cases resting only on broad lineage, same crate/package residue, generic
artifact tokens, shared constraints, or weak/generic terms should be reclassified to `Unrelated`,
`SharedConstraintOnly`, or `WeakOrGenericOnly` as appropriate rather than automatically falling into
`NoClaim`.

### 4. Tighten suppressive-family guardrails

#### 4.1 `SameArtifactFamily`
- must not suppress on shared crate-name residue or sibling-stem crumbs alone;
- require distinctive non-generic lineage evidence, compatible artifact shape, or another explicit continuity
  signal;
- `SameArtifactFamily` is not a fallback bucket; when the continuity evidence is not distinctive enough, the
  pair should be reclassified to `Unrelated`, `SharedConstraintOnly`, or `WeakOrGenericOnly` as appropriate;
- add false-negative guards for same-crate but unrelated pivots.

#### 4.2 `SameWorkItemFamily`
- lineage alone is insufficient when the only overlap is a broad prefix like `R6-3`;
- require lineage plus at least one stronger continuity signal such as shared doc family, shared parent path,
  shared non-generic target token, or a recognized role-shift pairing;
- `SameWorkItemFamily` is not a fallback bucket; when the continuity evidence is not distinctive enough, the
  pair should be reclassified to `Unrelated`, `SharedConstraintOnly`, or `WeakOrGenericOnly` as appropriate;
- add false-negative guards for same-lineage unrelated docs/workstreams.

#### 4.3 `SameDocFamily` / doc-bundle-member handling
- keep the landed bundle/member narrowing fix,
- add explicit coverage for:
  - bundle → member suppression,
  - member → bundle suppression,
  - bundle/member + unrelated addition still firing,
  - bundle → unrelated member firing,
- carry at least one live acceptance case, not only scorer-local coverage.
- treat the exact rerun-observed root-level doc-bundle → anchored-member residue as closeout evidence, not
  as a separately committed acceptance witness already locked by this packet.
- if feasible in the same bounded fixture style, add one acceptance-level false-negative guard for a
  high-risk `SameArtifactFamily` or `SameWorkItemFamily` case; scorer-local tests remain the main wall for
  the rest.

#### 4.4 Role classification ordering
- detect code/test/verifier anchors from extension/path markers before substring workflow hints;
- only then apply `plan` / `spec` / `design` / `review` / `findings` semantics;
- add scorer-local guards so obvious code paths like `src/spec_parser.rs` and `src/design_tokens.rs` do not
  misclassify as doc roles.
- keep any broader non-`src` substring-collision risk out of the closeout claim set unless separately proved;
  this packet's role-ordering witnesses are strongest on the touched `src/...` / test / verifier-style cases.

### 5. Keep false-negative guards first-class

Every suppressive family touched here must have at least one explicit counter-example proving a real pivot
still fires. At minimum:
- same-crate unrelated code pivot,
- same-lineage unrelated docs pivot,
- doc-bundle-member with unrelated addition,
- role-shift false positives blocked by code-path role ordering.

Required route-result matrix:
- `Exact` + High confidence + evidence => `Suppress`
- `StructuralContainment` + High confidence + evidence => `Suppress`
- `SameArtifactFamily` + Medium confidence + decisive evidence + no counter-evidence => `Suppress`
- `SameArtifactFamily` + Low confidence => `NoClaim` or reclassified `Fire` if unrelated
- `SameArtifactFamily` + counter-evidence => `NoClaim` or `Fire`, never `Suppress`
- `SameWorkItemFamily` + lineage only => `Fire` or `NoClaim`, never `Suppress`
- `SameWorkItemFamily` + lineage + stronger continuity => `Suppress`
- `SharedConstraintOnly` => `Fire`
- `WeakOrGenericOnly` => `Fire`
- `Unrelated` => `Fire`
- `Unknown` => `NoClaim`

### 6. Reconcile routing docs and reporting language

The packet must update closeout docs so they say what the code really does.

Required wording after implementation:
- relation type is the primary routing authority;
- numeric score is explanatory/audit-only;
- confidence and decisive/counter evidence gate weaker suppressive relations;
- `Unknown` and under-supported relations no-claim rather than suppress;
- validation-strata all-unknown handling is a coarse sanity check unless and until tooling proves more.

### 7. Rerun gate and packet closeout

This packet is not complete without:
- focused scorer tests,
- acceptance rerun,
- full analyzer wall,
- the seed-42 corpus rerun again by default, unless explicitly waived with an honest confidence-loss note,
- `FINDINGS` / `MAP` / `R6-3` ledger updates that:
  - record `R6-3.X.2B` as landed but not the final routing-contract closeout,
  - record `R6-3.X.2C` as the routing-contract and guardrails follow-up,
  - keep `R6-3.X.3` deferred unless new evidence proves otherwise.

## Acceptance

This packet is complete only when all are true:
1. Kickoff and rolling paths route through an explicit decision contract, not `claims_drift()`.
2. Relation family remains the primary routing authority.
3. Numeric score remains explanatory only in code, tests, and docs.
4. Weaker suppressive families are gated by confidence and decisive/counter evidence.
5. `SharedConstraintOnly`, `WeakOrGenericOnly`, and `Unrelated` still fire.
6. `Unknown` and under-supported suppressive-family matches no-claim rather than suppress, and `NoClaim`
   remains distinguishable from `Suppress` in tests/comments/reporting.
7. `SameArtifactFamily` has explicit false-negative guards against same-crate unrelated pivots.
8. `SameWorkItemFamily` has explicit false-negative guards against same-lineage unrelated pivots.
9. Doc-bundle-member logic has scorer-local and acceptance-level positive and negative coverage for clear
   same-doc-family bundle/member behavior, while the exact root-level bundle → member residue remains a
   rerun-observed conservative family rather than a separately committed witness.
10. Role classification no longer misclassifies obvious code/test paths due to substring hits.
11. Prior witnesses remain locked with no silent rebaseline.
12. Focused tests and the full analyzer wall pass.
13. The rerun truth is recorded honestly; if the seed-42 corpus rerun is skipped, the docs say exactly why
    and what confidence is lost.
14. `FINDINGS`, `MAP`, and the `R6-3` ledger no longer overstate `R6-3.X.2B` as the final
    routing-contract closeout.
15. `R6-3.X.3` remains deferred unless the new evidence clearly proves it should reopen.

## Non-Goals

- loosening the shared eligibility bar
- adding new relation families beyond the current taxonomy
- reopening extraction hardening
- sentinel/operator-surface changes
- compactor normalization changes
- export/schema/checkpoint seam widening
- repo-relative cwd-strip equivalence without new scorer-local seam proof

## Locked Decisions

1. The seed-42 corpus rerun is default-required for this packet. It may be waived only with an explicit
   confidence-loss note in `FINDINGS`, `MAP`, and the `R6-3` ledger.
2. No suppressive family except `Exact` and `StructuralContainment` may suppress at Low confidence.
   Low-confidence `SameArtifactFamily`, `SameWorkItemFamily`, `SameDocFamily`, `PlanCodeRoleShift`, and
   `ReviewFixVerifyRoleShift` must resolve to `NoClaim` or be reclassified as `Unrelated`,
   `SharedConstraintOnly`, or `WeakOrGenericOnly` when evidence is inadequate.
