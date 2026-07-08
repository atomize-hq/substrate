# R6-3.X.2D TASKS — Decision Semantics And Residue Witness Closeout For Semantic Goal Drift

Status: OPEN (created 2026-07-08 as a docs-only packet draft after `R6-3.X.2C`).
This packet owns the narrow follow-up seam around internal/public decision semantics, the named
root-level doc-bundle residue witness, non-`src` code-path role proof, and continuity-threshold edge
coverage. It does **not** reopen eligibility loosening, add relation families, broadly suppress doc
bundles, or expand into sentinel/compactor/schema/delegation work.

## R6-3.X.2D.0: Docs lock and scope fence

- [x] Task R6-3.X.2D.0.1: Create the packet-local SPEC / PLAN / TASKS family.
  - Acceptance:
    - `docs/specs/r6/R6-3.X.2D/` contains packet-local spec, plan, and tasks docs.
    - The docs state explicitly that `2D` is a bounded follow-up after `R6-3.X.2C`.
    - The docs state explicitly that `R6-3.X.3` remains deferred.
  - Verify:
    - manual audit of packet-local docs
  - Dependencies: none
  - Files likely touched:
    - `docs/specs/r6/R6-3.X.2D/agent-drift-analyzer-semantic-goal-drift-decision-semantics-and-residue-witness-closeout-spec.md`
    - `docs/specs/r6/R6-3.X.2D/agent-drift-analyzer-semantic-goal-drift-decision-semantics-and-residue-witness-closeout-plan.md`
    - `docs/specs/r6/R6-3.X.2D/agent-drift-analyzer-semantic-goal-drift-decision-semantics-and-residue-witness-closeout-tasks.md`
  - Estimated scope: S
  - Result (2026-07-08): packet-local SPEC / PLAN / TASKS now exist and lock `2D` as a narrow
    follow-up after `R6-3.X.2C`, not a reopening of `R6-3.X.3`.

- [x] Task R6-3.X.2D.0.2: Record the write-set and non-goal fence.
  - Acceptance:
    - The docs state explicitly that this draft does not edit `MAP`, `FINDINGS`, or the parent `R6-3`
      ledger.
    - The docs state explicitly that eligibility loosening, new relation families, broad doc-bundle
      suppression, repo-relative cwd-strip equivalence, and sentinel/compactor/schema/delegation
      expansion are out of scope.
  - Verify:
    - manual review of packet-local docs
  - Dependencies: `R6-3.X.2D.0.1`
  - Files likely touched:
    - packet-local spec/plan/tasks docs
  - Estimated scope: XS
  - Result (2026-07-08): the packet docs now lock the draft to the owned write set and repeat the
    narrow non-goal fence in all three packet files.

## R6-3.X.2D.1: `NoClaim` decision-semantics boundary

- [x] Task R6-3.X.2D.1.1: Lock `NoClaim` as internal abstention and public non-fire.
  - Acceptance:
    - The scorer design states explicitly that `NoClaim` remains internal abstention.
    - The docs state explicitly that `NoClaim` remains non-fire at the public boundary.
    - The docs state explicitly that `NoClaim` is not separately exported unless a later packet
      reopens that surface.
    - The public non-fire shape is named explicitly as the current `DriftScore` contract
      (`flagged: false`, `state: Cleared`, `raw_score: 0`).
  - Verify:
    - scorer-local tests may assert `DriftDecision::NoClaim`
    - checkpoint/progress tests assert the public non-fire surface remains intact
  - Dependencies: `R6-3.X.2D.0.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - packet-local docs
  - Estimated scope: M
  - Result (2026-07-08): `NoClaim` remains internal-only in the scorer, the explicit non-fire
    boundary shape stayed `Cleared` / `flagged: false` / `raw_score: 0`, and live checkpoint/progress
    proof stayed on the existing public surface rather than gaining a new exported abstention shape.

- [x] Task R6-3.X.2D.1.2: Add regression proof that weak suppressive abstention stays distinct from suppression.
  - Acceptance:
    - At least one scorer-local regression asserts an under-supported weak suppressive relation resolves to
      `NoClaim`, not `Suppress`.
    - At least one public-surface regression proves the same case remains non-fire rather than gaining a new
      exported abstention shape.
    - The docs do not describe `NoClaim` as positive continuity.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer progress_acceptance -- --nocapture`
  - Dependencies: `R6-3.X.2D.1.1`
  - Files likely touched:
    - scorer tests
    - checkpoint/progress tests
    - packet-local docs
  - Estimated scope: M
  - Result (2026-07-08): scorer-local regressions now pin under-supported weak doc-family
    assessments to `NoClaim`, and a live checkpoint regression keeps the public boundary on the
    existing non-fire shape.

