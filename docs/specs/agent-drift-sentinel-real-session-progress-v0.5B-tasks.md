# Tasks: Agent Drift Sentinel Real Session Progress v0.5B

This task list implements:

- `docs/specs/agent-drift-sentinel-real-session-progress-v0.5B-spec.md`
- `docs/specs/agent-drift-sentinel-real-session-progress-v0.5B-plan.md`

## Task List

## Packet v0.5B.1: Live-Progress Seam

- [x] Task: Lock the real-session progress and sub-packet boundary in repo docs
  - Acceptance: the docs explicitly define:
    - `v0.5B` as coordinator/progress work inside `real_session_live.rs`
    - a dedicated internal progress/state seam
    - persisted continuity beyond only `last_delivered_cursor`
    - `v0.5B.1` as progress seam extraction/persistence
    - `v0.5B.2` as restart hardening, regression coverage, and doc refresh
    - restart hardening as separate from posture and analyzer work
  - Verify: doc review against the current live coordinator and existing `v0.5` / `v0.5A` docs
  - Files:
    - `docs/specs/agent-drift-sentinel-real-session-progress-v0.5B-spec.md`
    - `docs/specs/agent-drift-sentinel-real-session-progress-v0.5B-plan.md`
    - `docs/specs/agent-drift-sentinel-real-session-progress-v0.5B-tasks.md`

- [x] Task: Add a shared live-progress seam for coordinator continuity
  - Acceptance: sentinel code has one internal progress/state seam that owns:
    - persisted source progress
    - persisted delivery progress
    - restart restoration for the target session
    and `LiveSessionCoordinator` stops storing those rules only as loosely related fields.
  - Verify: `cargo test -p agent-drift-sentinel real_session_live -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/real_session_live.rs`
    - `crates/agent-drift-sentinel/tests/real_session_live.rs`

Packet `v0.5B.1` exit condition:

- the shared live-progress seam exists
- persisted continuity restores more than only `last_delivered_cursor`
- `real_session_live` coverage proves the new seam is wired without broadening scope

## Packet v0.5B.2: Restart Hardening And Regression Proof

- [ ] Task: Preserve deterministic restart behavior for unchanged and appended sources
  - Acceptance: restart against an unchanged rollout does not emit already-delivered checkpoints,
    and appended growth after restart emits only newer checkpoints while preserving monotonic
    emission ordering.
  - Verify: `cargo test -p agent-drift-sentinel real_session_live -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/real_session_live.rs`
    - `crates/agent-drift-sentinel/tests/real_session_live.rs`

- [ ] Task: Keep invalid persisted state fail-closed through the new progress seam
  - Acceptance: schema/session mismatch and other invalid persisted-state cases still surface
    explicit errors rather than silently resetting progress.
  - Verify: `cargo test -p agent-drift-sentinel real_session_live -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/real_session_live.rs`
    - `crates/agent-drift-sentinel/tests/real_session_live.rs`

- [ ] Task: Revalidate the bounded live seam without broadening into posture or analyzer changes
  - Acceptance: the existing live CLI path and regression wall stay green, and any continuity
    docs/tests reflect the hardened restart behavior without reopening `v0.5A` or analyzer work.
  - Verify:
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
    - `cargo test -p agent-drift-sentinel -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/tests/real_session_live.rs`
    - `docs/internals/testing/hybrid-drift-stack-smoke-guide.md`

Packet `v0.5B.2` exit condition:

- unchanged and appended restart cases are both covered
- invalid persisted-state cases remain fail-closed through the hardened seam
- the live seam remains green without spilling into posture or analyzer packets
