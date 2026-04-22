use agent_api_types::{
    GatewayCliCodexIntegratedAuthV1, GatewayClientWiringV1, GatewayIntegratedAuthPayloadV1,
    GatewayLifecycleResponseV1, GatewayStatusV1,
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener as StdTcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;

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
const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";
const GATEWAY_BINARY_OVERRIDE_ENV: &str = "SUBSTRATE_GATEWAY_BINARY";
const HEALTH_PATH: &str = "/health";
const DEFAULT_BACKEND: &str = "cli:codex";
const API_OPENAI_BACKEND: &str = "api:openai";
const DEFAULT_ROUTED_MODEL: &str = "codex";
const DEFAULT_ACTUAL_MODEL: &str = "codex-mini-latest";
const DEFAULT_PROVIDER_NAME: &str = "openai-codex";
const OPENAI_ROUTED_MODEL: &str = "gpt-4.1-mini";
const OPENAI_ACTUAL_MODEL: &str = "gpt-4.1-mini";
const OPENAI_PROVIDER_NAME: &str = "openai-api";
const GATEWAY_RUNTIME_ROOT_DIR: &str = "substrate-gateway-runtime";
const GATEWAY_RUNTIME_MANIFEST_NAME: &str = "runtime.json";
const DEFAULT_READY_TIMEOUT: Duration = Duration::from_secs(8);
const GATEWAY_RUNTIME_DIR_MODE: u32 = 0o750;
const GATEWAY_RUNTIME_FILE_MODE: u32 = 0o640;
const KNOWN_GATEWAY_AUTH_ENV_VARS: &[&str] = &[
    CODEX_ACCOUNT_ID_ENV,
    CODEX_ACCESS_TOKEN_ENV,
    OPENAI_API_KEY_ENV,
];
const REQUIRED_GATEWAY_CAPABILITIES: &[&str] =
    &["agent_api.run", "agent_api.events", "agent_api.events.live"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
            .unwrap_or_else(|| DEFAULT_BACKEND.to_string());

        if default_backend.is_empty() {
            return Err(GatewayRuntimeFailure::invalid_integration(format!(
                "{} must be a non-empty backend id",
                GATEWAY_REQUEST_DEFAULT_BACKEND_ENV
            )));
        }

        Ok(Self { default_backend })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GatewayIntegratedAuthKind {
    CliCodex,
    ApiEnv,
}

#[derive(Debug, Clone, Copy)]
enum GatewayProviderAuthConfig {
    OAuth { oauth_provider: &'static str },
    ApiKey { env_var: &'static str },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GatewayBackendBinding {
    pub(crate) backend_id: &'static str,
    pub(crate) routed_model: &'static str,
    pub(crate) actual_model: &'static str,
    pub(crate) provider_name: &'static str,
    pub(crate) provider_type: &'static str,
    pub(crate) advertised_capabilities: &'static [&'static str],
    pub(crate) required_capabilities: &'static [&'static str],
    provider_auth: GatewayProviderAuthConfig,
    auth_kind: GatewayIntegratedAuthKind,
}

const CLI_CODEX_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: DEFAULT_BACKEND,
    routed_model: DEFAULT_ROUTED_MODEL,
    actual_model: DEFAULT_ACTUAL_MODEL,
    provider_name: DEFAULT_PROVIDER_NAME,
    provider_type: "openai",
    advertised_capabilities: &[
        "agent_api.run",
        "agent_api.events",
        "agent_api.events.live",
        "agent_api.exec.non_interactive",
        "agent_api.exec.add_dirs.v1",
        "agent_api.session.resume.v1",
        "agent_api.session.fork.v1",
        "agent_api.session.handle.v1",
        "agent_api.control.cancel.v1",
        "agent_api.tools.structured.v1",
        "agent_api.tools.results.v1",
        "agent_api.artifacts.final_text.v1",
    ],
    required_capabilities: REQUIRED_GATEWAY_CAPABILITIES,
    provider_auth: GatewayProviderAuthConfig::OAuth {
        oauth_provider: DEFAULT_PROVIDER_NAME,
    },
    auth_kind: GatewayIntegratedAuthKind::CliCodex,
};

const API_OPENAI_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: API_OPENAI_BACKEND,
    routed_model: OPENAI_ROUTED_MODEL,
    actual_model: OPENAI_ACTUAL_MODEL,
    provider_name: OPENAI_PROVIDER_NAME,
    provider_type: "openai",
    advertised_capabilities: CLI_CODEX_BACKEND_BINDING.advertised_capabilities,
    required_capabilities: REQUIRED_GATEWAY_CAPABILITIES,
    provider_auth: GatewayProviderAuthConfig::ApiKey {
        env_var: OPENAI_API_KEY_ENV,
    },
    auth_kind: GatewayIntegratedAuthKind::ApiEnv,
};

const GATEWAY_BACKEND_BINDINGS: &[GatewayBackendBinding] =
    &[CLI_CODEX_BACKEND_BINDING, API_OPENAI_BACKEND_BINDING];

pub(crate) fn resolve_gateway_backend_binding(
    backend_id: &str,
) -> Option<&'static GatewayBackendBinding> {
    GATEWAY_BACKEND_BINDINGS
        .iter()
        .find(|binding| binding.backend_id == backend_id)
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
    pub binding: &'static GatewayBackendBinding,
    pub integrated_auth: Option<GatewayIntegratedAuthPayloadV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayRuntimeManifest {
    world_id: String,
    backend_id: String,
    pid: u32,
    pid_start_time_ticks: u64,
    port: u16,
    runtime_dir: PathBuf,
    config_path: PathBuf,
    state: GatewayRuntimeState,
}

#[derive(Debug)]
enum ManagedGatewayProcess {
    Child(Child),
    RediscoveredPid(u32),
}

#[derive(Debug, Clone)]
struct ManagedGatewayRuntime {
    world_id: String,
    backend_id: String,
    port: u16,
    runtime_dir: PathBuf,
    config_path: PathBuf,
    manifest_path: PathBuf,
    pid_start_time_ticks: u64,
    process: Arc<Mutex<ManagedGatewayProcess>>,
    state: Arc<RwLock<GatewayRuntimeState>>,
}

impl ManagedGatewayRuntime {
    fn set_state(&self, state: GatewayRuntimeState) {
        if let Ok(mut guard) = self.state.write() {
            *guard = state;
        }
        let _ = self.persist_manifest();
    }

    fn state(&self) -> GatewayRuntimeState {
        self.state
            .read()
            .map(|guard| *guard)
            .unwrap_or(GatewayRuntimeState::ProvisionedStopped)
    }

