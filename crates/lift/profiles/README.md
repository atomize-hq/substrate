# Profiles

This directory is part of the pack-authoring surface. Phase A landed the shipped profile compiler foundation, Phase B added topology selection, and Phase C extends profiles to select advanced pack families for deterministic bundle resolution.

Profiles are declarative inputs consumed through the crate-private `pack` compiler seam. The compiler can build one standalone profile from builtin, file-backed, or inline sources, validate it against the embedded common/profile schemas, and produce deterministic compiled output plus typed diagnostics.

The builtin profile name currently guaranteed is `builtin:generic/default`. That builtin is embedded in the compiler, and its topology slots now point at the builtin topology packs `builtin:generic/boundaries` and `builtin:generic/components`. Profiles may also name file- or builtin-backed score/query/rule/recipe packs. Resolving those refs now produces a crate-internal `CompiledPackSet`; it still does not imply repo walking, path classification, query execution, scoring, or runtime bootstrap.

This directory still does not carry bundled profile files. Later seams may add bundled Substrate profiles, migration shims, runtime bootstrap, and broader authoring UX here. Those remain deferred beyond the current compiler-only bundle-resolution work.
