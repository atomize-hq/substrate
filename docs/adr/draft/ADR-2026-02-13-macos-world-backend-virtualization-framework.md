# ADR-2026-02-13 — macOS World Backend via Virtualization.framework

## Status

- Status: Draft
- Queue state: Proposed
- Original date (UTC): 2026-02-13
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Substrate runtime team

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-2026-02-13-macos-world-backend-virtualization-framework.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Proposed Direction

Substrate may add a macOS world backend based on Apple Virtualization.framework, including both
Linux-guest and macOS-guest VM-backed execution modes on Apple Silicon.

The proposed direction that still matters is:

- prefer a VM-backed macOS-native backend over non-VM sandbox emulation
- support VF-Linux as a Lima-reduction path and VF-macOS as a macOS-tooling path
- keep filesystem, command, and egress policy enforcement aligned with the existing world model
- stage rollout behind explicit backend selection and compatibility fallback

## Why Proposed

This remains a platform architecture proposal, not an implemented backend contract.

When implementation is ready, it should be restated against:

- `docs/WORLD.md`
- `docs/ISOLATION_SUPPORT_MATRIX.md`
- `docs/adr/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`

## Draft Note

Keep the project-management ADR for the original architecture proposal, but treat this curated
draft as the proposed Virtualization.framework backend placeholder.
