use super::*;

struct RestoreEnvVar {
    key: &'static str,
    original: Option<OsString>,
}

impl RestoreEnvVar {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            original: env::var_os(key),
        }
    }

    fn set(&self, value: impl Into<OsString>) {
        env::set_var(self.key, value.into());
    }

    fn clear(&self) {
        env::remove_var(self.key);
    }
}

impl Drop for RestoreEnvVar {
    fn drop(&mut self) {
        match self.original.take() {
            Some(value) => env::set_var(self.key, value),
            None => env::remove_var(self.key),
        }
    }
}

fn write_env_sensitive_add_dir_probe(dir: &std::path::Path) -> PathBuf {
    write_fake_codex(
        dir,
        r#"#!/bin/bash
if [[ "$1" == "--version" ]]; then
  echo "codex 1.0.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  if [[ "${CODEX_ENABLE_ADD_DIR_PROBE:-}" == "1" ]]; then
    echo '{"features":["add_dir"]}'
  else
    echo '{"features":[]}'
  fi
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  if [[ "${CODEX_ENABLE_ADD_DIR_PROBE:-}" == "1" ]]; then
    echo "add_dir"
  else
    echo ""
  fi
elif [[ "$1" == "--help" ]]; then
  if [[ "${CODEX_ENABLE_ADD_DIR_PROBE:-}" == "1" ]]; then
    echo "Usage: codex --add-dir"
  else
    echo "Usage: codex exec"
  fi
fi
"#,
    )
}

#[tokio::test]
async fn capability_snapshot_short_circuits_probes() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("probe.log");
    let script = format!(
        r#"#!/bin/bash
echo "$@" >> "{log}"
exit 99
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);

    let snapshot = CodexCapabilities {
        cache_key: CapabilityCacheKey {
            binary_path: PathBuf::from("codex"),
        },
        fingerprint: None,
        version: Some(version::parse_version_output("codex 9.9.9-custom")),
        features: CodexFeatureFlags {
            supports_features_list: true,
            supports_output_schema: true,
            supports_add_dir: false,
            supports_mcp_login: true,
        },
        probe_plan: CapabilityProbePlan::default(),
        collected_at: SystemTime::now(),
    };

    let client = CodexClient::builder()
        .binary(&binary)
        .capability_snapshot(snapshot)
        .timeout(Duration::from_secs(5))
        .build();

    let capabilities = client.probe_capabilities().await;
    assert_eq!(
        capabilities.cache_key.binary_path,
        std_fs::canonicalize(&binary).unwrap()
    );
    assert!(capabilities.fingerprint.is_some());
    assert!(capabilities.features.supports_output_schema);
    assert!(capabilities.features.supports_mcp_login);
    assert_eq!(
        capabilities.version.as_ref().and_then(|v| v.semantic),
        Some((9, 9, 9))
    );
    assert!(capabilities
        .probe_plan
        .steps
        .contains(&CapabilityProbeStep::ManualOverride));
    assert!(!log_path.exists());
}

#[tokio::test]
async fn capability_feature_overrides_apply_to_cached_entries() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let script = r#"#!/bin/bash
if [[ "$1" == "--version" ]]; then
  echo "codex 1.0.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{"features":[]}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "features list"
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex exec"
fi
"#;
    let binary = write_fake_codex(temp.path(), script);

    let base_client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .build();
    let base_capabilities = base_client.probe_capabilities().await;
    assert!(base_capabilities.features.supports_features_list);
    assert!(!base_capabilities.features.supports_output_schema);

    let overrides = CapabilityFeatureOverrides::enabling(CodexFeatureFlags {
        supports_features_list: false,
        supports_output_schema: true,
        supports_add_dir: false,
        supports_mcp_login: true,
    });

    let client = CodexClient::builder()
        .binary(&binary)
        .capability_feature_overrides(overrides)
        .timeout(Duration::from_secs(5))
        .build();

    let capabilities = client.probe_capabilities().await;
    assert!(capabilities.features.supports_output_schema);
    assert!(capabilities.features.supports_mcp_login);
    assert!(capabilities
        .probe_plan
        .steps
        .contains(&CapabilityProbeStep::ManualOverride));
    assert_eq!(
        capabilities.guard_output_schema().support,
        CapabilitySupport::Supported
    );
}

