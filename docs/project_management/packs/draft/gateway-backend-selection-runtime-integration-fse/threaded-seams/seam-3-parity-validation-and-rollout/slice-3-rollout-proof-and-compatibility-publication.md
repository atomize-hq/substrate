---
slice_id: S3
seam_id: SEAM-3
slice_kind: adoption
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - rollout notes or compatibility messaging drift away from the landed parity matrix before closeout
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced: []
contracts_consumed:
  - C-05
open_remediations: []
---
### S3 - Publish rollout proof and compatibility surfaces

- **User/system value**:
  - Makes the parity result consumable without turning rollout guidance into a shadow contract system.
- **Scope (in/out)**:
  - In: compatibility-spec publication, rollout proof framing, and evidence references for operators and reviewers
  - Out: new operator semantics, new status fields, or platform-specific support promises beyond the named proof target
- **Acceptance criteria**:
  - rollout and compatibility surfaces name `api:openai` as the first additional-backend proof target
  - rollout guidance preserves `cli:codex` as the regression floor and keeps unsupported backends explicit
  - documentation remains subordinate to canonical operator/runtime contracts and automated/platform evidence
- **Dependencies**:
  - `S1`
  - `S2`
  - `THR-03`
  - `C-05`
- **Verification**:
  - review of the compatibility-spec publication and rollout framing against upstream parity and platform evidence
- **Rollout/safety**:
  - do not let rollout prose imply broader support or surface-area widening than the evidence actually proves
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S3.T1 - Make rollout publication evidence-backed

- **Outcome**:
  - rollout and compatibility publication consume upstream evidence rather than inventing new contract truth.
- **Inputs/outputs**:
  - Inputs: upstream automated parity evidence, upstream platform evidence, canonical runtime parity contract
  - Outputs: evidence-backed compatibility publication and rollout framing
- **Thread/contract refs**:
  - `THR-03`
  - `C-05`
- **Implementation notes**:
  - keep support claims bounded to `cli:codex` and the named `api:openai` proof target
  - keep unsupported-backend posture explicit in rollout messaging
- **Acceptance criteria**:
  - closeout can publish `THR-03` without adding a new shadow contract or support matrix
  - rollout surfaces read as evidence-backed adoption guidance rather than speculative future planning
- **Test notes**:
  - verify rollout references line up with upstream automated and platform evidence targets
- **Risk/rollback notes**:
  - vague rollout prose will undermine the proof bundle even if tests and smoke evidence land correctly

Checklist:
- Implement:
  - publish compatibility-spec and rollout framing from evidence
- Test:
  - confirm every support claim traces back to upstream parity and platform evidence
- Validate:
  - confirm rollout surfaces stay subordinate to canonical contracts
