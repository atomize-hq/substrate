use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use substrate_broker::{validate_backend_id, Policy};

use crate::execution::agent_inventory::{
    project_inventory_entry, AgentCapabilitiesV1, AgentConfigKind, AgentInventoryBaselineOrigin,
    AgentInventoryEntryV1, ProjectedInventoryEntryV1, ProjectedInventoryValueOrigin,
};
use crate::execution::agent_runtime::orchestration_session::{
    HostAttachContract, HostAttachExecutionClientStart, HostAttachModePreference,
    OrchestrationSessionPosture, OrchestrationSessionState,
};
use crate::execution::config_model::{AgentCliMode, AgentExecutionScope, SubstrateConfig};
use crate::execution::policy_model::{apply_policy_patch, PolicyPatch};

use super::mapping::{
    protocol_validation_error, resolve_shell_owned_runtime_family, AgentRuntimeBackendKind,
    PURE_AGENT_PROTOCOL,
};
use super::session::AgentRuntimeSessionState;

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

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldDispatchActionV1 {
    RunWorldTask,
    SpawnWorldWorker,
    ForkWorldWorker,
    ContinueWorldWorker,
    InspectWorldWorker,
    CancelWorldWork,
    StopWorldWorker,
}

impl WorldDispatchActionV1 {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::RunWorldTask => "run_world_task",
            Self::SpawnWorldWorker => "spawn_world_worker",
            Self::ForkWorldWorker => "fork_world_worker",
            Self::ContinueWorldWorker => "continue_world_worker",
            Self::InspectWorldWorker => "inspect_world_worker",
            Self::CancelWorldWork => "cancel_world_work",
            Self::StopWorldWorker => "stop_world_worker",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldDispatchModeV1 {
    Ephemeral,
    Retained,
}

impl WorldDispatchModeV1 {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Ephemeral => "ephemeral",
            Self::Retained => "retained",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorldDispatchSteeringDenialV1 {
    WorldDispatchDisabled,
    BackendNotAllowed,
    ModeNotAllowed,
    ActionNotAllowed,
    CrossSessionSteeringDenied,
    CrossWorldBindingSteeringDenied,
    CapabilityNarrowingNotAllowed,
    WorkerConcurrencyCapExceeded,
    InvalidatedWorkerNotRoutable,
}

impl WorldDispatchSteeringDenialV1 {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::WorldDispatchDisabled => "world_dispatch_disabled",
            Self::BackendNotAllowed => "backend_not_allowed",
            Self::ModeNotAllowed => "mode_not_allowed",
            Self::ActionNotAllowed => "action_not_allowed",
            Self::CrossSessionSteeringDenied => "cross_session_steering_denied",
            Self::CrossWorldBindingSteeringDenied => "cross_world_binding_steering_denied",
            Self::CapabilityNarrowingNotAllowed => "capability_narrowing_not_allowed",
            Self::WorkerConcurrencyCapExceeded => "worker_concurrency_cap_exceeded",
            Self::InvalidatedWorkerNotRoutable => "invalidated_worker_not_routable",
        }
    }

    pub(crate) fn format_message(self, detail: impl AsRef<str>) -> String {
        let detail = detail.as_ref().trim();
        if detail.is_empty() {
            self.as_str().to_string()
        } else {
            format!("{}: {}", self.as_str(), detail)
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct TaskPayloadV1 {
    pub prompt: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct WorkerSpawnPayloadV1 {
    pub prompt: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerForkPayloadV1 {
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork_strategy: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerContinueApprovalResponsePayloadV1 {
    pub approval_obligation_id: String,
    pub decision: ApprovalResponseDecisionV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerContinueClarificationResponsePayloadV1 {
    pub follow_up_obligation_id: String,
    pub clarification_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerContinueForkCommandPayloadV1 {
    pub child_prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerContinueProgressAckPayloadV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlDirectiveKindV1 {
    Pause,
    ReduceScope,
    Summarize,
    Checkpoint,
    PrepareHandoff,
}

impl ControlDirectiveKindV1 {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::ReduceScope => "reduce_scope",
            Self::Summarize => "summarize",
            Self::Checkpoint => "checkpoint",
            Self::PrepareHandoff => "prepare_handoff",
        }
    }

    #[cfg(any(target_os = "linux", test))]
    fn canonical_instruction(self) -> &'static str {
        match self {
            Self::Pause => "Pause the current line of work and wait for further host guidance.",
            Self::ReduceScope => {
                "Narrow the work to the smallest remaining scope that still moves the task forward."
            }
            Self::Summarize => {
                "Produce a concise summary of the current state, recent progress, and immediate next steps."
            }
            Self::Checkpoint => {
                "Capture a concrete checkpoint of the current state before continuing."
            }
            Self::PrepareHandoff => {
                "Prepare a concise handoff covering current state, next steps, and notable risks."
            }
        }
    }

    #[cfg(any(target_os = "linux", test))]
    fn detail_label(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::ReduceScope => "scope",
            Self::Summarize => "summary",
            Self::Checkpoint => "checkpoint",
            Self::PrepareHandoff => "handoff",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerContinueControlDirectivePayloadV1 {
    pub directive_kind: ControlDirectiveKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directive_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ApprovalResponseDecisionV1 {
    Approve,
    Deny,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct WorkerContinuePayloadV1 {
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerInspectPayloadV1 {}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerCancelPayloadV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graceful: Option<bool>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerStopPayloadV1 {}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "payload_kind", rename_all = "snake_case")]
pub(crate) enum WorldDispatchPayloadV1 {
    Task(TaskPayloadV1),
    WorkerSpawn(WorkerSpawnPayloadV1),
    WorkerFork(WorkerForkPayloadV1),
    WorkerContinue(WorkerContinuePayloadV1),
    WorkerContinueApprovalResponse(WorkerContinueApprovalResponsePayloadV1),
    WorkerContinueClarificationResponse(WorkerContinueClarificationResponsePayloadV1),
    WorkerContinueForkCommand(WorkerContinueForkCommandPayloadV1),
    WorkerContinueProgressAck(WorkerContinueProgressAckPayloadV1),
    WorkerContinueControlDirective(WorkerContinueControlDirectivePayloadV1),
    WorkerInspect(WorkerInspectPayloadV1),
    WorkerCancel(WorkerCancelPayloadV1),
    WorkerStop(WorkerStopPayloadV1),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct WorldDispatchRequestV1 {
    pub request_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub orchestration_session_id: Option<String>,
    pub caller_participant_id: Option<String>,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub target_backend_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_run_id: Option<String>,
    pub target_participant_id: Option<String>,
    pub world_id: Option<String>,
    pub world_generation: Option<u64>,
    pub payload: WorldDispatchPayloadV1,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidatedWorldDispatchRequestV1 {
    pub request_id: String,
    pub idempotency_key: String,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub target_backend_id: String,
    pub target_participant_id: Option<String>,
    pub task_run_id: Option<String>,
    pub world_id: String,
    pub world_generation: u64,
    pub payload: WorldDispatchPayloadV1,
}

impl WorldDispatchRequestV1 {
    pub(crate) fn validate(mut self) -> anyhow::Result<ValidatedWorldDispatchRequestV1> {
        validate_world_dispatch_action_mode(self.action, self.mode)?;
        validate_world_dispatch_payload(self.action, &mut self.payload)?;
        canonicalize_optional_world_dispatch_task_run_id(self.action, &mut self.task_run_id)?;

        let request_id = required_world_dispatch_string("request_id", self.request_id)?;
        let idempotency_key =
            required_world_dispatch_string("idempotency_key", self.idempotency_key)?;
        let orchestration_session_id = required_world_dispatch_string(
            "orchestration_session_id",
            self.orchestration_session_id,
        )?;
        let caller_participant_id =
            required_world_dispatch_string("caller_participant_id", self.caller_participant_id)?;
        let target_participant_id = validate_world_dispatch_target(
            self.action,
            self.mode,
            self.target_participant_id,
            self.task_run_id.as_deref(),
        )?;
        let world_id = required_world_dispatch_string("world_id", self.world_id)?;
        let world_generation = self.world_generation.ok_or_else(|| {
            anyhow::anyhow!(
                "missing_dispatch_field: world dispatch request requires world_generation"
            )
        })?;
        let target_backend_id =
            required_world_dispatch_string("target_backend_id", self.target_backend_id)?;
        validate_backend_id(&target_backend_id).map_err(|err| anyhow::anyhow!(err.to_string()))?;

        Ok(ValidatedWorldDispatchRequestV1 {
            request_id,
            idempotency_key,
            orchestration_session_id,
            caller_participant_id,
            action: self.action,
            mode: self.mode,
            target_backend_id,
            target_participant_id,
            task_run_id: self.task_run_id,
            world_id,
            world_generation,
            payload: self.payload,
        })
    }
}

fn required_world_dispatch_string(
    field: &'static str,
    value: Option<String>,
) -> anyhow::Result<String> {
    let value = value.ok_or_else(|| {
        anyhow::anyhow!("missing_dispatch_field: world dispatch request requires {field}")
    })?;
    if value.trim().is_empty() {
        anyhow::bail!("missing_dispatch_field: world dispatch request requires {field}");
    }
    Ok(value)
}

fn validate_world_dispatch_action_mode(
    action: WorldDispatchActionV1,
    mode: WorldDispatchModeV1,
) -> anyhow::Result<()> {
    match (action, mode) {
        (WorldDispatchActionV1::RunWorldTask, WorldDispatchModeV1::Ephemeral)
        | (WorldDispatchActionV1::SpawnWorldWorker, WorldDispatchModeV1::Retained)
        | (WorldDispatchActionV1::ForkWorldWorker, WorldDispatchModeV1::Retained)
        | (WorldDispatchActionV1::ContinueWorldWorker, WorldDispatchModeV1::Retained)
        | (WorldDispatchActionV1::InspectWorldWorker, WorldDispatchModeV1::Ephemeral)
        | (WorldDispatchActionV1::InspectWorldWorker, WorldDispatchModeV1::Retained)
        | (WorldDispatchActionV1::CancelWorldWork, WorldDispatchModeV1::Ephemeral)
        | (WorldDispatchActionV1::CancelWorldWork, WorldDispatchModeV1::Retained)
        | (WorldDispatchActionV1::StopWorldWorker, WorldDispatchModeV1::Retained) => Ok(()),
        _ => anyhow::bail!(
            "invalid_dispatch_action_mode: action {} is incompatible with mode {}",
            action.as_str(),
            mode.as_str(),
        ),
    }
}

fn validate_world_dispatch_payload(
    action: WorldDispatchActionV1,
    payload: &mut WorldDispatchPayloadV1,
) -> anyhow::Result<()> {
    match (action, payload) {
        (WorldDispatchActionV1::RunWorldTask, WorldDispatchPayloadV1::Task(task)) => {
            validate_world_dispatch_prompt(action, &task.prompt)
        }
        (WorldDispatchActionV1::SpawnWorldWorker, WorldDispatchPayloadV1::WorkerSpawn(worker)) => {
            validate_world_dispatch_prompt(action, &worker.prompt)
        }
        (WorldDispatchActionV1::ForkWorldWorker, WorldDispatchPayloadV1::WorkerFork(worker)) => {
            validate_world_dispatch_prompt(action, &worker.prompt)?;
            validate_optional_world_dispatch_string(action, "fork_reason", &worker.fork_reason)?;
            validate_optional_world_dispatch_string(action, "fork_strategy", &worker.fork_strategy)
        }
        (
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchPayloadV1::WorkerContinue(worker),
        ) => {
            validate_world_dispatch_prompt(action, &worker.prompt)?;
            validate_optional_world_dispatch_string(action, "thread_id", &worker.thread_id)
        }
        (
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchPayloadV1::WorkerContinueApprovalResponse(response),
        ) => validate_approval_response_continue_payload(action, response),
        (
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchPayloadV1::WorkerContinueClarificationResponse(response),
        ) => validate_clarification_response_continue_payload(action, response),
        (
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchPayloadV1::WorkerContinueForkCommand(command),
        ) => validate_fork_command_continue_payload(action, command),
        (
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchPayloadV1::WorkerContinueProgressAck(ack),
        ) => validate_progress_ack_continue_payload(action, ack),
        (
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchPayloadV1::WorkerContinueControlDirective(directive),
        ) => validate_control_directive_continue_payload(action, directive),
        (WorldDispatchActionV1::InspectWorldWorker, WorldDispatchPayloadV1::WorkerInspect(_)) => {
            Ok(())
        }
        (WorldDispatchActionV1::CancelWorldWork, WorldDispatchPayloadV1::WorkerCancel(worker)) => {
            validate_optional_world_dispatch_string(action, "reason", &worker.reason)
        }
        (WorldDispatchActionV1::StopWorldWorker, WorldDispatchPayloadV1::WorkerStop(_)) => Ok(()),
        _ => anyhow::bail!(
            "invalid_dispatch_payload: action {} requires matching typed payload",
            action.as_str(),
        ),
    }
}

fn validate_approval_response_continue_payload(
    action: WorldDispatchActionV1,
    response: &WorkerContinueApprovalResponsePayloadV1,
) -> anyhow::Result<()> {
    if response.approval_obligation_id.trim().is_empty() {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires non-empty approval_obligation_id when payload_kind is worker_continue_approval_response",
            action.as_str(),
        );
    }
    validate_optional_world_dispatch_string(action, "thread_id", &response.thread_id)?;
    Ok(())
}

fn validate_clarification_response_continue_payload(
    action: WorldDispatchActionV1,
    response: &WorkerContinueClarificationResponsePayloadV1,
) -> anyhow::Result<()> {
    if response.follow_up_obligation_id.trim().is_empty() {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires non-empty follow_up_obligation_id when payload_kind is worker_continue_clarification_response",
            action.as_str(),
        );
    }
    if response.clarification_text.trim().is_empty() {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires non-empty clarification_text when payload_kind is worker_continue_clarification_response",
            action.as_str(),
        );
    }
    validate_optional_world_dispatch_string(action, "thread_id", &response.thread_id)?;
    Ok(())
}

fn validate_control_directive_continue_payload(
    action: WorldDispatchActionV1,
    directive: &mut WorkerContinueControlDirectivePayloadV1,
) -> anyhow::Result<()> {
    validate_optional_world_dispatch_string(action, "directive_text", &directive.directive_text)?;
    validate_optional_world_dispatch_string(action, "thread_id", &directive.thread_id)?;
    directive.directive_text = canonicalize_control_directive_detail_fragment(action, directive)?;
    Ok(())
}

fn validate_progress_ack_continue_payload(
    action: WorldDispatchActionV1,
    ack: &WorkerContinueProgressAckPayloadV1,
) -> anyhow::Result<()> {
    validate_optional_world_dispatch_string(action, "thread_id", &ack.thread_id)
}

fn validate_fork_command_continue_payload(
    action: WorldDispatchActionV1,
    command: &mut WorkerContinueForkCommandPayloadV1,
) -> anyhow::Result<()> {
    if command.child_prompt.trim().is_empty() {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires non-empty child_prompt when payload_kind is worker_continue_fork_command",
            action.as_str(),
        );
    }
    command.child_prompt = command.child_prompt.trim().to_string();
    validate_optional_world_dispatch_string(action, "thread_id", &command.thread_id)?;
    command.fork_reason = canonicalize_bounded_fork_command_metadata_label(
        action,
        "fork_reason",
        &command.fork_reason,
    )?;
    command.fork_strategy = canonicalize_bounded_fork_command_metadata_label(
        action,
        "fork_strategy",
        &command.fork_strategy,
    )?;
    Ok(())
}

fn canonicalize_bounded_fork_command_metadata_label(
    action: WorldDispatchActionV1,
    field_name: &'static str,
    value: &Option<String>,
) -> anyhow::Result<Option<String>> {
    let Some(value) = value.as_deref() else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires non-empty {} when provided",
            action.as_str(),
            field_name,
        );
    }
    if trimmed.len() > 64
        || trimmed.contains(char::is_whitespace)
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-'))
    {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires {} to be a bounded metadata label when payload_kind is worker_continue_fork_command",
            action.as_str(),
            field_name,
        );
    }
    Ok(Some(trimmed.to_string()))
}

fn canonicalize_control_directive_detail_fragment(
    action: WorldDispatchActionV1,
    directive: &WorkerContinueControlDirectivePayloadV1,
) -> anyhow::Result<Option<String>> {
    let Some(detail) = directive.directive_text.as_deref() else {
        return Ok(None);
    };

    let trimmed = detail.trim();
    if trimmed.len() > 48
        || trimmed.contains(['\n', '\r', '\t', ' '])
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, ':' | '_'))
    {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires directive_text to be a recognized bounded metadata label for directive_kind {}",
            action.as_str(),
            directive.directive_kind.as_str(),
        );
    }

    let Some((detail_key, detail_value)) = trimmed.split_once(':') else {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires directive_text to be a recognized bounded metadata label for directive_kind {}",
            action.as_str(),
            directive.directive_kind.as_str(),
        );
    };
    if detail_key.is_empty() || detail_value.is_empty() || detail_value.contains(':') {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires directive_text to be a recognized bounded metadata label for directive_kind {}",
            action.as_str(),
            directive.directive_kind.as_str(),
        );
    }

    let recognized = matches!(
        (directive.directive_kind, detail_key, detail_value),
        (
            ControlDirectiveKindV1::Pause,
            "scope",
            "current_branch" | "current_task"
        ) | (
            ControlDirectiveKindV1::Pause,
            "timing",
            "after_current_step"
        ) | (
            ControlDirectiveKindV1::ReduceScope,
            "scope",
            "tests_only" | "single_path" | "current_failure",
        ) | (
            ControlDirectiveKindV1::Summarize,
            "focus",
            "current_state" | "recent_progress" | "next_steps",
        ) | (
            ControlDirectiveKindV1::Checkpoint,
            "artifact",
            "current_state" | "before_retry" | "before_handoff",
        ) | (
            ControlDirectiveKindV1::PrepareHandoff,
            "focus",
            "current_state" | "next_steps" | "known_risks",
        ) | (
            ControlDirectiveKindV1::PrepareHandoff,
            "timing",
            "before_stop"
        )
    );

    if !recognized {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires directive_text to be a recognized bounded metadata label for directive_kind {}",
            action.as_str(),
            directive.directive_kind.as_str(),
        );
    }

    Ok(Some(trimmed.to_string()))
}

