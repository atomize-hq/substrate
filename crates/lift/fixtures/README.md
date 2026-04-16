# Fixtures

This directory contains deterministic fixture inputs and golden artifacts for crate seams.

Current live fixture coverage includes:

- `fixtures/kernel/` for kernel schema, canonical JSON, and identity determinism coverage
- `fixtures/pack/` for landed pack validation and fingerprint determinism coverage

Current pack fixtures are still organized by intent rather than by runtime feature:

- `valid/` for standalone profile happy paths, including profiles that carry topology refs
- `invalid/` for TOML/schema/reference rejection cases
- `canonical/` for source and semantic fingerprint determinism cases

Named pack fixtures in the current tree are still profile-oriented, for example `profile_minimal.json`, `profile_full.json`, and `profile_file_backed.toml`. The Phase B topology expansion is reflected here through profile topology refs and builtin topology resolution, not through a separate runtime classification fixture tree.

The current fixture tree should not be read as proof that repo walking, boundary overlap detection, path classification, or runtime bootstrap exists. Standalone topology compilation is a compiler capability, but topology packs are still crate-internal compiler artifacts in this slice. Query/rule/recipe packs, repo-walking fixtures, and runtime bootstrap fixtures remain deferred to later seams.
