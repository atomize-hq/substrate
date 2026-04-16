# Schemas

This directory contains hand-authored JSON schemas for crate contracts.

Current shipped schema sets are:

- `schemas/kernel/primitives.v1.json` for shared kernel primitives
- `schemas/pack/common.v1.json` for pack-local shared definitions
- `schemas/pack/profile.v1.json` for profile documents
- `schemas/pack/boundary_taxonomy.v1.json` for boundary taxonomy packs
- `schemas/pack/component_map.v1.json` for component map packs

The landed pack compiler embeds the `pack/common`, `pack/profile`, `pack/boundary_taxonomy`, and `pack/component_map` schemas through `src/pack/schema.rs`, following the same embed-and-access pattern used by `src/kernel/schema.rs`. Compilation uses typed validation aligned to those embedded schema contracts and does not load schema files from disk at runtime.

Phase A established the common/profile schema foundation. Phase B adds only the boundary taxonomy and component map topology schemas. That topology work remains compiler-internal: it does not mean repo classification, overlap analysis against a snapshot, or app/runtime-facing topology consumption has landed.

Score-model, query-pack, rule-pack, recipe-pack, and runtime-facing schema work remain deferred to later seams.
