use agent_api_types::{GatewayClientWiringV1, GatewayLifecycleResponseV1, GatewayStatusV1};
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener as StdTcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

pub(crate) const GATEWAY_REQUEST_ENABLED_ENV: &str = "SUBSTRATE_LLM_GATEWAY_ENABLED";
pub(crate) const GATEWAY_REQUEST_MODE_ENV: &str = "SUBSTRATE_LLM_GATEWAY_MODE";
pub(crate) const GATEWAY_REQUEST_DEFAULT_BACKEND_ENV: &str = "SUBSTRATE_LLM_DEFAULT_BACKEND";

const GATEWAY_LAUNCH_MODE_ENV: &str = "SUBSTRATE_LLM_GATEWAY_MODE";
const GATEWAY_LAUNCH_CONFIG_PATH_ENV: &str = "SUBSTRATE_LLM_GATEWAY_CONFIG_PATH";
const GATEWAY_LAUNCH_DISABLE_TOKEN_PERSISTENCE_ENV: &str =
    "SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE";
const GATEWAY_MODE_IN_WORLD: &str = "in_world";
const GATEWAY_MODE_HOST_ONLY: &str = "host_only";

const CODEX_ACCOUNT_ID_ENV: &str = "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID";
const CODEX_ACCESS_TOKEN_ENV: &str = "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN";
const GATEWAY_BINARY_OVERRIDE_ENV: &str = "SUBSTRATE_GATEWAY_BINARY";
const HEALTH_PATH: &str = "/health";
const DEFAULT_BACKEND: &str = "cli:codex";
const DEFAULT_ROUTED_MODEL: &str = "codex";
const DEFAULT_ACTUAL_MODEL: &str = "codex-mini-latest";
const DEFAULT_PROVIDER_NAME: &str = "openai-codex";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GatewayRuntimeState {
    AbsentComponent,
    ProvisionedStopped,
    Starting,
    Ready,
    RestartInProgress,
}

#[derive(Debug, Clone)]
pub(crate) struct GatewayControlSettings {
    pub default_backend: String,
}

impl GatewayControlSettings {
    pub(crate) fn from_request_env(
        env: Option<&HashMap<String, String>>,
    ) -> Result<Self, GatewayRuntimeFailure> {
        let enabled = parse_bool_env(env, GATEWAY_REQUEST_ENABLED_ENV)?.unwrap_or(true);
        if !enabled {
            return Err(GatewayRuntimeFailure::policy(
                "gateway lifecycle is disabled by effective config",
            ));
        }

        let mode = env
            .and_then(|values| values.get(GATEWAY_REQUEST_MODE_ENV))
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| GATEWAY_MODE_IN_WORLD.to_string());

        match mode.as_str() {
            GATEWAY_MODE_IN_WORLD => {}
            GATEWAY_MODE_HOST_ONLY => {
                return Err(GatewayRuntimeFailure::policy(
                    "gateway lifecycle is unavailable while llm.gateway.mode=host_only",
                ));
            }
            other => {
                return Err(GatewayRuntimeFailure::invalid_integration(format!(
                    "unsupported gateway mode '{}'",
                    other
                )));
            }
        }

        let default_backend = env
            .and_then(|values| values.get(GATEWAY_REQUEST_DEFAULT_BACKEND_ENV))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_BACKEND.to_string());

        if default_backend != DEFAULT_BACKEND {
            return Err(GatewayRuntimeFailure::invalid_integration(format!(
                "default backend '{}' is not supported by the integrated gateway lifecycle yet",
                default_backend
            )));
        }

        Ok(Self { default_backend })
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GatewayRuntimeFailure {
    #[error("gateway_invalid_integration: {0}")]
    InvalidIntegration(String),
    #[error("gateway_transient_failure: {0}")]
    Transient(String),
    #[error("gateway_policy_blocked: {0}")]
    PolicyBlocked(String),
}

