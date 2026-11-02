use std::{
    collections::{BTreeMap, HashMap},
    env,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    time,
};

use super::{
    McpConfigError, McpConfigManager, McpServerDefinition, McpServerEntry, McpToolConfig,
    McpTransport, StdioServerConfig, StdioServerDefinition, StreamableHttpDefinition,
};

/// Resolved runtime configuration for an MCP server, ready for spawning or connecting.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpRuntimeServer {
    pub name: String,
    pub transport: McpRuntimeTransport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<McpToolConfig>,
}

/// Transport-specific runtime configuration.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum McpRuntimeTransport {
    Stdio(StdioServerDefinition),
    StreamableHttp(ResolvedStreamableHttpDefinition),
}

/// HTTP runtime config with bearer tokens resolved from the environment.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedStreamableHttpDefinition {
    pub url: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer_env_var: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connect_timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_timeout_ms: Option<u64>,
}

/// Launcher/connector wrapper around a resolved MCP runtime server.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerLauncher {
    pub name: String,
    pub transport: McpServerLauncherTransport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<McpToolConfig>,
}

/// Transport-specific launcher/connector.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum McpServerLauncherTransport {
    Stdio(StdioLauncher),
    StreamableHttp(StreamableHttpConnector),
}

/// Prepared stdio launcher with merged env and startup timeout.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StdioLauncher {
    pub command: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(OsString, OsString)>,
    pub current_dir: Option<PathBuf>,
    pub timeout: Duration,
    pub mirror_stdio: bool,
}

/// Prepared HTTP connector with resolved headers and timeouts.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamableHttpConnector {
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub bearer_env_var: Option<String>,
    pub bearer_token: Option<String>,
    pub connect_timeout: Option<Duration>,
    pub request_timeout: Option<Duration>,
}

impl From<McpServerEntry> for McpRuntimeServer {
    fn from(entry: McpServerEntry) -> Self {
        let McpServerEntry { name, definition } = entry;
        McpRuntimeServer::from_definition(name, definition)
    }
}

impl McpRuntimeServer {
    /// Builds a runtime config from a stored server definition.
    pub fn from_definition(name: impl Into<String>, definition: McpServerDefinition) -> Self {
        let McpServerDefinition {
            transport,
            description,
            tags,
            tools,
        } = definition;

        Self {
            name: name.into(),
            transport: McpRuntimeTransport::from_transport(transport),
            description,
            tags,
            tools,
        }
    }

    /// Converts a runtime server into a launcher/connector, merging stdio defaults.
    pub fn into_launcher(self, defaults: &StdioServerConfig) -> McpServerLauncher {
        let McpRuntimeServer {
            name,
            transport,
            description,
            tags,
            tools,
        } = self;

        let transport = match transport {
            McpRuntimeTransport::Stdio(def) => {
                McpServerLauncherTransport::Stdio(StdioLauncher::from_runtime(def, defaults))
            }
            McpRuntimeTransport::StreamableHttp(def) => {
                McpServerLauncherTransport::StreamableHttp(def.into())
            }
        };

        McpServerLauncher {
            name,
            transport,
            description,
            tags,
            tools,
        }
    }

    /// Convenience clone-preserving conversion to a launcher/connector.
    pub fn to_launcher(&self, defaults: &StdioServerConfig) -> McpServerLauncher {
        self.clone().into_launcher(defaults)
    }
}

impl McpRuntimeTransport {
    fn from_transport(transport: McpTransport) -> Self {
        match transport {
            McpTransport::Stdio(definition) => McpRuntimeTransport::Stdio(definition),
            McpTransport::StreamableHttp(definition) => {
                McpRuntimeTransport::StreamableHttp(resolve_streamable_http(definition))
            }
        }
    }
}

