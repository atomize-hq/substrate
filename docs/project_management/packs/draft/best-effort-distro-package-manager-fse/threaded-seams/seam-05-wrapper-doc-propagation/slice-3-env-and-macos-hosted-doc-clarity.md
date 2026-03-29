---
slice_id: S3
seam_id: SEAM-05
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - env-hook semantics change
    - macOS-hosted wording drift
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
  - THR-04
  - THR-05
contracts_produced:
  - C-09
contracts_consumed:
  - C-01
  - C-02
  - C-07
open_remediations: []
candidate_subslices: []
---
### S3 - Env and macOS-hosted doc clarity

- **User/system value**: env docs and macOS-hosted wording stay exact about Linux-only installer semantics and Lima-backed hosted-install coverage.
- **Scope (in/out)**:
  - In: `docs/reference/env/contract.md` propagation of `PKG_MANAGER` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, plus macOS-hosted Lima-backed wording in documentation surfaces
  - Out: direct macOS-native package-manager-selection logic, wrapper implementation, and validation evidence execution
- **Acceptance criteria**:
  - env docs keep allowed values, precedence, and hook semantics exact
  - docs state that package-manager selection remains Linux-scoped
  - macOS-hosted wording makes the Lima-backed Linux path explicit without overstating native macOS behavior
- **Dependencies**:
  - `S2`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../../seam-01-os-release-input-parser.md`
- **Verification**:
  - doc review proves Linux-only scope and Lima-backed wording remain coherent and non-conflicting

