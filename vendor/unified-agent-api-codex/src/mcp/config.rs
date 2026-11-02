use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use toml::{value::Table as TomlTable, Value as TomlValue};

use super::{
    AppRuntime, AppRuntimeLauncher, McpRuntimeServer, McpServerLauncher, StdioServerConfig,
};

/// Default config filename placed under CODEX_HOME.
pub const DEFAULT_CONFIG_FILE: &str = "config.toml";
const MCP_SERVERS_KEY: &str = "mcp_servers";
const APP_RUNTIMES_KEY: &str = "app_runtimes";

/// MCP server definition coupled with its name.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerEntry {
    pub name: String,
    pub definition: McpServerDefinition,
}

/// App runtime definition coupled with its name.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppRuntimeEntry {
    pub name: String,
    pub definition: AppRuntimeDefinition,
}

/// JSON-serializable MCP server configuration stored under `[mcp_servers]`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerDefinition {
    pub transport: McpTransport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<McpToolConfig>,
}

/// Supported transport definitions for MCP servers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum McpTransport {
    Stdio(StdioServerDefinition),
    StreamableHttp(StreamableHttpDefinition),
}

/// Stdio transport configuration for an MCP server.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StdioServerDefinition {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

/// HTTP transport configuration that supports streaming responses.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamableHttpDefinition {
    pub url: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer_env_var: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connect_timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_timeout_ms: Option<u64>,
}

/// Tool allow/deny lists for a given MCP server.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpToolConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled: Vec<String>,
}

/// Stored definition for launching an app-server runtime.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppRuntimeDefinition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_home: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_stdio: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub metadata: Value,
}

/// Input for adding or updating an app runtime entry.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AddAppRuntimeRequest {
    pub name: String,
    pub definition: AppRuntimeDefinition,
    #[serde(default)]
    pub overwrite: bool,
}

/// Input for adding or updating an MCP server entry.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddServerRequest {
    pub name: String,
    pub definition: McpServerDefinition,
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub bearer_token: Option<String>,
}

/// Result of logging into a server (auth token set in env var).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpLoginResult {
    pub server: String,
    pub env_var: Option<String>,
}

/// Result of clearing a stored auth token.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpLogoutResult {
    pub server: String,
    pub env_var: Option<String>,
    pub cleared: bool,
}

/// Errors surfaced while managing MCP config entries.
#[derive(Debug, Error)]
pub enum McpConfigError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("config root at {path} must be a table")]
    InvalidRoot { path: PathBuf },
    #[error("`mcp_servers` must be a table in {path}")]
    InvalidServers { path: PathBuf },
    #[error("failed to decode mcp_servers: {source}")]
    DecodeServers {
        #[source]
        source: toml::de::Error,
    },
    #[error("`app_runtimes` must be a table in {path}")]
    InvalidAppRuntimes { path: PathBuf },
    #[error("failed to decode app_runtimes: {source}")]
    DecodeAppRuntimes {
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to serialize config: {source}")]
    Serialize {
        #[source]
        source: toml::ser::Error,
    },
    #[error("server `{0}` already exists")]
    ServerAlreadyExists(String),
    #[error("server `{0}` not found")]
    ServerNotFound(String),
    #[error("server name may not be empty")]
    InvalidServerName,
    #[error("app runtime `{0}` already exists")]
    AppRuntimeAlreadyExists(String),
    #[error("app runtime `{0}` not found")]
    AppRuntimeNotFound(String),
    #[error("app runtime name may not be empty")]
    InvalidAppRuntimeName,
    #[error("invalid env var name `{name}`")]
    InvalidEnvVarName { name: String },
    #[error("server `{server}` missing bearer_env_var for auth token")]
    MissingBearerEnvVar { server: String },
    #[error("server `{server}` transport does not support login/logout")]
    UnsupportedAuthTransport { server: String },
}

/// Helper to load and mutate MCP + app runtime config stored under `[mcp_servers]` and
/// `[app_runtimes]`.
///
/// Runtime, API, and pool helpers consume this manager in a read-only fashion so stored
/// definitions, auth hints, and metadata are left untouched while preparing launch configs.
pub struct McpConfigManager {
    config_path: PathBuf,
}

impl McpConfigManager {
    /// Create a manager that reads/writes the given config path.
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    /// Convenience constructor for a CODEX_HOME directory.
    pub fn from_code_home(code_home: impl AsRef<Path>) -> Self {
        Self::new(code_home.as_ref().join(DEFAULT_CONFIG_FILE))
    }

