---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
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
  - THR-03
contracts_produced:
  - C-02
  - C-03
contracts_consumed:
  - C-01
open_remediations: []
---
### S00 - Status schema and policy contract definition

- **User/system value**: downstream runtime and docs work inherit one explicit schema and policy boundary instead of improvising around the operator contract.
- **Scope (in/out)**:
  - In:
    - define the owned boundary for the `status --json` envelope and the `client_wiring.*` field family
    - define the owned fail-closed placement and trust-boundary rules over existing ADR-0027 inputs
    - name the publication and verification surfaces for `C-02` and `C-03`
  - Out:
    - field-by-field examples beyond the owned schema surface
    - typed world-agent transport and parity guarantees
    - terminal docs-validation lock-in
- **Acceptance criteria**:
  - `C-02` names the owned top-level envelope, `client_wiring.*` boundary, and absence-semantics ownership
  - `C-03` names the no-host-fallback rule, host-to-world secret boundary, and non-trust rule for gateway-local control-plane surfaces
  - later slices do not need to reopen which surface owns schema or policy truth
- **Dependencies**:
  - `C-01`
  - `THR-01`
  - `../../threading.md`
  - `../../governance/seam-1-closeout.md`
- **Verification**:
  - the contract-definition bundle must map directly to `gateway-status-schema-spec.md`, `policy-spec.md`, docs, tests, and downstream contract refs
- **Rollout/safety**:
  - keep owned boundaries explicit and fail closed
- **Review surface refs**: `review.md` R1 and R2
