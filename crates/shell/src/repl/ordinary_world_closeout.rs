//! World-bound local controller settlement and physical closeout evidence.
//! Construction requires the genuine issued controller capability; a World
//! binding or local native terminal cannot certify independent remote work.

use super::*;
use crate::execution::agent_runtime::{
    host_session_authority::{
        schema::{DirectHostNativeTerminalEvidenceV1, PostTurnCompletionHashInputV4},
        store::hold_existing_accepted_work_authority_snapshot,
        world_bound_controller::{
            AuthenticatedWorldBoundControllerRunV1, WorldBoundControllerEntryV1,
        },
    },
    obligation_ledger::world_bound_controller::authenticate_world_controller_work_attention_cut,
};

pub(super) struct WorldControllerResumeSettlementV1 {
    pub(super) controller: Arc<AuthenticatedWorldBoundControllerRunV1>,
    pub(super) launch: Arc<AuthorityManagedSuccessorLaunchPlanV1>,
    pub(super) outcome: PostTurnResolutionOutcomeV1,
    pub(super) completion: PostTurnCompletionHashInputV4,
}

pub(super) enum WorldNativeHelperSettlementV1 {
    Start {
        controller: Arc<AuthenticatedWorldBoundControllerRunV1>,
        settlement: Box<AuthenticatedStartHelperSettlementV1>,
    },
    Resume(Arc<WorldControllerResumeSettlementV1>),
}

/// Local ingress has already closed. Retain this same helper runtime until its
/// actual issued remote observers finish, including B1 observers outside the
/// member map. Public response delivery is independent of this process wait.
pub(super) async fn join_world_controller_observers_after_local_closeout(
    context: Option<&RuntimeOrchestrationContext>,
    members: &mut RetainedMemberRuntimeMap,
) -> Result<()> {
    let Some(controller) = context.and_then(|context| context.ordinary_world_controller.as_ref())
    else {
        return Ok(());
    };
    let mut failures = Vec::new();
    let mut issued_members = std::mem::take(members).into_values().collect::<Vec<_>>();
    for member in &mut issued_members {
        if let Err(error) = finish_world_controller_member_observer(member).await {
            failures.push(format!("{error:#}"));
        }
    }
    // Every member's ingress is closed before waiting on any remote attempt.
    for member in &mut issued_members {
        match &mut member.retained_control {
            RetainedRunControl::Remote(control) => {
                if let Some(task) = control.observe_task.take() {
                    // The observer records terminal/interruption truth. Its
                    // join alone supplies no native or attention certificate.
                    if let Err(error) = task.await {
                        failures.push(error.to_string());
                    }
                }
            }
            _ => failures.push("genuine World member lost its remote observer population".into()),
        }
    }
    if let Err(error) = controller.selection().join_issued_world_observers().await {
        failures.push(format!("{error:#}"));
    }
    if !failures.is_empty() {
        anyhow::bail!(
            "World observer lifetime closeout unresolved: {}",
            failures.join("; ")
        );
    }
    Ok(())
}

async fn finish_world_controller_member_observer(
    runtime: &mut AsyncReplAgentRuntime,
) -> Result<()> {
    let mut failures = Vec::new();
    // Dispose only this helper's private ingress and accepted handlers. No
    // member Stop/cancel/park operation follows from parent native completion.
    if let Some(mut transport) = runtime.cancel_transport.take() {
        if let Err(error) = transport
            .close_owned(LOCAL_RETAINED_STOP_COMPLETION_TIMEOUT)
            .await
        {
            failures.push(format!("{error:#}"));
        }
    }
    if let Some(mut transport) = runtime.stop_transport.take() {
        if let Err(error) = transport
            .close_owned(
                runtime.private_stop_rx.as_mut(),
                LOCAL_RETAINED_STOP_COMPLETION_TIMEOUT,
            )
            .await
        {
            failures.push(format!("{error:#}"));
        }
    }
    if let Some(mut transport) = runtime.prompt_transport.take() {
        if let Err(error) = transport
            .close_owned(LOCAL_RETAINED_STOP_COMPLETION_TIMEOUT)
            .await
        {
            failures.push(format!("{error:#}"));
        }
    }
    if let Some(mut transport) = runtime.toolbox_transport.take() {
        runtime
            .host_toolbox_surface_authoritative
            .store(false, Ordering::SeqCst);
        if let Err(error) = transport
            .close_owned(LOCAL_RETAINED_STOP_COMPLETION_TIMEOUT)
            .await
        {
            failures.push(format!("{error:#}"));
        }
    }
    for (task, name) in [
        (
            &mut runtime.cancel_owner_task,
            "World member cancel ingress",
        ),
        (&mut runtime.stop_owner_task, "World member stop ingress"),
        (
            &mut runtime.prompt_owner_task,
            "World member prompt ingress",
        ),
    ] {
        if task.is_some() {
            if let Err(error) = join_direct_host_closeout_task(task, name).await {
                failures.push(format!("{error:#}"));
            }
        }
    }
    if let Some(stop) = runtime.heartbeat_stop_tx.take() {
        let _ = stop.send(());
    }
    if runtime.heartbeat_task.is_some() {
        if let Err(error) =
            join_direct_host_closeout_task(&mut runtime.heartbeat_task, "World member heartbeat")
                .await
        {
            failures.push(format!("{error:#}"));
        }
    }
    if !failures.is_empty() {
        anyhow::bail!(
            "World member observer closeout failed: {}",
            failures.join("; ")
        );
    }
    Ok(())
}

