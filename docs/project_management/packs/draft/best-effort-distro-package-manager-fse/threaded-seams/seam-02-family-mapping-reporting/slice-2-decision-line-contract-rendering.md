---
slice_id: S2
seam_id: SEAM-02
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - decision-line wording changes
    - decision-line placement or suppression changes
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
  - THR-08
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S2 - Decision-line contract and rendering

- **User/system value**: os-release-based manager selection becomes operator-visible and downstream-stable instead of emerging from incidental logging.
- **Scope (in/out)**:
  - In: stable decision-line template, timing before installation starts, suppression posture for no-selection branches, and `pkg_manager.source=os_release`
  - Out: explicit selector reporting, PATH fallback warning text, wrapper propagation, and no-manager remediation
- **Acceptance criteria**:
  - when S1 produces a concrete mapped manager, the installer emits the exact stable decision line once to stderr
  - the decision line uses normalized `distro_id`, normalized `distro_id_like`, selected manager spelling, and `pkg_manager.source=os_release`
  - the decision line appears after mapped selection is complete and before any package-manager install command begins
  - no-selection branches remain silent here and fall through cleanly to later seams
- **Dependencies**:
  - `S1`
  - `scripts/substrate/install-substrate.sh`
  - `../../../best-effort-distro-package-manager/contract.md`
- **Verification**:
  - fixture coverage proves wording, placement, and one-time emission
  - pass condition: the stable decision line remains confined to the os-release selection stage and does not leak into explicit-selector or fallback work
- **Rollout/safety**:
  - stabilizes the operator-facing selection/reporting truth before later seams add override or fallback branches
  - prevents later seams from inventing their own reporting vocabulary
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
  - `../../review_surfaces.md` R2

#### S2.T1 - Freeze the `C-04` reporting contract in execution work

- **Outcome**: decision-line wording, timing, and suppression are concrete enough to implement without reopening reporting ownership later.
- **Inputs/outputs**:
  - Inputs: mapped manager selection state from `S1`
  - Outputs: one stable stderr decision line and `pkg_manager.source=os_release`
- **Thread/contract refs**:
  - `THR-01`, `THR-02`, `THR-08`
  - `C-04`
- **Implementation notes**:
  - do not absorb explicit-selector or fallback reporting branches; those remain later seams
