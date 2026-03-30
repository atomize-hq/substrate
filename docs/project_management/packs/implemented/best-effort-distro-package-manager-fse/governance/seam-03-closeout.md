---
seam_id: SEAM-03
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-03-explicit-override-selection/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - SEAM-01
    - SEAM-02
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - parser/input truth changes
    - mapping/reporting truth changes
    - supported manager vocabulary changes
    - exit `2` / `3` remediation requirements change
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-03 Explicit Override Selection

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-03-explicit-override-selection/slice-4-seam-exit-gate.md`
- **Landed evidence**:
  - `scripts/substrate/install-substrate.sh` now accepts `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`, captures `PKG_MANAGER` as the env override input, and enforces the explicit-selector precedence chain so the flag wins over env and both explicit sources run before os-release mapping or fixed-order `PATH` probing.
  - `scripts/substrate/install-substrate.sh` now records `PKG_MANAGER_SOURCE=flag|env` for successful explicit selection, reuses the inherited stable decision-line template from `SEAM-02`, and keeps explicit-selector success on the same operator-facing reporting vocabulary instead of introducing a second decision line.
  - `scripts/substrate/install-substrate.sh` now fails invalid explicit selectors with exit `2` and missing explicit managers with exit `3`, naming the selecting source, the offending or selected value, the exact allowed manager vocabulary, and the required rerun or install remediation without recovering into lower-precedence selection stages.
  - `tests/installers/pkg_manager_detection_smoke.sh` now covers flag-over-env precedence, env-over-os-release behavior, one-time `source: flag|env` decision-line emission before install commands, invalid explicit-selector exit `2` remediation, and missing explicit-manager exit `3` remediation and ordering.
- **Contracts published or changed**: `C-05`, `C-06`
- **Threads published / advanced**: `THR-03` published
- **Review-surface delta**: `review.md` R1 and R2 concerns about explicit-selector fallthrough, decision-line drift, and exit `2` / `3` taxonomy drift are resolved by landed installer behavior plus smoke coverage; no new delta widened this seam into `SEAM-04` PATH-warning or exit `4` work, wrapper propagation, or docs work.
- **Planned-vs-landed delta**: no contract-scope expansion landed. Explicit-selector precedence, inherited reporting, and fail-closed explicit failure taxonomy shipped in the installer and smoke harness only, and the landed work stayed within the four-slice seam boundary.
- **Downstream stale triggers raised**:
  - selector precedence changes
  - supported manager vocabulary changes
  - exit `2` / `3` remediation wording changes
  - decision-line interaction for explicit-selector success changes
- **Remediation disposition**: no post-exec remediations opened
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