impl WorldNativeHelperSettlementV1 {
    pub(super) fn posture(&self) -> HostSessionPostureV1 {
        match self {
            Self::Start { settlement, .. } => settlement.receipt.resulting_posture,
            Self::Resume(settlement) => settlement.completion.resulting_posture(),
        }
    }
}

pub(super) fn settle_world_controller_start_from_native(
    controller: &Arc<AuthenticatedWorldBoundControllerRunV1>,
    resolved: &Arc<ResolvedCurrentAuthorityV1>,
    registration: &StartContinuationRegistrationV1,
    event_id: String,
    terminal: &AuthoritativeStartTerminalEvidenceV1,
    kind: StartTurnCompletionKindV1,
) -> std::result::Result<AuthenticatedStartHelperSettlementV1, RuntimeBootstrapFailure> {
    let failure = |error: String| start_continuity_failure("World controller settlement", error);
    let authority = controller.selection().held_authority();
    let root = authority
        .read_a12b_root()
        .map_err(|error| failure(error.to_string()))?;
    let current = authority
        .resolve_current_exact(&resolved.authority.orchestration_session_id, None)
        .map_err(|error| failure(error.to_string()))?;
    if current.observation.root_revision != root.root_revision
        || current.caller != resolved.caller
        || current.caller != controller.origin().caller
        || current.authority.origin != resolved.authority.origin
    {
        return Err(failure(
            "World Start settlement current/original actor differs".into(),
        ));
    }
    let crate::execution::agent_runtime::host_session_authority::schema::DurableSessionAuthorityOriginV1::StartIntent {
        intent_id, issuer_request_id, payload_commitment,
    } = &resolved.authority.origin;
    let intent = root
        .transition_intent_map
        .get(intent_id)
        .ok_or_else(|| failure("original Start intent missing".into()))?;
    let transaction = exact_start_transaction(&root, resolved)?;
    let HostSessionTransitionIntentStateV2::Applied {
        application_result_ref,
        ..
    } = &intent.state
    else {
        return Err(failure("World Start intent is not Applied".into()));
    };
    let request = SettleStartTurnRequestV1 {
        start_transaction_id: transaction.transaction_id.clone(),
        request_key_sha256: transaction.request_key_sha256.clone(),
        authority_store_id: root.authority_store_id.clone(),
        orchestration_session_id: resolved.authority.orchestration_session_id.clone(),
        intent_id: intent_id.clone(),
        issuer_request_id: issuer_request_id.clone(),
        payload_commitment: payload_commitment.clone(),
        application_result_ref: application_result_ref.clone(),
        run_id: intent.run_id.clone(),
        authoritative_participant_id: resolved.caller.participant_id.clone(),
        backend_id: resolved.caller.descriptor.backend_id.clone(),
        protocol: resolved.caller.descriptor.protocol.clone(),
        registered_resume_handle_ref: registration.resume_handle_ref.clone(),
        expected_authority_revision: current.observation.authority_revision,
        protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: resolved.caller.participant_id.clone(),
        },
        event_id,
        event_sequence: 2,
        provider_event_kind: terminal.provider_event_kind.clone(),
        thread_id: terminal.thread_id.clone(),
        turn_id: terminal.turn_id.clone(),
        completion_evidence_sha256: terminal.raw_event_sha256.clone(),
        kind,
        obligation_ledger_read: None,
        completed_at: successor_timestamp_now()?,
    };
    let held = hold_existing_accepted_work_authority_snapshot(authority)
        .map_err(|error| failure(format!("World Start physical source: {error:?}")))?;
    let sources = held
        .capture_world_controller_completion_sources(&request.orchestration_session_id)
        .map_err(|error| failure(format!("World Start attention source: {error:?}")))?;
    let cut =
        authenticate_world_controller_work_attention_cut(&sources, controller.world_binding())
            .map_err(|error| failure(error.to_string()))?;
    let outcome = authority
        .settle_ordinary_world_controller_start(&request, controller, &cut)
        .map_err(|error| failure(error.to_string()))?;
    let receipt = match outcome {
        StartTurnSettlementOutcomeV1::Applied(receipt)
        | StartTurnSettlementOutcomeV1::Joined(receipt) => receipt,
    };
    Ok(AuthenticatedStartHelperSettlementV1 {
        origin: Arc::clone(resolved),
        request,
        receipt,
    })
}

