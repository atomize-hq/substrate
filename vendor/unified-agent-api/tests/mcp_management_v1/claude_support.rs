use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Duration};

use agent_api::{
    backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperGateway, AgentWrapperKind,
};

use super::support::McpTestSandbox;

pub(crate) const CAPABILITY_MCP_LIST_V1: &str = "agent_api.tools.mcp.list.v1";
pub(crate) const CAPABILITY_MCP_GET_V1: &str = "agent_api.tools.mcp.get.v1";
pub(crate) const CAPABILITY_MCP_ADD_V1: &str = "agent_api.tools.mcp.add.v1";
pub(crate) const CAPABILITY_MCP_REMOVE_V1: &str = "agent_api.tools.mcp.remove.v1";
pub(crate) const FAKE_CLAUDE_RECORD_PATH_ENV: &str = "FAKE_CLAUDE_MCP_RECORD_PATH";
pub(crate) const FAKE_CLAUDE_RECORD_ENV_KEYS_ENV: &str = "FAKE_CLAUDE_MCP_RECORD_ENV_KEYS";
pub(crate) const FAKE_CLAUDE_SCENARIO_ENV: &str = "FAKE_CLAUDE_MCP_SCENARIO";
pub(crate) const ALL_RECORDED_ENV_KEYS: &str =
    "CLI_ONLY,CONFIG_ONLY,OVERRIDE_ME,REQUEST_ONLY,MY_TOKEN,MCP_SERVER_ENV";

pub(crate) fn claude_gateway(
    sandbox: &McpTestSandbox,
    allow_mcp_write: bool,
    env: BTreeMap<String, String>,
    default_working_dir: Option<PathBuf>,
    default_timeout: Option<Duration>,
) -> (
    Arc<ClaudeCodeBackend>,
    AgentWrapperGateway,
    AgentWrapperKind,
) {
    claude_gateway_with_home(
        sandbox,
        sandbox.claude_home().to_path_buf(),
        allow_mcp_write,
        env,
        default_working_dir,
        default_timeout,
    )
}

pub(crate) fn claude_gateway_with_home(
    sandbox: &McpTestSandbox,
    claude_home: PathBuf,
    allow_mcp_write: bool,
    env: BTreeMap<String, String>,
    default_working_dir: Option<PathBuf>,
    default_timeout: Option<Duration>,
) -> (
    Arc<ClaudeCodeBackend>,
    AgentWrapperGateway,
    AgentWrapperKind,
) {
    let binary = sandbox.install_fake_claude().expect("install fake claude");
    let backend = Arc::new(ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(binary),
        claude_home: Some(claude_home),
        default_timeout,
        default_working_dir,
        env,
        allow_mcp_write,
        ..Default::default()
    }));

    let kind = backend.kind();
    let mut gateway = AgentWrapperGateway::new();
    gateway
        .register(backend.clone())
        .expect("register claude backend");
    (backend, gateway, kind)
}

pub(crate) fn claude_config_env(
    sandbox: &McpTestSandbox,
    extra: impl IntoIterator<Item = (String, String)>,
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([
        (
            FAKE_CLAUDE_RECORD_PATH_ENV.to_string(),
            sandbox.record_path().to_string_lossy().into_owned(),
        ),
        (
            FAKE_CLAUDE_RECORD_ENV_KEYS_ENV.to_string(),
            ALL_RECORDED_ENV_KEYS.to_string(),
        ),
    ]);
    env.extend(extra);
    env
}

pub(crate) fn claude_list_supported() -> bool {
    cfg!(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    ))
}

pub(crate) fn claude_get_supported() -> bool {
    cfg!(all(target_os = "windows", target_arch = "x86_64"))
}

pub(crate) fn assert_unsupported_capability(err: AgentWrapperError, expected_capability: &str) {
    match err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "claude_code");
            assert_eq!(capability, expected_capability);
        }
        other => panic!("expected UnsupportedCapability, got {other:?}"),
    }
}

pub(crate) fn backend_error_message(err: AgentWrapperError) -> String {
    match err {
        AgentWrapperError::Backend { message } => message,
        other => panic!("expected Backend error, got {other:?}"),
    }
}
