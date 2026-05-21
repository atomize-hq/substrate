# Profiles

This directory is part of the pack-authoring surface. Phase A landed the shipped profile compiler foundation, Phase B added topology selection, Phase C extends profiles to select advanced pack families for deterministic bundle resolution, and Phase D adds the narrow runtime bootstrap handoff over that compiled bundle.

Profiles are declarative inputs consumed through the crate-private `pack` compiler seam. The compiler can build one standalone profile from builtin, file-backed, or inline sources, validate it against the embedded common/profile schemas, and produce deterministic compiled output plus typed diagnostics.

The builtin profile name currently guaranteed is `builtin:generic/default`. That builtin is embedded in the compiler, and its topology slots now point at the builtin topology packs `builtin:generic/boundaries` and `builtin:generic/components`. Profiles may also name file- or builtin-backed score/query/rule/recipe packs. Resolving those refs now produces a crate-internal `CompiledPackSet`; that still does not imply repo walking, path classification, query execution, scoring, or rule/recipe execution. Phase D does add a narrow crate-internal activation boundary in `app::runtime`, where `bootstrap_profile(...)` wraps the resolved bundle as `ProfileBootstrap` for later runtime consumers.

This directory still does not carry bundled profile files. Later seams may add bundled Substrate profiles, migration shims, and broader authoring UX here. Those remain deferred beyond the current compiler-plus-bootstrap boundary. The current runtime surface is bootstrap ingress only, not a public registry, dispatch layer, or broader app container.
