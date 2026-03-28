---
seam_id: SEAM-4
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-4-world-doctor-netfilter-status-observability/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - seam-1-closeout.md
    - seam-2-closeout.md
    - seam-3-closeout.md
  required_threads:
    - THR-05
  stale_triggers:
    - "Any change to doctor endpoint schema, field naming, or JSON placement for the netfilter block requires SEAM-5 revalidation."
    - "Any change to shell-side world doctor rendering or passthrough on Linux, macOS, or Windows requires SEAM-5 revalidation."
    - "Any change to runtime failure taxonomy or WORLD_NETFILTER_ENABLE wording inherited from SEAM-2 requires SEAM-5 revalidation."
    - "Any change to requested-state derivation inherited from SEAM-1 or SEAM-3 requires SEAM-5 revalidation."
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-4 Observability: doctor output makes enforcement status obvious

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-4-world-doctor-netfilter-status-observability/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `crates/agent-api-types/src/lib.rs` now publishes the additive `WorldDoctorNetfilterStatusV1` block with `requested`, `enabled`, `world_netfilter_enable_present`, and nullable `last_failure_reason`; serialization coverage is recorded by `cargo test -p agent-api-types -- --nocapture` with `14 passed; 0 failed`, including `world_doctor_report_v1_schema_round_trip`, `world_doctor_report_v1_serializes_null_last_failure_reason_when_absent`, and `world_doctor_report_v1_defaults_last_failure_reason_when_missing`.
  - `crates/world-agent/src/handlers.rs` populates the doctor block from the published request, guard, and failure-state surfaces via `service.last_netfilter_requested()`, `doctor_world_netfilter_enable_present()`, and `service.last_netfilter_failure_reason()`.
  - `crates/world-agent/src/handlers.rs` also carries Linux-gated handler coverage for the doctor netfilter permutations (`doctor_world_defaults_netfilter_status_when_no_request_seen`, `doctor_world_reports_requested_and_guard_presence`, `doctor_world_reports_requested_without_guard_as_disabled`, `doctor_world_surfaces_last_netfilter_failure_reason`); those tests are present in the landed source but were not runnable on this Darwin host because they are guarded with `#[cfg(target_os = "linux")]`.
  - `crates/world-agent/src/service.rs` records the published runtime failure taxonomy into `last_netfilter_failure_reason`, including missing `WORLD_NETFILTER_ENABLE`, nft failure, resolution failure, and cgroup attach failure classes.
  - `crates/shell/tests/doctor_scopes_ds0.rs` passed via `cargo test -p shell --test doctor_scopes_ds0 -- --nocapture` with `3 passed; 0 failed`, proving the world doctor JSON envelope preserves `world.netfilter_status.requested`, `enabled`, `world_netfilter_enable_present`, and `last_failure_reason`.
  - `crates/shell/tests/shim_doctor.rs` passed via `cargo test -p shell --test shim_doctor -- --nocapture` with `9 passed; 0 failed`, including `shim_doctor_json_preserves_world_netfilter_failure_reason_details`, proving downstream shell/shim doctor surfaces preserve the actionable failure-reason detail.
- **Contracts published or changed**:
  - `C-07`: `world doctor --json` now publishes the additive netfilter status block with the final landed field names `requested`, `enabled`, `world_netfilter_enable_present`, and `last_failure_reason`.
- **Threads published / advanced**:
  - `THR-05` is now published as the downstream handoff from `SEAM-4` to `SEAM-5`, carrying the additive doctor observability contract `C-07`.
- **Review-surface delta**:
  - The seam now publishes one stable operator-facing doctor contract instead of leaving requested state, env-guard presence, and failure taxonomy split across implementation details.
  - Downstream conformance can consume one shared doctor JSON shape across shell/shim surfaces instead of treating Linux/macOS/Windows as separate contracts.
- **Planned-vs-landed delta**:
  - No contract drift was found: the landed implementation matches the planned `C-07` block and field names exactly.
  - Local verification on this Darwin host could not execute the Linux-only `world-agent` doctor handler tests, but the landed source still contains those tests and the contract remained evidenced by the schema tests plus shell/shim doctor coverage.
- **Downstream stale triggers raised**:
  - Any change to doctor endpoint schema, field naming, or JSON placement for the netfilter block requires `SEAM-5` revalidation.
  - Any change to shell-side world doctor rendering or passthrough on Linux, macOS, or Windows requires `SEAM-5` revalidation.
  - Any change to runtime failure taxonomy or `WORLD_NETFILTER_ENABLE` wording inherited from `SEAM-2` requires `SEAM-5` revalidation.
  - Any change to requested-state derivation inherited from `SEAM-1` or `SEAM-3` requires `SEAM-5` revalidation.
- **Remediation disposition**:
  - No new seam-local remediations were opened; the slice closed as a governance publication pass over already-landed doctor observability work.
- **Promotion blockers**:
  - none at the `SEAM-4` boundary; `SEAM-5` remains the next seam and now owns downstream conformance and smoke revalidation against the published doctor contract.
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**:
  - none
- **Carried-forward remediations**:
  - none; downstream `SEAM-5` work is follow-on conformance, not a carried `SEAM-4` remediation.
