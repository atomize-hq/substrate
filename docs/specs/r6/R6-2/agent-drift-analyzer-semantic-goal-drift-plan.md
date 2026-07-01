# Plan: Agent Drift Analyzer Semantic Goal Drift From Kickoff/Plan/Docs (R6-2)

Status: draft plan created on 2026-06-27 from the `R6-2` SPEC in this directory, the resolved DESIGN Gate
0, and a read of the live structured-objective surface (`context/objective.rs`,
`checkpoint/{schema.rs,mod.rs}`) and the `DriftClass`/sentinel coupling. This plan is reviewable before
any code lands.

## Objective

Add the `semantic drift from kickoff/plan/docs` dimension as a new rule-based consumer of the structured
objective sidecar (`comparison_key` + typed goal anchor) with a mandatory sidecar-presence guard, giving
`comparison_key` its first live consumer, without touching `progress.rs` comparability/reset.

## Planning Decisions Locked For This Draft

1. The drift signal reads structured state (`StructuredObjective` / `comparison_key_from_structured`)
   only — never the `R5.75-6` bridge-patched `task_frame.objective` (SPEC Resolved Decision 1).
2. The sidecar-presence guard has three test-visible states: present+confident → score; absent → no claim;
   present-but-unknown → no claim.
3. A sanctioned explicit replan is excluded from drift via existing replan signals.
4. `progress.rs` comparability/reset is **not** migrated here (SPEC Resolved Decision 2); that is the
   conditional `R6-4`, honoring Guardrail 4.
5. Whether semantic goal drift surfaces as a new `DriftClass` variant or as evidence within an existing
   class is resolved with `gitnexus_impact` before landing, defaulting toward the additive variant.
6. The scorer stays rule-based and interpretable; learned monitors deferred.
7. Packet-prompt rule: verify `R5.75-1` and `R6-1` are landed before editing.

## Why This Packet Exists

`comparison_key` is computed by `R5.75-1` but has no live consumer; the `R6-1` cutover deliberately stays
objective-independent. The one `R6` scoring dimension that genuinely needs the structured objective is
`semantic drift from kickoff/plan/docs` — "is the agent still working the stated goal?" — which cannot be
computed honestly from the heuristic-patched display string. `R6-2` builds that dimension as a fresh,
direct consumer of the sidecar, which is the cleanest way to give `comparison_key` a first consumer
without migrating the high-risk reset surface.

## Dependency Graph

```text
docs lock (this SPEC/PLAN/TASKS)
  -> kickoff-anchor capture probe (first confident TaskStatement vs session kickoff signal)
  -> DriftClass-vs-evidence impact decision (gitnexus_impact)
  -> semantic_goal_drift scorer + sidecar-presence guard (structured-state read)
  -> drift-vs-replan + structured-source regressions + acceptance fixture
  -> R5.75-3/R5.75-4 + R6-1 non-regression + full (+ sentinel) walls

explicitly deferred / out of scope:
  -> progress.rs comparability/reset migration (conditional R6-4)
  -> TaskFrame Phase-2 migration (later phase)
  -> learned/hybrid scoring
```

## Recommended Landing Sequence

## R6-2.0: Docs Lock (This SPEC / PLAN / TASKS)

### Scope

- commit the bounded SPEC/PLAN/TASKS family under `docs/specs/r6/R6-2/`
- record the structured-state-only contract, the three-state presence guard, and the no-`progress.rs`
  boundary

### Why First

The DESIGN fixes the architecture; the per-packet contract (anchor source, presence guard, surfacing
shape) must be explicit before editing.

### Verification

Manual review against `docs/specs/r6/MAP.md`, the DESIGN doc, and live `context/objective.rs`.

## R6-2.1: Capture The Kickoff Anchor

### Scope

- determine, from the per-checkpoint `structured_objective`, how to capture the session's kickoff anchor:
  the first confident `TaskStatement` checkpoint goal vs a session-level kickoff signal — investigation
  plus a minimal, committed anchor-capture helper
- the anchor is read once and reused; it does not recompute objective extraction

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs   (anchor capture from existing structured_objective)
crates/agent-drift-analyzer/src/context/objective.rs (read-only)
```

### Why Before The Scorer

The scorer's correctness depends on a stable, confident anchor; resolve the anchor source before writing
the distance logic.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## R6-2.2: Decide The Surfacing Shape (Impact-Gated)

### Scope

- run `gitnexus_impact` on `score_session` and a candidate `SemanticGoalDrift` `DriftClass` variant;
  report the blast radius (schema, sentinel `operator_surface.rs` mapping, `score_session` ordering)
- decide variant vs. evidence-within-existing-class from that report; record the decision

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs   (DriftClass, if a variant lands)
crates/agent-drift-sentinel/src/operator_surface.rs    (mapping, lockstep, if a variant lands)
```

