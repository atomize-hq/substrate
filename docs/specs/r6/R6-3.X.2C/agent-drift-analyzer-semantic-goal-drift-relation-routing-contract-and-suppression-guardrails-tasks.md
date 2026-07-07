# R6-3.X.2C TASKS — Relation Routing Contract And Suppression Guardrails For Semantic Goal Drift

Status: OPEN (created 2026-07-06 from the July 6 external review follow-up plus live `R6` repo truth).
This packet owns the routing-contract and guardrail remainder left after `R6-3.X.2B`. It does not reopen
eligibility loosening, extraction hardening, containment widening, or downstream sentinel/schema work.

## R6-3.X.2C.0: Docs reopen and live-truth framing

- [ ] Task R6-3.X.2C.0.1: Create the packet-local SPEC / PLAN / TASKS family and lock the live truth.
  - Acceptance:
    - `docs/specs/r6/R6-3.X.2C/` contains packet-local spec, plan, and tasks docs.
    - The docs state explicitly that `R6-3.X.2B` landed useful relation machinery but did not settle the final
      routing contract.
    - The docs state explicitly that the strongest confirmed issue is the relation-only routing mismatch.
    - The docs state explicitly that `R6-3.X.3` remains deferred unless this packet proves otherwise.
  - Verify:
    - manual audit against `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
    - manual audit against `docs/specs/r6/MAP.md`
    - manual audit against `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
  - Dependencies: none
  - Files likely touched:
    - `docs/specs/r6/R6-3.X.2C/agent-drift-analyzer-semantic-goal-drift-relation-routing-contract-and-suppression-guardrails-{spec,plan,tasks}.md`
  - Estimated scope: S

- [ ] Task R6-3.X.2C.0.2: Lock the analyzer-local boundary and explicit defers.
  - Acceptance:
    - The docs fence work to scorer logic, analyzer tests/fixtures, rerun/tooling truth, and routing docs.
    - The docs explicitly defer `context/objective.rs`, sentinel, compactor, `TaskFrame`, `working_set`,
      `progress`, export/schema, repo-relative cwd-strip equivalence, and `R6-3.X.3`.
    - The docs explicitly say this packet adds no new relation families.
  - Verify:
    - manual review of packet-local spec/plan/tasks docs
  - Dependencies: `R6-3.X.2C.0.1`
  - Files likely touched:
    - packet-local spec/plan/tasks docs
  - Estimated scope: XS

- [ ] Task R6-3.X.2C.0.3: Lock preflight requirements for later implementation.
  - Acceptance:
    - The docs require GitNexus impact analysis before editing scorer symbols.
    - The docs require warning on HIGH/CRITICAL impact before proceeding.
    - The docs require `gitnexus detect-changes` before commit.
  - Verify:
    - manual review of packet-local docs
  - Dependencies: `R6-3.X.2C.0.2`
  - Files likely touched:
    - packet-local spec/plan/tasks docs
  - Estimated scope: XS

## R6-3.X.2C.1: Routing contract

- [x] Task R6-3.X.2C.1.1: Define the explicit drift-decision contract.
  - Acceptance:
    - The scorer design names an explicit decision surface such as `Suppress`, `Fire`, and `NoClaim`.
    - The design says relation family selects the candidate routing class and remains the primary routing
      authority.
    - The design says numeric score is explanatory only.
    - The design says confidence plus decisive/counter evidence gate weaker suppressive families.
    - The design says `NoClaim` is distinct from `Suppress`: `NoClaim` means under-supported or ambiguous,
      not positive relatedness.
    - The design locks a route-result matrix covering exact, containment, weak suppressive-family, shared-
      constraint, weak/generic, unrelated, and unknown outcomes.
  - Verify:
    - scorer-local tests for route selection by relation family
    - packet docs reference the same decision contract without drift
  - Dependencies: `R6-3.X.2C.0.3`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - packet-local docs
  - Estimated scope: M

