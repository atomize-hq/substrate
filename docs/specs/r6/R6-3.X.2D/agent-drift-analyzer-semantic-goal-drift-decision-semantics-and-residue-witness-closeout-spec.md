# R6-3.X.2D SPEC — Decision Semantics And Residue Witness Closeout For Semantic Goal Drift

Status: LANDED / CLOSED (implemented and validated 2026-07-08 after the `R6-3.X.2C` closeout).
Packet-scoped to analyzer-local semantic-goal-drift decision semantics, the exact remaining
root-level doc-bundle residue witness, non-`src` code-path role-classification proof, and
threshold-boundary regression coverage.

Assumptions carried into this spec:
1. `R6-3.5` target hygiene remains landed and authoritative.
2. The `R6-3.X.2` containment first cut remains landed and must stay behaviorally intact.
3. `R6-3.X.2C` landed the internal `Suppress` / `Fire` / `NoClaim` decision surface and the live
   scorer currently treats `NoClaim` as a non-fire outcome through the existing public `DriftScore`
   boundary.
4. The only explicitly named live residue from the current seed-42 rerun is the conservative
   root-level doc-bundle -> anchored member-doc narrowing family
   (`architecture-overview.md|README.md|authentication-security.md|graphql-federation.md|module-development.md|monitoring.md`
   -> `README.md`) where the bundle lacks a stable shared prefix.
5. Implementation stayed analyzer-local: scorer logic, scorer/analyzer tests, acceptance fixtures,
   packet-local docs, and rerun proof only. The later docs-only closeout reconciliation updated `MAP`,
   `FINDINGS`, and the parent `R6-3` ledger without widening scorer behavior.
6. `context/objective.rs`, sentinel, compactor, delegation surfaces, checkpoint export/schema, and
   `R6-3.X.3` eligibility loosening remain out of scope unless explicitly reopened.

Authority / cross-references:
- `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
- `docs/specs/r6/MAP.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
- `docs/specs/r6/R6-3.X.2C/agent-drift-analyzer-semantic-goal-drift-relation-routing-contract-and-suppression-guardrails-{spec,plan,tasks}.md`
- live scorer/tests/tooling:
  - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - `scripts/dev/drift-batch-scan/**`
- `AGENTS.md`

Closeout result:
- `NoClaim` remains internal abstention and public non-fire.
- The exact root-level README doc-bundle narrowing and unrelated-root-doc addition guard are committed
  scorer-locally and through the live analyzer acceptance path.
- Non-`src` role witnesses and continuity-threshold edge tests are committed.
- The fresh seed-42 rerun completed with `110/110` sessions clean across `45` repos, `884` checkpoints,
  and `0` emitted `semantic_goal_drift` fires.
- `R6-3.X.3` remains deferred.

## Objective

Close the narrow follow-up debt left after `R6-3.X.2C` without widening back into broader scorer or
export work.

Success means this packet:
- locks `NoClaim` as **internal abstention** that remains a non-fire at the public boundary,
- refuses to export or productize `NoClaim` separately unless a later packet explicitly reopens that
  surface,
- adds the exact root-level doc-bundle residue witness as a required proof item,
- expands role-classification proof to non-`src` code paths (`build.rs`, root `examples/*.rs`, root
  `benches/*.rs`, and `tests/*.rs`),
- adds boundary tests around the numeric continuity thresholds that currently carry suppression,
- keeps `R6-3.X.3` deferred,
- requires the seed-42 rerun whenever scorer/classifier semantics change.

This is a closeout-hardening packet, not a license to reopen eligibility, invent new relation
families, or broadly suppress doc bundles.

## Commands

Read-only grounding commands used for this packet:

