use agent_api::{
    backends::{
        claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
        codex::{CodexBackend, CodexBackendConfig},
    },
    AgentWrapperCancelHandle, AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent,
    AgentWrapperEventKind, AgentWrapperGateway, AgentWrapperKind, AgentWrapperRunControl,
    AgentWrapperRunRequest,
};
use agent_api_types::{
    ExecuteStreamFrame, MemberDispatchRequestV1, MemberRuntimeBackendKindV1, ProcessTelemetry,
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
    path::Path,
    sync::{Arc, Mutex, RwLock},
};
use substrate_common::agent_events::{AgentEvent, AgentEventKind, MessageEventKind};
use tokio_stream::wrappers::UnboundedReceiverStream;
use world_api::SharedWorldBindingSnapshot;

const MEMBER_ROLE: &str = "member";
const SESSION_HANDLE_SCHEMA_V1: &str = "agent_api.session.handle.v1";
const CANCELLED_MESSAGE: &str = "cancelled";

#[derive(Clone, Default)]
pub(crate) struct MemberRuntimeManager {
    active: Arc<RwLock<HashMap<String, Arc<ActiveMemberRuntime>>>>,
}

struct ActiveMemberRuntime {
    cancel: AgentWrapperCancelHandle,
    last_signal: Mutex<Option<String>>,
}

impl MemberRuntimeManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn launch(
        &self,
        agent_id: String,
        cwd: std::path::PathBuf,
        env: HashMap<String, String>,
        span_id: String,
        dispatch: MemberDispatchRequestV1,
        binding: SharedWorldBindingSnapshot,
    ) -> Result<Response> {
        validate_member_runtime_binary(&dispatch)?;

        let (gateway, agent_kind) = build_gateway_for_dispatch(&dispatch)?;
        let AgentWrapperRunControl { handle, cancel } = gateway
            .run_control(
                &agent_kind,
                AgentWrapperRunRequest {
                    prompt: runtime_bootstrap_prompt().to_string(),
                    working_dir: Some(cwd),
                    timeout: None,
                    env: env.into_iter().collect::<BTreeMap<_, _>>(),
                    extensions: BTreeMap::new(),
                },
            )
            .await
            .map_err(map_wrapper_error)?;

        let active = Arc::new(ActiveMemberRuntime {
            cancel,
            last_signal: Mutex::new(None),
        });
        self.register(span_id.clone(), active);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(ExecuteStreamFrame::Start {
            span_id: span_id.clone(),
        });

        let manager = self.clone();
        tokio::spawn(async move {
            let mut events = handle.events;
            let completion = handle.completion;
            let mut emitted_registered = false;

            while let Some(wrapper_event) = events.next().await {
                if let Some(frame) = frame_from_wrapper_event(
                    &agent_id,
                    &dispatch,
                    &binding,
                    &span_id,
                    wrapper_event,
                    &mut emitted_registered,
                ) {
                    let _ = tx.send(frame);
                }
            }

            if let Some(frame) = frame_from_completion(
                &agent_id,
                &dispatch,
                &binding,
                &span_id,
                completion.await,
                manager.last_signal_for(&span_id),
                &mut emitted_registered,
            ) {
                let _ = tx.send(frame);
            }

            manager.unregister(&span_id);
        });

        stream_response(rx)
    }

    pub(crate) fn cancel(&self, span_id: &str, sig: &str) -> Result<bool> {
        validate_cancel_signal(sig)?;

        let active = self
            .active
            .read()
            .expect("member runtime registry lock poisoned")
            .get(span_id)
            .cloned();
        let Some(active) = active else {
            return Ok(false);
        };

        if let Ok(mut guard) = active.last_signal.lock() {
            *guard = Some(sig.trim().to_ascii_uppercase());
        }
        active.cancel.cancel();
        Ok(true)
    }

    fn register(&self, span_id: String, active: Arc<ActiveMemberRuntime>) {
        self.active
            .write()
            .expect("member runtime registry lock poisoned")
            .insert(span_id, active);
    }

    fn unregister(&self, span_id: &str) {
        if let Ok(mut guard) = self.active.write() {
            guard.remove(span_id);
        }
    }

    fn last_signal_for(&self, span_id: &str) -> Option<String> {
        self.active
            .read()
            .ok()
            .and_then(|guard| guard.get(span_id).cloned())
            .and_then(|active| {
                active
                    .last_signal
                    .lock()
                    .ok()
                    .and_then(|guard| guard.clone())
            })
    }
}

