# Milestone 3.1: Substrate-Managed Diagnostics and Lifecycle

## Status

Partial

Last updated: 2026-06-11

## Purpose / outcome

Consolidate and productize the macOS/Lima lifecycle and diagnostic actions
that Substrate already partly owns so operators can use one legible contract
for create, warm, check, repair, and inspection without treating raw guest
administration commands as the primary path.

## Why this milestone exists

The current backend is operationally split between already-landed Substrate
surfaces and still-normalized helper-script or guest commands. In repo truth,
doctor and gateway status ownership are further along than the lifecycle
cutover, while create/warm/repair remains substantially owned by
`scripts/mac/lima-warm.sh`.

- `substrate host doctor` and `substrate world doctor` already exist as
  canonical diagnostics.
- `substrate world gateway sync|status|restart` and status JSON already exist as
  canonical gateway lifecycle/status surfaces.
- `scripts/mac/smoke.sh` already exercises gateway lifecycle smoke coverage.
- `scripts/mac/lima-doctor.sh` and direct `limactl shell` probes are still
  treated too often as normal operator paths even though the doctor surfaces
  themselves already exist.
- `scripts/mac/lima-warm.sh` still owns most create/warm/repair automation
  through direct guest mutation rather than through a clearly consolidated and
  productized Substrate-managed operator contract.
- `docs/reference/world/platforms/macos-lima-setup.md` still teaches manual
  build/install/service flows inside the guest.

This milestone exists to define and consolidate what “Substrate-owned
operations” means in concrete command terms before the docs are cut over. The
gap is not absence of automation; the gap is split ownership and an incomplete
operator contract.

## In-scope

- Identify the remaining macOS lifecycle/diagnostic action set that still needs clearer Substrate ownership or contract consolidation.
- Replace, wrap, or explicitly productize the current Lima-specific flows with Substrate-owned commands where feasible.
- Make doctor output the primary structured evidence surface for macOS backend health.
- Make gateway lifecycle/status a primary structured evidence and support
  surface alongside doctor JSON.
- Ensure the operator contract clearly covers warm/create, service
  reachability, guest repair, gateway runtime state, and state reporting at a
  level operators can trust, even where implementation still routes through
  `scripts/mac/lima-warm.sh` today.
- Define the Substrate-owned sync or copy path that replaces any remaining
  normal-path dependence on broad host mounts after Phase 2 ingress
  minimization.

## Out-of-scope

- Removing all internal uses of `limactl shell` from implementation code if the backend still needs it under the hood.
- Documentation-only cleanup without an actual owned command surface.
- Pretending create/warm/repair automation does not already exist in `scripts/mac/lima-warm.sh`.
- Replacing smoke coverage or transport internals unrelated to operator ownership.
- Closing the remaining MacLimaBackend policy-parity gaps or the same-user host ownership limitation.

## Architectural approach

- Treat the current helper scripts as implementation assets and transitional operator surfaces, not as the final operator contract.
- Promote the already-landed Substrate commands as the entry points for:
  - backend readiness
  - gateway lifecycle and status
  - workspace ingress or sync needed for normal backend operation
  - health diagnosis
  - service/socket verification
- Stabilize a clearer operator contract for controlled create/warm/repair or
  refresh flows that still route substantially through `scripts/mac/lima-warm.sh`.
- Keep any required Lima shelling internal to those commands or clearly classified as breakglass, rather than letting the current helper implementation details define the happy path.
- Make structured doctor output the main evidence channel so operators do not
  need to infer health from scattered shell commands.
- Treat managed gateway runtime artifacts under
  `/run/substrate/substrate-gateway-runtime/` as part of the supported
  operator-facing lifecycle surface.
- Preserve the phase-0 support taxonomy:
  - supported
  - degraded-but-supported
  - breakglass

This milestone should also preserve already-supported shared-world/orchestration
behavior rather than hard-coding an owned-operations story that only works for
single-session happy paths.

## Dependencies / sequencing

- Depends on Phase 2 hardening decisions, especially the listener and unit contracts.
- Must land before milestone 3.2 because doc reclassification should reference the owned commands that replace today’s manual paths.

## Concrete repo surfaces and file pointers

- `crates/shell/src/execution/platform/macos.rs`
  - primary CLI surface for macOS doctor, readiness evidence, transport
    checks, and runtime-status reporting; not the primary lifecycle owner
