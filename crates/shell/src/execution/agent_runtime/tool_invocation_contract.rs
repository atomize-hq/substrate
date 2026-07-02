#![allow(dead_code)]

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::execution::config_model::AgentExecutionScope;

use super::{
    dispatch_contract::{
        CancelWorldWorkOutcomeV1, CancelWorldWorkTerminalStateV1, ContinueWorldWorkerEventV1,
        ContinueWorldWorkerOutcomeV1, ForkWorldWorkerOutcomeV1, InspectWorldWorkerOutcomeV1,
        RetainedWorkerCancelCloseoutV1, RetainedWorkerInspectSnapshotV1,
        RetainedWorkerStopCloseoutV1, RunWorldTaskOutcomeV1, SpawnWorldWorkerOutcomeV1,
        StopWorldWorkerOutcomeV1, TaskPayloadV1, WorkerSpawnPayloadV1, WorldDispatchActionV1,
        WorldDispatchModeV1, WorldDispatchOutcomeV1, WorldDispatchPayloadV1,
        WorldDispatchRequestV1, WorldTaskTerminalStateV1,
    },
    mapping::MEMBER_ROLE,
    state_store::{
        exact_continue_retained_worker_is_routable, validate_retained_worker_authoritative_lineage,
        AgentRuntimeStateStore,
    },
};

/// Frozen adapter-visible contract for host-owned tool invocation above the
/// already-landed internal `WorldDispatchRequestV1` transport.
///
/// This module is intentionally bounded to vocabulary and exact-handle
/// semantics. It freezes the tool vocabulary, exact follow-up handles, and the
/// bounded translation of runtime-owned fields into internal dispatch requests
/// without registering tools in any runtime family or widening the transport.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum HostToolNameV1 {
    RunWorldTask,
    SpawnWorldWorker,
    ForkWorldWorker,
    ContinueWorldWorker,
    InspectWorldWorker,
    CancelWorldWork,
    StopWorldWorker,
}

impl HostToolNameV1 {
    pub(crate) const ALL: [Self; 7] = [
        Self::RunWorldTask,
        Self::SpawnWorldWorker,
        Self::ForkWorldWorker,
        Self::ContinueWorldWorker,
        Self::InspectWorldWorker,
        Self::CancelWorldWork,
        Self::StopWorldWorker,
    ];

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

    pub(crate) fn dispatch_action(self) -> WorldDispatchActionV1 {
        match self {
            Self::RunWorldTask => WorldDispatchActionV1::RunWorldTask,
            Self::SpawnWorldWorker => WorldDispatchActionV1::SpawnWorldWorker,
            Self::ForkWorldWorker => WorldDispatchActionV1::ForkWorldWorker,
            Self::ContinueWorldWorker => WorldDispatchActionV1::ContinueWorldWorker,
            Self::InspectWorldWorker => WorldDispatchActionV1::InspectWorldWorker,
            Self::CancelWorldWork => WorldDispatchActionV1::CancelWorldWork,
            Self::StopWorldWorker => WorldDispatchActionV1::StopWorldWorker,
        }
    }

