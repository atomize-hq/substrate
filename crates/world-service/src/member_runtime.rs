use agent_api::{
    AgentWrapperCancelHandle, AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent,
    AgentWrapperEventKind, AgentWrapperRunControl, AgentWrapperRunRequest,
};
use anyhow::{anyhow, Context, Result};
use axum::{
    body::{boxed, Bytes, StreamBody},
    http::StatusCode,
    response::Response,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    convert::Infallible,
    fs,
    path::Path,
    sync::{Arc, Mutex, RwLock},
};
use substrate_common::agent_events::{AgentEvent, AgentEventKind, MessageEventKind};
use tokio_stream::wrappers::UnboundedReceiverStream;
use transport_api_types::{
    ExecuteStreamFrame, MemberDispatchRequestV1, MemberRuntimeBackendKindV1,
    MemberTurnSubmitRequestV1, ProcessTelemetry,
};
use world_api::SharedWorldBindingSnapshot;

use crate::gateway_runtime::{prepare_linux_world_entry_launcher, LinuxWorldPlacementContext};
use crate::prompt_fulfillment::PromptFulfillmentBridge;
use crate::runtime_replay::{publish_replayable_frame, RuntimeReplayRegistry};
use crate::service::RuntimeEventStreamProducer;

const MEMBER_ROLE: &str = "member";
const SESSION_HANDLE_SCHEMA_V1: &str = "agent_api.session.handle.v1";
const CANCELLED_MESSAGE: &str = "cancelled";
const ADD_DIRS_EXTENSION_V1: &str = "agent_api.exec.add_dirs.v1";
const SESSION_RESUME_EXTENSION_V1: &str = "agent_api.session.resume.v1";
const SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV: &str = "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME";

#[derive(Clone, Default)]
pub(crate) struct MemberRuntimeManager {
    active_members: Arc<RwLock<ActiveMemberRegistry>>,
    active_turns_by_span_id: Arc<RwLock<HashMap<String, Arc<ActiveSubmittedTurn>>>>,
    runtime_replay: RuntimeReplayRegistry,
}

#[derive(Default)]
struct ActiveMemberRegistry {
    by_participant_id: HashMap<String, Arc<ActiveMemberRuntime>>,
    by_retained_key: HashMap<RetainedMemberKey, RetainedMemberSlot>,
}

pub(crate) struct MemberRuntimeLaunchAdmissionV1 {
    pub(crate) dispatch: MemberDispatchRequestV1,
    pub(crate) acceptance_context: Option<transport_api_types::WorldWorkAcceptanceContextV1>,
}

struct ActiveMemberRuntime {
    agent_id: String,
    participant_id: String,
    orchestration_session_id: String,
    orchestrator_participant_id: String,
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    backend_kind: MemberRuntimeBackendKindV1,
    binary_path: std::path::PathBuf,
    launcher_dir: std::path::PathBuf,
    workspace_dir: std::path::PathBuf,
    process_working_dir: std::path::PathBuf,
    env: BTreeMap<String, String>,
    binding: SharedWorldBindingSnapshot,
    protocol: serde_json::Value,
    bootstrap: Mutex<Option<ActiveBootstrapRuntime>>,
    active_turn_span_id: Mutex<Option<String>>,
    uaa_session_id: Mutex<Option<String>>,
}

struct ActiveBootstrapRuntime {
    span_id: String,
    cancel: Option<AgentWrapperCancelHandle>,
    last_signal: Option<String>,
}

struct ActiveSubmittedTurn {
    participant_id: String,
    cancel: AgentWrapperCancelHandle,
    last_signal: Mutex<Option<String>>,
}

