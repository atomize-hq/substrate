# Fixtures

This directory contains deterministic fixture inputs and golden artifacts for crate seams.

Current live fixture coverage includes:

- `fixtures/kernel/` for kernel schema, canonical JSON, and identity determinism coverage
- `fixtures/pack/` for landed pack validation and fingerprint determinism coverage
- `fixtures/repo/` for Phase A repo snapshot manifests and checked-in worktree fixture trees

Current pack fixtures are organized by intent rather than by runtime feature:

- `valid/` for standalone pack happy paths and file-backed bundle resolution cases
- `invalid/` for TOML/schema/reference rejection cases plus bundle-resolution edge cases
- `canonical/` for source and semantic fingerprint determinism cases across standalone packs and bundles

Named pack fixtures now span the Phase C pack families as well as profiles, for example `valid/score/generic_lift_v2.json`, `valid/queries/rust_core.json`, `valid/rules/generic_policy.json`, `valid/recipes/generic_core_recipes.json`, and the `valid/bundle/` and `canonical/bundle_order_*` trees.

The landed repo seam is narrower and internal-only: crate-private worktree snapshot materialization, deterministic inventory/blob assembly, fingerprinting, ignore-policy handling, snapshot diagnostics, and the snapshot-manifest schema contract used by `fixtures/repo/`.

The current fixture tree should not be read as proof that git-backed diffs, boundary overlap detection, path classification, query execution, scoring, recipe application, or runtime bootstrap exists. The repo seam proves filesystem-first immutable snapshots only, and the pack seam proves structural compilation and deterministic bundle closure only.
