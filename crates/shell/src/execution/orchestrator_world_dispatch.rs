use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use substrate_broker::Policy;
use uuid::Uuid;

use crate::execution::agent_inventory::{load_effective_agent_inventory, AgentInventoryEntryV1};
use crate::execution::agent_runtime::control::world_task_terminal_state_from_exit_code;
use crate::execution::agent_runtime::mapping::AgentRuntimeBackendKind;
use crate::execution::agent_runtime::validator::materialize_runtime_descriptor;
use crate::execution::agent_runtime::{
    resolve_inventory_contract_for_exact_backend, AgentRuntimeParticipantRecord,
    AgentRuntimeStateStore, AttachLaunchKnobs, AttachModePreference, DispatchBaselineKind,
    DispatchCallerKind, DispatchCapabilityOverrideSet, DispatchRequestEnvelope,
    HostExecutionClientStart, OrchestrationSessionRecord, ResolvedLaunchContract,
    RunWorldTaskOutcomeV1, TaskPayloadV1, ValidatedWorldDispatchRequestV1, WorldDispatchActionV1,
    WorldDispatchModeV1, WorldDispatchOutcomeV1, WorldDispatchPayloadV1, WorldDispatchRequestV1,
    WorldTaskTerminalStateV1,
};
use crate::execution::config_model::{
    self, AgentExecutionScope, CliConfigOverrides, SubstrateConfig,
};
use crate::execution::routing::{
    build_agent_client_and_member_dispatch_request_for_cwd, MemberDispatchTransportRequest,
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct PreparedOrchestratorWorldDispatch {
    pub request: ValidatedWorldDispatchRequestV1,
    pub session: OrchestrationSessionRecord,
    pub caller_participant: AgentRuntimeParticipantRecord,
}

#[allow(dead_code)]
pub(crate) fn prepare_orchestrator_world_dispatch(
    store: &AgentRuntimeStateStore,
    request: WorldDispatchRequestV1,
) -> Result<PreparedOrchestratorWorldDispatch> {
    let request = request.validate()?;
    let authority = store.resolve_internal_world_dispatch_caller(
        &request.orchestration_session_id,
        &request.caller_participant_id,
    )?;

    Ok(PreparedOrchestratorWorldDispatch {
        request,
        session: authority.session,
        caller_participant: authority.caller_participant,
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
        WorldDispatchActionV1::SpawnWorldWorker => anyhow::bail!(
            "unsupported_dispatch_action: action spawn_world_worker is not implemented until packet 3"
        ),
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
async fn run_world_task(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    validate_authoritative_world_binding(&prepared.session, &prepared.request)?;

    let workspace_root = PathBuf::from(&prepared.session.workspace_root);
    let context = resolve_internal_dispatch_context(&workspace_root)?;
    let resolved = resolve_run_world_task_contract(&workspace_root, &context, &prepared.request)?;
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

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn resolve_run_world_task_contract(
    workspace_root: &Path,
    context: &InternalDispatchContext,
    request: &ValidatedWorldDispatchRequestV1,
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
                    "unsupported_platform_or_posture: backend '{}' resolves only to a host-scoped runtime; run_world_task requires an exact world-scoped backend",
                    request.target_backend_id
                );
            }

            anyhow::bail!(
                "unknown_backend: baseline truth rejected field 'target_backend_id': no exact world-scoped backend match found for '{}'",
                request.target_backend_id
            );
        }
    }
}

#[cfg(target_os = "linux")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::agent_runtime::orchestration_session::{
        OrchestrationSessionPosture, OrchestrationSessionState,
    };

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
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            payload: WorldDispatchPayloadV1::Task(TaskPayloadV1 {
                prompt: "hello world".to_string(),
            }),
        }
        .validate()
        .expect("validated request")
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
}
