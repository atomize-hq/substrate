Phase A repo fixtures cover the filesystem-first snapshot substrate.

Static trees in this directory are used for:
- root detection semantics
- deterministic snapshot manifests
- explicit ignore behavior
- default inclusion of common cache/build directories

Some policy-sensitive cases are created dynamically inside the tests:
- non-UTF8 filesystem paths
- symlinks on platforms that support them
- large-file thresholds
- traversal-order and post-snapshot mutation cases
