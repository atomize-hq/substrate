---
seam_id: SEAM-3
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-3-cross-surface-parity-and-drift-guards/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - S1 bd89c909
    - S2 92e4de25
    - S3 9128c6c2
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - `docs/REPLAY.md`, `docs/TRACE.md`, `docs/USAGE.md`, or the WPEP playbook/smoke assets drift away from the published `SEAM-1` or `SEAM-2` closeouts
    - replay-routing, tracing-validation, or abnormal-terminal-loss regression surfaces stop proving the same contracts described in the operator docs
    - `SEAM-1` or `SEAM-2` closeout stale-trigger subjects change after this seam decomposes
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-3 Cross-surface parity and drift guards

Execution landed and the consumer seam now records the cross-surface revalidation evidence.

## Seam-exit gate record

- **Source artifact**: `S99` at `docs/project_management/packs/draft/execution-surface-parity-hardening-fse/threaded-seams/seam-3-cross-surface-parity-and-drift-guards/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - `S1` `bd89c909`:
    - `docs/REPLAY.md`
    - `docs/TRACE.md`
    - `docs/USAGE.md`
  - `S2` `92e4de25`:
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
    - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`
  - `S3` `9128c6c2`:
    - `crates/replay/src/replay/executor.rs`
    - `crates/shell/tests/logging.rs`
    - `crates/shell/tests/world_process_exec_tracing_parity_wpep3.rs`
  - upstream closeouts:
    - `docs/project_management/packs/draft/execution-surface-parity-hardening-fse/governance/seam-1-closeout.md`
    - `docs/project_management/packs/draft/execution-surface-parity-hardening-fse/governance/seam-2-closeout.md`
- **Contracts published or changed**: none. SEAM-3 consumes the already-published `C-01`, `C-02`, and `C-03` contracts and does not publish a new runtime contract.
- **Threads published / advanced**:
  - `THR-01`: remains `revalidated`
  - `THR-02`: remains `revalidated`
- **Review-surface delta**:
  - downstream docs now name the canonical shared replay-routing contract instead of implying replay-local heuristics
  - the tracing docs now state the Linux-backed `world_process_*` posture explicitly and keep builtin/preexec trace-body omission non-negotiable
  - the operator usage doc now preserves the `0` versus `1` exit split and bounded diagnostic posture without overstating platform proof
  - the WPEP playbook and smoke script now speak one contract language for Case B, Linux joinability, argv omission/capture posture, and non-Linux degrade summaries
  - regression coverage now pins replay-routing and WPEP3 Linux capture posture directly, including the Linux-only WPEP3 test surface
- **Planned-vs-landed delta**:
  - this seam remained conformance-only; the landed result is documentation, playbook, smoke, and regression lock-in on already-published contracts
  - S3 added Linux-only WPEP3 regression coverage, but on this macOS host that test surface is evidence by readback rather than local execution
- **Downstream stale triggers raised**:
  - any future drift in `docs/REPLAY.md`, `docs/TRACE.md`, `docs/USAGE.md`, or the WPEP playbook/smoke assets away from the published `SEAM-1` or `SEAM-2` closeouts
  - any future change that weakens replay-routing, tracing-validation, or abnormal-terminal-loss regression surfaces as proof of the published contracts
  - any future change to the `SEAM-1` or `SEAM-2` closeout subject surfaces that would invalidate the revalidation basis carried here
- **Remediation disposition**: none. No open remediation was created by SEAM-3, and no existing remediation was carried forward.
- **Promotion blockers**: none.
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none

## Validation record

- `bash -n docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`
- `cargo test -p shell test_wrap_mode_builtin_command_omits_command_body_in_trace -- --nocapture`
- `cargo test -p shell world_process_exec_tracing_parity_wpep1 -- --nocapture`
- `cargo test -p shell world_process_exec_tracing_parity_wpep2 -- --nocapture`
- `cargo test -p shell shell_env_script_mode_sets_preexec_flag_for_bash -- --nocapture`
- `cargo test -p shell --test repl_tty_disconnect_macos -- --nocapture --test-threads=1`
- `cargo test -p agent-api-types policy_snapshot_v3_resolve_world_network_routing_matches_four_case_matrix -- --nocapture`
- `cargo test -p substrate-replay world_net_filter_from_process_env_reads_exported_runtime_state -- --nocapture`
- `cargo test -p shell world_process_exec_tracing_parity_wpep3 -- --nocapture`
- The WPEP3 test is Linux-only (`#![cfg(all(unix, target_os = "linux"))]` in `crates/shell/tests/world_process_exec_tracing_parity_wpep3.rs`), so on this macOS host that surface is evidenced by the landed code/readback, not by local test execution.
