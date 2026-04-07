---
slice_id: S1
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: next
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
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
### S1 - Prompt-worker unwind and exit cause

- **User/system value**: abnormal terminal loss becomes a deterministic runtime outcome instead of an ambiguous prompt failure or a hung REPL session.
- **Scope (in/out)**:
  - In:
    - add explicit abnormal session-termination cause tracking for the async REPL path
    - make the prompt worker unwind or become safely abandonable when Reedline is blocked in `read_line()`
    - ensure the abnormal path emits one bounded stderr diagnostic and returns exit code `1`
  - Out:
    - macOS-specific revoke harness proof and documentation publication
    - broad prompt-backend rewrites or generic REPL redesign
- **Acceptance criteria**:
  - the async REPL no longer treats controlling-TTY loss as normal `CtrlD` success or an unclassified prompt error
  - prompt-worker shutdown cannot hang indefinitely waiting for a blocked Reedline read to finish
  - the landed runtime emits at most one best-effort abnormal-terminal-loss diagnostic and exits with code `1`
- **Dependencies**:
  - `C-03`
  - `THR-02`
  - `crates/shell/src/repl/async_repl.rs`
  - optionally `crates/shell/src/repl/editor.rs` if the unwind path needs prompt-backend support
- **Verification**:
  - targeted async REPL tests or harness assertions proving abnormal termination cause, exit code, and non-hanging shutdown behavior
  - manual readback against the `C-03` contract-definition checklist
- **Rollout/safety**: preserve normal operator exit semantics and keep classification tied to concrete abnormal terminal-loss conditions.
- **Review surface refs**: `../../review_surfaces.md` R4 and R5

#### S1.T1 - Add explicit abnormal terminal-loss session state

- **Outcome**: the REPL can distinguish normal `exit`/`quit` or `CtrlD` from abnormal post-startup terminal loss.
- **Inputs/outputs**:
  - Inputs: `C-03`, async REPL response handling, current prompt-worker response taxonomy
  - Outputs: explicit session termination cause or equivalent state that drives one abnormal exit path
- **Thread/contract refs**: `THR-02`, `C-03`
- **Implementation notes**:
  - keep the state machine narrow; this slice should not redefine per-command exit reporting or world-session semantics
  - make abnormal exit code handling explicit rather than deriving it from generic error printing
- **Acceptance criteria**:
  - the abnormal path is observable in code review and tests without inferring intent from unrelated error strings
- **Test notes**:
  - exercise the path that currently routes through `PromptWorkerResponse::Error`
- **Risk/rollback notes**:
  - avoid broad error bucketing that would misclassify legitimate operator-driven EOF or transient warnings as terminal loss

Checklist:
- Implement: explicit abnormal termination cause wiring
- Test: targeted async REPL assertions for exit-code selection
- Validate: compare behavior to `C-03`
- Cleanup: remove ambiguous or duplicated terminal-loss handling branches

#### S1.T2 - Make Reedline prompt-worker shutdown bounded

- **Outcome**: the host REPL can unwind cleanly even when Reedline is blocked in `read_line()`.
- **Inputs/outputs**:
  - Inputs: current `PromptWorker::shutdown()`, worker thread lifecycle, any needed editor/backend hooks
  - Outputs: bounded shutdown behavior that does not wait forever on a blocked prompt worker
- **Thread/contract refs**: `THR-02`, `C-03`
- **Implementation notes**:
  - prioritize the smallest change that guarantees prompt-worker unwind or safe abandonment on the abnormal path
  - avoid conflating this bounded fix with a general prompt-backend rewrite
- **Acceptance criteria**:
  - the async REPL can terminate promptly after terminal loss without leaving a join path blocked forever
- **Test notes**:
  - pair runtime assertions with the macOS revoke harness from `S2`
- **Risk/rollback notes**:
  - if true interruptibility is not available from Reedline, document and implement the narrowest safe alternative that still satisfies the bounded-exit contract

Checklist:
- Implement: bounded unwind path for the prompt worker
- Test: targeted prompt-worker shutdown assertions
- Validate: confirm no orphaned busy-spin or blocked join remains
- Cleanup: none
