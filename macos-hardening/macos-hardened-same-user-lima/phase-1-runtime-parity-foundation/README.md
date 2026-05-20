# Phase 1: Runtime Parity Foundation

Status: Draft

Owner: Substrate world backend / macOS runtime parity

Last updated: 2026-05-19

## Purpose / outcome

Converge the current same-user Lima runtime with the already-landed Substrate shell and operator contract before deeper hardening work begins. The outcome of this phase is not "invent doctor and gateway commands for macOS"; it is "the backend-mediated Lima path matches the transport, policy, shared-world, and readiness story that the CLI already exposes."

Phase 1 is complete when the macOS backend no longer depends on transport drift, backend-local policy synthesis, or direct-guest operational shortcuts to appear healthy.

## Why this phase exists

The current macOS backend still diverges from Linux in a few foundational places, but the gap is narrower than older docs suggest:

- `substrate host doctor`, `substrate world doctor`, and `substrate world gateway sync|status|restart` already exist as canonical operator surfaces, and `scripts/mac/smoke.sh` already exercises gateway lifecycle on macOS.
- Shared-world/orchestration support already exists on the Lima-backed path, and `scripts/mac/orchestration-smoke.sh` already covers shared-owner bootstrap, ready-proof acceptance, replacement, lazy member launch, and mismatch rejection.
- The shell-side direct routed request builders already resolve and forward `policy_snapshot`, `world_network`, and `world_fs_mode` in:
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/shell/src/repl/async_repl.rs`
  - `crates/shell/src/builtins/world_gateway.rs`

The remaining phase-1 gaps are therefore specific and concrete:

- transport behavior is still split across `crates/world-mac-lima/src/lib.rs`, `crates/world-mac-lima/src/forwarding.rs`, and `crates/world-mac-lima/src/transport.rs`, including the current `17788` versus `7788` inconsistency.
- backend policy semantics are not actually applied on the backend-mediated Lima path because `MacLimaBackend::convert_exec_request` synthesizes a permissive snapshot and `MacLimaBackend::apply_policy` is a no-op.
- readiness and troubleshooting evidence still normalize direct guest entry in `scripts/mac/lima-warm.sh`, `scripts/mac/lima-doctor.sh`, `docs/WORLD.md`, and `docs/cross-platform/mac_world_setup.md` more than the supported Lima path should.
- `SUBSTRATE_WORLD_SOCKET` still bypasses Lima detection/startup and remains an advanced/test/breakglass override, not the standard macOS path. That matters because explicit shared-owner reuse already rejects under this bypass on macOS.

Until those layers align, later same-user hardening work would sit on top of a backend that still proves itself using bypass-heavy workflows and still diverges from the shell-owned policy/runtime contract.

## In scope

- Define one canonical macOS host-to-guest transport contract and remove internal constant drift.
- Make PTY, non-PTY, readiness probing, and doctor reporting consume that same transport contract.
- Remove backend-local policy synthesis from `MacLimaBackend` and make backend-mediated Lima execution consume the same resolved policy/world inputs the shell already computes.
- Keep shared-world/orchestration parity explicit on the Lima-backed path, including the fail-closed behavior around `SUBSTRATE_WORLD_SOCKET`.
- Make doctor/smoke/setup evidence prove the routed Substrate path first, with direct guest entry moved out of the normal operator path.

## Out of scope

- Changing the host ownership boundary of Lima or `LIMA_HOME`.
- Claiming Linux-equivalent ownership isolation on macOS.
- Removing all same-user hardening gaps such as broad host mounts, extra guest listeners, or direct control-plane ownership.
- Designing an ownership-separated macOS architecture.
- Rewriting provisioning to eliminate every internal `limactl` dependency.

## Architectural approach

Phase 1 treats `/run/substrate.sock` inside the guest as the only canonical service endpoint. VSock, SSH UDS, and any retained TCP surface are host-side transport adapters, not separate behavioral modes.

The shell/operator baseline already exists:

- `substrate host doctor`
- `substrate world doctor`
- `substrate world gateway sync`
- `substrate world gateway status`
- `substrate world gateway restart`

Phase 1 hardens around those surfaces instead of introducing new ones.

The policy-parity work is deliberately scoped. The shell already resolves and serializes the effective policy/world inputs for direct routed requests; the unresolved gap is that backend-mediated Lima execution still invents or drops those semantics inside `MacLimaBackend`. If `world_api::WorldSpec` or adjacent backend seams must be extended to carry the right inputs for parity, that is preferable to preserving backend-local synthesis.

Shared-world parity is also already partially present. Phase 1 does not add shared-owner support from scratch; it ensures the Lima-backed path remains the authoritative macOS path for that support, while `SUBSTRATE_WORLD_SOCKET` remains classified as an advanced/test/breakglass bypass.

## Dependencies / sequencing

This phase should be executed in the following sequence:

1. [Milestone 1.1: Transport Contract Unification](./milestone-1-1-transport-contract-unification-sow.md)
2. [Milestone 1.2: Policy Application Parity](./milestone-1-2-policy-application-parity-sow.md)
3. [Milestone 1.3: Doctor / Smoke Readiness Parity](./milestone-1-3-doctor-smoke-readiness-parity-sow.md)

Milestone 1.3 can prepare doc rewrites in parallel, but final readiness criteria should not be signed off until Milestones 1.1 and 1.2 land because the doctor/smoke evidence needs to prove the new transport and backend-policy behavior.

## Concrete repo surfaces and file pointers

Primary runtime/backend surfaces:

- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/src/forwarding.rs`
- `crates/world-mac-lima/src/transport.rs`
- `crates/world-api/src/lib.rs`
- `crates/shell/src/execution/policy_snapshot.rs`
- `crates/shell/src/execution/routing/world.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/src/builtins/world_gateway.rs`

