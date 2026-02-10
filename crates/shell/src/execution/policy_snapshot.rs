use agent_api_types::{
    PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3,
    PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3, WorldFsDenyEnforcementV3,
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

const WORLD_FS_ENFORCEMENT_PLAN_B64_ENV: &str = "SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64";

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPolicySnapshot {
    pub(crate) snapshot: PolicySnapshotV3,
    pub(crate) snapshot_hash: String,
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