- [x] Task R6-3.X.2C.1.2: Replace `claims_drift()` routing with the explicit decision helper.
  - Acceptance:
    - kickoff-anchor and rolling comparisons no longer route by relation-only `claims_drift()`.
    - `Exact` and `StructuralContainment` suppress directly.
    - `SharedConstraintOnly`, `WeakOrGenericOnly`, and `Unrelated` fire.
    - `Unknown` and under-supported suppressive-family matches no-claim.
    - low-confidence weak suppressive-family candidates never suppress.
    - weak suppressive-family candidates with material counter-evidence never suppress.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Dependencies: `R6-3.X.2C.1.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: M

- [x] Task R6-3.X.2C.1.3: Make the weighted assessment fields load-bearing where intended.
  - Acceptance:
    - callers no longer ignore `confidence`, `decisive_evidence`, and `counter_evidence`.
    - weaker suppressive families require explicit evidence support before suppressing.
    - any non-empty material `counter_evidence` blocks weak suppression in this packet.
    - under-supported `SameArtifactFamily` or `SameWorkItemFamily` candidates are reclassified to `Fire`-
      aligned relations where appropriate instead of automatically becoming `NoClaim`.
    - score remains non-authoritative in code comments, tests, and docs.
  - Verify:
    - scorer-local tests that distinguish supported vs unsupported suppressive-family matches
  - Dependencies: `R6-3.X.2C.1.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - packet-local docs
  - Estimated scope: M

## R6-3.X.2C.2: Suppressive-family guardrails

- [x] Task R6-3.X.2C.2.1: Tighten `SameArtifactFamily` so same-crate residue alone cannot suppress.
  - Acceptance:
    - shared crate/package tokens or sibling-stem crumbs alone do not justify suppression.
    - the scorer requires distinctive non-generic lineage evidence or another explicit continuity signal.
    - `SameArtifactFamily` is not used as a fallback bucket when the evidence is only broad residue.
    - at least one same-crate unrelated pivot still fires.
  - Verify:
    - scorer-local regression tests for same-crate unrelated pivots
  - Dependencies: `R6-3.X.2C.1.3`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: M

- [x] Task R6-3.X.2C.2.2: Tighten `SameWorkItemFamily` so broad lineage alone cannot suppress.
  - Acceptance:
    - shared prefixes like `R6-3` do not suppress by themselves.
    - the scorer requires lineage plus at least one stronger continuity signal.
    - `SameWorkItemFamily` is not used as a fallback bucket when the evidence is only broad lineage residue.
    - at least one same-lineage unrelated docs/workstream pivot still fires.
  - Verify:
    - scorer-local regression tests for same-lineage unrelated pivots
  - Dependencies: `R6-3.X.2C.2.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: M

- [x] Task R6-3.X.2C.2.3: Expand doc-bundle-member behavior into explicit positive and negative coverage.
  - Acceptance:
    - bundle → member suppresses when it is clearly the same doc family.
    - member → bundle suppresses when it is clearly the same doc family.
    - bundle/member with an unrelated addition still fires.
    - bundle → unrelated doc still fires.
    - at least one live acceptance fixture covers doc-bundle behavior, not only scorer-local tests.
    - if feasible in the same bounded fixture style, one acceptance-level false-negative guard covers a
      high-risk `SameArtifactFamily` or `SameWorkItemFamily` case.
  - Verify:
    - scorer-local regression tests for doc-bundle positive/negative cases
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
  - Dependencies: `R6-3.X.2C.2.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - Estimated scope: M

- [x] Task R6-3.X.2C.2.4: Reorder role classification so code/test markers beat substring doc hints.
  - Acceptance:
    - obvious code/test/verifier paths are classified by path/extension markers before substring semantics.
    - paths like `src/spec_parser.rs` and `src/design_tokens.rs` no longer misclassify as doc roles.
    - existing plan/review/verify progression cases stay covered.
  - Verify:
    - scorer-local role-classification regression tests
  - Dependencies: `R6-3.X.2C.2.3`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: S

## R6-3.X.2C.3: Proof wall and non-regression