impl GatewayRuntimeFailure {
    pub(crate) fn invalid_integration(message: impl Into<String>) -> Self {
        Self::InvalidIntegration(message.into())
    }

    pub(crate) fn transient(message: impl Into<String>) -> Self {
        Self::Transient(message.into())
    }

    pub(crate) fn policy(message: impl Into<String>) -> Self {
        Self::PolicyBlocked(message.into())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GatewayRuntimeStartContext {
    pub world_id: String,
    pub project_dir: PathBuf,
    pub cgroup_path: PathBuf,
    pub require_cgroup_attach: bool,
    pub control: GatewayControlSettings,
}

#[derive(Debug, Clone)]
struct ManagedGatewayRuntime {
    world_id: String,
    port: u16,
    runtime_dir: PathBuf,
    config_path: PathBuf,
    child: Arc<Mutex<Child>>,
    state: Arc<RwLock<GatewayRuntimeState>>,
}

impl ManagedGatewayRuntime {
    fn set_state(&self, state: GatewayRuntimeState) {
        if let Ok(mut guard) = self.state.write() {
            *guard = state;
        }
    }

    fn state(&self) -> GatewayRuntimeState {
        self.state
            .read()
            .map(|guard| *guard)
            .unwrap_or(GatewayRuntimeState::ProvisionedStopped)
    }
}

#[derive(Default)]
pub(crate) struct GatewayRuntimeManager {
    runtimes: Mutex<HashMap<String, ManagedGatewayRuntime>>,
}

impl GatewayRuntimeManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn status(
        &self,
        world_id: &str,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        let Some(runtime) = self.runtime_for_world(world_id) else {
            return Ok(unavailable_response());
        };

        let state = self.observe_runtime_state(&runtime).await?;
        Ok(match state {
            GatewayRuntimeState::Ready => available_response(runtime.port),
            GatewayRuntimeState::Starting
            | GatewayRuntimeState::RestartInProgress
            | GatewayRuntimeState::ProvisionedStopped
            | GatewayRuntimeState::AbsentComponent => unavailable_response(),
        })
    }

    pub(crate) async fn sync(
        &self,
        ctx: GatewayRuntimeStartContext,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        if let Some(runtime) = self.runtime_for_world(&ctx.world_id) {
            match self.observe_runtime_state(&runtime).await? {
                GatewayRuntimeState::Ready => return Ok(available_response(runtime.port)),
                GatewayRuntimeState::Starting | GatewayRuntimeState::RestartInProgress => {
                    return self.wait_until_ready(runtime, Duration::from_secs(8)).await;
                }
                GatewayRuntimeState::ProvisionedStopped | GatewayRuntimeState::AbsentComponent => {
                    self.remove_runtime(&ctx.world_id);
                }
            }
        }

        let Some(binary_path) = resolve_gateway_binary()? else {
            return Ok(unavailable_response());
        };

        let runtime = start_runtime(binary_path, ctx)?;
        let world_id = runtime.world_id.clone();
        self.insert_runtime(runtime.clone());
        match self.wait_until_ready(runtime, Duration::from_secs(8)).await {
            Ok(response) => Ok(response),
            Err(err) => {
                self.remove_runtime(&world_id);
                Err(err)
            }
        }
    }

    pub(crate) async fn restart(
        &self,
        ctx: GatewayRuntimeStartContext,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        if let Some(existing) = self.remove_runtime(&ctx.world_id) {
            existing.set_state(GatewayRuntimeState::RestartInProgress);
            stop_runtime(existing)
                .with_context(|| format!("failed to stop gateway runtime for {}", ctx.world_id))
                .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
        }

        self.sync(ctx).await
    }

    #[cfg(test)]
    pub(crate) fn pid_for_world(&self, world_id: &str) -> Option<u32> {
        let runtime = self.runtime_for_world(world_id)?;
        runtime.child.lock().ok().map(|child| child.id())
    }

    fn runtime_for_world(&self, world_id: &str) -> Option<ManagedGatewayRuntime> {
        self.runtimes
            .lock()
            .ok()
            .and_then(|guard| guard.get(world_id).cloned())
    }

    fn insert_runtime(&self, runtime: ManagedGatewayRuntime) {
        if let Ok(mut guard) = self.runtimes.lock() {
            guard.insert(runtime.world_id.clone(), runtime);
        }
    }

    fn remove_runtime(&self, world_id: &str) -> Option<ManagedGatewayRuntime> {
        self.runtimes
            .lock()
            .ok()
            .and_then(|mut guard| guard.remove(world_id))
    }

    async fn observe_runtime_state(
        &self,
        runtime: &ManagedGatewayRuntime,
    ) -> Result<GatewayRuntimeState, GatewayRuntimeFailure> {
        let status = {
            let mut child = runtime
                .child
                .lock()
                .map_err(|_| GatewayRuntimeFailure::transient("gateway child lock poisoned"))?;
            child.try_wait().with_context(|| {
                format!(
                    "failed to inspect gateway child status for {}",
                    runtime.runtime_dir.display()
                )
            })
        }
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

        if status.is_some() {
            runtime.set_state(GatewayRuntimeState::ProvisionedStopped);
            return Ok(GatewayRuntimeState::ProvisionedStopped);
        }

        if gateway_health_ready(runtime.port).await {
            runtime.set_state(GatewayRuntimeState::Ready);
            return Ok(GatewayRuntimeState::Ready);
        }

        Ok(runtime.state())
    }

    async fn wait_until_ready(
        &self,
        runtime: ManagedGatewayRuntime,
        timeout: Duration,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        let deadline = Instant::now() + timeout;
        loop {
            match self.observe_runtime_state(&runtime).await? {
                GatewayRuntimeState::Ready => return Ok(available_response(runtime.port)),
                GatewayRuntimeState::ProvisionedStopped => {
                    return Err(GatewayRuntimeFailure::transient(format!(
                        "gateway process exited before it became ready; inspect {} and {}",
                        runtime.runtime_dir.join("stdout.log").display(),
                        runtime.runtime_dir.join("stderr.log").display()
                    )));
                }
                GatewayRuntimeState::Starting | GatewayRuntimeState::RestartInProgress => {
                    if Instant::now() >= deadline {
                        return Err(GatewayRuntimeFailure::transient(format!(
                            "gateway did not become ready before timeout; inspect {} and {}",
                            runtime.runtime_dir.join("stdout.log").display(),
                            runtime.runtime_dir.join("stderr.log").display()
                        )));
                    }
                    tokio::time::sleep(Duration::from_millis(125)).await;
                }
                GatewayRuntimeState::AbsentComponent => return Ok(unavailable_response()),
            }
        }
    }
}

fn start_runtime(
    binary_path: PathBuf,
    ctx: GatewayRuntimeStartContext,
) -> Result<ManagedGatewayRuntime, GatewayRuntimeFailure> {
    let port = pick_free_port().map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    let runtime_dir = std::env::temp_dir()
        .join("substrate-gateway-runtime")
        .join(&ctx.world_id);
    let home_dir = runtime_dir.join("home");
    fs::create_dir_all(&home_dir)
        .with_context(|| {
            format!(
                "failed to create gateway runtime directory {}",
                home_dir.display()
            )
        })
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let config_path = runtime_dir.join("config.toml");
    let config = render_integrated_config(port, &ctx.control.default_backend)?;
    fs::write(&config_path, config)
        .with_context(|| format!("failed to write {}", config_path.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let stdout_log = runtime_dir.join("stdout.log");
    let stderr_log = runtime_dir.join("stderr.log");
    let stdout = fs::File::create(&stdout_log)
        .with_context(|| format!("failed to create {}", stdout_log.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    let stderr = fs::File::create(&stderr_log)
        .with_context(|| format!("failed to create {}", stderr_log.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let auth = resolve_codex_auth_handoff()?;

    let mut command = Command::new(&binary_path);
    command
        .arg("start")
        .arg("--config")
        .arg(&config_path)
        .current_dir(&ctx.project_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .env("HOME", &home_dir)
        .env(GATEWAY_LAUNCH_MODE_ENV, GATEWAY_MODE_IN_WORLD)
        .env(GATEWAY_LAUNCH_CONFIG_PATH_ENV, &config_path)
        .env(GATEWAY_LAUNCH_DISABLE_TOKEN_PERSISTENCE_ENV, "1")
        .env(CODEX_ACCESS_TOKEN_ENV, auth.access_token);

    if let Some(account_id) = auth.account_id {
        command.env(CODEX_ACCOUNT_ID_ENV, account_id);
    } else {
        command.env_remove(CODEX_ACCOUNT_ID_ENV);
    }

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", binary_path.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    attach_child_to_cgroup(child.id(), &ctx.cgroup_path, ctx.require_cgroup_attach)?;

    Ok(ManagedGatewayRuntime {
        world_id: ctx.world_id,
        port,
        runtime_dir,
        config_path,
        child: Arc::new(Mutex::new(child)),
        state: Arc::new(RwLock::new(GatewayRuntimeState::Starting)),
    })
}

fn stop_runtime(runtime: ManagedGatewayRuntime) -> Result<()> {
    let mut child = runtime
        .child
        .lock()
        .map_err(|_| anyhow!("gateway child lock poisoned"))?;
    if child.try_wait()?.is_none() {
        child.kill().context("failed to kill gateway child")?;
        let _ = child.wait();
    }
    Ok(())
}

fn attach_child_to_cgroup(
    pid: u32,
    cgroup_path: &Path,
    required: bool,
) -> Result<(), GatewayRuntimeFailure> {
    let cgroup_procs = cgroup_path.join("cgroup.procs");
    match fs::write(&cgroup_procs, pid.to_string()) {
        Ok(()) => Ok(()),
        Err(err) if !required => Ok(()),
        Err(err) => Err(GatewayRuntimeFailure::transient(format!(
            "failed to attach gateway pid {} to {}: {}",
            pid,
            cgroup_procs.display(),
            err
        ))),
    }
}

fn resolve_gateway_binary() -> Result<Option<PathBuf>, GatewayRuntimeFailure> {
    if let Some(path) = std::env::var_os(GATEWAY_BINARY_OVERRIDE_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(Some(path));
        }
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "{} points at a missing binary: {}",
            GATEWAY_BINARY_OVERRIDE_ENV,
            path.display()
        )));
    }

    for candidate in [
        PathBuf::from("/usr/local/bin/substrate-gateway"),
        PathBuf::from("/usr/bin/substrate-gateway"),
    ] {
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let candidate = parent.join("substrate-gateway");
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
    }

    Ok(None)
}

fn render_integrated_config(
    port: u16,
    default_backend: &str,
) -> Result<String, GatewayRuntimeFailure> {
    if default_backend != DEFAULT_BACKEND {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "unsupported integrated backend '{}'",
            default_backend
        )));
    }

    Ok(format!(
        r#"[server]
host = "127.0.0.1"
port = {port}
log_level = "info"

[router]
default = "{routed_model}"

[[providers]]
name = "{provider_name}"
provider_type = "openai"
auth_type = "oauth"
oauth_provider = "{provider_name}"
models = ["{actual_model}"]
enabled = true

[[models]]
name = "{routed_model}"

[[models.mappings]]
priority = 1
provider = "{provider_name}"
actual_model = "{actual_model}"
"#,
        routed_model = DEFAULT_ROUTED_MODEL,
        provider_name = DEFAULT_PROVIDER_NAME,
        actual_model = DEFAULT_ACTUAL_MODEL,
    ))
}

struct ResolvedCodexAuth {
    account_id: Option<String>,
    access_token: String,
}

fn resolve_codex_auth_handoff() -> Result<ResolvedCodexAuth, GatewayRuntimeFailure> {
    if let Some(access_token) = read_trimmed_env(CODEX_ACCESS_TOKEN_ENV) {
        let account_id = read_trimmed_env(CODEX_ACCOUNT_ID_ENV);
        return Ok(ResolvedCodexAuth {
            account_id,
            access_token,
        });
    }

    let path = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
        .join("auth.json");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read Codex auth state from {}", path.display()))
        .map_err(|err| GatewayRuntimeFailure::invalid_integration(err.to_string()))?;
    let json: Value = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse Codex auth state at {}", path.display()))
        .map_err(|err| GatewayRuntimeFailure::invalid_integration(err.to_string()))?;

    let access_token = find_json_string(&json, &["access_token"]).ok_or_else(|| {
        GatewayRuntimeFailure::invalid_integration("Codex auth state is missing access_token")
    })?;
    let account_id = find_json_string(&json, &["account_id"]);

    Ok(ResolvedCodexAuth {
        account_id,
        access_token,
    })
}

fn find_json_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(raw) = map.get(*key).and_then(Value::as_str) {
                    let trimmed = raw.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
            map.values().find_map(|value| find_json_string(value, keys))
        }
        Value::Array(items) => items.iter().find_map(|value| find_json_string(value, keys)),
        _ => None,
    }
}

