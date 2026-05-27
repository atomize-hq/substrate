use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use substrate_broker::{validate_backend_id, Policy};

use crate::execution::agent_inventory::{
    project_inventory_entry, AgentCapabilitiesV1, AgentConfigKind, AgentInventoryBaselineOrigin,
    AgentInventoryEntryV1, ProjectedInventoryEntryV1, ProjectedInventoryValueOrigin,
};
use crate::execution::agent_runtime::orchestration_session::{
    HostAttachContract, HostAttachExecutionClientStart, HostAttachModePreference,
};
use crate::execution::config_model::{AgentCliMode, AgentExecutionScope, SubstrateConfig};
use crate::execution::policy_model::{apply_policy_patch, PolicyPatch};

use super::mapping::{
    orchestrator_backend_kind, protocol_validation_error, AgentRuntimeBackendKind,
    PURE_AGENT_PROTOCOL,
};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DispatchCallerKind {
    HumanStart,
    HumanTurn,
    HumanReattach,
    HumanFork,
    OrchestratorMemberStart,
    OrchestratorMemberTurn,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DispatchBaselineKind {
    InventoryLaunch,
    PersistedHostAttach,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldBaselineOrigin {
    GlobalInventory,
    WorkspaceInventory,
    PersistedHostAttachContract,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldValueOrigin {
    InventoryExplicit,
    EffectiveConfigDefault,
    DispatchOverrideAccepted,
    DispatchOverrideNarrowedByPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FieldProvenance {
    pub baseline_origin: FieldBaselineOrigin,
    pub value_origin: FieldValueOrigin,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DispatchCapabilityOverrideSet {
    pub session_start: Option<bool>,
    pub session_resume: Option<bool>,
    pub session_fork: Option<bool>,
    pub session_stop: Option<bool>,
    pub status_snapshot: Option<bool>,
    pub event_stream: Option<bool>,
    pub llm: Option<bool>,
    pub mcp_client: Option<bool>,
}

impl DispatchCapabilityOverrideSet {
    pub(crate) fn is_empty(&self) -> bool {
        [
            self.session_start,
            self.session_resume,
            self.session_fork,
            self.session_stop,
            self.status_snapshot,
            self.event_stream,
            self.llm,
            self.mcp_client,
        ]
        .into_iter()
        .all(|value| value.is_none())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostExecutionClientStart {
    StartNow,
    Defer,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AttachModePreference {
    ContinuityRequired,
    ContinuityPreferred,
    FreshAllowed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AttachLaunchKnobs {
    pub requested_execution_scope: AgentExecutionScope,
    pub host_execution_client_start: HostExecutionClientStart,
    pub attach_mode_preference: AttachModePreference,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DispatchRequestEnvelope {
    pub caller_kind: DispatchCallerKind,
    pub baseline_kind: DispatchBaselineKind,
    pub backend_id: Option<String>,
    pub orchestration_session_id: Option<String>,
    pub requested_execution_scope_override: Option<AgentExecutionScope>,
    pub capability_overrides: DispatchCapabilityOverrideSet,
    pub attach_launch_knobs: AttachLaunchKnobs,
    pub has_prompt_payload: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedLaunchRuntime {
    pub kind: AgentConfigKind,
    pub cli_mode: AgentCliMode,
    pub cli_binary: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BaselineSourceMetadata {
    pub baseline_kind: DispatchBaselineKind,
    pub baseline_origin: FieldBaselineOrigin,
    pub inventory_path: Option<PathBuf>,
    pub orchestration_session_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct ResolvedLaunchContract {
    pub caller_kind: DispatchCallerKind,
    pub baseline_kind: DispatchBaselineKind,
    pub agent_id: String,
    pub backend_id: String,
    pub backend_kind: AgentRuntimeBackendKind,
    pub protocol: String,
    pub execution_scope: AgentExecutionScope,
    pub runtime: ResolvedLaunchRuntime,
    pub capabilities: AgentCapabilitiesV1,
    pub attach_launch_knobs: AttachLaunchKnobs,
    pub effective_policy: Policy,
    pub baseline_source: BaselineSourceMetadata,
    pub field_provenance: BTreeMap<String, FieldProvenance>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DispatchResolutionErrorKind {
    UnknownOverrideFamily,
    OverrideNotSupportedForCaller,
    OverrideExceedsBaseline,
    InvalidPolicyOverlay,
    OverrideDeniedByPolicy,
    RuntimeUnrealizableAfterResolution,
    MissingRequiredAttachContinuity,
    BaselineNotFound,
    AmbiguousBaselineSelection,
    BaselineIneligible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DispatchRejectingLayer {
    CallerContract,
    BaselineTruth,
    Policy,
    RuntimeMaterialization,
}

impl DispatchRejectingLayer {
    fn as_str(self) -> &'static str {
        match self {
            Self::CallerContract => "caller contract",
            Self::BaselineTruth => "baseline truth",
            Self::Policy => "policy",
            Self::RuntimeMaterialization => "runtime materialization",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DispatchResolutionError {
    pub kind: DispatchResolutionErrorKind,
    pub field: &'static str,
    pub rejecting_layer: DispatchRejectingLayer,
    pub reason: String,
}

impl std::fmt::Display for DispatchResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} rejected field '{}': {}",
            self.rejecting_layer.as_str(),
            self.field,
            self.reason
        )
    }
}

impl std::error::Error for DispatchResolutionError {}

pub(crate) fn resolve_inventory_contract_for_exact_backend(
    cwd: &Path,
    effective_config: &SubstrateConfig,
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
    base_policy: &Policy,
    envelope: &DispatchRequestEnvelope,
    scope: AgentExecutionScope,
) -> Result<Option<ResolvedLaunchContract>, DispatchResolutionError> {
    let backend_id = envelope
        .backend_id
        .as_deref()
        .ok_or_else(|| DispatchResolutionError {
            kind: DispatchResolutionErrorKind::BaselineNotFound,
            field: "backend_id",
            rejecting_layer: DispatchRejectingLayer::CallerContract,
            reason: "exact inventory-backed dispatch requires backend_id".to_string(),
        })?;
    validate_backend_id(backend_id).map_err(|err| DispatchResolutionError {
        kind: DispatchResolutionErrorKind::BaselineNotFound,
        field: "backend_id",
        rejecting_layer: DispatchRejectingLayer::CallerContract,
        reason: err.to_string(),
    })?;

    let mut matches = inventory
        .values()
        .map(|entry| project_inventory_entry(cwd, entry, effective_config))
        .filter(|entry| entry.execution_scope == scope && entry.backend_id == backend_id)
        .collect::<Vec<_>>();

    match matches.len() {
        0 => Ok(None),
        1 => resolve_inventory_projected_contract(
            base_policy,
            envelope,
            matches.pop().expect("single projected match"),
        )
        .map(Some),
        _ => {
            let agent_ids = matches
                .iter()
                .map(|entry| entry.agent_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(DispatchResolutionError {
                kind: DispatchResolutionErrorKind::AmbiguousBaselineSelection,
                field: "backend_id",
                rejecting_layer: DispatchRejectingLayer::BaselineTruth,
                reason: format!(
                    "ambiguous exact backend selection: multiple {} runtime entries advertise backend '{}' ({agent_ids})",
                    runtime_scope_label(scope),
                    backend_id,
                ),
            })
        }
    }
}

pub(crate) fn resolve_inventory_contract_for_unique_scope(
    cwd: &Path,
    effective_config: &SubstrateConfig,
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
    base_policy: &Policy,
    envelope: &DispatchRequestEnvelope,
    scope: AgentExecutionScope,
) -> Result<Option<ResolvedLaunchContract>, DispatchResolutionError> {
    let mut selected = Vec::new();

    for entry in inventory.values() {
        let projected = project_inventory_entry(cwd, entry, effective_config);
        if projected.execution_scope != scope {
            continue;
        }
        selected.push(resolve_inventory_projected_contract(
            base_policy,
            envelope,
            projected,
        )?);
    }

    match selected.len() {
        0 => Ok(None),
        1 => Ok(selected.into_iter().next()),
        _ => {
            let agent_ids = selected
                .iter()
                .map(|entry| entry.agent_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(DispatchResolutionError {
                kind: DispatchResolutionErrorKind::AmbiguousBaselineSelection,
                field: "execution_scope",
                rejecting_layer: DispatchRejectingLayer::BaselineTruth,
                reason: format!(
                    "ambiguous world member selection: multiple eligible {} members found ({agent_ids})",
                    runtime_scope_label(scope),
                ),
            })
        }
    }
}

#[allow(dead_code)]
pub(crate) fn resolve_persisted_host_attach_contract(
    envelope: &DispatchRequestEnvelope,
    contract: &HostAttachContract,
) -> Result<ResolvedLaunchContract, DispatchResolutionError> {
    if envelope.baseline_kind != DispatchBaselineKind::PersistedHostAttach {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::OverrideNotSupportedForCaller,
            field: "baseline_kind",
            rejecting_layer: DispatchRejectingLayer::CallerContract,
            reason: "persisted attach resolver requires baseline_kind=persisted_host_attach"
                .to_string(),
        });
    }
    if envelope
        .requested_execution_scope_override
        .is_some_and(|scope| scope != contract.execution_scope)
    {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::OverrideExceedsBaseline,
            field: "requested_execution_scope",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: format!(
                "persisted attach launch cannot replace execution scope {} with {}",
                runtime_scope_label(contract.execution_scope),
                runtime_scope_label(envelope.requested_execution_scope_override.unwrap()),
            ),
        });
    }
    if !envelope.capability_overrides.is_empty() {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::OverrideNotSupportedForCaller,
            field: "capability_overrides",
            rejecting_layer: DispatchRejectingLayer::CallerContract,
            reason: "persisted attach launches do not accept dispatch-time capability overrides in slice 29".to_string(),
        });
    }
    if envelope
        .backend_id
        .as_deref()
        .is_some_and(|backend_id| backend_id != contract.backend_id)
    {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::OverrideExceedsBaseline,
            field: "backend_id",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: format!(
                "persisted attach launch cannot replace backend '{}' with '{}'",
                contract.backend_id,
                envelope.backend_id.as_deref().unwrap_or_default(),
            ),
        });
    }

    let backend_kind = contract
        .launch_descriptor
        .backend_kind
        .try_into()
        .map_err(|err| DispatchResolutionError {
            kind: DispatchResolutionErrorKind::BaselineIneligible,
            field: "backend_kind",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: format!("{err:#}"),
        })?;
    let agent_id = contract.launch_descriptor.agent_id.clone();
    let protocol = contract.protocol.clone();
    if protocol != PURE_AGENT_PROTOCOL {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::BaselineIneligible,
            field: "protocol",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: protocol_validation_error(
                &format!(
                    "persisted host attach contract backend '{}'",
                    contract.backend_id
                ),
                Some(protocol.as_str()),
            ),
        });
    }

    let effective_policy = serde_json::from_value::<Policy>(contract.effective_policy.clone())
        .map_err(|err| DispatchResolutionError {
            kind: DispatchResolutionErrorKind::BaselineIneligible,
            field: "effective_policy",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: format!(
                "persisted host attach contract stored an invalid policy snapshot: {err}"
            ),
        })?;
    let attach_launch_knobs = resolve_persisted_attach_launch_knobs(envelope, contract)?;

    let mut field_provenance = BTreeMap::new();
    for field in [
        "agent_id",
        "backend_id",
        "protocol",
        "execution_scope",
        "cli_mode",
        "cli_binary",
        "session_resume",
        "session_fork",
        "session_stop",
        "status_snapshot",
        "event_stream",
        "effective_policy",
        "requested_execution_scope",
        "host_execution_client_start",
        "attach_mode_preference",
    ] {
        field_provenance.insert(
            field.to_string(),
            FieldProvenance {
                baseline_origin: FieldBaselineOrigin::PersistedHostAttachContract,
                value_origin: FieldValueOrigin::InventoryExplicit,
            },
        );
    }

    Ok(ResolvedLaunchContract {
        caller_kind: envelope.caller_kind,
        baseline_kind: DispatchBaselineKind::PersistedHostAttach,
        agent_id,
        backend_id: contract.backend_id.clone(),
        backend_kind,
        protocol,
        execution_scope: contract.execution_scope,
        runtime: ResolvedLaunchRuntime {
            kind: AgentConfigKind::Cli,
            cli_mode: AgentCliMode::Persistent,
            cli_binary: Some(contract.launch_descriptor.binary_path.clone()),
        },
        capabilities: AgentCapabilitiesV1 {
            session_start: true,
            session_resume: contract.capabilities.session_resume,
            session_fork: contract.capabilities.session_fork,
            session_stop: contract.capabilities.session_stop,
            status_snapshot: contract.capabilities.status_snapshot,
            event_stream: contract.capabilities.event_stream,
            llm: true,
            mcp_client: false,
        },
        attach_launch_knobs,
        effective_policy,
        baseline_source: BaselineSourceMetadata {
            baseline_kind: DispatchBaselineKind::PersistedHostAttach,
            baseline_origin: FieldBaselineOrigin::PersistedHostAttachContract,
            inventory_path: None,
            orchestration_session_id: envelope.orchestration_session_id.clone(),
        },
        field_provenance,
    })
}

fn resolve_persisted_attach_launch_knobs(
    envelope: &DispatchRequestEnvelope,
    contract: &HostAttachContract,
) -> Result<AttachLaunchKnobs, DispatchResolutionError> {
    let host_execution_client_start = match (
        contract.attach_launch_knobs.host_execution_client_start,
        envelope.attach_launch_knobs.host_execution_client_start,
    ) {
        (HostAttachExecutionClientStart::StartNow, HostExecutionClientStart::StartNow)
        | (HostAttachExecutionClientStart::StartNow, HostExecutionClientStart::Defer) => {
            envelope.attach_launch_knobs.host_execution_client_start
        }
        (HostAttachExecutionClientStart::Defer, HostExecutionClientStart::Defer) => {
            HostExecutionClientStart::Defer
        }
        (HostAttachExecutionClientStart::Defer, HostExecutionClientStart::StartNow) => {
            return Err(DispatchResolutionError {
                kind: DispatchResolutionErrorKind::OverrideExceedsBaseline,
                field: "host_execution_client_start",
                rejecting_layer: DispatchRejectingLayer::BaselineTruth,
                reason:
                    "persisted attach launch cannot broaden host execution client start from defer to start_now"
                        .to_string(),
            });
        }
    };

    let attach_mode_preference =
        if persisted_attach_mode_rank(envelope.attach_launch_knobs.attach_mode_preference)
            <= persisted_host_attach_mode_rank(contract.attach_launch_knobs.attach_mode_preference)
        {
            envelope.attach_launch_knobs.attach_mode_preference
        } else {
            return Err(DispatchResolutionError {
                kind: DispatchResolutionErrorKind::OverrideExceedsBaseline,
                field: "attach_mode_preference",
                rejecting_layer: DispatchRejectingLayer::BaselineTruth,
                reason: format!(
                    "persisted attach launch cannot broaden attach mode from {} to {}",
                    persisted_host_attach_mode_label(
                        contract.attach_launch_knobs.attach_mode_preference
                    ),
                    persisted_attach_mode_label(
                        envelope.attach_launch_knobs.attach_mode_preference
                    ),
                ),
            });
        };

    if matches!(
        attach_mode_preference,
        AttachModePreference::ContinuityRequired
    ) && contract.continuity_uaa_session_id.is_none()
    {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::MissingRequiredAttachContinuity,
            field: "continuity_uaa_session_id",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason:
                "persisted host attach contract no longer has continuity required for this attach launch"
                    .to_string(),
        });
    }

    Ok(AttachLaunchKnobs {
        requested_execution_scope: contract.attach_launch_knobs.requested_execution_scope,
        host_execution_client_start,
        attach_mode_preference,
    })
}

fn persisted_attach_mode_rank(value: AttachModePreference) -> u8 {
    match value {
        AttachModePreference::ContinuityRequired => 0,
        AttachModePreference::ContinuityPreferred => 1,
        AttachModePreference::FreshAllowed => 2,
    }
}

fn persisted_host_attach_mode_rank(value: HostAttachModePreference) -> u8 {
    match value {
        HostAttachModePreference::ContinuityRequired => 0,
        HostAttachModePreference::ContinuityPreferred => 1,
        HostAttachModePreference::FreshAllowed => 2,
    }
}

fn persisted_attach_mode_label(value: AttachModePreference) -> &'static str {
    match value {
        AttachModePreference::ContinuityRequired => "continuity_required",
        AttachModePreference::ContinuityPreferred => "continuity_preferred",
        AttachModePreference::FreshAllowed => "fresh_allowed",
    }
}

fn persisted_host_attach_mode_label(value: HostAttachModePreference) -> &'static str {
    match value {
        HostAttachModePreference::ContinuityRequired => "continuity_required",
        HostAttachModePreference::ContinuityPreferred => "continuity_preferred",
        HostAttachModePreference::FreshAllowed => "fresh_allowed",
    }
}

fn resolve_inventory_projected_contract(
    base_policy: &Policy,
    envelope: &DispatchRequestEnvelope,
    projected: ProjectedInventoryEntryV1,
) -> Result<ResolvedLaunchContract, DispatchResolutionError> {
    validate_inventory_projected_candidate(&projected)?;
    validate_dispatch_overrides(envelope, projected.execution_scope)?;
    let effective_policy =
        resolve_inventory_effective_policy(base_policy, projected.policy_overlay.as_ref());
    let (capabilities, capability_origins) = resolve_inventory_capabilities(
        projected.capabilities.clone(),
        &envelope.capability_overrides,
    )?;
    if !base_policy
        .agents_allowed_backends
        .iter()
        .any(|allowed| allowed == &projected.backend_id)
    {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::OverrideDeniedByPolicy,
            field: "backend_id",
            rejecting_layer: DispatchRejectingLayer::Policy,
            reason: format!(
                "selected orchestrator backend '{}' is not allowlisted by effective policy agents.allowed_backends",
                projected.backend_id
            ),
        });
    }

    let baseline_origin = match projected.origin {
        AgentInventoryBaselineOrigin::GlobalInventory => FieldBaselineOrigin::GlobalInventory,
        AgentInventoryBaselineOrigin::WorkspaceInventory => FieldBaselineOrigin::WorkspaceInventory,
    };

    let mut field_provenance = BTreeMap::new();
    field_provenance.insert(
        "agent_id".to_string(),
        FieldProvenance {
            baseline_origin,
            value_origin: FieldValueOrigin::InventoryExplicit,
        },
    );
    field_provenance.insert(
        "backend_id".to_string(),
        FieldProvenance {
            baseline_origin,
            value_origin: FieldValueOrigin::InventoryExplicit,
        },
    );
    field_provenance.insert(
        "protocol".to_string(),
        FieldProvenance {
            baseline_origin,
            value_origin: FieldValueOrigin::InventoryExplicit,
        },
    );
    field_provenance.insert(
        "execution_scope".to_string(),
        FieldProvenance {
            baseline_origin,
            value_origin: map_value_origin(projected.execution_scope_origin),
        },
    );
    field_provenance.insert(
        "cli_mode".to_string(),
        FieldProvenance {
            baseline_origin,
            value_origin: map_value_origin(projected.cli_mode_origin),
        },
    );
    field_provenance.insert(
        "cli_binary".to_string(),
        FieldProvenance {
            baseline_origin,
            value_origin: FieldValueOrigin::InventoryExplicit,
        },
    );
    field_provenance.insert(
        "effective_policy".to_string(),
        FieldProvenance {
            baseline_origin,
            value_origin: if projected.policy_overlay.is_some() {
                FieldValueOrigin::DispatchOverrideNarrowedByPolicy
            } else {
                FieldValueOrigin::InventoryExplicit
            },
        },
    );
    for (field, value_origin) in capability_origins {
        field_provenance.insert(
            field,
            FieldProvenance {
                baseline_origin,
                value_origin,
            },
        );
    }

    let backend_kind = orchestrator_backend_kind(projected.agent_id.as_str()).map_err(|err| {
        DispatchResolutionError {
            kind: DispatchResolutionErrorKind::BaselineIneligible,
            field: "agent_id",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: err
                .to_string()
                .replace("selected orchestrator backend", "selected runtime backend"),
        }
    })?;

    Ok(ResolvedLaunchContract {
        caller_kind: envelope.caller_kind,
        baseline_kind: DispatchBaselineKind::InventoryLaunch,
        agent_id: projected.agent_id,
        backend_id: projected.backend_id,
        backend_kind,
        protocol: PURE_AGENT_PROTOCOL.to_string(),
        execution_scope: projected.execution_scope,
        runtime: ResolvedLaunchRuntime {
            kind: projected.kind,
            cli_mode: projected.cli_mode,
            cli_binary: projected.cli_binary,
        },
        capabilities,
        attach_launch_knobs: envelope.attach_launch_knobs,
        effective_policy,
        baseline_source: BaselineSourceMetadata {
            baseline_kind: DispatchBaselineKind::InventoryLaunch,
            baseline_origin,
            inventory_path: Some(projected.path),
            orchestration_session_id: envelope.orchestration_session_id.clone(),
        },
        field_provenance,
    })
}

