---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - stable backend-id grammar changes
    - allowlist order or deny-by-default semantics change
    - additive adapter-visible status fields widen without an owner-line decision
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
contracts_produced:
  - C-01
  - C-02
contracts_consumed: []
open_remediations: []
---
### S00 - Define the `C-01` and `C-02` contract baseline

#### Goal

Make the owned contract bundle concrete enough that downstream seams can cite one durable selection-boundary truth instead of inferring it from ADR prose or pre-planning notes.

#### Dependencies

- none inbound; this is the first seam in the pack
- authoritative basis references:
  - `../../threading.md`
  - `../../seam-1-adapter-selection-boundary.md`
  - `../../pre-planning/spec_manifest.md`
  - `../../pre-planning/impact_map.md`

#### S00.T1 - Create the canonical `C-01` backend-selection baseline

- **Outcome**:
  - `docs/contracts/gateway/backend-adapter-selection.md` exists and states the stable backend-id semantics, ordered evaluation inputs, one-id-to-one-adapter rule, and failure taxonomy before dispatch.
- **Files**:
  - `docs/contracts/gateway/backend-adapter-selection.md`
  - `../../contract.md`
  - `../../policy-spec.md`
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Acceptance criteria**:
  - one stable `<kind>:<name>` backend id maps to one adapter identity
  - config, policy, and inventory evaluation order is explicit
  - invalid selection, dependency unavailable, and policy denial are named separately
  - the document does not invent new config keys, env vars, or gateway-local policy inputs
- **Test notes**:
  - doc review against ADR-0027, the implemented config-policy pack, and ADR-0041

Checklist:
- Implement:
  - author the canonical `C-01` contract baseline
  - mirror the selection-boundary summary into `contract.md` and `policy-spec.md`
- Test:
  - compare the field names and failure labels against ADR-0041 and `../../threading.md`
- Validate:
  - confirm downstream seams can cite `C-01` without referencing planning-pack-only prose

#### S00.T2 - Bind `C-02` to one explicit external owner line

- **Outcome**:
  - the planning packet states the exact v1 publication boundary: no additive adapter-visible status field family is currently published beyond `status` and `client_wiring.*`, and any future additive family requires an explicit status-schema owner update.
- **Files**:
  - `../../contract.md`
  - `../../policy-spec.md`
  - `docs/contracts/gateway/status-schema.md`
  - `docs/contracts/gateway/operator-contract.md`
- **Thread/contract refs**:
  - `THR-01`
  - `C-02`
- **Acceptance criteria**:
  - the owner line for any additive adapter-visible `status --json` data is explicit
  - the existing top-level envelope and `client_wiring.*` family remain externally owned
  - the boundary does not implicitly widen operator-visible status semantics
  - current code and tests remain aligned with the narrow v1 shape
- **Test notes**:
  - owner-line review against `REM-001` and `../../pre-planning/workstream_triage.md`

Checklist:
- Implement:
  - record the v1 decision that no additive adapter-visible field family is currently published
  - record the status-schema owner requirement for any future additive field family
- Test:
  - compare the proposed subset against the existing status schema and operator contract
- Validate:
  - confirm `SEAM-2` can consume the boundary without inventing a second schema owner
