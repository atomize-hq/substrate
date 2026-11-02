use std::ffi::OsString;

use tokio::process::Command;

use crate::{
    builder::{apply_cli_overrides, resolve_cli_overrides},
    process::spawn_with_retry,
    ApplyDiffArtifacts, CodexClient, CodexError, McpAddRequest, McpAddTransport, McpGetRequest,
    McpListOutput, McpListRequest, McpLogoutRequest, McpOauthLoginRequest, McpOverviewRequest,
    McpRemoveRequest,
};

impl CodexClient {
    /// Runs `codex mcp --help` and returns captured output.
    pub async fn mcp_overview(
        &self,
        request: McpOverviewRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        self.run_simple_command_with_overrides(
            vec![OsString::from("mcp"), OsString::from("--help")],
            request.overrides,
        )
        .await
    }

    /// Lists configured MCP servers via `codex mcp list`.
    pub async fn mcp_list(&self, request: McpListRequest) -> Result<McpListOutput, CodexError> {
        let McpListRequest { json, overrides } = request;
        let mut args = vec![OsString::from("mcp"), OsString::from("list")];
        if json {
            args.push(OsString::from("--json"));
        }

        let artifacts = self
            .run_simple_command_with_overrides(args, overrides)
            .await?;
        let parsed = if json {
            Some(serde_json::from_str(&artifacts.stdout).map_err(|source| {
                CodexError::JsonParse {
                    context: "mcp list",
                    stdout: artifacts.stdout.clone(),
                    source,
                }
            })?)
        } else {
            None
        };

        Ok(McpListOutput {
            status: artifacts.status,
            stdout: artifacts.stdout,
            stderr: artifacts.stderr,
            json: parsed,
        })
    }

    /// Gets a configured MCP server entry via `codex mcp get <NAME>`.
    pub async fn mcp_get(&self, request: McpGetRequest) -> Result<McpListOutput, CodexError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(CodexError::EmptyMcpServerName);
        }

        let mut args = vec![OsString::from("mcp"), OsString::from("get")];
        if request.json {
            args.push(OsString::from("--json"));
        }
        args.push(OsString::from(name));

        let artifacts = self
            .run_simple_command_with_overrides(args, request.overrides)
            .await?;
        let parsed = if request.json {
            Some(serde_json::from_str(&artifacts.stdout).map_err(|source| {
                CodexError::JsonParse {
                    context: "mcp get",
                    stdout: artifacts.stdout.clone(),
                    source,
                }
            })?)
        } else {
            None
        };

        Ok(McpListOutput {
            status: artifacts.status,
            stdout: artifacts.stdout,
            stderr: artifacts.stderr,
            json: parsed,
        })
    }

    /// Adds an MCP server configuration entry via `codex mcp add`.
    pub async fn mcp_add(&self, request: McpAddRequest) -> Result<ApplyDiffArtifacts, CodexError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(CodexError::EmptyMcpServerName);
        }

        let mut args = vec![
            OsString::from("mcp"),
            OsString::from("add"),
            OsString::from(name),
        ];
        match request.transport {
            McpAddTransport::StreamableHttp {
                url,
                bearer_token_env_var,
            } => {
                let url = url.trim();
                if url.is_empty() {
                    return Err(CodexError::EmptyMcpUrl);
                }
                args.push(OsString::from("--url"));
                args.push(OsString::from(url));
                if let Some(env_var) = bearer_token_env_var {
                    if !env_var.trim().is_empty() {
                        args.push(OsString::from("--bearer-token-env-var"));
                        args.push(OsString::from(env_var));
                    }
                }
            }
            McpAddTransport::Stdio { env, command } => {
                if command.is_empty() {
                    return Err(CodexError::EmptyMcpCommand);
                }
                for (key, value) in env {
                    let key = key.trim();
                    if key.is_empty() {
                        continue;
                    }
                    args.push(OsString::from("--env"));
                    args.push(OsString::from(format!("{key}={value}")));
                }
                args.push(OsString::from("--"));
                args.extend(command);
            }
        }

        self.run_simple_command_with_overrides(args, request.overrides)
            .await
    }

    /// Removes an MCP server configuration entry via `codex mcp remove <NAME>`.
    pub async fn mcp_remove(
        &self,
        request: McpRemoveRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(CodexError::EmptyMcpServerName);
        }

        self.run_simple_command_with_overrides(
            vec![
                OsString::from("mcp"),
                OsString::from("remove"),
                OsString::from(name),
            ],
            request.overrides,
        )
        .await
    }

    /// Deauthenticates from an MCP server via `codex mcp logout <NAME>`.
    pub async fn mcp_logout(
        &self,
        request: McpLogoutRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(CodexError::EmptyMcpServerName);
        }

        self.run_simple_command_with_overrides(
            vec![
                OsString::from("mcp"),
                OsString::from("logout"),
                OsString::from(name),
            ],
            request.overrides,
        )
        .await
    }

    /// Spawns `codex mcp login <NAME> [--scopes ...]`.
    pub fn spawn_mcp_oauth_login_process(
        &self,
        request: McpOauthLoginRequest,
    ) -> Result<tokio::process::Child, CodexError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(CodexError::EmptyMcpServerName);
        }

        let resolved_overrides = resolve_cli_overrides(
            &self.cli_overrides,
            &request.overrides,
            self.model.as_deref(),
        );

        let mut command = Command::new(self.command_env.binary_path());
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        apply_cli_overrides(&mut command, &resolved_overrides, true);
        command.arg("mcp").arg("login").arg(name);

        if !request.scopes.is_empty() {
            command.arg("--scopes").arg(request.scopes.join(","));
        }

        self.command_env.apply(&mut command)?;

        spawn_with_retry(&mut command, self.command_env.binary_path())
    }
}