pub(super) fn settle_world_controller_resume_from_native(
    controller: &Arc<AuthenticatedWorldBoundControllerRunV1>,
    launch: &Arc<AuthorityManagedSuccessorLaunchPlanV1>,
    native: Option<&DirectHostNativeTerminalEvidenceV1>,
    kind: HostPostTurnProtocolEventKindV1,
) -> std::result::Result<Arc<WorldControllerResumeSettlementV1>, RuntimeBootstrapFailure> {
    let failure = |message: String| RuntimeBootstrapFailure {
        exit_code: 1,
        message: format!("World controller Resume settlement unresolved: {message}"),
    };
    let native = native.ok_or_else(|| failure("exact native terminal sequence absent".into()))?;
    if controller.entry() != WorldBoundControllerEntryV1::ResumeOneTurn
        || controller.intent_id() != launch.applied_transition.intent_id
        || controller.run_id() != launch.applied_transition.run_id
    {
        return Err(failure(
            "genuine controller and issued Resume differ".into(),
        ));
    }
    let authority = controller.selection().held_authority();
    let held = hold_existing_accepted_work_authority_snapshot(authority)
        .map_err(|error| failure(format!("physical source: {error:?}")))?;
    let sources = held
        .capture_world_controller_completion_sources(
            &launch.applied_transition.orchestration_session_id,
        )
        .map_err(|error| failure(format!("attention source: {error:?}")))?;
    let cut =
        authenticate_world_controller_work_attention_cut(&sources, controller.world_binding())
            .map_err(|error| failure(error.to_string()))?;
    let outcome = authority
        .settle_ordinary_world_controller_resume(
            controller,
            native,
            kind,
            &successor_timestamp_now()?,
            &cut,
        )
        .map_err(|error| failure(error.to_string()))?;
    let intent = match &outcome {
        PostTurnResolutionOutcomeV1::Applied(intent)
        | PostTurnResolutionOutcomeV1::Joined(intent) => intent,
        PostTurnResolutionOutcomeV1::AwaitingObligationCut(_) => {
            return Err(failure("controller returned a retained-worker cut".into()))
        }
    };
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &intent.state else {
        return Err(failure("Applied identity lost".into()));
    };
    let HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } = post_turn.as_ref()
    else {
        return Err(failure("post-turn completion missing".into()));
    };
    if completion_ref.schema_version != 4 {
        return Err(failure("controller completion version differs".into()));
    }
    let root = authority
        .read_a12b_root()
        .map_err(|error| failure(error.to_string()))?;
    let bytes = authority
        .read_authority_object_v2_at(root.root_revision, completion_ref)
        .map_err(|error| failure(error.to_string()))?;
    let completion =
        canonical_json::from_slice(&bytes).map_err(|error| failure(error.to_string()))?;
    Ok(Arc::new(WorldControllerResumeSettlementV1 {
        controller: Arc::clone(controller),
        launch: Arc::clone(launch),
        outcome,
        completion,
    }))
}