fn resolve_streamable_http(
    definition: StreamableHttpDefinition,
) -> ResolvedStreamableHttpDefinition {
    let StreamableHttpDefinition {
        url,
        headers,
        bearer_env_var,
        connect_timeout_ms,
        request_timeout_ms,
    } = definition;

    let mut headers = headers;
    let mut bearer_token = None;
    if let Some(env_var) = bearer_env_var.as_deref() {
        if let Ok(token) = env::var(env_var) {
            if !token.is_empty() {
                let has_auth_header = headers
                    .keys()
                    .any(|key| key.eq_ignore_ascii_case("authorization"));
                if !has_auth_header {
                    headers.insert("Authorization".into(), format!("Bearer {token}"));
                }
                bearer_token = Some(token);
            }
        }
    }

    ResolvedStreamableHttpDefinition {
        url,
        headers,
        bearer_env_var,
        bearer_token,
        connect_timeout_ms,
        request_timeout_ms,
    }
}

impl StdioLauncher {
    fn from_runtime(definition: StdioServerDefinition, defaults: &StdioServerConfig) -> Self {
        let env = merge_stdio_env(
            defaults.code_home.as_deref(),
            &defaults.env,
            &definition.env,
        );

        Self {
            command: PathBuf::from(definition.command),
            args: definition.args,
            env,
            current_dir: defaults.current_dir.clone(),
            timeout: definition
                .timeout_ms
                .map(Duration::from_millis)
                .unwrap_or(defaults.startup_timeout),
            mirror_stdio: defaults.mirror_stdio,
        }
    }

    /// Builds a `tokio::process::Command` with merged env/dirs applied.
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.command);
        command
            .args(&self.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        if let Some(dir) = &self.current_dir {
            command.current_dir(dir);
        }

        for (key, value) in &self.env {
            command.env(key, value);
        }

        command
    }
}

impl From<ResolvedStreamableHttpDefinition> for StreamableHttpConnector {
    fn from(definition: ResolvedStreamableHttpDefinition) -> Self {
        let ResolvedStreamableHttpDefinition {
            url,
            headers,
            bearer_env_var,
            bearer_token,
            connect_timeout_ms,
            request_timeout_ms,
        } = definition;

        Self {
            url,
            headers,
            bearer_env_var,
            bearer_token,
            connect_timeout: connect_timeout_ms.map(Duration::from_millis),
            request_timeout: request_timeout_ms.map(Duration::from_millis),
        }
    }
}

pub(crate) fn merge_stdio_env(
    code_home: Option<&Path>,
    base_env: &[(OsString, OsString)],
    runtime_env: &BTreeMap<String, String>,
) -> Vec<(OsString, OsString)> {
    let mut merged: HashMap<OsString, OsString> = HashMap::new();

    if let Some(code_home) = code_home {
        merged.insert(
            OsString::from("CODEX_HOME"),
            code_home.as_os_str().to_os_string(),
        );
    }

    for (key, value) in base_env {
        merged.insert(key.clone(), value.clone());
    }

    for (key, value) in runtime_env {
        merged.insert(OsString::from(key), OsString::from(value));
    }

    merged.into_iter().collect()
}

/// Summarized runtime metadata for listing available MCP servers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpRuntimeSummary {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub tools: Option<McpToolConfig>,
    pub transport: McpRuntimeSummaryTransport,
}

/// Transport kind used by a runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum McpRuntimeSummaryTransport {
    Stdio,
    StreamableHttp,
}

impl From<&McpServerLauncher> for McpRuntimeSummary {
    fn from(launcher: &McpServerLauncher) -> Self {
        let transport = match launcher.transport {
            McpServerLauncherTransport::Stdio(_) => McpRuntimeSummaryTransport::Stdio,
            McpServerLauncherTransport::StreamableHttp(_) => {
                McpRuntimeSummaryTransport::StreamableHttp
            }
        };

        Self {
            name: launcher.name.clone(),
            description: launcher.description.clone(),
            tags: launcher.tags.clone(),
            tools: launcher.tools.clone(),
            transport,
        }
    }
}

