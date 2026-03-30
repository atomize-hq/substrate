---
slice_id: S1
seam_id: SEAM-6
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - platform support matrix drifts from the published manager-aware contracts
  - manual playbook stops naming unsupported Linux/Windows lanes or the manual Arch-on-macOS fixture assumptions
gates:
  pre_exec:
    review: inherited
    contract: inherited
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
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
  - C-05
open_remediations:
  - REM-002
candidate_subslices: []
---
### S1 - Platform parity and playbook evidence

- **User/system value**:
  - operators and maintainers get one authoritative support matrix and manual evidence playbook that matches the landed manager-aware behavior.
- **Scope (in/out)**:
  - In:
    - publish one accepted Linux/macOS/Windows support matrix for provisioning and runtime fail-early behavior
    - update the manual testing playbook so unsupported Linux/Windows lanes and the manual Arch-on-macOS fixture assumptions are explicit
    - align parity-spec language with the published manager-aware contracts
  - Out:
    - smoke-script execution changes
    - shared ADR or contract reconciliation beyond the parity/playbook evidence set
- **Acceptance criteria**:
  - the platform parity spec names supported, unsupported, and manual-only lanes without contradicting `C-01` through `C-05`
  - the manual testing playbook makes `REM-002` concrete by naming the manual Arch-on-macOS fixture assumptions
  - Linux host-native and Windows provisioning remain explicitly unsupported and fail-closed in the evidence package
- **Dependencies**:
  - landed closeouts `../../governance/seam-1-closeout.md` through `../../governance/seam-5-closeout.md`
- **Verification**:
  - review of `platform-parity-spec.md`
  - review of `manual_testing_playbook.md`
  - closeout citation readiness for `../../governance/seam-6-closeout.md`
- **Rollout/safety**:
  - documentation and evidence only; no new product behavior
- **Review surface refs**:
  - `review.md`

#### S1.T1 - Publish the cross-platform support matrix

- **Outcome**:
  - one accepted Linux/macOS/Windows matrix reflects the published provisioning and runtime contracts.
- **Files**:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md`
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `THR-03`
  - `THR-04`
  - `THR-05`
- **Acceptance criteria**:
  - supported, unsupported, and manual-only evidence lanes are explicit
  - no platform row presents APT-only or runtime-mutation truth

#### S1.T2 - Make the manual playbook closeout-ready

- **Outcome**:
  - the manual testing playbook becomes a closeout-ready evidence input instead of a loose follow-up note.
- **Files**:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
- **Thread/contract refs**:
  - `THR-04`
  - `THR-05`
  - `REM-002`
- **Acceptance criteria**:
  - manual Arch-on-macOS fixture assumptions are explicit
  - unsupported Linux/Windows lanes remain fail-closed and operator-readable
