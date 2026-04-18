# Lang fixtures

These fixtures cover the landed crate-private Seam 3 parse-manifest contract.

- `valid/` contains manifest-shaped examples that should validate against `schemas/lang/parse_manifest.v1.json` and deserialize into the current `ParseSet`-backed test wrapper.
- `invalid/` contains schema failures only. Runtime-only invariants such as request fingerprints, unit fingerprints, span ordering, and deterministic sort order are checked in Rust tests instead of the JSON Schema.

The manifest envelope is fixture-facing only. It adds `version` and `case` around the serialized `ParseSet` fields so fixture files can be named, validated, and evolved deterministically without making `lang` public.
