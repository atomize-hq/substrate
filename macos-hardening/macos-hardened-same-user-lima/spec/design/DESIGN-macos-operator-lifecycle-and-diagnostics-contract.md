# Design: macOS Operator Lifecycle and Diagnostics Contract

Status: draft design input  
Last updated: 2026-06-11

## Why this doc exists

The macOS hardening work is not only about backend/runtime behavior. It also
needs one shared view of which lifecycle, diagnosis, status, and evidence
surfaces constitute the supported operator contract.

The phase docs already say the repo is farther along than older narratives
admit. This design exists to freeze that direction before Phase 3 specs start
cutting over docs or consolidating owned flows.

## Relationship to the phase docs

This design composes with:

1. [`../../phase-3-substrate-owned-operations/README.md`](../../phase-3-substrate-owned-operations/README.md)
2. [`../../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md`](../../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md)
3. [`../../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md`](../../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md)
4. [`DESIGN-supported-mode-and-breakglass-taxonomy.md`](./DESIGN-supported-mode-and-breakglass-taxonomy.md)
5. [`DESIGN-macos-ingress-and-mount-contract.md`](./DESIGN-macos-ingress-and-mount-contract.md)

## Problem statement

How should the repo define the normal operator contract for macOS Lima so that:

1. already-landed Substrate-owned surfaces are treated as baseline,
2. helper scripts and direct guest commands stop defining the happy path,
3. future lifecycle or sync work stays inside the hardened same-user scope,
4. docs and validation can point to one coherent supported story?

## Frozen direction

This design freezes the following:

1. **Current baseline owned surfaces already matter**
   - `substrate host doctor`
   - `substrate world doctor`
   - `substrate world gateway sync|status|restart`
2. **Structured evidence should be first-class**
   - doctor JSON and gateway status JSON should be primary support surfaces
3. **Helper scripts are implementation assets or transitional wrappers**
   - `scripts/mac/lima-warm.sh` and `scripts/mac/lima-doctor.sh` may still play
     a role, but they should not define the final operator contract
4. **Direct guest administration is not the primary workflow**
5. **Ingress and sync must eventually become legible as owned operations**
   - especially after broad mounts are narrowed

## Non-goals

This design does not:

1. lock the exact final command surface for every future warm/repair action,
2. require that internal `limactl` use disappear immediately,
3. redefine the support taxonomy,
4. erase all breakglass documentation.

## Operator contract layers

Future specs should structure the operator story into these layers:

### 1. Supported primary surfaces

Expected to remain first-class:

1. `substrate host doctor --json`
2. `substrate world doctor --json`
3. `substrate world gateway status --json`
4. gateway lifecycle flows owned by Substrate

### 2. Transitional or consolidated wrappers

Possible near-term landing zone:

1. clearer owned create/warm/repair flows that may still route through existing
   helper logic internally
2. clearer owned sync/copy flows that replace reliance on broad mounts

### 3. Breakglass diagnostics and recovery

Expected to remain exceptional:

1. direct guest service manipulation
2. raw guest shelling
3. raw socket probing outside the owned doctor/gateway contract

## Evidence expectations

Future specs and docs should keep these surfaces central when relevant:

```bash
substrate host doctor --json
substrate world doctor --json
substrate world gateway status --json
scripts/mac/lima-doctor.sh
scripts/mac/smoke.sh
scripts/mac/orchestration-smoke.sh
```

The intent is not that every one of these is equally user-facing forever. The
intent is that supported behavior should be explainable and testable through the
owned contract rather than inferred from ad hoc guest commands.

## File and surface implications

This design should guide future work across:

1. `scripts/mac/lima-warm.sh`
2. `scripts/mac/lima-doctor.sh`
3. `scripts/mac/smoke.sh`
4. `scripts/mac/orchestration-smoke.sh`
5. `docs/WORLD.md`
6. `docs/reference/world/platforms/macos-lima-setup.md`
7. `docs/USAGE.md`
8. `docs/contracts/gateway/operator-contract.md`

## Questions future specs should answer

1. Which current lifecycle actions need a clearer owned command or wrapper?
2. Which current helper-script flows are good enough to remain internal
   implementation details?
3. Which docs still teach breakglass as the normal path?
