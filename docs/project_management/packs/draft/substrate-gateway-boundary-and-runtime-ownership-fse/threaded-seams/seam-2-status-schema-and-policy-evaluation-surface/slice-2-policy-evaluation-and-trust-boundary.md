---
slice_id: S2
seam_id: SEAM-2
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
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-03
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S2 - Policy evaluation and trust boundary

- **User/system value**: fail-closed placement and non-trust rules become explicit before runtime code or docs invent their own decision flow.
- **Scope (in/out)**:
  - In:
    - evaluation over existing ADR-0027 config and policy inputs
    - fail-closed no-host-fallback rule
    - host secret sourcing and host-to-world secret delivery boundary
    - ban on trusting gateway-local config, admin, and persistence surfaces
  - Out:
    - top-level status schema and `client_wiring.*` inventory
    - typed world-service transport and parity guarantees
- **Acceptance criteria**:
  - invalid integration state, dependency unavailability, and policy denial stay distinct
  - no-host-fallback posture is explicit
  - gateway-local control-plane surfaces remain out of trust
- **Dependencies**:
  - `review.md`
  - `../../governance/seam-1-closeout.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md`
- **Verification**:
  - pass condition: runtime and docs consumers can inherit one fail-closed and non-trust policy surface
- **Rollout/safety**:
  - keep fail-closed and non-trust rules explicit
- **Review surface refs**: `review.md` R2
