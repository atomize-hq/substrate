use std::{collections::BTreeMap, path::PathBuf, time::Duration};

impl super::termination::TerminationHandle for codex::ExecTerminationHandle {
    fn request_termination(&self) {
        codex::ExecTerminationHandle::request_termination(self);
    }
}

#[derive(Clone, Debug, Default)]
pub struct CodexBackendConfig {
    pub binary: Option<PathBuf>,
    pub codex_home: Option<PathBuf>,
    pub model: Option<String>,
    pub default_timeout: Option<Duration>,
    pub default_working_dir: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
    pub allow_mcp_write: bool,
    pub allow_external_sandbox_exec: bool,
}

pub struct CodexBackend {
    config: CodexBackendConfig,
}

impl CodexBackend {
    pub fn new(config: CodexBackendConfig) -> Self {
        Self { config }
    }
}

const PINNED_APPROVAL_REQUIRED: &str = "approval required";
const PINNED_ADD_DIRS_UNSUPPORTED_FOR_FORK: &str = "add_dirs unsupported for codex fork";
const PINNED_MODEL_OVERRIDE_UNSUPPORTED_FOR_FORK: &str =
    "model override unsupported for codex fork";
const PINNED_ADD_DIRS_RUNTIME_REJECTION: &str = "add_dirs rejected by runtime";
const PINNED_MODEL_RUNTIME_REJECTION: &str =
    "codex backend error: model rejected by runtime (details redacted)";
const PINNED_TIMEOUT: &str = "codex backend error: timeout (details redacted when unsafe)";
const PINNED_NO_SESSION_FOUND: &str = "no session found";
const PINNED_SESSION_NOT_FOUND: &str = "session not found";
const PINNED_EXTERNAL_SANDBOX_WARNING: &str =
    "DANGEROUS: external sandbox exec policy enabled (agent_api.exec.external_sandbox.v1=true)";
const PINNED_EXTERNAL_SANDBOX_FLAG_UNSUPPORTED: &str =
    "codex backend error: installed codex does not support --dangerously-bypass-approvals-and-sandbox (details redacted)";

const CAP_TOOLS_STRUCTURED_V1: &str = "agent_api.tools.structured.v1";
const CAP_TOOLS_RESULTS_V1: &str = "agent_api.tools.results.v1";
const CAP_ARTIFACTS_FINAL_TEXT_V1: &str = "agent_api.artifacts.final_text.v1";
const CAP_SESSION_HANDLE_V1: &str = "agent_api.session.handle.v1";

const TOOLS_FACET_SCHEMA: &str = "agent_api.tools.structured.v1";

const SESSION_HANDLE_ID_BOUND_BYTES: usize = 1024;
const SESSION_HANDLE_OVERSIZE_WARNING_MARKER: &str = "session handle id oversize";

fn pinned_selection_failure_message(selector: &SessionSelectorV1) -> &'static str {
    match selector {
        SessionSelectorV1::Last => PINNED_NO_SESSION_FOUND,
        SessionSelectorV1::Id { .. } => PINNED_SESSION_NOT_FOUND,
    }
}

fn is_add_dirs_runtime_rejection_signal(text: &str) -> bool {
    text == PINNED_ADD_DIRS_RUNTIME_REJECTION
}

fn is_not_found_signal(text: &str) -> bool {
    let text = text.to_ascii_lowercase();

    (text.contains("not found") && (text.contains("session") || text.contains("thread")))
        || text.contains("no session")
        || text.contains("no sessions")
        || text.contains("unknown session")
        || text.contains("no thread")
        || text.contains("no threads")
        || text.contains("unknown thread")
}

fn codex_mcp_supported_on_target() -> bool {
    // The pinned Codex MCP artifact is the Linux x86_64 musl binary, but it is intended to run
    // on standard Linux hosts regardless of whether the wrapper itself is built against musl or
    // glibc.
    cfg!(all(target_os = "linux", target_arch = "x86_64"))
}

mod backend;
mod exec;
mod fork;
mod harness;
mod mapping;
mod mcp_management;
mod policy;

use super::session_selectors::{SessionSelectorV1, EXT_SESSION_FORK_V1, EXT_SESSION_RESUME_V1};
use harness::{
    CodexBackendCompletion, CodexBackendError, CodexBackendEvent, CodexHandleFacetState,
    CodexTerminationHandle,
};
use policy::{
    validate_and_extract_exec_policy, CodexApprovalPolicy, CodexExecPolicy, CodexSandboxMode,
    EXT_ADD_DIRS_V1, EXT_CODEX_APPROVAL_POLICY, EXT_CODEX_SANDBOX_MODE, EXT_EXTERNAL_SANDBOX_V1,
    EXT_NON_INTERACTIVE, SUPPORTED_EXTENSION_KEYS_DEFAULT,
    SUPPORTED_EXTENSION_KEYS_EXTERNAL_SANDBOX_OPT_IN,
};

#[cfg(test)]
use harness::{
    new_test_adapter, new_test_adapter_with_run_start_cwd, redact_exec_stream_error,
    CodexHarnessAdapter,
};
#[cfg(test)]
use mapping::map_thread_event;

#[cfg(test)]
mod tests;
