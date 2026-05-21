# Lang fixtures

These fixtures cover the landed crate-private Seam 3 parse-manifest contract across the Phase A v1
and Phase B v2 manifest envelopes.

- `valid/` contains manifest-shaped examples for both schema versions. The original v1 fixtures
  stay unchanged, and the `_v2` fixtures add the Phase B `cache_hits` / `cache_misses` counters.
- `invalid/` contains schema failures only. Runtime-only invariants such as request fingerprints, unit fingerprints, span ordering, and deterministic sort order are checked in Rust tests instead of the JSON Schema.

The manifest envelope is fixture-facing only. It adds `version` and `case` around the serialized `ParseSet` fields so fixture files can be named, validated, and evolved deterministically without making `lang` public.
