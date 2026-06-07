# Tasks: Agent Drift Sentinel Trigger Headline Canonicalization R3.5

This task list implements:

- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-spec.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-plan.md`

## Task List

Completion status on `2026-06-07`:

- `R3` is landed on this worktree.
- `R3.5` authority docs are now present and intentionally use `r3_5` naming to avoid confusion
  with the already-landed `R3-5` packet from the turn-context family.
- `R4`, `R5`, `R6`, and `R7` remain outside the `R3.5` boundary.

## Packet R3.5-1: Repo Doc Contract Lock

- [x] Task: Lock the `R3.5` packet boundary and naming distinction in repo docs
  - Acceptance:
    - docs define `R3.5` as sentinel-local trigger-headline canonicalization only
    - docs explicitly keep analyzer semantics and `R4+` work out of scope
    - docs define the canonical distinction between ordinary checkpoint presentation and synthetic
      scheduler fast-path events
    - docs explicitly state that ordinary checkpoint arrivals headline `checkpoint_ready` while
      `scheduler_repeated_failure_trigger` is reserved for true synthetic scheduler fast-path
      events
    - docs explicitly note the file-naming distinction from the already-landed `R3-5` packet
  - Verify:
    - doc review against `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - doc review against `docs/specs/hybrid-drift-sentinel-implementation-order.md`
  - Files:
    - `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-spec.md`
    - `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-plan.md`
    - `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-tasks.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

## Packet R3.5-2: Canonical Checkpoint Headlines

- [ ] Task: Canonicalize ordinary checkpoint presentation across replay and live
  - Acceptance:
    - replay ordinary checkpoint presentation headlines `checkpoint_ready`
    - live ordinary checkpoint-ready presentation continues to headline `checkpoint_ready`
    - ordinary flagged checkpoints no longer render
      `scheduler_repeated_failure_trigger` solely because `checkpoint.flagged` is true
    - analyzer posture remains unchanged
  - Verify:
    - `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

## Packet R3.5-3: Fast-Path Preservation And Focused Proof

- [ ] Task: Preserve synthetic repeated-failure labeling and lock focused regressions
  - Acceptance:
    - synthetic repeated-failure events still render
      `scheduler_repeated_failure_trigger`
    - recovered and historical-only checkpoints keep their current posture rendering after the
      headline cutover
    - replay/live parity tests cover both ordinary checkpoint presentation and true fast-path
      events
  - Verify:
    - `cargo test -p agent-drift-sentinel operator_sink -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_runtime -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/live_runtime.rs`
    - `crates/agent-drift-sentinel/tests/operator_sink.rs`
    - `crates/agent-drift-sentinel/tests/live_runtime.rs`
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

Packet `R3.5` exit condition:

- matched replay/live checkpoints headline the same way
- ordinary flagged checkpoints no longer overclaim scheduler-trigger status
- synthetic repeated-failure fast paths remain visibly distinct
- analyzer posture, diagnostics, drift summary, and turn-context rendering remain intact
- the packet lands without widening into analyzer semantics or sentinel interpretation
  consolidation
