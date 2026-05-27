# ADR-0035 — Make `substrate world enable` Work After `dev-install-substrate.sh --no-world`

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

The dev-install `--no-world` flow must still leave a deterministic later path for
`substrate world enable`.

The stable decision is:

- `dev-install-substrate.sh --no-world` stages the world-service/runtime artifacts needed for later
  `substrate world enable`
- `substrate world enable` remains provisioning-focused and performs deterministic preflight checks
  instead of trying ad hoc build work
- missing staged artifacts fail fast with actionable remediation
- production installs remain unchanged

## Stable Owned Surface

This ADR owns the stable dev-install delayed-enable contract documented in:

- `docs/INSTALLATION.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `scripts/substrate/dev-install-substrate.sh`
- `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
- `crates/shell/src/builtins/world_enable/runner/verify.rs`
- `crates/shell/src/builtins/world_enable/runner/manager_env.rs`
- `tests/installers/install_smoke.sh`

## Related ADRs

- `docs/adr/implemented/ADR-0034-stabilize-dev-install-helper-discovery-under-substrate-home.md`

## Historical Note

The original ADR captured the option tradeoffs around enable-time builds versus install-time
staging. The stable delayed-enable contract now lives here and in the installation guide.
