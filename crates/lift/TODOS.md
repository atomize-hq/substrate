# lift TODOs

- Track `QueryEngineKind` serde coupling in `src/lang/capabilities.rs`.
  If a new query engine variant is added in `src/pack/compiled/query_pack.rs`, update the manual
  serialize/deserialize mapping here and extend the capability coverage tests.
