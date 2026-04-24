//! Core service implementation for world agent.

#[cfg(target_os = "linux")]
use agent_api_types::ExecuteStreamFrame;
#[cfg(not(target_os = "linux"))]
use agent_api_types::GatewayStatusV1;
#[cfg(any(target_os = "linux", test))]
use agent_api_types::PendingDiffBucketV1;
#[cfg(target_os = "linux")]
use agent_api_types::WorldFsEntryTypeV1;
use agent_api_types::{
    Budget, ExecuteCancelRequestV1, ExecuteCancelResponseV1, ExecuteRequest, ExecuteResponse,
    GatewayLifecycleRequestV1, GatewayLifecycleResponseV1, PendingDiffClearRequestV1,
    PendingDiffClearResponseV1, PendingDiffReconcileRequestV1, PendingDiffReconcileResponseV1,
    PendingDiffRecordV1, PendingDiffRequestV1, ProcessTelemetry, WorldFsReadRequestV1,
    WorldFsReadResponseV1, WorldNetworkRoutingV1,
};
#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::{anyhow, Result};
use axum::response::Response;
#[cfg(target_os = "linux")]
use axum::{
    body::{boxed, Bytes, StreamBody},
    http::StatusCode,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
#[cfg(target_os = "linux")]
use chrono::SecondsFormat;
#[cfg(target_os = "linux")]
use futures_util::StreamExt;
#[cfg(any(target_os = "linux", test))]
use sha2::{Digest, Sha256};
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::collections::HashSet;
#[cfg(target_os = "linux")]
use std::convert::Infallible;
#[cfg(target_os = "linux")]
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
#[cfg(target_os = "linux")]
use substrate_common::agent_events::{AgentEvent, AgentEventKind};
use substrate_common::{WorldFsMode, WorldRootMode};
#[cfg(target_os = "linux")]
use tokio::task;
#[cfg(target_os = "linux")]
use tokio_stream::wrappers::UnboundedReceiverStream;
#[cfg(target_os = "linux")]
use world::stream::{install_stream_sink, StreamKind, StreamSink};
use world_api::{WorldBackend, WorldHandle, WorldSpec};

use crate::enforcement_plan;
#[cfg(target_os = "linux")]
use crate::gateway_runtime::{
    resolve_gateway_backend_binding, unavailable_response as gateway_unavailable_response_impl,
    GatewayControlSettings, GatewayRuntimeFailure, GatewayRuntimeManager,
    GatewayRuntimeStartContext,
};
use crate::request_routing::resolve_snapshot_routing;

pub(crate) const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
pub(crate) const ANCHOR_PATH_ENV: &str = "SUBSTRATE_ANCHOR_PATH";
pub(crate) const WORLD_PROJECT_DIR_OVERRIDE_ENV: &str = "SUBSTRATE_WORLD_PROJECT_DIR";
#[cfg(target_os = "linux")]
pub(crate) const WORLD_FS_MODE_ENV: &str = "SUBSTRATE_WORLD_FS_MODE";
pub(crate) const WORLD_FS_ISOLATION_ENV: &str = "SUBSTRATE_WORLD_FS_ISOLATION";
pub(crate) const WORLD_FS_WRITE_ALLOWLIST_ENV: &str = "SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST";
pub(crate) const WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV: &str =
    "SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST";
pub(crate) const WORLD_FS_LANDLOCK_WRITE_ALLOWLIST_ENV: &str =
    "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST";
pub(crate) const WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST_ENV: &str =
    "SUBSTRATE_WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST";
pub(crate) const LANDLOCK_HELPER_SRC_ENV: &str = "SUBSTRATE_LANDLOCK_HELPER_SRC";

const CARGO_BIN_EXE_WORLD_AGENT_ENV: &str = "CARGO_BIN_EXE_world-agent";
const CARGO_BIN_EXE_WORLD_AGENT_ALT_ENV: &str = "CARGO_BIN_EXE_world_agent";

const NETFILTER_ENABLE_REQUIRED_TEXT: &str =
    "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules";
const NETFILTER_NFT_FAILURE_TEXT: &str = "nft command failed";
const NETFILTER_RESOLUTION_FAILURE_TEXT: &str = "failed to resolve allowed domain `";
const NETFILTER_NO_ADDRESS_TEXT: &str = "resolved to no addresses";
const NETFILTER_CGROUP_ATTACH_TEXT: &str = ": cgroup attach failed:";
const NETFILTER_CGROUP_HELPER_REFUSAL_TEXT: &str =
    "refused isolated execution before command start";
const NETFILTER_CGROUP_FORCE_DIRECT_TEXT: &str =
    "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT is unsupported when isolate_network=true because cgroup attach is not guaranteed";

fn resolve_landlock_helper_src_from_exe(exe: &std::path::Path) -> Option<std::path::PathBuf> {
    let is_world_agent_exe = exe.file_name().is_some_and(|name| {
        name == std::ffi::OsStr::new("world-agent")
            || name == std::ffi::OsStr::new("substrate-world-agent")
    });
    if is_world_agent_exe && exe.is_file() {
        return Some(exe.to_owned());
    }

    // Common case in `cargo test`: current_exe is the test harness under `target/*/deps/...`.
    // Walk up a few parents and look for the workspace-built `world-agent` binary.
    let mut dir = exe.parent();
    for _ in 0..8 {
        let Some(d) = dir else { break };
        let candidate = d.join("world-agent");
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }

    None
}

fn resolve_landlock_helper_src() -> Option<String> {
    fn accept_candidate(candidate: &str) -> Option<std::path::PathBuf> {
        let candidate = candidate.trim();
        if candidate.is_empty() {
            return None;
        }
        let path = std::path::Path::new(candidate);
        if path.is_file() {
            return Some(path.to_owned());
        }
        None
    }

    fn same_file(a: &std::path::Path, b: &std::path::Path) -> bool {
        if a == b {
            return true;
        }
        match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    let exe = std::env::current_exe().ok();

    // Prefer explicitly provided helper src (mainly for tests/tools), but avoid accepting the
    // current process when it is a `cargo test` harness (it will swallow the helper argv and exit 0).
    if let Ok(candidate) = std::env::var(LANDLOCK_HELPER_SRC_ENV) {
        if let Some(path) = accept_candidate(&candidate) {
            let is_test_harness_self = exe.as_ref().is_some_and(|exe| {
                exe.file_name().is_none_or(|n| n != "world-agent") && same_file(&path, exe)
            });
            if !is_test_harness_self {
                return Some(path.display().to_string());
            }
        }
    }

    // Best-effort: if a runtime env var exists, honor it.
    for key in [
        CARGO_BIN_EXE_WORLD_AGENT_ENV,
        CARGO_BIN_EXE_WORLD_AGENT_ALT_ENV,
    ] {
        if let Ok(candidate) = std::env::var(key) {
            if let Some(path) = accept_candidate(&candidate) {
                return Some(path.display().to_string());
            }
        }
    }

    let exe = exe?;
    resolve_landlock_helper_src_from_exe(&exe).map(|p| p.display().to_string())
}

/// Main service running inside the world.
#[derive(Clone)]
pub struct WorldAgentService {
    backend: Arc<dyn WorldBackend>,
    #[cfg(target_os = "linux")]
    linux_backend: Arc<world::LinuxLocalBackend>,
    #[cfg(target_os = "linux")]
    gateway_runtime: Arc<GatewayRuntimeManager>,
    #[cfg(target_os = "linux")]
    pending_diff_origin: Arc<RwLock<HashMap<String, PendingDiffOriginTracker>>>,
    #[allow(dead_code)]
    worlds: Arc<RwLock<HashMap<String, WorldHandle>>>,
    budgets: Arc<RwLock<HashMap<String, AgentBudgetTracker>>>,
    last_policy_resolution_mode: Arc<AtomicU8>,
    last_netfilter_requested: Arc<AtomicU8>,
    last_netfilter_failure_reason: Arc<RwLock<Option<String>>>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingDiffOrigin {
    NonPty,
    Pty,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Default)]
struct PendingDiffOriginTracker {
    last_seen_paths: HashSet<String>,
    origin_by_path: HashMap<String, PendingDiffOrigin>,
}

pub struct AgentBudgetTracker {
    #[allow(dead_code)]
    agent_id: String,
    execs_remaining: std::sync::atomic::AtomicU32,
    #[allow(dead_code)]
    runtime_remaining_ms: std::sync::atomic::AtomicU64,
    #[allow(dead_code)]
    egress_remaining: std::sync::atomic::AtomicU64,
}

impl AgentBudgetTracker {
    pub fn new(agent_id: &str, budget: Budget) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            execs_remaining: std::sync::atomic::AtomicU32::new(budget.max_execs.unwrap_or(1000)),
            runtime_remaining_ms: std::sync::atomic::AtomicU64::new(
                budget.max_runtime_ms.unwrap_or(300_000),
            ),
            egress_remaining: std::sync::atomic::AtomicU64::new(
                budget.max_egress_bytes.unwrap_or(100_000_000),
            ),
        }
    }

    pub fn can_execute(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.execs_remaining.load(Ordering::SeqCst) > 0
    }

    pub fn decrement_exec(&self) {
        use std::sync::atomic::Ordering;
        self.execs_remaining.fetch_sub(1, Ordering::SeqCst);
    }
}

impl WorldAgentService {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let linux_backend = Arc::new(world::LinuxLocalBackend::new());
            let backend: Arc<dyn WorldBackend> = linux_backend.clone();

            Ok(Self {
                backend,
                linux_backend,
                gateway_runtime: Arc::new(GatewayRuntimeManager::new()),
                pending_diff_origin: Arc::new(RwLock::new(HashMap::new())),
                worlds: Arc::new(RwLock::new(HashMap::new())),
                budgets: Arc::new(RwLock::new(HashMap::new())),
                last_policy_resolution_mode: Arc::new(AtomicU8::new(0)),
                last_netfilter_requested: Arc::new(AtomicU8::new(0)),
                last_netfilter_failure_reason: Arc::new(RwLock::new(None)),
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let backend = Self::create_backend()?;

            Ok(Self {
                backend,
                worlds: Arc::new(RwLock::new(HashMap::new())),
                budgets: Arc::new(RwLock::new(HashMap::new())),
                last_policy_resolution_mode: Arc::new(AtomicU8::new(0)),
                last_netfilter_requested: Arc::new(AtomicU8::new(0)),
                last_netfilter_failure_reason: Arc::new(RwLock::new(None)),
            })
        }
    }

    /// Ensure a session world (thin wrapper over backend)
    #[cfg(target_os = "linux")]
    pub fn ensure_session_world(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        self.backend.ensure_session(spec)
    }

    /// Ensure the session world exists and return the merged overlay root for PTY sessions.
    #[cfg(target_os = "linux")]
    pub fn ensure_session_overlay_root(&self, spec: &WorldSpec) -> Result<(WorldHandle, PathBuf)> {
        let world = self.linux_backend.ensure_session(spec)?;
        let merged = self.linux_backend.ensure_overlay_root(&world)?;
        Ok((world, merged))
    }

