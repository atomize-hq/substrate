---
slice_id: S2
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - docs examples drift from the published runtime fragments, codes, or omission rules
    - manual playbook filters or expected assertions drift from replay tests
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S2 - Docs and playbook alignment

- **User/system value**: reviewers and operators read the same published contract in runtime docs and manual validation guidance that tests already prove.
- **Scope (in/out)**:
  - In: `docs/REPLAY.md`, `docs/TRACE.md`, `docs/COMMANDS.md`, and `manual_testing_playbook.md`
  - Out: smoke-wrapper implementation details
- **Acceptance criteria**:
  - docs examples use the exact published fragments and field names
  - the manual playbook references the same test filters and expected assertions as the runtime lock-in
  - no doc example introduces an unsupported path display, env value, or extra key
- **Verification**:
  - doc examples and playbook steps reconcile directly against `cargo test -p shell --test replay_world -- --nocapture`

#### S2.T1 - Align docs and manual validation to the published runtime contract

- **Outcome**: docs and manual validation stop being a second source of truth and instead cite the runtime contract already published by `SEAM-2`.
- **Thread/contract refs**: `THR-03`, `THR-04`; `C-03`, `C-04`
