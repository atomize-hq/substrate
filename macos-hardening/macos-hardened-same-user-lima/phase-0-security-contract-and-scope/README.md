# Phase 0: Security Contract and Scope

Status: landed contract authority
Last updated: 2026-06-22

## Purpose

Record the **frozen contract** that later runtime and docs work already landed
against:

1. same-user Lima is supported only as a same-user model,
2. the normal operator path is Substrate-owned commands first,
3. guest-direct and override flows are exceptions, not the default story,
4. the taxonomy labels are fixed as `supported`, `degraded-but-supported`, and
   `breakglass`.

## What Phase 0 landed

Phase 0 established the feature-wide contract that later slices reused:

1. `substrate host doctor`, `substrate world doctor`, and
   `substrate world gateway sync|status|restart` were treated as already-landed
   baseline operator surfaces rather than future inventions.
2. `SUBSTRATE_WORLD_SOCKET` override use and direct guest administration were
   frozen as non-default flows.
3. the same-user non-parity boundary with Linux was made explicit.

## What remains authoritative from this phase

Read these first for contract wording:

1. [`../spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](../spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
2. [`milestone-0-1-target-mode-and-support-contract-sow.md`](./milestone-0-1-target-mode-and-support-contract-sow.md)
3. [`milestone-0-2-lima-version-and-breakglass-contract-sow.md`](./milestone-0-2-lima-version-and-breakglass-contract-sow.md)

## What this phase does **not** claim

1. Linux-equivalent host ownership isolation
2. a privilege boundary against the owning macOS user
3. that direct `limactl shell`, guest `systemctl`, guest `journalctl`,
   guest socket `curl`, or `SUBSTRATE_WORLD_SOCKET` override use are normal
   operator steps

## Current downstream authority

This phase is no longer the place to describe current runtime gaps as if they
are still open. For current landed behavior, defer to:

1. [`../README.md`](../README.md)
2. [`../../docs/WORLD.md`](../../docs/WORLD.md)
3. [`../../docs/reference/world/platforms/macos-lima-setup.md`](../../docs/reference/world/platforms/macos-lima-setup.md)
4. committed `SPEC-*`, `PLAN-*`, and `TASKS-*` files under [`../spec/`](../spec/)
