---
slice_id: S3
seam_id: SEAM-04
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - exit `4` remediation wording changes
    - no-manager posture stops failing closed
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
  - THR-04
contracts_produced:
  - C-07
contracts_consumed:
  - C-05
  - C-06
open_remediations: []
candidate_subslices: []
---
### S3 - No-manager exit `4`

- **User/system value**: hosts that reach the fallback stage without any supported manager fail with one deterministic, actionable no-manager posture instead of generic installer failure.
- **Scope (in/out)**:
  - In: exit `4` behavior, required remediation elements, and no-fallthrough posture after the fallback stage fails to choose a manager
  - Out: explicit-selector exit `2` / `3` ownership, wrapper/docs propagation, validation topology ownership, and downstream checkpoint work
- **Acceptance criteria**:
  - when no supported manager is selected after explicit selectors, os-release mapping, and the fixed path probe, the installer exits with code `4`
  - exit `4` names the missing prerequisite commands for the current branch and tells the operator to install them manually and rerun
  - exit `4` tells the operator that rerun may also use `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` or `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`
  - no-manager branches never collapse into generic installer failure or recover into lower-precedence behavior
- **Dependencies**:
  - `S1`
  - `S2`
  - `scripts/substrate/install-substrate.sh`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../../../best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- **Verification**:
  - fixture coverage proves exit `4` and required remediation text when no supported manager is available
  - pass condition: failure remains deterministic and preserves upstream explicit-selector and reporting semantics
- **Rollout/safety**:
  - prevents silent success or generic failure when the decision pipeline exhausts all supported managers
  - isolates no-manager behavior from wrapper/docs propagation and later validation evidence work
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` R2

#### S3.T1 - Freeze no-manager remediation posture

- **Outcome**: the exit `4` branch becomes concrete enough to implement and verify without reopening explicit-selector or wrapper ownership.
- **Inputs/outputs**:
  - Inputs: no selected manager after the fallback stage, missing prerequisites, explicit override forms
  - Outputs: exit `4` behavior with required remediation elements
- **Thread/contract refs**:
  - `THR-04`
  - `C-07`
- **Implementation notes**:
  - once fallback owns the no-manager branch, do not collapse into generic installer failure
