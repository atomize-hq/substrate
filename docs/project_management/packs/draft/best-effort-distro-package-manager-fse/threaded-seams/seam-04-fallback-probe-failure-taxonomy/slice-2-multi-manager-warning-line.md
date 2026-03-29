---
slice_id: S2
seam_id: SEAM-04
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - warning template changes
    - warning placement changes
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
  - THR-03
  - THR-04
contracts_produced:
  - C-07
contracts_consumed:
  - C-04
open_remediations: []
candidate_subslices: []
---
### S2 - Multi-manager warning line

- **User/system value**: operators on hosts with several supported managers receive one exact explanation before the inherited decision line, while fallback selection remains deterministic.
- **Scope (in/out)**:
  - In: warning-line template, emitted-manager list ordering, and placement before the inherited decision line
  - Out: fixed-order selection implementation, exit `4` remediation wording, wrapper/docs propagation, and validation ownership
- **Acceptance criteria**:
  - when more than one supported manager is present in PATH, the installer emits the exact warning line once
  - the warning lists detected managers in the same fixed probe-order order used for selection
  - the warning line appears before the inherited decision line and does not fork that decision-line wording
- **Dependencies**:
  - `S1`
  - `scripts/substrate/install-substrate.sh`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../../../best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- **Verification**:
  - fixture coverage proves one warning on multi-manager hosts and no warning on single-manager hosts
  - pass condition: warning wording and ordering match the contract-owned text and compose with the inherited decision line
- **Rollout/safety**:
  - keeps the operator-facing warning exact without widening into docs or wrapper work
  - preserves `SEAM-02` ownership for the base decision-line template
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` next seam focus

#### S2.T1 - Bind the warning to fixed-order selection

- **Outcome**: multi-manager warning behavior becomes concrete enough to implement and verify without ambiguity or duplicated reporting vocabulary.
- **Inputs/outputs**:
  - Inputs: detected manager set, selected manager, inherited decision-line contract
  - Outputs: warning line before the inherited decision line
- **Thread/contract refs**:
  - `THR-04`
  - `C-07`
- **Implementation notes**:
  - warning emission must stay tied to the fixed probe order rather than raw PATH order
