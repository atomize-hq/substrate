use crate::paths::{normalize_diff, to_wsl_path};
use crate::transport::{detect_tcp_forwarder, DEFAULT_AGENT_PIPE, DEFAULT_DISTRO};
use crate::warm::WarmCmd;
use agent_api_client::{AgentClient, Transport};
use agent_api_types::{ExecuteRequest, ExecuteResponse, WorldFsMode};
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use substrate_common::FsDiff;
use tokio::runtime::{self, Runtime};
use tracing::{debug, warn};
use uuid::Uuid;
use world_api::{ExecRequest, ExecResult, WorldBackend, WorldHandle, WorldSpec};

#[cfg(test)]
pub(crate) trait AgentApiMock: Send + Sync {
    fn capabilities(&self) -> Result<Value>;
    fn execute(&self, request: ExecuteRequest) -> Result<ExecuteResponse>;
    fn get_trace(&self, span_id: &str) -> Result<Value>;
}

/// Windows backend delegating to world-agent inside WSL.
pub struct WindowsWslBackend {
    pub(crate) distro: String,
    pub(crate) project_path: PathBuf,
    pub(crate) agent_pipe: PathBuf,
    pub(crate) forwarder_tcp: Option<(String, u16)>,
    pub(crate) agent_id: String,
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) warm_cmd: WarmCmd,
    pub(crate) session_cache: Mutex<Option<WorldHandle>>,
    #[cfg(test)]
    pub(crate) agent_override: Option<Arc<dyn AgentApiMock>>,
}

impl WindowsWslBackend {
    /// Create backend using environment defaults.
    pub fn new() -> Result<Self> {
        let distro =
            std::env::var("SUBSTRATE_WSL_DISTRO").unwrap_or_else(|_| DEFAULT_DISTRO.to_string());
        let project_path = std::env::var("SUBSTRATE_PROJECT_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().expect("current directory"));
        let warm_cmd = WarmCmd::enabled(distro.clone(), project_path.clone());
        Self::build(distro, project_path, warm_cmd)
    }

    #[cfg(test)]
    pub(crate) fn with_mock_agent(
        distro: String,
        project_path: PathBuf,
        warm_cmd: WarmCmd,
        agent: Arc<dyn AgentApiMock>,
    ) -> Result<Self> {
        let mut backend = Self::build(distro, project_path, warm_cmd)?;
        backend.agent_override = Some(agent);
        Ok(backend)
    }

    fn build(distro: String, project_path: PathBuf, warm_cmd: WarmCmd) -> Result<Self> {
        let agent_pipe = std::env::var("SUBSTRATE_FORWARDER_PIPE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_AGENT_PIPE));

        let forwarder_tcp = detect_tcp_forwarder()?;

        let agent_id =
            std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "world-windows-wsl".to_string());
        let runtime = Arc::new(
            runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("failed to construct tokio runtime")?,
        );

        Ok(Self {
            distro,
            project_path,
            agent_pipe,
            forwarder_tcp,
            agent_id,
            runtime,
            warm_cmd,
            session_cache: Mutex::new(None),
            #[cfg(test)]
            agent_override: None,
        })
    }

    fn block_on<F, T>(&self, fut: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
        } else {
            self.runtime.block_on(fut)
        }
    }

    pub fn agent_transport(&self) -> Transport {
        if let Some((host, port)) = &self.forwarder_tcp {
            Transport::Tcp {
                host: host.clone(),
                port: *port,
            }
        } else {
            Transport::NamedPipe {
                path: self.agent_pipe.clone(),
            }
        }
    }

    pub fn build_agent_client(&self) -> Result<AgentClient> {
        AgentClient::new(self.agent_transport())
    }

    fn capabilities(&self) -> Result<Value> {
        #[cfg(test)]
        if let Some(mock) = &self.agent_override {
            return mock.capabilities();
        }

        let client = AgentClient::new(self.agent_transport())?;
        self.block_on(async { client.capabilities().await })
    }

    fn execute_agent(&self, request: ExecuteRequest) -> Result<ExecuteResponse> {
        #[cfg(test)]
        if let Some(mock) = &self.agent_override {
            return mock.execute(request);
        }

        let client = AgentClient::new(self.agent_transport())?;
        self.block_on(async move { client.execute(request).await })
    }

    fn fetch_trace(&self, span_id: &str) -> Result<Value> {
        #[cfg(test)]
        if let Some(mock) = &self.agent_override {
            return mock.get_trace(span_id);
        }

        let client = AgentClient::new(self.agent_transport())?;
        self.block_on(async move { client.get_trace(span_id).await })
    }

