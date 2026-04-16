# Fixtures

This directory contains deterministic fixture inputs and golden artifacts for crate seams.

Current live fixture coverage includes:

- `fixtures/kernel/` for kernel schema, canonical JSON, and identity determinism coverage
- `fixtures/pack/` for Phase A pack validation and fingerprint determinism coverage

Phase A pack fixtures are organized by intent rather than by runtime feature:

- `valid/` for standalone profile happy paths
- `invalid/` for TOML/schema/reference rejection cases
- `canonical/` for source and semantic fingerprint determinism cases

Those pack fixtures are intended to cover the Phase A compiler surface only: profile compilation from builtin, file, and inline sources; ordered diagnostics; and stable fingerprints across equivalent TOML documents. Topology packs, query/rule/recipe packs, repo-walking fixtures, and runtime bootstrap fixtures remain deferred to later seams.
