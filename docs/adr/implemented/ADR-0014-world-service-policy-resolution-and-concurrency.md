# ADR-0014 — World-Service Policy Resolution and Concurrency

## Status

- Status: Implemented
- Original date (UTC): 2026-01-18
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Substrate core team

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0014-world-service-policy-resolution-and-concurrency.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

World execution must consume a host-resolved effective policy snapshot rather than resolving policy
inside world-service from shared mutable broker state.

The stable decision is:

- the shell is the authoritative resolver for the effective policy attached to world execution
  requests
- world-service consumes the request snapshot as its policy input for both non-PTY and PTY paths
- world-service must not derive per-request effective policy from local filesystem state or shared
  broker mutation
- trace and execution surfaces must preserve snapshot provenance so policy enforcement remains
  deterministic and auditable

## Stable Owned Surface

This ADR owns the stable policy-snapshot contract documented in:

- `docs/WORLD.md`
- `docs/TRACE.md`
- `docs/reference/world/contract.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/policy_snapshot.rs`
- `crates/world-service/src/service.rs`
- `crates/world-service/src/pty.rs`
- `crates/transport-api-types/src/lib.rs`
- `crates/trace/src/tests.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`
- `docs/adr/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`
- `docs/adr/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Historical Note

The original ADR captured the migration away from world-service-local broker resolution. The stable
policy snapshot contract now lives here and in the world and trace docs.