    fn ensure_agent_ready(&self) -> Result<()> {
        match self.capabilities() {
            Ok(_) => Ok(()),
            Err(initial_err) => {
                warn!(
                    target: "world_windows_wsl::backend",
                    error = %initial_err,
                    "Agent capabilities check failed; invoking warm script"
                );
                self.warm_cmd
                    .run()
                    .context("wsl warm script failed to execute")?;

                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                let mut last_err: Option<anyhow::Error> = None;
                while std::time::Instant::now() < deadline {
                    match self.capabilities() {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            last_err = Some(e);
                            std::thread::sleep(std::time::Duration::from_millis(200));
                        }
                    }
                }
                Err(last_err.unwrap_or_else(|| anyhow!("capabilities check failed after warm")))
            }
        }
    }

    fn convert_exec_request(&self, req: &ExecRequest) -> Result<ExecuteRequest> {
        let cwd = to_wsl_path(&self.project_path, &req.cwd)?;
        let env = if req.env.is_empty() {
            None
        } else {
            Some(req.env.clone())
        };

        Ok(ExecuteRequest {
            profile: None,
            cmd: req.cmd.clone(),
            cwd: Some(cwd),
            env,
            pty: req.pty,
            agent_id: self.agent_id.clone(),
            budget: None,
            world_fs_mode: Some(self.resolve_fs_mode()),
        })
    }

    fn convert_exec_response(&self, resp: ExecuteResponse) -> ExecResult {
        let stdout = BASE64_STANDARD
            .decode(&resp.stdout_b64)
            .unwrap_or_else(|_| resp.stdout_b64.clone().into_bytes());
        let stderr = BASE64_STANDARD
            .decode(&resp.stderr_b64)
            .unwrap_or_else(|_| resp.stderr_b64.clone().into_bytes());

        let mut result = ExecResult {
            exit: resp.exit,
            stdout,
            stderr,
            scopes_used: resp.scopes_used,
            fs_diff: resp.fs_diff,
        };
        if let Some(ref mut diff) = result.fs_diff {
            normalize_diff(diff);
        }
        result
    }

    fn generate_world_handle(&self) -> WorldHandle {
        WorldHandle {
            id: format!("wsl:{}:{}", self.distro, Uuid::now_v7()),
        }
    }

    fn resolve_fs_mode(&self) -> WorldFsMode {
        std::env::var("SUBSTRATE_WORLD_FS_MODE")
            .ok()
            .and_then(|value| WorldFsMode::parse(&value))
            .unwrap_or(WorldFsMode::Writable)
    }
}

impl WorldBackend for WindowsWslBackend {
    fn ensure_session(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        if spec.reuse_session {
            if let Some(handle) = self
                .session_cache
                .lock()
                .expect("session cache poisoned")
                .clone()
            {
                self.ensure_agent_ready()?;
                return Ok(handle);
            }
        }

        self.ensure_agent_ready()?;

        let handle = self.generate_world_handle();
        if spec.reuse_session {
            let mut cache = self.session_cache.lock().expect("session cache poisoned");
            *cache = Some(handle.clone());
        }
        Ok(handle)
    }

    fn exec(&self, world: &WorldHandle, req: ExecRequest) -> Result<ExecResult> {
        debug!(
            target: "world_windows_wsl::backend",
            world_id = %world.id,
            command = %req.cmd,
            "executing command via WSL backend"
        );
        let agent_request = self.convert_exec_request(&req)?;
        let response = self.execute_agent(agent_request)?;
        Ok(self.convert_exec_response(response))
    }

    fn fs_diff(&self, _world: &WorldHandle, span_id: &str) -> Result<FsDiff> {
        let trace = self.fetch_trace(span_id)?;
        if let Some(fs_diff) = trace.get("fs_diff") {
            let mut diff: FsDiff = serde_json::from_value(fs_diff.clone())
                .context("failed to deserialize fs_diff from trace")?;
            normalize_diff(&mut diff);
            Ok(diff)
        } else {
            Ok(FsDiff::default())
        }
    }

    fn apply_policy(&self, _world: &WorldHandle, _spec: &WorldSpec) -> Result<()> {
        Ok(())
    }
}

impl Drop for WindowsWslBackend {
    fn drop(&mut self) {
        if tokio::runtime::Handle::try_current().is_ok() {
            let rt = self.runtime.clone();
            let _ = std::thread::spawn(move || drop(rt)).join();
        }
    }
}