    fn pid(&self) -> Result<u32> {
        let guard = self
            .process
            .lock()
            .map_err(|_| anyhow!("gateway child lock poisoned"))?;
        Ok(match &*guard {
            ManagedGatewayProcess::Child(child) => child.id(),
            ManagedGatewayProcess::RediscoveredPid(pid) => *pid,
        })
    }

    fn persist_manifest(&self) -> Result<()> {
        let manifest = GatewayRuntimeManifest {
            world_id: self.world_id.clone(),
            backend_id: self.backend_id.clone(),
            pid: self.pid()?,
            pid_start_time_ticks: self.pid_start_time_ticks,
            port: self.port,
            runtime_dir: self.runtime_dir.clone(),
            config_path: self.config_path.clone(),
            state: self.state(),
        };
        write_runtime_manifest(&self.manifest_path, &manifest)
    }
}

#[derive(Default)]
struct GatewayLifecycleWorldState {
    op_lock: AsyncMutex<()>,
    state: RwLock<Option<GatewayRuntimeState>>,
}

impl GatewayLifecycleWorldState {
    fn state(&self) -> Option<GatewayRuntimeState> {
        self.state.read().ok().and_then(|guard| *guard)
    }

    fn replace_state(&self, state: Option<GatewayRuntimeState>) -> Option<GatewayRuntimeState> {
        match self.state.write() {
            Ok(mut guard) => std::mem::replace(&mut *guard, state),
            Err(_) => None,
        }
    }

    fn scoped_state(self: &Arc<Self>, state: GatewayRuntimeState) -> GatewayLifecycleStateGuard {
        let previous = self.replace_state(Some(state));
        GatewayLifecycleStateGuard {
            state: Arc::clone(self),
            previous,
        }
    }
}

struct GatewayLifecycleStateGuard {
    state: Arc<GatewayLifecycleWorldState>,
    previous: Option<GatewayRuntimeState>,
}

impl Drop for GatewayLifecycleStateGuard {
    fn drop(&mut self) {
        let _ = self.state.replace_state(self.previous);
    }
}

#[derive(Default)]
pub(crate) struct GatewayRuntimeManager {
    runtimes: Mutex<HashMap<String, ManagedGatewayRuntime>>,
    lifecycle: Mutex<HashMap<String, Arc<GatewayLifecycleWorldState>>>,
}

