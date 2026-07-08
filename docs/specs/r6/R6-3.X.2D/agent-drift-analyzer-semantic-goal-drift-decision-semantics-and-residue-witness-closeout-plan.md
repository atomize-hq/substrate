# R6-3.X.2D PLAN — Decision Semantics And Residue Witness Closeout For Semantic Goal Drift

Companion to
`agent-drift-analyzer-semantic-goal-drift-decision-semantics-and-residue-witness-closeout-spec.md`.
This remains a narrow analyzer-local follow-up packet. Execution order:
docs lock -> `NoClaim` boundary semantics -> exact residue witness -> non-`src` role witnesses ->
threshold edges -> proof wall / rerun.

## Live repo truth this plan assumes

- `R6-3.5` target hygiene is already landed and remains authoritative.
- `R6-3.X.2` containment first cut is already landed and remains intact.
- `R6-3.X.2C` already landed the internal `Suppress` / `Fire` / `NoClaim` decision seam.
- The live public boundary still treats abstention as the current non-fire `DriftScore` shape.
- The named remaining conservative residue is the root-level doc-bundle -> member-doc narrowing family.
- `R6-3.X.3` remains deferred.
- This draft does not edit `MAP`, `FINDINGS`, or the `R6-3` ledger.

## Dependency graph

1. **Authority reconciliation + scope fence**
   - packet-local docs,
   - live scorer/tests,
   - current `R6` authority docs,
   - explicit do-not-touch fence for export/schema/sentinel/compactor/delegation.
2. **Decision semantics boundary**
   - `NoClaim` stays internal,
   - public boundary remains non-fire,
   - no exported abstention state.
3. **Exact residue witness**
   - depends on the decision semantics staying narrow,
   - closes only the named root-level doc-bundle witness, not doc bundles broadly.
4. **Role-classification and threshold edges**
   - depends on the scorer remaining analyzer-local,
   - proves non-`src` code-path classification and exact cutoff behavior.
5. **Proof wall and rerun**
   - depends on all semantic changes being settled,
   - rerun required if scorer/classifier behavior changes.

## Vertical task sequence

1. **Docs lock + packet framing**
   - Create the `R6-3.X.2D` spec/plan/tasks family.
   - Record that `2D` is a bounded hardening follow-up after `2C`, not a reopening of `R6-3.X.3`.
   - Lock the write-set fence: packet-local docs only for this draft.

2. **`NoClaim` boundary slice**
   - Lock that `NoClaim` remains internal abstention.
   - Add packet requirements proving the public boundary still reads as current non-fire output.
   - Explicitly defer any exported abstention surface.

3. **Exact residue-witness slice**
   - Add the exact root-level doc-bundle -> member-doc witness as required proof.
   - Add the paired negative guard so unrelated root-level bundle additions still fire.
   - Keep the fix bounded: no broad root-level doc-bundle suppression.

4. **Non-`src` role-classification slice**
   - Add witnesses for `build.rs`, root `examples/*.rs`, root `benches/*.rs`, and `tests/*.rs`.
   - Lock root `benches/*.rs` as `Verify` explicitly because benchmark harnesses are proof surfaces.
   - Keep path/extension semantics ahead of substring doc hints.

5. **Threshold-boundary slice**
   - Add off-by-one proof around family-token count, doc-prefix depth, work-item-lineage depth, and
     weak suppressive evidence gates.
   - Keep the thresholds explicit rather than allowing fuzzy score-based drift.

6. **Proof-wall / rerun slice**
   - Run the focused scorer/acceptance/checkpoint/progress tests.
   - Run the full analyzer wall.
   - If scorer/classifier semantics changed, rerun the seed-42 corpus and record the root-level
     residue outcome honestly.

## Checkpoints

### Checkpoint A — framing and fences locked
- packet-local docs exist
- analyzer-local scope is explicit
- `R6-3.X.3` defer is explicit
- no export/schema/sentinel/compactor/delegation widening is allowed
- this draft does not claim authority-doc edits

### Checkpoint B — `NoClaim` semantics locked
- `NoClaim` is explicitly internal-only
- the public boundary remains non-fire
- no exported abstention state is introduced
- checkpoint/progress proof requirements are explicit

### Checkpoint C — exact residue witness locked
- the exact root-level bundle/member example is named
- a scorer-local witness is required
- a live analyzer acceptance witness is required
- a paired negative guard is required
- broad doc-bundle suppression remains forbidden

### Checkpoint D — classifier and threshold proof locked
- non-`src` code-path witness set is explicit
- any `benches/*.rs` role decision is explicit
- off-by-one continuity-threshold tests are explicit
- weak suppressive evidence-gate tests are explicit

### Checkpoint E — closeout proof locked
- focused test commands are explicit
- full analyzer wall is explicit
- rerun-required rule is explicit for scorer/classifier changes
- docs/tests-only rerun waiver is narrowly defined and not treated as closeout proof

## Verification wall

Focused scorer/analyzer wall:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer progress_acceptance -- --nocapture
```

Full analyzer closeout:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Default rerun when scorer/classifier logic changes:

```bash
cargo build -p agent-session-compactor -p agent-drift-analyzer
python3 scripts/dev/drift-batch-scan/sample_sessions.py --seed 42 --out /tmp/r6_3_x_2d_selected.jsonl
python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2d_selected.jsonl --batch-dir /tmp/r6_3_x_2d_batch
python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2d_batch/checkpoints
python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2d_batch/checkpoints
python3 scripts/dev/drift-batch-scan/filter_junk.py --checkpoints-dir /tmp/r6_3_x_2d_batch/checkpoints
```

## Risks and mitigations

1. **`NoClaim` quietly escapes onto a public surface**
   - Risk: a well-meant “make abstention visible” cleanup mutates checkpoint/schema/sentinel truth.
   - Mitigation: lock `NoClaim` as internal-only and require explicit ask-first for any export.

2. **The root-level residue fix becomes broad doc-bundle suppression**
   - Risk: the scorer stops firing for unrelated top-level doc bundles.
   - Mitigation: require the exact witness plus a paired unrelated-addition negative guard.

3. **Non-`src` code paths stay under-specified**
   - Risk: future reviewers cannot tell whether `build.rs` / `examples` / `benches` / `tests` are
     deliberate or accidental role outcomes.
   - Mitigation: require explicit witnesses and an explicit `benches/*.rs` role decision.

4. **Threshold drift by “cleanup”**
   - Risk: a one-token or one-segment loosen/tighten slips through without being called a semantic
     change.
   - Mitigation: require off-by-one edge tests at the exact continuity thresholds.

5. **Rerun skipped after real semantic change**
   - Risk: packet closeout claims remain detached from corpus truth.
   - Mitigation: make seed-42 rerun default-required whenever scorer/classifier semantics change.

## Locked decisions

- `NoClaim` remains a non-fire at the public boundary and internal abstention unless separately
  exported later.
- `R6-3.X.2D` does not loosen `R6-3.X.3` eligibility.
- `R6-3.X.2D` adds no new relation families.
- `R6-3.X.2D` does not authorize broad doc-bundle suppression.
- `RepoRelativeEquivalentAfterCwdStrip` stays deferred unless separately approved.
- Sentinel/compactor/schema/delegation expansion stays out of scope.
- If scorer/classifier logic changes, the seed-42 rerun is required.
