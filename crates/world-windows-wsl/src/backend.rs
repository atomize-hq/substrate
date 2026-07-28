use crate::paths::{normalize_diff, to_wsl_path};
use crate::warm::WarmCmd;
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde_json::Value;
#[cfg(test)]
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use substrate_common::FsDiff;
use tokio::runtime::{self, Runtime};
use tracing::{debug, warn};
use transport_api_client::{AgentClient, Transport};
use transport_api_types::{
    normalize_windows_install_bootstrap_path, normalize_windows_pipe_path, ExecuteRequest,
    ExecuteResponse, InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1,
    PlatformInstanceIdentityV1, PlatformPrincipalV1, PlatformTransportIdentityV1, PolicySnapshotV3,
    PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
    PolicySnapshotWorldFsWriteV3, ProcessTelemetry, WindowsForwarderScopeV1, WorldFsMode,
};
use uuid::Uuid;
use world_api::{ExecRequest, ExecResult, WorldBackend, WorldHandle, WorldSpec};

#[cfg(test)]
use crate::transport::DEFAULT_AGENT_PIPE;

#[cfg(test)]
pub(crate) trait AgentApiMock: Send + Sync {
    fn capabilities(&self) -> Result<Value>;
    fn execute(&self, request: ExecuteRequest) -> Result<ExecuteResponse>;
    fn get_trace(&self, span_id: &str) -> Result<Value>;
}

#[cfg(test)]
type TestWindowsHostObservation = std::result::Result<(String, String, String), String>;
#[cfg(test)]
type TestWslObservation = std::result::Result<(String, String, String, u32, String), String>;
#[cfg(test)]
type TestWslCommandOutputs = (String, String, String);

#[cfg(test)]
std::thread_local! {
    static TEST_WINDOWS_HOST_OBSERVATION: RefCell<Option<TestWindowsHostObservation>> =
        RefCell::new(None);
    static TEST_WSL_OBSERVATION: RefCell<Option<TestWslObservation>> = RefCell::new(None);
    static TEST_WSL_COMMAND_OUTPUTS: RefCell<Option<TestWslCommandOutputs>> = RefCell::new(None);
}

/// Windows backend delegating to world-service inside WSL.
pub struct WindowsWslBackend {
    pub(crate) distro: String,
    pub(crate) project_path: PathBuf,
    pub(crate) agent_pipe: PathBuf,
    pub(crate) host_carrier: InstallBootstrapContextCarrierV1,
    pub(crate) platform_mapping: PlatformBootstrapMappingV1,
    pub(crate) agent_id: String,
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) warm_cmd: WarmCmd,
    pub(crate) session_cache: Mutex<Option<WorldHandle>>,
    #[cfg(test)]
    pub(crate) agent_override: Option<Arc<dyn AgentApiMock>>,
}

impl WindowsWslBackend {
    fn parse_timeout_ms(var: &str) -> Option<Duration> {
        std::env::var(var)
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .map(Duration::from_millis)
    }

    fn cap_timeout(&self) -> Duration {
        Self::parse_timeout_ms("SUBSTRATE_WSL_AGENT_CAP_TIMEOUT_MS")
            .unwrap_or(Duration::from_secs(10))
    }

    fn exec_timeout(&self) -> Duration {
        Self::parse_timeout_ms("SUBSTRATE_WSL_AGENT_EXEC_TIMEOUT_MS")
            .unwrap_or(Duration::from_secs(120))
    }

    fn trace_timeout(&self) -> Duration {
        Self::parse_timeout_ms("SUBSTRATE_WSL_AGENT_TRACE_TIMEOUT_MS")
            .unwrap_or(Duration::from_secs(20))
    }

    fn agent_timeout_hint(&self) -> String {
        let transport = self.agent_transport();
        format!(
            "Agent transport: {}\nHint: ensure the Windows forwarder is running and the WSL agent is healthy (try `pwsh -File scripts/windows/wsl-warm.ps1 -DistroName {}`) and inspect forwarder logs under %LOCALAPPDATA%\\Substrate\\logs",
            transport.description(),
            self.distro
        )
    }

