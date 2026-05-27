# ADR-0004 — World OverlayFS Directory Enumeration Reliability

## Status

- Status: Implemented
- Original date (UTC): 2025-12-29
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): World backend maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Linux world execution must refuse filesystem strategies that cannot provide correct directory
enumeration, and it must make strategy selection observable.

The stable decision is:

- a world filesystem strategy is only usable if its enumeration health check passes
- Linux strategy selection follows a deterministic primary/fallback chain rather than silently
  guessing
- required-world execution fails closed when no viable strategy exists; optional-world execution
  warns once and falls back to host
- doctor and trace surfaces expose the selected strategy and fallback reason so backend behavior is
  reproducible

## Stable Owned Surface

This ADR owns the stable strategy-selection and observability contract documented in:

- `docs/WORLD.md`
- `docs/TRACE.md`
- `docs/ISOLATION_SUPPORT_MATRIX.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/world/src/overlayfs/strategy.rs`
- `crates/world/src/overlayfs/strategy_state.rs`
- `crates/world/src/exec.rs`
- `crates/world-service/src/service.rs`
- `crates/world/tests/overlayfs_enumeration_fallback.rs`
- `crates/shell/tests/world_overlayfs_enumeration_wo0.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`
- `docs/adr/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Historical Note

The original ADR captured the rollout detail for the Linux overlay and fuse fallback chain. The
stable operator/runtime contract now lives here and in the world and trace docs.
