---
seam_id: SEAM-3
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-3-smoke-and-operator-conformance/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
  required_threads:
    - THR-02
    - THR-03
  stale_triggers:
    - any later change to the canonical path wording or the four `host_state.platform.*` field names
    - any later change to the successful-Linux producer matrix or explicit no-write boundaries
    - any later change to the checkpoint or pack-closeout artifact set used for this seam
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-3 Smoke and operator conformance

This record captures the landed exit-gate evidence for SEAM-3.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-3-smoke-and-operator-conformance/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `tests/installers/install_state_smoke.sh` now freezes the Linux conformance surface for the successful-hosted and successful-dev producer paths, explicit `--no-world` branches, explicit no-write boundaries, missing `/etc/os-release` degradation, additive compatibility, and the warning-only write / replace failure posture.
  - `docs/INSTALLATION.md` now names the canonical `<effective_prefix>/install_state.json` path, keeps `~/.substrate/install_state.json` as the default alias, states `schema_version = 1`, and calls out the four persisted `host_state.platform.*` fields.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`, `tasks.json`, and `session_log.md` carry the checkpoint and evidence story that matches the landed smoke and documentation contract.
- **Contracts published or changed**: `C-05`, `C-06`
- **Threads published / advanced**: `THR-02`, `THR-03`
- **Review-surface delta**: The seam moved from source-pack intent to landed conformance proof. Smoke, operator wording, and checkpoint evidence now point at one validation story instead of three loosely coupled surfaces.
- **Planned-vs-landed delta**: No runtime writer mechanics, schema authority, or non-Linux behavior changed. The seam stayed documentary/conformance only.
- **Downstream stale triggers raised**:
  - any later change to the canonical path wording or the four `host_state.platform.*` field names
  - any later change to the successful-Linux producer matrix or explicit no-write boundaries
  - any later change to the checkpoint or pack-closeout artifact set used for this seam
- **Remediation disposition**: `REM-002` remains resolved by the landed operator wording in `docs/INSTALLATION.md` and is now carried as closeout evidence, not as a blocker.
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