#[derive(Clone)]
struct MemberStreamContext {
    orchestration_session_id: String,
    run_id: String,
    acceptance_context: Option<transport_api_types::WorldWorkAcceptanceContextV1>,
    participant_id: String,
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    protocol: serde_json::Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemberStreamMode {
    Bootstrap,
    SubmittedTurn,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct RetainedMemberKey {
    orchestration_session_id: String,
    world_generation: u64,
    backend_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RetainedMemberSlot {
    primary_participant_id: String,
    fork_child_participant_id: Option<String>,
}

impl MemberRuntimeManager {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_replay_registry(runtime_replay: RuntimeReplayRegistry) -> Self {
        Self {
            runtime_replay,
            ..Self::default()
        }
    }

    pub(crate) async fn launch(
        &self,
        agent_id: String,
        env: HashMap<String, String>,
        span_id: String,
        admission: MemberRuntimeLaunchAdmissionV1,
        binding: SharedWorldBindingSnapshot,
        placement: LinuxWorldPlacementContext,
    ) -> Result<Response> {
        let MemberRuntimeLaunchAdmissionV1 {
            dispatch,
            acceptance_context,
        } = admission;
        if let Some(context) = acceptance_context.as_ref() {
            context
                .validate()
                .map_err(crate::service::BadRequestError::new)?;
            if context.request_id != dispatch.run_id || context.message_id.is_some() {
                return Err(crate::service::BadRequestError::new(
                    "task acceptance context does not match member dispatch".to_string(),
                )
                .into());
            }
        }
        let actual_binary = validate_member_runtime_binary(&dispatch)?;
        let PreparedMemberRuntimeLauncher {
            launcher_path,
            launcher_dir,
            env: launcher_env,
        } = prepare_member_runtime_launcher(&actual_binary, &placement)?;
        let runtime_env = env
            .into_iter()
            .chain(launcher_env.into_iter())
            .collect::<BTreeMap<_, _>>();
        let mut runtime_env = runtime_env;
        let process_working_dir = member_runtime_process_working_dir(
            dispatch.resolved_runtime.backend_kind,
            &placement.working_dir,
            &launcher_dir,
        );
        let mut prepared_launcher_dir = Some(launcher_dir.clone());
        if let Err(err) = prepare_runtime_env_for_member_backend(
            &mut runtime_env,
            dispatch.resolved_runtime.backend_kind,
            &placement.working_dir,
            &launcher_dir,
        ) {
            cleanup_prepared_launcher_dir(&mut prepared_launcher_dir);
            return Err(err);
        }

        let prompt_fulfillment = match PromptFulfillmentBridge::for_member_backend(
            &dispatch.resolved_runtime.backend_kind,
            launcher_path,
        ) {
            Ok(prompt_fulfillment) => prompt_fulfillment,
            Err(err) => {
                cleanup_prepared_launcher_dir(&mut prepared_launcher_dir);
                return Err(err);
            }
        };
        let initial_prompt = match dispatch.initial_prompt.clone() {
            Some(initial_prompt) => initial_prompt,
            None => {
                cleanup_prepared_launcher_dir(&mut prepared_launcher_dir);
                return Err(crate::service::BadRequestError::new(
                    "member_dispatch.initial_prompt is required for launch-time first turn"
                        .to_string(),
                )
                .into());
            }
        };
        let launch_extensions = member_runtime_workspace_access_extensions(
            dispatch.resolved_runtime.backend_kind,
            &placement.working_dir,
        );
        let AgentWrapperRunControl { handle, cancel } = match prompt_fulfillment
            .run_control(AgentWrapperRunRequest {
                prompt: initial_prompt,
                working_dir: Some(process_working_dir.clone()),
                timeout: None,
                env: runtime_env.clone(),
                extensions: launch_extensions,
            })
            .await
        {
            Ok(control) => control,
            Err(err) => {
                cleanup_prepared_launcher_dir(&mut prepared_launcher_dir);
                return Err(map_wrapper_error(err));
            }
        };

        let active = Arc::new(ActiveMemberRuntime {
            agent_id,
            participant_id: dispatch.participant_id.clone(),
            orchestration_session_id: dispatch.orchestration_session_id.clone(),
            orchestrator_participant_id: dispatch.orchestrator_participant_id.clone(),
            parent_participant_id: dispatch.parent_participant_id.clone(),
            resumed_from_participant_id: dispatch.resumed_from_participant_id.clone(),
            backend_id: dispatch.backend_id.clone(),
            backend_kind: dispatch.resolved_runtime.backend_kind,
            binary_path: actual_binary,
            launcher_dir: prepared_launcher_dir
                .take()
                .expect("prepared launcher dir must exist while bootstrap is active"),
            workspace_dir: placement.working_dir.clone(),
            process_working_dir,
            env: runtime_env,
            binding: binding.clone(),
            protocol: json!(dispatch.protocol),
            bootstrap: Mutex::new(Some(ActiveBootstrapRuntime {
                span_id: span_id.clone(),
                cancel: Some(cancel),
                last_signal: None,
            })),
            active_turn_span_id: Mutex::new(None),
            uaa_session_id: Mutex::new(None),
        });
        if let Err(err) = self.register_member(active.clone()) {
            active.cancel_bootstrap();
            active.close_bootstrap();
            active.cleanup_launcher_dir();
            return Err(err);
        }

        let context = MemberStreamContext {
            orchestration_session_id: dispatch.orchestration_session_id.clone(),
            run_id: dispatch.run_id.clone(),
            acceptance_context,
            participant_id: dispatch.participant_id.clone(),
            parent_participant_id: dispatch.parent_participant_id.clone(),
            resumed_from_participant_id: dispatch.resumed_from_participant_id.clone(),
            backend_id: dispatch.backend_id.clone(),
            protocol: json!(dispatch.protocol),
        };
        let (mut producer, start) = match start_member_runtime_stream(
            &context,
            MemberStreamMode::Bootstrap,
            span_id.clone(),
        ) {
            Ok(stream) => stream,
            Err(error) => {
                active.cancel_bootstrap();
                self.unregister_member(&active.participant_id);
                return Err(error);
            }
        };
        let replay_publisher = match self
            .runtime_replay
            .begin_for_acceptance(context.acceptance_context.as_ref(), &start)
        {
            Ok(publisher) => publisher,
            Err(error) => {
                active.cancel_bootstrap();
                self.unregister_member(&active.participant_id);
                return Err(error);
            }
        };
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(start);

        let manager = self.clone();
        let participant_id = dispatch.participant_id.clone();
        tokio::spawn(async move {
            let mut events = handle.events;
            let completion = handle.completion;
            let mut emitted_registered = false;

            while let Some(wrapper_event) = events.next().await {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(wrapper_event.data.as_ref())
                {
                    manager.remember_uaa_session_id(&participant_id, session_id);
                }
                match frame_from_wrapper_event(
                    &context,
                    &binding,
                    &span_id,
                    wrapper_event,
                    &mut emitted_registered,
                    MemberStreamMode::Bootstrap,
                    active.agent_id.as_str(),
                    &mut producer,
                ) {
                    Ok(Some(frame)) => {
                        if let Err(err) =
                            publish_replayable_frame(replay_publisher.as_ref(), &tx, frame)
                        {
                            tracing::error!(error = %err, "failed to publish replayable member bootstrap Event");
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        tracing::error!(error = %err, "failed to identify member bootstrap Event");
                        break;
                    }
                }
            }

            let completion = completion.await;
            if let Ok(ref completion) = completion {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(completion.data.as_ref())
                {
                    manager.remember_uaa_session_id(&participant_id, session_id);
                }
            }
            let preserve_retained_member = completion.as_ref().ok().is_some_and(|completion| {
                should_preserve_retained_member_after_clean_bootstrap(
                    &dispatch.participant_id,
                    &completion.status,
                    active.uaa_session_id().is_some(),
                )
            });
            let frames = frames_from_completion(
                &context,
                &binding,
                &span_id,
                completion,
                active.bootstrap_last_signal(),
                &mut emitted_registered,
                MemberStreamMode::Bootstrap,
                active.agent_id.as_str(),
                &mut producer,
            );
            match frames {
                Ok(frames) => {
                    for frame in frames {
                        if let Err(err) =
                            publish_replayable_frame(replay_publisher.as_ref(), &tx, frame)
                        {
                            tracing::error!(error = %err, "failed to publish replayable member bootstrap completion");
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(error = %err, "failed to identify member bootstrap completion");
                }
            }

            manager.finish_bootstrap(&participant_id, preserve_retained_member);
        });

        stream_response(rx)
    }

    pub(crate) async fn submit_turn(&self, req: MemberTurnSubmitRequestV1) -> Result<Response> {
        req.validate()
            .map_err(crate::service::BadRequestError::new)?;
        let active = self.find_submit_target(&req)?;
        validate_submit_turn_request(&req, RetainedMemberIdentity::from_active(active.as_ref()))?;
        self.validate_submit_target_slot(&req)?;
        let uaa_session_id = active.uaa_session_id().ok_or_else(|| {
            crate::service::BadRequestError::new(format!(
                "member_turn_submit.participant_id {} has no surfaced uaa_session_id",
                req.participant_id
            ))
        })?;

        let span_id = format!("spn_{}", uuid::Uuid::now_v7());
        self.reserve_turn_slot(&active, &span_id)?;

        let prompt_fulfillment = PromptFulfillmentBridge::for_member_backend(
            &active.backend_kind,
            active.binary_path.clone(),
        )?;
        let AgentWrapperRunControl { handle, cancel } = match prompt_fulfillment
            .run_control(build_submitted_turn_run_request(
                active.as_ref(),
                req.prompt.clone(),
                &uaa_session_id,
            ))
            .await
        {
            Ok(control) => control,
            Err(err) => {
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(map_wrapper_error(err));
            }
        };

        let context = active.submit_context(req.run_id.clone(), req.acceptance_context.clone());
        let (mut producer, start) = match start_member_runtime_stream(
            &context,
            MemberStreamMode::SubmittedTurn,
            span_id.clone(),
        ) {
            Ok(stream) => stream,
            Err(error) => {
                cancel.cancel();
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(error);
            }
        };
        let replay_publisher = match self
            .runtime_replay
            .begin_for_acceptance(context.acceptance_context.as_ref(), &start)
        {
            Ok(publisher) => publisher,
            Err(error) => {
                cancel.cancel();
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(error);
            }
        };
        let turn = Arc::new(ActiveSubmittedTurn {
            participant_id: active.participant_id.clone(),
            cancel,
            last_signal: Mutex::new(None),
        });
        self.register_turn(span_id.clone(), turn.clone());
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(start);

        let manager = self.clone();
        let binding = active.binding.clone();
        tokio::spawn(async move {
            let mut events = handle.events;
            let completion = handle.completion;
            let mut emitted_registered = false;

            while let Some(wrapper_event) = events.next().await {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(wrapper_event.data.as_ref())
                {
                    manager.remember_uaa_session_id(&turn.participant_id, session_id);
                }
                match frame_from_wrapper_event(
                    &context,
                    &binding,
                    &span_id,
                    wrapper_event,
                    &mut emitted_registered,
                    MemberStreamMode::SubmittedTurn,
                    active.agent_id.as_str(),
                    &mut producer,
                ) {
                    Ok(Some(frame)) => {
                        if let Err(err) =
                            publish_replayable_frame(replay_publisher.as_ref(), &tx, frame)
                        {
                            tracing::error!(error = %err, "failed to publish replayable submitted member Event");
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        tracing::error!(error = %err, "failed to identify submitted member Event");
                        break;
                    }
                }
            }

            let completion = completion.await;
            if let Ok(ref completion) = completion {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(completion.data.as_ref())
                {
                    manager.remember_uaa_session_id(&turn.participant_id, session_id);
                }
            }
            let frames = frames_from_completion(
                &context,
                &binding,
                &span_id,
                completion,
                turn.last_signal(),
                &mut emitted_registered,
                MemberStreamMode::SubmittedTurn,
                active.agent_id.as_str(),
                &mut producer,
            );
            match frames {
                Ok(frames) => {
                    for frame in frames {
                        if let Err(err) =
                            publish_replayable_frame(replay_publisher.as_ref(), &tx, frame)
                        {
                            tracing::error!(error = %err, "failed to publish replayable submitted member completion");
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(error = %err, "failed to identify submitted member completion");
                }
            }

            manager.unregister_turn(&span_id);
        });

        stream_response(rx)
    }

    pub(crate) fn cancel(&self, span_id: &str, sig: &str) -> Result<bool> {
        validate_cancel_signal(sig)?;
        let normalized_signal = sig.trim().to_ascii_uppercase();

        let submitted_turn = self
            .active_turns_by_span_id
            .read()
            .expect("member runtime registry lock poisoned")
            .get(span_id)
            .cloned();
        if let Some(submitted_turn) = submitted_turn {
            if let Ok(mut guard) = submitted_turn.last_signal.lock() {
                *guard = Some(normalized_signal.clone());
            }
            submitted_turn.cancel.cancel();
            return Ok(true);
        }

        let active_members = self
            .active_members
            .read()
            .expect("member runtime registry lock poisoned")
            .by_participant_id
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for active in active_members {
            if active.cancel_bootstrap_if_span_matches(span_id, normalized_signal.as_str()) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn register_member(&self, active: Arc<ActiveMemberRuntime>) -> Result<()> {
        let retained_key = RetainedMemberKey::from_active(active.as_ref());
        let mut guard = self
            .active_members
            .write()
            .expect("member runtime registry lock poisoned");
        if guard.by_participant_id.contains_key(&active.participant_id) {
            return Err(crate::service::BadRequestError::new(format!(
                "member_dispatch.participant_id {} is already retained",
                active.participant_id
            ))
            .into());
        }
        match guard.by_retained_key.get(&retained_key).cloned() {
            Some(slot) => {
                let slot = prune_stale_fork_child_slot(&mut guard, &retained_key, slot);
                let updated_slot = slot.with_registered_fork_child(
                    &retained_key,
                    &guard.by_participant_id,
                    active.as_ref(),
                )?;
                guard
                    .by_retained_key
                    .insert(retained_key.clone(), updated_slot);
            }
            None => {
                guard.by_retained_key.insert(
                    retained_key.clone(),
                    RetainedMemberSlot::new(active.participant_id.clone()),
                );
            }
        }
        guard
            .by_participant_id
            .insert(active.participant_id.clone(), active);
        Ok(())
    }

    fn unregister_member(&self, participant_id: &str) {
        if let Ok(mut guard) = self.active_members.write() {
            if let Some(active) = guard.by_participant_id.remove(participant_id) {
                let retained_key = RetainedMemberKey::from_active(active.as_ref());
                if let Some(slot) = guard.by_retained_key.get(&retained_key).cloned() {
                    match slot.without_participant(participant_id) {
                        Some(updated_slot) => {
                            guard.by_retained_key.insert(retained_key, updated_slot);
                        }
                        None => {
                            guard.by_retained_key.remove(&retained_key);
                        }
                    }
                }
                active.close_bootstrap();
                active.cleanup_launcher_dir();
            }
        }
    }

    fn finish_bootstrap(&self, participant_id: &str, preserve_retained_member: bool) {
        let active = self
            .active_members
            .read()
            .ok()
            .and_then(|guard| guard.by_participant_id.get(participant_id).cloned());
        let Some(active) = active else {
            return;
        };

        if preserve_retained_member {
            active.close_bootstrap();
        } else {
            self.unregister_member(participant_id);
        }
    }

    fn register_turn(&self, span_id: String, turn: Arc<ActiveSubmittedTurn>) {
        self.active_turns_by_span_id
            .write()
            .expect("member runtime registry lock poisoned")
            .insert(span_id, turn);
    }

    fn unregister_turn(&self, span_id: &str) {
        if let Ok(mut guard) = self.active_turns_by_span_id.write() {
            if let Some(turn) = guard.remove(span_id) {
                if let Some(active) =
                    self.active_members.read().ok().and_then(|active| {
                        active.by_participant_id.get(&turn.participant_id).cloned()
                    })
                {
                    self.clear_reserved_turn_slot(&active, span_id);
                }
            }
        }
    }

    fn reserve_turn_slot(&self, active: &Arc<ActiveMemberRuntime>, span_id: &str) -> Result<()> {
        let mut guard = active
            .active_turn_span_id
            .lock()
            .map_err(|_| anyhow!("member runtime turn slot lock poisoned"))?;
        if let Some(existing) = guard.as_ref() {
            return Err(crate::service::BadRequestError::new(format!(
                "member_turn_submit.participant_id {} already has an active submitted turn ({existing})",
                active.participant_id
            ))
            .into());
        }
        *guard = Some(span_id.to_string());
        Ok(())
    }

    fn clear_reserved_turn_slot(&self, active: &Arc<ActiveMemberRuntime>, span_id: &str) {
        if let Ok(mut guard) = active.active_turn_span_id.lock() {
            if guard.as_deref() == Some(span_id) {
                *guard = None;
            }
        }
    }

    fn remember_uaa_session_id(&self, participant_id: &str, session_id: String) {
        if let Some(active) = self
            .active_members
            .read()
            .ok()
            .and_then(|guard| guard.by_participant_id.get(participant_id).cloned())
        {
            active.remember_uaa_session_id(session_id);
        }
    }

    fn find_submit_target(
        &self,
        req: &MemberTurnSubmitRequestV1,
    ) -> Result<Arc<ActiveMemberRuntime>> {
        let retained_key = RetainedMemberKey::from_submit(req);
        let guard = self
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        if let Some(active) = guard.by_participant_id.get(&req.participant_id).cloned() {
            return Ok(active);
        }

        if let Some(existing_slot) = guard.by_retained_key.get(&retained_key) {
            let existing_participant_ids = existing_slot.participant_ids();
            return Err(retained_slot_owner_mismatch_error(
                &retained_key,
                &existing_participant_ids,
                &req.participant_id,
            )
            .into());
        }

        Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.participant_id {} is not retained",
            req.participant_id
        ))
        .into())
    }

    fn validate_submit_target_slot(&self, req: &MemberTurnSubmitRequestV1) -> Result<()> {
        let retained_key = RetainedMemberKey::from_submit(req);
        let guard = self
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        match guard.by_retained_key.get(&retained_key) {
            Some(slot) if slot.contains(&req.participant_id) => Ok(()),
            Some(slot) => {
                let participant_ids = slot.participant_ids();
                Err(retained_slot_owner_mismatch_error(
                    &retained_key,
                    &participant_ids,
                    &req.participant_id,
                )
                .into())
            }
            None => Err(missing_retained_slot_error(&retained_key).into()),
        }
    }
}

fn validate_member_runtime_binary(
    dispatch: &MemberDispatchRequestV1,
) -> Result<std::path::PathBuf> {
    let path = Path::new(&dispatch.resolved_runtime.binary_path);
    if !path.is_file() {
        return Err(anyhow!(
            "member_dispatch.resolved_runtime.binary_path does not exist or is not a file: {}",
            dispatch.resolved_runtime.binary_path
        ));
    }
    Ok(path.to_path_buf())
}

struct PreparedMemberRuntimeLauncher {
    launcher_path: std::path::PathBuf,
    launcher_dir: std::path::PathBuf,
    env: Vec<(String, String)>,
}

#[derive(Debug, Default, Deserialize)]
struct CodexSeedHomeConfig {
    model: Option<String>,
    model_provider: Option<String>,
    provider: Option<String>,
    base_url: Option<String>,
    openai_base_url: Option<String>,
    #[serde(default)]
    model_providers: BTreeMap<String, CodexSeedProviderRoutingConfig>,
    #[serde(default)]
    providers: BTreeMap<String, CodexSeedProviderRoutingConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct CodexSeedProviderRoutingConfig {
    base_url: Option<String>,
    #[serde(default, flatten)]
    unsupported_fields: BTreeMap<String, toml::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct CodexBaseUrlConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct CodexProjectTrustConfig {
    trust_level: String,
}

#[derive(Debug, Serialize)]
struct CodexStartupSubset {
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    openai_base_url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    model_providers: BTreeMap<String, CodexBaseUrlConfig>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    providers: BTreeMap<String, CodexBaseUrlConfig>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    projects: BTreeMap<String, CodexProjectTrustConfig>,
}

fn prepare_runtime_env_for_member_backend(
    runtime_env: &mut BTreeMap<String, String>,
    backend_kind: MemberRuntimeBackendKindV1,
    workspace_dir: &Path,
    launcher_dir: &Path,
) -> Result<()> {
    match backend_kind {
        MemberRuntimeBackendKindV1::Codex => {
            prepare_codex_runtime_env(runtime_env, workspace_dir, launcher_dir)?;
        }
        MemberRuntimeBackendKindV1::ClaudeCode => {}
    }
    Ok(())
}

fn prepare_codex_runtime_env(
    runtime_env: &mut BTreeMap<String, String>,
    workspace_dir: &Path,
    launcher_dir: &Path,
) -> Result<()> {
    let authoritative_project_dir =
        codex_authoritative_project_dir(runtime_env, workspace_dir).to_path_buf();
    let seed_home_raw = runtime_env
        .remove(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV)
        .ok_or_else(|| {
            anyhow!(
                "direct cli:codex-world compatibility bridge requires the policy-gated bootstrap seed source; without that bounded bridge input this Packet 2 seam would otherwise launch against the real workspace with ambient repo-local .codex still active"
            )
        })?;

    let seed_home = Path::new(seed_home_raw.trim());
    let codex_home = launcher_dir.join("codex-home");
    let layout = codex::CodexHomeLayout::new(codex_home.clone());
    layout.materialize(true)?;
    layout.seed_auth_from(
        seed_home,
        codex::AuthSeedOptions {
            require_auth: true,
            require_credentials: false,
            ..codex::AuthSeedOptions::default()
        },
    )?;
    write_codex_startup_subset(seed_home, &authoritative_project_dir, &layout)?;
    runtime_env.insert("CODEX_HOME".to_string(), codex_home.display().to_string());
    Ok(())
}

fn codex_authoritative_project_dir<'a>(
    runtime_env: &'a BTreeMap<String, String>,
    workspace_dir: &'a Path,
) -> &'a Path {
    runtime_env
        .get(crate::service::WORLD_PROJECT_DIR_OVERRIDE_ENV)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(Path::new)
        .unwrap_or(workspace_dir)
}

fn write_codex_startup_subset(
    seed_home: &Path,
    workspace_dir: &Path,
    layout: &codex::CodexHomeLayout,
) -> Result<()> {
    let startup_subset = read_codex_startup_subset(seed_home, workspace_dir)?;
    let rendered = toml::to_string(&startup_subset)
        .context("render bounded Codex startup subset for isolated CODEX_HOME")?;
    fs::write(layout.config_path(), rendered).with_context(|| {
        format!(
            "write bounded Codex startup subset into isolated CODEX_HOME at {}",
            layout.config_path().display()
        )
    })?;
    Ok(())
}

fn read_codex_startup_subset(seed_home: &Path, workspace_dir: &Path) -> Result<CodexStartupSubset> {
    let config_path = seed_home.join("config.toml");
    let raw = fs::read_to_string(&config_path).with_context(|| {
        format!(
            "direct cli:codex-world compatibility bridge could not derive bounded startup config from {}; config.toml is missing or unreadable, so isolated CODEX_HOME would otherwise fall back to Codex defaults",
            config_path.display()
        )
    })?;
    let parsed: CodexSeedHomeConfig = toml::from_str(&raw).with_context(|| {
        format!(
            "direct cli:codex-world compatibility bridge could not parse bounded startup config from {}; config.toml must contain a top-level model for truthful isolated startup",
            config_path.display()
        )
    })?;

    let model = normalize_non_empty_value(parsed.model).ok_or_else(|| {
        anyhow!(
            "direct cli:codex-world compatibility bridge could not derive a bounded startup model from {}; isolated CODEX_HOME would otherwise fall back to an unsupported default model",
            config_path.display()
        )
    })?;

    let model_provider = normalize_non_empty_value(parsed.model_provider);
    let provider = normalize_non_empty_value(parsed.provider);
    let base_url = normalize_non_empty_value(parsed.base_url);
    let openai_base_url = normalize_non_empty_value(parsed.openai_base_url);

    let mut model_providers = BTreeMap::new();
    if let Some(provider_name) = model_provider.as_deref() {
        let selected_base_url = require_selected_provider_base_url(
            &config_path,
            "model_providers",
            provider_name,
            parsed.model_providers.get(provider_name),
        )?;
        model_providers.insert(
            provider_name.to_string(),
            CodexBaseUrlConfig {
                base_url: Some(selected_base_url),
            },
        );
    }

    let mut providers = BTreeMap::new();
    if let Some(provider_name) = provider.as_deref() {
        let selected_base_url = require_selected_provider_base_url(
            &config_path,
            "providers",
            provider_name,
            parsed.providers.get(provider_name),
        )?;
        providers.insert(
            provider_name.to_string(),
            CodexBaseUrlConfig {
                base_url: Some(selected_base_url),
            },
        );
    }

    let projects = BTreeMap::from([(
        workspace_dir.display().to_string(),
        CodexProjectTrustConfig {
            trust_level: "untrusted".to_string(),
        },
    )]);

    Ok(CodexStartupSubset {
        model,
        model_provider,
        provider,
        base_url,
        openai_base_url,
        model_providers,
        providers,
        projects,
    })
}

fn ensure_supported_provider_routing_shape(
    config_path: &Path,
    routing_family: &str,
    provider_name: &str,
    selected: &CodexSeedProviderRoutingConfig,
) -> Result<()> {
    if selected.unsupported_fields.is_empty() {
        return Ok(());
    }

    let unsupported_fields = selected
        .unsupported_fields
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");

    Err(anyhow!(
        "direct cli:codex-world compatibility bridge found unsupported provider routing fields under [{routing_family}.{provider_name}] in {}; Packet 2 only supports base_url for the selected provider block, so isolated CODEX_HOME would otherwise receive incomplete routing config ({unsupported_fields})",
        config_path.display()
    ))
}

fn require_selected_provider_base_url(
    config_path: &Path,
    routing_family: &str,
    provider_name: &str,
    selected: Option<&CodexSeedProviderRoutingConfig>,
) -> Result<String> {
    let selected = selected.ok_or_else(|| {
        anyhow!(
            "direct cli:codex-world compatibility bridge could not derive truthful startup routing from {}; {routing_family}.{provider_name} is selected but its coupled provider block is missing, so isolated CODEX_HOME would otherwise receive an incomplete provider selection",
            config_path.display()
        )
    })?;
    ensure_supported_provider_routing_shape(config_path, routing_family, provider_name, selected)?;
    normalize_non_empty_value(selected.base_url.clone()).ok_or_else(|| {
        anyhow!(
            "direct cli:codex-world compatibility bridge could not derive truthful startup routing from {}; [{routing_family}.{provider_name}] must provide a supported non-empty base_url for the selected provider, so isolated CODEX_HOME would otherwise receive an incomplete provider selection",
            config_path.display()
        )
    })
}

fn normalize_non_empty_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn member_runtime_process_working_dir(
    backend_kind: MemberRuntimeBackendKindV1,
    workspace_dir: &Path,
    _launcher_dir: &Path,
) -> std::path::PathBuf {
    match backend_kind {
        // Keep Codex cwd aligned with the real workspace so repo-root and relative-path
        // semantics stay truthful, but mark that workspace untrusted in the isolated
        // user-level config so repo-local `.codex` layers cannot piggyback on this bridge.
        MemberRuntimeBackendKindV1::Codex => workspace_dir.to_path_buf(),
        MemberRuntimeBackendKindV1::ClaudeCode => workspace_dir.to_path_buf(),
    }
}

fn member_runtime_workspace_access_extensions(
    backend_kind: MemberRuntimeBackendKindV1,
    workspace_dir: &Path,
) -> BTreeMap<String, serde_json::Value> {
    match backend_kind {
        MemberRuntimeBackendKindV1::Codex => BTreeMap::from([(
            ADD_DIRS_EXTENSION_V1.to_string(),
            json!({
                "dirs": [workspace_dir.display().to_string()],
            }),
        )]),
        MemberRuntimeBackendKindV1::ClaudeCode => BTreeMap::new(),
    }
}

fn prepare_member_runtime_launcher(
    actual_binary_path: &Path,
    placement: &LinuxWorldPlacementContext,
) -> Result<PreparedMemberRuntimeLauncher> {
    let launcher_dir = std::env::temp_dir().join(format!(
        "substrate-member-runtime-entry-{}",
        uuid::Uuid::now_v7()
    ));
    let launcher = prepare_linux_world_entry_launcher(&launcher_dir, actual_binary_path, placement)
        .map_err(|err| anyhow!(err.to_string()))?;

    Ok(PreparedMemberRuntimeLauncher {
        launcher_path: launcher.launcher_path,
        launcher_dir,
        env: launcher.env,
    })
}

fn cleanup_prepared_launcher_dir(launcher_dir: &mut Option<std::path::PathBuf>) {
    if let Some(launcher_dir) = launcher_dir.take() {
        let _ = fs::remove_dir_all(launcher_dir);
    }
}

fn build_submitted_turn_run_request(
    active: &ActiveMemberRuntime,
    prompt: String,
    uaa_session_id: &str,
) -> AgentWrapperRunRequest {
    let mut extensions =
        member_runtime_workspace_access_extensions(active.backend_kind, &active.workspace_dir);
    extensions.insert(
        SESSION_RESUME_EXTENSION_V1.to_string(),
        json!({
            "selector": "id",
            "id": uaa_session_id,
        }),
    );

    AgentWrapperRunRequest {
        prompt,
        working_dir: Some(active.process_working_dir.clone()),
        timeout: None,
        env: active.env.clone(),
        extensions,
    }
}

#[allow(clippy::too_many_arguments)]
fn frame_from_wrapper_event(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    wrapper_event: AgentWrapperEvent,
    emitted_registered: &mut bool,
    mode: MemberStreamMode,
    agent_id: &str,
    producer: &mut RuntimeEventStreamProducer,
) -> Result<Option<ExecuteStreamFrame>> {
    let Some(event) = agent_event_from_wrapper_event(
        context,
        binding,
        span_id,
        wrapper_event,
        emitted_registered,
        mode,
        agent_id,
    ) else {
        return Ok(None);
    };
    producer.event(event).map(Some)
}

#[allow(clippy::too_many_arguments)]
fn frames_from_completion(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    completion: std::result::Result<AgentWrapperCompletion, AgentWrapperError>,
    cancel_signal: Option<String>,
    emitted_registered: &mut bool,
    mode: MemberStreamMode,
    agent_id: &str,
    producer: &mut RuntimeEventStreamProducer,
) -> Result<Vec<ExecuteStreamFrame>> {
    let frames = match completion {
        Ok(completion) => {
            let mut frames = Vec::new();
            if mode == MemberStreamMode::Bootstrap && !*emitted_registered {
                if let Some(event) = registered_event_from_data(
                    context,
                    binding,
                    span_id,
                    completion.data.as_ref(),
                    agent_id,
                ) {
                    *emitted_registered = true;
                    frames.push(producer.event(event)?);
                }
            }

            frames.push(producer.exit(
                exit_code_from_status(&completion.status),
                span_id.to_string(),
                Vec::new(),
                None,
                ProcessTelemetry::default(),
            )?);
            frames
        }
        Err(AgentWrapperError::Backend { message }) if message == CANCELLED_MESSAGE => {
            vec![producer.exit(
                cancel_exit_code(cancel_signal.as_deref()),
                span_id.to_string(),
                Vec::new(),
                None,
                ProcessTelemetry::default(),
            )?]
        }
        Err(err) => vec![producer.transport_error(format!("member runtime failed: {err}"))?],
    };
    Ok(frames)
}

fn agent_event_from_wrapper_event(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    wrapper_event: AgentWrapperEvent,
    emitted_registered: &mut bool,
    mode: MemberStreamMode,
    agent_id: &str,
) -> Option<AgentEvent> {
    let surfaced_thread_id = surfaced_uaa_thread_id_from_data(wrapper_event.data.as_ref());
    if mode == MemberStreamMode::Bootstrap {
        if let Some(event) = registered_event_from_data(
            context,
            binding,
            span_id,
            wrapper_event.data.as_ref(),
            agent_id,
        ) {
            *emitted_registered = true;
            return Some(event);
        }
    }

    let mut event = match wrapper_event.kind {
        AgentWrapperEventKind::Status => AgentEvent::message(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::Status,
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "member runtime status".to_string()),
        ),
        AgentWrapperEventKind::TextOutput => AgentEvent::message(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::TaskProgress,
            wrapper_event
                .text
                .clone()
                .unwrap_or_else(|| "member runtime output".to_string()),
        ),
        AgentWrapperEventKind::ToolCall | AgentWrapperEventKind::ToolResult => AgentEvent::message(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::TaskProgress,
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "member runtime tool activity".to_string()),
        ),
        AgentWrapperEventKind::Error => AgentEvent::alert(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            "agent_wrapper_error",
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "member runtime error".to_string()),
        ),
        AgentWrapperEventKind::Unknown => AgentEvent::message(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::TaskProgress,
            "member runtime emitted an unknown event".to_string(),
        ),
    };

    stamp_event_identity(
        &mut event,
        agent_id,
        context,
        binding,
        span_id,
        wrapper_event.channel,
    );
    event.thread_id = surfaced_thread_id;

    if let Some(data) = wrapper_event.data {
        if let Some(obj) = event.data.as_object_mut() {
            obj.insert("uaa_event".to_string(), data);
            obj.insert("protocol".to_string(), context.protocol.clone());
        }
    }

    Some(event)
}

fn registered_event_from_data(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    data: Option<&serde_json::Value>,
    agent_id: &str,
) -> Option<AgentEvent> {
    let data = data?;
    if data.get("schema").and_then(serde_json::Value::as_str) != Some(SESSION_HANDLE_SCHEMA_V1) {
        return None;
    }

    let mut event = AgentEvent {
        ts: chrono::Utc::now(),
        kind: AgentEventKind::Registered,
        data: data.clone(),
        agent_id: agent_id.to_string(),
        orchestration_session_id: context.orchestration_session_id.clone(),
        run_id: context.run_id.clone(),
        parent_run_id: None,
        participant_id: Some(context.participant_id.clone()),
        parent_participant_id: context.parent_participant_id.clone(),
        resumed_from_participant_id: context.resumed_from_participant_id.clone(),
        backend_id: Some(context.backend_id.clone()),
        thread_id: surfaced_uaa_thread_id_from_data(Some(data)),
        role: Some(MEMBER_ROLE.to_string()),
        world_id: Some(binding.world_id.clone()),
        world_generation: Some(binding.world_generation),
        cmd_id: None,
        span_id: Some(span_id.to_string()),
        event_identity: None,
        channel: None,
        identity_tuple: None,
        placement_posture: None,
        project: None,
    };
    event.set_pure_agent_telemetry_identity(agent_id.to_string());
    Some(event)
}

fn stamp_event_identity(
    event: &mut AgentEvent,
    agent_id: &str,
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    channel: Option<String>,
) {
    event.role = Some(MEMBER_ROLE.to_string());
    event.backend_id = Some(context.backend_id.clone());
    event.participant_id = Some(context.participant_id.clone());
    event.parent_participant_id = context.parent_participant_id.clone();
    event.resumed_from_participant_id = context.resumed_from_participant_id.clone();
    event.world_id = Some(binding.world_id.clone());
    event.world_generation = Some(binding.world_generation);
    event.span_id = Some(span_id.to_string());
    event.set_channel(channel);
    event.set_pure_agent_telemetry_identity(agent_id.to_string());
}

fn surfaced_uaa_session_id_from_data(data: Option<&serde_json::Value>) -> Option<String> {
    let data = data?;
    for pointer in ["/internal/uaa_session_id", "/session/id"] {
        if let Some(session_id) = data.pointer(pointer).and_then(serde_json::Value::as_str) {
            let trimmed = session_id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn surfaced_uaa_thread_id_from_data(data: Option<&serde_json::Value>) -> Option<String> {
    let data = data?;
    for pointer in [
        "/thread_id",
        "/session/id",
        "/raw_event/thread_id",
        "/raw_event/session/id",
    ] {
        if let Some(thread_id) = data.pointer(pointer).and_then(serde_json::Value::as_str) {
            let trimmed = thread_id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[derive(Clone, Copy)]
struct RetainedMemberIdentity<'a> {
    orchestration_session_id: &'a str,
    orchestrator_participant_id: &'a str,
    backend_id: &'a str,
    world_id: &'a str,
    world_generation: u64,
}

impl<'a> RetainedMemberIdentity<'a> {
    fn from_active(active: &'a ActiveMemberRuntime) -> Self {
        Self {
            orchestration_session_id: &active.orchestration_session_id,
            orchestrator_participant_id: &active.orchestrator_participant_id,
            backend_id: &active.backend_id,
            world_id: &active.binding.world_id,
            world_generation: active.binding.world_generation,
        }
    }
}

fn validate_submit_turn_request(
    req: &MemberTurnSubmitRequestV1,
    retained: RetainedMemberIdentity<'_>,
) -> Result<()> {
    if retained.orchestration_session_id != req.orchestration_session_id {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.orchestration_session_id mismatch (expected {}, got {})",
            retained.orchestration_session_id, req.orchestration_session_id
        ))
        .into());
    }
    if retained.orchestrator_participant_id != req.orchestrator_participant_id {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.orchestrator_participant_id mismatch (expected {}, got {})",
            retained.orchestrator_participant_id, req.orchestrator_participant_id
        ))
        .into());
    }
    if retained.backend_id != req.backend_id {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.backend_id mismatch (expected {}, got {})",
            retained.backend_id, req.backend_id
        ))
        .into());
    }
    if retained.world_id != req.world_id {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.world_id mismatch (expected {}, got {})",
            retained.world_id, req.world_id
        ))
        .into());
    }
    if retained.world_generation != req.world_generation {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.world_generation mismatch (expected {}, got {})",
            retained.world_generation, req.world_generation
        ))
        .into());
    }
    Ok(())
}

