---
slice_id: S3
seam_id: SEAM-03
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - exit `2` remediation wording changes
    - exit `3` remediation wording changes
    - explicit selectors stop failing closed
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
contracts_produced:
  - C-06
contracts_consumed:
  - C-04
  - C-05
open_remediations: []
candidate_subslices: []
---
### S3 - Explicit failure taxonomy

- **User/system value**: invalid or unavailable explicit selectors fail with deterministic, actionable operator feedback instead of collapsing into generic errors or lower-precedence recovery.
- **Scope (in/out)**:
  - In: invalid explicit-selector exit `2`, missing-explicit-manager exit `3`, required remediation elements, and no-fallthrough behavior after an explicit selector chooses the branch
  - Out: ordered `PATH` fallback, multi-manager warning, no-manager exit `4`, wrapper pass-through, and docs propagation
- **Acceptance criteria**:
  - invalid `--pkg-manager` and invalid `PKG_MANAGER` each exit with code `2`
  - a valid explicit manager missing from `PATH` exits with code `3`
  - exit `2` and `3` messages include the offending source, the selected or invalid value, and the contract-owned remediation elements
  - explicit-selector failure branches never fall through to os-release mapping or the ordered `PATH` probe
- **Dependencies**:
  - `S1`
  - `S2`
  - `scripts/substrate/install-substrate.sh`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../../../best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- **Verification**:
  - fixture coverage proves invalid flag/env produces exit `2` and missing explicit manager produces exit `3`
  - pass condition: failure branches stay fail-closed and keep fallback ownership in `SEAM-04`
- **Rollout/safety**:
  - protects operators from silent recovery after an explicit choice
  - isolates failure taxonomy from fallback and wrapper propagation work
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` R2

#### S3.T1 - Freeze exit `2` / `3` contract execution

- **Outcome**: explicit-selector failure posture is concrete enough to implement and verify without reopening fallback or wrapper ownership.
- **Inputs/outputs**:
  - Inputs: explicit selector source, allowed manager vocabulary, `PATH` availability
  - Outputs: exit `2` / `3` behavior with required remediation elements
- **Thread/contract refs**:
  - `THR-03`
  - `C-06`
- **Implementation notes**:
  - once an explicit selector owns the branch, do not recover into os-release mapping or path probe
