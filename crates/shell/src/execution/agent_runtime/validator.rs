use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use substrate_broker::Policy;

use crate::execution::agent_inventory::{
    AgentCapabilitiesV1, AgentConfigKind, AgentInventoryEntryV1,
};
use crate::execution::agent_runtime::dispatch_contract::{
    resolve_inventory_contract_for_exact_backend, resolve_inventory_contract_for_unique_scope,
    AttachLaunchKnobs, AttachModePreference, DispatchBaselineKind, DispatchCallerKind,
    DispatchCapabilityOverrideSet, DispatchRequestEnvelope, HostExecutionClientStart,
    LiveToolSupportPosture, ResolvedLaunchContract,
};
use crate::execution::config_model::{AgentCliMode, AgentExecutionScope, SubstrateConfig};

use super::mapping::{
    protocol_validation_error, resolve_shell_owned_runtime_family, AgentRuntimeBackendKind,
    PURE_AGENT_PROTOCOL,
};

pub(crate) const CODEX_WORLD_GUEST_ENTRYPOINT: &str = "/var/lib/substrate/world-deps/bin/codex";

#[derive(Clone, Debug)]
pub(crate) struct RuntimeSelectionDescriptor {
    pub agent_id: String,
    pub backend_id: String,
    pub backend_kind: AgentRuntimeBackendKind,
    pub protocol: String,
    pub execution_scope: AgentExecutionScope,
    pub binary_path: PathBuf,
}

impl RuntimeSelectionDescriptor {
    pub(crate) fn live_tool_support_posture(&self) -> LiveToolSupportPosture {
        LiveToolSupportPosture::for_selected_launch_backend_kind(
            self.backend_kind,
            self.execution_scope,
        )
    }
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

    let protocol = entry
        .file
        .config
        .protocol
        .clone()
        .unwrap_or_else(|| PURE_AGENT_PROTOCOL.to_string());
    let backend_kind =
        resolve_shell_owned_runtime_family(&entry.file.id, entry.cli_runtime_family()).map_err(
            |err| RuntimeRealizabilityError {
                exit_code: 2,
                reason: err.to_string(),
            },
        )?;
    let binary = entry
        .effective_cli_binary()
        .ok_or_else(|| RuntimeRealizabilityError {
            exit_code: 4,
            reason: format!(
                "selected runtime '{}' is not runtime-realizable because config.cli.binary is missing",
                entry.file.id
            ),
        })?;
    let execution_scope = entry.effective_scope(effective_config);
    let binary_path =
        resolve_runtime_binary_path(&entry.file.id, backend_kind, execution_scope, binary)?;

