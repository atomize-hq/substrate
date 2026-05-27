# ADR-0034 — Stabilize Dev-Install Helper Discovery Under `SUBSTRATE_HOME`

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Dev installs must stage the helper/runtime bundle needed by `substrate world enable` under
`$SUBSTRATE_HOME` instead of relying on brittle repo-target discovery.

The stable decision is:

- dev installs keep the live host binary symlink model, but stage world-enable helpers under
  `$SUBSTRATE_HOME`
- helper discovery must survive `cargo clean` and missing `<repo>/target/scripts/...` bridges
- dev uninstall only removes the staged helper bundle it owns
- production install layout remains unchanged

## Stable Owned Surface

This ADR owns the stable dev-install helper discovery behavior documented in:

- `docs/INSTALLATION.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/dev-uninstall-substrate.sh`
- `crates/shell/src/builtins/world_enable/runner/paths.rs`
- `crates/shell/tests/world_enable.rs`
- `tests/installers/install_smoke.sh`

## Related ADRs

- `docs/adr/implemented/ADR-0035-make-substrate-world-enable-work-after-dev-install-no-world.md`

## Historical Note

The original ADR captured the staging-vs-parity option analysis for dev installs. The stable
helper-discovery contract now lives here and in the installation guide.
