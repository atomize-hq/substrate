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

#[derive(Clone, Debug)]
pub(crate) struct MemberSelectionError {
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
                "selected runtime '{}' is not runtime-realizable by the shell-owned UAA runtime because config.kind={} is unsupported; only config.kind=cli is supported in v1",
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
                "selected runtime '{}' is not runtime-realizable because cli.mode={} is unsupported; only cli.mode=persistent is supported for the first caller path",
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
            reason: err
                .to_string()
                .replace("selected orchestrator backend", "selected runtime backend"),
        })?;

    let binary = entry
        .effective_cli_binary()
        .ok_or_else(|| RuntimeRealizabilityError {
            exit_code: 4,
            reason: format!(
            "selected runtime '{}' is not runtime-realizable because config.cli.binary is missing",
            entry.file.id
        ),
        })?;
    let binary_path = which::which(binary).map_err(|err| RuntimeRealizabilityError {
        exit_code: 4,
        reason: format!(
            "selected runtime '{}' is not runtime-realizable because config.cli.binary '{}' did not resolve on the host: {}",
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

pub(crate) fn validate_member_selection(
    effective_config: &SubstrateConfig,
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
) -> std::result::Result<Option<RuntimeSelectionDescriptor>, MemberSelectionError> {
    let mut selected = Vec::new();

    for entry in inventory.values() {
        if !entry.file.config.enabled
            || entry.effective_scope(effective_config) != AgentExecutionScope::World
        {
            continue;
        }

        if entry.file.config.protocol.as_deref() != Some(PURE_AGENT_PROTOCOL) {
            return Err(MemberSelectionError {
                exit_code: 2,
                reason: format!(
                    "world-scoped member '{}' is not eligible because it does not advertise protocol '{}'",
                    entry.file.id, PURE_AGENT_PROTOCOL
                ),
            });
        }

        if let Some(capability) =
            missing_required_orchestrator_capability(&entry.file.config.capabilities)
        {
            return Err(MemberSelectionError {
                exit_code: 2,
                reason: format!(
                    "world-scoped member '{}' is not eligible because it is missing required capability '{}'",
                    entry.file.id, capability
                ),
            });
        }

        let descriptor =
            validate_runtime_realizability(entry, effective_config).map_err(|err| {
                MemberSelectionError {
                    exit_code: err.exit_code,
                    reason: err.reason,
                }
            })?;
        selected.push(descriptor);
    }

    match selected.len() {
        0 => Ok(None),
        1 => Ok(selected.into_iter().next()),
        _ => {
            let agent_ids = selected
                .iter()
                .map(|descriptor| descriptor.agent_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(MemberSelectionError {
                exit_code: 2,
                reason: format!(
                    "ambiguous world member selection: multiple eligible world-scoped members found ({agent_ids})"
                ),
            })
        }
    }
}

pub(crate) fn runtime_realizability_error_exit_code(error: &RuntimeRealizabilityError) -> i32 {
    error.exit_code
}

pub(crate) fn member_selection_error_exit_code(error: &MemberSelectionError) -> i32 {
    error.exit_code
}

#[allow(dead_code)]
fn _assert_result_type(_: Result<RuntimeSelectionDescriptor>) {}

#[cfg(test)]
mod tests {
    use super::{
        validate_member_selection, validate_runtime_realizability, AgentRuntimeBackendKind,
        MemberSelectionError, RuntimeSelectionDescriptor, PURE_AGENT_PROTOCOL,
    };
    use crate::execution::agent_inventory::{
        AgentCapabilitiesV1, AgentCliConfigV1, AgentConfigKind, AgentConfigV1,
        AgentExecutionConfigV1, AgentFileV1, AgentInventoryEntryV1,
    };
    use crate::execution::config_model::{AgentCliMode, AgentExecutionScope, SubstrateConfig};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn make_entry(
        agent_id: &str,
        scope: AgentExecutionScope,
        protocol: Option<&str>,
        cli_mode: AgentCliMode,
        capabilities: AgentCapabilitiesV1,
    ) -> AgentInventoryEntryV1 {
        AgentInventoryEntryV1 {
            path: PathBuf::from(format!("{agent_id}.yaml")),
            file: AgentFileV1 {
                version: 1,
                id: agent_id.to_string(),
                config: AgentConfigV1 {
                    enabled: true,
                    kind: AgentConfigKind::Cli,
                    protocol: protocol.map(str::to_string),
                    execution: AgentExecutionConfigV1 { scope: Some(scope) },
                    cli: Some(AgentCliConfigV1 {
                        binary: "cargo".to_string(),
                        mode: Some(cli_mode),
                    }),
                    api: None,
                    capabilities,
                },
                policy_overlay: None,
            },
        }
    }

    fn required_capabilities() -> AgentCapabilitiesV1 {
        AgentCapabilitiesV1 {
            session_start: true,
            session_resume: true,
            session_fork: true,
            session_stop: true,
            status_snapshot: true,
            event_stream: true,
            llm: true,
            mcp_client: false,
        }
    }

    fn assert_selected_descriptor(
        result: std::result::Result<Option<RuntimeSelectionDescriptor>, MemberSelectionError>,
    ) -> RuntimeSelectionDescriptor {
        result
            .expect("selection should succeed")
            .expect("descriptor")
    }

    #[test]
    fn validate_member_selection_returns_none_when_no_world_members_exist() {
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "claude_code".to_string(),
            make_entry(
                "claude_code",
                AgentExecutionScope::Host,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );

        assert!(validate_member_selection(&config, &inventory)
            .expect("selection should not fail")
            .is_none());
    }

    #[test]
    fn validate_member_selection_returns_descriptor_for_unique_world_member() {
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                "codex",
                AgentExecutionScope::World,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );

        let descriptor = assert_selected_descriptor(validate_member_selection(&config, &inventory));
        assert_eq!(descriptor.agent_id, "codex");
        assert_eq!(descriptor.backend_id, "cli:codex");
        assert_eq!(descriptor.backend_kind, AgentRuntimeBackendKind::Codex);
        assert_eq!(descriptor.execution_scope, AgentExecutionScope::World);
    }

    #[test]
    fn validate_member_selection_fails_closed_on_ambiguity() {
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                "codex",
                AgentExecutionScope::World,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );
        inventory.insert(
            "claude_code".to_string(),
            make_entry(
                "claude_code",
                AgentExecutionScope::World,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );

        let error = validate_member_selection(&config, &inventory).expect_err("must fail closed");
        assert_eq!(error.exit_code, 2);
        assert!(
            error.reason.contains("ambiguous world member selection"),
            "unexpected reason: {}",
            error.reason
        );
        assert!(
            error.reason.contains("claude_code, codex"),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    fn validate_member_selection_fails_for_wrong_protocol() {
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                "codex",
                AgentExecutionScope::World,
                Some("other.protocol"),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );

        let error = validate_member_selection(&config, &inventory).expect_err("must fail closed");
        assert_eq!(error.exit_code, 2);
        assert!(
            error
                .reason
                .contains("does not advertise protocol 'uaa.agent.session'"),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    fn validate_runtime_realizability_uses_generic_runtime_wording() {
        let config = SubstrateConfig::default();
        let entry = make_entry(
            "claude_code",
            AgentExecutionScope::Host,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::PerRequest,
            required_capabilities(),
        );

        let error = validate_runtime_realizability(&entry, &config).expect_err("must fail");
        assert!(
            error.reason.starts_with("selected runtime 'claude_code'"),
            "unexpected reason: {}",
            error.reason
        );
    }
}
