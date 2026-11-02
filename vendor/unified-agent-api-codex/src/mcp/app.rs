use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;

use super::{
    runtime::merge_stdio_env, AppRuntimeDefinition, AppRuntimeEntry, ClientInfo, CodexAppServer,
    McpConfigError, McpConfigManager, McpError, StdioServerConfig,
};

/// Stored app runtime converted into launch-ready config with metadata intact.
#[derive(Clone, Debug, PartialEq)]
pub struct AppRuntime {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Value,
    pub env: BTreeMap<String, String>,
    pub code_home: Option<PathBuf>,
    pub current_dir: Option<PathBuf>,
    pub mirror_stdio: Option<bool>,
    pub startup_timeout_ms: Option<u64>,
    pub binary: Option<PathBuf>,
}

impl From<AppRuntimeEntry> for AppRuntime {
    fn from(entry: AppRuntimeEntry) -> Self {
        let AppRuntimeEntry { name, definition } = entry;
        let AppRuntimeDefinition {
            description,
            tags,
            env,
            code_home,
            current_dir,
            mirror_stdio,
            startup_timeout_ms,
            binary,
            metadata,
        } = definition;

        Self {
            name,
            description,
            tags,
            metadata,
            env,
            code_home,
            current_dir,
            mirror_stdio,
            startup_timeout_ms,
            binary,
        }
    }
}

impl AppRuntime {
    /// Converts an app runtime into a launch-ready config using provided defaults.
    pub fn into_launcher(self, defaults: &StdioServerConfig) -> AppRuntimeLauncher {
        let code_home = self
            .code_home
            .clone()
            .or_else(|| defaults.code_home.clone());
        let env = merge_stdio_env(code_home.as_deref(), &defaults.env, &self.env);

        let config = StdioServerConfig {
            binary: self
                .binary
                .clone()
                .unwrap_or_else(|| defaults.binary.clone()),
            code_home,
            current_dir: self
                .current_dir
                .clone()
                .or_else(|| defaults.current_dir.clone()),
            env,
            app_server_analytics_default_enabled: defaults.app_server_analytics_default_enabled,
            mirror_stdio: self.mirror_stdio.unwrap_or(defaults.mirror_stdio),
            startup_timeout: self
                .startup_timeout_ms
                .map(Duration::from_millis)
                .unwrap_or(defaults.startup_timeout),
        };

        AppRuntimeLauncher {
            name: self.name,
            description: self.description,
            tags: self.tags,
            metadata: self.metadata,
            config,
        }
    }

    /// Convenience clone-preserving conversion to a launcher.
    pub fn to_launcher(&self, defaults: &StdioServerConfig) -> AppRuntimeLauncher {
        self.clone().into_launcher(defaults)
    }
}

/// Launch-ready stdio config bundled with app metadata.
#[derive(Clone, Debug)]
pub struct AppRuntimeLauncher {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Value,
    pub config: StdioServerConfig,
}

/// Summarized app runtime metadata for listing.
#[derive(Clone, Debug, PartialEq)]
pub struct AppRuntimeSummary {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Value,
}

impl From<&AppRuntimeLauncher> for AppRuntimeSummary {
    fn from(launcher: &AppRuntimeLauncher) -> Self {
        Self {
            name: launcher.name.clone(),
            description: launcher.description.clone(),
            tags: launcher.tags.clone(),
            metadata: launcher.metadata.clone(),
        }
    }
}

/// Errors surfaced while reading app runtimes.
#[derive(Debug, Error)]
pub enum AppRuntimeError {
    #[error("runtime `{0}` not found")]
    NotFound(String),
    #[error("failed to start runtime `{name}`: {source}")]
    Start {
        name: String,
        #[source]
        source: McpError,
    },
    #[error("failed to stop runtime `{name}`: {source}")]
    Stop {
        name: String,
        #[source]
        source: McpError,
    },
}

/// Prepared app runtime with merged stdio config and metadata.
#[derive(Clone, Debug)]
pub struct AppRuntimeHandle {
    pub name: String,
    pub metadata: Value,
    pub config: StdioServerConfig,
}

