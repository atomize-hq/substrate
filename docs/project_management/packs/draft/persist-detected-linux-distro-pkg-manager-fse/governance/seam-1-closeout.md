---
seam_id: SEAM-1
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-1-persisted-platform-metadata-contract/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-03
  stale_triggers:
    - upstream detection contract changes selected-manager or source vocabulary
    - field-path, alias wording, or authority-boundary truth changes
    - outbound thread publication state changes
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-1 Persisted platform metadata contract

This record captures the landed exit-gate evidence for SEAM-1.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-1-persisted-platform-metadata-contract/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` now freezes one canonical `<effective_prefix>/install_state.json` path, keeps `$SUBSTRATE_HOME/install_state.json` as the default-prefix alias, states additive `schema_version = 1` behavior, preserves `host_state.group`, `host_state.linger`, and unknown keys, and records upstream ownership for `pkg_manager.selected` and `pkg_manager.source`.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` now matches the canonical path story, defines the exact `host_state.platform.*` schema shape, and defers package-manager vocabulary ownership to `best-effort-distro-package-manager/contract.md`.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` now aligns the accepted decisions with additive `schema_version = 1`, nested `host_state.platform.*` placement, upstream vocabulary ownership, and the shared successful-Linux write contract.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager-fse/threading.md` now reflects `THR-01` and `THR-03` as published in the live registry.
- **Contracts published or changed**: `C-01`, `C-02`
- **Threads published / advanced**: `THR-01` published, `THR-03` published
- **Review-surface delta**: The payload-schema and path-authority views moved from inferred seam intent to landed documentary truth; downstream seams now inherit one explicit additive schema and one explicit canonical-path rule.
- **Planned-vs-landed delta**: No contract-scope expansion landed. The seam stayed documentary and did not pull in runtime writer behavior, smoke assertions, or docs rewrites.
- **Downstream stale triggers raised**:
  - any later change to `host_state.platform.*` field paths or additive-merge rules
  - any later change to `<effective_prefix>/install_state.json` versus `$SUBSTRATE_HOME/install_state.json` wording
  - any later change to the upstream-owned `pkg_manager.selected` or `pkg_manager.source` vocabulary
- **Remediation disposition**: `REM-001` is resolved by the accepted canonical-path override and the landed contract/spec alignment; `REM-002` and `REM-003` remain open in their owning seams and do not block SEAM-1 promotion.
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
