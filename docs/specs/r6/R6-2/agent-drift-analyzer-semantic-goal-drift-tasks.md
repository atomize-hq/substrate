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

## R6-2.1: Capture The Kickoff Anchor

- [ ] Task R6-2.1.1: Fix and capture the session kickoff structured-goal anchor.
  - Acceptance: a minimal, committed helper captures the kickoff anchor from the existing per-checkpoint
    `structured_objective` (the first confident `TaskStatement` goal, or a session-level kickoff signal —
    resolve SPEC Open Question 1), read once and reused; it does not recompute objective extraction.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - (read-only) `crates/agent-drift-analyzer/src/context/objective.rs`
  - Finding: _(record the anchor-source decision here when complete)_

## R6-2.2: Decide The Surfacing Shape (Impact-Gated, No Scorer Code)

- [ ] Task R6-2.2.1: Resolve new `DriftClass` variant vs. evidence-within-existing-class.
  - Acceptance: run `gitnexus_impact` on `score_session` and a candidate `SemanticGoalDrift` variant;
    record the blast radius (schema `DriftClass`, sentinel `operator_surface.rs` mapping —
    `drift_class_name` / `historical_reason_prefixes` / `checkpoint_had_active_class` — and `score_session`
    ordering). Decide the surfacing shape from that report and record it (default toward the additive
    variant per SPEC Assumption 4). Resolves SPEC Open Question 2.
  - Verify: impact report recorded in this ledger; no scorer code committed from this task.
  - Files:
    - (read-only) `crates/agent-drift-analyzer/src/checkpoint/schema.rs`,
      `crates/agent-drift-sentinel/src/operator_surface.rs`
  - Finding: _(record the surfacing decision + impact summary here when complete)_

## R6-2.3: Semantic-Goal-Drift Scorer + Presence Guard

- [ ] Task R6-2.3.1: Implement the rule-based scorer with the three-state sidecar-presence guard.
  - Acceptance: a new `scoring/semantic_goal_drift.rs` first applies the presence guard (present+confident
    → score; absent → no claim; present-but-unknown → no claim), then compares the current goal to the
    kickoff anchor over `comparison_key`/structured terms, excluding sanctioned explicit replans, and
    attaches named evidence (anchor + drifted goal). It is wired into `score_session` per the R6-2.2
    decision. The drift signal reads structured state only — never `task_frame.objective`. Add the minimal
    presence-guard + drift-vs-replan proof here (TDD); the full matrix is R6-2.4.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (new)
    - `crates/agent-drift-analyzer/src/scoring/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs` (only if a variant lands)
    - `crates/agent-drift-sentinel/src/operator_surface.rs` (only if a variant lands, lockstep)
    - `crates/agent-drift-analyzer/tests/...` (minimal proof only)

## R6-2.4: Regressions And Acceptance Fixture

- [ ] Task R6-2.4.1: Complete the presence-guard + drift matrix (do not re-add R6-2.3's minimal proof).
  - Acceptance: tests assert all three presence-guard states (score / absent-no-claim / unknown-no-claim);
    an unauthorized pivot → flagged with anchor-naming evidence; a sanctioned explicit replan → not
    flagged; and a structured-source proof where the bridge-patched display string and the structured goal
    disagree scores off the structured goal.
  - Verify: `cargo test -p agent-drift-analyzer semantic_goal_drift checkpoints -- --nocapture`
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

- [ ] Task R6-2.5.1: Full (+ sentinel) walls, the `R6-3` open/defer decision, and MAP status update.
  - Acceptance: the full analyzer wall and (if a `DriftClass` variant landed) the full sentinel walls are
    green; the closeout records whether any `R6-1`/`R6-2` replay evidence showed a `progress.rs` reset
    error caused by objective-string quality — if yes, route to the conditional `R6-3`; if no, close `R6`
    with `R6-3` deferred to the later full-migration phase. The `R6-2` entry in `docs/specs/r6/MAP.md` is
    updated with status, routing, and the `R6-3` decision.
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - `cargo test -p agent-drift-sentinel -- --nocapture`
  - Files:
    - `docs/specs/r6/MAP.md`

## Deferred / Ask-First

- [ ] Task R6-2.X.1: Open the conditional `R6-3` (progress.rs reset onto `comparison_key`).
  - Acceptance: only if R6-1/R6-2 replay evidence shows reset/continuity errors caused by objective-string
    quality; migrate `explicit_replan_boundary` / `material_objective_delta` onto `comparison_key` with a
    sidecar-presence guard, preserving `R5.75-3`/`R5.75-4`. Otherwise this folds into the later
    full-migration phase and is not written here.
  - Verify: to be defined when (and if) opened, with its own SPEC/PLAN/TASKS.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
