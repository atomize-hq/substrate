# Schemas

This directory contains hand-authored JSON schemas for crate contracts.

Current shipped schema sets are:

- `schemas/kernel/primitives.v1.json` for shared kernel primitives
- `schemas/pack/common.v1.json` for pack-local shared definitions
- `schemas/pack/profile.v1.json` for Phase A profile documents

The Phase A pack compiler embeds the `pack/common` and `pack/profile` schemas through `src/pack/schema.rs`, following the same embed-and-access pattern used by `src/kernel/schema.rs`. Phase A compilation uses typed validation aligned to those embedded schema contracts and does not load schema files from disk at runtime.

Phase A pack schema scope is intentionally narrow. Only the common/profile schemas ship here today. Topology, score-model, query-pack, rule-pack, recipe-pack, and runtime-facing schema work stays deferred to later seams.
