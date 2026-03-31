---
seam_id: SEAM-1
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-1-doctor-text-disable-attribution/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: blocked
basis:
  currentness: current
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - doctor message-body set changed after seam-local review
    - precedence or source-unknown fallback truth changed after landing
    - downstream health or JSON consumers report drift against published doctor truth
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations:
  - REM-001
---

# Closeout - SEAM-1 Doctor text disable attribution

## Seam-exit gate record

- **Source artifact**: [`threaded-seams/seam-1-doctor-text-disable-attribution/slice-4-seam-exit-gate.md`](/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/make-doctor-health-output-explain-why-fse/threaded-seams/seam-1-doctor-text-disable-attribution/slice-4-seam-exit-gate.md)
- **Landed evidence**:
  - `b7dfad40` (`SEAM-1: complete slice-2-shared-helper-and-winner-mapping-tests`) added the provenance-backed `world_disable_attribution_message(...)` helper plus the winner-mapping and fallback tests in [`crates/shell/src/execution/config_model.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/execution/config_model.rs).
  - `90104f8b` (`fix: wire doctor disable attribution text`) plumbed the helper through [`crates/shell/src/execution/platform/mod.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/execution/platform/mod.rs), [`crates/shell/src/execution/platform/linux.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/execution/platform/linux.rs), [`crates/shell/src/execution/platform/macos.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/execution/platform/macos.rs), and [`crates/shell/src/execution/platform/windows.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/execution/platform/windows.rs), and added text-surface coverage in [`crates/shell/tests/doctor_scopes_ds0.rs`](/home/spenser/__Active_code/substrate/crates/shell/tests/doctor_scopes_ds0.rs).
  - Verified locally on Linux:
    - `cargo test -p shell --test doctor_scopes_ds0 host_doctor_text_ -- --nocapture`
    - `cargo test -p shell --test doctor_scopes_ds0 world_doctor_text_ -- --nocapture`
    - `cargo fmt --all -- --check`
- **Contracts published or changed**: `C-01`, `C-02`
- **Threads published / advanced**: `THR-01`, `THR-02`
- **Review-surface delta**: doctor entrypoints now resolve explain provenance once, derive the attribution line from the shared helper, and preserve enabled-case omission while keeping the existing exit codes and status framing intact
- **Planned-vs-landed delta**: the planned split between proof and publication held; Linux text-surface proof landed, but macOS and Windows runtime parity were updated only at the code level in this environment and were not executed here
- **Downstream stale triggers raised**: any future change to the `world.enabled` precedence chain, exact message bodies, tokenized display-path rules, or safe fallback behavior must revalidate `SEAM-1` and downstream consumers
- **Remediation disposition**: `REM-001` records the missing macOS/Windows runtime parity proof for doctor text surfaces; the code paths are landed, the current closeout already publishes `C-01` / `C-02` for downstream consumption, and any later proof drift must revalidate downstream consumers rather than silently changing the handoff
- **Promotion blockers**: native macOS and Windows runtime doctor parity evidence is still missing for `SEAM-1` promotion readiness; this is a closeout blocker, not a missing consumed-contract blocker for `SEAM-2`
- **Promotion readiness**: blocked

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**:
  - `REM-001`
- **Carried-forward remediations**:
  - `REM-001`
