# R6-3.X.2B TASKS — Graduated / Weighted Semantic Distance For Semantic Goal Drift

Status: OPEN (created 2026-07-06 from the next-packet planning request plus live `R6` repo truth).
This packet owns the **remaining** weighted-distance remainder inside the analyzer scorer. It does not reopen
extraction hardening, the containment first cut, or `R6-3.X.3` eligibility loosening.

## R6-3.X.2B.0: Docs lock and live-truth framing

- [ ] Task R6-3.X.2B.0.1: Lock the packet docs against live repo truth.
  - Acceptance:
    - `docs/specs/r6/R6-3.X.2B/` contains packet-local spec, plan, and tasks docs.
    - The docs state explicitly that `R6-3.5` and the containment first cut are already landed.
    - The docs state explicitly that the next blocker is weighted relation grading, not eligibility loosening.
  - Verify:
    - manual audit against `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
    - manual audit against `docs/specs/r6/MAP.md`
    - manual audit against `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
  - Dependencies: none
  - Files likely touched:
    - `docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-spec.md`
    - `docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-plan.md`
    - `docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-tasks.md`
  - Estimated scope: S

- [ ] Task R6-3.X.2B.0.2: Record the implementation boundary.
  - Acceptance:
    - The docs lock the analyzer-local boundary: scorer, analyzer tests/fixtures, batch tooling, findings/map/ledger.
    - The docs explicitly exclude sentinel, compactor, `TaskFrame`, `working_set`, `progress`, and full subagent semantics.
    - The docs define the default `R6-3.X.3` defer rule.
  - Verify: manual review of the packet spec boundaries and non-goals.
  - Dependencies: `R6-3.X.2B.0.1`
  - Files likely touched:
    - packet-local spec/plan/tasks docs
  - Estimated scope: XS

## R6-3.X.2B.1: Stable-anchor relation model

- [ ] Task R6-3.X.2B.1.1: Define the stable-anchor comparison record and ordered relation taxonomy.
  - Acceptance:
    - The scorer design names the relation taxonomy: exact, structural containment, repo-relative after cwd stripping,
      same artifact family, same work-item family, same doc family, plan-code-plan cycle, review-fix-verify cycle,
      shared-constraint-only, weak/generic-only, unrelated, unknown.
    - The taxonomy states which relations may suppress and which may not.
    - The design preserves multi-anchor symmetry.
  - Verify:
    - scorer-local unit tests for relation precedence
    - spec/plan/tasks docs reference the same taxonomy names without drift
  - Dependencies: `R6-3.X.2B.0.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - packet-local docs
  - Estimated scope: M

- [ ] Task R6-3.X.2B.1.2: Add repo-relative normalization after cwd stripping without widening containment.
  - Acceptance:
    - Absolute-vs-repo-relative spellings of the same subtree can be classified as related after safe cwd stripping.
    - The existing raw containment helper remains unchanged except for calling the new later-stage relation path.
    - Case-sensitive symbol/path protections remain intact.
  - Verify:
    - scorer-local regression tests for absolute-vs-repo-relative same-subtree progression
    - regression test proving `docs/specs/r6-map` vs `docs/specs/r6/map.md` is still not containment
  - Dependencies: `R6-3.X.2B.1.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - fixture directories as needed
  - Estimated scope: M

## R6-3.X.2B.2: Weighted assessment and drift routing

- [ ] Task R6-3.X.2B.2.1: Add the weighted relation assessment shape.
  - Acceptance:
    - The scorer has an analyzer-local assessment object with relation, score, confidence, decisive evidence,
      and counter-evidence.
    - Score-band meanings are documented and tested.
    - Kickoff and rolling comparisons use the same assessment surface after exact/containment checks.
  - Verify:
    - scorer-local tests for score-band routing
    - focused semantic-goal-drift test target
  - Dependencies: `R6-3.X.2B.1.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: M

- [ ] Task R6-3.X.2B.2.2: Route shared-constraint-only and weak/generic-only cases correctly.
  - Acceptance:
    - shared-constraint-only overlap no longer suppresses an unrelated pivot.
    - weak/generic-only overlap no longer suppresses an unrelated pivot.
    - ambiguous/unknown middle-band cases prefer conservative no-claim over speculative suppression.
  - Verify:
    - scorer-local tests covering shared-constraint and weak/generic-only paths
    - acceptance-level positive control for shared-constraint false-negative guard
  - Dependencies: `R6-3.X.2B.2.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - Estimated scope: M