Primary scripts and operator-doc surfaces:

- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `scripts/mac/orchestration-smoke.sh`
- `docs/WORLD.md`
- `docs/cross-platform/mac_world_setup.md`
- `docs/USAGE.md`

Research input anchoring this phase:

- `macos-hardening/research/2026-04-28-macos-lima-parity-lockdown.md`

## Deliverables

- A phase-local milestone set with explicit sequencing and acceptance gates.
- A canonical macOS transport contract covering backend, shell, doctor, and smoke consumers.
- A backend policy-application plan that removes synthetic permissive snapshots from `MacLimaBackend`.
- A readiness-evidence plan that keeps CLI doctors, gateway lifecycle, and smoke/orchestration flows authoritative while relegating direct guest entry to breakglass.

## Acceptance criteria

- There is one documented and implemented macOS transport contract for routed world traffic, and no remaining `17788` versus `7788` disagreement.
- Mac PTY and non-PTY requests consume the same transport-selection rules and endpoint semantics.
- `MacLimaBackend` no longer invents a permissive policy snapshot for backend-mediated execution.
- The phase docs explicitly acknowledge that shell-side direct routed request builders already carry resolved `policy_snapshot`, `world_network`, and `world_fs_mode`.
- macOS doctor/smoke readiness proves routed Substrate behavior before any direct guest fallback checks.
- The phase docs make explicit that same-user Lima still does not provide the Linux ownership boundary.

## Validation / evidence plan

Implementation work under this phase should attach evidence from:

- targeted Rust tests for `world-mac-lima`, `world-api`, and shell routing/doctor surfaces
- `cargo test -p world-mac-lima`
- relevant shell tests covering world routing, shared-world proof handling, gateway status, and doctor output
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `scripts/mac/orchestration-smoke.sh`
- `substrate host doctor --json`
- `substrate world doctor --json`
- `substrate world gateway status --json`

Evidence must show selected transport, routed agent reachability, gateway lifecycle visibility, shared-world proof handling, and backend-policy parity through the CLI path, not only through direct in-guest `curl`.

## Risks / open questions

- Extending `WorldSpec` or adjacent backend contracts may ripple into the Windows backend and shared factory code.
- The repo mixes backend abstraction and direct agent API request paths; parity work must avoid creating two competing policy sources of truth.
- Some direct guest checks may remain necessary during bootstrap failure, but those must be classified as breakglass diagnostics rather than normal setup.
- Phase 1 improves backend/runtime parity, not ownership isolation; future hardening phases must still address broad mounts, extra listeners, and same-user Lima control-plane ownership.
