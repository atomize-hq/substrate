# ADR-0011 — World Deps Packages/Bundles Inventory and Enabled Contract

## Status

- Status: Implemented
- Original date (UTC): 2026-01-13
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell and world maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

World deps must use an inventory-directory plus enabled-patch model rather than legacy
selection-file and overlay-file plumbing.

The stable decision is:

- available world deps come from built-ins plus inventory directories under `$SUBSTRATE_HOME/deps/`
  and `<workspace_root>/.substrate/deps/`
- desired world deps come from `world.deps.enabled` patch keys in global and workspace config
  rather than separate selection files
- `substrate world deps` surfaces must distinguish inventory, enabled, and applied state
  explicitly through `current`, `global`, and `workspace` scopes
- legacy manifest, overlay, and selection files must not influence the active world-deps model

## Stable Owned Surface

This ADR owns the stable inventory and enabled-set contract documented in:

- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`
- `docs/reference/world/deps/authoring_packages.md`
- `docs/reference/world/deps/authoring_bundles.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/builtins/world_deps/inventory.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/shell/src/execution/config_model.rs`
- `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
- `crates/shell/tests/world_deps_applied_wdp2.rs`

## Related ADRs

- `docs/adr/historical/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- `docs/adr/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/adr/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- `docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md`

## Historical Note

The original ADR captured the migration from legacy selection and overlay files to the current
inventory-directory and enabled-patch contract. The stable operator/runtime contract now lives here
and in the world-deps reference docs.
