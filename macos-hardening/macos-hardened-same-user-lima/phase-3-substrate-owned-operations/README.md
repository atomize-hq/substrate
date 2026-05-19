# Phase 3: Substrate-Owned Operations

## Status

Draft

## Purpose / outcome

Finish moving macOS Lima lifecycle, diagnostics, and operator guidance from direct guest administration toward explicit Substrate-owned commands and clearly bounded breakglass procedures. Phase 3 ends when the normal operator path no longer depends on raw `limactl shell substrate ...`, direct guest `systemctl` manipulation, or ad hoc socket probing as the primary workflow.

This phase is an operational cutover, not an invention phase. The canonical CLI surfaces already exist in HEAD:

- `substrate host doctor`
- `substrate world doctor`
- `substrate world gateway sync`
- `substrate world gateway status`
- `substrate world gateway restart`

## Why this phase exists

The current macOS backend works, but many repo surfaces still normalize direct guest operations even though the CLI-owned path already exists:

- `docs/cross-platform/mac_world_setup.md` still includes step-by-step `limactl shell substrate ...` build, install, service, and troubleshooting flows.
- `docs/WORLD.md` already names CLI doctor surfaces, shared-world support, and the `SUBSTRATE_WORLD_SOCKET` bypass, but it still preserves too much direct guest guidance for the hardened default.
- `scripts/mac/lima-doctor.sh` remains a guest-admin-oriented diagnostic script rather than a thin wrapper around a fully Substrate-owned support story.
- `scripts/mac/lima-warm.sh` still relies heavily on direct `limactl shell` mutation, which is functional but not the posture that operator docs should normalize.
- `scripts/mac/smoke.sh` and `scripts/mac/orchestration-smoke.sh` already prove important parts of the supported path, but the top-level docs still present too much of the exception path as routine setup.

That matters because a hardened same-user Lima deployment should make "what Substrate owns" legible. Phase 2 narrows the technical surface. Phase 3 narrows the operational surface.

## In-scope

- Define the normal macOS lifecycle and diagnostics actions that should be taught through Substrate commands first.
- Define the normal macOS ingress and sync actions that should be taught through Substrate commands after Phase 2 mount minimization.
- Reclassify direct guest administration as breakglass where it remains necessary.
- Reclassify `SUBSTRATE_WORLD_SOCKET` as an advanced/test/breakglass bypass rather than a standard Lima operator workflow.
- Cut documentation over so the primary operator story runs through Substrate-owned commands first.
- Align doctor, gateway, smoke, orchestration, and troubleshooting evidence around the new operational boundary.

## Out-of-scope

- Replacing Lima itself.
- Solving the underlying same-user ownership limitation.
- Inventing a native macOS world implementation.
- Broad CLI redesign outside the macOS world lifecycle and diagnostics surface needed here.

## Architectural approach

Phase 3 treats the Lima guest as an internal implementation detail for normal operations. The user-visible contract becomes:

1. Substrate-owned commands decide when the VM must exist, start, sync, diagnose, or repair.
2. Remaining fallback flows are classified using the phase-0 taxonomy:
   - supported
   - degraded-but-supported
   - breakglass/unsupported
3. Direct `limactl shell` access is preserved only for explicit breakglass or deep debugging.
4. `SUBSTRATE_WORLD_SOCKET` remains available for advanced testing and breakglass bypass use, but it is not documented as the standard Lima path and must stay outside the normal shared-owner/orchestration story.
5. Docs, scripts, and evidence surfaces teach the Substrate-owned path first and label manual guest administration as exception handling.

This phase does not require hiding Lima from advanced users. It requires reclassifying Lima administration from the default happy path to a controlled fallback.

## Dependencies / sequencing

- Phase 2 must land first because CLI-owned lifecycle commands should manage the hardened listener, mount, and unit contracts rather than today's wider posture.
- Milestone 3.1 comes before 3.2 because the docs cutover should follow a concrete owned command surface.
- The current doctor/gateway/shared-world surfaces are baseline inputs to this phase, not open design questions.

## Concrete repo surfaces and file pointers

- `scripts/mac/lima-warm.sh`
  - current lifecycle/provisioning authority
- `scripts/mac/lima-doctor.sh`
  - current diagnostic authority
- `scripts/mac/smoke.sh`
  - already validates doctor/gateway/runtime behavior on the Lima-backed path
- `scripts/mac/orchestration-smoke.sh`
  - already validates shared-owner/orchestration behavior on the Lima-backed path
- `docs/WORLD.md`
  - documents the runtime model, shared-world behavior, and `SUBSTRATE_WORLD_SOCKET` bypass
- `docs/cross-platform/mac_world_setup.md`
  - still teaches direct Lima administration too prominently
- `docs/USAGE.md`
  - already treats `substrate world gateway sync|status|restart` as operator entrypoints
- `docs/contracts/substrate-gateway-operator-contract.md`
  - already treats `substrate world gateway status --json` as the authoritative machine-readable wiring surface

## Deliverables

- One phase packet sequencing the operational cutover into milestones 3.1 and 3.2.
- A defined Substrate-owned lifecycle and diagnostics story for hardened same-user Lima.
- A defined Substrate-owned ingress and sync story that replaces any remaining dependence on broad convenience mounts for normal operation.
- A breakglass policy for remaining direct guest operations and bypasses.
- Documentation and validation aligned to the new operational boundary.

## Acceptance criteria

- The normal macOS world lifecycle and diagnostic story is driven by Substrate-owned commands, not by raw `limactl shell` recipes.
- The normal macOS workspace ingress and sync story is driven by Substrate-owned commands, not by implicit broad mounts or ad hoc guest steps.
- Direct guest operations that remain are explicitly labeled breakglass and bounded.
- `SUBSTRATE_WORLD_SOCKET` is documented as an advanced/test/breakglass bypass, not the standard Lima-backed path.
- Doctor, gateway, smoke, orchestration, and troubleshooting evidence can be captured through the owned path with manual guest access reserved for exceptions.
- A reviewer can distinguish implementation detail from operator contract in the macOS docs.

## Validation / evidence plan

- Inventory every current direct guest administration command in the macOS docs and scripts.
- Define which ones become owned commands, which ones remain internal implementation details, and which ones are breakglass only.
- Re-run doctor, gateway, and smoke/orchestration coverage using the owned command path after the cutover plan is implemented.
- Review doc text for any remaining places that still present direct Lima administration as the default happy path.

## Risks / open questions

- Some diagnostics may still require direct guest access until Substrate exposes enough structured detail.
- The repo may need transitional wrappers before the final CLI contract is fully simplified.
- "Breakglass" needs a precise meaning in docs: allowed for deep debugging, but not the default install or day-to-day maintenance path.

## Milestones

1. [milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md](./milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md)
2. [milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md](./milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md)
