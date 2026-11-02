use std::ffi::OsString;

use crate::{
    mcp::{
        AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext, AgentWrapperMcpCommandOutput,
    },
    AgentWrapperError,
};

mod argv;
mod resolve;
mod runner;

const CODEX_BINARY_ENV: &str = "CODEX_BINARY";
const CODEX_HOME_ENV: &str = "CODEX_HOME";
const PATH_ENV: &str = "PATH";

const PINNED_SPAWN_FAILURE: &str = "codex backend error: spawn (details redacted when unsafe)";
const PINNED_WAIT_FAILURE: &str = "codex backend error: wait (details redacted when unsafe)";
const PINNED_CAPTURE_FAILURE: &str = "codex backend error: capture (details redacted when unsafe)";
const PINNED_PREPARE_CODEX_HOME_FAILURE: &str =
    "codex backend error: prepare CODEX_HOME (details redacted when unsafe)";
const PINNED_MCP_RUNTIME_CONFLICT: &str =
    "codex backend error: installed codex does not support pinned mcp management command shape (details redacted)";

pub(super) fn codex_mcp_list_argv() -> Vec<OsString> {
    argv::codex_mcp_list_argv()
}

pub(super) fn codex_mcp_get_argv(name: &str) -> Vec<OsString> {
    argv::codex_mcp_get_argv(name)
}

pub(super) fn codex_mcp_remove_argv(name: &str) -> Vec<OsString> {
    argv::codex_mcp_remove_argv(name)
}

pub(super) fn codex_mcp_add_argv(
    name: &str,
    transport: &AgentWrapperMcpAddTransport,
) -> Vec<OsString> {
    argv::codex_mcp_add_argv(name, transport)
}

pub(super) async fn run_codex_mcp(
    config: super::CodexBackendConfig,
    argv: Vec<OsString>,
    context: AgentWrapperMcpCommandContext,
) -> Result<AgentWrapperMcpCommandOutput, AgentWrapperError> {
    let resolved = resolve::resolve_codex_mcp_command(&config, &context)?;
    let captured = runner::capture_codex_mcp_output(&resolved, &argv).await?;
    runner::finalize_codex_mcp_output(&argv, captured)
}

fn backend_error(message: &'static str) -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests;