    pub(crate) fn contract(self) -> &'static HostToolContractV1 {
        host_tool_contract(self)
    }

    pub(crate) fn from_dispatch_action(action: WorldDispatchActionV1) -> Self {
        match action {
            WorldDispatchActionV1::RunWorldTask => Self::RunWorldTask,
            WorldDispatchActionV1::SpawnWorldWorker => Self::SpawnWorldWorker,
            WorldDispatchActionV1::ForkWorldWorker => Self::ForkWorldWorker,
            WorldDispatchActionV1::ContinueWorldWorker => Self::ContinueWorldWorker,
            WorldDispatchActionV1::InspectWorldWorker => Self::InspectWorldWorker,
            WorldDispatchActionV1::CancelWorldWork => Self::CancelWorldWork,
            WorldDispatchActionV1::StopWorldWorker => Self::StopWorldWorker,
        }
    }

    pub(crate) fn from_str(raw: &str) -> anyhow::Result<Self> {
        match raw.trim() {
            "run_world_task" => Ok(Self::RunWorldTask),
            "spawn_world_worker" => Ok(Self::SpawnWorldWorker),
            "fork_world_worker" => Ok(Self::ForkWorldWorker),
            "continue_world_worker" => Ok(Self::ContinueWorldWorker),
            "inspect_world_worker" => Ok(Self::InspectWorldWorker),
            "cancel_world_work" => Ok(Self::CancelWorldWork),
            "stop_world_worker" => Ok(Self::StopWorldWorker),
            other => bail!("unknown_host_tool: unsupported tool_name {other:?}"),
        }
    }

    fn retained_follow_up_target_requirement(self) -> RetainedFollowUpTargetRequirementV1 {
        match self {
            Self::ContinueWorldWorker => RetainedFollowUpTargetRequirementV1::ContinueRoutable,
            Self::CancelWorldWork => RetainedFollowUpTargetRequirementV1::AuthoritativeLive,
            Self::ForkWorldWorker | Self::StopWorldWorker => {
                RetainedFollowUpTargetRequirementV1::NonTerminal
            }
            Self::InspectWorldWorker | Self::RunWorldTask | Self::SpawnWorldWorker => {
                RetainedFollowUpTargetRequirementV1::LinkedOnly
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RetainedFollowUpTargetRequirementV1 {
    LinkedOnly,
    NonTerminal,
    AuthoritativeLive,
    ContinueRoutable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostToolModelArgumentFamilyV1 {
    FreshTargetBackend,
    ExactRetainedWorkerHandle,
    EitherExactFollowUpHandle,
    TaskPayload,
    WorkerSpawnPayload,
    WorkerForkPayload,
    ContinueWorldWorkerPayload,
    InspectWorldWorkerPayload,
    CancelWorldWorkPayload,
    StopWorldWorkerPayload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostToolFollowUpHandleRequirementV1 {
    None,
    ExactRetainedWorker,
    EitherExactActiveTaskOrRetainedWorker,
}

impl HostToolFollowUpHandleRequirementV1 {
    pub(crate) fn resolve_exact_handle(
        self,
        task_run_id: Option<&str>,
        participant_id: Option<&str>,
    ) -> anyhow::Result<Option<HostToolFollowUpHandleV1>> {
        let task_run_id = canonicalize_handle_value("task_run_id", task_run_id)?;
        let participant_id = canonicalize_handle_value("participant_id", participant_id)?;

        match self {
            Self::None => {
                if task_run_id.is_some() || participant_id.is_some() {
                    bail!(
                        "invalid_follow_up_handle: this tool does not accept task_run_id or participant_id"
                    );
                }
                Ok(None)
            }
            Self::ExactRetainedWorker => match (task_run_id, participant_id) {
                (Some(_), _) => bail!(
                    "invalid_follow_up_handle: retained-worker follow-up requires exact participant_id and does not accept task_run_id"
                ),
                (None, Some(participant_id)) => Ok(Some(HostToolFollowUpHandleV1::RetainedWorker(
                    RetainedWorkerHandleV1 { participant_id },
                ))),
                (None, None) => bail!(
                    "missing_follow_up_handle: retained-worker follow-up requires exact participant_id"
                ),
            },
            Self::EitherExactActiveTaskOrRetainedWorker => match (task_run_id, participant_id) {
                (Some(task_run_id), None) => Ok(Some(HostToolFollowUpHandleV1::ActiveTask(
                    ActiveTaskHandleV1 { task_run_id },
                ))),
                (None, Some(participant_id)) => Ok(Some(HostToolFollowUpHandleV1::RetainedWorker(
                    RetainedWorkerHandleV1 { participant_id },
                ))),
                (Some(_), Some(_)) => bail!(
                    "invalid_follow_up_handle: follow-up requires exactly one exact handle family (task_run_id or participant_id, not both)"
                ),
                (None, None) => bail!(
                    "missing_follow_up_handle: follow-up requires exact task_run_id or exact participant_id"
                ),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActiveTaskHandleV1 {
    pub task_run_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RetainedWorkerHandleV1 {
    pub participant_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HostToolFollowUpHandleV1 {
    ActiveTask(ActiveTaskHandleV1),
    RetainedWorker(RetainedWorkerHandleV1),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostToolRunWorldTaskReceiptV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub task_run_id: String,
    pub state: WorldTaskTerminalStateV1,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostToolSpawnWorldWorkerReceiptV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_participant_id: Option<String>,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub launch_span_id: String,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostToolForkWorldWorkerReceiptV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    pub source_participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostToolContinueWorldWorkerOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    pub participant_id: String,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostToolInspectWorldWorkerOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub snapshot: RetainedWorkerInspectSnapshotV1,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostToolCancelWorldWorkOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub state: CancelWorldWorkTerminalStateV1,
    pub closeout: RetainedWorkerCancelCloseoutV1,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostToolStopWorldWorkerOutcomeV1 {
    pub request_id: String,
    pub orchestration_session_id: String,
    pub action: WorldDispatchActionV1,
    pub mode: WorldDispatchModeV1,
    pub orchestrator_participant_id: String,
    pub participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub closeout: RetainedWorkerStopCloseoutV1,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HostToolContractV1 {
    pub tool_name: HostToolNameV1,
    pub allowed_modes: &'static [WorldDispatchModeV1],
    pub follow_up_handle_requirement: HostToolFollowUpHandleRequirementV1,
    pub primary_model_argument_families: &'static [HostToolModelArgumentFamilyV1],
}

impl HostToolContractV1 {
    pub(crate) fn dispatch_action(self) -> WorldDispatchActionV1 {
        self.tool_name.dispatch_action()
    }

    pub(crate) fn accepts_mode(self, mode: WorldDispatchModeV1) -> bool {
        self.allowed_modes.contains(&mode)
    }
}

const MODE_EPHEMERAL_ONLY: &[WorldDispatchModeV1] = &[WorldDispatchModeV1::Ephemeral];
const MODE_RETAINED_ONLY: &[WorldDispatchModeV1] = &[WorldDispatchModeV1::Retained];
const MODE_EPHEMERAL_OR_RETAINED: &[WorldDispatchModeV1] = &[
    WorldDispatchModeV1::Ephemeral,
    WorldDispatchModeV1::Retained,
];

const RUN_WORLD_TASK_ARGUMENTS: &[HostToolModelArgumentFamilyV1] = &[
    HostToolModelArgumentFamilyV1::FreshTargetBackend,
    HostToolModelArgumentFamilyV1::TaskPayload,
];
const SPAWN_WORLD_WORKER_ARGUMENTS: &[HostToolModelArgumentFamilyV1] = &[
    HostToolModelArgumentFamilyV1::FreshTargetBackend,
    HostToolModelArgumentFamilyV1::WorkerSpawnPayload,
];
const FORK_WORLD_WORKER_ARGUMENTS: &[HostToolModelArgumentFamilyV1] = &[
    HostToolModelArgumentFamilyV1::ExactRetainedWorkerHandle,
    HostToolModelArgumentFamilyV1::WorkerForkPayload,
];
const CONTINUE_WORLD_WORKER_ARGUMENTS: &[HostToolModelArgumentFamilyV1] = &[
    HostToolModelArgumentFamilyV1::ExactRetainedWorkerHandle,
    HostToolModelArgumentFamilyV1::ContinueWorldWorkerPayload,
];
const INSPECT_WORLD_WORKER_ARGUMENTS: &[HostToolModelArgumentFamilyV1] = &[
    HostToolModelArgumentFamilyV1::EitherExactFollowUpHandle,
    HostToolModelArgumentFamilyV1::InspectWorldWorkerPayload,
];
const CANCEL_WORLD_WORK_ARGUMENTS: &[HostToolModelArgumentFamilyV1] = &[
    HostToolModelArgumentFamilyV1::EitherExactFollowUpHandle,
    HostToolModelArgumentFamilyV1::CancelWorldWorkPayload,
];
const STOP_WORLD_WORKER_ARGUMENTS: &[HostToolModelArgumentFamilyV1] = &[
    HostToolModelArgumentFamilyV1::ExactRetainedWorkerHandle,
    HostToolModelArgumentFamilyV1::StopWorldWorkerPayload,
];

const HOST_TOOL_CONTRACTS_V1: &[HostToolContractV1] = &[
    HostToolContractV1 {
        tool_name: HostToolNameV1::RunWorldTask,
        allowed_modes: MODE_EPHEMERAL_ONLY,
        follow_up_handle_requirement: HostToolFollowUpHandleRequirementV1::None,
        primary_model_argument_families: RUN_WORLD_TASK_ARGUMENTS,
    },
    HostToolContractV1 {
        tool_name: HostToolNameV1::SpawnWorldWorker,
        allowed_modes: MODE_RETAINED_ONLY,
        follow_up_handle_requirement: HostToolFollowUpHandleRequirementV1::None,
        primary_model_argument_families: SPAWN_WORLD_WORKER_ARGUMENTS,
    },
    HostToolContractV1 {
        tool_name: HostToolNameV1::ForkWorldWorker,
        allowed_modes: MODE_RETAINED_ONLY,
        follow_up_handle_requirement: HostToolFollowUpHandleRequirementV1::ExactRetainedWorker,
        primary_model_argument_families: FORK_WORLD_WORKER_ARGUMENTS,
    },
    HostToolContractV1 {
        tool_name: HostToolNameV1::ContinueWorldWorker,
        allowed_modes: MODE_RETAINED_ONLY,
        follow_up_handle_requirement: HostToolFollowUpHandleRequirementV1::ExactRetainedWorker,
        primary_model_argument_families: CONTINUE_WORLD_WORKER_ARGUMENTS,
    },
    HostToolContractV1 {
        tool_name: HostToolNameV1::InspectWorldWorker,
        allowed_modes: MODE_EPHEMERAL_OR_RETAINED,
        follow_up_handle_requirement:
            HostToolFollowUpHandleRequirementV1::EitherExactActiveTaskOrRetainedWorker,
        primary_model_argument_families: INSPECT_WORLD_WORKER_ARGUMENTS,
    },
    HostToolContractV1 {
        tool_name: HostToolNameV1::CancelWorldWork,
        allowed_modes: MODE_EPHEMERAL_OR_RETAINED,
        follow_up_handle_requirement:
            HostToolFollowUpHandleRequirementV1::EitherExactActiveTaskOrRetainedWorker,
        primary_model_argument_families: CANCEL_WORLD_WORK_ARGUMENTS,
    },
    HostToolContractV1 {
        tool_name: HostToolNameV1::StopWorldWorker,
        allowed_modes: MODE_RETAINED_ONLY,
        follow_up_handle_requirement: HostToolFollowUpHandleRequirementV1::ExactRetainedWorker,
        primary_model_argument_families: STOP_WORLD_WORKER_ARGUMENTS,
    },
];

pub(crate) fn host_tool_contracts_v1() -> &'static [HostToolContractV1] {
    HOST_TOOL_CONTRACTS_V1
}

pub(crate) fn host_tool_contract(tool_name: HostToolNameV1) -> &'static HostToolContractV1 {
    HOST_TOOL_CONTRACTS_V1
        .iter()
        .find(|contract| contract.tool_name == tool_name)
        .expect("host tool contract table must define every frozen tool name")
}

fn canonicalize_handle_value(
    label: &'static str,
    value: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("invalid_follow_up_handle: {label} must be non-empty when provided");
    }

    Ok(Some(trimmed.to_string()))
}

fn canonicalize_required_identity_value(
    field: &'static str,
    value: Option<&str>,
    action: &'static str,
    contract_surface: &'static str,
) -> anyhow::Result<String> {
    canonicalize_handle_value(field, value)?.ok_or_else(|| {
        anyhow::anyhow!(
            "missing_adapter_visible_identity: action {} requires {} for {}",
            action,
            field,
            contract_surface
        )
    })
}

fn ensure_expected_mode(
    action: WorldDispatchActionV1,
    actual_mode: WorldDispatchModeV1,
    expected_mode: WorldDispatchModeV1,
    contract_surface: &'static str,
) -> anyhow::Result<()> {
    if actual_mode != expected_mode {
        bail!(
            "invalid_adapter_visible_mode: action {} emitted mode {} but adapter {} requires {}",
            action.as_str(),
            actual_mode.as_str(),
            contract_surface,
            expected_mode.as_str()
        );
    }
    Ok(())
}

fn normalize_dual_handle_identity(
    action: WorldDispatchActionV1,
    mode: WorldDispatchModeV1,
    target_participant_id: &str,
) -> anyhow::Result<(Option<String>, Option<String>)> {
    let normalized_target = canonicalize_required_identity_value(
        "target_participant_id",
        Some(target_participant_id),
        action.as_str(),
        "exact follow-up result",
    )?;

    match mode {
        WorldDispatchModeV1::Ephemeral => Ok((Some(normalized_target), None)),
        WorldDispatchModeV1::Retained => Ok((None, Some(normalized_target))),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostToolRuntimeDispatchMetadataV1 {
    pub request_id: String,
    pub idempotency_key: String,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
}

impl HostToolRuntimeDispatchMetadataV1 {
    fn validate(&self) -> anyhow::Result<()> {
        require_runtime_owned_field("request_id", &self.request_id)?;
        require_runtime_owned_field("idempotency_key", &self.idempotency_key)?;
        require_runtime_owned_field("orchestration_session_id", &self.orchestration_session_id)?;
        require_runtime_owned_field("caller_participant_id", &self.caller_participant_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostToolRuntimeWorldBindingV1 {
    pub world_id: String,
    pub world_generation: u64,
}

impl HostToolRuntimeWorldBindingV1 {
    fn validate(&self) -> anyhow::Result<()> {
        require_runtime_owned_field("world_id", &self.world_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunWorldTaskToolCallV1 {
    pub target_backend_id: String,
    pub payload: TaskPayloadV1,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpawnWorldWorkerToolCallV1 {
    pub target_backend_id: String,
    pub payload: WorkerSpawnPayloadV1,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostToolInvocationRequestEnvelopeV1 {
    #[serde(default = "host_tool_contract_version_v1")]
    pub version: u32,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct HostToolFollowUpArgumentsV1<P> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    task_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    participant_id: Option<String>,
    payload: P,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TranslatedHostToolInvocationV1 {
    pub tool_name: HostToolNameV1,
    pub dispatch_request: WorldDispatchRequestV1,
    pub tool_call_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedFollowUpDispatchAuthorityV1 {
    pub mode: WorldDispatchModeV1,
    pub target_backend_id: String,
    pub task_run_id: Option<String>,
    pub target_participant_id: Option<String>,
    pub world_binding: HostToolRuntimeWorldBindingV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BuildDispatchRequestArgsV1 {
    tool_name: HostToolNameV1,
    mode: WorldDispatchModeV1,
    target_backend_id: String,
    task_run_id: Option<String>,
    target_participant_id: Option<String>,
    payload: WorldDispatchPayloadV1,
}

fn host_tool_contract_version_v1() -> u32 {
    1
}

pub(crate) fn translate_run_world_task_to_internal_dispatch_request_v1(
    metadata: &HostToolRuntimeDispatchMetadataV1,
    world_binding: &HostToolRuntimeWorldBindingV1,
    call: RunWorldTaskToolCallV1,
) -> anyhow::Result<WorldDispatchRequestV1> {
    build_dispatch_request_v1(
        metadata,
        world_binding,
        BuildDispatchRequestArgsV1 {
            tool_name: HostToolNameV1::RunWorldTask,
            mode: WorldDispatchModeV1::Ephemeral,
            target_backend_id: call.target_backend_id,
            task_run_id: None,
            target_participant_id: None,
            payload: WorldDispatchPayloadV1::Task(call.payload),
        },
    )
}

pub(crate) fn translate_spawn_world_worker_to_internal_dispatch_request_v1(
    metadata: &HostToolRuntimeDispatchMetadataV1,
    world_binding: &HostToolRuntimeWorldBindingV1,
    call: SpawnWorldWorkerToolCallV1,
) -> anyhow::Result<WorldDispatchRequestV1> {
    build_dispatch_request_v1(
        metadata,
        world_binding,
        BuildDispatchRequestArgsV1 {
            tool_name: HostToolNameV1::SpawnWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: call.target_backend_id,
            task_run_id: None,
            target_participant_id: None,
            payload: WorldDispatchPayloadV1::WorkerSpawn(call.payload),
        },
    )
}

pub(crate) fn normalize_run_world_task_receipt_v1(
    outcome: &RunWorldTaskOutcomeV1,
) -> anyhow::Result<HostToolRunWorldTaskReceiptV1> {
    let task_run_id = canonicalize_required_identity_value(
        "task_run_id",
        outcome.task_run_id.as_deref(),
        "run_world_task",
        "exact active-task receipt",
    )?;

    Ok(HostToolRunWorldTaskReceiptV1 {
        request_id: outcome.request_id.clone(),
        orchestration_session_id: outcome.orchestration_session_id.clone(),
        action: outcome.action,
        mode: outcome.mode,
        task_run_id,
        state: outcome.state,
        summary: outcome.summary.clone(),
    })
}

pub(crate) fn normalize_spawn_world_worker_receipt_v1(
    outcome: &SpawnWorldWorkerOutcomeV1,
) -> anyhow::Result<HostToolSpawnWorldWorkerReceiptV1> {
    Ok(HostToolSpawnWorldWorkerReceiptV1 {
        request_id: outcome.request_id.clone(),
        orchestration_session_id: outcome.orchestration_session_id.clone(),
        action: outcome.action,
        mode: outcome.mode,
        participant_id: canonicalize_required_identity_value(
            "participant_id",
            Some(&outcome.participant_id),
            "spawn_world_worker",
            "exact retained-worker receipt",
        )?,
        orchestrator_participant_id: outcome.orchestrator_participant_id.clone(),
        parent_participant_id: outcome.parent_participant_id.clone(),
        resumed_from_participant_id: outcome.resumed_from_participant_id.clone(),
        target_backend_id: outcome.target_backend_id.clone(),
        world_id: outcome.world_id.clone(),
        world_generation: outcome.world_generation,
        launch_span_id: outcome.launch_span_id.clone(),
        summary: outcome.summary.clone(),
    })
}

pub(crate) fn normalize_fork_world_worker_receipt_v1(
    outcome: &ForkWorldWorkerOutcomeV1,
) -> anyhow::Result<HostToolForkWorldWorkerReceiptV1> {
    ensure_expected_mode(
        outcome.action,
        outcome.mode,
        WorldDispatchModeV1::Retained,
        "exact retained-worker receipt",
    )?;
    Ok(HostToolForkWorldWorkerReceiptV1 {
        request_id: outcome.request_id.clone(),
        orchestration_session_id: outcome.orchestration_session_id.clone(),
        action: outcome.action,
        mode: outcome.mode,
        participant_id: canonicalize_required_identity_value(
            "child_participant_id",
            Some(&outcome.child_participant_id),
            "fork_world_worker",
            "exact retained-worker receipt",
        )?,
        orchestrator_participant_id: outcome.orchestrator_participant_id.clone(),
        source_participant_id: canonicalize_required_identity_value(
            "source_participant_id",
            Some(&outcome.source_participant_id),
            "fork_world_worker",
            "exact source retained-worker linkage",
        )?,
        target_backend_id: outcome.target_backend_id.clone(),
        world_id: outcome.world_id.clone(),
        world_generation: outcome.world_generation,
        summary: outcome.summary.clone(),
    })
}

pub(crate) fn normalize_continue_world_worker_outcome_v1(
    outcome: &ContinueWorldWorkerOutcomeV1,
) -> anyhow::Result<HostToolContinueWorldWorkerOutcomeV1> {
    ensure_expected_mode(
        outcome.action,
        outcome.mode,
        WorldDispatchModeV1::Retained,
        "retained-only follow-up result",
    )?;
    Ok(HostToolContinueWorldWorkerOutcomeV1 {
        request_id: outcome.request_id.clone(),
        orchestration_session_id: outcome.orchestration_session_id.clone(),
        action: outcome.action,
        mode: outcome.mode,
        orchestrator_participant_id: outcome.orchestrator_participant_id.clone(),
        participant_id: canonicalize_required_identity_value(
            "target_participant_id",
            Some(&outcome.target_participant_id),
            "continue_world_worker",
            "retained-only follow-up result",
        )?,
        target_backend_id: outcome.target_backend_id.clone(),
        world_id: outcome.world_id.clone(),
        world_generation: outcome.world_generation,
        source_participant_id: outcome.source_participant_id.clone(),
        child_participant_id: outcome.child_participant_id.clone(),
        thread_id: outcome.thread_id.clone(),
        worker_event: outcome.worker_event.clone(),
        summary: outcome.summary.clone(),
    })
}

pub(crate) fn normalize_inspect_world_worker_outcome_v1(
    outcome: &InspectWorldWorkerOutcomeV1,
) -> anyhow::Result<HostToolInspectWorldWorkerOutcomeV1> {
    let (task_run_id, participant_id) = normalize_dual_handle_identity(
        outcome.action,
        outcome.mode,
        &outcome.target_participant_id,
    )?;

    Ok(HostToolInspectWorldWorkerOutcomeV1 {
        request_id: outcome.request_id.clone(),
        orchestration_session_id: outcome.orchestration_session_id.clone(),
        action: outcome.action,
        mode: outcome.mode,
        orchestrator_participant_id: outcome.orchestrator_participant_id.clone(),
        task_run_id,
        participant_id,
        target_backend_id: outcome.target_backend_id.clone(),
        world_id: outcome.world_id.clone(),
        world_generation: outcome.world_generation,
        snapshot: outcome.snapshot.clone(),
        summary: outcome.summary.clone(),
    })
}

pub(crate) fn normalize_cancel_world_work_outcome_v1(
    outcome: &CancelWorldWorkOutcomeV1,
) -> anyhow::Result<HostToolCancelWorldWorkOutcomeV1> {
    let (task_run_id, participant_id) = normalize_dual_handle_identity(
        outcome.action,
        outcome.mode,
        &outcome.target_participant_id,
    )?;

    Ok(HostToolCancelWorldWorkOutcomeV1 {
        request_id: outcome.request_id.clone(),
        orchestration_session_id: outcome.orchestration_session_id.clone(),
        action: outcome.action,
        mode: outcome.mode,
        orchestrator_participant_id: outcome.orchestrator_participant_id.clone(),
        task_run_id,
        participant_id,
        target_backend_id: outcome.target_backend_id.clone(),
        world_id: outcome.world_id.clone(),
        world_generation: outcome.world_generation,
        state: outcome.state,
        closeout: outcome.closeout.clone(),
        summary: outcome.summary.clone(),
    })
}

pub(crate) fn normalize_stop_world_worker_outcome_v1(
    outcome: &StopWorldWorkerOutcomeV1,
) -> anyhow::Result<HostToolStopWorldWorkerOutcomeV1> {
    ensure_expected_mode(
        outcome.action,
        outcome.mode,
        WorldDispatchModeV1::Retained,
        "retained-only follow-up result",
    )?;
    Ok(HostToolStopWorldWorkerOutcomeV1 {
        request_id: outcome.request_id.clone(),
        orchestration_session_id: outcome.orchestration_session_id.clone(),
        action: outcome.action,
        mode: outcome.mode,
        orchestrator_participant_id: outcome.orchestrator_participant_id.clone(),
        participant_id: canonicalize_required_identity_value(
            "target_participant_id",
            Some(&outcome.target_participant_id),
            "stop_world_worker",
            "retained-only follow-up result",
        )?,
        target_backend_id: outcome.target_backend_id.clone(),
        world_id: outcome.world_id.clone(),
        world_generation: outcome.world_generation,
        closeout: outcome.closeout.clone(),
        summary: outcome.summary.clone(),
    })
}

pub(crate) fn resolve_follow_up_dispatch_authority_v1(
    store: &AgentRuntimeStateStore,
    metadata: &HostToolRuntimeDispatchMetadataV1,
    tool_name: HostToolNameV1,
    handle: &HostToolFollowUpHandleV1,
) -> anyhow::Result<ResolvedFollowUpDispatchAuthorityV1> {
    metadata.validate()?;
    ensure_tool_accepts_follow_up_handle(tool_name, handle)?;

    let authority = store.resolve_internal_world_dispatch_caller(
        &metadata.orchestration_session_id,
        &metadata.caller_participant_id,
    )?;
    let world_binding = authoritative_world_binding(&authority.session)?;

    match handle {
        HostToolFollowUpHandleV1::ActiveTask(active_task) => {
            let Some(task) = store.load_active_ephemeral_world_task(
                &metadata.orchestration_session_id,
                &active_task.task_run_id,
            )?
            else {
                bail!(
                    "active_task_not_found: orchestration session {} has no exact active ephemeral task {}",
                    metadata.orchestration_session_id,
                    active_task.task_run_id
                );
            };

            if task.caller_participant_id != authority.caller_participant.participant_id() {
                bail!(
                    "stale_linkage: orchestration session {} active ephemeral task {} is not linked to authoritative orchestrator {}",
                    metadata.orchestration_session_id,
                    active_task.task_run_id,
                    authority.caller_participant.participant_id()
                );
            }
            if task.world_id != world_binding.world_id
                || task.world_generation != world_binding.world_generation
            {
                bail!(
                    "world_binding_mismatch: orchestration session {} active ephemeral task {} no longer matches the authoritative world binding",
                    metadata.orchestration_session_id,
                    active_task.task_run_id
                );
            }

            Ok(ResolvedFollowUpDispatchAuthorityV1 {
                mode: WorldDispatchModeV1::Ephemeral,
                target_backend_id: task.target_backend_id,
                task_run_id: Some(task.task_run_id),
                target_participant_id: None,
                world_binding,
            })
        }
        HostToolFollowUpHandleV1::RetainedWorker(retained_worker) => {
            let Some(record) = store.load_session(&metadata.orchestration_session_id)? else {
                bail!(
                    "missing_orchestration_session: internal world dispatch requires authoritative orchestration session {}",
                    metadata.orchestration_session_id
                );
            };
            let mut matching_participants = record
                .participants
                .iter()
                .filter(|participant| {
                    participant.participant_id() == retained_worker.participant_id
                })
                .cloned()
                .collect::<Vec<_>>();

            if matching_participants.is_empty() {
                bail!(
                    "target_not_in_session: orchestration session {} has no exact retained worker {}",
                    metadata.orchestration_session_id,
                    retained_worker.participant_id
                );
            }
            if matching_participants.len() > 1 {
                bail!(
                    "ambiguous_target_participant: orchestration session {} has multiple retained worker records for {}",
                    metadata.orchestration_session_id,
                    retained_worker.participant_id
                );
            }

            let target_participant = matching_participants
                .pop()
                .expect("retained follow-up target count checked above");
            if target_participant.handle.role != MEMBER_ROLE
                || target_participant.handle.execution.scope != AgentExecutionScope::World
            {
                bail!(
                    "invalid_target_participant: orchestration session {} participant {} is not a retained world worker",
                    metadata.orchestration_session_id,
                    retained_worker.participant_id
                );
            }
            validate_retained_worker_authoritative_lineage(
                &record,
                &authority.caller_participant,
                &target_participant,
            )?;
            if !target_participant.matches_authoritative_parent_world_binding(&authority.session) {
                bail!(
                    "world_binding_mismatch: orchestration session {} retained worker {} no longer matches the authoritative world binding",
                    metadata.orchestration_session_id,
                    retained_worker.participant_id
                );
            }

            match tool_name.retained_follow_up_target_requirement() {
                RetainedFollowUpTargetRequirementV1::LinkedOnly => {}
                RetainedFollowUpTargetRequirementV1::NonTerminal => {
                    if !target_participant.handle.state.is_live()
                        || target_participant.internal.terminal_observed_at.is_some()
                    {
                        bail!(
                            "target_already_terminal: orchestration session {} retained worker {} is already terminal ({})",
                            metadata.orchestration_session_id,
                            retained_worker.participant_id,
                            target_participant.reviewable_terminal_state_label()
                        );
                    }
                }
                RetainedFollowUpTargetRequirementV1::AuthoritativeLive => {
                    if record.live_participants().into_iter().all(|participant| {
                        participant.participant_id() != target_participant.participant_id()
                    }) {
                        bail!(
                            "stale_linkage: orchestration session {} retained worker {} is no longer authoritative-live",
                            metadata.orchestration_session_id,
                            retained_worker.participant_id
                        );
                    }
                }
                RetainedFollowUpTargetRequirementV1::ContinueRoutable => {
                    if !exact_continue_retained_worker_is_routable(
                        &authority.session,
                        &authority.caller_participant,
                        &target_participant,
                    ) {
                        bail!(
                            "stale_linkage: orchestration session {} retained worker {} is no longer authoritative-live",
                            metadata.orchestration_session_id,
                            retained_worker.participant_id
                        );
                    }
                }
            }

            Ok(ResolvedFollowUpDispatchAuthorityV1 {
                mode: WorldDispatchModeV1::Retained,
                target_backend_id: target_participant.handle.backend_id,
                task_run_id: None,
                target_participant_id: Some(retained_worker.participant_id.clone()),
                world_binding,
            })
        }
    }
}

pub(crate) fn translate_follow_up_tool_to_internal_dispatch_request_v1(
    store: &AgentRuntimeStateStore,
    metadata: &HostToolRuntimeDispatchMetadataV1,
    tool_name: HostToolNameV1,
    handle: HostToolFollowUpHandleV1,
    payload: WorldDispatchPayloadV1,
) -> anyhow::Result<WorldDispatchRequestV1> {
    let authority = resolve_follow_up_dispatch_authority_v1(store, metadata, tool_name, &handle)?;
    build_dispatch_request_v1(
        metadata,
        &authority.world_binding,
        BuildDispatchRequestArgsV1 {
            tool_name,
            mode: authority.mode,
            target_backend_id: authority.target_backend_id,
            task_run_id: authority.task_run_id,
            target_participant_id: authority.target_participant_id,
            payload,
        },
    )
}

fn build_dispatch_request_v1(
    metadata: &HostToolRuntimeDispatchMetadataV1,
    world_binding: &HostToolRuntimeWorldBindingV1,
    args: BuildDispatchRequestArgsV1,
) -> anyhow::Result<WorldDispatchRequestV1> {
    let BuildDispatchRequestArgsV1 {
        tool_name,
        mode,
        target_backend_id,
        task_run_id,
        target_participant_id,
        payload,
    } = args;

    metadata.validate()?;
    world_binding.validate()?;
    let contract = tool_name.contract();
    if !contract.accepts_mode(mode) {
        bail!(
            "invalid_dispatch_action_mode: action {} is incompatible with mode {}",
            tool_name.as_str(),
            mode.as_str(),
        );
    }

    let request = WorldDispatchRequestV1 {
        request_id: Some(metadata.request_id.clone()),
        idempotency_key: Some(metadata.idempotency_key.clone()),
        orchestration_session_id: Some(metadata.orchestration_session_id.clone()),
        caller_participant_id: Some(metadata.caller_participant_id.clone()),
        action: tool_name.dispatch_action(),
        mode,
        target_backend_id: Some(target_backend_id),
        task_run_id,
        target_participant_id,
        world_id: Some(world_binding.world_id.clone()),
        world_generation: Some(world_binding.world_generation),
        payload,
    };
    request.clone().validate()?;
    Ok(request)
}

pub(crate) fn authoritative_world_binding_for_session_v1(
    session: &crate::execution::agent_runtime::OrchestrationSessionRecord,
) -> anyhow::Result<HostToolRuntimeWorldBindingV1> {
    authoritative_world_binding(session)
}

pub(crate) fn translate_host_tool_invocation_request_to_internal_dispatch_request_v1(
    store: &AgentRuntimeStateStore,
    metadata: &HostToolRuntimeDispatchMetadataV1,
    world_binding: &HostToolRuntimeWorldBindingV1,
    request: HostToolInvocationRequestEnvelopeV1,
) -> anyhow::Result<TranslatedHostToolInvocationV1> {
    if request.version != host_tool_contract_version_v1() {
        bail!(
            "unsupported_host_toolbox_version: expected version {} but received {}",
            host_tool_contract_version_v1(),
            request.version
        );
    }

    let tool_name = HostToolNameV1::from_str(&request.tool_name)?;
    let dispatch_request = match tool_name {
        HostToolNameV1::RunWorldTask => {
            let call =
                decode_tool_arguments_v1::<RunWorldTaskToolCallV1>(tool_name, request.arguments)?;
            translate_run_world_task_to_internal_dispatch_request_v1(metadata, world_binding, call)?
        }
        HostToolNameV1::SpawnWorldWorker => {
            let call = decode_tool_arguments_v1::<SpawnWorldWorkerToolCallV1>(
                tool_name,
                request.arguments,
            )?;
            translate_spawn_world_worker_to_internal_dispatch_request_v1(
                metadata,
                world_binding,
                call,
            )?
        }
        HostToolNameV1::ForkWorldWorker => translate_follow_up_arguments_v1(
            store,
            metadata,
            tool_name,
            request.arguments,
            WorldDispatchPayloadV1::WorkerFork,
        )?,
        HostToolNameV1::ContinueWorldWorker => translate_follow_up_arguments_v1(
            store,
            metadata,
            tool_name,
            request.arguments,
            WorldDispatchPayloadV1::WorkerContinue,
        )?,
        HostToolNameV1::InspectWorldWorker => translate_follow_up_arguments_v1(
            store,
            metadata,
            tool_name,
            request.arguments,
            WorldDispatchPayloadV1::WorkerInspect,
        )?,
        HostToolNameV1::CancelWorldWork => translate_follow_up_arguments_v1(
            store,
            metadata,
            tool_name,
            request.arguments,
            WorldDispatchPayloadV1::WorkerCancel,
        )?,
        HostToolNameV1::StopWorldWorker => translate_follow_up_arguments_v1(
            store,
            metadata,
            tool_name,
            request.arguments,
            WorldDispatchPayloadV1::WorkerStop,
        )?,
    };

    Ok(TranslatedHostToolInvocationV1 {
        tool_name,
        dispatch_request,
        tool_call_id: canonicalize_handle_value("tool_call_id", request.tool_call_id.as_deref())?,
    })
}

pub(crate) fn normalize_host_tool_invocation_outcome_v1(
    tool_name: HostToolNameV1,
    outcome: &WorldDispatchOutcomeV1,
) -> anyhow::Result<Value> {
    match (tool_name, outcome) {
        (HostToolNameV1::RunWorldTask, WorldDispatchOutcomeV1::RunWorldTask(run)) => {
            serde_json::to_value(normalize_run_world_task_receipt_v1(run)?)
                .context("serialize normalized run_world_task receipt")
        }
        (HostToolNameV1::SpawnWorldWorker, WorldDispatchOutcomeV1::SpawnWorldWorker(spawn)) => {
            serde_json::to_value(normalize_spawn_world_worker_receipt_v1(spawn)?)
                .context("serialize normalized spawn_world_worker receipt")
        }
        (HostToolNameV1::ForkWorldWorker, WorldDispatchOutcomeV1::ForkWorldWorker(fork)) => {
            serde_json::to_value(normalize_fork_world_worker_receipt_v1(fork)?)
                .context("serialize normalized fork_world_worker receipt")
        }
        (
            HostToolNameV1::ContinueWorldWorker,
            WorldDispatchOutcomeV1::ContinueWorldWorker(continue_outcome),
        ) => serde_json::to_value(normalize_continue_world_worker_outcome_v1(
            continue_outcome,
        )?)
        .context("serialize normalized continue_world_worker outcome"),
        (
            HostToolNameV1::InspectWorldWorker,
            WorldDispatchOutcomeV1::InspectWorldWorker(inspect),
        ) => serde_json::to_value(normalize_inspect_world_worker_outcome_v1(inspect)?)
            .context("serialize normalized inspect_world_worker outcome"),
        (HostToolNameV1::CancelWorldWork, WorldDispatchOutcomeV1::CancelWorldWork(cancel)) => {
            serde_json::to_value(normalize_cancel_world_work_outcome_v1(cancel)?)
                .context("serialize normalized cancel_world_work outcome")
        }
        (HostToolNameV1::StopWorldWorker, WorldDispatchOutcomeV1::StopWorldWorker(stop)) => {
            serde_json::to_value(normalize_stop_world_worker_outcome_v1(stop)?)
                .context("serialize normalized stop_world_worker outcome")
        }
        _ => bail!(
            "host_tool_outcome_mismatch: tool {} received incompatible internal outcome",
            tool_name.as_str()
        ),
    }
}

fn decode_tool_arguments_v1<T>(tool_name: HostToolNameV1, arguments: Value) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let decoded: T = serde_json::from_value(arguments.clone()).with_context(|| {
        format!(
            "invalid_tool_arguments: tool {} received arguments that do not match the frozen contract",
            tool_name.as_str()
        )
    })?;
    let normalized = serde_json::to_value(&decoded).with_context(|| {
        format!(
            "invalid_tool_arguments: tool {} failed to re-serialize decoded arguments for exact contract comparison",
            tool_name.as_str()
        )
    })?;
    if normalized != arguments {
        bail!(
            "invalid_tool_arguments: tool {} received arguments that do not match the frozen contract",
            tool_name.as_str()
        );
    }
    Ok(decoded)
}

fn translate_follow_up_arguments_v1<P>(
    store: &AgentRuntimeStateStore,
    metadata: &HostToolRuntimeDispatchMetadataV1,
    tool_name: HostToolNameV1,
    arguments: Value,
    payload_builder: impl FnOnce(P) -> WorldDispatchPayloadV1,
) -> anyhow::Result<WorldDispatchRequestV1>
where
    P: for<'de> Deserialize<'de> + Serialize,
{
    let call = decode_tool_arguments_v1::<HostToolFollowUpArgumentsV1<P>>(tool_name, arguments)?;
    let handle = tool_name
        .contract()
        .follow_up_handle_requirement
        .resolve_exact_handle(call.task_run_id.as_deref(), call.participant_id.as_deref())?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing_follow_up_handle: tool {} requires an exact follow-up handle",
                tool_name.as_str()
            )
        })?;
    translate_follow_up_tool_to_internal_dispatch_request_v1(
        store,
        metadata,
        tool_name,
        handle,
        payload_builder(call.payload),
    )
}

fn ensure_tool_accepts_follow_up_handle(
    tool_name: HostToolNameV1,
    handle: &HostToolFollowUpHandleV1,
) -> anyhow::Result<()> {
    match (tool_name.contract().follow_up_handle_requirement, handle) {
        (HostToolFollowUpHandleRequirementV1::None, _) => bail!(
            "invalid_follow_up_handle: this tool does not accept task_run_id or participant_id"
        ),
        (
            HostToolFollowUpHandleRequirementV1::ExactRetainedWorker,
            HostToolFollowUpHandleV1::ActiveTask(_),
        ) => bail!(
            "invalid_follow_up_handle: retained-worker follow-up requires exact participant_id and does not accept task_run_id"
        ),
        (
            HostToolFollowUpHandleRequirementV1::ExactRetainedWorker,
            HostToolFollowUpHandleV1::RetainedWorker(_),
        )
        | (
            HostToolFollowUpHandleRequirementV1::EitherExactActiveTaskOrRetainedWorker,
            _,
        ) => Ok(()),
    }
}

fn authoritative_world_binding(
    session: &crate::execution::agent_runtime::OrchestrationSessionRecord,
) -> anyhow::Result<HostToolRuntimeWorldBindingV1> {
    let world_id = session.world_id.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "missing_world_binding: orchestration session {} has no authoritative world binding",
            session.orchestration_session_id
        )
    })?;
    let world_generation = session.world_generation.ok_or_else(|| {
        anyhow::anyhow!(
            "missing_world_binding: orchestration session {} has no authoritative world binding",
            session.orchestration_session_id
        )
    })?;
    Ok(HostToolRuntimeWorldBindingV1 {
        world_id,
        world_generation,
    })
}

fn require_runtime_owned_field(field: &'static str, value: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        bail!("missing_runtime_owned_field: host tool adapter requires {field}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::{
        host_tool_contract, host_tool_contracts_v1, normalize_cancel_world_work_outcome_v1,
        normalize_continue_world_worker_outcome_v1, normalize_inspect_world_worker_outcome_v1,
        normalize_run_world_task_receipt_v1, normalize_spawn_world_worker_receipt_v1,
        normalize_stop_world_worker_outcome_v1, resolve_follow_up_dispatch_authority_v1,
        translate_follow_up_tool_to_internal_dispatch_request_v1,
        translate_host_tool_invocation_request_to_internal_dispatch_request_v1,
        translate_run_world_task_to_internal_dispatch_request_v1,
        translate_spawn_world_worker_to_internal_dispatch_request_v1,
        HostToolFollowUpHandleRequirementV1, HostToolFollowUpHandleV1,
        HostToolInvocationRequestEnvelopeV1, HostToolModelArgumentFamilyV1, HostToolNameV1,
        HostToolRuntimeDispatchMetadataV1, HostToolRuntimeWorldBindingV1, RunWorldTaskToolCallV1,
        SpawnWorldWorkerToolCallV1,
    };
    use crate::execution::agent_runtime::dispatch_contract::{
        CancelWorldWorkOutcomeV1, ContinueWorldWorkerEventClassV1, ContinueWorldWorkerEventV1,
        ContinueWorldWorkerOutcomeV1, InspectWorldWorkerOutcomeV1, RetainedWorkerCancelCloseoutV1,
        RetainedWorkerInspectSnapshotV1, RetainedWorkerStopCloseoutV1, RunWorldTaskOutcomeV1,
        SpawnWorldWorkerOutcomeV1, StopWorldWorkerOutcomeV1, WorkerCancelPayloadV1,
        WorkerContinuePayloadV1, WorkerForkPayloadV1, WorkerInspectPayloadV1, WorkerStopPayloadV1,
        WorldTaskTerminalStateV1,
    };
    use crate::execution::agent_runtime::mapping::AgentRuntimeBackendKind;
    use crate::execution::agent_runtime::orchestration_session::{
        HostAttachContract, OrchestrationSessionState,
    };
    use crate::execution::agent_runtime::session::{
        AgentRuntimeParticipantRecord, AgentRuntimeSessionState,
    };
    use crate::execution::agent_runtime::state_store::ActiveEphemeralWorldTaskRecord;
    use crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor;
    use crate::execution::agent_runtime::{
        AgentRuntimeStateStore, WorldDispatchActionV1, WorldDispatchModeV1, WorldDispatchPayloadV1,
    };
    use crate::execution::config_model::AgentExecutionScope;

    #[test]
    fn dispatch_contract_adapter_freezes_exactly_seven_host_tool_names() {
        let names: Vec<_> = host_tool_contracts_v1()
            .iter()
            .map(|contract| contract.tool_name.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "run_world_task",
                "spawn_world_worker",
                "fork_world_worker",
                "continue_world_worker",
                "inspect_world_worker",
                "cancel_world_work",
                "stop_world_worker",
            ]
        );
        assert_eq!(
            names.iter().copied().collect::<BTreeSet<_>>().len(),
            HostToolNameV1::ALL.len(),
            "tool vocabulary must stay one-to-one and duplicate-free"
        );
    }

    #[test]
    fn dispatch_contract_adapter_maps_one_to_one_to_internal_dispatch_actions() {
        let adapter_names: Vec<_> = host_tool_contracts_v1()
            .iter()
            .map(|contract| contract.dispatch_action().as_str())
            .collect();
        let internal_names: Vec<_> = HostToolNameV1::ALL
            .into_iter()
            .map(|tool_name| tool_name.dispatch_action().as_str())
            .collect();
        assert_eq!(adapter_names, internal_names);
        assert_eq!(
            adapter_names.iter().copied().collect::<BTreeSet<_>>(),
            [
                WorldDispatchActionV1::RunWorldTask.as_str(),
                WorldDispatchActionV1::SpawnWorldWorker.as_str(),
                WorldDispatchActionV1::ForkWorldWorker.as_str(),
                WorldDispatchActionV1::ContinueWorldWorker.as_str(),
                WorldDispatchActionV1::InspectWorldWorker.as_str(),
                WorldDispatchActionV1::CancelWorldWork.as_str(),
                WorldDispatchActionV1::StopWorldWorker.as_str(),
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn dispatch_contract_adapter_freezes_primary_model_argument_families() {
        let run_world_task = host_tool_contract(HostToolNameV1::RunWorldTask);
        assert_eq!(
            run_world_task.primary_model_argument_families,
            &[
                HostToolModelArgumentFamilyV1::FreshTargetBackend,
                HostToolModelArgumentFamilyV1::TaskPayload,
            ]
        );
        assert_eq!(
            host_tool_contract(HostToolNameV1::SpawnWorldWorker).primary_model_argument_families,
            &[
                HostToolModelArgumentFamilyV1::FreshTargetBackend,
                HostToolModelArgumentFamilyV1::WorkerSpawnPayload,
            ]
        );
        assert_eq!(
            host_tool_contract(HostToolNameV1::ForkWorldWorker).primary_model_argument_families,
            &[
                HostToolModelArgumentFamilyV1::ExactRetainedWorkerHandle,
                HostToolModelArgumentFamilyV1::WorkerForkPayload,
            ]
        );
        assert_eq!(
            host_tool_contract(HostToolNameV1::ContinueWorldWorker).primary_model_argument_families,
            &[
                HostToolModelArgumentFamilyV1::ExactRetainedWorkerHandle,
                HostToolModelArgumentFamilyV1::ContinueWorldWorkerPayload,
            ]
        );
        assert_eq!(
            host_tool_contract(HostToolNameV1::InspectWorldWorker).primary_model_argument_families,
            &[
                HostToolModelArgumentFamilyV1::EitherExactFollowUpHandle,
                HostToolModelArgumentFamilyV1::InspectWorldWorkerPayload,
            ]
        );
        assert_eq!(
            host_tool_contract(HostToolNameV1::CancelWorldWork).primary_model_argument_families,
            &[
                HostToolModelArgumentFamilyV1::EitherExactFollowUpHandle,
                HostToolModelArgumentFamilyV1::CancelWorldWorkPayload,
            ]
        );
        assert_eq!(
            host_tool_contract(HostToolNameV1::StopWorldWorker).primary_model_argument_families,
            &[
                HostToolModelArgumentFamilyV1::ExactRetainedWorkerHandle,
                HostToolModelArgumentFamilyV1::StopWorldWorkerPayload,
            ]
        );
    }

    #[test]
    fn dispatch_contract_adapter_freezes_expected_mode_surface_per_tool() {
        assert!(host_tool_contract(HostToolNameV1::RunWorldTask)
            .accepts_mode(WorldDispatchModeV1::Ephemeral));
        assert!(host_tool_contract(HostToolNameV1::SpawnWorldWorker)
            .accepts_mode(WorldDispatchModeV1::Retained));
        assert!(host_tool_contract(HostToolNameV1::ForkWorldWorker)
            .accepts_mode(WorldDispatchModeV1::Retained));
        assert!(host_tool_contract(HostToolNameV1::ContinueWorldWorker)
            .accepts_mode(WorldDispatchModeV1::Retained));
        assert!(host_tool_contract(HostToolNameV1::InspectWorldWorker)
            .accepts_mode(WorldDispatchModeV1::Ephemeral));
        assert!(host_tool_contract(HostToolNameV1::InspectWorldWorker)
            .accepts_mode(WorldDispatchModeV1::Retained));
        assert!(host_tool_contract(HostToolNameV1::CancelWorldWork)
            .accepts_mode(WorldDispatchModeV1::Ephemeral));
        assert!(host_tool_contract(HostToolNameV1::CancelWorldWork)
            .accepts_mode(WorldDispatchModeV1::Retained));
        assert!(host_tool_contract(HostToolNameV1::StopWorldWorker)
            .accepts_mode(WorldDispatchModeV1::Retained));
    }

    #[test]
    fn dispatch_contract_adapter_rejects_handles_for_fresh_allocation_tools() {
        let err = HostToolFollowUpHandleRequirementV1::None
            .resolve_exact_handle(Some("task-run-1"), None)
            .expect_err("fresh-allocation tools must reject follow-up handles");
        assert_eq!(
            err.to_string(),
            "invalid_follow_up_handle: this tool does not accept task_run_id or participant_id"
        );
    }

    #[test]
    fn dispatch_contract_adapter_requires_exact_retained_worker_handle_for_retained_only_follow_up_tools(
    ) {
        let handle = HostToolFollowUpHandleRequirementV1::ExactRetainedWorker
            .resolve_exact_handle(None, Some(" ash-worker-41 "))
            .expect("retained worker handle");
        assert_eq!(
            handle,
            Some(HostToolFollowUpHandleV1::RetainedWorker(
                super::RetainedWorkerHandleV1 {
                    participant_id: "ash-worker-41".to_string(),
                }
            ))
        );

        let err = HostToolFollowUpHandleRequirementV1::ExactRetainedWorker
            .resolve_exact_handle(Some("task-run-41"), None)
            .expect_err("task_run_id must fail for retained-only tools");
        assert_eq!(
            err.to_string(),
            "invalid_follow_up_handle: retained-worker follow-up requires exact participant_id and does not accept task_run_id"
        );
    }

    #[test]
    fn dispatch_contract_adapter_requires_exactly_one_follow_up_handle_family_for_dual_handle_tools(
    ) {
        let task_handle =
            HostToolFollowUpHandleRequirementV1::EitherExactActiveTaskOrRetainedWorker
                .resolve_exact_handle(Some(" task-run-51 "), None)
                .expect("active task handle");
        assert_eq!(
            task_handle,
            Some(HostToolFollowUpHandleV1::ActiveTask(
                super::ActiveTaskHandleV1 {
                    task_run_id: "task-run-51".to_string(),
                }
            ))
        );

        let retained_handle =
            HostToolFollowUpHandleRequirementV1::EitherExactActiveTaskOrRetainedWorker
                .resolve_exact_handle(None, Some(" ash-worker-51 "))
                .expect("retained worker handle");
        assert_eq!(
            retained_handle,
            Some(HostToolFollowUpHandleV1::RetainedWorker(
                super::RetainedWorkerHandleV1 {
                    participant_id: "ash-worker-51".to_string(),
                }
            ))
        );

        let mixed_err = HostToolFollowUpHandleRequirementV1::EitherExactActiveTaskOrRetainedWorker
            .resolve_exact_handle(Some("task-run-51"), Some("ash-worker-51"))
            .expect_err("mixed handle kinds must fail closed");
        assert_eq!(
            mixed_err.to_string(),
            "invalid_follow_up_handle: follow-up requires exactly one exact handle family (task_run_id or participant_id, not both)"
        );

        let missing_err =
            HostToolFollowUpHandleRequirementV1::EitherExactActiveTaskOrRetainedWorker
                .resolve_exact_handle(None, None)
                .expect_err("missing handle must fail closed");
        assert_eq!(
            missing_err.to_string(),
            "missing_follow_up_handle: follow-up requires exact task_run_id or exact participant_id"
        );
    }

    #[test]
    fn dispatch_contract_adapter_rejects_blank_follow_up_handle_values() {
        let err = HostToolFollowUpHandleRequirementV1::EitherExactActiveTaskOrRetainedWorker
            .resolve_exact_handle(Some("   "), None)
            .expect_err("blank task_run_id must fail closed");
        assert_eq!(
            err.to_string(),
            "invalid_follow_up_handle: task_run_id must be non-empty when provided"
        );
    }

    #[test]
    fn dispatch_contract_adapter_translates_run_world_task_with_runtime_owned_injection() {
        let metadata = sample_runtime_metadata();
        let world_binding = sample_world_binding();

        let request = translate_run_world_task_to_internal_dispatch_request_v1(
            &metadata,
            &world_binding,
            RunWorldTaskToolCallV1 {
                target_backend_id: "cli:codex_world".to_string(),
                payload: crate::execution::agent_runtime::TaskPayloadV1 {
                    prompt: "Scan the workspace and summarize the failing test.".to_string(),
                },
            },
        )
        .expect("translate run_world_task");

        let validated = request
            .clone()
            .validate()
            .expect("validated dispatch request");
        assert_eq!(request.action, WorldDispatchActionV1::RunWorldTask);
        assert_eq!(request.mode, WorldDispatchModeV1::Ephemeral);
        assert_eq!(request.request_id.as_deref(), Some("req-packet2-run"));
        assert_eq!(request.idempotency_key.as_deref(), Some("idem-packet2-run"));
        assert_eq!(
            request.orchestration_session_id.as_deref(),
            Some("sess_packet2")
        );
        assert_eq!(
            request.caller_participant_id.as_deref(),
            Some("orch_packet2")
        );
        assert_eq!(
            request.target_backend_id.as_deref(),
            Some("cli:codex_world")
        );
        assert_eq!(request.world_id.as_deref(), Some("world-17"));
        assert_eq!(request.world_generation, Some(2));
        assert!(request.task_run_id.is_none());
        assert!(request.target_participant_id.is_none());
        assert_eq!(validated.idempotency_key, "idem-packet2-run");
    }

    #[test]
    fn dispatch_contract_adapter_translates_spawn_world_worker_with_runtime_owned_injection() {
        let metadata = sample_runtime_metadata();
        let world_binding = sample_world_binding();

        let request = translate_spawn_world_worker_to_internal_dispatch_request_v1(
            &metadata,
            &world_binding,
            SpawnWorldWorkerToolCallV1 {
                target_backend_id: "cli:claude_code_world".to_string(),
                payload: crate::execution::agent_runtime::WorkerSpawnPayloadV1 {
                    prompt: "Bootstrap a retained worker for the integration test triage."
                        .to_string(),
                },
            },
        )
        .expect("translate spawn_world_worker");

        let validated = request
            .clone()
            .validate()
            .expect("validated dispatch request");
        assert_eq!(request.action, WorldDispatchActionV1::SpawnWorldWorker);
        assert_eq!(request.mode, WorldDispatchModeV1::Retained);
        assert_eq!(request.request_id.as_deref(), Some("req-packet2-run"));
        assert_eq!(request.idempotency_key.as_deref(), Some("idem-packet2-run"));
        assert_eq!(
            request.target_backend_id.as_deref(),
            Some("cli:claude_code_world")
        );
        assert_eq!(request.world_id.as_deref(), Some("world-17"));
        assert_eq!(request.world_generation, Some(2));
        assert!(request.task_run_id.is_none());
        assert!(request.target_participant_id.is_none());
        assert_eq!(validated.target_backend_id, "cli:claude_code_world");
    }

    #[test]
    fn dispatch_contract_adapter_rejects_host_tool_envelope_unknown_top_level_fields() {
        let err =
            serde_json::from_value::<HostToolInvocationRequestEnvelopeV1>(serde_json::json!({
                "version": 1,
                "tool_name": "run_world_task",
                "tool_call_id": "call-packet3",
                "arguments": {
                    "target_backend_id": "cli:codex_world",
                    "payload": {
                        "prompt": "Run the task."
                    }
                },
                "request_id": "runtime-owned"
            }))
            .expect_err("host tool envelope must fail closed on unknown top-level fields");

        assert!(
            err.to_string().contains("unknown field `request_id`"),
            "unexpected serde error for unknown host-tool envelope field: {err}"
        );
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_rejects_unknown_runtime_owned_argument_keys_for_run_world_task() {
        with_store(|store| {
            let err = translate_host_tool_invocation_request_to_internal_dispatch_request_v1(
                store,
                &sample_runtime_metadata(),
                &sample_world_binding(),
                HostToolInvocationRequestEnvelopeV1 {
                    version: 1,
                    tool_name: "run_world_task".to_string(),
                    tool_call_id: Some("call-packet3-run".to_string()),
                    arguments: serde_json::json!({
                        "target_backend_id": "cli:codex_world",
                        "payload": {
                            "prompt": "Run the task."
                        },
                        "world_id": "runtime-owned-field-must-be-rejected"
                    }),
                },
            )
            .expect_err(
                "run_world_task bridge must fail closed on unknown/runtime-owned argument keys",
            );

            assert_eq!(
                err.to_string(),
                "invalid_tool_arguments: tool run_world_task received arguments that do not match the frozen contract"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_rejects_unknown_nested_payload_keys_for_run_world_task() {
        with_store(|store| {
            let err = translate_host_tool_invocation_request_to_internal_dispatch_request_v1(
                store,
                &sample_runtime_metadata(),
                &sample_world_binding(),
                HostToolInvocationRequestEnvelopeV1 {
                    version: 1,
                    tool_name: "run_world_task".to_string(),
                    tool_call_id: Some("call-packet3-run-nested".to_string()),
                    arguments: serde_json::json!({
                        "target_backend_id": "cli:codex_world",
                        "payload": {
                            "prompt": "Run the task.",
                            "world_generation": 99
                        }
                    }),
                },
            )
            .expect_err("run_world_task bridge must fail closed on unknown nested payload keys");

            assert_eq!(
                err.to_string(),
                "invalid_tool_arguments: tool run_world_task received arguments that do not match the frozen contract"
            );
        });
    }

    #[test]
    fn dispatch_contract_adapter_normalizes_run_world_task_into_exact_active_task_receipt() {
        let receipt = normalize_run_world_task_receipt_v1(&RunWorldTaskOutcomeV1 {
            request_id: "req-packet3-run".to_string(),
            orchestration_session_id: "sess_packet3".to_string(),
            action: WorldDispatchActionV1::RunWorldTask,
            mode: WorldDispatchModeV1::Ephemeral,
            task_run_id: Some("task-run-packet3".to_string()),
            state: WorldTaskTerminalStateV1::NeedsRetainedFollowup,
            summary: "run receipt keeps exact task identity".to_string(),
        })
        .expect("normalize run receipt");

        assert_eq!(receipt.task_run_id, "task-run-packet3");
        assert_eq!(receipt.mode, WorldDispatchModeV1::Ephemeral);
        assert_eq!(
            receipt.state,
            WorldTaskTerminalStateV1::NeedsRetainedFollowup
        );
    }

    #[test]
    fn dispatch_contract_adapter_rejects_run_world_task_receipt_without_exact_task_run_id() {
        let err = normalize_run_world_task_receipt_v1(&RunWorldTaskOutcomeV1 {
            request_id: "req-packet3-run".to_string(),
            orchestration_session_id: "sess_packet3".to_string(),
            action: WorldDispatchActionV1::RunWorldTask,
            mode: WorldDispatchModeV1::Ephemeral,
            task_run_id: None,
            state: WorldTaskTerminalStateV1::Failed,
            summary: "run receipt is missing task identity".to_string(),
        })
        .expect_err("missing task_run_id must fail closed");

        assert_eq!(
            err.to_string(),
            "missing_adapter_visible_identity: action run_world_task requires task_run_id for exact active-task receipt"
        );
    }

    #[test]
    fn dispatch_contract_adapter_normalizes_spawn_world_worker_into_exact_retained_receipt() {
        let receipt = normalize_spawn_world_worker_receipt_v1(&SpawnWorldWorkerOutcomeV1 {
            request_id: "req-packet3-spawn".to_string(),
            orchestration_session_id: "sess_packet3".to_string(),
            action: WorldDispatchActionV1::SpawnWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            participant_id: "ash_packet3".to_string(),
            orchestrator_participant_id: "orch_packet3".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-17".to_string(),
            world_generation: 2,
            launch_span_id: "spn-packet3".to_string(),
            summary: "spawn receipt keeps exact retained identity".to_string(),
        })
        .expect("normalize spawn receipt");

        assert_eq!(receipt.participant_id, "ash_packet3");
        assert_eq!(receipt.target_backend_id, "cli:codex_world");
        assert_eq!(receipt.world_id, "world-17");
    }

    #[test]
    fn dispatch_contract_adapter_normalizes_dual_handle_outcomes_to_exact_identity_family() {
        let snapshot = RetainedWorkerInspectSnapshotV1 {
            participant_state: crate::execution::agent_runtime::AgentRuntimeSessionState::Running,
            session_state: OrchestrationSessionState::Active,
            session_posture:
                crate::execution::agent_runtime::orchestration_session::OrchestrationSessionPosture::ActiveAttached,
            authoritative_live: true,
            attention_required: false,
            parent_participant_id: None,
            resumed_from_participant_id: None,
        };
        let retained_cancel_closeout = RetainedWorkerCancelCloseoutV1 {
            participant_state: None,
            session_state: None,
        };

        let inspect_ephemeral =
            normalize_inspect_world_worker_outcome_v1(&InspectWorldWorkerOutcomeV1 {
                request_id: "req-packet3-inspect-e".to_string(),
                orchestration_session_id: "sess_packet3".to_string(),
                action: WorldDispatchActionV1::InspectWorldWorker,
                mode: WorldDispatchModeV1::Ephemeral,
                orchestrator_participant_id: "orch_packet3".to_string(),
                target_participant_id: "task-run-packet3".to_string(),
                target_backend_id: "cli:codex_world".to_string(),
                world_id: "world-17".to_string(),
                world_generation: 2,
                snapshot: snapshot.clone(),
                summary: "inspect exposes canonical active-task identity".to_string(),
            })
            .expect("normalize ephemeral inspect");
        assert_eq!(
            inspect_ephemeral.task_run_id.as_deref(),
            Some("task-run-packet3")
        );
        assert_eq!(inspect_ephemeral.participant_id, None);

        let inspect_retained =
            normalize_inspect_world_worker_outcome_v1(&InspectWorldWorkerOutcomeV1 {
                request_id: "req-packet3-inspect-r".to_string(),
                orchestration_session_id: "sess_packet3".to_string(),
                action: WorldDispatchActionV1::InspectWorldWorker,
                mode: WorldDispatchModeV1::Retained,
                orchestrator_participant_id: "orch_packet3".to_string(),
                target_participant_id: "ash_packet3".to_string(),
                target_backend_id: "cli:codex_world".to_string(),
                world_id: "world-17".to_string(),
                world_generation: 2,
                snapshot: snapshot.clone(),
                summary: "inspect exposes canonical retained identity".to_string(),
            })
            .expect("normalize retained inspect");
        assert_eq!(inspect_retained.task_run_id, None);
        assert_eq!(
            inspect_retained.participant_id.as_deref(),
            Some("ash_packet3")
        );

        let cancel_ephemeral = normalize_cancel_world_work_outcome_v1(&CancelWorldWorkOutcomeV1 {
            request_id: "req-packet3-cancel-e".to_string(),
            orchestration_session_id: "sess_packet3".to_string(),
            action: WorldDispatchActionV1::CancelWorldWork,
            mode: WorldDispatchModeV1::Ephemeral,
            orchestrator_participant_id: "orch_packet3".to_string(),
            target_participant_id: "task-run-cancel-packet3".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-17".to_string(),
            world_generation: 2,
            state: crate::execution::agent_runtime::dispatch_contract::CancelWorldWorkTerminalStateV1::Cancelled,
            closeout: retained_cancel_closeout.clone(),
            summary: "cancel exposes canonical active-task identity".to_string(),
        })
        .expect("normalize ephemeral cancel");
        assert_eq!(
            cancel_ephemeral.task_run_id.as_deref(),
            Some("task-run-cancel-packet3")
        );
        assert_eq!(cancel_ephemeral.participant_id, None);

        let cancel_retained = normalize_cancel_world_work_outcome_v1(&CancelWorldWorkOutcomeV1 {
            request_id: "req-packet3-cancel-r".to_string(),
            orchestration_session_id: "sess_packet3".to_string(),
            action: WorldDispatchActionV1::CancelWorldWork,
            mode: WorldDispatchModeV1::Retained,
            orchestrator_participant_id: "orch_packet3".to_string(),
            target_participant_id: "ash_packet3".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-17".to_string(),
            world_generation: 2,
            state: crate::execution::agent_runtime::dispatch_contract::CancelWorldWorkTerminalStateV1::Cancelled,
            closeout: retained_cancel_closeout,
            summary: "cancel exposes canonical retained identity".to_string(),
        })
        .expect("normalize retained cancel");
        assert_eq!(cancel_retained.task_run_id, None);
        assert_eq!(
            cancel_retained.participant_id.as_deref(),
            Some("ash_packet3")
        );
    }

    #[test]
    fn dispatch_contract_adapter_normalizes_retained_only_follow_up_outcomes() {
        let continue_outcome =
            normalize_continue_world_worker_outcome_v1(&ContinueWorldWorkerOutcomeV1 {
                request_id: "req-packet3-continue".to_string(),
                orchestration_session_id: "sess_packet3".to_string(),
                action: WorldDispatchActionV1::ContinueWorldWorker,
                mode: WorldDispatchModeV1::Retained,
                orchestrator_participant_id: "orch_packet3".to_string(),
                target_participant_id: "ash_packet3".to_string(),
                target_backend_id: "cli:codex_world".to_string(),
                world_id: "world-17".to_string(),
                world_generation: 2,
                source_participant_id: None,
                child_participant_id: None,
                thread_id: Some("thread-packet3".to_string()),
                worker_event: Some(ContinueWorldWorkerEventV1 {
                    event_class: ContinueWorldWorkerEventClassV1::ProgressUpdate,
                    source_participant_id: "ash_packet3".to_string(),
                    target_participant_id: "ash_packet3".to_string(),
                    source_backend_id: "cli:codex_world".to_string(),
                    attention_required: false,
                    thread_id: Some("thread-packet3".to_string()),
                    stream_channel: Some("stderr".to_string()),
                    payload: serde_json::json!({
                        "event_kind": "progress_update",
                        "detail": "kept retained-only"
                    }),
                }),
                summary: "continue stays retained-only".to_string(),
            })
            .expect("normalize continue");
        assert_eq!(continue_outcome.participant_id, "ash_packet3");
        assert_eq!(
            continue_outcome.thread_id.as_deref(),
            Some("thread-packet3")
        );

        let stop_outcome = normalize_stop_world_worker_outcome_v1(&StopWorldWorkerOutcomeV1 {
            request_id: "req-packet3-stop".to_string(),
            orchestration_session_id: "sess_packet3".to_string(),
            action: WorldDispatchActionV1::StopWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            orchestrator_participant_id: "orch_packet3".to_string(),
            target_participant_id: "ash_packet3".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-17".to_string(),
            world_generation: 2,
            closeout: RetainedWorkerStopCloseoutV1 {
                participant_state:
                    crate::execution::agent_runtime::AgentRuntimeSessionState::Stopped,
                session_state: OrchestrationSessionState::Stopped,
            },
            summary: "stop stays retained-only".to_string(),
        })
        .expect("normalize stop");
        assert_eq!(stop_outcome.participant_id, "ash_packet3");
    }

    #[test]
    fn dispatch_contract_adapter_rejects_non_retained_mode_for_retained_only_outcomes() {
        let err = normalize_stop_world_worker_outcome_v1(&StopWorldWorkerOutcomeV1 {
            request_id: "req-packet3-stop".to_string(),
            orchestration_session_id: "sess_packet3".to_string(),
            action: WorldDispatchActionV1::StopWorldWorker,
            mode: WorldDispatchModeV1::Ephemeral,
            orchestrator_participant_id: "orch_packet3".to_string(),
            target_participant_id: "ash_packet3".to_string(),
            target_backend_id: "cli:codex_world".to_string(),
            world_id: "world-17".to_string(),
            world_generation: 2,
            closeout: RetainedWorkerStopCloseoutV1 {
                participant_state:
                    crate::execution::agent_runtime::AgentRuntimeSessionState::Stopped,
                session_state: OrchestrationSessionState::Stopped,
            },
            summary: "invalid stop mode".to_string(),
        })
        .expect_err("stop normalization must fail closed on non-retained mode");

        assert_eq!(
            err.to_string(),
            "invalid_adapter_visible_mode: action stop_world_worker emitted mode ephemeral but adapter retained-only follow-up result requires retained"
        );
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_uses_authoritative_retained_runtime_state() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_packet2", "orch_packet2");
            let member = live_member(
                "codex_world",
                "sess_packet2",
                "worker_packet2",
                "orch_packet2",
            );
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let metadata = sample_runtime_metadata();
            let resolved = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::ContinueWorldWorker,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2".to_string(),
                }),
            )
            .expect("resolve retained follow-up authority");

            assert_eq!(resolved.mode, WorldDispatchModeV1::Retained);
            assert_eq!(resolved.target_backend_id, "cli:codex_world");
            assert_eq!(
                resolved.target_participant_id.as_deref(),
                Some("worker_packet2")
            );
            assert_eq!(resolved.world_binding.world_id, "world-17");
            assert_eq!(resolved.world_binding.world_generation, 2);

            let request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::ContinueWorldWorker,
                HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2".to_string(),
                }),
                WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                    prompt: "Continue from the latest retained worker checkpoint.".to_string(),
                    thread_id: Some("thread-packet2-retained".to_string()),
                }),
            )
            .expect("translate retained follow-up request");

            let validated = request.validate().expect("validate retained follow-up");
            assert_eq!(validated.mode, WorldDispatchModeV1::Retained);
            assert_eq!(validated.target_backend_id, "cli:codex_world");
            assert_eq!(
                validated.target_participant_id.as_deref(),
                Some("worker_packet2")
            );
            assert_eq!(validated.world_id, "world-17");
            assert_eq!(validated.world_generation, 2);
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_uses_authoritative_active_task_state() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_packet2", "orch_packet2");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");

            let guard = store
                .register_active_ephemeral_world_task(ActiveEphemeralWorldTaskRecord {
                    orchestration_session_id: "sess_packet2".to_string(),
                    task_run_id: "task-run-packet2".to_string(),
                    caller_participant_id: "orch_packet2".to_string(),
                    target_backend_id: "cli:codex_world".to_string(),
                    world_id: "world-17".to_string(),
                    world_generation: 2,
                })
                .expect("register active task");

            let metadata = sample_runtime_metadata();
            let resolved = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::CancelWorldWork,
                &HostToolFollowUpHandleV1::ActiveTask(super::ActiveTaskHandleV1 {
                    task_run_id: "task-run-packet2".to_string(),
                }),
            )
            .expect("resolve active-task follow-up authority");

            assert_eq!(resolved.mode, WorldDispatchModeV1::Ephemeral);
            assert_eq!(resolved.target_backend_id, "cli:codex_world");
            assert_eq!(resolved.task_run_id.as_deref(), Some("task-run-packet2"));
            assert!(resolved.target_participant_id.is_none());
            assert_eq!(resolved.world_binding.world_id, "world-17");
            assert_eq!(resolved.world_binding.world_generation, 2);

            let inspect_request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::InspectWorldWorker,
                HostToolFollowUpHandleV1::ActiveTask(super::ActiveTaskHandleV1 {
                    task_run_id: "task-run-packet2".to_string(),
                }),
                WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
            )
            .expect("translate active inspect request");
            let cancel_request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::CancelWorldWork,
                HostToolFollowUpHandleV1::ActiveTask(super::ActiveTaskHandleV1 {
                    task_run_id: "task-run-packet2".to_string(),
                }),
                WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1 {
                    reason: Some("host_requested_stop".to_string()),
                    graceful: Some(true),
                }),
            )
            .expect("translate active cancel request");

            let inspect_validated = inspect_request
                .validate()
                .expect("validate inspect request");
            assert_eq!(inspect_validated.mode, WorldDispatchModeV1::Ephemeral);
            assert_eq!(
                inspect_validated.task_run_id.as_deref(),
                Some("task-run-packet2")
            );
            assert!(inspect_validated.target_participant_id.is_none());

            let cancel_validated = cancel_request.validate().expect("validate cancel request");
            assert_eq!(cancel_validated.mode, WorldDispatchModeV1::Ephemeral);
            assert_eq!(
                cancel_validated.task_run_id.as_deref(),
                Some("task-run-packet2")
            );
            assert_eq!(cancel_validated.world_id, "world-17");

            drop(guard);
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_allows_retained_inspect_for_non_live_worker()
    {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_packet2", "orch_packet2");
            let mut member = live_member(
                "codex_world",
                "sess_packet2",
                "worker_packet2_terminal",
                "orch_packet2",
            );
            member.mark_terminal_state("worker_finished");
            member.transition_state(
                crate::execution::agent_runtime::AgentRuntimeSessionState::Stopped,
            );

            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let metadata = sample_runtime_metadata();
            let resolved = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::InspectWorldWorker,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_terminal".to_string(),
                }),
            )
            .expect("resolve retained inspect authority for non-live worker");

            assert_eq!(resolved.mode, WorldDispatchModeV1::Retained);
            assert_eq!(resolved.target_backend_id, "cli:codex_world");
            assert_eq!(
                resolved.target_participant_id.as_deref(),
                Some("worker_packet2_terminal")
            );

            let request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::InspectWorldWorker,
                HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_terminal".to_string(),
                }),
                WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
            )
            .expect("translate retained inspect request for non-live worker");

            let validated = request
                .validate()
                .expect("validate retained inspect request");
            assert_eq!(validated.mode, WorldDispatchModeV1::Retained);
            assert_eq!(
                validated.target_participant_id.as_deref(),
                Some("worker_packet2_terminal")
            );
            assert_eq!(validated.target_backend_id, "cli:codex_world");
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_accepts_successor_authority_only_for_retained_fork_and_cancel(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_packet2", "orch_packet2_launch");
            let mut successor =
                live_orchestrator("codex", "sess_packet2", "orch_packet2_successor");
            successor.handle.resumed_from_participant_id = Some("orch_packet2_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_packet2_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let member = live_member(
                "codex_world",
                "sess_packet2",
                "worker_packet2_retained",
                "orch_packet2_launch",
            );

            let mut parent = active_parent(&launch_orchestrator);
            parent.bind_active_session_handle("orch_packet2_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let metadata = HostToolRuntimeDispatchMetadataV1 {
                caller_participant_id: "orch_packet2_successor".to_string(),
                ..sample_runtime_metadata()
            };

            let fork = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::ForkWorldWorker,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_retained".to_string(),
                }),
            )
            .expect("resolve fork follow-up through authoritative successor");
            assert_eq!(fork.mode, WorldDispatchModeV1::Retained);
            assert_eq!(
                fork.target_participant_id.as_deref(),
                Some("worker_packet2_retained")
            );

            let fork_request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::ForkWorldWorker,
                HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_retained".to_string(),
                }),
                crate::execution::agent_runtime::dispatch_contract::WorldDispatchPayloadV1::WorkerFork(
                    WorkerForkPayloadV1 {
                        prompt: "Fork a retained worker to isolate the replay trace.".to_string(),
                        fork_reason: None,
                        fork_strategy: None,
                    },
                ),
            )
            .expect("translate successor-authorized fork request");
            let fork_validated = fork_request.validate().expect("validate fork request");
            assert_eq!(fork_validated.mode, WorldDispatchModeV1::Retained);
            assert_eq!(
                fork_validated.target_participant_id.as_deref(),
                Some("worker_packet2_retained")
            );

            let cancel = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::CancelWorldWork,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_retained".to_string(),
                }),
            )
            .expect("resolve cancel follow-up through authoritative successor");
            assert_eq!(cancel.mode, WorldDispatchModeV1::Retained);
            assert_eq!(
                cancel.target_participant_id.as_deref(),
                Some("worker_packet2_retained")
            );

            let cancel_request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::CancelWorldWork,
                HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_retained".to_string(),
                }),
                crate::execution::agent_runtime::dispatch_contract::WorldDispatchPayloadV1::WorkerCancel(
                    WorkerCancelPayloadV1 {
                        reason: Some("host_requested_stop".to_string()),
                        graceful: Some(true),
                    },
                ),
            )
            .expect("translate successor-authorized cancel request");
            let cancel_validated = cancel_request.validate().expect("validate cancel request");
            assert_eq!(cancel_validated.mode, WorldDispatchModeV1::Retained);
            assert_eq!(
                cancel_validated.target_participant_id.as_deref(),
                Some("worker_packet2_retained")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_accepts_successor_authority_for_retained_continue_inspect_and_stop(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_packet2", "orch_packet2_launch");
            let mut successor =
                live_orchestrator("codex", "sess_packet2", "orch_packet2_successor");
            successor.handle.resumed_from_participant_id = Some("orch_packet2_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_packet2_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let member = live_member(
                "codex_world",
                "sess_packet2",
                "worker_packet2_retained",
                "orch_packet2_launch",
            );

            let mut parent = active_parent(&launch_orchestrator);
            parent.bind_active_session_handle("orch_packet2_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let metadata = HostToolRuntimeDispatchMetadataV1 {
                caller_participant_id: "orch_packet2_successor".to_string(),
                ..sample_runtime_metadata()
            };

            for tool_name in [
                HostToolNameV1::ContinueWorldWorker,
                HostToolNameV1::InspectWorldWorker,
                HostToolNameV1::StopWorldWorker,
            ] {
                let resolved = resolve_follow_up_dispatch_authority_v1(
                    store,
                    &metadata,
                    tool_name,
                    &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                        participant_id: "worker_packet2_retained".to_string(),
                    }),
                )
                .expect("successor lineage should authorize retained follow-up");

                assert_eq!(resolved.mode, WorldDispatchModeV1::Retained);
                assert_eq!(resolved.target_backend_id, "cli:codex_world");
                assert_eq!(
                    resolved.target_participant_id.as_deref(),
                    Some("worker_packet2_retained")
                );
            }
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_rejects_retained_worker_outside_authoritative_lineage(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_packet2", "orch_packet2_launch");
            let mut successor =
                live_orchestrator("codex", "sess_packet2", "orch_packet2_successor");
            successor.handle.resumed_from_participant_id = Some("orch_packet2_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_packet2_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let unrelated_orchestrator =
                detached_orchestrator("codex", "sess_packet2", "orch_packet2_unrelated");
            let unrelated_member = live_member(
                "codex_world",
                "sess_packet2",
                "worker_packet2_unrelated",
                "orch_packet2_unrelated",
            );

            let mut parent = active_parent(&launch_orchestrator);
            parent.bind_active_session_handle("orch_packet2_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store
                .persist_participant(&unrelated_orchestrator)
                .expect("persist unrelated orchestrator");
            store
                .persist_participant(&unrelated_member)
                .expect("persist unrelated member");

            let metadata = HostToolRuntimeDispatchMetadataV1 {
                caller_participant_id: "orch_packet2_successor".to_string(),
                ..sample_runtime_metadata()
            };

            let err = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::InspectWorldWorker,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_unrelated".to_string(),
                }),
            )
            .expect_err("linked-only follow-up must reject unrelated retained lineage");

            assert_eq!(
                err.to_string(),
                "stale_linkage: orchestration session sess_packet2 retained worker worker_packet2_unrelated is not linked to authoritative orchestrator orch_packet2_successor"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_allows_stop_for_non_authoritative_live_worker(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_packet2", "orch_packet2");
            let mut member = live_member(
                "codex_world",
                "sess_packet2",
                "worker_packet2_detached",
                "orch_packet2",
            );
            member.release_runtime_ownership();

            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let metadata = sample_runtime_metadata();
            let resolved = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::StopWorldWorker,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_detached".to_string(),
                }),
            )
            .expect("resolve retained stop authority for non-authoritative-live worker");

            assert_eq!(resolved.mode, WorldDispatchModeV1::Retained);
            assert_eq!(resolved.target_backend_id, "cli:codex_world");
            assert_eq!(
                resolved.target_participant_id.as_deref(),
                Some("worker_packet2_detached")
            );

            let request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::StopWorldWorker,
                HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_detached".to_string(),
                }),
                WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
            )
            .expect("translate retained stop request for non-authoritative-live worker");

            let validated = request.validate().expect("validate retained stop request");
            assert_eq!(validated.mode, WorldDispatchModeV1::Retained);
            assert_eq!(
                validated.target_participant_id.as_deref(),
                Some("worker_packet2_detached")
            );
            assert_eq!(validated.target_backend_id, "cli:codex_world");
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_allows_successor_fork_after_retained_owner_exits(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_packet2", "orch_packet2_launch");
            let mut successor =
                live_orchestrator("codex", "sess_packet2", "orch_packet2_successor");
            successor.handle.resumed_from_participant_id = Some("orch_packet2_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_packet2_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let mut member = live_member(
                "codex_world",
                "sess_packet2",
                "worker_packet2_retained",
                "orch_packet2_launch",
            );
            member.internal.shell_owner_pid = 999_999_999;

            let mut parent = active_parent(&launch_orchestrator);
            parent.bind_active_session_handle("orch_packet2_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let metadata = HostToolRuntimeDispatchMetadataV1 {
                caller_participant_id: "orch_packet2_successor".to_string(),
                ..sample_runtime_metadata()
            };

            let resolved = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::ForkWorldWorker,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_retained".to_string(),
                }),
            )
            .expect("resolve successor-authorized fork authority after retained owner exits");

            assert_eq!(resolved.mode, WorldDispatchModeV1::Retained);
            assert_eq!(resolved.target_backend_id, "cli:codex_world");
            assert_eq!(
                resolved.target_participant_id.as_deref(),
                Some("worker_packet2_retained")
            );

            let request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::ForkWorldWorker,
                HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_retained".to_string(),
                }),
                WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
                    prompt: "Fork from the parked retained worker.".to_string(),
                    fork_reason: None,
                    fork_strategy: None,
                }),
            )
            .expect("translate successor-authorized fork request after retained owner exits");

            let validated = request.validate().expect("validate retained fork request");
            assert_eq!(validated.mode, WorldDispatchModeV1::Retained);
            assert_eq!(
                validated.target_participant_id.as_deref(),
                Some("worker_packet2_retained")
            );
            assert_eq!(validated.target_backend_id, "cli:codex_world");
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_allows_continue_for_parked_resumable_retained_worker(
    ) {
        with_store(|store| {
            let orchestrator =
                detached_orchestrator("codex", "sess_packet2_parked", "orch_packet2_parked");
            let parent = parked_parent(&orchestrator);
            let mut member = live_member(
                "codex_world",
                "sess_packet2_parked",
                "worker_packet2_parked",
                "orch_packet2_parked",
            );
            member.release_runtime_ownership();
            member.internal.shell_owner_pid = 999_999_999;

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let metadata = HostToolRuntimeDispatchMetadataV1 {
                orchestration_session_id: "sess_packet2_parked".to_string(),
                caller_participant_id: "orch_packet2_parked".to_string(),
                ..sample_runtime_metadata()
            };

            let resolved = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::ContinueWorldWorker,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_parked".to_string(),
                }),
            )
            .expect("parked resumable retained worker should remain continue-routable");

            assert_eq!(resolved.mode, WorldDispatchModeV1::Retained);
            assert_eq!(resolved.target_backend_id, "cli:codex_world");
            assert_eq!(
                resolved.target_participant_id.as_deref(),
                Some("worker_packet2_parked")
            );

            let request = translate_follow_up_tool_to_internal_dispatch_request_v1(
                store,
                &metadata,
                HostToolNameV1::ContinueWorldWorker,
                HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_parked".to_string(),
                }),
                WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                    prompt: "Resume the parked retained worker.".to_string(),
                    thread_id: Some("thread-packet2-parked".to_string()),
                }),
            )
            .expect("translate parked retained continue request");

            let validated = request
                .validate()
                .expect("validate retained continue request");
            assert_eq!(validated.mode, WorldDispatchModeV1::Retained);
            assert_eq!(
                validated.target_participant_id.as_deref(),
                Some("worker_packet2_parked")
            );
            assert_eq!(validated.target_backend_id, "cli:codex_world");
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_allows_continue_for_exact_retained_worker_after_owner_exit(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_packet2", "orch_packet2");
            let mut member = live_member(
                "codex_world",
                "sess_packet2",
                "worker_packet2_detached_continue",
                "orch_packet2",
            );
            member.release_runtime_ownership();
            member.internal.shell_owner_pid = 999_999_999;

            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let metadata = sample_runtime_metadata();
            let resolved = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::ContinueWorldWorker,
                &HostToolFollowUpHandleV1::RetainedWorker(super::RetainedWorkerHandleV1 {
                    participant_id: "worker_packet2_detached_continue".to_string(),
                }),
            )
            .expect(
                "exact retained continue should not depend on owner PID liveness or live authority",
            );

            assert_eq!(resolved.mode, WorldDispatchModeV1::Retained);
            assert_eq!(resolved.target_backend_id, "cli:codex_world");
            assert_eq!(
                resolved.target_participant_id.as_deref(),
                Some("worker_packet2_detached_continue")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn dispatch_contract_adapter_follow_up_resolution_rejects_active_handle_for_retained_only_tool()
    {
        with_store(|store| {
            let metadata = sample_runtime_metadata();
            let err = resolve_follow_up_dispatch_authority_v1(
                store,
                &metadata,
                HostToolNameV1::StopWorldWorker,
                &HostToolFollowUpHandleV1::ActiveTask(super::ActiveTaskHandleV1 {
                    task_run_id: "task-run-packet2".to_string(),
                }),
            )
            .expect_err("retained-only tool must reject task handle");
            assert_eq!(
                err.to_string(),
                "invalid_follow_up_handle: retained-worker follow-up requires exact participant_id and does not accept task_run_id"
            );
        });
    }

    fn sample_runtime_metadata() -> HostToolRuntimeDispatchMetadataV1 {
        HostToolRuntimeDispatchMetadataV1 {
            request_id: "req-packet2-run".to_string(),
            idempotency_key: "idem-packet2-run".to_string(),
            orchestration_session_id: "sess_packet2".to_string(),
            caller_participant_id: "orch_packet2".to_string(),
        }
    }

    fn sample_world_binding() -> HostToolRuntimeWorldBindingV1 {
        HostToolRuntimeWorldBindingV1 {
            world_id: "world-17".to_string(),
            world_generation: 2,
        }
    }

    fn descriptor(agent_id: &str, scope: AgentExecutionScope) -> RuntimeSelectionDescriptor {
        RuntimeSelectionDescriptor {
            agent_id: agent_id.to_string(),
            backend_id: format!("cli:{agent_id}"),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: "substrate.agent.session".to_string(),
            execution_scope: scope,
            binary_path: PathBuf::from("/usr/bin/codex"),
        }
    }

    fn set_live(participant: &mut AgentRuntimeParticipantRecord) {
        participant
            .transition_state(crate::execution::agent_runtime::AgentRuntimeSessionState::Ready);
        participant.mark_runtime_ownership_retained();
        participant.set_uaa_session_id("uaa_session");
    }

    fn live_orchestrator(
        agent_id: &str,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> AgentRuntimeParticipantRecord {
        let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor(agent_id, AgentExecutionScope::Host),
            orchestration_session_id.to_string(),
            participant_id.to_string(),
            format!("lease_{participant_id}"),
        )
        .expect("orchestrator participant");
        set_live(&mut participant);
        participant
    }

    fn detached_orchestrator(
        agent_id: &str,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> AgentRuntimeParticipantRecord {
        let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor(agent_id, AgentExecutionScope::Host),
            orchestration_session_id.to_string(),
            participant_id.to_string(),
            format!("lease_{participant_id}"),
        )
        .expect("orchestrator participant");
        participant.transition_state(AgentRuntimeSessionState::Ready);
        participant.set_uaa_session_id("uaa_session");
        participant.mark_client_detached("owner detached cleanly");
        participant
    }

    fn live_member(
        agent_id: &str,
        orchestration_session_id: &str,
        participant_id: &str,
        orchestrator_participant_id: &str,
    ) -> AgentRuntimeParticipantRecord {
        let mut participant = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor(agent_id, AgentExecutionScope::World),
            orchestration_session_id.to_string(),
            participant_id.to_string(),
            orchestrator_participant_id.to_string(),
            None,
            Some(
                crate::execution::agent_runtime::AgentRuntimeParticipantWorldBinding {
                    world_id: "world-17".to_string(),
                    world_generation: 2,
                },
            ),
            format!("lease_{participant_id}"),
        )
        .expect("member participant");
        set_live(&mut participant);
        participant
    }

    fn active_parent(
        participant: &AgentRuntimeParticipantRecord,
    ) -> crate::execution::agent_runtime::OrchestrationSessionRecord {
        let mut parent = crate::execution::agent_runtime::OrchestrationSessionRecord::new(
            participant.handle.orchestration_session_id.clone(),
            "trace_session".to_string(),
            "/workspace".to_string(),
            participant,
            HostAttachContract::from_manifest_for_test(participant),
        );
        parent.transition_state(OrchestrationSessionState::Active);
        parent.world_id = Some("world-17".to_string());
        parent.world_generation = Some(2);
        parent.bind_active_session_handle(participant.handle.participant_id.clone());
        parent
    }

    fn parked_parent(
        participant: &AgentRuntimeParticipantRecord,
    ) -> crate::execution::agent_runtime::OrchestrationSessionRecord {
        let mut parent = active_parent(participant);
        parent.mark_parked_resumable("owner detached cleanly");
        parent
    }

    fn with_store(test: impl FnOnce(&AgentRuntimeStateStore)) {
        let temp = TempDir::new().expect("tempdir");
        std::env::set_var("SUBSTRATE_HOME", temp.path());
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
        std::env::remove_var("SUBSTRATE_HOME");
    }
}
