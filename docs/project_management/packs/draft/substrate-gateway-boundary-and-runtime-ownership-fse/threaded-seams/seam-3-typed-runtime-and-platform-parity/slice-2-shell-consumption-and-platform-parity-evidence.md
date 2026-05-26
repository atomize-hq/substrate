---
slice_id: S2
seam_id: SEAM-3
slice_kind: implementation
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-04
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S2 - Shell consumption and platform parity evidence

- **User/system value**: operator-facing lifecycle/status behavior stays stable across Linux, macOS, and Windows because shell and docs consume one parity contract with explicit evidence expectations.
- **Scope (in/out)**:
  - In:
    - shell builtin consumption path
    - Linux/macOS/Windows parity guarantees for lifecycle visibility and status semantics
    - allowed divergence list and required evidence
    - explicit out-of-scope boundary for provisioning changes
  - Out:
    - command-family semantics
    - `status --json` field list
    - policy/trust-boundary rules
    - final cross-doc/manual-playbook lock-in
- **Acceptance criteria**:
  - platform guarantees are explicit and testable
  - any allowed divergence is named without creating separate user contracts
  - provisioning remains explicitly outside this seam's owned contract
- **Dependencies**:
  - `review.md`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - `docs/contracts/gateway/operator-contract.md`
  - `docs/contracts/gateway/runtime-parity.md`
- **Verification**:
  - pass condition: parity planning can describe one operator-facing runtime contract and one evidence model across Linux, macOS, and Windows
- **Rollout/safety**:
  - keep backend quirks and provisioning behavior from becoming the user contract
- **Review surface refs**: `review.md` R2