fn validate_inventory_projected_candidate(
    projected: &ProjectedInventoryEntryV1,
) -> Result<(), DispatchResolutionError> {
    if projected.protocol.as_deref() != Some(PURE_AGENT_PROTOCOL) {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::BaselineIneligible,
            field: "protocol",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: format!(
                "selected {} runtime '{}' for backend '{}' {}",
                runtime_scope_label(projected.execution_scope),
                projected.agent_id,
                projected.backend_id,
                protocol_validation_error("does not advertise", projected.protocol.as_deref())
                    .replacen("does not advertise ", "", 1),
            ),
        });
    }

    if let Some(capability) = missing_required_dispatch_capability(&projected.capabilities) {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::BaselineIneligible,
            field: capability,
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: format!(
                "selected {} runtime '{}' for backend '{}' is missing required capability '{}'",
                runtime_scope_label(projected.execution_scope),
                projected.agent_id,
                projected.backend_id,
                capability,
            ),
        });
    }

    if projected.kind != AgentConfigKind::Cli {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::RuntimeUnrealizableAfterResolution,
            field: "config.kind",
            rejecting_layer: DispatchRejectingLayer::RuntimeMaterialization,
            reason: format!(
                "selected runtime '{}' is not runtime-realizable by the shell-owned UAA runtime because config.kind={} is unsupported; only config.kind=cli is supported in v1",
                projected.agent_id,
                projected.kind.as_str()
            ),
        });
    }

    Ok(())
}