impl GatewayRuntimeManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn status(
        &self,
        world_id: &str,
        backend_id: &str,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        if let Some(state) = self.lifecycle_state_for_world(world_id) {
            return Err(lifecycle_status_transient_failure(world_id, state));
        }

        let Some(runtime) = self.runtime_for_world_or_manifest(world_id, backend_id) else {
            return Ok(unavailable_response());
        };

        let state = self.observe_runtime_state(&runtime).await?;
        Ok(match state {
            GatewayRuntimeState::Ready => available_response(runtime.port),
            GatewayRuntimeState::Starting | GatewayRuntimeState::RestartInProgress => {
                return Err(lifecycle_status_transient_failure(world_id, state));
            }
            GatewayRuntimeState::ProvisionedStopped | GatewayRuntimeState::AbsentComponent => {
                self.remove_runtime(world_id, backend_id);
                unavailable_response()
            }
        })
    }

    pub(crate) async fn sync(
        &self,
        ctx: GatewayRuntimeStartContext,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        self.sync_with_timeout(ctx, DEFAULT_READY_TIMEOUT).await
    }

    async fn sync_with_timeout(
        &self,
        ctx: GatewayRuntimeStartContext,
        ready_timeout: Duration,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        let lifecycle = self.lifecycle_for_world(&ctx.world_id);
        let _world_guard = lifecycle.op_lock.lock().await;
        self.sync_with_timeout_locked(ctx, ready_timeout, Some(&lifecycle))
            .await
    }

    async fn sync_with_timeout_locked(
        &self,
        ctx: GatewayRuntimeStartContext,
        ready_timeout: Duration,
        lifecycle: Option<&Arc<GatewayLifecycleWorldState>>,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        if let Some(runtime) =
            self.runtime_for_world_or_manifest(&ctx.world_id, ctx.binding.backend_id)
        {
            match self.observe_runtime_state(&runtime).await? {
                GatewayRuntimeState::Ready => return Ok(available_response(runtime.port)),
                GatewayRuntimeState::Starting | GatewayRuntimeState::RestartInProgress => {
                    return self.wait_until_ready(runtime, ready_timeout).await;
                }
                GatewayRuntimeState::ProvisionedStopped | GatewayRuntimeState::AbsentComponent => {
                    self.remove_runtime(&ctx.world_id, ctx.binding.backend_id);
                }
            }
        }

        let _state_guard = lifecycle.map(|state| state.scoped_state(GatewayRuntimeState::Starting));
        let Some(binary_path) = resolve_gateway_binary()? else {
            return Ok(unavailable_response());
        };

        let backend_id = ctx.binding.backend_id;
        let runtime = start_runtime(binary_path, ctx)?;
        let world_id = runtime.world_id.clone();
        self.insert_runtime(runtime.clone());
        match self.wait_until_ready(runtime, ready_timeout).await {
            Ok(response) => Ok(response),
            Err(err) => {
                if let Some(runtime) = self.take_runtime(&world_id) {
                    let _ = stop_runtime(runtime);
                }
                delete_runtime_manifest(&manifest_path_for_world(&world_id, backend_id));
                Err(err)
            }
        }
    }

    pub(crate) async fn restart(
        &self,
        ctx: GatewayRuntimeStartContext,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        let lifecycle = self.lifecycle_for_world(&ctx.world_id);
        let _world_guard = lifecycle.op_lock.lock().await;
        let _restart_state = lifecycle.scoped_state(GatewayRuntimeState::RestartInProgress);

        if let Some(existing) =
            self.runtime_for_world_or_manifest(&ctx.world_id, ctx.binding.backend_id)
        {
            existing.set_state(GatewayRuntimeState::RestartInProgress);
            self.take_runtime(&ctx.world_id);
            delete_runtime_manifest(&manifest_path_for_world(
                &ctx.world_id,
                ctx.binding.backend_id,
            ));
            stop_runtime(existing)
                .with_context(|| format!("failed to stop gateway runtime for {}", ctx.world_id))
                .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
        }

        self.sync_with_timeout_locked(ctx, DEFAULT_READY_TIMEOUT, None)
            .await
    }

    pub(crate) fn pid_for_world(&self, world_id: &str) -> Option<u32> {
        let runtime = self.runtime_for_world(world_id)?;
        runtime.pid().ok()
    }

    pub(crate) fn forget_runtime_for_test(&self, world_id: &str) {
        let _ = self.take_runtime(world_id);
    }

    fn runtime_for_world(&self, world_id: &str) -> Option<ManagedGatewayRuntime> {
        self.runtimes
            .lock()
            .ok()
            .and_then(|guard| guard.get(world_id).cloned())
    }

    fn lifecycle_for_world(&self, world_id: &str) -> Arc<GatewayLifecycleWorldState> {
        match self.lifecycle.lock() {
            Ok(mut guard) => Arc::clone(
                guard
                    .entry(world_id.to_string())
                    .or_insert_with(|| Arc::new(GatewayLifecycleWorldState::default())),
            ),
            Err(_) => Arc::new(GatewayLifecycleWorldState::default()),
        }
    }

    fn lifecycle_state_for_world(&self, world_id: &str) -> Option<GatewayRuntimeState> {
        self.lifecycle
            .lock()
            .ok()
            .and_then(|guard| guard.get(world_id).cloned())
            .and_then(|state| state.state())
    }

    fn runtime_for_world_or_manifest(
        &self,
        world_id: &str,
        backend_id: &str,
    ) -> Option<ManagedGatewayRuntime> {
        self.runtime_for_world(world_id)
            .or_else(|| self.recover_runtime(world_id, backend_id))
    }

    fn insert_runtime(&self, runtime: ManagedGatewayRuntime) {
        if let Ok(mut guard) = self.runtimes.lock() {
            guard.insert(runtime.world_id.clone(), runtime);
        }
    }

    fn remove_runtime(&self, world_id: &str, backend_id: &str) -> Option<ManagedGatewayRuntime> {
        let runtime = self.take_runtime(world_id);
        delete_runtime_manifest(&manifest_path_for_world(world_id, backend_id));
        runtime
    }

    fn take_runtime(&self, world_id: &str) -> Option<ManagedGatewayRuntime> {
        self.runtimes
            .lock()
            .ok()
            .and_then(|mut guard| guard.remove(world_id))
    }

    fn recover_runtime(&self, world_id: &str, backend_id: &str) -> Option<ManagedGatewayRuntime> {
        let manifest_path = manifest_path_for_world(world_id, backend_id);
        let manifest = match read_runtime_manifest(&manifest_path) {
            Ok(manifest) => manifest,
            Err(_) => {
                delete_runtime_manifest(&manifest_path);
                return None;
            }
        };
        let expected_runtime_dir = runtime_dir_for_world(world_id, backend_id);
        let artifacts_exist = manifest.runtime_dir.is_dir() && manifest.config_path.is_file();
        let start_time_matches = read_pid_start_time_ticks(manifest.pid)
            .map(|start_time| start_time == manifest.pid_start_time_ticks)
            .unwrap_or(false);
        if manifest.world_id != world_id
            || manifest.backend_id != backend_id
            || manifest.runtime_dir != expected_runtime_dir
            || !artifacts_exist
            || !pid_is_running(manifest.pid)
            || !start_time_matches
            || !gateway_health_ready_blocking(manifest.port)
        {
            delete_runtime_manifest(&manifest_path);
            return None;
        }

        let runtime = ManagedGatewayRuntime {
            world_id: manifest.world_id,
            backend_id: manifest.backend_id,
            port: manifest.port,
            runtime_dir: manifest.runtime_dir,
            config_path: manifest.config_path,
            manifest_path,
            pid_start_time_ticks: manifest.pid_start_time_ticks,
            process: Arc::new(Mutex::new(ManagedGatewayProcess::RediscoveredPid(
                manifest.pid,
            ))),
            state: Arc::new(RwLock::new(manifest.state)),
        };
        self.insert_runtime(runtime.clone());
        Some(runtime)
    }

    async fn observe_runtime_state(
        &self,
        runtime: &ManagedGatewayRuntime,
    ) -> Result<GatewayRuntimeState, GatewayRuntimeFailure> {
        let is_running = {
            let mut process = runtime
                .process
                .lock()
                .map_err(|_| GatewayRuntimeFailure::transient("gateway child lock poisoned"))?;
            process_is_running(&mut process, &runtime.runtime_dir)
        }
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

        if !is_running {
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
    validate_binding_capabilities(ctx.binding)?;
    let port = pick_free_port().map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    let runtime_dir = runtime_dir_for_world(&ctx.world_id, ctx.binding.backend_id);
    let home_dir = runtime_dir.join("home");
    ensure_directory_with_mode(&runtime_dir, GATEWAY_RUNTIME_DIR_MODE)
        .with_context(|| {
            format!(
                "failed to create gateway runtime directory {}",
                runtime_dir.display()
            )
        })
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    ensure_directory_with_mode(&home_dir, GATEWAY_RUNTIME_DIR_MODE)
        .with_context(|| {
            format!(
                "failed to create gateway runtime directory {}",
                home_dir.display()
            )
        })
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let config_path = runtime_dir.join("config.toml");
    let config = render_integrated_config(port, ctx.binding);
    write_file_with_mode(&config_path, config.as_bytes(), GATEWAY_RUNTIME_FILE_MODE)
        .with_context(|| format!("failed to write {}", config_path.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let stdout_log = runtime_dir.join("stdout.log");
    let stderr_log = runtime_dir.join("stderr.log");
    let stdout = create_file_with_mode(&stdout_log, GATEWAY_RUNTIME_FILE_MODE)
        .with_context(|| format!("failed to create {}", stdout_log.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    let stderr = create_file_with_mode(&stderr_log, GATEWAY_RUNTIME_FILE_MODE)
        .with_context(|| format!("failed to create {}", stderr_log.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let auth = resolve_integrated_auth_handoff(ctx.binding, ctx.integrated_auth)?;

    let mut command = Command::new(&binary_path);
    command
        .current_dir(&ctx.project_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .env("HOME", &home_dir)
        .env(GATEWAY_LAUNCH_MODE_ENV, GATEWAY_MODE_IN_WORLD)
        .env(GATEWAY_LAUNCH_CONFIG_PATH_ENV, &config_path)
        .env(GATEWAY_LAUNCH_DISABLE_TOKEN_PERSISTENCE_ENV, "1");

    for env_key in KNOWN_GATEWAY_AUTH_ENV_VARS {
        command.env_remove(env_key);
    }
    for (env_key, value) in auth.env_vars {
        command.env(env_key, value);
    }
    append_gateway_start_args(&mut command, &config_path);

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", binary_path.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    if let Err(err) =
        attach_child_to_cgroup(child.id(), &ctx.cgroup_path, ctx.require_cgroup_attach)
    {
        let _ = kill_child_process(&mut child);
        return Err(err);
    }
    let pid_start_time_ticks = read_pid_start_time_ticks(child.id()).map_err(|err| {
        let _ = kill_child_process(&mut child);
        GatewayRuntimeFailure::transient(err.to_string())
    })?;

    let world_id = ctx.world_id;
    let runtime = ManagedGatewayRuntime {
        world_id: world_id.clone(),
        backend_id: ctx.binding.backend_id.to_string(),
        port,
        runtime_dir,
        config_path,
        manifest_path: manifest_path_for_world(&world_id, ctx.binding.backend_id),
        pid_start_time_ticks,
        process: Arc::new(Mutex::new(ManagedGatewayProcess::Child(child))),
        state: Arc::new(RwLock::new(GatewayRuntimeState::Starting)),
    };
    if let Err(err) = runtime.persist_manifest() {
        if let Ok(mut process) = runtime.process.lock() {
            let _ = stop_process(&mut process);
        }
        delete_runtime_manifest(&runtime.manifest_path);
        return Err(GatewayRuntimeFailure::transient(err.to_string()));
    }

    Ok(runtime)
}

fn stop_runtime(runtime: ManagedGatewayRuntime) -> Result<()> {
    let mut process = runtime
        .process
        .lock()
        .map_err(|_| anyhow!("gateway child lock poisoned"))?;
    stop_process(&mut process)
}

fn attach_child_to_cgroup(
    pid: u32,
    cgroup_path: &Path,
    required: bool,
) -> Result<(), GatewayRuntimeFailure> {
    let cgroup_procs = cgroup_path.join("cgroup.procs");
    match fs::write(&cgroup_procs, pid.to_string()) {
        Ok(()) => Ok(()),
        Err(_err) if !required => Ok(()),
        Err(err) => Err(GatewayRuntimeFailure::transient(format!(
            "failed to attach gateway pid {} to {}: {}",
            pid,
            cgroup_procs.display(),
            err
        ))),
    }
}

fn append_gateway_start_args(command: &mut Command, config_path: &Path) {
    command.arg("--config").arg(config_path).arg("start");
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

fn runtime_dir_for_world(world_id: &str, backend_id: &str) -> PathBuf {
    backend_runtime_root_dir(backend_id).join(world_id)
}

fn gateway_runtime_root_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("SUBSTRATE_GATEWAY_RUNTIME_ROOT") {
        let path = PathBuf::from(path);
        let _ = ensure_directory_with_mode(&path, GATEWAY_RUNTIME_DIR_MODE);
        return path;
    }

    let run_dir = PathBuf::from("/run/substrate").join(GATEWAY_RUNTIME_ROOT_DIR);
    if ensure_directory_with_mode(&run_dir, GATEWAY_RUNTIME_DIR_MODE).is_ok() {
        return run_dir;
    }

    let temp_dir = std::env::temp_dir().join(GATEWAY_RUNTIME_ROOT_DIR);
    let _ = ensure_directory_with_mode(&temp_dir, GATEWAY_RUNTIME_DIR_MODE);
    temp_dir
}

fn backend_runtime_root_dir(backend_id: &str) -> PathBuf {
    let path = gateway_runtime_root_dir().join(runtime_backend_dir_name(backend_id));
    let _ = ensure_directory_with_mode(&path, GATEWAY_RUNTIME_DIR_MODE);
    path
}

fn runtime_backend_dir_name(backend_id: &str) -> String {
    backend_id
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' | ':' => ch,
            _ => '_',
        })
        .collect()
}

fn manifest_path_for_world(world_id: &str, backend_id: &str) -> PathBuf {
    runtime_dir_for_world(world_id, backend_id).join(GATEWAY_RUNTIME_MANIFEST_NAME)
}

fn write_runtime_manifest(path: &Path, manifest: &GatewayRuntimeManifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory_with_mode(parent, GATEWAY_RUNTIME_DIR_MODE).with_context(|| {
            format!("failed to create gateway manifest dir {}", parent.display())
        })?;
    }
    let encoded =
        serde_json::to_vec_pretty(manifest).context("failed to encode gateway manifest")?;
    write_file_with_mode(path, &encoded, GATEWAY_RUNTIME_FILE_MODE)
        .with_context(|| format!("failed to write gateway manifest {}", path.display()))
}

fn ensure_directory_with_mode(path: &Path, mode: u32) -> Result<()> {
    fs::create_dir_all(path)?;
    set_path_mode(path, mode)
}

fn create_file_with_mode(path: &Path, mode: u32) -> Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    options.mode(mode);
    let file = options.open(path)?;
    set_path_mode(path, mode)?;
    Ok(file)
}

fn write_file_with_mode(path: &Path, content: &[u8], mode: u32) -> Result<()> {
    let mut file = create_file_with_mode(path, mode)?;
    file.write_all(content)?;
    Ok(())
}

fn set_path_mode(path: &Path, mode: u32) -> Result<()> {
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

fn read_runtime_manifest(path: &Path) -> Result<GatewayRuntimeManifest> {
    let content = fs::read(path)
        .with_context(|| format!("failed to read gateway manifest {}", path.display()))?;
    serde_json::from_slice(&content)
        .with_context(|| format!("failed to parse gateway manifest {}", path.display()))
}

fn delete_runtime_manifest(path: &Path) {
    if let Err(err) = fs::remove_file(path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(error = %err, manifest = %path.display(), "failed to remove gateway runtime manifest");
        }
    }
}

fn render_integrated_config(port: u16, binding: &GatewayBackendBinding) -> String {
    let provider_auth = render_provider_auth_config(binding);
    format!(
        r#"[server]
host = "127.0.0.1"
port = {port}
log_level = "info"

[router]
default = "{routed_model}"

[[providers]]
name = "{provider_name}"
provider_type = "{provider_type}"
{provider_auth}
models = ["{actual_model}"]
enabled = true

[[models]]
name = "{routed_model}"

[[models.mappings]]
priority = 1
provider = "{provider_name}"
actual_model = "{actual_model}"
"#,
        routed_model = binding.routed_model,
        provider_name = binding.provider_name,
        provider_type = binding.provider_type,
        provider_auth = provider_auth,
        actual_model = binding.actual_model,
    )
}

fn render_provider_auth_config(binding: &GatewayBackendBinding) -> String {
    match binding.provider_auth {
        GatewayProviderAuthConfig::OAuth { oauth_provider } => {
            format!("auth_type = \"oauth\"\noauth_provider = \"{oauth_provider}\"")
        }
        GatewayProviderAuthConfig::ApiKey { env_var } => {
            format!("auth_type = \"apikey\"\napi_key = \"${env_var}\"")
        }
    }
}

struct ResolvedGatewayAuthHandoff {
    env_vars: Vec<(&'static str, String)>,
}

fn resolve_integrated_auth_handoff(
    binding: &GatewayBackendBinding,
    auth: Option<GatewayIntegratedAuthPayloadV1>,
) -> Result<ResolvedGatewayAuthHandoff, GatewayRuntimeFailure> {
    let Some(auth) = auth else {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "missing request-provided integrated auth handoff for {}",
            binding.backend_id
        )));
    };

    if auth.backend_id.trim() != binding.backend_id {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "request-provided integrated auth payload for '{}' does not match selected backend '{}'",
            auth.backend_id.trim(),
            binding.backend_id
        )));
    }

    match binding.auth_kind {
        GatewayIntegratedAuthKind::CliCodex => resolve_codex_auth_handoff(auth.cli_codex.clone()),
        GatewayIntegratedAuthKind::ApiEnv => resolve_api_env_auth_handoff(binding, &auth),
    }
}

