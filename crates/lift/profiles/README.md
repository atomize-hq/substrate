# Profiles

This directory is part of the pack-authoring surface. Phase A landed the shipped profile compiler foundation, and Phase B keeps the shipped profile story deliberately small while letting profiles select compiler-internal topology packs.

Profiles are declarative inputs consumed through the crate-private `pack` compiler seam. The compiler can build one standalone profile from builtin, file-backed, or inline sources, validate it against the embedded common/profile schemas, and produce deterministic compiled output plus typed diagnostics.

The builtin profile name currently guaranteed is `builtin:generic/default`. That builtin is embedded in the compiler, and its topology slots now point at the builtin topology packs `builtin:generic/boundaries` and `builtin:generic/components`. Resolving those refs produces a crate-internal `ResolvedProfileTopology`; it does not yet imply repo walking, path classification, or runtime bootstrap.

This directory still does not carry bundled profile files. Later seams may add bundled Substrate profiles, migration shims, and broader pack-family authoring here. Those remain deferred beyond the current compiler-only topology work.