### Why Here

Surfacing shape is a cross-crate decision; settle it with impact evidence before the scorer is written, so
the scorer targets the chosen shape.

### Verification

Impact report recorded in the TASKS ledger; no scorer code from this step.

## R6-2.3: Semantic-Goal-Drift Scorer + Presence Guard

### Scope

- add `scoring/semantic_goal_drift.rs`: the three-state sidecar-presence guard, then a structured-state
  distance over `comparison_key`/structured terms between the current goal and the kickoff anchor, with
  sanctioned-replan exclusion
- wire it into `score_session` per the R6-2.2 decision; attach named evidence (anchor + drifted goal)
- add the minimal presence-guard + drift-vs-replan proof here (TDD); the full matrix is R6-2.4

### Primary Files

```text
crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs (new)
crates/agent-drift-analyzer/src/scoring/mod.rs
crates/agent-drift-analyzer/src/checkpoint/schema.rs           (only if a variant lands)
crates/agent-drift-sentinel/src/operator_surface.rs            (only if a variant lands)
crates/agent-drift-analyzer/tests/...                          (minimal proof only)
```

### Verification

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R6-2.4: Regressions And Acceptance Fixture

### Scope

- complete the matrix: all three presence-guard states; drift-flagged vs sanctioned-replan-not-flagged;
  structured-source proof (patched string and structured goal disagree → score off structured goal)
- commit a kickoff-anchored drift acceptance fixture, locked like `objective_acceptance`
- assert `R5.75-3`/`R5.75-4` + `R6-1` non-regression

### Primary Files

```text
crates/agent-drift-analyzer/tests/ (new acceptance + scorer regressions)
crates/agent-drift-analyzer/tests/checkpoints.rs
crates/agent-drift-analyzer/tests/progress_acceptance.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

## R6-2.5: Smoke And Closeout

### Scope

- full analyzer wall + (if a variant landed) full sentinel walls
- record whether `R6-4` opens: did any `R6-1`/`R6-2` replay evidence show a `progress.rs` reset error
  caused by objective-string quality? If yes, route to `R6-4`; if no, close `R6` with `R6-4` deferred to
  the later full-migration phase
- update the MAP `R6-2` status and the `R6-4` decision

### Primary Files

```text
docs/specs/r6/MAP.md   (status/routing + R6-4 open/defer decision)
```

### Verification

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

## Risks And Mitigations

### Risk: the scorer silently falls back to the patched string when the sidecar is weak

Mitigation: the three-state presence guard is the scorer's first step and each state is asserted; an
absent/unknown sidecar yields an explicit no-claim, not a string fallback.

### Risk: a sanctioned replan is flagged as drift

Mitigation: sanctioned explicit replans are excluded via existing replan signals; a drift-vs-replan
regression locks it.

### Risk: a new DriftClass variant breaks the sentinel surface

Mitigation: the variant is impact-gated (`R6-2.2`), additive, and updated in lockstep with
`operator_surface.rs`; the sentinel walls gate closeout. If the impact is judged too broad, fall back to
evidence-within-existing-class.

### Risk: scope creep into the progress.rs reset migration

Mitigation: Boundaries forbid `progress.rs` comparability/reset changes here; that is the conditional
`R6-4`, evidence-gated, honoring Guardrail 4.

## Verification Checkpoints

1. **After R6-2.1** — the kickoff-anchor source is fixed and stable.
2. **After R6-2.2** — the surfacing-shape decision is recorded with its impact report.
3. **After R6-2.3** — the presence guard + minimal drift-vs-replan proof pass; structured-state read only.
4. **After R6-2.4** — full presence-guard matrix, structured-source proof, acceptance fixture, and
   `R5.75`/`R6-1` non-regression green.
5. **Packet closeout** — full analyzer (+ sentinel) walls green; the `R6-4` open/defer decision recorded.

## Out Of Scope

- `progress.rs` comparability/reset migration onto `comparison_key` (conditional `R6-4`)
- `TaskFrame` Phase-2 coexistence migration (later phase)
- `dead_end_thrash` / `wrong_plan_branch` / `truth_grounding_gap` rescoring (R6-1 / not this packet)
- learned/hybrid scoring monitors
