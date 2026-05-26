# ADR-0003 — Policy and Config Mental Model Simplification

## Status

- Status: Implemented
- Original date (UTC): 2025-12-27
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): spenser

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0003-policy-and-config-mental-model-simplification.md`
- Original untemplated draft retained for history:
  - `docs/project_management/adrs/draft/ADR-0003-policy-and-config-mental-model-simplification_OG.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Substrate config and policy must follow one strict, explainable mental model with canonical file
names, explicit scope boundaries, and no legacy discovery fallbacks.

The stable decision is:

- canonical config and policy inputs live under `$SUBSTRATE_HOME` and `.substrate/` using
  `config.yaml`, `policy.yaml`, and `workspace.yaml`
- workspace discovery is explicit and deterministic
- `anchor` terminology replaces older `root` naming for world-root selection
- policy mode is an explicit contract surface rather than an implicit runtime posture
- cached exported state belongs to generated env scripts and must not create ambiguous config
  discovery

## Stable Owned Surface

This ADR anchors the stable config and policy mental model documented in:

- `docs/reference/config/contract.md`
- `docs/reference/config/world.md`
- `docs/CONFIGURATION.md`
- `docs/internals/config/world_root_and_caging.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/common/src/paths.rs`
- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/workspace.rs`
- `crates/shell/src/execution/settings/builder.rs`
- `crates/shell/src/execution/manager.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0005-workspace-config-precedence-over-env.md`
- `docs/adr/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
- `docs/adr/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/adr/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- `docs/adr/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`

## Historical Note

The original ADR captured the large cleanup that removed overlapping config and policy concepts.
The stable operator and runtime contract now lives here and in the reference docs.
