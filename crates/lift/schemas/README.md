# Schemas

This directory contains hand-authored JSON schemas for crate contracts.

Current shipped schema sets are:

- `schemas/kernel/primitives.v1.json` for shared kernel primitives
- `schemas/pack/common.v1.json` for pack-local shared definitions
- `schemas/pack/profile.v1.json` for profile documents
- `schemas/repo/snapshot_manifest.v1.json` for deterministic repo snapshot fixture manifests
- `schemas/repo/diff_manifest.v1.json` for deterministic repo pure-diff fixture manifests
- `schemas/pack/boundary_taxonomy.v1.json` for boundary taxonomy packs
- `schemas/pack/component_map.v1.json` for component map packs
- `schemas/pack/score_model.v1.json` for structural score-model packs
- `schemas/pack/query_pack.v1.json` for structural query packs
- `schemas/pack/rule_pack.v1.json` for structural rule packs
- `schemas/pack/recipe_pack.v1.json` for structural recipe packs

The landed pack compiler embeds the pack schemas through `src/pack/schema.rs`, following the same embed-and-access pattern used by `src/kernel/schema.rs`. The internal repo seam uses that same pattern through `src/repo/schema.rs` for the snapshot and diff manifest contracts. Compilation and schema access use those embedded contracts and do not load schema files from disk at runtime.

Phase A established two schema foundations: the common/profile pack contracts and the internal repo snapshot-manifest contract for filesystem-first immutable snapshot cases. The landed repo Phase B surface adds the internal diff-manifest contract for pure snapshot diffs, while the pack compiler Phase B surface added boundary taxonomy and component map topology schemas. Phase C extends the pack surface with score models, query packs, rule packs, and recipe packs, plus deterministic profile bundle resolution into a crate-internal `CompiledPackSet`.

The repo schema is intentionally narrow: it describes worktree snapshot manifests plus pure diff manifests over already-materialized snapshots, with recorded options, file inventory metadata, aggregate fingerprints, and snapshot stats for deterministic fixture or golden use. It does not imply git-backed snapshot sources, rename detection, public serialization APIs, or runtime wiring have landed.

The advanced schemas remain structural only in this seam. They do not imply score execution, tree-sitter query execution, rule runtime, recipe application, broader repo analysis, or app/runtime bootstrap.