fn resolve_codex_auth_handoff(
    auth: Option<GatewayCliCodexIntegratedAuthV1>,
) -> Result<ResolvedGatewayAuthHandoff, GatewayRuntimeFailure> {
    let Some(auth) = auth else {
        return Err(GatewayRuntimeFailure::invalid_integration(
            "missing request-provided integrated auth handoff for cli:codex",
        ));
    };

    let access_token = auth.access_token.trim().to_string();
    if access_token.is_empty() {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "request-provided {} is empty",
            CODEX_ACCESS_TOKEN_ENV
        )));
    }

    let account_id = auth
        .account_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut env_vars = vec![(CODEX_ACCESS_TOKEN_ENV, access_token)];
    if let Some(account_id) = account_id {
        env_vars.push((CODEX_ACCOUNT_ID_ENV, account_id));
    }

    Ok(ResolvedGatewayAuthHandoff { env_vars })
}

fn resolve_api_env_auth_handoff(
    binding: &GatewayBackendBinding,
    auth: &GatewayIntegratedAuthPayloadV1,
) -> Result<ResolvedGatewayAuthHandoff, GatewayRuntimeFailure> {
    let GatewayProviderAuthConfig::ApiKey { env_var } = binding.provider_auth else {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "backend '{}' is not configured for api env auth",
            binding.backend_id
        )));
    };

    let Some(api_env) = auth.api_env.as_ref() else {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "missing request-provided integrated auth handoff for {} (expected api_env '{}')",
            binding.backend_id, env_var
        )));
    };

    let Some(value) = api_env.env.get(env_var) else {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "request-provided integrated auth payload for '{}' is missing api_env '{}'",
            auth.backend_id, env_var
        )));
    };
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "request-provided api_env '{}' is empty",
            env_var
        )));
    }

    Ok(ResolvedGatewayAuthHandoff {
        env_vars: vec![(env_var, value)],
    })
}

