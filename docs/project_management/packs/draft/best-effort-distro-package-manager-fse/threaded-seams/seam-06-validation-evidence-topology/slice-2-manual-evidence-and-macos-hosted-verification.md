---
slice_id: S2
seam_id: SEAM-06
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - manual evidence expectations change
    - macOS Lima-backed verification path changes
    - wrapper/doc contract changes
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
  - C-02
  - C-07
  - C-08
  - C-09
open_remediations: []
candidate_subslices: []
---
### S2 - Manual evidence and macOS-hosted verification

- **User/system value**: manual evidence and macOS-hosted verification prove the same Linux installer contract the harness asserts instead of widening into separate behavior stories.
- **Scope (in/out)**:
  - In: `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`, `scripts/mac/smoke.sh`, and supporting documentation for Lima-backed Linux verification on macOS hosts
  - Out: repo harness ownership changes, checkpoint execution, and seam-exit publication
- **Acceptance criteria**:
  - manual evidence cases reuse the same selected-manager, warning, remediation, and wrapper-parity truth as the harness
  - macOS-hosted verification explicitly exercises the Lima-backed Linux installer path
  - conformance docs do not imply native macOS package-manager-selection behavior
- **Dependencies**:
  - `S1`
  - `../../seam-06-validation-evidence-topology.md`
  - `../../governance/seam-05-closeout.md`
  - `../../../best-effort-distro-package-manager/contract.md`
- **Verification**:
  - validation review proves manual and macOS-hosted evidence stay aligned to the authoritative topology
