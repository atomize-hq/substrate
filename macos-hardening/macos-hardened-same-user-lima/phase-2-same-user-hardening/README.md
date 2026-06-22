# Phase 2: Same-User Hardening

Status: landed technical hardening phase
Last updated: 2026-06-22

## Purpose

Describe the **landed** technical hardening outcomes for the same-user Lima
path, not the pre-landing wishlist.

## What Phase 2 landed

Phase 2 closed the main technical posture gaps that earlier docs still
described as open:

1. the hardened macOS path no longer injects the default extra guest TCP
   listener,
2. the hardened default no longer depends on broad mounted host-home visibility
   as the normal ingress path,
3. guest service authority was unified under `scripts/mac/lima/units/`.

## Current hardening posture

The meaningful same-user Lima hardening summary is now:

1. the canonical guest endpoint is `/run/substrate.sock`,
2. the default ingress path is staged-workspace copy into
   `/var/lib/substrate/staged-workspace/current`,
3. canonical unit sources are:
   - `scripts/mac/lima/units/substrate-world-service.service.tmpl`
   - `scripts/mac/lima/units/substrate-world-service.socket`
4. `SUBSTRATE_WORLD_SOCKET` remains breakglass / advanced override material,
   not the normal path.

## What remains intentionally deferred

Phase 2 did **not** turn the whole macOS lifecycle into a single polished
owned-CLI experience.

The main intentionally deferred items are:

1. the current create/warm/repair path still runs through the
   degraded-but-supported helper `scripts/mac/lima-warm.sh`,
2. `substrate workspace sync` is still not the frozen normal macOS sync/copy
   contract,
3. Phase 3 remains responsible for keeping the operator-facing story honest and
   clearly classified.

## Current downstream authority

1. [`../../docs/WORLD.md`](../../docs/WORLD.md)
2. [`../../docs/reference/world/platforms/macos-lima-setup.md`](../../docs/reference/world/platforms/macos-lima-setup.md)
3. [`../spec/SPEC-07-remove-default-extra-listener-surface.md`](../spec/SPEC-07-remove-default-extra-listener-surface.md)
4. [`../spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`](../spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md)
5. [`../spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md`](../spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md)
6. [`../spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`](../spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md)
