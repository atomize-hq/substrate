# Plan: Agent Drift Analyzer Dead-End-Thrash Cutover And Frontier-Aware Process Dimensions (R6-1)

Status: draft plan created on 2026-06-27 from the `R6-1` SPEC in this directory and a read of the live
scorer (`scoring/dead_end_thrash.rs`) and the existing frontier signals (`checkpoint/progress.rs`). This
plan is reviewable: it should be possible to read it and say "yes, that approach" or "no, change X"
before any code lands.

## Objective

Turn `dead_end_thrash` from a streak-length scorer into a frontier-aware decisive-step scorer that
distinguishes a stall (repeated activity, no frontier movement) from expected debugging churn (repeated
activity, advancing frontier), with the smallest additive change that keeps the frozen replay corpus and
`R5.75` witnesses honest. Land the Guardrail-5 bridge coverage.

## Planning Decisions Locked For This Draft

1. The cutover lands in `score_dead_end_thrash` and **consumes** the existing `progress.rs` frontier
   judgment rather than recomputing it. That judgment is **not** on `CheckpointAnalysis` today and
   `build_session_progress` runs after `score_session`, so the access path (how the signal reaches the
   scorer) is an explicit `R6-1.1` decision, not an assumed free read. Recommended path: compute
   progress/archetype before scoring and pass the frontier signal into the scorer (additive input change
   + reorder; verified safe because progress/archetype depend only on `analysis`, not `drift_scores`).
2. The two new process dimensions are `DeadEndThrash` score/state/evidence, not new `DriftClass` variants
   (SPEC Resolved Decision 1) — additive, no sentinel/schema churn.
3. `R6-1` reads no objective text or structured objective (SPEC Resolved Decision 2). Objective integration
   is `R6-2`.
4. The frozen `dead_end_thrash` acceptance corpus is the regression floor: the cutover must not move
   `019e93fa`/`019e940c`/`019e943c` off `cleared`/`0` or `019e894a` off `recovered`/`20`/`flagged=false`.
5. The exact "advancing vs stalled" predicate and whether the decisive-step score replaces or re-weights
   the streak bands are resolved empirically against the corpus before the scorer change is committed.
6. Packet-prompt rule: verify `R5.75` is landed before editing (it is, at HEAD).

## Why This Packet Exists

The original `R6` intent is to move scoring from "how long was the flagged streak?" toward "what was the
decisive bad step?" (the AgentRx framing). The live `score_dead_end_thrash` scores on streak length
(`repeated_verification_loops.len() * 40 + repeated_failure_loops.len() * 30`) gated only by
`recovery.active_repeated_*`. That over-flags long troubleshooting sessions whose frontier is actually
advancing, and under-weights a single decisive stall. The analyzer already computes frontier movement in
`progress.rs` (`frontier_advanced`, the private `frontier_rank`), so the cutover reuses that judgment
rather than inventing one. But the signal is local to `build_session_progress` (surfaced only as
`SessionProgress.signals`), is not a `CheckpointAnalysis` field, and is built *after* scoring today, so
the cutover is a re-scoring **plus** a bounded access-path change (reorder + additive scorer input) to
make the existing signal reachable — not a zero-plumbing read. The frontier *model* itself is unchanged.

## Dependency Graph

```text
docs lock (this SPEC/PLAN/TASKS)
  -> R6-1.1 decision: frontier-access path (reorder + pass signal in) + which predicate separates
     advancing vs stalled, without moving the corpus
  -> dead_end_thrash cutover (frontier-aware decisive-step scoring; the two states)
  -> churn-vs-stall regressions + frozen-corpus invariance
  -> Guardrail-5 bridge coverage
  -> R5.75-3/R5.75-4 non-regression + full + sentinel walls

explicitly deferred / out of scope:
  -> any structured-objective read (R6-2)
  -> new DriftClass variant (ask-first)
  -> progress.rs frontier-model changes (read-only here)
```

## Recommended Landing Sequence

## R6-1.0: Docs Lock (This SPEC / PLAN / TASKS)

### Scope

- commit the bounded SPEC/PLAN/TASKS family under `docs/specs/r6/R6-1/`
- record the objective-independence boundary, the within-class modeling decision, and the corpus floor

### Why First

The MAP/DESIGN fix the family decision; the per-packet scoring contract (frontier-aware, within-class,
corpus floor) must be explicit before editing the scorer.

### Verification

Manual review against `docs/specs/r6/MAP.md`, the DESIGN doc, and the live `dead_end_thrash.rs`.

## R6-1.1: Choose The Frontier-Access Path And Predicate

### Scope

- **Decide the access path.** The frontier judgment is not on `CheckpointAnalysis` and
  `build_session_progress` runs after `score_session`, so pick how the scorer reaches it. Default
  recommendation: compute `build_session_progress`/`build_session_archetype` before scoring and pass the
  frontier signal (or `SessionProgress`) into `score_dead_end_thrash` — single source of truth, additive
  input change, reorder is safe because both depend only on `analysis`, not `drift_scores` (confirmed at
  `checkpoint/mod.rs`). Reject recomputing the predicate inside the scorer (divergence). Record the chosen
  path and its surface change in the TASKS ledger.
- **Decide the predicate.** Determine, empirically, which frontier signal from `SessionProgress` cleanly
  expresses "frontier advanced in this interval" vs "no frontier movement" — investigation only, no
  committed code. Note the frontier judgment is **not** a single standalone flag: it is produced by
  archetype-specific progress assessment with a zero-verifier fallback and delegation visibility caps
  (`build_session_progress`, `progress.rs:30-82`; archetype at `mod.rs:430`). The chosen signal must not
  read a fallback lane or a delegation cap as a genuine advance, or churn-suppression will fire on a
  session that did not actually move its frontier. Pin the predicate against the corpus + `R5.75-3`
  (delegated) / `R5.75-4` (zero-verifier) witnesses specifically.