fn resolve_inventory_effective_policy(
    base_policy: &Policy,
    overlay: Option<&PolicyPatch>,
) -> Policy {
    match overlay {
        Some(overlay) => apply_policy_patch(base_policy, overlay),
        None => base_policy.clone(),
    }
}

fn validate_dispatch_overrides(
    envelope: &DispatchRequestEnvelope,
    baseline_scope: AgentExecutionScope,
) -> Result<(), DispatchResolutionError> {
    if envelope
        .requested_execution_scope_override
        .is_some_and(|scope| scope != baseline_scope)
    {
        return Err(DispatchResolutionError {
            kind: DispatchResolutionErrorKind::OverrideExceedsBaseline,
            field: "requested_execution_scope",
            rejecting_layer: DispatchRejectingLayer::BaselineTruth,
            reason: format!(
                "dispatch scope override {} broadens or changes baseline {}",
                runtime_scope_label(envelope.requested_execution_scope_override.unwrap()),
                runtime_scope_label(baseline_scope),
            ),
        });
    }

    validate_capability_override_shape(&envelope.capability_overrides)?;

    Ok(())
}

fn validate_capability_override_shape(
    overrides: &DispatchCapabilityOverrideSet,
) -> Result<(), DispatchResolutionError> {
    for (field, value) in [
        ("session_start", overrides.session_start),
        ("llm", overrides.llm),
        ("mcp_client", overrides.mcp_client),
    ] {
        if value.is_some() {
            return Err(DispatchResolutionError {
                kind: DispatchResolutionErrorKind::OverrideNotSupportedForCaller,
                field,
                rejecting_layer: DispatchRejectingLayer::CallerContract,
                reason: "dispatch-time capability override is unsupported for this field in slice 29.5; only session_resume, session_fork, session_stop, status_snapshot, and event_stream may narrow from true to false".to_string(),
            });
        }
    }

    for (field, value) in [
        ("session_resume", overrides.session_resume),
        ("session_fork", overrides.session_fork),
        ("session_stop", overrides.session_stop),
        ("status_snapshot", overrides.status_snapshot),
        ("event_stream", overrides.event_stream),
    ] {
        if value == Some(true) {
            return Err(DispatchResolutionError {
                kind: DispatchResolutionErrorKind::OverrideExceedsBaseline,
                field,
                rejecting_layer: DispatchRejectingLayer::CallerContract,
                reason:
                    "dispatch-time capability override must be narrowing-only; only true-to-false is supported in slice 29.5"
                        .to_string(),
            });
        }
    }

    Ok(())
}