/// Errors surfaced while starting or stopping MCP runtimes.
#[derive(Debug, Error)]
pub enum McpRuntimeError {
    #[error("runtime `{0}` not found")]
    NotFound(String),
    #[error("runtime `{name}` uses `{actual}` transport (expected {expected})")]
    UnsupportedTransport {
        name: String,
        expected: &'static str,
        actual: &'static str,
    },
    #[error("failed to spawn `{command:?}`: {source}")]
    Spawn {
        command: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("stdio pipes unavailable for `{name}`")]
    MissingPipes { name: String },
    #[error("failed to stop `{name}`: {source}")]
    Stop {
        name: String,
        #[source]
        source: io::Error,
    },
    #[error("timed out stopping `{name}` after {timeout:?}")]
    StopTimeout { name: String, timeout: Duration },
}

/// Lightweight runtime manager that owns resolved launchers/connectors.
///
/// The manager is non-destructive: launchers remain available after `prepare`
/// is called so callers can reuse connectors or restart stdio servers as
/// needed.
#[derive(Clone, Debug)]
pub struct McpRuntimeManager {
    launchers: BTreeMap<String, McpServerLauncher>,
}

impl McpRuntimeManager {
    /// Construct a runtime manager from resolved launchers.
    pub fn new(launchers: Vec<McpServerLauncher>) -> Self {
        let mut map = BTreeMap::new();
        for launcher in launchers {
            map.insert(launcher.name.clone(), launcher);
        }
        Self { launchers: map }
    }

    /// Returns the available runtimes with tool hints intact.
    pub fn available(&self) -> Vec<McpRuntimeSummary> {
        self.launchers
            .values()
            .map(McpRuntimeSummary::from)
            .collect()
    }

    /// Returns a cloned launcher/connector by name without mutating storage.
    pub fn launcher(&self, name: &str) -> Option<McpServerLauncher> {
        self.launchers.get(name).cloned()
    }

    /// Start a stdio runtime or hand back HTTP connector metadata.
    pub fn prepare(&self, name: &str) -> Result<McpRuntimeHandle, McpRuntimeError> {
        let Some(launcher) = self.launcher(name) else {
            return Err(McpRuntimeError::NotFound(name.to_string()));
        };

        let tools = launcher.tools.clone();
        match launcher.transport {
            McpServerLauncherTransport::Stdio(launch) => {
                let mut command = launch.command();
                let spawn_target = launch.command.clone();
                let mut child = command.spawn().map_err(|source| McpRuntimeError::Spawn {
                    command: spawn_target,
                    source,
                })?;

                let stdout = child.stdout.take();
                let stdin = child.stdin.take();
                if let (Some(stdout), Some(stdin)) = (stdout, stdin) {
                    let stderr = child.stderr.take();
                    Ok(McpRuntimeHandle::Stdio(ManagedStdioRuntime {
                        name: launcher.name,
                        tools,
                        child,
                        stdin,
                        stdout,
                        stderr,
                        timeout: launch.timeout,
                    }))
                } else {
                    let _ = child.start_kill();
                    Err(McpRuntimeError::MissingPipes {
                        name: launcher.name,
                    })
                }
            }
            McpServerLauncherTransport::StreamableHttp(connector) => {
                Ok(McpRuntimeHandle::StreamableHttp(ManagedHttpRuntime {
                    name: launcher.name,
                    connector,
                    tools,
                }))
            }
        }
    }
}

/// Read-only helpers around [`McpRuntimeManager`] backed by stored config.
#[derive(Clone, Debug)]
pub struct McpRuntimeApi {
    manager: McpRuntimeManager,
}

impl McpRuntimeApi {
    /// Build a runtime API from already prepared launchers/connectors.
    pub fn new(launchers: Vec<McpServerLauncher>) -> Self {
        Self {
            manager: McpRuntimeManager::new(launchers),
        }
    }

    /// Load runtime launchers from disk and merge Workstream A stdio defaults.
    ///
    /// This is non-destructive: stored definitions are read, resolved, and left untouched.
    pub fn from_config(
        config: &McpConfigManager,
        defaults: &StdioServerConfig,
    ) -> Result<Self, McpConfigError> {
        let launchers = config.runtime_launchers(defaults)?;
        Ok(Self::new(launchers))
    }

    /// List available runtimes along with tool hints.
    pub fn available(&self) -> Vec<McpRuntimeSummary> {
        self.manager.available()
    }