    /// Create backend using explicit authenticated Windows host context and WSL mapping.
    pub fn new() -> Result<Self> {
        Err(anyhow!(
            "Windows WSL backend requires an authenticated install bootstrap carrier and platform bootstrap mapping"
        ))
    }

    pub fn new_with_mapping(
        host_carrier: InstallBootstrapContextCarrierV1,
        mapping: PlatformBootstrapMappingV1,
        project_path: PathBuf,
    ) -> Result<Self> {
        if project_path.as_os_str().is_empty() {
            return Err(anyhow!(
                "Windows WSL backend requires an explicit project path"
            ));
        }
        if project_path.to_str().is_none() {
            return Err(anyhow!(
                "Windows WSL backend project path must be valid UTF-8"
            ));
        }

        let encoded_host_carrier = host_carrier
            .encode()
            .context("Windows WSL backend host carrier is invalid")?;
        let host_carrier = InstallBootstrapContextCarrierV1::decode(&encoded_host_carrier)
            .context("Windows WSL backend host carrier is not canonical")?;
        let encoded_mapping = mapping
            .encode(&host_carrier)
            .context("Windows WSL backend platform bootstrap mapping is invalid")?;
        let mapping = PlatformBootstrapMappingV1::decode(&encoded_mapping, &host_carrier)
            .context("Windows WSL backend platform bootstrap mapping is not canonical")?;

        validate_wsl_mapping_v1(&host_carrier, &mapping)?;

        let (distro, agent_pipe) = match (&mapping.platform_instance, &mapping.realized_transport) {
            (
                PlatformInstanceIdentityV1::Wsl { distro_name, .. },
                PlatformTransportIdentityV1::Wsl { pipe_path, .. },
            ) => (distro_name.clone(), pipe_path.clone()),
            _ => {
                return Err(anyhow!(
                    "Windows WSL backend requires a WSL platform bootstrap mapping"
                ));
            }
        };

        let warm_cmd = WarmCmd::enabled(
            distro.clone(),
            project_path.clone(),
            agent_pipe.clone(),
            host_carrier.context.selected_host_prefix.clone(),
            encoded_host_carrier,
            encoded_mapping,
        );
        Self::build(
            distro,
            project_path,
            PathBuf::from(agent_pipe),
            host_carrier,
            mapping,
            warm_cmd,
        )
    }

