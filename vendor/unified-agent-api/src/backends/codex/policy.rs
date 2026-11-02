use serde_json::Value;

use crate::{AgentWrapperError, AgentWrapperRunRequest, EXT_AGENT_API_CONFIG_MODEL_V1};

pub(super) const EXT_ADD_DIRS_V1: &str = "agent_api.exec.add_dirs.v1";
pub(super) const EXT_NON_INTERACTIVE: &str = "agent_api.exec.non_interactive";
pub(super) const EXT_EXTERNAL_SANDBOX_V1: &str = "agent_api.exec.external_sandbox.v1";
pub(super) const EXT_CODEX_APPROVAL_POLICY: &str = "backend.codex.exec.approval_policy";
pub(super) const EXT_CODEX_SANDBOX_MODE: &str = "backend.codex.exec.sandbox_mode";

pub(super) const SUPPORTED_EXTENSION_KEYS_DEFAULT: &[&str] = &[
    EXT_ADD_DIRS_V1,
    EXT_NON_INTERACTIVE,
    EXT_AGENT_API_CONFIG_MODEL_V1,
    EXT_CODEX_APPROVAL_POLICY,
    EXT_CODEX_SANDBOX_MODE,
    super::EXT_SESSION_RESUME_V1,
    super::EXT_SESSION_FORK_V1,
];

pub(super) const SUPPORTED_EXTENSION_KEYS_EXTERNAL_SANDBOX_OPT_IN: &[&str] = &[
    EXT_ADD_DIRS_V1,
    EXT_NON_INTERACTIVE,
    EXT_AGENT_API_CONFIG_MODEL_V1,
    EXT_CODEX_APPROVAL_POLICY,
    EXT_CODEX_SANDBOX_MODE,
    super::EXT_SESSION_RESUME_V1,
    super::EXT_SESSION_FORK_V1,
    EXT_EXTERNAL_SANDBOX_V1,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum CodexApprovalPolicy {
    Untrusted,
    OnFailure,
    OnRequest,
    Never,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum CodexSandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

pub(super) fn parse_bool(value: &Value, key: &str) -> Result<bool, AgentWrapperError> {
    value
        .as_bool()
        .ok_or_else(|| AgentWrapperError::InvalidRequest {
            message: format!("{key} must be a boolean"),
        })
}

fn parse_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, AgentWrapperError> {
    value
        .as_str()
        .ok_or_else(|| AgentWrapperError::InvalidRequest {
            message: format!("{key} must be a string"),
        })
}

fn parse_codex_approval_policy(value: &Value) -> Result<CodexApprovalPolicy, AgentWrapperError> {
    let raw = parse_string(value, EXT_CODEX_APPROVAL_POLICY)?;
    match raw {
        "untrusted" => Ok(CodexApprovalPolicy::Untrusted),
        "on-failure" => Ok(CodexApprovalPolicy::OnFailure),
        "on-request" => Ok(CodexApprovalPolicy::OnRequest),
        "never" => Ok(CodexApprovalPolicy::Never),
        other => Err(AgentWrapperError::InvalidRequest {
            message: format!(
                "{EXT_CODEX_APPROVAL_POLICY} must be one of: untrusted | on-failure | on-request | never (got {other:?})"
            ),
        }),
    }
}

fn parse_codex_sandbox_mode(value: &Value) -> Result<CodexSandboxMode, AgentWrapperError> {
    let raw = parse_string(value, EXT_CODEX_SANDBOX_MODE)?;
    match raw {
        "read-only" => Ok(CodexSandboxMode::ReadOnly),
        "workspace-write" => Ok(CodexSandboxMode::WorkspaceWrite),
        "danger-full-access" => Ok(CodexSandboxMode::DangerFullAccess),
        other => Err(AgentWrapperError::InvalidRequest {
            message: format!(
                "{EXT_CODEX_SANDBOX_MODE} must be one of: read-only | workspace-write | danger-full-access (got {other:?})"
            ),
        }),
    }
}

#[derive(Clone, Debug)]
pub(super) struct CodexExecPolicy {
    pub(super) add_dirs: Vec<std::path::PathBuf>,
    pub(super) non_interactive: bool,
    pub(super) external_sandbox: bool,
    pub(super) approval_policy: Option<CodexApprovalPolicy>,
    pub(super) sandbox_mode: CodexSandboxMode,
    pub(super) resume: Option<super::SessionSelectorV1>,
    pub(super) fork: Option<super::SessionSelectorV1>,
}

pub(super) fn validate_and_extract_exec_policy(
    request: &AgentWrapperRunRequest,
) -> Result<CodexExecPolicy, AgentWrapperError> {
    let non_interactive_requested: Option<bool> = request
        .extensions
        .get(EXT_NON_INTERACTIVE)
        .map(|value| parse_bool(value, EXT_NON_INTERACTIVE))
        .transpose()?;
    let non_interactive = non_interactive_requested.unwrap_or(true);

    let external_sandbox = request
        .extensions
        .get(EXT_EXTERNAL_SANDBOX_V1)
        .map(|value| parse_bool(value, EXT_EXTERNAL_SANDBOX_V1))
        .transpose()?
        .unwrap_or(false);

    if external_sandbox {
        if non_interactive_requested == Some(false) {
            return Err(AgentWrapperError::InvalidRequest {
                message: format!(
                    "{EXT_EXTERNAL_SANDBOX_V1}=true must not be combined with {EXT_NON_INTERACTIVE}=false"
                ),
            });
        }

        if request
            .extensions
            .keys()
            .any(|key| key.starts_with("backend.codex.exec."))
        {
            return Err(AgentWrapperError::InvalidRequest {
                message: format!(
                    "{EXT_EXTERNAL_SANDBOX_V1}=true must not be combined with backend.codex.exec.* keys"
                ),
            });
        }
    }

    let approval_policy = request
        .extensions
        .get(EXT_CODEX_APPROVAL_POLICY)
        .map(parse_codex_approval_policy)
        .transpose()?;

    let sandbox_mode = request
        .extensions
        .get(EXT_CODEX_SANDBOX_MODE)
        .map(parse_codex_sandbox_mode)
        .transpose()?
        .unwrap_or(CodexSandboxMode::WorkspaceWrite);

    if non_interactive
        && matches!(
            approval_policy,
            Some(ref policy) if policy != &CodexApprovalPolicy::Never
        )
    {
        return Err(AgentWrapperError::InvalidRequest {
            message: format!(
                "{EXT_CODEX_APPROVAL_POLICY} must be \"never\" when {EXT_NON_INTERACTIVE} is true"
            ),
        });
    }

    Ok(CodexExecPolicy {
        add_dirs: Vec::new(),
        non_interactive,
        external_sandbox,
        approval_policy,
        sandbox_mode,
        resume: None,
        fork: None,
    })
}