    Ok(RuntimeSelectionDescriptor {
        agent_id: entry.file.id.clone(),
        backend_id: entry.derived_backend_id(),
        backend_kind,
        protocol,
        execution_scope,
        binary_path,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn resolve_live_tool_support_posture(
    entry: &AgentInventoryEntryV1,
) -> std::result::Result<LiveToolSupportPosture, RuntimeRealizabilityError> {
    let backend_kind =
        resolve_shell_owned_runtime_family(&entry.file.id, entry.cli_runtime_family()).map_err(
            |err| RuntimeRealizabilityError {
                exit_code: 2,
                reason: err.to_string(),
            },
        )?;

    Ok(LiveToolSupportPosture::for_backend_kind(backend_kind))
}

pub(crate) fn resolve_selected_orchestrator_live_tool_support_posture(
    entry: &AgentInventoryEntryV1,
) -> std::result::Result<LiveToolSupportPosture, RuntimeRealizabilityError> {
    let backend_kind =
        resolve_shell_owned_runtime_family(&entry.file.id, entry.cli_runtime_family()).map_err(
            |err| RuntimeRealizabilityError {
                exit_code: 2,
                reason: err.to_string(),
            },
        )?;

    Ok(LiveToolSupportPosture::for_selected_launch_backend_kind(
        backend_kind,
        AgentExecutionScope::Host,
    ))
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
    let binary_path = resolve_runtime_binary_path(
        &contract.agent_id,
        contract.backend_kind,
        contract.execution_scope,
        binary,
    )?;

    Ok(RuntimeSelectionDescriptor {
        agent_id: contract.agent_id.clone(),
        backend_id: contract.backend_id.clone(),
        backend_kind: contract.backend_kind,
        protocol: contract.protocol.clone(),
        execution_scope: contract.execution_scope,
        binary_path,
    })
}

fn resolve_runtime_binary_path(
    agent_id: &str,
    backend_kind: AgentRuntimeBackendKind,
    execution_scope: AgentExecutionScope,
    configured_binary: &str,
) -> std::result::Result<PathBuf, RuntimeRealizabilityError> {
    match (backend_kind, execution_scope) {
        (AgentRuntimeBackendKind::Codex, AgentExecutionScope::World) => {
            resolve_world_scoped_codex_binary_path(agent_id, configured_binary)
        }
        _ => resolve_host_runtime_binary_path(agent_id, configured_binary),
    }
}

fn resolve_host_runtime_binary_path(
    agent_id: &str,
    configured_binary: &str,
) -> std::result::Result<PathBuf, RuntimeRealizabilityError> {
    which::which(configured_binary).map_err(|err| RuntimeRealizabilityError {
        exit_code: 4,
        reason: format!(
            "selected runtime '{}' is not runtime-realizable because config.cli.binary '{}' did not resolve on the host: {}",
            agent_id, configured_binary, err
        ),
    })
}

fn resolve_world_scoped_codex_binary_path(
    agent_id: &str,
    configured_binary: &str,
) -> std::result::Result<PathBuf, RuntimeRealizabilityError> {
    if configured_binary != CODEX_WORLD_GUEST_ENTRYPOINT {
        return Err(RuntimeRealizabilityError {
            exit_code: 4,
            reason: format!(
                "selected runtime '{}' is not runtime-realizable in world scope because guest entrypoint '{}' is required for the world-scoped Codex runtime and config.cli.binary '{}' still describes host-local truth; install the world runtime and rerun 'substrate world deps current sync'",
                agent_id,
                CODEX_WORLD_GUEST_ENTRYPOINT,
                configured_binary,
            ),
        });
    }

    let probe_path = world_scoped_codex_guest_entrypoint_probe_path();
    if !guest_entrypoint_is_available(probe_path.as_path()) {
        return Err(RuntimeRealizabilityError {
            exit_code: 4,
            reason: format!(
                "selected runtime '{}' is not runtime-realizable in world scope because guest entrypoint '{}' is unavailable; install the world runtime and rerun 'substrate world deps current sync'",
                agent_id,
                CODEX_WORLD_GUEST_ENTRYPOINT,
            ),
        });
    }

    Ok(PathBuf::from(CODEX_WORLD_GUEST_ENTRYPOINT))
}

fn world_scoped_codex_guest_entrypoint_probe_path() -> PathBuf {
    const DEFAULT_WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    let world_deps_bin = std::env::var("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_WORLD_DEPS_BIN.to_string());
    Path::new(&world_deps_bin).join("codex")
}

fn guest_entrypoint_is_available(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
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
        exact_backend_selection_error_exit_code, resolve_live_tool_support_posture,
        resolve_selected_orchestrator_live_tool_support_posture, validate_exact_backend_selection,
        validate_member_selection, validate_runtime_realizability, AgentRuntimeBackendKind,
        ExactBackendSelectionError, MemberSelectionError, RuntimeSelectionDescriptor,
        CODEX_WORLD_GUEST_ENTRYPOINT, PURE_AGENT_PROTOCOL,
    };
    use crate::execution::agent_inventory::{
        AgentCapabilitiesV1, AgentCliConfigV1, AgentCliRuntimeFamily, AgentConfigKind,
        AgentConfigV1, AgentExecutionConfigV1, AgentFileV1, AgentInventoryEntryV1,
    };
    use crate::execution::agent_runtime::dispatch_contract::{
        LiveToolSupportState, LiveToolValidationState, SelectedClaudeCodeUpliftContext,
    };
    use crate::execution::agent_runtime::mapping::LEGACY_PURE_AGENT_PROTOCOL;
    use crate::execution::config_model::{AgentCliMode, AgentExecutionScope, SubstrateConfig};
    use serial_test::serial;
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use tempfile::TempDir;

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set_path(key: &'static str, value: &std::path::Path) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn test_world_codex_runtime_bin() -> &'static PathBuf {
        static TEST_WORLD_DEPS_BIN: OnceLock<PathBuf> = OnceLock::new();
        TEST_WORLD_DEPS_BIN.get_or_init(|| {
            let temp = TempDir::new().expect("tempdir for world codex runtime");
            let bin_dir = temp.path().to_path_buf();
            let codex = bin_dir.join("codex");
            fs::write(&codex, "#!/bin/sh\nexit 0\n").expect("write fake guest codex");
            #[cfg(unix)]
            {
                let mut perms = fs::metadata(&codex)
                    .expect("guest codex metadata")
                    .permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&codex, perms).expect("guest codex permissions");
            }
            let leaked = Box::leak(Box::new(temp));
            leaked.path().to_path_buf()
        })
    }

    fn set_test_world_codex_runtime() -> EnvVarGuard {
        EnvVarGuard::set_path(
            "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR",
            test_world_codex_runtime_bin().as_path(),
        )
    }

    fn make_entry(
        agent_id: &str,
        scope: AgentExecutionScope,
        protocol: Option<&str>,
        cli_mode: AgentCliMode,
        capabilities: AgentCapabilitiesV1,
    ) -> AgentInventoryEntryV1 {
        let runtime_family = match agent_id {
            "claude_code" => Some(AgentCliRuntimeFamily::ClaudeCode),
            _ => Some(AgentCliRuntimeFamily::Codex),
        };
        make_entry_with_runtime_family(
            agent_id,
            scope,
            protocol,
            cli_mode,
            runtime_family,
            capabilities,
        )
    }

    fn make_entry_with_runtime_family(
        agent_id: &str,
        scope: AgentExecutionScope,
        protocol: Option<&str>,
        cli_mode: AgentCliMode,
        runtime_family: Option<AgentCliRuntimeFamily>,
        capabilities: AgentCapabilitiesV1,
    ) -> AgentInventoryEntryV1 {
        let test_binary = test_binary_for(scope, runtime_family);
        make_entry_with_runtime_family_and_binary(
            agent_id,
            scope,
            protocol,
            cli_mode,
            runtime_family,
            &test_binary,
            capabilities,
        )
    }

    fn make_entry_with_runtime_family_and_binary(
        agent_id: &str,
        scope: AgentExecutionScope,
        protocol: Option<&str>,
        cli_mode: AgentCliMode,
        runtime_family: Option<AgentCliRuntimeFamily>,
        binary: &str,
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
                        binary: binary.to_string(),
                        mode: Some(cli_mode),
                        runtime_family,
                    }),
                    api: None,
                    capabilities,
                },
                policy_overlay: None,
            },
        }
    }

    fn test_binary_for(
        scope: AgentExecutionScope,
        runtime_family: Option<AgentCliRuntimeFamily>,
    ) -> String {
        match (scope, runtime_family) {
            (AgentExecutionScope::World, Some(AgentCliRuntimeFamily::Codex)) => {
                CODEX_WORLD_GUEST_ENTRYPOINT.to_string()
            }
            _ => std::env::current_exe()
                .expect("current test binary should resolve")
                .display()
                .to_string(),
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
    #[serial]
    fn validate_member_selection_returns_descriptor_for_unique_world_member() {
        let _env_guard = crate::execution::world_env_guard();
        let _world_codex_guard = set_test_world_codex_runtime();
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
    #[serial]
    fn validate_member_selection_resolves_alias_backend_to_canonical_runtime_family() {
        let _env_guard = crate::execution::world_env_guard();
        let _world_codex_guard = set_test_world_codex_runtime();
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex_world".to_string(),
            make_entry_with_runtime_family(
                "codex_world",
                AgentExecutionScope::World,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                Some(AgentCliRuntimeFamily::Codex),
                required_capabilities(),
            ),
        );

        let descriptor = assert_selected_descriptor(validate_member_selection(&config, &inventory));
        assert_eq!(descriptor.agent_id, "codex_world");
        assert_eq!(descriptor.backend_id, "cli:codex_world");
        assert_eq!(descriptor.backend_kind, AgentRuntimeBackendKind::Codex);
    }

    #[test]
    #[serial]
    fn validate_member_selection_prefers_world_alias_while_host_codex_remains_distinct() {
        let _env_guard = crate::execution::world_env_guard();
        let _world_codex_guard = set_test_world_codex_runtime();
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                "codex",
                AgentExecutionScope::Host,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );
        inventory.insert(
            "codex_world".to_string(),
            make_entry_with_runtime_family(
                "codex_world",
                AgentExecutionScope::World,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                Some(AgentCliRuntimeFamily::Codex),
                required_capabilities(),
            ),
        );

        let descriptor = assert_selected_descriptor(validate_member_selection(&config, &inventory));
        assert_eq!(descriptor.agent_id, "codex_world");
        assert_eq!(descriptor.backend_id, "cli:codex_world");
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
    #[serial]
    fn validate_exact_backend_selection_bypasses_world_ambiguity_when_backend_matches_exactly() {
        let _env_guard = crate::execution::world_env_guard();
        let _world_codex_guard = set_test_world_codex_runtime();
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
    #[serial]
    fn validate_exact_backend_selection_preserves_codex_world_alias_identity() {
        let _env_guard = crate::execution::world_env_guard();
        let _world_codex_guard = set_test_world_codex_runtime();
        let config = SubstrateConfig::default();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                "codex",
                AgentExecutionScope::Host,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                required_capabilities(),
            ),
        );
        inventory.insert(
            "codex_world".to_string(),
            make_entry_with_runtime_family(
                "codex_world",
                AgentExecutionScope::World,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                Some(AgentCliRuntimeFamily::Codex),
                required_capabilities(),
            ),
        );

        let descriptor = assert_exact_selected_descriptor(validate_exact_backend_selection(
            &config,
            &inventory,
            AgentExecutionScope::World,
            "cli:codex_world",
        ));
        assert_eq!(descriptor.agent_id, "codex_world");
        assert_eq!(descriptor.backend_id, "cli:codex_world");
        assert_eq!(descriptor.backend_kind, AgentRuntimeBackendKind::Codex);
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

    #[test]
    fn validate_runtime_realizability_requires_runtime_family_for_shell_owned_path() {
        let config = SubstrateConfig::default();
        let entry = make_entry_with_runtime_family(
            "codex_world",
            AgentExecutionScope::Host,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::Persistent,
            None,
            required_capabilities(),
        );

        let error = validate_runtime_realizability(&entry, &config).expect_err("must fail");
        assert!(
            error
                .reason
                .contains("config.cli.runtime_family is required"),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    fn validate_runtime_realizability_keeps_cli_mode_failure_ahead_of_runtime_family_checks() {
        let config = SubstrateConfig::default();
        let entry = make_entry_with_runtime_family(
            "codex_world",
            AgentExecutionScope::Host,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::PerRequest,
            None,
            required_capabilities(),
        );

        let error = validate_runtime_realizability(&entry, &config).expect_err("must fail");
        assert!(
            error.reason.contains("cli.mode=per_request"),
            "unexpected reason: {}",
            error.reason
        );
        assert!(
            !error.reason.contains("runtime_family"),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    fn validate_runtime_realizability_rejects_world_scoped_codex_host_truth_with_remediation() {
        let config = SubstrateConfig::default();
        let host_binary = std::env::current_exe()
            .expect("current test binary should resolve")
            .display()
            .to_string();
        let entry = make_entry_with_runtime_family_and_binary(
            "codex_world",
            AgentExecutionScope::World,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::Persistent,
            Some(AgentCliRuntimeFamily::Codex),
            &host_binary,
            required_capabilities(),
        );

        let error = validate_runtime_realizability(&entry, &config).expect_err("must fail");
        assert_eq!(error.exit_code, 4);
        assert!(
            error
                .reason
                .contains("not runtime-realizable in world scope"),
            "unexpected reason: {}",
            error.reason
        );
        assert!(
            error.reason.contains(CODEX_WORLD_GUEST_ENTRYPOINT),
            "unexpected reason: {}",
            error.reason
        );
        assert!(
            error.reason.contains("substrate world deps current sync"),
            "unexpected reason: {}",
            error.reason
        );
        assert!(
            error.reason.contains("host-local truth"),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    #[serial]
    fn validate_runtime_realizability_accepts_world_scoped_codex_guest_entrypoint_contract() {
        let _env_guard = crate::execution::world_env_guard();
        let _world_codex_guard = set_test_world_codex_runtime();
        let config = SubstrateConfig::default();
        let entry = make_entry_with_runtime_family(
            "codex_world",
            AgentExecutionScope::World,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::Persistent,
            Some(AgentCliRuntimeFamily::Codex),
            required_capabilities(),
        );

        let descriptor =
            validate_runtime_realizability(&entry, &config).expect("world codex should resolve");
        assert_eq!(
            descriptor.binary_path,
            PathBuf::from(CODEX_WORLD_GUEST_ENTRYPOINT)
        );
    }

    #[test]
    #[serial]
    fn validate_runtime_realizability_rejects_world_scoped_codex_when_guest_entrypoint_is_absent() {
        let _env_guard = crate::execution::world_env_guard();
        let config = SubstrateConfig::default();
        let temp = TempDir::new().expect("tempdir for absent guest entrypoint");
        let _world_codex_guard =
            EnvVarGuard::set_path("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", temp.path());

        let entry = make_entry_with_runtime_family_and_binary(
            "codex_world",
            AgentExecutionScope::World,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::Persistent,
            Some(AgentCliRuntimeFamily::Codex),
            CODEX_WORLD_GUEST_ENTRYPOINT,
            required_capabilities(),
        );

        let error = validate_runtime_realizability(&entry, &config).expect_err("must fail");
        assert_eq!(error.exit_code, 4);
        assert!(
            error.reason.contains("guest entrypoint"),
            "unexpected reason: {}",
            error.reason
        );
        assert!(
            error.reason.contains("is unavailable"),
            "unexpected reason: {}",
            error.reason
        );
        assert!(
            error.reason.contains("substrate world deps current sync"),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    fn validate_exact_backend_selection_fails_closed_when_world_codex_keeps_host_binary_truth() {
        let config = SubstrateConfig::default();
        let host_binary = std::env::current_exe()
            .expect("current test binary should resolve")
            .display()
            .to_string();
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex_world".to_string(),
            make_entry_with_runtime_family_and_binary(
                "codex_world",
                AgentExecutionScope::World,
                Some(PURE_AGENT_PROTOCOL),
                AgentCliMode::Persistent,
                Some(AgentCliRuntimeFamily::Codex),
                &host_binary,
                required_capabilities(),
            ),
        );

        let error = validate_exact_backend_selection(
            &config,
            &inventory,
            AgentExecutionScope::World,
            "cli:codex_world",
        )
        .expect_err("must fail closed");
        assert_eq!(exact_backend_selection_error_exit_code(&error), 4);
        assert!(
            error.reason.contains(CODEX_WORLD_GUEST_ENTRYPOINT),
            "unexpected reason: {}",
            error.reason
        );
        assert!(
            error.reason.contains("substrate world deps current sync"),
            "unexpected reason: {}",
            error.reason
        );
    }

    #[test]
    fn resolve_live_tool_support_posture_uses_runtime_family_not_agent_id() {
        let entry = make_entry_with_runtime_family(
            "workspace_orchestrator_alias",
            AgentExecutionScope::Host,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::Persistent,
            Some(AgentCliRuntimeFamily::Codex),
            required_capabilities(),
        );

        let posture = resolve_live_tool_support_posture(&entry).expect("posture should resolve");
        assert_eq!(posture.runtime_family, AgentRuntimeBackendKind::Codex);
        assert_eq!(
            posture.validation_state,
            LiveToolValidationState::SmokeValidated
        );
        assert_eq!(
            posture.support_state,
            LiveToolSupportState::FirstSupportedFloor
        );
        assert_eq!(
            posture.selected_claude_code_uplift_context,
            SelectedClaudeCodeUpliftContext::inventory_entry()
        );
    }

    #[test]
    fn resolve_live_tool_support_posture_keeps_non_codex_host_sessions_supported_but_unvalidated() {
        let entry = make_entry_with_runtime_family(
            "host_orchestrator_alias",
            AgentExecutionScope::Host,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::Persistent,
            Some(AgentCliRuntimeFamily::ClaudeCode),
            required_capabilities(),
        );

        let posture = resolve_live_tool_support_posture(&entry).expect("posture should resolve");
        assert_eq!(posture.runtime_family, AgentRuntimeBackendKind::ClaudeCode);
        assert_eq!(
            posture.validation_state,
            LiveToolValidationState::NotYetSmokeValidated
        );
        assert_eq!(
            posture.support_state,
            LiveToolSupportState::NotYetGuaranteed
        );
        assert_eq!(
            posture.selected_claude_code_uplift_context,
            SelectedClaudeCodeUpliftContext::inventory_entry()
        );
        assert!(
            posture
                .reason
                .contains("ordinary host-session behavior remains unchanged"),
            "unexpected reason: {}",
            posture.reason
        );
    }

    #[test]
    fn selected_orchestrator_live_tool_support_posture_uses_selected_host_context() {
        let entry = make_entry_with_runtime_family(
            "host_orchestrator_alias",
            AgentExecutionScope::Host,
            Some(PURE_AGENT_PROTOCOL),
            AgentCliMode::Persistent,
            Some(AgentCliRuntimeFamily::ClaudeCode),
            required_capabilities(),
        );

        let posture = resolve_selected_orchestrator_live_tool_support_posture(&entry)
            .expect("posture should resolve");
        assert_eq!(posture.runtime_family, AgentRuntimeBackendKind::ClaudeCode);
        assert_eq!(
            posture.selected_claude_code_uplift_context,
            SelectedClaudeCodeUpliftContext::selected_launch(AgentExecutionScope::Host)
        );
        assert_eq!(
            posture.validation_state,
            LiveToolValidationState::SmokeValidated
        );
        assert_eq!(
            posture.support_state,
            LiveToolSupportState::SelectedRuntimeSupported
        );
    }

    #[test]
    fn runtime_selection_descriptor_live_tool_support_posture_tracks_selected_scope_context() {
        let descriptor = RuntimeSelectionDescriptor {
            agent_id: "claude_code".to_string(),
            backend_id: "cli:claude_code".to_string(),
            backend_kind: AgentRuntimeBackendKind::ClaudeCode,
            protocol: PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: AgentExecutionScope::World,
            binary_path: PathBuf::from("/bin/claude"),
        };

        let posture = descriptor.live_tool_support_posture();
        assert_eq!(
            posture.selected_claude_code_uplift_context,
            SelectedClaudeCodeUpliftContext::selected_launch(AgentExecutionScope::World)
        );
        assert_eq!(
            posture.support_state,
            LiveToolSupportState::NotYetGuaranteed
        );
    }
}