fn resolve_inventory_capabilities(
    baseline: AgentCapabilitiesV1,
    overrides: &DispatchCapabilityOverrideSet,
) -> Result<(AgentCapabilitiesV1, BTreeMap<String, FieldValueOrigin>), DispatchResolutionError> {
    let mut capabilities = baseline.clone();
    let mut origins = BTreeMap::new();

    origins.insert(
        "session_start".to_string(),
        FieldValueOrigin::InventoryExplicit,
    );
    origins.insert("llm".to_string(), FieldValueOrigin::InventoryExplicit);
    origins.insert(
        "mcp_client".to_string(),
        FieldValueOrigin::InventoryExplicit,
    );

    apply_supported_capability_override(
        "session_resume",
        baseline.session_resume,
        overrides.session_resume,
        &mut capabilities.session_resume,
        &mut origins,
    )?;
    apply_supported_capability_override(
        "session_fork",
        baseline.session_fork,
        overrides.session_fork,
        &mut capabilities.session_fork,
        &mut origins,
    )?;
    apply_supported_capability_override(
        "session_stop",
        baseline.session_stop,
        overrides.session_stop,
        &mut capabilities.session_stop,
        &mut origins,
    )?;
    apply_supported_capability_override(
        "status_snapshot",
        baseline.status_snapshot,
        overrides.status_snapshot,
        &mut capabilities.status_snapshot,
        &mut origins,
    )?;
    apply_supported_capability_override(
        "event_stream",
        baseline.event_stream,
        overrides.event_stream,
        &mut capabilities.event_stream,
        &mut origins,
    )?;

    Ok((capabilities, origins))
}

