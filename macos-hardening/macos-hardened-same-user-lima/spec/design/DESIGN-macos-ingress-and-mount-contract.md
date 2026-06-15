# Design: macOS Ingress and Mount Contract

Status: draft design input  
Last updated: 2026-06-11

## Why this doc exists

The phase docs already identify broad host visibility as one of the remaining
same-user hardening gaps, but a later `SPEC-*` should not begin by assuming the
current mount layout is the right default.

This design exists to freeze the direction for:

1. how guest-visible host inputs should be classified,
2. which classes may remain direct mounts,
3. which classes should move to Substrate-managed sync/copy flows,
4. which classes should be removed from the default posture.

## Relationship to the phase docs

This design composes with:

1. [`../../phase-2-same-user-hardening/README.md`](../../phase-2-same-user-hardening/README.md)
2. [`../../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md`](../../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md)
3. [`DESIGN-supported-mode-and-breakglass-taxonomy.md`](./DESIGN-supported-mode-and-breakglass-taxonomy.md)
4. [`DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)

## Problem statement

How should the repo define the hardened guest ingress posture so that:

1. the default contract is narrower than a broad read-only `$HOME` mount,
2. workspace, auth, runtime, and troubleshooting inputs are treated as
   different classes,
3. future operator flows can replace convenience mounts with owned sync/copy
   flows where needed?

## Frozen direction

This design freezes the following:

1. **Ingress must be classified by need, not by convenience**
2. **Broad host-home visibility is not the hardened default**
3. **The ingress decision space has three primitives**
   - direct mount
   - Substrate-managed copy/sync
   - no default ingress
4. **Gateway lifecycle/runtime support must survive hardening**
   - supported guest runtime artifacts under
     `/run/substrate/substrate-gateway-runtime/` remain part of the supported
     contract
5. **Auth should prefer narrow handoff over ambient home visibility**
6. **Breakglass ingress remains possible but must not define the normal
   contract**

## Non-goals

This design does not:

1. define the final sync UX in detail,
2. require that every current mount disappear immediately,
3. settle every host credential flow yet,
4. redesign the operator taxonomy.

## Ingress classes

Future specs should classify guest-visible inputs into at least these classes:

### 1. Workspace source input

Needed for builds, smoke runs, and execution against project state.

Decision options:

1. narrow direct mount where justified
2. Substrate-managed sync/copy if direct mount is too broad

### 2. Auth and credential input

Needed for specific gateway or package flows.

Preferred direction:

1. narrow request-provided or managed handoff
2. avoid default broad host-home exposure

### 3. Runtime artifacts

Needed for supported guest operation.

Expected to remain supported:

1. `/run/substrate.sock`
2. `/run/substrate/substrate-gateway-runtime/`

### 4. Troubleshooting or operator convenience input

Direction:

1. classify as breakglass unless it becomes a supported owned flow

## File and surface implications

This design should guide future work across:

1. `scripts/mac/lima/substrate.yaml`
2. `scripts/mac/lima-warm.sh`
3. `docs/reference/world/platforms/macos-lima-setup.md`
4. `docs/WORLD.md`
5. any future Substrate-managed sync/copy surface introduced for macOS hardening

## Questions future specs should answer

1. Which exact paths are currently mounted only for convenience?
2. Which paths are truly required for supported runtime operation?
3. Which current workflows break if `$HOME` visibility is narrowed?
4. Which of those should become owned sync/copy flows instead of retained
   mounts?
