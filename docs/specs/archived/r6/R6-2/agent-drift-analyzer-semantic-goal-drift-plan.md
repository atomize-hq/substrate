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
2. Two presence guards (SPEC Resolved Decision 5): a three-state current-goal guard (present+confident →
   score; absent → no claim; present-but-unknown → no claim) plus a separate anchor-presence guard. The
   confidence bar is anchor `High` + current `Medium`-or-`High` (both `TaskStatement`, empty `unknowns`);
   `High`-only-both-sides is dormant given the `Medium`-heavy extractor.
3. A sanctioned explicit replan is excluded via a new `CheckpointAnalysis.sanctioned_replan` field (SPEC
   Resolved Decision 4), computed from steer-row evidence at analysis-assembly time — not a threaded bool
   and not the private `progress.rs` string heuristic.
4. `progress.rs` comparability/reset is **not** migrated here (SPEC Resolved Decision 2); that is the
   conditional `R6-4`, honoring Guardrail 4.
5. Semantic goal drift surfaces as a new `DriftClass::SemanticGoalDrift` variant (SPEC Resolved
   Decision 6), landing with lockstep updates in analyzer sort order, `export.rs` class lists, and sentinel
   `operator_surface.rs`. Adding it is a hard forward-compat break (old readers fail before any
   `schema_version` gate); `R6-2.2` confirms the blast radius and the `v0.7` bump decision via
   `gitnexus_impact`. `R6-3` variant reuse is deferred to the `R6-3` spec.
6. The scorer stays rule-based and interpretable; learned monitors deferred.
7. Packet-prompt rule: verify `R5.75-1` and `R6-1` are landed before editing.
8. The kickoff anchor is *session-level* and not on `CheckpointAnalysis`, so it is threaded into
   `score_session` as additive session-running input (mirroring `previous_truth_grounding_gap` in
   `lib.rs`), never read via `session_kickoff_anchor(analysis)`. The current goal is read from
   `analysis.current`. The anchor *source* is the first confident `TaskStatement` checkpoint (the
   session-level kickoff-signal hook is disabled); `R6-2.1` confirms this empirically — it is not an
   access-path or source decision.

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
  -> kickoff-anchor capture + empirical corpus check
     (source settled: first confident TaskStatement, kickoff-signal hook disabled;
      access settled: threaded into score_session like previous_truth_grounding_gap, not read off analysis)
  -> SemanticGoalDrift variant blast-radius + schema_version confirmation (gitnexus_impact)
  -> semantic_goal_drift scorer + sidecar-presence guard (structured-state read)
  -> drift-vs-replan + structured-source regressions + acceptance fixture
  -> R5.75-3/R5.75-4 + R6-1 non-regression + full (+ sentinel) walls

explicitly deferred / out of scope:
  -> progress.rs comparability/reset migration (conditional R6-4)
  -> TaskFrame Phase-2 migration (later phase)
  -> learned/hybrid scoring