/// Running app-server instance with metadata preserved.
pub struct ManagedAppRuntime {
    pub name: String,
    pub metadata: Value,
    pub config: StdioServerConfig,
    pub server: CodexAppServer,
}

impl fmt::Debug for ManagedAppRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManagedAppRuntime")
            .field("name", &self.name)
            .field("metadata", &self.metadata)
            .field("config", &self.config)
            .finish()
    }
}

impl ManagedAppRuntime {
    /// Gracefully shut down the app-server.
    pub async fn stop(&self) -> Result<(), McpError> {
        self.server.shutdown().await
    }
}

impl AppRuntimeHandle {
    /// Launch the app-server using the prepared stdio config.
    pub async fn start(self, client: ClientInfo) -> Result<ManagedAppRuntime, McpError> {
        let AppRuntimeHandle {
            name,
            metadata,
            config,
        } = self;

        let server = CodexAppServer::start(config.clone(), client).await?;

        Ok(ManagedAppRuntime {
            name,
            metadata,
            config,
            server,
        })
    }
}

/// Non-destructive manager for app runtimes backed by launch-ready configs.
#[derive(Clone, Debug)]
pub struct AppRuntimeManager {
    launchers: BTreeMap<String, AppRuntimeLauncher>,
}

impl AppRuntimeManager {
    /// Construct a runtime manager from prepared launchers.
    pub fn new(launchers: Vec<AppRuntimeLauncher>) -> Self {
        let mut map = BTreeMap::new();
        for launcher in launchers {
            map.insert(launcher.name.clone(), launcher);
        }
        Self { launchers: map }
    }

    /// Returns the available app runtimes with metadata intact.
    pub fn available(&self) -> Vec<AppRuntimeSummary> {
        self.launchers
            .values()
            .map(AppRuntimeSummary::from)
            .collect()
    }

    /// Returns a cloned launcher by name without mutating storage.
    pub fn launcher(&self, name: &str) -> Option<AppRuntimeLauncher> {
        self.launchers.get(name).cloned()
    }

    /// Returns a prepared config + metadata for launching the app server.
    pub fn prepare(&self, name: &str) -> Result<AppRuntimeHandle, AppRuntimeError> {
        let Some(launcher) = self.launcher(name) else {
            return Err(AppRuntimeError::NotFound(name.to_string()));
        };

        Ok(AppRuntimeHandle {
            name: launcher.name,
            metadata: launcher.metadata,
            config: launcher.config,
        })
    }

    /// Start an app-server runtime using the prepared config and metadata.
    pub async fn start(
        &self,
        name: &str,
        client: ClientInfo,
    ) -> Result<ManagedAppRuntime, AppRuntimeError> {
        let handle = self.prepare(name)?;
        handle
            .start(client)
            .await
            .map_err(|source| AppRuntimeError::Start {
                name: name.to_string(),
                source,
            })
    }
}

/// Read-only helpers around [`AppRuntimeManager`] backed by stored config.
#[derive(Clone, Debug)]
pub struct AppRuntimeApi {
    manager: AppRuntimeManager,
}

impl AppRuntimeApi {
    /// Build an API from already prepared launchers.
    pub fn new(launchers: Vec<AppRuntimeLauncher>) -> Self {
        Self {
            manager: AppRuntimeManager::new(launchers),
        }
    }

    /// Load app runtimes from disk and merge Workstream A stdio defaults.
    pub fn from_config(
        config: &McpConfigManager,
        defaults: &StdioServerConfig,
    ) -> Result<Self, McpConfigError> {
        let launchers = config.app_runtime_launchers(defaults)?;
        Ok(Self::new(launchers))
    }

    /// List available runtimes and metadata.
    pub fn available(&self) -> Vec<AppRuntimeSummary> {
        self.manager.available()
    }

    /// Returns the launch-ready config bundle for the given runtime.
    pub fn launcher(&self, name: &str) -> Result<AppRuntimeLauncher, AppRuntimeError> {
        self.manager
            .launcher(name)
            .ok_or_else(|| AppRuntimeError::NotFound(name.to_string()))
    }

