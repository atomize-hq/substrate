# Tasks: Agent Drift Analyzer Semantic Goal Drift From Kickoff/Plan/Docs (R6-2)

Status: task ledger created on 2026-06-27 from the `R6-2` SPEC/PLAN in this directory. Packet not yet
started; all tasks open. Sequenced after `R6-1`. This ledger is the closeout record as tasks land.

Packet prerequisite rule: this packet names `R5.75-1` (structured sidecar + `comparison_key_from_structured`)
and `R6-1` (dead_end_thrash cutover) as landed. Verify both in live code/tests before editing. If a named
prerequisite is missing, stop and report it instead of compensating inside this packet.

## R6-2.0: Docs Lock

- [ ] Task R6-2.0.1: Commit the SPEC/PLAN/TASKS family.
  - Acceptance: `docs/specs/r6/R6-2/` contains the spec, plan, and this tasks ledger, and they record the
    structured-state-only read contract, the three-state sidecar-presence guard, the sanctioned-replan
    exclusion, and the no-`progress.rs`-migration boundary.
  - Verify: manual review against the `R6` MAP, the DESIGN doc, and live `context/objective.rs`.
  - Files:
    - `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md`
    - `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md`
    - `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md`

## R6-2.1: Capture And Thread The Kickoff Anchor

- [ ] Task R6-2.1.1: Capture the session kickoff structured-goal anchor and decide its scorer access path.
  - Acceptance: (a) a minimal, committed helper captures the kickoff anchor from the existing per-checkpoint
    `structured_objective` (the first confident `TaskStatement` goal — the only concrete source; the
    session-level kickoff-signal hook is disabled — Open Question 1a), read once and reused, not
    recomputing objective extraction; and (b) the ledger
    records the chosen **access path** by which the scorer receives the session-level anchor (Open Question
    1b) — recommended: thread a running anchor through the per-session analyze loop into `score_session`
    (additive input, mirroring `previous_truth_grounding_gap` at `lib.rs`); fallback: capture onto
    `CheckpointAnalysis`. Reject `session_kickoff_anchor(analysis)` — the anchor is not on
    `CheckpointAnalysis`. The current goal stays read from `analysis.current`. Also: (c) record the
    confidence-bar corpus check (`High`-anchor frequency; current `Medium`-vs-`High`), confirming the
    resolved bar (anchor `High` + current `Medium+`) is not dormant — or relaxing the anchor bar to
    `Medium+` if `High` anchors are scarce (SPEC Resolved Decision 5); and (d) add a
    `CheckpointAnalysis.sanctioned_replan` field computed at analysis-assembly time from steer-row evidence
    (SPEC Resolved Decision 4), with every construction site (incl. test fixtures) setting it. Resolves SPEC
    Open Question 1 and sizes R6-2.3.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs` (anchor capture + `sanctioned_replan` field)
    - `crates/agent-drift-analyzer/src/lib.rs` (thread the anchor into `score_session`)
    - `crates/agent-drift-analyzer/src/scoring/mod.rs` (`score_session` additive anchor input)
    - (read-only) `crates/agent-drift-analyzer/src/context/objective.rs`
  - Finding: _(record anchor source + access path + confidence-bar check + `sanctioned_replan` derivation)_

## R6-2.2: Confirm The Variant Blast Radius And Schema Decision (Impact-Gated, No Scorer Code)

- [ ] Task R6-2.2.1: Confirm the `SemanticGoalDrift` variant blast radius and the `schema_version` decision.
  - Acceptance: the variant is the decided shape (SPEC Resolved Decision 6). Run `gitnexus_impact` on
    `score_session` and the `SemanticGoalDrift` variant; record the full blast radius (schema `DriftClass`,
    analyzer `export.rs` class lists/labels, analyzer sort order, sentinel `operator_surface.rs` mappings —
    `drift_class_name` / `historical_reason_prefixes` / `checkpoint_had_active_class`). From that report,
    decide the `schema_version` `v0.7` bump (honest labeling + sentinel allowlist/gate updates, not
    compat protection — old readers break at deserialization regardless). Record the decision.
  - Verify: impact report recorded in this ledger; no scorer code committed from this task.
  - Files:
    - (read-only) `crates/agent-drift-analyzer/src/checkpoint/schema.rs`,
      `crates/agent-drift-analyzer/src/checkpoint/export.rs`,
      `crates/agent-drift-sentinel/src/operator_surface.rs`
  - Finding: _(record the blast radius + schema_version decision here when complete)_

## R6-2.3: Semantic-Goal-Drift Scorer + Presence Guard

- [ ] Task R6-2.3.1: Implement the rule-based scorer with the two presence guards.
  - Acceptance: a new `scoring/semantic_goal_drift.rs` first applies the current-goal three-state guard
    (present+confident → score; absent → no claim; present-but-unknown → no claim) and the separate anchor
    guard, at the resolved confidence bar (anchor `High` + current `Medium+`, both `TaskStatement`, empty
    `unknowns` — SPEC Resolved Decision 5), then compares the current goal (from `analysis.current`) to the
    kickoff anchor (from the R6-2.1 threaded `score_session` input, not from `analysis`) over
    `comparison_key`/structured terms, excluding sanctioned explicit replans **via `analysis.sanctioned_replan`**
    (SPEC Resolved Decision 4; the field added in R6-2.1 — the private `progress.rs` detectors are not
    reachable from `score_session`), and attaches named evidence (anchor + drifted goal). It emits the new
    `DriftClass::SemanticGoalDrift` (SPEC Resolved Decision 6), wired into `score_session` per the R6-2.1
    access path. The drift signal reads structured state only — never `task_frame.objective`. Add the
    minimal presence-guard + drift-vs-replan proof here (TDD); the full matrix is R6-2.4.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (new)
    - `crates/agent-drift-analyzer/src/scoring/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs` (`SemanticGoalDrift` variant)
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs` (class lists/labels, lockstep)
    - `crates/agent-drift-sentinel/src/operator_surface.rs` (mappings, lockstep)
    - `crates/agent-drift-analyzer/tests/...` (minimal proof only)

