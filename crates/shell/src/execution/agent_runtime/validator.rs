use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use substrate_broker::Policy;

use crate::execution::agent_inventory::{AgentCapabilitiesV1, AgentInventoryEntryV1};
use crate::execution::agent_runtime::dispatch_contract::{
    resolve_inventory_contract_for_exact_backend, resolve_inventory_contract_for_unique_scope,
    AttachLaunchKnobs, AttachModePreference, DispatchBaselineKind, DispatchCallerKind,
    DispatchCapabilityOverrideSet, DispatchRequestEnvelope, HostExecutionClientStart,
    ResolvedLaunchContract,
};
use crate::execution::config_model::{AgentCliMode, AgentExecutionScope, SubstrateConfig};

use super::mapping::{
    orchestrator_backend_kind, protocol_validation_error, AgentRuntimeBackendKind,
    PURE_AGENT_PROTOCOL,
};

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

#[derive(Clone, Debug)]
pub(crate) struct ExactBackendSelectionError {
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
        return Err(protocol_validation_error(
            &format!("orchestrator agent '{orchestrator_agent_id}'"),
            entry.file.config.protocol.as_deref(),
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
    let contract = ResolvedLaunchContract {
        caller_kind: DispatchCallerKind::HumanStart,
        baseline_kind: DispatchBaselineKind::InventoryLaunch,
        agent_id: entry.file.id.clone(),
        backend_id: entry.derived_backend_id(),
        backend_kind: orchestrator_backend_kind(&entry.file.id).map_err(|err| {
            RuntimeRealizabilityError {
                exit_code: 2,
                reason: err
                    .to_string()
                    .replace("selected orchestrator backend", "selected runtime backend"),
            }
        })?,
        protocol: entry
            .file
            .config
            .protocol
            .clone()
            .unwrap_or_else(|| PURE_AGENT_PROTOCOL.to_string()),
        execution_scope: entry.effective_scope(effective_config),
        runtime: crate::execution::agent_runtime::dispatch_contract::ResolvedLaunchRuntime {
            kind: entry.file.config.kind,
            cli_mode: entry.effective_cli_mode(effective_config),
            cli_binary: entry.effective_cli_binary().map(ToOwned::to_owned),
        },
        capabilities: entry.file.config.capabilities.clone(),
        attach_launch_knobs: AttachLaunchKnobs {
            requested_execution_scope: entry.effective_scope(effective_config),
            host_execution_client_start: HostExecutionClientStart::StartNow,
            attach_mode_preference: AttachModePreference::ContinuityRequired,
        },
        effective_policy: Policy::default(),
        baseline_source: crate::execution::agent_runtime::dispatch_contract::BaselineSourceMetadata {
            baseline_kind: DispatchBaselineKind::InventoryLaunch,
            baseline_origin:
                crate::execution::agent_runtime::dispatch_contract::FieldBaselineOrigin::GlobalInventory,
            inventory_path: Some(entry.path.clone()),
            orchestration_session_id: None,
        },
        field_provenance: BTreeMap::new(),
    };

    materialize_runtime_descriptor(&contract)
}

pub(crate) fn materialize_runtime_descriptor(
    contract: &ResolvedLaunchContract,
) -> std::result::Result<RuntimeSelectionDescriptor, RuntimeRealizabilityError> {
    if contract.runtime.kind != crate::execution::agent_inventory::AgentConfigKind::Cli {
        return Err(RuntimeRealizabilityError {
            exit_code: 2,
            reason: format!(
                "selected runtime '{}' is not runtime-realizable by the shell-owned UAA runtime because config.kind={} is unsupported; only config.kind=cli is supported in v1",
                contract.agent_id,
                contract.runtime.kind.as_str()
            ),
        });
    }

    if contract.runtime.cli_mode != AgentCliMode::Persistent {
        return Err(RuntimeRealizabilityError {
            exit_code: 2,
            reason: format!(
                "selected runtime '{}' is not runtime-realizable because cli.mode={} is unsupported; only cli.mode=persistent is supported for the first caller path",
                contract.agent_id,
                match contract.runtime.cli_mode {
                    AgentCliMode::Persistent => "persistent",
                    AgentCliMode::PerRequest => "per_request",
                }
            ),
        });
    }

    let binary =
        contract
            .runtime
            .cli_binary
            .as_deref()
            .ok_or_else(|| RuntimeRealizabilityError {
                exit_code: 4,
                reason: format!(
            "selected runtime '{}' is not runtime-realizable because config.cli.binary is missing",
            contract.agent_id
        ),
            })?;
    let binary_path = which::which(binary).map_err(|err| RuntimeRealizabilityError {
        exit_code: 4,
        reason: format!(
            "selected runtime '{}' is not runtime-realizable because config.cli.binary '{}' did not resolve on the host: {}",
            contract.agent_id, binary, err
        ),
    })?;

    Ok(RuntimeSelectionDescriptor {
        agent_id: contract.agent_id.clone(),
        backend_id: contract.backend_id.clone(),
        backend_kind: contract.backend_kind,
        protocol: contract.protocol.clone(),
        execution_scope: contract.execution_scope,
        binary_path,
    })
}

pub(crate) fn validate_member_selection(
    effective_config: &SubstrateConfig,
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
) -> std::result::Result<Option<RuntimeSelectionDescriptor>, MemberSelectionError> {
    let envelope = inventory_dispatch_envelope(
        DispatchCallerKind::OrchestratorMemberStart,
        DispatchBaselineKind::InventoryLaunch,
        None,
        AgentExecutionScope::World,
    );
    let selection_policy = permissive_inventory_selection_policy(inventory);
    let resolved = resolve_inventory_contract_for_unique_scope(
        PathBuf::from(".").as_path(),
        effective_config,
        inventory,
        &selection_policy,
        &envelope,
        AgentExecutionScope::World,
    )
    .map_err(|err| MemberSelectionError {
        exit_code: 2,
        reason: err.reason,
    })?;

    resolved
        .as_ref()
        .map(materialize_runtime_descriptor)
        .transpose()
        .map_err(|err| MemberSelectionError {
            exit_code: err.exit_code,
            reason: err.reason,
        })
}

pub(crate) fn validate_exact_backend_selection(
    effective_config: &SubstrateConfig,
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
    scope: AgentExecutionScope,
    backend_id: &str,
) -> std::result::Result<Option<RuntimeSelectionDescriptor>, ExactBackendSelectionError> {
    let envelope = inventory_dispatch_envelope(
        if scope == AgentExecutionScope::Host {
            DispatchCallerKind::HumanStart
        } else {
            DispatchCallerKind::OrchestratorMemberStart
        },
        DispatchBaselineKind::InventoryLaunch,
        Some(backend_id),
        scope,
    );
    let selection_policy = permissive_inventory_selection_policy(inventory);
    let resolved = resolve_inventory_contract_for_exact_backend(
        PathBuf::from(".").as_path(),
        effective_config,
        inventory,
        &selection_policy,
        &envelope,
        scope,
    )
    .map_err(|err| ExactBackendSelectionError {
        exit_code: 2,
        reason: err.reason,
    })?;

    resolved
        .as_ref()
        .map(materialize_runtime_descriptor)
        .transpose()
        .map_err(|err| ExactBackendSelectionError {
            exit_code: err.exit_code,
            reason: err.reason,
        })
}

fn inventory_dispatch_envelope(
    caller_kind: DispatchCallerKind,
    baseline_kind: DispatchBaselineKind,
    backend_id: Option<&str>,
    scope: AgentExecutionScope,
) -> DispatchRequestEnvelope {
    DispatchRequestEnvelope {
        caller_kind,
        baseline_kind,
        backend_id: backend_id.map(ToOwned::to_owned),
        orchestration_session_id: None,
        requested_execution_scope_override: None,
        capability_overrides: DispatchCapabilityOverrideSet::default(),
        attach_launch_knobs: AttachLaunchKnobs {
            requested_execution_scope: scope,
            host_execution_client_start: if scope == AgentExecutionScope::Host {
                HostExecutionClientStart::StartNow
            } else {
                HostExecutionClientStart::Defer
            },
            attach_mode_preference: AttachModePreference::ContinuityRequired,
        },
        has_prompt_payload: false,
    }
}

fn permissive_inventory_selection_policy(
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
) -> Policy {
    Policy {
        agents_allowed_backends: inventory
            .values()
            .map(AgentInventoryEntryV1::derived_backend_id)
            .collect(),
        ..Policy::default()
    }
}

pub(crate) fn runtime_realizability_error_exit_code(error: &RuntimeRealizabilityError) -> i32 {
    error.exit_code
}

pub(crate) fn member_selection_error_exit_code(error: &MemberSelectionError) -> i32 {
    error.exit_code
}

pub(crate) fn exact_backend_selection_error_exit_code(error: &ExactBackendSelectionError) -> i32 {
    error.exit_code
}

#[allow(dead_code)]
fn _assert_result_type(_: Result<RuntimeSelectionDescriptor>) {}

#[cfg(test)]
mod tests {
    use super::{
        exact_backend_selection_error_exit_code, validate_exact_backend_selection,
        validate_member_selection, validate_runtime_realizability, AgentRuntimeBackendKind,
        ExactBackendSelectionError, MemberSelectionError, RuntimeSelectionDescriptor,
        PURE_AGENT_PROTOCOL,
    };
    use crate::execution::agent_inventory::{
        AgentCapabilitiesV1, AgentCliConfigV1, AgentConfigKind, AgentConfigV1,
        AgentExecutionConfigV1, AgentFileV1, AgentInventoryEntryV1,
    };
    use crate::execution::agent_runtime::mapping::LEGACY_PURE_AGENT_PROTOCOL;
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
        let test_binary = std::env::current_exe()
            .expect("current test binary should resolve")
            .display()
            .to_string();
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
                        binary: test_binary,
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

    fn assert_exact_selected_descriptor(
        result: std::result::Result<Option<RuntimeSelectionDescriptor>, ExactBackendSelectionError>,
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
                .contains("does not advertise protocol 'substrate.agent.session'"),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    fn validate_member_selection_rejects_legacy_protocol_with_rename_guidance() {
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                "codex",
                AgentExecutionScope::World,
                Some(LEGACY_PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );

        let error = validate_member_selection(&config, &inventory).expect_err("must fail closed");
        assert_eq!(error.exit_code, 2);
        assert!(
            error.reason.contains(LEGACY_PURE_AGENT_PROTOCOL)
                && error.reason.contains(PURE_AGENT_PROTOCOL),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    fn validate_exact_backend_selection_returns_none_when_backend_is_missing() {
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

        let selected = validate_exact_backend_selection(
            &config,
            &inventory,
            AgentExecutionScope::World,
            "cli:claude-code",
        )
        .expect("selection should not fail");
        assert!(selected.is_none());
    }

    #[test]
    fn validate_exact_backend_selection_bypasses_world_ambiguity_when_backend_matches_exactly() {
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

        let descriptor = assert_exact_selected_descriptor(validate_exact_backend_selection(
            &config,
            &inventory,
            AgentExecutionScope::World,
            "cli:codex",
        ));
        assert_eq!(descriptor.agent_id, "codex");
        assert_eq!(descriptor.backend_id, "cli:codex");
    }

    #[test]
    fn validate_exact_backend_selection_reports_scope_specific_protocol_error() {
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "claude_code".to_string(),
            make_entry(
                "claude_code",
                AgentExecutionScope::Host,
                Some("other.protocol"),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );

        let error = validate_exact_backend_selection(
            &config,
            &inventory,
            AgentExecutionScope::Host,
            "cli:claude_code",
        )
        .expect_err("must fail closed");
        assert_eq!(exact_backend_selection_error_exit_code(&error), 2);
        assert!(
            error
                .reason
                .contains("selected host-scoped runtime 'claude_code' for backend 'cli:claude_code' does not advertise protocol 'substrate.agent.session'"),
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
