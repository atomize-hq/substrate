use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Duration};

use tokio::sync::OnceCell;

use crate::EXT_AGENT_API_CONFIG_MODEL_V1;

impl super::termination::TerminationHandle for claude_code::ClaudeTerminationHandle {
    fn request_termination(&self) {
        claude_code::ClaudeTerminationHandle::request_termination(self);
    }
}

const AGENT_KIND: &str = "claude_code";
const CHANNEL_ASSISTANT: &str = "assistant";
const CHANNEL_TOOL: &str = "tool";

const EXT_ADD_DIRS_V1: &str = "agent_api.exec.add_dirs.v1";
const EXT_NON_INTERACTIVE: &str = "agent_api.exec.non_interactive";
const EXT_EXTERNAL_SANDBOX_V1: &str = "agent_api.exec.external_sandbox.v1";
const CLAUDE_EXEC_POLICY_PREFIX: &str = "backend.claude_code.exec.";

const SUPPORTED_EXTENSION_KEYS_DEFAULT: &[&str] = &[
    EXT_ADD_DIRS_V1,
    EXT_NON_INTERACTIVE,
    EXT_AGENT_API_CONFIG_MODEL_V1,
    EXT_SESSION_RESUME_V1,
    EXT_SESSION_FORK_V1,
];

const SUPPORTED_EXTENSION_KEYS_EXTERNAL_SANDBOX_OPT_IN: &[&str] = &[
    EXT_ADD_DIRS_V1,
    EXT_NON_INTERACTIVE,
    EXT_AGENT_API_CONFIG_MODEL_V1,
    EXT_SESSION_RESUME_V1,
    EXT_SESSION_FORK_V1,
    EXT_EXTERNAL_SANDBOX_V1,
];

const CAP_TOOLS_STRUCTURED_V1: &str = "agent_api.tools.structured.v1";
const CAP_TOOLS_RESULTS_V1: &str = "agent_api.tools.results.v1";
const CAP_ARTIFACTS_FINAL_TEXT_V1: &str = "agent_api.artifacts.final_text.v1";
const CAP_SESSION_HANDLE_V1: &str = "agent_api.session.handle.v1";

const SESSION_HANDLE_ID_BOUND_BYTES: usize = 1024;
const SESSION_HANDLE_OVERSIZE_WARNING: &str = "session handle omitted: id exceeds 1024 bytes";
const PINNED_EXTERNAL_SANDBOX_WARNING: &str =
    "DANGEROUS: external sandbox exec policy enabled (agent_api.exec.external_sandbox.v1=true)";

fn claude_mcp_list_supported_on_target() -> bool {
    cfg!(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    ))
}

fn claude_mcp_get_supported_on_target() -> bool {
    cfg!(all(target_os = "windows", target_arch = "x86_64"))
}

mod backend;
mod harness;
mod mapping;
mod mcp_management;
mod util;

use super::session_selectors::{EXT_SESSION_FORK_V1, EXT_SESSION_RESUME_V1};

#[cfg(test)]
use harness::{new_test_adapter, new_test_adapter_with_run_start_cwd, ClaudeHarnessAdapter};
#[cfg(test)]
use mapping::{map_assistant_message, map_stream_event};

#[derive(Clone, Debug, Default)]
pub struct ClaudeCodeBackendConfig {
    pub binary: Option<PathBuf>,
    pub claude_home: Option<PathBuf>,
    pub default_timeout: Option<Duration>,
    pub default_working_dir: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
    pub allow_mcp_write: bool,
    pub allow_external_sandbox_exec: bool,
}

pub struct ClaudeCodeBackend {
    config: ClaudeCodeBackendConfig,
    allow_flag_preflight: Arc<OnceCell<bool>>,
}

impl ClaudeCodeBackend {
    pub fn new(config: ClaudeCodeBackendConfig) -> Self {
        Self {
            config,
            allow_flag_preflight: Arc::new(OnceCell::new()),
        }
    }
}

#[cfg(test)]
mod tests;
