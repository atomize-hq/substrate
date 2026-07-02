# Plan: Agent Drift Analyzer Rolling / Previous-Checkpoint Semantic Goal Drift (R6-3)

Status: draft plan created on 2026-07-02 from the `R6-3` SPEC in this directory, the `R6-3` charter in the
DESIGN doc ("Packet Decomposition") and `docs/specs/r6/MAP.md` item 6, a `codex exec` consult on the
surfacing decision (session `019f2399`), and a read of the landed `R6-2` scorer
(`crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`, `scoring/mod.rs`) and
`CheckpointAnalysis` (`checkpoint/mod.rs`). This plan is reviewable before any code lands.

## Objective

Add the **rolling / previous-checkpoint** semantic-goal-drift signal — the current checkpoint's structured
goal vs the immediately-previous checkpoint's — as tagged evidence on the existing
`DriftClass::SemanticGoalDrift` class, reusing the landed disjoint-set distance primitive with symmetric
extraction and the existing `sanctioned_replan` exclusion, with no new `DriftClass` variant, no
`schema_version` bump, and no sentinel/`export.rs` source change.

## Planning Decisions Locked For This Draft

1. Surface rolling drift as tagged evidence on the existing `SemanticGoalDrift` class, **not** a new variant
   (SPEC Resolved Decision 1). The three flip-conditions that would promote it to its own variant later are
   recorded in the SPEC; none holds today.
2. No `schema_version` bump; the change is analyzer-local to `scoring/semantic_goal_drift.rs` + evidence
   (SPEC Resolved Decision 2). There is no `R6-2.2`-style cross-crate variant-blast-radius packet.
3. The previous goal is read from `analysis.previous` with no new plumbing — the scorer already receives
   `analysis` (SPEC Assumption 2). `score_semantic_goal_drift` keeps its current signature.
4. Symmetric eligibility bar: both adjacent goals must pass the landed `eligible_current_goal` bar
   (`TaskStatement`, `Medium+`, empty `unknowns`, non-empty `comparison_key`); first checkpoint /
   ineligible previous → conservative no rolling claim (SPEC Resolved Decision 3).
5. Reuse the disjoint-set distance primitive, but with symmetric extraction (`Some(summary)` on both
   sides), since both adjacent goals are full `ObjectiveSummary`s — the `R6-2` anchor asymmetry does not
   apply (SPEC Resolved Decision 4). No graduated distance here.
6. Reuse the existing `analysis.sanctioned_replan` exclusion; add no new replan detection (SPEC Resolved
   Decision 5).
7. Rolling and kickoff-anchored are computed independently in the same scorer; the class flags if either
   fires, with distinct evidence tags and co-fire allowed (SPEC Resolved Decision 6).
8. `progress.rs` comparability/reset is **not** migrated here (SPEC Resolved Decision 7); that is the
   conditional `R6-4`, honoring Guardrail 4.
9. The scorer stays rule-based and interpretable; learned monitors deferred.
10. Packet-prompt rule: verify `R6-2` (the kickoff-anchored scorer + `SemanticGoalDrift` class) is landed
    and that `score_semantic_goal_drift` matches the SPEC's described signature before editing.

## Why This Packet Exists

`R6-2` shipped the kickoff-anchored measure: current goal vs the session's origin, the *cumulative* signal
that catches gradual drift from the original ask. On its own it can miss an abrupt single-checkpoint pivot
whose accumulated distance from the origin still shares a term with the kickoff. Rolling drift is the
*step-size* complement: current goal vs the immediately-previous goal, which flags a sharp turn the
checkpoint it happens. The two are complementary, not redundant (DESIGN "do not conflate them"). Rolling is
the cheap half — the previous goal is already reachable via the `analysis.previous` slice
(`Option<CheckpointSlice>`, unwrapped to `.context.objective`) with no threading — so it extends the landed
scorer rather than adding infrastructure, and (per the codex consult) rides the existing `SemanticGoalDrift`
class as tagged evidence instead of paying a second forward-compat break for a new variant. Caveat carried
throughout: rolling reuses the binary disjoint-set distance primitive, so it inherits (and is *more* exposed
to than kickoff-anchored) the known over-fire on legitimate narrowing — adjacent checkpoints evolve more
often than kickoff-vs-current. That exposure is gated by the `R6-3.1` stop/go corpus check and locked by the
step-size-vs-evolution regression; see "Risks And Mitigations" below. Rolling is cheap to *wire*, not
automatically safe to *fire*.

