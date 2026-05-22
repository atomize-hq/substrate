---
seam_id: SEAM-2
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-2-linux-dev-install-world-service-staging/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-03
  stale_triggers:
    - accepted staged path set or sufficiency rule changed after landing
    - selected-profile mapping changed after landing
    - ln -sfn refresh semantics changed after landing
    - scripts/substrate/install-substrate.sh scope changed after landing
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Linux dev-install world-service staging

This record captures the landed exit-gate evidence for SEAM-2.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-2-linux-dev-install-world-service-staging/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `b10ce730` (`SEAM-2: complete slice-1-c-04-contract-and-installer-scope`) published the `C-04` contract definition and froze the installer scope boundary as regression-only reference posture.
  - `97166902` (`SEAM-2: complete slice-2-linux-staging-and-refresh-behavior`) landed the Linux staging behavior for the accepted bridge paths, selected-profile mapping, refresh semantics, and disabled-world invariants.
  - `bash -n scripts/substrate/dev-install-substrate.sh`
  - `bash -n tests/installers/install_smoke.sh`
  - `tests/installers/install_smoke.sh --scenario dev`
  - `tests/installers/install_smoke.sh --scenario dev-no-world`
- **Contracts published or changed**: `C-04`
- **Threads published / advanced**: `THR-03`
- **Review-surface delta**: the Linux dev-install path now stages the accepted `world-service` bridge layout for `debug` and `release`, while `scripts/substrate/install-substrate.sh` remains reference-only for this seam.
- **Planned-vs-landed delta**: the seam-exit gate now binds to landed staging and smoke evidence instead of the initial placeholder scaffold.
- **Downstream stale triggers raised**:
  - any later change to selected-profile mapping or staged-link paths
  - any later change to `ln -sfn` refresh semantics or disabled-world posture
  - any later change that broadens production-installer scope beyond regression-only status
- **Remediation disposition**: `REM-002` is resolved by the landed slice-1 contract and installer-scope boundary.
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
