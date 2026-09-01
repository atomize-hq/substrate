use crate::execution::agent_runtime::host_session_authority::{
    hash::canonical_sha256, schema::PolicyObjectHashInputV1,
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
use transport_api_types::{
    validate_net_allowed_for_enforcement, PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    WorldFsDenyEnforcementV3,
};
use world_api::{
    BackendPolicyInputV1, BackendPolicySnapshotV3, BackendPolicySnapshotWorldFsDimensionV3,
    BackendPolicySnapshotWorldFsFailClosedV3, BackendPolicySnapshotWorldFsV3,
    BackendPolicySnapshotWorldFsWriteV3, BackendWorldFsDenyEnforcementV3,
    BackendWorldNetworkRoutingV1, ResourceLimits, WorldReuseMode, WorldSpec,
};
const WORLD_FS_ENFORCEMENT_PLAN_B64_ENV: &str = "SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64";

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPolicySnapshot {
    pub(crate) snapshot: PolicySnapshotV3,
    pub(crate) snapshot_hash: String,
}

#[allow(
    dead_code,
    reason = "E1 exposes the authenticated narrowing entrypoint before dispatch adoption"
)]
#[derive(Debug, Clone)]
pub(crate) struct AuthenticatedDispatchPolicyNarrowingContextV1 {
    authority: ResolvedDispatchPolicyNarrowingAuthorityV1,
}

/// Values independently resolved from the live request, session, runtime, world, and policy
/// authorities. This carrier is intentionally distinct from the untrusted transport request.
#[allow(
    dead_code,
    reason = "E1 exposes the resolved authority input before dispatch adoption"
)]
#[derive(Debug, Clone)]
pub(crate) struct ResolvedDispatchPolicyNarrowingAuthorityV1 {
    pub(crate) request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) target_backend_id: String,
    pub(crate) target_world: transport_api_types::WorldBindingRefV1,
    pub(crate) applies_to: transport_api_types::DispatchCapabilitySubjectV1,
    pub(crate) parent_policy_ref: transport_api_types::PolicyRefV1,
    pub(crate) parent_policy: PolicyObjectHashInputV1,
    pub(crate) parent_allows_capability_narrowing: bool,
}

#[allow(
    dead_code,
    reason = "E1 exposes the authenticated narrowing entrypoint before dispatch adoption"
)]
impl AuthenticatedDispatchPolicyNarrowingContextV1 {
    pub(crate) fn from_resolved_authority(
        authority: ResolvedDispatchPolicyNarrowingAuthorityV1,
    ) -> Result<Self> {
        authority
            .parent_policy_ref
            .validate()
            .map_err(|error| anyhow!("invalid resolved parent policy reference: {error}"))?;
        canonical_sha256(&authority.parent_policy)
            .map_err(|error| anyhow!("invalid resolved parent policy identity: {error}"))?;
        Ok(Self { authority })
    }
}

#[allow(
    dead_code,
    reason = "E1 exposes the narrowed snapshot result before dispatch adoption"
)]
#[derive(Debug, Clone)]
pub(crate) struct ResolvedDispatchPolicySnapshotV1 {
    pub(crate) snapshot: PolicySnapshotV3,
    pub(crate) snapshot_hash: String,
    pub(crate) bindings: substrate_broker::DispatchPolicyNarrowingBindingsV1,
    pub(crate) reason: Option<String>,
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

#[allow(
    dead_code,
    reason = "A1.1e establishes the explicit-home entry point before A1.3 adopts it"
)]
pub(crate) fn resolve_policy_snapshot_for_bootstrap_home(
    cwd: &Path,
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
) -> Result<ResolvedPolicySnapshot> {
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        cwd,
        bootstrap_home,
    )?;
    let snapshot = snapshot_from_policy(&policy)?;
    let snapshot_hash = compute_snapshot_hash(&snapshot)?;
    Ok(ResolvedPolicySnapshot {
        snapshot,
        snapshot_hash,
    })
}

#[cfg_attr(
    not(unix),
    allow(
        dead_code,
        reason = "authenticated non-Unix projection remains R2-3-owned"
    )
)]
pub(crate) fn resolve_world_network_policy_for_bootstrap_home(
    cwd: &Path,
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
    effective_config: &crate::execution::config_model::SubstrateConfig,
) -> Result<ResolvedWorldNetworkPolicy> {
    let snapshot = resolve_policy_snapshot_for_bootstrap_home(cwd, bootstrap_home)?.snapshot;
    resolve_world_network_policy(snapshot, effective_config.world.net.filter)
}

pub(crate) fn resolve_world_network_policy_for_cwd(
    cwd: &Path,
) -> Result<ResolvedWorldNetworkPolicy> {
    let snapshot = resolve_policy_snapshot_for_cwd(cwd)?.snapshot;
    resolve_world_network_policy_for_snapshot(snapshot, cwd)
}

