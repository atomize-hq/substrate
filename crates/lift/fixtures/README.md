# Fixtures

This directory contains deterministic fixture inputs and golden artifacts for crate seams.

Current live fixture coverage includes:

- `fixtures/kernel/` for kernel schema, canonical JSON, and identity determinism coverage
- `fixtures/pack/` for landed pack validation and fingerprint determinism coverage

Current pack fixtures are organized by intent rather than by runtime feature:

- `valid/` for standalone pack happy paths and file-backed bundle resolution cases
- `invalid/` for TOML/schema/reference rejection cases plus bundle-resolution edge cases
- `canonical/` for source and semantic fingerprint determinism cases across standalone packs and bundles

Named pack fixtures now span the Phase C pack families as well as profiles, for example `valid/score/generic_lift_v2.json`, `valid/queries/rust_core.json`, `valid/rules/generic_policy.json`, `valid/recipes/generic_core_recipes.json`, and the `valid/bundle/` and `canonical/bundle_order_*` trees.

The current fixture tree should not be read as proof that repo walking, boundary overlap detection, path classification, query execution, scoring, recipe application, or runtime bootstrap exists. Phase C proves structural compilation and deterministic bundle closure only.