impl AuthorityManagedResumeSettlement {
    pub(super) fn into_native_helper_closeout(
        self,
    ) -> Option<Arc<NativeHelperCloseoutSettlementV1>> {
        match self {
            Self::DirectHost(value) => Some(Arc::new(
                NativeHelperCloseoutSettlementV1::DirectHostResume(value),
            )),
            Self::OrdinaryWorld(value) => {
                Some(Arc::new(NativeHelperCloseoutSettlementV1::OrdinaryWorld(
                    WorldNativeHelperSettlementV1::Resume(value),
                )))
            }
            Self::World(_) => None,
        }
    }
}

pub(super) fn world_controller_terminal_native(
    controller: &AuthenticatedWorldBoundControllerRunV1,
    launch: &AuthorityManagedSuccessorLaunchPlanV1,
    observation: &TurnLifecycleObservationV1,
    state: Option<&AuthoritativeStartLifecycleStateV1>,
) -> Option<DirectHostNativeTerminalEvidenceV1> {
    if controller.entry() != WorldBoundControllerEntryV1::ResumeOneTurn
        || controller.intent_id() != launch.applied_transition.intent_id
        || controller.run_id() != launch.applied_transition.run_id
        || launch.applied_transition.world_binding.as_ref() != Some(controller.world_binding())
        || launch.helper_plan.descriptor.backend_id
            != controller.selection().native_selection().backend_id
        || launch.helper_plan.descriptor.execution_scope != AgentExecutionScope::Host
    {
        return None;
    }
    // This existing helper extracts wrapper sequence data only. The genuine
    // World controller above is the separate admission; no None guard changes.
    let active = direct_host_resume_active_native(launch, state?)?;
    if active.native_session_id != observation.session_id
        || active.thread_id != observation.thread_id
        || active.turn_id != observation.turn_id
        || !matches!(
            observation.phase,
            TurnLifecyclePhaseV1::Completed | TurnLifecyclePhaseV1::Failed
        )
    {
        return None;
    }
    Some(DirectHostNativeTerminalEvidenceV1 {
        schema_version: 1,
        backend_id: active.backend_id,
        protocol: active.protocol,
        native_session_id: active.native_session_id,
        thread_id: active.thread_id,
        turn_id: active.turn_id,
        sequence: active.sequence,
        terminal_event_kind: observation.provider_event_kind.clone(),
        raw_event_sha256: observation.raw_event_sha256.clone(),
        terminal_ordinal: 1,
    })
}

