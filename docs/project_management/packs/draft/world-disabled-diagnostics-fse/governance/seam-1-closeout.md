---
seam_id: SEAM-1
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-1-effective-config-classifier/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - effective-config precedence drift in docs/reference/env/contract.md
    - resolve_effective_config semantics or CliConfigOverrides.world_enabled changes in crates/shell/src/execution/config_model.rs
    - diagnostics routing or exit-code taxonomy changes for config errors in crates/shell/src/execution/routing.rs
    - adjacent diagnostics packs modifying health or shim-doctor call paths before SEAM-2 / SEAM-3 consume THR-01
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-1 Effective config classifier

This is a post-exec scaffold. Do not treat it as landed evidence until the seam-local exit slice exists and the fields below are populated from real landed behavior.

## Seam-exit gate record

- **Source artifact**: [`threaded-seams/seam-1-effective-config-classifier/slice-3-seam-exit-gate.md`](../threaded-seams/seam-1-effective-config-classifier/slice-3-seam-exit-gate.md)
- **Landed evidence**:
  - [`crates/shell/src/execution/config_model.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/execution/config_model.rs) defines `resolve_diagnostics_world_enabled(cwd: &Path, cli: &CliConfigOverrides) -> Result<bool>` as the named helper for diagnostics classification; it remains resolver-backed and preserves `resolve_effective_config` as the source of truth.
  - [`crates/shell/src/builtins/shim_doctor/report.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/shim_doctor/report.rs) is the shared reporting path used by both diagnostics commands; `substrate shim doctor` routes `commands::shim_doctor::run_doctor -> shim_doctor::collect_report -> build_report`, and `substrate health` routes `commands::health::run -> shim_doctor::collect_report`, with the same helper gathering world and world-deps snapshots.
  - [`crates/shell/src/execution/routing.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/execution/routing.rs) provides the explicit `shim doctor` user-error mapping, converting config/user errors into exit code `2` with stderr emission and preserving the fail-fast posture required by `THR-01`.
  - [`crates/shell/tests/shim_doctor.rs`](/home/spenser/__Active_code/substrate/crates/shell/tests/shim_doctor.rs) includes `shim_doctor_json_exits_2_before_output_on_invalid_workspace_yaml`, which proves invalid workspace YAML exits `2` and emits no stdout.
  - [`crates/shell/tests/shim_health.rs`](/home/spenser/__Active_code/substrate/crates/shell/tests/shim_health.rs) includes `shim_health_json_exits_2_before_output_on_invalid_workspace_yaml`, which proves the same exit-2-before-output posture for `health`.
  - Manual repro evidence: `substrate shim doctor --json` with invalid `.substrate/workspace.yaml` exited `2`, stderr began `substrate shim doctor failed: invalid YAML ...`, and `stdout_bytes=0`; `substrate health --json` with invalid `.substrate/workspace.yaml` exited `2`, stderr began `invalid YAML ...`, and `stdout_bytes=0`.
- **Contracts published or changed**: `C-01`
- **Threads published / advanced**: `THR-01` published
- **Review-surface delta**:
  - one canonical effective-config classifier now backs both diagnostics entrypoints
  - config-resolution failures terminate before probes or report output
  - `shim doctor` and `health` share the same resolver-backed classification boundary instead of local precedence heuristics
- **Planned-vs-landed delta**:
  - landed behavior matches the planned seam outcome: a shared resolver-backed classifier, exit-2-on-config-error posture, and no probe/output on invalid config
  - the only additional clarity is the explicit top-level error wrapper on `shim doctor` stderr, which is already covered by the manual repro evidence
- **Verification commands**:
  - `cargo test -p shell test_diagnostics_world_enabled --lib -- --nocapture`
  - `cargo test -p shell --test shim_doctor -- --nocapture`
  - `cargo test -p shell --test shim_health -- --nocapture`
- **Downstream stale triggers raised**:
  - effective-config precedence drift in `docs/reference/env/contract.md`
  - resolver signature or `CliConfigOverrides.world_enabled` changes in `crates/shell/src/execution/config_model.rs`
  - diagnostics routing or config-error taxonomy changes in `crates/shell/src/execution/routing.rs`
  - any later work that reintroduces probe-first behavior or local world-enabled heuristics in `shim doctor` or `health`
- **Remediation disposition**:
  - none open at closeout
- **Promotion blockers**:
  - none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
