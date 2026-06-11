# Design: Supported Mode and Breakglass Taxonomy

Status: draft design input  
Last updated: 2026-06-11

## Why this doc exists

The current `macos-hardened-same-user-lima` phase docs already establish the
high-level hardening direction, but the feature still needs one cross-cutting
definition of:

1. the supported same-user Lima mode,
2. what Linux parity claims macOS may still make,
3. what remains explicitly non-parity,
4. how to classify supported, degraded-but-supported, and breakglass flows.

Without a single taxonomy doc, later transport, policy, ingress, and docs
slices will keep rephrasing the support boundary independently.

## Relationship to the phase docs

This design composes with:

1. [`../../phase-0-security-contract-and-scope/README.md`](../../phase-0-security-contract-and-scope/README.md)
2. [`../../phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`](../../phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md)
3. [`../../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md`](../../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md)
4. [`../README.md`](../README.md)
5. [`../../ROADMAP.md`](../../ROADMAP.md)

This document does not replace those phase docs. It freezes shared terminology
and classification rules that future `SPEC-*` docs should inherit.

## Problem statement

How should the repo describe the hardened same-user Lima posture so that:

1. the supported operator path is explicit,
2. the same-user limitation remains impossible to miss,
3. direct guest and bypass workflows are classified consistently,
4. later implementation slices do not widen support claims accidentally?

## Frozen direction

This design freezes the following:

1. **Supported mode**
   - one macOS host user owns the Substrate process and Lima lifecycle
   - the world executes inside the Lima guest
   - Substrate-owned commands are the supported operator control plane
2. **Linux parity claims kept**
   - in-world execution remains the default
   - guest-local service/socket behavior should converge toward Linux semantics
   - routed CLI/operator surfaces remain first-class support surfaces
   - shared-world/orchestration support on the Lima-backed path is part of the
     supported runtime story
3. **Linux parity claims not made**
   - same-user Lima is not a host privilege boundary against the owning user
   - host-side ownership is not equivalent to Linux `root:substrate 0660`
   - direct guest administration is not part of the normal operator path
4. **Support taxonomy**
   - `supported`
   - `degraded-but-supported`
   - `breakglass`

## Non-goals

This design does not:

1. pick the final Lima version floor,
2. define every transport detail,
3. define the final command list for future lifecycle consolidation,
4. remove all breakglass procedures,
5. claim a future ownership-separated macOS model.

## Support taxonomy

### 1. Supported

Definition:

1. the workflow runs through Substrate-owned commands or clearly owned scripts,
2. the workflow is documented first,
3. the workflow is expected to work for ordinary setup, health checks, and
   normal operation,
4. failures should be diagnosable through structured Substrate evidence.

Examples expected to remain in this category:

1. `substrate host doctor`
2. `substrate world doctor`
3. `substrate world gateway sync|status|restart`
4. routed Lima-backed world execution and orchestration flows

### 2. Degraded-but-supported

Definition:

1. the workflow is still part of the supported story,
2. but it exists with narrower guarantees, transitional caveats, or known UX
   debt,
3. and it should normally be converging toward a more owned surface.

Examples that may temporarily sit here:

1. transitional wrappers that still route substantially through
   `scripts/mac/lima-warm.sh`
2. temporary sync/copy flows introduced while broad mounts are being removed

### 3. Breakglass

Definition:

1. the workflow is available for recovery, emergency repair, deep debugging, or
   advanced testing,
2. it is not the default happy path,
3. docs should clearly label it as exceptional,
4. it must not be used to justify broad support claims.

Examples expected to fall here:

1. direct `limactl shell substrate ...`
2. direct in-guest `systemctl` manipulation
3. direct guest socket curls as the primary health check
4. host-side `SUBSTRATE_WORLD_SOCKET` override use on macOS

## Operator posture rules

Implementation and docs should follow these rules:

1. the normal operator story must start with Substrate-owned commands,
2. manual guest flows may remain documented only when clearly marked
   breakglass,
3. supported-mode docs must explicitly mention the unresolved same-user host
   ownership limitation,
4. breakglass availability must never be described as parity with Linux.

## File and surface implications

This taxonomy should govern wording and behavior across:

1. `docs/WORLD.md`
2. `docs/reference/world/platforms/macos-lima-setup.md`
3. `docs/USAGE.md`
4. `scripts/mac/lima-warm.sh`
5. `scripts/mac/lima-doctor.sh`
6. `scripts/mac/smoke.sh`
7. `scripts/mac/orchestration-smoke.sh`
8. `crates/world-mac-lima/src/lib.rs`
9. `crates/shell/src/builtins/world_gateway.rs`

## Questions future specs should answer

1. Which current flows can move fully into `supported` first?
2. Which flows need a temporary `degraded-but-supported` stop on the way to a
   more owned operator contract?
3. Which docs and scripts still teach breakglass as if it were normal?
