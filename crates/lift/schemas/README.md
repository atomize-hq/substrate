# Schemas

This directory contains hand-authored JSON schemas for crate contracts.

Current shipped schema sets are:

- `schemas/kernel/primitives.v1.json` for shared kernel primitives
- `schemas/pack/common.v1.json` for pack-local shared definitions
- `schemas/pack/profile.v1.json` for profile documents
- `schemas/pack/boundary_taxonomy.v1.json` for boundary taxonomy packs
- `schemas/pack/component_map.v1.json` for component map packs
- `schemas/pack/score_model.v1.json` for structural score-model packs
- `schemas/pack/query_pack.v1.json` for structural query packs
- `schemas/pack/rule_pack.v1.json` for structural rule packs
- `schemas/pack/recipe_pack.v1.json` for structural recipe packs

The landed pack compiler embeds the pack schemas through `src/pack/schema.rs`, following the same embed-and-access pattern used by `src/kernel/schema.rs`. Compilation uses typed validation aligned to those embedded schema contracts and does not load schema files from disk at runtime.

Phase A established the common/profile schema foundation. Phase B added boundary taxonomy and component map topology schemas. Phase C extends that surface with score models, query packs, rule packs, and recipe packs, plus deterministic profile bundle resolution into a crate-internal `CompiledPackSet`.

The advanced schemas remain structural only in this seam. They do not imply score execution, tree-sitter query execution, rule runtime, recipe application, repo walking, or app/runtime bootstrap.
