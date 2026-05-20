# Milestone 3.1: Substrate-Managed Diagnostics and Lifecycle

## Status

Draft

Last updated: 2026-05-19

## Purpose / outcome

Define and implement the macOS/Lima lifecycle and diagnostic actions that
Substrate must own directly so operators can create, warm, check, repair, and
inspect the backend without relying on raw guest administration commands.

## Why this milestone exists

The current backend is operationally split between already-landed Substrate
surfaces and still-normalized helper or guest commands.

- `substrate host doctor` and `substrate world doctor` already exist as
  canonical diagnostics.
- `substrate world gateway sync|status|restart` and status JSON already exist as
  canonical gateway lifecycle/status surfaces.
- `scripts/mac/smoke.sh` already exercises gateway lifecycle smoke coverage.
- `scripts/mac/lima-doctor.sh` and direct `limactl shell` probes are still
  treated too often as normal operator paths.
- `scripts/mac/lima-warm.sh` performs lifecycle repair through direct guest
  mutation rather than through a clearly productized Substrate-managed
  contract.
- `docs/cross-platform/mac_world_setup.md` still teaches manual
  build/install/service flows inside the guest.

This milestone exists to define what “Substrate-owned operations” means in concrete command terms before the docs are cut over.

## In-scope

- Identify the full macOS lifecycle/diagnostic action set that should be owned by Substrate.
- Replace or wrap the current Lima-specific flows with Substrate-owned commands where feasible.
- Make doctor output the primary structured evidence surface for macOS backend health.
- Make gateway lifecycle/status a primary structured evidence and support
  surface alongside doctor JSON.
- Ensure lifecycle ownership covers warm/create, service reachability, guest
  repair, gateway runtime state, and state reporting at a level operators can
  trust.
- Define the Substrate-owned sync or copy path that replaces any remaining
  normal-path dependence on broad host mounts after Phase 2 ingress
  minimization.

## Out-of-scope

- Removing all internal uses of `limactl shell` from implementation code if the backend still needs it under the hood.
- Documentation-only cleanup without an actual owned command surface.
- Replacing smoke coverage or transport internals unrelated to operator ownership.

## Architectural approach

- Treat the current helper scripts as implementation staging points, not as the final operator contract.
- Promote Substrate commands as the entry points for:
  - backend readiness
  - gateway lifecycle and status
  - workspace ingress or sync needed for normal backend operation
  - health diagnosis
  - service/socket verification
  - controlled repair or refresh
- Keep any required Lima shelling internal to those commands or clearly classified as breakglass.
- Make structured doctor output the main evidence channel so operators do not
  need to infer health from scattered shell commands.
- Treat managed gateway runtime artifacts under
  `/run/substrate/substrate-gateway-runtime/` as part of the supported
  operator-facing lifecycle surface.
- Preserve the phase-0 support taxonomy:
  - supported
  - degraded-but-supported
  - breakglass/unsupported

This milestone should also preserve already-supported shared-world/orchestration
behavior rather than hard-coding an owned-operations story that only works for
single-session happy paths.

## Dependencies / sequencing

- Depends on Phase 2 hardening decisions, especially the listener and unit contracts.
- Must land before milestone 3.2 because doc reclassification should reference the owned commands that replace today’s manual paths.

## Concrete repo surfaces and file pointers

- `crates/shell/src/execution/platform/macos.rs`
  - primary CLI/runtime surface for owned macOS doctor, lifecycle, and status
    commands
- `crates/shell/src/builtins/world_gateway.rs`
  - already-owned gateway lifecycle/status surface that Phase 3 should elevate,
    not reinvent
- `docs/contracts/substrate-gateway-operator-contract.md`
  - canonical operator contract for the already-landed gateway lifecycle family
- `docs/contracts/substrate-gateway-status-schema.md`
  - authoritative machine-readable status contract
- `crates/shell/src/builtins/world_enable`
  - likely owner for provisioning and enablement flows that should stop
    delegating operator responsibility to raw Lima commands
- `crates/shell/src/execution/workspace_cmd.rs`
  - likely owner for any Substrate-managed workspace ingress or sync command
    surface that replaces broad convenience mounts
- `crates/world-mac-lima/src/lib.rs`
  - backend surface that must honor the owned command contract once lifecycle
    and sync behavior are driven through the CLI
- `crates/world-agent/src/gateway_runtime.rs`
  - authoritative managed runtime state and artifact model under
    `/run/substrate/substrate-gateway-runtime/`
- `scripts/mac/lima-warm.sh`
  - current owner of create/start/repair behavior
- `scripts/mac/lima-doctor.sh`
  - current owner of host and guest health checks
- `scripts/mac/smoke.sh`
  - can verify the managed lifecycle path after the contract is defined
- `docs/WORLD.md`
  - already names `substrate host doctor` and `substrate world doctor`, but still mixes them with direct guest commands
- `docs/cross-platform/mac_world_setup.md`
  - primary doc surface that will need cutover once the owned command set is frozen

## Deliverables

- A concrete owned-command matrix for macOS lifecycle and diagnostics.
- A concrete owned-command matrix for gateway lifecycle/status, including
  `substrate world gateway sync|status|restart`.
- A concrete owned-command matrix for macOS workspace ingress or sync needed by
  the hardened same-user backend.
- A migration plan from helper-script and raw guest workflows to Substrate-owned entry points.
- Updated evidence expectations that treat doctor JSON and owned lifecycle commands as the canonical verification path.
- Identification of any remaining operations that cannot yet be owned and must stay breakglass.

## Acceptance criteria

- The macOS backend has a defined Substrate-owned path for readiness, health, and repair operations.
- The macOS backend has a defined Substrate-owned path for managed gateway
  lifecycle and status operations.
- The macOS backend has a defined Substrate-owned path for any required normal
  workspace ingress or sync after Phase 2 mount minimization.
- Operators can gather the primary health evidence without running raw `limactl shell substrate ...` commands.
- Any degraded-but-supported helper path is explicitly identified as such and
  is distinct from breakglass/unsupported flows.
- A reviewer can list the normal macOS operational commands from one doc section without cross-referencing guest-admin recipes.
- Host-side `SUBSTRATE_WORLD_SOCKET` override use is not presented as the
  default supported Lima path.

## Validation / evidence plan

- Build a command inventory mapping current helper and guest-admin actions to their owned replacements.
- Build an ingress inventory mapping any remaining Phase 2 copy/sync needs to
  owned commands rather than implicit mounts.
- Capture `substrate host doctor --json` and `substrate world doctor --json` as the primary health artifacts.
- Capture `substrate world gateway sync`, `substrate world gateway status --json`,
  and `substrate world gateway restart` as primary lifecycle artifacts.
- Run `scripts/mac/smoke.sh` or successor smoke coverage through the owned lifecycle path to prove the contract is usable end to end.
- Record the set of residual manual actions that still lack owned coverage and carry them forward into milestone 3.2 as explicit breakglass exceptions.

## Risks / open questions

- Some repair actions may still need raw Lima capabilities internally even if the user-facing contract is owned by Substrate.
- If the existing CLI does not yet have the right verbs, there may be a transitional period where helper scripts remain user-facing but must be clearly scoped.
- Diagnostics can only be reclassified if the doctor payload exposes enough structure to replace direct guest shell inspection in common cases.
