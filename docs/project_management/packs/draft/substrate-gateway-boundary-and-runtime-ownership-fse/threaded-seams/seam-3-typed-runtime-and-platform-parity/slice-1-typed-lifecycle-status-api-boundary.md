---
slice_id: S1
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
  - THR-02
  - THR-03
  - THR-04
contracts_produced:
  - C-04
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S1 - Typed lifecycle/status API boundary

- **User/system value**: world-service and shared clients converge on one typed lifecycle/status contract before shell behavior or docs bake in backend-private assumptions.
- **Scope (in/out)**:
  - In:
    - typed world-service lifecycle/status ownership
    - shared `transport-api-types` and `transport-api-client` alignment
    - the boundary between published schema/policy inputs and the typed runtime contract
  - Out:
    - operator command-family wording
    - `status --json` field ownership
    - final platform evidence and docs lock-in
- **Acceptance criteria**:
  - shell and shared clients consume one typed lifecycle/status path
  - published schema/policy inputs are reused without redefining them
  - runtime-private probing does not become the contract source
- **Dependencies**:
  - `review.md`
  - `../../governance/seam-2-closeout.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
- **Verification**:
  - pass condition: runtime planning can name one authoritative lifecycle/status path without reopening schema or policy truth
- **Rollout/safety**:
  - keep the typed contract detached from backend-private status assembly
- **Review surface refs**: `review.md` R1
