#![allow(dead_code)]

use std::ffi::OsString;

use crate::{
    mcp::{AgentWrapperMcpCommandContext, AgentWrapperMcpCommandOutput},
    AgentWrapperError,
};

mod argv;
mod resolve;
mod runner;

const CLAUDE_BINARY_ENV: &str = "CLAUDE_BINARY";
const CLAUDE_HOME_ENV: &str = "CLAUDE_HOME";
const DISABLE_AUTOUPDATER_ENV: &str = "DISABLE_AUTOUPDATER";
const HOME_ENV: &str = "HOME";
const PATH_ENV: &str = "PATH";
const XDG_CACHE_HOME_ENV: &str = "XDG_CACHE_HOME";
const XDG_CONFIG_HOME_ENV: &str = "XDG_CONFIG_HOME";
const XDG_DATA_HOME_ENV: &str = "XDG_DATA_HOME";
#[cfg(windows)]
const USERPROFILE_ENV: &str = "USERPROFILE";
#[cfg(windows)]
const APPDATA_ENV: &str = "APPDATA";
#[cfg(windows)]
const LOCALAPPDATA_ENV: &str = "LOCALAPPDATA";

const PINNED_CAPTURE_FAILURE: &str =
    "claude_code backend error: capture (details redacted when unsafe)";
const PINNED_MCP_RUNTIME_CONFLICT: &str =
    "claude_code backend error: installed claude does not support pinned mcp management command shape (details redacted)";
const PINNED_PREPARE_CLAUDE_HOME_FAILURE: &str =
    "claude_code backend error: prepare CLAUDE_HOME (details redacted when unsafe)";
const PINNED_SPAWN_FAILURE: &str =
    "claude_code backend error: spawn (details redacted when unsafe)";
const PINNED_TIMEOUT_FAILURE: &str =
    "claude_code backend error: timeout (details redacted when unsafe)";
const PINNED_WAIT_FAILURE: &str = "claude_code backend error: wait (details redacted when unsafe)";
const PINNED_URL_BEARER_TOKEN_ENV_VAR_UNSUPPORTED: &str =
    "claude mcp add url transport does not support bearer_token_env_var";

pub(super) fn claude_mcp_list_argv() -> Vec<OsString> {
    argv::claude_mcp_list_argv()
}

pub(super) fn claude_mcp_get_argv(name: &str) -> Vec<OsString> {
    argv::claude_mcp_get_argv(name)
}

pub(super) fn claude_mcp_remove_argv(name: &str) -> Vec<OsString> {
    argv::claude_mcp_remove_argv(name)
}

pub(super) fn claude_mcp_add_argv(
    name: &str,
    transport: &crate::mcp::AgentWrapperMcpAddTransport,
) -> Result<Vec<OsString>, AgentWrapperError> {
    argv::claude_mcp_add_argv(name, transport)
}

pub(super) async fn run_claude_mcp(
    config: super::ClaudeCodeBackendConfig,
    argv: Vec<OsString>,
    context: AgentWrapperMcpCommandContext,
) -> Result<AgentWrapperMcpCommandOutput, AgentWrapperError> {
    let resolved = resolve::resolve_claude_mcp_command(&config, &context)?;
    let captured = runner::capture_claude_mcp_output(&resolved, &argv).await?;
    runner::finalize_claude_mcp_output(&argv, captured)
}

fn backend_error(message: &'static str) -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: message.to_string(),
    }
}

fn invalid_request(message: &'static str) -> AgentWrapperError {
    AgentWrapperError::InvalidRequest {
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests;
