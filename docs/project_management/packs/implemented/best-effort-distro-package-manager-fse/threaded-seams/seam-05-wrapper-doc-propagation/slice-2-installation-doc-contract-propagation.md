---
slice_id: S2
seam_id: SEAM-05
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - decision-line wording changes
    - warning or remediation wording changes
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
  - THR-05
contracts_produced:
  - C-09
contracts_consumed:
  - C-04
  - C-06
  - C-07
open_remediations: []
candidate_subslices: []
---
### S2 - Installation doc contract propagation

- **User/system value**: installation docs become a faithful consumer of the landed installer contract instead of a second authority with drift.
- **Scope (in/out)**:
  - In: `docs/INSTALLATION.md` propagation of precedence, decision line, warning, and remediation truth
  - Out: env-contract details unique to `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, wrapper implementation, and validation topology
- **Acceptance criteria**:
  - installation docs restate the precedence chain exactly
  - warning and remediation wording stay aligned with upstream operator-facing truth
  - docs do not weaken the Linux-scoped contract or restate alternate meanings
- **Dependencies**:
  - `S1`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../seam-04-fallback-probe-failure-taxonomy/seam.md`
- **Verification**:
  - doc review proves no wording drift against the contract-owned vocabulary