    /// Prepare a stdio config + metadata for a runtime.
    pub fn prepare(&self, name: &str) -> Result<AppRuntimeHandle, AppRuntimeError> {
        self.manager.prepare(name)
    }

    /// Start an app runtime and return a managed handle.
    pub async fn start(
        &self,
        name: &str,
        client: ClientInfo,
    ) -> Result<ManagedAppRuntime, AppRuntimeError> {
        self.manager.start(name, client).await
    }

    /// Convenience accessor for the merged stdio config.
    pub fn stdio_config(&self, name: &str) -> Result<StdioServerConfig, AppRuntimeError> {
        self.prepare(name).map(|handle| handle.config)
    }

    /// Build a pooled lifecycle manager that can reuse running runtimes.
    pub fn pool(&self) -> AppRuntimePool {
        AppRuntimePool::new(self.manager.clone())
    }

    /// Consume the API and return a pooled lifecycle manager.
    pub fn into_pool(self) -> AppRuntimePool {
        AppRuntimePool::new(self.manager)
    }

    /// Build a pooled lifecycle API that can reuse running runtimes.
    pub fn pool_api(&self) -> AppRuntimePoolApi {
        AppRuntimePoolApi::from_manager(self.manager.clone())
    }

    /// Consume the API and return a pooled lifecycle API.
    pub fn into_pool_api(self) -> AppRuntimePoolApi {
        AppRuntimePoolApi::from_manager(self.manager)
    }
}

/// Async pool that starts, reuses, and stops app runtimes without mutating config.
///
/// Runtime metadata and resume hints remain intact when runtimes are reused or restarted.
#[derive(Clone, Debug)]
pub struct AppRuntimePool {
    manager: AppRuntimeManager,
    running: Arc<Mutex<HashMap<String, Arc<ManagedAppRuntime>>>>,
}

