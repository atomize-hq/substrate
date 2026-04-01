---
slice_id: S1
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - replay tests stop pinning one of the published runtime cases
    - trace assertions drift from the published `world_disabled_*` codes or `world_disable_source` object shape
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
### S1 - Regression coverage and trace locks

- **User/system value**: replay regression coverage becomes the authoritative lock on the published runtime contract.
- **Scope (in/out)**:
  - In: `crates/shell/tests/replay_world.rs` coverage for override env, workspace config, global config, unknown-source fallback, and replay-local opt-out omission rules.
  - Out: docs, smoke wrappers, and manual playbook updates.
- **Acceptance criteria**:
  - tests pin the exact published fragments and `world_disabled_*` codes
  - tests pin tokenized `world_disable_source` objects and omission for replay-local opt-outs
  - tests do not widen runtime behavior beyond the published contract
- **Verification**:
  - `cargo test -p shell --test replay_world -- --nocapture`

#### S1.T1 - Lock the published runtime contract in replay tests

- **Outcome**: regression coverage proves the landed runtime truth before docs or parity wrappers depend on it.
- **Thread/contract refs**: `THR-02`, `THR-03`, `THR-04`; `C-02`, `C-03`, `C-04`