## Dependency Graph

```text
docs lock (this SPEC/PLAN/TASKS)
  -> previous-checkpoint reachability + eligibility/threshold corpus check
     (reachability: analysis.previous.context.objective.structured, no new plumbing;
      surfacing settled: tagged evidence on SemanticGoalDrift, no variant/schema bump)
  -> rolling scorer extension + rolling evidence tag (symmetric extraction, sanctioned_replan reuse)
  -> step-size-vs-evolution + rolling-vs-replan + independence/co-fire regressions + acceptance fixture
  -> R6-2 kickoff non-regression + R5.75-3/R5.75-4 + R6-1 + full (+ sentinel) walls

explicitly deferred / out of scope:
  -> a separate RollingSemanticGoalDrift DriftClass variant + schema_version bump (only if a SPEC
     Resolved Decision 1 flip-condition is met)
  -> graduated/weighted semantic distance (later R6 iteration; R6-2 Resolved Decision 7)
  -> progress.rs comparability/reset migration (conditional R6-4)
  -> TaskFrame Phase-2 migration (later phase)
  -> learned/hybrid scoring
```

## Recommended Landing Sequence

## R6-3.0: Docs Lock (This SPEC / PLAN / TASKS)

### Scope

- commit the bounded SPEC/PLAN/TASKS family under `docs/specs/r6/R6-3/`
- record the evidence-not-variant surfacing decision and its flip-conditions, the no-schema-bump boundary,
  the symmetric eligibility bar, the symmetric-extraction reuse of the disjoint-set primitive, the
  `sanctioned_replan` reuse, and the independent-compute / co-fire contract
- optional parity: a `agent-drift-analyzer-rolling-semantic-goal-drift-packet-prompts.md` artifact matching
  the `R6-1`/`R6-2` four-file convention, if full parity is wanted (not required for implementation)

### Why First

The surfacing decision (evidence vs variant) is the load-bearing choice; it must be explicit, with its
flip-conditions, before editing, so the scorer targets the chosen shape and no one re-opens the variant
question mid-implementation.

### Verification

Manual review against `docs/specs/r6/MAP.md` item 6, the DESIGN `R6-3` charter, the landed `R6-2`
SPEC/scorer, and live `checkpoint/mod.rs` (`analysis.previous`, `sanctioned_replan`).

## R6-3.1: Confirm Previous-Checkpoint Reachability And The Eligibility/Threshold Corpus Check

### Scope

- **Reachability confirmation.** Verify in live code that `CheckpointAnalysis` exposes `previous` and that
  the previous goal is reachable at `analysis.previous.<slice>.context.objective` with the same
  `ObjectiveSummary` shape the current goal uses — no new plumbing, no threading. Record the exact access
  path in the TASKS ledger (do not repeat the `R6-2` first-draft mistake of asserting a non-existent
  access path).
- **Surfacing/impact confirmation (light).** Confirm the change is analyzer-local: extending
  `score_semantic_goal_drift` + adding rolling evidence prefixes touches no `DriftClass` enum, `export.rs`
  class list, or sentinel `operator_surface.rs` source, and needs no `schema_version` bump. Record this as
  the `R6-3` analogue of `R6-2.2`, but note it is a *confirmation*, not an impact-gated variant decision
  (SPEC Resolved Decisions 1-2).
