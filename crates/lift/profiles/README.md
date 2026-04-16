# Profiles

This directory is part of the pack-authoring surface, but Phase A keeps the shipped profile story deliberately small.

Profiles are declarative inputs consumed through the crate-private `pack` compiler seam. In Phase A, the compiler can build one standalone profile from builtin, file-backed, or inline sources, validate it against the embedded common/profile schemas, and produce deterministic compiled output plus typed diagnostics.

The only builtin profile guaranteed in Phase A is `builtin:generic/default`. That builtin is embedded in the compiler; this directory does not yet carry bundled profile files.

Later seams may add bundled Substrate profiles, migration shims, and broader pack-family authoring here. Those are explicitly deferred until topology, advanced pack families, and runtime consumers land.