    /// Returns the underlying config path.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Returns all configured MCP servers.
    pub fn list_servers(&self) -> Result<Vec<McpServerEntry>, McpConfigError> {
        let servers = self.read_servers()?;
        Ok(servers
            .into_iter()
            .map(|(name, definition)| McpServerEntry { name, definition })
            .collect())
    }

    /// Returns a single MCP server by name.
    pub fn get_server(&self, name: &str) -> Result<McpServerEntry, McpConfigError> {
        let servers = self.read_servers()?;
        let Some(definition) = servers.get(name).cloned() else {
            return Err(McpConfigError::ServerNotFound(name.to_string()));
        };

        Ok(McpServerEntry {
            name: name.to_string(),
            definition,
        })
    }

    /// Returns all configured app runtimes.
    pub fn list_app_runtimes(&self) -> Result<Vec<AppRuntimeEntry>, McpConfigError> {
        let runtimes = self.read_app_runtimes()?;
        Ok(runtimes
            .into_iter()
            .map(|(name, definition)| AppRuntimeEntry { name, definition })
            .collect())
    }

    /// Returns a single app runtime by name.
    pub fn get_app_runtime(&self, name: &str) -> Result<AppRuntimeEntry, McpConfigError> {
        let runtimes = self.read_app_runtimes()?;
        let Some(definition) = runtimes.get(name).cloned() else {
            return Err(McpConfigError::AppRuntimeNotFound(name.to_string()));
        };

        Ok(AppRuntimeEntry {
            name: name.to_string(),
            definition,
        })
    }

    /// Returns runtime-ready app configs with metadata preserved.
    pub fn app_runtimes(&self) -> Result<Vec<AppRuntime>, McpConfigError> {
        Ok(self
            .list_app_runtimes()?
            .into_iter()
            .map(AppRuntime::from)
            .collect())
    }

    /// Returns a runtime-ready app config for a single entry.
    pub fn app_runtime(&self, name: &str) -> Result<AppRuntime, McpConfigError> {
        self.get_app_runtime(name).map(AppRuntime::from)
    }

    /// Returns prepared launchers for all app runtimes.
    pub fn app_runtime_launchers(
        &self,
        defaults: &StdioServerConfig,
    ) -> Result<Vec<AppRuntimeLauncher>, McpConfigError> {
        self.app_runtimes().map(|runtimes| {
            runtimes
                .into_iter()
                .map(|runtime| runtime.into_launcher(defaults))
                .collect()
        })
    }

    /// Returns a prepared launcher for an app runtime by name.
    pub fn app_runtime_launcher(
        &self,
        name: &str,
        defaults: &StdioServerConfig,
    ) -> Result<AppRuntimeLauncher, McpConfigError> {
        self.app_runtime(name)
            .map(|runtime| runtime.into_launcher(defaults))
    }

    /// Returns runtime-ready configs for all servers, resolving bearer tokens from the environment.
    pub fn runtime_servers(&self) -> Result<Vec<McpRuntimeServer>, McpConfigError> {
        Ok(self
            .list_servers()?
            .into_iter()
            .map(McpRuntimeServer::from)
            .collect())
    }

    /// Returns a runtime-ready config for a single server by name.
    pub fn runtime_server(&self, name: &str) -> Result<McpRuntimeServer, McpConfigError> {
        self.get_server(name).map(McpRuntimeServer::from)
    }

    /// Returns prepared launchers/connectors for all runtime servers.
    pub fn runtime_launchers(
        &self,
        defaults: &StdioServerConfig,
    ) -> Result<Vec<McpServerLauncher>, McpConfigError> {
        self.runtime_servers().map(|servers| {
            servers
                .into_iter()
                .map(|server| server.into_launcher(defaults))
                .collect()
        })
    }

    /// Returns a prepared launcher/connector for a single runtime server by name.
    pub fn runtime_launcher(
        &self,
        name: &str,
        defaults: &StdioServerConfig,
    ) -> Result<McpServerLauncher, McpConfigError> {
        self.runtime_server(name)
            .map(|server| server.into_launcher(defaults))
    }

