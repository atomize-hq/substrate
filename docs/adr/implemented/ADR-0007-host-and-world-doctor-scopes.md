# ADR-0007 — Host and World Doctor Scopes

## Status

- Status: Implemented
- Original date (UTC): 2026-01-07
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell and world backend maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0007-host-and-world-doctor-scopes.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Doctor output must distinguish host-side transport readiness from world-side enforcement readiness.

The stable decision is:

- `substrate host doctor` reports host prerequisites and transport reachability only
- `substrate world doctor` reports both the host summary and a world-service-reported world summary
- guest-kernel enforcement facts must come from the backend doctor API rather than ad hoc
  host-side inference
- doctor exit behavior must distinguish “not provisioned / not supported” from “expected backend
  path exists but the world-service is unreachable”

## Stable Owned Surface

This ADR owns the stable doctor-scope contract documented in:

- `docs/COMMANDS.md`
- `docs/USAGE.md`
- `docs/INSTALLATION.md`
- `docs/reference/world/platforms/macos-lima-setup.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/platform/linux.rs`
- `crates/shell/src/execution/platform/macos.rs`
- `crates/transport-api-types/src/lib.rs`
- `crates/world-service/src/service.rs`
- `crates/shell/tests/doctor_scopes_ds0.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
- `docs/adr/implemented/ADR-0014-world-service-policy-resolution-and-concurrency.md`

## Historical Note

The original ADR captured the rollout work that split doctor output into host and world scopes. The
stable doctor contract now lives here and in the CLI, installation, and platform verification docs.