    #[cfg(target_os = "linux")]
    pub fn refresh_session_network_filter(&self, world: &WorldHandle) -> Result<()> {
        self.linux_backend.refresh_network_filter(world)
    }

    #[cfg(target_os = "linux")]
    pub fn session_cgroup_path(&self, world: &WorldHandle) -> Result<PathBuf> {
        self.linux_backend.cgroup_path(world)
    }

    #[cfg(any(target_os = "linux", test))]
    fn normalize_pending_diff_bucket(diff: &substrate_common::FsDiff) -> PendingDiffBucketV1 {
        fn normalize(path: &std::path::Path) -> String {
            path.to_string_lossy().replace('\\', "/")
        }

        let mut writes: Vec<String> = diff.writes.iter().map(|p| normalize(p)).collect();
        let mut mods: Vec<String> = diff.mods.iter().map(|p| normalize(p)).collect();
        let mut deletes: Vec<String> = diff.deletes.iter().map(|p| normalize(p)).collect();
        writes.sort();
        mods.sort();
        deletes.sort();
        writes.dedup();
        mods.dedup();
        deletes.dedup();

        PendingDiffBucketV1 {
            writes,
            mods,
            deletes,
        }
    }

    #[cfg(any(target_os = "linux", test))]
    fn pending_diff_id_for_snapshot(snapshot: &PendingDiffBucketV1) -> String {
        // `diff_id` is used as a conditional clear token (Clear/ack semantics).
        //
        // It must remain stable across harmless reclassification of a path between `writes` and
        // `mods` (e.g., when a sync to the host causes the overlay lower layer to start reporting
        // the path as "existing", flipping upper->lower classification without any new world
        // mutation). To avoid false "diff_id mismatch" failures, compute the id from:
        // - updates = (writes ∪ mods) as a set
        // - deletes as a set
        //
        // The UX buckets are still preserved in the record; only the clear token is stabilized.
        let mut updates: Vec<&str> =
            Vec::with_capacity(snapshot.writes.len() + snapshot.mods.len());
        for item in &snapshot.writes {
            updates.push(item.as_str());
        }
        for item in &snapshot.mods {
            updates.push(item.as_str());
        }
        updates.sort();
        updates.dedup();

        let mut deletes: Vec<&str> = snapshot.deletes.iter().map(|s| s.as_str()).collect();
        deletes.sort();
        deletes.dedup();

        let mut hasher = Sha256::new();
        hasher.update(b"updates\0");
        for item in updates {
            hasher.update(item.as_bytes());
            hasher.update(b"\n");
        }
        hasher.update(b"deletes\0");
        for item in deletes {
            hasher.update(item.as_bytes());
            hasher.update(b"\n");
        }

        let digest = hasher.finalize();
        let mut hex = String::with_capacity(digest.len() * 2);
        for b in digest {
            use std::fmt::Write as _;
            let _ = write!(&mut hex, "{:02x}", b);
        }
        hex
    }

    #[cfg(any(target_os = "linux", test))]
    fn pending_diff_id_for_diff(diff: &substrate_common::FsDiff) -> String {
        let snapshot = Self::normalize_pending_diff_bucket(diff);
        Self::pending_diff_id_for_snapshot(&snapshot)
    }