    /// Adds or updates an app runtime definition.
    pub fn add_app_runtime(
        &self,
        request: AddAppRuntimeRequest,
    ) -> Result<AppRuntimeEntry, McpConfigError> {
        let AddAppRuntimeRequest {
            name,
            definition,
            overwrite,
        } = request;

        if name.trim().is_empty() {
            return Err(McpConfigError::InvalidAppRuntimeName);
        }

        let (table, mut runtimes) = self.read_table_and_app_runtimes()?;
        if !overwrite && runtimes.contains_key(&name) {
            return Err(McpConfigError::AppRuntimeAlreadyExists(name));
        }

        runtimes.insert(name.clone(), definition.clone());
        self.persist_app_runtimes(table, &runtimes)?;

        Ok(AppRuntimeEntry { name, definition })
    }

    /// Adds or updates a server definition and injects any provided env vars.
    pub fn add_server(
        &self,
        mut request: AddServerRequest,
    ) -> Result<McpServerEntry, McpConfigError> {
        if request.name.trim().is_empty() {
            return Err(McpConfigError::InvalidServerName);
        }

        let mut env_injections = request.env.clone();
        if let Some(token) = request.bearer_token.take() {
            let var = Self::bearer_env_var(&request.name, &request.definition)?;
            env_injections.entry(var).or_insert(token);
        }

        if let McpTransport::Stdio(transport) = &mut request.definition.transport {
            for (key, value) in &env_injections {
                transport.env.entry(key.clone()).or_insert(value.clone());
            }
        }

        self.set_env_vars(&env_injections)?;

        let (table, mut servers) = self.read_table_and_servers()?;
        if !request.overwrite && servers.contains_key(&request.name) {
            return Err(McpConfigError::ServerAlreadyExists(request.name));
        }

        servers.insert(request.name.clone(), request.definition.clone());
        self.persist_servers(table, &servers)?;

        Ok(McpServerEntry {
            name: request.name,
            definition: request.definition,
        })
    }

    /// Removes a server definition. Returns the removed entry if it existed.
    pub fn remove_server(&self, name: &str) -> Result<Option<McpServerEntry>, McpConfigError> {
        let (table, mut servers) = self.read_table_and_servers()?;
        let removed = servers.remove(name).map(|definition| McpServerEntry {
            name: name.to_string(),
            definition,
        });

        if removed.is_some() {
            self.persist_servers(table, &servers)?;
        }

        Ok(removed)
    }

    /// Writes the provided token into the server's bearer env var.
    pub fn login(
        &self,
        name: &str,
        token: impl AsRef<str>,
    ) -> Result<McpLoginResult, McpConfigError> {
        let servers = self.read_servers()?;
        let definition = servers
            .get(name)
            .ok_or_else(|| McpConfigError::ServerNotFound(name.to_string()))?;
        let env_var = Self::bearer_env_var(name, definition)?;
        self.validate_env_key(&env_var)?;
        env::set_var(&env_var, token.as_ref());
        Ok(McpLoginResult {
            server: name.to_string(),
            env_var: Some(env_var),
        })
    }

    /// Clears the bearer env var used for the given server.
    pub fn logout(&self, name: &str) -> Result<McpLogoutResult, McpConfigError> {
        let servers = self.read_servers()?;
        let definition = servers
            .get(name)
            .ok_or_else(|| McpConfigError::ServerNotFound(name.to_string()))?;
        let env_var = Self::bearer_env_var(name, definition)?;
        let cleared = env::var(&env_var).is_ok();
        env::remove_var(&env_var);
        Ok(McpLogoutResult {
            server: name.to_string(),
            env_var: Some(env_var),
            cleared,
        })
    }

    fn bearer_env_var(
        name: &str,
        definition: &McpServerDefinition,
    ) -> Result<String, McpConfigError> {
        match &definition.transport {
            McpTransport::StreamableHttp(http) => {
                http.bearer_env_var
                    .clone()
                    .ok_or_else(|| McpConfigError::MissingBearerEnvVar {
                        server: name.to_string(),
                    })
            }
            McpTransport::Stdio(_) => Err(McpConfigError::UnsupportedAuthTransport {
                server: name.to_string(),
            }),
        }
    }

    fn read_servers(&self) -> Result<BTreeMap<String, McpServerDefinition>, McpConfigError> {
        let table = self.load_table()?;
        self.parse_servers(table.get(MCP_SERVERS_KEY))
    }

    fn read_table_and_servers(
        &self,
    ) -> Result<(TomlTable, BTreeMap<String, McpServerDefinition>), McpConfigError> {
        let table = self.load_table()?;
        let servers = self.parse_servers(table.get(MCP_SERVERS_KEY))?;
        Ok((table, servers))
    }