pub(crate) fn resolve_world_network_policy_for_snapshot(
    snapshot: PolicySnapshotV3,
    cwd: &Path,
) -> Result<ResolvedWorldNetworkPolicy> {
    let config = crate::execution::config_model::resolve_effective_config(
        cwd,
        &crate::execution::config_model::CliConfigOverrides::default(),
    )?;

    resolve_world_network_policy(snapshot, config.world.net.filter)
}

pub(crate) fn bootstrap_world_spec(project_dir: PathBuf, fs_mode: WorldFsMode) -> WorldSpec {
    match resolve_world_network_policy_for_cwd(&project_dir) {
        Ok(network_policy) => world_spec_for_network_policy(project_dir, fs_mode, &network_policy),
        Err(_) => WorldSpec {
            reuse_session: true,
            reuse_mode: WorldReuseMode::GenericCompatible,
            isolate_network: false,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: Vec::new(),
            project_dir,
            always_isolate: false,
            fs_mode,
            // Keep the widened carrier empty instead of inventing non-authoritative
            // backend policy inputs when broker resolution fails.
            backend_policy: None,
        },
    }
}

pub(crate) fn request_world_network_routing(
    network_policy: &ResolvedWorldNetworkPolicy,
) -> transport_api_types::WorldNetworkRoutingV1 {
    transport_api_types::WorldNetworkRoutingV1 {
        isolate_network: network_policy.isolate_network,
        allowed_domains: network_policy.allowed_domains.clone(),
    }
}

pub(crate) fn world_spec_for_network_policy(
    project_dir: PathBuf,
    fs_mode: WorldFsMode,
    network_policy: &ResolvedWorldNetworkPolicy,
) -> WorldSpec {
    WorldSpec {
        reuse_session: true,
        reuse_mode: WorldReuseMode::GenericCompatible,
        isolate_network: network_policy.isolate_network,
        limits: ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: network_policy.allowed_domains.clone(),
        project_dir,
        always_isolate: false,
        fs_mode,
        backend_policy: Some(backend_policy_input_for_network_policy(network_policy)),
    }
}

pub(crate) fn backend_policy_input_for_network_policy(
    network_policy: &ResolvedWorldNetworkPolicy,
) -> BackendPolicyInputV1 {
    BackendPolicyInputV1 {
        schema_version: 1,
        policy_snapshot: backend_policy_snapshot(&network_policy.snapshot),
        world_network: BackendWorldNetworkRoutingV1 {
            isolate_network: network_policy.isolate_network,
            allowed_domains: network_policy.allowed_domains.clone(),
        },
    }
}

fn resolve_world_network_policy(
    snapshot: PolicySnapshotV3,
    world_net_filter: bool,
) -> Result<ResolvedWorldNetworkPolicy> {
    let snapshot = snapshot
        .canonicalize()
        .map_err(|err| anyhow!("invalid PolicySnapshotV3: {err}"))?;

    if world_net_filter && snapshot.net_allowed.as_slice() != ["*"] {
        validate_net_allowed_for_enforcement(&snapshot.net_allowed).map_err(|err| {
            crate::execution::config_model::user_error(format!(
                "invalid policy net_allowed for world netfilter enforcement: {err}"
            ))
        })?;
    }

    let routing = snapshot
        .resolve_world_network_routing(world_net_filter)
        .map_err(|err| anyhow!("invalid PolicySnapshotV3: {err}"))?;

    Ok(ResolvedWorldNetworkPolicy {
        snapshot,
        isolate_network: routing.isolate_network,
        allowed_domains: routing.allowed_domains,
    })
}

fn backend_policy_snapshot(snapshot: &PolicySnapshotV3) -> BackendPolicySnapshotV3 {
    BackendPolicySnapshotV3 {
        schema_version: snapshot.schema_version,
        net_allowed: snapshot.net_allowed.clone(),
        world_fs: BackendPolicySnapshotWorldFsV3 {
            host_visible: snapshot.world_fs.host_visible,
            fail_closed: BackendPolicySnapshotWorldFsFailClosedV3 {
                routing: snapshot.world_fs.fail_closed.routing,
            },
            deny_enforcement: snapshot
                .world_fs
                .deny_enforcement
                .map(backend_world_fs_deny_enforcement),
            caged_required: snapshot.world_fs.caged_required,
            discover: snapshot
                .world_fs
                .discover
                .as_ref()
                .map(backend_world_fs_dimension),
            read: snapshot
                .world_fs
                .read
                .as_ref()
                .map(backend_world_fs_dimension),
            write: BackendPolicySnapshotWorldFsWriteV3 {
                enabled: snapshot.world_fs.write.enabled,
                allow_list: snapshot.world_fs.write.allow_list.clone(),
                deny_list: snapshot.world_fs.write.deny_list.clone(),
            },
        },
    }
}

fn backend_world_fs_dimension(
    dimension: &PolicySnapshotWorldFsDimensionV3,
) -> BackendPolicySnapshotWorldFsDimensionV3 {
    BackendPolicySnapshotWorldFsDimensionV3 {
        allow_list: dimension.allow_list.clone(),
        deny_list: dimension.deny_list.clone(),
    }
}

