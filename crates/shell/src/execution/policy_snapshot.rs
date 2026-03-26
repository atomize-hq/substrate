use agent_api_types::{
    validate_net_allowed_for_enforcement, PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    WorldFsDenyEnforcementV3,
};
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use substrate_common::WorldFsMode;
use world_api::{ResourceLimits, WorldSpec};

const WORLD_FS_ENFORCEMENT_PLAN_B64_ENV: &str = "SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64";

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPolicySnapshot {
    pub(crate) snapshot: PolicySnapshotV3,
    pub(crate) snapshot_hash: String,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Debug, Clone)]
pub(crate) struct ResolvedWorldNetworkPolicy {
    pub(crate) snapshot: PolicySnapshotV3,
    pub(crate) isolate_network: bool,
    pub(crate) allowed_domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileStatKey {
    exists: bool,
    mtime: Option<SystemTime>,
    size: Option<u64>,
    sha256: Option<[u8; 32]>,
}

impl FileStatKey {
    fn for_path(path: &Path) -> Result<Self> {
        match fs::metadata(path) {
            Ok(meta) => Ok(Self {
                exists: true,
                mtime: meta.modified().ok(),
                size: Some(meta.len()),
                sha256: Some(sha256_file(path)?),
            }),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                exists: false,
                mtime: None,
                size: None,
                sha256: None,
            }),
            Err(err) => Err(err).with_context(|| format!("failed to stat {}", path.display())),
        }
    }
}

fn sha256_file(path: &Path) -> Result<[u8; 32]> {
    let mut file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 16 * 1024];
    loop {
        let read = file
            .read(&mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hasher.finalize().into())
}

#[derive(Debug, Clone)]
struct CacheEntry {
    workspace_root: Option<PathBuf>,
    global_path: PathBuf,
    workspace_path: Option<PathBuf>,
    global_stat: FileStatKey,
    workspace_stat: Option<FileStatKey>,
    snapshot: PolicySnapshotV3,
    snapshot_hash: String,
}

static POLICY_SNAPSHOT_CACHE: OnceLock<Mutex<Option<CacheEntry>>> = OnceLock::new();

pub(crate) fn invalidate_policy_snapshot_cache() {
    let cache = POLICY_SNAPSHOT_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = cache.lock() {
        *guard = None;
    }
}

pub(crate) fn resolve_policy_snapshot_for_cwd(cwd: &Path) -> Result<ResolvedPolicySnapshot> {
    let workspace_root = crate::execution::workspace::find_workspace_root(cwd);
    let global_path = crate::execution::policy_model::global_policy_path()?;
    let workspace_path = workspace_root
        .as_ref()
        .map(|root| crate::execution::policy_model::workspace_policy_path(root));

    let global_stat = FileStatKey::for_path(&global_path)?;
    let workspace_stat = workspace_path
        .as_ref()
        .map(|path| FileStatKey::for_path(path))
        .transpose()?;

    let cache = POLICY_SNAPSHOT_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock() {
        if let Some(entry) = guard.as_ref() {
            if entry.workspace_root == workspace_root
                && entry.global_path == global_path
                && entry.workspace_path == workspace_path
                && entry.global_stat == global_stat
                && entry.workspace_stat == workspace_stat
            {
                return Ok(ResolvedPolicySnapshot {
                    snapshot: entry.snapshot.clone(),
                    snapshot_hash: entry.snapshot_hash.clone(),
                });
            }
        }
    }

    let (policy, _) = substrate_broker::resolve_effective_policy_with_explain(cwd, false)
        .map_err(|err| crate::execution::config_model::user_error(err.to_string()))?;
    let snapshot = snapshot_from_policy(&policy)?;
    let snapshot_hash = compute_snapshot_hash(&snapshot)?;

    if let Ok(mut guard) = cache.lock() {
        *guard = Some(CacheEntry {
            workspace_root,
            global_path,
            workspace_path,
            global_stat,
            workspace_stat,
            snapshot: snapshot.clone(),
            snapshot_hash: snapshot_hash.clone(),
        });
    }

    Ok(ResolvedPolicySnapshot {
        snapshot,
        snapshot_hash,
    })
}

pub(crate) fn resolve_world_network_policy_for_cwd(
    cwd: &Path,
) -> Result<ResolvedWorldNetworkPolicy> {
    let snapshot = resolve_policy_snapshot_for_cwd(cwd)?.snapshot;
    let config = crate::execution::config_model::resolve_effective_config(
        cwd,
        &crate::execution::config_model::CliConfigOverrides::default(),
    )?;

    resolve_world_network_policy(snapshot, config.world.net.filter)
}

