# Phase 3: Substrate-Owned Operations

Status: landed operator-contract cutover with explicit interim helpers
Last updated: 2026-06-22

## Purpose

Describe the current operator contract for hardened same-user Lima after the
Phase 3 cutover work landed.

## Current operator contract

### supported

1. `substrate host doctor [--json]`
2. `substrate world doctor [--json]`
3. `substrate world gateway sync|status|restart`
4. `substrate world enable`
5. `substrate world deps current sync`

### degraded-but-supported

1. `scripts/mac/lima-warm.sh` for create/warm/repair plus staged-workspace copy
2. `scripts/mac/lima-doctor.sh` for routed-first deeper troubleshooting

### breakglass

1. raw `limactl shell`
2. plain SSH
3. direct guest `systemctl`
4. direct guest `journalctl`
5. direct guest socket `curl`
6. host-side `SUBSTRATE_WORLD_SOCKET` override use

## What Phase 3 landed

1. top-level macOS operator docs now teach the owned CLI path first,
2. helper scripts are classified as degraded-but-supported rather than as the
   primary authority,
3. guest-direct diagnosis and administration are explicitly breakglass,
4. the sync/copy story remains honest: `substrate workspace sync` exists but is
   not yet the frozen normal macOS same-user Lima path.

## What remains intentionally true

Phase 3 does **not** claim:

1. full Linux-equivalent host ownership parity,
2. complete retirement of the current helper-backed create/warm/repair flow,
3. that direct guest administration disappeared,
4. that workspace lifecycle/sync productization is finished.

Instead, it makes those remaining caveats visible and correctly classified.

## Current downstream authority

1. [`../../docs/WORLD.md`](../../docs/WORLD.md)
2. [`../../docs/reference/world/platforms/macos-lima-setup.md`](../../docs/reference/world/platforms/macos-lima-setup.md)
3. [`../../docs/USAGE.md`](../../docs/USAGE.md)
4. [`../../docs/contracts/gateway/operator-contract.md`](../../docs/contracts/gateway/operator-contract.md)
5. [`../spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`](../spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md)
6. [`../spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md`](../spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md)