#[cfg(any(target_os = "linux", test))]
pub(crate) fn render_continue_world_worker_transport_prompt(
    payload: &WorldDispatchPayloadV1,
) -> anyhow::Result<String> {
    match payload {
        WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 { prompt, .. }) => {
            Ok(prompt.clone())
        }
        WorldDispatchPayloadV1::WorkerContinueApprovalResponse(response) => Ok(
            render_continue_world_worker_approval_response_prompt(response),
        ),
        WorldDispatchPayloadV1::WorkerContinueClarificationResponse(response) => Ok(
            render_continue_world_worker_clarification_response_prompt(response),
        ),
        WorldDispatchPayloadV1::WorkerContinueForkCommand(command) => {
            Ok(render_continue_world_worker_fork_command_prompt(command))
        }
        WorldDispatchPayloadV1::WorkerContinueProgressAck(ack) => {
            Ok(render_continue_world_worker_progress_ack_prompt(ack))
        }
        WorldDispatchPayloadV1::WorkerContinueControlDirective(directive) => {
            render_continue_world_worker_control_directive_prompt(directive)
        }
        _ => anyhow::bail!(
            "invalid_dispatch_payload: action continue_world_worker requires matching typed payload"
        ),
    }
}

#[cfg(any(target_os = "linux", test))]
fn render_continue_world_worker_approval_response_prompt(
    response: &WorkerContinueApprovalResponsePayloadV1,
) -> String {
    let rendered = serde_json::json!({
        "kind": "approval_response",
        "approval_obligation_id": response.approval_obligation_id,
        "decision": match response.decision {
            ApprovalResponseDecisionV1::Approve => "approve",
            ApprovalResponseDecisionV1::Deny => "deny",
        },
        "thread_id": response.thread_id,
    });
    format!(
        "SUBSTRATE_INTERNAL_HOST_APPROVAL_RESPONSE_V1\n{}\nTreat this as the host's typed approval_response for the matching pending approval request. Apply decision=approve as permission granted and decision=deny as permission denied.",
        rendered
    )
}

#[cfg(any(target_os = "linux", test))]
fn render_continue_world_worker_clarification_response_prompt(
    response: &WorkerContinueClarificationResponsePayloadV1,
) -> String {
    let rendered = serde_json::json!({
        "kind": "clarification_response",
        "follow_up_obligation_id": response.follow_up_obligation_id,
        "clarification_text": response.clarification_text,
        "thread_id": response.thread_id,
    });
    format!(
        "SUBSTRATE_INTERNAL_HOST_CLARIFICATION_RESPONSE_V1\n{}\nTreat this as the host's typed clarification_response for the matching pending follow-up obligation. Use clarification_text as authoritative host guidance before continuing work.",
        rendered
    )
}

#[cfg(any(target_os = "linux", test))]
fn render_continue_world_worker_fork_command_prompt(
    command: &WorkerContinueForkCommandPayloadV1,
) -> String {
    let rendered = serde_json::json!({
        "kind": "fork_command",
        "child_prompt": command.child_prompt,
        "fork_reason": command.fork_reason,
        "fork_strategy": command.fork_strategy,
        "thread_id": command.thread_id,
    });
    format!(
        "SUBSTRATE_INTERNAL_HOST_FORK_COMMAND_V1\n{}\nTreat this as the host's typed fork_command for the retained worker. Treat child_prompt as the authoritative child-work intent to prepare for exact-source retained fork bootstrap. Treat fork_reason and fork_strategy only as bounded routing metadata labels for host-mediated fork handling; they do not authorize autonomous child allocation.",
        rendered
    )
}

#[cfg(any(target_os = "linux", test))]
fn render_continue_world_worker_progress_ack_prompt(
    ack: &WorkerContinueProgressAckPayloadV1,
) -> String {
    let rendered = serde_json::json!({
        "kind": "progress_ack",
        "thread_id": ack.thread_id,
    });
    format!(
        "SUBSTRATE_INTERNAL_HOST_PROGRESS_ACK_V1\n{}\nTreat this as the host's typed progress_ack for the retained worker. It only acknowledges that the host saw the worker's recent progress. It does not imply completion, pause, new scope, or durable closeout.",
        rendered
    )
}

#[cfg(any(target_os = "linux", test))]
fn render_continue_world_worker_control_directive_prompt(
    directive: &WorkerContinueControlDirectivePayloadV1,
) -> anyhow::Result<String> {
    let directive_text = canonicalize_control_directive_detail_fragment(
        WorldDispatchActionV1::ContinueWorldWorker,
        directive,
    )?;
    let rendered = serde_json::json!({
        "kind": "control_directive",
        "directive_kind": directive.directive_kind.as_str(),
        "directive_text": directive_text,
        "thread_id": directive.thread_id,
    });
    let detail_guidance = match directive_text.as_deref() {
        Some(_) => {
            let detail_label = directive.directive_kind.detail_label();
            format!(
                "Treat directive_text only as bounded {detail_label} metadata label for this directive kind; it does not add new instructions."
            )
        }
        None => {
            "No directive_text detail was provided beyond the typed directive kind.".to_string()
        }
    };
    Ok(format!(
        "SUBSTRATE_INTERNAL_HOST_CONTROL_DIRECTIVE_V1\n{}\nTreat this as the host's typed control_directive for the retained worker. Apply directive_kind={} as authoritative host guidance. {} {}",
        rendered,
        directive.directive_kind.as_str(),
        directive.directive_kind.canonical_instruction(),
        detail_guidance,
    ))
}

fn validate_world_dispatch_target(
    action: WorldDispatchActionV1,
    mode: WorldDispatchModeV1,
    value: Option<String>,
    task_run_id: Option<&str>,
) -> anyhow::Result<Option<String>> {
    match (action, mode) {
        (WorldDispatchActionV1::ForkWorldWorker, WorldDispatchModeV1::Retained) => {
            reject_task_run_id(action, None, task_run_id)?;
            let value = value.ok_or_else(|| {
                anyhow::anyhow!(
                    "missing_dispatch_field: fork_world_worker requires target_participant_id"
                )
            })?;
            if value.trim().is_empty() {
                anyhow::bail!(
                    "missing_dispatch_field: fork_world_worker requires target_participant_id"
                );
            }
            Ok(Some(value))
        }
        (WorldDispatchActionV1::ContinueWorldWorker, WorldDispatchModeV1::Retained)
        | (WorldDispatchActionV1::StopWorldWorker, WorldDispatchModeV1::Retained)
        | (WorldDispatchActionV1::InspectWorldWorker, WorldDispatchModeV1::Retained)
        | (WorldDispatchActionV1::CancelWorldWork, WorldDispatchModeV1::Retained) => {
            reject_task_run_id(action, Some(mode), task_run_id)?;
            Ok(Some(required_world_dispatch_string(
                "target_participant_id",
                value,
            )?))
        }
        (WorldDispatchActionV1::InspectWorldWorker, WorldDispatchModeV1::Ephemeral)
        | (WorldDispatchActionV1::CancelWorldWork, WorldDispatchModeV1::Ephemeral) => {
            if value.is_some() {
                anyhow::bail!(
                    "invalid_dispatch_target: action {} mode {} does not accept target_participant_id",
                    action.as_str(),
                    mode.as_str(),
                );
            }
            if task_run_id.is_none() {
                anyhow::bail!(
                    "missing_dispatch_field: {} requires task_run_id for mode {}",
                    action.as_str(),
                    mode.as_str(),
                );
            }
            Ok(None)
        }
        _ => {
            if value.is_some() {
                anyhow::bail!(
                    "invalid_dispatch_target: action {} does not accept target_participant_id",
                    action.as_str(),
                );
            }
            reject_task_run_id(action, None, task_run_id)?;
            Ok(None)
        }
    }
}

