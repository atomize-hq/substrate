---
slice_id: S1
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
  - THR-02
contracts_produced:
  - C-02
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S1 - Status JSON envelope and wiring boundary

- **User/system value**: machine-readable status becomes single-source before runtime and docs consume it.
- **Scope (in/out)**:
  - In:
    - top-level `status --json` object shape
    - `client_wiring.*` field family
    - non-secret guarantees and conditional omission rules
    - hard boundary against ADR-0042 additive metadata outside the owned family
  - Out:
    - fail-closed placement and trust-boundary decision logic
    - typed runtime transport and parity work
- **Acceptance criteria**:
  - the top-level schema and omission rules are explicit
  - `client_wiring.*` is the only owned endpoint-discovery family in this pack
  - additive metadata outside that family remains out of scope
- **Dependencies**:
  - `review.md`
  - `../../governance/seam-1-closeout.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
- **Verification**:
  - pass condition: downstream runtime and docs work can consume one schema surface without reopening operator-boundary questions
- **Rollout/safety**:
  - keep the surface non-secret and deterministic
- **Review surface refs**: `review.md` R1
