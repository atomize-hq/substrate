---
slice_id: S2
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
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
  - THR-01
contracts_produced: []
contracts_consumed:
  - C-02
open_remediations: []
---
### S2 - Tracing behavior matrix

- **User/system value**: maintainers and tests get one explicit answer for what tracing behavior to expect by mode and platform, instead of relying on ambiguous WPEP Case B assumptions.
- **Scope (in/out)**:
  - In:
    - define and land the runtime-facing behavior matrix for `world_process_*`, `builtin_command`, and `SUBSTRATE_ENABLE_PREEXEC`
    - align the relevant manager/dispatch behavior and validation assets to the chosen matrix
    - keep safe canonical trace omission intact
  - Out:
    - replay routing implementation work from `S1`
    - downstream docs/drift-guard consumption work in `SEAM-3`
- **Acceptance criteria**:
  - each relevant execution mode/platform cell has one explicit expected outcome
  - WPEP Case B and nearby smoke/manual guidance assert that matrix directly
  - canonical trace continues omitting builtin/preexec command bodies
- **Dependencies**:
  - `C-02`
  - `THR-01`
  - `crates/shell/src/execution/manager.rs`
  - `crates/shell/src/execution/routing/dispatch/exec.rs`
  - `crates/shell/src/scripts/bash_preexec.rs`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`
- **Verification**:
  - targeted runtime/unit tests where behavior is enforced
  - deterministic manual/smoke assertions for the published matrix
- **Rollout/safety**: preserve `command_omitted: true` and avoid introducing raw builtin/preexec bodies into canonical trace.
- **Review surface refs**: `../../review_surfaces.md` R3

#### S2.T1 - Publish the explicit execution-mode behavior matrix

- **Outcome**: the repo has one authoritative answer for how tracing behavior differs across wrap, script, and interactive paths and across platforms in scope.
- **Inputs/outputs**:
  - Inputs: `C-02`, manager/dispatch behavior, current WPEP playbook language
  - Outputs: explicit matrix rows and the exact Case B target assertion
- **Thread/contract refs**: `THR-01`, `C-02`
- **Implementation notes**:
  - keep the matrix tied to the concrete surfaces already in this pack
  - record "ignored", "best-effort", or "honored" explicitly rather than implying behavior from absence
- **Acceptance criteria**:
  - every in-scope row answers:
    - are `world_process_*` records expected?
    - is `builtin_command` expected?
    - is `SUBSTRATE_ENABLE_PREEXEC` honored?
- **Test notes**:
  - cross-check with playbook cases and any existing command-mode tests
- **Risk/rollback notes**:
  - if a cell is genuinely unresolved, do not paper it over; capture the narrowest decision record needed and keep the matrix explicit

Checklist:
- Implement: runtime or validation-surface updates needed for the matrix
- Test: targeted checks covering the chosen rows
- Validate: confirm no row contradicts safe trace omission
- Cleanup: remove obsolete Case B wording

#### S2.T2 - Align WPEP Case B and smoke/manual assertions to the matrix

- **Outcome**: the active tracing parity pack validates the contract this seam publishes, not stale proxy expectations.
- **Inputs/outputs**:
  - Inputs: behavior matrix from `S2.T1`, WPEP playbook, smoke scripts, nearby pack contract docs
  - Outputs: updated Case B language and smoke assertions that match the published matrix
- **Thread/contract refs**: `THR-01`, `C-02`
- **Implementation notes**:
  - update manual and smoke assets in the same slice so the operator story stays coherent
  - keep assertions focused on published behavior, not incidental debug detail
- **Acceptance criteria**:
  - Case B can be read without local interpretation
  - smoke assertions distinguish expected omission from expected absence
- **Test notes**:
  - re-run targeted WPEP smoke/manual checks where practical
- **Risk/rollback notes**:
  - do not broaden the WPEP pack's scope beyond the behavior matrix needed for this seam

Checklist:
- Implement: playbook/smoke alignment
- Test: rerun targeted smoke or validation probes
- Validate: compare results to the matrix
- Cleanup: remove contradictory wording from the touched assets
