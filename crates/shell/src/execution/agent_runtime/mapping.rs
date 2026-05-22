use anyhow::Result;

pub(crate) const PURE_AGENT_PROTOCOL: &str = "substrate.agent.session";
pub(crate) const LEGACY_PURE_AGENT_PROTOCOL: &str = concat!("uaa.agent", ".session");
pub(crate) const PURE_AGENT_ROUTER: &str = "agent_hub";
pub(crate) const NESTED_ROUTER: &str = "substrate_gateway";
pub(crate) const ORCHESTRATOR_ROLE: &str = "orchestrator";
pub(crate) const MEMBER_ROLE: &str = "member";
pub(crate) const SESSION_HANDLE_SCHEMA_V1: &str = "agent_api.session.handle.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AgentRuntimeBackendKind {
    Codex,
    ClaudeCode,
}

impl AgentRuntimeBackendKind {
    pub(crate) fn as_agent_kind_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude_code",
        }
    }
}

pub(crate) fn orchestrator_backend_kind(agent_id: &str) -> Result<AgentRuntimeBackendKind> {
    match agent_id {
        "codex" => Ok(AgentRuntimeBackendKind::Codex),
        "claude_code" => Ok(AgentRuntimeBackendKind::ClaudeCode),
        other => Err(anyhow::anyhow!(
            "selected orchestrator backend '{other}' is not supported by the shell-owned UAA runtime; supported backends are cli:codex and cli:claude_code"
        )),
    }
}

pub(crate) fn protocol_validation_error(subject: &str, actual: Option<&str>) -> String {
    match actual {
        Some(LEGACY_PURE_AGENT_PROTOCOL) => format!(
            "{subject} advertises legacy unsupported protocol '{LEGACY_PURE_AGENT_PROTOCOL}'; rename it to '{PURE_AGENT_PROTOCOL}'"
        ),
        _ => format!("{subject} does not advertise protocol '{PURE_AGENT_PROTOCOL}'"),
    }
}
