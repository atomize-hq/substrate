---
seam_id: SEAM-2
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-2-managed-cleanup-protected-path-guard/slice-3-seam-exit-gate.md
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
    - manifest location or schema changed after landing
    - cleanup ownership rules changed after landing
    - protected-path refusal wording or exit mapping changed after landing
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Managed cleanup + protected-path guard

This record captures the landed exit-gate evidence for SEAM-2.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-2-managed-cleanup-protected-path-guard/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `42865b2a` published `C-04` in `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/contract.md`, freezing the managed-cleanup + protected-path refusal contract for downstream consumers.
  - `e9d77493` landed protected-path refusal and reporting in `scripts/substrate/dev-uninstall-substrate.sh`, with fixture coverage in `tests/mac/installer_parity_fixture.sh`.
  - Verification evidence for the seam-exit slice:
    - `bash -n scripts/substrate/dev-uninstall-substrate.sh`
    - `bash -n tests/mac/installer_parity_fixture.sh`
    - `git diff --check`
    - `tests/mac/installer_parity_fixture.sh --scenario dev-runtime-bundle-self-contained`
    - `tests/mac/installer_parity_fixture.sh --scenario dev-runtime-bundle-protected-path-conflicts`
- **Contracts published or changed**: `C-04`
- **Threads published / advanced**: `THR-03` published
- **Review-surface delta**:
  - protected-path refusal is now closeout-backed rather than scaffold-only
  - refusal reporting and preserved-path behavior are recorded as landed evidence for downstream conformance
- **Planned-vs-landed delta**: no scope expansion landed; the exit gate recorded the managed-cleanup truth and evidence only
- **Downstream stale triggers raised**:
  - any later change to refusal classification or preserved-path messaging
  - any later change to manifest-backed Linux guest binary cleanup rules
  - any later change to directory-pruning behavior around the fixed bundle surface
- **Remediation disposition**: none open
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