fn backend_world_fs_deny_enforcement(
    deny_enforcement: WorldFsDenyEnforcementV3,
) -> BackendWorldFsDenyEnforcementV3 {
    match deny_enforcement {
        WorldFsDenyEnforcementV3::Strict => BackendWorldFsDenyEnforcementV3::Strict,
        WorldFsDenyEnforcementV3::PreferStrict => BackendWorldFsDenyEnforcementV3::PreferStrict,
        WorldFsDenyEnforcementV3::Weak => BackendWorldFsDenyEnforcementV3::Weak,
    }
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

#[allow(
    dead_code,
    reason = "E1 exposes the broker-backed snapshot entrypoint before dispatch adoption"
)]
pub(crate) fn resolve_dispatch_narrowed_policy_snapshot(
    parent: &substrate_broker::Policy,
    carrier: &transport_api_types::DispatchPolicyNarrowingPatchV1,
    authenticated: &AuthenticatedDispatchPolicyNarrowingContextV1,
    world_root: &Path,
) -> Result<ResolvedDispatchPolicySnapshotV1> {
    carrier
        .validate()
        .map_err(|error| anyhow!("invalid dispatch narrowing carrier: {error}"))?;
    authenticate_parent_policy(parent, authenticated)?;
    let request = substrate_broker::DispatchWorldFsNarrowingRequestV1 {
        bindings: broker_bindings_from_carrier(carrier)?,
        world_fs: carrier
            .restricted_policy_patch
            .world_fs
            .as_ref()
            .map(broker_world_fs_patch_from_transport),
        reason: carrier.reason.clone(),
    };
    let authenticated = substrate_broker::AuthenticatedDispatchPolicyNarrowingContextV1 {
        bindings: broker_bindings_from_authenticated(authenticated)?,
    };
    let resolved = substrate_broker::EffectivePolicyResolver::resolve_dispatch_world_fs_narrowing(
        parent,
        &request,
        &authenticated,
        world_root,
    )?;
    let mut snapshot = snapshot_from_policy(&resolved.policy)?;
    if request
        .world_fs
        .as_ref()
        .is_some_and(|patch| !patch.is_empty())
        && !snapshot.world_fs.write.enabled
    {
        let read = snapshot
            .world_fs
            .read
            .as_ref()
            .ok_or_else(|| anyhow!("narrowed PolicySnapshotV3 is missing world_fs.read"))?;
        snapshot.world_fs.write.allow_list = read.allow_list.clone();
        snapshot = snapshot.canonicalize().map_err(|error| {
            anyhow!("invalid PolicySnapshotV3 after restricted write/read alignment: {error}")
        })?;
    }
    let snapshot_hash = compute_snapshot_hash(&snapshot)?;
    Ok(ResolvedDispatchPolicySnapshotV1 {
        snapshot,
        snapshot_hash,
        bindings: resolved.bindings,
        reason: resolved.reason,
    })
}

#[allow(dead_code)]
fn broker_bindings_from_carrier(
    carrier: &transport_api_types::DispatchPolicyNarrowingPatchV1,
) -> Result<substrate_broker::DispatchPolicyNarrowingBindingsV1> {
    broker_bindings(
        carrier.schema_version,
        &carrier.request_id,
        &carrier.orchestration_session_id,
        &carrier.caller_participant_id,
        &carrier.target_backend_id,
        &carrier.target_world,
        &carrier.applies_to,
        &carrier.parent_policy_ref,
        &carrier.parent_policy_revision,
    )
}

#[allow(dead_code)]
fn broker_bindings_from_authenticated(
    authenticated: &AuthenticatedDispatchPolicyNarrowingContextV1,
) -> Result<substrate_broker::DispatchPolicyNarrowingBindingsV1> {
    let authenticated = &authenticated.authority;
    broker_bindings(
        1,
        &authenticated.request_id,
        &authenticated.orchestration_session_id,
        &authenticated.caller_participant_id,
        &authenticated.target_backend_id,
        &authenticated.target_world,
        &authenticated.applies_to,
        &authenticated.parent_policy_ref,
        &authenticated.parent_policy.policy_revision,
    )
}