    fn parse_servers(
        &self,
        value: Option<&TomlValue>,
    ) -> Result<BTreeMap<String, McpServerDefinition>, McpConfigError> {
        let Some(value) = value else {
            return Ok(BTreeMap::new());
        };

        let table = value
            .as_table()
            .ok_or_else(|| McpConfigError::InvalidServers {
                path: self.config_path.clone(),
            })?;
        let cloned = TomlValue::Table(table.clone());
        cloned
            .try_into()
            .map_err(|source| McpConfigError::DecodeServers { source })
    }

    fn persist_servers(
        &self,
        mut table: TomlTable,
        servers: &BTreeMap<String, McpServerDefinition>,
    ) -> Result<(), McpConfigError> {
        if servers.is_empty() {
            table.remove(MCP_SERVERS_KEY);
        } else {
            let value = TomlValue::try_from(servers.clone())
                .map_err(|source| McpConfigError::Serialize { source })?;
            table.insert(MCP_SERVERS_KEY.to_string(), value);
        }

        self.write_table(table)
    }

    fn read_app_runtimes(&self) -> Result<BTreeMap<String, AppRuntimeDefinition>, McpConfigError> {
        let table = self.load_table()?;
        self.parse_app_runtimes(table.get(APP_RUNTIMES_KEY))
    }

    fn read_table_and_app_runtimes(
        &self,
    ) -> Result<(TomlTable, BTreeMap<String, AppRuntimeDefinition>), McpConfigError> {
        let table = self.load_table()?;
        let runtimes = self.parse_app_runtimes(table.get(APP_RUNTIMES_KEY))?;
        Ok((table, runtimes))
    }

    fn parse_app_runtimes(
        &self,
        value: Option<&TomlValue>,
    ) -> Result<BTreeMap<String, AppRuntimeDefinition>, McpConfigError> {
        let Some(value) = value else {
            return Ok(BTreeMap::new());
        };

        let table = value
            .as_table()
            .ok_or_else(|| McpConfigError::InvalidAppRuntimes {
                path: self.config_path.clone(),
            })?;
        let cloned = TomlValue::Table(table.clone());
        cloned
            .try_into()
            .map_err(|source| McpConfigError::DecodeAppRuntimes { source })
    }

    fn persist_app_runtimes(
        &self,
        mut table: TomlTable,
        runtimes: &BTreeMap<String, AppRuntimeDefinition>,
    ) -> Result<(), McpConfigError> {
        if runtimes.is_empty() {
            table.remove(APP_RUNTIMES_KEY);
        } else {
            let value = TomlValue::try_from(runtimes.clone())
                .map_err(|source| McpConfigError::Serialize { source })?;
            table.insert(APP_RUNTIMES_KEY.to_string(), value);
        }

        self.write_table(table)
    }

    fn load_table(&self) -> Result<TomlTable, McpConfigError> {
        if !self.config_path.exists() {
            return Ok(TomlTable::new());
        }

        let contents =
            fs::read_to_string(&self.config_path).map_err(|source| McpConfigError::Read {
                path: self.config_path.clone(),
                source,
            })?;

        if contents.trim().is_empty() {
            return Ok(TomlTable::new());
        }

        let value: TomlValue = contents.parse().map_err(|source| McpConfigError::Parse {
            path: self.config_path.clone(),
            source,
        })?;

        value
            .as_table()
            .cloned()
            .ok_or_else(|| McpConfigError::InvalidRoot {
                path: self.config_path.clone(),
            })
    }

    fn write_table(&self, table: TomlTable) -> Result<(), McpConfigError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|source| McpConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&TomlValue::Table(table))
            .map_err(|source| McpConfigError::Serialize { source })?;

        fs::write(&self.config_path, serialized).map_err(|source| McpConfigError::Write {
            path: self.config_path.clone(),
            source,
        })
    }

    fn set_env_vars(&self, vars: &BTreeMap<String, String>) -> Result<(), McpConfigError> {
        for (key, value) in vars {
            self.validate_env_key(key)?;
            env::set_var(key, value);
        }
        Ok(())
    }

    fn validate_env_key(&self, key: &str) -> Result<(), McpConfigError> {
        let invalid = key.is_empty() || key.contains('=') || key.contains('\0');
        if invalid {
            return Err(McpConfigError::InvalidEnvVarName {
                name: key.to_string(),
            });
        }
        Ok(())
    }
}
