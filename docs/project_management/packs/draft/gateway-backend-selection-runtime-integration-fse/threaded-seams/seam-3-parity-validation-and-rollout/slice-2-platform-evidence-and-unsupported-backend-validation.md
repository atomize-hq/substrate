---
slice_id: S2
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - Linux/macOS/Windows evidence ownership or smoke expectations change after S1
    - unsupported-backend behavior changes outside the parity matrix before landing
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
contracts_produced: []
contracts_consumed:
  - C-03
  - C-04
  - C-05
open_remediations: []
---
### S2 - Land platform evidence and unsupported-backend validation

- **User/system value**:
  - Turns the published runtime handoff into platform-readable validation evidence that downstream rollout can trust.
- **Scope (in/out)**:
  - In: Linux/macOS/Windows evidence expectations, smoke/manual validation steps, and unsupported-backend validation across platforms
  - Out: backend selection/runtime contract changes and rollout publication wording
- **Acceptance criteria**:
  - Linux/macOS/Windows evidence consumes the same parity matrix and contract truth
  - unsupported backends remain explicit negative cases across platform validation surfaces
  - smoke/manual steps do not rely on widened operator or status surfaces
- **Dependencies**:
  - `S1`
  - `THR-02`
  - `C-05`
- **Verification**:
  - refreshed parity-supporting specs or playbooks plus any smoke/manual evidence targets needed for closeout
- **Rollout/safety**:
  - platform-specific transport differences may appear in evidence, but operator-facing lifecycle/status semantics must remain one contract
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S2.T1 - Align platform evidence with the canonical parity contract

- **Outcome**:
  - platform validation surfaces become closeout-ready proof rather than informal notes.
- **Inputs/outputs**:
  - Inputs: canonical runtime parity contract, current smoke/manual expectations, automated parity matrix
  - Outputs: platform parity and manual/smoke evidence surfaces aligned to the contract
- **Thread/contract refs**:
  - `THR-03`
  - `C-05`
- **Implementation notes**:
  - keep Linux/macOS/Windows differences confined to verification evidence
  - preserve one operator-facing lifecycle/status contract
- **Acceptance criteria**:
  - closeout can cite Linux/macOS/Windows evidence without redefining lifecycle semantics
  - unsupported-backend handling stays explicit in the platform evidence bundle
- **Test notes**:
  - verify evidence references map back to the parity matrix and existing operator/runtime contract
- **Risk/rollback notes**:
  - weak platform evidence will make rollout publication look stronger than the actual proof

Checklist:
- Implement:
  - align platform evidence surfaces to the canonical parity contract
- Test:
  - confirm unsupported-backend behavior is visible in platform validation
- Validate:
  - confirm platform differences stay underneath one operator-facing contract
