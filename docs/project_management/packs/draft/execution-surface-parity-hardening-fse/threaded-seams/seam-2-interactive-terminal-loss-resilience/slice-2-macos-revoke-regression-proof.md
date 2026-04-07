---
slice_id: S2
seam_id: SEAM-2
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
contracts_produced: []
contracts_consumed:
  - C-03
open_remediations: []
---
### S2 - macOS revoke regression proof

- **User/system value**: the pack gets one deterministic proof that the exact Reedline-backed failure mode is fixed, instead of relying on informal reproduction or broader PTY smoke tests.
- **Scope (in/out)**:
  - In:
    - add a macOS-targeted revoke or disconnect regression harness that drives the Reedline path
    - assert bounded exit timing, exit code `1`, one abnormal stderr diagnostic, and reliable child reap on failure paths
    - reuse existing PTY REPL harness patterns where possible
  - Out:
    - generic cross-platform terminal-loss coverage
    - downstream docs-only lock-in or broader shell resilience initiatives
- **Acceptance criteria**:
  - the harness exercises the async Reedline path rather than the stdio prompt-worker fallback
  - the harness fails if the REPL exits `0`, hangs, or leaves an unreaped child behind
  - the harness records enough evidence to support `THR-02` publication in closeout
- **Dependencies**:
  - `C-03`
  - `THR-02`
  - landed runtime behavior from `S1`
  - existing PTY harnesses in `crates/shell/tests/repl_world_first_routing_v1.rs` and `crates/shell/tests/repl_world_first_rendering_v1.rs`
  - a new or adjacent macOS-targeted test surface under `crates/shell/tests/`
- **Verification**:
  - targeted macOS regression run plus cleanup assertions for timeout and failure paths
  - readback of harness evidence against the `C-03` proof checklist
- **Rollout/safety**: keep the proof scoped to the bounded revoke/disconnect path so it does not become a catch-all PTY test bucket.
- **Review surface refs**: `../../review_surfaces.md` R4 and R5

#### S2.T1 - Build the revoke/disconnect harness on top of existing PTY REPL test patterns

- **Outcome**: the seam gains a reproducible host-only test that reaches the actual Reedline failure mode.
- **Inputs/outputs**:
  - Inputs: existing PTY test helpers, macOS revoke mechanics, `C-03`
  - Outputs: a new or adjacent regression test that drives async REPL startup, revokes/disconnects the controlling TTY, and captures process outcome
- **Thread/contract refs**: `THR-02`, `C-03`
- **Implementation notes**:
  - keep the harness close to the existing REPL PTY tests so cleanup and query-response helpers are reused
  - explicitly guard against silently taking the stdio prompt-worker fallback path
- **Acceptance criteria**:
  - the test can prove it exercised the Reedline path and the revoke/disconnect transition
- **Test notes**:
  - include bounded timeout behavior so hangs become deterministic failures
- **Risk/rollback notes**:
  - if the revoke primitive is too brittle for broad CI, keep the proof macOS-targeted and document the exact execution environment required

Checklist:
- Implement: targeted revoke/disconnect harness
- Test: run the macOS-targeted regression
- Validate: confirm the harness reaches the Reedline path
- Cleanup: reuse or factor shared PTY helpers where it reduces test drift

#### S2.T2 - Prove bounded cleanup and abnormal exit semantics

- **Outcome**: closeout can rely on concrete evidence that the bad path no longer leaks a spinning or unreaped process.
- **Inputs/outputs**:
  - Inputs: revoke harness from `S2.T1`, landed runtime from `S1`
  - Outputs: assertions for exit code `1`, singular diagnostic posture, bounded exit time, and child reap/cleanup guarantees
- **Thread/contract refs**: `THR-02`, `C-03`
- **Implementation notes**:
  - make cleanup assertions first-class; the seam brief treats orphaned-child behavior as part of the bug, not as incidental fallout
  - keep stderr assertions bounded to the intended diagnostic contract rather than brittle full-text snapshots
- **Acceptance criteria**:
  - the test fails if the REPL still hangs, returns success, or leaves child cleanup ambiguous
- **Test notes**:
  - capture exact commands, timeout thresholds, and child-reap signals for later closeout evidence
- **Risk/rollback notes**:
  - if cleanup cannot be observed directly, add the smallest necessary harness instrumentation rather than weakening the proof requirement

Checklist:
- Implement: bounded cleanup assertions
- Test: rerun the macOS regression until failure modes are deterministic
- Validate: compare evidence to `C-03`
- Cleanup: remove any temporary debug logging before landing