fn runtime_bootstrap_prompt() -> &'static str {
    "Enter persistent Substrate world-scoped member mode. Keep this control session attached for the lifetime of the parent REPL session and do not exit until the client cancels the run."
}

fn validate_member_runtime_binary(dispatch: &MemberDispatchRequestV1) -> Result<()> {
    let path = Path::new(&dispatch.resolved_runtime.binary_path);
    if !path.is_file() {
        return Err(anyhow!(
            "member_dispatch.resolved_runtime.binary_path does not exist or is not a file: {}",
            dispatch.resolved_runtime.binary_path
        ));
    }
    Ok(())
}

fn build_gateway_for_dispatch(
    dispatch: &MemberDispatchRequestV1,
) -> Result<(AgentWrapperGateway, AgentWrapperKind)> {
    let mut gateway = AgentWrapperGateway::new();
    let binary_path = Some(dispatch.resolved_runtime.binary_path.clone().into());

    let agent_kind = match dispatch.resolved_runtime.backend_kind {
        MemberRuntimeBackendKindV1::Codex => {
            gateway
                .register(Arc::new(CodexBackend::new(CodexBackendConfig {
                    binary: binary_path,
                    ..Default::default()
                })))
                .map_err(map_wrapper_error)?;
            AgentWrapperKind::new("codex").map_err(map_wrapper_error)?
        }
        MemberRuntimeBackendKindV1::ClaudeCode => {
            gateway
                .register(Arc::new(ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
                    binary: binary_path,
                    ..Default::default()
                })))
                .map_err(map_wrapper_error)?;
            AgentWrapperKind::new("claude_code").map_err(map_wrapper_error)?
        }
    };

    Ok((gateway, agent_kind))
}

fn frame_from_wrapper_event(
    agent_id: &str,
    dispatch: &MemberDispatchRequestV1,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    wrapper_event: AgentWrapperEvent,
    emitted_registered: &mut bool,
) -> Option<ExecuteStreamFrame> {
    Some(ExecuteStreamFrame::Event {
        event: agent_event_from_wrapper_event(
            agent_id,
            dispatch,
            binding,
            span_id,
            wrapper_event,
            emitted_registered,
        )?,
    })
}

fn frame_from_completion(
    agent_id: &str,
    dispatch: &MemberDispatchRequestV1,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    completion: std::result::Result<AgentWrapperCompletion, AgentWrapperError>,
    cancel_signal: Option<String>,
    emitted_registered: &mut bool,
) -> Option<ExecuteStreamFrame> {
    match completion {
        Ok(completion) => {
            if !*emitted_registered {
                if let Some(event) = registered_event_from_data(
                    agent_id,
                    dispatch,
                    binding,
                    span_id,
                    completion.data.as_ref(),
                ) {
                    *emitted_registered = true;
                    return Some(ExecuteStreamFrame::Event { event });
                }
            }

            Some(ExecuteStreamFrame::Exit {
                exit: exit_code_from_status(&completion.status),
                span_id: span_id.to_string(),
                scopes_used: Vec::new(),
                fs_diff: None,
                process_telemetry: ProcessTelemetry::default(),
            })
        }
        Err(AgentWrapperError::Backend { message }) if message == CANCELLED_MESSAGE => {
            Some(ExecuteStreamFrame::Exit {
                exit: cancel_exit_code(cancel_signal.as_deref()),
                span_id: span_id.to_string(),
                scopes_used: Vec::new(),
                fs_diff: None,
                process_telemetry: ProcessTelemetry::default(),
            })
        }
        Err(err) => Some(ExecuteStreamFrame::Error {
            message: format!("member runtime failed: {err}"),
        }),
    }
}

