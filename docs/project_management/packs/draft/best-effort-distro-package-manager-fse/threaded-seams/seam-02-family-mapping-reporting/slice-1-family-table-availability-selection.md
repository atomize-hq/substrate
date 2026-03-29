---
slice_id: S1
seam_id: SEAM-02
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - parser/input truth from `SEAM-01` changes
    - family-table rules change
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
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S1 - Family-table and availability-based selection

- **User/system value**: normalized SEAM-01 parser truth becomes one deterministic os-release selection stage instead of forcing later seams to restate family mapping.
- **Scope (in/out)**:
  - In: Debian or Ubuntu, Fedora or RHEL, Arch, and SUSE family matching; mapped-manager availability checks; `pkg_manager.source=os_release`
  - Out: decision-line wording, explicit selector handling, ordered PATH fallback, warning text, and exit taxonomy
- **Acceptance criteria**:
  - Debian or Ubuntu truth selects `apt-get` only when `apt-get` is available
  - Fedora or RHEL truth selects `dnf` when available, otherwise `yum` when available
  - Arch truth selects `pacman` only when `pacman` is available
  - SUSE truth selects `zypper` only when `zypper` is available
  - when no mapped manager is available, control falls through cleanly without partial reporting or fallback behavior
- **Dependencies**:
  - `../../seam-02-family-mapping-reporting.md`
  - `../../threading.md`
  - `../seam-01-os-release-input-parser/seam.md`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../../../best-effort-distro-package-manager/decision_register.md`
- **Verification**:
  - fixture coverage proves each family branch consumes only `DETECTED_DISTRO_ID` and `DETECTED_DISTRO_ID_LIKE`
  - pass condition: mapped selection never re-reads raw os-release input and never reports success when the mapped manager is unavailable
- **Rollout/safety**:
  - preserves SEAM-01 ownership for parser/input truth
  - keeps PATH fallback ownership entirely out of this slice
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` R1

#### S1.T1 - Land the family-table contract for `C-03`

- **Outcome**: one exact family table and availability rule set is attached to execution work instead of staying implicit in the seam brief.
- **Inputs/outputs**:
  - Inputs: `DETECTED_DISTRO_ID`, `DETECTED_DISTRO_ID_LIKE`, host `PATH`
  - Outputs: mapped manager selection state plus `pkg_manager.source=os_release`
- **Thread/contract refs**:
  - `THR-01`, `THR-02`, `THR-08`
  - `C-03`
- **Implementation notes**:
  - consume only SEAM-01 normalized fields; do not reopen parser semantics or add reporting here
