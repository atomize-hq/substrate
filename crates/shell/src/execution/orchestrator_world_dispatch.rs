#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::collections::BTreeMap;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};
#[cfg(any(target_os = "linux", target_os = "macos", test))]
use std::sync::{LazyLock, Mutex};

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
use crate::execution::agent_runtime::control::{
    private_stop_transport_path, request_private_stop, PrivateStopOutcome,
};
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::dispatch_contract::{
    ContinueWorldWorkerEventClassV1, ContinueWorldWorkerEventV1, ContinueWorldWorkerOutcomeV1,
    InspectWorldWorkerOutcomeV1, RetainedWorkerStopCloseoutV1, StopWorldWorkerOutcomeV1,
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
    WorldDispatchRequestV1, WorldDispatchSteeringDenialV1,
};
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::{
    AgentRuntimeSessionState, RunWorldTaskOutcomeV1, SpawnWorldWorkerOutcomeV1, TaskPayloadV1,
    WorkerSpawnPayloadV1,
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
#[cfg(target_os = "linux")]
use transport_api_types::ExecuteCancelRequestV1;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct PreparedOrchestratorWorldDispatch {
    pub store: AgentRuntimeStateStore,
    pub request: ValidatedWorldDispatchRequestV1,
    pub session: OrchestrationSessionRecord,
    pub caller_participant: AgentRuntimeParticipantRecord,
    pub target_participant: Option<AgentRuntimeParticipantRecord>,
    pub live_retained_worker_count: usize,
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
#[derive(Default)]
struct WorldDispatchConcurrencyTracker {
    ephemeral_by_session: BTreeMap<String, usize>,
    retained_bootstrap_by_session: BTreeMap<String, usize>,
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
#[derive(Clone, Copy, Debug)]
enum WorldDispatchConcurrencyKind {
    Ephemeral,
    RetainedBootstrap,
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
#[derive(Debug)]
struct WorldDispatchConcurrencyGuard {
    session_id: String,
    kind: WorldDispatchConcurrencyKind,
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
impl Drop for WorldDispatchConcurrencyGuard {
    fn drop(&mut self) {
        let mut tracker = world_dispatch_concurrency_tracker()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let counts = match self.kind {
            WorldDispatchConcurrencyKind::Ephemeral => &mut tracker.ephemeral_by_session,
            WorldDispatchConcurrencyKind::RetainedBootstrap => {
                &mut tracker.retained_bootstrap_by_session
            }
        };
        match counts.get_mut(&self.session_id) {
            Some(count) if *count > 1 => *count -= 1,
            Some(_) => {
                counts.remove(&self.session_id);
            }
            None => {}
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn world_dispatch_concurrency_tracker() -> &'static Mutex<WorldDispatchConcurrencyTracker> {
    static TRACKER: LazyLock<Mutex<WorldDispatchConcurrencyTracker>> =
        LazyLock::new(|| Mutex::new(WorldDispatchConcurrencyTracker::default()));
    &TRACKER
}

#[allow(dead_code)]
pub(crate) fn prepare_orchestrator_world_dispatch(
    store: &AgentRuntimeStateStore,
    request: WorldDispatchRequestV1,
) -> Result<PreparedOrchestratorWorldDispatch> {
    let request = request.validate()?;
    let authority = store
        .resolve_internal_world_dispatch_caller(
            &request.orchestration_session_id,
            &request.caller_participant_id,
        )
        .map_err(map_world_dispatch_resolution_error)?;
    let session = authority.session;
    let caller_participant = authority.caller_participant;
    let live_retained_worker_count = store.count_authoritative_live_retained_workers(
        &session.orchestration_session_id,
        caller_participant.participant_id(),
    )?;

    Ok(PreparedOrchestratorWorldDispatch {
        store: store.clone(),
        request,
        session,
        caller_participant,
        target_participant: None,
        live_retained_worker_count,
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
        WorldDispatchActionV1::InspectWorldWorker => inspect_world_worker(prepared).await,
        WorldDispatchActionV1::StopWorldWorker => stop_world_worker(prepared).await,
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

#[cfg(not(target_os = "linux"))]
async fn inspect_world_worker(
    _prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    anyhow::bail!(
        "unsupported_platform_or_posture: inspect_world_worker retained snapshot routing is supported only on linux in v1"
    );
}

#[cfg(not(target_os = "linux"))]
async fn stop_world_worker(
    _prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    anyhow::bail!(
        "unsupported_platform_or_posture: stop_world_worker retained closeout routing is supported only on linux in v1"
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
const STOP_WORLD_WORKER_CLOSEOUT_WAIT_TIMEOUT: Duration = Duration::from_secs(5);
#[cfg(target_os = "linux")]
const STOP_WORLD_WORKER_CLOSEOUT_POLL_INTERVAL: Duration = Duration::from_millis(25);

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
    _concurrency_guard: Option<WorldDispatchConcurrencyGuard>,
}

#[cfg(target_os = "linux")]
async fn run_world_task(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    let workspace_root = PathBuf::from(&prepared.session.workspace_root);
    let context = resolve_internal_dispatch_context(&workspace_root)?;
    enforce_world_dispatch_steering_policy(&prepared, &context.base_policy)?;
    let resolved = resolve_world_dispatch_contract(
        &workspace_root,
        &context,
        &prepared.request,
        "run_world_task",
    )?;
    let _concurrency_guard =
        acquire_world_dispatch_concurrency_guard(&prepared, &context.base_policy)?;
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
    let workspace_root = PathBuf::from(&prepared.session.workspace_root);
    let context = resolve_internal_dispatch_context(&workspace_root)?;
    enforce_world_dispatch_steering_policy(&prepared, &context.base_policy)?;
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
    let concurrency_guard =
        acquire_world_dispatch_concurrency_guard(&prepared, &context.base_policy)?;

    Ok(PreparedSpawnWorldWorkerBootstrap {
        request: prepared.request,
        descriptor,
        _concurrency_guard: concurrency_guard,
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
    let workspace_root = PathBuf::from(&prepared.session.workspace_root);
    let base_policy = resolve_internal_dispatch_policy(&workspace_root)?;
    enforce_world_dispatch_steering_policy(&prepared, &base_policy)?;
    let prepared = resolve_continue_world_dispatch_target_for_routing(prepared)?;

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

#[cfg(target_os = "linux")]
async fn inspect_world_worker(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    let workspace_root = PathBuf::from(&prepared.session.workspace_root);
    let base_policy = resolve_internal_dispatch_policy(&workspace_root)?;
    enforce_world_dispatch_steering_policy(&prepared, &base_policy)?;

    let resolved = prepared
        .store
        .resolve_internal_inspect_world_dispatch_target(
            &prepared.request.orchestration_session_id,
            &prepared.request.caller_participant_id,
            prepared
                .request
                .target_participant_id
                .as_deref()
                .expect("validated inspect request must include target_participant_id"),
            &prepared.request.target_backend_id,
        )
        .map_err(map_world_dispatch_resolution_error)?;

    let target_participant_id = resolved.target_participant.participant_id().to_string();
    let target_backend_id = resolved.target_participant.handle.backend_id.clone();
    let world_id = resolved.session.world_id.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "missing_world_binding: orchestration session {} has no authoritative world binding",
            resolved.session.orchestration_session_id
        )
    })?;
    let world_generation = resolved.session.world_generation.ok_or_else(|| {
        anyhow::anyhow!(
            "missing_world_binding: orchestration session {} has no authoritative world binding",
            resolved.session.orchestration_session_id
        )
    })?;
    let snapshot = resolved.project_snapshot();
    let summary = summarize_inspect_world_worker_result(&target_participant_id, &target_backend_id);

    Ok(WorldDispatchOutcomeV1::InspectWorldWorker(
        InspectWorldWorkerOutcomeV1 {
            request_id: prepared.request.request_id,
            orchestration_session_id: resolved.session.orchestration_session_id.clone(),
            action: WorldDispatchActionV1::InspectWorldWorker,
            mode: prepared.request.mode,
            orchestrator_participant_id: resolved.caller_participant.participant_id().to_string(),
            target_participant_id,
            target_backend_id,
            world_id,
            world_generation,
            snapshot,
            summary,
        },
    ))
}

#[cfg(target_os = "linux")]
async fn stop_world_worker(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<WorldDispatchOutcomeV1> {
    let workspace_root = PathBuf::from(&prepared.session.workspace_root);
    let base_policy = resolve_internal_dispatch_policy(&workspace_root)?;
    enforce_world_dispatch_steering_policy(&prepared, &base_policy)?;

    let resolved = prepared
        .store
        .resolve_internal_stop_world_dispatch_target(
            &prepared.request.orchestration_session_id,
            &prepared.request.caller_participant_id,
            prepared
                .request
                .target_participant_id
                .as_deref()
                .expect("validated stop request must include target_participant_id"),
            &prepared.request.target_backend_id,
        )
        .map_err(map_world_dispatch_resolution_error)?;

    let transport_path = private_stop_transport_path(
        &prepared.store,
        &resolved.session.orchestration_session_id,
        resolved.target_participant.participant_id(),
    );
    let transport_result = request_private_stop(&transport_path).await;
    let closeout = wait_for_stop_world_worker_closeout(
        &prepared.store,
        &resolved.session.orchestration_session_id,
        resolved.target_participant.participant_id(),
    )
    .await;
    let closeout = match (transport_result, closeout) {
        (Ok(PrivateStopOutcome::Accepted | PrivateStopOutcome::AlreadyTerminal), Ok(closeout)) => {
            closeout
        }
        (Ok(PrivateStopOutcome::OwnerUnreachable), Ok(closeout))
        | (Ok(PrivateStopOutcome::ProtocolError), Ok(closeout))
        | (Err(_), Ok(closeout)) => closeout,
        (Ok(PrivateStopOutcome::Accepted | PrivateStopOutcome::AlreadyTerminal), Err(err)) => {
            return Err(err);
        }
        (Ok(PrivateStopOutcome::OwnerUnreachable), Err(err)) => {
            return Err(anyhow::anyhow!(
                "owner_unreachable: private stop transport for retained worker {} did not stay reachable until durable stop closeout completed: {err}",
                resolved.target_participant.participant_id()
            ));
        }
        (Ok(PrivateStopOutcome::ProtocolError), Err(err)) => {
            return Err(anyhow::anyhow!(
                "owner_unreachable: private stop transport for retained worker {} returned a protocol error before durable stop closeout completed: {err}",
                resolved.target_participant.participant_id()
            ));
        }
        (Err(connect_err), Err(closeout_err)) => {
            return Err(anyhow::anyhow!(
                "owner_unreachable: failed to deliver stop_world_worker to retained worker {} and durable stop closeout was not observed ({connect_err:#}; {closeout_err})",
                resolved.target_participant.participant_id()
            ));
        }
    };
    let summary = summarize_stop_world_worker_result(
        resolved.target_participant.participant_id(),
        &resolved.target_participant.handle.backend_id,
    );

    Ok(WorldDispatchOutcomeV1::StopWorldWorker(
        StopWorldWorkerOutcomeV1 {
            request_id: prepared.request.request_id,
            orchestration_session_id: resolved.session.orchestration_session_id.clone(),
            action: WorldDispatchActionV1::StopWorldWorker,
            mode: prepared.request.mode,
            orchestrator_participant_id: resolved.caller_participant.participant_id().to_string(),
            target_participant_id: resolved.target_participant.participant_id().to_string(),
            target_backend_id: resolved.target_participant.handle.backend_id.clone(),
            world_id: resolved.session.world_id.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "missing_world_binding: orchestration session {} has no authoritative world binding",
                    resolved.session.orchestration_session_id
                )
            })?,
            world_generation: resolved.session.world_generation.ok_or_else(|| {
                anyhow::anyhow!(
                    "missing_world_binding: orchestration session {} has no authoritative world binding",
                    resolved.session.orchestration_session_id
                )
            })?,
            closeout,
            summary,
        },
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn resolve_internal_dispatch_context(workspace_root: &Path) -> Result<InternalDispatchContext> {
    let effective_config =
        config_model::resolve_effective_config(workspace_root, &CliConfigOverrides::default())?;
    let base_policy = resolve_internal_dispatch_policy(workspace_root)?;
    let inventory = load_effective_agent_inventory(workspace_root, &base_policy)?;

    Ok(InternalDispatchContext {
        effective_config,
        base_policy,
        inventory,
    })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn resolve_internal_dispatch_policy(workspace_root: &Path) -> Result<Policy> {
    let (base_policy, _) =
        substrate_broker::resolve_effective_policy_with_explain(workspace_root, false)
            .map_err(|err| config_model::user_error(err.to_string()))?;
    Ok(base_policy)
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

#[allow(dead_code)]
fn steering_policy_denial(
    bucket: WorldDispatchSteeringDenialV1,
    detail: impl AsRef<str>,
) -> anyhow::Error {
    anyhow::anyhow!("{}", bucket.format_message(detail))
}

fn map_world_dispatch_resolution_error(err: anyhow::Error) -> anyhow::Error {
    let message = err.to_string();
    if message.starts_with("caller_not_authoritative:")
        || message.starts_with("target_not_in_session:")
        || message.starts_with("ambiguous_target_participant:")
    {
        return steering_policy_denial(
            WorldDispatchSteeringDenialV1::CrossSessionSteeringDenied,
            message,
        );
    }
    if message.starts_with("world_binding_mismatch:") {
        return steering_policy_denial(
            WorldDispatchSteeringDenialV1::CrossWorldBindingSteeringDenied,
            message,
        );
    }

    err
}

#[cfg(target_os = "linux")]
fn map_continue_world_dispatch_resolution_error(err: anyhow::Error) -> anyhow::Error {
    let message = err.to_string();
    if message.starts_with("stale_linkage:") {
        return steering_policy_denial(
            WorldDispatchSteeringDenialV1::InvalidatedWorkerNotRoutable,
            message,
        );
    }

    map_world_dispatch_resolution_error(err)
}

#[cfg(target_os = "linux")]
fn resolve_continue_world_dispatch_target_for_routing(
    prepared: PreparedOrchestratorWorldDispatch,
) -> Result<PreparedOrchestratorWorldDispatch> {
    if prepared.target_participant.is_some() {
        return Ok(prepared);
    }

    let resolved = prepared
        .store
        .resolve_internal_continue_world_dispatch_target(
            &prepared.request.orchestration_session_id,
            &prepared.request.caller_participant_id,
            prepared
                .request
                .target_participant_id
                .as_deref()
                .expect("validated continue request must include target_participant_id"),
            &prepared.request.target_backend_id,
        )
        .map_err(map_continue_world_dispatch_resolution_error)?;

    Ok(PreparedOrchestratorWorldDispatch {
        store: prepared.store,
        request: prepared.request,
        session: resolved.session,
        caller_participant: resolved.caller_participant,
        target_participant: Some(resolved.target_participant),
        live_retained_worker_count: prepared.live_retained_worker_count,
    })
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn acquire_world_dispatch_concurrency_guard(
    prepared: &PreparedOrchestratorWorldDispatch,
    base_policy: &Policy,
) -> Result<Option<WorldDispatchConcurrencyGuard>> {
    let steering_policy = base_policy.world_dispatch_policy();
    let session_id = prepared.request.orchestration_session_id.clone();
    let mut tracker = world_dispatch_concurrency_tracker()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    match prepared.request.action {
        WorldDispatchActionV1::RunWorldTask => {
            let cap = steering_policy.max_concurrent_ephemeral as usize;
            let current = tracker
                .ephemeral_by_session
                .get(&session_id)
                .copied()
                .unwrap_or(0);
            if current >= cap {
                return Err(steering_policy_denial(
                    WorldDispatchSteeringDenialV1::WorkerConcurrencyCapExceeded,
                    format!(
                        "effective policy allows at most {cap} concurrent ephemeral world dispatches for orchestration session {}",
                        prepared.request.orchestration_session_id
                    ),
                ));
            }
            *tracker
                .ephemeral_by_session
                .entry(session_id.clone())
                .or_default() += 1;
            Ok(Some(WorldDispatchConcurrencyGuard {
                session_id,
                kind: WorldDispatchConcurrencyKind::Ephemeral,
            }))
        }
        WorldDispatchActionV1::SpawnWorldWorker => {
            let cap = steering_policy.max_live_retained_workers as usize;
            let reserved = tracker
                .retained_bootstrap_by_session
                .get(&session_id)
                .copied()
                .unwrap_or(0);
            if prepared.live_retained_worker_count.saturating_add(reserved) >= cap {
                return Err(steering_policy_denial(
                    WorldDispatchSteeringDenialV1::WorkerConcurrencyCapExceeded,
                    format!(
                        "effective policy allows at most {cap} live retained workers for orchestration session {}; authoritative live count is {}",
                        prepared.request.orchestration_session_id,
                        prepared.live_retained_worker_count
                    ),
                ));
            }
            *tracker
                .retained_bootstrap_by_session
                .entry(session_id.clone())
                .or_default() += 1;
            Ok(Some(WorldDispatchConcurrencyGuard {
                session_id,
                kind: WorldDispatchConcurrencyKind::RetainedBootstrap,
            }))
        }
        WorldDispatchActionV1::ContinueWorldWorker
        | WorldDispatchActionV1::InspectWorldWorker
        | WorldDispatchActionV1::StopWorldWorker => Ok(None),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn enforce_world_dispatch_steering_policy(
    prepared: &PreparedOrchestratorWorldDispatch,
    base_policy: &Policy,
) -> Result<()> {
    let steering_policy = base_policy.world_dispatch_policy();
    if !steering_policy.enabled {
        return Err(steering_policy_denial(
            WorldDispatchSteeringDenialV1::WorldDispatchDisabled,
            "host-to-world steering is disabled by effective policy",
        ));
    }
    if !steering_policy
        .allowed_actions
        .iter()
        .any(|value| value == prepared.request.action.as_str())
    {
        return Err(steering_policy_denial(
            WorldDispatchSteeringDenialV1::ActionNotAllowed,
            format!(
                "effective policy does not allow {}",
                prepared.request.action.as_str()
            ),
        ));
    }
    if !steering_policy
        .allowed_modes
        .iter()
        .any(|value| value == prepared.request.mode.as_str())
    {
        return Err(steering_policy_denial(
            WorldDispatchSteeringDenialV1::ModeNotAllowed,
            format!(
                "effective policy does not allow {}",
                prepared.request.mode.as_str()
            ),
        ));
    }
    if !steering_policy
        .allowed_backends
        .iter()
        .any(|value| value == &prepared.request.target_backend_id)
    {
        return Err(steering_policy_denial(
            WorldDispatchSteeringDenialV1::BackendNotAllowed,
            format!(
                "effective policy does not allow backend {}",
                prepared.request.target_backend_id
            ),
        ));
    }

    if steering_policy.same_session_only {
        validate_authoritative_session_boundary(prepared)?;
    }
    if steering_policy.same_world_binding_only {
        validate_authoritative_world_binding_for_steering(prepared)?;
    }

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn validate_authoritative_session_boundary(
    prepared: &PreparedOrchestratorWorldDispatch,
) -> Result<()> {
    let authoritative_session_id = &prepared.session.orchestration_session_id;
    let target_crosses_session = prepared
        .target_participant
        .as_ref()
        .is_some_and(|participant| {
            participant.handle.orchestration_session_id != *authoritative_session_id
        });
    if prepared.request.orchestration_session_id != *authoritative_session_id
        || prepared.caller_participant.handle.orchestration_session_id != *authoritative_session_id
        || target_crosses_session
    {
        return Err(steering_policy_denial(
            WorldDispatchSteeringDenialV1::CrossSessionSteeringDenied,
            format!(
                "request must stay within authoritative orchestration session {}",
                authoritative_session_id
            ),
        ));
    }

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn validate_authoritative_world_binding_for_steering(
    prepared: &PreparedOrchestratorWorldDispatch,
) -> Result<()> {
    validate_authoritative_world_binding(&prepared.session, &prepared.request).map_err(|err| {
        steering_policy_denial(
            WorldDispatchSteeringDenialV1::CrossWorldBindingSteeringDenied,
            err.to_string(),
        )
    })?;

    if let Some(target_participant) = prepared.target_participant.as_ref() {
        if !target_participant.matches_authoritative_parent_world_binding(&prepared.session) {
            return Err(steering_policy_denial(
                WorldDispatchSteeringDenialV1::CrossWorldBindingSteeringDenied,
                format!(
                    "retained worker {} no longer matches the authoritative world binding for session {}",
                    target_participant.participant_id(),
                    prepared.session.orchestration_session_id
                ),
            ));
        }
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
    let mut active_span_id = None::<String>;
    let mut exit_code = None::<i32>;
    let mut surfaced_thread_id = None::<String>;
    let mut surfaced_worker_event = None::<ContinueWorldWorkerEventV1>;

    while let Some(frame) = body.as_mut().frame().await {
        let frame = match frame {
            Ok(frame) => frame,
            Err(err) => {
                cancel_continue_world_worker_turn(&client, active_span_id.as_deref()).await;
                return Err(anyhow::anyhow!("stream frame error: {err}"));
            }
        };
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
            let frame: ExecuteStreamFrame =
                match serde_json::from_slice(payload).with_context(|| {
                    format!(
                        "invalid continue_world_worker stream frame: {}",
                        String::from_utf8_lossy(payload)
                    )
                }) {
                    Ok(frame) => frame,
                    Err(err) => {
                        cancel_continue_world_worker_turn(&client, active_span_id.as_deref()).await;
                        return Err(err);
                    }
                };

            match frame {
                ExecuteStreamFrame::Start { span_id } => {
                    active_span_id = Some(span_id);
                }
                ExecuteStreamFrame::Event { event } => {
                    if surfaced_thread_id.is_none() {
                        surfaced_thread_id = surfaced_thread_id_from_event(&event);
                    }
                    let classified_event =
                        match classify_continue_world_worker_event(request, &event) {
                            Ok(classified_event) => classified_event,
                            Err(err) => {
                                cancel_continue_world_worker_turn(
                                    &client,
                                    active_span_id.as_deref(),
                                )
                                .await;
                                return Err(err);
                            }
                        };
                    if let Some(classified_event) = classified_event {
                        surfaced_worker_event = Some(classified_event);
                    }
                }
                ExecuteStreamFrame::Exit { exit, .. } => {
                    exit_code = Some(exit);
                    break;
                }
                ExecuteStreamFrame::Error { message } => {
                    cancel_continue_world_worker_turn(&client, active_span_id.as_deref()).await;
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
    });
    if exit_code.is_err() {
        cancel_continue_world_worker_turn(&client, active_span_id.as_deref()).await;
    }
    let exit_code = exit_code?;

    Ok(ContinueWorldWorkerStreamResult {
        exit_code,
        surfaced_thread_id,
        surfaced_worker_event,
    })
}

#[cfg(target_os = "linux")]
async fn cancel_continue_world_worker_turn(
    client: &transport_api_client::AgentClient,
    span_id: Option<&str>,
) {
    let Some(span_id) = span_id.map(str::trim).filter(|span_id| !span_id.is_empty()) else {
        return;
    };
    let _ = client
        .cancel_execute(ExecuteCancelRequestV1 {
            span_id: span_id.to_string(),
            sig: "INT".to_string(),
        })
        .await;
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
    let event_class = if let Some(event_class_label) =
        continue_world_worker_event_class_label(event)
    {
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
        event_class
    } else {
        let Some(event_class) = continue_world_worker_event_class_from_stream_shape(event)
            .or_else(|| continue_world_worker_event_class_from_runtime_shape(event))
        else {
            return Ok(None);
        };
        event_class
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
fn continue_world_worker_event_class_from_stream_shape(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<ContinueWorldWorkerEventClassV1> {
    let event_type = continue_world_worker_stream_event_type(event)?;
    match event_type {
        "thread.started" | "thread.resumed" | "turn.started" => None,
        "turn.completed" => Some(ContinueWorldWorkerEventClassV1::Result),
        "turn.failed" | "item.failed" | "error" => Some(ContinueWorldWorkerEventClassV1::Failure),
        "item.started" | "item.created" | "item.delta" | "item.updated" | "item.completed" => {
            match continue_world_worker_stream_item_type(event) {
                Some("agent_message")
                    if event_type == "item.completed"
                        || continue_world_worker_stream_item_status(event) == Some("completed") =>
                {
                    Some(ContinueWorldWorkerEventClassV1::Reply)
                }
                Some("error") => Some(ContinueWorldWorkerEventClassV1::Failure),
                Some(_) | None => Some(ContinueWorldWorkerEventClassV1::ProgressUpdate),
            }
        }
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn continue_world_worker_event_class_from_runtime_shape(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<ContinueWorldWorkerEventClassV1> {
    if continue_world_worker_runtime_tools_facet(event) {
        if continue_world_worker_runtime_tools_facet_failed(event) {
            return Some(ContinueWorldWorkerEventClassV1::Failure);
        }
        return Some(ContinueWorldWorkerEventClassV1::ProgressUpdate);
    }

    if matches!(
        event.kind,
        substrate_common::agent_events::AgentEventKind::Alert
    ) || matches!(event.channel.as_deref(), Some("error"))
    {
        return Some(ContinueWorldWorkerEventClassV1::Failure);
    }

    if matches!(
        event.kind,
        substrate_common::agent_events::AgentEventKind::TaskProgress
    ) && matches!(event.channel.as_deref(), Some("assistant"))
    {
        return Some(ContinueWorldWorkerEventClassV1::Reply);
    }

    if matches!(
        event.kind,
        substrate_common::agent_events::AgentEventKind::Status
    ) && continue_world_worker_runtime_status_message(event)
        .is_some_and(|message| message.eq_ignore_ascii_case("turn failed"))
    {
        return Some(ContinueWorldWorkerEventClassV1::Failure);
    }

    None
}

#[cfg(target_os = "linux")]
fn continue_world_worker_runtime_tools_facet(
    event: &substrate_common::agent_events::AgentEvent,
) -> bool {
    continue_world_worker_string_field(
        &event.data,
        &[
            "/uaa_event/schema",
            "/uaa_event/raw_event/schema",
            "/raw_event/schema",
        ],
    ) == Some("agent_api.tools.structured.v1")
}

#[cfg(target_os = "linux")]
fn continue_world_worker_runtime_tools_facet_failed(
    event: &substrate_common::agent_events::AgentEvent,
) -> bool {
    continue_world_worker_string_field(
        &event.data,
        &[
            "/uaa_event/tool/status",
            "/uaa_event/raw_event/tool/status",
            "/raw_event/tool/status",
        ],
    ) == Some("failed")
        || continue_world_worker_string_field(
            &event.data,
            &[
                "/uaa_event/tool/phase",
                "/uaa_event/raw_event/tool/phase",
                "/raw_event/tool/phase",
            ],
        ) == Some("fail")
}

#[cfg(target_os = "linux")]
fn continue_world_worker_runtime_status_message(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<&str> {
    continue_world_worker_string_field(&event.data, &["/message"])
}

#[cfg(target_os = "linux")]
fn continue_world_worker_stream_event_type(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<&str> {
    continue_world_worker_string_field(
        &event.data,
        &[
            "/type",
            "/uaa_event/type",
            "/uaa_event/raw_event/type",
            "/raw_event/type",
        ],
    )
}

#[cfg(target_os = "linux")]
fn continue_world_worker_stream_item_type(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<&str> {
    continue_world_worker_string_field(
        &event.data,
        &[
            "/item_type",
            "/item/item_type",
            "/item/type",
            "/uaa_event/item_type",
            "/uaa_event/item/item_type",
            "/uaa_event/item/type",
            "/uaa_event/raw_event/item_type",
            "/uaa_event/raw_event/item/item_type",
            "/uaa_event/raw_event/item/type",
            "/raw_event/item_type",
            "/raw_event/item/item_type",
            "/raw_event/item/type",
        ],
    )
}

#[cfg(target_os = "linux")]
fn continue_world_worker_stream_item_status(
    event: &substrate_common::agent_events::AgentEvent,
) -> Option<&str> {
    continue_world_worker_string_field(
        &event.data,
        &[
            "/status",
            "/item/status",
            "/uaa_event/status",
            "/uaa_event/item/status",
            "/uaa_event/raw_event/status",
            "/uaa_event/raw_event/item/status",
            "/raw_event/status",
            "/raw_event/item/status",
        ],
    )
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

#[cfg(target_os = "linux")]
fn summarize_inspect_world_worker_result(participant_id: &str, backend_id: &str) -> String {
    format!(
        "inspect_world_worker returned an authoritative retained snapshot for worker {participant_id} on backend {backend_id} without invoking world-side execution transport"
    )
}

#[cfg(target_os = "linux")]
async fn wait_for_stop_world_worker_closeout(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<RetainedWorkerStopCloseoutV1> {
    let started_at = Instant::now();
    loop {
        let participant = store.load_participant(participant_id)?.ok_or_else(|| {
            anyhow::anyhow!(
                "missing_target_participant: retained worker {} disappeared before durable stop closeout was observed",
                participant_id
            )
        })?;
        if participant.handle.state == AgentRuntimeSessionState::Stopped {
            let session = store
                .load_orchestration_session(orchestration_session_id)?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "missing_orchestration_session: orchestration session {} disappeared before durable stop closeout was observed",
                        orchestration_session_id
                    )
                })?;
            return Ok(RetainedWorkerStopCloseoutV1 {
                participant_state: participant.handle.state,
                session_state: session.state,
            });
        }
        if !participant.handle.state.is_live() {
            anyhow::bail!(
                "stop_closeout_failed: retained worker {} reached terminal state {} instead of stopped",
                participant_id,
                agent_runtime_session_state_label(&participant.handle.state)
            );
        }
        if started_at.elapsed() >= STOP_WORLD_WORKER_CLOSEOUT_WAIT_TIMEOUT {
            anyhow::bail!(
                "owner_unreachable: timed out waiting for retained worker {} to reach durable stopped closeout",
                participant_id
            );
        }
        tokio::time::sleep(STOP_WORLD_WORKER_CLOSEOUT_POLL_INTERVAL).await;
    }
}

#[cfg(target_os = "linux")]
fn summarize_stop_world_worker_result(participant_id: &str, backend_id: &str) -> String {
    format!(
        "stop_world_worker drove durable stopped closeout for retained worker {participant_id} on backend {backend_id} via the existing private owner stop surface"
    )
}

#[cfg(target_os = "linux")]
fn agent_runtime_session_state_label(state: &AgentRuntimeSessionState) -> &'static str {
    match state {
        AgentRuntimeSessionState::Allocating => "allocating",
        AgentRuntimeSessionState::Ready => "ready",
        AgentRuntimeSessionState::Running => "running",
        AgentRuntimeSessionState::Restarting => "restarting",
        AgentRuntimeSessionState::Stopping => "stopping",
        AgentRuntimeSessionState::Stopped => "stopped",
        AgentRuntimeSessionState::Failed => "failed",
        AgentRuntimeSessionState::Invalidated => "invalidated",
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
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::dispatch_contract::WorkerInspectPayloadV1;
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::orchestration_session::HostAttachContract;
    use crate::execution::agent_runtime::orchestration_session::{
        OrchestrationSessionPosture, OrchestrationSessionState,
    };
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::AgentRuntimeSessionState;
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::WorkerSpawnPayloadV1;
    use crate::execution::agent_runtime::{
        TaskPayloadV1, WorldDispatchModeV1, WorldDispatchPayloadV1,
    };
    #[cfg(target_os = "linux")]
    use crate::execution::config_model::AgentCliMode;
    #[cfg(target_os = "linux")]
    use crate::execution::world_env_guard;
    #[cfg(target_os = "linux")]
    use hyper014::body::{to_bytes, HttpBody};
    #[cfg(target_os = "linux")]
    use serde_json::json;
    #[cfg(target_os = "linux")]
    use serial_test::serial;
    #[cfg(target_os = "linux")]
    use std::collections::HashMap;
    #[cfg(target_os = "linux")]
    use std::fs;
    #[cfg(target_os = "linux")]
    use std::path::Path;
    #[cfg(target_os = "linux")]
    use std::sync::{Arc, Mutex};
    #[cfg(target_os = "linux")]
    use substrate_common::agent_events::AgentEventKind;
    #[cfg(target_os = "linux")]
    use tempfile::tempdir;
    #[cfg(target_os = "linux")]
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    #[cfg(target_os = "linux")]
    use tokio::net::UnixListener;
    #[cfg(target_os = "linux")]
    use tokio::time::timeout;
    #[cfg(target_os = "linux")]
    use transport_api_types::{
        ExecuteCancelRequestV1, ExecuteRequest, MemberDispatchRequestV1,
        MemberRuntimeBackendKindV1, PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3,
        PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3, ResolvedMemberRuntimeDescriptorV1,
    };
    #[cfg(target_os = "linux")]
    use world_api::{SharedWorldOwnerAction, SharedWorldOwnerSpec, WorldReuseMode, WorldSpec};
    #[cfg(target_os = "linux")]
    use world_service::WorldService;

    #[cfg(target_os = "linux")]
    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    #[cfg(target_os = "linux")]
    impl EnvVarGuard {
        fn set_path(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_deref() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

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
    fn sample_world_dispatch_policy() -> Policy {
        Policy {
            agents_world_dispatch_enabled: true,
            agents_world_dispatch_allowed_backends: vec!["cli:codex_world".to_string()],
            agents_world_dispatch_allowed_actions: vec![
                "run_world_task".to_string(),
                "spawn_world_worker".to_string(),
                "continue_world_worker".to_string(),
                "inspect_world_worker".to_string(),
                "stop_world_worker".to_string(),
            ],
            agents_world_dispatch_allowed_modes: vec![
                "ephemeral".to_string(),
                "retained".to_string(),
            ],
            agents_world_dispatch_same_session_only: true,
            agents_world_dispatch_same_world_binding_only: true,
            agents_world_dispatch_max_live_retained_workers: 4,
            agents_world_dispatch_max_concurrent_ephemeral: 4,
            ..Policy::default()
        }
    }

    #[cfg(target_os = "linux")]
    fn sample_state_store() -> AgentRuntimeStateStore {
        AgentRuntimeStateStore::new().expect("state store")
    }

    #[cfg(target_os = "linux")]
    fn write_allowed_world_dispatch_policy(
        substrate_home: &Path,
        backend_id: &str,
        allowed_actions: &[&str],
        allowed_modes: &[&str],
    ) {
        write_world_dispatch_policy(
            substrate_home,
            true,
            &[backend_id],
            allowed_actions,
            allowed_modes,
        );
    }

    #[cfg(target_os = "linux")]
    fn write_world_dispatch_policy(
        substrate_home: &Path,
        enabled: bool,
        allowed_backends: &[&str],
        allowed_actions: &[&str],
        allowed_modes: &[&str],
    ) {
        let enabled = if enabled { "true" } else { "false" };
        let backends = allowed_backends
            .iter()
            .map(|value| format!("      - \"{value}\""))
            .collect::<Vec<_>>()
            .join("\n");
        let actions = allowed_actions
            .iter()
            .map(|value| format!("      - \"{value}\""))
            .collect::<Vec<_>>()
            .join("\n");
        let modes = allowed_modes
            .iter()
            .map(|value| format!("      - \"{value}\""))
            .collect::<Vec<_>>()
            .join("\n");
        let policy = format!(
            "id: test-global-policy\nname: Test Global Policy\nagents:\n  world_dispatch:\n    enabled: {enabled}\n    allowed_backends:\n{backends}\n    allowed_actions:\n{actions}\n    allowed_modes:\n{modes}\n    same_session_only: true\n    same_world_binding_only: true\n    allow_capability_narrowing: false\n    max_live_retained_workers: 4\n    max_concurrent_ephemeral: 4\n"
        );
        fs::write(substrate_home.join("policy.yaml"), policy).expect("write policy");
    }

    #[cfg(target_os = "linux")]
    fn sample_continue_world_dispatch_request() -> WorldDispatchRequestV1 {
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
    }

    #[cfg(target_os = "linux")]
    fn sample_continue_request() -> ValidatedWorldDispatchRequestV1 {
        sample_continue_world_dispatch_request()
            .validate()
            .expect("validated continue request")
    }

    #[cfg(target_os = "linux")]
    fn sample_inspect_world_dispatch_request() -> WorldDispatchRequestV1 {
        WorldDispatchRequestV1 {
            request_id: Some("req_inspect".to_string()),
            idempotency_key: Some("idem_inspect".to_string()),
            orchestration_session_id: Some("sess_dispatch".to_string()),
            caller_participant_id: Some("orch_dispatch".to_string()),
            action: WorldDispatchActionV1::InspectWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some("cli:codex_world".to_string()),
            target_participant_id: Some("ash_member".to_string()),
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            payload: WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        }
    }

    #[cfg(target_os = "linux")]
    fn sample_stop_world_dispatch_request() -> WorldDispatchRequestV1 {
        WorldDispatchRequestV1 {
            request_id: Some("req_stop".to_string()),
            idempotency_key: Some("idem_stop".to_string()),
            orchestration_session_id: Some("sess_dispatch".to_string()),
            caller_participant_id: Some("orch_dispatch".to_string()),
            action: WorldDispatchActionV1::StopWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some("cli:codex_world".to_string()),
            target_participant_id: Some("ash_member".to_string()),
            world_id: Some("world-17".to_string()),
            world_generation: Some(2),
            payload: WorldDispatchPayloadV1::WorkerStop(
                crate::execution::agent_runtime::dispatch_contract::WorkerStopPayloadV1::default(),
            ),
        }
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
    fn sample_continue_stream_uaa_event(
        raw_event: serde_json::Value,
    ) -> substrate_common::agent_events::AgentEvent {
        sample_continue_stream_event(json!({
            "uaa_event": raw_event,
            "protocol": "substrate.agent.session",
        }))
    }

    #[cfg(target_os = "linux")]
    fn minimal_policy_snapshot() -> PolicySnapshotV3 {
        PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: None,
                read: None,
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        }
    }

    #[cfg(target_os = "linux")]
    fn make_member_dispatch_execute_request(
        cwd: &Path,
        binary_path: &Path,
        world_id: &str,
        world_generation: u64,
        run_id: &str,
    ) -> ExecuteRequest {
        let mut env = HashMap::new();
        env.insert(
            "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT".to_string(),
            "1".to_string(),
        );

        ExecuteRequest {
            profile: None,
            cmd: String::new(),
            cwd: Some(cwd.display().to_string()),
            env: Some(env),
            pty: false,
            agent_id: "continue_world_worker_e2e".to_string(),
            budget: None,
            policy_snapshot: minimal_policy_snapshot(),
            shared_world: None,
            world_network: None,
            world_fs_mode: None,
            member_dispatch: Some(MemberDispatchRequestV1 {
                schema_version: 1,
                orchestration_session_id: "sess_dispatch".to_string(),
                participant_id: "ash_member".to_string(),
                orchestrator_participant_id: "orch_dispatch".to_string(),
                parent_participant_id: None,
                resumed_from_participant_id: None,
                backend_id: "cli:codex_world".to_string(),
                protocol: "substrate.agent.session".to_string(),
                run_id: run_id.to_string(),
                world_id: world_id.to_string(),
                world_generation,
                initial_prompt: Some("bootstrap retained worker".to_string()),
                resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path: binary_path.display().to_string(),
                },
            }),
        }
    }

    #[cfg(target_os = "linux")]
    fn sample_continue_submit_request_for_run(
        run_id: &str,
        world_id: &str,
        world_generation: u64,
    ) -> transport_api_types::MemberTurnSubmitRequestV1 {
        transport_api_types::MemberTurnSubmitRequestV1 {
            run_id: run_id.to_string(),
            world_id: world_id.to_string(),
            world_generation,
            ..sample_continue_submit_request()
        }
    }

    #[cfg(target_os = "linux")]
    fn persist_authoritative_continue_dispatch_state(
        store: &AgentRuntimeStateStore,
        workspace_root: &Path,
        world_id: &str,
        world_generation: u64,
    ) {
        let mut session = sample_session();
        session.workspace_root = workspace_root.display().to_string();
        session.shell_owner_pid = std::process::id();
        session.world_id = Some(world_id.to_string());
        session.world_generation = Some(world_generation);

        let mut orchestrator = sample_orchestrator_participant();
        orchestrator.internal.shell_owner_pid = std::process::id();
        orchestrator.internal.uaa_session_id = Some("uaa_orch_dispatch".to_string());
        orchestrator.internal.attached_client_present = true;
        orchestrator.internal.last_attached_at = Some(chrono::Utc::now());
        orchestrator.internal.resume_eligible = true;
        session.host_attach_contract = HostAttachContract::from_manifest_for_test(&orchestrator);

        let mut member = sample_member_participant();
        member.internal.shell_owner_pid = std::process::id();
        member.internal.uaa_session_id = Some("uaa_member_dispatch".to_string());
        member.handle.world_id = Some(world_id.to_string());
        member.handle.world_generation = Some(world_generation);

        store
            .persist_orchestration_session(&session)
            .expect("persist authoritative session");
        store
            .persist_participant(&orchestrator)
            .expect("persist authoritative orchestrator");
        store
            .persist_participant(&member)
            .expect("persist retained worker");
    }

    #[cfg(target_os = "linux")]
    fn persist_stale_continue_dispatch_state(
        store: &AgentRuntimeStateStore,
        workspace_root: &Path,
        world_id: &str,
        world_generation: u64,
    ) {
        let mut session = sample_session();
        session.workspace_root = workspace_root.display().to_string();
        session.shell_owner_pid = std::process::id();
        session.world_id = Some(world_id.to_string());
        session.world_generation = Some(world_generation);

        let mut orchestrator = sample_orchestrator_participant();
        orchestrator.internal.shell_owner_pid = std::process::id();
        orchestrator.internal.uaa_session_id = Some("uaa_orch_dispatch".to_string());
        orchestrator.internal.attached_client_present = true;
        orchestrator.internal.last_attached_at = Some(chrono::Utc::now());
        orchestrator.internal.resume_eligible = true;
        session.host_attach_contract = HostAttachContract::from_manifest_for_test(&orchestrator);

        let mut member = sample_member_participant();
        member.internal.shell_owner_pid = 999_999_999;
        member.internal.uaa_session_id = Some("uaa_member_dispatch".to_string());
        member.handle.world_id = Some(world_id.to_string());
        member.handle.world_generation = Some(world_generation);

        store
            .persist_orchestration_session(&session)
            .expect("persist authoritative session");
        store
            .persist_participant(&orchestrator)
            .expect("persist authoritative orchestrator");
        store
            .persist_participant(&member)
            .expect("persist stale retained worker");
    }

    #[cfg(target_os = "linux")]
    async fn dispatch_real_continue_world_worker_request(
        store: &AgentRuntimeStateStore,
        request_id: &str,
        idempotency_key: &str,
        world_id: &str,
        world_generation: u64,
    ) -> ContinueWorldWorkerOutcomeV1 {
        let mut request = sample_continue_world_dispatch_request();
        request.request_id = Some(request_id.to_string());
        request.idempotency_key = Some(idempotency_key.to_string());
        request.world_id = Some(world_id.to_string());
        request.world_generation = Some(world_generation);

        let prepared = prepare_orchestrator_world_dispatch(store, request)
            .expect("prepare continue dispatch request");
        let outcome = dispatch_prepared_orchestrator_world_request(prepared)
            .await
            .expect("dispatch prepared continue request");

        let WorldDispatchOutcomeV1::ContinueWorldWorker(outcome) = outcome else {
            panic!("expected continue_world_worker outcome envelope");
        };
        outcome
    }

    #[cfg(target_os = "linux")]
    async fn dispatch_real_inspect_world_worker_request(
        store: &AgentRuntimeStateStore,
    ) -> InspectWorldWorkerOutcomeV1 {
        let prepared =
            prepare_orchestrator_world_dispatch(store, sample_inspect_world_dispatch_request())
                .expect("prepare inspect dispatch request");
        let outcome = dispatch_prepared_orchestrator_world_request(prepared)
            .await
            .expect("dispatch prepared inspect request");

        let WorldDispatchOutcomeV1::InspectWorldWorker(outcome) = outcome else {
            panic!("expected inspect_world_worker outcome envelope");
        };
        outcome
    }

    #[cfg(target_os = "linux")]
    fn write_fake_continue_world_worker_runtime(temp: &Path) -> std::path::PathBuf {
        let path = temp.join("fake-continue-world-worker-runtime.sh");
        let state_file = temp.join("continue-world-worker.count");
        let body = format!(
            "#!/bin/sh\nSTATE_FILE='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nif [ \"$count\" -eq 1 ]; then\n  trap 'exit 0' INT TERM HUP QUIT\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-real\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  while :; do sleep 1; done\nfi\nif [ \"$count\" -eq 2 ]; then\n  printf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-real\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-2\"}}\\r\\n'\n  printf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-2\",\"item_id\":\"msg-2\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"reply from live runtime\"}}}}\\r\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-2\",\"last_item_id\":\"msg-2\"}}\\r\\n'\n  exit 0\nfi\nif [ \"$count\" -eq 3 ]; then\n  printf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-real\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-3\"}}\\r\\n'\n  printf '{{\"type\":\"item.started\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-3\",\"item_id\":\"cmd-3\",\"status\":\"in_progress\",\"item_type\":\"command_execution\",\"content\":{{\"command\":\"echo hi\"}}}}\\r\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-3\",\"last_item_id\":\"cmd-3\"}}\\r\\n'\n  exit 0\nfi\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-real\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-4\"}}\\r\\n'\nprintf '{{\"type\":\"turn.failed\",\"thread_id\":\"thread-real\",\"turn_id\":\"turn-4\",\"error\":{{\"message\":\"boom\"}}}}\\r\\n'\nexit 1\n",
            state_file.display()
        );
        fs::write(&path, body).expect("write fake continue runtime");
        let mut perms = fs::metadata(&path)
            .expect("fake continue runtime metadata")
            .permissions();
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("set fake continue runtime permissions");
        path
    }

    #[cfg(target_os = "linux")]
    async fn read_http_request(stream: &mut tokio::net::UnixStream) -> Option<(String, Vec<u8>)> {
        let mut buf = Vec::new();
        let mut header_end = None;
        let mut expected_len = None;

        for _ in 0..64 {
            let mut tmp = [0u8; 1024];
            let n =
                tokio::time::timeout(std::time::Duration::from_millis(250), stream.read(&mut tmp))
                    .await
                    .ok()?
                    .ok()?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);

            if header_end.is_none() {
                if let Some(pos) = buf.windows(4).position(|window| window == b"\r\n\r\n") {
                    header_end = Some(pos + 4);
                    let header = String::from_utf8_lossy(&buf[..pos + 4]).to_string();
                    expected_len = header.lines().find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        if key.eq_ignore_ascii_case("content-length") {
                            value.trim().parse::<usize>().ok()
                        } else {
                            None
                        }
                    });
                }
            }

            match (header_end, expected_len) {
                (Some(header_end), Some(len)) if buf.len() >= header_end + len => {
                    let header = String::from_utf8_lossy(&buf[..header_end]).to_string();
                    let body = buf[header_end..header_end + len].to_vec();
                    return Some((header, body));
                }
                (Some(header_end), None) => {
                    let header = String::from_utf8_lossy(&buf[..header_end]).to_string();
                    return Some((header, Vec::new()));
                }
                _ => {}
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    async fn write_http_json(stream: &mut tokio::net::UnixStream, status_line: &str, body: &str) {
        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.shutdown().await;
    }

    #[cfg(target_os = "linux")]
    async fn write_http_body(
        stream: &mut tokio::net::UnixStream,
        status_line: &str,
        content_type: &str,
        body: &[u8],
    ) {
        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.write_all(body).await;
        let _ = stream.shutdown().await;
    }

    #[cfg(target_os = "linux")]
    async fn write_http_stream_start(stream: &mut tokio::net::UnixStream) {
        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n";
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;
    }

    #[cfg(target_os = "linux")]
    async fn write_chunked_bytes(stream: &mut tokio::net::UnixStream, payload: &[u8]) {
        let header = format!("{:X}\r\n", payload.len());
        let _ = stream.write_all(header.as_bytes()).await;
        let _ = stream.write_all(payload).await;
        let _ = stream.write_all(b"\r\n").await;
        let _ = stream.flush().await;
    }

    #[cfg(target_os = "linux")]
    async fn write_chunked_frame(
        stream: &mut tokio::net::UnixStream,
        frame: &transport_api_types::ExecuteStreamFrame,
    ) {
        let mut payload = serde_json::to_vec(frame).expect("serialize stream frame");
        payload.push(b'\n');
        write_chunked_bytes(stream, &payload).await;
    }

    #[cfg(target_os = "linux")]
    async fn finish_chunked_stream(stream: &mut tokio::net::UnixStream) {
        let _ = stream.write_all(b"0\r\n\r\n").await;
        let _ = stream.flush().await;
        let _ = stream.shutdown().await;
    }

    #[cfg(target_os = "linux")]
    async fn next_stream_frame_value<B>(body: &mut B, buffer: &mut Vec<u8>) -> serde_json::Value
    where
        B: HttpBody<Data = hyper014::body::Bytes> + Unpin,
        B::Error: std::fmt::Debug + std::fmt::Display,
    {
        loop {
            if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buffer.drain(..=pos).collect();
                let payload = &line[..line.len() - 1];
                return serde_json::from_slice(payload).expect("valid stream frame json value");
            }

            let chunk = timeout(std::time::Duration::from_secs(5), body.data())
                .await
                .expect("timed out waiting for stream chunk")
                .expect("stream ended unexpectedly")
                .expect("stream chunk error");
            buffer.extend_from_slice(&chunk);
        }
    }

    #[cfg(target_os = "linux")]
    fn frame_start_span_id(frame: &serde_json::Value) -> Option<&str> {
        if frame.get("type")?.as_str() == Some("start") {
            return frame.get("span_id")?.as_str();
        }
        frame.get("Start")?.get("span_id")?.as_str()
    }

    #[cfg(target_os = "linux")]
    fn frame_event(frame: &serde_json::Value) -> Option<&serde_json::Value> {
        if frame.get("type").and_then(serde_json::Value::as_str) == Some("event") {
            return frame.get("event");
        }
        frame.get("Event")?.get("event")
    }

    #[cfg(target_os = "linux")]
    async fn next_registered_frame<B>(body: &mut B, buffer: &mut Vec<u8>) -> serde_json::Value
    where
        B: HttpBody<Data = hyper014::body::Bytes> + Unpin,
        B::Error: std::fmt::Debug + std::fmt::Display,
    {
        loop {
            let frame = next_stream_frame_value(body, buffer).await;
            if let Some(event) = frame_event(&frame) {
                if event.get("kind").and_then(serde_json::Value::as_str) == Some("registered") {
                    return frame;
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn continue_world_worker_submit_request_uses_authoritative_retained_identity() {
        let prepared = PreparedOrchestratorWorldDispatch {
            store: sample_state_store(),
            request: sample_continue_request(),
            session: sample_session(),
            caller_participant: sample_orchestrator_participant(),
            target_participant: Some(sample_member_participant()),
            live_retained_worker_count: 1,
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
            (
                sample_continue_stream_uaa_event(json!({
                    "type": "item.completed",
                    "thread_id": "thread-from-uaa",
                    "turn_id": "turn-1",
                    "item_id": "msg-1",
                    "status": "completed",
                    "item_type": "agent_message",
                    "content": {
                        "text": "reply from worker"
                    }
                })),
                "reply",
                false,
                "/uaa_event/content/text",
                Some("reply from worker"),
            ),
            (
                sample_continue_stream_uaa_event(json!({
                    "type": "item.started",
                    "thread_id": "thread-from-uaa",
                    "turn_id": "turn-1",
                    "item_id": "cmd-1",
                    "status": "in_progress",
                    "item_type": "command_execution",
                    "content": {
                        "command": "cargo test"
                    }
                })),
                "progress_update",
                false,
                "/uaa_event/content/command",
                Some("cargo test"),
            ),
            (
                sample_continue_stream_uaa_event(json!({
                    "type": "turn.completed",
                    "thread_id": "thread-from-uaa",
                    "turn_id": "turn-1",
                    "last_item_id": "msg-1"
                })),
                "result",
                false,
                "/uaa_event/type",
                Some("turn.completed"),
            ),
            (
                sample_continue_stream_uaa_event(json!({
                    "type": "turn.failed",
                    "thread_id": "thread-from-uaa",
                    "turn_id": "turn-1",
                    "message": "worker failed"
                })),
                "failure",
                false,
                "/uaa_event/message",
                Some("worker failed"),
            ),
            (
                sample_continue_stream_event(json!({
                    "event_class": "follow_up_question",
                    "payload": {
                        "message": "need clarification"
                    }
                })),
                "follow_up_question",
                true,
                "/message",
                Some("need clarification"),
            ),
            (
                sample_continue_stream_event(json!({
                    "event_class": "blocked",
                    "payload": {
                        "message": "blocked on input"
                    }
                })),
                "blocked",
                true,
                "/message",
                Some("blocked on input"),
            ),
        ];

        for (event, event_class, attention_required, payload_pointer, payload_value) in cases {
            let classified = classify_continue_world_worker_event(&submit, &event)
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
                    .pointer(payload_pointer)
                    .and_then(serde_json::Value::as_str),
                payload_value
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn continue_world_worker_dispatch_contract_respects_explicit_attention_for_in_scope_events() {
        let submit = sample_continue_submit_request();
        let classified = classify_continue_world_worker_event(
            &submit,
            &sample_continue_stream_uaa_event(json!({
                "type": "turn.completed",
                "thread_id": "thread-from-uaa",
                "turn_id": "turn-1",
                "attention_required": true,
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
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn continue_world_worker_fail_closed_stream_errors_cancel_live_submitted_turn() {
        let _env_guard = world_env_guard();
        let socket_home = tempdir().expect("socket tempdir");
        let socket_path = socket_home.path().join("world.sock");
        let cancel_requests = Arc::new(Mutex::new(Vec::<ExecuteCancelRequestV1>::new()));
        let cancel_requests_for_server = cancel_requests.clone();

        let listener = UnixListener::bind(&socket_path).expect("bind stub world socket");
        let server = tokio::spawn(async move {
            while let Ok((mut stream, _addr)) = listener.accept().await {
                let Some((header, body)) = read_http_request(&mut stream).await else {
                    continue;
                };
                let first_line = header.lines().next().unwrap_or("");

                if first_line.starts_with("GET /v1/capabilities ") {
                    write_http_json(
                        &mut stream,
                        "200 OK",
                        r#"{"schema_version":1,"policy_snapshot_v1_supported":true}"#,
                    )
                    .await;
                    continue;
                }

                if first_line.starts_with("POST /v1/member_turn/stream ") {
                    let _: transport_api_types::MemberTurnSubmitRequestV1 =
                        serde_json::from_slice(&body).expect("member turn submit request");
                    write_http_stream_start(&mut stream).await;
                    write_chunked_frame(
                        &mut stream,
                        &transport_api_types::ExecuteStreamFrame::Start {
                            span_id: "member-turn-span".to_string(),
                        },
                    )
                    .await;
                    write_chunked_frame(
                        &mut stream,
                        &transport_api_types::ExecuteStreamFrame::Event {
                            event: sample_continue_stream_event(json!({
                                "event_class": "approval_request",
                                "payload": {
                                    "message": "requires approval"
                                }
                            })),
                        },
                    )
                    .await;
                    finish_chunked_stream(&mut stream).await;
                    continue;
                }

                if first_line.starts_with("POST /v1/execute/cancel ") {
                    let parsed: ExecuteCancelRequestV1 =
                        serde_json::from_slice(&body).expect("execute cancel request");
                    cancel_requests_for_server
                        .lock()
                        .expect("cancel request mutex poisoned")
                        .push(parsed);
                    write_http_json(
                        &mut stream,
                        "200 OK",
                        r#"{"schema_version":1,"delivered":true}"#,
                    )
                    .await;
                    break;
                }

                write_http_json(&mut stream, "404 Not Found", r#"{"error":"not_found"}"#).await;
            }
        });

        let previous_socket = std::env::var("SUBSTRATE_WORLD_SOCKET").ok();
        std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket_path);

        let err =
            match execute_continue_world_worker_stream(&sample_continue_submit_request()).await {
                Ok(_) => panic!("deferred worker event classes must fail closed"),
                Err(err) => err,
            };
        assert!(
            err.to_string().contains("unsupported_worker_event_class"),
            "unexpected continue stream error: {err}"
        );

        let recorded = cancel_requests
            .lock()
            .expect("cancel request mutex poisoned")
            .clone();
        assert_eq!(
            recorded.len(),
            1,
            "expected one cancel request: {recorded:?}"
        );
        assert_eq!(recorded[0].span_id, "member-turn-span");
        assert_eq!(recorded[0].sig, "INT");

        if let Some(previous_socket) = previous_socket {
            std::env::set_var("SUBSTRATE_WORLD_SOCKET", previous_socket);
        } else {
            std::env::remove_var("SUBSTRATE_WORLD_SOCKET");
        }

        server.await.expect("stub world server task");
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn dispatch_contract_prepare_orchestrator_world_dispatch_defers_continue_target_resolution_until_after_steering(
    ) {
        let substrate_home = tempdir().expect("substrate home tempdir");
        let _substrate_home_guard = EnvVarGuard::set_path("SUBSTRATE_HOME", substrate_home.path());
        let workspace_root = tempdir().expect("workspace root tempdir");
        let store = AgentRuntimeStateStore::new().expect("state store");
        persist_stale_continue_dispatch_state(&store, workspace_root.path(), "world-17", 2);

        let prepared =
            prepare_orchestrator_world_dispatch(&store, sample_continue_world_dispatch_request())
                .expect("prepare should only resolve authoritative caller truth");

        assert_eq!(
            prepared.caller_participant.participant_id(),
            "orch_dispatch"
        );
        assert!(
            prepared.target_participant.is_none(),
            "prepare must not resolve retained continue targets before steering policy runs"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn dispatch_contract_continue_target_resolution_reuses_prepared_authoritative_store() {
        let authoritative_home = tempdir().expect("authoritative substrate home tempdir");
        let _authoritative_home_guard =
            EnvVarGuard::set_path("SUBSTRATE_HOME", authoritative_home.path());
        let workspace_root = tempdir().expect("workspace root tempdir");
        let store = AgentRuntimeStateStore::new().expect("authoritative state store");
        persist_authoritative_continue_dispatch_state(&store, workspace_root.path(), "world-17", 2);

        let prepared =
            prepare_orchestrator_world_dispatch(&store, sample_continue_world_dispatch_request())
                .expect("prepare should capture authoritative caller truth");

        let ambient_home = tempdir().expect("ambient substrate home tempdir");
        let _ambient_home_guard = EnvVarGuard::set_path("SUBSTRATE_HOME", ambient_home.path());

        let resolved = resolve_continue_world_dispatch_target_for_routing(prepared)
            .expect("continue target resolution should reuse the prepared authoritative store");

        assert_eq!(resolved.session.orchestration_session_id, "sess_dispatch");
        assert_eq!(
            resolved.caller_participant.participant_id(),
            "orch_dispatch"
        );
        assert_eq!(
            resolved
                .target_participant
                .as_ref()
                .expect("resolved retained target")
                .participant_id(),
            "ash_member"
        );
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn dispatch_contract_continue_world_worker_policy_denials_precede_stale_target_resolution(
    ) {
        struct DenialCase<'a> {
            enabled: bool,
            allowed_backends: &'a [&'a str],
            allowed_actions: &'a [&'a str],
            allowed_modes: &'a [&'a str],
            expected_denial: &'a str,
        }

        let cases = [
            DenialCase {
                enabled: false,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["continue_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "world_dispatch_disabled:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["spawn_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "action_not_allowed:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["continue_world_worker"],
                allowed_modes: &["ephemeral"],
                expected_denial: "mode_not_allowed:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:other_world"],
                allowed_actions: &["continue_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "backend_not_allowed:",
            },
        ];

        for case in cases {
            let substrate_home = tempdir().expect("substrate home tempdir");
            let _substrate_home_guard =
                EnvVarGuard::set_path("SUBSTRATE_HOME", substrate_home.path());
            write_world_dispatch_policy(
                substrate_home.path(),
                case.enabled,
                case.allowed_backends,
                case.allowed_actions,
                case.allowed_modes,
            );

            let workspace_root = tempdir().expect("workspace root tempdir");
            let store = AgentRuntimeStateStore::new().expect("state store");
            persist_stale_continue_dispatch_state(&store, workspace_root.path(), "world-17", 2);

            let err = dispatch_orchestrator_world_request(
                &store,
                sample_continue_world_dispatch_request(),
            )
            .await
            .expect_err("steering denial must fail closed before stale target routing");
            let message = err.to_string();

            assert!(
                message.contains(case.expected_denial),
                "expected {} in {message}",
                case.expected_denial
            );
            assert!(
                !message.contains("stale_linkage:"),
                "steering denial must not leak retained-worker lifecycle truth first: {message}"
            );
            assert!(
                !message.contains(
                    "world_binding_mismatch: orchestration session sess_dispatch retained worker"
                ),
                "steering denial must not leak retained-worker topology drift first: {message}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn dispatch_contract_inspect_world_worker_policy_denials_precede_target_resolution() {
        struct DenialCase<'a> {
            enabled: bool,
            allowed_backends: &'a [&'a str],
            allowed_actions: &'a [&'a str],
            allowed_modes: &'a [&'a str],
            expected_denial: &'a str,
        }

        let cases = [
            DenialCase {
                enabled: false,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["inspect_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "world_dispatch_disabled:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["continue_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "action_not_allowed:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["inspect_world_worker"],
                allowed_modes: &["ephemeral"],
                expected_denial: "mode_not_allowed:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:other_world"],
                allowed_actions: &["inspect_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "backend_not_allowed:",
            },
        ];

        for case in cases {
            let substrate_home = tempdir().expect("substrate home tempdir");
            let _substrate_home_guard =
                EnvVarGuard::set_path("SUBSTRATE_HOME", substrate_home.path());
            write_world_dispatch_policy(
                substrate_home.path(),
                case.enabled,
                case.allowed_backends,
                case.allowed_actions,
                case.allowed_modes,
            );

            let workspace_root = tempdir().expect("workspace root tempdir");
            let store = AgentRuntimeStateStore::new().expect("state store");
            persist_authoritative_continue_dispatch_state(
                &store,
                workspace_root.path(),
                "world-17",
                2,
            );

            let mut request = sample_inspect_world_dispatch_request();
            request.target_participant_id = Some("ash_missing".to_string());
            let err = dispatch_orchestrator_world_request(&store, request)
                .await
                .expect_err("steering denial must fail closed before inspect target resolution");
            let message = err.to_string();

            assert!(
                message.contains(case.expected_denial),
                "expected {} in {message}",
                case.expected_denial
            );
            assert!(
                !message.contains("target_not_in_session:"),
                "steering denial must not leak inspect target resolution truth first: {message}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn dispatch_contract_stop_world_worker_policy_denials_precede_target_resolution() {
        struct DenialCase<'a> {
            enabled: bool,
            allowed_backends: &'a [&'a str],
            allowed_actions: &'a [&'a str],
            allowed_modes: &'a [&'a str],
            expected_denial: &'a str,
        }

        let cases = [
            DenialCase {
                enabled: false,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["stop_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "world_dispatch_disabled:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["inspect_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "action_not_allowed:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:codex_world"],
                allowed_actions: &["stop_world_worker"],
                allowed_modes: &["ephemeral"],
                expected_denial: "mode_not_allowed:",
            },
            DenialCase {
                enabled: true,
                allowed_backends: &["cli:other_world"],
                allowed_actions: &["stop_world_worker"],
                allowed_modes: &["retained"],
                expected_denial: "backend_not_allowed:",
            },
        ];

        for case in cases {
            let substrate_home = tempdir().expect("substrate home tempdir");
            let _substrate_home_guard =
                EnvVarGuard::set_path("SUBSTRATE_HOME", substrate_home.path());
            write_world_dispatch_policy(
                substrate_home.path(),
                case.enabled,
                case.allowed_backends,
                case.allowed_actions,
                case.allowed_modes,
            );

            let workspace_root = tempdir().expect("workspace root tempdir");
            let store = AgentRuntimeStateStore::new().expect("state store");
            persist_authoritative_continue_dispatch_state(
                &store,
                workspace_root.path(),
                "world-17",
                2,
            );

            let mut request = sample_stop_world_dispatch_request();
            request.target_participant_id = Some("ash_missing".to_string());
            let err = dispatch_orchestrator_world_request(&store, request)
                .await
                .expect_err("steering denial must fail closed before stop target resolution");
            let message = err.to_string();

            assert!(
                message.contains(case.expected_denial),
                "expected {} in {message}",
                case.expected_denial
            );
            assert!(
                !message.contains("target_not_in_session:"),
                "steering denial must not leak stop target resolution truth first: {message}"
            );
            assert!(
                !message.contains("target_already_terminal:"),
                "steering denial must not leak stop terminal state truth first: {message}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn dispatch_contract_inspect_world_worker_returns_authoritative_snapshot_without_mutation(
    ) {
        let substrate_home = tempdir().expect("substrate home tempdir");
        let _substrate_home_guard = EnvVarGuard::set_path("SUBSTRATE_HOME", substrate_home.path());
        write_allowed_world_dispatch_policy(
            substrate_home.path(),
            "cli:codex_world",
            &["inspect_world_worker"],
            &["retained"],
        );

        let workspace_root = tempdir().expect("workspace root tempdir");
        let store = AgentRuntimeStateStore::new().expect("state store");
        persist_authoritative_continue_dispatch_state(&store, workspace_root.path(), "world-17", 2);

        let session_before = store
            .load_session("sess_dispatch")
            .expect("load session before inspect")
            .expect("authoritative session before inspect");
        let participant_before = store
            .load_participant("ash_member")
            .expect("load participant before inspect")
            .expect("retained participant before inspect");

        let outcome = dispatch_real_inspect_world_worker_request(&store).await;

        assert_eq!(outcome.request_id, "req_inspect");
        assert_eq!(outcome.orchestration_session_id, "sess_dispatch");
        assert_eq!(outcome.action, WorldDispatchActionV1::InspectWorldWorker);
        assert_eq!(outcome.mode, WorldDispatchModeV1::Retained);
        assert_eq!(outcome.orchestrator_participant_id, "orch_dispatch");
        assert_eq!(outcome.target_participant_id, "ash_member");
        assert_eq!(outcome.target_backend_id, "cli:codex_world");
        assert_eq!(outcome.world_id, "world-17");
        assert_eq!(outcome.world_generation, 2);
        assert_eq!(
            outcome.snapshot.participant_state,
            AgentRuntimeSessionState::Running
        );
        assert_eq!(
            outcome.snapshot.session_state,
            OrchestrationSessionState::Active
        );
        assert_eq!(
            outcome.snapshot.session_posture,
            OrchestrationSessionPosture::ActiveAttached
        );
        assert!(outcome.snapshot.authoritative_live);
        assert!(!outcome.snapshot.attention_required);
        assert_eq!(outcome.snapshot.parent_participant_id, None);
        assert_eq!(outcome.snapshot.resumed_from_participant_id, None);
        assert!(
            outcome
                .summary
                .contains("without invoking world-side execution transport"),
            "summary should stay explicit about inspect's snapshot-only seam: {}",
            outcome.summary
        );

        let session_after = store
            .load_session("sess_dispatch")
            .expect("load session after inspect")
            .expect("authoritative session after inspect");
        let participant_after = store
            .load_participant("ash_member")
            .expect("load participant after inspect")
            .expect("retained participant after inspect");

        assert_eq!(session_after.session, session_before.session);
        assert_eq!(session_after.participants, session_before.participants);
        assert_eq!(participant_after, participant_before);
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn dispatch_contract_stop_world_worker_returns_typed_closeout_after_authoritative_stop()
    {
        let substrate_home = tempdir().expect("substrate home tempdir");
        let _substrate_home_guard = EnvVarGuard::set_path("SUBSTRATE_HOME", substrate_home.path());
        write_allowed_world_dispatch_policy(
            substrate_home.path(),
            "cli:codex_world",
            &["stop_world_worker"],
            &["retained"],
        );

        let workspace_root = tempdir().expect("workspace root tempdir");
        let store = AgentRuntimeStateStore::new().expect("state store");
        persist_authoritative_continue_dispatch_state(&store, workspace_root.path(), "world-17", 2);

        let (stop_tx, mut stop_rx) =
            crate::execution::agent_runtime::control::private_stop_request_channel();
        let mut stop_transport = crate::execution::agent_runtime::control::register_private_stop_transport(
            &store,
            "sess_dispatch",
            "ash_member",
            stop_tx,
        )
        .await
        .expect("register private stop transport");
        let store_for_task = store.clone();
        let stop_owner = tokio::spawn(async move {
            let request = tokio::time::timeout(Duration::from_secs(3), stop_rx.recv())
                .await
                .expect("timed out waiting for private stop request")
                .expect("private stop request");
            let mut session = store_for_task
                .load_orchestration_session("sess_dispatch")
                .expect("load orchestration session for stop closeout")
                .expect("authoritative orchestration session for stop closeout");
            let mut participant = store_for_task
                .load_participant("ash_member")
                .expect("load retained participant for stop closeout")
                .expect("authoritative retained participant for stop closeout");
            participant.transition_state(AgentRuntimeSessionState::Stopped);
            participant.mark_terminal_state("worker stopped");
            participant.touch_heartbeat();
            session.touch_active();
            crate::execution::agent_runtime::control::persist_runtime_snapshots(
                &store_for_task,
                &session,
                &participant,
            )
            .expect("persist retained stop closeout");
            let _ = request
                .response_tx
                .send(crate::execution::agent_runtime::control::PrivateStopOutcome::Accepted);
        });

        let prepared = prepare_orchestrator_world_dispatch(&store, sample_stop_world_dispatch_request())
            .expect("prepare stop dispatch request");
        let outcome = dispatch_prepared_orchestrator_world_request(prepared)
            .await
            .expect("dispatch prepared stop request");

        let WorldDispatchOutcomeV1::StopWorldWorker(outcome) = outcome else {
            panic!("expected stop_world_worker outcome envelope");
        };
        assert_eq!(outcome.request_id, "req_stop");
        assert_eq!(outcome.orchestration_session_id, "sess_dispatch");
        assert_eq!(outcome.action, WorldDispatchActionV1::StopWorldWorker);
        assert_eq!(outcome.mode, WorldDispatchModeV1::Retained);
        assert_eq!(outcome.orchestrator_participant_id, "orch_dispatch");
        assert_eq!(outcome.target_participant_id, "ash_member");
        assert_eq!(outcome.target_backend_id, "cli:codex_world");
        assert_eq!(outcome.world_id, "world-17");
        assert_eq!(outcome.world_generation, 2);
        assert_eq!(
            outcome.closeout.participant_state,
            AgentRuntimeSessionState::Stopped
        );
        assert_eq!(outcome.closeout.session_state, OrchestrationSessionState::Active);
        assert!(
            outcome
                .summary
                .contains("existing private owner stop surface"),
            "summary should stay explicit about stop reusing the private owner seam: {}",
            outcome.summary
        );

        let participant_after = store
            .load_participant("ash_member")
            .expect("load retained participant after stop")
            .expect("retained participant after stop");
        assert_eq!(participant_after.handle.state, AgentRuntimeSessionState::Stopped);
        assert_eq!(
            participant_after.internal.termination_reason.as_deref(),
            Some("worker stopped")
        );

        stop_owner.await.expect("stop owner task should join");
        stop_transport.close().await;
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn continue_world_worker_classifies_real_retained_member_turn_streams() {
        let _env_guard = world_env_guard();
        let service = match WorldService::new() {
            Ok(service) => service,
            Err(err) => {
                eprintln!("skipping continue_world_worker e2e test: service init failed: {err}");
                return;
            }
        };

        let temp = tempdir().expect("tempdir");
        let runtime_path = write_fake_continue_world_worker_runtime(temp.path());
        let world_spec = WorldSpec {
            reuse_session: true,
            reuse_mode: WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
                orchestration_session_id: "sess_dispatch".to_string(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
            isolate_network: false,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: Vec::new(),
            project_dir: temp.path().to_path_buf(),
            always_isolate: true,
            fs_mode: substrate_common::WorldFsMode::Writable,
        };
        let world = match service.ensure_session_world(&world_spec) {
            Ok(world) => world,
            Err(err) => {
                eprintln!(
                    "skipping continue_world_worker e2e test: failed to ensure shared world: {err}"
                );
                return;
            }
        };
        let Some(binding) = world.shared_binding.clone() else {
            eprintln!("skipping continue_world_worker e2e test: shared world binding missing");
            return;
        };

        let launch = service
            .execute_stream(make_member_dispatch_execute_request(
                temp.path(),
                &runtime_path,
                &binding.world_id,
                binding.world_generation,
                "run-bootstrap",
            ))
            .await
            .expect("member bootstrap should succeed");
        let mut launch_body = launch.into_body();
        let mut launch_buffer = Vec::new();
        let launch_start = next_stream_frame_value(&mut launch_body, &mut launch_buffer).await;
        let launch_span_id = frame_start_span_id(&launch_start)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| panic!("expected start frame, got {launch_start:?}"));
        let registered = next_registered_frame(&mut launch_body, &mut launch_buffer).await;
        assert_eq!(
            frame_event(&registered)
                .and_then(|event| event.get("participant_id"))
                .and_then(serde_json::Value::as_str),
            Some("ash_member")
        );

        let socket_home = tempdir().expect("socket tempdir");
        let socket_path = socket_home.path().join("world.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind world socket");
        let service_for_server = service.clone();
        let server = tokio::spawn(async move {
            while let Ok((mut stream, _addr)) = listener.accept().await {
                let Some((header, body)) = read_http_request(&mut stream).await else {
                    continue;
                };
                let first_line = header.lines().next().unwrap_or("");

                if first_line.starts_with("GET /v1/capabilities ") {
                    write_http_json(
                        &mut stream,
                        "200 OK",
                        r#"{"schema_version":1,"policy_snapshot_v1_supported":true}"#,
                    )
                    .await;
                    continue;
                }

                if first_line.starts_with("POST /v1/member_turn/stream ") {
                    let parsed: transport_api_types::MemberTurnSubmitRequestV1 =
                        serde_json::from_slice(&body).expect("member turn submit request");
                    let response = service_for_server
                        .submit_member_turn_stream(parsed)
                        .await
                        .expect("world-service member turn stream");
                    let bytes = to_bytes(response.into_body())
                        .await
                        .expect("collect member turn response bytes");
                    write_http_body(
                        &mut stream,
                        "200 OK",
                        "application/x-ndjson",
                        bytes.as_ref(),
                    )
                    .await;
                    continue;
                }

                if first_line.starts_with("POST /v1/execute/cancel ") {
                    let parsed: ExecuteCancelRequestV1 =
                        serde_json::from_slice(&body).expect("execute cancel request");
                    let response = service_for_server
                        .execute_cancel(parsed)
                        .await
                        .expect("world-service execute cancel");
                    let body = serde_json::to_vec(&response).expect("serialize cancel response");
                    write_http_body(&mut stream, "200 OK", "application/json", &body).await;
                    continue;
                }

                write_http_json(&mut stream, "404 Not Found", r#"{"error":"not_found"}"#).await;
            }
        });

        let previous_socket = std::env::var("SUBSTRATE_WORLD_SOCKET").ok();
        std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket_path);

        let reply = execute_continue_world_worker_stream(&sample_continue_submit_request_for_run(
            "run-reply",
            &binding.world_id,
            binding.world_generation,
        ))
        .await
        .expect("reply turn should succeed");
        assert_eq!(reply.exit_code, 0);
        assert_eq!(reply.surfaced_thread_id.as_deref(), Some("thread-real"));
        assert_eq!(
            reply
                .surfaced_worker_event
                .as_ref()
                .map(|event| event.event_class),
            Some(ContinueWorldWorkerEventClassV1::Reply)
        );
        assert_eq!(
            reply
                .surfaced_worker_event
                .as_ref()
                .and_then(|event| event.payload.get("message"))
                .and_then(serde_json::Value::as_str),
            Some("reply from live runtime")
        );

        let progress =
            execute_continue_world_worker_stream(&sample_continue_submit_request_for_run(
                "run-progress",
                &binding.world_id,
                binding.world_generation,
            ))
            .await
            .expect("progress turn should succeed");
        assert_eq!(progress.exit_code, 0);
        assert_eq!(progress.surfaced_thread_id.as_deref(), Some("thread-real"));
        assert_eq!(
            progress
                .surfaced_worker_event
                .as_ref()
                .map(|event| event.event_class),
            Some(ContinueWorldWorkerEventClassV1::ProgressUpdate)
        );
        assert_eq!(
            progress
                .surfaced_worker_event
                .as_ref()
                .and_then(|event| event.payload.pointer("/uaa_event/tool/kind"))
                .and_then(serde_json::Value::as_str),
            Some("command_execution")
        );

        let failure =
            execute_continue_world_worker_stream(&sample_continue_submit_request_for_run(
                "run-failure",
                &binding.world_id,
                binding.world_generation,
            ))
            .await
            .expect("failure turn should still return typed outcome");
        assert_eq!(failure.exit_code, 1);
        assert_eq!(failure.surfaced_thread_id.as_deref(), Some("thread-real"));
        assert_eq!(
            failure
                .surfaced_worker_event
                .as_ref()
                .map(|event| event.event_class),
            Some(ContinueWorldWorkerEventClassV1::Failure)
        );
        assert!(
            failure
                .surfaced_worker_event
                .as_ref()
                .and_then(|event| event.payload.get("message"))
                .and_then(serde_json::Value::as_str)
                .is_some_and(|message| message.contains("codex exited non-zero")),
            "failure payload should preserve the runtime non-zero exit alert"
        );

        let delivered = service
            .execute_cancel(ExecuteCancelRequestV1 {
                span_id: launch_span_id,
                sig: "INT".to_string(),
            })
            .await
            .expect("bootstrap cancel should succeed");
        assert!(
            delivered.delivered,
            "expected retained bootstrap cancel delivery"
        );

        if let Some(previous_socket) = previous_socket {
            std::env::set_var("SUBSTRATE_WORLD_SOCKET", previous_socket);
        } else {
            std::env::remove_var("SUBSTRATE_WORLD_SOCKET");
        }

        server.abort();
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial]
    async fn continue_world_worker_dispatch_returns_real_typed_internal_outcome() {
        let _env_guard = world_env_guard();
        let service = match WorldService::new() {
            Ok(service) => service,
            Err(err) => {
                eprintln!(
                    "skipping continue_world_worker dispatch test: service init failed: {err}"
                );
                return;
            }
        };

        let temp = tempdir().expect("tempdir");
        let runtime_path = write_fake_continue_world_worker_runtime(temp.path());
        let world_spec = WorldSpec {
            reuse_session: true,
            reuse_mode: WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
                orchestration_session_id: "sess_dispatch".to_string(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
            isolate_network: false,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: Vec::new(),
            project_dir: temp.path().to_path_buf(),
            always_isolate: true,
            fs_mode: substrate_common::WorldFsMode::Writable,
        };
        let world = match service.ensure_session_world(&world_spec) {
            Ok(world) => world,
            Err(err) => {
                eprintln!(
                    "skipping continue_world_worker dispatch test: failed to ensure shared world: {err}"
                );
                return;
            }
        };
        let Some(binding) = world.shared_binding.clone() else {
            eprintln!("skipping continue_world_worker dispatch test: shared world binding missing");
            return;
        };

        let launch = service
            .execute_stream(make_member_dispatch_execute_request(
                temp.path(),
                &runtime_path,
                &binding.world_id,
                binding.world_generation,
                "run-bootstrap",
            ))
            .await
            .expect("member bootstrap should succeed");
        let mut launch_body = launch.into_body();
        let mut launch_buffer = Vec::new();
        let launch_start = next_stream_frame_value(&mut launch_body, &mut launch_buffer).await;
        let launch_span_id = frame_start_span_id(&launch_start)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| panic!("expected start frame, got {launch_start:?}"));
        let registered = next_registered_frame(&mut launch_body, &mut launch_buffer).await;
        assert_eq!(
            frame_event(&registered)
                .and_then(|event| event.get("participant_id"))
                .and_then(serde_json::Value::as_str),
            Some("ash_member")
        );

        let socket_home = tempdir().expect("socket tempdir");
        let socket_path = socket_home.path().join("world.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind world socket");
        let service_for_server = service.clone();
        let server = tokio::spawn(async move {
            while let Ok((mut stream, _addr)) = listener.accept().await {
                let Some((header, body)) = read_http_request(&mut stream).await else {
                    continue;
                };
                let first_line = header.lines().next().unwrap_or("");

                if first_line.starts_with("GET /v1/capabilities ") {
                    write_http_json(
                        &mut stream,
                        "200 OK",
                        r#"{"schema_version":1,"policy_snapshot_v1_supported":true}"#,
                    )
                    .await;
                    continue;
                }

                if first_line.starts_with("POST /v1/member_turn/stream ") {
                    let parsed: transport_api_types::MemberTurnSubmitRequestV1 =
                        serde_json::from_slice(&body).expect("member turn submit request");
                    let response = service_for_server
                        .submit_member_turn_stream(parsed)
                        .await
                        .expect("world-service member turn stream");
                    let bytes = to_bytes(response.into_body())
                        .await
                        .expect("collect member turn response bytes");
                    write_http_body(
                        &mut stream,
                        "200 OK",
                        "application/x-ndjson",
                        bytes.as_ref(),
                    )
                    .await;
                    continue;
                }

                if first_line.starts_with("POST /v1/execute/cancel ") {
                    let parsed: ExecuteCancelRequestV1 =
                        serde_json::from_slice(&body).expect("execute cancel request");
                    let response = service_for_server
                        .execute_cancel(parsed)
                        .await
                        .expect("world-service execute cancel");
                    let body = serde_json::to_vec(&response).expect("serialize cancel response");
                    write_http_body(&mut stream, "200 OK", "application/json", &body).await;
                    continue;
                }

                write_http_json(&mut stream, "404 Not Found", r#"{"error":"not_found"}"#).await;
            }
        });

        let _socket_guard = EnvVarGuard::set_path("SUBSTRATE_WORLD_SOCKET", &socket_path);
        let substrate_home = tempdir().expect("substrate home tempdir");
        let _substrate_home_guard = EnvVarGuard::set_path("SUBSTRATE_HOME", substrate_home.path());
        write_allowed_world_dispatch_policy(
            substrate_home.path(),
            "cli:codex_world",
            &["continue_world_worker"],
            &["retained"],
        );
        let store = AgentRuntimeStateStore::new().expect("state store");
        persist_authoritative_continue_dispatch_state(
            &store,
            temp.path(),
            &binding.world_id,
            binding.world_generation,
        );

        let reply_outcome = dispatch_real_continue_world_worker_request(
            &store,
            "req_continue_reply",
            "idem_continue_reply",
            &binding.world_id,
            binding.world_generation,
        )
        .await;
        assert_eq!(reply_outcome.thread_id.as_deref(), Some("thread-real"));

        let outcome = dispatch_real_continue_world_worker_request(
            &store,
            "req_continue_progress",
            "idem_continue_progress",
            &binding.world_id,
            binding.world_generation,
        )
        .await;
        assert_eq!(outcome.thread_id.as_deref(), Some("thread-real"));
        let worker_event = outcome
            .worker_event
            .expect("continue dispatch should surface worker event");
        assert_eq!(
            worker_event.event_class,
            ContinueWorldWorkerEventClassV1::ProgressUpdate
        );
        assert_eq!(
            worker_event
                .payload
                .pointer("/uaa_event/tool/kind")
                .and_then(serde_json::Value::as_str),
            Some("command_execution")
        );

        let delivered = service
            .execute_cancel(ExecuteCancelRequestV1 {
                span_id: launch_span_id,
                sig: "INT".to_string(),
            })
            .await
            .expect("bootstrap cancel should succeed");
        assert!(
            delivered.delivered,
            "expected retained bootstrap cancel delivery"
        );

        server.abort();
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

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_rejects_disabled_world_dispatch() {
        let err = enforce_world_dispatch_steering_policy(
            &PreparedOrchestratorWorldDispatch {
                store: sample_state_store(),
                request: sample_request(),
                session: sample_session(),
                caller_participant: sample_orchestrator_participant(),
                target_participant: None,
                live_retained_worker_count: 0,
            },
            &Policy::default(),
        )
        .expect_err("disabled steering must fail closed");

        assert_eq!(
            err.to_string(),
            "world_dispatch_disabled: host-to-world steering is disabled by effective policy"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_rejects_action_not_allowed() {
        let mut policy = sample_world_dispatch_policy();
        policy.agents_world_dispatch_allowed_actions = vec!["spawn_world_worker".to_string()];

        let err = enforce_world_dispatch_steering_policy(
            &PreparedOrchestratorWorldDispatch {
                store: sample_state_store(),
                request: sample_request(),
                session: sample_session(),
                caller_participant: sample_orchestrator_participant(),
                target_participant: None,
                live_retained_worker_count: 0,
            },
            &policy,
        )
        .expect_err("disallowed action must fail closed");

        assert_eq!(
            err.to_string(),
            "action_not_allowed: effective policy does not allow run_world_task"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_rejects_mode_not_allowed() {
        let mut policy = sample_world_dispatch_policy();
        policy.agents_world_dispatch_allowed_modes = vec!["retained".to_string()];

        let err = enforce_world_dispatch_steering_policy(
            &PreparedOrchestratorWorldDispatch {
                store: sample_state_store(),
                request: sample_request(),
                session: sample_session(),
                caller_participant: sample_orchestrator_participant(),
                target_participant: None,
                live_retained_worker_count: 0,
            },
            &policy,
        )
        .expect_err("disallowed mode must fail closed");

        assert_eq!(
            err.to_string(),
            "mode_not_allowed: effective policy does not allow ephemeral"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_rejects_backend_not_allowed() {
        let mut policy = sample_world_dispatch_policy();
        policy.agents_world_dispatch_allowed_backends = vec!["cli:other_world".to_string()];

        let err = enforce_world_dispatch_steering_policy(
            &PreparedOrchestratorWorldDispatch {
                store: sample_state_store(),
                request: sample_request(),
                session: sample_session(),
                caller_participant: sample_orchestrator_participant(),
                target_participant: None,
                live_retained_worker_count: 0,
            },
            &policy,
        )
        .expect_err("disallowed backend must fail closed");

        assert_eq!(
            err.to_string(),
            "backend_not_allowed: effective policy does not allow backend cli:codex_world"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_rejects_cross_session_boundary() {
        let mut caller = sample_orchestrator_participant();
        caller.handle.orchestration_session_id = "sess_other".to_string();

        let err = enforce_world_dispatch_steering_policy(
            &PreparedOrchestratorWorldDispatch {
                store: sample_state_store(),
                request: sample_request(),
                session: sample_session(),
                caller_participant: caller,
                target_participant: None,
                live_retained_worker_count: 0,
            },
            &sample_world_dispatch_policy(),
        )
        .expect_err("cross-session steering must fail closed");

        assert_eq!(
            err.to_string(),
            "cross_session_steering_denied: request must stay within authoritative orchestration session sess_dispatch"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_allows_cross_session_boundary_when_policy_relaxed() {
        let mut caller = sample_orchestrator_participant();
        caller.handle.orchestration_session_id = "sess_other".to_string();
        let mut policy = sample_world_dispatch_policy();
        policy.agents_world_dispatch_same_session_only = false;

        enforce_world_dispatch_steering_policy(
            &PreparedOrchestratorWorldDispatch {
                store: sample_state_store(),
                request: sample_request(),
                session: sample_session(),
                caller_participant: caller,
                target_participant: None,
                live_retained_worker_count: 0,
            },
            &policy,
        )
        .expect("relaxed same-session policy should not reject cross-session prepared truth");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_rejects_cross_world_binding_boundary() {
        let mut request = sample_continue_request();
        request.world_generation = 3;

        let err = enforce_world_dispatch_steering_policy(
            &PreparedOrchestratorWorldDispatch {
                store: sample_state_store(),
                request,
                session: sample_session(),
                caller_participant: sample_orchestrator_participant(),
                target_participant: Some(sample_member_participant()),
                live_retained_worker_count: 1,
            },
            &sample_world_dispatch_policy(),
        )
        .expect_err("cross-world steering must fail closed");

        assert_eq!(
            err.to_string(),
            "cross_world_binding_steering_denied: world_binding_mismatch: orchestration session sess_dispatch authoritative world_generation is 2 not 3"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_allows_cross_world_binding_when_policy_relaxed() {
        let mut request = sample_continue_request();
        request.world_generation = 3;
        let mut policy = sample_world_dispatch_policy();
        policy.agents_world_dispatch_same_world_binding_only = false;

        enforce_world_dispatch_steering_policy(
            &PreparedOrchestratorWorldDispatch {
                store: sample_state_store(),
                request,
                session: sample_session(),
                caller_participant: sample_orchestrator_participant(),
                target_participant: Some(sample_member_participant()),
                live_retained_worker_count: 1,
            },
            &policy,
        )
        .expect("relaxed world-binding policy should not reject drifted prepared truth");
    }

    #[test]
    fn dispatch_contract_steering_policy_denial_helper_formats_packet34_bucket_and_detail() {
        let err = steering_policy_denial(
            WorldDispatchSteeringDenialV1::CrossSessionSteeringDenied,
            "request tried to steer outside the authoritative orchestration session",
        );

        assert_eq!(
            err.to_string(),
            "cross_session_steering_denied: request tried to steer outside the authoritative orchestration session"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_maps_stale_retained_worker_to_invalidated_worker_denial() {
        let err = map_continue_world_dispatch_resolution_error(anyhow::anyhow!(
            "stale_linkage: orchestration session sess_dispatch retained worker ash_member is no longer authoritative-live"
        ));

        assert_eq!(
            err.to_string(),
            "invalidated_worker_not_routable: stale_linkage: orchestration session sess_dispatch retained worker ash_member is no longer authoritative-live"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_rejects_ephemeral_concurrency_cap_exceeded() {
        let mut policy = sample_world_dispatch_policy();
        policy.agents_world_dispatch_max_concurrent_ephemeral = 1;

        let prepared = PreparedOrchestratorWorldDispatch {
            store: sample_state_store(),
            request: sample_request(),
            session: sample_session(),
            caller_participant: sample_orchestrator_participant(),
            target_participant: None,
            live_retained_worker_count: 0,
        };
        let _guard = acquire_world_dispatch_concurrency_guard(&prepared, &policy)
            .expect("first ephemeral dispatch should reserve the only slot");

        let err = acquire_world_dispatch_concurrency_guard(&prepared, &policy)
            .expect_err("second ephemeral dispatch should fail closed");

        assert_eq!(
            err.to_string(),
            "worker_concurrency_cap_exceeded: effective policy allows at most 1 concurrent ephemeral world dispatches for orchestration session sess_dispatch"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn dispatch_contract_steering_policy_rejects_live_retained_worker_cap_exceeded() {
        let mut policy = sample_world_dispatch_policy();
        policy.agents_world_dispatch_max_live_retained_workers = 1;

        let prepared = PreparedOrchestratorWorldDispatch {
            store: sample_state_store(),
            request: sample_spawn_request(),
            session: sample_session(),
            caller_participant: sample_orchestrator_participant(),
            target_participant: None,
            live_retained_worker_count: 1,
        };

        let err = acquire_world_dispatch_concurrency_guard(&prepared, &policy)
            .expect_err("spawn above retained worker cap must fail closed");

        assert_eq!(
            err.to_string(),
            "worker_concurrency_cap_exceeded: effective policy allows at most 1 live retained workers for orchestration session sess_dispatch; authoritative live count is 1"
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

    #[cfg(target_os = "linux")]
    #[test]
    fn inspect_world_worker_summary_stays_snapshot_only() {
        let summary = summarize_inspect_world_worker_result("ash_member", "cli:codex_world");
        assert!(
            summary.contains("authoritative retained snapshot"),
            "summary should describe inspect as a snapshot surface: {summary}"
        );
        assert!(
            summary.contains("without invoking world-side execution transport"),
            "summary must stay explicit about inspect remaining store-backed: {summary}"
        );
    }
}
