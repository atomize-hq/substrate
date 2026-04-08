---
slice_id: S2
seam_id: SEAM-3
slice_kind: conformance
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
  - THR-02
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations: []
---
### S2 - WPEP playbook and smoke alignment

- **User/system value**: the active validation playbook and smoke assets stop asserting proxy behavior and instead validate the published execution contracts directly.
- **Scope (in/out)**:
  - In: align the active WPEP manual playbook and smoke assertions with `THR-01`, and incorporate any `THR-02` operator-facing conformance surfaces that belong in shared guidance.
  - Out: runtime behavior changes or unrelated playbook cleanup.
- **Acceptance criteria**:
  - Case B and related smoke assertions align to the landed behavior matrix from `SEAM-1`
  - no shared guidance contradicts the published abnormal-terminal-loss contract from `SEAM-2`
- **Verification**:
  - readback against `governance/seam-1-closeout.md`, `governance/seam-2-closeout.md`, and the touched playbook/smoke surfaces

Checklist:
- Implement: playbook and smoke guidance alignment
- Test: `bash -n` or equivalent lightweight validation for touched scripts if needed
- Validate: compare every changed assertion to published upstream evidence
- Cleanup: remove stale proxy assertions
