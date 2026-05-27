# ADR-0009 — Linux Guest RootFS Backend and Linux System-Package Provisioning

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-05-24
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Shell, world, and installer maintainers

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

Substrate may add a Linux guest-rootfs backend that decouples the in-world userspace from the host
distro and allows provisioning-time system-package mutation without touching the workstation OS.

The queued direction that still matters is:

- explicit Linux backend selection between host-native and guest-rootfs modes
- guest-rootfs warmup as a separate, doctor-visible prerequisite
- Linux provisioning support only when the active backend and guest image make host OS mutation
  avoidable
- full-isolation semantics rooted in the guest userspace rather than host system-directory bind
  mounts

## Why Queued

This is still active platform/input work, but it is not landed and should not yet be treated as a
stable runtime contract.

When implementation is ready, it should be restated against:

- `docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md`
- `docs/adr/implemented/ADR-0033-manager-aware-system-package-provisioning-for-world-deps.md`
- `docs/adr/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`

## Draft Note

Keep the project-management ADR for original planning detail, but treat this curated draft as the
queued Linux guest-rootfs placeholder.