    #[cfg(target_os = "linux")]
    fn note_pending_diff_origin_for_world(
        &self,
        world_id: &str,
        origin: PendingDiffOrigin,
        snapshot: &PendingDiffBucketV1,
    ) {
        let mut current_paths: HashSet<String> = HashSet::new();
        current_paths.extend(snapshot.writes.iter().cloned());
        current_paths.extend(snapshot.mods.iter().cloned());
        current_paths.extend(snapshot.deletes.iter().cloned());

        let mut guard = self
            .pending_diff_origin
            .write()
            .expect("pending diff origin tracker lock poisoned");

        if current_paths.is_empty() {
            guard.remove(world_id);
            return;
        }

        let tracker = guard.entry(world_id.to_string()).or_default();

        for removed in tracker.last_seen_paths.difference(&current_paths) {
            tracker.origin_by_path.remove(removed);
        }
        for added in current_paths.difference(&tracker.last_seen_paths) {
            tracker.origin_by_path.insert(added.clone(), origin);
        }

        tracker.last_seen_paths = current_paths;
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn note_pty_pending_diff(&self, world_id: &str) {
        let world = WorldHandle {
            id: world_id.to_string(),
        };
        let (_started_at, diff) = match self.linux_backend.pending_diff(&world) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, world_id = world_id, "failed to snapshot pending diff after PTY command");
                return;
            }
        };

        let snapshot = Self::normalize_pending_diff_bucket(&diff);
        self.note_pending_diff_origin_for_world(world_id, PendingDiffOrigin::Pty, &snapshot);
    }

    #[cfg(not(target_os = "linux"))]
    fn create_backend() -> Result<Arc<dyn WorldBackend>> {
        anyhow::bail!("World agent only supported on Linux inside VMs")
    }

    /// Execute a command with budget tracking.
    pub async fn execute(&self, req: ExecuteRequest) -> Result<ExecuteResponse> {
        // Validate agent_id
        if req.agent_id.is_empty() {
            anyhow::bail!("agent_id is required for API calls");
        }

        let always_isolate = should_always_isolate(&req);

        // Apply and track budget
        if let Some(budget) = req.budget.clone() {
            {
                let mut budgets = self.budgets.write().unwrap();
                let tracker = budgets
                    .entry(req.agent_id.clone())
                    .or_insert_with(|| AgentBudgetTracker::new(&req.agent_id, budget));

                // Check budget before execution
                if !tracker.can_execute() {
                    anyhow::bail!("Budget exceeded: max executions reached");
                }

                tracker.decrement_exec();
            }
        }

        let cwd = req
            .cwd
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let env_ref = req.env.as_ref();
        let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;
        let (policy_resolution_mode, policy_inputs) = resolve_policy_inputs(
            &req.policy_snapshot,
            req.world_network.as_ref(),
            &cwd,
            &project_dir,
        )?;

        let PolicyInputs {
            fs_mode,
            isolation_full,
            isolate_network,
            allowed_domains,
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        } = policy_inputs;
        self.record_doctor_request_context(policy_resolution_mode, isolate_network);

        let host_visible = !isolation_full;
        let empty_env: HashMap<String, String> = HashMap::new();
        let guard_env = req.env.as_ref().unwrap_or(&empty_env);
        if let Some(deny) =
            crate::world_exec_guard::check_command(&req.cmd, &cwd, guard_env, host_visible)
        {
            let span_id = format!("spn_{}", uuid::Uuid::now_v7());
            let message = crate::world_exec_guard::deny_message(&deny);
            return Ok(ExecuteResponse {
                exit: 5,
                span_id,
                stdout_b64: BASE64.encode(b""),
                stderr_b64: BASE64.encode(message.as_bytes()),
                scopes_used: Vec::new(),
                fs_diff: None,
                process_telemetry: ProcessTelemetry::default(),
            });
        }

        // Create world spec from request
        let spec = build_world_spec(
            project_dir,
            always_isolate,
            fs_mode,
            isolate_network,
            allowed_domains,
        );

        // Ensure world exists
        let world = match self.backend.ensure_session(&spec) {
            Ok(w) => w,
            Err(e) => {
                self.record_last_netfilter_failure_for_error(isolate_network, &e);
                tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                return Err(anyhow::anyhow!("Failed to ensure session world"));
            }
        };

        // Prepare execution request
        let mut env_map = req.env.unwrap_or_default();
        env_map.remove(WORLD_PROJECT_DIR_OVERRIDE_ENV);
        // Snapshot isolation should not rely on callers setting legacy env toggles.
        let isolation = if isolation_full { "full" } else { "workspace" };
        env_map.insert(WORLD_FS_ISOLATION_ENV.to_string(), isolation.to_string());
        if isolation_full && !write_allowlist_prefixes.is_empty() {
            env_map.insert(
                WORLD_FS_WRITE_ALLOWLIST_ENV.to_string(),
                write_allowlist_prefixes.join("\n"),
            );
        }
        #[cfg(target_os = "linux")]
        let landlock_supported = world::landlock::detect_support().supported;
        #[cfg(not(target_os = "linux"))]
        let landlock_supported = false;

        if isolation_full {
            apply_full_isolation_helper_env(
                &mut env_map,
                landlock_supported,
                &landlock_discover_paths,
                &landlock_read_paths,
                &landlock_write_paths,
                enforcement_plan_b64.as_deref(),
            );
        }
        let exec_req = world_api::ExecRequest {
            cmd: wrap_command_for_profile(req.profile.as_deref(), &cwd, &req.cmd),
            cwd,
            env: env_map,
            pty: req.pty,
            span_id: None,
        };

        // Execute command
        let result = match self.backend.exec(&world, exec_req) {
            Ok(r) => r,
            Err(e) => {
                self.record_last_netfilter_failure_for_error(isolate_network, &e);
                tracing::error!(error = %e, "exec failed");
                return Err(anyhow::anyhow!("Command execution failed"));
            }
        };

        self.clear_last_netfilter_failure_on_success(isolate_network);

        #[cfg(target_os = "linux")]
        {
            if let Some(ref diff) = result.fs_diff {
                let snapshot = Self::normalize_pending_diff_bucket(diff);
                self.note_pending_diff_origin_for_world(
                    &world.id,
                    PendingDiffOrigin::NonPty,
                    &snapshot,
                );
            }
        }

        // Generate span ID
        let span_id = format!("spn_{}", uuid::Uuid::now_v7());

        Ok(ExecuteResponse {
            exit: result.exit,
            span_id,
            stdout_b64: BASE64.encode(result.stdout),
            stderr_b64: BASE64.encode(result.stderr),
            scopes_used: result.scopes_used,
            fs_diff: result.fs_diff,
            process_telemetry: result.process_telemetry,
        })
    }

    /// Retrieve the current session's pending diff record.
    pub async fn pending_diff(&self, req: PendingDiffRequestV1) -> Result<PendingDiffRecordV1> {
        #[cfg(target_os = "linux")]
        {
            if req.agent_id.is_empty() {
                anyhow::bail!("agent_id is required for API calls");
            }

            let cwd = req
                .cwd
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let env_ref = req.env.as_ref();
            let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;

            let snapshot = req.policy_snapshot.clone();
            let (_, policy_inputs) =
                resolve_policy_inputs(&snapshot, req.world_network.as_ref(), &cwd, &project_dir)?;
            let fs_mode = policy_inputs.fs_mode;
            let isolate_network = policy_inputs.isolate_network;
            let allowed_domains = policy_inputs.allowed_domains.clone();

            let always_isolate = should_always_isolate_for_profile(req.profile.as_deref());

            let spec = build_world_spec(
                project_dir,
                always_isolate,
                fs_mode,
                isolate_network,
                allowed_domains,
            );

            let world = match self.backend.ensure_session(&spec) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };

            let (started_at, diff) = self
                .linux_backend
                .pending_diff(&world)
                .context("pending_diff failed")?;

            let snapshot = Self::normalize_pending_diff_bucket(&diff);
            let hex = Self::pending_diff_id_for_snapshot(&snapshot);

            let started_at: chrono::DateTime<chrono::Utc> = started_at.into();
            let session_started_at =
                started_at.to_rfc3339_opts(SecondsFormat::Nanos, /* use_z */ true);

            let mut non_pty = PendingDiffBucketV1::default();
            let mut pty = PendingDiffBucketV1::default();

            let origin_map: Option<HashMap<String, PendingDiffOrigin>> = self
                .pending_diff_origin
                .read()
                .ok()
                .and_then(|guard| guard.get(&world.id).map(|t| t.origin_by_path.clone()));

            let classify = |path: &str| -> PendingDiffOrigin {
                origin_map
                    .as_ref()
                    .and_then(|m| m.get(path).copied())
                    .unwrap_or(PendingDiffOrigin::NonPty)
            };

            for item in &snapshot.writes {
                match classify(item) {
                    PendingDiffOrigin::NonPty => non_pty.writes.push(item.clone()),
                    PendingDiffOrigin::Pty => pty.writes.push(item.clone()),
                }
            }
            for item in &snapshot.mods {
                match classify(item) {
                    PendingDiffOrigin::NonPty => non_pty.mods.push(item.clone()),
                    PendingDiffOrigin::Pty => pty.mods.push(item.clone()),
                }
            }
            for item in &snapshot.deletes {
                match classify(item) {
                    PendingDiffOrigin::NonPty => non_pty.deletes.push(item.clone()),
                    PendingDiffOrigin::Pty => pty.deletes.push(item.clone()),
                }
            }

            Ok(PendingDiffRecordV1 {
                schema_version: 1,
                session_started_at,
                diff_id: hex,
                non_pty,
                pty: Some(pty),
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("pending diff discovery is only supported on Linux agents")
        }
    }

    /// Conditionally clear the current session's pending diff snapshot.
    pub async fn pending_diff_clear(
        &self,
        req: PendingDiffClearRequestV1,
    ) -> Result<PendingDiffClearResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.agent_id.is_empty() {
                anyhow::bail!("agent_id is required for API calls");
            }

            let cwd = req
                .cwd
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let env_ref = req.env.as_ref();
            let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;

            let snapshot = req.policy_snapshot.clone();
            let (_, policy_inputs) =
                resolve_policy_inputs(&snapshot, req.world_network.as_ref(), &cwd, &project_dir)?;
            let fs_mode = policy_inputs.fs_mode;
            let isolate_network = policy_inputs.isolate_network;
            let allowed_domains = policy_inputs.allowed_domains.clone();

            let always_isolate = should_always_isolate_for_profile(req.profile.as_deref());

            let spec = build_world_spec(
                project_dir,
                always_isolate,
                fs_mode,
                isolate_network,
                allowed_domains,
            );

            let world = match self.backend.ensure_session(&spec) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };

            let (_started_at, diff) = self
                .linux_backend
                .pending_diff(&world)
                .context("pending_diff failed")?;
            let current_hex = Self::pending_diff_id_for_diff(&diff);

            if current_hex != req.diff_id {
                return Ok(PendingDiffClearResponseV1 {
                    schema_version: 1,
                    cleared: false,
                });
            }

            self.linux_backend
                .clear_pending_diff(&world)
                .context("clear_pending_diff failed")?;

            if let Ok(mut guard) = self.pending_diff_origin.write() {
                guard.remove(&world.id);
            }

            Ok(PendingDiffClearResponseV1 {
                schema_version: 1,
                cleared: true,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("pending diff clear is only supported on Linux agents")
        }
    }

    /// Conditionally reconcile (discard) specific pending diff paths by mutating the overlay upper/work layers.
    pub async fn pending_diff_reconcile(
        &self,
        req: PendingDiffReconcileRequestV1,
    ) -> Result<PendingDiffReconcileResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.agent_id.is_empty() {
                anyhow::bail!("agent_id is required for API calls");
            }

            let cwd = req
                .cwd
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let env_ref = req.env.as_ref();
            let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;

            let snapshot = req.policy_snapshot.clone();
            let (_, policy_inputs) =
                resolve_policy_inputs(&snapshot, req.world_network.as_ref(), &cwd, &project_dir)?;
            let fs_mode = policy_inputs.fs_mode;
            let isolate_network = policy_inputs.isolate_network;
            let allowed_domains = policy_inputs.allowed_domains.clone();

            let always_isolate = should_always_isolate_for_profile(req.profile.as_deref());

            let spec = build_world_spec(
                project_dir,
                always_isolate,
                fs_mode,
                isolate_network,
                allowed_domains,
            );

            let world = match self.backend.ensure_session(&spec) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };

            let (_started_at, diff) = self
                .linux_backend
                .pending_diff(&world)
                .context("pending_diff failed")?;
            let current_hex = Self::pending_diff_id_for_diff(&diff);

            if current_hex != req.diff_id {
                return Ok(PendingDiffReconcileResponseV1 {
                    schema_version: 1,
                    reconciled: false,
                    discarded: 0,
                });
            }

            let mut rel_paths: Vec<PathBuf> = Vec::new();
            let mut normalized_for_tracker: Vec<String> = Vec::new();
            for raw in &req.discard_paths {
                let mut rel = raw.trim().replace('\\', "/");
                while let Some(stripped) = rel.strip_prefix("./") {
                    rel = stripped.to_string();
                }
                if rel.is_empty() {
                    return Err(BadRequestError::new("empty discard path".to_string()).into());
                }
                if rel.starts_with('/') {
                    return Err(BadRequestError::new(format!(
                        "absolute paths are not allowed: {rel}"
                    ))
                    .into());
                }
                if rel.len() >= 2 && rel.as_bytes()[1] == b':' {
                    return Err(BadRequestError::new(format!(
                        "absolute paths are not allowed: {rel}"
                    ))
                    .into());
                }
                if rel.split('/').any(|segment| segment == "..") {
                    return Err(BadRequestError::new(format!(
                        "path segments must not be '..': {rel}"
                    ))
                    .into());
                }
                rel_paths.push(PathBuf::from(&rel));
                normalized_for_tracker.push(rel);
            }

            let removed = self
                .linux_backend
                .discard_pending_paths(&world, &rel_paths)
                .context("discard_pending_paths failed")?;

            if !normalized_for_tracker.is_empty() {
                if let Ok(mut guard) = self.pending_diff_origin.write() {
                    if let Some(tracker) = guard.get_mut(&world.id) {
                        for p in &normalized_for_tracker {
                            tracker.origin_by_path.remove(p);
                            tracker.last_seen_paths.remove(p);
                        }
                    }
                }
            }

            Ok(PendingDiffReconcileResponseV1 {
                schema_version: 1,
                reconciled: true,
                discarded: removed,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("pending diff reconcile is only supported on Linux agents")
        }
    }

    /// Read metadata and optionally contents from the current session's overlay filesystem.
    pub async fn world_fs_read(&self, req: WorldFsReadRequestV1) -> Result<WorldFsReadResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.agent_id.is_empty() {
                anyhow::bail!("agent_id is required for API calls");
            }

            let mut rel = req.path.trim().replace('\\', "/");
            while let Some(stripped) = rel.strip_prefix("./") {
                rel = stripped.to_string();
            }
            if rel.is_empty() {
                return Err(BadRequestError::new("empty path".to_string()).into());
            }
            if rel.starts_with('/') {
                return Err(
                    BadRequestError::new(format!("absolute paths are not allowed: {rel}")).into(),
                );
            }
            if rel.len() >= 2 && rel.as_bytes()[1] == b':' {
                return Err(
                    BadRequestError::new(format!("absolute paths are not allowed: {rel}")).into(),
                );
            }
            if rel.split('/').any(|segment| segment == "..") {
                return Err(
                    BadRequestError::new(format!("path segments must not be '..': {rel}")).into(),
                );
            }

            let cwd = req
                .cwd
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let env_ref = req.env.as_ref();
            let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;

            let snapshot = req.policy_snapshot.clone();
            let (_, policy_inputs) =
                resolve_policy_inputs(&snapshot, req.world_network.as_ref(), &cwd, &project_dir)?;
            let fs_mode = policy_inputs.fs_mode;
            let isolate_network = policy_inputs.isolate_network;
            let allowed_domains = policy_inputs.allowed_domains.clone();

            let always_isolate = should_always_isolate_for_profile(req.profile.as_deref());

            let spec = build_world_spec(
                project_dir,
                always_isolate,
                fs_mode,
                isolate_network,
                allowed_domains,
            );

            let world = match self.backend.ensure_session(&spec) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };

            let overlay_root = self
                .linux_backend
                .ensure_overlay_root(&world)
                .context("ensure_overlay_root failed")?;

            let full_path = overlay_root.join(&rel);
            let meta = std::fs::symlink_metadata(&full_path).with_context(|| {
                format!(
                    "failed to read metadata for world fs path {}",
                    full_path.display()
                )
            })?;

            let ft = meta.file_type();
            let entry_type = if ft.is_file() {
                WorldFsEntryTypeV1::RegularFile
            } else if ft.is_dir() {
                WorldFsEntryTypeV1::Directory
            } else if ft.is_symlink() {
                WorldFsEntryTypeV1::Symlink
            } else if ft.is_socket() {
                WorldFsEntryTypeV1::Socket
            } else if ft.is_fifo() {
                WorldFsEntryTypeV1::Fifo
            } else if ft.is_block_device() {
                WorldFsEntryTypeV1::BlockDevice
            } else if ft.is_char_device() {
                WorldFsEntryTypeV1::CharDevice
            } else {
                WorldFsEntryTypeV1::Unknown
            };

            let mode = Some(meta.permissions().mode());
            let size = match entry_type {
                WorldFsEntryTypeV1::RegularFile => Some(meta.len()),
                _ => None,
            };

            let contents_b64 =
                if req.include_contents && entry_type == WorldFsEntryTypeV1::RegularFile {
                    let bytes = std::fs::read(&full_path).with_context(|| {
                        format!(
                            "failed to read bytes for world fs path {}",
                            full_path.display()
                        )
                    })?;
                    Some(BASE64.encode(bytes))
                } else {
                    None
                };

            Ok(WorldFsReadResponseV1 {
                schema_version: 1,
                path: rel,
                entry_type,
                size,
                mode,
                contents_b64,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("world fs read is only supported on Linux agents")
        }
    }

    /// Return the typed gateway lifecycle/status surface.
    pub async fn gateway_status(
        &self,
        req: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        #[cfg(target_os = "linux")]
        {
            let prepared = self.prepare_gateway_runtime_request(&req)?;
            let Some(binding) = self
                .resolve_gateway_runtime_binding(
                    prepared,
                    GatewayRuntimeBindingMode::ExistingSessionOnly,
                )
                .map_err(gateway_runtime_error)?
            else {
                return Self::attach_gateway_request_metadata(
                    Self::gateway_unavailable_response(),
                    &req,
                );
            };

            let response = self
                .gateway_runtime
                .status(
                    &binding.runtime_id,
                    binding.start_context.binding.backend_id,
                )
                .await
                .map_err(gateway_runtime_error)?;

            Self::attach_gateway_request_metadata(response, &req)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self::attach_gateway_request_metadata(Self::gateway_unavailable_response(), &req)
        }
    }

    /// Return the typed gateway lifecycle sync surface.
    pub async fn gateway_sync(
        &self,
        req: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        #[cfg(target_os = "linux")]
        {
            let prepared = self.prepare_gateway_runtime_request(&req)?;
            let Some(binding) = self
                .resolve_gateway_runtime_binding(prepared, GatewayRuntimeBindingMode::EnsureSession)
                .map_err(gateway_runtime_error)?
            else {
                return Self::attach_gateway_request_metadata(
                    Self::gateway_unavailable_response(),
                    &req,
                );
            };

            let response = self
                .gateway_runtime
                .sync(binding.start_context)
                .await
                .map_err(gateway_runtime_error)?;

            Self::attach_gateway_request_metadata(response, &req)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self::attach_gateway_request_metadata(Self::gateway_unavailable_response(), &req)
        }
    }

    /// Return the typed gateway lifecycle restart surface.
    pub async fn gateway_restart(
        &self,
        req: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        #[cfg(target_os = "linux")]
        {
            let prepared = self.prepare_gateway_runtime_request(&req)?;
            let Some(binding) = self
                .resolve_gateway_runtime_binding(
                    prepared,
                    GatewayRuntimeBindingMode::ExistingSessionOnly,
                )
                .map_err(gateway_runtime_error)?
            else {
                return Self::attach_gateway_request_metadata(
                    Self::gateway_unavailable_response(),
                    &req,
                );
            };

            let response = self
                .gateway_runtime
                .restart(binding.start_context)
                .await
                .map_err(gateway_runtime_error)?;

            Self::attach_gateway_request_metadata(response, &req)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self::attach_gateway_request_metadata(Self::gateway_unavailable_response(), &req)
        }
    }

    /// Execute a command and stream incremental output frames via NDJSON.
    #[cfg(target_os = "linux")]
    pub async fn execute_stream(&self, req: ExecuteRequest) -> Result<Response> {
        if req.agent_id.is_empty() {
            anyhow::bail!("agent_id is required for API calls");
        }

        if req.pty {
            anyhow::bail!("PTY streaming is handled via /v1/stream");
        }

        if let Some(budget) = req.budget.clone() {
            let mut budgets = self.budgets.write().unwrap();
            let tracker = budgets
                .entry(req.agent_id.clone())
                .or_insert_with(|| AgentBudgetTracker::new(&req.agent_id, budget));
            if !tracker.can_execute() {
                anyhow::bail!("Budget exceeded: max executions reached");
            }
            tracker.decrement_exec();
        }

        let cwd = req
            .cwd
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let env_ref = req.env.as_ref();
        let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;
        let (policy_resolution_mode, policy_inputs) = resolve_policy_inputs(
            &req.policy_snapshot,
            req.world_network.as_ref(),
            &cwd,
            &project_dir,
        )?;

        let PolicyInputs {
            fs_mode,
            isolation_full,
            isolate_network,
            allowed_domains,
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        } = policy_inputs;
        self.record_doctor_request_context(policy_resolution_mode, isolate_network);
        let always_isolate = should_always_isolate(&req);

        let host_visible = !isolation_full;
        let empty_env: HashMap<String, String> = HashMap::new();
        let guard_env = req.env.as_ref().unwrap_or(&empty_env);
        if let Some(deny) =
            crate::world_exec_guard::check_command(&req.cmd, &cwd, guard_env, host_visible)
        {
            let span_id = format!("spn_{}", uuid::Uuid::now_v7());
            let message = crate::world_exec_guard::deny_message(&deny);
            let frames = vec![
                ExecuteStreamFrame::Start {
                    span_id: span_id.clone(),
                },
                ExecuteStreamFrame::Stderr {
                    chunk_b64: BASE64.encode(message.as_bytes()),
                },
                ExecuteStreamFrame::Exit {
                    exit: 5,
                    span_id,
                    scopes_used: Vec::new(),
                    fs_diff: None,
                    process_telemetry: ProcessTelemetry::default(),
                },
            ];

            let stream = futures_util::stream::iter(frames.into_iter().map(|frame| {
                let mut payload = serde_json::to_vec(&frame).expect("serialize frame");
                payload.push(b'\n');
                Ok::<Bytes, Infallible>(Bytes::from(payload))
            }));

            let body = boxed(StreamBody::new(stream));
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/x-ndjson")
                .body(body)
                .map_err(|e| anyhow!("failed to build stream response: {e}"))?;
            return Ok(response);
        }

        let spec = build_world_spec(
            project_dir,
            always_isolate,
            fs_mode,
            isolate_network,
            allowed_domains,
        );

        let world = match self.backend.ensure_session(&spec) {
            Ok(w) => w,
            Err(e) => {
                self.record_last_netfilter_failure_for_error(isolate_network, &e);
                tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                anyhow::bail!("Failed to ensure session world");
            }
        };

        let mut exec_req = world_api::ExecRequest {
            cmd: req.cmd.clone(),
            cwd: cwd.clone(),
            env: {
                let mut env_map = req.env.clone().unwrap_or_default();
                env_map.remove(WORLD_PROJECT_DIR_OVERRIDE_ENV);
                let isolation = if isolation_full { "full" } else { "workspace" };
                env_map.insert(WORLD_FS_ISOLATION_ENV.to_string(), isolation.to_string());
                if isolation_full && !write_allowlist_prefixes.is_empty() {
                    env_map.insert(
                        WORLD_FS_WRITE_ALLOWLIST_ENV.to_string(),
                        write_allowlist_prefixes.join("\n"),
                    );
                }
                #[cfg(target_os = "linux")]
                let landlock_supported = world::landlock::detect_support().supported;
                #[cfg(not(target_os = "linux"))]
                let landlock_supported = false;

                if isolation_full {
                    apply_full_isolation_helper_env(
                        &mut env_map,
                        landlock_supported,
                        &landlock_discover_paths,
                        &landlock_read_paths,
                        &landlock_write_paths,
                        enforcement_plan_b64.as_deref(),
                    );
                }
                env_map
            },
            pty: false,
            span_id: None,
        };

        let span_id = format!("spn_{}", uuid::Uuid::now_v7());
        exec_req.span_id = Some(span_id.clone());
        world::exec::note_pending_exec(&span_id);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(ExecuteStreamFrame::Start {
            span_id: span_id.clone(),
        });

        let backend = self.backend.clone();
        #[cfg(target_os = "linux")]
        let service = self.clone();
        let agent_id = req.agent_id.clone();
        let span_id_for_cleanup = span_id.clone();
        task::spawn_blocking(move || {
            let sink = Arc::new(StreamingSink::new(tx.clone()));
            let guard = install_stream_sink(sink);
            let result = backend.exec(&world, exec_req);
            drop(guard);

            match result {
                Ok(exec_result) => {
                    service.clear_last_netfilter_failure_on_success(isolate_network);
                    #[cfg(target_os = "linux")]
                    if let Some(ref diff) = exec_result.fs_diff {
                        let snapshot = WorldAgentService::normalize_pending_diff_bucket(diff);
                        service.note_pending_diff_origin_for_world(
                            &world.id,
                            PendingDiffOrigin::NonPty,
                            &snapshot,
                        );
                    }

                    if let (Some(primary), Some(final_strategy), Some(reason)) = (
                        exec_result.world_fs_strategy_primary,
                        exec_result.world_fs_strategy_final,
                        exec_result.world_fs_strategy_fallback_reason,
                    ) {
                        let _ = tx.send(ExecuteStreamFrame::Event {
                            event: AgentEvent {
                                ts: chrono::Utc::now(),
                                agent_id: agent_id.clone(),
                                kind: AgentEventKind::Status,
                                orchestration_session_id: span_id.clone(),
                                run_id: span_id.clone(),
                                backend_id: None,
                                thread_id: None,
                                role: None,
                                world_id: Some(world.id.clone()),
                                cmd_id: None,
                                span_id: Some(span_id.clone()),
                                channel: None,
                                identity_tuple: None,
                                placement_posture: None,
                                project: None,
                                data: serde_json::json!({
                                    "world_fs_strategy_primary": primary.as_str(),
                                    "world_fs_strategy_final": final_strategy.as_str(),
                                    "world_fs_strategy_fallback_reason": reason.as_str(),
                                }),
                            },
                        });
                    }
                    let frame = ExecuteStreamFrame::Exit {
                        exit: exec_result.exit,
                        span_id,
                        scopes_used: exec_result.scopes_used,
                        fs_diff: exec_result.fs_diff,
                        process_telemetry: exec_result.process_telemetry,
                    };
                    let _ = tx.send(frame);
                }
                Err(e) => {
                    service.record_last_netfilter_failure_for_error(isolate_network, &e);
                    tracing::error!(error = %e, agent = agent_id, "exec failed");
                    let _ = tx.send(ExecuteStreamFrame::Error {
                        message: e.to_string(),
                    });
                }
            }
            world::exec::clear_registered_exec(&span_id_for_cleanup);
        });

        let stream = UnboundedReceiverStream::new(rx).map(|frame| {
            let mut payload = serde_json::to_vec(&frame).expect("serialize frame");
            payload.push(b'\n');
            Ok::<Bytes, Infallible>(Bytes::from(payload))
        });

        let body = boxed(StreamBody::new(stream));
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/x-ndjson")
            .body(body)
            .context("Failed to build streaming response")?;

        Ok(response)
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn execute_stream(&self, _req: ExecuteRequest) -> Result<Response> {
        anyhow::bail!("World agent streaming is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    pub async fn execute_cancel(
        &self,
        req: ExecuteCancelRequestV1,
    ) -> Result<ExecuteCancelResponseV1> {
        if req.span_id.trim().is_empty() {
            return Err(BadRequestError::new("span_id is required".to_string()).into());
        }

        let mut delivered = false;
        for _ in 0..80 {
            delivered =
                world::exec::signal_registered_exec(&req.span_id, &req.sig).with_context(|| {
                    format!("failed to cancel streamed execute span {}", req.span_id)
                })?;
            if delivered {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        Ok(ExecuteCancelResponseV1 {
            schema_version: 1,
            delivered,
        })
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn execute_cancel(
        &self,
        _req: ExecuteCancelRequestV1,
    ) -> Result<ExecuteCancelResponseV1> {
        anyhow::bail!("World agent execute cancellation is only supported on Linux");
    }

    /// Get trace information for a span.
    pub async fn get_trace(&self, span_id: &str) -> Result<serde_json::Value> {
        // TODO: Implement trace retrieval
        Ok(serde_json::json!({
            "span_id": span_id,
            "status": "not_implemented"
        }))
    }

    /// Request additional scopes.
    pub async fn request_scopes(&self, _scopes: Vec<String>) -> Result<serde_json::Value> {
        // TODO: Implement scope request handling
        Ok(serde_json::json!({
            "status": "not_implemented"
        }))
    }

    #[cfg(target_os = "linux")]
    fn prepare_gateway_runtime_request(
        &self,
        req: &GatewayLifecycleRequestV1,
    ) -> Result<PreparedGatewayRuntimeRequest> {
        req.validate_identity_contract()
            .map_err(|err| anyhow!("gateway_invalid_integration: {err}"))?;
        let cwd = req
            .cwd
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let env_ref = req.env.as_ref();
        let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;
        let (policy_resolution_mode, policy_inputs) = resolve_policy_inputs(
            &req.policy_snapshot,
            req.world_network.as_ref(),
            &cwd,
            &project_dir,
        )?;
        self.record_doctor_request_context(policy_resolution_mode, policy_inputs.isolate_network);

        let world_spec = build_world_spec(
            project_dir.clone(),
            should_always_isolate_for_profile(req.profile.as_deref()),
            policy_inputs.fs_mode,
            policy_inputs.isolate_network,
            policy_inputs.allowed_domains,
        );
        let control =
            GatewayControlSettings::from_request_env(env_ref).map_err(gateway_runtime_error)?;
        let selected_backend = control.default_backend.clone();
        let integrated_auth = match req.integrated_auth.as_ref() {
            Some(payload) => {
                payload
                    .validate_for_selected_backend(&selected_backend)
                    .map_err(|message| {
                        gateway_runtime_error(GatewayRuntimeFailure::invalid_integration(message))
                    })?;
                Some(payload.clone())
            }
            None => None,
        };

        Ok(PreparedGatewayRuntimeRequest {
            project_dir,
            world_spec,
            selected_backend,
            integrated_auth,
        })
    }

    #[cfg(target_os = "linux")]
    fn resolve_gateway_runtime_binding(
        &self,
        prepared: PreparedGatewayRuntimeRequest,
        mode: GatewayRuntimeBindingMode,
    ) -> std::result::Result<Option<ResolvedGatewayRuntimeBinding>, GatewayRuntimeFailure> {
        let Some(binding) = resolve_gateway_backend_binding(&prepared.selected_backend) else {
            return Ok(None);
        };

        if !prepared.world_spec.isolate_network {
            return Ok(Some(ResolvedGatewayRuntimeBinding::non_isolated(
                prepared, binding,
            )));
        }

        let maybe_world = match mode {
            GatewayRuntimeBindingMode::EnsureSession => Some(
                self.backend
                    .ensure_session(&prepared.world_spec)
                    .map_err(|err| {
                        self.record_last_netfilter_failure_for_error(
                            prepared.world_spec.isolate_network,
                            &err,
                        );
                        GatewayRuntimeFailure::transient(format!(
                            "failed to ensure session world: {}",
                            err
                        ))
                    })?,
            ),
            GatewayRuntimeBindingMode::ExistingSessionOnly => self
                .linux_backend
                .find_compatible_session(&prepared.world_spec)
                .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?,
        };

        let Some(world) = maybe_world else {
            return Ok(None);
        };
        let cgroup_path = self
            .session_cgroup_path(&world)
            .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
        let runtime_id = isolated_gateway_runtime_id(&world.id, binding.backend_id);

        Ok(Some(ResolvedGatewayRuntimeBinding {
            runtime_id: runtime_id.clone(),
            start_context: GatewayRuntimeStartContext {
                world_id: runtime_id,
                project_dir: prepared.project_dir,
                cgroup_path,
                require_cgroup_attach: true,
                binding,
                integrated_auth: prepared.integrated_auth,
            },
        }))
    }

    #[cfg(target_os = "linux")]
    pub fn gateway_runtime_pid_for_test(
        &self,
        req: &GatewayLifecycleRequestV1,
    ) -> Result<Option<u32>> {
        let prepared = self.prepare_gateway_runtime_request(req)?;
        let Some(binding) = self
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .map_err(gateway_runtime_error)?
        else {
            return Ok(None);
        };
        Ok(self.gateway_runtime.pid_for_world(&binding.runtime_id))
    }

    #[cfg(target_os = "linux")]
    pub fn forget_gateway_runtime_for_test(&self, req: &GatewayLifecycleRequestV1) -> Result<()> {
        let prepared = self.prepare_gateway_runtime_request(req)?;
        let Some(binding) = self
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .map_err(gateway_runtime_error)?
        else {
            return Ok(());
        };
        self.gateway_runtime
            .forget_runtime_for_test(&binding.runtime_id);
        Ok(())
    }

    fn gateway_unavailable_response() -> GatewayLifecycleResponseV1 {
        #[cfg(target_os = "linux")]
        {
            gateway_unavailable_response_impl()
        }
        #[cfg(not(target_os = "linux"))]
        {
            GatewayLifecycleResponseV1 {
                status: GatewayStatusV1::Unavailable,
                client_wiring: None,
                identity_tuple: None,
                placement_posture: None,
            }
        }
    }

    fn attach_gateway_request_metadata(
        mut response: GatewayLifecycleResponseV1,
        req: &GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        if response.identity_tuple.is_none() {
            response.identity_tuple = req.identity_tuple.clone();
        }
        if response.placement_posture.is_none() {
            response.placement_posture = req.placement_posture.clone();
        }
        response
            .validate_identity_contract()
            .map_err(|err| anyhow!("gateway_invalid_integration: {err}"))?;
        Ok(response)
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct PreparedGatewayRuntimeRequest {
    project_dir: PathBuf,
    world_spec: WorldSpec,
    selected_backend: String,
    integrated_auth: Option<agent_api_types::GatewayIntegratedAuthPayloadV1>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
enum GatewayRuntimeBindingMode {
    EnsureSession,
    ExistingSessionOnly,
}

#[cfg(target_os = "linux")]
struct ResolvedGatewayRuntimeBinding {
    runtime_id: String,
    start_context: GatewayRuntimeStartContext,
}

#[cfg(target_os = "linux")]
impl ResolvedGatewayRuntimeBinding {
    fn non_isolated(
        prepared: PreparedGatewayRuntimeRequest,
        binding: &'static crate::gateway_runtime::GatewayBackendBinding,
    ) -> Self {
        let runtime_id = non_isolated_gateway_runtime_id(
            &prepared.project_dir,
            &prepared.world_spec,
            binding.backend_id,
        );
        Self {
            runtime_id: runtime_id.clone(),
            start_context: GatewayRuntimeStartContext {
                world_id: runtime_id,
                project_dir: prepared.project_dir,
                cgroup_path: PathBuf::from("/nonexistent"),
                require_cgroup_attach: false,
                binding,
                integrated_auth: prepared.integrated_auth,
            },
        }
    }
}

#[cfg(target_os = "linux")]
fn non_isolated_gateway_runtime_id(
    project_dir: &Path,
    world_spec: &WorldSpec,
    backend_id: &str,
) -> String {
    let canonical_project_dir = std::fs::canonicalize(project_dir)
        .unwrap_or_else(|_| project_dir.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    let mut allowed_domains = world_spec.allowed_domains.clone();
    allowed_domains.sort();
    allowed_domains.dedup();

    let mut hasher = Sha256::new();
    hasher.update(b"project_dir\0");
    hasher.update(canonical_project_dir.as_bytes());
    hasher.update(b"\0isolate_network\0");
    hasher.update(if world_spec.isolate_network {
        b"1"
    } else {
        b"0"
    });
    hasher.update(b"\0always_isolate\0");
    hasher.update(if world_spec.always_isolate {
        b"1"
    } else {
        b"0"
    });
    hasher.update(b"\0allowed_domains\0");
    for domain in allowed_domains {
        hasher.update(domain.as_bytes());
        hasher.update(b"\n");
    }
    hasher.update(b"\0backend_id\0");
    hasher.update(backend_id.as_bytes());

    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        encoded.push_str(&format!("{byte:02x}"));
    }
    format!("gwrt_{encoded}")
}

#[cfg(target_os = "linux")]
fn isolated_gateway_runtime_id(world_id: &str, backend_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"world_id\0");
    hasher.update(world_id.as_bytes());
    hasher.update(b"\0backend_id\0");
    hasher.update(backend_id.as_bytes());

    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        encoded.push_str(&format!("{byte:02x}"));
    }
    format!("gwrt_{encoded}")
}

#[cfg(target_os = "linux")]
fn gateway_runtime_error(err: GatewayRuntimeFailure) -> anyhow::Error {
    anyhow!(err.to_string())
}

#[cfg(test)]
mod pending_diff_id_tests {
    use super::WorldAgentService;
    use agent_api_types::PendingDiffBucketV1;
    use std::path::PathBuf;
    use substrate_common::FsDiff;

    #[test]
    fn diff_id_is_stable_across_writes_vs_mods_reclassification() {
        let s1 = PendingDiffBucketV1 {
            writes: vec!["flipcheck.md".to_string()],
            mods: vec![],
            deletes: vec![],
        };
        let s2 = PendingDiffBucketV1 {
            writes: vec![],
            mods: vec!["flipcheck.md".to_string()],
            deletes: vec![],
        };
        assert_eq!(
            WorldAgentService::pending_diff_id_for_snapshot(&s1),
            WorldAgentService::pending_diff_id_for_snapshot(&s2),
            "diff_id must not change when a path moves between writes and mods"
        );
    }

    #[test]
    fn diff_id_changes_when_updates_or_deletes_change() {
        let base = PendingDiffBucketV1 {
            writes: vec!["a.txt".to_string()],
            mods: vec![],
            deletes: vec![],
        };
        let with_more_updates = PendingDiffBucketV1 {
            writes: vec!["a.txt".to_string(), "b.txt".to_string()],
            mods: vec![],
            deletes: vec![],
        };
        let with_delete = PendingDiffBucketV1 {
            writes: vec!["a.txt".to_string()],
            mods: vec![],
            deletes: vec!["gone.txt".to_string()],
        };
        assert_ne!(
            WorldAgentService::pending_diff_id_for_snapshot(&base),
            WorldAgentService::pending_diff_id_for_snapshot(&with_more_updates),
            "diff_id must change when the update set changes"
        );
        assert_ne!(
            WorldAgentService::pending_diff_id_for_snapshot(&base),
            WorldAgentService::pending_diff_id_for_snapshot(&with_delete),
            "diff_id must change when the delete set changes"
        );
    }

    #[test]
    fn diff_id_for_diff_is_stable_across_fs_diff_writes_mods_flip() {
        let d1 = FsDiff {
            writes: vec![PathBuf::from("flipcheck.md")],
            ..Default::default()
        };
        let d2 = FsDiff {
            mods: vec![PathBuf::from("flipcheck.md")],
            ..Default::default()
        };
        assert_eq!(
            WorldAgentService::pending_diff_id_for_diff(&d1),
            WorldAgentService::pending_diff_id_for_diff(&d2),
            "diff_id must not change when FsDiff reclassifies a path between writes and mods"
        );
    }
}

#[cfg(all(target_os = "linux", test))]
mod gateway_runtime_binding_tests {
    use super::*;
    use agent_api_types::{
        GatewayApiEnvIntegratedAuthV1, GatewayCliCodexIntegratedAuthV1,
        GatewayIntegratedAuthPayloadV1, PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3,
        PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3, WorldNetworkRoutingV1,
    };
    use serde_json::json;
    use tempfile::TempDir;

    fn minimal_policy_snapshot() -> PolicySnapshotV3 {
        PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: None,
                read: None,
                write: PolicySnapshotWorldFsWriteV3::default(),
            },
        }
    }

    fn gateway_request(cwd: &Path) -> GatewayLifecycleRequestV1 {
        GatewayLifecycleRequestV1 {
            profile: None,
            cwd: Some(cwd.display().to_string()),
            env: None,
            agent_id: "gateway-binding-test".to_string(),
            policy_snapshot: minimal_policy_snapshot(),
            world_network: Some(WorldNetworkRoutingV1 {
                isolate_network: false,
                allowed_domains: Vec::new(),
            }),
            integrated_auth: Some(GatewayIntegratedAuthPayloadV1 {
                backend_id: "cli:codex".to_string(),
                cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                    account_id: Some("acct_test".to_string()),
                    access_token: "header.payload.signature".to_string(),
                }),
                api_env: None,
            }),
            identity_tuple: None,
            placement_posture: None,
        }
    }

    fn gateway_request_with_backend(cwd: &Path, backend_id: &str) -> GatewayLifecycleRequestV1 {
        let mut request = gateway_request(cwd);
        let mut env = HashMap::new();
        env.insert(
            "SUBSTRATE_LLM_DEFAULT_BACKEND".to_string(),
            backend_id.to_string(),
        );
        request.env = Some(env);
        request
    }

    #[test]
    fn request_preparation_rejects_integrated_auth_for_different_backend() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldAgentService::new().expect("service");
        let mut request = gateway_request_with_backend(temp_dir.path(), "cli:codex");
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "api:openai".to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        });

        let err = service
            .prepare_gateway_runtime_request(&request)
            .expect_err("mismatched auth payload should fail");
        assert!(
            err.to_string().contains("does not match selected backend"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn non_isolated_runtime_id_is_stable_for_equivalent_specs() {
        let project_dir = PathBuf::from("/tmp/project");
        let mut world_spec = build_world_spec(
            project_dir.clone(),
            false,
            WorldFsMode::Writable,
            false,
            vec!["b.example.com".to_string(), "a.example.com".to_string()],
        );
        let first = non_isolated_gateway_runtime_id(&project_dir, &world_spec, "cli:codex");
        world_spec.allowed_domains.reverse();
        let second = non_isolated_gateway_runtime_id(&project_dir, &world_spec, "cli:codex");
        assert_eq!(first, second);
    }

    #[test]
    fn non_isolated_runtime_id_changes_with_binding_inputs() {
        let project_dir = PathBuf::from("/tmp/project");
        let base = build_world_spec(
            project_dir.clone(),
            false,
            WorldFsMode::Writable,
            false,
            vec!["example.com".to_string()],
        );
        let changed_project =
            non_isolated_gateway_runtime_id(Path::new("/tmp/other"), &base, "cli:codex");
        let changed_always_isolate = non_isolated_gateway_runtime_id(
            &project_dir,
            &build_world_spec(
                project_dir.clone(),
                true,
                WorldFsMode::Writable,
                false,
                vec!["example.com".to_string()],
            ),
            "cli:codex",
        );
        let changed_domains = non_isolated_gateway_runtime_id(
            &project_dir,
            &build_world_spec(
                project_dir.clone(),
                false,
                WorldFsMode::Writable,
                false,
                vec!["other.example.com".to_string()],
            ),
            "cli:codex",
        );
        let changed_backend = non_isolated_gateway_runtime_id(&project_dir, &base, "api:openai");
        let base_id = non_isolated_gateway_runtime_id(&project_dir, &base, "cli:codex");
        assert_ne!(base_id, changed_project);
        assert_ne!(base_id, changed_always_isolate);
        assert_ne!(base_id, changed_domains);
        assert_ne!(base_id, changed_backend);
    }

    #[test]
    fn request_preparation_accepts_valid_cli_codex() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldAgentService::new().expect("service");

        service
            .prepare_gateway_runtime_request(&gateway_request(temp_dir.path()))
            .expect("valid cli:codex payload should prepare");
    }

    #[test]
    fn non_isolated_binding_uses_synthetic_runtime_key() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldAgentService::new().expect("service");
        let prepared = service
            .prepare_gateway_runtime_request(&gateway_request(temp_dir.path()))
            .expect("prepared request");
        let binding = service
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .expect("binding resolution")
            .expect("non-isolated binding");

        assert!(binding.runtime_id.starts_with("gwrt_"));
        assert_eq!(binding.start_context.world_id, binding.runtime_id);
        assert!(!binding.start_context.require_cgroup_attach);
        assert_eq!(
            binding.start_context.cgroup_path,
            PathBuf::from("/nonexistent")
        );
    }

    #[test]
    fn unbound_api_backend_returns_none_without_invalid_integration() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldAgentService::new().expect("service");
        let mut request = gateway_request_with_backend(temp_dir.path(), "api:anthropic");
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "api:anthropic".to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        });
        let prepared = service
            .prepare_gateway_runtime_request(&request)
            .expect("prepared request");
        let binding = service
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .expect("binding resolution");

        assert!(
            binding.is_none(),
            "unbound backends should not map to a runtime"
        );
    }

    #[test]
    fn isolated_binding_does_not_synthesize_runtime_without_session() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldAgentService::new().expect("service");
        let mut request = gateway_request(temp_dir.path());
        request.world_network = Some(WorldNetworkRoutingV1 {
            isolate_network: true,
            allowed_domains: Vec::new(),
        });
        let prepared = service
            .prepare_gateway_runtime_request(&request)
            .expect("prepared request");
        let binding = service
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .expect("binding resolution");

        assert!(binding.is_none());
    }

    #[test]
    fn request_preparation_rejects_unknown_lifecycle_request_fields() {
        let err = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
            "profile": null,
            "cwd": "/tmp",
            "env": null,
            "agent_id": "gateway-binding-test",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
                }
            },
            "world_network": {
                "isolate_network": false,
                "allowed_domains": []
            },
            "integrated_auth": {
                "backend_id": "cli:codex",
                "cli_codex": {
                    "access_token": "header.payload.signature"
                }
            },
            "unexpected": true
        }))
        .expect_err("unknown request field should fail");

        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn request_preparation_rejects_multi_facet_payloads() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldAgentService::new().expect("service");
        let mut request = gateway_request(temp_dir.path());
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:codex".to_string(),
            cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                account_id: Some("acct_test".to_string()),
                access_token: "header.payload.signature".to_string(),
            }),
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        });

        let err = service
            .prepare_gateway_runtime_request(&request)
            .expect_err("multi-facet payload should fail");
        assert!(err.to_string().contains("exactly one auth facet"));
    }

    #[test]
    fn request_preparation_rejects_blank_required_values() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldAgentService::new().expect("service");
        let mut request = gateway_request(temp_dir.path());
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:codex".to_string(),
            cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                account_id: Some("acct_test".to_string()),
                access_token: "   ".to_string(),
            }),
            api_env: None,
        });

        let err = service
            .prepare_gateway_runtime_request(&request)
            .expect_err("blank required value should fail");
        assert!(err.to_string().contains("empty cli_codex.access_token"));
    }
}