fn stream_response(
    rx: tokio::sync::mpsc::UnboundedReceiver<ExecuteStreamFrame>,
) -> Result<Response> {
    let stream = UnboundedReceiverStream::new(rx).map(|frame| {
        let payload = frame
            .canonical_ndjson_bytes()
            .expect("serialize identified member runtime frame");
        Ok::<Bytes, Infallible>(Bytes::from(payload))
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/x-ndjson")
        .body(boxed(StreamBody::new(stream)))
        .map_err(|err| anyhow!("failed to build member runtime stream response: {err}"))
}

fn validate_cancel_signal(sig: &str) -> Result<()> {
    match sig.trim().to_ascii_uppercase().as_str() {
        "INT" | "SIGINT" | "TERM" | "SIGTERM" | "HUP" | "SIGHUP" | "QUIT" | "SIGQUIT" => Ok(()),
        _ => Err(anyhow!("unsupported execute cancellation signal: {sig}")),
    }
}

fn cancel_exit_code(sig: Option<&str>) -> i32 {
    match sig.unwrap_or("INT").trim().to_ascii_uppercase().as_str() {
        "HUP" | "SIGHUP" => 129,
        "QUIT" | "SIGQUIT" => 131,
        "TERM" | "SIGTERM" => 143,
        _ => 130,
    }
}

fn exit_code_from_status(status: &std::process::ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        if let Some(code) = status.code() {
            return code;
        }
        if let Some(sig) = status.signal() {
            return 128 + sig;
        }
    }

    status.code().unwrap_or(1)
}

fn map_wrapper_error(err: AgentWrapperError) -> anyhow::Error {
    match err {
        AgentWrapperError::UnknownBackend { agent_kind } => {
            anyhow!("unsupported member runtime backend: {agent_kind}")
        }
        AgentWrapperError::UnknownRuntimeFamily { runtime_family } => {
            anyhow!("unsupported member runtime family: {runtime_family}")
        }
        AgentWrapperError::UnsupportedTargetTriple {
            runtime_family,
            target_triple,
        } => anyhow!(
            "member runtime target triple '{target_triple}' is unsupported for runtime family '{runtime_family}'"
        ),
        AgentWrapperError::MissingValidatedRuntime {
            runtime_family,
            target_triple,
        } => anyhow!(
            "member runtime target triple '{target_triple}' has no validated runtime for runtime family '{runtime_family}'"
        ),
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => anyhow!("member runtime backend {agent_kind} does not support {capability}"),
        AgentWrapperError::InvalidAgentKind { message }
        | AgentWrapperError::InvalidRequest { message }
        | AgentWrapperError::Backend { message } => anyhow!(message),
    }
}

impl ActiveMemberRuntime {
    fn remember_uaa_session_id(&self, session_id: String) {
        if let Ok(mut guard) = self.uaa_session_id.lock() {
            *guard = Some(session_id);
        }
    }

    fn uaa_session_id(&self) -> Option<String> {
        self.uaa_session_id
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    fn bootstrap_last_signal(&self) -> Option<String> {
        self.bootstrap.lock().ok().and_then(|guard| {
            guard
                .as_ref()
                .and_then(|bootstrap| bootstrap.last_signal.clone())
        })
    }

    fn has_active_bootstrap(&self) -> bool {
        self.bootstrap
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|_| ()))
            .is_some()
    }

    fn is_bootstrapping_or_resumable(&self) -> bool {
        self.has_active_bootstrap() || self.uaa_session_id().is_some()
    }

    fn cancel_bootstrap(&self) {
        if let Ok(guard) = self.bootstrap.lock() {
            if let Some(bootstrap) = guard.as_ref() {
                if let Some(cancel) = bootstrap.cancel.as_ref() {
                    cancel.cancel();
                }
            }
        }
    }

    fn cancel_bootstrap_if_span_matches(&self, span_id: &str, signal: &str) -> bool {
        let Ok(mut guard) = self.bootstrap.lock() else {
            return false;
        };
        let Some(bootstrap) = guard.as_mut() else {
            return false;
        };
        if bootstrap.span_id != span_id {
            return false;
        }

        bootstrap.last_signal = Some(signal.to_string());
        if let Some(cancel) = bootstrap.cancel.as_ref() {
            cancel.cancel();
        }
        true
    }

    fn close_bootstrap(&self) {
        if let Ok(mut guard) = self.bootstrap.lock() {
            guard.take();
        }
    }

    fn cleanup_launcher_dir(&self) {
        let _ = fs::remove_dir_all(&self.launcher_dir);
    }

    fn submit_context(
        &self,
        run_id: String,
        acceptance_context: Option<transport_api_types::WorldWorkAcceptanceContextV1>,
    ) -> MemberStreamContext {
        MemberStreamContext {
            orchestration_session_id: self.orchestration_session_id.clone(),
            run_id,
            acceptance_context,
            participant_id: self.participant_id.clone(),
            parent_participant_id: self.parent_participant_id.clone(),
            resumed_from_participant_id: self.resumed_from_participant_id.clone(),
            backend_id: self.backend_id.clone(),
            protocol: self.protocol.clone(),
        }
    }
}

