# ADR-0030 — Provisioning-Time System-Package Mutation for World Deps

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell maintainers; World backend maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

World-deps system-package mutation must be explicit, provisioning-time only, and isolated from
runtime dependency application.

The stable decision is:

- `substrate world enable --provision-deps` is the only Substrate command that may mutate
  system packages for world deps
- runtime `substrate world deps current sync` and `install` remain probe-only for
  system-package-backed items
- requirement derivation comes from the effective enabled world-deps set for the current
  directory rather than ad hoc package-manager invocation at runtime
- provisioning is allowed only on supported guest-world backends; Linux host-native and Windows
  remain fail-closed and must not mutate the host OS
- provisioning uses a reserved internal request posture so explicit guest mutation does not relax
  the normal hardened runtime execution profile

## Stable Owned Surface

This ADR owns the stable provisioning-time contract documented in:

- `docs/reference/world/deps/provisioning.md`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`
- `docs/WORLD.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/builtins/world_enable/runner/provision_deps.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/world-service/src/service.rs`
- `crates/shell/tests/world_enable_provision_deps_wdap0.rs`
- `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0033-manager-aware-system-package-provisioning-for-world-deps.md`
- `docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

## Historical Note

The original ADR captures the option analysis and rollout framing for the shift away from runtime
APT mutation. The stable operator contract now lives here and in the world-deps reference docs.
