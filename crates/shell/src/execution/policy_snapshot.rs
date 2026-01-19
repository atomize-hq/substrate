use anyhow::{Context, Result};
use agent_api_types::{
    PolicySnapshotLimitsV1, PolicySnapshotV1, PolicySnapshotWorldFsIsolationV1,
    PolicySnapshotWorldFsV1,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPolicySnapshot {
    pub(crate) snapshot: PolicySnapshotV1,
    pub(crate) snapshot_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileStatKey {
    exists: bool,
    mtime: Option<SystemTime>,
    size: Option<u64>,
}

impl FileStatKey {
    fn for_path(path: &Path) -> Result<Self> {
        match fs::metadata(path) {
            Ok(meta) => Ok(Self {
                exists: true,
                mtime: meta.modified().ok(),
                size: Some(meta.len()),
            }),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                exists: false,
                mtime: None,
                size: None,
            }),
            Err(err) => Err(err).with_context(|| format!("failed to stat {}", path.display())),
        }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    workspace_root: Option<PathBuf>,
    global_path: PathBuf,
    workspace_path: Option<PathBuf>,
    global_stat: FileStatKey,
    workspace_stat: Option<FileStatKey>,
    snapshot: PolicySnapshotV1,
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
    let snapshot = snapshot_from_policy(&policy);
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

fn snapshot_from_policy(policy: &substrate_broker::Policy) -> PolicySnapshotV1 {
    let isolation = match policy.world_fs_isolation {
        substrate_broker::WorldFsIsolation::Workspace => PolicySnapshotWorldFsIsolationV1::Workspace,
        substrate_broker::WorldFsIsolation::Full => PolicySnapshotWorldFsIsolationV1::Full,
    };

    PolicySnapshotV1 {
        schema_version: 1,
        world_fs: PolicySnapshotWorldFsV1 {
            mode: policy.world_fs_mode,
            isolation,
            require_world: policy.world_fs_require_world,
            read_allowlist: policy.fs_read.clone(),
            write_allowlist: policy.fs_write.clone(),
        },
        net_allowed: policy.net_allowed.clone(),
        limits: PolicySnapshotLimitsV1 {
            max_memory_mb: policy.limits.max_memory_mb,
            max_cpu_percent: policy.limits.max_cpu_percent,
            max_runtime_ms: policy.limits.max_runtime_ms,
            max_egress_bytes: policy.limits.max_egress_bytes,
        },
    }
}

fn compute_snapshot_hash(snapshot: &PolicySnapshotV1) -> Result<String> {
    let bytes = serde_json::to_vec(snapshot).context("serialize PolicySnapshotV1")?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

