# ADR-0008 — Workspace Config and Policy Scope Unification

## Status

- Status: Implemented
- Original date (UTC): 2026-01-10
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): spenser

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Config and policy use sparse patch files at global and workspace scopes under one `.substrate/`
layout and one shared workspace discovery model.

The stable decision is:

- global patches live under `$SUBSTRATE_HOME`
- workspace patches live under `<workspace_root>/.substrate/`
- config and policy commands operate on patch files, not hidden alternate formats
- workspace discovery, workspace disable markers, and patch-file ownership are shared across
  config and policy flows
- current/global/workspace command surfaces expose effective views separately from raw patch files

## Stable Owned Surface

This ADR owns the patch-file scope model documented in:

- `docs/reference/config/contract.md`
- `docs/reference/env/contract.md`
- `docs/reference/world/deps/README.md`
- `docs/CONFIGURATION.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/common/src/paths.rs`
- `crates/shell/src/execution/config_cmd.rs`
- `crates/shell/src/execution/policy_cmd.rs`
- `crates/shell/src/execution/workspace_cmd.rs`
- `crates/shell/src/execution/workspace.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0003-policy-and-config-mental-model-simplification.md`
- `docs/adr/implemented/ADR-0005-workspace-config-precedence-over-env.md`
- `docs/adr/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
- `docs/adr/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- `docs/adr/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`

## Historical Note

The original ADR captured the migration from ad hoc config and policy files to the shared
patch-file scope model. The stable contract now lives here and in the config, env, and world-deps
reference docs.