- **Eligibility/threshold corpus check (Open Question 1).** Measure, across the fixture corpus, how often
  adjacent confident `TaskStatement` goals occur and how often they are fully disjoint under the reused
  distance primitive. Confirm the symmetric `Medium+` bar (SPEC Resolved Decision 3) fires on real abrupt
  pivots without over-firing on ordinary evolution. **This is a hard stop/go gate: `R6-3.2` does not write
  the scorer on the symmetric `Medium+` bar until this check comes back clean.** If abrupt pivots are too
  rare in the corpus to prove the signal, add a synthetic acceptance fixture first (as `R6-2.4` did). If
  ordinary evolution over-fires, the bar is retuned (tighten eligibility, hold `previous` to a higher bar)
  or the graduated-distance revisit is opened as a prerequisite — do not proceed to `R6-3.2` on an
  over-firing bar. Record the finding and the go/no-go decision.

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs      (read: CheckpointAnalysis.previous, sanctioned_replan)
crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs (read: eligible_current_goal, semantic_goal_diverged, goal_specific_terms)
crates/agent-drift-analyzer/src/context/objective.rs   (read-only)
```

### Why Before The Scorer

The scorer's correctness depends on a real, confirmed access path for the previous goal and on the
eligibility bar actually firing on the corpus. Settle both before writing the rolling distance logic.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Reachability access path + corpus finding + analyzer-local confirmation recorded in the TASKS ledger; no
scorer code from this step.

## R6-3.2: Rolling Scorer Extension + Evidence Tag

### Scope

- extend `scoring/semantic_goal_drift.rs`: add the previous-goal eligibility read from `analysis.previous`,
  a symmetric `rolling_goal_diverged` over `goal_specific_terms(.., Some(summary))` on both sides, the
  `sanctioned_replan` exclusion, and rolling-tagged evidence (new reason prefixes) attached to the single
  `SemanticGoalDrift` claim
- compute kickoff-anchored and rolling independently; flag the class if either fires; allow co-fire with
  distinct evidence lines; decide and assert the co-fire evidence ordering / current-goal de-dup (Open
  Question 2)
- add the minimal rolling-guard + step-size-vs-evolution + rolling-vs-replan proof here (TDD); the full
  matrix is `R6-3.3`

### Primary Files

```text
crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs (extend)
crates/agent-drift-analyzer/tests/...                          (minimal proof only)
```

Note: `scoring/mod.rs` needs **no** signature change — `score_semantic_goal_drift(analysis, kickoff_anchor)`
already receives `analysis`, which carries `.previous`.

### Verification

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R6-3.3: Regressions And Acceptance Fixture

### Scope

- complete the matrix: rolling guard states (flag / first-checkpoint-None / previous-unknown); step-size
  pivot vs slow evolution; rolling-vs-replan; independence/co-fire (kickoff-only, rolling-only, both);
  structured-source proof (patched string and structured goal disagree → score off structured goal)
- commit a rolling-drift acceptance fixture (an abrupt mid-session pivot), locked like
  `semantic_goal_drift_acceptance`
- add the sentinel rendering test: a rolling-tagged `SemanticGoalDrift` checkpoint renders its evidence
  through the operator surface (test-only, no sentinel source change), mirroring the `R6-2`
  `operator_surface_renders_flagged_semantic_goal_drift_*` coverage
- assert `R6-2` kickoff-anchored witnesses + `R5.75-3`/`R5.75-4` + `R6-1` non-regression

### Primary Files

```text
crates/agent-drift-analyzer/tests/ (new rolling acceptance + scorer regressions)
crates/agent-drift-analyzer/tests/checkpoints.rs
crates/agent-drift-sentinel/tests/operator_surface.rs (rolling evidence rendering assertion, test-only)
```

### Verification

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
```

## R6-3.4: Smoke And Closeout

### Scope

- full analyzer wall + full sentinel walls (rolling evidence renders through the sentinel surface even
  though no sentinel source changes)
