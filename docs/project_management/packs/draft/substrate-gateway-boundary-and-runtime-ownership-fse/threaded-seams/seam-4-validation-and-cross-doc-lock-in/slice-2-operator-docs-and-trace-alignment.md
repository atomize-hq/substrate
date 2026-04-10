---
slice_id: S2
seam_id: SEAM-4
slice_kind: documentation
execution_horizon: active
status: exec-ready
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
  - THR-02
  - THR-03
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations: []
---
### S2 - Operator docs and trace alignment

- **User/system value**: operator-facing and maintainer docs describe one stable truth for command ownership, status wiring, policy posture, and runtime parity.
- **Scope (in/out)**:
  - In:
    - `docs/CONFIGURATION.md`
    - `docs/USAGE.md`
    - `docs/WORLD.md`
    - `docs/TRACE.md`
    - stale cross-doc or archived link normalization relevant to those files
  - Out:
    - command implementation changes
    - schema or policy ownership changes
    - platform provisioning changes
- **Acceptance criteria**:
  - docs consume the landed contracts rather than restating superseded ADR or archived-pack wording
  - command, schema, policy, and runtime parity language stay consistent across the doc set
  - trace documentation preserves Substrate-owned canonical tracing authority
- **Dependencies**:
  - `review.md`
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - `../../governance/seam-3-closeout.md`
- **Verification**:
  - pass condition: the operator doc set can be read end to end without surfacing contradictory ownership or validation wording
- **Rollout/safety**:
  - preserve descriptive docs as consumers of the contracts, not alternate contract authorities
- **Review surface refs**: `review.md` R2
