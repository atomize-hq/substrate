//! Core service implementation for world agent.

#[cfg(target_os = "linux")]
use agent_api_types::ExecuteStreamFrame;
use agent_api_types::{Budget, ExecuteRequest, ExecuteResponse};
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
use futures_util::StreamExt;
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use substrate_common::WorldRootMode;
#[cfg(target_os = "linux")]
use tokio::task;
#[cfg(target_os = "linux")]
use tokio_stream::wrappers::UnboundedReceiverStream;
#[cfg(target_os = "linux")]
use world::stream::{install_stream_sink, StreamKind, StreamSink};
use world_api::{WorldBackend, WorldHandle, WorldSpec};

/// Main service running inside the world.
#[derive(Clone)]
pub struct WorldAgentService {
    backend: Arc<dyn WorldBackend>,
    #[allow(dead_code)]
    worlds: Arc<RwLock<HashMap<String, WorldHandle>>>,
    budgets: Arc<RwLock<HashMap<String, AgentBudgetTracker>>>,
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
        let backend = Self::create_backend()?;

        Ok(Self {
            backend,
            worlds: Arc::new(RwLock::new(HashMap::new())),
            budgets: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Ensure a session world (thin wrapper over backend)
    #[cfg(target_os = "linux")]
    pub fn ensure_session_world(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        self.backend.ensure_session(spec)
    }

    #[cfg(target_os = "linux")]
    fn create_backend() -> Result<Arc<dyn WorldBackend>> {
        use world::LinuxLocalBackend;
        Ok(Arc::new(LinuxLocalBackend::new()))
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

        // Apply and track budget
        if let Some(budget) = req.budget {
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
        let project_dir = resolve_project_dir(req.env.as_ref(), Some(&cwd))?;

        // Create world spec from request
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: substrate_broker::allowed_domains(),
            project_dir,
            // For agent non-PTY path, prefer consistent fs_diff collection
            // to enable immediate span enrichment in the shell.
            always_isolate: true,
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
        let exec_req = world_api::ExecRequest {
            cmd: req.cmd,
            cwd,
            env: req.env.unwrap_or_default(),
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
        let project_dir = resolve_project_dir(req.env.as_ref(), Some(&cwd))?;

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: substrate_broker::allowed_domains(),
            project_dir,
            always_isolate: true,
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
            env: req.env.clone().unwrap_or_default(),
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

pub(crate) fn resolve_project_dir(
    env: Option<&HashMap<String, String>>,
    cwd: Option<&Path>,
) -> Result<PathBuf> {
    let cwd_path = cwd
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let mode = if let Some(value) = env.and_then(|map| map.get("SUBSTRATE_WORLD_ROOT_MODE")) {
        WorldRootMode::parse(value)
            .ok_or_else(|| anyhow!("invalid SUBSTRATE_WORLD_ROOT_MODE value: {}", value))?
    } else {
        WorldRootMode::Project
    };

    let root_path = env
        .and_then(|map| map.get("SUBSTRATE_WORLD_ROOT_PATH"))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    let base_dir = match mode {
        WorldRootMode::Project => root_path.unwrap_or_else(|| cwd_path.clone()),
        WorldRootMode::FollowCwd => cwd_path.clone(),
        WorldRootMode::Custom => root_path.ok_or_else(|| {
            anyhow!("world root mode 'custom' requires SUBSTRATE_WORLD_ROOT_PATH")
        })?,
    };

    Ok(base_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

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
