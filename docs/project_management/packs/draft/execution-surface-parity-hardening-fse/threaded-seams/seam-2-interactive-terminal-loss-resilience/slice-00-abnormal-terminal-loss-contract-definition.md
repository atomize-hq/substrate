---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
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
contracts_produced:
  - C-03
contracts_consumed: []
open_remediations: []
---
### S00 - Abnormal terminal-loss contract definition

- **User/system value**: implementation work proceeds from one explicit producer-side contract for abnormal terminal loss instead of ad-hoc prompt error handling or test-specific assumptions.
- **Scope (in/out)**:
  - In: define `C-03` rules, boundaries, and verification checklists tightly enough that the producer seam can later satisfy `gates.pre_exec.contract`.
  - Out: final publication evidence and closeout accounting, which belong to `S99`.
- **Acceptance criteria**:
  - `C-03` names the abnormal trigger boundary for controlling-TTY revoke or disconnect after startup.
  - `C-03` states the required unwind behavior, the singular stderr posture, the exit code split (`0` for normal exit, `1` for abnormal terminal loss), and the bounded cleanup expectations.
  - `C-03` includes explicit runtime and regression-proof surfaces with pass/fail conditions.
- **Dependencies**:
  - `threading.md` contract registry for `C-03` and `THR-02`
  - `crates/shell/src/repl/async_repl.rs`
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - existing PTY-based REPL harnesses under `crates/shell/tests/`
- **Verification**:
  - contract rules below must map directly to executable runtime checks or deterministic revoke/disconnect probes in later slices.
- **Rollout/safety**: no new public flags or config keys; keep the contract narrow to abnormal post-startup terminal loss on the async REPL path.
- **Review surface refs**: `../../review_surfaces.md` R4 and R5

#### C-03 contract rules

1. **Authority**: the async REPL path owns abnormal controlling-TTY loss handling; it must not rely on `CtrlD` semantics or generic prompt-error printing to define operator-visible behavior.
2. **Trigger boundary**:
   - applies only after REPL startup succeeds and the process is actively using the async Reedline-backed input path
   - covers controlling-TTY revoke or disconnect conditions concrete enough to invalidate interactive input
   - does not redefine normal operator `exit`, `quit`, or ordinary `CtrlD`
3. **Runtime outcome**:
   - prompt-worker unwind is required even if Reedline is blocked in `read_line()`
   - one best-effort stderr diagnostic is allowed and expected
   - the REPL process exits with code `1`
   - no orphaned child or CPU-spinning process remains after the terminal-loss path
4. **Verification checklist**:
   - runtime behavior carries an explicit abnormal termination cause distinct from normal exit
   - a macOS-targeted revoke/disconnect proof exercises the Reedline path specifically and asserts bounded exit plus cleanup
   - authoritative docs describe the same exit-code and diagnostic contract without introducing broader promises than the landed runtime supports

#### S00.T1 - Record the runtime contract for abnormal terminal loss

- **Outcome**: later slices can harden the async REPL without re-deciding what counts as abnormal terminal loss or what operator-facing outcome it must produce.
- **Inputs/outputs**:
  - Inputs: `threading.md`, `crates/shell/src/repl/async_repl.rs`, `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - Outputs: locked `C-03` trigger boundary, exit-code split, diagnostic posture, and cleanup requirements
- **Thread/contract refs**: `THR-02`, `C-03`
- **Implementation notes**:
  - keep the classification tied to concrete TTY-invalid signals or failure surfaces rather than generic transient terminal state changes
  - keep the owned contract inside `SEAM-2`; downstream docs and conformance consume the published result later
- **Acceptance criteria**:
  - no later implementation slice needs to infer whether abnormal terminal loss maps to `CtrlD`, generic error, or exit code `1`
- **Test notes**:
  - use the later macOS revoke proof as the highest-confidence validation surface for the Reedline path
- **Risk/rollback notes**:
  - if runtime evidence shows the trigger boundary is broader or narrower than planned, refine the contract here rather than hiding ambiguity in test logic

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: cross-check the contract against the ADR's exit-code wording and the current async REPL control flow
- Cleanup: none

#### S00.T2 - Record the regression-proof contract for bounded cleanup

- **Outcome**: the producer seam has one explicit definition of what the macOS revoke harness must prove before closeout can publish `THR-02`.
- **Inputs/outputs**:
  - Inputs: existing PTY-based REPL tests, planned macOS revoke harness surface, `C-03`
  - Outputs: explicit proof obligations for Reedline-path coverage, bounded exit timing, and guaranteed child reap on failure paths
- **Thread/contract refs**: `THR-02`, `C-03`
- **Implementation notes**:
  - keep the proof host-only and macOS-targeted, as required by the seam brief
  - do not treat a stdio-prompt fallback or generic PTY test as sufficient evidence for this seam
- **Acceptance criteria**:
  - every closeout claim about abnormal exit, singular diagnostic posture, or cleanup has one named proof surface
- **Test notes**:
  - prefer adjacent harness reuse from current PTY REPL tests before inventing a totally separate integration harness
- **Risk/rollback notes**:
  - if the harness cannot deterministically prove cleanup, keep the issue visible in the contract and block publication rather than weakening the proof bar

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: confirm every proof obligation maps to a later runtime or regression slice
- Cleanup: none
