---
seam_id: SEAM-5
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-5-verification-and-smoke-conformance/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - seam-1-closeout.md
    - seam-2-closeout.md
    - seam-3-closeout.md
    - seam-4-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - "Any change to net_allowed canonicalization/validation rules or world_network routing semantics requires SEAM-5 revalidation."
    - "Any change to world.net.filter precedence, override applicability, or exported parity env semantics requires SEAM-5 revalidation."
    - "Any change to runtime failure taxonomy, attach-or-fail behavior, or deny-all DNS semantics requires SEAM-5 revalidation."
    - "Any change to doctor endpoint schema, field naming, or shell-side rendering/passthrough requires SEAM-5 revalidation."
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-5 Conformance: tests + smoke prevent drift

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-5-verification-and-smoke-conformance/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `cargo test -p transport-api-types -- --nocapture` passed with `15 passed; 0 failed`, including `world_doctor_report_v1_schema_round_trip`, `world_doctor_report_v1_serializes_null_last_failure_reason_when_absent`, `world_doctor_report_v1_defaults_last_failure_reason_when_missing`, and `world_doctor_report_v1_serializes_exact_netfilter_status_field_names`, preserving the additive schema boundary that `SEAM-5` consumes.
  - `cargo test -p shell --test world_request_net_allowed_snapshot -- --nocapture` passed with `1 passed; 0 failed`; `nonpty_world_request_obeys_net_allowed_routing_matrix` keeps allow-all (`["*"]`), deny-all (`[]`), and restrictive routing semantics aligned with the effective `world.net.filter` host gate.
  - `cargo test -p shell --test doctor_scopes_ds0 -- --nocapture` passed with `3 passed; 0 failed`, preserving the world-doctor JSON envelope for the published `requested`, `enabled`, `world_netfilter_enable_present`, and `last_failure_reason` fields.
  - `cargo test -p shell --test shim_doctor -- --nocapture` passed with `11 passed; 0 failed`, including `shim_doctor_json_preserves_world_netfilter_default_details`, `shim_doctor_json_preserves_world_netfilter_enabled_details`, and `shim_doctor_json_preserves_world_netfilter_failure_reason_details`, proving downstream shim surfaces preserve the same doctor contract.
  - `docs/manual_verification/netfilter_enforcement.md` now publishes the privileged Linux verification path around `cargo test -p world -- --ignored --nocapture`, including prerequisites, expected pass/skip/failure evidence, and the exact closeout capture requirements for the ignored `crates/world/src/netfilter.rs` nftables coverage.
  - `scripts/mac/smoke.sh` now includes `--netfilter-conformance`, which warms Lima with `SUBSTRATE_WORLD_NETFILTER_ENABLE=1`, exercises allow-all and deny-all postures, and writes the doctor JSON plus probe transcripts expected by closeout; `docs/cross-platform/mac_world_setup.md` and `docs/manual_verification/netfilter_enforcement.md` publish the same warm/smoke commands, expected doctor states, and artifact names (`allow-all-world-doctor.json`, `deny-all-world-doctor.json`).
- **Contracts published or changed**:
  - none; `SEAM-5` consumes `C-01` through `C-07` and turns them into conformance evidence without publishing a new contract.
- **Threads published / advanced**:
  - `THR-05` is consumed and revalidated at the terminal boundary: the doctor observability contract now has schema, shell, and shim conformance coverage plus operator smoke guidance.
  - `THR-01` and `THR-02` are revalidated and closed at the pack boundary through the landed routing/conformance matrix, which binds Snapshot V3 `net_allowed`, host gate semantics, and `world_network` request construction together.
  - `THR-03` and `THR-04` are revalidated and closed at the pack boundary through the landed privileged-verification and macOS smoke surfaces, which keep host gate/env propagation and fail-closed runtime expectations operator-visible.
- **Review-surface delta**:
  - The pack now has one terminal conformance bundle instead of separate upstream truths: schema compatibility, host routing, runtime failure semantics, doctor passthrough, privileged verification guidance, and macOS Lima smoke all point at the same three-way gate story.
  - Operators and maintainers can now verify allow-all versus deny-all behavior against the same doctor fields and runtime expectations that the regression suites pin in code.
- **Planned-vs-landed delta**:
  - No contract drift was found; the landed regression suites and operator-facing conformance surfaces match the planned `SEAM-5` scope.
  - Local execution on this Darwin host could not produce privileged Linux nftables evidence: `cargo test -p world -- --ignored --nocapture` completed with `0 tests` because the ignored privileged coverage is Linux-gated. The landed closeout therefore cites the published Linux verification command and capture requirements from `docs/manual_verification/netfilter_enforcement.md` rather than a local Linux transcript.
- **Downstream stale triggers raised**:
  - none inside this pack; future drift against the recorded basis triggers reopens conformance work outside the terminal seam rather than creating a downstream carry here.
- **Remediation disposition**:
  - none; no new seam-local remediations were opened, and no carried-forward remediation remains at the terminal boundary.
- **Promotion blockers**:
  - none; `SEAM-5` is terminal and its landed closeout completes the pack instead of promoting a downstream seam.
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**:
  - none
- **Carried-forward remediations**:
  - none; future conformance drift would reopen the pack as new work rather than carry an unresolved `SEAM-5` owner gap.
