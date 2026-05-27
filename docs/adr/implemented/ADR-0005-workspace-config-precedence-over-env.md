# ADR-0005 — Workspace Config Precedence Over Env

## Status

- Status: Implemented
- Original date (UTC): 2026-01-02
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): spenser

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

When an enabled workspace exists, workspace config is authoritative over environment-exported
state and no-workspace override env inputs.

The stable decision is:

- workspace config at `<workspace_root>/.substrate/workspace.yaml` overrides global config for
  effective config resolution
- when an enabled workspace exists, `SUBSTRATE_OVERRIDE_*` config inputs are ignored
- CLI flags remain the highest-precedence config inputs where a flag exists
- no-workspace runs may still use `SUBSTRATE_OVERRIDE_*` as run-only override inputs

## Stable Owned Surface

This ADR owns the current precedence contract documented in:

- `docs/reference/config/contract.md`
- `docs/reference/config/world.md`
- `docs/reference/env/contract.md`
- `docs/CONFIGURATION.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/config_cmd.rs`
- `crates/shell/src/execution/workspace.rs`
- `crates/shell/tests/config_show.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0003-policy-and-config-mental-model-simplification.md`
- `docs/adr/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
- `docs/adr/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

## Historical Note

The original ADR captured the correction to the earlier precedence model once stable env exports
made workspace-vs-env ambiguity visible. The stable precedence contract now lives here and in the
config and env reference docs.
