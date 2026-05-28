use anyhow::Result;

use crate::execution::agent_inventory::AgentCliRuntimeFamily;

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

fn backend_kind_from_runtime_family(
    runtime_family: AgentCliRuntimeFamily,
) -> AgentRuntimeBackendKind {
    match runtime_family {
        AgentCliRuntimeFamily::Codex => AgentRuntimeBackendKind::Codex,
        AgentCliRuntimeFamily::ClaudeCode => AgentRuntimeBackendKind::ClaudeCode,
    }
}

pub(crate) fn resolve_shell_owned_runtime_family(
    agent_id: &str,
    runtime_family: Option<AgentCliRuntimeFamily>,
) -> Result<AgentRuntimeBackendKind> {
    runtime_family
        .map(backend_kind_from_runtime_family)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "selected runtime '{}' is not runtime-realizable because config.cli.runtime_family is required for shell-owned UAA runtimes",
                agent_id
            )
        })
}

pub(crate) fn protocol_validation_error(subject: &str, actual: Option<&str>) -> String {
    match actual {
        Some(LEGACY_PURE_AGENT_PROTOCOL) => format!(
            "{subject} advertises legacy unsupported protocol '{LEGACY_PURE_AGENT_PROTOCOL}'; rename it to '{PURE_AGENT_PROTOCOL}'"
        ),
        _ => format!("{subject} does not advertise protocol '{PURE_AGENT_PROTOCOL}'"),
    }
}