```bash
sed -n '300,380p' docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md
sed -n '280,320p' docs/specs/r6/MAP.md
sed -n '420,470p' docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
sed -n '1,260p' docs/specs/r6/R6-3.X.2C/agent-drift-analyzer-semantic-goal-drift-relation-routing-contract-and-suppression-guardrails-spec.md
sed -n '1,220p' crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs
sed -n '150,260p' crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
sed -n '520,1160p' crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
rg -n "NoClaim|doc_bundle|build\\.rs|examples/|benches/|tests/|shared_family.len|shared_segment_prefix_len|shared_work_item_lineage" \
  crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs \
  crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs \
  crates/agent-drift-analyzer/tests/{checkpoints,progress_acceptance}.rs
```

Implementation verification commands this packet must carry explicitly:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo build -p agent-session-compactor -p agent-drift-analyzer
python3 scripts/dev/drift-batch-scan/sample_sessions.py --seed 42 --out /tmp/r6_3_x_2d_selected.jsonl
python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2d_selected.jsonl --batch-dir /tmp/r6_3_x_2d_batch
python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2d_batch/checkpoints
python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2d_batch/checkpoints
python3 scripts/dev/drift-batch-scan/filter_junk.py --checkpoints-dir /tmp/r6_3_x_2d_batch/checkpoints
```

Implementation preflight when code work begins:
- before editing any scorer/classifier symbol, run GitNexus impact analysis on that symbol and record
  the blast radius;
- warn before proceeding if impact returns HIGH or CRITICAL;
- run `gitnexus detect-changes` (or the repo-qualified equivalent) before commit.

## Project Structure

Primary packet surfaces:
- `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - `DriftDecision` semantics
  - root-level doc-bundle residue handling
  - role classification for non-`src` code paths
  - continuity-threshold edge behavior
- `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - live analyzer acceptance cases for the residue witness and any added public-boundary guard case
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - public `DriftScore` surface guards when `NoClaim` stays non-fire
- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
  - regression proof that packet-local semantic-goal-drift abstention still reads as current
    non-fire/cleared output
- `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - exact bundle/member residue witness fixtures
- `docs/specs/r6/R6-3.X.2D/**`
  - packet-local spec / plan / tasks only
- `scripts/dev/drift-batch-scan/**`
  - rerun proof only when scorer/classifier semantics change

Secondary scope only if the smallest analyzer-local move needs it:
- `crates/agent-drift-analyzer/tests/support/mod.rs`

