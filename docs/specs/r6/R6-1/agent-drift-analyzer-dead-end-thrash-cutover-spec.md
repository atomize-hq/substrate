# Spec: Agent Drift Analyzer Dead-End-Thrash Cutover And Frontier-Aware Process Dimensions (R6-1)

Status: draft spec created on 2026-06-27 after `R5.75` closed and `Decision Gate 0` was resolved to
scoped Option C in `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`. This spec is
the implementation authority for the `R6-1` packet defined in `docs/specs/r6/MAP.md`.

Authority order for this packet:
`docs/specs/r6/MAP.md` owns landing order and the `R6` family routing;
`docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md` owns the resolved
objective-consumption boundary (this packet is the **objective-independent** half — Option C's legacy
surface);
this SPEC/PLAN/TASKS family owns the implementation contract; the live crate
(`crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`, `src/checkpoint/progress.rs`) is the
ground truth for current behavior.

## Assumptions I'm Making

1. **`R6-1` is objective-independent and lands on the legacy surface.** The DESIGN evidence shows
   `score_dead_end_thrash` reads only repetition/recovery signals (`analysis.repetition.repeated_*_loops`,
   `analysis.recovery.active_repeated_*`, `command_observations`) — no objective text, no truth
   artifacts. This packet therefore does **not** read or migrate the structured objective: structured
   reads are `R6-2`, and the `progress.rs` reset/comparability migration onto `comparison_key` is the
   conditional `R6-4`.
2. **The frontier judgment already exists in `progress.rs` and is reused, not reinvented — but it is not
   reachable by the scorer today.** `progress.rs` already computes troubleshooting-frontier movement
   (`frontier_advanced`, the private `frontier_rank`, the frontier fallback reasons), but only as a
   *local* computation inside `build_session_progress`, surfaced as `SessionProgress.signals`. It is
   **not** a field on `CheckpointAnalysis`, and at HEAD `build_session_progress` runs *after*
   `score_session` in the pipeline (`checkpoint/mod.rs`: scoring feeds `drift_scores` in before
   `build_session_progress` at ~`:411`). So the cutover reuses the existing frontier *judgment* rather
   than inventing a new model, but making that judgment reachable at scoring time requires a bounded
   access-path change selected in `R6-1.1` (see Open Question 1). It is not a zero-plumbing read of
   `CheckpointAnalysis` as currently shaped.
3. **The two new process dimensions are modeled within the existing `DeadEndThrash` class, not as new
   `DriftClass` variants (resolved decision below).** A new `DriftClass` variant has real blast radius
   into the sentinel (`operator_surface.rs`: `drift_class_name`, `historical_reason_prefixes`,
   `checkpoint_had_active_class`). `stall without frontier movement` and `expected debugging churn with
   positive progress` are scoring *states/evidence* of dead-end-thrash, not new top-level classes, so they
   are expressed through `DeadEndThrash`'s score/state/evidence — additive, no schema or sentinel churn.
4. **The cutover is a re-scoring of an existing class, not a new pipeline.** It changes how
   `raw_score`/`flagged`/`evidence` are derived (streak-length → frontier-aware decisive-step), keeping
   `DriftScore`'s shape and the `DriftClass` enum unchanged and not touching the other two scorers. The
   one shape that may change is `score_session`/`score_dead_end_thrash`'s **input**: if `R6-1.1` selects
   the "compute progress first, pass the frontier signal into the scorer" access path, that is an
   additive input-signature change plus a pipeline reorder (verified safe below — archetype/progress
   depend only on `analysis`, not on `drift_scores`), not a change to the scorer's output vector.
5. **Guardrail-5 bridge coverage belongs here.** Because `R6-1` scorers consume the legacy
   `task_frame.objective` indirectly (via `progress.rs` reset/comparability), this packet owns the
   acceptance coverage that keeps the `R5.75-6` bridge honest (`anchor_text_looks_grounded_goal`,
   `narrowed_objective_looks_subordinate`), so a phrasing the bridge misses is caught at the analyzer
   wall, not silently mis-scored.