fn validate_binding_capabilities(
    binding: &GatewayBackendBinding,
) -> Result<(), GatewayRuntimeFailure> {
    for capability in binding.required_capabilities {
        if !binding
            .advertised_capabilities
            .iter()
            .any(|advertised| advertised == capability)
        {
            return Err(GatewayRuntimeFailure::transient(format!(
                "backend '{}' does not advertise required capability '{}'",
                binding.backend_id, capability
            )));
        }
    }

    Ok(())
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

fn process_is_running(process: &mut ManagedGatewayProcess, runtime_dir: &Path) -> Result<bool> {
    match process {
        ManagedGatewayProcess::Child(child) => child
            .try_wait()
            .with_context(|| {
                format!(
                    "failed to inspect gateway child status for {}",
                    runtime_dir.display()
                )
            })
            .map(|status| status.is_none()),
        ManagedGatewayProcess::RediscoveredPid(pid) => Ok(pid_is_running(*pid)),
    }
}

fn stop_process(process: &mut ManagedGatewayProcess) -> Result<()> {
    match process {
        ManagedGatewayProcess::Child(child) => kill_child_process(child),
        ManagedGatewayProcess::RediscoveredPid(pid) => kill_pid(*pid),
    }
}

fn kill_child_process(child: &mut Child) -> Result<()> {
    if child.try_wait()?.is_none() {
        child.kill().context("failed to kill gateway child")?;
        let _ = child.wait();
    }
    Ok(())
}

fn kill_pid(pid: u32) -> Result<()> {
    if process_has_exited(pid) {
        return Ok(());
    }

    let rc = unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() != Some(libc::ESRCH) {
            return Err(err).context(format!("failed to kill gateway pid {pid}"));
        }
        return Ok(());
    }

    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        if process_has_exited(pid) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    Err(anyhow!("gateway pid {pid} did not exit after SIGKILL"))
}

fn pid_is_running(pid: u32) -> bool {
    let rc = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if rc == 0 {
        return true;
    }

    match std::io::Error::last_os_error().raw_os_error() {
        Some(code) if code == libc::EPERM => true,
        Some(code) if code == libc::ESRCH => false,
        _ => false,
    }
}

fn process_has_exited(pid: u32) -> bool {
    reap_pid_if_possible(pid) || !pid_is_running(pid)
}

fn reap_pid_if_possible(pid: u32) -> bool {
    let mut status = 0;
    let rc = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::WNOHANG) };
    if rc > 0 {
        return true;
    }
    false
}

fn read_pid_start_time_ticks(pid: u32) -> Result<u64> {
    let stat_path = PathBuf::from("/proc").join(pid.to_string()).join("stat");
    let stat = fs::read_to_string(&stat_path)
        .with_context(|| format!("failed to read {}", stat_path.display()))?;
    parse_pid_start_time_ticks(&stat)
        .with_context(|| format!("failed to parse {}", stat_path.display()))
}

