# ADR-0036 — Treat World Disabled as a First-Class Status in Health and Shim Doctor

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Diagnostics must treat `world.enabled=false` as an explicit disabled state rather than as a backend
failure.

The stable decision is:

- `substrate shim doctor` short-circuits world backend and applied world-deps probes when world is
  disabled
- `substrate health` and `substrate shim doctor` report explicit disabled/skip statuses instead of
  generic errors for that case
- JSON output carries stable additive status fields for disabled and skipped-disabled states
- enabled-world failures remain actionable attention-required diagnostics

## Stable Owned Surface

This ADR owns the stable disabled-diagnostics contract documented in:

- `docs/USAGE.md`
- `docs/INSTALLATION.md`
- `docs/contracts/diagnostics-json.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/builtins/shim_doctor/report.rs`
- `crates/shell/src/builtins/shim_doctor/output.rs`
- `crates/shell/src/builtins/health.rs`
- `crates/shell/tests/shim_doctor.rs`
- `crates/shell/tests/shim_health.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0037-doctor-health-attribute-why-world-is-disabled.md`

## Historical Note

The original ADR captured the shift from noisy probe failures to explicit disabled-state reporting.
The stable diagnostics contract now lives here and in the operator docs.
