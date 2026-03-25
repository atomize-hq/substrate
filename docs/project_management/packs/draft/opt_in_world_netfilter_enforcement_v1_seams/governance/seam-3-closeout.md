---
seam_id: SEAM-3
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref:
  status: pending
  promotion_readiness: blocked
basis:
  currentness: stale
  upstream_closeouts: []
  required_threads:
    - THR-03
  stale_triggers: []
gates:
  post_exec:
    landing: pending
    closeout: pending
open_remediations:
  - REM-003
  - REM-004
---

# Closeout - SEAM-3 Config opt-in `world.net.filter` + CLI patching + env parity

## Seam-exit gate record

- **Source artifact**: pending; `SEAM-3` has not yet landed its final exit slice / closeout evidence.
- **Landed evidence**: none recorded yet for `world.net.filter` config plumbing, CLI patching, override handling, export parity, or operator docs.
- **Contracts published or changed**: planned `C-04`, `C-05`, `C-06`; the owner execution surfaces now live in `threaded-seams/seam-3-host-config-opt-in-and-parity-env-plumbing/slice-1-publish-world-net-filter-config-contract.md`, `slice-2-override-and-parity-env-plumbing.md`, and `slice-3-operator-docs-and-routing-handoff.md`.
- **Threads published / advanced**: `THR-03` remains `identified`; it is not yet published from a landed `SEAM-3` artifact.
- **Review-surface delta**: once implemented, operators must be able to see one consistent story across config precedence, parity export, `WORLD_NETFILTER_ENABLE`, and policy `net_allowed`.
- **Planned-vs-landed delta**:
  - `crates/shell/src/execution/config_model.rs` does not yet expose `world.net.filter`.
  - `crates/shell/src/execution/env_scripts.rs` does not yet export `SUBSTRATE_WORLD_NET_FILTER`.
  - `docs/reference/config/world.md` and `docs/CONFIGURATION.md` do not yet publish the three-way gate contract.
  - Host routing still uses broker-derived allowlist plumbing in multiple paths, so `SEAM-1` cannot claim an upstream-published config gate yet.
- **Downstream stale triggers raised**:
  - Any change to `world.net.filter` precedence, override parsing, or exported env naming
  - Any change to when host routing treats policy `net_allowed=["*"]` as allow-all instead of an enforcement request
- **Remediation disposition**:
  - `REM-003` remains open until the operator workflow is documented with examples.
  - `REM-004` remains open until `C-04` / `THR-03` are implemented and this closeout can cite landed evidence instead of a planning artifact.
- **Promotion blockers**:
  - Land `threaded-seams/seam-3-host-config-opt-in-and-parity-env-plumbing/slice-1-publish-world-net-filter-config-contract.md`.
  - Land `threaded-seams/seam-3-host-config-opt-in-and-parity-env-plumbing/slice-2-override-and-parity-env-plumbing.md`.
  - Land `threaded-seams/seam-3-host-config-opt-in-and-parity-env-plumbing/slice-3-operator-docs-and-routing-handoff.md`.
  - Revalidate next `SEAM-1` against the published host gate and parity outputs before attempting `exec-ready`.
- **Promotion readiness**: blocked

## Post-exec gate disposition

- **Landing gate**: pending
- **Closeout gate**: pending
- **Unresolved remediations**:
  - `REM-003`
  - `REM-004`
- **Carried-forward remediations**: none