#[allow(dead_code)]
fn authenticate_parent_policy(
    parent: &substrate_broker::Policy,
    authenticated: &AuthenticatedDispatchPolicyNarrowingContextV1,
) -> Result<()> {
    let authority = &authenticated.authority;
    if authority.parent_policy_ref.schema_version != authority.parent_policy.schema_version {
        return Err(anyhow!(
            "resolved parent policy reference schema does not match policy identity"
        ));
    }

    let actual_snapshot = snapshot_from_policy(parent)?;
    let actual_snapshot_hash = compute_snapshot_hash(&actual_snapshot)?;
    if actual_snapshot_hash != authority.parent_policy.canonical_policy_snapshot_sha256 {
        return Err(anyhow!(
            "supplied parent policy does not match the resolved canonical parent snapshot"
        ));
    }
    if parent.agents_world_dispatch_allow_capability_narrowing
        != authority.parent_allows_capability_narrowing
    {
        return Err(anyhow!(
            "supplied parent policy does not match the resolved narrowing gate"
        ));
    }

    let expected_commitment = canonical_sha256(&authority.parent_policy)
        .map_err(|error| anyhow!("hash resolved parent policy identity: {error}"))?;
    match &authority.parent_policy_ref.commitment {
        transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 { digest_hex }
            if digest_hex == &expected_commitment =>
        {
            Ok(())
        }
        transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 { .. } => Err(anyhow!(
            "resolved parent policy reference commitment does not match policy identity"
        )),
        transport_api_types::OpaqueAuthorityCommitmentV1::StoreHmacSha256 { .. } => Err(anyhow!(
            "resolved parent policy reference must use canonical sha256"
        )),
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
fn broker_bindings(
    schema_version: u32,
    request_id: &str,
    orchestration_session_id: &str,
    caller_participant_id: &str,
    target_backend_id: &str,
    target_world: &transport_api_types::WorldBindingRefV1,
    applies_to: &transport_api_types::DispatchCapabilitySubjectV1,
    parent_policy_ref: &transport_api_types::PolicyRefV1,
    parent_policy_revision: &str,
) -> Result<substrate_broker::DispatchPolicyNarrowingBindingsV1> {
    let applies_to = match applies_to {
        transport_api_types::DispatchCapabilitySubjectV1::EphemeralTask => {
            substrate_broker::DispatchCapabilitySubjectV1::EphemeralTask
        }
        transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerSpawn => {
            substrate_broker::DispatchCapabilitySubjectV1::RetainedWorkerSpawn
        }
        transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerTurn {
            retained_participant_id,
        } => substrate_broker::DispatchCapabilitySubjectV1::RetainedWorkerTurn {
            retained_participant_id: retained_participant_id.clone(),
        },
        transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerFork {
            source_participant_id,
        } => substrate_broker::DispatchCapabilitySubjectV1::RetainedWorkerFork {
            source_participant_id: source_participant_id.clone(),
        },
    };
    let object_kind = match parent_policy_ref.object_kind {
        transport_api_types::AuthorityObjectKindV1::Policy => "policy",
        _ => "non_policy",
    };
    Ok(substrate_broker::DispatchPolicyNarrowingBindingsV1 {
        schema_version,
        request_id: request_id.to_string(),
        orchestration_session_id: orchestration_session_id.to_string(),
        caller_participant_id: caller_participant_id.to_string(),
        target_backend_id: target_backend_id.to_string(),
        target_world: substrate_broker::WorldBindingRefV1 {
            world_id: target_world.world_id.clone(),
            world_generation: target_world.world_generation,
        },
        applies_to,
        parent_policy_ref: substrate_broker::PolicyReferenceBindingV1 {
            ref_id: parent_policy_ref.ref_id.clone(),
            object_kind: object_kind.to_string(),
            schema_version: parent_policy_ref.schema_version,
            commitment: serde_json::to_string(&parent_policy_ref.commitment)
                .context("serialize exact parent policy commitment")?,
        },
        parent_policy_revision: parent_policy_revision.to_string(),
    })
}

#[allow(dead_code)]
fn broker_world_fs_patch_from_transport(
    patch: &transport_api_types::RestrictedWorldFsPatchV1,
) -> substrate_broker::RestrictedWorldFsPatchV1 {
    let dimension = |patch: &transport_api_types::RestrictedWorldFsDimensionPatchV1| {
        substrate_broker::RestrictedWorldFsDimensionPatchV1 {
            allow_list: patch.allow_list.clone(),
            deny_list: patch.deny_list.clone(),
        }
    };
    substrate_broker::RestrictedWorldFsPatchV1 {
        host_visible: patch.host_visible,
        fail_closed_routing: patch.fail_closed.as_ref().and_then(|value| value.routing),
        deny_enforcement: patch.deny_enforcement.map(|value| match value {
            WorldFsDenyEnforcementV3::Strict => substrate_broker::WorldFsDenyEnforcement::Strict,
            WorldFsDenyEnforcementV3::PreferStrict => {
                substrate_broker::WorldFsDenyEnforcement::PreferStrict
            }
            WorldFsDenyEnforcementV3::Weak => substrate_broker::WorldFsDenyEnforcement::Weak,
        }),
        caged_required: patch.caged_required,
        discover: patch.discover.as_ref().map(dimension),
        read: patch.read.as_ref().map(dimension),
        write: patch
            .write
            .as_ref()
            .map(|write| substrate_broker::RestrictedWorldFsWritePatchV1 {
                enabled: write.enabled,
                allow_list: write.allow_list.clone(),
                deny_list: write.deny_list.clone(),
            }),
    }
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

    const E1_PARENT_POLICY_REVISION: &str = "policy-revision-e1-0001";

    fn e1_policy_identity(parent: &substrate_broker::Policy) -> PolicyObjectHashInputV1 {
        let snapshot = snapshot_from_policy(parent).unwrap();
        PolicyObjectHashInputV1 {
            schema_version: 1,
            policy_revision: E1_PARENT_POLICY_REVISION.to_string(),
            canonical_policy_snapshot_sha256: compute_snapshot_hash(&snapshot).unwrap(),
        }
    }

    fn e1_policy_reference(parent: &substrate_broker::Policy) -> transport_api_types::PolicyRefV1 {
        let identity = e1_policy_identity(parent);
        transport_api_types::PolicyRefV1 {
            ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
            object_kind: transport_api_types::AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&identity).unwrap(),
            },
        }
    }

    fn e1_carrier(
        parent: &substrate_broker::Policy,
        world_fs: Option<transport_api_types::RestrictedWorldFsPatchV1>,
    ) -> transport_api_types::DispatchPolicyNarrowingPatchV1 {
        transport_api_types::DispatchPolicyNarrowingPatchV1 {
            schema_version: 1,
            request_id: "req_e1_0001".to_string(),
            orchestration_session_id: "session_e1_0001".to_string(),
            caller_participant_id: "participant_e1_caller".to_string(),
            target_backend_id: "codex".to_string(),
            target_world: transport_api_types::WorldBindingRefV1 {
                world_id: "world_e1_0001".to_string(),
                world_generation: 7,
            },
            applies_to: transport_api_types::DispatchCapabilitySubjectV1::EphemeralTask,
            parent_policy_ref: e1_policy_reference(parent),
            parent_policy_revision: E1_PARENT_POLICY_REVISION.to_string(),
            restricted_policy_patch: transport_api_types::RestrictedPolicyPatchV1 { world_fs },
            reason: Some("narrow to delegated file".to_string()),
        }
    }

    fn e1_authenticated_context(
        parent: &substrate_broker::Policy,
    ) -> AuthenticatedDispatchPolicyNarrowingContextV1 {
        AuthenticatedDispatchPolicyNarrowingContextV1::from_resolved_authority(
            ResolvedDispatchPolicyNarrowingAuthorityV1 {
                request_id: "req_e1_0001".to_string(),
                orchestration_session_id: "session_e1_0001".to_string(),
                caller_participant_id: "participant_e1_caller".to_string(),
                target_backend_id: "codex".to_string(),
                target_world: transport_api_types::WorldBindingRefV1 {
                    world_id: "world_e1_0001".to_string(),
                    world_generation: 7,
                },
                applies_to: transport_api_types::DispatchCapabilitySubjectV1::EphemeralTask,
                parent_policy_ref: e1_policy_reference(parent),
                parent_policy: e1_policy_identity(parent),
                parent_allows_capability_narrowing: parent
                    .agents_world_dispatch_allow_capability_narrowing,
            },
        )
        .expect("independently resolved authenticated context")
    }

    fn e1_parent_policy() -> substrate_broker::Policy {
        let mut policy = substrate_broker::Policy::default();
        policy.agents_world_dispatch_allow_capability_narrowing = true;
        policy.world_fs_host_visible = false;
        policy.world_fs_fail_closed_routing = true;
        policy.world_fs_write_enabled = false;
        policy.world_fs_deny_enforcement = None;
        policy.world_fs_read = Some(substrate_broker::WorldFsDimensionPolicy {
            allow_list: vec!["src".to_string()],
            deny_list: Vec::new(),
        });
        policy.world_fs_discover = policy.world_fs_read.clone();
        policy.world_fs_write = None;
        policy
    }

    #[test]
    fn e1_narrowed_snapshot_is_deterministic_schema_3_and_parent_is_unchanged() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        fs::write(root.path().join("src/lib.rs"), "lib").unwrap();
        let parent = e1_parent_policy();
        let original_allow = parent.world_fs_read.as_ref().unwrap().allow_list.clone();
        let carrier = e1_carrier(
            &parent,
            Some(transport_api_types::RestrictedWorldFsPatchV1 {
                read: Some(transport_api_types::RestrictedWorldFsDimensionPatchV1 {
                    allow_list: Some(vec!["src/lib.rs".to_string()]),
                    deny_list: None,
                }),
                ..Default::default()
            }),
        );
        let authenticated = e1_authenticated_context(&parent);

        let first = resolve_dispatch_narrowed_policy_snapshot(
            &parent,
            &carrier,
            &authenticated,
            root.path(),
        )
        .unwrap();
        let second = resolve_dispatch_narrowed_policy_snapshot(
            &parent,
            &carrier,
            &authenticated,
            root.path(),
        )
        .unwrap();
        assert_eq!(first.snapshot.schema_version, 3);
        assert_eq!(first.snapshot_hash, second.snapshot_hash);
        assert_eq!(
            serde_json::to_vec(&first.snapshot).unwrap(),
            serde_json::to_vec(&second.snapshot).unwrap()
        );
        assert_eq!(
            first.snapshot.world_fs.read.unwrap().allow_list,
            vec!["src/lib.rs"]
        );
        assert_eq!(
            parent.world_fs_read.as_ref().unwrap().allow_list,
            original_allow
        );
        assert_eq!(first.bindings.request_id, "req_e1_0001");
        assert_eq!(first.reason.as_deref(), Some("narrow to delegated file"));
    }

    #[test]
    fn e1_omitted_patch_preserves_parent_snapshot_identity_and_binding_mismatch_fails() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        let parent = e1_parent_policy();
        let carrier = e1_carrier(&parent, None);
        let authenticated = e1_authenticated_context(&parent);
        let resolved = resolve_dispatch_narrowed_policy_snapshot(
            &parent,
            &carrier,
            &authenticated,
            root.path(),
        )
        .unwrap();
        let parent_snapshot = snapshot_from_policy(&parent).unwrap();
        assert_eq!(
            resolved.snapshot_hash,
            compute_snapshot_hash(&parent_snapshot).unwrap()
        );

        let mut mismatch = authenticated;
        mismatch.authority.parent_policy.policy_revision.push('x');
        assert!(resolve_dispatch_narrowed_policy_snapshot(
            &parent,
            &carrier,
            &mismatch,
            root.path(),
        )
        .is_err());
    }

    #[test]
    fn e1_authenticated_parent_reference_rejects_substituted_policy_material() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        fs::write(root.path().join("src/lib.rs"), "lib").unwrap();
        let parent = e1_parent_policy();
        let carrier = e1_carrier(
            &parent,
            Some(transport_api_types::RestrictedWorldFsPatchV1 {
                read: Some(transport_api_types::RestrictedWorldFsDimensionPatchV1 {
                    allow_list: Some(vec!["src/lib.rs".to_string()]),
                    deny_list: None,
                }),
                ..Default::default()
            }),
        );
        let authenticated = e1_authenticated_context(&parent);

        let mut substituted = parent;
        substituted.world_fs_read.as_mut().unwrap().allow_list = vec![".".to_string()];
        substituted.world_fs_discover = substituted.world_fs_read.clone();
        assert!(resolve_dispatch_narrowed_policy_snapshot(
            &substituted,
            &carrier,
            &authenticated,
            root.path(),
        )
        .is_err());
    }

    #[test]
    fn e1_authenticated_parent_rejects_gate_only_policy_substitution() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        fs::write(root.path().join("src/lib.rs"), "lib").unwrap();
        let mut parent = e1_parent_policy();
        parent.agents_world_dispatch_allow_capability_narrowing = false;
        let carrier = e1_carrier(
            &parent,
            Some(transport_api_types::RestrictedWorldFsPatchV1 {
                read: Some(transport_api_types::RestrictedWorldFsDimensionPatchV1 {
                    allow_list: Some(vec!["src/lib.rs".to_string()]),
                    deny_list: None,
                }),
                ..Default::default()
            }),
        );
        let authenticated = e1_authenticated_context(&parent);

        let mut substituted = parent;
        substituted.agents_world_dispatch_allow_capability_narrowing = true;
        assert!(resolve_dispatch_narrowed_policy_snapshot(
            &substituted,
            &carrier,
            &authenticated,
            root.path(),
        )
        .is_err());
    }

    #[test]
    fn e1_shell_authentication_rejects_every_carrier_identity_substitution() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        fs::write(root.path().join("src/lib.rs"), "lib").unwrap();
        let parent = e1_parent_policy();
        let authenticated = e1_authenticated_context(&parent);
        let carrier = e1_carrier(&parent, None);

        for mutation in 0..10 {
            let mut substituted = carrier.clone();
            match mutation {
                0 => substituted.request_id.push('x'),
                1 => substituted.orchestration_session_id.push('x'),
                2 => substituted.caller_participant_id.push('x'),
                3 => substituted.target_backend_id.push('x'),
                4 => substituted.target_world.world_id.push('x'),
                5 => substituted.target_world.world_generation += 1,
                6 => {
                    substituted.applies_to =
                        transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerSpawn
                }
                7 => substituted.parent_policy_revision.push('x'),
                8 => {
                    substituted.parent_policy_ref.ref_id =
                        "ao_1123456789abcdef0123456789abcdef".to_string()
                }
                _ => {
                    substituted.parent_policy_ref.commitment =
                        transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                            digest_hex: "f".repeat(64),
                        }
                }
            }
            assert!(
                resolve_dispatch_narrowed_policy_snapshot(
                    &parent,
                    &substituted,
                    &authenticated,
                    root.path(),
                )
                .is_err(),
                "carrier identity substitution {mutation} was accepted"
            );
        }
    }

    #[cfg(all(unix, target_os = "linux"))]
    #[test]
    fn e1_linux_world_service_enforces_candidate_narrowed_snapshot() {
        use base64::Engine as _;
        use std::collections::HashMap;
        use std::os::unix::fs::symlink;
        use substrate_broker::{set_global_broker, BrokerHandle};
        use transport_api_types::ExecuteRequest;
        use world_service::WorldService;

        if unsafe { libc::geteuid() } != 0
            || !fs::read_to_string("/proc/filesystems")
                .map(|contents| contents.contains("overlay"))
                .unwrap_or(false)
        {
            eprintln!("E1 live proof requires root and overlayfs; run the compiled test as root");
            return;
        }

        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("delegated")).unwrap();
        fs::write(root.path().join("delegated/allowed.txt"), "ALLOWED\n").unwrap();
        fs::write(root.path().join("delegated/sibling.txt"), "SIBLING\n").unwrap();
        fs::create_dir(root.path().join("outside")).unwrap();
        fs::write(root.path().join("outside/outside.txt"), "OUTSIDE\n").unwrap();
        let external = tempfile::tempdir().unwrap();
        fs::write(external.path().join("secret.txt"), "ESCAPE\n").unwrap();
        symlink(external.path(), root.path().join("delegated/escape")).unwrap();

        let mut parent = e1_parent_policy();
        parent.world_fs_read.as_mut().unwrap().allow_list = vec![".".to_string()];
        parent.world_fs_discover = parent.world_fs_read.clone();
        let parent_snapshot_before = snapshot_from_policy(&parent).unwrap();
        let parent_hash_before = compute_snapshot_hash(&parent_snapshot_before).unwrap();
        let carrier = e1_carrier(
            &parent,
            Some(transport_api_types::RestrictedWorldFsPatchV1 {
                deny_enforcement: Some(WorldFsDenyEnforcementV3::Weak),
                read: Some(transport_api_types::RestrictedWorldFsDimensionPatchV1 {
                    allow_list: Some(vec!["delegated/allowed.txt".to_string()]),
                    deny_list: Some(vec![
                        "delegated/sibling.txt".to_string(),
                        "outside/**".to_string(),
                    ]),
                }),
                ..Default::default()
            }),
        );
        let authenticated = e1_authenticated_context(&parent);
        let candidate = resolve_dispatch_narrowed_policy_snapshot(
            &parent,
            &carrier,
            &authenticated,
            root.path(),
        )
        .unwrap();
        let repeated = resolve_dispatch_narrowed_policy_snapshot(
            &parent,
            &carrier,
            &authenticated,
            root.path(),
        )
        .unwrap();
        assert_eq!(candidate.snapshot_hash, repeated.snapshot_hash);
        assert_eq!(
            serde_json::to_vec(&candidate.snapshot).unwrap(),
            serde_json::to_vec(&repeated.snapshot).unwrap()
        );
        assert_ne!(candidate.snapshot_hash, parent_hash_before);

        let _ = set_global_broker(BrokerHandle::new());
        let service = WorldService::new().expect("real world service");
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let execute = |command: &str, snapshot: &PolicySnapshotV3| {
            let mut env = HashMap::new();
            env.insert("SUBSTRATE_WORLD_REQUIRE_WORLD".to_string(), "1".to_string());
            runtime
                .block_on(service.execute(ExecuteRequest {
                    profile: None,
                    cmd: format!("sh -lc 'cat {command}'"),
                    cwd: Some(root.path().display().to_string()),
                    env: Some(env),
                    pty: false,
                    agent_id: "e1-live-proof".to_string(),
                    budget: None,
                    policy_snapshot: snapshot.clone(),
                    shared_world: None,
                    world_network: None,
                    world_fs_mode: None,
                    acceptance_context: None,
                    member_dispatch: None,
                }))
                .expect("execute through real world service")
        };
        let output = |response: &transport_api_types::ExecuteResponse| {
            String::from_utf8_lossy(
                &base64::engine::general_purpose::STANDARD
                    .decode(&response.stdout_b64)
                    .unwrap(),
            )
            .into_owned()
        };
        let diagnostic = |response: &transport_api_types::ExecuteResponse| {
            format!(
                "stdout={} stderr={}",
                output(response),
                String::from_utf8_lossy(
                    &base64::engine::general_purpose::STANDARD
                        .decode(&response.stderr_b64)
                        .unwrap(),
                )
            )
        };

        let allowed = execute("./delegated/allowed.txt", &candidate.snapshot);
        assert_eq!(
            allowed.exit,
            0,
            "named file must remain readable: {}",
            diagnostic(&allowed)
        );
        assert!(output(&allowed).contains("ALLOWED"));
        for denied in [
            "./delegated/sibling.txt",
            "./outside/outside.txt",
            "./delegated/escape/secret.txt",
        ] {
            let response = execute(denied, &candidate.snapshot);
            assert_ne!(
                response.exit, 0,
                "narrowed snapshot unexpectedly read {denied}"
            );
        }

        let late_carrier = e1_carrier(
            &parent,
            Some(transport_api_types::RestrictedWorldFsPatchV1 {
                read: Some(transport_api_types::RestrictedWorldFsDimensionPatchV1 {
                    allow_list: Some(vec!["late/secret.txt".to_string()]),
                    deny_list: None,
                }),
                ..Default::default()
            }),
        );
        let late_authenticated = e1_authenticated_context(&parent);
        let late_candidate = resolve_dispatch_narrowed_policy_snapshot(
            &parent,
            &late_carrier,
            &late_authenticated,
            root.path(),
        )
        .expect("lexically contained nonexistent target");
        symlink(external.path(), root.path().join("late")).unwrap();
        let late_escape = execute("./late/secret.txt", &late_candidate.snapshot);
        assert_ne!(
            late_escape.exit, 0,
            "runtime must recheck a lexical target that becomes a symlink escape"
        );

        let parent_snapshot_after = snapshot_from_policy(&parent).unwrap();
        assert_eq!(parent_snapshot_after.schema_version, 3);
        assert_eq!(
            compute_snapshot_hash(&parent_snapshot_after).unwrap(),
            parent_hash_before,
            "child request must not alter parent snapshot identity"
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn explicit_snapshot_fixture(
        policy: &[u8],
    ) -> (
        tempfile::TempDir,
        crate::execution::agent_runtime::HostSessionAuthority,
    ) {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(home.join("policy.yaml"), policy).unwrap();
        fs::set_permissions(home.join("policy.yaml"), fs::Permissions::from_mode(0o600)).unwrap();
        let authority = crate::execution::agent_runtime::HostSessionAuthority::open(&home).unwrap();
        (parent, authority)
    }

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
    fn world_spec_for_network_policy_populates_backend_policy_from_authoritative_inputs() {
        let resolved = resolve_world_network_policy(
            snapshot_with_net_allowed(&[" Example.COM. ", "api.example.com"]),
            true,
        )
        .expect("resolve");

        let spec = world_spec_for_network_policy(
            std::path::PathBuf::from("/tmp/substrate-policy-snapshot"),
            WorldFsMode::Writable,
            &resolved,
        );
        let backend_policy = spec
            .backend_policy
            .as_ref()
            .expect("backend policy should be attached to WorldSpec");

        assert!(spec.isolate_network);
        assert_eq!(
            spec.allowed_domains,
            vec!["example.com".to_string(), "api.example.com".to_string()]
        );
        assert_eq!(
            backend_policy.policy_snapshot.net_allowed,
            resolved.snapshot.net_allowed
        );
        assert_eq!(
            backend_policy.world_network.allowed_domains,
            spec.allowed_domains
        );
        assert_eq!(
            backend_policy.world_network.isolate_network,
            spec.isolate_network
        );
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

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn explicit_snapshot_matches_canonical_broker_policy_projection() {
        let (parent, authority) = explicit_snapshot_fixture(b"id: explicit-snapshot\n");
        let bootstrap_home = authority.bootstrap_home();
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
            parent.path(),
            &bootstrap_home,
        )
        .unwrap();
        let resolved =
            resolve_policy_snapshot_for_bootstrap_home(parent.path(), &bootstrap_home).unwrap();
        let expected = snapshot_from_policy(&policy).unwrap();
        assert_eq!(
            serde_json::to_value(&resolved.snapshot).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        assert_eq!(
            resolved.snapshot_hash,
            compute_snapshot_hash(&expected).unwrap()
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn explicit_snapshot_contains_canonical_finalized_world_fs_defaults() {
        let (parent, authority) = explicit_snapshot_fixture(
            b"world_fs:\n  host_visible: false\n  write:\n    enabled: false\n  fail_closed:\n    routing: true\n",
        );
        let resolved =
            resolve_policy_snapshot_for_bootstrap_home(parent.path(), &authority.bootstrap_home())
                .unwrap();
        assert!(!resolved.snapshot.world_fs.host_visible);
        assert!(resolved.snapshot.world_fs.fail_closed.routing);
        assert!(resolved.snapshot.world_fs.discover.is_some());
        assert!(resolved.snapshot.world_fs.read.is_some());
        assert!(!resolved.snapshot.world_fs.write.enabled);
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn explicit_world_network_policy_uses_bootstrap_home_and_explicit_config() {
        let (parent, authority) =
            explicit_snapshot_fixture(b"id: selected-policy\nnet_allowed: [selected.example]\n");
        let mut config = crate::execution::config_model::SubstrateConfig::default();
        config.world.net.filter = true;

        let resolved = resolve_world_network_policy_for_bootstrap_home(
            parent.path(),
            &authority.bootstrap_home(),
            &config,
        )
        .expect("resolve explicit world network policy");

        assert_eq!(
            resolved.snapshot.net_allowed,
            vec!["selected.example".to_string()]
        );
        assert!(resolved.isolate_network);
        assert_eq!(
            resolved.allowed_domains,
            vec!["selected.example".to_string()]
        );
    }
}
