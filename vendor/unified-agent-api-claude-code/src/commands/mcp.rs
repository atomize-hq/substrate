use crate::commands::command::ClaudeCommandRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpScope {
    Local,
    User,
    Project,
}

impl McpScope {
    fn as_arg_value(self) -> &'static str {
        match self {
            McpScope::Local => "local",
            McpScope::User => "user",
            McpScope::Project => "project",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTransport {
    Stdio,
    Sse,
    Http,
}

impl McpTransport {
    fn as_arg_value(self) -> &'static str {
        match self {
            McpTransport::Stdio => "stdio",
            McpTransport::Sse => "sse",
            McpTransport::Http => "http",
        }
    }
}

#[derive(Debug, Clone)]
pub struct McpAddRequest {
    pub name: String,
    pub command_or_url: String,
    pub args: Vec<String>,
    pub scope: Option<McpScope>,
    pub transport: Option<McpTransport>,
    pub env: Vec<String>,
    pub headers: Vec<String>,
}

impl McpAddRequest {
    pub fn new(name: impl Into<String>, command_or_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command_or_url: command_or_url.into(),
            args: Vec::new(),
            scope: None,
            transport: None,
            env: Vec::new(),
            headers: Vec::new(),
        }
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn scope(mut self, scope: McpScope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn transport(mut self, transport: McpTransport) -> Self {
        self.transport = Some(transport);
        self
    }

    pub fn env(mut self, env: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.env = env.into_iter().map(Into::into).collect();
        self
    }

    pub fn headers(mut self, headers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.headers = headers.into_iter().map(Into::into).collect();
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut req = ClaudeCommandRequest::new(["mcp", "add"]);
        if let Some(scope) = self.scope {
            req = req.args(["--scope", scope.as_arg_value()]);
        }
        if let Some(transport) = self.transport {
            req = req.args(["--transport", transport.as_arg_value()]);
        }
        for e in self.env {
            req = req.args(["--env".to_string(), e]);
        }
        for h in self.headers {
            req = req.args(["--header".to_string(), h]);
        }
        req.args([self.name, self.command_or_url]).args(self.args)
    }
}

#[derive(Debug, Clone)]
pub struct McpRemoveRequest {
    pub name: String,
    pub scope: Option<McpScope>,
}

impl McpRemoveRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            scope: None,
        }
    }

    pub fn scope(mut self, scope: McpScope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut req = ClaudeCommandRequest::new(["mcp", "remove"]);
        if let Some(scope) = self.scope {
            req = req.args(["--scope", scope.as_arg_value()]);
        }
        req.arg(self.name)
    }
}

#[derive(Debug, Clone)]
pub struct McpGetRequest {
    pub name: String,
}

impl McpGetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        ClaudeCommandRequest::new(["mcp", "get"]).arg(self.name)
    }
}

#[derive(Debug, Clone)]
pub struct McpAddJsonRequest {
    pub name: String,
    pub json: String,
    pub scope: Option<McpScope>,
}

impl McpAddJsonRequest {
    pub fn new(name: impl Into<String>, json: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            json: json.into(),
            scope: None,
        }
    }

    pub fn scope(mut self, scope: McpScope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut req = ClaudeCommandRequest::new(["mcp", "add-json"]);
        if let Some(scope) = self.scope {
            req = req.args(["--scope", scope.as_arg_value()]);
        }
        req.args([self.name, self.json])
    }
}

#[derive(Debug, Clone)]
pub struct McpServeRequest {
    pub args: Vec<String>,
}

impl McpServeRequest {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        ClaudeCommandRequest::new(["mcp", "serve"]).args(self.args)
    }
}

impl Default for McpServeRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct McpAddFromClaudeDesktopRequest {
    pub scope: Option<McpScope>,
}

impl McpAddFromClaudeDesktopRequest {
    pub fn new() -> Self {
        Self { scope: None }
    }

    pub fn scope(mut self, scope: McpScope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut req = ClaudeCommandRequest::new(["mcp", "add-from-claude-desktop"]);
        if let Some(scope) = self.scope {
            req = req.args(["--scope", scope.as_arg_value()]);
        }
        req
    }
}

impl Default for McpAddFromClaudeDesktopRequest {
    fn default() -> Self {
        Self::new()
    }
}
