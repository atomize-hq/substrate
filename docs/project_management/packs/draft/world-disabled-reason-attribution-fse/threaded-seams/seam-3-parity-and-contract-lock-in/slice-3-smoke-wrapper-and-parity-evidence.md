---
slice_id: S3
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - platform smoke wrappers drift from the published runtime assertions
    - parity evidence permits unsupported platform divergence
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
  - THR-03
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-02
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S3 - Smoke wrapper and parity evidence

- **User/system value**: Linux, macOS, and Windows validation wrappers all enforce the same published contract instead of drift-prone local expectations.
- **Scope (in/out)**:
  - In: platform smoke wrappers and parity evidence capture
  - Out: new platform behavior or runtime code
- **Acceptance criteria**:
  - smoke wrappers assert the same fragments, codes, tokenized displays, and omission rules as tests/docs
  - allowed platform divergence remains limited to backend/transport details outside the published fragments
  - parity evidence cites the same runtime truth across Linux, macOS, and Windows
- **Verification**:
  - smoke wrapper checks align with the replay test matrix and manual playbook

#### S3.T1 - Preserve cross-platform parity against one published contract

- **Outcome**: platform validation uses the published runtime contract instead of ad hoc local expectations.
- **Thread/contract refs**: `THR-02`, `THR-03`, `THR-04`; `C-02`, `C-03`, `C-04`
