use crate::CliOverridesPatch;
use serde_json::Value;
use std::{ffi::OsString, process::ExitStatus};

/// Request for `codex mcp` (overview/help).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpOverviewRequest {
    pub overrides: CliOverridesPatch,
}

impl McpOverviewRequest {
    pub fn new() -> Self {
        Self {
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for McpOverviewRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex mcp list`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpListRequest {
    pub json: bool,
    pub overrides: CliOverridesPatch,
}

impl McpListRequest {
    pub fn new() -> Self {
        Self {
            json: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn json(mut self, enable: bool) -> Self {
        self.json = enable;
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for McpListRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from `codex mcp list`.
#[derive(Clone, Debug, PartialEq)]
pub struct McpListOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub json: Option<Value>,
}

/// Request for `codex mcp get <NAME>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpGetRequest {
    pub name: String,
    pub json: bool,
    pub overrides: CliOverridesPatch,
}

impl McpGetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            json: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn json(mut self, enable: bool) -> Self {
        self.json = enable;
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Transport for `codex mcp add`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum McpAddTransport {
    Stdio {
        env: Vec<(String, String)>,
        command: Vec<OsString>,
    },
    StreamableHttp {
        url: String,
        bearer_token_env_var: Option<String>,
    },
}

/// Request for `codex mcp add`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpAddRequest {
    pub name: String,
    pub transport: McpAddTransport,
    pub overrides: CliOverridesPatch,
}

impl McpAddRequest {
    pub fn stdio(name: impl Into<String>, command: Vec<OsString>) -> Self {
        Self {
            name: name.into(),
            transport: McpAddTransport::Stdio {
                env: Vec::new(),
                command,
            },
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn streamable_http(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpAddTransport::StreamableHttp {
                url: url.into(),
                bearer_token_env_var: None,
            },
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let McpAddTransport::Stdio { env, .. } = &mut self.transport {
            env.push((key.into(), value.into()));
        }
        self
    }

    pub fn bearer_token_env_var(mut self, env_var: impl Into<String>) -> Self {
        if let McpAddTransport::StreamableHttp {
            bearer_token_env_var,
            ..
        } = &mut self.transport
        {
            let env_var = env_var.into();
            *bearer_token_env_var = (!env_var.trim().is_empty()).then_some(env_var);
        }
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex mcp remove <NAME>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpRemoveRequest {
    pub name: String,
    pub overrides: CliOverridesPatch,
}

impl McpRemoveRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex mcp logout <NAME>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpLogoutRequest {
    pub name: String,
    pub overrides: CliOverridesPatch,
}

impl McpLogoutRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex mcp login <NAME>` (OAuth).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpOauthLoginRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub overrides: CliOverridesPatch,
}

impl McpOauthLoginRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            scopes: Vec::new(),
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn scopes<I, S>(mut self, scopes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.scopes.extend(
            scopes
                .into_iter()
                .map(|s| s.into())
                .filter(|s| !s.trim().is_empty()),
        );
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}
