#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::collections::BTreeMap;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::Result;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use substrate_broker::Policy;
#[cfg(target_os = "linux")]
use uuid::Uuid;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::execution::agent_inventory::{load_effective_agent_inventory, AgentInventoryEntryV1};
#[cfg(any(target_os = "linux", test))]
use crate::execution::agent_runtime::control::world_task_terminal_state_from_exit_code;
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::dispatch_contract::{
    ContinueWorldWorkerEventClassV1, ContinueWorldWorkerEventV1, ContinueWorldWorkerOutcomeV1,
    WorkerContinuePayloadV1,
};
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::mapping::AgentRuntimeBackendKind;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::execution::agent_runtime::validator::materialize_runtime_descriptor;
#[cfg(any(target_os = "linux", test))]
use crate::execution::agent_runtime::WorldTaskTerminalStateV1;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::execution::agent_runtime::{
    resolve_inventory_contract_for_exact_backend, AttachLaunchKnobs, AttachModePreference,
    DispatchBaselineKind, DispatchCallerKind, DispatchCapabilityOverrideSet,
    DispatchRequestEnvelope, HostExecutionClientStart, ResolvedLaunchContract,
};
use crate::execution::agent_runtime::{
    AgentRuntimeParticipantRecord, AgentRuntimeStateStore, OrchestrationSessionRecord,
    ValidatedWorldDispatchRequestV1, WorldDispatchActionV1, WorldDispatchOutcomeV1,
    WorldDispatchRequestV1,
};
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::{
    RunWorldTaskOutcomeV1, SpawnWorldWorkerOutcomeV1, TaskPayloadV1, WorkerSpawnPayloadV1,
    WorldDispatchModeV1, WorldDispatchPayloadV1,
};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::execution::config_model::{
    self, AgentExecutionScope, CliConfigOverrides, SubstrateConfig,
};
#[cfg(target_os = "linux")]
use crate::execution::routing::{
    build_agent_client_and_member_dispatch_request_for_cwd,
    build_agent_client_and_pending_diff_request, MemberDispatchTransportRequest,
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct PreparedOrchestratorWorldDispatch {
    pub request: ValidatedWorldDispatchRequestV1,
    pub session: OrchestrationSessionRecord,
    pub caller_participant: AgentRuntimeParticipantRecord,
    pub target_participant: Option<AgentRuntimeParticipantRecord>,
}

#[allow(dead_code)]
pub(crate) fn prepare_orchestrator_world_dispatch(
    store: &AgentRuntimeStateStore,
    request: WorldDispatchRequestV1,
) -> Result<PreparedOrchestratorWorldDispatch> {
    let request = request.validate()?;
    let (session, caller_participant, target_participant) = match request.action {
        WorldDispatchActionV1::ContinueWorldWorker => {
            let resolved = store.resolve_internal_continue_world_dispatch_target(
                &request.orchestration_session_id,
                &request.caller_participant_id,
                request
                    .target_participant_id
                    .as_deref()
                    .expect("validated continue request must include target_participant_id"),
                &request.target_backend_id,
            )?;
            (
                resolved.session,
                resolved.caller_participant,
                Some(resolved.target_participant),
            )
        }
        _ => {
            let authority = store.resolve_internal_world_dispatch_caller(
                &request.orchestration_session_id,
                &request.caller_participant_id,
            )?;
            (authority.session, authority.caller_participant, None)
        }
    };

    Ok(PreparedOrchestratorWorldDispatch {
        request,
        session,
        caller_participant,
        target_participant,
    })
}

#[allow(dead_code)]
pub(crate) async fn dispatch_orchestrator_world_request(
    store: &AgentRuntimeStateStore,
    request: WorldDispatchRequestV1,
) -> Result<WorldDispatchOutcomeV1> {
    let prepared = prepare_orchestrator_world_dispatch(store, request)?;
    dispatch_prepared_orchestrator_world_request(prepared).await
}

#[allow(dead_code)]
pub(crate) async fn dispatch_prepared_orchestrator_world_request(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    match prepared.request.action {
        WorldDispatchActionV1::RunWorldTask => run_world_task(prepared).await,
        WorldDispatchActionV1::SpawnWorldWorker => spawn_world_worker(prepared).await,
        WorldDispatchActionV1::ContinueWorldWorker => continue_world_worker(prepared).await,
    }
}

#[cfg(not(target_os = "linux"))]
async fn run_world_task(
    _prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    anyhow::bail!(
        "unsupported_platform_or_posture: run_world_task world dispatch bootstrap is supported only on linux in v1"
    );
}

#[cfg(not(target_os = "linux"))]
async fn spawn_world_worker(
    _prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    anyhow::bail!(
        "unsupported_platform_or_posture: spawn_world_worker world dispatch bootstrap is supported only on linux in v1"
    );
}

#[cfg(not(target_os = "linux"))]
async fn continue_world_worker(
    _prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    anyhow::bail!(
        "unsupported_platform_or_posture: continue_world_worker exact-target validation is available in v1, but retained-worker routing is supported only on linux in v1"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct InternalDispatchContext {
    effective_config: SubstrateConfig,
    base_policy: Policy,
    inventory: BTreeMap<String, AgentInventoryEntryV1>,
}

#[cfg(target_os = "linux")]
struct RunWorldTaskStreamResult {
    exit_code: i32,
    saw_registered_event: bool,
}

#[cfg(target_os = "linux")]
struct ContinueWorldWorkerStreamResult {
    exit_code: i32,
    surfaced_thread_id: Option<String>,
    surfaced_worker_event: Option<ContinueWorldWorkerEventV1>,
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct SpawnWorldWorkerReceipt {
    participant_id: String,
    orchestrator_participant_id: String,
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    world_id: String,
    world_generation: u64,
    launch_span_id: String,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) struct PreparedSpawnWorldWorkerBootstrap {
    pub request: ValidatedWorldDispatchRequestV1,
    pub descriptor: crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor,
}

#[cfg(target_os = "linux")]
async fn run_world_task(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    validate_authoritative_world_binding(&prepared.session, &prepared.request)?;

    let workspace_root = PathBuf::from(&prepared.session.workspace_root);
    let context = resolve_internal_dispatch_context(&workspace_root)?;
    let resolved = resolve_world_dispatch_contract(
        &workspace_root,
        &context,
        &prepared.request,
        "run_world_task",
    )?;
    let descriptor = materialize_runtime_descriptor(&resolved).map_err(|err| {
        anyhow::anyhow!(
            "runtime_start_failed: selected runtime '{}' is not runtime-realizable: {}",
            resolved.agent_id,
            err.reason
        )
    })?;
    let transport_request = build_run_world_task_transport_request(&prepared.request, &descriptor)?;
    let stream_result = execute_run_world_task_stream(&workspace_root, &transport_request).await?;
    let state = world_task_terminal_state_from_exit_code(stream_result.exit_code);
    let summary = summarize_run_world_task_result(
        &prepared.request.target_backend_id,
        stream_result.exit_code,
        stream_result.saw_registered_event,
    );

    Ok(WorldDispatchOutcomeV1::RunWorldTask(
        RunWorldTaskOutcomeV1 {
            request_id: prepared.request.request_id,
            orchestration_session_id: prepared.request.orchestration_session_id,
            action: WorldDispatchActionV1::RunWorldTask,
            mode: WorldDispatchModeV1::Ephemeral,
            state,
            summary,
        },
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) fn prepare_spawn_world_worker_bootstrap(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<PreparedSpawnWorldWorkerBootstrap> {
    validate_authoritative_world_binding(&prepared.session, &prepared.request)?;

    let workspace_root = PathBuf::from(&prepared.session.workspace_root);
    let context = resolve_internal_dispatch_context(&workspace_root)?;
    let resolved = resolve_world_dispatch_contract(
        &workspace_root,
        &context,
        &prepared.request,
        "spawn_world_worker",
    )?;
    let descriptor = materialize_runtime_descriptor(&resolved).map_err(|err| {
        anyhow::anyhow!(
            "runtime_start_failed: selected runtime '{}' is not runtime-realizable: {}",
            resolved.agent_id,
            err.reason
        )
    })?;

    Ok(PreparedSpawnWorldWorkerBootstrap {
        request: prepared.request,
        descriptor,
    })
}

#[cfg(target_os = "linux")]
async fn spawn_world_worker(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    let prepared = prepare_spawn_world_worker_bootstrap(prepared)?;
    let workspace_root = std::env::current_dir()
        .context("failed to resolve cwd for direct spawn_world_worker bootstrap")?;
    let transport_request =
        build_spawn_world_worker_transport_request(&prepared.request, &prepared.descriptor)?;
    let receipt =
        execute_spawn_world_worker_stream(&workspace_root, &transport_request, &prepared.request)
            .await?;
    let summary = summarize_spawn_world_worker_result(&receipt);

    Ok(WorldDispatchOutcomeV1::SpawnWorldWorker(
        SpawnWorldWorkerOutcomeV1 {
            request_id: prepared.request.request_id,
            orchestration_session_id: prepared.request.orchestration_session_id,
            action: WorldDispatchActionV1::SpawnWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            participant_id: receipt.participant_id,
            orchestrator_participant_id: receipt.orchestrator_participant_id,
            parent_participant_id: receipt.parent_participant_id,
            resumed_from_participant_id: receipt.resumed_from_participant_id,
            target_backend_id: receipt.backend_id,
            world_id: receipt.world_id,
            world_generation: receipt.world_generation,
            launch_span_id: receipt.launch_span_id,
            summary,
        },
    ))
}

#[cfg(target_os = "linux")]
async fn continue_world_worker(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    validate_authoritative_world_binding(&prepared.session, &prepared.request)?;

    let submit_request = build_continue_world_worker_submit_request(&prepared)?;
    let stream_result = execute_continue_world_worker_stream(&submit_request).await?;
    let summary = summarize_continue_world_worker_result(&submit_request, stream_result.exit_code);

    Ok(WorldDispatchOutcomeV1::ContinueWorldWorker(
        ContinueWorldWorkerOutcomeV1 {
            request_id: prepared.request.request_id,
            orchestration_session_id: submit_request.orchestration_session_id.clone(),
            action: WorldDispatchActionV1::ContinueWorldWorker,
            mode: prepared.request.mode,
            orchestrator_participant_id: submit_request.orchestrator_participant_id.clone(),
            target_participant_id: submit_request.participant_id.clone(),
            target_backend_id: submit_request.backend_id.clone(),
            world_id: submit_request.world_id.clone(),
            world_generation: submit_request.world_generation,
            thread_id: stream_result.surfaced_thread_id,
            worker_event: stream_result.surfaced_worker_event,
            summary,
        },
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn resolve_internal_dispatch_context(workspace_root: &Path) -> Result<InternalDispatchContext> {
    let effective_config =
        config_model::resolve_effective_config(workspace_root, &CliConfigOverrides::default())?;
    let (base_policy, _) =
        substrate_broker::resolve_effective_policy_with_explain(workspace_root, false)
            .map_err(|err| config_model::user_error(err.to_string()))?;
    let inventory = load_effective_agent_inventory(workspace_root, &base_policy)?;

    Ok(InternalDispatchContext {
        effective_config,
        base_policy,
        inventory,
    })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn resolve_world_dispatch_contract(
    workspace_root: &Path,
    context: &InternalDispatchContext,
    request: &ValidatedWorldDispatchRequestV1,
    action_label: &str,
) -> Result<ResolvedLaunchContract> {
    let world_envelope = DispatchRequestEnvelope {
        caller_kind: DispatchCallerKind::OrchestratorMemberStart,
        baseline_kind: DispatchBaselineKind::InventoryLaunch,
        backend_id: Some(request.target_backend_id.clone()),
        orchestration_session_id: Some(request.orchestration_session_id.clone()),
        requested_execution_scope_override: Some(AgentExecutionScope::World),
        capability_overrides: DispatchCapabilityOverrideSet::default(),
        attach_launch_knobs: AttachLaunchKnobs {
            requested_execution_scope: AgentExecutionScope::World,
            host_execution_client_start: HostExecutionClientStart::Defer,
            attach_mode_preference: AttachModePreference::FreshAllowed,
        },
        has_prompt_payload: true,
    };

    match resolve_inventory_contract_for_exact_backend(
        workspace_root,
        &context.effective_config,
        &context.inventory,
        &context.base_policy,
        &world_envelope,
        AgentExecutionScope::World,
    )
    .map_err(|err| anyhow::anyhow!("{err}"))?
    {
        Some(contract) => Ok(contract),
        None => {
            let host_envelope = DispatchRequestEnvelope {
                requested_execution_scope_override: Some(AgentExecutionScope::Host),
                ..world_envelope
            };
            let permissive_policy = permissive_inventory_selection_policy(&context.inventory);
            let host_match = resolve_inventory_contract_for_exact_backend(
                workspace_root,
                &context.effective_config,
                &context.inventory,
                &permissive_policy,
                &host_envelope,
                AgentExecutionScope::Host,
            )
            .map_err(|err| anyhow::anyhow!("{err}"))?;

            if host_match.is_some() {
                anyhow::bail!(
                    "unsupported_platform_or_posture: backend '{}' resolves only to a host-scoped runtime; {} requires an exact world-scoped backend",
                    request.target_backend_id,
                    action_label,
                );
            }

            anyhow::bail!(
                "unknown_backend: baseline truth rejected field 'target_backend_id': no exact world-scoped backend match found for '{}'",
                request.target_backend_id
            );
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
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

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn validate_authoritative_world_binding(
    session: &OrchestrationSessionRecord,
    request: &ValidatedWorldDispatchRequestV1,
) -> Result<()> {
    let Some(authoritative_world_id) = session.world_id.as_deref() else {
        anyhow::bail!(
            "missing_world_binding: orchestration session {} has no authoritative world binding",
            session.orchestration_session_id
        );
    };
    let Some(authoritative_world_generation) = session.world_generation else {
        anyhow::bail!(
            "missing_world_binding: orchestration session {} has no authoritative world binding",
            session.orchestration_session_id
        );
    };

    if authoritative_world_id != request.world_id {
        anyhow::bail!(
            "world_binding_mismatch: orchestration session {} authoritative world_id is {} not {}",
            session.orchestration_session_id,
            authoritative_world_id,
            request.world_id
        );
    }
    if authoritative_world_generation != request.world_generation {
        anyhow::bail!(
            "world_binding_mismatch: orchestration session {} authoritative world_generation is {} not {}",
            session.orchestration_session_id,
            authoritative_world_generation,
            request.world_generation
        );
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn build_run_world_task_transport_request(
    request: &ValidatedWorldDispatchRequestV1,
    descriptor: &crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor,
) -> Result<MemberDispatchTransportRequest> {
    let WorldDispatchPayloadV1::Task(TaskPayloadV1 { prompt }) = &request.payload else {
        anyhow::bail!(
            "invalid_dispatch_payload: action run_world_task requires matching typed payload"
        );
    };

    Ok(MemberDispatchTransportRequest {
        orchestration_session_id: request.orchestration_session_id.clone(),
        participant_id: format!("awm_{}", Uuid::now_v7()),
        orchestrator_participant_id: request.caller_participant_id.clone(),
        parent_participant_id: None,
        resumed_from_participant_id: None,
        backend_id: descriptor.backend_id.clone(),
        protocol: descriptor.protocol.clone(),
        run_id: request.request_id.clone(),
        world_id: request.world_id.clone(),
        world_generation: request.world_generation,
        initial_prompt: Some(prompt.clone()),
        backend_kind: member_runtime_backend_kind(descriptor.backend_kind),
        binary_path: descriptor.binary_path.display().to_string(),
    })
}

#[cfg(target_os = "linux")]
fn build_spawn_world_worker_transport_request(
    request: &ValidatedWorldDispatchRequestV1,
    descriptor: &crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor,
) -> Result<MemberDispatchTransportRequest> {
    let WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 { prompt }) = &request.payload
    else {
        anyhow::bail!(
            "invalid_dispatch_payload: action spawn_world_worker requires matching typed payload"
        );
    };

    Ok(MemberDispatchTransportRequest {
        orchestration_session_id: request.orchestration_session_id.clone(),
        participant_id: format!("ash_{}", Uuid::now_v7()),
        orchestrator_participant_id: request.caller_participant_id.clone(),
        parent_participant_id: None,
        resumed_from_participant_id: None,
        backend_id: descriptor.backend_id.clone(),
        protocol: descriptor.protocol.clone(),
        run_id: request.request_id.clone(),
        world_id: request.world_id.clone(),
        world_generation: request.world_generation,
        initial_prompt: Some(prompt.clone()),
        backend_kind: member_runtime_backend_kind(descriptor.backend_kind),
        binary_path: descriptor.binary_path.display().to_string(),
    })
}

#[cfg(target_os = "linux")]
fn build_continue_world_worker_submit_request(
    prepared: &PreparedOrchestratorWorldDispatch,
) -> Result<transport_api_types::MemberTurnSubmitRequestV1> {
    let target = prepared.target_participant.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "invalid_dispatch_target: continue_world_worker requires an exact retained target participant"
        )
    })?;
    let WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 { prompt, .. }) =
        &prepared.request.payload
    else {
        anyhow::bail!(
            "invalid_dispatch_payload: action continue_world_worker requires matching typed payload"
        );
    };
    let orchestrator_participant_id = target
        .handle
        .orchestrator_participant_id
        .clone()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "invalid_dispatch_target: retained worker {} omitted orchestrator_participant_id",
                target.handle.participant_id
            )
        })?;
    let world_id = target.handle.world_id.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "invalid_dispatch_target: retained worker {} omitted world_id",
            target.handle.participant_id
        )
    })?;
    let world_generation = target.handle.world_generation.ok_or_else(|| {
        anyhow::anyhow!(
            "invalid_dispatch_target: retained worker {} omitted world_generation",
            target.handle.participant_id
        )
    })?;

    Ok(transport_api_types::MemberTurnSubmitRequestV1 {
        schema_version: 1,
        orchestration_session_id: target.handle.orchestration_session_id.clone(),
        participant_id: target.handle.participant_id.clone(),
        orchestrator_participant_id,
        backend_id: target.handle.backend_id.clone(),
        run_id: prepared.request.request_id.clone(),
        world_id,
        world_generation,
        prompt: prompt.clone(),
    })
}

#[cfg(target_os = "linux")]
fn member_runtime_backend_kind(
    backend_kind: AgentRuntimeBackendKind,
) -> transport_api_types::MemberRuntimeBackendKindV1 {
    match backend_kind {
        AgentRuntimeBackendKind::Codex => transport_api_types::MemberRuntimeBackendKindV1::Codex,
        AgentRuntimeBackendKind::ClaudeCode => {
            transport_api_types::MemberRuntimeBackendKindV1::ClaudeCode
        }
    }
}

#[cfg(target_os = "linux")]
async fn execute_run_world_task_stream(
    workspace_root: &Path,
    request: &MemberDispatchTransportRequest,
) -> Result<RunWorldTaskStreamResult> {
    use http_body_util::BodyExt as _;
    use substrate_common::agent_events::AgentEventKind;
    use transport_api_types::{ExecuteCancelRequestV1, ExecuteStreamFrame};

    let (client, execute_request, _agent_id) =
        build_agent_client_and_member_dispatch_request_for_cwd(request, workspace_root)
            .context("failed to build member dispatch execute request for run_world_task")?;
    let response = client
        .execute_stream(execute_request)
        .await
        .context("failed to launch run_world_task over world member dispatch")?;

    let mut body = std::pin::pin!(response.into_body());
    let mut buffer = Vec::new();
    let mut active_span_id = None::<String>;
    let mut saw_registered_event = false;
    let mut exit_code = None::<i32>;

    while let Some(frame) = body.as_mut().frame().await {
        let frame = frame.map_err(|err| anyhow::anyhow!("stream frame error: {err}"))?;
        let Some(data) = frame.data_ref() else {
            continue;
        };
        buffer.extend_from_slice(data);

        while let Some(pos) = buffer.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            if line.len() <= 1 {
                continue;
            }
            let payload = &line[..line.len() - 1];
            if payload.is_empty() {
                continue;
            }
            let frame: ExecuteStreamFrame = serde_json::from_slice(payload).with_context(|| {
                format!(
                    "invalid run_world_task stream frame: {}",
                    String::from_utf8_lossy(payload)
                )
            })?;

            match frame {
                ExecuteStreamFrame::Start { span_id } => {
                    active_span_id = Some(span_id);
                }
                ExecuteStreamFrame::Event { event } => {
                    if event.kind == AgentEventKind::Registered {
                        saw_registered_event = true;
                    }
                }
                ExecuteStreamFrame::Exit { exit, .. } => {
                    exit_code = Some(exit);
                    break;
                }
                ExecuteStreamFrame::Error { message } => {
                    if saw_registered_event {
                        if let Some(span_id) = active_span_id.as_ref() {
                            let _ = client
                                .cancel_execute(ExecuteCancelRequestV1 {
                                    span_id: span_id.clone(),
                                    sig: "INT".to_string(),
                                })
                                .await;
                        }
                    }
                    anyhow::bail!(message);
                }
                ExecuteStreamFrame::Stdout { .. } | ExecuteStreamFrame::Stderr { .. } => {}
            }
        }

        if exit_code.is_some() {
            break;
        }
    }

    let exit_code = exit_code.ok_or_else(|| {
        anyhow::anyhow!("run_world_task stream ended without a terminal exit frame")
    })?;

    Ok(RunWorldTaskStreamResult {
        exit_code,
        saw_registered_event,
    })
}

#[cfg(target_os = "linux")]
async fn execute_spawn_world_worker_stream(
    workspace_root: &Path,
    request: &MemberDispatchTransportRequest,
    dispatch_request: &ValidatedWorldDispatchRequestV1,
) -> Result<SpawnWorldWorkerReceipt> {
    use http_body_util::BodyExt as _;
    use substrate_common::agent_events::AgentEventKind;
    use transport_api_types::ExecuteStreamFrame;

    let (client, execute_request, _agent_id) =
        build_agent_client_and_member_dispatch_request_for_cwd(request, workspace_root)
            .context("failed to build member dispatch execute request for spawn_world_worker")?;
    let response = client
        .execute_stream(execute_request)
        .await
        .context("failed to launch spawn_world_worker over world member dispatch")?;

    let mut body = std::pin::pin!(response.into_body());
    let mut buffer = Vec::new();
    let mut launch_span_id = None::<String>;

    while let Some(frame) = body.as_mut().frame().await {
        let frame = frame.map_err(|err| anyhow::anyhow!("stream frame error: {err}"))?;
        let Some(data) = frame.data_ref() else {
            continue;
        };
        buffer.extend_from_slice(data);

        while let Some(pos) = buffer.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            if line.len() <= 1 {
                continue;
            }
            let payload = &line[..line.len() - 1];
            if payload.is_empty() {
                continue;
            }
            let frame: ExecuteStreamFrame = serde_json::from_slice(payload).with_context(|| {
                format!(
                    "invalid spawn_world_worker stream frame: {}",
                    String::from_utf8_lossy(payload)
                )
            })?;

            match frame {
                ExecuteStreamFrame::Start { span_id } => {
                    launch_span_id = Some(span_id);
                }
                ExecuteStreamFrame::Event { event } if event.kind == AgentEventKind::Registered => {
                    let launch_span_id = launch_span_id.clone().ok_or_else(|| {
                        anyhow::anyhow!(
                            "spawn_world_worker registered without a streamed execute span_id"
                        )
                    })?;
                    return receipt_from_registered_event(
                        event,
                        request,
                        dispatch_request,
                        launch_span_id,
                    );
                }
                ExecuteStreamFrame::Event { .. }
                | ExecuteStreamFrame::Stdout { .. }
                | ExecuteStreamFrame::Stderr { .. } => {}
                ExecuteStreamFrame::Exit { exit, .. } => {
                    anyhow::bail!(
                        "retained_bootstrap_failed: spawn_world_worker exited with status {} before authoritative registration",
                        exit
                    );
                }
                ExecuteStreamFrame::Error { message } => {
                    anyhow::bail!(message);
                }
            }
        }
    }

    anyhow::bail!(
        "retained_bootstrap_failed: spawn_world_worker ended without authoritative registration"
    );
}

#[cfg(target_os = "linux")]
async fn execute_continue_world_worker_stream(
    request: &transport_api_types::MemberTurnSubmitRequestV1,
) -> Result<ContinueWorldWorkerStreamResult> {
    use http_body_util::BodyExt as _;
    use transport_api_types::ExecuteStreamFrame;

    let (client, _pending_diff_request, _agent_id) = build_agent_client_and_pending_diff_request()
        .context("failed to build member turn submit client for continue_world_worker")?;
    let response = client
        .submit_member_turn_stream(request.clone())
        .await
        .context("failed to submit continue_world_worker over world member turn seam")?;

    let mut body = std::pin::pin!(response.into_body());
    let mut buffer = Vec::new();
    let mut exit_code = None::<i32>;
    let mut surfaced_thread_id = None::<String>;
    let mut surfaced_worker_event = None::<ContinueWorldWorkerEventV1>;

    while let Some(frame) = body.as_mut().frame().await {
        let frame = frame.map_err(|err| anyhow::anyhow!("stream frame error: {err}"))?;
        let Some(data) = frame.data_ref() else {
            continue;
        };
        buffer.extend_from_slice(data);

        while let Some(pos) = buffer.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            if line.len() <= 1 {
                continue;
            }
            let payload = &line[..line.len() - 1];
            if payload.is_empty() {
                continue;
            }
            let frame: ExecuteStreamFrame = serde_json::from_slice(payload).with_context(|| {
                format!(
                    "invalid continue_world_worker stream frame: {}",
                    String::from_utf8_lossy(payload)
                )
            })?;

            match frame {
                ExecuteStreamFrame::Start { .. } => {}
                ExecuteStreamFrame::Event { event } => {
                    if surfaced_thread_id.is_none() {
                        surfaced_thread_id = surfaced_thread_id_from_event(&event);
                    }
                    if let Some(classified_event) =
                        classify_continue_world_worker_event(request, &event)?
                    {
                        surfaced_worker_event = Some(classified_event);
                    }
                }
                ExecuteStreamFrame::Exit { exit, .. } => {
                    exit_code = Some(exit);
                    break;
                }
                ExecuteStreamFrame::Error { message } => {
                    anyhow::bail!(message);
                }
                ExecuteStreamFrame::Stdout { .. } | ExecuteStreamFrame::Stderr { .. } => {}
            }
        }

        if exit_code.is_some() {
            break;
        }
    }

    let exit_code = exit_code.ok_or_else(|| {
        anyhow::anyhow!("continue_world_worker stream ended without a terminal exit frame")
    })?;

    Ok(ContinueWorldWorkerStreamResult {
        exit_code,
        surfaced_thread_id,
        surfaced_worker_event,
    })
}

#[cfg(target_os = "linux")]
fn surfaced_thread_id_from_event(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<String> {
    if let Some(thread_id) = event
        .thread_id
        .as_deref()
        .map(str::trim)
        .filter(|thread_id| !thread_id.is_empty())
    {
        return Some(thread_id.to_string());
    }

    for pointer in [
        "/uaa_event/thread_id",
        "/uaa_event/session/id",
        "/uaa_event/raw_event/thread_id",
        "/uaa_event/raw_event/session/id",
    ] {
        if let Some(thread_id) = event
            .data
            .pointer(pointer)
            .and_then(serde_json::Value::as_str)
        {
            let trimmed = thread_id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn classify_continue_world_worker_event(
    request: &transport_api_types::MemberTurnSubmitRequestV1,
    event: &substrate_common::agent_events::AgentEvent,
) -> Result<Option<ContinueWorldWorkerEventV1>> {
    let Some(event_class_label) = continue_world_worker_event_class_label(event) else {
        return Ok(None);
    };

    let Some(event_class) = ContinueWorldWorkerEventClassV1::from_wire_label(event_class_label)
    else {
        if ContinueWorldWorkerEventClassV1::is_deferred_wire_label(event_class_label) {
            anyhow::bail!(
                "unsupported_worker_event_class: continue_world_worker does not yet accept worker event class {}",
                event_class_label.trim()
            );
        }
        return Ok(None);
    };

    let source_participant_id =
        continue_world_worker_identity_field(event.participant_id.as_deref())
            .unwrap_or(&request.participant_id);
    if source_participant_id != request.participant_id {
        anyhow::bail!(
            "protocol error: continue_world_worker surfaced worker event participant_id {} did not match targeted retained worker {}",
            source_participant_id,
            request.participant_id,
        );
    }

    let source_backend_id = continue_world_worker_identity_field(event.backend_id.as_deref())
        .unwrap_or(&request.backend_id);
    if source_backend_id != request.backend_id {
        anyhow::bail!(
            "protocol error: continue_world_worker surfaced worker event backend_id {} did not match targeted backend {}",
            source_backend_id,
            request.backend_id,
        );
    }

    let attention_required = event_class.attention_required_by_default()
        || continue_world_worker_attention_required(event).unwrap_or(false);

    Ok(Some(ContinueWorldWorkerEventV1 {
        event_class,
        source_participant_id: source_participant_id.to_string(),
        target_participant_id: request.orchestrator_participant_id.clone(),
        source_backend_id: source_backend_id.to_string(),
        attention_required,
        thread_id: surfaced_thread_id_from_event(event),
        stream_channel: event.channel.clone(),
        payload: continue_world_worker_event_payload(event),
    }))
}

#[cfg(target_os = "linux")]
fn continue_world_worker_event_class_label(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<&str> {
    continue_world_worker_string_field(
        &event.data,
        &[
            "/event_class",
            "/uaa_event/event_class",
            "/uaa_event/raw_event/event_class",
            "/raw_event/event_class",
        ],
    )
}

#[cfg(target_os = "linux")]
fn continue_world_worker_attention_required(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<bool> {
    continue_world_worker_bool_field(
        &event.data,
        &[
            "/attention_required",
            "/uaa_event/attention_required",
            "/uaa_event/raw_event/attention_required",
            "/raw_event/attention_required",
        ],
    )
}

#[cfg(target_os = "linux")]
fn continue_world_worker_event_payload(
    event: &substrate_common::agent_events::AgentEvent,
) -> serde_json::Value {
    for pointer in [
        "/payload",
        "/uaa_event/payload",
        "/uaa_event/raw_event/payload",
        "/raw_event/payload",
    ] {
        if let Some(payload) = event.data.pointer(pointer) {
            return payload.clone();
        }
    }

    event.data.clone()
}

#[cfg(target_os = "linux")]
fn continue_world_worker_string_field<'a>(
    data: &'a serde_json::Value,
    pointers: &[&str],
) -> Option<&'a str> {
    pointers.iter().find_map(|pointer| {
        data.pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

#[cfg(target_os = "linux")]
fn continue_world_worker_bool_field(data: &serde_json::Value, pointers: &[&str]) -> Option<bool> {
    pointers
        .iter()
        .find_map(|pointer| data.pointer(pointer).and_then(serde_json::Value::as_bool))
}

#[cfg(target_os = "linux")]
fn continue_world_worker_identity_field(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

#[cfg(target_os = "linux")]
fn receipt_from_registered_event(
    event: substrate_common::agent_events::AgentEvent,
    transport_request: &MemberDispatchTransportRequest,
    dispatch_request: &ValidatedWorldDispatchRequestV1,
    launch_span_id: String,
) -> Result<SpawnWorldWorkerReceipt> {
    let participant_id = event.participant_id.ok_or_else(|| {
        anyhow::anyhow!(
            "protocol error: spawn_world_worker registered event omitted participant_id"
        )
    })?;
    if participant_id != transport_request.participant_id {
        anyhow::bail!(
            "protocol error: spawn_world_worker registered participant_id {} did not match requested {}",
            participant_id,
            transport_request.participant_id,
        );
    }

    let backend_id = event.backend_id.ok_or_else(|| {
        anyhow::anyhow!("protocol error: spawn_world_worker registered event omitted backend_id")
    })?;
    if backend_id != dispatch_request.target_backend_id {
        anyhow::bail!(
            "protocol error: spawn_world_worker registered backend_id {} did not match requested {}",
            backend_id,
            dispatch_request.target_backend_id,
        );
    }

    let world_id = event.world_id.ok_or_else(|| {
        anyhow::anyhow!("protocol error: spawn_world_worker registered event omitted world_id")
    })?;
    if world_id != dispatch_request.world_id {
        anyhow::bail!(
            "protocol error: spawn_world_worker registered world_id {} did not match requested {}",
            world_id,
            dispatch_request.world_id,
        );
    }

    let world_generation = event.world_generation.ok_or_else(|| {
        anyhow::anyhow!(
            "protocol error: spawn_world_worker registered event omitted world_generation"
        )
    })?;
    if world_generation != dispatch_request.world_generation {
        anyhow::bail!(
            "protocol error: spawn_world_worker registered world_generation {} did not match requested {}",
            world_generation,
            dispatch_request.world_generation,
        );
    }

    if event.parent_participant_id != transport_request.parent_participant_id {
        anyhow::bail!(
            "protocol error: spawn_world_worker registered parent_participant_id {:?} did not match requested {:?}",
            event.parent_participant_id,
            transport_request.parent_participant_id,
        );
    }
    if event.resumed_from_participant_id != transport_request.resumed_from_participant_id {
        anyhow::bail!(
            "protocol error: spawn_world_worker registered resumed_from_participant_id {:?} did not match requested {:?}",
            event.resumed_from_participant_id,
            transport_request.resumed_from_participant_id,
        );
    }

    Ok(SpawnWorldWorkerReceipt {
        participant_id,
        orchestrator_participant_id: transport_request.orchestrator_participant_id.clone(),
        parent_participant_id: event.parent_participant_id,
        resumed_from_participant_id: event.resumed_from_participant_id,
        backend_id,
        world_id,
        world_generation,
        launch_span_id,
    })
}

#[cfg(any(target_os = "linux", test))]
fn summarize_run_world_task_result(
    backend_id: &str,
    exit_code: i32,
    saw_registered_event: bool,
) -> String {
    let continuity_suffix = if saw_registered_event {
        "; backend surfaced continuity metadata but the dispatch returned terminally without retained shell state"
    } else {
        ""
    };

    match world_task_terminal_state_from_exit_code(exit_code) {
        WorldTaskTerminalStateV1::Completed => {
            format!("run_world_task completed on backend {backend_id}{continuity_suffix}")
        }
        WorldTaskTerminalStateV1::Cancelled => {
            format!(
                "run_world_task was cancelled on backend {backend_id} (exit status 130){continuity_suffix}"
            )
        }
        WorldTaskTerminalStateV1::Failed => {
            format!(
                "run_world_task failed on backend {backend_id} with exit status {exit_code}{continuity_suffix}"
            )
        }
        WorldTaskTerminalStateV1::NeedsRetainedFollowup => {
            format!(
                "run_world_task on backend {backend_id} requires retained follow-up before it can complete"
            )
        }
    }
}

#[cfg(target_os = "linux")]
fn summarize_spawn_world_worker_result(receipt: &SpawnWorldWorkerReceipt) -> String {
    format!(
        "spawn_world_worker launched retained worker {} on backend {}; launch receipt is authoritative but ongoing steering remains out of scope for this packet",
        receipt.participant_id, receipt.backend_id
    )
}

#[cfg(target_os = "linux")]
fn summarize_continue_world_worker_result(
    request: &transport_api_types::MemberTurnSubmitRequestV1,
    exit_code: i32,
) -> String {
    if exit_code == 0 {
        format!(
            "continue_world_worker completed on retained worker {} via the existing member-turn seam",
            request.participant_id
        )
    } else {
        format!(
            "continue_world_worker submitted retained worker {} via the existing member-turn seam, but the turn exited with status {}",
            request.participant_id, exit_code
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use crate::execution::agent_inventory::{
        AgentCapabilitiesV1, AgentCliConfigV1, AgentCliRuntimeFamily, AgentConfigKind,
        AgentConfigV1, AgentExecutionConfigV1, AgentFileV1, AgentInventoryEntryV1,
    };
    use crate::execution::agent_runtime::orchestration_session::{
        OrchestrationSessionPosture, OrchestrationSessionState,
    };
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::WorkerSpawnPayloadV1;
    use crate::execution::agent_runtime::{
        TaskPayloadV1, WorldDispatchModeV1, WorldDispatchPayloadV1,
    };
    #[cfg(target_os = "linux")]
    use crate::execution::config_model::AgentCliMode;
    #[cfg(target_os = "linux")]
    use serde_json::json;
    #[cfg(target_os = "linux")]
    use substrate_common::agent_events::AgentEventKind;
    #[cfg(target_os = "linux")]
    use tempfile::tempdir;

    fn sample_session() -> OrchestrationSessionRecord {
        OrchestrationSessionRecord {
            orchestration_session_id: "sess_dispatch".to_string(),
            shell_trace_session_id: "trace_dispatch".to_string(),
            workspace_root: "/tmp/substrate-dispatch".to_string(),
            shell_owner_pid: 1,
            state: OrchestrationSessionState::Active,
            opened_at: chrono::Utc::now(),
            last_active_at: chrono::Utc::now(),
            orchestrator_agent_id: "codex".to_string(),
            orchestrator_backend_id: "cli:codex".to_string(),
            orchestrator_protocol: "substrate.agent.session".to_string(),
            active_session_handle_id: Some("orch_dispatch".to_string()),
            latest_run_id: Some("run_dispatch".to_string()),
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            invalidation_reason: None,
            closed_at: None,
            posture: OrchestrationSessionPosture::ActiveAttached,
            posture_changed_at: chrono::Utc::now(),
            attached_participant_id: Some("orch_dispatch".to_string()),
            pending_inbox_count: 0,
            last_parked_at: None,
            last_attention_at: None,
            parked_reason: None,
            startup_prompt: None,
            host_attach_contract: None,
        }
    }

    fn sample_request() -> ValidatedWorldDispatchRequestV1 {
        WorldDispatchRequestV1 {
            request_id: Some("req_dispatch".to_string()),
            idempotency_key: Some("idem_dispatch".to_string()),
            orchestration_session_id: Some("sess_dispatch".to_string()),
            caller_participant_id: Some("orch_dispatch".to_string()),
            action: WorldDispatchActionV1::RunWorldTask,
            mode: WorldDispatchModeV1::Ephemeral,
            target_backend_id: Some("cli:codex_world".to_string()),
            target_participant_id: None,
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            payload: WorldDispatchPayloadV1::Task(TaskPayloadV1 {
                prompt: "hello world".to_string(),
            }),
        }
        .validate()
        .expect("validated request")
    }

    #[cfg(target_os = "linux")]
    fn sample_continue_request() -> ValidatedWorldDispatchRequestV1 {
        WorldDispatchRequestV1 {
            request_id: Some("req_continue".to_string()),
            idempotency_key: Some("idem_continue".to_string()),
            orchestration_session_id: Some("sess_dispatch".to_string()),
            caller_participant_id: Some("orch_dispatch".to_string()),
            action: WorldDispatchActionV1::ContinueWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some("cli:codex_world".to_string()),
            target_participant_id: Some("ash_member".to_string()),
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            payload: WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                prompt: "follow up".to_string(),
                thread_id: Some("thread-root".to_string()),
            }),
        }
        .validate()
        .expect("validated continue request")
    }

    #[cfg(target_os = "linux")]
    fn sample_orchestrator_participant() -> AgentRuntimeParticipantRecord {
        serde_json::from_value(json!({
            "participant_id": "orch_dispatch",
            "orchestration_session_id": "sess_dispatch",
            "agent_id": "codex",
            "backend_id": "cli:codex",
            "role": "orchestrator",
            "protocol": "substrate.agent.session",
            "execution": { "scope": "host" },
            "state": "running",
            "opened_at": "2026-05-01T00:00:00Z",
            "last_transition_at": "2026-05-01T00:00:00Z",
            "internal": {
                "resolved_agent_kind": "codex",
                "resolved_binary_path": "/usr/bin/codex",
                "shell_owner_pid": 42,
                "lease_token": "lease_orch",
                "cancel_supported": true,
                "control_owner_retained": true,
                "event_stream_active": true,
                "completion_observer_retained": true,
                "ownership_mode": "attached_control",
                "ownership_valid": true
            }
        }))
        .expect("orchestrator participant")
    }

    #[cfg(target_os = "linux")]
    fn sample_member_participant() -> AgentRuntimeParticipantRecord {
        serde_json::from_value(json!({
            "participant_id": "ash_member",
            "orchestration_session_id": "sess_dispatch",
            "agent_id": "codex_world",
            "backend_id": "cli:codex_world",
            "role": "member",
            "protocol": "substrate.agent.session",
            "execution": { "scope": "world" },
            "state": "running",
            "opened_at": "2026-05-01T00:00:00Z",
            "last_transition_at": "2026-05-01T00:00:00Z",
            "world_id": "world-17",
            "world_generation": 2,
            "orchestrator_participant_id": "orch_dispatch",
            "internal": {
                "resolved_agent_kind": "codex",
                "resolved_binary_path": "/usr/bin/codex",
                "shell_owner_pid": 42,
                "lease_token": "lease_member",
                "cancel_supported": true,
                "control_owner_retained": true,
                "event_stream_active": true,
                "completion_observer_retained": true,
                "ownership_mode": "member_runtime",
                "ownership_valid": true
            }
        }))
        .expect("member participant")
    }

    #[cfg(target_os = "linux")]
    fn sample_spawn_request() -> ValidatedWorldDispatchRequestV1 {
        WorldDispatchRequestV1 {
            request_id: Some("req_spawn".to_string()),
            idempotency_key: Some("idem_spawn".to_string()),
            orchestration_session_id: Some("sess_dispatch".to_string()),
            caller_participant_id: Some("orch_dispatch".to_string()),
            action: WorldDispatchActionV1::SpawnWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some("cli:codex_world".to_string()),
            target_participant_id: None,
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
                prompt: "open a retained worker".to_string(),
            }),
        }
        .validate()
        .expect("validated spawn request")
    }

    #[cfg(target_os = "linux")]
    fn sample_continue_submit_request() -> transport_api_types::MemberTurnSubmitRequestV1 {
        transport_api_types::MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: "sess_dispatch".to_string(),
            participant_id: "ash_member".to_string(),
            orchestrator_participant_id: "orch_dispatch".to_string(),
            backend_id: "cli:codex_world".to_string(),
            run_id: "req_continue".to_string(),
            world_id: "world-17".to_string(),
            world_generation: 2,
            prompt: "follow up".to_string(),
        }
    }

    #[cfg(target_os = "linux")]
    fn sample_continue_stream_event(
        data: serde_json::Value,
    ) -> substrate_common::agent_events::AgentEvent {
        substrate_common::agent_events::AgentEvent {
            ts: chrono::Utc::now(),
            kind: AgentEventKind::TaskProgress,
            data,
            agent_id: "codex_world".to_string(),
            orchestration_session_id: "sess_dispatch".to_string(),
            run_id: "req_continue".to_string(),
            parent_run_id: None,
            participant_id: Some("ash_member".to_string()),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: Some("cli:codex_world".to_string()),
            thread_id: Some("thread-direct".to_string()),
            role: Some("member".to_string()),
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            cmd_id: None,
            span_id: Some("spn_continue".to_string()),
            channel: Some("worker.reply".to_string()),
            identity_tuple: None,
            placement_posture: None,
            project: None,
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn continue_world_worker_submit_request_uses_authoritative_retained_identity() {
        let prepared = PreparedOrchestratorWorldDispatch {
            request: sample_continue_request(),
            session: sample_session(),
            caller_participant: sample_orchestrator_participant(),
            target_participant: Some(sample_member_participant()),
        };

        let submit =
            build_continue_world_worker_submit_request(&prepared).expect("continue submit request");

        assert_eq!(submit.orchestration_session_id, "sess_dispatch");
        assert_eq!(submit.participant_id, "ash_member");
        assert_eq!(submit.orchestrator_participant_id, "orch_dispatch");
        assert_eq!(submit.backend_id, "cli:codex_world");
        assert_eq!(submit.run_id, "req_continue");
        assert_eq!(submit.world_id, "world-17");
        assert_eq!(submit.world_generation, 2);
        assert_eq!(submit.prompt, "follow up");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn continue_world_worker_thread_surface_prefers_event_field_then_uaa_payload() {
        let direct = sample_continue_stream_event(json!({}));
        assert_eq!(
            surfaced_thread_id_from_event(&direct).as_deref(),
            Some("thread-direct")
        );

        let fallback = substrate_common::agent_events::AgentEvent {
            thread_id: None,
            data: json!({
                "uaa_event": {
                    "thread_id": "thread-from-uaa"
                }
            }),
            ..direct.clone()
        };
        assert_eq!(
            surfaced_thread_id_from_event(&fallback).as_deref(),
            Some("thread-from-uaa")
        );

        let raw_event_fallback = substrate_common::agent_events::AgentEvent {
            thread_id: None,
            data: json!({
                "uaa_event": {
                    "raw_event": {
                        "thread_id": "thread-from-raw-event"
                    }
                }
            }),
            ..direct
        };
        assert_eq!(
            surfaced_thread_id_from_event(&raw_event_fallback).as_deref(),
            Some("thread-from-raw-event")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn continue_world_worker_dispatch_contract_classifies_packet_three_worker_events() {
        let submit = sample_continue_submit_request();
        let cases = [
            ("reply", false),
            ("progress_update", false),
            ("result", false),
            ("failure", false),
            ("follow_up_question", true),
            ("blocked", true),
        ];

        for (event_class, attention_required) in cases {
            let classified = classify_continue_world_worker_event(
                &submit,
                &sample_continue_stream_event(json!({
                    "event_class": event_class,
                    "payload": {
                        "message": format!("payload for {event_class}")
                    }
                })),
            )
            .unwrap_or_else(|_| panic!("classification must accept {event_class}"))
            .unwrap_or_else(|| panic!("classification must surface {event_class}"));

            assert_eq!(
                serde_json::to_value(&classified)
                    .expect("serialize classified event")
                    .get("event_class")
                    .and_then(serde_json::Value::as_str),
                Some(event_class)
            );
            assert_eq!(classified.source_participant_id, "ash_member");
            assert_eq!(classified.target_participant_id, "orch_dispatch");
            assert_eq!(classified.source_backend_id, "cli:codex_world");
            assert_eq!(classified.thread_id.as_deref(), Some("thread-direct"));
            assert_eq!(classified.stream_channel.as_deref(), Some("worker.reply"));
            assert_eq!(classified.attention_required, attention_required);
            assert_eq!(
                classified
                    .payload
                    .get("message")
                    .and_then(serde_json::Value::as_str),
                Some(format!("payload for {event_class}").as_str())
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn continue_world_worker_dispatch_contract_respects_explicit_attention_for_in_scope_events() {
        let submit = sample_continue_submit_request();
        let classified = classify_continue_world_worker_event(
            &submit,
            &sample_continue_stream_event(json!({
                "event_class": "result",
                "attention_required": true,
                "payload": {
                    "summary": "needs review"
                }
            })),
        )
        .expect("classification should succeed")
        .expect("result should surface");

        assert!(classified.attention_required);
        assert_eq!(
            classified.event_class,
            ContinueWorldWorkerEventClassV1::Result
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn continue_world_worker_dispatch_contract_rejects_deferred_worker_event_classes() {
        let submit = sample_continue_submit_request();
        for deferred in [
            "approval_request",
            "fork_request",
            "control_directive",
            "attention_required",
        ] {
            let err = classify_continue_world_worker_event(
                &submit,
                &sample_continue_stream_event(json!({
                    "event_class": deferred,
                    "payload": {
                        "message": "not yet in packet 3"
                    }
                })),
            )
            .expect_err("deferred worker event classes must fail closed");
            assert!(
                err.to_string().contains("unsupported_worker_event_class"),
                "unexpected error for {deferred}: {err}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn continue_world_worker_dispatch_contract_rejects_worker_event_identity_drift() {
        let submit = sample_continue_submit_request();
        let drifted = substrate_common::agent_events::AgentEvent {
            participant_id: Some("ash_other".to_string()),
            ..sample_continue_stream_event(json!({
                "event_class": "reply",
                "payload": {
                    "message": "identity drift"
                }
            }))
        };

        let err = classify_continue_world_worker_event(&submit, &drifted)
            .expect_err("participant drift must fail");
        assert!(
            err.to_string().contains(
                "participant_id ash_other did not match targeted retained worker ash_member"
            ),
            "unexpected error: {err}"
        );
    }

    #[cfg(target_os = "linux")]
    fn inventory_entry(agent_id: &str, scope: AgentExecutionScope) -> AgentInventoryEntryV1 {
        AgentInventoryEntryV1 {
            path: PathBuf::from(format!("{agent_id}.yaml")),
            file: AgentFileV1 {
                version: 1,
                id: agent_id.to_string(),
                config: AgentConfigV1 {
                    enabled: true,
                    kind: AgentConfigKind::Cli,
                    protocol: Some("substrate.agent.session".to_string()),
                    execution: AgentExecutionConfigV1 { scope: Some(scope) },
                    cli: Some(AgentCliConfigV1 {
                        binary: "sh".to_string(),
                        mode: Some(AgentCliMode::Persistent),
                        runtime_family: Some(AgentCliRuntimeFamily::Codex),
                    }),
                    api: None,
                    capabilities: AgentCapabilitiesV1 {
                        session_start: true,
                        session_resume: true,
                        session_fork: true,
                        session_stop: true,
                        status_snapshot: true,
                        event_stream: true,
                        llm: true,
                        mcp_client: true,
                    },
                },
                policy_overlay: None,
            },
        }
    }

    #[test]
    fn authoritative_world_binding_accepts_matching_binding() {
        validate_authoritative_world_binding(&sample_session(), &sample_request())
            .expect("matching world binding should validate");
    }

    #[test]
    fn authoritative_world_binding_rejects_world_id_drift() {
        let mut request = sample_request();
        request.world_id = "world-18".to_string();

        let err = validate_authoritative_world_binding(&sample_session(), &request)
            .expect_err("world id drift must fail");
        assert_eq!(
            err.to_string(),
            "world_binding_mismatch: orchestration session sess_dispatch authoritative world_id is world-17 not world-18"
        );
    }

    #[test]
    fn authoritative_world_binding_rejects_world_generation_drift() {
        let mut request = sample_request();
        request.world_generation = 3;

        let err = validate_authoritative_world_binding(&sample_session(), &request)
            .expect_err("world generation drift must fail");
        assert_eq!(
            err.to_string(),
            "world_binding_mismatch: orchestration session sess_dispatch authoritative world_generation is 2 not 3"
        );
    }

    #[test]
    fn authoritative_world_binding_rejects_missing_authoritative_binding() {
        let mut session = sample_session();
        session.world_id = None;
        session.world_generation = None;

        let err = validate_authoritative_world_binding(&session, &sample_request())
            .expect_err("missing world binding must fail");
        assert_eq!(
            err.to_string(),
            "missing_world_binding: orchestration session sess_dispatch has no authoritative world binding"
        );
    }

    #[test]
    fn run_world_task_summary_mentions_terminal_registered_metadata_without_retention() {
        let summary = summarize_run_world_task_result("cli:codex_world", 0, true);
        assert!(
            summary.contains("surfaced continuity metadata"),
            "summary should explain terminal continuity metadata handling: {summary}"
        );
        assert!(
            summary.contains("without retained shell state"),
            "summary should stay explicit about non-retained behavior: {summary}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn build_spawn_world_worker_transport_request_preserves_authoritative_binding() {
        let descriptor = crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor {
            agent_id: "codex_world".to_string(),
            backend_id: "cli:codex_world".to_string(),
            protocol: "substrate.agent.session".to_string(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            execution_scope: crate::execution::config_model::AgentExecutionScope::World,
            binary_path: PathBuf::from("/bin/true"),
        };

        let request = sample_spawn_request();
        let transport =
            build_spawn_world_worker_transport_request(&request, &descriptor).expect("transport");

        assert_eq!(transport.orchestration_session_id, "sess_dispatch");
        assert_eq!(transport.orchestrator_participant_id, "orch_dispatch");
        assert_eq!(transport.backend_id, "cli:codex_world");
        assert_eq!(transport.world_id, "world-17");
        assert_eq!(transport.world_generation, 2);
        assert_eq!(
            transport.initial_prompt.as_deref(),
            Some("open a retained worker")
        );
        assert!(
            transport.participant_id.starts_with("ash_"),
            "retained worker receipt should use participant-style identity: {}",
            transport.participant_id
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn resolve_world_dispatch_contract_rejects_host_scoped_backend_target() {
        let temp = tempdir().expect("tempdir");
        let mut inventory = BTreeMap::new();
        inventory.insert(
            "codex_world".to_string(),
            inventory_entry("codex_world", AgentExecutionScope::Host),
        );
        let context = InternalDispatchContext {
            effective_config: SubstrateConfig::default(),
            base_policy: Policy {
                agents_allowed_backends: vec!["cli:codex_world".to_string()],
                ..Policy::default()
            },
            inventory,
        };

        let err = resolve_world_dispatch_contract(
            temp.path(),
            &context,
            &sample_request(),
            "run_world_task",
        )
        .expect_err("host-scoped backend must fail closed");

        assert_eq!(
            err.to_string(),
            "unsupported_platform_or_posture: backend 'cli:codex_world' resolves only to a host-scoped runtime; run_world_task requires an exact world-scoped backend"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn registered_receipt_requires_exact_authoritative_identity() {
        let request = sample_spawn_request();
        let transport_request = MemberDispatchTransportRequest {
            orchestration_session_id: request.orchestration_session_id.clone(),
            participant_id: "ash_member_receipt".to_string(),
            orchestrator_participant_id: request.caller_participant_id.clone(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: request.target_backend_id.clone(),
            protocol: "substrate.agent.session".to_string(),
            run_id: request.request_id.clone(),
            world_id: request.world_id.clone(),
            world_generation: request.world_generation,
            initial_prompt: Some("open a retained worker".to_string()),
            backend_kind: transport_api_types::MemberRuntimeBackendKindV1::Codex,
            binary_path: "/bin/true".to_string(),
        };

        let mut event = substrate_common::agent_events::AgentEvent {
            ts: chrono::Utc::now(),
            kind: AgentEventKind::Registered,
            data: json!({}),
            agent_id: "codex_world".to_string(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            run_id: request.request_id.clone(),
            parent_run_id: None,
            participant_id: Some("ash_member_receipt".to_string()),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: Some(request.target_backend_id.clone()),
            thread_id: None,
            role: Some("member".to_string()),
            world_id: Some(request.world_id.clone()),
            world_generation: Some(request.world_generation),
            cmd_id: None,
            span_id: Some("spn_spawn".to_string()),
            channel: None,
            identity_tuple: None,
            placement_posture: None,
            project: None,
        };
        event.set_pure_agent_telemetry_identity("codex_world".to_string());

        let receipt = receipt_from_registered_event(
            event.clone(),
            &transport_request,
            &request,
            "spn_spawn".to_string(),
        )
        .expect("receipt");
        assert_eq!(receipt.participant_id, "ash_member_receipt");
        assert_eq!(receipt.orchestrator_participant_id, "orch_dispatch");
        assert_eq!(receipt.backend_id, "cli:codex_world");
        assert_eq!(receipt.world_id, "world-17");
        assert_eq!(receipt.world_generation, 2);
        assert_eq!(receipt.launch_span_id, "spn_spawn");

        event.backend_id = Some("cli:other_world".to_string());
        let err = receipt_from_registered_event(
            event,
            &transport_request,
            &request,
            "spn_spawn".to_string(),
        )
        .expect_err("backend drift must fail");
        assert!(
            err.to_string().contains(
                "registered backend_id cli:other_world did not match requested cli:codex_world"
            ),
            "unexpected error: {err}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn spawn_world_worker_summary_keeps_follow_up_out_of_scope() {
        let summary = summarize_spawn_world_worker_result(&SpawnWorldWorkerReceipt {
            participant_id: "ash_member_receipt".to_string(),
            orchestrator_participant_id: "orch_dispatch".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex_world".to_string(),
            world_id: "world-17".to_string(),
            world_generation: 2,
            launch_span_id: "spn_spawn".to_string(),
        });
        assert!(
            summary.contains("ongoing steering remains out of scope"),
            "summary must stay explicit about Packet 3 scope: {summary}"
        );
    }
}
