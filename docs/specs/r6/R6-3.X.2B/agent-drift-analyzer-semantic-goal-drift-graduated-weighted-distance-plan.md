# R6-3.X.2B PLAN — Graduated / Weighted Semantic Distance For Semantic Goal Drift

Companion to `agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-spec.md`.
This is a scorer-redesign packet, but it remains analyzer-local and precision-first. Execution order:
docs lock → relation model → scorer-local relation plumbing → proof wall expansion → corpus rerun → routing docs.

## Live repo truth this plan assumes

- `R6-3.5` target hygiene is already landed and remains authoritative.
- `R6-3.X.2` containment first cut is already landed and must remain intact.
- `R6-3.6` already proved recall can be demonstrated with bounded positive controls.
- The next work is the **remaining** weighted-distance remainder, not eligibility loosening.
- The current residue families are progression/family/cycle cases, not genuine pivots.

## Dependency graph

1. **Authority reconciliation**
   - `MAP`, `FINDINGS`, `R6-3` ledger, scorer, acceptance harness, batch scripts, structured-objective docs.
2. **Stable-anchor preparation and relation taxonomy**
   - must exist before any weighted-score routing logic.
3. **Scorer decision plumbing**
   - depends on the taxonomy and assessment shape.
4. **Unit and acceptance controls**
   - depend on the scorer-local model so relation families are pinned at both local and live-analyzer layers.
5. **Corpus tooling strata and rerun**
   - depends on the control set and finalized relation family names.
6. **Findings/routing closeout**
   - depends on the rerun output.

## Vertical task sequence

1. **Docs lock + packet framing**
   - Confirm live repo truth against `FINDINGS`, `MAP`, the `R6-3` ledger, the scorer, acceptance corpus,
     batch scripts, and structured-objective design docs.
   - Write the packet-local SPEC/PLAN/TASKS family in the `docs/specs/r6/R6-3.X.2B/` directory.
   - Record the default routing posture: `R6-3.X.3` stays deferred unless this packet changes the evidence.

2. **Stable-anchor and taxonomy slice**
   - Design the bounded stable-anchor comparison record.
   - Define relation precedence and multi-anchor matching semantics.
   - Specify the non-suppressive families (`SharedConstraintOnly`, `WeakOrGenericOnly`, `Unknown`) explicitly.

3. **Weighted assessment slice**
   - Add the analyzer-local assessment shape and score-band policy.
   - Route kickoff and rolling comparisons through the same relation-classification surface after exact and
     containment checks.
   - Keep no-claim behavior conservative for ambiguous cases.

4. **Regression and acceptance slice**
   - Add scorer-local regression coverage for each relation family and each paired false-negative guard.
   - Expand acceptance fixtures to include the required positive and negative control families.
   - Keep the curated allowlist explicit and bounded.

5. **Corpus-tooling and strata slice**
   - Extend reporting only as far as current export/tooling allows.
   - Add best-effort strata reporting for language/repo type, workflow type, tooling type, and delegation.
   - Preserve honest non-derivable buckets rather than duplicating scorer logic silently.

6. **Rerun + routing slice**
   - Run the focused tests, full analyzer wall, and 110-session batch rerun.
   - Hand-label remaining residue by relation family.
   - Update `FINDINGS`, `MAP`, and the `R6-3` ledger with the results and the `R6-3.X.3` decision.

## Checkpoints

### Checkpoint A — docs and taxonomy locked
- packet docs exist and match live repo truth
- relation taxonomy and score-band meanings are explicit
- non-goals and routing posture are locked

### Checkpoint B — scorer-local design settled
- stable-anchor preparation shape is defined
- weighted assessment shape is defined
- exact/containment preservation is explicit
- shared-constraint-only is marked non-suppressive

### Checkpoint C — proof wall defined
- positive and negative control families are enumerated
- each new suppressive relation has a paired false-negative guard
- corpus strata/reporting requirements are explicit

### Checkpoint D — closeout criteria locked
- rerun commands are explicit
- findings/map/ledger closeout targets are listed
- `R6-3.X.3` defer/reopen rule is explicit

## Verification wall

Focused scorer/analyzer wall:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Full analyzer closeout:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Corpus rerun:

```bash
cargo build -p agent-session-compactor -p agent-drift-analyzer
python3 scripts/dev/drift-batch-scan/sample_sessions.py --out /tmp/r6_3_x_2b_selected.jsonl
python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2b_selected.jsonl --batch-dir /tmp/r6_3_x_2b_batch
python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2b_batch/checkpoints
python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2b_batch/checkpoints
```

## Risks and mitigations

1. **Overfitting to current Rust/doc residue**
   - Risk: relation helpers become a bespoke patch set for seven known pairs.
   - Mitigation: require anchor-kind-driven rules, cross-language strata, and paired false-negative guards.

2. **Hidden containment widening**
   - Risk: family or cycle logic leaks into the structural-containment helper.
   - Mitigation: keep containment helper frozen and add later ordered detectors only.

3. **Ambiguous relation families create silent misses**
   - Risk: too many cases fall into suppressive buckets with weak evidence.
   - Mitigation: keep `Unknown` and `WeakOrGenericOnly` non-suppressive, and prefer no-claim over forced suppression.

4. **Tooling strata require export widening**
   - Risk: the plan promises reporting that current checkpoints cannot support.
   - Mitigation: allow `unknown` buckets and document non-derivable categories explicitly.

5. **Premature reopening of `R6-3.X.3`**
   - Risk: after weighted work lands, the run reopens eligibility without isolating the true blocker.
   - Mitigation: make the routing decision evidence-gated: only reopen if weighted scoring lands cleanly and
     genuine pivots still remain blocked solely by the eligibility bar.
