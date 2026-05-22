---
seam_id: SEAM-2
seam_slug: interactive-terminal-loss-resilience
type: capability
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads: []
  stale_triggers:
    - Reedline or crossterm prompt-worker behavior changes in async_repl.rs
    - the REPL loop or auto-sync exit-code contract changes
    - exit-code taxonomy or REPL contract docs change abnormal runtime termination wording
    - the macOS PTY revoke harness or CI environment assumptions change the Reedline path under test
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---

# SEAM-2 - Interactive terminal-loss resilience

- **Goal / value**:
  Stop async REPL terminal-loss failures from looking like success by exiting promptly, unwinding the prompt worker, and preventing orphaned CPU-spinning processes.
- **Scope**
  - In:
    - classify abnormal controlling-TTY loss distinctly from normal REPL exit
    - ensure the prompt worker can unwind even when Reedline is stuck in `read_line()`
    - track explicit REPL session termination cause so abnormal terminal loss exits with code `1`
    - add a macOS-targeted regression proof that exercises the Reedline path and reaps the child on failure
    - update the small authoritative set of docs that describe REPL exit semantics
  - Out:
    - world-service, shim, or backend transport changes
    - new CLI flags or config keys
    - a general prompt-backend rewrite or fallback strategy beyond what is required to fix the bounded failure mode
- **Primary interfaces**
  - Inputs:
    - async REPL prompt-worker lifecycle in `crates/shell/src/repl/async_repl.rs`
    - Reedline and crossterm behavior under revoked or disconnected TTYs
    - shared exit-code taxonomy and current REPL contract wording
  - Outputs:
    - abnormal terminal-loss runtime contract and diagnostic behavior
    - drop-safe regression proof for the revoke/disconnect path
- **Key invariants / rules**:
  - normal operator exit remains `0`
  - abnormal controlling-TTY loss after startup exits `1`
  - the process must not remain orphaned and CPU-pinning after terminal loss
  - the regression test must exercise the Reedline path specifically, not the stdio fallback path
- **Dependencies**
  - Direct blockers:
    - existing async REPL worker model and shutdown path
    - macOS revoke-based reproduction and the repo's existing PTY integration-test patterns
  - Transitive blockers:
    - authoritative docs that currently describe non-zero REPL exits too narrowly
    - future conformance work that will need one stable operator-facing contract to lock against
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - future shell-resilience work
    - operator-facing usage and troubleshooting guidance
- **Touch surface**:
  `crates/shell/src/repl/async_repl.rs`, optionally `crates/shell/src/repl/editor.rs`, a new `crates/shell/tests/repl_tty_disconnect_macos.rs` regression test or adjacent test support, `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`, `docs/reference/env/contract.md`, and `docs/USAGE.md`.
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - For this seam specifically: seam-local review should prove the worker can unwind from the revoke path, the process exits within a bounded timeout with code `1`, the diagnostic posture is singular and bounded, and the regression harness always reaps the child on failure.
- **Risks / unknowns**:
  - Risk: error classification alone is insufficient if the worker never returns from `read_line()`.
  - De-risk plan: treat worker unwind as part of the seam's owned contract, not an optional detail.
  - Risk: generic Unix handling introduces false positives during legitimate terminal state changes.
  - De-risk plan: keep the reproduction and highest-confidence proof on macOS while designing the runtime classification around concrete TTY-invalid signals only.
  - Risk: test cleanup gaps reintroduce the same orphaned-child pathology under failure paths.
  - De-risk plan: make child reap and timeout cleanup part of the first seam-local review focus.
- **Rollout / safety**:
  This seam is intentionally host-only and shell-local. It should improve safety by making abnormal interactive termination explicit and bounded without changing backend behavior or adding operator knobs.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `future` only because it has landed and left the forward planning window after publishing `C-03` and `THR-02`.
  - Which threads matter most: `THR-02`
  - What the first seam-local review should focus on: prompt-worker unwind strategy, exit-code separation from auto-sync, and child-cleanup guarantees in the macOS regression harness
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-03`
  - Threads likely to advance: `THR-02`
  - Review-surface areas likely to shift after landing: abnormal-terminal-loss workflow, REPL exit wording, and targeted PTY regression coverage
  - Downstream seams most likely to require revalidation: `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