#[tokio::test]
async fn capability_version_override_replaces_probe_version() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let script = r#"#!/bin/bash
if [[ "$1" == "--version" ]]; then
  echo "codex 0.1.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{"features":["add_dir"]}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "add_dir"
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex add-dir"
fi
	"#;
    let binary = write_fake_codex(temp.path(), script);
    let version_override = version::parse_version_output("codex 9.9.9-nightly (commit beefcafe)");

    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .capability_version_override(version_override)
        .build();

    let capabilities = client.probe_capabilities().await;
    assert_eq!(
        capabilities.version.as_ref().and_then(|v| v.semantic),
        Some((9, 9, 9))
    );
    assert!(matches!(
        capabilities.version.as_ref().map(|v| v.channel),
        Some(CodexReleaseChannel::Nightly)
    ));
    assert!(capabilities.features.supports_add_dir);
    assert!(capabilities
        .probe_plan
        .steps
        .contains(&CapabilityProbeStep::ManualOverride));
    assert_eq!(
        capabilities.guard_add_dir().support,
        CapabilitySupport::Supported
    );
}

#[tokio::test]
async fn capability_probe_with_env_overrides_uses_effective_env() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let binary = write_env_sensitive_add_dir_probe(temp.path());
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .build();

    let base = client.probe_capabilities().await;
    assert!(!base.features.supports_add_dir);

    let env_overrides =
        BTreeMap::from([("CODEX_ENABLE_ADD_DIR_PROBE".to_string(), "1".to_string())]);
    let env_sensitive = client
        .probe_capabilities_with_env_overrides(&env_overrides)
        .await;
    assert!(env_sensitive.features.supports_add_dir);
}

#[tokio::test]
async fn capability_probe_with_env_overrides_uses_effective_path() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let ambient = tempfile::tempdir().unwrap();
    let override_dir = tempfile::tempdir().unwrap();
    let path_restore = RestoreEnvVar::capture("PATH");
    let binary_restore = RestoreEnvVar::capture("CODEX_BINARY");

    let ambient_binary = write_env_sensitive_add_dir_probe(ambient.path());
    let override_binary = write_env_sensitive_add_dir_probe(override_dir.path());
    binary_restore.clear();
    path_restore.set(ambient.path().as_os_str().to_os_string());

    let client = CodexClient::builder()
        .timeout(Duration::from_secs(5))
        .build();

    let base = client.probe_capabilities().await;
    assert!(!base.features.supports_add_dir);
    assert_eq!(
        base.cache_key.binary_path,
        std_fs::canonicalize(&ambient_binary).unwrap()
    );

    let env_overrides = BTreeMap::from([
        (
            "PATH".to_string(),
            override_dir.path().to_string_lossy().to_string(),
        ),
        ("CODEX_ENABLE_ADD_DIR_PROBE".to_string(), "1".to_string()),
    ]);
    let env_sensitive = client
        .probe_capabilities_with_env_overrides(&env_overrides)
        .await;
    assert!(env_sensitive.features.supports_add_dir);
    assert_eq!(
        env_sensitive.cache_key.binary_path,
        std_fs::canonicalize(&override_binary).unwrap()
    );
}

#[tokio::test]
async fn capability_probe_with_env_overrides_bypasses_cache_reads_and_writes() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let binary = write_env_sensitive_add_dir_probe(temp.path());
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .build();

    let cached_base = client.probe_capabilities().await;
    assert!(!cached_base.features.supports_add_dir);
    assert!(
        !capability_cache_entry(&binary)
            .expect("plain probe should populate the cache")
            .features
            .supports_add_dir
    );

    let env_overrides =
        BTreeMap::from([("CODEX_ENABLE_ADD_DIR_PROBE".to_string(), "1".to_string())]);
    let env_sensitive = client
        .probe_capabilities_with_env_overrides(&env_overrides)
        .await;
    assert!(
        env_sensitive.features.supports_add_dir,
        "env-aware probes must bypass stale cached snapshots"
    );

    assert!(
        !capability_cache_entry(&binary)
            .expect("env-aware probe must not replace the shared cache entry")
            .features
            .supports_add_dir
    );
    assert!(
        !client.probe_capabilities().await.features.supports_add_dir,
        "plain probes should continue to see the cached binary-path snapshot"
    );
}