impl MemberStreamContext {
    fn validate_acceptance_for_mode(&self, mode: MemberStreamMode) -> Result<()> {
        let Some(context) = self.acceptance_context.as_ref() else {
            return Ok(());
        };
        context
            .validate()
            .map_err(crate::service::BadRequestError::new)?;
        let message_shape_matches = match mode {
            MemberStreamMode::Bootstrap => context.message_id.is_none(),
            MemberStreamMode::SubmittedTurn => context.message_id.is_some(),
        };
        if context.request_id != self.run_id || !message_shape_matches {
            return Err(crate::service::BadRequestError::new(
                "member stream acceptance context does not match runtime request".to_string(),
            )
            .into());
        }
        Ok(())
    }
}

fn start_member_runtime_stream(
    context: &MemberStreamContext,
    mode: MemberStreamMode,
    span_id: String,
) -> Result<(RuntimeEventStreamProducer, ExecuteStreamFrame)> {
    context.validate_acceptance_for_mode(mode)?;
    let mut producer = RuntimeEventStreamProducer::new();
    let start = producer.start(span_id)?;
    Ok((producer, start))
}

impl ActiveSubmittedTurn {
    fn last_signal(&self) -> Option<String> {
        self.last_signal.lock().ok().and_then(|guard| guard.clone())
    }
}

impl RetainedMemberKey {
    fn from_active(active: &ActiveMemberRuntime) -> Self {
        Self {
            orchestration_session_id: active.orchestration_session_id.clone(),
            world_generation: active.binding.world_generation,
            backend_id: active.backend_id.clone(),
        }
    }

    fn from_submit(req: &MemberTurnSubmitRequestV1) -> Self {
        Self {
            orchestration_session_id: req.orchestration_session_id.clone(),
            world_generation: req.world_generation,
            backend_id: req.backend_id.clone(),
        }
    }
}

