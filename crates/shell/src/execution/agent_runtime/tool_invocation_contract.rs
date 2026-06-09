#![allow(dead_code)]

use anyhow::bail;

use super::dispatch_contract::{WorldDispatchActionV1, WorldDispatchModeV1};

/// Frozen adapter-visible contract for host-owned tool invocation above the
/// already-landed internal `WorldDispatchRequestV1` transport.
///
/// This module is intentionally bounded to vocabulary and exact-handle
/// semantics only. It does not register tools in any runtime family, translate
/// runtime-owned fields into internal dispatch requests, or widen the transport.
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        host_tool_contract, host_tool_contracts_v1, HostToolFollowUpHandleRequirementV1,
        HostToolFollowUpHandleV1, HostToolModelArgumentFamilyV1, HostToolNameV1,
    };
    use crate::execution::agent_runtime::{WorldDispatchActionV1, WorldDispatchModeV1};

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
}
