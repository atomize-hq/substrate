---
slice_id: S3
seam_id: SEAM-2
slice_kind: documentation
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
### S3 - Exit semantics publication surfaces

- **User/system value**: once the runtime and regression proof land, operators and downstream planners get one explicit abnormal-terminal-loss contract instead of inferring it from code or half-aligned planning notes.
- **Scope (in/out)**:
  - In:
    - update the small authoritative doc set that describes interactive REPL exit semantics for abnormal terminal loss
    - align the ADR, operator-facing env contract, and usage guide to the landed runtime behavior
    - capture only the publication work owned by this producer seam, leaving downstream cross-surface drift guards to `SEAM-3`
  - Out:
    - net-new runtime or harness behavior
    - broader conformance work across unrelated execution surfaces
- **Acceptance criteria**:
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`, `docs/reference/env/contract.md`, and `docs/USAGE.md` describe the same abnormal-terminal-loss contract
  - touched docs preserve the normal exit `0` versus abnormal terminal-loss `1` split without expanding scope beyond the landed Reedline/macOS proof
  - docs point at the regression proof surface and do not imply broader platform guarantees than the seam actually lands
- **Dependencies**:
  - `C-03`
  - `THR-02`
  - landed runtime and proof from `S1` and `S2`
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - `docs/reference/env/contract.md`
  - `docs/USAGE.md`
- **Verification**:
  - readback diff across all touched docs and planning surfaces
  - cross-check wording against the landed macOS revoke proof
- **Rollout/safety**: documentation must stay narrowly truthful about the abnormal terminal-loss contract and must not imply a general REPL backend redesign.
- **Review surface refs**: `../../review_surfaces.md` R4

#### S3.T1 - Publish the abnormal terminal-loss contract in operator-facing docs

- **Outcome**: operator-facing docs stop treating abnormal terminal-loss behavior as implicit ADR-only knowledge.
- **Inputs/outputs**:
  - Inputs: landed `C-03` runtime and proof, `docs/reference/env/contract.md`, `docs/USAGE.md`
  - Outputs: one explicit operator contract for abnormal terminal-loss exit code, diagnostic posture, and bounded failure semantics
- **Thread/contract refs**: `THR-02`, `C-03`
- **Acceptance criteria**:
  - docs explain the abnormal exit path without requiring readers to reverse-engineer `async_repl.rs`
- **Test notes**:
  - cross-check wording against the regression proof and runtime behavior from `S1` and `S2`

Checklist:
- Implement: operator-facing docs updates only
- Test: N/A
- Validate: compare wording to `C-03`
- Cleanup: remove stale or overly narrow exit wording from touched docs

#### S3.T2 - Align ADR and planning publication surfaces to the landed contract

- **Outcome**: the producer seam publishes one coherent planning-facing truth for downstream consumption.
- **Inputs/outputs**:
  - Inputs: landed `C-03`, `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`, seam-local planning artifacts
  - Outputs: updated ADR wording and planning references that match the landed runtime contract and proof scope
- **Thread/contract refs**: `THR-02`, `C-03`
- **Acceptance criteria**:
  - downstream planning can cite one abnormal-terminal-loss contract and one proof surface
  - ADR wording does not promise more than the macOS-targeted revoke proof or the landed runtime actually establishes
- **Test notes**:
  - verify planning-facing references match the same contract language used in operator docs

Checklist:
- Implement: ADR/planning docs updates only
- Test: N/A
- Validate: compare wording to `C-03`
- Cleanup: remove stale planning-only wording that diverges from runtime truth