- confirm no new `DriftClass` variant, `schema_version` bump, or sentinel/`export.rs` source change landed
- record whether `R6-4` opens: did any `R6-1`/`R6-2`/`R6-3` replay evidence show a `progress.rs` reset
  error caused by objective-string quality? If yes, route to `R6-4`; if no, keep `R6-4` deferred to the
  later full-migration phase
- update the MAP `R6-3` status (promote to landed) and the `R6-4` decision

### Primary Files

```text
docs/specs/r6/MAP.md   (R6-3 status/routing + R6-4 open/defer decision)
```

### Verification

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

## Risks And Mitigations

### Risk: rolling over-fires on ordinary checkpoint-to-checkpoint evolution

The disjoint-set primitive reads a legitimate narrowing as fully disjoint (`R6-2` Resolved Decision 7 v1
debt). Adjacent checkpoints evolve more often than kickoff-vs-current, so rolling could over-fire.
Mitigation: the `R6-3.1` corpus check measures adjacent-goal disjointness before the scorer is written; the
symmetric `Medium+` eligibility bar plus the `sanctioned_replan` exclusion suppress the common benign
cases; the step-size-vs-evolution regression (adjacent goals sharing one specific term → not flagged) locks
the intended behavior. If the corpus shows real over-fire, tighten eligibility or route the graduated
distance to a later `R6` iteration (do not silently ship an over-firing signal).

### Risk: the evidence-not-variant choice conflates the two signals for operators

Mitigation: distinct rolling reason prefixes keep the cumulative-vs-step-size distinction visible on the
one `SemanticGoalDrift` posture; the co-fire regression proves both tags surface; the sentinel rendering
test proves the operator surface shows the rolling reason. If a SPEC Resolved Decision 1 flip-condition
later holds (separate threshold/posture/handling, or acceptance shows aggregate reporting hides
gradual-vs-abrupt), promoting rolling to its own variant is the recorded escape hatch.

### Risk: touching the landed `R6-2` scorer regresses kickoff-anchored behavior

Mitigation: rolling and kickoff-anchored are computed independently; the independence regression
(kickoff-only checkpoint still flags with kickoff evidence, rolling silent) locks that `R6-2` behavior is
unchanged; the `R6-2` acceptance witnesses run in the non-regression wall.

### Risk: scope creep into a new variant / schema bump / progress.rs

Mitigation: Boundaries forbid a new `DriftClass` variant, `schema_version` bump, `export.rs`/sentinel
source change, and `progress.rs` comparability/reset changes here. A variant is reopened only via the
recorded flip-conditions (Ask-first); `progress.rs` stays the conditional `R6-4`, honoring Guardrail 4.

## Verification Checkpoints

1. **After R6-3.1** — the previous-goal access path is confirmed in live code, the analyzer-local (no
   variant / no schema bump) surfacing is confirmed, and the eligibility/threshold corpus finding is
   recorded.
2. **After R6-3.2** — the rolling comparison + minimal step-size / replan proof pass; structured-state read
   only; kickoff-anchored path untouched.
3. **After R6-3.3** — full rolling matrix, independence/co-fire, structured-source proof, acceptance
   fixture, sentinel rendering test, and `R6-2`/`R5.75`/`R6-1` non-regression green.
4. **Packet closeout** — full analyzer (+ sentinel) walls green; confirmed no variant/schema/sentinel-source
   change; the `R6-4` open/defer decision recorded and the MAP `R6-3` status promoted.

## Out Of Scope

- a separate `RollingSemanticGoalDrift` `DriftClass` variant + `schema_version` bump (only via a SPEC
  Resolved Decision 1 flip-condition)
- graduated/weighted semantic distance (later `R6` iteration; `R6-2` Resolved Decision 7)
- `progress.rs` comparability/reset migration onto `comparison_key` (conditional `R6-4`)
- `TaskFrame` Phase-2 coexistence migration (later phase)
- `dead_end_thrash` / `wrong_plan_branch` / `truth_grounding_gap` rescoring (R6-1 / not this packet)
- learned/hybrid scoring monitors