fn verify_world_helper_issued_identity(
    runtime: &AsyncReplAgentRuntime,
    binding: &DirectHostResumeHelperCloseoutBinding,
    committed: &NativeHelperCloseoutSettlementV1,
) -> Result<()> {
    let (
        NativeHelperCloseoutLaunchV1::OrdinaryWorld {
            controller,
            successor,
        },
        NativeHelperCloseoutSettlementV1::OrdinaryWorld(settlement),
    ) = (&binding.launch, committed)
    else {
        anyhow::bail!("World helper requires its own issued launch and settlement");
    };
    if !binding
        .session
        .ptr_eq(&Arc::downgrade(&runtime.orchestration_session))
        || !binding.manifest.ptr_eq(&Arc::downgrade(&runtime.manifest))
        || binding.native_completed.get().is_none()
        || runtime.descriptor.execution_scope != AgentExecutionScope::Host
        || !matches!(runtime.retained_control, RetainedRunControl::Local(_))
    {
        anyhow::bail!("World helper physical ownership differs");
    }
    controller.verify_workspace()?;
    let session = runtime
        .orchestration_session
        .lock()
        .map_err(|_| anyhow!("World helper session poisoned"))?;
    let manifest = runtime
        .manifest
        .lock()
        .map_err(|_| anyhow!("World helper manifest poisoned"))?;
    if !manifest.is_host_orchestrator()
        || session.orchestration_session_id != controller.selection().original_session_id()
        || manifest.handle.orchestration_session_id != session.orchestration_session_id
        || manifest.internal.latest_run_id.as_deref() != Some(controller.run_id())
        || runtime.descriptor.backend_id != controller.selection().native_selection().backend_id
    {
        anyhow::bail!("World helper session, run or configured local descriptor differs");
    }
    match settlement {
        WorldNativeHelperSettlementV1::Start {
            controller: issued,
            settlement,
        } => {
            let proof = &settlement.request;
            let receipt = &settlement.receipt;
            if !Arc::ptr_eq(controller, issued)
                || successor.is_some()
                || controller.entry() != WorldBoundControllerEntryV1::Start
                || settlement.origin.authority.origin != controller.origin().authority.origin
                || settlement.origin.caller != controller.origin().caller
                || settlement.origin.authority.world_binding.as_ref()
                    != Some(controller.world_binding())
                || proof.obligation_ledger_read.is_some()
                || proof.intent_id != controller.intent_id()
                || proof.run_id != controller.run_id()
                || proof.authority_store_id != controller.selection().authority_store_id()
                || proof.orchestration_session_id != session.orchestration_session_id
                || proof.authoritative_participant_id != manifest.handle.participant_id
                || proof.backend_id != runtime.descriptor.backend_id
                || proof.protocol != runtime.descriptor.protocol
                || proof.event_sequence != 2
                || proof.expected_authority_revision.checked_add(1)
                    != Some(receipt.authority_revision_after)
                || receipt.continuation_resume_handle_ref.schema_version != 3
                || manifest.internal.uaa_session_id.as_deref() != Some(proof.thread_id.as_str())
                || runtime.uaa_session_handle_id != proof.thread_id
                || proof.turn_id.is_empty()
            {
                anyhow::bail!("World Start settlement does not bind this actual native helper");
            }
        }
        WorldNativeHelperSettlementV1::Resume(settlement) => {
            let launch = successor
                .as_ref()
                .context("World Resume issued launch absent")?;
            let c = &settlement.completion;
            let b = &c.binding;
            if !Arc::ptr_eq(controller, &settlement.controller)
                || !Arc::ptr_eq(launch, &settlement.launch)
                || controller.entry() != WorldBoundControllerEntryV1::ResumeOneTurn
                || c.schema_version != 4
                || b.intent_id != controller.intent_id()
                || b.run_id != controller.run_id()
                || b.world_binding != *controller.world_binding()
                || b.native_selection != *controller.selection().native_selection()
                || b.requested_selection != *controller.selection().requested_selection()
                || b.authoritative_participant_id != manifest.handle.participant_id
                || b.orchestration_session_id != session.orchestration_session_id
                || c.native_terminal.backend_id != runtime.descriptor.backend_id
                || c.native_terminal.protocol != runtime.descriptor.protocol
                || manifest.internal.uaa_session_id.as_deref()
                    != Some(c.native_terminal.native_session_id.as_str())
                || runtime.uaa_session_handle_id != c.native_terminal.native_session_id
                || !matches!(&settlement.outcome, PostTurnResolutionOutcomeV1::Applied(intent) | PostTurnResolutionOutcomeV1::Joined(intent)
                    if intent.intent_id == b.intent_id && intent.run_id == b.run_id
                    && intent.world_binding.as_ref() == Some(&b.world_binding)
                    && matches!(&intent.input_handoff, HostSessionTransitionInputHandoffV1::Accepted { run_id, .. } if run_id == &b.run_id))
            {
                anyhow::bail!("World Resume settlement does not bind this actual native helper");
            }
        }
    }
    Ok(())
}

pub(super) fn validate_world_controller_helper_closeout(
    runtime: &AsyncReplAgentRuntime,
    request: &Arc<DirectHostResumeHelperCloseoutRequest>,
) -> Result<()> {
    let binding = &runtime
        .direct_host_closeout
        .as_ref()
        .context("World helper closeout owner absent")?
        .binding;
    let committed = binding
        .committed
        .get()
        .context("World helper committed result absent")?;
    if !Arc::ptr_eq(binding, &request.binding)
        || !Arc::ptr_eq(committed, &request.settlement)
        || binding.native_completed.get() != Some(&request.native_status)
        || !request.native_status.success()
        || runtime.shutdown_requested.load(Ordering::SeqCst)
        || runtime.cancel_requested.load(Ordering::SeqCst)
        || runtime
            .private_stop_rx
            .as_ref()
            .is_some_and(|rx| !rx.is_empty())
        || runtime.cancel_transport.is_some()
        || runtime.cancel_owner_task.is_some()
        || !matches!(
            committed.posture(),
            HostSessionPostureV1::ParkedResumable | HostSessionPostureV1::AwaitingAttention
        )
        || !runtime
            .manifest
            .lock()
            .map_err(|_| anyhow!("World helper manifest poisoned"))?
            .handle
            .state
            .is_live()
    {
        anyhow::bail!("World helper successful physical closeout changed or competed");
    }
    verify_world_helper_issued_identity(runtime, binding, committed)
}