```

## Recommended Landing Sequence

## R6-2.0: Docs Lock (This SPEC / PLAN / TASKS + Packet Prompts)

### Scope

- commit the bounded SPEC/PLAN/TASKS family plus the packet-prompts artifact under
  `docs/specs/r6/R6-2/`
- record the structured-state-only drift-read contract, the current-goal three-state guard **plus** the
  separate anchor-presence guard, the sanctioned-replan exclusion via `analysis.sanctioned_replan`, and
  the no-`progress.rs` boundary
- make the packet-prompts artifact match the live packet numbering (`R6-2.0` through `R6-2.5`) and these
  source-of-truth constraints before any implementation packet starts

### Why First

The DESIGN fixes the architecture; the per-packet contract (anchor source, guard shape, sanctioned-replan
exclusion surface, packet numbering, surfacing shape) must be explicit before editing.

### Verification

Manual review against `docs/specs/r6/MAP.md`, the DESIGN doc, and live `context/objective.rs`, including
the packet-prompts numbering/constraint alignment.

## R6-2.1: Capture And Thread The Kickoff Anchor

### Scope

- **Anchor source.** Determine, from the per-checkpoint `structured_objective`, how to capture the
  session's kickoff anchor: the first confident `TaskStatement` checkpoint goal — the only concrete source,
  since the `R4` session-level kickoff-signal hook is disabled (`checkpoint/mod.rs`), so it is not a live
  alternative (the Open Question 1 corpus check confirms it holds) — investigation plus a minimal, committed
  anchor-capture helper. The anchor is read once and reused; it does not recompute objective extraction.
- **Access path (settled — thread the anchor).** The anchor is session-level and is **not** on
  `CheckpointAnalysis`, so it **is threaded** through the per-session analyze loop and passed into the
  scorer, exactly as `previous_truth_grounding_gap` is threaded today (`lib.rs`). `session_kickoff_anchor(analysis)`
  is not viable, and stamping a session-level value onto every `CheckpointAnalysis` was rejected as
  redundant. (Contrast `R6-1`: its frontier signal is checkpoint-local, so `build_scoring_session_progress`
  could derive it inside `score_session`; the anchor is session-level, so that pattern does not extend.)
  Record the source and the corpus-check finding in the TASKS ledger.
- **Confidence-bar corpus check (Open Question 1 / SPEC Resolved Decision 5).** Measure across the fixture
  corpus how often `High`-confidence `TaskStatement` anchors occur and how often current goals are `Medium`
  vs `High`. Confirm the resolved bar (anchor `High` + current `Medium+`) is not dormant; if `High` anchors
  are scarce, relax the anchor bar to `Medium+` and note the tradeoff (the distance threshold then does
  more work). Record the finding.
- **`sanctioned_replan` field (SPEC Resolved Decision 4).** Add the `CheckpointAnalysis.sanctioned_replan`
  field, computed at analysis-assembly time primarily from steer-row evidence (not the private
  `progress.rs` string heuristic). This is analysis-time infra the scorer reads in `R6-2.3`; every
  `CheckpointAnalysis` construction site (incl. test fixtures) must set it.

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs    (anchor capture from existing structured_objective)
crates/agent-drift-analyzer/src/lib.rs               (thread the running anchor into score_session, like previous_truth_grounding_gap)
crates/agent-drift-analyzer/src/scoring/mod.rs       (score_session additive anchor input)
crates/agent-drift-analyzer/src/context/objective.rs (read-only)
```

### Why Before The Scorer

The scorer's correctness depends on a stable, confident anchor *and* a real path for that anchor to reach
scoring. Resolving the source without the access path is what left the first draft asserting a
non-existent `session_kickoff_anchor(analysis)`; settle both before writing the distance logic.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## R6-2.2: Confirm The Variant Blast Radius And Schema Decision (Impact-Gated)

### Scope

- run `gitnexus_impact` on `score_session` and the `SemanticGoalDrift` `DriftClass` variant; report the
  full blast radius (analyzer sort order, `export.rs` class lists/labels, sentinel `operator_surface.rs`
  mappings, and any `schema_version` gate)
- the variant is already the decided shape (SPEC Resolved Decision 6); this step **confirms** the blast
  radius and decides the `schema_version` `v0.7` bump (honest labeling + sentinel allowlist, not compat
  protection) from that report; record the decision

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs   (DriftClass variant + any schema_version gate)
crates/agent-drift-analyzer/src/checkpoint/export.rs   (class lists/labels, lockstep)
crates/agent-drift-sentinel/src/operator_surface.rs    (mappings, lockstep)
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
crates/agent-drift-analyzer/src/checkpoint/schema.rs           (SemanticGoalDrift variant)
crates/agent-drift-analyzer/src/checkpoint/export.rs           (class lists/labels, lockstep)
crates/agent-drift-sentinel/src/operator_surface.rs            (mappings, lockstep)
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

- full analyzer wall + full sentinel walls (the `SemanticGoalDrift` variant touches the sentinel surface)
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

Mitigation: the two presence guards are the scorer's first step and each state is asserted; an
absent/unknown sidecar yields an explicit no-claim, not a string fallback.

### Risk: a sanctioned replan is flagged as drift

Mitigation: sanctioned explicit replans are excluded via the `analysis.sanctioned_replan` field (SPEC
Resolved Decision 4), computed from steer-row evidence; a drift-vs-replan regression locks it.

### Risk: a new DriftClass variant breaks the sentinel surface

Mitigation: the variant is impact-gated (`R6-2.2`) and updated in lockstep with `operator_surface.rs` and
`export.rs`; the sentinel walls gate closeout. Adding it is a forward-compat break (old readers fail at
deserialization before any `schema_version` gate), mitigated only by lockstep deploy — not by a version
bump. Falling back to evidence-within-an-existing-class is **not** an option (SPEC Resolved Decision 6).

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