fn apply_supported_capability_override(
    field: &'static str,
    baseline: bool,
    override_value: Option<bool>,
    target: &mut bool,
    origins: &mut BTreeMap<String, FieldValueOrigin>,
) -> Result<(), DispatchResolutionError> {
    let value_origin = match override_value {
        Some(false) if baseline => {
            *target = false;
            FieldValueOrigin::DispatchOverrideAccepted
        }
        Some(false) => {
            return Err(DispatchResolutionError {
                kind: DispatchResolutionErrorKind::OverrideExceedsBaseline,
                field,
                rejecting_layer: DispatchRejectingLayer::BaselineTruth,
                reason: "dispatch-time capability override cannot narrow a baseline capability that is already false".to_string(),
            });
        }
        Some(true) => {
            return Err(DispatchResolutionError {
                kind: DispatchResolutionErrorKind::OverrideExceedsBaseline,
                field,
                rejecting_layer: DispatchRejectingLayer::CallerContract,
                reason:
                    "dispatch-time capability override must be narrowing-only; only true-to-false is supported in slice 29.5"
                        .to_string(),
            });
        }
        None => FieldValueOrigin::InventoryExplicit,
    };

    origins.insert(field.to_string(), value_origin);
    Ok(())
}

fn map_value_origin(origin: ProjectedInventoryValueOrigin) -> FieldValueOrigin {
    match origin {
        ProjectedInventoryValueOrigin::InventoryExplicit => FieldValueOrigin::InventoryExplicit,
        ProjectedInventoryValueOrigin::EffectiveConfigDefault => {
            FieldValueOrigin::EffectiveConfigDefault
        }
    }
}

