use anyhow::Result;

pub(crate) const PURE_AGENT_PROTOCOL: &str = "uaa.agent.session";
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
