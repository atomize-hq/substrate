use agent_api::{list_runtime_support, resolve_runtime_support, RuntimeSupportRecord};

#[test]
fn codex_runtime_support_metadata_is_available_without_backend_features() {
    let resolved = resolve_runtime_support("codex", "x86_64-unknown-linux-musl")
        .expect("resolve codex tuple without backend feature");
    assert_eq!(
        resolved,
        RuntimeSupportRecord {
            runtime_family: "codex".to_string(),
            target_triple: "x86_64-unknown-linux-musl".to_string(),
            version: "0.125.0".to_string(),
        }
    );

    let listed = list_runtime_support("codex").expect("list codex tuples without backend feature");
    assert!(listed.contains(&resolved), "{listed:?}");
}