Out of scope unless explicitly reopened:
- `crates/agent-drift-analyzer/src/context/objective.rs`
- `crates/agent-drift-analyzer/src/checkpoint/{progress.rs,working_set.rs,export.rs,schema.rs}`
- `crates/agent-drift-sentinel/**`
- `crates/agent-session-compactor/**`
- `docs/specs/r6/{MAP.md,FINDINGS-r6-3-real-world-drift-validation.md}`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`

## Code Style

Implementation constraints:
- keep the scorer deterministic and analyzer-local;
- keep the current relation family set fixed; no new relation families in `2D`;
- keep `NoClaim` **internal** to the scorer decision seam unless a later packet explicitly reopens a
  public/exported abstention state;
- keep the current public boundary unchanged: `NoClaim` continues to surface as the existing
  non-fire `DriftScore` shape (`flagged: false`, `state: Cleared`, `raw_score: 0`) rather than a new
  exported enum, schema field, or sentinel surface;
- do not make root-level doc-bundle handling a broad top-level doc suppression rule; if the residue is
  closed, it must be closed by exact anchored member reuse plus bounded same-family proof, not by
  suppressing arbitrary root-level bundles;
- classify code-like paths from path structure and extension before substring doc hints, including
  non-`src` paths;
- treat `build.rs` and root `examples/*.rs` as code-path witnesses, `tests/*.rs` as verify-path
  witnesses, and lock root `benches/*.rs` as `Verify` because benchmark harnesses are
  measurement/proof surfaces rather than implementation-owned source targets; do not leave that
  choice implicit;
- keep the load-bearing thresholds explicit and tested rather than “cleaning them up” into fuzzy
  heuristics:
  - `shared_family_tokens >= 2` for family/workstream continuity,
  - `shared_segment_prefix_len >= 3` for same-doc-family / doc-bundle family proof,
  - shared work-item lineage common depth `>= 2`,
  - weak suppressive relations require `confidence >= Medium`, non-empty decisive evidence, and empty
    material counter-evidence;
- do not widen into repo-relative cwd-strip equivalence, eligibility loosening, sentinel work,
  compactor work, schema/export work, or delegation semantics.

## Testing Strategy

This packet needs five proof layers:
1. scorer-local unit coverage for `DriftDecision` semantics where `NoClaim` remains distinct from
   `Suppress` internally but still non-fire publicly;
2. the exact root-level doc-bundle residue witness in both scorer-local and live analyzer acceptance
   form;
3. role-classification witnesses for `build.rs`, root `examples/*.rs`, root `benches/*.rs`, and
   `tests/*.rs`;
4. threshold-boundary regressions for every numeric/load-bearing continuity gate listed above;
5. the focused wall, full analyzer wall, and default seed-42 rerun whenever scorer/classifier logic
   changes.

Acceptance is not “tests pass somehow.” The packet is incomplete unless the residue witness is exact,
`NoClaim` stays non-fire at the public boundary, and the threshold edges are pinned so later packets
cannot silently loosen or over-tighten them.

## Boundaries

Always do:
- keep `NoClaim` non-fire at the public boundary;
- keep `NoClaim` internal abstention unless separately exported later;
- add the exact root-level doc-bundle residue witness as a required proof item;
- add non-`src` code-path role-classification witnesses;
- add threshold-boundary tests for the current load-bearing continuity cutoffs;
- keep `R6-3.X.3` deferred;
- require the seed-42 rerun whenever scorer/classifier logic changes.

Ask first before:
- exporting `NoClaim` on any public/checkpoint/schema/sentinel surface,
- loosening the eligibility bar,
- adding new relation families,
- broadly suppressing doc bundles beyond the exact residue witness,
- implementing repo-relative cwd-strip equivalence,
- widening into sentinel/compactor/schema/delegation work,
- reopening `MAP`, `FINDINGS`, or the `R6-3` ledger inside this packet draft.

Never do in this packet:
- loosen `R6-3.X.3` eligibility,
- add new relation families,
- claim root-level doc-bundle residue is fixed by a broad “doc bundle means suppress” shortcut,
- treat repo-relative cwd-strip equivalence as already approved,
- expand into sentinel/compactor/export/schema/delegation surfaces,
- silently convert threshold boundaries into score-band fuzziness.

## Live Repo Truth Reconciled

The live repo currently shows:
- `R6-3.X.2C` already replaced relation-only `claims_drift()` routing with the internal
  `Suppress` / `Fire` / `NoClaim` decision surface.
- The live scorer currently maps non-claims back to the existing public non-fire shape via
  `no_claim(...)`, so `NoClaim` is already internal-only in practice.
- Existing doc-bundle tests prove nested-family bundle/member suppression under a stable shared prefix,
  and the live acceptance corpus already includes `synthetic-rolling-doc-bundle-broadening`.
- The current closeout docs still name one conservative residue family from the seed-42 rerun:
  root-level doc-bundle -> anchored member-doc narrowing with no stable shared prefix.
- Current role-classification regression proves `src/spec_parser.rs`, `src/design_tokens.rs`, and
  `tests/spec_parser.rs`, but does not yet explicitly pin `build.rs`, root `examples/*.rs`, or root
  `benches/*.rs`.
- Current suppression continuity depends on exact numeric thresholds (`>= 2` family tokens,
  `>= 3` shared doc prefix depth, lineage common depth `>= 2`, and Medium-or-higher evidence-gated
  weak suppressions), but the edge cases around those exact cutoffs are not yet all called out as
  first-class packet proof.

Therefore this packet should not reopen the broad routing contract. It should close the named follow-up
seam precisely: decision semantics stay internal/publicly non-fire, the exact residue witness gets
proved, non-`src` code paths get role proof, and threshold edges get pinned.

## Design

### 1. Keep `NoClaim` internal and non-fire at the public boundary

`R6-3.X.2C` introduced `DriftDecision::NoClaim` inside the scorer. `2D` must lock the intent that:
- `NoClaim` is an internal abstention state,
- `NoClaim` is **not** equivalent to positive continuity,
- `NoClaim` is **not** a public `DriftScore` variant,
- `NoClaim` remains a non-fire at current public surfaces.

Required outcome:
- internal tests may assert `DriftDecision::NoClaim`,
- public/checkpoint/progress tests must still observe the current non-fire output shape,
- any separate exported abstention surface is future work and must not be smuggled into `2D`.

### 2. Add the exact root-level doc-bundle residue witness

The residue to close is the exact conservative family already called out in live docs:

- previous/other side bundle:
  `architecture-overview.md|README.md|authentication-security.md|graphql-federation.md|module-development.md|monitoring.md`
- current side member:
  `README.md`

Required proof shape:
- add a scorer-local witness for this exact root-level bundle/member narrowing;
- add a live analyzer acceptance witness for the same family;
- pair it with at least one negative guard showing that a root-level bundle with an unrelated added
  doc still fires.

Design constraint:
- fixing this witness must **not** create a broad “root-level doc bundle means same-doc-family” rule.
  The closure must remain tied to exact anchored member reuse and bounded family evidence.

### 3. Add non-`src` code-path role-classification witnesses

The current classifier already prefers code/test path markers over substring doc hints, but `2D` must
make the remaining code-path witnesses explicit.

Required witness set:
- `build.rs`
- root `examples/*.rs`
- root `benches/*.rs`
- `tests/*.rs`

Minimum expectation:
- these witnesses are classified by path structure / extension first, not because their names happen
  to avoid `spec` / `design` substrings;
- `tests/*.rs` remains verification-shaped;
- `build.rs`, `examples/*.rs`, and `benches/*.rs` cannot drift into doc roles because of substring
  hints.

Ambiguity to resolve explicitly, not implicitly:
- `2D` locks root `benches/*.rs` as `Verify` rather than `Code` because benchmark harnesses are
  verifier/performance-proof targets, and the choice is justified in tests/docs rather than left as
  accidental behavior.

### 4. Pin the load-bearing continuity thresholds with edge tests

`2D` must add boundary tests around the exact thresholds that currently control suppressive continuity.

Required edge matrix:
1. **Family continuity threshold**
   - one shared non-generic family token -> does **not** suppress;
   - two shared non-generic family tokens + valid leaf/workstream evidence -> may suppress.
2. **Doc-family prefix depth threshold**
   - shared segment prefix depth `2` -> does **not** count as same-doc-family by default;
   - shared segment prefix depth `3` -> may count as same-doc-family.
3. **Work-item lineage threshold**
   - shared lineage depth `1` (`R6` only) -> does **not** suppress;
   - shared lineage depth `2+` (`R6-3`, `Packet-4`) still requires stronger continuity and then may
     suppress.
4. **Weak suppressive relation gate**
   - Low confidence -> never suppress;
   - empty decisive evidence -> never suppress;
   - any material counter-evidence -> never suppress;
   - Medium-or-higher confidence + decisive evidence + empty counter-evidence -> may suppress.

The point is to stop future packets from silently moving the bar by one token, one path segment, or
one evidence gate and calling it cleanup.

### 5. Default rerun rule

If scorer/classifier logic changes in `2D`, the seed-42 rerun is required.

Default rule:
- scorer/classifier change -> rerun required;
- docs-only or docs-plus-test-only draft with **no** semantic scorer/classifier change -> rerun may be
  omitted during draft authoring, but the omission must be called out explicitly and cannot be used as
  honest packet closeout proof.

Because the likely `2D` path touches scorer/classifier semantics, the honest default is: **assume the
rerun is required unless proven otherwise**.