fn read_trimmed_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_bool_env(
    env: Option<&HashMap<String, String>>,
    key: &str,
) -> Result<Option<bool>, GatewayRuntimeFailure> {
    let Some(raw) = env.and_then(|values| values.get(key)) else {
        return Ok(None);
    };

    let raw_trimmed = raw.trim().to_ascii_lowercase();
    match raw_trimmed.as_str() {
        "" => Ok(None),
        "1" | "true" | "yes" | "on" => Ok(Some(true)),
        "0" | "false" | "no" | "off" => Ok(Some(false)),
        _ => Err(GatewayRuntimeFailure::invalid_integration(format!(
            "invalid boolean value '{}' for {}",
            raw, key
        ))),
    }
}

fn pick_free_port() -> Result<u16> {
    let listener =
        StdTcpListener::bind(("127.0.0.1", 0)).context("failed to allocate gateway port")?;
    let port = listener
        .local_addr()
        .context("failed to inspect allocated gateway port")?
        .port();
    drop(listener);
    Ok(port)
}

async fn gateway_health_ready(port: u16) -> bool {
    tokio::task::spawn_blocking(move || gateway_health_ready_blocking(port))
        .await
        .ok()
        .unwrap_or(false)
}

fn gateway_health_ready_blocking(port: u16) -> bool {
    let mut stream = match TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}")
            .parse()
            .expect("health socket address"),
        Duration::from_millis(250),
    ) {
        Ok(stream) => stream,
        Err(_) => return false,
    };

    if stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .is_err()
        || stream
            .set_write_timeout(Some(Duration::from_millis(250)))
            .is_err()
    {
        return false;
    }

    let request =
        format!("GET {HEALTH_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let _ = stream.shutdown(Shutdown::Write);

    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }
    response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200")
}

fn available_response(port: u16) -> GatewayLifecycleResponseV1 {
    let base_url = format!("http://127.0.0.1:{port}");
    GatewayLifecycleResponseV1 {
        status: GatewayStatusV1::Available,
        client_wiring: Some(GatewayClientWiringV1 {
            openai_base_url: base_url.clone(),
            anthropic_base_url: base_url,
        }),
    }
}

pub(crate) fn unavailable_response() -> GatewayLifecycleResponseV1 {
    GatewayLifecycleResponseV1 {
        status: GatewayStatusV1::Unavailable,
        client_wiring: None,
    }
}
