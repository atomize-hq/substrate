# Tasks: Agent Drift Sentinel Live Posture v0.5A

This task list implements:

- `docs/specs/agent-drift-sentinel-live-posture-v0.5A-spec.md`
- `docs/specs/agent-drift-sentinel-live-posture-v0.5A-plan.md`

## Task List

## Packet v0.5A.1: Posture Contract And Replay Presentation

- [ ] Task: Lock the sentinel posture contract and sub-packet boundary in repo docs
  - Acceptance: the docs explicitly define:
    - sentinel-owned `active`, `recovered`, and `historical-only` posture
    - posture as separate from `WarningDisposition`
    - historical-evidence detection from current analyzer reason prefixes
    - `v0.5A.1` as posture contract plus replay presentation
    - `v0.5A.2` as sink wiring, parity, and regression refresh
    - `v0.5B` as separate restart/coordinator work
  - Verify: doc review against the current analyzer checkpoint schema and sentinel presentation
    seams
  - Files:
    - `docs/specs/agent-drift-sentinel-live-posture-v0.5A-spec.md`
    - `docs/specs/agent-drift-sentinel-live-posture-v0.5A-plan.md`
    - `docs/specs/agent-drift-sentinel-live-posture-v0.5A-tasks.md`

- [ ] Task: Add one shared sentinel posture classifier and historical-evidence detector
  - Acceptance: sentinel code has one centralized posture seam that:
    - identifies `active` from flagged checkpoint/drift-score state
    - identifies `recovered` from previous-checkpoint active state plus current historical
      evidence
    - identifies `historical-only` from current historical evidence without a recovery transition
    - centralizes supported historical reason prefixes instead of duplicating them ad hoc
  - Verify: `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/operator_surface.rs`

- [ ] Task: Carry posture through replay checkpoint presentation without changing visibility rules
  - Acceptance: `CheckpointPresentation` and replay rendering expose posture separately from
    `Visible` / `Silent`, and replay output can distinguish:
    - active visible warnings
    - silent recovered checkpoints
    - silent historical-only checkpoints
    without changing warning selection, ordering, or scheduler behavior.
  - Verify: `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/operator_surface.rs`

Packet `v0.5A.1` exit condition:

- the shared posture classifier exists
- replay presentation exposes posture separately from visibility
- `operator_surface` coverage proves `active`, `recovered`, and `historical-only` behavior

## Packet v0.5A.2: Live Sink Wiring And Parity Proof

- [ ] Task: Add posture to checkpoint-bearing live operator events
  - Acceptance: `VisibleWarningEvent` and `SilentCheckpointEvent` carry posture as structured
    data, while existing event families remain stable and heartbeat/status behavior does not
    regress.
  - Verify: `cargo test -p agent-drift-sentinel operator_sink -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_sink.rs`
    - `crates/agent-drift-sentinel/tests/operator_sink.rs`

- [ ] Task: Prove replay and live posture stay aligned for the same checkpoint sequences
  - Acceptance: shared fixture-backed checkpoint sequences prove replay and live paths assign the
    same posture to matching checkpoints, including recovered and historical-only cases.
  - Verify:
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
    - `crates/agent-drift-sentinel/tests/fixtures/live/`

- [ ] Task: Revalidate the real-session live seam without broadening into coordinator work
  - Acceptance: the existing real-session live path can surface posture-bearing checkpoint output
    without requiring restart-continuity or persisted-cursor changes, and the smoke guide no
    longer describes posture only as `Visible` vs `Silent`.
  - Verify:
    - `cargo test -p agent-drift-sentinel real_session_live -- --nocapture`
    - `cargo test -p agent-drift-sentinel -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/tests/real_session_live.rs`
    - `docs/internals/testing/hybrid-drift-stack-smoke-guide.md`

Packet `v0.5A.2` exit condition:

- checkpoint-bearing live events expose the same posture contract replay already uses
- replay/live parity holds for the same ordered checkpoint sequences
- the real-session regression wall stays green without spilling into `v0.5B`