fn missing_required_dispatch_capability(
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

fn runtime_scope_label(scope: AgentExecutionScope) -> &'static str {
    match scope {
        AgentExecutionScope::Host => "host-scoped",
        AgentExecutionScope::World => "world-scoped",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::{
        resolve_inventory_contract_for_exact_backend, resolve_persisted_host_attach_contract,
        AttachLaunchKnobs, AttachModePreference, DispatchBaselineKind, DispatchCallerKind,
        DispatchCapabilityOverrideSet, DispatchRejectingLayer, DispatchRequestEnvelope,
        DispatchResolutionErrorKind, FieldBaselineOrigin, FieldValueOrigin,
        HostExecutionClientStart,
    };
    use crate::execution::agent_inventory::{
        AgentCapabilitiesV1, AgentCliConfigV1, AgentConfigKind, AgentConfigV1,
        AgentExecutionConfigV1, AgentFileV1, AgentInventoryEntryV1,
    };
    use crate::execution::agent_runtime::control::{
        ResolvedRuntimeBackendKind, ResolvedRuntimeDescriptor,
    };
    use crate::execution::agent_runtime::orchestration_session::HostAttachContract;
    use crate::execution::config_model::{AgentCliMode, AgentExecutionScope, SubstrateConfig};
    use crate::execution::policy_model::PolicyPatch;
    use crate::execution::workspace::{workspace_marker_path, SUBSTRATE_DIR_NAME};
    use substrate_broker::Policy;

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

    fn exact_backend_envelope(
        caller_kind: DispatchCallerKind,
        baseline_kind: DispatchBaselineKind,
        backend_id: &str,
    ) -> DispatchRequestEnvelope {
        DispatchRequestEnvelope {
            caller_kind,
            baseline_kind,
            backend_id: Some(backend_id.to_string()),
            orchestration_session_id: Some("sess_123".to_string()),
            requested_execution_scope_override: None,
            capability_overrides: DispatchCapabilityOverrideSet::default(),
            attach_launch_knobs: AttachLaunchKnobs {
                requested_execution_scope: AgentExecutionScope::Host,
                host_execution_client_start: HostExecutionClientStart::StartNow,
                attach_mode_preference: AttachModePreference::ContinuityRequired,
            },
            has_prompt_payload: true,
        }
    }

    fn make_entry(
        path: PathBuf,
        agent_id: &str,
        scope: Option<AgentExecutionScope>,
        cli_mode: Option<AgentCliMode>,
        capabilities: AgentCapabilitiesV1,
    ) -> AgentInventoryEntryV1 {
        make_entry_with_overlay(path, agent_id, scope, cli_mode, capabilities, None)
    }

    fn make_entry_with_overlay(
        path: PathBuf,
        agent_id: &str,
        scope: Option<AgentExecutionScope>,
        cli_mode: Option<AgentCliMode>,
        capabilities: AgentCapabilitiesV1,
        policy_overlay: Option<PolicyPatch>,
    ) -> AgentInventoryEntryV1 {
        AgentInventoryEntryV1 {
            path,
            file: AgentFileV1 {
                version: 1,
                id: agent_id.to_string(),
                config: AgentConfigV1 {
                    enabled: true,
                    kind: AgentConfigKind::Cli,
                    protocol: Some(super::PURE_AGENT_PROTOCOL.to_string()),
                    execution: AgentExecutionConfigV1 { scope },
                    cli: Some(AgentCliConfigV1 {
                        binary: "cargo".to_string(),
                        mode: cli_mode,
                    }),
                    api: None,
                    capabilities,
                },
                policy_overlay,
            },
        }
    }

    #[test]
    fn inventory_contract_tracks_workspace_origin_and_config_defaults() {
        let temp = tempdir().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let workspace_agents = workspace_root.join(SUBSTRATE_DIR_NAME).join("agents");
        std::fs::create_dir_all(&workspace_agents).expect("workspace agents");
        std::fs::create_dir_all(
            workspace_marker_path(&workspace_root)
                .parent()
                .expect("marker parent"),
        )
        .expect("workspace marker dir");
        std::fs::write(workspace_marker_path(&workspace_root), "version: 1\n").expect("marker");

        let cwd = workspace_root.join("src");
        std::fs::create_dir_all(&cwd).expect("cwd");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                workspace_agents.join("codex.yaml"),
                "codex",
                None,
                None,
                required_capabilities(),
            ),
        );

        let mut config = SubstrateConfig::default();
        config.agents.defaults.execution.scope = AgentExecutionScope::Host;
        config.agents.defaults.cli.mode = AgentCliMode::Persistent;
        let policy = Policy {
            agents_allowed_backends: vec!["cli:codex".to_string()],
            ..Policy::default()
        };

        let resolved = resolve_inventory_contract_for_exact_backend(
            &cwd,
            &config,
            &inventory,
            &policy,
            &exact_backend_envelope(
                DispatchCallerKind::HumanStart,
                DispatchBaselineKind::InventoryLaunch,
                "cli:codex",
            ),
            AgentExecutionScope::Host,
        )
        .expect("resolution should succeed")
        .expect("contract");

        assert_eq!(
            resolved.baseline_source.baseline_origin,
            FieldBaselineOrigin::WorkspaceInventory
        );
        assert_eq!(
            resolved
                .field_provenance
                .get("execution_scope")
                .expect("scope provenance")
                .value_origin,
            FieldValueOrigin::EffectiveConfigDefault
        );
        assert_eq!(
            resolved
                .field_provenance
                .get("cli_mode")
                .expect("cli provenance")
                .value_origin,
            FieldValueOrigin::EffectiveConfigDefault
        );
        assert_eq!(resolved.backend_id, "cli:codex");
        assert_eq!(resolved.execution_scope, AgentExecutionScope::Host);
    }

    #[test]
    fn inventory_contract_merges_policy_overlay_into_effective_policy() {
        let cwd = PathBuf::from(".");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry_with_overlay(
                PathBuf::from("codex.yaml"),
                "codex",
                Some(AgentExecutionScope::Host),
                Some(AgentCliMode::Persistent),
                required_capabilities(),
                Some(PolicyPatch {
                    require_approval: Some(true),
                    ..PolicyPatch::default()
                }),
            ),
        );
        let policy = Policy {
            agents_allowed_backends: vec!["cli:codex".to_string()],
            require_approval: false,
            ..Policy::default()
        };
        let config = SubstrateConfig::default();

        let resolved = resolve_inventory_contract_for_exact_backend(
            &cwd,
            &config,
            &inventory,
            &policy,
            &exact_backend_envelope(
                DispatchCallerKind::HumanStart,
                DispatchBaselineKind::InventoryLaunch,
                "cli:codex",
            ),
            AgentExecutionScope::Host,
        )
        .expect("resolution should succeed")
        .expect("contract");

        assert!(resolved.effective_policy.require_approval);
        assert_eq!(
            resolved
                .field_provenance
                .get("effective_policy")
                .expect("policy provenance")
                .value_origin,
            FieldValueOrigin::DispatchOverrideNarrowedByPolicy
        );
    }

    #[test]
    fn inventory_contract_without_policy_overlay_keeps_base_policy() {
        let cwd = PathBuf::from(".");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                PathBuf::from("codex.yaml"),
                "codex",
                Some(AgentExecutionScope::Host),
                Some(AgentCliMode::Persistent),
                required_capabilities(),
            ),
        );
        let policy = Policy {
            agents_allowed_backends: vec!["cli:codex".to_string()],
            require_approval: false,
            ..Policy::default()
        };
        let config = SubstrateConfig::default();

        let resolved = resolve_inventory_contract_for_exact_backend(
            &cwd,
            &config,
            &inventory,
            &policy,
            &exact_backend_envelope(
                DispatchCallerKind::HumanStart,
                DispatchBaselineKind::InventoryLaunch,
                "cli:codex",
            ),
            AgentExecutionScope::Host,
        )
        .expect("resolution should succeed")
        .expect("contract");

        assert!(!resolved.effective_policy.require_approval);
        assert_eq!(
            resolved
                .field_provenance
                .get("effective_policy")
                .expect("policy provenance")
                .value_origin,
            FieldValueOrigin::InventoryExplicit
        );
    }

    #[test]
    fn persisted_attach_contract_is_explicit_baseline_domain() {
        let contract = HostAttachContract {
            backend_id: "cli:codex".to_string(),
            execution_scope: AgentExecutionScope::Host,
            protocol: super::PURE_AGENT_PROTOCOL.to_string(),
            launch_descriptor: ResolvedRuntimeDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: ResolvedRuntimeBackendKind::Codex,
                protocol: super::PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: "cargo".to_string(),
            },
            capabilities:
                crate::execution::agent_runtime::orchestration_session::HostAttachCapabilities::default(),
            attach_launch_knobs:
                crate::execution::agent_runtime::orchestration_session::HostAttachLaunchKnobs::default(),
            effective_policy: serde_json::to_value(Policy {
                agents_allowed_backends: vec!["cli:codex".to_string()],
                ..Policy::default()
            })
            .expect("serialize policy"),
            continuity_uaa_session_id: Some("uaa_123".to_string()),
        };

        let resolved = resolve_persisted_host_attach_contract(
            &exact_backend_envelope(
                DispatchCallerKind::HumanReattach,
                DispatchBaselineKind::PersistedHostAttach,
                "cli:codex",
            ),
            &contract,
        )
        .expect("persisted attach resolution");

        assert_eq!(
            resolved.baseline_kind,
            DispatchBaselineKind::PersistedHostAttach
        );
        assert_eq!(
            resolved.baseline_source.baseline_origin,
            FieldBaselineOrigin::PersistedHostAttachContract
        );
        assert_eq!(resolved.backend_id, "cli:codex");
        assert_eq!(resolved.agent_id, "codex");
        assert_eq!(
            resolved.effective_policy.agents_allowed_backends,
            vec!["cli:codex".to_string()]
        );
    }

    #[test]
    fn persisted_attach_contract_reuses_persisted_capabilities_and_only_honors_or_narrows_knobs() {
        let contract = HostAttachContract {
            backend_id: "cli:codex".to_string(),
            execution_scope: AgentExecutionScope::Host,
            protocol: super::PURE_AGENT_PROTOCOL.to_string(),
            launch_descriptor: ResolvedRuntimeDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: ResolvedRuntimeBackendKind::Codex,
                protocol: super::PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: "cargo".to_string(),
            },
            capabilities:
                crate::execution::agent_runtime::orchestration_session::HostAttachCapabilities {
                    session_resume: false,
                    session_fork: true,
                    session_stop: false,
                    status_snapshot: true,
                    event_stream: false,
                },
            attach_launch_knobs:
                crate::execution::agent_runtime::orchestration_session::HostAttachLaunchKnobs {
                    requested_execution_scope: AgentExecutionScope::Host,
                    host_execution_client_start:
                        crate::execution::agent_runtime::orchestration_session::HostAttachExecutionClientStart::StartNow,
                    attach_mode_preference:
                        crate::execution::agent_runtime::orchestration_session::HostAttachModePreference::FreshAllowed,
                },
            effective_policy: serde_json::to_value(Policy {
                agents_allowed_backends: vec!["cli:codex".to_string()],
                ..Policy::default()
            })
            .expect("serialize policy"),
            continuity_uaa_session_id: Some("uaa_123".to_string()),
        };
        let mut envelope = exact_backend_envelope(
            DispatchCallerKind::HumanFork,
            DispatchBaselineKind::PersistedHostAttach,
            "cli:codex",
        );
        envelope.attach_launch_knobs.host_execution_client_start = HostExecutionClientStart::Defer;
        envelope.attach_launch_knobs.attach_mode_preference =
            AttachModePreference::ContinuityPreferred;

        let resolved =
            resolve_persisted_host_attach_contract(&envelope, &contract).expect("resolution");

        assert!(!resolved.capabilities.session_resume);
        assert!(resolved.capabilities.session_fork);
        assert!(!resolved.capabilities.session_stop);
        assert!(resolved.capabilities.status_snapshot);
        assert!(!resolved.capabilities.event_stream);
        assert_eq!(
            resolved.attach_launch_knobs.requested_execution_scope,
            AgentExecutionScope::Host
        );
        assert_eq!(
            resolved.attach_launch_knobs.host_execution_client_start,
            HostExecutionClientStart::Defer
        );
        assert_eq!(
            resolved.attach_launch_knobs.attach_mode_preference,
            AttachModePreference::ContinuityPreferred
        );
    }

    #[test]
    fn persisted_attach_contract_missing_policy_snapshot_fails_closed() {
        let mut payload = serde_json::json!({
            "backend_id": "cli:codex",
            "execution_scope": "host",
            "protocol": super::PURE_AGENT_PROTOCOL,
            "launch_descriptor": {
                "agent_id": "codex",
                "backend_id": "cli:codex",
                "backend_kind": "codex",
                "protocol": super::PURE_AGENT_PROTOCOL,
                "execution_scope": "host",
                "binary_path": "cargo"
            },
            "capabilities": {
                "session_resume": true,
                "session_fork": true,
                "session_stop": true,
                "status_snapshot": true,
                "event_stream": true
            },
            "attach_launch_knobs": {
                "requested_execution_scope": "host",
                "host_execution_client_start": "start_now",
                "attach_mode_preference": "continuity_required"
            },
            "continuity_uaa_session_id": "uaa_123"
        });
        payload
            .as_object_mut()
            .expect("object")
            .remove("effective_policy");

        let err = serde_json::from_value::<HostAttachContract>(payload)
            .expect_err("missing persisted policy must fail closed");
        assert!(err.to_string().contains("effective_policy"));
    }

    #[test]
    fn persisted_attach_contract_broadening_knobs_fails_closed() {
        let contract = HostAttachContract {
            backend_id: "cli:codex".to_string(),
            execution_scope: AgentExecutionScope::Host,
            protocol: super::PURE_AGENT_PROTOCOL.to_string(),
            launch_descriptor: ResolvedRuntimeDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: ResolvedRuntimeBackendKind::Codex,
                protocol: super::PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: "cargo".to_string(),
            },
            capabilities:
                crate::execution::agent_runtime::orchestration_session::HostAttachCapabilities::default(),
            attach_launch_knobs:
                crate::execution::agent_runtime::orchestration_session::HostAttachLaunchKnobs {
                    requested_execution_scope: AgentExecutionScope::Host,
                    host_execution_client_start:
                        crate::execution::agent_runtime::orchestration_session::HostAttachExecutionClientStart::Defer,
                    attach_mode_preference:
                        crate::execution::agent_runtime::orchestration_session::HostAttachModePreference::ContinuityRequired,
                },
            effective_policy: serde_json::to_value(Policy {
                agents_allowed_backends: vec!["cli:codex".to_string()],
                ..Policy::default()
            })
            .expect("serialize policy"),
            continuity_uaa_session_id: Some("uaa_123".to_string()),
        };
        let mut envelope = exact_backend_envelope(
            DispatchCallerKind::HumanReattach,
            DispatchBaselineKind::PersistedHostAttach,
            "cli:codex",
        );
        envelope.attach_launch_knobs.host_execution_client_start =
            HostExecutionClientStart::StartNow;
        envelope.attach_launch_knobs.attach_mode_preference = AttachModePreference::FreshAllowed;

        let err = resolve_persisted_host_attach_contract(&envelope, &contract)
            .expect_err("broadening persisted attach knobs must fail closed");
        assert!(
            err.field == "host_execution_client_start" || err.field == "attach_mode_preference"
        );
    }

    #[test]
    fn scope_override_that_changes_baseline_fails_closed() {
        let cwd = PathBuf::from(".");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                PathBuf::from("codex.yaml"),
                "codex",
                Some(AgentExecutionScope::Host),
                Some(AgentCliMode::Persistent),
                required_capabilities(),
            ),
        );
        let policy = Policy {
            agents_allowed_backends: vec!["cli:codex".to_string()],
            ..Policy::default()
        };
        let config = SubstrateConfig::default();
        let mut envelope = exact_backend_envelope(
            DispatchCallerKind::HumanStart,
            DispatchBaselineKind::InventoryLaunch,
            "cli:codex",
        );
        envelope.requested_execution_scope_override = Some(AgentExecutionScope::World);

        let error = resolve_inventory_contract_for_exact_backend(
            &cwd,
            &config,
            &inventory,
            &policy,
            &envelope,
            AgentExecutionScope::Host,
        )
        .expect_err("override must fail closed");

        assert_eq!(
            error.kind,
            DispatchResolutionErrorKind::OverrideExceedsBaseline
        );
        assert_eq!(error.field, "requested_execution_scope");
    }

    #[test]
    fn policy_denial_names_field_and_layer() {
        let cwd = PathBuf::from(".");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                PathBuf::from("codex.yaml"),
                "codex",
                Some(AgentExecutionScope::Host),
                Some(AgentCliMode::Persistent),
                required_capabilities(),
            ),
        );
        let config = SubstrateConfig::default();
        let policy = Policy::default();

        let error = resolve_inventory_contract_for_exact_backend(
            &cwd,
            &config,
            &inventory,
            &policy,
            &exact_backend_envelope(
                DispatchCallerKind::HumanStart,
                DispatchBaselineKind::InventoryLaunch,
                "cli:codex",
            ),
            AgentExecutionScope::Host,
        )
        .expect_err("policy denial expected");

        assert_eq!(
            error.kind,
            DispatchResolutionErrorKind::OverrideDeniedByPolicy
        );
        assert_eq!(error.field, "backend_id");
        assert!(error
            .to_string()
            .contains("policy rejected field 'backend_id'"));
    }

    #[test]
    fn supported_capability_override_family_narrows_from_true_to_false() {
        let cwd = PathBuf::from(".");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                PathBuf::from("codex.yaml"),
                "codex",
                Some(AgentExecutionScope::Host),
                Some(AgentCliMode::Persistent),
                required_capabilities(),
            ),
        );
        let policy = Policy {
            agents_allowed_backends: vec!["cli:codex".to_string()],
            ..Policy::default()
        };
        let config = SubstrateConfig::default();
        let mut envelope = exact_backend_envelope(
            DispatchCallerKind::HumanStart,
            DispatchBaselineKind::InventoryLaunch,
            "cli:codex",
        );
        envelope.capability_overrides.session_resume = Some(false);
        envelope.capability_overrides.session_fork = Some(false);
        envelope.capability_overrides.session_stop = Some(false);
        envelope.capability_overrides.status_snapshot = Some(false);
        envelope.capability_overrides.event_stream = Some(false);

        let resolved = resolve_inventory_contract_for_exact_backend(
            &cwd,
            &config,
            &inventory,
            &policy,
            &envelope,
            AgentExecutionScope::Host,
        )
        .expect("resolution should succeed")
        .expect("contract");

        assert!(!resolved.capabilities.session_resume);
        assert!(!resolved.capabilities.session_fork);
        assert!(!resolved.capabilities.session_stop);
        assert!(!resolved.capabilities.status_snapshot);
        assert!(!resolved.capabilities.event_stream);
        for field in [
            "session_resume",
            "session_fork",
            "session_stop",
            "status_snapshot",
            "event_stream",
        ] {
            assert_eq!(
                resolved
                    .field_provenance
                    .get(field)
                    .expect("capability provenance")
                    .value_origin,
                FieldValueOrigin::DispatchOverrideAccepted
            );
        }
    }

    #[test]
    fn unsupported_capability_override_fields_fail_closed_with_field_names() {
        let cwd = PathBuf::from(".");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                PathBuf::from("codex.yaml"),
                "codex",
                Some(AgentExecutionScope::Host),
                Some(AgentCliMode::Persistent),
                required_capabilities(),
            ),
        );
        let policy = Policy {
            agents_allowed_backends: vec!["cli:codex".to_string()],
            ..Policy::default()
        };
        let config = SubstrateConfig::default();

        for field in ["session_start", "llm", "mcp_client"] {
            let mut envelope = exact_backend_envelope(
                DispatchCallerKind::HumanStart,
                DispatchBaselineKind::InventoryLaunch,
                "cli:codex",
            );
            match field {
                "session_start" => envelope.capability_overrides.session_start = Some(false),
                "llm" => envelope.capability_overrides.llm = Some(false),
                "mcp_client" => envelope.capability_overrides.mcp_client = Some(false),
                _ => unreachable!(),
            }

            let error = resolve_inventory_contract_for_exact_backend(
                &cwd,
                &config,
                &inventory,
                &policy,
                &envelope,
                AgentExecutionScope::Host,
            )
            .expect_err("override must fail closed");

            assert_eq!(error.field, field);
            assert_eq!(
                error.rejecting_layer,
                DispatchRejectingLayer::CallerContract
            );
            assert_eq!(
                error.kind,
                DispatchResolutionErrorKind::OverrideNotSupportedForCaller
            );
            assert_eq!(
                error.reason,
                "dispatch-time capability override is unsupported for this field in slice 29.5; only session_resume, session_fork, session_stop, status_snapshot, and event_stream may narrow from true to false"
            );
        }
    }

    #[test]
    fn supported_capability_override_rejects_true_value() {
        let cwd = PathBuf::from(".");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex".to_string(),
            make_entry(
                PathBuf::from("codex.yaml"),
                "codex",
                Some(AgentExecutionScope::Host),
                Some(AgentCliMode::Persistent),
                required_capabilities(),
            ),
        );
        let policy = Policy {
            agents_allowed_backends: vec!["cli:codex".to_string()],
            ..Policy::default()
        };
        let config = SubstrateConfig::default();
        let mut envelope = exact_backend_envelope(
            DispatchCallerKind::HumanStart,
            DispatchBaselineKind::InventoryLaunch,
            "cli:codex",
        );
        envelope.capability_overrides.session_resume = Some(true);

        let error = resolve_inventory_contract_for_exact_backend(
            &cwd,
            &config,
            &inventory,
            &policy,
            &envelope,
            AgentExecutionScope::Host,
        )
        .expect_err("override must fail closed");

        assert_eq!(error.field, "session_resume");
        assert_eq!(
            error.kind,
            DispatchResolutionErrorKind::OverrideExceedsBaseline
        );
    }
}
