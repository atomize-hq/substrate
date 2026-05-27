# ADR-0015 — Full Isolation Landlock OverlayFS Backing Dirs

## Status

- Status: Implemented
- Original date (UTC): 2026-01-20
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): World backend maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Full-isolation writable allowlists must remain usable even when overlayfs requires internal backing
directory writes.

The stable decision is:

- allowlisted project writes in full isolation must not fail solely because overlayfs needs
  internal upper/work directory writes
- runtime-derived internal write roots are execution details and must not become part of the
  user-facing policy schema or snapshot identity
- when required internal write roots cannot be derived for required-world execution, the runtime
  fails closed with high-signal diagnostics
- non-allowlisted writes remain denied; this ADR fixes spurious `EPERM`, not the deny contract

## Stable Owned Surface

This ADR owns the stable full-isolation writable-path contract documented in:

- `docs/WORLD.md`
- `docs/ISOLATION_SUPPORT_MATRIX.md`
- `docs/INSTALLATION.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/world/src/exec.rs`
- `crates/world-service/src/internal_exec.rs`
- `crates/world-service/src/service.rs`
- `crates/world-service/src/pty.rs`
- `crates/world-service/tests/full_isolation_nonpty.rs`
- `crates/world-service/tests/full_isolation_pty.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0014-world-service-policy-resolution-and-concurrency.md`
- `docs/adr/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Historical Note

The original ADR captured the overlayfs/Landlock compatibility investigation and rollout steps. The
stable allowlisted-write contract now lives here and in the world installation/isolation docs.