fn reject_task_run_id(
    action: WorldDispatchActionV1,
    mode: Option<WorldDispatchModeV1>,
    task_run_id: Option<&str>,
) -> anyhow::Result<()> {
    if task_run_id.is_none() {
        return Ok(());
    }
    if let Some(mode) = mode {
        anyhow::bail!(
            "invalid_dispatch_target: action {} mode {} does not accept task_run_id",
            action.as_str(),
            mode.as_str(),
        );
    }
    anyhow::bail!(
        "invalid_dispatch_target: action {} does not accept task_run_id",
        action.as_str(),
    );
}

fn canonicalize_optional_world_dispatch_task_run_id(
    action: WorldDispatchActionV1,
    task_run_id: &mut Option<String>,
) -> anyhow::Result<()> {
    let Some(value) = task_run_id.as_deref() else {
        return Ok(());
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "invalid_dispatch_payload: action {} requires non-empty task_run_id when provided",
            action.as_str(),
        );
    }
    *task_run_id = Some(trimmed.to_string());
    Ok(())
}

fn validate_world_dispatch_prompt(
    action: WorldDispatchActionV1,
    prompt: &str,
) -> anyhow::Result<()> {
    if prompt.trim().is_empty() {
        anyhow::bail!(
            "missing_dispatch_payload: action {} requires non-empty prompt",
            action.as_str(),
        );
    }
    Ok(())
}

