---
slice_id: S1
seam_id: SEAM-04
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - fixed probe order changes
    - `pkg_manager.source=path_probe` semantics change
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
  - THR-03
  - THR-04
contracts_produced:
  - C-07
contracts_consumed:
  - C-01
  - C-03
  - C-04
  - C-05
  - C-06
open_remediations: []
candidate_subslices: []
---
### S1 - Path probe selection

- **User/system value**: fallback selection remains deterministic when earlier explicit and os-release stages do not choose a manager.
- **Scope (in/out)**:
  - In: fixed ordered probe list, earliest-manager selection, and `pkg_manager.source=path_probe`
  - Out: warning-line wording and placement, exit `4` remediation wording, wrapper/docs propagation, and validation ownership
- **Acceptance criteria**:
  - when exactly one supported manager is present in PATH after upstream stages make no selection, the installer selects that manager with `pkg_manager.source=path_probe`
  - when multiple supported managers are present, the installer still selects the earliest manager in the fixed order `apt-get -> dnf -> yum -> pacman -> zypper`
  - fallback selection never reopens explicit-selector or os-release ownership
- **Dependencies**:
  - `../../seam-04-fallback-probe-failure-taxonomy.md`
  - `../../threading.md`
  - `../seam-03-explicit-override-selection/seam.md`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../../../best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- **Verification**:
  - fixture coverage proves single-manager and multi-manager probe-order selection behavior
  - pass condition: fallback selection uses only the supported-manager vocabulary and the contract-owned fixed order
- **Rollout/safety**:
  - keeps fallback logic deterministic even on hosts with multiple supported managers
  - preserves upstream ownership for parser/input, mapping/reporting, and explicit selectors
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` next seam focus

#### S1.T1 - Freeze fixed-order fallback selection

- **Outcome**: the fallback selection stage becomes concrete enough to implement without ambiguity or raw-PATH drift.
- **Inputs/outputs**:
  - Inputs: no upstream selection, supported manager vocabulary, host PATH
  - Outputs: selected manager plus `pkg_manager.source=path_probe`
- **Thread/contract refs**:
  - `THR-03`, `THR-04`
  - `C-07`
- **Implementation notes**:
  - do not let warning or exit `4` behavior redefine the fixed selection order