## R6-3.X.2B.3: Proof wall expansion

- [ ] Task R6-3.X.2B.3.1: Add the required positive controls.
  - Acceptance:
    - the acceptance corpus includes at least the following new true-positive families:
      - crate/package pivot
      - workspace-ref pivot
      - verification-target pivot
      - repo/work-item pivot
      - shared-constraint false-negative guard
    - every new positive control clears the current eligibility bar and uses stable target anchors.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Dependencies: `R6-3.X.2B.2.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - Estimated scope: M

- [ ] Task R6-3.X.2B.3.2: Add the required negative controls.
  - Acceptance:
    - the acceptance corpus includes at least the following legitimate non-pivot families:
      - absolute-vs-repo-relative subtree
      - same artifact family
      - same work-item family
      - plan→code→plan
      - review→fix→verify
    - existing containment negatives remain intact.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - focused scorer-local regressions for each suppressive relation family
  - Dependencies: `R6-3.X.2B.3.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: M

- [ ] Task R6-3.X.2B.3.3: Pair every new suppressive relation with at least one false-negative guard.
  - Acceptance:
    - same-artifact-family, same-work-item-family, same-doc-family, plan-code-plan, and review-fix-verify each have
      at least one counter-example proving an unrelated pivot still fires.
    - the packet docs call out any residual over-fire or undecidable relation honestly instead of silently widening.
  - Verify:
    - scorer-local tests
    - acceptance corpus assertions
  - Dependencies: `R6-3.X.2B.3.2`
  - Files likely touched:
    - scorer tests and acceptance fixtures
    - packet docs if residual limits remain
  - Estimated scope: M

## R6-3.X.2B.4: Corpus tooling, rerun, and findings update

- [ ] Task R6-3.X.2B.4.1: Add validation strata reporting when inferable.
  - Acceptance:
    - batch tooling reports best-effort splits for language/repo type, workflow type, tooling type, and delegation topology.
    - any non-derivable category is documented honestly with an `unknown` bucket or a stated blocker.
  - Verify:
    - `python3 scripts/dev/drift-batch-scan/tabulate.py --help`
    - sample or real run output inspection
  - Dependencies: `R6-3.X.2B.3.3`
  - Files likely touched:
    - `scripts/dev/drift-batch-scan/README.md`
    - `scripts/dev/drift-batch-scan/tabulate.py`
    - `scripts/dev/drift-batch-scan/run_batch.py` (only if the smallest tag addition is required)
    - `scripts/dev/drift-batch-scan/inspect_targets.py` (only if residue labeling needs it)
  - Estimated scope: S

- [ ] Task R6-3.X.2B.4.2: Run the focused wall, full analyzer wall, and 110-session corpus rerun.
  - Acceptance:
    - focused semantic-goal-drift tests pass
    - full analyzer wall passes
    - the 110-session rerun completes or the blocker is explicit and actionable
    - the remaining disjoint residue is labeled by relation family
  - Verify:
    - the full verification wall from the packet spec
    - the full batch command sequence from the packet spec
  - Dependencies: `R6-3.X.2B.4.1`
  - Files likely touched:
    - docs only unless a tiny tooling/report fix is required during rerun
  - Estimated scope: M

- [ ] Task R6-3.X.2B.4.3: Update findings, map, and routing decision.
  - Acceptance:
    - `FINDINGS`, `MAP`, and the `R6-3` task ledger record:
      - relation families landed
      - positive/negative control results
      - rerun totals and strata
      - remaining residue
      - final yes/no decision on whether `R6-3.X.3` stays deferred
    - default expected decision remains deferred unless the rerun proves otherwise.
  - Verify: manual audit against the final test and rerun outputs.
  - Dependencies: `R6-3.X.2B.4.2`
  - Files likely touched:
    - `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
    - `docs/specs/r6/MAP.md`
    - `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
  - Estimated scope: S

## Final checkpoint

Before implementation is considered packet-complete:
- [ ] the weighted relation taxonomy is explicit and tested
- [ ] the assessment shape is explicit and tested
- [ ] shared-constraint-only no longer masks real pivots
- [ ] new progression families are absorbed without widening containment
- [ ] the 110-session rerun is complete or blocked honestly
- [ ] closeout docs make an explicit `R6-3.X.3` defer/reopen decision
