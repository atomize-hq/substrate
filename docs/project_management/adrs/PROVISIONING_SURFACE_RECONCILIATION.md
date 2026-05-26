# Provisioning Surface Reconciliation

Status: reference note  
Date: 2026-05-24

## Purpose

This note reconciles the currently implemented world-deps provisioning surface with older ADRs and archived planning material that still describe superseded commands or selection models.

## Current authoritative sources

The live operator-facing provisioning surface is:

- `substrate world enable --provision-deps`

The most authoritative current references are:

- [`docs/reference/world/deps/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/README.md)
  - authoritative operator-facing summary
  - states that runtime `substrate world deps current sync|install` never mutates system packages
  - points missing-package remediation at `substrate world enable --provision-deps`
- [`docs/reference/world/deps/provisioning.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/provisioning.md)
  - authoritative operator-facing contract for provisioning-time mutation, runtime fail-early behavior, and supported backend posture
- [`docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md)
  - curated stable ADR for the explicit provisioning-time system-package posture
- [`docs/adr/implemented/ADR-0033-manager-aware-system-package-provisioning-for-world-deps.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/adr/implemented/ADR-0033-manager-aware-system-package-provisioning-for-world-deps.md)
  - curated stable ADR for manager-aware system-package provisioning
- [`docs/internals/world/deps.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/world/deps.md)
  - authoritative implementation-oriented reference for inventory structure, enabled-set resolution, wrapper behavior, and runtime probe-only posture
- [`docs/WORLD.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
  - authoritative world/runtime architecture summary for provisioning request profiles

The current code surfaces that implement the live contract are:

- [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
  - defines `--provision-deps` on `substrate world enable`
- [`crates/shell/src/builtins/world_enable/runner.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_enable/runner.rs)
  - implements `world enable --provision-deps`
- [`crates/shell/src/builtins/world_enable/runner/provision_deps.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_enable/runner/provision_deps.rs)
  - implements backend support gates and manager-aware provisioning
- [`crates/shell/src/builtins/world_deps/surfaces.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/surfaces.rs)
  - implements runtime probe-only behavior and missing-package remediation

## Current implemented support model

The current implemented model is:

- Inventory may declare system-package requirements using `install.method=apt` or `install.method=pacman`.
- Runtime `substrate world deps current sync|install` is probe-only for system-package items.
- Runtime probes use read-only package-manager queries and fail early with remediation.
- The only Substrate surface that performs system-package mutation for world-deps is `substrate world enable --provision-deps`.
- Mixed `apt` and `pacman` requirement sets are rejected before mutation.

Current platform/backend posture:

| Platform/backend | `substrate world enable --provision-deps` | Runtime `substrate world deps current sync|install` |
| --- | --- | --- |
| macOS Lima guest backend | Supported | Probe-only; fails early when required system packages are missing |
| Linux host-native backend | Unsupported; no host OS mutation | Probe-only; fails early with “no host OS mutation” guidance |
| Windows | Unsupported | Probe-only; fails early with “unsupported on Windows” guidance |

## Historically useful but stale on command surface

These documents are still useful for understanding why the system evolved the way it did, but they are not authoritative for the current operator-facing provisioning command or current world-deps selection model.

- [`docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md)
  - historically useful for install-class framing
  - stale on command surface because it still specifies `substrate world deps provision`
  - stale on enabled-set selection because it still specifies `world-deps.selection.yaml`
- [`docs/project_management/_archived/world_deps_selection_layer/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/_archived/world_deps_selection_layer/)
  - historical planning/archive material
  - stale on both command surface and selection model
- [`docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md)
  - planning-rich historical source for the same decision
  - includes option analysis for the older `substrate world deps provision` command

## Relationship between the ADR chain

- [`ADR-0002`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md) is best treated as historical framing, not as the current command contract.
- [`ADR-0011`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md) is the stronger source for today’s inventory and enabled-set model.
- [`ADR-0030`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md) defines the explicit provisioning-time posture that the current implementation follows.
- [`ADR-0033`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/adr/implemented/ADR-0033-manager-aware-system-package-provisioning-for-world-deps.md) extends that provisioning surface to manager-aware routing for `pacman` as well as `apt`.
- [`ADR-0009`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md) extends the existing `world enable --provision-deps` contract for future Linux guest-rootfs support; it must not reintroduce `world deps provision`.

## What should be updated next

To make the documentation chain consistent end to end, the next updates should be:

1. `ADR-0002` now carries an explicit supersession note that points current readers at
   `substrate world enable --provision-deps`, the stable world-deps docs, and the curated ADRs.
2. Update any lingering non-archived docs that still say `substrate world deps provision` to `substrate world enable --provision-deps`, unless they are intentionally discussing rejected historical options.
3. Update any lingering non-archived docs that still present `world-deps.selection.yaml` as current operator truth.
4. Keep archived materials archived; do not “fix” them into looking current unless they are explicitly being revived.
5. `ADR-0030` and `ADR-0033` are now curated into `docs/adr/implemented/`; any future cleanup should
   treat the project-management copies as historical source rather than the stable ADR home.

## Practical reading order

If someone wants the current truth quickly, they should read in this order:

1. [`docs/reference/world/deps/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/README.md)
2. [`docs/reference/world/deps/provisioning.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/provisioning.md)
3. [`docs/internals/world/deps.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/world/deps.md)
4. The implementing code in [`crates/shell/src/builtins/world_enable/runner.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_enable/runner.rs), [`crates/shell/src/builtins/world_enable/runner/provision_deps.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_enable/runner/provision_deps.rs), and [`crates/shell/src/builtins/world_deps/surfaces.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/surfaces.rs)
