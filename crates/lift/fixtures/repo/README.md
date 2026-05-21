Repo fixtures cover the landed crate-private repo seam: Phase A snapshot materialization plus Phase B pure diff fixtures over already-materialized snapshots.

Static trees in this directory are used for:
- root detection semantics
- deterministic snapshot manifests
- paired-snapshot pure diff manifests under `fixtures/repo/diff/**`
- explicit ignore behavior
- default inclusion of common cache/build directories
- canonical typed well-known exclude directory examples under `fixtures/repo/trees/well_known_excludes`

Some policy-sensitive cases are created dynamically inside the tests:
- non-UTF8 filesystem paths
- symlinks on platforms that support them
- large-file thresholds
- traversal-order and post-snapshot mutation cases