    #[cfg(test)]
    pub(crate) fn with_mock_agent(
        distro: String,
        project_path: PathBuf,
        warm_cmd: WarmCmd,
        agent: Arc<dyn AgentApiMock>,
    ) -> Result<Self> {
        let host_carrier = InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_windows(
                r"C:\Substrate",
                r"ACME\Alice",
                "S-1-5-21-1000",
            )
            .expect("test host carrier"),
        )
        .expect("test host carrier");
        let scope = WindowsForwarderScopeV1::derive(
            "S-1-5-21-1000",
            &distro,
            "0123456789abcdef0123456789abcdef",
            DEFAULT_AGENT_PIPE,
        )
        .expect("test scope");
        let mapping = PlatformBootstrapMappingV1::new_wsl(
            &host_carrier,
            &distro,
            "0123456789abcdef0123456789abcdef",
            &format!(
                r"C:\Users\Alice\AppData\Local\Substrate\forwarder\{}",
                scope.0
            ),
            "/home/substrate/.substrate",
            "substrate",
            1000,
            DEFAULT_AGENT_PIPE,
            "/run/substrate.sock",
        )
        .expect("test platform mapping");
        let mut backend = Self::build(
            distro,
            project_path,
            PathBuf::from(DEFAULT_AGENT_PIPE),
            host_carrier,
            mapping,
            warm_cmd,
        )?;
        backend.agent_override = Some(agent);
        Ok(backend)
    }

    fn build(
        distro: String,
        project_path: PathBuf,
        agent_pipe: PathBuf,
        host_carrier: InstallBootstrapContextCarrierV1,
        platform_mapping: PlatformBootstrapMappingV1,
        warm_cmd: WarmCmd,
    ) -> Result<Self> {
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
            host_carrier,
            platform_mapping,
            agent_id: "world-windows-wsl".to_string(),
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
        Transport::NamedPipe {
            path: self.agent_pipe.clone(),
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
        let timeout = self.cap_timeout();
        self.block_on(async move {
            tokio::time::timeout(timeout, client.capabilities())
                .await
                .with_context(|| {
                    format!(
                        "Timed out after {}s waiting for agent capabilities.\n{}",
                        timeout.as_secs(),
                        self.agent_timeout_hint()
                    )
                })?
        })
    }

    fn execute_agent(&self, request: ExecuteRequest) -> Result<ExecuteResponse> {
        #[cfg(test)]
        if let Some(mock) = &self.agent_override {
            return mock.execute(request);
        }

        let client = AgentClient::new(self.agent_transport())?;
        let timeout = self.exec_timeout();
        self.block_on(async move {
            tokio::time::timeout(timeout, client.execute(request))
                .await
                .with_context(|| {
                    format!(
                        "Timed out after {}s waiting for agent execute.\n{}",
                        timeout.as_secs(),
                        self.agent_timeout_hint()
                    )
                })?
        })
    }

    fn fetch_trace(&self, span_id: &str) -> Result<Value> {
        #[cfg(test)]
        if let Some(mock) = &self.agent_override {
            return mock.get_trace(span_id);
        }

        let client = AgentClient::new(self.agent_transport())?;
        let timeout = self.trace_timeout();
        let span_id = span_id.to_string();
        self.block_on(async move {
            tokio::time::timeout(timeout, client.get_trace(&span_id))
                .await
                .with_context(|| {
                    format!(
                        "Timed out after {}s waiting for agent trace for span {span_id}.\n{}",
                        timeout.as_secs(),
                        self.agent_timeout_hint()
                    )
                })?
        })
    }

    fn ensure_agent_ready(&self) -> Result<()> {
        validate_wsl_mapping_v1(&self.host_carrier, &self.platform_mapping)?;
        match self.capabilities() {
            Ok(_) => Ok(()),
            Err(initial_err) => {
                warn!(
                    target: "world_windows_wsl::backend",
                    error = %initial_err,
                    "Agent capabilities check failed; invoking warm script"
                );

                if self.warm_cmd.enabled {
                    let script_path = self
                        .warm_cmd
                        .project_path
                        .join("scripts/windows/wsl-warm.ps1");
                    if !script_path.is_file() {
                        return Err(anyhow!(
                            "world backend unavailable (WSL agent not ready) and warm script was not found at {}\nHint: run `pwsh -File scripts/windows/wsl-warm.ps1 -DistroName {} -ProjectPath (Resolve-Path .)` from the Substrate repo root, then retry.",
                            script_path.display(),
                            self.distro
                        ))
                        .context(initial_err);
                    }
                }

                self.warm_cmd
                    .run()
                    .context("wsl warm script failed to execute")?;

                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
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

    pub(crate) fn ensure_ready(&self) -> Result<()> {
        self.ensure_agent_ready()
    }

    pub(crate) fn ensure_persistent_session_ready(&self) -> Result<()> {
        self.ensure_agent_ready()
    }

    fn convert_exec_request(&self, req: &ExecRequest) -> Result<ExecuteRequest> {
        let cwd = to_wsl_path(&self.project_path, &req.cwd)?;
        let env = if req.env.is_empty() {
            None
        } else {
            Some(req.env.clone())
        };

        let fs_mode = self.resolve_fs_mode();
        let write_enabled = matches!(fs_mode, WorldFsMode::Writable);
        let policy_snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
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
                    enabled: write_enabled,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        Ok(ExecuteRequest {
            profile: None,
            cmd: req.cmd.clone(),
            cwd: Some(cwd),
            env,
            pty: req.pty,
            agent_id: self.agent_id.clone(),
            budget: None,
            policy_snapshot,
            shared_world: None,
            world_network: None,
            world_fs_mode: Some(fs_mode),
            member_dispatch: None,
            acceptance_context: None,
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
            world_fs_strategy_primary: None,
            world_fs_strategy_final: None,
            world_fs_strategy_fallback_reason: None,
            process_telemetry: ProcessTelemetry::not_supported_platform(),
        };
        if let Some(ref mut diff) = result.fs_diff {
            normalize_diff(diff);
        }
        result
    }

    fn generate_world_handle(&self) -> WorldHandle {
        WorldHandle {
            id: format!("wsl:{}:{}", self.distro, Uuid::now_v7()),
            shared_binding: None,
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
                self.ensure_persistent_session_ready()?;
                return Ok(handle);
            }
        }

        self.ensure_ready()?;

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

pub(crate) fn observe_wsl_mapping_v1(
    declared_distro_name: &str,
) -> Result<(String, String, String, u32, String)> {
    #[cfg(test)]
    {
        if let Some(observation) = TEST_WSL_OBSERVATION.with(|cell| cell.borrow().clone()) {
            return observation.map_err(anyhow::Error::msg);
        }
    }

    let list_names = |args: &[&str], context: &str| -> Result<String> {
        let output = Command::new("wsl.exe")
            .args(args)
            .output()
            .with_context(|| {
                format!(
                    "failed to observe WSL state via `wsl.exe {}`",
                    args.join(" ")
                )
            })?;
        if !output.status.success() {
            return Err(anyhow!(
                "{context}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim(),
            ));
        }
        String::from_utf8(output.stdout).context("WSL observation produced non-UTF-8 output")
    };

    #[cfg(test)]
    let test_command_outputs = TEST_WSL_COMMAND_OUTPUTS.with(|cell| cell.borrow().clone());
    #[cfg(not(test))]
    let test_command_outputs: Option<(String, String, String)> = None;

    let registered = if let Some((registered, _, _)) = &test_command_outputs {
        registered.clone()
    } else {
        list_names(&["-l", "-q"], "unable to enumerate registered WSL distros")?
    };
    let registered_matches = registered
        .lines()
        .map(|line| {
            line.trim_matches(|ch| ch == '\r' || ch == '\u{feff}')
                .trim()
        })
        .filter(|line| !line.is_empty())
        .filter(|line| line.eq_ignore_ascii_case(declared_distro_name))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if registered_matches.is_empty() {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` is not registered"
        ));
    }
    if registered_matches.len() != 1 {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` matched multiple registered distros"
        ));
    }
    let exact_distro_name = registered_matches[0].clone();

    let running = if let Some((_, running, _)) = &test_command_outputs {
        running.clone()
    } else {
        list_names(&["-l", "-v"], "unable to enumerate running WSL distros")?
    };
    let ordered_names = registered_matches
        .iter()
        .cloned()
        .chain([exact_distro_name.clone()])
        .collect::<Vec<_>>();
    let running_matches = running
        .lines()
        .map(|line| line.trim_matches('\r'))
        .filter_map(|line| {
            let mut trimmed = line.trim_start();
            if trimmed.is_empty()
                || trimmed.starts_with("NAME")
                || trimmed.starts_with("Windows Subsystem for Linux")
            {
                return None;
            }
            if let Some(stripped) = trimmed.strip_prefix('*') {
                trimmed = stripped.trim_start();
            }
            let matched_name = ordered_names
                .iter()
                .filter(|name| trimmed.starts_with(name.as_str()))
                .max_by_key(|name| name.len())?;
            let remainder = &trimmed[matched_name.len()..];
            let state = remainder
                .split_once(char::is_whitespace)
                .map(|_| remainder.trim_start().split_whitespace().next())
                .flatten()?;
            state
                .eq_ignore_ascii_case("Running")
                .then(|| matched_name.to_string())
        })
        .filter(|name| name.eq_ignore_ascii_case(declared_distro_name))
        .collect::<Vec<_>>();

    if running_matches.is_empty() {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` is not running"
        ));
    }
    if running_matches.len() != 1 {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` matched multiple running distros"
        ));
    }
    if running_matches[0] != exact_distro_name {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` resolved to an ambiguous registered spelling"
        ));
    }

    let stdout = if let Some((_, _, guest_identity)) = &test_command_outputs {
        guest_identity.clone()
    } else {
        let output = Command::new("wsl.exe")
            .args([
                "-d",
                &exact_distro_name,
                "--",
                "bash",
                "-lc",
                "set -euo pipefail\nmachine_id=\"$(tr -d '\\n' </etc/machine-id)\"\naccount=\"$(id -un)\"\nuid=\"$(id -u)\"\npasswd_by_name=\"$(getent passwd \"$account\")\"\npasswd_by_uid=\"$(getent passwd \"$uid\")\"\nprintf 'machine_id=%s\\n' \"$machine_id\"\nprintf 'account=%s\\n' \"$account\"\nprintf 'uid=%s\\n' \"$uid\"\nprintf 'passwd_by_name=%s\\n' \"$passwd_by_name\"\nprintf 'passwd_by_uid=%s\\n' \"$passwd_by_uid\"\n",
            ])
            .output()
            .context("failed to observe WSL guest identity")?;
        if !output.status.success() {
            return Err(anyhow!(
                "unable to observe WSL guest identity\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim(),
            ));
        }
        String::from_utf8(output.stdout).context("WSL guest identity output is not valid UTF-8")?
    };
    let mut machine_id = None;
    let mut account = None;
    let mut uid = None;
    let mut passwd_by_name = None;
    let mut passwd_by_uid = None;
    for line in stdout.lines() {
        let trimmed = line.trim_matches('\r');
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        match key {
            "machine_id" => machine_id = Some(value.to_string()),
            "account" => account = Some(value.to_string()),
            "uid" => uid = Some(value.to_string()),
            "passwd_by_name" => passwd_by_name = Some(value.to_string()),
            "passwd_by_uid" => passwd_by_uid = Some(value.to_string()),
            _ => {}
        }
    }

    let machine_id = machine_id.ok_or_else(|| anyhow!("WSL guest machine ID is missing"))?;
    if machine_id.len() != 32
        || !machine_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(anyhow!("WSL guest machine ID is malformed"));
    }
    let account = account.ok_or_else(|| anyhow!("WSL guest account is missing"))?;
    if account.is_empty()
        || account
            .chars()
            .any(|ch| matches!(ch, '\0' | '\n' | '\r' | ':' | '/'))
    {
        return Err(anyhow!("WSL guest account is malformed"));
    }
    let uid_text = uid.ok_or_else(|| anyhow!("WSL guest UID is missing"))?;
    let uid = uid_text
        .parse::<u32>()
        .context("WSL guest UID is malformed")?;
    let passwd_by_name = passwd_by_name.ok_or_else(|| anyhow!("WSL passwd entry is missing"))?;
    let passwd_by_uid = passwd_by_uid.ok_or_else(|| anyhow!("WSL passwd UID entry is missing"))?;
    if passwd_by_name != passwd_by_uid {
        return Err(anyhow!(
            "WSL account-database lookup by name and UID did not round-trip"
        ));
    }
    let passwd_fields = passwd_by_name.split(':').collect::<Vec<_>>();
    if passwd_fields.len() < 7 {
        return Err(anyhow!("WSL passwd entry is malformed"));
    }
    if passwd_fields[0] != account {
        return Err(anyhow!(
            "WSL passwd entry account does not match the active guest account"
        ));
    }
    if passwd_fields[2]
        .parse::<u32>()
        .context("WSL passwd UID is malformed")?
        != uid
    {
        return Err(anyhow!(
            "WSL passwd entry UID does not match the active guest UID"
        ));
    }
    let home = passwd_fields[5];
    let normalized_home = transport_api_types::normalize_unix_install_bootstrap_path(home)
        .context("WSL passwd home directory is invalid")?;
    if normalized_home.starts_with("/mnt/") {
        return Err(anyhow!(
            "WSL passwd home directory may not resolve to a host-mounted path"
        ));
    }

    Ok((exact_distro_name, machine_id, account, uid, normalized_home))
}

pub(crate) fn validate_wsl_mapping_v1(
    host_carrier: &InstallBootstrapContextCarrierV1,
    mapping: &PlatformBootstrapMappingV1,
) -> Result<()> {
    host_carrier
        .validate()
        .context("Windows WSL backend host carrier is invalid")?;
    mapping
        .validate(host_carrier)
        .context("Windows WSL backend platform bootstrap mapping is invalid")?;

    let PlatformPrincipalV1::Windows {
        account: expected_account,
        sid: expected_sid,
    } = &host_carrier.context.intended_host_principal
    else {
        return Err(anyhow!(
            "Windows WSL backend requires a Windows install bootstrap carrier"
        ));
    };

    let (
        declared_distro_name,
        declared_guest_machine_id,
        declared_pipe_path,
        declared_guest_socket,
    ) = match (&mapping.platform_instance, &mapping.realized_transport) {
        (
            PlatformInstanceIdentityV1::Wsl {
                distro_name,
                guest_machine_id,
            },
            PlatformTransportIdentityV1::Wsl {
                pipe_path,
                guest_socket,
            },
        ) => (
            distro_name.as_str(),
            guest_machine_id.as_str(),
            pipe_path.as_str(),
            guest_socket.as_str(),
        ),
        _ => {
            return Err(anyhow!(
                "Windows WSL backend requires a WSL platform bootstrap mapping"
            ));
        }
    };
    if declared_guest_socket != "/run/substrate.sock" {
        return Err(anyhow!(
            "Windows WSL backend requires the guest socket to be /run/substrate.sock"
        ));
    }
    if normalize_windows_pipe_path(declared_pipe_path)
        .context("Windows WSL backend pipe path is invalid")?
        != declared_pipe_path
    {
        return Err(anyhow!("Windows WSL backend pipe path is not canonical"));
    }

    #[cfg(test)]
    let current_windows_host = if let Some(observation) =
        TEST_WINDOWS_HOST_OBSERVATION.with(|cell| cell.borrow().clone())
    {
        let (account, sid, local_app_data) = observation.map_err(anyhow::Error::msg)?;
        (
            account,
            sid,
            normalize_windows_install_bootstrap_path(&local_app_data)
                .context("current token LocalApplicationData is invalid")?,
        )
    } else {
        let output = Command::new("pwsh")
            .args([
                "-NoProfile",
                "-NoLogo",
                "-Command",
                "$ErrorActionPreference = 'Stop'\nif (-not ('SubstrateKnownFolderNative' -as [type])) {\nAdd-Type -TypeDefinition @'\nusing System;\nusing System.Runtime.InteropServices;\npublic static class SubstrateKnownFolderNative {\n    [DllImport(\"shell32.dll\")]\n    public static extern int SHGetKnownFolderPath(ref Guid rfid, uint dwFlags, IntPtr hToken, out IntPtr ppszPath);\n}\n'@\n}\n$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()\nif ($null -eq $identity -or $null -eq $identity.User) { throw 'current Windows principal is unavailable' }\n$folderId = [Guid]'F1B32785-6FBA-4FCF-9D55-7B8E7F157091'\n$raw = [IntPtr]::Zero\n$hr = [SubstrateKnownFolderNative]::SHGetKnownFolderPath([ref]$folderId, 0, $identity.Token, [ref]$raw)\nif ($hr -lt 0 -or $raw -eq [IntPtr]::Zero) { throw 'current token LocalApplicationData is unavailable' }\ntry {\n    $path = [Runtime.InteropServices.Marshal]::PtrToStringUni($raw)\n    if ([string]::IsNullOrEmpty($path)) { throw 'current token LocalApplicationData is malformed' }\n    Write-Output ('account=' + $identity.Name)\n    Write-Output ('sid=' + $identity.User.Value)\n    Write-Output ('local_app_data=' + $path)\n} finally {\n    if ($raw -ne [IntPtr]::Zero) {\n        [Runtime.InteropServices.Marshal]::FreeCoTaskMem($raw)\n    }\n}\n",
            ])
            .output()
            .context("failed to observe current Windows principal")?;
        if !output.status.success() {
            return Err(anyhow!(
                "unable to observe current Windows principal\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim(),
            ));
        }
        let stdout = String::from_utf8(output.stdout)
            .context("Windows principal observation output is not valid UTF-8")?;
        let mut account = None;
        let mut sid = None;
        let mut local_app_data = None;
        for line in stdout.lines() {
            let trimmed = line.trim_matches('\r');
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            match key {
                "account" => account = Some(value.to_string()),
                "sid" => sid = Some(value.to_string()),
                "local_app_data" => local_app_data = Some(value.to_string()),
                _ => {}
            }
        }
        (
            account.ok_or_else(|| anyhow!("current Windows account is missing"))?,
            sid.ok_or_else(|| anyhow!("current Windows SID is missing"))?,
            normalize_windows_install_bootstrap_path(
                &local_app_data
                    .ok_or_else(|| anyhow!("current token LocalApplicationData is missing"))?,
            )
            .context("current token LocalApplicationData is invalid")?,
        )
    };

    #[cfg(not(test))]
    let current_windows_host = {
        let output = Command::new("pwsh")
            .args([
                "-NoProfile",
                "-NoLogo",
                "-Command",
                "$ErrorActionPreference = 'Stop'\nif (-not ('SubstrateKnownFolderNative' -as [type])) {\nAdd-Type -TypeDefinition @'\nusing System;\nusing System.Runtime.InteropServices;\npublic static class SubstrateKnownFolderNative {\n    [DllImport(\"shell32.dll\")]\n    public static extern int SHGetKnownFolderPath(ref Guid rfid, uint dwFlags, IntPtr hToken, out IntPtr ppszPath);\n}\n'@\n}\n$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()\nif ($null -eq $identity -or $null -eq $identity.User) { throw 'current Windows principal is unavailable' }\n$folderId = [Guid]'F1B32785-6FBA-4FCF-9D55-7B8E7F157091'\n$raw = [IntPtr]::Zero\n$hr = [SubstrateKnownFolderNative]::SHGetKnownFolderPath([ref]$folderId, 0, $identity.Token, [ref]$raw)\nif ($hr -lt 0 -or $raw -eq [IntPtr]::Zero) { throw 'current token LocalApplicationData is unavailable' }\ntry {\n    $path = [Runtime.InteropServices.Marshal]::PtrToStringUni($raw)\n    if ([string]::IsNullOrEmpty($path)) { throw 'current token LocalApplicationData is malformed' }\n    Write-Output ('account=' + $identity.Name)\n    Write-Output ('sid=' + $identity.User.Value)\n    Write-Output ('local_app_data=' + $path)\n} finally {\n    if ($raw -ne [IntPtr]::Zero) {\n        [Runtime.InteropServices.Marshal]::FreeCoTaskMem($raw)\n    }\n}\n",
            ])
            .output()
            .context("failed to observe current Windows principal")?;
        if !output.status.success() {
            return Err(anyhow!(
                "unable to observe current Windows principal\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim(),
            ));
        }
        let stdout = String::from_utf8(output.stdout)
            .context("Windows principal observation output is not valid UTF-8")?;
        let mut account = None;
        let mut sid = None;
        let mut local_app_data = None;
        for line in stdout.lines() {
            let trimmed = line.trim_matches('\r');
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            match key {
                "account" => account = Some(value.to_string()),
                "sid" => sid = Some(value.to_string()),
                "local_app_data" => local_app_data = Some(value.to_string()),
                _ => {}
            }
        }
        (
            account.ok_or_else(|| anyhow!("current Windows account is missing"))?,
            sid.ok_or_else(|| anyhow!("current Windows SID is missing"))?,
            normalize_windows_install_bootstrap_path(
                &local_app_data
                    .ok_or_else(|| anyhow!("current token LocalApplicationData is missing"))?,
            )
            .context("current token LocalApplicationData is invalid")?,
        )
    };

    let (current_account, current_sid, local_app_data) = current_windows_host;

    if &current_account != expected_account || &current_sid != expected_sid {
        return Err(anyhow!(
            "Windows WSL backend host carrier does not match the current Windows principal"
        ));
    }

    let expected_scope = WindowsForwarderScopeV1::derive(
        expected_sid,
        declared_distro_name,
        declared_guest_machine_id,
        declared_pipe_path,
    )
    .context("Windows WSL backend forwarder scope is invalid")?;
    let expected_control_root = normalize_windows_install_bootstrap_path(&format!(
        r"{}\Substrate\forwarder\{}",
        local_app_data, expected_scope.0
    ))
    .context("Windows WSL backend control root is invalid")?;
    if mapping.host_platform_control_root != expected_control_root {
        return Err(anyhow!(
            "Windows WSL backend platform control root does not match the current Windows scope"
        ));
    }

    let (observed_distro_name, observed_machine_id, observed_account, observed_uid, observed_home) =
        observe_wsl_mapping_v1(declared_distro_name)?;
    if observed_distro_name != declared_distro_name {
        return Err(anyhow!(
            "Windows WSL backend distro spelling does not match the registered running distro"
        ));
    }
    if observed_machine_id != declared_guest_machine_id {
        return Err(anyhow!(
            "Windows WSL backend guest machine ID does not match the live WSL distro"
        ));
    }
    let PlatformPrincipalV1::Unix {
        account: expected_guest_account,
        uid: expected_guest_uid,
    } = &mapping.realized_principal
    else {
        return Err(anyhow!(
            "Windows WSL backend requires a Unix guest principal"
        ));
    };
    if &observed_account != expected_guest_account || observed_uid != *expected_guest_uid {
        return Err(anyhow!(
            "Windows WSL backend guest account or UID does not match the live WSL distro"
        ));
    }
    let expected_substrate_home = format!("{observed_home}/.substrate");
    if mapping.realized_substrate_home != expected_substrate_home {
        return Err(anyhow!(
            "Windows WSL backend guest substrate home does not match the live WSL account database"
        ));
    }

    Ok(())
}

#[cfg(test)]
pub(crate) fn set_test_windows_host_observation(observation: TestWindowsHostObservation) {
    TEST_WINDOWS_HOST_OBSERVATION.with(|cell| {
        *cell.borrow_mut() = Some(observation);
    });
}

#[cfg(test)]
pub(crate) fn set_test_wsl_observation(observation: TestWslObservation) {
    TEST_WSL_OBSERVATION.with(|cell| {
        *cell.borrow_mut() = Some(observation);
    });
}

#[cfg(test)]
pub(crate) fn set_test_wsl_command_outputs(
    registered: String,
    running: String,
    guest_identity: String,
) {
    TEST_WSL_COMMAND_OUTPUTS.with(|cell| {
        *cell.borrow_mut() = Some((registered, running, guest_identity));
    });
}