#[cfg(target_os = "linux")]
struct StreamingSink {
    tx: tokio::sync::mpsc::UnboundedSender<ExecuteStreamFrame>,
}

#[cfg(target_os = "linux")]
impl StreamingSink {
    fn new(tx: tokio::sync::mpsc::UnboundedSender<ExecuteStreamFrame>) -> Self {
        Self { tx }
    }
}

#[cfg(target_os = "linux")]
impl StreamSink for StreamingSink {
    fn write(&self, kind: StreamKind, chunk: &[u8]) {
        if chunk.is_empty() {
            return;
        }
        let encoded = BASE64.encode(chunk);
        let frame = match kind {
            StreamKind::Stdout => ExecuteStreamFrame::Stdout { chunk_b64: encoded },
            StreamKind::Stderr => ExecuteStreamFrame::Stderr { chunk_b64: encoded },
        };
        let _ = self.tx.send(frame);
    }
}

fn should_always_isolate(req: &ExecuteRequest) -> bool {
    should_always_isolate_for_profile(req.profile.as_deref())
}

fn should_always_isolate_for_profile(profile: Option<&str>) -> bool {
    // `world deps provision` is explicitly intended to mutate guest system packages (apt/dpkg),
    // which is incompatible with the default cage's write restrictions. The internal
    // `world-deps-probe` profile shares that relaxed posture for read-only dpkg-query probes so
    // fail-early validation still works on runners where unprivileged user namespaces are blocked.
    !matches!(profile, Some("world-deps-provision" | "world-deps-probe"))
}