pub(crate) fn bootstrap_world_spec(project_dir: PathBuf, fs_mode: WorldFsMode) -> WorldSpec {
    WorldSpec {
        reuse_session: true,
        isolate_network: false,
        limits: ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: Vec::new(),
        project_dir,
        always_isolate: false,
        fs_mode,
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn world_spec_for_network_policy(
    project_dir: PathBuf,
    fs_mode: WorldFsMode,
    network_policy: &ResolvedWorldNetworkPolicy,
) -> WorldSpec {
    WorldSpec {
        reuse_session: true,
        isolate_network: network_policy.isolate_network,
        limits: ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: network_policy.allowed_domains.clone(),
        project_dir,
        always_isolate: false,
        fs_mode,
    }
}

fn resolve_world_network_policy(
    snapshot: PolicySnapshotV3,
    world_net_filter: bool,
) -> Result<ResolvedWorldNetworkPolicy> {
    let snapshot = snapshot
        .canonicalize()
        .map_err(|err| anyhow!("invalid PolicySnapshotV3: {err}"))?;

    let restrictive = snapshot.net_allowed.as_slice() != ["*"];
    let isolate_network = world_net_filter && restrictive;

    if isolate_network {
        validate_net_allowed_for_enforcement(&snapshot.net_allowed).map_err(|err| {
            crate::execution::config_model::user_error(format!(
                "invalid policy net_allowed for world netfilter enforcement: {err}"
            ))
        })?;
    }

    let allowed_domains = if isolate_network {
        snapshot.net_allowed.clone()
    } else {
        Vec::new()
    };

    Ok(ResolvedWorldNetworkPolicy {
        snapshot,
        isolate_network,
        allowed_domains,
    })
}

pub(crate) fn inject_world_fs_enforcement_plan_env(
    snapshot: &PolicySnapshotV3,
    env: &mut std::collections::HashMap<String, String>,
) -> Result<()> {
    if env.contains_key(WORLD_FS_ENFORCEMENT_PLAN_B64_ENV) {
        return Ok(());
    }

    let Some(encoded) = maybe_encode_world_fs_enforcement_plan_b64(snapshot)? else {
        return Ok(());
    };

    env.insert(WORLD_FS_ENFORCEMENT_PLAN_B64_ENV.to_string(), encoded);
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum EnforcementPlanModeV1 {
    Strict,
    BestEffort,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
struct EnforcementPlanV1 {
    version: u32,
    enforcement: EnforcementPlanModeV1,
    read_deny: Vec<String>,
    discover_deny: Vec<String>,
    write_deny: Vec<String>,
}

fn maybe_encode_world_fs_enforcement_plan_b64(
    snapshot: &PolicySnapshotV3,
) -> Result<Option<String>> {
    let canonical = snapshot
        .canonicalize()
        .map_err(|err| anyhow!("invalid PolicySnapshotV3: {err}"))?;

    let read_deny = canonical
        .world_fs
        .read
        .as_ref()
        .map(|d| d.deny_list.clone())
        .unwrap_or_default();
    let discover_deny = canonical
        .world_fs
        .discover
        .as_ref()
        .map(|d| d.deny_list.clone())
        .unwrap_or_else(|| read_deny.clone());
    let write_deny = canonical.world_fs.write.deny_list.clone();

    let any_deny = !read_deny.is_empty() || !discover_deny.is_empty() || !write_deny.is_empty();
    if !any_deny {
        return Ok(None);
    }

    let deny_enforcement = canonical
        .world_fs
        .deny_enforcement
        .ok_or_else(|| anyhow!("world_fs.deny_enforcement missing for deny_list configuration"))?;

    let enforcement = match deny_enforcement {
        WorldFsDenyEnforcementV3::Strict => EnforcementPlanModeV1::Strict,
        WorldFsDenyEnforcementV3::PreferStrict | WorldFsDenyEnforcementV3::Weak => {
            EnforcementPlanModeV1::BestEffort
        }
    };

    let plan = EnforcementPlanV1 {
        version: 1,
        enforcement,
        read_deny,
        discover_deny,
        write_deny,
    };

    let json_bytes = serde_json::to_vec(&plan).context("serialize enforcement plan JSON")?;
    Ok(Some(BASE64.encode(json_bytes)))
}

fn snapshot_from_policy(policy: &substrate_broker::Policy) -> Result<PolicySnapshotV3> {
    let dim = |dim: &substrate_broker::WorldFsDimensionPolicy| PolicySnapshotWorldFsDimensionV3 {
        allow_list: dim.allow_list.clone(),
        deny_list: dim.deny_list.clone(),
    };

    let read = policy
        .world_fs_read
        .as_ref()
        .map(dim)
        .unwrap_or(PolicySnapshotWorldFsDimensionV3 {
            allow_list: vec![".".to_string()],
            deny_list: Vec::new(),
        });

    let discover = policy
        .world_fs_discover
        .as_ref()
        .map(dim)
        .unwrap_or_else(|| read.clone());

    let write_lists =
        policy
            .world_fs_write
            .as_ref()
            .map(dim)
            .unwrap_or(PolicySnapshotWorldFsDimensionV3 {
                allow_list: vec![".".to_string()],
                deny_list: Vec::new(),
            });

    let deny_enforcement = policy.world_fs_deny_enforcement.map(|mode| match mode {
        substrate_broker::WorldFsDenyEnforcement::Strict => WorldFsDenyEnforcementV3::Strict,
        substrate_broker::WorldFsDenyEnforcement::PreferStrict => {
            WorldFsDenyEnforcementV3::PreferStrict
        }
        substrate_broker::WorldFsDenyEnforcement::Weak => WorldFsDenyEnforcementV3::Weak,
    });

    let snapshot = PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: policy.net_allowed.clone(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: policy.world_fs_host_visible,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 {
                routing: policy.world_fs_fail_closed_routing,
            },
            deny_enforcement,
            caged_required: policy.world_fs_caged_required,
            discover: Some(discover),
            read: Some(read),
            write: PolicySnapshotWorldFsWriteV3 {
                enabled: policy.world_fs_write_enabled,
                allow_list: write_lists.allow_list,
                deny_list: write_lists.deny_list,
            },
        },
    };

    let canonical = snapshot
        .canonicalize()
        .map_err(|err| anyhow!("invalid PolicySnapshotV3 derived from broker policy: {err}"))?;

    Ok(canonical)
}

fn compute_snapshot_hash(snapshot: &PolicySnapshotV3) -> Result<String> {
    let bytes = serde_json::to_vec(snapshot).context("serialize PolicySnapshotV3")?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_with_net_allowed(net_allowed: &[&str]) -> PolicySnapshotV3 {
        PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: net_allowed
                .iter()
                .map(|entry| (*entry).to_string())
                .collect(),
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
    fn world_network_policy_canonicalizes_snapshot_net_allowed() {
        let resolved = resolve_world_network_policy(
            snapshot_with_net_allowed(&[" Example.COM. ", "example.com", ""]),
            false,
        )
        .expect("resolve network policy");

        assert_eq!(
            resolved.snapshot.net_allowed,
            vec!["example.com".to_string()]
        );
        assert!(!resolved.isolate_network);
        assert!(resolved.allowed_domains.is_empty());
    }

    #[test]
    fn world_network_policy_keeps_allow_all_unisolated_when_gate_disabled() {
        let resolved = resolve_world_network_policy(snapshot_with_net_allowed(&["*"]), false)
            .expect("resolve");

        assert_eq!(resolved.snapshot.net_allowed, vec!["*".to_string()]);
        assert!(!resolved.isolate_network);
        assert!(resolved.allowed_domains.is_empty());
    }

    #[test]
    fn world_network_policy_keeps_allow_all_unisolated_when_gate_enabled() {
        let resolved =
            resolve_world_network_policy(snapshot_with_net_allowed(&["*"]), true).expect("resolve");

        assert_eq!(resolved.snapshot.net_allowed, vec!["*".to_string()]);
        assert!(!resolved.isolate_network);
        assert!(resolved.allowed_domains.is_empty());
    }

    #[test]
    fn world_network_policy_requests_isolation_for_restrictive_allowlist() {
        let resolved = resolve_world_network_policy(
            snapshot_with_net_allowed(&[" Example.COM. ", "api.example.com"]),
            true,
        )
        .expect("resolve");

        assert!(resolved.isolate_network);
        assert_eq!(
            resolved.allowed_domains,
            vec!["example.com".to_string(), "api.example.com".to_string()]
        );
    }

    #[test]
    fn world_network_policy_requests_deny_all_for_empty_allowlist() {
        let resolved =
            resolve_world_network_policy(snapshot_with_net_allowed(&[]), true).expect("resolve");

        assert!(resolved.isolate_network);
        assert!(resolved.allowed_domains.is_empty());
    }

    #[test]
    fn world_network_policy_rejects_invalid_wildcards_when_enforcement_requested() {
        let err = resolve_world_network_policy(snapshot_with_net_allowed(&["*.example.com"]), true)
            .expect_err("wildcard enforcement should fail");

        assert!(
            err.to_string()
                .contains("wildcard forms other than '*' are not supported"),
            "unexpected error: {err}"
        );
    }
}
