---
slice_id: S00
seam_id: SEAM-3
slice_kind: contract_definition
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
    contract: pending
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations:
  - REM-001
---
### S00 - Runtime parity contract definition

- **User/system value**: downstream runtime and docs work inherit one explicit typed lifecycle/status contract instead of improvising around backend-specific behavior.
- **Scope (in/out)**:
  - In:
    - define the owned boundary for the typed world-agent lifecycle/status contract
    - define the allowed divergence list and the evidence expectations for Linux, macOS, and Windows
    - name the publication surfaces for the feature-local parity spec and the durable runtime/parity contract
  - Out:
    - world-agent handler implementation
    - shell/client adoption work
    - seam-exit evidence and downstream docs lock-in
- **Acceptance criteria**:
  - `C-04` names the typed lifecycle/status ownership boundary and its canonical contract refs
  - the feature-local parity spec and durable runtime/parity contract exist and align
  - later slices do not need to reopen which surface owns runtime/parity truth
- **Dependencies**:
  - `C-01`
  - `C-02`
  - `C-03`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
- **Verification**:
  - the contract-definition bundle must map directly to `platform-parity-spec.md`, `docs/contracts/substrate-gateway-runtime-parity.md`, seam-local review, and later runtime/parity implementation surfaces
- **Rollout/safety**:
  - keep provisioning out of scope and keep raw exec probing out of the operator contract
- **Review surface refs**: `review.md` R1 and R2
