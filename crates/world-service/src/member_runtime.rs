use agent_api::{
    AgentWrapperCancelHandle, AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent,
    AgentWrapperEventKind, AgentWrapperRunControl, AgentWrapperRunRequest,
};
use anyhow::{anyhow, Result};
use axum::{
    body::{boxed, Bytes, StreamBody},
    http::StatusCode,
    response::Response,
};
use futures_util::StreamExt;
use serde_json::json;
use std::{
    collections::{BTreeMap, HashMap},
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

const MEMBER_ROLE: &str = "member";
const SESSION_HANDLE_SCHEMA_V1: &str = "agent_api.session.handle.v1";
const CANCELLED_MESSAGE: &str = "cancelled";
const SESSION_RESUME_EXTENSION_V1: &str = "agent_api.session.resume.v1";

#[derive(Clone, Default)]
pub(crate) struct MemberRuntimeManager {
    active_members: Arc<RwLock<ActiveMemberRegistry>>,
    active_turns_by_span_id: Arc<RwLock<HashMap<String, Arc<ActiveSubmittedTurn>>>>,
}

#[derive(Default)]
struct ActiveMemberRegistry {
    by_participant_id: HashMap<String, Arc<ActiveMemberRuntime>>,
    by_retained_key: HashMap<RetainedMemberKey, String>,
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
    working_dir: std::path::PathBuf,
    env: BTreeMap<String, String>,
    binding: SharedWorldBindingSnapshot,
    protocol: serde_json::Value,
    bootstrap_span_id: String,
    bootstrap_cancel: AgentWrapperCancelHandle,
    bootstrap_last_signal: Mutex<Option<String>>,
    active_turn_span_id: Mutex<Option<String>>,
    uaa_session_id: Mutex<Option<String>>,
    launcher_dir: Option<std::path::PathBuf>,
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

impl MemberRuntimeManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn launch(
        &self,
        agent_id: String,
        env: HashMap<String, String>,
        span_id: String,
        dispatch: MemberDispatchRequestV1,
        binding: SharedWorldBindingSnapshot,
        placement: LinuxWorldPlacementContext,
    ) -> Result<Response> {
        let actual_binary = validate_member_runtime_binary(&dispatch)?;
        let prepared_launcher = prepare_member_runtime_launcher(&actual_binary, &placement)?;
        let runtime_env = env
            .into_iter()
            .chain(prepared_launcher.env.iter().cloned())
            .collect::<BTreeMap<_, _>>();

        let prompt_fulfillment = PromptFulfillmentBridge::for_member_backend(
            &dispatch.resolved_runtime.backend_kind,
            prepared_launcher.launcher_path.clone(),
        )?;
        let initial_prompt = dispatch.initial_prompt.clone().ok_or_else(|| {
            crate::service::BadRequestError::new(
                "member_dispatch.initial_prompt is required for launch-time first turn".to_string(),
            )
        })?;
        let AgentWrapperRunControl { handle, cancel } = match prompt_fulfillment
            .run_control(AgentWrapperRunRequest {
                prompt: initial_prompt,
                working_dir: Some(placement.working_dir.clone()),
                timeout: None,
                env: runtime_env.clone(),
                extensions: BTreeMap::new(),
            })
            .await
        {
            Ok(control) => control,
            Err(err) => {
                let _ = fs::remove_dir_all(&prepared_launcher.launcher_dir);
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
            working_dir: placement.working_dir.clone(),
            env: runtime_env,
            binding: binding.clone(),
            protocol: json!(dispatch.protocol),
            bootstrap_span_id: span_id.clone(),
            bootstrap_cancel: cancel,
            bootstrap_last_signal: Mutex::new(None),
            active_turn_span_id: Mutex::new(None),
            uaa_session_id: Mutex::new(None),
            launcher_dir: Some(prepared_launcher.launcher_dir),
        });
        if let Err(err) = self.register_member(active.clone()) {
            active.bootstrap_cancel.cancel();
            if let Some(launcher_dir) = active.launcher_dir.as_ref() {
                let _ = fs::remove_dir_all(launcher_dir);
            }
            return Err(err);
        }

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(ExecuteStreamFrame::Start {
            span_id: span_id.clone(),
        });

        let manager = self.clone();
        let participant_id = dispatch.participant_id.clone();
        let context = MemberStreamContext {
            orchestration_session_id: dispatch.orchestration_session_id.clone(),
            run_id: dispatch.run_id.clone(),
            participant_id: dispatch.participant_id.clone(),
            parent_participant_id: dispatch.parent_participant_id.clone(),
            resumed_from_participant_id: dispatch.resumed_from_participant_id.clone(),
            backend_id: dispatch.backend_id.clone(),
            protocol: json!(dispatch.protocol),
        };
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
                if let Some(frame) = frame_from_wrapper_event(
                    &context,
                    &binding,
                    &span_id,
                    wrapper_event,
                    &mut emitted_registered,
                    MemberStreamMode::Bootstrap,
                    active.agent_id.as_str(),
                ) {
                    let _ = tx.send(frame);
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
            for frame in frames_from_completion(
                &context,
                &binding,
                &span_id,
                completion,
                active.bootstrap_last_signal(),
                &mut emitted_registered,
                MemberStreamMode::Bootstrap,
                active.agent_id.as_str(),
            ) {
                let _ = tx.send(frame);
            }

            manager.unregister_member(&participant_id);
        });

        stream_response(rx)
    }

    pub(crate) async fn submit_turn(&self, req: MemberTurnSubmitRequestV1) -> Result<Response> {
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
        let mut extensions = BTreeMap::new();
        extensions.insert(
            SESSION_RESUME_EXTENSION_V1.to_string(),
            json!({
                "selector": "id",
                "id": uaa_session_id,
            }),
        );

        let AgentWrapperRunControl { handle, cancel } = match prompt_fulfillment
            .run_control(AgentWrapperRunRequest {
                prompt: req.prompt.clone(),
                working_dir: Some(active.working_dir.clone()),
                timeout: None,
                env: active.env.clone(),
                extensions,
            })
            .await
        {
            Ok(control) => control,
            Err(err) => {
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(map_wrapper_error(err));
            }
        };

        let turn = Arc::new(ActiveSubmittedTurn {
            participant_id: active.participant_id.clone(),
            cancel,
            last_signal: Mutex::new(None),
        });
        self.register_turn(span_id.clone(), turn.clone());

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(ExecuteStreamFrame::Start {
            span_id: span_id.clone(),
        });

        let manager = self.clone();
        let context = active.submit_context(req.run_id.clone());
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
                if let Some(frame) = frame_from_wrapper_event(
                    &context,
                    &binding,
                    &span_id,
                    wrapper_event,
                    &mut emitted_registered,
                    MemberStreamMode::SubmittedTurn,
                    active.agent_id.as_str(),
                ) {
                    let _ = tx.send(frame);
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
            for frame in frames_from_completion(
                &context,
                &binding,
                &span_id,
                completion,
                turn.last_signal(),
                &mut emitted_registered,
                MemberStreamMode::SubmittedTurn,
                active.agent_id.as_str(),
            ) {
                let _ = tx.send(frame);
            }

            manager.unregister_turn(&span_id);
        });

        stream_response(rx)
    }

    pub(crate) fn cancel(&self, span_id: &str, sig: &str) -> Result<bool> {
        validate_cancel_signal(sig)?;

        let submitted_turn = self
            .active_turns_by_span_id
            .read()
            .expect("member runtime registry lock poisoned")
            .get(span_id)
            .cloned();
        if let Some(submitted_turn) = submitted_turn {
            if let Ok(mut guard) = submitted_turn.last_signal.lock() {
                *guard = Some(sig.trim().to_ascii_uppercase());
            }
            submitted_turn.cancel.cancel();
            return Ok(true);
        }

        let bootstrap = self
            .active_members
            .read()
            .expect("member runtime registry lock poisoned")
            .by_participant_id
            .values()
            .find(|active| active.bootstrap_span_id == span_id)
            .cloned();
        let Some(bootstrap) = bootstrap else {
            return Ok(false);
        };

        if let Ok(mut guard) = bootstrap.bootstrap_last_signal.lock() {
            *guard = Some(sig.trim().to_ascii_uppercase());
        }
        bootstrap.bootstrap_cancel.cancel();
        Ok(true)
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
        if let Some(existing_participant_id) = guard.by_retained_key.get(&retained_key) {
            return Err(
                duplicate_retained_member_error(&retained_key, existing_participant_id).into(),
            );
        }

        guard
            .by_retained_key
            .insert(retained_key, active.participant_id.clone());
        guard
            .by_participant_id
            .insert(active.participant_id.clone(), active);
        Ok(())
    }

    fn unregister_member(&self, participant_id: &str) {
        if let Ok(mut guard) = self.active_members.write() {
            if let Some(active) = guard.by_participant_id.remove(participant_id) {
                let retained_key = RetainedMemberKey::from_active(active.as_ref());
                guard.by_retained_key.remove(&retained_key);
                if let Some(launcher_dir) = active.launcher_dir.as_ref() {
                    let _ = fs::remove_dir_all(launcher_dir);
                }
            }
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

        if let Some(existing_participant_id) = guard.by_retained_key.get(&retained_key) {
            return Err(retained_slot_owner_mismatch_error(
                &retained_key,
                existing_participant_id,
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
            Some(participant_id) if participant_id == &req.participant_id => Ok(()),
            Some(participant_id) => Err(retained_slot_owner_mismatch_error(
                &retained_key,
                participant_id,
                &req.participant_id,
            )
            .into()),
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

fn frame_from_wrapper_event(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    wrapper_event: AgentWrapperEvent,
    emitted_registered: &mut bool,
    mode: MemberStreamMode,
    agent_id: &str,
) -> Option<ExecuteStreamFrame> {
    Some(ExecuteStreamFrame::Event {
        event: agent_event_from_wrapper_event(
            context,
            binding,
            span_id,
            wrapper_event,
            emitted_registered,
            mode,
            agent_id,
        )?,
    })
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
) -> Vec<ExecuteStreamFrame> {
    match completion {
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
                    frames.push(ExecuteStreamFrame::Event { event });
                }
            }

            frames.push(ExecuteStreamFrame::Exit {
                exit: exit_code_from_status(&completion.status),
                span_id: span_id.to_string(),
                scopes_used: Vec::new(),
                fs_diff: None,
                process_telemetry: ProcessTelemetry::default(),
            });
            frames
        }
        Err(AgentWrapperError::Backend { message }) if message == CANCELLED_MESSAGE => {
            vec![ExecuteStreamFrame::Exit {
                exit: cancel_exit_code(cancel_signal.as_deref()),
                span_id: span_id.to_string(),
                scopes_used: Vec::new(),
                fs_diff: None,
                process_telemetry: ProcessTelemetry::default(),
            }]
        }
        Err(err) => vec![ExecuteStreamFrame::Error {
            message: format!("member runtime failed: {err}"),
        }],
    }
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
        thread_id: None,
        role: Some(MEMBER_ROLE.to_string()),
        world_id: Some(binding.world_id.clone()),
        world_generation: Some(binding.world_generation),
        cmd_id: None,
        span_id: Some(span_id.to_string()),
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
        let mut payload = serde_json::to_vec(&frame).expect("serialize member runtime frame");
        payload.push(b'\n');
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
        self.bootstrap_last_signal
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    fn submit_context(&self, run_id: String) -> MemberStreamContext {
        MemberStreamContext {
            orchestration_session_id: self.orchestration_session_id.clone(),
            run_id,
            participant_id: self.participant_id.clone(),
            parent_participant_id: self.parent_participant_id.clone(),
            resumed_from_participant_id: self.resumed_from_participant_id.clone(),
            backend_id: self.backend_id.clone(),
            protocol: self.protocol.clone(),
        }
    }
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

fn duplicate_retained_member_error(
    retained_key: &RetainedMemberKey,
    existing_participant_id: &str,
) -> crate::service::BadRequestError {
    crate::service::BadRequestError::new(format!(
        "a retained world member is already active for orchestration_session_id {} world_generation {} backend_id {} (participant_id {})",
        retained_key.orchestration_session_id,
        retained_key.world_generation,
        retained_key.backend_id,
        existing_participant_id,
    ))
}

fn retained_slot_owner_mismatch_error(
    retained_key: &RetainedMemberKey,
    expected_participant_id: &str,
    actual_participant_id: &str,
) -> crate::service::BadRequestError {
    crate::service::BadRequestError::new(format!(
        "member_turn_submit.participant_id mismatch for retained member orchestration_session_id {} world_generation {} backend_id {} (expected {}, got {})",
        retained_key.orchestration_session_id,
        retained_key.world_generation,
        retained_key.backend_id,
        expected_participant_id,
        actual_participant_id,
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

#[cfg(test)]
mod tests {
    use super::*;

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
        }
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

    #[test]
    fn duplicate_retained_member_error_mentions_backend_slot_identity() {
        let err = duplicate_retained_member_error(
            &RetainedMemberKey {
                orchestration_session_id: "orch_123".to_string(),
                world_generation: 7,
                backend_id: "cli:codex".to_string(),
            },
            "ash_member_existing",
        );

        assert!(
            err.to_string().contains("backend_id cli:codex"),
            "unexpected error: {err}"
        );
        assert!(
            err.to_string()
                .contains("participant_id ash_member_existing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn retained_slot_owner_mismatch_error_mentions_expected_participant() {
        let err = retained_slot_owner_mismatch_error(
            &RetainedMemberKey {
                orchestration_session_id: "orch_123".to_string(),
                world_generation: 7,
                backend_id: "cli:codex".to_string(),
            },
            "ash_member_existing",
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
            .insert(retained_key, "ash_member_existing".to_string());

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
}
