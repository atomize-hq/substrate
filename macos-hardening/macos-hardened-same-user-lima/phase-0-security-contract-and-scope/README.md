# Phase 0: Security Contract and Scope

Status: Draft  
Last updated: 2026-05-19

## Purpose / outcome

Freeze the security and support contract for hardened same-user Lima before later phases change backend behavior, provisioning, or operator docs. The outcome of phase 0 is one unambiguous planning baseline for what macOS should guarantee, what it cannot guarantee, which already-landed CLI surfaces are the supported baseline, and which existing direct guest workflows are breakglass only.

## Why this phase exists

The current macOS backend is already close enough to Linux behavior that implementation changes are tempting, but the repo still lacks a locked contract for several disagreements:

- The repo already has canonical operator surfaces in `substrate host doctor`, `substrate world doctor`, and `substrate world gateway sync|status|restart`, but the hardening docs still talk as if those surfaces are mostly future work.
- The code still claims Linux-like policy semantics while `MacLimaBackend` synthesizes a permissive policy snapshot in `crates/world-mac-lima/src/lib.rs`.
- The shell-side routed request builders already forward resolved `policy_snapshot`, `world_network`, and `world_fs_mode`, so the unresolved policy gap must be scoped to backend-mediated Lima behavior and bootstrap/ensure-session paths.
- The Lima profile and setup docs still normalize broad host mounts and manual guest administration.
- The repo already supports shared-world/orchestration flows on the Lima-backed path, while `SUBSTRATE_WORLD_SOCKET` remains an advanced/test/breakglass bypass rather than the normal same-user operator path.

If phase 0 is skipped, later work risks hardening one layer while another layer keeps the older, looser support story.

## In-scope

- Define the target security posture for same-user Lima.
- Define the support boundary versus Linux parity claims.
- Define the baseline CLI/operator surfaces that are already supported.
- Define breakglass operator rules, including the status of `SUBSTRATE_WORLD_SOCKET` and direct `limactl shell` procedures.
- Define the versioning and environment assumptions that later implementation can rely on.
- Produce milestone SOWs that later code phases can execute against.

## Out-of-scope

- Editing runtime code, provisioning scripts, or top-level docs outside this phase directory.
- Closing the current transport, policy, mount, or docs gaps in this phase.
- Replacing Lima, redesigning the entire macOS backend, or solving multi-user host isolation.

## Architectural approach

Phase 0 uses two planning milestones:

1. `milestone-0-1-target-mode-and-support-contract-sow.md`
   - names the supported mode
   - names the non-goals
   - freezes the Linux-versus-macOS claims we will and will not make
   - treats existing doctor/gateway/shared-world surfaces as baseline
2. `milestone-0-2-lima-version-and-breakglass-contract-sow.md`
   - defines required Lima/runtime assumptions
   - defines which direct guest workflows remain available only for breakglass
   - defines how `SUBSTRATE_WORLD_SOCKET` is classified
   - defines which Substrate-owned workflows must remain the normal operator path

This phase is complete when later implementation can treat the phase docs as authoritative inputs instead of rediscovering support posture from scattered code comments and setup guides.

## Dependencies / sequencing

- Depends on the current macOS state captured in:
  - `crates/world-mac-lima/src/lib.rs`
  - `crates/world-mac-lima/src/forwarding.rs`
  - `crates/world-mac-lima/src/transport.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/shell/src/repl/async_repl.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/lima/substrate.yaml`
  - `docs/WORLD.md`
  - `docs/cross-platform/mac_world_setup.md`
  - `scripts/mac/lima-doctor.sh`
  - `scripts/mac/smoke.sh`
  - `scripts/mac/orchestration-smoke.sh`
- Milestone 0.1 must land conceptually before milestone 0.2, because the version and breakglass contract depend on the chosen target mode.
- No later hardening phase should describe doctor/gateway/shared-world support as future-only work after phase 0 is accepted.

## Concrete repo surfaces and file pointers

- Policy snapshot injection and backend policy no-op: `crates/world-mac-lima/src/lib.rs`
- Forwarding endpoint and transport fallback behavior: `crates/world-mac-lima/src/forwarding.rs`, `crates/world-mac-lima/src/transport.rs`
- Shell-side routed request/world input propagation: `crates/shell/src/execution/routing/dispatch/world_ops.rs`, `crates/shell/src/repl/async_repl.rs`, `crates/shell/src/builtins/world_gateway.rs`
- Lima guest image, mounts, and unit definitions: `scripts/mac/lima/substrate.yaml`
- Provisioning and lifecycle commands: `scripts/mac/lima-warm.sh`
- Troubleshooting and smoke entry points: `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`, `scripts/mac/orchestration-smoke.sh`
- Operator-facing macOS setup and world architecture docs: `docs/cross-platform/mac_world_setup.md`, `docs/WORLD.md`, `docs/USAGE.md`

## Deliverables

- Phase overview README.
- Milestone 0.1 SOW.
- Milestone 0.2 SOW.
- A phase-local issue ledger, embedded in those SOWs, that later implementation phases can use as their scope baseline.

## Acceptance criteria

- The phase documents define one supported same-user Lima posture and reject vague "Linux parity except where different" language.
- The phase documents treat `substrate host doctor`, `substrate world doctor`, and `substrate world gateway sync|status|restart` as already-landed operator surfaces.
- The phase documents distinguish supported flows from breakglass flows, including `SUBSTRATE_WORLD_SOCKET` and direct guest administration.
- The phase documents scope unresolved policy parity claims to `MacLimaBackend` and backend-mediated Lima paths.
- The milestone ordering is clear enough that a future implementation owner can start with milestone 0.1 decisions and then execute 0.2 without reopening scope.

## Validation / evidence plan

- Review the phase docs against the cited repo surfaces and confirm each claimed gap is observable today.
- Require sign-off from backend, security, and docs owners before phase completion.
- Use current CLI/smoke/orchestration surfaces as the baseline evidence set:
  - `substrate host doctor --json`
  - `substrate world doctor --json`
  - `substrate world gateway status --json`
  - `scripts/mac/smoke.sh`
  - `scripts/mac/orchestration-smoke.sh`

## Risks / open questions

- Some breakglass workflows are still embedded in normal setup docs, so reclassification will create short-term friction for operators.
- The right Lima version floor may force changes in forwarding assumptions, especially around `vsock-proxy` availability and SSH config stability.
- The project may need an ADR later if phase-0 decisions materially narrow the macOS support matrix.
