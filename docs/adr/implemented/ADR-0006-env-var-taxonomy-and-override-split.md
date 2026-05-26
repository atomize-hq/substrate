# ADR-0006 — Env Var Taxonomy and Override Split

## Status

- Status: Implemented
- Original date (UTC): 2026-01-04
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): spenser

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Exported state and operator override inputs must use distinct environment-variable roles so cached
state does not masquerade as intentional config override.

The stable decision is:

- exported session state remains under `SUBSTRATE_*`
- config-shaped operator override inputs use `SUBSTRATE_OVERRIDE_*`
- exported state variables are outputs, not supported override inputs
- environment taxonomy is explicit across shell, shim, world, and install surfaces

## Stable Owned Surface

This ADR owns the supported env-variable contract documented in:

- `docs/reference/env/contract.md`
- `docs/ENVIRONMENT_VARIABLES.md`
- `docs/internals/env/inventory.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/env_scripts.rs`
- `crates/shell/src/execution/manager.rs`
- `scripts/substrate/install-substrate.sh`
- `crates/shell/tests/config_show.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0003-policy-and-config-mental-model-simplification.md`
- `docs/adr/implemented/ADR-0005-workspace-config-precedence-over-env.md`
- `docs/adr/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

## Historical Note

The original ADR captured the cleanup from dual-use `SUBSTRATE_*` variables to a clearer env
contract. The stable operator-facing contract now lives here and in the env reference docs.