fn wrap_command_for_profile(profile: Option<&str>, cwd: &Path, cmd: &str) -> String {
    match profile {
        Some("world-deps-provision") => build_systemd_provision_wrapper(cwd, cmd),
        _ => cmd.to_string(),
    }
}

fn build_systemd_provision_wrapper(cwd: &Path, cmd: &str) -> String {
    let working_directory = cwd.display().to_string();
    [
        "systemd-run".to_string(),
        "--quiet".to_string(),
        "--wait".to_string(),
        "--pipe".to_string(),
        "--collect".to_string(),
        "--service-type=exec".to_string(),
        format!("--working-directory={}", sh_quote(&working_directory)),
        format!(
            "--description={}",
            sh_quote("Substrate world-deps provisioning")
        ),
        "/bin/sh".to_string(),
        "-lc".to_string(),
        sh_quote(cmd),
    ]
    .join(" ")
}

fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub(crate) struct BadRequestError {
    message: String,
}

impl BadRequestError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug)]
struct PolicyInputs {
    fs_mode: WorldFsMode,
    isolation_full: bool,
    isolate_network: bool,
    allowed_domains: Vec<String>,
    write_allowlist_prefixes: Vec<String>,
    landlock_discover_paths: Vec<String>,
    landlock_read_paths: Vec<String>,
    landlock_write_paths: Vec<String>,
    enforcement_plan_b64: Option<String>,
}

