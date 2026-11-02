use std::path::PathBuf;

use codex::ThreadEvent;

pub(super) use super::super::*;
pub(super) use crate::{
    backend_harness::{BackendDefaults, BackendHarnessAdapter},
    backends::test_support::{test_env_lock, CurrentDirGuard},
    mcp::{
        AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpGetRequest,
        AgentWrapperMcpListRequest, AgentWrapperMcpRemoveRequest, CAPABILITY_MCP_ADD_V1,
        CAPABILITY_MCP_GET_V1, CAPABILITY_MCP_LIST_V1, CAPABILITY_MCP_REMOVE_V1,
    },
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperRunRequest,
};
pub(super) use futures_util::StreamExt;
use serde_json::{json, Value};

pub(super) use super::super::super::session_selectors::EXT_SESSION_FORK_V1;

pub(super) fn success_exit_status() -> std::process::ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
}

pub(super) fn parse_thread_event(json: &str) -> ThreadEvent {
    serde_json::from_str(json).expect("valid codex::ThreadEvent JSON")
}

pub(super) fn map(json: &str) -> AgentWrapperEvent {
    let event = parse_thread_event(json);
    map_thread_event(&event)
}

pub(super) fn tool_schema(event: &AgentWrapperEvent) -> Option<&str> {
    event
        .data
        .as_ref()
        .and_then(|data| data.get("schema"))
        .and_then(Value::as_str)
}

pub(super) fn handle_schema(event: &AgentWrapperEvent) -> Option<&str> {
    event
        .data
        .as_ref()
        .and_then(|data| data.get("schema"))
        .and_then(Value::as_str)
}

pub(super) fn tool_field<'a>(event: &'a AgentWrapperEvent, field: &str) -> Option<&'a Value> {
    event
        .data
        .as_ref()
        .and_then(|data| data.get("tool"))
        .and_then(|tool| tool.get(field))
}

pub(super) fn sample_mcp_add_request() -> AgentWrapperMcpAddRequest {
    AgentWrapperMcpAddRequest {
        name: "demo".to_string(),
        transport: AgentWrapperMcpAddTransport::Stdio {
            command: vec!["node".to_string()],
            args: vec!["server.js".to_string()],
            env: std::collections::BTreeMap::from([(
                "SERVER_ONLY".to_string(),
                "server-value".to_string(),
            )]),
        },
        context: Default::default(),
    }
}

pub(super) fn sample_mcp_remove_request() -> AgentWrapperMcpRemoveRequest {
    AgentWrapperMcpRemoveRequest {
        name: "demo".to_string(),
        context: Default::default(),
    }
}

pub(super) fn test_adapter_with_config(config: CodexBackendConfig) -> CodexHarnessAdapter {
    new_test_adapter(config)
}

pub(super) fn test_adapter_with_run_start_cwd(
    run_start_cwd: Option<PathBuf>,
) -> CodexHarnessAdapter {
    new_test_adapter_with_run_start_cwd(CodexBackendConfig::default(), run_start_cwd)
}

pub(super) fn test_adapter_with_config_and_run_start_cwd(
    config: CodexBackendConfig,
    run_start_cwd: Option<PathBuf>,
) -> CodexHarnessAdapter {
    new_test_adapter_with_run_start_cwd(config, run_start_cwd)
}

pub(super) fn test_adapter() -> CodexHarnessAdapter {
    test_adapter_with_config(CodexBackendConfig::default())
}

pub(super) fn add_dirs_payload(dirs: &[impl AsRef<str>]) -> Value {
    json!({
        "dirs": dirs.iter().map(|dir| dir.as_ref()).collect::<Vec<_>>()
    })
}