fn parse_pid_start_time_ticks(stat: &str) -> Result<u64> {
    let (_, rest) = stat
        .rsplit_once(") ")
        .ok_or_else(|| anyhow!("missing comm terminator in /proc stat"))?;
    let field = rest
        .split_whitespace()
        .nth(19)
        .ok_or_else(|| anyhow!("missing start time field in /proc stat"))?;
    field
        .parse::<u64>()
        .context("invalid start time field in /proc stat")
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

    let mut status_line = String::new();
    let mut reader = BufReader::new(stream);
    if reader.read_line(&mut status_line).is_err() {
        return false;
    }
    status_line.starts_with("HTTP/1.1 200") || status_line.starts_with("HTTP/1.0 200")
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

fn lifecycle_status_transient_failure(
    world_id: &str,
    state: GatewayRuntimeState,
) -> GatewayRuntimeFailure {
    let action = match state {
        GatewayRuntimeState::Starting => "starting",
        GatewayRuntimeState::RestartInProgress => "restarting",
        GatewayRuntimeState::Ready => "ready",
        GatewayRuntimeState::ProvisionedStopped => "stopped",
        GatewayRuntimeState::AbsentComponent => "absent",
    };
    GatewayRuntimeFailure::transient(format!(
        "gateway runtime for world {world_id} is still {action}"
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use agent_api_types::{
        GatewayApiEnvIntegratedAuthV1, GatewayCliCodexIntegratedAuthV1,
        GatewayIntegratedAuthPayloadV1,
    };
    use once_cell::sync::Lazy;
    use std::io::{Read, Write};
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    static ENV_LOCK: Lazy<AsyncMutex<()>> = Lazy::new(|| AsyncMutex::new(()));
    const MISSING_CAPABILITY_BINDING: GatewayBackendBinding = GatewayBackendBinding {
        backend_id: "test:missing-capability",
        routed_model: "broken",
        actual_model: "broken",
        provider_name: "broken-provider",
        provider_type: "openai",
        advertised_capabilities: &["agent_api.events"],
        required_capabilities: REQUIRED_GATEWAY_CAPABILITIES,
        provider_auth: GatewayProviderAuthConfig::OAuth {
            oauth_provider: "broken-provider",
        },
        auth_kind: GatewayIntegratedAuthKind::CliCodex,
    };

    struct EnvGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl Into<std::ffi::OsString>) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value.into());
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn start_context(project_dir: &Path, world_id: &str) -> GatewayRuntimeStartContext {
        let binding = resolve_gateway_backend_binding(DEFAULT_BACKEND).expect("codex binding");
        start_context_with_binding(project_dir, world_id, binding)
    }

    fn start_context_with_binding(
        project_dir: &Path,
        world_id: &str,
        binding: &'static GatewayBackendBinding,
    ) -> GatewayRuntimeStartContext {
        GatewayRuntimeStartContext {
            world_id: world_id.to_string(),
            project_dir: project_dir.to_path_buf(),
            cgroup_path: project_dir.join("missing-cgroup"),
            require_cgroup_attach: false,
            binding,
            integrated_auth: integrated_auth_for_binding(binding),
        }
    }

    fn integrated_auth_for_binding(
        binding: &GatewayBackendBinding,
    ) -> Option<GatewayIntegratedAuthPayloadV1> {
        match binding.backend_id {
            DEFAULT_BACKEND => Some(GatewayIntegratedAuthPayloadV1 {
                backend_id: binding.backend_id.to_string(),
                cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                    account_id: Some("acct_test".to_string()),
                    access_token: "header.payload.signature".to_string(),
                }),
                api_env: None,
            }),
            API_OPENAI_BACKEND => Some(openai_integrated_auth_payload("sk-openai-test")),
            _ => None,
        }
    }

    fn openai_integrated_auth_payload(api_key: &str) -> GatewayIntegratedAuthPayloadV1 {
        let mut env = HashMap::new();
        env.insert(OPENAI_API_KEY_ENV.to_string(), api_key.to_string());
        GatewayIntegratedAuthPayloadV1 {
            backend_id: API_OPENAI_BACKEND.to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        }
    }

    fn delayed_gateway_binary(temp_dir: &TempDir, delay_ms: u64) -> (PathBuf, PathBuf, PathBuf) {
        let path = temp_dir.path().join("delayed-gateway.sh");
        let pid_dir = temp_dir.path().join("pids");
        let launch_count_path = temp_dir.path().join("launch-count.txt");
        fs::write(
            &path,
            format!(
                r#"#!/bin/sh
set -eu
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      config="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$config" ]; then
  echo "missing --config" >&2
  exit 64
fi

if [ -z "${{SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN:-}}" ]; then
  echo "missing Codex access token env" >&2
  exit 65
fi

launch="$(python3 - "{launch_count_path}" <<'PY'
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
count = int(path.read_text(encoding='utf-8').strip()) if path.exists() else 0
count += 1
path.write_text(str(count), encoding='utf-8')
print(count)
PY
)"
mkdir -p "{pid_dir}"
printf '%s\n' "$$" >"{pid_dir}/$launch.pid"

port="$(python3 - "$config" <<'PY'
import re
import sys
text = open(sys.argv[1], 'r', encoding='utf-8').read()
match = re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M)
if not match:
    raise SystemExit(64)
print(match.group(1))
PY
)"

sleep {delay_s}
root="$(dirname "$config")/serve"
mkdir -p "$root"
printf 'ok' >"$root/health"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root"
"#,
                launch_count_path = launch_count_path.display(),
                pid_dir = pid_dir.display(),
                delay_s = format_delay_seconds(delay_ms),
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        (path, pid_dir, launch_count_path)
    }

    fn first_launch_hangs_second_ready_binary(temp_dir: &TempDir) -> (PathBuf, PathBuf, PathBuf) {
        let path = temp_dir.path().join("phased-gateway.sh");
        let pid_dir = temp_dir.path().join("pids");
        let launch_count_path = temp_dir.path().join("launch-count.txt");
        fs::write(
            &path,
            format!(
                r#"#!/bin/sh
set -eu
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      config="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$config" ]; then
  echo "missing --config" >&2
  exit 64
fi

if [ -z "${{SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN:-}}" ]; then
  echo "missing Codex access token env" >&2
  exit 65
fi

launch="$(python3 - "{launch_count_path}" <<'PY'
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
count = int(path.read_text(encoding='utf-8').strip()) if path.exists() else 0
count += 1
path.write_text(str(count), encoding='utf-8')
print(count)
PY
)"
mkdir -p "{pid_dir}"
printf '%s\n' "$$" >"{pid_dir}/$launch.pid"

if [ "$launch" = "1" ]; then
  sleep 30
  exit 0
fi

port="$(python3 - "$config" <<'PY'
import re
import sys
text = open(sys.argv[1], 'r', encoding='utf-8').read()
match = re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M)
if not match:
    raise SystemExit(64)