fn resolve_policy_inputs(
    policy_snapshot: &agent_api_types::PolicySnapshotV3,
    world_network: Option<&WorldNetworkRoutingV1>,
    _cwd: &Path,
    project_dir: &Path,
) -> Result<(agent_api_types::PolicyResolutionModeV1, PolicyInputs)> {
    use agent_api_types::PolicyResolutionModeV1;

    let resolved =
        resolve_snapshot_routing(policy_snapshot, world_network).map_err(BadRequestError::new)?;
    let snapshot = resolved.snapshot;

    let isolation_full = resolved.isolation_full;
    let fs_mode = resolved.fs_mode;

    let enforcement_plan_b64 = enforcement_plan::maybe_encode_from_snapshot(&snapshot)
        .map_err(|err| BadRequestError::new(err.to_string()))?;

    let (
        write_allowlist_prefixes,
        landlock_discover_paths,
        landlock_read_paths,
        landlock_write_paths,
    ) = if isolation_full {
        let read_allowlist = snapshot
            .world_fs
            .read
            .as_ref()
            .map(|d| d.allow_list.as_slice())
            .unwrap_or(&[]);
        let discover_allowlist = snapshot
            .world_fs
            .discover
            .as_ref()
            .map(|d| d.allow_list.as_slice())
            .unwrap_or(read_allowlist);
        let write_allowlist = snapshot.world_fs.write.allow_list.as_slice();

        (
            resolve_project_write_allowlist_prefixes(project_dir, write_allowlist),
            resolve_landlock_allowlist_paths(project_dir, discover_allowlist),
            resolve_landlock_allowlist_paths(project_dir, read_allowlist),
            resolve_landlock_allowlist_paths(project_dir, write_allowlist),
        )
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };

    Ok((
        PolicyResolutionModeV1::SnapshotV3,
        PolicyInputs {
            fs_mode,
            isolation_full,
            isolate_network: resolved.world_network.isolate_network,
            allowed_domains: resolved.world_network.allowed_domains,
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        },
    ))
}

