---
seam_id: SEAM-02
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-02-family-mapping-reporting/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - SEAM-01
  required_threads:
    - THR-01
  stale_triggers:
    - parser/input truth from SEAM-01 changes
    - family-table rules change
    - decision-line wording or placement changes
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-02 Family Mapping And Decision-Line Reporting

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-02-family-mapping-reporting/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `scripts/substrate/install-substrate.sh` now maps Debian or Ubuntu, Fedora or RHEL, Arch, and SUSE families using only `DETECTED_DISTRO_ID` and `DETECTED_DISTRO_ID_LIKE`, applies availability-based selection, and keeps Fedora or RHEL on `dnf` before `yum`.
  - `scripts/substrate/install-substrate.sh` now records `PKG_MANAGER_SOURCE=os_release` for mapped selections and emits the exact raw decision line once to stderr before the first package-manager install command when the os-release stage selects a manager.
  - `scripts/substrate/install-substrate.sh` keeps no-selection branches silent at this stage so later PATH fallback work remains separate.
  - `tests/installers/pkg_manager_detection_smoke.sh` now covers Debian-family, Fedora or RHEL, Arch, and SUSE mapping; mapped-manager unavailability fallthrough; and one-time decision-line placement and suppression.
- **Contracts published or changed**: `C-03`, `C-04`
- **Threads published / advanced**: `THR-02` published, `THR-08` published
- **Review-surface delta**: `review.md` R1 and R2 concerns about family-table drift, Fedora or RHEL preference drift, and decision-line leakage are resolved by landed installer behavior plus smoke coverage; no new delta widened this seam into explicit-selector, PATH-fallback, wrapper, or docs work.
- **Planned-vs-landed delta**: no contract-scope expansion landed. Mapping and decision-line work shipped in the installer and smoke harness only, and the final correction stayed within the same decision-line slice boundary.
- **Downstream stale triggers raised**:
  - family-table rules change
  - decision-line wording, placement, or suppression changes
  - `pkg_manager.source=os_release` semantics change
- **Remediation disposition**: no post-exec remediations opened
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
