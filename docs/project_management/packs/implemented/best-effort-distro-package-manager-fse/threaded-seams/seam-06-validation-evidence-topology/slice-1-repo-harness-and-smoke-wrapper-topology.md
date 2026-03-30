---
slice_id: S1
seam_id: SEAM-06
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper/doc contract changes
    - repo harness path changes
    - smoke-wrapper topology changes
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
  - THR-05
  - THR-06
contracts_produced:
  - C-10
contracts_consumed:
  - C-01
  - C-03
  - C-04
  - C-05
  - C-06
  - C-07
  - C-08
  - C-09
open_remediations: []
candidate_subslices: []
---
### S1 - Repo harness and smoke-wrapper topology

- **User/system value**: one authoritative repo harness defines the installer contract assertions, while the feature-local smoke wrapper stays thin and non-authoritative.
- **Scope (in/out)**:
  - In: `tests/installers/pkg_manager_detection_smoke.sh` as the authoritative harness and `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh` alignment as a thin wrapper
  - Out: manual evidence updates, macOS-hosted verification updates, and seam-exit publication
- **Acceptance criteria**:
  - repo harness explicitly covers the published installer, wrapper, and doc contracts
  - smoke wrapper delegates to the harness without creating new assertions or vocabulary
  - validation topology names one authority for behavior truth
- **Dependencies**:
  - `../../seam-06-validation-evidence-topology.md`
  - `../../governance/seam-05-closeout.md`
  - `../../../best-effort-distro-package-manager/contract.md`
- **Verification**:
  - validation review proves harness authority and smoke-wrapper thinness remain coherent
