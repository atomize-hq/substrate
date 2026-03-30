---
seam_id: SEAM-05
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-05-wrapper-doc-propagation/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - SEAM-02
    - SEAM-03
    - SEAM-04
  required_threads:
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - wrapper exit handling changes
    - decision-line wording or placement changes
    - warning or remediation wording changes
    - env-hook semantics change
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-05 Wrapper And Doc Propagation

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-05-wrapper-doc-propagation/slice-4-seam-exit-gate.md`
- **Landed evidence**:
  - Commit `e3f9689b` landed wrapper pass-through parity in `scripts/substrate/install.sh`, preserving the direct installer exit statuses `0`, `2`, `3`, and `4` instead of collapsing feature-specific failures to `1`; `tests/installers/install_wrapper_smoke.sh` now covers wrapper exit pass-through for those exact statuses and preserves the wrapper success/failure output posture.
  - Commit `7406f2b3` landed the `docs/INSTALLATION.md` propagation for the Linux hosted installer contract, carrying forward the exact package-manager precedence chain, the stable decision-line template, the fixed-order multi-manager warning template, and the exit `2` / `3` / `4` remediation posture without introducing a second operator-facing authority.
  - Commit `c316c40b` landed `docs/reference/env/contract.md` entries for `PKG_MANAGER` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, preserving their exact precedence, Linux-hosted scope, and absence semantics, and updated `docs/WORLD.md` so macOS-hosted installer coverage is explicitly described as traversing the Lima-backed Linux guest path without claiming native macOS package-manager-selection behavior.
- **Contracts published or changed**: `C-08`, `C-09`
- **Threads published / advanced**: `THR-05`
- **Review-surface delta**: `review.md` concerns about wrapper exit-class collapse, installation/env doc wording drift, and macOS-hosted wording ambiguity are resolved by the landed wrapper smoke coverage plus the doc propagation commits; no delta widened `SEAM-05` into repo-harness ownership, checkpoint execution, or downstream handoff work.
- **Planned-vs-landed delta**: no contract-scope expansion landed. `SEAM-05` shipped only the wrapper pass-through parity, no-drift installation/env doc propagation, and Lima-backed macOS-hosted wording clarification described in the seam plan.
- **Downstream stale triggers raised**:
  - wrapper exit handling changes
  - decision-line wording or placement changes
  - warning or remediation wording changes
  - env-hook semantics change
- **Remediation disposition**: no post-exec remediations opened
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