print(match.group(1))
PY
)"
root="$(dirname "$config")/serve"
mkdir -p "$root"
printf 'ok' >"$root/health"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root"
"#,
                launch_count_path = launch_count_path.display(),
                pid_dir = pid_dir.display(),
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        (path, pid_dir, launch_count_path)
    }

    fn format_delay_seconds(delay_ms: u64) -> String {
        format!("{:.3}", delay_ms as f64 / 1000.0)
    }

    fn wait_for_pid(pid_dir: &Path, launch: u32) -> u32 {
        let path = pid_dir.join(format!("{launch}.pid"));
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Ok(raw) = fs::read_to_string(&path) {
                let pid = raw.trim().parse::<u32>().expect("parse pid");
                if pid > 0 {
                    return pid;
                }
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for {}",
                path.display()
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    fn read_launch_count(path: &Path) -> u32 {
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| raw.trim().parse::<u32>().ok())
            .unwrap_or(0)
    }

    fn assert_process_exited(pid: u32) {
        let rc = unsafe { libc::kill(pid as i32, 0) };
        assert_eq!(rc, -1, "expected pid {pid} to be gone");
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH),
            "pid {pid} should be gone",
        );
    }

    fn assert_mode(path: &Path, expected: u32) {
        let actual = fs::metadata(path)
            .unwrap_or_else(|_| panic!("missing {}", path.display()))
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(actual, expected, "unexpected mode for {}", path.display());
    }

    fn start_strict_health_server() -> u16 {
        let listener = StdTcpListener::bind(("127.0.0.1", 0)).expect("bind strict health server");
        let port = listener
            .local_addr()
            .expect("strict health server addr")
            .port();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept strict health connection");
            stream
                .set_read_timeout(Some(Duration::from_millis(150)))
                .expect("set strict health read timeout");

            let mut request = Vec::new();
            let mut buf = [0_u8; 256];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => return,
                    Ok(read) => {
                        request.extend_from_slice(&buf[..read]);
                        if request.windows(4).any(|window| window == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(err) => panic!("failed reading strict health request: {err}"),
                }
            }

            let mut probe = [0_u8; 1];
            match stream.read(&mut probe) {
                Ok(0) => {}
                Ok(_) => panic!("unexpected extra bytes after strict health request"),
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        || err.kind() == std::io::ErrorKind::TimedOut =>
                {
                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
                        )
                        .expect("write strict health response");
                }
                Err(err) => panic!("failed probing strict health client state: {err}"),
            }
        });
        port
    }

    fn legacy_gateway_health_probe_blocking(port: u16) -> bool {
        let mut stream = match TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}")
                .parse()
                .expect("legacy health socket address"),
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
        let _ = stream.shutdown(std::net::Shutdown::Write);

        let mut response = String::new();
        if stream.read_to_string(&mut response).is_err() {
            return false;
        }
        response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200")
    }

    #[test]
    fn append_gateway_start_args_uses_global_config_flag_position() {
        let mut command = Command::new("/bin/true");
        let config_path = Path::new("/tmp/config.toml");
        append_gateway_start_args(&mut command, config_path);

        let args: Vec<_> = command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["--config", "/tmp/config.toml", "start"]);
    }

    #[test]
    fn empty_default_backend_stays_invalid() {
        let mut env = HashMap::new();
        env.insert(
            GATEWAY_REQUEST_DEFAULT_BACKEND_ENV.to_string(),
            "   ".to_string(),
        );

        let err = GatewayControlSettings::from_request_env(Some(&env)).expect_err("empty backend");
        assert!(
            matches!(err, GatewayRuntimeFailure::InvalidIntegration(_)),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn nonempty_unbound_backend_is_accepted_as_selected_input() {
        let mut env = HashMap::new();
        env.insert(
            GATEWAY_REQUEST_DEFAULT_BACKEND_ENV.to_string(),
            "api:anthropic".to_string(),
        );

        let control =
            GatewayControlSettings::from_request_env(Some(&env)).expect("selected backend");
        assert_eq!(control.default_backend, "api:anthropic");
    }

    #[test]
    fn binding_lookup_includes_explicit_openai_proof_target() {
        let binding = resolve_gateway_backend_binding(API_OPENAI_BACKEND).expect("openai binding");
        assert_eq!(binding.backend_id, API_OPENAI_BACKEND);
        assert_eq!(binding.provider_name, OPENAI_PROVIDER_NAME);
    }

    #[test]
    fn binding_lookup_returns_none_for_unbound_backend() {
        assert!(resolve_gateway_backend_binding("api:anthropic").is_none());
    }

    #[test]
    fn integrated_config_rendering_is_binding_driven() {
        let codex_binding =
            resolve_gateway_backend_binding(DEFAULT_BACKEND).expect("codex binding");
        let codex_config = render_integrated_config(4317, codex_binding);
        assert!(codex_config.contains("auth_type = \"oauth\""));
        assert!(codex_config.contains("oauth_provider = \"openai-codex\""));
        assert!(!codex_config.contains("api_key = \"$OPENAI_API_KEY\""));

        let openai_binding =
            resolve_gateway_backend_binding(API_OPENAI_BACKEND).expect("openai binding");
        let openai_config = render_integrated_config(4318, openai_binding);
        assert!(openai_config.contains("auth_type = \"apikey\""));
        assert!(openai_config.contains("api_key = \"$OPENAI_API_KEY\""));
        assert!(!openai_config.contains("oauth_provider ="));
    }

    #[test]
    fn openai_auth_handoff_uses_api_env_when_available() {
        let binding = resolve_gateway_backend_binding(API_OPENAI_BACKEND).expect("openai binding");
        let auth = resolve_integrated_auth_handoff(
            binding,
            Some(openai_integrated_auth_payload("sk-openai-proof")),
        )
        .expect("openai auth handoff");

        assert_eq!(
            auth.env_vars,
            vec![(OPENAI_API_KEY_ENV, "sk-openai-proof".to_string())]
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn capability_gate_fails_before_runtime_artifacts_are_created() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let world_id = "missing-capability";

        let err = start_runtime(
            temp_dir.path().join("missing-binary"),
            start_context_with_binding(temp_dir.path(), world_id, &MISSING_CAPABILITY_BINDING),
        )
        .expect_err("capability gate should fail before spawn");

        assert!(
            matches!(err, GatewayRuntimeFailure::Transient(_)),
            "unexpected error: {err:?}"
        );
        assert!(
            err.to_string().contains("required capability"),
            "capability gate should explain the missing requirement: {err}"
        );
        assert!(
            !runtime_dir_for_world(world_id, MISSING_CAPABILITY_BINDING.backend_id).exists(),
            "pre-spawn capability gating should not create runtime artifacts"
        );
    }

    #[test]
    fn gateway_health_probe_accepts_server_that_rejects_half_closed_clients() {
        let port = start_strict_health_server();
        assert!(
            gateway_health_ready_blocking(port),
            "fixed readiness probe should accept strict health server",
        );
    }

    #[test]
    fn legacy_gateway_health_probe_fails_against_strict_server() {
        let port = start_strict_health_server();
        assert!(
            !legacy_gateway_health_probe_blocking(port),
            "legacy half-close readiness probe should fail against strict health server",
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn managed_runtime_artifacts_use_group_readable_modes() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, _pid_dir, _launch_count_path) = delayed_gateway_binary(&temp_dir, 0);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);

        let project_dir = temp_dir.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let runtime = start_runtime(
            PathBuf::from(std::env::var_os(GATEWAY_BINARY_OVERRIDE_ENV).unwrap()),
            start_context(&project_dir, "world-modes"),
        )
        .expect("create runtime");

        assert_mode(&runtime_root, GATEWAY_RUNTIME_DIR_MODE);
        let backend_root = backend_runtime_root_dir(DEFAULT_BACKEND);
        assert_mode(&backend_root, GATEWAY_RUNTIME_DIR_MODE);
        assert_eq!(runtime.runtime_dir.parent(), Some(backend_root.as_path()));
        assert_mode(&runtime.runtime_dir, GATEWAY_RUNTIME_DIR_MODE);
        assert_mode(&runtime.runtime_dir.join("home"), GATEWAY_RUNTIME_DIR_MODE);
        assert_mode(&runtime.config_path, GATEWAY_RUNTIME_FILE_MODE);
        assert_mode(
            &runtime.runtime_dir.join("stdout.log"),
            GATEWAY_RUNTIME_FILE_MODE,
        );
        assert_mode(
            &runtime.runtime_dir.join("stderr.log"),
            GATEWAY_RUNTIME_FILE_MODE,
        );
        assert_mode(&runtime.manifest_path, GATEWAY_RUNTIME_FILE_MODE);

        stop_runtime(runtime).expect("stop runtime");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_same_world_sync_reuses_one_runtime() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, pid_dir, launch_count_path) = delayed_gateway_binary(&temp_dir, 350);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);
        let manager = Arc::new(GatewayRuntimeManager::new());
        let ctx = start_context(temp_dir.path(), "same-world");

        let left = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.sync_with_timeout(ctx, Duration::from_secs(3)).await }
        });
        let right = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.sync_with_timeout(ctx, Duration::from_secs(3)).await }
        });

        let left = left.await.unwrap().expect("left sync");
        let right = right.await.unwrap().expect("right sync");
        assert_eq!(left.status, GatewayStatusV1::Available);
        assert_eq!(right.status, GatewayStatusV1::Available);
        assert_eq!(read_launch_count(&launch_count_path), 1);
        wait_for_pid(&pid_dir, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn timed_out_cleanup_does_not_kill_next_same_world_runtime() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, pid_dir, launch_count_path) =
            first_launch_hangs_second_ready_binary(&temp_dir);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);
        let manager = Arc::new(GatewayRuntimeManager::new());
        let ctx = start_context(temp_dir.path(), "cleanup-safe");

        let first = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move {
                manager
                    .sync_with_timeout(ctx, Duration::from_millis(250))
                    .await
            }
        });
        let first_pid = wait_for_pid(&pid_dir, 1);
        let second = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.sync_with_timeout(ctx, Duration::from_secs(3)).await }
        });

        let first_err = first
            .await
            .unwrap()
            .expect_err("first sync should time out");
        assert!(
            matches!(first_err, GatewayRuntimeFailure::Transient(_)),
            "unexpected error: {first_err:?}"
        );
        let second = second.await.unwrap().expect("second sync should recover");
        assert_eq!(second.status, GatewayStatusV1::Available);
        assert_eq!(read_launch_count(&launch_count_path), 2);
        assert_process_exited(first_pid);
        let second_pid = wait_for_pid(&pid_dir, 2);
        assert_ne!(first_pid, second_pid);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn status_during_start_returns_transient_failure() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, pid_dir, _) = delayed_gateway_binary(&temp_dir, 1000);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);
        let manager = Arc::new(GatewayRuntimeManager::new());
        let ctx = start_context(temp_dir.path(), "status-start");

        let sync = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.sync_with_timeout(ctx, Duration::from_secs(3)).await }
        });
        wait_for_pid(&pid_dir, 1);

        let err = manager
            .status("status-start", DEFAULT_BACKEND)
            .await
            .expect_err("status should surface a transient start failure");
        assert!(
            matches!(err, GatewayRuntimeFailure::Transient(_)),
            "unexpected error: {err:?}"
        );
        assert!(
            err.to_string().contains("starting"),
            "unexpected error text: {err}"
        );

        let response = sync.await.unwrap().expect("sync should finish");
        assert_eq!(response.status, GatewayStatusV1::Available);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn status_during_restart_returns_transient_failure() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (ready_binary, _, _) = delayed_gateway_binary(&temp_dir, 0);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, &ready_binary);
        let manager = Arc::new(GatewayRuntimeManager::new());
        let ctx = start_context(temp_dir.path(), "restart-start");

        manager
            .sync_with_timeout(ctx.clone(), Duration::from_secs(3))
            .await
            .expect("initial sync");

        let (restart_binary, pid_dir, _) = delayed_gateway_binary(&temp_dir, 1000);
        std::env::set_var(GATEWAY_BINARY_OVERRIDE_ENV, restart_binary);
        let restart = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.restart(ctx).await }
        });
        wait_for_pid(&pid_dir, 1);

        let deadline = Instant::now() + Duration::from_secs(1);
        let err = loop {
            match manager.status("restart-start", DEFAULT_BACKEND).await {
                Err(err) => break err,
                Ok(response) => {
                    assert_ne!(response.status, GatewayStatusV1::Unavailable);
                }
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for restart-in-progress status"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        };
        assert!(
            matches!(err, GatewayRuntimeFailure::Transient(_)),
            "unexpected error: {err:?}"
        );
        assert!(
            err.to_string().contains("restarting"),
            "unexpected error text: {err}"
        );

        let response = restart.await.unwrap().expect("restart should finish");
        assert_eq!(response.status, GatewayStatusV1::Available);
    }
}
