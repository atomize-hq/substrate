---
seam_id: SEAM-2
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-2-runtime-realization-and-artifacts/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - revalidate downstream seams if runtime binding behavior, integrated auth/request payload shape, or runtime artifact semantics change
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Runtime realization and artifacts

This closeout records the landed post-exec state for `SEAM-2`.
The seam now publishes `THR-02` because the integrated runtime handoff is adapter-driven for
`cli:codex` and the first real non-`cli:codex` proof target, `api:openai`, while keeping
unsupported and unbound backends explicit with no fallback.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-2-runtime-realization-and-artifacts/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - canonical contract truth remains the primary baseline for `C-03` and `C-04`:
    - `docs/contracts/gateway/backend-adapter-protocol.md`
    - `docs/contracts/gateway/backend-adapter-schema.md`
  - landed runtime and shell evidence on the current tree:
    - `crates/transport-api-types/src/lib.rs` now defines a closed backend-neutral `api_env` auth facet beside `cli_codex`, hardens `GatewayLifecycleRequestV1` with `deny_unknown_fields`, and validates backend/facet coherence before runtime execution.
    - `crates/world-service/src/service.rs` now uses the shared request/auth validator, preserves selected-backend continuity, and exposes the Linux-only runtime inspection helpers that the parity suite exercises as a real integration test.
    - `crates/world-service/src/gateway_runtime.rs` now resolves an explicit runtime registry for `cli:codex` and `api:openai`, renders binding-driven runtime config, injects binding-specific auth env, and keeps unsupported/unbound backends explicit with no fallback.
    - `crates/shell/src/builtins/world_gateway.rs` now carries the resolved inventory entry into request construction and emits backend-aware integrated auth without reopening selection or auth-precedence ownership.
    - `crates/world-service/tests/gateway_runtime_parity.rs` now proves `api:openai` through unavailable-before-sync, sync, status, idempotent sync, restart, manifest recovery, and explicit no-fallback behavior while preserving the existing `cli:codex` regression floor.
    - `crates/shell/tests/world_gateway.rs` now proves bounded `api_env` emission for `api:openai`, preserves the no-Codex-fallback negative case when API env auth is absent, and keeps policy and invalid-integration failure buckets explicit.
  - validation commands that passed on the landed state:
    - `cargo fmt --all`
    - `cargo test -p transport-api-types -- --nocapture`
    - `cargo test -p shell --test world_gateway -- --nocapture`
    - `cargo test -p world-service --lib -- --nocapture`
    - `limactl shell substrate -- bash -lc 'cd /Users/spensermcconnell/__Active_Code/atomize-hq/substrate && CARGO_TARGET_DIR=/tmp/substrate-target cargo test -p world-service --test gateway_runtime_parity -- --nocapture'`
  - subordinate planning-pack prose may support implementation context when present, but canonical contract truth remains the authoritative evidence baseline
- **Contracts consumed or narrowly aligned**:
  - expected: `C-03`, `C-04`
- **Threads published / advanced**:
  - `THR-02`
- **Review-surface delta**:
  - the landed runtime handoff is now truly multi-backend at the bounded runtime layer: `backend_id` stays an adapter selector only, `api_env` is the backend-neutral auth extension, and the first supported non-`cli:codex` path is `api:openai`
- **Planned-vs-landed delta**:
  - the landed request/auth widening chose a backend-neutral `api_env` facet rather than an `api_openai` one-off because the real inventory and gateway provider config already model API auth generically
  - no `status --json` widening, tuple-surface widening, or auth-precedence ownership change was required to land the proof target
- **Downstream stale triggers raised**:
  - revalidate `SEAM-3` if the first proof target changes away from `api:openai`
  - revalidate `SEAM-3` if binding lookup expands again or if runtime artifact naming, permissions, or restart/readiness semantics change for additional supported backends
  - revalidate `SEAM-3` if the closed `api_env` request/auth shape changes or if a later backend requires a different bounded auth facet
- **Remediation disposition**:
  - `REM-003` resolved
  - `REM-004` resolved
- **Promotion blockers**:
  - none
- **Promotion readiness**:
  - ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**:
  - none
- **Carried-forward remediations**:
  - none