## R6-3.X.2D.2: Exact root-level doc-bundle residue witness

- [x] Task R6-3.X.2D.2.1: Add the exact root-level bundle -> member narrowing witness scorer-locally.
  - Acceptance:
    - The scorer test surface includes the exact residue family already called out in live docs:
      `architecture-overview.md|README.md|authentication-security.md|graphql-federation.md|module-development.md|monitoring.md`
      -> `README.md`.
    - The witness is root-level (no stable shared prefix baked in).
    - The packet docs state explicitly that this witness is required closeout proof.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Dependencies: `R6-3.X.2D.1.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - packet-local docs
  - Estimated scope: M
  - Result (2026-07-08): the scorer now carries an exact root-level canonical
    `README.md`-anchored doc-bundle residue closure, proved scorer-locally without broad root-level
    doc-bundle suppression.

- [x] Task R6-3.X.2D.2.2: Add the same residue witness through the live analyzer acceptance path.
  - Acceptance:
    - A dedicated acceptance fixture covers the same root-level bundle/member witness.
    - The acceptance allowlist stays explicit and bounded.
    - The fixture exercises the live analyzer checkpoint path, not only scorer-local helpers.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
  - Dependencies: `R6-3.X.2D.2.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - Estimated scope: M
  - Result (2026-07-08): the acceptance corpus grew in a bounded way to include the exact root
    `README.md` doc-bundle narrowing witness through the live analyzer checkpoint path.

- [x] Task R6-3.X.2D.2.3: Pair the residue witness with a negative guard against broad root-level suppression.
  - Acceptance:
    - A root-level bundle with an unrelated added doc still fires.
    - The docs explicitly forbid solving the residue by broad root-level doc-bundle suppression.
    - If the scorer cannot close the residue without broad suppression, the packet records that limit honestly
      instead of widening scope silently.
  - Verify:
    - scorer-local regression tests
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
  - Dependencies: `R6-3.X.2D.2.2`
  - Files likely touched:
    - scorer tests
    - acceptance fixtures/harness
    - packet-local docs
  - Estimated scope: M
  - Result (2026-07-08): scorer-local and live acceptance guards now prove that adding an unrelated
    root-level doc (`CHANGELOG.md`) still fires, so the residue closure did not become broad root-level
    doc-bundle suppression.

## R6-3.X.2D.3: Non-`src` role-classification witnesses

- [x] Task R6-3.X.2D.3.1: Add path-classification witnesses for `build.rs`, root `examples/*.rs`, and `tests/*.rs`.
  - Acceptance:
    - `build.rs` is explicitly classified by path structure / extension rather than substring doc hints.
    - root `examples/*.rs` is explicitly classified by path structure / extension rather than substring doc
      hints.
    - `tests/*.rs` remains explicitly verification-shaped.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Dependencies: `R6-3.X.2D.2.3`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: S
  - Result (2026-07-08): `build.rs` and root `examples/*.rs` now classify as `Code`, while root
    `tests/*.rs` stays `Verify`, all from path structure/extension before substring doc hints.

- [x] Task R6-3.X.2D.3.2: Make the `benches/*.rs` role explicit and tested.
  - Acceptance:
    - The packet states whether root `benches/*.rs` is locked as `Code` or `Verify`.
    - The corresponding scorer-local witness exists.
    - The choice is justified in packet docs rather than left as accidental behavior.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Dependencies: `R6-3.X.2D.3.1`
  - Files likely touched:
    - scorer tests
    - packet-local docs
  - Estimated scope: S
  - Result (2026-07-08): root `benches/*.rs` is now locked and tested as `Verify`, with the packet
    docs explicitly justifying benchmark harnesses as proof/performance verification surfaces.

## R6-3.X.2D.4: Load-bearing threshold boundary tests

