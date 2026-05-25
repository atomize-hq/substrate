use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use agent_api::{
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperKind, AgentWrapperRunHandle, AgentWrapperRunRequest,
};
use anyhow::Context;
use futures::StreamExt;
use substrate_gateway::adapter_runtime::{GatewayAdapterBackendKind, GatewayAdapterRuntime};

use crate::execution::agent_runtime::{
    mapping::AgentRuntimeBackendKind, validator::RuntimeSelectionDescriptor,
};

const SESSION_HANDLE_SCHEMA_V1: &str = "agent_api.session.handle.v1";

#[derive(Clone)]
pub(crate) enum PromptFulfillmentCancelHandle {
    Gateway {
        handle: agent_api::AgentWrapperCancelHandle,
        cancel_requested: Arc<AtomicBool>,
    },
    Codex {
        handle: codex::ExecTerminationHandle,
        cancel_requested: Arc<AtomicBool>,
    },
    Claude {
        handle: claude_code::ClaudeTerminationHandle,
        cancel_requested: Arc<AtomicBool>,
    },
}

impl PromptFulfillmentCancelHandle {
    pub(crate) fn cancel(&self) {
        match self {
            Self::Gateway {
                handle,
                cancel_requested,
            } => {
                cancel_requested.store(true, Ordering::SeqCst);
                handle.cancel();
            }
            Self::Codex {
                handle,
                cancel_requested,
            } => {
                cancel_requested.store(true, Ordering::SeqCst);
                handle.request_termination();
            }
            Self::Claude {
                handle,
                cancel_requested,
            } => {
                cancel_requested.store(true, Ordering::SeqCst);
                handle.request_termination();
            }
        }
    }
}

pub(crate) struct PromptFulfillmentRunControl {
    pub(crate) handle: AgentWrapperRunHandle,
    pub(crate) cancel: PromptFulfillmentCancelHandle,
}

pub(crate) struct PromptFulfillmentBridge {
    runtime: GatewayAdapterRuntime,
    backend_kind: GatewayAdapterBackendKind,
    binary_path: PathBuf,
}

impl PromptFulfillmentBridge {
    pub(crate) fn for_descriptor(descriptor: &RuntimeSelectionDescriptor) -> anyhow::Result<Self> {
        let backend_kind = match descriptor.backend_kind {
            AgentRuntimeBackendKind::Codex => GatewayAdapterBackendKind::Codex,
            AgentRuntimeBackendKind::ClaudeCode => GatewayAdapterBackendKind::ClaudeCode,
        };
        let runtime =
            GatewayAdapterRuntime::for_backend(backend_kind, descriptor.binary_path.clone())?;

        Ok(Self {
            runtime,
            backend_kind,
            binary_path: descriptor.binary_path.clone(),
        })
    }

    pub(crate) async fn run_control(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let control = self.runtime.run_control(request).await?;
        Ok(PromptFulfillmentRunControl {
            handle: control.handle,
            cancel: PromptFulfillmentCancelHandle::Gateway {
                handle: control.cancel,
                cancel_requested,
            },
        })
    }

    pub(crate) async fn run_attach_control(
        &self,
        session_id: &str,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        match self.backend_kind {
            GatewayAdapterBackendKind::Codex => self.run_codex_attach_control(session_id).await,
            GatewayAdapterBackendKind::ClaudeCode => {
                self.run_claude_attach_control(session_id).await
            }
        }
    }

    async fn run_codex_attach_control(
        &self,
        session_id: &str,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let mut builder = codex::CodexClient::builder()
            .json(true)
            .mirror_stdout(false)
            .quiet(true)
            .color_mode(codex::ColorMode::Never)
            .approval_policy(codex::ApprovalPolicy::Never)
            .sandbox_mode(codex::SandboxMode::WorkspaceWrite)
            .binary(self.binary_path.clone());
        if let Some(working_dir) = current_working_dir()? {
            builder = builder.working_dir(working_dir);
        }

        let client = builder.build();
        let control = client
            .stream_resume_with_env_overrides_control(
                codex::ResumeRequest::with_id(session_id),
                &BTreeMap::new(),
            )
            .await
            .map_err(map_codex_stream_error)?;
        let codex::ExecStreamControl {
            events,
            completion,
            termination,
        } = control;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let completion_cancel_requested = Arc::clone(&cancel_requested);

        let events = Box::pin(events.map(|event| match event {
            Ok(event) => map_codex_attach_event(event),
            Err(err) => AgentWrapperEvent {
                agent_kind: codex_kind(),
                kind: AgentWrapperEventKind::Error,
                channel: Some("error".to_string()),
                text: None,
                message: Some(format!("codex attach stream failed: {err}")),
                data: None,
            },
        }));
        let completion = Box::pin(async move {
            let completion = completion.await.map_err(|err| {
                if completion_cancel_requested.load(Ordering::SeqCst) {
                    cancelled_wrapper_error()
                } else {
                    map_codex_stream_error(err)
                }
            })?;
            Ok(AgentWrapperCompletion {
                status: completion.status,
                final_text: completion.last_message,
                data: None,
            })
        });

        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::Codex {
                handle: termination,
                cancel_requested,
            },
        })
    }

    async fn run_claude_attach_control(
        &self,
        session_id: &str,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let mut builder = claude_code::ClaudeClient::builder()
            .binary(&self.binary_path)
            .mirror_stdout(false)
            .mirror_stderr(false);
        if let Some(working_dir) = current_working_dir()? {
            builder = builder.working_dir(working_dir);
        }

        let client = builder.build();
        let request = claude_code::ClaudePrintRequest::new("substrate-attach")
            .no_prompt()
            .output_format(claude_code::ClaudeOutputFormat::StreamJson)
            .resume_value(session_id);
        let control = client
            .print_stream_json_control(request)
            .await
            .map_err(map_claude_error)?;
        let claude_code::ClaudePrintStreamJsonControlHandle {
            events,
            completion,
            termination,
        } = control;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let completion_cancel_requested = Arc::clone(&cancel_requested);

        let events = Box::pin(events.map(|event| match event {
            Ok(event) => map_claude_attach_event(event),
            Err(err) => AgentWrapperEvent {
                agent_kind: claude_kind(),
                kind: AgentWrapperEventKind::Error,
                channel: Some("error".to_string()),
                text: None,
                message: Some(format!("claude_code attach stream parse failed: {err}")),
                data: None,
            },
        }));
        let completion = Box::pin(async move {
            let status = completion.await.map_err(|err| {
                if completion_cancel_requested.load(Ordering::SeqCst) {
                    cancelled_wrapper_error()
                } else {
                    map_claude_error(err)
                }
            })?;
            Ok(AgentWrapperCompletion {
                status,
                final_text: None,
                data: None,
            })
        });

        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::Claude {
                handle: termination,
                cancel_requested,
            },
        })
    }
}

