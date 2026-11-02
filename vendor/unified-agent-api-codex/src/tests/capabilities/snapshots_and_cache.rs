use super::*;

#[test]
fn capability_snapshots_serialize_to_json_and_toml() {
    let snapshot = sample_capabilities_snapshot();

    let json = serialize_capabilities_snapshot(&snapshot, CapabilitySnapshotFormat::Json)
        .expect("serialize json");
    let parsed_json = deserialize_capabilities_snapshot(&json, CapabilitySnapshotFormat::Json)
        .expect("parse json");
    assert_eq!(parsed_json, snapshot);

    let toml = serialize_capabilities_snapshot(&snapshot, CapabilitySnapshotFormat::Toml)
        .expect("serialize toml");
    let parsed_toml = deserialize_capabilities_snapshot(&toml, CapabilitySnapshotFormat::Toml)
        .expect("parse toml");
    assert_eq!(parsed_toml, snapshot);
}

#[test]
fn capability_snapshots_and_overrides_round_trip_via_files() {
    let snapshot = sample_capabilities_snapshot();
    let overrides = sample_capability_overrides();
    let temp = tempfile::tempdir().unwrap();

    let snapshot_path = temp.path().join("capabilities.toml");
    write_capabilities_snapshot(&snapshot_path, &snapshot, None).unwrap();
    let loaded_snapshot = read_capabilities_snapshot(&snapshot_path, None).unwrap();
    assert_eq!(loaded_snapshot, snapshot);

    let overrides_path = temp.path().join("overrides.json");
    write_capability_overrides(&overrides_path, &overrides, None).unwrap();
    let loaded_overrides = read_capability_overrides(&overrides_path, None).unwrap();
    assert_eq!(loaded_overrides, overrides);
}

#[test]
fn capability_snapshot_match_checks_fingerprint() {
    let temp = tempfile::tempdir().unwrap();
    let script = "#!/bin/bash\necho ok";
    let binary = write_fake_codex(temp.path(), script);
    let cache_key = capability_cache_key(&binary);
    let fingerprint = current_fingerprint(&cache_key);

    let snapshot = CodexCapabilities {
        cache_key: cache_key.clone(),
        fingerprint: fingerprint.clone(),
        version: None,
        features: CodexFeatureFlags::default(),
        probe_plan: CapabilityProbePlan::default(),
        collected_at: SystemTime::UNIX_EPOCH,
    };

    assert!(capability_snapshot_matches_binary(&snapshot, &binary));
    let mut missing_fingerprint = snapshot.clone();
    missing_fingerprint.fingerprint = None;
    assert!(!capability_snapshot_matches_binary(
        &missing_fingerprint,
        &binary
    ));

    std_fs::write(&binary, "#!/bin/bash\necho changed").unwrap();
    let mut perms = std_fs::metadata(&binary).unwrap().permissions();
    perms.set_mode(0o755);
    std_fs::set_permissions(&binary, perms).unwrap();

    assert!(!capability_snapshot_matches_binary(&snapshot, &binary));
}

#[test]
fn capability_cache_entries_exposes_cache_state() {
    let _guard = env_guard();
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let binary = write_fake_codex(temp.path(), "#!/bin/bash\necho ok");
    let cache_key = capability_cache_key(&binary);
    let fingerprint = current_fingerprint(&cache_key);

    let snapshot = CodexCapabilities {
        cache_key: cache_key.clone(),
        fingerprint: fingerprint.clone(),
        version: Some(version::parse_version_output("codex 0.0.1")),
        features: CodexFeatureFlags {
            supports_features_list: true,
            supports_output_schema: true,
            supports_add_dir: false,
            supports_mcp_login: false,
        },
        probe_plan: CapabilityProbePlan {
            steps: vec![CapabilityProbeStep::VersionFlag],
        },
        collected_at: SystemTime::UNIX_EPOCH,
    };

    update_capability_cache(snapshot.clone());

    let entries = capability_cache_entries();
    assert!(entries.iter().any(|entry| entry.cache_key == cache_key));

    let fetched = capability_cache_entry(&binary).expect("expected cache entry");
    assert_eq!(fetched.cache_key, cache_key);
    assert!(clear_capability_cache_entry(&binary));
    assert!(capability_cache_entry(&binary).is_none());
    assert!(capability_cache_entries().is_empty());
    clear_capability_cache();
}

#[test]
fn capability_ttl_decision_reuses_fresh_snapshot() {
    let collected_at = SystemTime::UNIX_EPOCH + Duration::from_secs(10);
    let snapshot = capability_snapshot_with_metadata(
        collected_at,
        Some(BinaryFingerprint {
            canonical_path: Some(PathBuf::from("/tmp/codex")),
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
            len: Some(123),
        }),
    );

    let decision = capability_cache_ttl_decision(
        Some(&snapshot),
        Duration::from_secs(300),
        SystemTime::UNIX_EPOCH + Duration::from_secs(100),
    );
    assert!(!decision.should_probe);
    assert_eq!(decision.policy, CapabilityCachePolicy::PreferCache);
}

#[test]
fn capability_ttl_decision_refreshes_after_ttl_with_fingerprint() {
    let collected_at = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
    let snapshot = capability_snapshot_with_metadata(
        collected_at,
        Some(BinaryFingerprint {
            canonical_path: Some(PathBuf::from("/tmp/codex")),
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
            len: Some(321),
        }),
    );

    let decision = capability_cache_ttl_decision(
        Some(&snapshot),
        Duration::from_secs(5),
        SystemTime::UNIX_EPOCH + Duration::from_secs(10),
    );
    assert!(decision.should_probe);
    assert_eq!(decision.policy, CapabilityCachePolicy::Refresh);
}

#[test]
fn capability_ttl_decision_bypasses_when_metadata_missing() {
    let collected_at = SystemTime::UNIX_EPOCH + Duration::from_secs(2);
    let snapshot = capability_snapshot_with_metadata(collected_at, None);

    let decision = capability_cache_ttl_decision(
        Some(&snapshot),
        Duration::from_secs(5),
        SystemTime::UNIX_EPOCH + Duration::from_secs(10),
    );
    assert!(decision.should_probe);
    assert_eq!(decision.policy, CapabilityCachePolicy::Bypass);
}