## R6-2.4: Regressions And Acceptance Fixture

- [ ] Task R6-2.4.1: Complete the presence-guard + drift matrix (do not re-add R6-2.3's minimal proof).
  - Acceptance: tests assert all three presence-guard states (score / absent-no-claim / unknown-no-claim);
    an unauthorized pivot → flagged with anchor-naming evidence; a sanctioned explicit replan → not
    flagged; and a structured-source proof where the bridge-patched display string and the structured goal
    disagree scores off the structured goal.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/...`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R6-2.4.2: Commit a kickoff-anchored drift acceptance fixture + assert non-regression.
  - Acceptance: a committed acceptance fixture (locked like `objective_acceptance`) proves the
    kickoff-anchored drift case end to end; `R5.75-3`/`R5.75-4` witnesses and the `R6-1` `dead_end_thrash`
    posture are asserted unchanged.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/` (new acceptance fixture + harness wiring)
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`

## R6-2.5: Smoke And Closeout

- [ ] Task R6-2.5.1: Full (+ sentinel) walls, the `R6-4` open/defer decision, and MAP status update.
  - Acceptance: the full analyzer wall and the full sentinel walls (the `SemanticGoalDrift` variant touches
    the sentinel surface) are green; the closeout records whether any `R6-1`/`R6-2` replay evidence showed a `progress.rs` reset
    error caused by objective-string quality — if yes, route to the conditional `R6-4`; if no, close `R6`
    with `R6-4` deferred to the later full-migration phase. The `R6-2` entry in `docs/specs/r6/MAP.md` is
    updated with status, routing, and the `R6-4` decision.
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - `cargo test -p agent-drift-sentinel -- --nocapture` (the `SemanticGoalDrift` variant touches
      `operator_surface.rs`, so this wall is required)
  - Files:
    - `docs/specs/r6/MAP.md`

## Deferred / Ask-First

- [ ] Task R6-2.X.1: Open the conditional `R6-4` (progress.rs reset onto `comparison_key`).
  - Acceptance: only if R6-1/R6-2 replay evidence shows reset/continuity errors caused by objective-string
    quality; migrate `explicit_replan_boundary` / `material_objective_delta` onto `comparison_key` with a
    sidecar-presence guard, preserving `R5.75-3`/`R5.75-4`. Otherwise this folds into the later
    full-migration phase and is not written here.
  - Verify: to be defined when (and if) opened, with its own SPEC/PLAN/TASKS.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