- [x] Task R6-3.X.2C.3.1: Pair every touched suppressive family with an explicit false-negative guard.
  - Acceptance:
    - same-artifact, same-work-item, same-doc/doc-bundle, and touched role-shift paths each have at least one
      counter-example proving an unrelated pivot still fires.
    - weak/generic overlap is never promoted into suppression to make a test pass.
    - route-result coverage includes at minimum:
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
  - Verify:
    - scorer-local tests
    - acceptance assertions where the behavior is live-analyzer-visible
  - Dependencies: `R6-3.X.2C.2.4`
  - Files likely touched:
    - scorer tests
    - acceptance harness/fixtures as needed
  - Estimated scope: M

- [x] Task R6-3.X.2C.3.2: Preserve prior witnesses without rebaseline.
  - Acceptance:
    - the existing `R6-3.6` bounded acceptance allowlist remains intact except for explicit additive cases.
    - existing `R6-1`, `R6-2`, `R6-3`, and `R6-3.X.2B` witnesses remain intact.
    - any changed witness is treated as regression, not silent rebaseline.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Dependencies: `R6-3.X.2C.3.1`
  - Files likely touched:
    - acceptance harness/fixtures
    - scorer tests as needed
  - Estimated scope: M

## R6-3.X.2C.4: Rerun and routing-doc reconciliation

- [x] Task R6-3.X.2C.4.1: Run the focused wall and full analyzer wall after the routing fix.
  - Acceptance:
    - focused semantic-goal-drift tests pass.
    - full analyzer wall passes.
    - any skipped verification is documented honestly.
  - Verify:
    - the focused and full verification wall from the packet spec
  - Dependencies: `R6-3.X.2C.3.2`
  - Files likely touched:
    - docs only unless a tiny test/fixture adjustment is required during verification
  - Estimated scope: S

- [x] Task R6-3.X.2C.4.2: Rerun the seed-42 corpus by default and record the residue honestly.
  - Acceptance:
    - the seed-42 110-session corpus is rerun with the committed harness unless explicitly waived.
    - the docs compare results directionally against the current published `R6-3.X.2B` closeout numbers.
    - any changed residue family is named explicitly.
    - if the corpus rerun is not done, the docs explain exactly why and what confidence is lost.
  - Verify:
    - the default batch command sequence from the packet spec, including `filter_junk.py`
  - Dependencies: `R6-3.X.2C.4.1`
  - Files likely touched:
    - docs only unless a tiny reporting fix is required
  - Estimated scope: M

- [x] Task R6-3.X.2C.4.3: Reconcile `FINDINGS`, `MAP`, and the `R6-3` ledger with the new routing truth.
  - Acceptance:
    - closeout docs no longer present `R6-3.X.2B` as the final routing-contract closeout.
    - the docs say relation type is primary, score explanatory only, and confidence/evidence gate weaker
      suppressive families.
    - the docs record whether the seed-42 rerun was rerun or intentionally skipped.
    - the docs soften unknown-strata wording if it is still only a coarse sanity check.
    - the docs keep `R6-3.X.3` deferred unless this packet proves a real need to reopen it.
  - Verify:
    - manual audit against final code/test/rerun outputs
  - Dependencies: `R6-3.X.2C.4.2`
  - Files likely touched:
    - `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
    - `docs/specs/r6/MAP.md`
    - `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
  - Estimated scope: S

## Final checkpoint

Before implementation is considered packet-complete:
- [ ] the routing contract is explicit and relation-authoritative
- [ ] `claims_drift()` is no longer the operative routing surface
- [ ] numeric score is explanatory only
- [ ] weaker suppressive families are gated by confidence/evidence
- [ ] same-artifact and same-work-item false-negative guards are explicit
- [ ] doc-bundle-member behavior has positive and negative scorer-local plus acceptance coverage
- [ ] role classification ordering is tightened
- [ ] prior witnesses stay locked with no rebaseline
- [ ] focused tests and the full analyzer wall pass
- [ ] the default corpus rerun is completed or explicitly waived with an honest confidence-loss note
- [ ] `FINDINGS`, `MAP`, and the `R6-3` ledger no longer overstate `R6-3.X.2B` as the final routing-contract closeout
- [ ] `R6-3.X.3` remains deferred unless new evidence truly proves otherwise
- [ ] repo-relative cwd-strip equivalence remains deferred unless explicitly reopened
