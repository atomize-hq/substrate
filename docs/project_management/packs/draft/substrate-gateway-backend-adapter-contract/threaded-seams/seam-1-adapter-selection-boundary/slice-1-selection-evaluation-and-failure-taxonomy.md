---
slice_id: S1
seam_id: SEAM-1
slice_kind: documentation
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - backend inventory filename-to-id matching changes
    - `llm.allowed_backends` semantics or precedence changes
    - failure-bucket names drift from the contract baseline
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
contracts_consumed: []
open_remediations: []
---
### S1 - Lock selection evaluation order and failure taxonomy

#### Goal

Turn the selection boundary into one deterministic decision tree that downstream protocol and parity work can consume without reopening ADR-0027 or collapsing fail-closed outcomes.

#### Dependencies

- `S00` defines the owned contract bundle
- basis authorities:
  - `../../pre-planning/minimal_spec_draft.md`
  - `../../pre-planning/spec_manifest.md`
  - `../../pre-planning/impact_map.md`

#### S1.T1 - Write the ordered selection-input contract

- **Outcome**:
  - `contract.md` and `policy-spec.md` state the exact order in which config, policy, and inventory inputs are evaluated before adapter dispatch.
- **Files**:
  - `../../contract.md`
  - `../../policy-spec.md`
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Acceptance criteria**:
  - allowlist gating occurs before adapter dispatch
  - inventory lookup uses the stable backend id and existing filename/id matching rules
  - gateway-local admin, config, persistence, or session state is explicitly excluded from authorization
- **Test notes**:
  - cross-check with ADR-0027 and the implemented config-policy pack

Checklist:
- Implement:
  - document the ordered evaluation inputs
  - document the trusted-input boundary and explicit non-inputs
- Test:
  - compare the order against ADR-0027 and the pre-planning impact map
- Validate:
  - confirm the flow reaches adapter dispatch only after allowlist approval

#### S1.T2 - Separate invalid, unavailable, and denied outcomes

- **Outcome**:
  - the seam-owned docs name one failure class each for invalid selection, dependency unavailability, and policy denial, with no overlap.
- **Files**:
  - `../../contract.md`
  - `../../policy-spec.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
- **Acceptance criteria**:
  - invalid selection covers unknown or malformed backend ids and inventory mismatch
  - dependency unavailable covers missing or unsupported adapter components
  - policy denial covers allowlist or safety rejection only
  - the taxonomy stays compatible with the shared exit-code standard
- **Test notes**:
  - compare wording against ADR-0041 exit-code buckets and `../../review.md`

Checklist:
- Implement:
  - write the failure matrix and example conditions
  - map each outcome to the seam-owned contract text
- Test:
  - verify each example case fits exactly one failure class
- Validate:
  - confirm downstream seams can cite the taxonomy without introducing new buckets