- [x] Task R6-3.X.2D.4.1: Add family-token and doc-prefix edge tests.
  - Acceptance:
    - One shared non-generic family token does not suppress.
    - Two shared non-generic family tokens may suppress only with the existing stronger continuity proof.
    - Shared doc prefix depth `2` does not count as same-doc-family by default.
    - Shared doc prefix depth `3` remains the default same-doc-family threshold.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Dependencies: `R6-3.X.2D.3.2`
  - Files likely touched:
    - scorer tests
  - Estimated scope: M
  - Result (2026-07-08): scorer-local regressions now pin the one-vs-two shared-family-token edge
    and the shared-doc-prefix depth `2` vs `3` edge explicitly.

- [x] Task R6-3.X.2D.4.2: Add work-item-lineage and evidence-gate edge tests.
  - Acceptance:
    - Shared lineage depth `1` does not suppress.
    - Shared lineage depth `2+` still needs stronger continuity and then may suppress.
    - Low-confidence weak suppressive relations never suppress.
    - Empty decisive evidence never suppresses.
    - Material counter-evidence never suppresses.
    - Medium-or-higher confidence + decisive evidence + empty counter-evidence remains the only weak
      suppressive path.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Dependencies: `R6-3.X.2D.4.1`
  - Files likely touched:
    - scorer tests
    - checkpoint tests if needed for public-surface proof
  - Estimated scope: M
  - Result (2026-07-08): scorer-local regressions now pin lineage depth `1` vs `2+` and the weak
    suppressive evidence gates (`confidence`, decisive evidence, and counter-evidence).

## R6-3.X.2D.5: Proof wall and honest rerun rule

- [x] Task R6-3.X.2D.5.1: Run the focused wall and full analyzer wall.
  - Acceptance:
    - focused semantic-goal-drift tests pass,
    - acceptance wall passes,
    - checkpoint and progress regressions pass,
    - full analyzer wall passes,
    - any skipped verification is documented honestly.
  - Verify:
    - the focused and full verification wall from the packet spec
  - Dependencies: `R6-3.X.2D.4.2`
  - Files likely touched:
    - docs only unless a tiny test/fixture adjustment is required during verification
  - Estimated scope: S
  - Result (2026-07-08): focused semantic-goal-drift, acceptance, checkpoint, progress, full
    analyzer, clippy, fmt, and targeted build verification all passed.

- [x] Task R6-3.X.2D.5.2: Require the seed-42 rerun whenever scorer/classifier semantics changed.
  - Acceptance:
    - if scorer/classifier logic changed, the packet runs the seed-42 rerun and records the result honestly;
    - if the packet truly stayed docs/tests-only with no semantic scorer/classifier change, the docs may note why
      the rerun was skipped, but do not treat that skip as closeout proof;
    - the docs state explicitly that scorer/classifier change is the default expected path, so rerun is the
      honest default.
  - Verify:
    - the batch command sequence from the packet spec
  - Dependencies: `R6-3.X.2D.5.1`
  - Files likely touched:
    - packet-local docs
    - rerun artifacts outside the repo during implementation, not in this draft
  - Estimated scope: M
  - Result (2026-07-08): scorer/classifier logic changed, so the seed-42 rerun was completed.
    Current proof run: `110/110` sessions clean across `45` repos, `884` checkpoints, `146`
    current-bar eligible checkpoints, `245` target-resolved eligible checkpoints, `206` adjacent
    target-eligible pairs, `199` same-target suppressions, `7` changed-target candidates, `6`
    disjoint pairs, and `0` emitted `semantic_goal_drift` fires.

## Final checkpoint

Before implementation is considered packet-complete:
- [x] `NoClaim` remains internal abstention and public non-fire
- [x] no new public/exported abstention surface was introduced
- [x] the exact root-level bundle/member residue witness exists scorer-locally and through the live
      analyzer acceptance path
- [x] root-level unrelated bundle additions still fire
- [x] `build.rs`, root `examples/*.rs`, root `benches/*.rs`, and `tests/*.rs` all have explicit role
      witnesses
- [x] the continuity thresholds are pinned with edge tests
- [x] `R6-3.X.3` stayed deferred
- [x] no new relation families were added
- [x] no broad doc-bundle suppression was introduced
- [x] no repo-relative cwd-strip equivalence was introduced without separate approval
- [x] no sentinel/compactor/schema/delegation expansion occurred
- [x] focused tests and full analyzer wall pass
- [x] the seed-42 rerun was completed whenever scorer/classifier semantics changed