- Confirm the candidate path + predicate leave the frozen replay corpus and the `R5.75` witnesses unmoved.

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/progress.rs     (read-only: the frontier judgment to reuse)
crates/agent-drift-analyzer/src/checkpoint/mod.rs          (read-only: pipeline order, reorder target)
crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs (read-only)
crates/agent-drift-analyzer/tests/acceptance_fixtures.rs   (the corpus to hold)
```

### Why Before The Cutover

The size and surface of `R6-1.2` depend on the access path and which signal is authoritative for frontier
movement. The wrong assumption here is what made the first SPEC draft claim a zero-plumbing read; decide
with evidence so the cutover neither moves the corpus nor smuggles in an unbudgeted recompute.

### Verification

Local experiment + recorded finding in the TASKS ledger (access path + predicate); no committed code from
this step.

## R6-1.2: Dead-End-Thrash Cutover

### Scope

- run `gitnexus_impact` on `score_dead_end_thrash` and report the blast radius before editing
- re-score so an advancing frontier suppresses the dead-end flag (expected churn → `flagged=false`,
  historical context), and a stall (no frontier movement) flags with stall-named evidence, scored by
  decisiveness rather than streak length alone — guarding reclassification on the existing active-thrash
  condition so cleared controls keep their `cleared`/`0` posture
- keep `DriftScore`'s output shape and the `DriftClass` enum unchanged (the scorer **input** may change
  additively only via the `R6-1.1` access path); attach named evidence to every decision
- add the minimal churn-vs-stall regression for this slice (TDD); the full matrix is `R6-1.3`

### Primary Files

```text
crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs
crates/agent-drift-analyzer/tests/dead_end_thrash.rs (minimal churn-vs-stall proof only)
```

### Verification

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer --test acceptance_fixtures -- --nocapture
```

## R6-1.3: Regressions And Corpus Invariance

### Scope

- complete the churn-vs-stall matrix in `tests/dead_end_thrash.rs` (advancing-frontier-not-flagged,
  no-movement-flagged, decisive single-step)
- assert the frozen corpus posture is unchanged in `tests/acceptance_fixtures.rs`
- assert `R5.75-3`/`R5.75-4` witnesses unchanged in `tests/checkpoints.rs` / `tests/progress_acceptance.rs`

### Primary Files

```text
crates/agent-drift-analyzer/tests/dead_end_thrash.rs
crates/agent-drift-analyzer/tests/acceptance_fixtures.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
crates/agent-drift-analyzer/tests/progress_acceptance.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

## R6-1.4: Guardrail-5 Bridge Coverage

### Scope

- add regressions that exercise the `R5.75-6` bridge needle lists (`anchor_text_looks_grounded_goal`,
  `narrowed_objective_looks_subordinate`) so a phrasing the bridge misses is caught at the analyzer wall
- record string-truth-on-the-legacy-surface as explicit migration debt in this packet's closeout

### Primary Files

```text
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## R6-1.5: Smoke And Closeout

### Scope

- run the full analyzer wall + touched sentinel spot-checks
- update the MAP `R6-1` status and route to `R6-2`

### Primary Files

```text
docs/specs/r6/MAP.md   (status/routing only at closeout)
```

### Verification

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

## Risks And Mitigations

### Risk: the cutover moves the frozen replay corpus

Mitigation: `R6-1.1` probes the frontier predicate against the corpus before any code lands; `R6-1.3`
asserts corpus invariance. The corpus is the regression floor.

### Risk: suppressing the flag on advancing frontiers hides a real dead end

Mitigation: suppression requires frontier *advance* in the interval; a stall (no movement) still flags.
The decisive-step evidence names which case fired, so operators can audit it.

### Risk: re-weighting raw_score breaks operator/sentinel expectations

Mitigation: `raw_score` band changes are Ask-First (SPEC Boundaries); the sentinel spot-checks
(`warning_policy`, `live_end_to_end`) gate closeout.

### Risk: scope creep into objective reads, the reset migration, or progress.rs frontier changes

Mitigation: Boundaries forbid all three, and each has a distinct home so they are not conflated.
Structured-objective *reads* are `R6-2`. The `progress.rs` reset/comparability migration onto
`comparison_key` is the conditional `R6-3` (DESIGN "Packet Decomposition"), not this packet. The
`progress.rs` frontier *computation* stays read-only here — `R6-1` may expose/plumb the already-computed
signal per `R6-1.1`, but editing the frontier model itself is Ask-First.

## Verification Checkpoints

1. **After R6-1.1** — recorded finding: the frontier predicate that separates advancing vs stalled
   without moving the corpus.
2. **After R6-1.2** — churn-vs-stall minimal proof passes; corpus unchanged.
3. **After R6-1.3/1.4** — full churn-vs-stall matrix, corpus invariance, `R5.75` non-regression, and the
   bridge coverage are green.
4. **Packet closeout** — full analyzer wall + sentinel spot-checks green; MAP routed to `R6-2`.

## Out Of Scope

- any structured-objective read (that is `R6-2`) or the reset/comparability migration onto
  `comparison_key` (the conditional `R6-3`)
- a new `DriftClass` variant (ask-first)
- changes to `progress.rs` frontier computation (read-only consumer here)
- `wrong_plan_branch` / `truth_grounding_gap` rescoring (only if obvious, and not in this packet)
- learned/hybrid scoring monitors
