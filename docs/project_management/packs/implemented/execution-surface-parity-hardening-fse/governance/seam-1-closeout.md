---
seam_id: SEAM-1
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-1-execution-contract-surfaces/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - S00 58e52bf0
    - S1 877018bb
    - S2 b0dd5157
    - S3 e94145f7
  required_threads:
    - THR-01
  stale_triggers:
    - any change to the canonical four-case routing matrix or allowed-domain canonicalization
    - any replay-local heuristic replacing the shared policy-snapshot contract
    - any change to `SUBSTRATE_ENABLE_PREEXEC`, `builtin_command`, or canonical trace omission semantics
    - any change to Case B expectations in the WPEP playbook or smoke script without matching published behavior
    - any doc that still implies `world_process_*` is merely planned or that canonical trace may contain raw builtin/preexec command bodies
    - any doc that blurs landed Linux-backed telemetry, Windows degrade-only behavior, and future WPEP3 redaction hardening
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-1 Execution contract surfaces

Execution landed and the producer contract is now published.

## Seam-exit gate record

- **Source artifact**: `S99` at `docs/project_management/packs/draft/execution-surface-parity-hardening-fse/threaded-seams/seam-1-execution-contract-surfaces/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - `S00` `58e52bf0`
  - `S1` `877018bb`
  - `S2` `b0dd5157`
  - `S3` `e94145f7`
  - publication surfaces:
    - `docs/REPLAY.md`
    - `docs/TRACE.md`
    - `docs/internals/env/inventory.md`
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/contract.md`
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP1-spec.md`
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP2-spec.md`
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP3-spec.md`
  - validation surfaces:
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`
  - validation commands:
    - `cargo test -p agent-api-types policy_snapshot_v3_resolve_world_network_routing_matches_four_case_matrix -- --nocapture`
    - `cargo test -p shell world_network_policy_ -- --nocapture`
    - `cargo test -p substrate-replay build_agent_execute_request_populates_world_network_from_shared_helper -- --nocapture`
    - `cargo test -p substrate-replay world_net_filter_from_process_env_reads_exported_runtime_state -- --nocapture`
    - `cargo test -p shell test_wrap_mode_builtin_command_omits_command_body_in_trace -- --nocapture`
    - `cargo test -p shell shell_env_script_mode_sets_preexec_flag_for_bash -- --nocapture`
    - `cargo test -p shell shell_env_wrap_mode_does_not_set_preexec_flag -- --nocapture`
    - `cargo test -p shell test_command_start_finish_json_roundtrip -- --nocapture`
    - `bash -n docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`
    - `rg -n "planned|current runtime does not emit|proxy signal|preexec enabled here|does not emit these records yet" docs/REPLAY.md docs/TRACE.md docs/internals/env/inventory.md docs/project_management/packs/active/world_process_exec_tracing_parity/contract.md docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP1-spec.md docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP2-spec.md docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP3-spec.md`
- **Contracts published or changed**:
  - `C-01`: the canonical four-case replay routing contract lives in `crates/shell/src/execution/policy_snapshot.rs` and is consumed by `crates/replay/src/replay/executor.rs`
  - `C-02`: the tracing behavior matrix and omission posture live across `docs/TRACE.md`, `docs/REPLAY.md`, `docs/internals/env/inventory.md`, `docs/project_management/packs/active/world_process_exec_tracing_parity/{contract.md,SCHEMA.md,PROTOCOL.md,SECURITY.md}`, and the Case B validation surfaces `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md` and `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`
- **Threads published / advanced**:
  - `THR-01`: `identified` -> `published`
- **Review-surface delta**:
  - replay now cites the shared policy-snapshot routing contract instead of replay-local heuristics
  - the tracing contract now treats Linux-backed `world_process_*` telemetry as landed while keeping Windows degrade-only and WPEP3 redaction hardening separate
  - operator-facing validation surfaces now match the wrap/script/preexec matrix and builtin omission posture
- **Planned-vs-landed delta**:
  - planned routing, telemetry, and validation behavior are now published as the production contract instead of being described as future-only or proxy-only surfaces
- **Downstream stale triggers raised**:
  - any change to the canonical four-case routing matrix or allowed-domain canonicalization
  - any replay-local heuristic replacing the shared policy-snapshot contract
  - any change to `SUBSTRATE_ENABLE_PREEXEC`, `builtin_command`, or canonical trace omission semantics
  - any change to Case B expectations in the WPEP playbook or smoke script without matching published behavior
  - any doc that still implies `world_process_*` is merely planned or that canonical trace may contain raw builtin/preexec command bodies
  - any doc that blurs landed Linux-backed telemetry, Windows degrade-only behavior, and future WPEP3 redaction hardening
- **Remediation disposition**: none
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