- `crates/shell/src/builtins/world_gateway.rs`
  - already-owned gateway lifecycle/status surface that Phase 3 should elevate,
    not reinvent
- `docs/contracts/gateway/operator-contract.md`
  - canonical operator contract for the already-landed gateway lifecycle family
- `docs/contracts/gateway/status-schema.md`
  - authoritative machine-readable status contract
- `crates/shell/src/builtins/world_enable`
  - candidate future owner for provisioning and enablement flows if more of
    the current `scripts/mac/lima-warm.sh` responsibility gets productized into
    the CLI
- `crates/shell/src/execution/workspace_cmd.rs`
  - candidate future owner for any Substrate-managed workspace ingress or sync
    command surface that replaces broad convenience mounts
- `crates/world-mac-lima/src/lib.rs`
  - backend surface that must honor the owned command contract once lifecycle
    and sync behavior are driven through the CLI
- `crates/world-service/src/gateway_runtime.rs`
  - authoritative managed runtime state and artifact model under
    `/run/substrate/substrate-gateway-runtime/`
- `scripts/mac/lima-warm.sh`
  - current owner of create/start/repair behavior, including guest unit wiring
    that still injects `SUBSTRATE_AGENT_TCP_PORT=61337`
- `scripts/mac/lima-doctor.sh`
  - current owner of host and guest health checks
- `scripts/mac/smoke.sh`
  - can verify the managed lifecycle path after the contract is defined
- `docs/WORLD.md`
  - already names `substrate host doctor` and `substrate world doctor`, but still mixes them with direct guest commands
- `docs/reference/world/platforms/macos-lima-setup.md`
  - primary doc surface that will need cutover once the owned command set is frozen

## Deliverables

- A concrete owned-command matrix for macOS lifecycle and diagnostics that distinguishes already-landed owned surfaces from still-script-owned flows.
- A concrete owned-command matrix for gateway lifecycle/status, including
  `substrate world gateway sync|status|restart`.
- A concrete owned-command matrix for macOS workspace ingress or sync needed by
  the hardened same-user backend.
- A migration and productization plan from helper-script and raw guest workflows to Substrate-owned entry points.
- Updated evidence expectations that treat doctor JSON and owned lifecycle commands as the canonical verification path.
- Identification of any remaining operations that cannot yet be owned and must stay breakglass.

## Acceptance criteria

- The macOS backend has a defined primary Substrate-owned path for readiness, health, and repair operations, with any still-script-owned create/warm/repair steps called out explicitly.
- The macOS backend has a defined Substrate-owned path for managed gateway
  lifecycle and status operations.
- The macOS backend has a defined Substrate-owned path for any required normal
  workspace ingress or sync after Phase 2 mount minimization.
- Operators can gather the primary health evidence without running raw `limactl shell substrate ...` commands.
- Any degraded-but-supported helper path is explicitly identified as such and
  is distinct from breakglass flows.
- A reviewer can list the normal macOS operational commands from one doc section without cross-referencing guest-admin recipes.
- Host-side `SUBSTRATE_WORLD_SOCKET` override use is not presented as the
  default supported Lima path.

## Validation / evidence plan

- Build a command inventory mapping current helper and guest-admin actions to either their already-landed owned surfaces or their planned owned replacements.
- Build an ingress inventory mapping any remaining Phase 2 copy/sync needs to
  owned commands rather than implicit mounts.
- Capture `substrate host doctor --json` and `substrate world doctor --json` as the primary health artifacts.
- Capture `substrate world gateway sync`, `substrate world gateway status --json`,
  and `substrate world gateway restart` as primary lifecycle artifacts.
- Run `scripts/mac/smoke.sh` or successor smoke coverage through the owned lifecycle path to prove the contract is usable end to end.
- Record the set of residual manual actions that still lack owned coverage and carry them forward into milestone 3.2 as explicit breakglass exceptions.

## Risks / open questions

- Some repair actions may still need raw Lima capabilities internally even if the user-facing contract is owned by Substrate.
- Create/warm/repair may remain substantially routed through `scripts/mac/lima-warm.sh` for a while; the near-term requirement is an honest operator contract, not a fictional fully CLI-native lifecycle.
- If the existing CLI does not yet have the right verbs, there may be a transitional period where helper scripts remain user-facing but must be clearly scoped.
- Diagnostics can only be reclassified if the doctor payload exposes enough structure to replace direct guest shell inspection in common cases.
