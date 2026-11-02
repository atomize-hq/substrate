use super::*;

#[tokio::test]
async fn probe_reprobes_when_metadata_missing() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let binary = temp.path().join("missing_codex");
    let cache_key = capability_cache_key(&binary);

    {
        let mut cache = capability_cache().lock().unwrap();
        cache.insert(
            cache_key.clone(),
            CodexCapabilities {
                cache_key: cache_key.clone(),
                fingerprint: None,
                version: Some(version::parse_version_output("codex 9.9.9")),
                features: CodexFeatureFlags {
                    supports_features_list: true,
                    supports_output_schema: true,
                    supports_add_dir: true,
                    supports_mcp_login: true,
                },
                probe_plan: CapabilityProbePlan::default(),
                collected_at: SystemTime::UNIX_EPOCH,
            },
        );
    }

    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(1))
        .build();

    let capabilities = client.probe_capabilities().await;
    assert!(!capabilities.features.supports_output_schema);
    assert!(capabilities
        .probe_plan
        .steps
        .contains(&CapabilityProbeStep::VersionFlag));

    clear_capability_cache();
}

#[tokio::test]
async fn probe_refresh_policy_forces_new_snapshot() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("probe.log");
    let script = format!(
        r#"#!/bin/bash
echo "$@" >> "{log}"
if [[ "$1" == "--version" ]]; then
  echo "codex 1.0.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{{"features":["output_schema"]}}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "output_schema"
fi
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .build();

    let first = client.probe_capabilities().await;
    assert!(first.features.supports_output_schema);
    let first_lines = std_fs::read_to_string(&log_path).unwrap().lines().count();
    assert!(first_lines >= 2);

    let refreshed = client
        .probe_capabilities_with_policy(CapabilityCachePolicy::Refresh)
        .await;
    assert!(refreshed.features.supports_output_schema);
    let refreshed_lines = std_fs::read_to_string(&log_path).unwrap().lines().count();
    assert!(
        refreshed_lines > first_lines,
        "expected refresh policy to re-run probes"
    );
    clear_capability_cache();
}

#[tokio::test]
async fn probe_bypass_policy_skips_cache_writes() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let script = r#"#!/bin/bash
if [[ "$1" == "--version" ]]; then
  echo "codex 1.0.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{"features":["output_schema"]}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "output_schema"
fi
"#;
    let binary = write_fake_codex(temp.path(), script);

    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .build();

    let capabilities = client
        .probe_capabilities_with_policy(CapabilityCachePolicy::Bypass)
        .await;
    assert!(capabilities.features.supports_output_schema);
    assert!(capability_cache_entry(&binary).is_none());
    clear_capability_cache();
}
