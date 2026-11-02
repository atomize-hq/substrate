use super::*;

fn capabilities_with_version(raw_version: &str) -> CodexCapabilities {
    CodexCapabilities {
        cache_key: CapabilityCacheKey {
            binary_path: PathBuf::from("codex"),
        },
        fingerprint: None,
        version: Some(version::parse_version_output(raw_version)),
        features: CodexFeatureFlags::default(),
        probe_plan: CapabilityProbePlan::default(),
        collected_at: SystemTime::now(),
    }
}

fn capabilities_without_version() -> CodexCapabilities {
    CodexCapabilities {
        cache_key: CapabilityCacheKey {
            binary_path: PathBuf::from("codex"),
        },
        fingerprint: None,
        version: None,
        features: CodexFeatureFlags::default(),
        probe_plan: CapabilityProbePlan::default(),
        collected_at: SystemTime::now(),
    }
}

fn capabilities_with_feature_flags(features: CodexFeatureFlags) -> CodexCapabilities {
    CodexCapabilities {
        cache_key: CapabilityCacheKey {
            binary_path: PathBuf::from("codex"),
        },
        fingerprint: None,
        version: None,
        features,
        probe_plan: CapabilityProbePlan::default(),
        collected_at: SystemTime::now(),
    }
}

fn sample_capabilities_snapshot() -> CodexCapabilities {
    CodexCapabilities {
        cache_key: CapabilityCacheKey {
            binary_path: PathBuf::from("/tmp/codex"),
        },
        fingerprint: Some(BinaryFingerprint {
            canonical_path: Some(PathBuf::from("/tmp/codex")),
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(5)),
            len: Some(1234),
        }),
        version: Some(CodexVersionInfo {
            raw: "codex 3.4.5-beta (commit cafe)".to_string(),
            semantic: Some((3, 4, 5)),
            commit: Some("cafe".to_string()),
            channel: CodexReleaseChannel::Beta,
        }),
        features: CodexFeatureFlags {
            supports_features_list: true,
            supports_output_schema: true,
            supports_add_dir: false,
            supports_mcp_login: true,
        },
        probe_plan: CapabilityProbePlan {
            steps: vec![
                CapabilityProbeStep::VersionFlag,
                CapabilityProbeStep::FeaturesListJson,
                CapabilityProbeStep::ManualOverride,
            ],
        },
        collected_at: SystemTime::UNIX_EPOCH + Duration::from_secs(10),
    }
}

fn sample_capability_overrides() -> CapabilityOverrides {
    CapabilityOverrides {
        snapshot: Some(sample_capabilities_snapshot()),
        version: Some(version::parse_version_output("codex 9.9.9-nightly")),
        features: CapabilityFeatureOverrides {
            supports_features_list: Some(true),
            supports_output_schema: Some(true),
            supports_add_dir: Some(true),
            supports_mcp_login: None,
        },
    }
}

fn capability_snapshot_with_metadata(
    collected_at: SystemTime,
    fingerprint: Option<BinaryFingerprint>,
) -> CodexCapabilities {
    CodexCapabilities {
        cache_key: CapabilityCacheKey {
            binary_path: PathBuf::from("/tmp/codex"),
        },
        fingerprint,
        version: None,
        features: CodexFeatureFlags::default(),
        probe_plan: CapabilityProbePlan::default(),
        collected_at,
    }
}

#[cfg(unix)]
mod exec_and_login;
#[cfg(unix)]
mod feature_parsing_and_guards;
#[cfg(unix)]
mod overrides_and_probe;
#[cfg(unix)]
mod probe_cache_policy;
#[cfg(unix)]
mod probe_working_dir;
#[cfg(unix)]
mod snapshots_and_cache;
#[cfg(unix)]
mod version_and_advisory;
#[cfg(windows)]
mod windows_path_overrides;