impl RetainedMemberSlot {
    fn new(primary_participant_id: String) -> Self {
        Self {
            primary_participant_id,
            fork_child_participant_id: None,
        }
    }

    fn contains(&self, participant_id: &str) -> bool {
        self.primary_participant_id == participant_id
            || self.fork_child_participant_id.as_deref() == Some(participant_id)
    }

    fn participant_ids(&self) -> BTreeSet<String> {
        let mut participant_ids = BTreeSet::from([self.primary_participant_id.clone()]);
        if let Some(fork_child_participant_id) = self.fork_child_participant_id.as_ref() {
            participant_ids.insert(fork_child_participant_id.clone());
        }
        participant_ids
    }

    fn stale_fork_child_participant_id(
        &self,
        active_members: &HashMap<String, Arc<ActiveMemberRuntime>>,
    ) -> Option<String> {
        let fork_child_participant_id = self.fork_child_participant_id.as_ref()?;
        match active_members.get(fork_child_participant_id) {
            Some(active) if active.is_bootstrapping_or_resumable() => None,
            Some(_) | None => Some(fork_child_participant_id.clone()),
        }
    }

    fn with_registered_fork_child(
        self,
        retained_key: &RetainedMemberKey,
        active_members: &HashMap<String, Arc<ActiveMemberRuntime>>,
        candidate: &ActiveMemberRuntime,
    ) -> Result<Self> {
        if self.fork_child_participant_id.is_some() {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        }

        let Some(parent_participant_id) = candidate.parent_participant_id.as_deref() else {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        };
        if parent_participant_id != self.primary_participant_id {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        }
        if candidate.resumed_from_participant_id.is_some() {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        }

        let Some(parent) = active_members.get(parent_participant_id) else {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        };
        if parent.orchestration_session_id != candidate.orchestration_session_id
            || parent.orchestrator_participant_id != candidate.orchestrator_participant_id
            || parent.backend_id != candidate.backend_id
            || parent.binding.world_id != candidate.binding.world_id
            || parent.binding.world_generation != candidate.binding.world_generation
        {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        }

        Ok(Self {
            primary_participant_id: self.primary_participant_id,
            fork_child_participant_id: Some(candidate.participant_id.clone()),
        })
    }

    fn without_participant(self, participant_id: &str) -> Option<Self> {
        if self.fork_child_participant_id.as_deref() == Some(participant_id) {
            return Some(Self {
                primary_participant_id: self.primary_participant_id,
                fork_child_participant_id: None,
            });
        }
        if self.primary_participant_id == participant_id {
            return self.fork_child_participant_id.map(RetainedMemberSlot::new);
        }
        Some(self)
    }
}

fn prune_stale_fork_child_slot(
    registry: &mut ActiveMemberRegistry,
    retained_key: &RetainedMemberKey,
    slot: RetainedMemberSlot,
) -> RetainedMemberSlot {
    let Some(stale_child_participant_id) =
        slot.stale_fork_child_participant_id(&registry.by_participant_id)
    else {
        return slot;
    };

    if let Some(stale_child) = registry
        .by_participant_id
        .remove(&stale_child_participant_id)
    {
        stale_child.cancel_bootstrap();
        stale_child.close_bootstrap();
        stale_child.cleanup_launcher_dir();
    }

    let pruned_slot = RetainedMemberSlot {
        primary_participant_id: slot.primary_participant_id,
        fork_child_participant_id: None,
    };
    registry
        .by_retained_key
        .insert(retained_key.clone(), pruned_slot.clone());
    pruned_slot
}

fn retained_slot_owner_mismatch_error(
    retained_key: &RetainedMemberKey,
    expected_participant_ids: &BTreeSet<String>,
    actual_participant_id: &str,
) -> crate::service::BadRequestError {
    let expected = if expected_participant_ids.len() == 1 {
        format!(
            "expected {}, got {}",
            expected_participant_ids
                .iter()
                .next()
                .expect("non-empty participant set"),
            actual_participant_id
        )
    } else {
        format!(
            "active participant_ids [{}], got {}",
            expected_participant_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", "),
            actual_participant_id
        )
    };
    crate::service::BadRequestError::new(format!(
        "member_turn_submit.participant_id mismatch for retained member orchestration_session_id {} world_generation {} backend_id {} ({expected})",
        retained_key.orchestration_session_id,
        retained_key.world_generation,
        retained_key.backend_id,
    ))
}

fn missing_retained_slot_error(
    retained_key: &RetainedMemberKey,
) -> crate::service::BadRequestError {
    crate::service::BadRequestError::new(format!(
        "member_turn_submit retained member is not active for orchestration_session_id {} world_generation {} backend_id {}",
        retained_key.orchestration_session_id,
        retained_key.world_generation,
        retained_key.backend_id,
    ))
}

fn retained_slot_registration_conflict_error(
    retained_key: &RetainedMemberKey,
    slot: &RetainedMemberSlot,
    candidate: &ActiveMemberRuntime,
) -> crate::service::BadRequestError {
    let active_participant_ids = slot
        .participant_ids()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    crate::service::BadRequestError::new(format!(
        "member_dispatch retained member slot conflict for orchestration_session_id {} world_generation {} backend_id {} (active participant_ids [{}], requested participant_id {}, parent_participant_id {:?}, resumed_from_participant_id {:?}); only a direct fork child of the current retained participant may co-register",
        retained_key.orchestration_session_id,
        retained_key.world_generation,
        retained_key.backend_id,
        active_participant_ids,
        candidate.participant_id,
        candidate.parent_participant_id,
        candidate.resumed_from_participant_id,
    ))
}

