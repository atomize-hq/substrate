use std::{path::PathBuf, time::Duration};

use super::command::ClaudeCommandRequest;

#[derive(Debug, Clone)]
pub struct PluginRequest {
    pub(crate) timeout: Option<Duration>,
}

impl PluginRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginMarketplaceRequest {
    pub(crate) timeout: Option<Duration>,
}

impl PluginMarketplaceRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin", "marketplace"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginMarketplaceRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginDisableRequest {
    pub(crate) all: bool,
    pub(crate) scope: Option<String>,
    pub(crate) timeout: Option<Duration>,
}

impl PluginDisableRequest {
    pub fn new() -> Self {
        Self {
            all: false,
            scope: None,
            timeout: None,
        }
    }

    pub fn all(mut self, all: bool) -> Self {
        self.all = all;
        self
    }

    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut args = Vec::<String>::new();
        if self.all {
            args.push("--all".to_string());
        }
        if let Some(scope) = self.scope {
            args.push("--scope".to_string());
            args.push(scope);
        }

        let mut cmd = ClaudeCommandRequest::new(["plugin", "disable"]).args(args);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginDisableRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginEnableRequest {
    pub(crate) plugin: String,
    pub(crate) scope: Option<String>,
    pub(crate) timeout: Option<Duration>,
}

impl PluginEnableRequest {
    pub fn new(plugin: impl Into<String>) -> Self {
        Self {
            plugin: plugin.into(),
            scope: None,
            timeout: None,
        }
    }

    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut args = Vec::<String>::new();
        if let Some(scope) = self.scope {
            args.push("--scope".to_string());
            args.push(scope);
        }
        args.push(self.plugin);

        let mut cmd = ClaudeCommandRequest::new(["plugin", "enable"]).args(args);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

#[derive(Debug, Clone)]
pub struct PluginInstallRequest {
    pub(crate) scope: Option<String>,
    pub(crate) timeout: Option<Duration>,
}

impl PluginInstallRequest {
    pub fn new() -> Self {
        Self {
            scope: None,
            timeout: None,
        }
    }

    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut args = Vec::<String>::new();
        if let Some(scope) = self.scope {
            args.push("--scope".to_string());
            args.push(scope);
        }

        let mut cmd = ClaudeCommandRequest::new(["plugin", "install"]).args(args);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginInstallRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginUninstallRequest {
    pub(crate) scope: Option<String>,
    pub(crate) timeout: Option<Duration>,
}

impl PluginUninstallRequest {
    pub fn new() -> Self {
        Self {
            scope: None,
            timeout: None,
        }
    }

    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut args = Vec::<String>::new();
        if let Some(scope) = self.scope {
            args.push("--scope".to_string());
            args.push(scope);
        }

        let mut cmd = ClaudeCommandRequest::new(["plugin", "uninstall"]).args(args);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginUninstallRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginUpdateRequest {
    pub(crate) plugin: String,
    pub(crate) scope: Option<String>,
    pub(crate) timeout: Option<Duration>,
}

impl PluginUpdateRequest {
    pub fn new(plugin: impl Into<String>) -> Self {
        Self {
            plugin: plugin.into(),
            scope: None,
            timeout: None,
        }
    }

    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut args = Vec::<String>::new();
        if let Some(scope) = self.scope {
            args.push("--scope".to_string());
            args.push(scope);
        }
        args.push(self.plugin);

        let mut cmd = ClaudeCommandRequest::new(["plugin", "update"]).args(args);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

#[derive(Debug, Clone)]
pub struct PluginValidateRequest {
    pub(crate) path: PathBuf,
    pub(crate) timeout: Option<Duration>,
}

impl PluginValidateRequest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            timeout: None,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin", "validate"])
            .arg(self.path.to_string_lossy().to_string());
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

#[derive(Debug, Clone)]
pub struct PluginListRequest {
    pub(crate) available: bool,
    pub(crate) json: bool,
    pub(crate) timeout: Option<Duration>,
}

impl PluginListRequest {
    pub fn new() -> Self {
        Self {
            available: false,
            json: false,
            timeout: None,
        }
    }

    pub fn available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub fn json(mut self, json: bool) -> Self {
        self.json = json;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut args = Vec::<String>::new();
        if self.available {
            args.push("--available".to_string());
        }
        if self.json {
            args.push("--json".to_string());
        }

        let mut cmd = ClaudeCommandRequest::new(["plugin", "list"]).args(args);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginListRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginManifestRequest {
    pub(crate) timeout: Option<Duration>,
}

impl PluginManifestRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin", "manifest"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginManifestRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginManifestMarketplaceRequest {
    pub(crate) timeout: Option<Duration>,
}

impl PluginManifestMarketplaceRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin", "manifest", "marketplace"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginManifestMarketplaceRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginMarketplaceRepoRequest {
    pub(crate) timeout: Option<Duration>,
}

impl PluginMarketplaceRepoRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin", "marketplace", "repo"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginMarketplaceRepoRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginMarketplaceAddRequest {
    pub(crate) source: String,
    pub(crate) timeout: Option<Duration>,
}

impl PluginMarketplaceAddRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            timeout: None,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin", "marketplace", "add"]).arg(self.source);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

#[derive(Debug, Clone)]
pub struct PluginMarketplaceListRequest {
    pub(crate) json: bool,
    pub(crate) timeout: Option<Duration>,
}

impl PluginMarketplaceListRequest {
    pub fn new() -> Self {
        Self {
            json: false,
            timeout: None,
        }
    }

    pub fn json(mut self, json: bool) -> Self {
        self.json = json;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut args = Vec::<String>::new();
        if self.json {
            args.push("--json".to_string());
        }

        let mut cmd = ClaudeCommandRequest::new(["plugin", "marketplace", "list"]).args(args);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginMarketplaceListRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginMarketplaceRemoveRequest {
    pub(crate) timeout: Option<Duration>,
}

impl PluginMarketplaceRemoveRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin", "marketplace", "remove"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginMarketplaceRemoveRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginMarketplaceUpdateRequest {
    pub(crate) timeout: Option<Duration>,
}

impl PluginMarketplaceUpdateRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["plugin", "marketplace", "update"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for PluginMarketplaceUpdateRequest {
    fn default() -> Self {
        Self::new()
    }
}
