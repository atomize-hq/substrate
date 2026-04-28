use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use substrate_broker::Policy;

use crate::execution::agent_inventory::{
    AgentCapabilitiesV1, AgentConfigKind, AgentInventoryEntryV1,
};
use crate::execution::config_model::{AgentCliMode, AgentExecutionScope, SubstrateConfig};

use super::mapping::{orchestrator_backend_kind, AgentRuntimeBackendKind, PURE_AGENT_PROTOCOL};

#[derive(Clone, Debug)]
pub(crate) struct RuntimeSelectionDescriptor {
    pub agent_id: String,
    pub backend_id: String,
    pub backend_kind: AgentRuntimeBackendKind,
    pub protocol: String,
    pub execution_scope: AgentExecutionScope,
    pub binary_path: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct RuntimeRealizabilityError {
    pub exit_code: i32,
    pub reason: String,
}

pub(crate) fn validate_orchestrator_selection<'a>(
    effective_config: &SubstrateConfig,
    inventory: &'a BTreeMap<String, AgentInventoryEntryV1>,
) -> std::result::Result<&'a AgentInventoryEntryV1, String> {
    if !effective_config.agents.enabled {
        return Err("agents are disabled by effective config".to_string());
    }

    let orchestrator_agent_id = effective_config.agents.hub.orchestrator_agent_id.trim();
    if orchestrator_agent_id.is_empty() {
        return Err("agents.hub.orchestrator_agent_id must select an orchestrator".to_string());
    }

    let entry = inventory.get(orchestrator_agent_id).ok_or_else(|| {
        format!(
            "agents.hub.orchestrator_agent_id '{}' is not present in the effective agent inventory",
            orchestrator_agent_id
        )
    })?;

    if !entry.file.config.enabled {
        return Err(format!(
            "selected orchestrator '{}' is disabled in the effective inventory",
            orchestrator_agent_id
        ));
    }

    if entry.effective_scope(effective_config) != AgentExecutionScope::Host {
        return Err(format!(
            "selected orchestrator '{}' must resolve to execution.scope=host",
            orchestrator_agent_id
        ));
    }

    if entry.file.config.protocol.as_deref() != Some(PURE_AGENT_PROTOCOL) {
        return Err(format!(
            "orchestrator agent '{}' does not advertise protocol '{}'",
            orchestrator_agent_id, PURE_AGENT_PROTOCOL
        ));
    }

    if let Some(capability) =
        missing_required_orchestrator_capability(&entry.file.config.capabilities)
    {
        return Err(format!(
            "orchestrator agent '{}' is missing required capability '{}'",
            orchestrator_agent_id, capability
        ));
    }

    Ok(entry)
}

pub(crate) fn missing_required_orchestrator_capability(
    capabilities: &AgentCapabilitiesV1,
) -> Option<&'static str> {
    [
        ("session_start", capabilities.session_start),
        ("session_resume", capabilities.session_resume),
        ("session_fork", capabilities.session_fork),
        ("session_stop", capabilities.session_stop),
        ("status_snapshot", capabilities.status_snapshot),
        ("event_stream", capabilities.event_stream),
    ]
    .into_iter()
    .find_map(|(name, enabled)| (!enabled).then_some(name))
}

pub(crate) fn backend_allowed(policy: &Policy, backend_id: &str) -> bool {
    policy
        .agents_allowed_backends
        .iter()
        .any(|allowed| allowed == backend_id)
}

pub(crate) fn validate_runtime_realizability(
    entry: &AgentInventoryEntryV1,
    effective_config: &SubstrateConfig,
) -> std::result::Result<RuntimeSelectionDescriptor, RuntimeRealizabilityError> {
    if entry.file.config.kind != AgentConfigKind::Cli {
        return Err(RuntimeRealizabilityError {
            exit_code: 2,
            reason: format!(
                "selected orchestrator '{}' is not runtime-realizable by the shell-owned UAA runtime because config.kind={} is unsupported; only config.kind=cli is supported in v1",
                entry.file.id,
                entry.file.config.kind.as_str()
            ),
        });
    }

    let cli_mode = entry.effective_cli_mode(effective_config);
    if cli_mode != AgentCliMode::Persistent {
        return Err(RuntimeRealizabilityError {
            exit_code: 2,
            reason: format!(
                "selected orchestrator '{}' is not runtime-realizable because cli.mode={} is unsupported; only cli.mode=persistent is supported for the first caller path",
                entry.file.id,
                match cli_mode {
                    AgentCliMode::Persistent => "persistent",
                    AgentCliMode::PerRequest => "per_request",
                }
            ),
        });
    }

    let backend_kind =
        orchestrator_backend_kind(&entry.file.id).map_err(|err| RuntimeRealizabilityError {
            exit_code: 2,
            reason: err.to_string(),
        })?;

    let binary = entry.effective_cli_binary().ok_or_else(|| RuntimeRealizabilityError {
        exit_code: 4,
        reason: format!(
            "selected orchestrator '{}' is not runtime-realizable because config.cli.binary is missing",
            entry.file.id
        ),
    })?;
    let binary_path = which::which(binary).map_err(|err| RuntimeRealizabilityError {
        exit_code: 4,
        reason: format!(
            "selected orchestrator '{}' is not runtime-realizable because config.cli.binary '{}' did not resolve on the host: {}",
            entry.file.id, binary, err
        ),
    })?;

    Ok(RuntimeSelectionDescriptor {
        agent_id: entry.file.id.clone(),
        backend_id: entry.derived_backend_id(),
        backend_kind,
        protocol: PURE_AGENT_PROTOCOL.to_string(),
        execution_scope: entry.effective_scope(effective_config),
        binary_path,
    })
}

pub(crate) fn runtime_realizability_error_exit_code(error: &RuntimeRealizabilityError) -> i32 {
    error.exit_code
}

#[allow(dead_code)]
fn _assert_result_type(_: Result<RuntimeSelectionDescriptor>) {}
