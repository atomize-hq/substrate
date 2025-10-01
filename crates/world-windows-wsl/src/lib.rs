#![cfg(target_os = "windows")]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use agent_api_client::{AgentClient, Transport};
use agent_api_types::{ExecuteRequest, ExecuteResponse};
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde_json::Value;
use substrate_common::FsDiff;
use tokio::runtime::{self, Runtime};
use tracing::{debug, info, warn};
use uuid::Uuid;
use world_api::{ExecRequest, ExecResult, WorldBackend, WorldHandle, WorldSpec};

const DEFAULT_DISTRO: &str = "substrate-wsl";
const DEFAULT_AGENT_PIPE: &str = r"\\.\pipe\substrate-agent";
const DEFAULT_TCP_ADDR: &str = "127.0.0.1";
const DEFAULT_TCP_PORT: u16 = 17788;

/// Windows backend delegating to world-agent inside WSL.
pub struct WindowsWslBackend {
    distro: String,
    project_path: PathBuf,
    agent_pipe: PathBuf,
    forwarder_tcp: Option<(String, u16)>,
    agent_id: String,
    runtime: Arc<Runtime>,
    warm_cmd: WarmCmd,
    session_cache: Mutex<Option<WorldHandle>>,
    #[cfg(test)]
    agent_override: Option<Arc<dyn AgentApiMock>>,
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
    fn with_mock_agent(
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

    pub fn build_agent_client(&self) -> AgentClient {
        AgentClient::new(self.agent_transport())
    }

    fn capabilities(&self) -> Result<Value> {
        #[cfg(test)]
        if let Some(mock) = &self.agent_override {
            return mock.capabilities();
        }

        let client = AgentClient::new(self.agent_transport());
        self.block_on(async { client.capabilities().await })
    }

    fn execute_agent(&self, request: ExecuteRequest) -> Result<ExecuteResponse> {
        #[cfg(test)]
        if let Some(mock) = &self.agent_override {
            return mock.execute(request);
        }

        let client = AgentClient::new(self.agent_transport());
        self.block_on(async move { client.execute(request).await })
    }

    fn fetch_trace(&self, span_id: &str) -> Result<Value> {
        #[cfg(test)]
        if let Some(mock) = &self.agent_override {
            return mock.get_trace(span_id);
        }

        let client = AgentClient::new(self.agent_transport());
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
                self.capabilities().map(|_| ())
            }
        }
    }

    fn convert_exec_request(&self, req: &ExecRequest) -> Result<ExecuteRequest> {
        let cwd = self.to_wsl_path(&req.cwd)?;
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
            self.normalize_diff(diff);
        }
        result
    }

    fn to_wsl_path(&self, path: &Path) -> Result<String> {
        if path.is_relative() {
            let joined = self.project_path.join(path);
            return self.to_wsl_path(joined.as_path());
        }

        let raw = path
            .to_str()
            .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))?;
        let normalized = raw.replace('\\', "/");
        if let Some((drive, rest)) = normalized.split_once(':') {
            let rest = rest.trim_start_matches('/');
            Ok(format!("/mnt/{}/{}", drive.to_lowercase(), rest))
        } else {
            Ok(normalized)
        }
    }

    fn to_windows_display_path(&self, path: &Path) -> Option<String> {
        let raw = path.to_str()?;
        let stripped = raw.strip_prefix("/mnt/")?;
        if let Some((prefix, rest)) = stripped.split_once('/') {
            if prefix.eq_ignore_ascii_case("unc") {
                if rest.is_empty() {
                    return None;
                }
                let sep = std::path::MAIN_SEPARATOR.to_string();
                let converted = rest.replace('/', sep.as_str());
                return Some(format!("\\{}", converted));
            }

            if prefix.len() == 1 {
                let drive = prefix.chars().next()?.to_ascii_uppercase();
                let sep = std::path::MAIN_SEPARATOR.to_string();
                let converted = rest.replace('/', sep.as_str());
                if converted.is_empty() {
                    return Some(format!("{drive}:{sep}", sep = std::path::MAIN_SEPARATOR));
                }
                return Some(format!(
                    "{drive}:{sep}{converted}",
                    sep = std::path::MAIN_SEPARATOR,
                    converted = converted
                ));
            }
        } else if stripped.len() == 1 {
            let drive = stripped.chars().next()?.to_ascii_uppercase();
            return Some(format!("{drive}:{sep}", sep = std::path::MAIN_SEPARATOR));
        }

        None
    }

    fn normalize_diff(&self, diff: &mut FsDiff) {
        let mut display = HashMap::new();
        for path in diff
            .writes
            .iter()
            .chain(diff.mods.iter())
            .chain(diff.deletes.iter())
        {
            if let Some(display_path) = self.to_windows_display_path(path) {
                display.insert(path.to_string_lossy().to_string(), display_path);
            }
        }

        diff.display_path = if display.is_empty() {
            None
        } else {
            Some(display)
        };
    }

    fn generate_world_handle(&self) -> WorldHandle {
        WorldHandle {
            id: format!("wsl:{}:{}", self.distro, Uuid::now_v7()),
        }
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
            self.normalize_diff(&mut diff);
            Ok(diff)
        } else {
            Ok(FsDiff::default())
        }
    }

    fn apply_policy(&self, _world: &WorldHandle, _spec: &WorldSpec) -> Result<()> {
        // Policy support will be implemented in a later phase.
        Ok(())
    }
}