fn should_preserve_retained_member_after_clean_bootstrap(
    participant_id: &str,
    completion_status: &std::process::ExitStatus,
    has_uaa_session_id: bool,
) -> bool {
    // Only retained worker bootstraps (`ash_*`) may occupy the retained slot after
    // a clean bootstrap exit. Ephemeral `run_world_task` bootstraps (`awm_*`) can
    // surface resumable session identity, but they must still clean up immediately.
    participant_id.starts_with("ash_")
        && has_uaa_session_id
        && exit_code_from_status(completion_status) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sample_submit_turn_request() -> MemberTurnSubmitRequestV1 {
        MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch_123".to_string(),
            participant_id: "ash_member".to_string(),
            orchestrator_participant_id: "ash_orchestrator".to_string(),
            backend_id: "cli:codex".to_string(),
            run_id: "run_turn".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 7,
            prompt: "continue".to_string(),
            acceptance_context: None,
        }
    }

    fn sample_member_stream_context(message_id: Option<&str>) -> MemberStreamContext {
        MemberStreamContext {
            orchestration_session_id: "orch_123".to_string(),
            run_id: "run_turn".to_string(),
            acceptance_context: Some(transport_api_types::WorldWorkAcceptanceContextV1 {
                schema_version: 1,
                proposed_acceptance_record_id: "wwa_018f0f2e-7b4c-7aa1-8c22-123456789abc"
                    .to_string(),
                request_id: "run_turn".to_string(),
                message_id: message_id.map(str::to_string),
                caller_backend_id: "cli:codex".to_string(),
                host_transition_correlation: None,
            }),
            participant_id: "ash_member".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: json!("substrate.agent.session"),
        }
    }

    #[test]
    fn task_acceptance_context_is_retained_before_first_start_acknowledgement() {
        let context = sample_member_stream_context(None);
        let retained = context.acceptance_context.clone();
        let (_, start) = start_member_runtime_stream(
            &context,
            MemberStreamMode::Bootstrap,
            "spn_task_b1".to_string(),
        )
        .expect("start task stream after retaining exact context");
        assert_eq!(context.acceptance_context, retained);
        let ExecuteStreamFrame::Start {
            frame_identity,
            span_id,
        } = start
        else {
            panic!("first task acknowledgement must be Start");
        };
        assert_eq!(span_id, "spn_task_b1");
        assert_eq!(frame_identity.frame_sequence, 1);
    }

    #[test]
    fn retained_acceptance_context_requires_message_and_exact_run_before_start() {
        let context =
            sample_member_stream_context(Some("wwm_018f0f2e-7b4c-7aa1-8c22-123456789abd"));
        let retained = context.acceptance_context.clone();
        let (_, start) = start_member_runtime_stream(
            &context,
            MemberStreamMode::SubmittedTurn,
            "spn_turn_b1".to_string(),
        )
        .expect("start retained stream after retaining exact context");
        assert_eq!(context.acceptance_context, retained);
        assert!(matches!(
            start,
            ExecuteStreamFrame::Start {
                frame_identity,
                span_id,
            } if frame_identity.frame_sequence == 1 && span_id == "spn_turn_b1"
        ));

        let mut wrong_run = context.clone();
        wrong_run.run_id = "substituted-run".to_string();
        assert!(start_member_runtime_stream(
            &wrong_run,
            MemberStreamMode::SubmittedTurn,
            "spn_must_not_start".to_string(),
        )
        .is_err());
        assert!(start_member_runtime_stream(
            &sample_member_stream_context(None),
            MemberStreamMode::SubmittedTurn,
            "spn_must_not_start".to_string(),
        )
        .is_err());
    }

    fn sample_retained_identity() -> RetainedMemberIdentity<'static> {
        RetainedMemberIdentity {
            orchestration_session_id: "orch_123",
            orchestrator_participant_id: "ash_orchestrator",
            backend_id: "cli:codex",
            world_id: "world_123",
            world_generation: 7,
        }
    }

    fn sample_active_member_runtime(
        temp_dir: &tempfile::TempDir,
        participant_id: &str,
        bootstrap_span_id: &str,
    ) -> Arc<ActiveMemberRuntime> {
        sample_active_member_runtime_with_lineage(
            temp_dir,
            participant_id,
            bootstrap_span_id,
            None,
            None,
        )
    }

    fn sample_active_member_runtime_with_lineage(
        temp_dir: &tempfile::TempDir,
        participant_id: &str,
        bootstrap_span_id: &str,
        parent_participant_id: Option<&str>,
        resumed_from_participant_id: Option<&str>,
    ) -> Arc<ActiveMemberRuntime> {
        let workspace_dir = temp_dir.path().join("workspace");
        let process_working_dir = temp_dir.path().join("process");
        let binary_path = temp_dir.path().join("member-runtime");
        let launcher_dir = temp_dir.path().join("launcher");
        let codex_home = launcher_dir.join("codex-home");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::create_dir_all(&process_working_dir).expect("create process dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");
        fs::create_dir_all(&codex_home).expect("create codex home");
        fs::write(&binary_path, "#!/bin/sh\nexit 0\n").expect("write binary");

        Arc::new(ActiveMemberRuntime {
            agent_id: "codex_world".to_string(),
            participant_id: participant_id.to_string(),
            orchestration_session_id: "orch_123".to_string(),
            orchestrator_participant_id: "ash_orchestrator".to_string(),
            parent_participant_id: parent_participant_id.map(str::to_string),
            resumed_from_participant_id: resumed_from_participant_id.map(str::to_string),
            backend_id: "cli:codex".to_string(),
            backend_kind: MemberRuntimeBackendKindV1::Codex,
            binary_path,
            launcher_dir: launcher_dir.clone(),
            workspace_dir,
            process_working_dir,
            env: BTreeMap::from([("CODEX_HOME".to_string(), codex_home.display().to_string())]),
            binding: sample_world_binding(),
            protocol: json!("substrate.agent.session"),
            bootstrap: Mutex::new(Some(ActiveBootstrapRuntime {
                span_id: bootstrap_span_id.to_string(),
                cancel: None,
                last_signal: None,
            })),
            active_turn_span_id: Mutex::new(None),
            uaa_session_id: Mutex::new(None),
        })
    }

    #[test]
    fn prepare_codex_runtime_env_seeds_isolated_home_and_removes_internal_seed_env() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        let workspace_dir = temp_dir.path().join("workspace");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::write(
            seed_home.join("auth.json"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#,
        )
        .expect("write auth");
        fs::write(seed_home.join("config.toml"), "model = \"gpt-5.4\"\n")
            .expect("write startup config");
        fs::write(seed_home.join(".credentials.json"), "{}").expect("write credentials");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");
        let expected_codex_home = launcher_dir.join("codex-home").display().to_string();

        let mut runtime_env = BTreeMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            seed_home.display().to_string(),
        )]);

        prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect("seed auth");

        let codex_home = launcher_dir.join("codex-home");
        assert_eq!(
            runtime_env.get("CODEX_HOME").map(String::as_str),
            Some(expected_codex_home.as_str())
        );
        assert!(
            !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "internal seed env must be removed before spawning the member runtime"
        );
        assert_eq!(
            fs::read_to_string(codex_home.join("auth.json")).expect("seeded auth"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#
        );
        assert_eq!(
            fs::read_to_string(codex_home.join("config.toml")).expect("rendered config"),
            format!(
                "model = \"gpt-5.4\"\n\n[projects.\"{}\"]\ntrust_level = \"untrusted\"\n",
                workspace_dir.display()
            )
        );
        assert!(
            codex_home.join(".credentials.json").is_file(),
            "optional credentials file should be copied when present"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_materializes_bounded_user_startup_subset_only() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::write(
            seed_home.join("auth.json"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#,
        )
        .expect("write auth");
        fs::write(
            seed_home.join("config.toml"),
            r#"
model = "gpt-5.4"
model_provider = "compat-openai"
openai_base_url = "https://api.openai-proxy.example.invalid/v1"

[model_providers.compat-openai]
base_url = "https://gateway.example.invalid/v1"

[mcp_servers.hidden]
command = "should-not-copy"

[hooks.pre_exec]
command = "should-not-copy"

[agents.helper]
prompt = "should-not-copy"

[requirements]
mode = "should-not-copy"
"#,
        )
        .expect("write user config");
        fs::write(
            seed_home.join("engineering.config.toml"),
            "model = \"gpt-5.5\"\n",
        )
        .expect("write profile overlay");
        let workspace_dir = temp_dir.path().join("workspace");
        let project_codex_dir = workspace_dir.join(".codex");
        fs::create_dir_all(&project_codex_dir).expect("create project codex dir");
        fs::write(
            project_codex_dir.join("config.toml"),
            "model = \"gpt-5.6\"\n",
        )
        .expect("write project config");
        let launcher_dir = workspace_dir.join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let mut runtime_env = BTreeMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            seed_home.display().to_string(),
        )]);

        prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect("seed auth and bounded startup subset");

        let codex_home = launcher_dir.join("codex-home");
        let rendered = fs::read_to_string(codex_home.join("config.toml"))
            .expect("rendered bounded startup config");
        assert_eq!(
            rendered,
            format!(
                concat!(
                    "model = \"gpt-5.4\"\n",
                    "model_provider = \"compat-openai\"\n",
                    "openai_base_url = \"https://api.openai-proxy.example.invalid/v1\"\n\n",
                    "[model_providers.compat-openai]\n",
                    "base_url = \"https://gateway.example.invalid/v1\"\n\n",
                    "[projects.\"{}\"]\n",
                    "trust_level = \"untrusted\"\n"
                ),
                workspace_dir.display()
            )
        );
        assert!(
            !rendered.contains("mcp_servers"),
            "bounded startup subset must not project MCP configuration"
        );
        assert!(
            !rendered.contains("hooks"),
            "bounded startup subset must not project hook configuration"
        );
        assert!(
            !rendered.contains("agents"),
            "bounded startup subset must not project custom agent configuration"
        );
        assert!(
            !rendered.contains("requirements"),
            "bounded startup subset must not project managed requirements state"
        );
        let parsed: toml::Value = rendered
            .parse()
            .expect("parse rendered bounded startup config");
        assert_eq!(
            parsed
                .get("projects")
                .and_then(toml::Value::as_table)
                .and_then(|projects| projects.get(&workspace_dir.display().to_string()))
                .and_then(toml::Value::as_table)
                .and_then(|project| project.get("trust_level"))
                .and_then(toml::Value::as_str),
            Some("untrusted"),
            "isolated CODEX_HOME must mark the real workspace untrusted so repo-local .codex cannot apply"
        );
        assert!(
            !codex_home.join("engineering.config.toml").exists(),
            "Packet 2 must not replay profile overlays into isolated CODEX_HOME"
        );
        assert!(
            !codex_home.join(".codex").join("config.toml").exists(),
            "Packet 2 must not replay repo project config into isolated CODEX_HOME"
        );
        assert!(
            !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "internal seed env must be removed before spawning the member runtime"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_marks_authoritative_project_root_untrusted_when_override_is_present(
    ) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        let project_root = temp_dir.path().join("workspace");
        let nested_cwd = project_root.join("nested").join("pkg");
        let project_codex_dir = project_root.join(".codex");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::create_dir_all(&nested_cwd).expect("create nested cwd");
        fs::create_dir_all(&project_codex_dir).expect("create project codex dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");
        fs::write(
            seed_home.join("auth.json"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#,
        )
        .expect("write auth");
        fs::write(seed_home.join("config.toml"), "model = \"gpt-5.4\"\n")
            .expect("write startup config");
        fs::write(
            project_codex_dir.join("config.toml"),
            "model = \"gpt-5.6\"\n",
        )
        .expect("write project config");

        let mut runtime_env = BTreeMap::from([
            (
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                seed_home.display().to_string(),
            ),
            (
                crate::service::WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
                project_root.display().to_string(),
            ),
        ]);

        prepare_codex_runtime_env(&mut runtime_env, &nested_cwd, &launcher_dir)
            .expect("seed auth and bounded startup subset");

        let rendered = fs::read_to_string(launcher_dir.join("codex-home").join("config.toml"))
            .expect("rendered bounded startup config");
        let parsed: toml::Value = rendered.parse().expect("parse rendered config");
        let projects = parsed
            .get("projects")
            .and_then(toml::Value::as_table)
            .expect("rendered projects table");
        assert_eq!(
            projects.len(),
            1,
            "only the authoritative project root should be demoted in isolated CODEX_HOME"
        );
        assert_eq!(
            projects
                .get(&project_root.display().to_string())
                .and_then(toml::Value::as_table)
                .and_then(|project| project.get("trust_level"))
                .and_then(toml::Value::as_str),
            Some("untrusted"),
            "repo-root trust demotion must use SUBSTRATE_WORLD_PROJECT_DIR when launch cwd is nested"
        );
        assert!(
            !projects.contains_key(&nested_cwd.display().to_string()),
            "effective nested cwd must not become the trust-demotion key"
        );
        assert!(
            !launcher_dir
                .join("codex-home")
                .join(".codex")
                .join("config.toml")
                .exists(),
            "Packet 2 must still not replay repo project config into isolated CODEX_HOME"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_fails_closed_when_seed_home_bridge_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace_dir = temp_dir.path().join("workspace");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let mut runtime_env = BTreeMap::new();

        let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect_err("missing seed-home bridge must fail closed");
        let message = err.to_string();
        assert!(
            message.contains("policy-gated bootstrap seed source"),
            "unexpected error: {err:#}"
        );
        assert!(
            message.contains("Packet 2 seam"),
            "unexpected error: {err:#}"
        );
        assert!(
            message.contains("ambient repo-local .codex still active"),
            "unexpected error: {err:#}"
        );
        assert!(
            !message.contains(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "unexpected error leaked internal bridge env var name: {err:#}"
        );
        assert!(
            !runtime_env.contains_key("CODEX_HOME"),
            "fail-closed bridge errors must not materialize isolated CODEX_HOME"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_requires_auth_json_when_seed_home_is_declared() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        let workspace_dir = temp_dir.path().join("workspace");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let mut runtime_env = BTreeMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            seed_home.display().to_string(),
        )]);

        let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect_err("missing auth.json must fail closed");
        assert!(
            err.to_string().contains("auth.json"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_fails_closed_when_startup_model_cannot_be_derived() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        let workspace_dir = temp_dir.path().join("workspace");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::write(
            seed_home.join("auth.json"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#,
        )
        .expect("write auth");
        fs::write(
            seed_home.join("config.toml"),
            r#"
[model_providers.compat-openai]
base_url = "https://gateway.example.invalid/v1"
"#,
        )
        .expect("write incomplete config");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let mut runtime_env = BTreeMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            seed_home.display().to_string(),
        )]);

        let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect_err("missing startup model must fail closed");
        let message = err.to_string();
        assert!(
            message.contains("could not derive a bounded startup model"),
            "unexpected error: {err:#}"
        );
        assert!(
            message.contains("unsupported default model"),
            "unexpected error: {err:#}"
        );
        assert!(
            !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "internal seed env must still be removed on fail-closed startup subset errors"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_fails_closed_when_selected_provider_uses_unsupported_routing_shape(
    ) {
        for (selection, provider_table_header) in [
            (
                "model_provider = \"compat-openai\"",
                "[model_providers.compat-openai]",
            ),
            ("provider = \"compat-openai\"", "[providers.compat-openai]"),
        ] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let seed_home = temp_dir.path().join("seed-home");
            let workspace_dir = temp_dir.path().join("workspace");
            fs::create_dir_all(&seed_home).expect("create seed home");
            fs::create_dir_all(&workspace_dir).expect("create workspace dir");
            fs::write(
                seed_home.join("auth.json"),
                r#"{"account_id":"acct_test","access_token":"token_test"}"#,
            )
            .expect("write auth");
            fs::write(
                seed_home.join("config.toml"),
                format!(
                    "model = \"gpt-5.4\"\n{selection}\n\n{provider_table_header}\nbase_url = \"https://gateway.example.invalid/v1\"\nwire_api = \"responses\"\n",
                ),
            )
            .expect("write unsupported provider routing config");
            let launcher_dir = temp_dir.path().join("launcher");
            fs::create_dir_all(&launcher_dir).expect("create launcher dir");

            let mut runtime_env = BTreeMap::from([(
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                seed_home.display().to_string(),
            )]);

            let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
                .expect_err("unsupported selected provider routing shape must fail closed");
            let message = err.to_string();
            assert!(
                message.contains("unsupported provider routing fields"),
                "unexpected error: {err:#}"
            );
            assert!(message.contains("wire_api"), "unexpected error: {err:#}");
            assert!(
                !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
                "internal seed env must still be removed on fail-closed provider routing errors"
            );
        }
    }

    #[test]
    fn prepare_codex_runtime_env_fails_closed_when_selected_provider_block_is_missing_or_has_no_supported_base_url(
    ) {
        for (selection, provider_table_header, provider_body, expected_message_fragment) in [
            (
                "model_provider = \"compat-openai\"",
                "[model_providers.other-provider]",
                "base_url = \"https://gateway.example.invalid/v1\"\n",
                "coupled provider block is missing",
            ),
            (
                "provider = \"compat-openai\"",
                "[providers.compat-openai]",
                "",
                "must provide a supported non-empty base_url",
            ),
        ] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let seed_home = temp_dir.path().join("seed-home");
            let workspace_dir = temp_dir.path().join("workspace");
            fs::create_dir_all(&seed_home).expect("create seed home");
            fs::create_dir_all(&workspace_dir).expect("create workspace dir");
            fs::write(
                seed_home.join("auth.json"),
                r#"{"account_id":"acct_test","access_token":"token_test"}"#,
            )
            .expect("write auth");
            fs::write(
                seed_home.join("config.toml"),
                format!(
                    "model = \"gpt-5.4\"\n{selection}\n\n{provider_table_header}\n{provider_body}",
                ),
            )
            .expect("write incomplete provider routing config");
            let launcher_dir = temp_dir.path().join("launcher");
            fs::create_dir_all(&launcher_dir).expect("create launcher dir");

            let mut runtime_env = BTreeMap::from([(
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                seed_home.display().to_string(),
            )]);

            let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
                .expect_err("selected provider routing truth must fail closed");
            let message = err.to_string();
            assert!(
                message.contains(expected_message_fragment),
                "unexpected error: {err:#}"
            );
            assert!(
                !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
                "internal seed env must still be removed on fail-closed provider routing errors"
            );
        }
    }

    #[test]
    fn prepare_codex_runtime_env_cleanup_prepared_launcher_dir_removes_directory_once() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");
        fs::write(launcher_dir.join("auth.json"), "{}").expect("write launcher artifact");

        let mut cleanup_slot = Some(launcher_dir.clone());
        cleanup_prepared_launcher_dir(&mut cleanup_slot);
        cleanup_prepared_launcher_dir(&mut cleanup_slot);

        assert!(
            cleanup_slot.is_none(),
            "cleanup must clear the launcher slot"
        );
        assert!(
            !launcher_dir.exists(),
            "cleanup must remove prepared launcher artifacts on fail-closed exits"
        );
    }

    #[test]
    fn codex_member_runtime_launch_shape_keeps_workspace_cwd_and_explicit_workspace_add_dirs() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace_dir = temp_dir.path().join("workspace");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let process_working_dir = member_runtime_process_working_dir(
            MemberRuntimeBackendKindV1::Codex,
            &workspace_dir,
            &launcher_dir,
        );
        let extensions = member_runtime_workspace_access_extensions(
            MemberRuntimeBackendKindV1::Codex,
            &workspace_dir,
        );

        assert_eq!(
            process_working_dir, workspace_dir,
            "Codex retained sessions must preserve the real workspace cwd for truthful relative-path semantics"
        );
        assert_eq!(
            extensions.get(ADD_DIRS_EXTENSION_V1),
            Some(&json!({
                "dirs": [workspace_dir.display().to_string()],
            })),
            "Codex retained sessions must still declare workspace access explicitly via add_dirs"
        );
    }

    #[test]
    fn claude_member_runtime_launch_shape_keeps_workspace_cwd_without_add_dirs() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace_dir = temp_dir.path().join("workspace");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let process_working_dir = member_runtime_process_working_dir(
            MemberRuntimeBackendKindV1::ClaudeCode,
            &workspace_dir,
            &launcher_dir,
        );
        let extensions = member_runtime_workspace_access_extensions(
            MemberRuntimeBackendKindV1::ClaudeCode,
            &workspace_dir,
        );

        assert_eq!(
            process_working_dir, workspace_dir,
            "non-Codex backends should keep their existing workspace cwd behavior"
        );
        assert!(
            extensions.is_empty(),
            "non-Codex backends should not inherit Codex add_dirs behavior"
        );
    }

    #[test]
    fn surfaced_uaa_session_id_prefers_internal_then_session_id() {
        let payload = json!({
            "internal": {
                "uaa_session_id": "uaa_internal"
            },
            "session": {
                "id": "uaa_session"
            }
        });
        assert_eq!(
            surfaced_uaa_session_id_from_data(Some(&payload)).as_deref(),
            Some("uaa_internal")
        );

        let payload = json!({
            "session": {
                "id": "uaa_session"
            }
        });
        assert_eq!(
            surfaced_uaa_session_id_from_data(Some(&payload)).as_deref(),
            Some("uaa_session")
        );
    }

    #[test]
    fn surfaced_uaa_thread_id_prefers_explicit_thread_then_session_id() {
        let payload = json!({
            "thread_id": "thread-explicit",
            "session": {
                "id": "thread-session"
            }
        });
        assert_eq!(
            surfaced_uaa_thread_id_from_data(Some(&payload)).as_deref(),
            Some("thread-explicit")
        );

        let payload = json!({
            "session": {
                "id": "thread-session"
            }
        });
        assert_eq!(
            surfaced_uaa_thread_id_from_data(Some(&payload)).as_deref(),
            Some("thread-session")
        );
    }

    #[test]
    fn validate_submit_turn_request_accepts_matching_retained_identity() {
        validate_submit_turn_request(&sample_submit_turn_request(), sample_retained_identity())
            .expect("matching retained member identity should validate");
    }

    type SubmitTurnDriftCase = (
        &'static str,
        fn(&mut MemberTurnSubmitRequestV1),
        &'static str,
    );

    #[test]
    fn validate_submit_turn_request_rejects_retained_identity_drift() {
        let cases: [SubmitTurnDriftCase; 5] = [
            (
                "orchestration_session_id",
                |req| req.orchestration_session_id = "orch_999".to_string(),
                "member_turn_submit.orchestration_session_id mismatch",
            ),
            (
                "orchestrator_participant_id",
                |req| req.orchestrator_participant_id = "ash_orchestrator_other".to_string(),
                "member_turn_submit.orchestrator_participant_id mismatch",
            ),
            (
                "backend_id",
                |req| req.backend_id = "cli:anthropic".to_string(),
                "member_turn_submit.backend_id mismatch",
            ),
            (
                "world_id",
                |req| req.world_id = "world_999".to_string(),
                "member_turn_submit.world_id mismatch",
            ),
            (
                "world_generation",
                |req| req.world_generation = 9,
                "member_turn_submit.world_generation mismatch",
            ),
        ];

        for (field, mutate, expected) in cases {
            let mut req = sample_submit_turn_request();
            mutate(&mut req);

            let err = match validate_submit_turn_request(&req, sample_retained_identity()) {
                Ok(()) => panic!("expected {field} drift to be rejected"),
                Err(err) => err,
            };
            assert!(
                err.to_string().contains(expected),
                "expected {field} drift to mention {expected}, got: {err}"
            );
        }
    }

    #[test]
    fn retained_member_key_uses_session_generation_and_backend_id() {
        let req = sample_submit_turn_request();

        assert_eq!(
            RetainedMemberKey::from_submit(&req),
            RetainedMemberKey {
                orchestration_session_id: "orch_123".to_string(),
                world_generation: 7,
                backend_id: "cli:codex".to_string(),
            }
        );
    }

    fn sample_stream_context() -> MemberStreamContext {
        MemberStreamContext {
            orchestration_session_id: "orch_123".to_string(),
            run_id: "run_bootstrap".to_string(),
            acceptance_context: None,
            participant_id: "ash_member".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: json!("substrate.agent.session"),
        }
    }

    fn sample_world_binding() -> SharedWorldBindingSnapshot {
        SharedWorldBindingSnapshot {
            orchestration_session_id: "orch_123".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 7,
            binding_state: world_api::SharedWorldBindingState::Active,
        }
    }

    #[test]
    fn bootstrap_completion_without_session_handle_emits_only_exit() {
        let mut emitted_registered = false;
        let mut producer = RuntimeEventStreamProducer::new();
        producer.start("spn_bootstrap".to_string()).expect("Start");
        let frames = frames_from_completion(
            &sample_stream_context(),
            &sample_world_binding(),
            "spn_bootstrap",
            Ok(AgentWrapperCompletion {
                status: std::process::Command::new("sh")
                    .arg("-c")
                    .arg("exit 0")
                    .status()
                    .expect("completion status"),
                final_text: None,
                data: None,
            }),
            None,
            &mut emitted_registered,
            MemberStreamMode::Bootstrap,
            "codex_world",
            &mut producer,
        )
        .expect("completion frames");

        assert_eq!(
            frames.len(),
            1,
            "bootstrap without continuity metadata must stay terminal"
        );
        assert!(matches!(
            frames.first(),
            Some(ExecuteStreamFrame::Exit { exit: 0, .. })
        ));
        assert!(
            !emitted_registered,
            "non-retained bootstrap should not invent a registered event"
        );
    }

    #[test]
    fn registered_event_surfaces_authoritative_bootstrap_identity() {
        let event = registered_event_from_data(
            &sample_stream_context(),
            &sample_world_binding(),
            "spn_bootstrap",
            Some(&json!({
                "schema": SESSION_HANDLE_SCHEMA_V1,
                "session": {
                    "id": "uaa_session"
                }
            })),
            "codex_world",
        )
        .expect("registered event");

        assert_eq!(event.kind, AgentEventKind::Registered);
        assert_eq!(event.participant_id.as_deref(), Some("ash_member"));
        assert_eq!(event.parent_participant_id, None);
        assert_eq!(event.resumed_from_participant_id, None);
        assert_eq!(event.backend_id.as_deref(), Some("cli:codex"));
        assert_eq!(event.thread_id.as_deref(), Some("uaa_session"));
        assert_eq!(event.world_id.as_deref(), Some("world_123"));
        assert_eq!(event.world_generation, Some(7));
        assert_eq!(event.span_id.as_deref(), Some("spn_bootstrap"));
    }

    #[test]
    fn submitted_turn_event_surfaces_thread_id_from_uaa_payload() {
        let wrapper_event = AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
            kind: AgentWrapperEventKind::Status,
            channel: Some("status".to_string()),
            text: None,
            message: Some("submitted turn".to_string()),
            data: Some(json!({
                "thread_id": "thread-submitted",
                "turn_id": "turn-2"
            })),
        };

        let event = agent_event_from_wrapper_event(
            &sample_stream_context(),
            &sample_world_binding(),
            "spn_submitted",
            wrapper_event,
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        )
        .expect("submitted turn event");

        assert_eq!(event.thread_id.as_deref(), Some("thread-submitted"));
        assert_eq!(event.participant_id.as_deref(), Some("ash_member"));
        assert_eq!(event.backend_id.as_deref(), Some("cli:codex"));
        assert_eq!(event.world_id.as_deref(), Some("world_123"));
        assert_eq!(event.world_generation, Some(7));
    }

    #[test]
    fn bootstrap_completion_with_session_handle_emits_registered_then_exit() {
        let mut emitted_registered = false;
        let mut producer = RuntimeEventStreamProducer::new();
        let start = producer.start("spn_bootstrap".to_string()).expect("Start");
        let frames = frames_from_completion(
            &sample_stream_context(),
            &sample_world_binding(),
            "spn_bootstrap",
            Ok(AgentWrapperCompletion {
                status: std::process::Command::new("sh")
                    .arg("-c")
                    .arg("exit 0")
                    .status()
                    .expect("completion status"),
                final_text: None,
                data: Some(json!({
                    "schema": SESSION_HANDLE_SCHEMA_V1,
                    "session": {
                        "id": "uaa_session"
                    }
                })),
            }),
            None,
            &mut emitted_registered,
            MemberStreamMode::Bootstrap,
            "codex_world",
            &mut producer,
        )
        .expect("completion frames");

        assert_eq!(
            frames.len(),
            2,
            "bootstrap with continuity metadata must surface registration before exit"
        );
        assert!(matches!(
            frames.first(),
            Some(ExecuteStreamFrame::Event { event, .. }) if event.kind == AgentEventKind::Registered
        ));
        assert!(matches!(
            frames.get(1),
            Some(ExecuteStreamFrame::Exit { exit: 0, .. })
        ));
        assert!(
            emitted_registered,
            "registered event tracking must be updated"
        );

        let start_identity = match start {
            ExecuteStreamFrame::Start { frame_identity, .. } => frame_identity,
            other => panic!("unexpected frame: {other:?}"),
        };
        let (event_frame_identity, event_identity) = match &frames[0] {
            ExecuteStreamFrame::Event {
                frame_identity,
                event,
            } => (
                frame_identity,
                event.event_identity.as_ref().expect("event identity"),
            ),
            other => panic!("unexpected frame: {other:?}"),
        };
        let (exit_frame_identity, exit_event_identity, terminal_identity) = match &frames[1] {
            ExecuteStreamFrame::Exit {
                frame_identity,
                event_identity,
                terminal_identity,
                ..
            } => (frame_identity, event_identity, terminal_identity),
            other => panic!("unexpected frame: {other:?}"),
        };
        assert_eq!(start_identity.frame_sequence, 1);
        assert_eq!(event_frame_identity.frame_sequence, 2);
        assert_eq!(exit_frame_identity.frame_sequence, 3);
        assert_eq!(event_identity.event_sequence, 1);
        assert_eq!(exit_event_identity.event_sequence, 2);
        assert_eq!(event_frame_identity.stream_id, start_identity.stream_id);
        assert_eq!(exit_frame_identity.stream_id, start_identity.stream_id);
        assert!(terminal_identity.matches_event(exit_event_identity));
        assert!(producer
            .transport_error("late transport error".to_string())
            .is_err());
    }

    #[test]
    fn retained_slot_owner_mismatch_error_mentions_expected_participant() {
        let err = retained_slot_owner_mismatch_error(
            &RetainedMemberKey {
                orchestration_session_id: "orch_123".to_string(),
                world_generation: 7,
                backend_id: "cli:codex".to_string(),
            },
            &BTreeSet::from(["ash_member_existing".to_string()]),
            "ash_member_other",
        );

        assert!(
            err.to_string()
                .contains("member_turn_submit.participant_id mismatch"),
            "unexpected error: {err}"
        );
        assert!(
            err.to_string()
                .contains("expected ash_member_existing, got ash_member_other"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_submit_target_rejects_participant_id_drift_for_retained_slot() {
        let manager = MemberRuntimeManager::new();
        let retained_key = RetainedMemberKey {
            orchestration_session_id: "orch_123".to_string(),
            world_generation: 7,
            backend_id: "cli:codex".to_string(),
        };
        manager
            .active_members
            .write()
            .expect("member runtime registry lock poisoned")
            .by_retained_key
            .insert(
                retained_key,
                RetainedMemberSlot::new("ash_member_existing".to_string()),
            );

        let mut req = sample_submit_turn_request();
        req.participant_id = "ash_member_other".to_string();

        let err = match manager.find_submit_target(&req) {
            Ok(_) => panic!("participant drift should be rejected"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("member_turn_submit.participant_id mismatch"),
            "unexpected error: {err}"
        );
        assert!(
            err.to_string()
                .contains("expected ash_member_existing, got ash_member_other"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_submit_target_slot_accepts_retained_fork_child_in_shared_slot() {
        let manager = MemberRuntimeManager::new();
        manager
            .active_members
            .write()
            .expect("member runtime registry lock poisoned")
            .by_retained_key
            .insert(
                RetainedMemberKey {
                    orchestration_session_id: "orch_123".to_string(),
                    world_generation: 7,
                    backend_id: "cli:codex".to_string(),
                },
                RetainedMemberSlot {
                    primary_participant_id: "ash_member_source".to_string(),
                    fork_child_participant_id: Some("ash_member_child".to_string()),
                },
            );

        let mut req = sample_submit_turn_request();
        req.participant_id = "ash_member_child".to_string();

        manager
            .validate_submit_target_slot(&req)
            .expect("source plus direct fork child should accept the exact participant");
    }

    #[test]
    fn finish_bootstrap_preserves_retained_slot_when_session_handle_exists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_active_member_runtime(&temp_dir, "ash_member", "spn_bootstrap");
        active.remember_uaa_session_id("uaa_session".to_string());
        let manager = MemberRuntimeManager::new();
        manager
            .register_member(active.clone())
            .expect("register retained member");

        manager.finish_bootstrap(&active.participant_id, true);

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            guard.by_participant_id.contains_key(&active.participant_id),
            "retained member should survive clean bootstrap exit once resumable identity exists"
        );
        assert_eq!(
            guard
                .by_retained_key
                .get(&RetainedMemberKey::from_active(active.as_ref())),
            Some(&RetainedMemberSlot::new(active.participant_id.clone())),
            "retained slot ownership should stay exact after bootstrap cleanup"
        );
        drop(guard);
        assert!(
            active.bootstrap.lock().expect("bootstrap lock").is_none(),
            "preserved retained member should infer parked truth by clearing the active bootstrap slot"
        );
        assert!(
            temp_dir.path().join("launcher").exists(),
            "parked retained workers must keep launcher artifacts for later submit_turn resume"
        );
        assert!(
            temp_dir.path().join("launcher").join("codex-home").exists(),
            "parked retained workers must keep isolated CODEX_HOME for later resume"
        );
    }

    #[test]
    fn finish_bootstrap_unregisters_member_without_resumable_identity() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_active_member_runtime(&temp_dir, "ash_member", "spn_bootstrap");
        let manager = MemberRuntimeManager::new();
        manager
            .register_member(active.clone())
            .expect("register retained member");

        manager.finish_bootstrap(&active.participant_id, false);

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard.by_participant_id.contains_key(&active.participant_id),
            "bootstrap without resumable identity must still close out retained continuity"
        );
        assert!(
            !guard
                .by_retained_key
                .contains_key(&RetainedMemberKey::from_active(active.as_ref())),
            "retained slot must be removed when continuity was never surfaced"
        );
        drop(guard);
        assert!(
            active.bootstrap.lock().expect("bootstrap lock").is_none(),
            "terminal bootstrap cleanup should also clear the active bootstrap slot"
        );
        assert!(
            !temp_dir.path().join("launcher").exists(),
            "terminal bootstrap cleanup should still remove launcher artifacts"
        );
    }

    #[test]
    fn finish_bootstrap_unregisters_ephemeral_member_even_with_resumable_identity() {
        use std::os::unix::process::ExitStatusExt;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_active_member_runtime(&temp_dir, "awm_task", "spn_bootstrap");
        active.remember_uaa_session_id("uaa_session".to_string());
        let manager = MemberRuntimeManager::new();
        manager
            .register_member(active.clone())
            .expect("register ephemeral member");

        manager.finish_bootstrap(
            &active.participant_id,
            should_preserve_retained_member_after_clean_bootstrap(
                &active.participant_id,
                &std::process::ExitStatus::from_raw(0),
                active.uaa_session_id().is_some(),
            ),
        );

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard.by_participant_id.contains_key(&active.participant_id),
            "ephemeral run_world_task members must not remain retained after bootstrap exit"
        );
        assert!(
            !guard
                .by_retained_key
                .contains_key(&RetainedMemberKey::from_active(active.as_ref())),
            "ephemeral run_world_task members must not occupy retained slot identity"
        );
        drop(guard);
        assert!(
            !temp_dir.path().join("launcher").exists(),
            "ephemeral bootstrap cleanup should still remove launcher artifacts"
        );
    }

    #[test]
    fn register_member_allows_direct_fork_child_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let child_dir = tempfile::tempdir().expect("child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let child = sample_active_member_runtime_with_lineage(
            &child_dir,
            "ash_member_child",
            "spn_child",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source.clone())
            .expect("register source retained member");
        manager
            .register_member(child.clone())
            .expect("register child retained member in shared slot");

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            guard.by_participant_id.contains_key(&source.participant_id),
            "source retained member should stay registered"
        );
        assert!(
            guard.by_participant_id.contains_key(&child.participant_id),
            "child retained member should stay registered"
        );
        assert_eq!(
            guard
                .by_retained_key
                .get(&RetainedMemberKey::from_active(source.as_ref())),
            Some(&RetainedMemberSlot {
                primary_participant_id: source.participant_id.clone(),
                fork_child_participant_id: Some(child.participant_id.clone()),
            }),
            "shared retained slot should only track the source plus its direct fork child"
        );
    }

    #[test]
    fn register_member_rejects_unrelated_duplicate_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let duplicate_dir = tempfile::tempdir().expect("duplicate temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let duplicate =
            sample_active_member_runtime(&duplicate_dir, "ash_member_duplicate", "spn_duplicate");
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source)
            .expect("register source retained member");
        let err = match manager.register_member(duplicate) {
            Ok(_) => panic!("unrelated same-slot retained registration should fail closed"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("member_dispatch retained member slot conflict"),
            "unexpected error: {err}"
        );
        assert!(
            err.to_string().contains(
                "only a direct fork child of the current retained participant may co-register"
            ),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn register_member_replaces_stale_fork_child_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let stale_child_dir = tempfile::tempdir().expect("stale child temp dir");
        let replacement_child_dir = tempfile::tempdir().expect("replacement child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let stale_child = sample_active_member_runtime_with_lineage(
            &stale_child_dir,
            "ash_member_child_stale",
            "spn_child_stale",
            Some("ash_member_source"),
            None,
        );
        let replacement_child = sample_active_member_runtime_with_lineage(
            &replacement_child_dir,
            "ash_member_child_replacement",
            "spn_child_replacement",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source.clone())
            .expect("register source retained member");
        manager
            .register_member(stale_child.clone())
            .expect("register stale child retained member");

        stale_child.close_bootstrap();

        manager
            .register_member(replacement_child.clone())
            .expect("stale child should not block replacement registration");

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard
                .by_participant_id
                .contains_key(&stale_child.participant_id),
            "stale child should be evicted from active members"
        );
        assert!(
            guard
                .by_participant_id
                .contains_key(&replacement_child.participant_id),
            "replacement child should become the retained fork child"
        );
        assert_eq!(
            guard
                .by_retained_key
                .get(&RetainedMemberKey::from_active(source.as_ref())),
            Some(&RetainedMemberSlot {
                primary_participant_id: source.participant_id.clone(),
                fork_child_participant_id: Some(replacement_child.participant_id.clone()),
            }),
            "shared retained slot should track the replacement child after stale eviction"
        );
        assert!(
            !stale_child.launcher_dir.exists(),
            "stale child eviction should clean launcher artifacts"
        );
    }

    #[test]
    fn register_member_replaces_missing_stale_fork_child_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let missing_child_dir = tempfile::tempdir().expect("missing child temp dir");
        let replacement_child_dir = tempfile::tempdir().expect("replacement child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let missing_child = sample_active_member_runtime_with_lineage(
            &missing_child_dir,
            "ash_member_child_missing",
            "spn_child_missing",
            Some("ash_member_source"),
            None,
        );
        let replacement_child = sample_active_member_runtime_with_lineage(
            &replacement_child_dir,
            "ash_member_child_replacement",
            "spn_child_replacement",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();
        let retained_key = RetainedMemberKey::from_active(source.as_ref());

        manager
            .register_member(source.clone())
            .expect("register source retained member");
        manager
            .register_member(missing_child.clone())
            .expect("register missing child retained member");

        {
            let mut guard = manager
                .active_members
                .write()
                .expect("member runtime registry lock poisoned");
            guard
                .by_participant_id
                .remove(&missing_child.participant_id)
                .expect("missing child should start in active members");
            assert_eq!(
                guard.by_retained_key.get(&retained_key),
                Some(&RetainedMemberSlot {
                    primary_participant_id: source.participant_id.clone(),
                    fork_child_participant_id: Some(missing_child.participant_id.clone()),
                }),
                "test setup should leave a stale fork-child slot behind"
            );
        }

        manager
            .register_member(replacement_child.clone())
            .expect("missing stale child should not block replacement registration");

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard
                .by_participant_id
                .contains_key(&missing_child.participant_id),
            "missing child should stay absent from active members"
        );
        assert!(
            guard
                .by_participant_id
                .contains_key(&replacement_child.participant_id),
            "replacement child should become the retained fork child"
        );
        assert_eq!(
            guard.by_retained_key.get(&retained_key),
            Some(&RetainedMemberSlot {
                primary_participant_id: source.participant_id.clone(),
                fork_child_participant_id: Some(replacement_child.participant_id.clone()),
            }),
            "shared retained slot should repair itself when the stale child entry is missing"
        );
    }

    #[test]
    fn register_member_rejects_second_live_fork_child_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let first_child_dir = tempfile::tempdir().expect("first child temp dir");
        let second_child_dir = tempfile::tempdir().expect("second child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let first_child = sample_active_member_runtime_with_lineage(
            &first_child_dir,
            "ash_member_child_first",
            "spn_child_first",
            Some("ash_member_source"),
            None,
        );
        let second_child = sample_active_member_runtime_with_lineage(
            &second_child_dir,
            "ash_member_child_second",
            "spn_child_second",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source)
            .expect("register source retained member");
        manager
            .register_member(first_child)
            .expect("register first child retained member");
        let err = match manager.register_member(second_child) {
            Ok(_) => panic!("a live fork child should still block a second child"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("member_dispatch retained member slot conflict"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn unregister_member_preserves_direct_fork_child_slot_until_last_participant_leaves() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let child_dir = tempfile::tempdir().expect("child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let child = sample_active_member_runtime_with_lineage(
            &child_dir,
            "ash_member_child",
            "spn_child",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source.clone())
            .expect("register source retained member");
        manager
            .register_member(child.clone())
            .expect("register child retained member");

        manager.unregister_member(&source.participant_id);

        let retained_key = RetainedMemberKey::from_active(child.as_ref());
        {
            let guard = manager
                .active_members
                .read()
                .expect("member runtime registry lock poisoned");
            assert!(
                !guard.by_participant_id.contains_key(&source.participant_id),
                "source retained member should be removed"
            );
            assert!(
                guard.by_participant_id.contains_key(&child.participant_id),
                "child retained member should remain registered"
            );
            assert_eq!(
                guard.by_retained_key.get(&retained_key),
                Some(&RetainedMemberSlot::new(child.participant_id.clone())),
                "shared retained slot should promote the direct fork child once the source leaves"
            );
        }

        manager.unregister_member(&child.participant_id);

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard.by_retained_key.contains_key(&retained_key),
            "shared retained slot should clear once the last participant leaves"
        );
    }
}