fn validate_optional_world_dispatch_string(
    action: WorldDispatchActionV1,
    field: &'static str,
    value: &Option<String>,
) -> anyhow::Result<()> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            anyhow::bail!(
                "invalid_dispatch_payload: action {} requires non-empty {field} when provided",
                action.as_str(),
            );
        }
    }
    Ok(())
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldTaskTerminalStateV1 {
    Completed,
    Failed,
    Cancelled,
    NeedsRetainedFollowup,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct RunWorldTaskOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_run_id: Option<String>,
    pub state: WorldTaskTerminalStateV1,
    pub summary: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct SpawnWorldWorkerOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    pub parent_participant_id: Option<String>,
    pub resumed_from_participant_id: Option<String>,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub launch_span_id: String,
    pub summary: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ForkWorldWorkerOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    pub source_participant_id: String,
    pub child_participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub summary: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ContinueWorldWorkerOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    pub target_participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_event: Option<ContinueWorldWorkerEventV1>,
    pub summary: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct RetainedWorkerInspectSnapshotV1 {
    pub participant_state: AgentRuntimeSessionState,
    pub session_state: OrchestrationSessionState,
    pub session_posture: OrchestrationSessionPosture,
    pub authoritative_live: bool,
    pub attention_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_participant_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct InspectWorldWorkerOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    pub target_participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub snapshot: RetainedWorkerInspectSnapshotV1,
    pub summary: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct RetainedWorkerCancelCloseoutV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_state: Option<AgentRuntimeSessionState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_state: Option<OrchestrationSessionState>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CancelWorldWorkTerminalStateV1 {
    Cancelled,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CancelWorldWorkOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    pub target_participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub state: CancelWorldWorkTerminalStateV1,
    pub closeout: RetainedWorkerCancelCloseoutV1,
    pub summary: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct RetainedWorkerStopCloseoutV1 {
    pub participant_state: AgentRuntimeSessionState,
    pub session_state: OrchestrationSessionState,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct StopWorldWorkerOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    pub target_participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub closeout: RetainedWorkerStopCloseoutV1,
    pub summary: String,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ContinueWorldWorkerEventClassV1 {
    Reply,
    ProgressUpdate,
    Result,
    Failure,
    ControlAck,
    FollowUpQuestion,
    Blocked,
    ApprovalRequest,
    ForkRequest,
    ForkRecommendation,
}

#[cfg(any(target_os = "linux", test))]
impl ContinueWorldWorkerEventClassV1 {
    pub(crate) fn from_wire_label(label: &str) -> Option<Self> {
        match label.trim() {
            "reply" => Some(Self::Reply),
            "progress_update" => Some(Self::ProgressUpdate),
            "result" => Some(Self::Result),
            "failure" => Some(Self::Failure),
            "control_ack" => Some(Self::ControlAck),
            "follow_up_question" => Some(Self::FollowUpQuestion),
            "blocked" => Some(Self::Blocked),
            "approval_request" => Some(Self::ApprovalRequest),
            "fork_request" => Some(Self::ForkRequest),
            "fork_recommendation" => Some(Self::ForkRecommendation),
            _ => None,
        }
    }

    pub(crate) fn attention_required_by_default(self) -> bool {
        matches!(
            self,
            Self::FollowUpQuestion | Self::Blocked | Self::ApprovalRequest | Self::ForkRequest
        )
    }

    pub(crate) fn is_deferred_wire_label(label: &str) -> bool {
        matches!(
            label.trim(),
            "approval_response"
                | "fork_command"
                | "progress_ack"
                | "control_directive"
                | "attention_required"
        )
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ContinueWorldWorkerEventV1 {
    pub event_class: ContinueWorldWorkerEventClassV1,
    pub source_participant_id: String,
    pub target_participant_id: String,
    pub source_backend_id: String,
    pub attention_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_channel: Option<String>,
    pub payload: serde_json::Value,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "outcome_kind", rename_all = "snake_case")]
pub(crate) enum WorldDispatchOutcomeV1 {
    RunWorldTask(RunWorldTaskOutcomeV1),
    SpawnWorldWorker(SpawnWorldWorkerOutcomeV1),
    ForkWorldWorker(ForkWorldWorkerOutcomeV1),
    ContinueWorldWorker(ContinueWorldWorkerOutcomeV1),
    InspectWorldWorker(InspectWorldWorkerOutcomeV1),
    CancelWorldWork(CancelWorldWorkOutcomeV1),
    StopWorldWorker(StopWorldWorkerOutcomeV1),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LiveToolValidationState {
    SmokeValidated,
    NotYetSmokeValidated,
}

impl LiveToolValidationState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::SmokeValidated => "smoke_validated",
            Self::NotYetSmokeValidated => "not_yet_smoke_validated",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LiveToolSupportState {
    FirstSupportedFloor,
    SelectedRuntimeSupported,
    NotYetGuaranteed,
}

impl LiveToolSupportState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::FirstSupportedFloor => "first_supported_floor",
            Self::SelectedRuntimeSupported => "selected_runtime_supported",
            Self::NotYetGuaranteed => "not_yet_guaranteed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectedClaudeCodePathState {
    #[allow(dead_code)]
    Present,
    Missing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Slice52SemanticsState {
    #[allow(dead_code)]
    Preserved,
    NotYetProven,
    #[allow(dead_code)]
    NotPreserved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetedValidationState {
    #[allow(dead_code)]
    Green,
    NotYetGreen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HiddenFallbackState {
    #[allow(dead_code)]
    NoHiddenFallback,
    NotYetProven,
    #[allow(dead_code)]
    HiddenFallbackPresent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SelectedClaudeCodeUpliftGate {
    pub selected_start_turn_path: SelectedClaudeCodePathState,
    pub slice_52_semantics: Slice52SemanticsState,
    pub targeted_validation: TargetedValidationState,
    pub hidden_fallback: HiddenFallbackState,
}

impl SelectedClaudeCodeUpliftGate {
    pub(crate) const fn packet_1() -> Self {
        Self {
            selected_start_turn_path: SelectedClaudeCodePathState::Missing,
            slice_52_semantics: Slice52SemanticsState::NotYetProven,
            targeted_validation: TargetedValidationState::NotYetGreen,
            hidden_fallback: HiddenFallbackState::NotYetProven,
        }
    }

    pub(crate) const fn allows_selected_runtime_enablement(
        self,
        selected_claude_code_uplift_context: SelectedClaudeCodeUpliftContext,
    ) -> bool {
        selected_claude_code_uplift_context.is_selected_host_launch()
            && matches!(
                self.selected_start_turn_path,
                SelectedClaudeCodePathState::Present
            )
            && matches!(self.slice_52_semantics, Slice52SemanticsState::Preserved)
            && matches!(self.targeted_validation, TargetedValidationState::Green)
            && matches!(self.hidden_fallback, HiddenFallbackState::NoHiddenFallback)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectedClaudeCodeUpliftContext {
    InventoryEntry,
    SelectedLaunch {
        execution_scope: AgentExecutionScope,
    },
}

impl SelectedClaudeCodeUpliftContext {
    pub(crate) const fn inventory_entry() -> Self {
        Self::InventoryEntry
    }

    pub(crate) const fn selected_launch(execution_scope: AgentExecutionScope) -> Self {
        Self::SelectedLaunch { execution_scope }
    }

    pub(crate) const fn is_selected_host_launch(self) -> bool {
        matches!(
            self,
            Self::SelectedLaunch {
                execution_scope: AgentExecutionScope::Host,
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LiveToolSupportPosture {
    pub runtime_family: AgentRuntimeBackendKind,
    pub validation_state: LiveToolValidationState,
    pub support_state: LiveToolSupportState,
    pub selected_claude_code_uplift_gate: SelectedClaudeCodeUpliftGate,
    pub selected_claude_code_uplift_context: SelectedClaudeCodeUpliftContext,
    pub reason: &'static str,
}

impl LiveToolSupportPosture {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn for_backend_kind(backend_kind: AgentRuntimeBackendKind) -> Self {
        Self::for_backend_kind_with_selected_claude_code_uplift_gate(
            backend_kind,
            SelectedClaudeCodeUpliftContext::inventory_entry(),
            SelectedClaudeCodeUpliftGate::packet_1(),
        )
    }

    pub(crate) fn for_selected_launch_backend_kind(
        backend_kind: AgentRuntimeBackendKind,
        execution_scope: AgentExecutionScope,
    ) -> Self {
        Self::for_backend_kind_with_selected_claude_code_uplift_gate(
            backend_kind,
            SelectedClaudeCodeUpliftContext::selected_launch(execution_scope),
            SelectedClaudeCodeUpliftGate::packet_1(),
        )
    }

    fn for_backend_kind_with_selected_claude_code_uplift_gate(
        backend_kind: AgentRuntimeBackendKind,
        selected_claude_code_uplift_context: SelectedClaudeCodeUpliftContext,
        selected_claude_code_uplift_gate: SelectedClaudeCodeUpliftGate,
    ) -> Self {
        match backend_kind {
            AgentRuntimeBackendKind::Codex => Self {
                runtime_family: AgentRuntimeBackendKind::Codex,
                validation_state: LiveToolValidationState::SmokeValidated,
                support_state: LiveToolSupportState::FirstSupportedFloor,
                selected_claude_code_uplift_gate,
                selected_claude_code_uplift_context,
                reason:
                    "codex remains the first smoke-validated host-tool floor from Slice 53; this does not uplift claude_code until the selected claude_code host start/turn path reaches the same authoritative host-tool surface, preserves Slice 52 semantics, and passes targeted validation without hidden fallback",
            },
            AgentRuntimeBackendKind::ClaudeCode => {
                let (validation_state, support_state, reason) =
                    if selected_claude_code_uplift_gate
                        .allows_selected_runtime_enablement(selected_claude_code_uplift_context)
                    {
                        (
                            LiveToolValidationState::SmokeValidated,
                            LiveToolSupportState::SelectedRuntimeSupported,
                            "claude_code selected-runtime host-tool parity gate is open: the selected claude_code host start/turn path exists, preserves Slice 52 semantics, passed targeted validation, and proved no hidden fallback to codex",
                        )
                    } else {
                        (
                            LiveToolValidationState::NotYetSmokeValidated,
                            LiveToolSupportState::NotYetGuaranteed,
                            "claude_code host-tool parity target is the selected claude_code host start/turn path taking the same authoritative host-tool surface and Slice 52 semantics without hidden fallback to codex; keep reporting not yet smoke-validated and not yet guaranteed until that exact selected-runtime path exists and targeted validation is green; ordinary host-session behavior remains unchanged unless implementation truth proves an incompatibility",
                        )
                    };

                Self {
                    runtime_family: AgentRuntimeBackendKind::ClaudeCode,
                    validation_state,
                    support_state,
                    selected_claude_code_uplift_gate,
                    selected_claude_code_uplift_context,
                    reason,
                }
            }
        }
    }
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

    let backend_kind = resolve_shell_owned_runtime_family(
        projected.agent_id.as_str(),
        projected.cli_runtime_family,
    )
    .map_err(|err| DispatchResolutionError {
        kind: DispatchResolutionErrorKind::RuntimeUnrealizableAfterResolution,
        field: "config.cli.runtime_family",
        rejecting_layer: DispatchRejectingLayer::RuntimeMaterialization,
        reason: err.to_string(),
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
                reason: "dispatch-time capability override is unsupported for this field in slice 29.75; only session_resume, session_fork, session_stop, status_snapshot, and event_stream may narrow from true to false".to_string(),
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
                    "dispatch-time capability override must be narrowing-only; only true-to-false is supported in slice 29.75"
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
                    "dispatch-time capability override must be narrowing-only; only true-to-false is supported in slice 29.75"
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
        render_continue_world_worker_transport_prompt,
        resolve_inventory_contract_for_exact_backend, resolve_persisted_host_attach_contract,
        AgentRuntimeBackendKind, ApprovalResponseDecisionV1, AttachLaunchKnobs,
        AttachModePreference, CancelWorldWorkOutcomeV1, CancelWorldWorkTerminalStateV1,
        ContinueWorldWorkerEventClassV1, ContinueWorldWorkerOutcomeV1, ControlDirectiveKindV1,
        DispatchBaselineKind, DispatchCallerKind, DispatchCapabilityOverrideSet,
        DispatchRejectingLayer, DispatchRequestEnvelope, DispatchResolutionErrorKind,
        FieldBaselineOrigin, FieldValueOrigin, ForkWorldWorkerOutcomeV1, HiddenFallbackState,
        HostExecutionClientStart, InspectWorldWorkerOutcomeV1, LiveToolSupportPosture,
        LiveToolSupportState, LiveToolValidationState, RetainedWorkerCancelCloseoutV1,
        RetainedWorkerInspectSnapshotV1, RetainedWorkerStopCloseoutV1, RunWorldTaskOutcomeV1,
        SelectedClaudeCodePathState, SelectedClaudeCodeUpliftContext, SelectedClaudeCodeUpliftGate,
        Slice52SemanticsState, StopWorldWorkerOutcomeV1, TargetedValidationState, TaskPayloadV1,
        WorkerCancelPayloadV1, WorkerContinueApprovalResponsePayloadV1,
        WorkerContinueClarificationResponsePayloadV1, WorkerContinueControlDirectivePayloadV1,
        WorkerContinueForkCommandPayloadV1, WorkerContinuePayloadV1,
        WorkerContinueProgressAckPayloadV1, WorkerForkPayloadV1, WorkerInspectPayloadV1,
        WorkerSpawnPayloadV1, WorkerStopPayloadV1, WorldDispatchActionV1, WorldDispatchModeV1,
        WorldDispatchOutcomeV1, WorldDispatchPayloadV1, WorldDispatchRequestV1,
        WorldDispatchSteeringDenialV1, WorldTaskTerminalStateV1,
    };
    use crate::execution::agent_inventory::{
        AgentCapabilitiesV1, AgentCliConfigV1, AgentCliRuntimeFamily, AgentConfigKind,
        AgentConfigV1, AgentExecutionConfigV1, AgentFileV1, AgentInventoryEntryV1,
    };
    use crate::execution::agent_runtime::control::{
        ResolvedRuntimeBackendKind, ResolvedRuntimeDescriptor,
    };
    use crate::execution::agent_runtime::orchestration_session::{
        HostAttachContract, OrchestrationSessionPosture, OrchestrationSessionState,
    };
    use crate::execution::agent_runtime::session::AgentRuntimeSessionState;
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
                        runtime_family: Some(AgentCliRuntimeFamily::Codex),
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
    fn live_tool_support_posture_freezes_packet_1_uplift_gate_in_repo_truth() {
        let gate = SelectedClaudeCodeUpliftGate::packet_1();

        assert_eq!(
            gate.selected_start_turn_path,
            SelectedClaudeCodePathState::Missing
        );
        assert_eq!(gate.slice_52_semantics, Slice52SemanticsState::NotYetProven);
        assert_eq!(
            gate.targeted_validation,
            TargetedValidationState::NotYetGreen
        );
        assert_eq!(gate.hidden_fallback, HiddenFallbackState::NotYetProven);
        assert!(
            !gate.allows_selected_runtime_enablement(
                SelectedClaudeCodeUpliftContext::selected_launch(AgentExecutionScope::Host)
            ),
            "Packet 1 must keep selected claude_code uplift closed until every gate criterion is satisfied"
        );
    }

    #[test]
    fn live_tool_support_posture_requires_semantics_and_fallback_evidence_before_enablement() {
        let gate = SelectedClaudeCodeUpliftGate {
            selected_start_turn_path: SelectedClaudeCodePathState::Present,
            targeted_validation: TargetedValidationState::Green,
            ..SelectedClaudeCodeUpliftGate::packet_1()
        };

        assert_eq!(gate.slice_52_semantics, Slice52SemanticsState::NotYetProven);
        assert_eq!(gate.hidden_fallback, HiddenFallbackState::NotYetProven);
        assert!(
            !gate.allows_selected_runtime_enablement(
                SelectedClaudeCodeUpliftContext::selected_launch(AgentExecutionScope::Host)
            ),
            "Packet 1 must fail closed until semantics preservation and no-hidden-fallback evidence are both proven in code"
        );
    }

    #[test]
    fn live_tool_support_posture_marks_codex_as_first_validated_floor() {
        let posture = LiveToolSupportPosture::for_backend_kind(AgentRuntimeBackendKind::Codex);

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
            posture.selected_claude_code_uplift_gate,
            SelectedClaudeCodeUpliftGate::packet_1()
        );
        assert_eq!(
            posture.selected_claude_code_uplift_context,
            SelectedClaudeCodeUpliftContext::inventory_entry()
        );
    }

    #[test]
    fn live_tool_support_posture_keeps_non_codex_runtime_truthful_without_blocking() {
        let posture = LiveToolSupportPosture::for_backend_kind(AgentRuntimeBackendKind::ClaudeCode);

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
            posture.selected_claude_code_uplift_gate,
            SelectedClaudeCodeUpliftGate::packet_1()
        );
        assert_eq!(
            posture.selected_claude_code_uplift_context,
            SelectedClaudeCodeUpliftContext::inventory_entry()
        );
        assert!(!posture
            .selected_claude_code_uplift_gate
            .allows_selected_runtime_enablement(posture.selected_claude_code_uplift_context));
    }

    #[test]
    fn live_tool_support_posture_keeps_inventory_entry_context_from_overclaiming_when_gate_is_green(
    ) {
        let posture =
            LiveToolSupportPosture::for_backend_kind_with_selected_claude_code_uplift_gate(
                AgentRuntimeBackendKind::ClaudeCode,
                SelectedClaudeCodeUpliftContext::inventory_entry(),
                SelectedClaudeCodeUpliftGate {
                    selected_start_turn_path: SelectedClaudeCodePathState::Present,
                    slice_52_semantics: Slice52SemanticsState::Preserved,
                    targeted_validation: TargetedValidationState::Green,
                    hidden_fallback: HiddenFallbackState::NoHiddenFallback,
                },
            );

        assert_eq!(
            posture.support_state,
            LiveToolSupportState::NotYetGuaranteed
        );
        assert_eq!(
            posture.validation_state,
            LiveToolValidationState::NotYetSmokeValidated
        );
        assert_eq!(
            posture.selected_claude_code_uplift_context,
            SelectedClaudeCodeUpliftContext::inventory_entry()
        );
    }

    #[test]
    fn live_tool_support_posture_keeps_selected_world_scope_from_overclaiming_when_gate_is_green() {
        let posture =
            LiveToolSupportPosture::for_backend_kind_with_selected_claude_code_uplift_gate(
                AgentRuntimeBackendKind::ClaudeCode,
                SelectedClaudeCodeUpliftContext::selected_launch(AgentExecutionScope::World),
                SelectedClaudeCodeUpliftGate {
                    selected_start_turn_path: SelectedClaudeCodePathState::Present,
                    slice_52_semantics: Slice52SemanticsState::Preserved,
                    targeted_validation: TargetedValidationState::Green,
                    hidden_fallback: HiddenFallbackState::NoHiddenFallback,
                },
            );

        assert_eq!(
            posture.support_state,
            LiveToolSupportState::NotYetGuaranteed
        );
        assert_eq!(
            posture.validation_state,
            LiveToolValidationState::NotYetSmokeValidated
        );
        assert_eq!(
            posture.selected_claude_code_uplift_context,
            SelectedClaudeCodeUpliftContext::selected_launch(AgentExecutionScope::World)
        );
    }

    #[test]
    fn live_tool_support_posture_derives_claude_code_enablement_from_selected_host_gate_context() {
        let posture =
            LiveToolSupportPosture::for_backend_kind_with_selected_claude_code_uplift_gate(
                AgentRuntimeBackendKind::ClaudeCode,
                SelectedClaudeCodeUpliftContext::selected_launch(AgentExecutionScope::Host),
                SelectedClaudeCodeUpliftGate {
                    selected_start_turn_path: SelectedClaudeCodePathState::Present,
                    slice_52_semantics: Slice52SemanticsState::Preserved,
                    targeted_validation: TargetedValidationState::Green,
                    hidden_fallback: HiddenFallbackState::NoHiddenFallback,
                },
            );

        assert_eq!(posture.runtime_family, AgentRuntimeBackendKind::ClaudeCode);
        assert_eq!(
            posture.validation_state,
            LiveToolValidationState::SmokeValidated
        );
        assert_eq!(
            posture.support_state,
            LiveToolSupportState::SelectedRuntimeSupported
        );
        assert!(posture
            .selected_claude_code_uplift_gate
            .allows_selected_runtime_enablement(posture.selected_claude_code_uplift_context));
    }

    #[test]
    fn exact_backend_alias_resolves_canonical_runtime_family_from_inventory_truth() {
        let cwd = PathBuf::from(".");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex_world".to_string(),
            AgentInventoryEntryV1 {
                path: PathBuf::from("codex_world.yaml"),
                file: AgentFileV1 {
                    version: 1,
                    id: "codex_world".to_string(),
                    config: AgentConfigV1 {
                        enabled: true,
                        kind: AgentConfigKind::Cli,
                        protocol: Some(super::PURE_AGENT_PROTOCOL.to_string()),
                        execution: AgentExecutionConfigV1 {
                            scope: Some(AgentExecutionScope::World),
                        },
                        cli: Some(AgentCliConfigV1 {
                            binary: "cargo".to_string(),
                            mode: Some(AgentCliMode::Persistent),
                            runtime_family: Some(AgentCliRuntimeFamily::Codex),
                        }),
                        api: None,
                        capabilities: required_capabilities(),
                    },
                    policy_overlay: None,
                },
            },
        );
        let policy = Policy {
            agents_allowed_backends: vec!["cli:codex_world".to_string()],
            ..Policy::default()
        };
        let config = SubstrateConfig::default();

        let resolved = resolve_inventory_contract_for_exact_backend(
            &cwd,
            &config,
            &inventory,
            &policy,
            &exact_backend_envelope(
                DispatchCallerKind::OrchestratorMemberStart,
                DispatchBaselineKind::InventoryLaunch,
                "cli:codex_world",
            ),
            AgentExecutionScope::World,
        )
        .expect("resolution should succeed")
        .expect("contract");

        assert_eq!(resolved.agent_id, "codex_world");
        assert_eq!(resolved.backend_id, "cli:codex_world");
        assert_eq!(resolved.backend_kind, AgentRuntimeBackendKind::Codex);
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
                "dispatch-time capability override is unsupported for this field in slice 29.75; only session_resume, session_fork, session_stop, status_snapshot, and event_stream may narrow from true to false"
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

    fn base_world_dispatch_request(
        action: WorldDispatchActionV1,
        mode: WorldDispatchModeV1,
        payload: WorldDispatchPayloadV1,
    ) -> WorldDispatchRequestV1 {
        WorldDispatchRequestV1 {
            request_id: Some("req-32".to_string()),
            idempotency_key: Some("idem-32".to_string()),
            orchestration_session_id: Some("sess-32".to_string()),
            caller_participant_id: Some("orch-32".to_string()),
            action,
            mode,
            target_backend_id: Some("cli:codex_world".to_string()),
            task_run_id: None,
            target_participant_id: None,
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            payload,
        }
    }

    trait WorldDispatchRequestTestExt {
        fn with_target_participant_id(self, participant_id: &str) -> Self;
        fn with_task_run_id(self, task_run_id: &str) -> Self;
    }

    impl WorldDispatchRequestTestExt for WorldDispatchRequestV1 {
        fn with_target_participant_id(mut self, participant_id: &str) -> Self {
            self.target_participant_id = Some(participant_id.to_string());
            self
        }

        fn with_task_run_id(mut self, task_run_id: &str) -> Self {
            self.task_run_id = Some(task_run_id.to_string());
            self
        }
    }

    #[test]
    fn world_dispatch_contract_accepts_run_world_task_ephemeral_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::RunWorldTask,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::Task(TaskPayloadV1 {
                prompt: "index the repo".to_string(),
            }),
        )
        .validate()
        .expect("request should validate");

        assert_eq!(validated.request_id, "req-32");
        assert_eq!(validated.target_backend_id, "cli:codex_world");
        assert_eq!(validated.world_generation, 2);
    }

    #[test]
    fn world_dispatch_contract_accepts_spawn_world_worker_retained_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::SpawnWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
                prompt: "own the failing integration investigation".to_string(),
            }),
        )
        .validate()
        .expect("request should validate");

        assert_eq!(validated.orchestration_session_id, "sess-32");
        assert_eq!(validated.caller_participant_id, "orch-32");
    }

    #[test]
    fn world_dispatch_contract_accepts_fork_world_worker_retained_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ForkWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
                prompt: "split off the flaky integration investigation".to_string(),
                fork_reason: Some("parallelize root cause isolation".to_string()),
                fork_strategy: Some("exact_source_retained".to_string()),
            }),
        )
        .with_target_participant_id("ash-worker-source-38")
        .validate()
        .expect("fork request should validate");

        assert_eq!(
            validated.target_participant_id.as_deref(),
            Some("ash-worker-source-38")
        );
    }

    #[test]
    fn world_dispatch_contract_accepts_continue_world_worker_retained_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                prompt: "continue with the integration trace".to_string(),
                thread_id: Some("thread-root".to_string()),
            }),
        )
        .with_target_participant_id("ash-worker-32")
        .validate()
        .expect("continue request should validate");

        assert_eq!(
            validated.target_participant_id.as_deref(),
            Some("ash-worker-32")
        );
    }

    #[test]
    fn world_dispatch_contract_accepts_inspect_world_worker_retained_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::InspectWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        )
        .with_target_participant_id("ash-worker-35")
        .validate()
        .expect("inspect request should validate");

        assert_eq!(
            validated.target_participant_id.as_deref(),
            Some("ash-worker-35")
        );
        assert_eq!(validated.task_run_id, None);
    }

    #[test]
    fn world_dispatch_contract_accepts_inspect_world_worker_ephemeral_shape_with_exact_task_identity(
    ) {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::InspectWorldWorker,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        )
        .with_task_run_id("task-run-35")
        .validate()
        .expect("ephemeral inspect should validate with exact task identity");

        assert_eq!(validated.target_participant_id, None);
        assert_eq!(validated.task_run_id.as_deref(), Some("task-run-35"));
    }

    #[test]
    fn world_dispatch_contract_accepts_cancel_world_work_retained_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::CancelWorldWork,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1 {
                reason: Some("operator requested cancel".to_string()),
                graceful: Some(true),
            }),
        )
        .with_target_participant_id("ash-worker-37")
        .validate()
        .expect("cancel request should validate");

        assert_eq!(
            validated.target_participant_id.as_deref(),
            Some("ash-worker-37")
        );
        assert_eq!(validated.task_run_id, None);
    }

    #[test]
    fn world_dispatch_contract_accepts_cancel_world_work_ephemeral_shape_with_exact_task_identity()
    {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::CancelWorldWork,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
        )
        .with_task_run_id("task-run-37")
        .validate()
        .expect("ephemeral cancel should validate with exact task identity");

        assert_eq!(validated.target_participant_id, None);
        assert_eq!(validated.task_run_id.as_deref(), Some("task-run-37"));
    }

    #[test]
    fn world_dispatch_contract_accepts_stop_world_worker_retained_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::StopWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
        )
        .with_target_participant_id("ash-worker-36")
        .validate()
        .expect("stop request should validate");

        assert_eq!(
            validated.target_participant_id.as_deref(),
            Some("ash-worker-36")
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_invalid_action_mode_combination() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::RunWorldTask,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::Task(TaskPayloadV1 {
                prompt: "index the repo".to_string(),
            }),
        )
        .validate()
        .expect_err("action/mode mismatch must fail closed");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_action_mode: action run_world_task is incompatible with mode retained"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_missing_required_identity_fields() {
        let request = base_world_dispatch_request(
            WorldDispatchActionV1::SpawnWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
                prompt: "own the failing integration investigation".to_string(),
            }),
        );

        let mut orchestration_request = request.clone();
        orchestration_request.orchestration_session_id = None;
        let orchestration_error = orchestration_request
            .validate()
            .expect_err("missing orchestration session must fail");
        assert_eq!(
            orchestration_error.to_string(),
            "missing_dispatch_field: world dispatch request requires orchestration_session_id"
        );

        let mut caller_request = request.clone();
        caller_request.caller_participant_id = Some(" ".to_string());
        let caller_error = caller_request
            .validate()
            .expect_err("blank caller participant must fail");
        assert_eq!(
            caller_error.to_string(),
            "missing_dispatch_field: world dispatch request requires caller_participant_id"
        );

        let mut backend_request = request.clone();
        backend_request.target_backend_id = None;
        let backend_error = backend_request
            .validate()
            .expect_err("missing target backend must fail");
        assert_eq!(
            backend_error.to_string(),
            "missing_dispatch_field: world dispatch request requires target_backend_id"
        );

        let mut world_id_request = request.clone();
        world_id_request.world_id = Some(String::new());
        let world_id_error = world_id_request
            .validate()
            .expect_err("blank world id must fail");
        assert_eq!(
            world_id_error.to_string(),
            "missing_dispatch_field: world dispatch request requires world_id"
        );

        let mut world_generation_request = request;
        world_generation_request.world_generation = None;
        let world_generation_error = world_generation_request
            .validate()
            .expect_err("missing world_generation must fail");
        assert_eq!(
            world_generation_error.to_string(),
            "missing_dispatch_field: world dispatch request requires world_generation"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_continue_world_worker_without_exact_target() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                prompt: "continue with the integration trace".to_string(),
                thread_id: None,
            }),
        )
        .validate()
        .expect_err("continue target must be mandatory");

        assert_eq!(
            error.to_string(),
            "missing_dispatch_field: world dispatch request requires target_participant_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_fork_world_worker_without_exact_target() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ForkWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
                prompt: "split off the flaky integration investigation".to_string(),
                fork_reason: None,
                fork_strategy: None,
            }),
        )
        .validate()
        .expect_err("fork target must be mandatory");

        assert_eq!(
            error.to_string(),
            "missing_dispatch_field: fork_world_worker requires target_participant_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_fork_world_worker_ephemeral_mode() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ForkWorldWorker,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
                prompt: "split off the flaky integration investigation".to_string(),
                fork_reason: None,
                fork_strategy: None,
            }),
        )
        .with_target_participant_id("ash-worker-source-38")
        .validate()
        .expect_err("fork must stay retained-only in packet 1");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_action_mode: action fork_world_worker is incompatible with mode ephemeral"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_inspect_world_worker_without_exact_target() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::InspectWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        )
        .validate()
        .expect_err("inspect target must be mandatory");

        assert_eq!(
            error.to_string(),
            "missing_dispatch_field: world dispatch request requires target_participant_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_inspect_world_worker_ephemeral_mode_without_exact_task_identity(
    ) {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::InspectWorldWorker,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        )
        .validate()
        .expect_err("ephemeral inspect must fail closed without task identity");

        assert_eq!(
            error.to_string(),
            "missing_dispatch_field: inspect_world_worker requires task_run_id for mode ephemeral"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_inspect_world_worker_retained_mode_with_task_run_id() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::InspectWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        )
        .with_target_participant_id("ash-worker-35")
        .with_task_run_id("task-run-35")
        .validate()
        .expect_err("retained inspect must reject task identity");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_target: action inspect_world_worker mode retained does not accept task_run_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_inspect_world_worker_ephemeral_mode_with_mixed_identity() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::InspectWorldWorker,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        )
        .with_target_participant_id("ash-worker-35")
        .with_task_run_id("task-run-35")
        .validate()
        .expect_err("ephemeral inspect must reject mixed identity");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_target: action inspect_world_worker mode ephemeral does not accept target_participant_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_cancel_world_work_without_exact_target() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::CancelWorldWork,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
        )
        .validate()
        .expect_err("cancel target must be mandatory");

        assert_eq!(
            error.to_string(),
            "missing_dispatch_field: world dispatch request requires target_participant_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_cancel_world_work_ephemeral_mode_without_exact_task_identity(
    ) {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::CancelWorldWork,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
        )
        .validate()
        .expect_err("ephemeral cancel must fail closed without task identity");

        assert_eq!(
            error.to_string(),
            "missing_dispatch_field: cancel_world_work requires task_run_id for mode ephemeral"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_cancel_world_work_retained_mode_with_task_run_id() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::CancelWorldWork,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
        )
        .with_target_participant_id("ash-worker-37")
        .with_task_run_id("task-run-37")
        .validate()
        .expect_err("retained cancel must reject task identity");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_target: action cancel_world_work mode retained does not accept task_run_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_cancel_world_work_ephemeral_mode_with_mixed_identity() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::CancelWorldWork,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
        )
        .with_target_participant_id("ash-worker-37")
        .with_task_run_id("task-run-37")
        .validate()
        .expect_err("ephemeral cancel must reject mixed identity");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_target: action cancel_world_work mode ephemeral does not accept target_participant_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_stop_world_worker_without_exact_target() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::StopWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
        )
        .validate()
        .expect_err("stop target must be mandatory");

        assert_eq!(
            error.to_string(),
            "missing_dispatch_field: world dispatch request requires target_participant_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_stop_world_worker_ephemeral_mode() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::StopWorldWorker,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
        )
        .with_target_participant_id("ash-worker-36")
        .validate()
        .expect_err("stop must stay retained-only in packet 1");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_action_mode: action stop_world_worker is incompatible with mode ephemeral"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_unknown_inspect_payload_fields_during_deserialization() {
        let error = serde_json::from_value::<WorldDispatchRequestV1>(serde_json::json!({
            "request_id": "req-32",
            "idempotency_key": "idem-32",
            "orchestration_session_id": "sess-32",
            "caller_participant_id": "orch-32",
            "action": "inspect_world_worker",
            "mode": "retained",
            "target_backend_id": "cli:codex_world",
            "target_participant_id": "ash-worker-35",
            "world_id": "world-17",
            "world_generation": 2,
            "payload": {
                "payload_kind": "worker_inspect",
                "future_runtime_selector": "packet-2"
            }
        }))
        .expect_err("unknown inspect payload fields must fail closed");

        assert!(
            error
                .to_string()
                .contains("unknown field `future_runtime_selector`"),
            "unexpected inspect payload serde error: {error}"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_unknown_fork_payload_fields_during_deserialization() {
        let error = serde_json::from_value::<WorldDispatchRequestV1>(serde_json::json!({
            "request_id": "req-38",
            "idempotency_key": "idem-38",
            "orchestration_session_id": "sess-38",
            "caller_participant_id": "orch-38",
            "action": "fork_world_worker",
            "mode": "retained",
            "target_backend_id": "cli:codex_world",
            "target_participant_id": "ash-worker-source-38",
            "world_id": "world-38",
            "world_generation": 8,
            "payload": {
                "payload_kind": "worker_fork",
                "prompt": "split off the flaky integration investigation",
                "future_child_allocator": "packet-3"
            }
        }))
        .expect_err("unknown fork payload fields must fail closed");

        assert!(
            error
                .to_string()
                .contains("unknown field `future_child_allocator`"),
            "unexpected fork payload serde error: {error}"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_unknown_cancel_payload_fields_during_deserialization() {
        let error = serde_json::from_value::<WorldDispatchRequestV1>(serde_json::json!({
            "request_id": "req-37",
            "idempotency_key": "idem-37",
            "orchestration_session_id": "sess-37",
            "caller_participant_id": "orch-37",
            "action": "cancel_world_work",
            "mode": "retained",
            "target_backend_id": "cli:codex_world",
            "target_participant_id": "ash-worker-37",
            "world_id": "world-37",
            "world_generation": 7,
            "payload": {
                "payload_kind": "worker_cancel",
                "future_lifecycle_redirect": "packet-2"
            }
        }))
        .expect_err("unknown cancel payload fields must fail closed");

        assert!(
            error
                .to_string()
                .contains("unknown field `future_lifecycle_redirect`"),
            "unexpected cancel payload serde error: {error}"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_unknown_stop_payload_fields_during_deserialization() {
        let error = serde_json::from_value::<WorldDispatchRequestV1>(serde_json::json!({
            "request_id": "req-36",
            "idempotency_key": "idem-36",
            "orchestration_session_id": "sess-36",
            "caller_participant_id": "orch-36",
            "action": "stop_world_worker",
            "mode": "retained",
            "target_backend_id": "cli:codex_world",
            "target_participant_id": "ash-worker-36",
            "world_id": "world-36",
            "world_generation": 6,
            "payload": {
                "payload_kind": "worker_stop",
                "future_cancel_target": "packet-2"
            }
        }))
        .expect_err("unknown stop payload fields must fail closed");

        assert!(
            error
                .to_string()
                .contains("unknown field `future_cancel_target`"),
            "unexpected stop payload serde error: {error}"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_target_participant_id_for_non_continue_actions() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::RunWorldTask,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::Task(TaskPayloadV1 {
                prompt: "index the repo".to_string(),
            }),
        )
        .with_target_participant_id("ash-worker-32")
        .validate()
        .expect_err("non-continue target must fail closed");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_target: action run_world_task does not accept target_participant_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_task_run_id_for_non_inspect_cancel_actions() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::RunWorldTask,
            WorldDispatchModeV1::Ephemeral,
            WorldDispatchPayloadV1::Task(TaskPayloadV1 {
                prompt: "index the repo".to_string(),
            }),
        )
        .with_task_run_id("task-run-32")
        .validate()
        .expect_err("non inspect/cancel task identity must fail closed");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_target: action run_world_task does not accept task_run_id"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_payload_action_mismatch() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::SpawnWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::Task(TaskPayloadV1 {
                prompt: "index the repo".to_string(),
            }),
        )
        .validate()
        .expect_err("payload/action mismatch must fail closed");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action spawn_world_worker requires matching typed payload"
        );
    }

    #[test]
    fn world_dispatch_contract_round_trips_typed_fork_outcome_shape() {
        let outcome = WorldDispatchOutcomeV1::ForkWorldWorker(ForkWorldWorkerOutcomeV1 {
            request_id: "req-38".to_string(),
            orchestration_session_id: "sess-38".to_string(),
            action: WorldDispatchActionV1::ForkWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            orchestrator_participant_id: "orch-38".to_string(),
            source_participant_id: "ash-worker-source-38".to_string(),
            child_participant_id: "ash-worker-child-38".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-38".to_string(),
            world_generation: 8,
            summary: "fork outcome preserves explicit source-to-child lineage".to_string(),
        });

        let json = serde_json::to_value(&outcome).expect("serialize fork outcome");
        assert_eq!(
            json.get("outcome_kind").and_then(|value| value.as_str()),
            Some("fork_world_worker")
        );
        assert_eq!(
            json.get("source_participant_id")
                .and_then(|value| value.as_str()),
            Some("ash-worker-source-38")
        );
        assert_eq!(
            json.get("child_participant_id")
                .and_then(|value| value.as_str()),
            Some("ash-worker-child-38")
        );
    }

    #[test]
    fn world_dispatch_contract_round_trips_typed_run_world_task_outcome_shape() {
        let outcome = WorldDispatchOutcomeV1::RunWorldTask(RunWorldTaskOutcomeV1 {
            request_id: "req-47".to_string(),
            orchestration_session_id: "sess-47".to_string(),
            action: WorldDispatchActionV1::RunWorldTask,
            mode: WorldDispatchModeV1::Ephemeral,
            task_run_id: Some("task-run-47".to_string()),
            state: WorldTaskTerminalStateV1::Completed,
            summary: "run outcome surfaces exact task identity".to_string(),
        });

        let json = serde_json::to_value(&outcome).expect("serialize run outcome");
        assert_eq!(
            json.get("outcome_kind").and_then(|value| value.as_str()),
            Some("run_world_task")
        );
        assert_eq!(
            json.get("task_run_id").and_then(|value| value.as_str()),
            Some("task-run-47")
        );
    }

    #[test]
    fn world_dispatch_contract_round_trips_continue_fork_command_lineage_shape() {
        let outcome = WorldDispatchOutcomeV1::ContinueWorldWorker(ContinueWorldWorkerOutcomeV1 {
            request_id: "req-45".to_string(),
            orchestration_session_id: "sess-45".to_string(),
            action: WorldDispatchActionV1::ContinueWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            orchestrator_participant_id: "orch-45".to_string(),
            target_participant_id: "ash-worker-source-45".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-45".to_string(),
            world_generation: 9,
            source_participant_id: Some("ash-worker-source-45".to_string()),
            child_participant_id: Some("ash-worker-child-45".to_string()),
            thread_id: Some("thread-45".to_string()),
            worker_event: None,
            summary: "continue fork-command outcome preserves explicit lineage".to_string(),
        });

        let json = serde_json::to_value(&outcome).expect("serialize continue outcome");
        assert_eq!(
            json.get("outcome_kind").and_then(|value| value.as_str()),
            Some("continue_world_worker")
        );
        assert_eq!(
            json.get("source_participant_id")
                .and_then(|value| value.as_str()),
            Some("ash-worker-source-45")
        );
        assert_eq!(
            json.get("child_participant_id")
                .and_then(|value| value.as_str()),
            Some("ash-worker-child-45")
        );
    }

    #[test]
    fn world_dispatch_contract_round_trips_typed_inspect_outcome_shape() {
        let outcome = WorldDispatchOutcomeV1::InspectWorldWorker(InspectWorldWorkerOutcomeV1 {
            request_id: "req-35".to_string(),
            orchestration_session_id: "sess-35".to_string(),
            action: WorldDispatchActionV1::InspectWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            orchestrator_participant_id: "orch-35".to_string(),
            target_participant_id: "ash-worker-35".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-35".to_string(),
            world_generation: 5,
            snapshot: RetainedWorkerInspectSnapshotV1 {
                participant_state: AgentRuntimeSessionState::Running,
                session_state: OrchestrationSessionState::Active,
                session_posture: OrchestrationSessionPosture::AwaitingAttention,
                authoritative_live: true,
                attention_required: true,
                parent_participant_id: Some("ash-parent-35".to_string()),
                resumed_from_participant_id: None,
            },
            summary: "inspect snapshot is authoritative".to_string(),
        });

        let json = serde_json::to_value(&outcome).expect("serialize inspect outcome");
        assert_eq!(
            json.get("outcome_kind").and_then(|value| value.as_str()),
            Some("inspect_world_worker")
        );
        let snapshot = json.get("snapshot").expect("snapshot should serialize");
        assert_eq!(
            snapshot
                .get("session_posture")
                .and_then(|value| value.as_str()),
            Some("awaiting_attention")
        );
        assert_eq!(
            snapshot
                .get("participant_state")
                .and_then(|value| value.as_str()),
            Some("running")
        );
    }

    #[test]
    fn world_dispatch_contract_round_trips_typed_stop_outcome_shape() {
        let outcome = WorldDispatchOutcomeV1::StopWorldWorker(StopWorldWorkerOutcomeV1 {
            request_id: "req-36".to_string(),
            orchestration_session_id: "sess-36".to_string(),
            action: WorldDispatchActionV1::StopWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            orchestrator_participant_id: "orch-36".to_string(),
            target_participant_id: "ash-worker-36".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-36".to_string(),
            world_generation: 6,
            closeout: RetainedWorkerStopCloseoutV1 {
                participant_state: AgentRuntimeSessionState::Stopped,
                session_state: OrchestrationSessionState::Stopped,
            },
            summary: "stop closeout is authoritative".to_string(),
        });

        let json = serde_json::to_value(&outcome).expect("serialize stop outcome");
        assert_eq!(
            json.get("outcome_kind").and_then(|value| value.as_str()),
            Some("stop_world_worker")
        );
        let closeout = json.get("closeout").expect("closeout should serialize");
        assert_eq!(
            closeout
                .get("participant_state")
                .and_then(|value| value.as_str()),
            Some("stopped")
        );
        assert_eq!(
            closeout
                .get("session_state")
                .and_then(|value| value.as_str()),
            Some("stopped")
        );
        assert!(json.get("snapshot").is_none());
        assert!(json.get("worker_event").is_none());
    }

    #[test]
    fn world_dispatch_contract_round_trips_typed_cancel_outcome_shape() {
        let outcome = WorldDispatchOutcomeV1::CancelWorldWork(CancelWorldWorkOutcomeV1 {
            request_id: "req-37".to_string(),
            orchestration_session_id: "sess-37".to_string(),
            action: WorldDispatchActionV1::CancelWorldWork,
            mode: WorldDispatchModeV1::Retained,
            orchestrator_participant_id: "orch-37".to_string(),
            target_participant_id: "ash-worker-37".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-37".to_string(),
            world_generation: 7,
            state: CancelWorldWorkTerminalStateV1::Cancelled,
            closeout: RetainedWorkerCancelCloseoutV1 {
                participant_state: None,
                session_state: None,
            },
            summary: "cancel closeout is distinct from stop".to_string(),
        });

        let json = serde_json::to_value(&outcome).expect("serialize cancel outcome");
        assert_eq!(
            json.get("outcome_kind").and_then(|value| value.as_str()),
            Some("cancel_world_work")
        );
        assert_eq!(
            json.get("state").and_then(|value| value.as_str()),
            Some("cancelled")
        );
        let closeout = json.get("closeout").expect("closeout should serialize");
        assert_eq!(
            closeout
                .get("participant_state")
                .and_then(|value| value.as_str()),
            None
        );
        assert_eq!(
            closeout
                .get("session_state")
                .and_then(|value| value.as_str()),
            None
        );
        assert!(json.get("snapshot").is_none());
        assert!(json.get("cancelled").is_none());
    }

    #[test]
    fn world_dispatch_contract_rejects_cancel_outcome_without_closeout() {
        let error = serde_json::from_value::<WorldDispatchOutcomeV1>(serde_json::json!({
            "outcome_kind": "cancel_world_work",
            "request_id": "req-37",
            "orchestration_session_id": "sess-37",
            "action": "cancel_world_work",
            "mode": "retained",
            "orchestrator_participant_id": "orch-37",
            "target_participant_id": "ash-worker-37",
            "target_backend_id": "cli:codex_world",
            "world_id": "world-37",
            "world_generation": 7,
            "state": "cancelled",
            "summary": "cancel closeout is distinct from stop"
        }))
        .expect_err("missing cancel closeout must fail closed");

        assert!(
            error.to_string().contains("missing field `closeout`"),
            "unexpected cancel closeout serde error: {error}"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_non_cancelled_cancel_outcome_state() {
        let error = serde_json::from_value::<WorldDispatchOutcomeV1>(serde_json::json!({
            "outcome_kind": "cancel_world_work",
            "request_id": "req-37",
            "orchestration_session_id": "sess-37",
            "action": "cancel_world_work",
            "mode": "retained",
            "orchestrator_participant_id": "orch-37",
            "target_participant_id": "ash-worker-37",
            "target_backend_id": "cli:codex_world",
            "world_id": "world-37",
            "world_generation": 7,
            "state": "failed",
            "closeout": {},
            "summary": "cancel closeout is distinct from stop"
        }))
        .expect_err("non-cancelled cancel outcome state must fail closed");

        assert!(
            error
                .to_string()
                .contains("unknown variant `failed`, expected `cancelled`"),
            "unexpected cancel state serde error: {error}"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_blank_continue_thread_id_when_provided() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                prompt: "continue with the integration trace".to_string(),
                thread_id: Some(" ".to_string()),
            }),
        )
        .with_target_participant_id("ash-worker-32")
        .validate()
        .expect_err("blank thread id must fail closed");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty thread_id when provided"
        );
    }

    #[test]
    fn world_dispatch_contract_accepts_continue_world_worker_typed_approval_response_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueApprovalResponse(
                WorkerContinueApprovalResponsePayloadV1 {
                    approval_obligation_id: "obl-approval-40".to_string(),
                    decision: ApprovalResponseDecisionV1::Approve,
                    thread_id: Some("thread-approval".to_string()),
                },
            ),
        )
        .with_target_participant_id("ash-worker-40")
        .validate()
        .expect("typed approval response should validate");

        let WorldDispatchPayloadV1::WorkerContinueApprovalResponse(payload) = validated.payload
        else {
            panic!("validated payload should remain typed approval response payload");
        };
        assert_eq!(payload.approval_obligation_id, "obl-approval-40");
        assert_eq!(payload.decision, ApprovalResponseDecisionV1::Approve);
        assert_eq!(payload.thread_id.as_deref(), Some("thread-approval"));
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    fn world_dispatch_contract_renders_typed_approval_response_prompt_deterministically() {
        let prompt = render_continue_world_worker_transport_prompt(
            &WorldDispatchPayloadV1::WorkerContinueApprovalResponse(
                WorkerContinueApprovalResponsePayloadV1 {
                    approval_obligation_id: "obl-approval-40".to_string(),
                    decision: ApprovalResponseDecisionV1::Deny,
                    thread_id: Some("thread-approval".to_string()),
                },
            ),
        )
        .expect("typed approval response should render");

        assert_eq!(
            prompt,
            "SUBSTRATE_INTERNAL_HOST_APPROVAL_RESPONSE_V1\n{\"kind\":\"approval_response\",\"approval_obligation_id\":\"obl-approval-40\",\"decision\":\"deny\",\"thread_id\":\"thread-approval\"}\nTreat this as the host's typed approval_response for the matching pending approval request. Apply decision=approve as permission granted and decision=deny as permission denied."
        );
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    fn world_dispatch_contract_preserves_prompt_for_normal_continue_transport_rendering() {
        let prompt = render_continue_world_worker_transport_prompt(
            &WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                prompt: "continue with the integration trace".to_string(),
                thread_id: Some("thread-root".to_string()),
            }),
        )
        .expect("normal continue prompt should render");

        assert_eq!(prompt, "continue with the integration trace");
    }

    #[test]
    fn world_dispatch_contract_rejects_typed_approval_response_without_exact_causation() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueApprovalResponse(
                WorkerContinueApprovalResponsePayloadV1 {
                    approval_obligation_id: " ".to_string(),
                    decision: ApprovalResponseDecisionV1::Deny,
                    thread_id: None,
                },
            ),
        )
        .with_target_participant_id("ash-worker-40")
        .validate()
        .expect_err("typed approval responses must bind exact causation");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty approval_obligation_id when payload_kind is worker_continue_approval_response"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_blank_thread_id_in_typed_approval_response() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueApprovalResponse(
                WorkerContinueApprovalResponsePayloadV1 {
                    approval_obligation_id: "obl-approval-40".to_string(),
                    decision: ApprovalResponseDecisionV1::Approve,
                    thread_id: Some(" ".to_string()),
                },
            ),
        )
        .with_target_participant_id("ash-worker-40")
        .validate()
        .expect_err("typed approval responses must reject blank thread ids");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty thread_id when provided"
        );
    }

    #[test]
    fn world_dispatch_contract_accepts_continue_world_worker_typed_clarification_response_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueClarificationResponse(
                WorkerContinueClarificationResponsePayloadV1 {
                    follow_up_obligation_id: "obl-follow-up-42".to_string(),
                    clarification_text: "Use the latest local branch state when continuing."
                        .to_string(),
                    thread_id: Some("thread-follow-up".to_string()),
                },
            ),
        )
        .with_target_participant_id("ash-worker-42")
        .validate()
        .expect("typed clarification response should validate");

        let WorldDispatchPayloadV1::WorkerContinueClarificationResponse(payload) =
            validated.payload
        else {
            panic!("validated payload should remain typed clarification response payload");
        };
        assert_eq!(payload.follow_up_obligation_id, "obl-follow-up-42");
        assert_eq!(
            payload.clarification_text,
            "Use the latest local branch state when continuing."
        );
        assert_eq!(payload.thread_id.as_deref(), Some("thread-follow-up"));
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    fn world_dispatch_contract_renders_typed_clarification_response_prompt_deterministically() {
        let prompt = render_continue_world_worker_transport_prompt(
            &WorldDispatchPayloadV1::WorkerContinueClarificationResponse(
                WorkerContinueClarificationResponsePayloadV1 {
                    follow_up_obligation_id: "obl-follow-up-42".to_string(),
                    clarification_text: "Use the latest local branch state when continuing."
                        .to_string(),
                    thread_id: Some("thread-follow-up".to_string()),
                },
            ),
        )
        .expect("typed clarification response should render");

        assert_eq!(
            prompt,
            "SUBSTRATE_INTERNAL_HOST_CLARIFICATION_RESPONSE_V1\n{\"kind\":\"clarification_response\",\"follow_up_obligation_id\":\"obl-follow-up-42\",\"clarification_text\":\"Use the latest local branch state when continuing.\",\"thread_id\":\"thread-follow-up\"}\nTreat this as the host's typed clarification_response for the matching pending follow-up obligation. Use clarification_text as authoritative host guidance before continuing work."
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_typed_clarification_response_without_exact_causation() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueClarificationResponse(
                WorkerContinueClarificationResponsePayloadV1 {
                    follow_up_obligation_id: " ".to_string(),
                    clarification_text: "Please continue with the existing plan.".to_string(),
                    thread_id: None,
                },
            ),
        )
        .with_target_participant_id("ash-worker-42")
        .validate()
        .expect_err("typed clarification responses must bind exact follow-up causation");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty follow_up_obligation_id when payload_kind is worker_continue_clarification_response"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_blank_clarification_text() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueClarificationResponse(
                WorkerContinueClarificationResponsePayloadV1 {
                    follow_up_obligation_id: "obl-follow-up-42".to_string(),
                    clarification_text: " ".to_string(),
                    thread_id: None,
                },
            ),
        )
        .with_target_participant_id("ash-worker-42")
        .validate()
        .expect_err("typed clarification responses must carry non-empty clarification text");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty clarification_text when payload_kind is worker_continue_clarification_response"
        );
    }

    #[test]
    fn world_dispatch_contract_accepts_continue_world_worker_typed_fork_command_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueForkCommand(WorkerContinueForkCommandPayloadV1 {
                child_prompt: "Investigate the flaky Linux replay trace.".to_string(),
                fork_reason: Some("specialize:replay_trace".to_string()),
                fork_strategy: Some("parallelize_investigation".to_string()),
                thread_id: Some("thread-fork".to_string()),
            }),
        )
        .with_target_participant_id("ash-worker-45")
        .validate()
        .expect("typed fork command should validate");

        let WorldDispatchPayloadV1::WorkerContinueForkCommand(payload) = validated.payload else {
            panic!("validated payload should remain typed fork command payload");
        };
        assert_eq!(
            payload.child_prompt,
            "Investigate the flaky Linux replay trace."
        );
        assert_eq!(
            payload.fork_reason.as_deref(),
            Some("specialize:replay_trace")
        );
        assert_eq!(
            payload.fork_strategy.as_deref(),
            Some("parallelize_investigation")
        );
        assert_eq!(payload.thread_id.as_deref(), Some("thread-fork"));
    }

    #[test]
    fn world_dispatch_contract_rejects_blank_child_prompt_in_typed_fork_command() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueForkCommand(WorkerContinueForkCommandPayloadV1 {
                child_prompt: " ".to_string(),
                fork_reason: Some("specialize:replay_trace".to_string()),
                fork_strategy: Some("parallelize_investigation".to_string()),
                thread_id: None,
            }),
        )
        .with_target_participant_id("ash-worker-45")
        .validate()
        .expect_err("typed fork commands must carry child work intent");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty child_prompt when payload_kind is worker_continue_fork_command"
        );
    }

    #[test]
    fn world_dispatch_contract_canonicalizes_bounded_typed_fork_command_metadata_before_storing() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueForkCommand(WorkerContinueForkCommandPayloadV1 {
                child_prompt: "  Investigate the flaky Linux replay trace.  ".to_string(),
                fork_reason: Some("  specialize:replay_trace  ".to_string()),
                fork_strategy: Some("  parallelize_investigation  ".to_string()),
                thread_id: Some("thread-fork".to_string()),
            }),
        )
        .with_target_participant_id("ash-worker-45")
        .validate()
        .expect("bounded fork-command metadata labels should canonicalize");

        let WorldDispatchPayloadV1::WorkerContinueForkCommand(payload) = validated.payload else {
            panic!("validated payload should remain typed fork command payload");
        };
        assert_eq!(
            payload.child_prompt,
            "Investigate the flaky Linux replay trace."
        );
        assert_eq!(
            payload.fork_reason.as_deref(),
            Some("specialize:replay_trace")
        );
        assert_eq!(
            payload.fork_strategy.as_deref(),
            Some("parallelize_investigation")
        );
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    fn world_dispatch_contract_renders_typed_fork_command_prompt_deterministically() {
        let prompt = render_continue_world_worker_transport_prompt(
            &base_world_dispatch_request(
                WorldDispatchActionV1::ContinueWorldWorker,
                WorldDispatchModeV1::Retained,
                WorldDispatchPayloadV1::WorkerContinueForkCommand(
                    WorkerContinueForkCommandPayloadV1 {
                        child_prompt: "Investigate the flaky Linux replay trace.".to_string(),
                        fork_reason: Some("specialize:replay_trace".to_string()),
                        fork_strategy: Some("parallelize_investigation".to_string()),
                        thread_id: Some("thread-fork".to_string()),
                    },
                ),
            )
            .with_target_participant_id("ash-worker-45")
            .validate()
            .expect("validated typed fork command request")
            .payload,
        )
        .expect("render typed fork command prompt");

        assert_eq!(
            prompt,
            "SUBSTRATE_INTERNAL_HOST_FORK_COMMAND_V1\n{\"kind\":\"fork_command\",\"child_prompt\":\"Investigate the flaky Linux replay trace.\",\"fork_reason\":\"specialize:replay_trace\",\"fork_strategy\":\"parallelize_investigation\",\"thread_id\":\"thread-fork\"}\nTreat this as the host's typed fork_command for the retained worker. Treat child_prompt as the authoritative child-work intent to prepare for exact-source retained fork bootstrap. Treat fork_reason and fork_strategy only as bounded routing metadata labels for host-mediated fork handling; they do not authorize autonomous child allocation."
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_free_form_typed_fork_command_metadata() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueForkCommand(WorkerContinueForkCommandPayloadV1 {
                child_prompt: "Investigate the flaky Linux replay trace.".to_string(),
                fork_reason: Some("needs replay trace specialization".to_string()),
                fork_strategy: Some("parallelize_investigation".to_string()),
                thread_id: Some("thread-fork".to_string()),
            }),
        )
        .with_target_participant_id("ash-worker-45")
        .validate()
        .expect_err("fork command metadata must stay bounded");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires fork_reason to be a bounded metadata label when payload_kind is worker_continue_fork_command"
        );
    }

    #[test]
    fn world_dispatch_contract_accepts_continue_world_worker_typed_control_directive_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueControlDirective(
                WorkerContinueControlDirectivePayloadV1 {
                    directive_kind: ControlDirectiveKindV1::Pause,
                    directive_text: None,
                    thread_id: Some("thread-control".to_string()),
                },
            ),
        )
        .with_target_participant_id("ash-worker-43")
        .validate()
        .expect("typed control directive should validate");

        let WorldDispatchPayloadV1::WorkerContinueControlDirective(payload) = validated.payload
        else {
            panic!("validated payload should remain typed control directive payload");
        };
        assert_eq!(payload.directive_kind, ControlDirectiveKindV1::Pause);
        assert_eq!(payload.directive_text, None);
        assert_eq!(payload.thread_id.as_deref(), Some("thread-control"));
    }

    #[test]
    fn world_dispatch_contract_accepts_continue_world_worker_typed_progress_ack_shape() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueProgressAck(WorkerContinueProgressAckPayloadV1 {
                thread_id: Some("thread-progress".to_string()),
            }),
        )
        .with_target_participant_id("ash-worker-46")
        .validate()
        .expect("typed progress_ack should validate");

        let WorldDispatchPayloadV1::WorkerContinueProgressAck(payload) = validated.payload else {
            panic!("validated payload should remain typed progress_ack payload");
        };
        assert_eq!(payload.thread_id.as_deref(), Some("thread-progress"));
    }

    #[test]
    fn world_dispatch_contract_rejects_blank_thread_id_in_typed_progress_ack() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueProgressAck(WorkerContinueProgressAckPayloadV1 {
                thread_id: Some(" ".to_string()),
            }),
        )
        .with_target_participant_id("ash-worker-46")
        .validate()
        .expect_err("typed progress_ack must reject blank thread_id when provided");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty thread_id when provided"
        );
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    fn world_dispatch_contract_renders_typed_progress_ack_prompt_deterministically() {
        let prompt = render_continue_world_worker_transport_prompt(
            &WorldDispatchPayloadV1::WorkerContinueProgressAck(
                WorkerContinueProgressAckPayloadV1 {
                    thread_id: Some("thread-progress".to_string()),
                },
            ),
        )
        .expect("typed progress_ack should render");

        assert_eq!(
            prompt,
            "SUBSTRATE_INTERNAL_HOST_PROGRESS_ACK_V1\n{\"kind\":\"progress_ack\",\"thread_id\":\"thread-progress\"}\nTreat this as the host's typed progress_ack for the retained worker. It only acknowledges that the host saw the worker's recent progress. It does not imply completion, pause, new scope, or durable closeout."
        );
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    fn world_dispatch_contract_renders_typed_control_directive_prompt_deterministically() {
        let prompt = render_continue_world_worker_transport_prompt(
            &WorldDispatchPayloadV1::WorkerContinueControlDirective(
                WorkerContinueControlDirectivePayloadV1 {
                    directive_kind: ControlDirectiveKindV1::PrepareHandoff,
                    directive_text: Some("  timing:before_stop  ".to_string()),
                    thread_id: Some("thread-control".to_string()),
                },
            ),
        )
        .expect("typed control directive should render");

        assert_eq!(
            prompt,
            "SUBSTRATE_INTERNAL_HOST_CONTROL_DIRECTIVE_V1\n{\"kind\":\"control_directive\",\"directive_kind\":\"prepare_handoff\",\"directive_text\":\"timing:before_stop\",\"thread_id\":\"thread-control\"}\nTreat this as the host's typed control_directive for the retained worker. Apply directive_kind=prepare_handoff as authoritative host guidance. Prepare a concise handoff covering current state, next steps, and notable risks. Treat directive_text only as bounded handoff metadata label for this directive kind; it does not add new instructions."
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_blank_control_directive_directive_text() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueControlDirective(
                WorkerContinueControlDirectivePayloadV1 {
                    directive_kind: ControlDirectiveKindV1::ReduceScope,
                    directive_text: Some(" ".to_string()),
                    thread_id: None,
                },
            ),
        )
        .with_target_participant_id("ash-worker-43")
        .validate()
        .expect_err("typed control directives must reject blank directive text when provided");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty directive_text when provided"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_blank_thread_id_in_typed_control_directive() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueControlDirective(
                WorkerContinueControlDirectivePayloadV1 {
                    directive_kind: ControlDirectiveKindV1::Checkpoint,
                    directive_text: Some("artifact:current_state".to_string()),
                    thread_id: Some(" ".to_string()),
                },
            ),
        )
        .with_target_participant_id("ash-worker-43")
        .validate()
        .expect_err("typed control directives must reject blank thread_id when provided");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires non-empty thread_id when provided"
        );
    }

    #[test]
    fn world_dispatch_contract_accepts_bounded_directive_text_for_all_control_directive_kinds() {
        let cases = [
            (ControlDirectiveKindV1::Pause, "scope:current_branch"),
            (ControlDirectiveKindV1::ReduceScope, "scope:tests_only"),
            (ControlDirectiveKindV1::Summarize, "focus:current_state"),
            (ControlDirectiveKindV1::Checkpoint, "artifact:current_state"),
            (ControlDirectiveKindV1::PrepareHandoff, "timing:before_stop"),
        ];

        for (directive_kind, directive_text) in cases {
            let validated = base_world_dispatch_request(
                WorldDispatchActionV1::ContinueWorldWorker,
                WorldDispatchModeV1::Retained,
                WorldDispatchPayloadV1::WorkerContinueControlDirective(
                    WorkerContinueControlDirectivePayloadV1 {
                        directive_kind,
                        directive_text: Some(directive_text.to_string()),
                        thread_id: None,
                    },
                ),
            )
            .with_target_participant_id("ash-worker-43")
            .validate()
            .expect("control directives should accept recognized bounded metadata labels");

            let WorldDispatchPayloadV1::WorkerContinueControlDirective(payload) = validated.payload
            else {
                panic!("validated payload should remain typed control directive payload");
            };
            assert_eq!(payload.directive_kind, directive_kind);
            assert_eq!(payload.directive_text.as_deref(), Some(directive_text));
        }
    }

    #[test]
    fn world_dispatch_contract_canonicalizes_bounded_control_directive_text_before_storing() {
        let validated = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueControlDirective(
                WorkerContinueControlDirectivePayloadV1 {
                    directive_kind: ControlDirectiveKindV1::PrepareHandoff,
                    directive_text: Some("  timing:before_stop  ".to_string()),
                    thread_id: Some("thread-control".to_string()),
                },
            ),
        )
        .with_target_participant_id("ash-worker-43")
        .validate()
        .expect("recognized bounded directive text should canonicalize before storage");

        let WorldDispatchPayloadV1::WorkerContinueControlDirective(payload) = validated.payload
        else {
            panic!("validated payload should remain typed control directive payload");
        };
        assert_eq!(
            payload.directive_text.as_deref(),
            Some("timing:before_stop")
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_free_form_control_directive_text() {
        let error = base_world_dispatch_request(
            WorldDispatchActionV1::ContinueWorldWorker,
            WorldDispatchModeV1::Retained,
            WorldDispatchPayloadV1::WorkerContinueControlDirective(
                WorkerContinueControlDirectivePayloadV1 {
                    directive_kind: ControlDirectiveKindV1::PrepareHandoff,
                    directive_text: Some("before stopping".to_string()),
                    thread_id: Some("thread-control".to_string()),
                },
            ),
        )
        .with_target_participant_id("ash-worker-43")
        .validate()
        .expect_err("directive_text must stay a bounded metadata label");

        assert_eq!(
            error.to_string(),
            "invalid_dispatch_payload: action continue_world_worker requires directive_text to be a recognized bounded metadata label for directive_kind prepare_handoff"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_unsupported_typed_control_directive_kind() {
        let error = serde_json::from_value::<WorldDispatchRequestV1>(serde_json::json!({
            "request_id": "req-43",
            "idempotency_key": "idem-43",
            "orchestration_session_id": "sess-43",
            "caller_participant_id": "orch-43",
            "action": "continue_world_worker",
            "mode": "retained",
            "target_backend_id": "cli:codex_world",
            "target_participant_id": "ash-worker-43",
            "world_id": "world-43",
            "world_generation": 11,
            "payload": {
                "payload_kind": "worker_continue_control_directive",
                "directive_kind": "rewire_transport",
                "directive_text": "invent a broader control plane"
            }
        }))
        .expect_err("unsupported directive kinds must fail closed");

        assert!(
            error
                .to_string()
                .contains("unknown variant `rewire_transport`"),
            "unexpected unsupported directive-kind error: {error}"
        );
    }

    #[test]
    fn world_dispatch_contract_rejects_still_deferred_host_response_payload_kinds_during_deserialization(
    ) {
        let payload_kind = "worker_continue_control_ack";
        let error = serde_json::from_value::<WorldDispatchRequestV1>(serde_json::json!({
            "request_id": "req-43",
            "idempotency_key": "idem-43",
            "orchestration_session_id": "sess-43",
            "caller_participant_id": "orch-43",
            "action": "continue_world_worker",
            "mode": "retained",
            "target_backend_id": "cli:codex_world",
            "target_participant_id": "ash-worker-43",
            "world_id": "world-43",
            "world_generation": 11,
            "payload": {
                "payload_kind": payload_kind
            }
        }))
        .expect_err("deferred host response classes must stay out of packet 1");

        assert!(
            error
                .to_string()
                .contains(&format!("unknown variant `{payload_kind}`")),
            "unexpected deferred host response serde error for {payload_kind}: {error}"
        );
    }

    #[test]
    fn continue_world_worker_event_class_attention_semantics_accepts_packet_one_worker_requests() {
        assert!(!ContinueWorldWorkerEventClassV1::Reply.attention_required_by_default());
        assert!(!ContinueWorldWorkerEventClassV1::ProgressUpdate.attention_required_by_default());
        assert!(!ContinueWorldWorkerEventClassV1::Result.attention_required_by_default());
        assert!(!ContinueWorldWorkerEventClassV1::Failure.attention_required_by_default());
        assert!(!ContinueWorldWorkerEventClassV1::ControlAck.attention_required_by_default());
        assert!(ContinueWorldWorkerEventClassV1::FollowUpQuestion.attention_required_by_default());
        assert!(ContinueWorldWorkerEventClassV1::Blocked.attention_required_by_default());
        assert_eq!(
            ContinueWorldWorkerEventClassV1::from_wire_label("control_ack"),
            Some(ContinueWorldWorkerEventClassV1::ControlAck)
        );
        assert_eq!(
            ContinueWorldWorkerEventClassV1::from_wire_label("approval_request"),
            Some(ContinueWorldWorkerEventClassV1::ApprovalRequest)
        );
        assert_eq!(
            ContinueWorldWorkerEventClassV1::from_wire_label("fork_request"),
            Some(ContinueWorldWorkerEventClassV1::ForkRequest)
        );
        assert_eq!(
            ContinueWorldWorkerEventClassV1::from_wire_label("fork_recommendation"),
            Some(ContinueWorldWorkerEventClassV1::ForkRecommendation)
        );
        assert!(ContinueWorldWorkerEventClassV1::ApprovalRequest.attention_required_by_default());
        assert!(ContinueWorldWorkerEventClassV1::ForkRequest.attention_required_by_default());
        assert!(
            !ContinueWorldWorkerEventClassV1::ForkRecommendation.attention_required_by_default()
        );
        assert!(!ContinueWorldWorkerEventClassV1::is_deferred_wire_label(
            "approval_request"
        ));
        assert!(!ContinueWorldWorkerEventClassV1::is_deferred_wire_label(
            "fork_request"
        ));
        assert!(!ContinueWorldWorkerEventClassV1::is_deferred_wire_label(
            "fork_recommendation"
        ));
        assert!(!ContinueWorldWorkerEventClassV1::is_deferred_wire_label(
            "control_ack"
        ));
        assert!(ContinueWorldWorkerEventClassV1::is_deferred_wire_label(
            "approval_response"
        ));
        assert!(ContinueWorldWorkerEventClassV1::is_deferred_wire_label(
            "progress_ack"
        ));
        assert!(ContinueWorldWorkerEventClassV1::is_deferred_wire_label(
            "control_directive"
        ));
        assert!(!ContinueWorldWorkerEventClassV1::is_deferred_wire_label(
            "reply"
        ));
    }

    #[test]
    fn world_dispatch_contract_packet34_steering_denial_buckets_stay_stable() {
        let labels = [
            WorldDispatchSteeringDenialV1::WorldDispatchDisabled,
            WorldDispatchSteeringDenialV1::BackendNotAllowed,
            WorldDispatchSteeringDenialV1::ModeNotAllowed,
            WorldDispatchSteeringDenialV1::ActionNotAllowed,
            WorldDispatchSteeringDenialV1::CrossSessionSteeringDenied,
            WorldDispatchSteeringDenialV1::CrossWorldBindingSteeringDenied,
            WorldDispatchSteeringDenialV1::CapabilityNarrowingNotAllowed,
            WorldDispatchSteeringDenialV1::WorkerConcurrencyCapExceeded,
            WorldDispatchSteeringDenialV1::InvalidatedWorkerNotRoutable,
        ]
        .into_iter()
        .map(WorldDispatchSteeringDenialV1::as_str)
        .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec![
                "world_dispatch_disabled",
                "backend_not_allowed",
                "mode_not_allowed",
                "action_not_allowed",
                "cross_session_steering_denied",
                "cross_world_binding_steering_denied",
                "capability_narrowing_not_allowed",
                "worker_concurrency_cap_exceeded",
                "invalidated_worker_not_routable",
            ]
        );
    }

    #[test]
    fn world_dispatch_contract_packet34_steering_denial_formats_detail_stably() {
        assert_eq!(
            WorldDispatchSteeringDenialV1::ModeNotAllowed
                .format_message("effective policy allows only retained"),
            "mode_not_allowed: effective policy allows only retained"
        );
        assert_eq!(
            WorldDispatchSteeringDenialV1::WorldDispatchDisabled.format_message(""),
            "world_dispatch_disabled"
        );
    }
}
