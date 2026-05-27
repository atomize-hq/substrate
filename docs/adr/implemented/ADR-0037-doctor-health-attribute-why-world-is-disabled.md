# ADR-0037 — Doctor and Health Attribute Why World Is Disabled

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Doctor and health surfaces must attribute disabled world isolation to the effective winning source
instead of implying `--no-world` generically.

The stable decision is:

- disabled attribution follows the effective source precedence across CLI flag, environment
  override, workspace config, and global config
- text output uses stable, non-secret attribution wording
- JSON output gains additive disable-source fields suitable for automation
- attribution changes messaging only; it does not alter world enablement behavior or precedence

## Stable Owned Surface

This ADR owns the stable disable-attribution contract documented in:

- `docs/reference/env/contract.md`
- `docs/USAGE.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/platform/linux.rs`
- `crates/shell/src/execution/platform/macos.rs`
- `crates/shell/src/builtins/health.rs`
- `crates/shell/tests/doctor_scopes_ds0.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0036-world-disabled-first-class-status-in-health-and-shim-doctor.md`
- `docs/adr/implemented/ADR-0038-replay-attribute-why-world-is-disabled-in-warnings.md`

## Historical Note

The original ADR captured the attribution-source tradeoffs for disabled world messaging. The stable
operator-facing attribution contract now lives here and in the env and usage docs.
