---
slice_id: S3
seam_id: SEAM-2
slice_kind: adoption
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - protocol or schema authority moves
    - verification surfaces widen beyond the documented adapter boundary
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
contracts_produced:
  - C-03
  - C-04
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S3 - Align adoption surfaces and verification

#### Goal

Prepare the owned protocol/schema contract for downstream implementation and closeout by aligning the expected adoption surfaces and verification evidence without widening the seam boundary.

#### Dependencies

- `S1` and `S2` make the owned protocol and schema concrete
- basis authorities:
  - `../../seam-2-adapter-protocol-and-schema.md`
  - `review.md`
  - `../../threading.md`

#### S3.T1 - Align the adoption surfaces

- **Outcome**:
  - the seam-owned specs point at the runtime-adjacent surfaces that will consume `C-03` and `C-04` once implementation begins, without treating those surfaces as in-scope during promotion.
- **Files**:
  - `../../gateway-backend-adapter-protocol-spec.md`
  - `../../gateway-backend-adapter-schema-spec.md`
- **Thread/contract refs**:
  - `THR-02`
  - `C-03`
  - `C-04`
- **Acceptance criteria**:
  - adoption surfaces are listed consistently with the seam brief
  - no code or canonical contract edits are implied by the slice text itself
  - downstream work stays inside the published backend-id and status-boundary guardrails
- **Test notes**:
  - compare the touch-surface list against the seam brief and review bundle

Checklist:
- Implement:
  - state the intended consumer surfaces and boundaries
  - keep the scope at planning and verification preparation only
- Test:
  - confirm the adoption list matches the seam brief
- Validate:
  - confirm later execution can stage implementation work without pulling forward parity scope

#### S3.T2 - Define the verification bundle

- **Outcome**:
  - the seam-owned review and exit-gate plan state what evidence is required to prove `C-03` and `C-04` landed cleanly and can publish `THR-02`.
- **Files**:
  - `review.md`
  - `slice-99-seam-exit-gate.md`
- **Thread/contract refs**:
  - `THR-02`
- **Acceptance criteria**:
  - verification covers the lifecycle owner line, schema subset, and downstream stale triggers
  - exit-gate evidence is concrete enough for `SEAM-3` promotion to consume after landing
- **Test notes**:
  - compare the bundle against the pack threading and closeout expectations

Checklist:
- Implement:
  - state the required verification evidence and downstream stale triggers
- Test:
  - confirm each evidence item maps to either `C-03`, `C-04`, or `THR-02`
- Validate:
  - confirm the closeout can publish `THR-02` without inventing new proof categories