    /// Returns a launch-ready config for the given runtime.
    pub fn launcher(&self, name: &str) -> Result<McpServerLauncher, McpRuntimeError> {
        self.manager
            .launcher(name)
            .ok_or_else(|| McpRuntimeError::NotFound(name.to_string()))
    }

    /// Returns the stdio launcher for a runtime, erroring if it uses HTTP.
    pub fn stdio_launcher(&self, name: &str) -> Result<StdioLauncher, McpRuntimeError> {
        let launcher = self.launcher(name)?;
        match launcher.transport {
            McpServerLauncherTransport::Stdio(launch) => Ok(launch),
            McpServerLauncherTransport::StreamableHttp(_) => {
                Err(McpRuntimeError::UnsupportedTransport {
                    name: launcher.name,
                    expected: "stdio",
                    actual: "streamable_http",
                })
            }
        }
    }

    /// Returns the HTTP connector for a runtime, erroring if it uses stdio.
    pub fn http_connector(&self, name: &str) -> Result<StreamableHttpConnector, McpRuntimeError> {
        let launcher = self.launcher(name)?;
        match launcher.transport {
            McpServerLauncherTransport::StreamableHttp(connector) => Ok(connector),
            McpServerLauncherTransport::Stdio(_) => Err(McpRuntimeError::UnsupportedTransport {
                name: launcher.name,
                expected: "streamable_http",
                actual: "stdio",
            }),
        }
    }

    /// Prepare a runtime handle for connection or spawn.
    pub fn prepare(&self, name: &str) -> Result<McpRuntimeHandle, McpRuntimeError> {
        self.manager.prepare(name)
    }
}

/// Handle returned by [`McpRuntimeManager::prepare`] for either transport.
#[derive(Debug)]
pub enum McpRuntimeHandle {
    Stdio(ManagedStdioRuntime),
    StreamableHttp(ManagedHttpRuntime),
}

impl McpRuntimeHandle {
    /// Returns tool hints when present.
    pub fn tools(&self) -> Option<&McpToolConfig> {
        match self {
            McpRuntimeHandle::Stdio(handle) => handle.tools.as_ref(),
            McpRuntimeHandle::StreamableHttp(handle) => handle.tools.as_ref(),
        }
    }
}

/// Running stdio MCP server along with its pipes.
#[derive(Debug)]
pub struct ManagedStdioRuntime {
    name: String,
    tools: Option<McpToolConfig>,
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: Option<ChildStderr>,
    timeout: Duration,
}

impl ManagedStdioRuntime {
    /// Name of the runtime.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Tool allow/deny hints if provided.
    pub fn tools(&self) -> Option<&McpToolConfig> {
        self.tools.as_ref()
    }

    /// Writable pipe to the server.
    pub fn stdin_mut(&mut self) -> &mut ChildStdin {
        &mut self.stdin
    }

    /// Readable pipe from the server.
    pub fn stdout_mut(&mut self) -> &mut ChildStdout {
        &mut self.stdout
    }

    /// Optional stderr pipe from the server.
    pub fn stderr_mut(&mut self) -> Option<&mut ChildStderr> {
        self.stderr.as_mut()
    }

    /// Terminate the process and wait for exit (best-effort).
    pub async fn stop(&mut self) -> Result<(), McpRuntimeError> {
        if let Ok(Some(_)) = self.child.try_wait() {
            return Ok(());
        }

        let _ = self.child.start_kill();
        match time::timeout(self.timeout, self.child.wait()).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(source)) => Err(McpRuntimeError::Stop {
                name: self.name.clone(),
                source,
            }),
            Err(_) => Err(McpRuntimeError::StopTimeout {
                name: self.name.clone(),
                timeout: self.timeout,
            }),
        }
    }
}

impl Drop for ManagedStdioRuntime {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

/// HTTP runtime connector with tool hints preserved.
#[derive(Clone, Debug)]
pub struct ManagedHttpRuntime {
    pub name: String,
    pub connector: StreamableHttpConnector,
    pub tools: Option<McpToolConfig>,
}