6. **Rule-based and interpretable only.** No learned/hybrid monitors; every score change is traceable to
   named evidence rows, matching the existing scorer style.
7. **`R5.75` behavior is preserved.** Delegated-stability (`R5.75-3`) and zero-verifier anti-flap
   (`R5.75-4`) outcomes must not regress; their named acceptance/smoke witnesses carry forward.

If any of these assumptions drift, update this spec before implementation.

## Objective

Re-score `dead_end_thrash` so it distinguishes a **decisive stall** (repeated failure/verification
activity with no frontier movement) from **expected debugging churn** (repeated activity that coexists
with positive frontier progress), instead of scoring primarily on streak length. Add the two
objective-independent process dimensions the original `R6` intent named — `stall without frontier
movement` and `expected debugging churn with positive progress` — as frontier-aware states of the
existing `DeadEndThrash` class. Land the Guardrail-5 acceptance coverage for the `R5.75-6` objective
bridge.

Primary users:

1. operators reading drift output, who currently see `dead_end_thrash` flagged on long-but-progressing
   troubleshooting sessions that are actually advancing;
2. the sentinel/replay path, which should receive a `dead_end_thrash` posture that reflects whether the
   frontier moved, not just how long the streak was;
3. later `R6` packets (`R6-2`), which build on a scorer surface that is already frontier-honest.

This packet succeeds when:

1. a troubleshooting session whose frontier advances (expected debugging churn with positive progress)
   is **not** flagged as `dead_end_thrash`, even with repeated failures;
2. a session that repeats activity with **no** frontier movement (a decisive stall) is flagged with
   evidence naming the stall, not merely the streak length;
3. the known `dead_end_thrash` replay artifacts keep their honest posture — the cleared controls
   (`019e93fa`, `019e940c`, `019e943c`) stay `cleared`/`raw_score=0`, and the recovered sticky
   `019e894a` stays `recovered`/`raw_score=20`/`flagged=false`;
4. `R5.75-3`/`R5.75-4` outcomes do not regress;
5. the `R5.75-6` bridge needle lists are exercised by committed regressions;
6. no `DriftClass` variant is added and no schema version is bumped.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Live code seams for this packet:
  - `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs` (`score_dead_end_thrash`,
    `active_raw_score`, the evidence builders)
  - `crates/agent-drift-analyzer/src/checkpoint/progress.rs` (read-only: the existing
    `frontier_advanced` / `frontier_rank` / troubleshooting-frontier signals the cutover consumes)
  - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
  - `crates/agent-drift-analyzer/tests/checkpoints.rs` (Guardrail-5 bridge coverage)
  - `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs` (the frozen `dead_end_thrash` replay corpus)
- Touched sentinel spot-checks: `agent-drift-sentinel` `warning_policy` / `live_end_to_end` (the exported
  `dead_end_thrash` posture changes for churn-vs-stall sessions).

No structured-objective read, no `DriftClass` variant, no schema bump, no compactor change belongs in
this packet.

## Commands

Focused scorer + checkpoint regressions:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test acceptance_fixtures -- --nocapture
```

Full analyzer wall for closeout:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Touched-surface sentinel spot-checks:

```bash
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

## Project Structure

```text
docs/specs/r6/MAP.md
  Landing-order authority for the R6 family.
docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
  Resolved Decision Gate 0; R6-1 is the objective-independent half.
docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-{spec,plan,tasks}.md
  This packet's implementation authority.

crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs
  score_dead_end_thrash: the cutover lands here (frontier-aware decisive-step scoring + the two states).
crates/agent-drift-analyzer/src/checkpoint/progress.rs
  Read-only source of frontier-movement signals the cutover consumes.
crates/agent-drift-analyzer/tests/dead_end_thrash.rs
  Churn-vs-stall regressions.
crates/agent-drift-analyzer/tests/acceptance_fixtures.rs
  The frozen replay corpus must keep its honest dead_end_thrash posture.
crates/agent-drift-analyzer/tests/checkpoints.rs
  Guardrail-5 coverage of the R5.75-6 bridge needle lists.
```