impl Drop for WindowsWslBackend {
    fn drop(&mut self) {
        // Drop runtime on a separate thread if already inside a runtime.
        if tokio::runtime::Handle::try_current().is_ok() {
            let rt = self.runtime.clone();
            let _ = std::thread::spawn(move || drop(rt)).join();
        }
    }
}

struct WarmCmd {
    distro: String,
    project_path: PathBuf,
    enabled: bool,
    #[cfg(test)]
    invocations: Arc<std::sync::atomic::AtomicUsize>,
}

impl WarmCmd {
    fn enabled(distro: String, project_path: PathBuf) -> Self {
        Self {
            distro,
            project_path,
            enabled: true,
            #[cfg(test)]
            invocations: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    #[cfg(test)]
    fn disabled(
        distro: String,
        project_path: PathBuf,
    ) -> (Self, Arc<std::sync::atomic::AtomicUsize>) {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        (
            Self {
                distro,
                project_path,
                enabled: false,
                invocations: counter.clone(),
            },
            counter,
        )
    }

    fn run(&self) -> Result<()> {
        #[cfg(test)]
        if !self.enabled {
            self.invocations
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Ok(());
        }

        if !self.enabled {
            return Ok(());
        }

        let script_path = self.project_path.join("scripts/windows/wsl-warm.ps1");
        let script = script_path
            .to_str()
            .ok_or_else(|| anyhow!("warm script path contains invalid UTF-8"))?;

        info!(
            target: "world_windows_wsl::backend",
            distro = %self.distro,
            "running wsl warm script"
        );

        let status = Command::new("pwsh")
            .arg("-NoLogo")
            .arg("-File")
            .arg(script)
            .arg("-DistroName")
            .arg(&self.distro)
            .arg("-ProjectPath")
            .arg(&self.project_path)
            .status()
            .context("failed to spawn pwsh for warm script")?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "wsl warm script exited with status {}",
                status.code().unwrap_or(-1)
            ))
        }
    }
}

