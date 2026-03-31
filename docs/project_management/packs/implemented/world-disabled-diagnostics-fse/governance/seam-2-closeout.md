---
seam_id: SEAM-2
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-2-shim-doctor-disabled-aware-reporting/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - seam-1-closeout.md
  required_threads:
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - shim payload field-path drift in crates/shell/src/builtins/shim_doctor/report.rs
    - disabled-mode omission rule drift in crates/shell/src/builtins/shim_doctor/report.rs
    - exact disabled-mode copy drift in crates/shell/src/builtins/shim_doctor/output.rs
    - test coverage drift in crates/shell/tests/shim_doctor.rs
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Shim doctor disabled-aware reporting

This is a post-exec scaffold. Do not treat it as landed evidence until the seam-local exit slice exists and the fields below are populated from real landed behavior.

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-2-shim-doctor-disabled-aware-reporting/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - [`crates/shell/src/builtins/shim_doctor/report.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/shim_doctor/report.rs) is the disabled-gating locus. `build_report(...)` now resolves `effective_world_enabled`, short-circuits disabled mode before any world/world-deps probe helpers run, and publishes the canonical `.world.status` / `.world_deps.status` contract.
  - [`crates/shell/src/builtins/shim_doctor/output.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/shim_doctor/output.rs) is the rendering locus. The text path now renders from status enums, prints the exact disabled contract lines, and suppresses disabled/skipped `Error:` lines while preserving enabled-mode fail-visible output.
  - [`crates/shell/tests/shim_doctor.rs`](/home/spenser/__Active_code/substrate/crates/shell/tests/shim_doctor.rs) is the regression-evidence surface. It now covers disabled JSON statuses and omission rules, exact disabled text lines, disabled no-probe behavior with fixtures present, and forced-enabled probe-backed behavior.
  - Verified command result: `cargo test -p shell --test shim_doctor -- --nocapture` passed after the S2 landing.
  - Manual disabled-mode repro with `target/debug/substrate` and `SUBSTRATE_OVERRIDE_WORLD=disabled` produced the exact text lines:
    - `World backend:`
    - `  Status: disabled`
    - `  Next: run \`substrate world enable\` to provision`
    - `World deps:`
    - `  Status: skipped (world disabled)`
  - Matching manual JSON repro produced `.world.status = "disabled"` and `.world_deps.status = "skipped_disabled"` with no disabled-path `world.error`, `world.stderr`, `world.exit_code`, `world.details`, `world_deps.error`, or `world_deps.report` fields present.
- **Contracts published or changed**: `C-02`, `C-03`, `C-04`
- **Threads published / advanced**: `THR-02`, `THR-03`, `THR-04`
- **Review-surface delta**:
  - shim doctor now branches on the published effective-config classifier before any disabled-path probes
  - shim JSON now exposes canonical status enums instead of relying on legacy booleans or error strings alone
  - disabled text output is locked to the exact source-pack contract lines
  - disabled no-probe behavior is enforced even when fixtures exist and would otherwise surface probe-backed failures
- **Planned-vs-landed delta**:
  - landed behavior matches the planned seam outcome for disabled gating, status publication, omission rules, and exact disabled text
  - landed disabled JSON omission is slightly stricter than the seam-local shorthand: it also omits `world.stderr` and `world.exit_code`, matching the authoritative source-pack playbook
- **Downstream stale triggers raised**:
  - any reintroduced disabled-path world backend probe or `substrate world doctor --json` call in `shim_doctor/report.rs`
  - any disabled-path world-deps applied probe or legacy error/report backfill in `shim_doctor/report.rs`
  - any change to the exact disabled copy lines in `shim_doctor/output.rs`
  - any test removal or weakening that stops proving the disabled no-probe boundary in `shim_doctor.rs`
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
