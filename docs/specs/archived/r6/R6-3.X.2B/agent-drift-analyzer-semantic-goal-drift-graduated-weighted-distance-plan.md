# R6-3.X.2B PLAN — Graduated / Weighted Semantic Distance For Semantic Goal Drift

Companion to `agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-spec.md`.
This remains an analyzer-local, precision-first scorer packet. Execution order:
docs lock → implementation boundary + GitNexus preflight → relation model → scorer-local relation plumbing →
proof wall expansion → seed-pinned corpus rerun → routing docs.

## Live repo truth this plan assumes

- `R6-3.5` target hygiene is already landed and remains authoritative.
- `R6-3.X.2` containment first cut is already landed and must remain intact.
- `R6-3.6` already proved recall can be demonstrated with bounded positive controls.
- The next work is the **remaining** weighted-distance remainder, not eligibility loosening.
- The current residue families are progression/family/pairwise-role-shift cases, not genuine pivots.
- The current scorer seam does **not** yet show a live `cwd` / `session_meta` surface, so cwd-strip
  repo-relative equivalence is conditional/deferred until proven.

## Dependency graph

1. **Authority reconciliation**
   - `MAP`, `FINDINGS`, `R6-3` ledger, scorer, acceptance harness, batch scripts, structured-objective docs.
2. **Implementation boundary + shared-helper preflight**
   - define out-of-scope fences, GitNexus impact-analysis requirements, and prior-witness non-regression walls.
3. **Stable-anchor preparation and relation taxonomy**
   - relation taxonomy is the routing authority; pairwise role-shift relations replace any implied hidden cycle.
4. **Scorer decision plumbing**
   - depends on the taxonomy and assessment shape; relation drives routing, score only explains it.
5. **Unit and acceptance controls**
   - depend on the scorer-local model so relation families are pinned at both local and live-analyzer layers.
6. **Corpus tooling strata and rerun**
   - depends on the control set, seed pin/baseline pin, and finalized relation family names.
7. **Findings/routing closeout**
   - depends on the rerun output.

## Vertical task sequence

1. **Docs lock + packet framing**
   - Confirm live repo truth against `FINDINGS`, `MAP`, the `R6-3` ledger, the scorer, acceptance corpus,
     batch scripts, and structured-objective design docs.
   - Lock the packet-local SPEC/PLAN/TASKS family in `docs/specs/r6/R6-3.X.2B/`.
   - Record the default routing posture: `R6-3.X.3` stays deferred unless this packet changes the evidence.

2. **Implementation boundary + preflight slice**
   - Lock the do-not-touch fence around `context/objective.rs`, checkpoint/export/schema/progress seams,
     compactor, and sentinel.
   - Require GitNexus impact analysis before touching any shared scorer helper reused by kickoff and rolling.
   - Pin the prior-witness non-regression wall (`R6-1`, `R6-2`, `R6-3.X.2`, `R6-3.6`, carried `R5.75` witnesses).

3. **Stable-anchor and taxonomy slice**
   - Design the bounded stable-anchor comparison record.
   - Define relation precedence and multi-anchor matching semantics.
   - Make the relation taxonomy the authoritative routing surface.
   - Tighten `SameArtifactFamily`, `SameWorkItemFamily`, and `WeakOrGenericOnly` so generic or sibling-stem
     residue cannot silently suppress a pivot.
   - Treat repo-relative cwd-strip equivalence as conditional/deferred unless seam reachability is proven.

4. **Weighted assessment slice**
   - Add the analyzer-local assessment shape and explanatory score-band policy.
   - Route kickoff and rolling comparisons through the same relation-classification surface after exact and
     containment checks.
   - Keep no-claim behavior conservative for ambiguous cases.

5. **Regression and acceptance slice**
   - Add scorer-local regression coverage for each relation family and each paired false-negative guard.
   - Expand acceptance fixtures to include the required positive and negative control families.
   - Keep the curated allowlist explicit and bounded.
   - Preserve all prior witnesses without rebaseline.

