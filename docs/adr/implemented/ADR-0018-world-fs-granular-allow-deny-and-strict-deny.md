# ADR-0018 — World FS Granular Allow/Deny and Strict Deny

## Status

- Status: Implemented
- Original date (UTC): 2026-01-29
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): World backend maintainers; Shell maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

The world filesystem contract owns project-relative read, discover, and write controls plus
fail-closed deny behavior for hardened world execution.

The stable decision is:

- world filesystem controls are expressed per dimension (`read`, `discover`, `write`) rather than
  as one coarse allowlist
- deny semantics must be explicit, validated, and fail closed rather than silently ignored
- hardened world execution must apply deny behavior before user code runs and treat unsupported
  enforcement prerequisites as execution failures
- the same world-fs contract and snapshot semantics apply across non-PTY execution, PTY sessions,
  doctor diagnostics, and operator-facing config docs

## Stable Owned Surface

This ADR owns the stable world-fs policy contract documented in:

- `docs/CONFIGURATION.md`
- `docs/reference/config/world.md`
- `docs/WORLD.md`
- `docs/internals/config/world_root_and_caging.md`
- `docs/ISOLATION_SUPPORT_MATRIX.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/transport-api-types/src/lib.rs`
- `crates/shell/src/execution/policy_snapshot.rs`
- `crates/world-service/src/enforcement_plan.rs`
- `crates/world-service/src/internal_exec.rs`
- `crates/world-service/src/service.rs`
- `crates/world-service/src/pty.rs`
- `crates/shell/tests/policy_discovery.rs`
- `crates/world-service/tests/full_isolation_nonpty.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
- `docs/adr/implemented/ADR-0014-world-service-policy-resolution-and-concurrency.md`
- `docs/adr/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`
- `docs/adr/draft/ADR-0029-host-event-bus-and-router-daemon.md`

## Historical Note

The original ADR captured the schema break, enforcement rollout, and planning-pack detail for
granular world-fs controls. The stable operator/runtime contract now lives here and in the config,
world, and isolation docs.
