---
seam_id: SEAM-04
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-04-fallback-probe-failure-taxonomy/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - SEAM-01
    - SEAM-02
    - SEAM-03
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - fixed probe order changes
    - warning template or placement changes
    - `pkg_manager.source=path_probe` semantics changes
    - exit `4` remediation wording changes
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-04 Fallback Probe And Failure Taxonomy

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-04-fallback-probe-failure-taxonomy/slice-4-seam-exit-gate.md`
- **Landed evidence**:
  - Commit `0fb1a328` landed the fixed-order `PATH` probe selection in `scripts/substrate/install-substrate.sh`, probing the supported manager vocabulary in contract order, selecting the earliest detected manager, and recording `PKG_MANAGER_SOURCE=path_probe`; `tests/installers/pkg_manager_detection_smoke.sh` now covers single-manager and ordered multi-manager path-probe selection behavior.
  - Commit `61b7f631` landed the exact multi-manager warning behavior in `scripts/substrate/install-substrate.sh`, emitting the contract warning line exactly once to stderr before the inherited decision line when more than one supported manager is detected in `PATH`; `tests/installers/pkg_manager_detection_smoke.sh` now asserts warning text, one-time emission, and warning-before-decision-line ordering.
  - Commit `b72c1c52` landed the no-manager exit `4` remediation posture in `scripts/substrate/install-substrate.sh`, failing closed with the required remediation elements when no supported manager is selected after explicit selectors, os-release mapping, and fixed-order `PATH` probing; `tests/installers/pkg_manager_detection_smoke.sh` now asserts exit `4`, required remediation text, decision-line suppression, and no install attempt on the no-manager branch.
- **Contracts published or changed**: `C-07`
- **Threads published / advanced**: `THR-04` published
- **Review-surface delta**: `review.md` concerns about raw-PATH drift, warning-text or placement drift, and no-manager fallback collapse are resolved by landed installer behavior plus smoke-harness coverage; no delta widened `SEAM-04` into wrapper/docs propagation, validation topology, or other non-fallback delivery work.
- **Planned-vs-landed delta**: no contract-scope expansion landed. The final repo evidence stayed within `SEAM-04` fallback ownership: fixed-order path-probe selection, exact multi-manager warning behavior, and exit `4` remediation posture in the installer plus targeted smoke coverage.
- **Downstream stale triggers raised**:
  - fixed probe order changes
  - warning template or placement changes
  - `pkg_manager.source=path_probe` semantics changes
  - exit `4` remediation wording changes
- **Remediation disposition**: no post-exec remediations opened
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
