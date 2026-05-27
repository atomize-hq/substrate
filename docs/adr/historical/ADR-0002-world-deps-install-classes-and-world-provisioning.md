# ADR-0002 — World-Deps Install Classes and Provisioning-Time System Packages

## Status

- Status: Historical
- Original date (UTC): 2025-12-24
- Curated into `docs/adr/historical/`: 2026-05-26
- Owner(s): Shell, world, and installer maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md`

This curated ADR is kept only as historical context. The project-management ADR remains as the
planning-rich source retained for compatibility while `docs/project_management/**` is retired.

## Historical Decision Snapshot

This ADR established the early world-deps install-class framing:

- distinguish user-space installs from system-package installs
- keep OS-level package mutation out of runtime `world deps sync/install`
- require explicit provisioning-time handling for system packages

That framing still matters historically because it introduced the security posture later refined by
the current world-deps contract.

## Why Historical

This ADR is no longer the current operator contract.

Its command surface and configuration model were replaced by:

- `docs/adr/implemented/ADR-0011-world-deps-packages-bundles-contract.md`
- `docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md`
- `docs/adr/implemented/ADR-0033-manager-aware-system-package-provisioning-for-world-deps.md`

Those successor ADRs preserve the provisioning-time-only mutation posture while replacing the old
selection-file model and the obsolete `substrate world deps provision` command shape.

## Historical Note

Keep the original ADR for install-class and provisioning-posture history, not as a live contract
for the current world-deps CLI or inventory model.
