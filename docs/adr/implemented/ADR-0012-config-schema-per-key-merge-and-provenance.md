# ADR-0012 — Config Schema Per-Key Merge and Provenance

## Status

- Status: Implemented
- Original date (UTC): 2026-01-14
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Config patch files remain sparse YAML containers, but the schema owns how each key merges and how
effective config provenance is explained.

The stable decision is:

- merge behavior is defined per key rather than by one global file-level rule
- most keys use replace semantics, while selected keys intentionally merge across scopes
- effective config explainability must surface merge strategy and contributing sources
- `world.deps.enabled` is an additive key and follows deterministic ordered-set merge semantics

## Stable Owned Surface

This ADR owns the merge and provenance contract documented in:

- `docs/reference/config/contract.md`
- `docs/reference/world/deps/README.md`
- `docs/CONFIGURATION.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/config_cmd.rs`
- `crates/shell/src/execution/policy_cmd.rs`
- `crates/shell/tests/config_show.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/adr/implemented/ADR-0011-world-deps-packages-bundles-contract.md`
- `docs/adr/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`

## Historical Note

The original ADR captured the refinement that made additive config keys and explainable provenance
compatible with the shared patch-file model. The stable merge contract now lives here and in the
config and world-deps reference docs.