pub(super) fn validate_world_controller_terminal_failure(
    runtime: &AsyncReplAgentRuntime,
) -> Result<()> {
    let binding = &runtime
        .direct_host_closeout
        .as_ref()
        .context("World failure owner absent")?
        .binding;
    let committed = binding
        .committed
        .get()
        .context("World failure committed result absent")?;
    verify_world_helper_issued_identity(runtime, binding, committed)?;
    let NativeHelperCloseoutSettlementV1::OrdinaryWorld(settlement) = committed.as_ref() else {
        anyhow::bail!("World failure result class differs");
    };
    let failed = match settlement {
        WorldNativeHelperSettlementV1::Start { settlement, .. } => matches!(
            settlement.request.kind,
            StartTurnCompletionKindV1::TerminalFailure { .. }
        ),
        WorldNativeHelperSettlementV1::Resume(settlement) => matches!(
            settlement.completion.kind,
            HostPostTurnProtocolEventKindV1::TerminalFailure { .. }
        ),
    };
    if !failed
        || settlement.posture() != HostSessionPostureV1::Terminal
        || !matches!(
            runtime
                .manifest
                .lock()
                .map_err(|_| anyhow!("World failure manifest poisoned"))?
                .handle
                .state,
            AgentRuntimeSessionState::Failed | AgentRuntimeSessionState::Invalidated
        )
    {
        anyhow::bail!("World no-request outcome is not its genuine typed native failure");
    }
    // Failure classification never supplies resumable cleanup success.
    Ok(())
}

/// Dispose only one admitted Stop target's shell-local bootstrap observation
/// and private ingress. Bootstrap completion already came from its genuine
/// owner before intent admission. This cleanup neither cancels native work nor
/// certifies logical retirement, and never joins unrelated B1 observers.
pub(super) async fn join_stopping_world_member_observer(
    scope: &crate::execution::agent_runtime::retained_worker_runtime::PreparedOrdinaryWorkerStopV1,
    members: &mut RetainedMemberRuntimeMap,
) -> Result<()> {
    let request = scope.request();
    let candidates = members
        .values()
        .filter(|runtime| {
            let manifest = runtime_manifest_snapshot(runtime);
            manifest.handle.role == MEMBER_ROLE
                && manifest.handle.orchestration_session_id == request.orchestration_session_id
                && manifest.handle.participant_id == request.target_participant_id
        })
        .collect::<Vec<_>>();
    if candidates.len() > 1 {
        anyhow::bail!("stale_authority: Stop local observer target is ambiguous");
    }
    let Some(runtime) = candidates.first() else {
        return Ok(());
    };
    let manifest = runtime_manifest_snapshot(runtime);
    let crate::execution::agent_runtime::control::PromptSubmitAuthorityV1::Ordinary(selection) =
        &runtime.prompt_authority
    else {
        anyhow::bail!("stale_authority: Stop local observer is not its original ordinary owner");
    };
    if manifest.handle.backend_id != request.target_backend_id
        || manifest.handle.world_id.as_deref() != Some(request.world_id.as_str())
        || manifest.handle.world_generation != Some(request.world_generation)
        || manifest.internal.latest_run_id.as_deref() != Some(request.bootstrap_run_id.as_str())
        || crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
            selection,
        )? != *scope.source_start()
        || !matches!(runtime.retained_control, RetainedRunControl::Remote(_))
    {
        anyhow::bail!(
            "stale_authority: Stop local observer changed original backend/World/bootstrap source"
        );
    }
    let mut runtime = remove_retained_member_runtime_by_participant(
        members,
        &request.orchestration_session_id,
        &request.target_participant_id,
    )
    .context("stale_authority: Stop local observer disappeared")?;
    let ingress = finish_world_controller_member_observer(&mut runtime).await;
    let observer = match &mut runtime.retained_control {
        RetainedRunControl::Remote(control) => {
            join_direct_host_closeout_task(
                &mut control.observe_task,
                "ordinary Stop bootstrap observer",
            )
            .await
        }
        _ => anyhow::bail!("recovery_failed: ordinary Stop lost its exact remote observer"),
    };
    match (ingress, observer) {
        (Ok(()), Ok(())) => Ok(()),
        (left, right) => anyhow::bail!("recovery_failed: ordinary Stop local lifetime cleanup unresolved: ingress={:?}; observer={:?}", left.err(), right.err()),
    }
}