fn build_world_spec(
    project_dir: PathBuf,
    always_isolate: bool,
    fs_mode: WorldFsMode,
    isolate_network: bool,
    allowed_domains: Vec<String>,
) -> WorldSpec {
    WorldSpec {
        reuse_session: true,
        isolate_network,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains,
        project_dir,
        always_isolate,
        fs_mode,
    }
}

pub(crate) fn apply_full_isolation_helper_env(
    env_map: &mut HashMap<String, String>,
    landlock_supported: bool,
    landlock_discover_paths: &[String],
    landlock_read_paths: &[String],
    landlock_write_paths: &[String],
    enforcement_plan_b64: Option<&str>,
) {
    if let Some(plan) = enforcement_plan_b64 {
        env_map.insert(
            enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV.to_string(),
            plan.to_string(),
        );
    }

    let landlock_env_needed = landlock_supported
        && (!landlock_discover_paths.is_empty()
            || !landlock_read_paths.is_empty()
            || !landlock_write_paths.is_empty());
    if landlock_env_needed {
        if !landlock_discover_paths.is_empty() {
            env_map.insert(
                WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST_ENV.to_string(),
                landlock_discover_paths.join("\n"),
            );
        }
        if !landlock_read_paths.is_empty() {
            env_map.insert(
                WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV.to_string(),
                landlock_read_paths.join("\n"),
            );
        }
        if !landlock_write_paths.is_empty() {
            env_map.insert(
                WORLD_FS_LANDLOCK_WRITE_ALLOWLIST_ENV.to_string(),
                landlock_write_paths.join("\n"),
            );
        }
    }

    let helper_needed = landlock_env_needed || enforcement_plan_b64.is_some();
    if helper_needed {
        if let Some(helper_src) = resolve_landlock_helper_src() {
            env_map
                .entry(LANDLOCK_HELPER_SRC_ENV.to_string())
                .or_insert(helper_src);
        }
    }
}

impl WorldAgentService {
    pub(crate) fn policy_snapshot_v1_supported(&self) -> bool {
        false
    }

    pub(crate) fn last_netfilter_failure_reason(&self) -> Option<String> {
        self.last_netfilter_failure_reason
            .read()
            .expect("last netfilter failure reason lock poisoned")
            .clone()
    }

    pub(crate) fn last_netfilter_requested(&self) -> bool {
        self.last_netfilter_requested.load(Ordering::Relaxed) == 1
    }

    pub(crate) fn last_policy_resolution_mode(
        &self,
    ) -> Option<agent_api_types::PolicyResolutionModeV1> {
        match self.last_policy_resolution_mode.load(Ordering::Relaxed) {
            1 => Some(agent_api_types::PolicyResolutionModeV1::SnapshotV3),
            2 => Some(agent_api_types::PolicyResolutionModeV1::LegacyLocal),
            _ => None,
        }
    }

    pub(crate) fn set_last_policy_resolution_mode(
        &self,
        mode: agent_api_types::PolicyResolutionModeV1,
    ) {
        let encoded = match mode {
            agent_api_types::PolicyResolutionModeV1::SnapshotV3 => 1,
            agent_api_types::PolicyResolutionModeV1::LegacyLocal => 2,
        };
        self.last_policy_resolution_mode
            .store(encoded, Ordering::Relaxed);
    }

    pub(crate) fn set_last_netfilter_requested(&self, requested: bool) {
        self.last_netfilter_requested
            .store(u8::from(requested), Ordering::Relaxed);
    }

    pub(crate) fn set_last_netfilter_failure_reason(&self, reason: Option<String>) {
        *self
            .last_netfilter_failure_reason
            .write()
            .expect("last netfilter failure reason lock poisoned") = reason;
    }

    pub(crate) fn record_doctor_request_context(
        &self,
        mode: agent_api_types::PolicyResolutionModeV1,
        requested: bool,
    ) {
        self.set_last_policy_resolution_mode(mode);
        self.set_last_netfilter_requested(requested);
    }

    pub(crate) fn clear_last_netfilter_failure_on_success(&self, requested: bool) {
        if requested {
            self.set_last_netfilter_failure_reason(None);
        }
    }

    pub(crate) fn record_last_netfilter_failure_for_error(
        &self,
        requested: bool,
        error: &anyhow::Error,
    ) {
        if !requested {
            return;
        }
        if let Some(reason) = classify_last_netfilter_failure_reason(error) {
            self.set_last_netfilter_failure_reason(Some(reason));
        }
    }
}

fn classify_last_netfilter_failure_reason(error: &anyhow::Error) -> Option<String> {
    let message = format!("{error:#}");
    let is_known_failure = message.contains(NETFILTER_ENABLE_REQUIRED_TEXT)
        || message.contains(NETFILTER_NFT_FAILURE_TEXT)
        || message.contains(NETFILTER_RESOLUTION_FAILURE_TEXT)
        || message.contains(NETFILTER_NO_ADDRESS_TEXT)
        || message.contains(NETFILTER_CGROUP_ATTACH_TEXT)
        || message.contains(NETFILTER_CGROUP_HELPER_REFUSAL_TEXT)
        || message.contains(NETFILTER_CGROUP_FORCE_DIRECT_TEXT);

    if is_known_failure {
        Some(message)
    } else {
        None
    }
}

pub(crate) fn resolve_project_dir(
    env: Option<&HashMap<String, String>>,
    cwd: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(path) = env
        .and_then(|map| map.get(WORLD_PROJECT_DIR_OVERRIDE_ENV))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return Ok(PathBuf::from(path));
    }

    let cwd_path = cwd
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let mode = env
        .and_then(|map| map.get(ANCHOR_MODE_ENV))
        .and_then(|value| WorldRootMode::parse(value))
        .unwrap_or(WorldRootMode::Project);

    let root_path = env
        .and_then(|map| map.get(ANCHOR_PATH_ENV))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    let base_dir = match mode {
        WorldRootMode::Project => root_path.unwrap_or_else(|| cwd_path.clone()),
        WorldRootMode::FollowCwd => cwd_path.clone(),
        WorldRootMode::Custom => root_path
            .ok_or_else(|| anyhow!("anchor mode 'custom' requires SUBSTRATE_ANCHOR_PATH"))?,
    };

    Ok(base_dir)
}

#[cfg(target_os = "linux")]
pub(crate) fn is_full_isolation(env: Option<&HashMap<String, String>>) -> bool {
    if let Some(env) = env {
        if let Some(raw) = env.get(WORLD_FS_ISOLATION_ENV) {
            return raw.trim().eq_ignore_ascii_case("full");
        }
    }
    std::env::var(WORLD_FS_ISOLATION_ENV)
        .ok()
        .is_some_and(|raw| raw.trim().eq_ignore_ascii_case("full"))
}

pub(crate) fn resolve_project_write_allowlist_prefixes(
    project_dir: &Path,
    write_allowlist: &[String],
) -> Vec<String> {
    let project_str = project_dir.to_string_lossy();
    let mut prefixes: Vec<String> = Vec::new();

    for raw_pattern in write_allowlist {
        let pattern = raw_pattern.trim();
        if pattern.is_empty() {
            continue;
        }

        // Apply only allowlist entries that target paths under the project root. Reduce globs to
        // the directory prefix up to the first wildcard/meta character.
        let rel = if pattern.starts_with('/') {
            if pattern == project_str {
                "*"
            } else if let Some(stripped) = pattern.strip_prefix(&format!("{}/", project_str)) {
                stripped
            } else {
                continue;
            }
        } else {
            pattern
        };

        let rel = rel.trim_start_matches("./");

        if matches!(rel, "*" | "**" | "/*" | "/**") {
            prefixes.push(".".to_string());
            continue;
        }

        let mut prefix = rel;
        if let Some(idx) = rel.find(['*', '?', '[']) {
            prefix = &rel[..idx];
        }

        let prefix = prefix.trim_matches('/');
        if prefix.is_empty() {
            prefixes.push(".".to_string());
            continue;
        }

        if prefix.split('/').any(|c| c == "..") {
            continue;
        }

        prefixes.push(prefix.to_string());
    }

    prefixes.sort();
    prefixes.dedup();
    prefixes
}

