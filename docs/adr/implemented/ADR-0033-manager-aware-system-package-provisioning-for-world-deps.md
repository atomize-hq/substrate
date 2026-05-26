# ADR-0033 — Manager-Aware System-Package Provisioning for World Deps

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell maintainers; World backend maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

The world-deps provisioning surface must route system-package installation by world OS package
manager without reopening runtime mutation or host-OS mutation.

The stable decision is:

- `install.method=pacman` is a first-class world-deps schema surface alongside `apt`, `script`,
  and `manual`
- `substrate world enable --provision-deps` performs an in-world manager probe and provisions via
  the matching system-package manager on supported guest worlds
- runtime `substrate world deps current sync` and `install` never invoke `apt`, mutating `dpkg`,
  or `pacman`; they stay probe-only and fail early with remediation
- mixed-manager enabled sets fail before mutation rather than guessing or partially provisioning
- manager selection is derived from the world execution environment, not host PATH inspection or
  host package-manager availability

## Stable Owned Surface

This ADR extends the stable provisioning contract documented in:

- `docs/reference/world/deps/provisioning.md`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/builtins/world_deps/inventory.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/builtins/world_enable/runner/provision_deps.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/world-service/src/service.rs`
- `crates/shell/tests/world_enable_provision_deps_wdap0.rs`
- `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
- `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md`
- `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

## Historical Note

The original ADR captures the rollout and contract-reconciliation work needed to add pacman-backed
guest provisioning. The stable manager-aware provisioning contract now lives here and in the
world-deps reference docs.