## Code Style

Keep the scorer rule-based and evidence-first. Gate the flag on frontier movement, and attach an
explicit reason so the state is auditable.

Frontier reclassification only applies to sessions that would **otherwise flag as active thrash**.
Sessions with no active repeated thrash (the cleared controls) must fall through to the existing logic
and keep their `cleared`/`0` posture untouched — a `raw_score=20` + `DriftStateHint::HistoricalContext`
result resolves to `historical_only`/`recovered`, **never** `cleared` (`drift_state_for_score`,
`checkpoint/mod.rs`), so entering the suppression branch on a control would move the frozen corpus. Guard
it on the existing active-thrash condition first:

```rust
// `frontier` is the frontier signal made reachable per R6-1.1 (e.g. the SessionProgress passed in),
// not a recompute. `would_flag_active` is the *existing* active-thrash gate, so cleared controls
// never enter the reclassification branches and stay cleared/0.
let would_flag_active = has_repeated_activity
    && (analysis.recovery.active_repeated_failure
        || analysis.recovery.active_repeated_verification);

if would_flag_active && frontier_advanced_in_interval(frontier) {
    // Expected debugging churn with positive progress → NOT thrash.
    // Repeated failures that coexist with an advancing frontier are normal troubleshooting,
    // so suppress the dead-end flag and record why.
    return ScoredDrift::new(
        DriftScore {
            class: DriftClass::DeadEndThrash,
            flagged: false,
            raw_score: 20, // historical context, not an active dead end
            evidence: churn_with_progress_evidence(analysis),
            ..
        },
        DriftStateHint::HistoricalContext,
    );
}

if would_flag_active && !frontier_advanced_in_interval(frontier) {
    // Stall without frontier movement → decisive dead end.
    // flagged with stall-named evidence, scored by decisiveness, not streak length alone
}

// else: fall through to the existing cleared/recovered logic — corpus controls stay cleared/0.
```

Conventions for this packet:

- Consume the existing `progress.rs` frontier *judgment* via the access path selected in `R6-1.1`; do
  not recompute or duplicate the frontier model inside the scorer (single source of truth is
  `build_session_progress`).
- Model the two dimensions as `DeadEndThrash` score/state/evidence; do not add a `DriftClass` variant.
- Every flag/suppress decision carries a named `EvidenceRef` reason, matching the current evidence style.
- Do not read `task_frame.objective` or the structured objective here.

## Testing Strategy

1. **Churn-vs-stall scorer regressions** (`tests/dead_end_thrash.rs`):
   - repeated failures + advancing frontier → `flagged=false` with churn-with-progress evidence;
   - repeated activity + no frontier movement → flagged, evidence names the stall;
   - a single decisive stuck step is scored as decisive, not diluted by streak length.
2. **Replay-corpus invariance** (`tests/acceptance_fixtures.rs`): the frozen corpus keeps posture —
   `019e93fa`/`019e940c`/`019e943c` `cleared`/`0`; `019e894a` `recovered`/`20`/`flagged=false`.
3. **`R5.75` non-regression** (`tests/checkpoints.rs`, `tests/progress_acceptance.rs`): delegated-stability
   and zero-verifier anti-flap witnesses unchanged.
4. **Guardrail-5 bridge coverage** (`tests/checkpoints.rs`): regressions that exercise
   `anchor_text_looks_grounded_goal` / `narrowed_objective_looks_subordinate` so a novel phrasing the
   bridge misses fails the wall.
5. **Full analyzer + touched sentinel walls.**

## Boundaries

