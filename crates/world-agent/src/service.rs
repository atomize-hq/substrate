//! Core service implementation for world agent.

#[cfg(target_os = "linux")]
use agent_api_types::ExecuteStreamFrame;
use agent_api_types::{
    Budget, ExecuteRequest, ExecuteResponse, PendingDiffClearRequestV1, PendingDiffClearResponseV1,
    PendingDiffRecordV1, PendingDiffRequestV1, WorldFsReadRequestV1, WorldFsReadResponseV1,
};
#[cfg(target_os = "linux")]
use agent_api_types::{PendingDiffBucketV1, WorldFsEntryTypeV1};
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
#[cfg(target_os = "linux")]
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

pub(crate) const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
pub(crate) const ANCHOR_PATH_ENV: &str = "SUBSTRATE_ANCHOR_PATH";
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
    pending_diff_origin: Arc<RwLock<HashMap<String, PendingDiffOriginTracker>>>,
    #[allow(dead_code)]
    worlds: Arc<RwLock<HashMap<String, WorldHandle>>>,
    budgets: Arc<RwLock<HashMap<String, AgentBudgetTracker>>>,
    last_policy_resolution_mode: Arc<AtomicU8>,
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
                pending_diff_origin: Arc::new(RwLock::new(HashMap::new())),
                worlds: Arc::new(RwLock::new(HashMap::new())),
                budgets: Arc::new(RwLock::new(HashMap::new())),
                last_policy_resolution_mode: Arc::new(AtomicU8::new(0)),
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
        let (policy_resolution_mode, policy_inputs) =
            resolve_policy_inputs(&req, &cwd, &project_dir)?;
        self.set_last_policy_resolution_mode(policy_resolution_mode);

        let PolicyInputs {
            fs_mode,
            isolation_full,
            allowed_domains,
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        } = policy_inputs;

        // Create world spec from request
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains,
            project_dir,
            // For agent non-PTY path, prefer consistent fs_diff collection
            // to enable immediate span enrichment in the shell.
            always_isolate,
            fs_mode,
        };

        // Ensure world exists
        let world = match self.backend.ensure_session(&spec) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                return Err(anyhow::anyhow!("Failed to ensure session world"));
            }
        };

        // Prepare execution request
        let mut env_map = req.env.unwrap_or_default();
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
            cmd: req.cmd,
            cwd,
            env: env_map,
            pty: req.pty,
            span_id: None,
        };

        // Execute command
        let result = match self.backend.exec(&world, exec_req) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, "exec failed");
                return Err(anyhow::anyhow!("Command execution failed"));
            }
        };

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

            let snapshot = req
                .policy_snapshot
                .canonicalize()
                .map_err(BadRequestError::new)?;
            let fs_mode = if snapshot.world_fs.write.enabled {
                WorldFsMode::Writable
            } else {
                WorldFsMode::ReadOnly
            };

            let always_isolate = !matches!(req.profile.as_deref(), Some("world-deps-provision"));

            let spec = WorldSpec {
                reuse_session: true,
                isolate_network: true,
                limits: world_api::ResourceLimits::default(),
                enable_preload: false,
                allowed_domains: substrate_broker::allowed_domains(),
                project_dir,
                always_isolate,
                fs_mode,
            };

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

            let mut hasher = Sha256::new();
            hasher.update(b"writes\0");
            for item in &snapshot.writes {
                hasher.update(item.as_bytes());
                hasher.update(b"\n");
            }
            hasher.update(b"mods\0");
            for item in &snapshot.mods {
                hasher.update(item.as_bytes());
                hasher.update(b"\n");
            }
            hasher.update(b"deletes\0");
            for item in &snapshot.deletes {
                hasher.update(item.as_bytes());
                hasher.update(b"\n");
            }
            let digest = hasher.finalize();
            let mut hex = String::with_capacity(digest.len() * 2);
            for b in digest {
                use std::fmt::Write as _;
                let _ = write!(&mut hex, "{:02x}", b);
            }

            let started_at: chrono::DateTime<chrono::Utc> = started_at.into();
            let session_started_at =
                started_at.to_rfc3339_opts(SecondsFormat::Secs, /* use_z */ true);

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

            let snapshot = req
                .policy_snapshot
                .canonicalize()
                .map_err(BadRequestError::new)?;
            let fs_mode = if snapshot.world_fs.write.enabled {
                WorldFsMode::Writable
            } else {
                WorldFsMode::ReadOnly
            };

            let always_isolate = !matches!(req.profile.as_deref(), Some("world-deps-provision"));

            let spec = WorldSpec {
                reuse_session: true,
                isolate_network: true,
                limits: world_api::ResourceLimits::default(),
                enable_preload: false,
                allowed_domains: substrate_broker::allowed_domains(),
                project_dir,
                always_isolate,
                fs_mode,
            };

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

            let mut hasher = Sha256::new();
            hasher.update(b"writes\0");
            for item in &writes {
                hasher.update(item.as_bytes());
                hasher.update(b"\n");
            }
            hasher.update(b"mods\0");
            for item in &mods {
                hasher.update(item.as_bytes());
                hasher.update(b"\n");
            }
            hasher.update(b"deletes\0");
            for item in &deletes {
                hasher.update(item.as_bytes());
                hasher.update(b"\n");
            }
            let digest = hasher.finalize();
            let mut current_hex = String::with_capacity(digest.len() * 2);
            for b in digest {
                use std::fmt::Write as _;
                let _ = write!(&mut current_hex, "{:02x}", b);
            }

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

            let snapshot = req
                .policy_snapshot
                .canonicalize()
                .map_err(BadRequestError::new)?;
            let fs_mode = if snapshot.world_fs.write.enabled {
                WorldFsMode::Writable
            } else {
                WorldFsMode::ReadOnly
            };

            let always_isolate = !matches!(req.profile.as_deref(), Some("world-deps-provision"));

            let spec = WorldSpec {
                reuse_session: true,
                isolate_network: true,
                limits: world_api::ResourceLimits::default(),
                enable_preload: false,
                allowed_domains: substrate_broker::allowed_domains(),
                project_dir,
                always_isolate,
                fs_mode,
            };

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
        let (policy_resolution_mode, policy_inputs) =
            resolve_policy_inputs(&req, &cwd, &project_dir)?;
        self.set_last_policy_resolution_mode(policy_resolution_mode);

        let PolicyInputs {
            fs_mode,
            isolation_full,
            allowed_domains,
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        } = policy_inputs;
        let always_isolate = should_always_isolate(&req);

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains,
            project_dir,
            always_isolate,
            fs_mode,
        };

        let world = match self.backend.ensure_session(&spec) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                anyhow::bail!("Failed to ensure session world");
            }
        };

        let mut exec_req = world_api::ExecRequest {
            cmd: req.cmd.clone(),
            cwd: cwd.clone(),
            env: {
                let mut env_map = req.env.clone().unwrap_or_default();
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

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(ExecuteStreamFrame::Start {
            span_id: span_id.clone(),
        });

        let backend = self.backend.clone();
        let agent_id = req.agent_id.clone();
        task::spawn_blocking(move || {
            let sink = Arc::new(StreamingSink::new(tx.clone()));
            let guard = install_stream_sink(sink);
            let result = backend.exec(&world, exec_req);
            drop(guard);

            match result {
                Ok(exec_result) => {
                    if let (Some(primary), Some(final_strategy), Some(reason)) = (
                        exec_result.world_fs_strategy_primary,
                        exec_result.world_fs_strategy_final,
                        exec_result.world_fs_strategy_fallback_reason,
                    ) {
                        let _ = tx.send(ExecuteStreamFrame::Event {
                            event: AgentEvent {
                                ts: chrono::Utc::now(),
                                agent_id: agent_id.clone(),
                                project: None,
                                kind: AgentEventKind::Status,
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
                    };
                    let _ = tx.send(frame);
                }
                Err(e) => {
                    tracing::error!(error = %e, agent = agent_id, "exec failed");
                    let _ = tx.send(ExecuteStreamFrame::Error {
                        message: e.to_string(),
                    });
                }
            }
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
    // `world deps provision` is explicitly intended to mutate guest system packages (apt/dpkg),
    // which is incompatible with the default cage's write restrictions. Use a request profile to
    // opt out of `always_isolate` for that explicit provisioning workflow.
    !matches!(req.profile.as_deref(), Some("world-deps-provision"))
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
    allowed_domains: Vec<String>,
    write_allowlist_prefixes: Vec<String>,
    landlock_discover_paths: Vec<String>,
    landlock_read_paths: Vec<String>,
    landlock_write_paths: Vec<String>,
    enforcement_plan_b64: Option<String>,
}

fn resolve_policy_inputs(
    req: &ExecuteRequest,
    _cwd: &Path,
    project_dir: &Path,
) -> Result<(agent_api_types::PolicyResolutionModeV1, PolicyInputs)> {
    use agent_api_types::PolicyResolutionModeV1;

    let snapshot = req
        .policy_snapshot
        .canonicalize()
        .map_err(BadRequestError::new)?;

    let isolation_full = !snapshot.world_fs.host_visible;
    let fs_mode = if snapshot.world_fs.write.enabled {
        WorldFsMode::Writable
    } else {
        WorldFsMode::ReadOnly
    };

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
            allowed_domains: substrate_broker::allowed_domains(),
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        },
    ))
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
}

pub(crate) fn resolve_project_dir(
    env: Option<&HashMap<String, String>>,
    cwd: Option<&Path>,
) -> Result<PathBuf> {
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
        };

        let json = serde_json::to_string(&resp).expect("serialize ExecuteResponse");
        let back: agent_api_types::ExecuteResponse =
            serde_json::from_str(&json).expect("deserialize ExecuteResponse");

        assert_eq!(back.exit, 0);
        assert_eq!(back.span_id, "spn_test");
        assert_eq!(back.scopes_used, vec!["tcp:example.com:443".to_string()]);
        let fd = back.fs_diff.expect("fs_diff present");
        assert_eq!(fd.writes.len(), 1);
        assert_eq!(fd.writes[0], std::path::PathBuf::from("/tmp/a.txt"));
        assert!(fd.mods.is_empty());
        assert!(fd.deletes.is_empty());
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
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_service_creation_fails_on_non_linux() {
        let result = WorldAgentService::new();
        assert!(result.is_err());
    }
}