6. **Corpus-tooling and strata slice**
   - Extend reporting only as far as current export/tooling allows.
   - Require delegation topology output and any other stratum that can be meaningfully populated.
   - Do not treat `100% unknown` strata as satisfying the gate; record them as blockers if they remain unknown.

7. **Rerun + routing slice**
   - Run the focused tests, full analyzer wall, and seed-42 110-session batch rerun.
   - Baseline the results against the published `R6-3.6` numbers; if the exact prior manifest is unavailable,
     say so explicitly and compare directionally.
   - Hand-label remaining residue by relation family.
   - Update `FINDINGS`, `MAP`, and the `R6-3` ledger with the results and the `R6-3.X.3` decision.

## Checkpoints

### Checkpoint A — docs and boundaries locked
- packet docs exist and match live repo truth
- out-of-scope fences are explicit
- GitNexus preflight is explicit
- default routing posture is locked

### Checkpoint B — relation taxonomy locked
- relation taxonomy names and meanings are explicit
- pairwise role-shift relations replace implied hidden cycles
- `SameArtifactFamily` / `SameWorkItemFamily` / `WeakOrGenericOnly` are tightened
- repo-relative cwd-strip equivalence is either proven reachable or explicitly deferred

### Checkpoint C — scorer-local design settled
- stable-anchor preparation shape is defined
- weighted assessment shape is defined
- exact/containment preservation is explicit
- routing is relation-authoritative, not score-authoritative
- shared-constraint-only is marked non-suppressive

### Checkpoint D — proof wall defined
- positive and negative control families are enumerated
- each new suppressive relation has a paired false-negative guard
- prior witnesses are explicitly protected from rebaseline
- corpus strata/reporting requirements are explicit

### Checkpoint E — closeout criteria locked
- rerun commands are explicit
- seed/baseline pin against `R6-3.6` is explicit
- findings/map/ledger closeout targets are listed
- `R6-3.X.3` defer/reopen rule is explicit
- validation strata cannot pass by all-unknown reporting

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
python3 scripts/dev/drift-batch-scan/sample_sessions.py --seed 42 --out /tmp/r6_3_x_2b_selected.jsonl
python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2b_selected.jsonl --batch-dir /tmp/r6_3_x_2b_batch
python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2b_batch/checkpoints
python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2b_batch/checkpoints
```

Baseline comparison target from `R6-3.6`:
- `110` sessions
- `44` repos
- `938` checkpoints
- `221` adjacent target-eligible pairs
- `213` same-target exact-match suppressions
- `7` remaining disjoint pairs
- `0` fires

## Risks and mitigations

1. **Overfitting to current Rust/doc residue**
   - Risk: relation helpers become a bespoke patch set for seven known pairs.
   - Mitigation: require anchor-kind-driven rules, cross-language guardrails, and paired false-negative guards.

2. **Hidden containment widening**
   - Risk: family or role-shift logic leaks into the structural-containment helper.
   - Mitigation: keep containment helper frozen and add later ordered detectors only.

3. **Silent suppression from weak family evidence**
   - Risk: `SameArtifactFamily` / `SameWorkItemFamily` swallow sibling pivots or generic `spec/plan/tasks`
     overlaps.
   - Mitigation: make those weak/generic cases explicitly non-suppressive and pin them with counter-example tests.

4. **Hidden seam expansion through cwd or extraction work**
   - Risk: repo-relative equivalence or relation cleanup quietly reaches into `context/objective.rs` or unseen
     checkpoint/session metadata seams.
   - Mitigation: require explicit seam proof first; otherwise defer/ask-first.

5. **Tooling strata require export widening**
   - Risk: the plan promises reporting that current checkpoints cannot support.
   - Mitigation: require delegation output, allow blockers for other strata, and refuse to count all-unknown as done.

6. **Premature reopening of `R6-3.X.3`**
   - Risk: after weighted work lands, the run reopens eligibility without isolating the true blocker.
   - Mitigation: make the routing decision evidence-gated: only reopen if weighted scoring lands cleanly and
     genuine pivots still remain blocked solely by the eligibility bar.