fn current_working_dir() -> Result<Option<PathBuf>, AgentWrapperError> {
    std::env::current_dir()
        .map(Some)
        .with_context(|| "resolve current working directory for prompt fulfillment")
        .map_err(map_anyhow_error)
}

fn codex_kind() -> AgentWrapperKind {
    AgentWrapperKind::new("codex").expect("codex agent kind should be valid")
}

fn claude_kind() -> AgentWrapperKind {
    AgentWrapperKind::new("claude_code").expect("claude_code agent kind should be valid")
}

fn map_codex_attach_event(event: codex::ThreadEvent) -> AgentWrapperEvent {
    let raw = serde_json::to_value(&event).ok();
    let data = match event.thread_id() {
        Some(thread_id) => session_handle_facet(thread_id, raw),
        None => raw,
    };
    let (kind, channel, message) = match &event {
        codex::ThreadEvent::Error(err) => (
            AgentWrapperEventKind::Error,
            Some("error".to_string()),
            Some(err.message.clone()),
        ),
        codex::ThreadEvent::TurnFailed(_) => (
            AgentWrapperEventKind::Status,
            Some("status".to_string()),
            Some("turn failed".to_string()),
        ),
        _ => (
            AgentWrapperEventKind::Status,
            Some("status".to_string()),
            None,
        ),
    };

    AgentWrapperEvent {
        agent_kind: codex_kind(),
        kind,
        channel,
        text: None,
        message,
        data,
    }
}

fn map_claude_attach_event(event: claude_code::ClaudeStreamJsonEvent) -> AgentWrapperEvent {
    let raw = event.clone().into_raw();
    let data = match event.session_id() {
        Some(session_id) => session_handle_facet(session_id, Some(raw)),
        None => Some(raw),
    };
    let (kind, channel, message) = match &event {
        claude_code::ClaudeStreamJsonEvent::ResultError { .. } => (
            AgentWrapperEventKind::Error,
            Some("error".to_string()),
            Some("result error".to_string()),
        ),
        claude_code::ClaudeStreamJsonEvent::SystemInit { .. } => (
            AgentWrapperEventKind::Status,
            Some("status".to_string()),
            Some("system init".to_string()),
        ),
        claude_code::ClaudeStreamJsonEvent::SystemOther { subtype, .. } => (
            AgentWrapperEventKind::Status,
            Some("status".to_string()),
            Some(format!("system {subtype}")),
        ),
        claude_code::ClaudeStreamJsonEvent::ResultSuccess { .. } => (
            AgentWrapperEventKind::Status,
            Some("status".to_string()),
            Some("result success".to_string()),
        ),
        _ => (
            AgentWrapperEventKind::Status,
            Some("status".to_string()),
            None,
        ),
    };

    AgentWrapperEvent {
        agent_kind: claude_kind(),
        kind,
        channel,
        text: None,
        message,
        data,
    }
}

fn session_handle_facet(
    session_id: &str,
    raw_event: Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    if session_id.trim().is_empty() {
        return raw_event;
    }

    let mut value = serde_json::json!({
        "schema": SESSION_HANDLE_SCHEMA_V1,
        "session": { "id": session_id },
    });
    if let Some(raw_event) = raw_event {
        value["raw_event"] = raw_event;
    }
    Some(value)
}

fn map_anyhow_error(err: anyhow::Error) -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: format!("substrate-gateway adapter runtime: {err}"),
    }
}

fn cancelled_wrapper_error() -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: "cancelled".to_string(),
    }
}

fn map_codex_stream_error(err: codex::ExecStreamError) -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: format!("codex attach failed: {err}"),
    }
}

fn map_claude_error(err: claude_code::ClaudeCodeError) -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: format!("claude_code attach failed: {err}"),
    }
}