impl AppRuntimePool {
    /// Create a new pool backed by launch-ready runtime configs.
    pub fn new(manager: AppRuntimeManager) -> Self {
        Self {
            manager,
            running: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// List available runtimes and metadata without touching stored definitions.
    pub fn available(&self) -> Vec<AppRuntimeSummary> {
        self.manager.available()
    }

    /// List currently running runtimes with metadata/resume hints preserved.
    pub async fn running(&self) -> Vec<AppRuntimeSummary> {
        let mut names: Vec<String> = {
            let guard = self.running.lock().await;
            guard.keys().cloned().collect()
        };

        names.sort();

        names
            .into_iter()
            .filter_map(|name| self.manager.launcher(&name))
            .map(|launcher| AppRuntimeSummary::from(&launcher))
            .collect()
    }

    /// Returns a launch-ready config bundle for the given runtime.
    pub fn launcher(&self, name: &str) -> Option<AppRuntimeLauncher> {
        self.manager.launcher(name)
    }

    /// Prepare a stdio config + metadata for a runtime without starting it.
    pub fn prepare(&self, name: &str) -> Result<AppRuntimeHandle, AppRuntimeError> {
        self.manager.prepare(name)
    }

    /// Start (or reuse) an app runtime. Subsequent calls reuse an existing instance.
    pub async fn start(
        &self,
        name: &str,
        client: ClientInfo,
    ) -> Result<Arc<ManagedAppRuntime>, AppRuntimeError> {
        {
            let guard = self.running.lock().await;
            if let Some(existing) = guard.get(name) {
                return Ok(existing.clone());
            }
        }

        let runtime = Arc::new(self.manager.start(name, client).await?);

        let mut guard = self.running.lock().await;
        if let Some(existing) = guard.get(name) {
            runtime
                .stop()
                .await
                .map_err(|source| AppRuntimeError::Stop {
                    name: name.to_string(),
                    source,
                })?;
            return Ok(existing.clone());
        }

        guard.insert(name.to_string(), runtime.clone());
        Ok(runtime)
    }

    /// Stop a running runtime and remove it from the pool.
    pub async fn stop(&self, name: &str) -> Result<(), AppRuntimeError> {
        let runtime = {
            let mut guard = self.running.lock().await;
            guard.remove(name)
        };

        match runtime {
            Some(runtime) => runtime
                .stop()
                .await
                .map_err(|source| AppRuntimeError::Stop {
                    name: name.to_string(),
                    source,
                }),
            None => Err(AppRuntimeError::NotFound(name.to_string())),
        }
    }

    /// Stop all running runtimes (best-effort) and clear the pool.
    pub async fn stop_all(&self) -> Result<(), AppRuntimeError> {
        let runtimes: Vec<(String, Arc<ManagedAppRuntime>)> = {
            let mut guard = self.running.lock().await;
            guard.drain().collect()
        };

        let mut first_error: Option<AppRuntimeError> = None;

        for (name, runtime) in runtimes {
            if let Err(source) = runtime.stop().await {
                if first_error.is_none() {
                    first_error = Some(AppRuntimeError::Stop { name, source });
                }
            }
        }

        if let Some(err) = first_error {
            return Err(err);
        }

        Ok(())
    }
}

/// Public API around [`AppRuntimePool`] that exposes pooled lifecycle helpers.
///
/// Operations reuse running runtimes when available while preserving stored definitions
/// and app metadata/resume hints.
#[derive(Clone, Debug)]
pub struct AppRuntimePoolApi {
    pool: AppRuntimePool,
}

impl AppRuntimePoolApi {
    /// Build a pooled API from prepared launchers.
    pub fn new(launchers: Vec<AppRuntimeLauncher>) -> Self {
        Self::from_manager(AppRuntimeManager::new(launchers))
    }

    /// Load app runtimes from disk and merge Workstream A stdio defaults.
    pub fn from_config(
        config: &McpConfigManager,
        defaults: &StdioServerConfig,
    ) -> Result<Self, McpConfigError> {
        let launchers = config.app_runtime_launchers(defaults)?;
        Ok(Self::new(launchers))
    }

    /// Build a pooled API from a runtime manager.
    pub fn from_manager(manager: AppRuntimeManager) -> Self {
        Self::from_pool(AppRuntimePool::new(manager))
    }

    /// Wrap an existing pool in the API surface.
    pub fn from_pool(pool: AppRuntimePool) -> Self {
        Self { pool }
    }

    /// List available runtimes and metadata.
    pub fn available(&self) -> Vec<AppRuntimeSummary> {
        self.pool.available()
    }

    /// List running runtimes with metadata intact.
    pub async fn running(&self) -> Vec<AppRuntimeSummary> {
        self.pool.running().await
    }

    /// Returns the launch-ready config bundle for the given runtime.
    pub fn launcher(&self, name: &str) -> Result<AppRuntimeLauncher, AppRuntimeError> {
        self.pool
            .launcher(name)
            .ok_or_else(|| AppRuntimeError::NotFound(name.to_string()))
    }

    /// Prepare a stdio config + metadata for a runtime.
    pub fn prepare(&self, name: &str) -> Result<AppRuntimeHandle, AppRuntimeError> {
        self.pool.prepare(name)
    }

    /// Start (or reuse) an app runtime.
    pub async fn start(
        &self,
        name: &str,
        client: ClientInfo,
    ) -> Result<Arc<ManagedAppRuntime>, AppRuntimeError> {
        self.pool.start(name, client).await
    }

    /// Stop a running runtime and remove it from the pool.
    pub async fn stop(&self, name: &str) -> Result<(), AppRuntimeError> {
        self.pool.stop(name).await
    }

    /// Stop all running runtimes (best-effort) and clear the pool.
    pub async fn stop_all(&self) -> Result<(), AppRuntimeError> {
        self.pool.stop_all().await
    }

    /// Convenience accessor for the merged stdio config.
    pub fn stdio_config(&self, name: &str) -> Result<StdioServerConfig, AppRuntimeError> {
        self.prepare(name).map(|handle| handle.config)
    }
}