fn detect_tcp_forwarder() -> Result<Option<(String, u16)>> {
    if let Ok(addr) = std::env::var("SUBSTRATE_FORWARDER_TCP_ADDR") {
        let socket: std::net::SocketAddr = addr
            .parse()
            .context("invalid SUBSTRATE_FORWARDER_TCP_ADDR")?;
        return Ok(Some((socket.ip().to_string(), socket.port())));
    }

    let tcp_enabled = std::env::var("SUBSTRATE_FORWARDER_TCP")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(true);

    if !tcp_enabled {
        return Ok(None);
    }

    let host = std::env::var("SUBSTRATE_FORWARDER_TCP_HOST")
        .unwrap_or_else(|_| DEFAULT_TCP_ADDR.to_string());
    let port = std::env::var("SUBSTRATE_FORWARDER_TCP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_TCP_PORT);
    Ok(Some((host, port)))
}

#[cfg(test)]
pub(crate) trait AgentApiMock: Send + Sync {
    fn capabilities(&self) -> Result<Value>;
    fn execute(&self, request: ExecuteRequest) -> Result<ExecuteResponse>;
    fn get_trace(&self, span_id: &str) -> Result<Value>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_api_types::FsDiff as AgentFsDiff;
    use serde_json::json;
    use std::collections::{HashMap, VecDeque};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockAgent {
        capabilities: Mutex<VecDeque<Result<Value>>>,
        execute: Mutex<VecDeque<Result<ExecuteResponse>>>,
        traces: Mutex<HashMap<String, Value>>,
        requests: Mutex<Vec<ExecuteRequest>>,
        capability_calls: AtomicUsize,
    }

    impl MockAgent {
        fn new() -> Self {
            Self {
                capabilities: Mutex::new(VecDeque::new()),
                execute: Mutex::new(VecDeque::new()),
                traces: Mutex::new(HashMap::new()),
                requests: Mutex::new(Vec::new()),
                capability_calls: AtomicUsize::new(0),
            }
        }

        fn push_capabilities(&self, value: Result<Value>) {
            self.capabilities.lock().unwrap().push_back(value);
        }

        fn push_execute(&self, value: Result<ExecuteResponse>) {
            self.execute.lock().unwrap().push_back(value);
        }

        fn insert_trace(&self, span_id: &str, value: Value) {
            self.traces
                .lock()
                .unwrap()
                .insert(span_id.to_string(), value);
        }

        fn take_requests(&self) -> Vec<ExecuteRequest> {
            self.requests.lock().unwrap().clone()
        }

        fn capability_calls(&self) -> usize {
            self.capability_calls.load(Ordering::SeqCst)
        }
    }

    impl AgentApiMock for MockAgent {
        fn capabilities(&self) -> Result<Value> {
            self.capability_calls.fetch_add(1, Ordering::SeqCst);
            self.capabilities
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Ok(json!({})))
        }

        fn execute(&self, request: ExecuteRequest) -> Result<ExecuteResponse> {
            self.requests.lock().unwrap().push(request);
            self.execute.lock().unwrap().pop_front().unwrap_or_else(|| {
                Ok(ExecuteResponse {
                    exit: 0,
                    span_id: "span".to_string(),
                    stdout_b64: BASE64_STANDARD.encode(b""),
                    stderr_b64: BASE64_STANDARD.encode(b""),
                    scopes_used: vec![],
                    fs_diff: None,
                })
            })
        }

        fn get_trace(&self, span_id: &str) -> Result<Value> {
            self.traces
                .lock()
                .unwrap()
                .get(span_id)
                .cloned()
                .ok_or_else(|| anyhow!("missing trace for span {span_id}"))
        }
    }

    fn test_backend_with_agent() -> (
        WindowsWslBackend,
        Arc<MockAgent>,
        Arc<std::sync::atomic::AtomicUsize>,
    ) {
        let agent = Arc::new(MockAgent::new());
        let (warm_cmd, invocations) =
            WarmCmd::disabled("test-distro".to_string(), PathBuf::from("C:/repo"));
        let backend = WindowsWslBackend::with_mock_agent(
            "test-distro".to_string(),
            PathBuf::from("C:/repo"),
            warm_cmd,
            agent.clone(),
        )
        .expect("backend init");
        (backend, agent, invocations)
    }

    #[test]
    fn ensure_session_reuses_handle() {
        let (backend, agent, warm_invocations) = test_backend_with_agent();
        agent.push_capabilities(Ok(json!({"v": 1})));
        agent.push_capabilities(Ok(json!({"v": 1})));

        let spec = WorldSpec::default();
        let first = backend.ensure_session(&spec).expect("session");
        let second = backend.ensure_session(&spec).expect("session");

        assert_eq!(first.id, second.id);
        assert_eq!(agent.capability_calls(), 2);
        assert_eq!(warm_invocations.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn ensure_session_runs_warm_on_failure() {
        let (backend, agent, warm_invocations) = test_backend_with_agent();
        agent.push_capabilities(Err(anyhow!("pipe missing")));
        agent.push_capabilities(Ok(json!({"ok": true})));

        let spec = WorldSpec::default();
        let handle = backend.ensure_session(&spec).expect("session after warm");
        assert!(handle.id.starts_with("wsl:test-distro:"));
        assert_eq!(warm_invocations.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn exec_routes_to_agent() {
        let (backend, agent, _) = test_backend_with_agent();
        agent.push_capabilities(Ok(json!({})));
        agent.push_execute(Ok(ExecuteResponse {
            exit: 0,
            span_id: "span-123".to_string(),
            stdout_b64: BASE64_STANDARD.encode(b"hello"),
            stderr_b64: BASE64_STANDARD.encode(b""),
            scopes_used: vec!["fs.write:/project".to_string()],
            fs_diff: Some(AgentFsDiff {
                writes: vec![PathBuf::from("/mnt/c/repo/new.txt")],
                ..Default::default()
            }),
        }));

        let spec = WorldSpec::default();
        let world = backend.ensure_session(&spec).expect("session");
        let req = ExecRequest {
            cmd: "echo hello".to_string(),
            cwd: PathBuf::from("C:/repo"),
            env: std::iter::once(("KEY".to_string(), "VALUE".to_string())).collect(),
            pty: false,
            span_id: Some("span-123".to_string()),
        };

        let result = backend.exec(&world, req.clone()).expect("exec result");
        assert_eq!(result.exit, 0);
        assert_eq!(result.stdout, b"hello");
        assert_eq!(result.stderr, b"");
        let diff = result.fs_diff.expect("fs diff");
        let map = diff.display_path.expect("display map");
        assert_eq!(
            map.get("/mnt/c/repo/new.txt"),
            Some(&"C:\\repo\\new.txt".to_string())
        );

        let recorded = agent.take_requests();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].cmd, req.cmd);
        assert_eq!(recorded[0].cwd.as_deref().unwrap(), "/mnt/c/repo");
    }

    #[test]
    fn fs_diff_deserializes() {
        let (backend, agent, _) = test_backend_with_agent();
        agent.push_capabilities(Ok(json!({})));
        let world = backend
            .ensure_session(&WorldSpec::default())
            .expect("session");

        agent.insert_trace(
            "span",
            json!({
                "fs_diff": {
                    "writes": ["/mnt/c/repo/new.txt"],
                    "mods": [],
                    "deletes": []
                }
            }),
        );

        let diff = backend.fs_diff(&world, "span").expect("fs diff");
        assert_eq!(diff.writes.len(), 1);
        assert_eq!(diff.writes[0], PathBuf::from("/mnt/c/repo/new.txt"));
        let display = diff.display_path.expect("display map");
        assert_eq!(
            display.get("/mnt/c/repo/new.txt"),
            Some(&"C:\\repo\\new.txt".to_string())
        );
    }
}