pub(crate) fn resolve_landlock_allowlist_paths(
    project_dir: &Path,
    patterns: &[String],
) -> Vec<String> {
    let project_dir_str = project_dir.to_string_lossy();
    let mut out: Vec<String> = Vec::new();

    for raw_pattern in patterns {
        let pattern = raw_pattern.trim();
        if pattern.is_empty() {
            continue;
        }

        let pattern = pattern.trim_start_matches("./");

        // Landlock allowlists are enforced relative to the project root (mirrors the full-isolation
        // mount semantics). Absolute patterns are only honored when they refer to the project dir.
        let rel = if pattern.starts_with('/') {
            if pattern == project_dir_str {
                "*"
            } else if let Some(stripped) = pattern.strip_prefix(&format!("{}/", project_dir_str)) {
                stripped
            } else {
                continue;
            }
        } else {
            pattern
        };

        let rel = rel.trim_start_matches("./");

        if matches!(rel, "*" | "**" | "/*" | "/**") {
            out.push("/project".to_string());
            out.push(project_dir_str.to_string());
            continue;
        }

        let mut prefix = rel;
        if let Some(idx) = rel.find(['*', '?', '[']) {
            prefix = &rel[..idx];
        }

        let prefix = prefix.trim_matches('/');
        if prefix.is_empty() || prefix == "." {
            out.push("/project".to_string());
            out.push(project_dir_str.to_string());
            continue;
        }

        if prefix.split('/').any(|c| c == "..") {
            continue;
        }

        out.push(format!("/project/{prefix}"));
        out.push(format!("{}/{}", project_dir_str, prefix));
    }

    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landlock_helper_src_accepts_substrate_world_agent_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let candidate = tmp.path().join("substrate-world-agent");
        std::fs::write(&candidate, b"").expect("write");

        let resolved = resolve_landlock_helper_src_from_exe(&candidate);
        assert_eq!(resolved.as_deref(), Some(candidate.as_path()));
    }

    #[test]
    fn test_budget_tracker() {
        let budget = Budget {
            max_execs: Some(5),
            max_runtime_ms: Some(10000),
            max_egress_bytes: Some(1000000),
        };

        let tracker = AgentBudgetTracker::new("test-agent", budget);

        assert!(tracker.can_execute());

        // Use up all executions
        for _ in 0..5 {
            tracker.decrement_exec();
        }

        assert!(!tracker.can_execute());
    }

    #[test]
    fn test_execute_response_serde_fs_diff_roundtrip() {
        let resp = agent_api_types::ExecuteResponse {
            exit: 0,
            span_id: "spn_test".to_string(),
            stdout_b64: BASE64.encode(b"ok"),
            stderr_b64: BASE64.encode(b""),
            scopes_used: vec!["tcp:example.com:443".to_string()],
            fs_diff: Some(substrate_common::FsDiff {
                writes: vec![std::path::PathBuf::from("/tmp/a.txt")],
                mods: vec![],
                deletes: vec![],
                truncated: false,
                tree_hash: None,
                display_path: None,
                summary: None,
            }),
            process_telemetry: ProcessTelemetry {
                process_events: vec![substrate_common::ProcessEvent {
                    event_type: substrate_common::ProcessEventType::WorldProcessStart,
                    ts: "2026-04-01T00:00:00Z".to_string(),
                    ts_unix_ns: 1_743_465_600_000_000_000,
                    session_id: "ses_test".to_string(),
                    world_id: "wld_test".to_string(),
                    pid: 42,
                    ppid: 1,
                    cwd: "/tmp".to_string(),
                    parent_span: "spn_test".to_string(),
                    parent_cmd_id: Some("cmd_test".to_string()),
                    argv: None,
                    argv_omitted: Some(true),
                    exe: None,
                    exit_code: None,
                    signal: None,
                    duration_ms: None,
                    env: None,
                }],
                process_events_status: substrate_common::ProcessEventsStatus::Truncated,
                process_events_reason: Some("capture_overflow".to_string()),
                process_events_dropped: Some(2),
                process_events_max: None,
                process_events_backend: None,
                process_events_error: None,
            },
        };

        let json = serde_json::to_string(&resp).expect("serialize ExecuteResponse");
        let back: agent_api_types::ExecuteResponse =
            serde_json::from_str(&json).expect("deserialize ExecuteResponse");

        assert_eq!(back.exit, 0);
        assert_eq!(back.span_id, "spn_test");
        assert_eq!(back.scopes_used, vec!["tcp:example.com:443".to_string()]);
        assert_eq!(
            back.process_telemetry.process_events_status,
            substrate_common::ProcessEventsStatus::Truncated
        );
        assert_eq!(
            back.process_telemetry.process_events_reason.as_deref(),
            Some("capture_overflow")
        );
        assert_eq!(back.process_telemetry.process_events_dropped, Some(2));
        assert_eq!(back.process_telemetry.process_events.len(), 1);
        let fd = back.fs_diff.expect("fs_diff present");
        assert_eq!(fd.writes.len(), 1);
        assert_eq!(fd.writes[0], std::path::PathBuf::from("/tmp/a.txt"));
        assert!(fd.mods.is_empty());
        assert!(fd.deletes.is_empty());
    }

    #[test]
    fn process_telemetry_redaction_helpers_redact_common_secret_shapes() {
        let argv = vec![
            "curl".to_string(),
            "-H".to_string(),
            "Authorization: Bearer abc123".to_string(),
            "--token".to_string(),
            "secret".to_string(),
            "--password=supersecret".to_string(),
        ];

        let redacted = substrate_common::redact_process_argv(&argv);
        assert_eq!(redacted[2], "Authorization: Bearer ***");
        assert_eq!(redacted[4], "***");
        assert_eq!(redacted[5], "--password=***");

        assert!(substrate_common::process_env_key_allowlisted("HTTP_PROXY"));
        assert!(!substrate_common::process_env_key_allowlisted(
            "AWS_SECRET_ACCESS_KEY"
        ));
        assert_eq!(
            substrate_common::redact_process_env_value(
                "HTTP_PROXY",
                "http://user:pass@example.com"
            ),
            "http://***@example.com"
        );
    }

    #[test]
    fn world_deps_provision_profile_wraps_command_in_transient_systemd_unit() {
        let cmd = wrap_command_for_profile(
            Some("world-deps-provision"),
            Path::new("/tmp/substrate world"),
            "echo 'hello'",
        );

        assert!(cmd.starts_with("systemd-run "));
        assert!(cmd.contains("--wait"));
        assert!(cmd.contains("--pipe"));
        assert!(cmd.contains("--collect"));
        assert!(cmd.contains("--working-directory='/tmp/substrate world'"));
        assert!(cmd.contains("/bin/sh -lc 'echo '\"'\"'hello'\"'\"''"));
    }

    #[test]
    fn non_provision_profiles_do_not_wrap_command() {
        let cmd = wrap_command_for_profile(
            Some("world-deps-probe"),
            Path::new("/tmp/substrate world"),
            "dpkg-query -W",
        );
        assert_eq!(cmd, "dpkg-query -W");

        let cmd = wrap_command_for_profile(None, Path::new("/tmp"), "echo ok");
        assert_eq!(cmd, "echo ok");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_resolve_helper_src_prefers_sibling_world_agent_over_test_harness() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let deps = tmp.path().join("target").join("debug").join("deps");
        std::fs::create_dir_all(&deps).expect("create deps");

        let harness = deps.join("wfgad3_wildcard_deny_symlink_handling-abcdef");
        std::fs::write(&harness, b"").expect("write harness");

        let world_agent = deps.parent().unwrap().join("world-agent");
        std::fs::write(&world_agent, b"").expect("write world-agent");

        let resolved =
            resolve_landlock_helper_src_from_exe(&harness).expect("resolve from harness exe");
        assert_eq!(resolved, world_agent);
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_service_creation() {
        let service = WorldAgentService::new().unwrap();
        assert_eq!(service.worlds.read().unwrap().len(), 0);
        assert!(!service.last_netfilter_requested());
        assert!(service.last_netfilter_failure_reason().is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn record_doctor_request_context_updates_requested_state_for_non_pty_requests() {
        let service = WorldAgentService::new().expect("service");

        service.record_doctor_request_context(
            agent_api_types::PolicyResolutionModeV1::SnapshotV3,
            true,
        );

        assert_eq!(
            service.last_policy_resolution_mode(),
            Some(agent_api_types::PolicyResolutionModeV1::SnapshotV3)
        );
        assert!(service.last_netfilter_requested());

        service.record_doctor_request_context(
            agent_api_types::PolicyResolutionModeV1::LegacyLocal,
            false,
        );

        assert_eq!(
            service.last_policy_resolution_mode(),
            Some(agent_api_types::PolicyResolutionModeV1::LegacyLocal)
        );
        assert!(!service.last_netfilter_requested());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn record_last_netfilter_failure_for_error_tracks_only_published_failure_classes() {
        let service = WorldAgentService::new().expect("service");
        let known_errors = [
            anyhow::anyhow!(NETFILTER_ENABLE_REQUIRED_TEXT),
            anyhow::anyhow!("nft command failed: permission denied"),
            anyhow::anyhow!("failed to resolve allowed domain `example.com`: dns lookup failed"),
            anyhow::anyhow!("allowed domain `example.com` resolved to no addresses"),
            anyhow::anyhow!(
                "project bind mount helper refused isolated execution before command start: substrate: error: project_bind_mount: cgroup attach failed: /sys/fs/cgroup/substrate/world/cgroup.procs"
            ),
        ];

        for err in &known_errors {
            service.set_last_netfilter_failure_reason(None);
            service.record_last_netfilter_failure_for_error(true, err);
            assert_eq!(
                service.last_netfilter_failure_reason(),
                Some(format!("{err:#}"))
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn record_last_netfilter_failure_for_error_ignores_unrelated_or_unrequested_errors() {
        let service = WorldAgentService::new().expect("service");
        service.record_last_netfilter_failure_for_error(
            true,
            &anyhow::anyhow!("plain command failed"),
        );
        assert!(service.last_netfilter_failure_reason().is_none());

        service.record_last_netfilter_failure_for_error(
            false,
            &anyhow::anyhow!(NETFILTER_ENABLE_REQUIRED_TEXT),
        );
        assert!(service.last_netfilter_failure_reason().is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn clear_last_netfilter_failure_on_success_only_resets_isolated_runs() {
        let service = WorldAgentService::new().expect("service");
        service.set_last_netfilter_failure_reason(Some("nft command failed: denied".to_string()));

        service.clear_last_netfilter_failure_on_success(false);
        assert_eq!(
            service.last_netfilter_failure_reason(),
            Some("nft command failed: denied".to_string())
        );

        service.clear_last_netfilter_failure_on_success(true);
        assert!(service.last_netfilter_failure_reason().is_none());
    }

    #[test]
    fn resolve_project_dir_prefers_internal_override() {
        let mut env = HashMap::new();
        env.insert(
            WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
            "/tmp/substrate-project-root".to_string(),
        );
        env.insert(ANCHOR_MODE_ENV.to_string(), "follow-cwd".to_string());
        env.insert(
            ANCHOR_PATH_ENV.to_string(),
            "/tmp/ignored-anchor".to_string(),
        );

        let project_dir =
            resolve_project_dir(Some(&env), Some(Path::new("/tmp/overridden-cwd"))).unwrap();

        assert_eq!(project_dir, PathBuf::from("/tmp/substrate-project-root"));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_service_creation_fails_on_non_linux() {
        let result = WorldAgentService::new();
        assert!(result.is_err());
    }
}
