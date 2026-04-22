---
seam_id: SEAM-2
status: exec-ready
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-2-runtime-realization-and-artifacts/slice-99-seam-exit-gate.md
  status: blocked
  promotion_readiness: blocked
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
    landing: blocked
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Runtime realization and artifacts

This closeout records the current blocked `SEAM-2` exit state on the present tree.
The repo now has partial `S1` and `S3` realization evidence, but it does not yet publish
`THR-02` because the integrated runtime path still treats `cli:codex` as the only bound and
authenticated integrated backend.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-2-runtime-realization-and-artifacts/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - canonical contract truth remains the primary baseline for `C-03` and `C-04`:
    - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
    - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
  - current runtime and shell evidence on the landed tree:
    - `crates/world-agent/src/service.rs` now prepares lifecycle requests from the fixed selected backend id, rejects mismatched request-provided auth payloads, and returns no runtime binding when the selected backend has no bound runtime implementation.
    - `crates/world-agent/src/gateway_runtime.rs` now enforces pre-spawn capability checks, persists runtime manifests, and carries explicit readiness/restart state for the current bound runtime path.
    - `crates/world-agent/tests/gateway_runtime_parity.rs` covers missing-binding unavailable posture plus sync/status/restart, manifest recovery, timeout cleanup, and transient lifecycle behavior for the current `cli:codex` runtime path.
    - `crates/shell/tests/world_gateway.rs` covers shell-visible invalid-integration, transient, and policy failure mapping plus the current integrated auth handoff rules consumed from the canonical policy contract.
  - validation commands reviewed against the current tree:
    - `cargo test -p world-agent --test gateway_runtime_parity -- --nocapture`
    - `cargo test -p shell --test world_gateway -- --nocapture`
  - subordinate planning-pack prose may support implementation context when present, but canonical contract truth remains the authoritative evidence baseline
- **Contracts published or changed**:
  - none newly published; `SEAM-2` still depends on the existing canonical protocol/schema contracts because the runtime realization is not complete enough to publish `THR-02`
- **Threads published / advanced**:
  - none; `THR-02` remains unpublished on the current tree
- **Review-surface delta**:
  - the current tree partially realizes the planned seam: selected-backend request preparation, missing-binding unavailable posture, capability gating, runtime artifacts, and lifecycle drift guards exist, but the realized runtime boundary still collapses adapter binding and integrated auth handling to `cli:codex`
- **Planned-vs-landed delta**:
  - `S1` landed only in part: selected-backend preparation and missing-binding unavailable behavior exist, but `crates/world-agent/src/gateway_runtime.rs` still resolves `resolve_gateway_backend_binding` only for `cli:codex`
  - `S2` landed only in part: runtime artifact and lifecycle surfaces exist for the current bound path, but `crates/agent-api-types/src/lib.rs` still defines `GatewayIntegratedAuthPayloadV1` as `backend_id` plus optional `cli_codex` only, `crates/shell/src/builtins/world_gateway.rs` still returns `None` for non-`cli:codex` integrated auth payloads, and `crates/world-agent/src/gateway_runtime.rs` still resolves auth handoff only through `GatewayIntegratedAuthKind::CliCodex`
  - `S3` landed only for the current Codex runtime path: lifecycle, restart, manifest recovery, and transient-state coverage exist, but they do not prove a generalized adapter-driven runtime for more than `cli:codex`
- **Downstream stale triggers raised**:
  - revalidate `SEAM-3` when adapter binding lookup expands beyond `cli:codex`
  - revalidate `SEAM-3` when the bounded integrated auth/request payload ceases to be `cli_codex`-only
  - revalidate `SEAM-3` when runtime artifact naming, permissions, inspectability, or restart/readiness semantics change for a second supported backend
- **Remediation disposition**:
  - `REM-003` remains unresolved on the current tree: the runtime now distinguishes selected-backend preparation from missing-binding unavailable behavior, but the binding table still exposes only `cli:codex`, so the protocol-owned adapter lookup and capability-gate realization is incomplete
  - `REM-004` remains unresolved on the current tree: the shared request/auth surface, shell request construction, and runtime auth resolution remain `cli_codex`-specific, so the schema-owned widening and backend-aware artifact handoff is incomplete
  - `governance/remediation-log.md` is not updated by this slice because those remediations are still accurately recorded there as deferred implementation follow-through rather than newly reclassified pack-governance entries
- **Promotion blockers**:
  - `crates/world-agent/src/gateway_runtime.rs` still binds only `cli:codex` in `resolve_gateway_backend_binding`, so `SEAM-2` has not yet realized adapter dispatch for more than one integrated backend
  - `crates/agent-api-types/src/lib.rs` still defines `GatewayIntegratedAuthPayloadV1` with only the `cli_codex` auth facet, so the bounded request/auth shape is not yet generalized
  - `crates/shell/src/builtins/world_gateway.rs` still suppresses integrated auth payload emission for any selected backend other than `cli:codex`, so the shell-to-runtime handoff remains backend-specific
  - `crates/world-agent/src/gateway_runtime.rs` still resolves integrated auth through `GatewayIntegratedAuthKind::CliCodex` only, so runtime auth validation remains single-backend
- **Promotion readiness**:
  - blocked; `SEAM-3` must not promote on the current tree because `THR-02` is not yet publishable as one authoritative runtime handoff

## Post-exec gate disposition

- **Landing gate**: blocked
- **Closeout gate**: passed
- **Unresolved remediations**:
  - `REM-003`
  - `REM-004`
- **Carried-forward remediations**:
  - `REM-003`
  - `REM-004`
