# ADR-0010 — World Backend Contract and Capability Divergence Surfacing

## Status

- Status: Draft
- Queue state: Proposed
- Original date (UTC): 2026-01-11
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Shell, world, and broker maintainers

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Proposed Direction

Substrate needs a cross-backend world contract that keeps capability divergence explicit while
preserving stable operator-facing semantics.

The proposed direction that still matters is:

- doctor reports backend identity plus capability booleans and remediation
- trace reports backend kind, in-world/host posture, and fallback reasons explicitly
- fail-closed versus degrade behavior stays stable across backend implementations
- filesystem safety, provisioning posture, and observability remain contract-owned instead of
  backend-specific folklore

## Why Proposed

This is still a platform-contract proposal, not a landed behavior contract.

When implementation is ready, it should be restated against:

- `docs/WORLD.md`
- `docs/ISOLATION_SUPPORT_MATRIX.md`
- `docs/adr/implemented/ADR-0007-host-and-world-doctor-scopes.md`
- `docs/adr/implemented/ADR-0014-world-service-policy-resolution-and-concurrency.md`

## Draft Note

Keep the project-management ADR for the original proposal detail, but treat this curated draft as
the proposed backend-contract placeholder.