- **Always:**
  - verify `R5.75` is landed in live code/tests before editing (it is, at HEAD); if a named prerequisite
    were missing, stop and report rather than compensating here;
  - keep `DriftScore`'s output shape and the `DriftClass` enum unchanged, and do not alter the other two
    scorers; `score_session`/`score_dead_end_thrash`'s **input** may change additively only via the
    `R6-1.1`-selected frontier-access path (and the matching pipeline reorder), nothing more;
  - attach named evidence to every score/flag decision;
  - run `gitnexus_impact` on `score_dead_end_thrash` before editing it and report the blast radius;
  - run the focused scorer + acceptance walls before calling the packet complete.
- **Ask first:**
  - introducing a new `DriftClass` variant for either process dimension (default is to model within
    `DeadEndThrash`);
  - any change to how `progress.rs` *computes* frontier movement (this packet reuses that judgment as-is;
    exposing/plumbing the already-computed signal so the scorer can read it is in scope, but editing the
    frontier model itself is Ask-First);
  - changing the numeric `raw_score` bands the sentinel/operator surface keys off.
- **Never:**
  - read or migrate the structured objective here (structured reads are `R6-2`; the reset/comparability
    migration onto `comparison_key` is the conditional `R6-4`);
  - regress `R5.75-3`/`R5.75-4` behavior;
  - add a schema version bump or a learned scoring path;
  - flag thrash on a session whose frontier is advancing.

## Success Criteria

1. Expected debugging churn with positive frontier progress is not flagged as `dead_end_thrash`; a
   decisive stall with no frontier movement is, with stall-named evidence.
2. The frozen replay corpus keeps its honest posture (controls cleared/0; sticky recovered/20/unflagged).
3. `R5.75-3`/`R5.75-4` witnesses do not regress.
4. The `R5.75-6` bridge needle lists are exercised by committed regressions.
5. `cargo test -p agent-drift-analyzer -- --nocapture` and the touched sentinel spot-checks are green; no
   `DriftClass` variant added, no schema version bumped.

## Resolved Decisions

1. **Resolved (2026-06-27): the two process dimensions are `DeadEndThrash` states, not new `DriftClass`
   variants.** A new variant would touch the sentinel `operator_surface.rs` mapping (`drift_class_name`,
   `historical_reason_prefixes`, `checkpoint_had_active_class`) and the schema, for no semantic gain —
   "stall" and "expected churn" are dead-end-thrash sub-states. Revisit only if operators need them as
   distinct top-level classes (Ask-First).
2. **Resolved (2026-06-27): `R6-1` stays objective-independent.** Per the DESIGN evidence, the scorer
   does not depend on objective quality; the structured-objective integration is `R6-2`. This packet must
   not pull objective reads forward.

## Open Questions

1. **Two coupled sub-questions, both resolved in PLAN step `R6-1.1` before the scorer change is locked:**
   (a) *Access path* — how does the existing frontier judgment become reachable at scoring time, given it
   is not on `CheckpointAnalysis` and `build_session_progress` runs after `score_session` today? Options:
   (i) compute `build_session_progress`/archetype before scoring and pass the frontier signal into
   `score_dead_end_thrash` (recommended — single source of truth, additive input change, reorder verified
   safe since progress/archetype depend only on `analysis`); (ii) add a frontier field to
   `CheckpointAnalysis` populated before scoring (requires extracting the predicate, divergence risk);
   (iii) recompute in the scorer (rejected — duplicates the model). (b) *Predicate* — which exact frontier
   signal cleanly separates "advancing" from "stalled" without regressing the frozen corpus or the
   `R5.75` witnesses. The frontier judgment is archetype-specific and carries a zero-verifier fallback and
   delegation visibility caps; the chosen signal must not read a fallback lane or a delegation cap as a
   genuine advance, so pin it against the `R5.75-3` (delegated) and `R5.75-4` (zero-verifier) witnesses
   specifically, not just the cleared/sticky corpus.
2. Should the decisive-step raw score replace or only re-weight the existing streak-length bands
   (`loops*40` / `loops*30`)? (Resolve in `R6-1.2` against the corpus so operator-facing scores stay
   meaningful.)