fn agent_event_from_wrapper_event(
    agent_id: &str,
    dispatch: &MemberDispatchRequestV1,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    wrapper_event: AgentWrapperEvent,
    emitted_registered: &mut bool,
) -> Option<AgentEvent> {
    if let Some(event) = registered_event_from_data(
        agent_id,
        dispatch,
        binding,
        span_id,
        wrapper_event.data.as_ref(),
    ) {
        *emitted_registered = true;
        return Some(event);
    }

    let mut event = match wrapper_event.kind {
        AgentWrapperEventKind::Status => AgentEvent::message(
            agent_id,
            dispatch.orchestration_session_id.clone(),
            dispatch.run_id.clone(),
            MessageEventKind::Status,
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "member runtime status".to_string()),
        ),
        AgentWrapperEventKind::TextOutput => AgentEvent::message(
            agent_id,
            dispatch.orchestration_session_id.clone(),
            dispatch.run_id.clone(),
            MessageEventKind::TaskProgress,
            wrapper_event
                .text
                .clone()
                .unwrap_or_else(|| "member runtime output".to_string()),
        ),
        AgentWrapperEventKind::ToolCall | AgentWrapperEventKind::ToolResult => AgentEvent::message(
            agent_id,
            dispatch.orchestration_session_id.clone(),
            dispatch.run_id.clone(),
            MessageEventKind::TaskProgress,
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "member runtime tool activity".to_string()),
        ),
        AgentWrapperEventKind::Error => AgentEvent::alert(
            agent_id,
            dispatch.orchestration_session_id.clone(),
            dispatch.run_id.clone(),
            "agent_wrapper_error",
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "member runtime error".to_string()),
        ),
        AgentWrapperEventKind::Unknown => AgentEvent::message(
            agent_id,
            dispatch.orchestration_session_id.clone(),
            dispatch.run_id.clone(),
            MessageEventKind::TaskProgress,
            "member runtime emitted an unknown event".to_string(),
        ),
    };

    stamp_event_identity(
        &mut event,
        agent_id,
        dispatch,
        binding,
        span_id,
        wrapper_event.channel,
    );

    if let Some(data) = wrapper_event.data {
        if let Some(obj) = event.data.as_object_mut() {
            obj.insert("uaa_event".to_string(), data);
            obj.insert("protocol".to_string(), json!(dispatch.protocol));
        }
    }

    Some(event)
}

fn registered_event_from_data(
    agent_id: &str,
    dispatch: &MemberDispatchRequestV1,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    data: Option<&serde_json::Value>,
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
        orchestration_session_id: dispatch.orchestration_session_id.clone(),
        run_id: dispatch.run_id.clone(),
        parent_run_id: None,
        participant_id: Some(dispatch.participant_id.clone()),
        parent_participant_id: dispatch.parent_participant_id.clone(),
        resumed_from_participant_id: dispatch.resumed_from_participant_id.clone(),
        backend_id: Some(dispatch.backend_id.clone()),
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
    dispatch: &MemberDispatchRequestV1,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    channel: Option<String>,
) {
    event.role = Some(MEMBER_ROLE.to_string());
    event.backend_id = Some(dispatch.backend_id.clone());
    event.participant_id = Some(dispatch.participant_id.clone());
    event.parent_participant_id = dispatch.parent_participant_id.clone();
    event.resumed_from_participant_id = dispatch.resumed_from_participant_id.clone();
    event.world_id = Some(binding.world_id.clone());
    event.world_generation = Some(binding.world_generation);
    event.span_id = Some(span_id.to_string());
    event.set_channel(channel);
    event.set_pure_agent_telemetry_identity(agent_id.to_string());
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
