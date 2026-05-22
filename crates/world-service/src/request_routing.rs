use substrate_common::WorldFsMode;
use transport_api_types::{
    canonicalize_net_allowed, validate_net_allowed_for_enforcement, PolicySnapshotV3,
    WorldNetworkRoutingV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedWorldNetworkRouting {
    pub(crate) isolate_network: bool,
    pub(crate) allowed_domains: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedSnapshotRouting {
    pub(crate) snapshot: PolicySnapshotV3,
    pub(crate) fs_mode: WorldFsMode,
    pub(crate) isolation_full: bool,
    pub(crate) world_network: ResolvedWorldNetworkRouting,
}

pub(crate) fn resolve_snapshot_routing(
    policy_snapshot: &PolicySnapshotV3,
    world_network: Option<&WorldNetworkRoutingV1>,
) -> Result<ResolvedSnapshotRouting, String> {
    let snapshot = policy_snapshot.canonicalize()?;
    let fs_mode = if snapshot.world_fs.write.enabled {
        WorldFsMode::Writable
    } else {
        WorldFsMode::ReadOnly
    };
    let isolation_full = !snapshot.world_fs.host_visible;
    let world_network = resolve_world_network(&snapshot, world_network)?;

    Ok(ResolvedSnapshotRouting {
        snapshot,
        fs_mode,
        isolation_full,
        world_network,
    })
}

fn resolve_world_network(
    snapshot: &PolicySnapshotV3,
    world_network: Option<&WorldNetworkRoutingV1>,
) -> Result<ResolvedWorldNetworkRouting, String> {
    let Some(world_network) = world_network else {
        return Ok(ResolvedWorldNetworkRouting {
            isolate_network: false,
            allowed_domains: Vec::new(),
        });
    };

    if !world_network.isolate_network {
        return Ok(ResolvedWorldNetworkRouting {
            isolate_network: false,
            allowed_domains: Vec::new(),
        });
    }

    if snapshot.net_allowed.as_slice() == ["*"] {
        return Err(
            "world_network.isolate_network=true requires restrictive policy_snapshot.net_allowed"
                .to_string(),
        );
    }

    validate_net_allowed_for_enforcement(&snapshot.net_allowed).map_err(|err| {
        format!("invalid policy_snapshot.net_allowed for network enforcement: {err}")
    })?;

    let allowed_domains = canonicalize_net_allowed(&world_network.allowed_domains);
    validate_net_allowed_for_enforcement(&allowed_domains)
        .map_err(|err| format!("invalid world_network.allowed_domains: {err}"))?;

    if allowed_domains != snapshot.net_allowed {
        return Err(
            "world_network.allowed_domains must match canonical policy_snapshot.net_allowed when isolate_network=true"
                .to_string(),
        );
    }

    Ok(ResolvedWorldNetworkRouting {
        isolate_network: true,
        allowed_domains,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use transport_api_types::{
        PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3,
        PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    };

    fn snapshot_with_net_allowed(entries: &[&str]) -> PolicySnapshotV3 {
        PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: entries.iter().map(|entry| entry.to_string()).collect(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        }
    }

    #[test]
    fn restrictive_allowlist_requires_matching_world_network() {
        let resolved = resolve_snapshot_routing(
            &snapshot_with_net_allowed(&[" Example.com. "]),
            Some(&WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["example.com".to_string()],
            }),
        )
        .expect("resolve");

        assert!(resolved.world_network.isolate_network);
        assert_eq!(
            resolved.snapshot.net_allowed,
            vec!["example.com".to_string()]
        );
        assert_eq!(
            resolved.world_network.allowed_domains,
            vec!["example.com".to_string()]
        );
    }

    #[test]
    fn allow_all_without_world_network_is_compatibility_mode() {
        let resolved =
            resolve_snapshot_routing(&snapshot_with_net_allowed(&["*"]), None).expect("resolve");

        assert!(!resolved.world_network.isolate_network);
        assert!(resolved.world_network.allowed_domains.is_empty());
    }

    #[test]
    fn deny_all_is_valid_when_host_requests_isolation() {
        let resolved = resolve_snapshot_routing(
            &snapshot_with_net_allowed(&[]),
            Some(&WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: Vec::new(),
            }),
        )
        .expect("resolve");

        assert!(resolved.world_network.isolate_network);
        assert!(resolved.world_network.allowed_domains.is_empty());
    }

    #[test]
    fn invalid_snapshot_entries_fail_when_isolation_requested() {
        let err = resolve_snapshot_routing(
            &snapshot_with_net_allowed(&["*.example.com"]),
            Some(&WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["*.example.com".to_string()],
            }),
        )
        .expect_err("invalid snapshot");

        assert!(err.contains("invalid policy_snapshot.net_allowed"));
    }

    #[test]
    fn mismatched_world_network_is_rejected() {
        let err = resolve_snapshot_routing(
            &snapshot_with_net_allowed(&["example.com"]),
            Some(&WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["other.example".to_string()],
            }),
        )
        .expect_err("mismatch");

        assert!(err.contains("world_network.allowed_domains must match"));
    }

    #[test]
    fn allow_all_cannot_request_isolation() {
        let err = resolve_snapshot_routing(
            &snapshot_with_net_allowed(&["*"]),
            Some(&WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["*".to_string()],
            }),
        )
        .expect_err("allow-all isolation rejected");

        assert!(err.contains("requires restrictive policy_snapshot.net_allowed"));
    }
}
