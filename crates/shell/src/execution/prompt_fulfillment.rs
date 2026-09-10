use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use agent_api::{
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperKind, AgentWrapperRunHandle, AgentWrapperRunRequest,
};
use futures::StreamExt;
use sha2::{Digest, Sha256};
use substrate_gateway::adapter_runtime::{GatewayAdapterBackendKind, GatewayAdapterRuntime};

use crate::execution::agent_runtime::{
    control::AGENT_API_SESSION_RESUME_V1, mapping::AgentRuntimeBackendKind,
    tool_invocation_contract::host_tool_contracts_v1, validator::RuntimeSelectionDescriptor,
};

pub(crate) const SUBSTRATE_AGENT_TOOLBOX_ENDPOINT_ENV: &str = "SUBSTRATE_AGENT_TOOLBOX_ENDPOINT";
pub(crate) const SUBSTRATE_AGENT_TOOLBOX_VERSION_ENV: &str = "SUBSTRATE_AGENT_TOOLBOX_VERSION";
pub(crate) const HOST_TOOLBOX_CONTRACT_VERSION_V1: u32 = 1;
const HOST_TOOLBOX_PROMPT_PREAMBLE: &str = "Substrate host toolbox contract:";
const AGENT_API_TURN_LIFECYCLE_V1: &str = "agent_api.turn.lifecycle.v1";

#[derive(Clone)]
pub(crate) enum PromptFulfillmentCancelHandle {
    Gateway {
        handle: agent_api::AgentWrapperCancelHandle,
        cancel_requested: Arc<AtomicBool>,
    },
    CodexNative {
        handle: codex::ExecTerminationHandle,
        cancel_requested: Arc<AtomicBool>,
    },
    ClaudeNative {
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
                cancel_requested.store(true, std::sync::atomic::Ordering::SeqCst);
                handle.cancel();
            }
            Self::CodexNative {
                handle,
                cancel_requested,
            } => {
                cancel_requested.store(true, std::sync::atomic::Ordering::SeqCst);
                handle.request_termination();
            }
            Self::ClaudeNative {
                handle,
                cancel_requested,
            } => {
                cancel_requested.store(true, std::sync::atomic::Ordering::SeqCst);
                handle.request_termination();
            }
        }
    }
}

pub(crate) fn build_runtime_owned_toolbox_env(
    endpoint: impl Into<String>,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            SUBSTRATE_AGENT_TOOLBOX_ENDPOINT_ENV.to_string(),
            endpoint.into(),
        ),
        (
            SUBSTRATE_AGENT_TOOLBOX_VERSION_ENV.to_string(),
            HOST_TOOLBOX_CONTRACT_VERSION_V1.to_string(),
        ),
    ])
}

pub(crate) fn compose_prompt_with_host_toolbox_contract(prompt: &str) -> String {
    let tool_names = host_tool_contracts_v1()
        .iter()
        .map(|contract| contract.tool_name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "{HOST_TOOLBOX_PROMPT_PREAMBLE}
- The runtime exports {SUBSTRATE_AGENT_TOOLBOX_ENDPOINT_ENV} plus {SUBSTRATE_AGENT_TOOLBOX_VERSION_ENV}={HOST_TOOLBOX_CONTRACT_VERSION_V1}.
- Available tools: {tool_names}.
- Fresh world-dispatch requests require an authoritative world binding already attached to this orchestration session; host-only sessions cannot bootstrap the first binding through these tools.
- The supported public bootstrap for a fresh world-bound session is `substrate agent start --backend <placement-qualified world backend> --scope world`; once that session exists, Substrate injects `world_id` and `world_generation` automatically.
- Call the endpoint with versioned host-tool requests that provide only tool arguments; Substrate injects request_id, idempotency_key, orchestration_session_id, caller_participant_id, world_id, and world_generation before dispatch.
- Fresh requests: run_world_task uses target_backend_id plus task payload and returns task_run_id; spawn_world_worker uses target_backend_id plus worker payload and returns participant_id.
- Retained allocation receipts: fork_world_worker returns the child retained-worker participant_id plus explicit source_participant_id lineage.
- Retained follow-ups: fork_world_worker, continue_world_worker, and stop_world_worker require the exact retained-worker participant_id.
- Mixed follow-ups: inspect_world_worker and cancel_world_work require exactly one exact handle family: task_run_id for an active task, participant_id for a retained worker, or exact_target.accepted_retained_turn / exact_target.pending_retained_admission for receipt-targeted retained control.
- Do not provide runtime-owned fields. Substrate injects request_id, idempotency_key, orchestration_session_id, caller_participant_id, world_id, and world_generation.
- Never provide more than one follow-up handle family in the same call; reuse the exact receipt handle returned by Substrate.

User request:
{prompt}"
    )
}

pub(crate) struct PromptFulfillmentRunControl {
    pub(crate) handle: AgentWrapperRunHandle,
    pub(crate) cancel: PromptFulfillmentCancelHandle,
}

pub(crate) struct PromptFulfillmentBridge {
    runtime: GatewayAdapterRuntime,
    backend_kind: AgentRuntimeBackendKind,
    binary_path: PathBuf,
}

impl PromptFulfillmentBridge {
    pub(crate) fn for_descriptor(descriptor: &RuntimeSelectionDescriptor) -> anyhow::Result<Self> {
        let backend_kind = match descriptor.backend_kind {
            AgentRuntimeBackendKind::Codex => GatewayAdapterBackendKind::Codex,
            AgentRuntimeBackendKind::ClaudeCode => GatewayAdapterBackendKind::ClaudeCode,
        };
        Ok(Self {
            runtime: GatewayAdapterRuntime::for_backend(
                backend_kind,
                descriptor.binary_path.clone(),
            )?,
            backend_kind: descriptor.backend_kind,
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

    /// Runs only the inaugural durable public Start exchange. Unlike the
    /// compatibility gateway event mapping, this path retains the provider's
    /// typed lifecycle discriminators and exact native identities.
    pub(crate) async fn run_durable_start_control(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        match self.backend_kind {
            AgentRuntimeBackendKind::Codex => self.run_codex_durable_start(request).await,
            AgentRuntimeBackendKind::ClaudeCode => self.run_claude_durable_start(request).await,
        }
    }

    /// Runs only a previously authenticated HSA ResumeOneTurn successor. The
    /// expected session ID comes from durable authority, not from the request;
    /// the request extension must exact-match it before the provider is spawned.
    pub(crate) async fn run_authority_managed_resume_one_turn_control(
        &self,
        expected_session_id: &str,
        request: AgentWrapperRunRequest,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        validate_exact_resume_extension(&request.extensions, expected_session_id)?;
        match self.backend_kind {
            AgentRuntimeBackendKind::Codex => {
                self.run_codex_authority_managed_resume(expected_session_id, request)
                    .await
            }
            AgentRuntimeBackendKind::ClaudeCode => {
                self.run_claude_authority_managed_resume(expected_session_id, request)
                    .await
            }
        }
    }

    async fn run_codex_durable_start(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        if !request.extensions.is_empty() {
            return Err(AgentWrapperError::InvalidRequest {
                message: "durable Codex Start does not accept continuation extensions".to_string(),
            });
        }
        let mut builder = codex::CodexClient::builder()
            .binary(self.binary_path.clone())
            .json(true)
            .mirror_stdout(false)
            .quiet(true)
            .color_mode(codex::ColorMode::Never)
            .dangerously_bypass_approvals_and_sandbox(true)
            .timeout(request.timeout.unwrap_or(Duration::ZERO));
        if let Some(working_dir) = request.working_dir {
            builder = builder.working_dir(working_dir);
        }
        let control = builder
            .build()
            .stream_exec_with_env_overrides_control(
                codex::ExecStreamRequest {
                    prompt: request.prompt,
                    ephemeral: false,
                    ignore_rules: false,
                    ignore_user_config: false,
                    idle_timeout: None,
                    output_last_message: None,
                    output_schema: None,
                    json_event_log: None,
                },
                &request.env,
            )
            .await
            .map_err(|error| AgentWrapperError::Backend {
                message: format!("Codex durable Start failed: {error}"),
            })?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let events = Box::pin(control.events.map(|event| match event {
            Ok(event) => codex_start_event(event),
            Err(error) => typed_adapter_error("codex", format!("Codex event error: {error}")),
        }));
        let completion = Box::pin(async move {
            control
                .completion
                .await
                .map(|completion| AgentWrapperCompletion {
                    status: completion.status,
                    final_text: completion.last_message,
                    data: None,
                })
                .map_err(|error| AgentWrapperError::Backend {
                    message: format!("Codex durable Start completion failed: {error}"),
                })
        });
        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::CodexNative {
                handle: control.termination,
                cancel_requested,
            },
        })
    }

    async fn run_codex_authority_managed_resume(
        &self,
        expected_session_id: &str,
        request: AgentWrapperRunRequest,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let mut builder = codex::CodexClient::builder()
            .binary(self.binary_path.clone())
            .json(true)
            .mirror_stdout(false)
            .quiet(true)
            .color_mode(codex::ColorMode::Never)
            .dangerously_bypass_approvals_and_sandbox(true)
            .timeout(request.timeout.unwrap_or(Duration::ZERO));
        if let Some(working_dir) = request.working_dir {
            builder = builder.working_dir(working_dir);
        }
        let control = builder
            .build()
            .stream_resume_with_env_overrides_control(
                codex::ResumeRequest::with_id(expected_session_id).prompt(request.prompt),
                &request.env,
            )
            .await
            .map_err(|error| AgentWrapperError::Backend {
                message: format!("Codex authority-managed ResumeOneTurn failed: {error}"),
            })?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let events = Box::pin(control.events.map(|event| match event {
            Ok(event) => codex_resume_event(event),
            Err(error) => {
                typed_adapter_error("codex", format!("Codex ResumeOneTurn event error: {error}"))
            }
        }));
        let completion = Box::pin(async move {
            control
                .completion
                .await
                .map(|completion| AgentWrapperCompletion {
                    status: completion.status,
                    final_text: completion.last_message,
                    data: None,
                })
                .map_err(|error| AgentWrapperError::Backend {
                    message: format!(
                        "Codex authority-managed ResumeOneTurn completion failed: {error}"
                    ),
                })
        });
        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::CodexNative {
                handle: control.termination,
                cancel_requested,
            },
        })
    }

    async fn run_claude_durable_start(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        if !request.extensions.is_empty() {
            return Err(AgentWrapperError::InvalidRequest {
                message: "durable Claude Start does not accept continuation extensions".to_string(),
            });
        }
        let mut builder = claude_code::ClaudeClient::builder()
            .binary(self.binary_path.clone())
            .mirror_stdout(false)
            .mirror_stderr(false);
        if let Some(working_dir) = request.working_dir {
            builder = builder.working_dir(working_dir);
        }
        if let Some(timeout) = request.timeout {
            builder = builder.timeout(Some(timeout));
        }
        for (key, value) in request.env {
            builder = builder.env(key, value);
        }
        let control = builder
            .build()
            .print_stream_json_control(
                claude_code::ClaudePrintRequest::new(request.prompt)
                    .output_format(claude_code::ClaudeOutputFormat::StreamJson)
                    .permission_mode("bypassPermissions"),
            )
            .await
            .map_err(|error| AgentWrapperError::Backend {
                message: format!("Claude durable Start failed: {error}"),
            })?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let events = Box::pin(control.events.map(|event| match event {
            Ok(event) => claude_start_event(event),
            Err(error) => {
                typed_adapter_error("claude_code", format!("Claude event error: {error}"))
            }
        }));
        let completion = Box::pin(async move {
            control
                .completion
                .await
                .map(|status| AgentWrapperCompletion {
                    status,
                    final_text: None,
                    data: None,
                })
                .map_err(|error| AgentWrapperError::Backend {
                    message: format!("Claude durable Start completion failed: {error}"),
                })
        });
        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::ClaudeNative {
                handle: control.termination,
                cancel_requested,
            },
        })
    }

    async fn run_claude_authority_managed_resume(
        &self,
        expected_session_id: &str,
        request: AgentWrapperRunRequest,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let mut builder = claude_code::ClaudeClient::builder()
            .binary(self.binary_path.clone())
            .mirror_stdout(false)
            .mirror_stderr(false);
        if let Some(working_dir) = request.working_dir {
            builder = builder.working_dir(working_dir);
        }
        if let Some(timeout) = request.timeout {
            builder = builder.timeout(Some(timeout));
        }
        for (key, value) in request.env {
            builder = builder.env(key, value);
        }
        let control = builder
            .build()
            .print_stream_json_control(
                claude_code::ClaudePrintRequest::new(request.prompt)
                    .output_format(claude_code::ClaudeOutputFormat::StreamJson)
                    .permission_mode("bypassPermissions")
                    .resume_value(expected_session_id),
            )
            .await
            .map_err(|error| AgentWrapperError::Backend {
                message: format!("Claude authority-managed ResumeOneTurn failed: {error}"),
            })?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let events = Box::pin(control.events.map(|event| match event {
            Ok(event) => claude_resume_event(event),
            Err(error) => typed_adapter_error(
                "claude_code",
                format!("Claude ResumeOneTurn event error: {error}"),
            ),
        }));
        let completion = Box::pin(async move {
            control
                .completion
                .await
                .map(|status| AgentWrapperCompletion {
                    status,
                    final_text: None,
                    data: None,
                })
                .map_err(|error| AgentWrapperError::Backend {
                    message: format!(
                        "Claude authority-managed ResumeOneTurn completion failed: {error}"
                    ),
                })
        });
        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::ClaudeNative {
                handle: control.termination,
                cancel_requested,
            },
        })
    }
}

fn validate_exact_resume_extension(
    extensions: &BTreeMap<String, serde_json::Value>,
    expected_session_id: &str,
) -> Result<(), AgentWrapperError> {
    if expected_session_id.trim().is_empty() {
        return Err(AgentWrapperError::InvalidRequest {
            message: "authority-managed ResumeOneTurn is missing its authenticated session ID"
                .to_string(),
        });
    }
    if extensions.len() != 1 {
        return Err(AgentWrapperError::InvalidRequest {
            message: format!(
                "authority-managed ResumeOneTurn requires exactly {AGENT_API_SESSION_RESUME_V1}"
            ),
        });
    }
    let value = extensions.get(AGENT_API_SESSION_RESUME_V1).ok_or_else(|| {
        AgentWrapperError::InvalidRequest {
            message: format!(
                "authority-managed ResumeOneTurn requires {AGENT_API_SESSION_RESUME_V1}"
            ),
        }
    })?;
    let object = value
        .as_object()
        .ok_or_else(|| AgentWrapperError::InvalidRequest {
            message: format!("{AGENT_API_SESSION_RESUME_V1} must be an object"),
        })?;
    if object.len() != 2 || !object.contains_key("selector") || !object.contains_key("id") {
        return Err(AgentWrapperError::InvalidRequest {
            message: format!("{AGENT_API_SESSION_RESUME_V1} must contain exactly selector and id"),
        });
    }
    if object.get("selector").and_then(serde_json::Value::as_str) != Some("id") {
        return Err(AgentWrapperError::InvalidRequest {
            message: format!(
                "{AGENT_API_SESSION_RESUME_V1}.selector must be the authenticated id selector"
            ),
        });
    }
    if object.get("id").and_then(serde_json::Value::as_str) != Some(expected_session_id) {
        return Err(AgentWrapperError::InvalidRequest {
            message: "authority-managed ResumeOneTurn session ID was substituted".to_string(),
        });
    }
    Ok(())
}

fn lifecycle_data(
    provider: &str,
    session_id: &str,
    turn_id: Option<&str>,
    phase: &str,
    provider_event_kind: &str,
    raw: &serde_json::Value,
) -> serde_json::Value {
    let raw_bytes = serde_json::to_vec(raw).unwrap_or_default();
    let evidence_sha256 = format!("{:x}", Sha256::digest(raw_bytes));
    serde_json::json!({
        "schema": AGENT_API_TURN_LIFECYCLE_V1,
        "provider": provider,
        "session": { "id": session_id },
        "turn": {
            "thread_id": session_id,
            "turn_id": turn_id.unwrap_or(session_id),
            "phase": phase,
        },
        "evidence": {
            "provider_event_kind": provider_event_kind,
            "raw_event_sha256": evidence_sha256,
        },
    })
}

fn typed_event(
    agent_kind: &str,
    kind: AgentWrapperEventKind,
    message: Option<String>,
    data: serde_json::Value,
) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind::new(agent_kind).expect("static agent kind"),
        kind,
        channel: None,
        text: None,
        message,
        data: Some(data),
    }
}

fn codex_start_event(event: codex::ThreadEvent) -> AgentWrapperEvent {
    codex_lifecycle_event(event, "thread.started")
}

fn codex_resume_event(event: codex::ThreadEvent) -> AgentWrapperEvent {
    codex_lifecycle_event(event, "thread.resumed")
}

fn codex_lifecycle_event(
    event: codex::ThreadEvent,
    exchange_provider_event_kind: &str,
) -> AgentWrapperEvent {
    let raw = serde_json::to_value(&event).unwrap_or(serde_json::Value::Null);
    match event {
        codex::ThreadEvent::ThreadStarted(event) => typed_event(
            "codex",
            AgentWrapperEventKind::Status,
            None,
            lifecycle_data(
                "codex",
                &event.thread_id,
                None,
                "exchange_opened",
                exchange_provider_event_kind,
                &raw,
            ),
        ),
        codex::ThreadEvent::TurnStarted(event) => typed_event(
            "codex",
            AgentWrapperEventKind::Status,
            None,
            lifecycle_data(
                "codex",
                &event.thread_id,
                Some(&event.turn_id),
                "started",
                "turn.started",
                &raw,
            ),
        ),
        codex::ThreadEvent::TurnCompleted(event) => typed_event(
            "codex",
            AgentWrapperEventKind::Status,
            None,
            lifecycle_data(
                "codex",
                &event.thread_id,
                Some(&event.turn_id),
                "completed",
                "turn.completed",
                &raw,
            ),
        ),
        codex::ThreadEvent::TurnFailed(event) => typed_event(
            "codex",
            AgentWrapperEventKind::Error,
            Some(event.error.message),
            lifecycle_data(
                "codex",
                &event.thread_id,
                Some(&event.turn_id),
                "failed",
                "turn.failed",
                &raw,
            ),
        ),
        codex::ThreadEvent::Error(error) => typed_adapter_error("codex", error.message),
        _ => typed_event("codex", AgentWrapperEventKind::Unknown, None, raw),
    }
}

fn claude_start_event(event: claude_code::ClaudeStreamJsonEvent) -> AgentWrapperEvent {
    claude_lifecycle_event(event, "Claude reported an inaugural Start result error")
}

fn claude_resume_event(event: claude_code::ClaudeStreamJsonEvent) -> AgentWrapperEvent {
    claude_lifecycle_event(event, "Claude reported a ResumeOneTurn result error")
}

fn claude_lifecycle_event(
    event: claude_code::ClaudeStreamJsonEvent,
    result_error_message: &str,
) -> AgentWrapperEvent {
    let raw = event.raw().clone();
    match event {
        claude_code::ClaudeStreamJsonEvent::SystemInit { session_id, .. } => typed_event(
            "claude_code",
            AgentWrapperEventKind::Status,
            None,
            lifecycle_data(
                "claude_code",
                &session_id,
                None,
                "exchange_opened",
                "system.init",
                &raw,
            ),
        ),
        claude_code::ClaudeStreamJsonEvent::ResultSuccess { session_id, .. } => typed_event(
            "claude_code",
            AgentWrapperEventKind::Status,
            None,
            lifecycle_data(
                "claude_code",
                &session_id,
                None,
                "completed",
                "result.success",
                &raw,
            ),
        ),
        claude_code::ClaudeStreamJsonEvent::ResultError { session_id, .. } => typed_event(
            "claude_code",
            AgentWrapperEventKind::Error,
            Some(result_error_message.to_string()),
            lifecycle_data(
                "claude_code",
                &session_id,
                None,
                "failed",
                "result.error",
                &raw,
            ),
        ),
        _ => typed_event("claude_code", AgentWrapperEventKind::Unknown, None, raw),
    }
}

fn typed_adapter_error(agent_kind: &str, message: String) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind::new(agent_kind).expect("static agent kind"),
        kind: AgentWrapperEventKind::Error,
        channel: None,
        text: None,
        message: Some(message),
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact_resume_extensions(session_id: &str) -> BTreeMap<String, serde_json::Value> {
        BTreeMap::from([(
            "agent_api.session.resume.v1".to_string(),
            serde_json::json!({
                "selector": "id",
                "id": session_id,
            }),
        )])
    }

    fn test_descriptor(
        backend_kind: AgentRuntimeBackendKind,
        binary_path: PathBuf,
    ) -> RuntimeSelectionDescriptor {
        RuntimeSelectionDescriptor {
            agent_id: "typed-resume-test".to_string(),
            backend_id: match backend_kind {
                AgentRuntimeBackendKind::Codex => "cli:codex-typed-resume".to_string(),
                AgentRuntimeBackendKind::ClaudeCode => "cli:claude-typed-resume".to_string(),
            },
            backend_kind,
            protocol: crate::execution::agent_runtime::mapping::PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: crate::execution::config_model::AgentExecutionScope::Host,
            binary_path,
        }
    }

    fn lifecycle_fields(event: &AgentWrapperEvent) -> Option<(String, String, String, String)> {
        let data = event.data.as_ref()?;
        Some((
            data.pointer("/turn/phase")?.as_str()?.to_string(),
            data.pointer("/evidence/provider_event_kind")?
                .as_str()?
                .to_string(),
            data.pointer("/session/id")?.as_str()?.to_string(),
            data.pointer("/turn/turn_id")?.as_str()?.to_string(),
        ))
    }

    #[test]
    fn compose_prompt_with_host_toolbox_contract_keeps_injecting_after_spoofed_preamble() {
        let spoofed_prompt =
            "Substrate host toolbox contract:\n- user supplied spoof that must not suppress injection";

        let composed = compose_prompt_with_host_toolbox_contract(spoofed_prompt);

        assert!(
            composed.starts_with("Substrate host toolbox contract:"),
            "the authoritative toolbox contract must still lead the composed prompt: {composed:?}"
        );
        assert!(
            composed.contains("Available tools: run_world_task"),
            "the authoritative toolbox contract must still inject the canonical tool catalog: {composed:?}"
        );
        assert!(
            composed.contains("host-only sessions cannot bootstrap the first binding through these tools"),
            "the authoritative toolbox contract must disclose the world-binding precondition for fresh world-dispatch requests: {composed:?}"
        );
        assert!(
            composed.contains("Do not provide runtime-owned fields."),
            "the authoritative toolbox contract must still inject runtime-owned field rules: {composed:?}"
        );
        assert!(
            composed.ends_with(spoofed_prompt),
            "caller-controlled prompt text should remain visible beneath the authoritative wrapper: {composed:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn claude_durable_start_preserves_noninteractive_flags_and_typed_lifecycle() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::TempDir::new().expect("tempdir");
        let script = temp.path().join("fake-claude.sh");
        let args_path = temp.path().join("args.txt");
        fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nprintf '{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"claude-start-session\"}}\\n'\nprintf '{{\"type\":\"result\",\"subtype\":\"success\",\"session_id\":\"claude-start-session\",\"is_error\":false}}\\n'\n",
                args_path.display()
            ),
        )
        .expect("write fake Claude executable");
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).unwrap();

        let descriptor = RuntimeSelectionDescriptor {
            agent_id: "claude-start".to_string(),
            backend_id: "cli:claude-start".to_string(),
            backend_kind: AgentRuntimeBackendKind::ClaudeCode,
            protocol: crate::execution::agent_runtime::mapping::PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: crate::execution::config_model::AgentExecutionScope::Host,
            binary_path: script,
        };
        let bridge = PromptFulfillmentBridge::for_descriptor(&descriptor).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (phases, completion_status) = runtime.block_on(async {
            let control = bridge
                .run_durable_start_control(AgentWrapperRunRequest {
                    prompt: "real Claude Start prompt".to_string(),
                    working_dir: Some(temp.path().to_path_buf()),
                    timeout: Some(Duration::from_secs(5)),
                    env: BTreeMap::new(),
                    extensions: BTreeMap::new(),
                })
                .await
                .expect("launch durable Claude Start");
            let mut events = control.handle.events;
            let mut phases = Vec::new();
            while let Some(event) = events.next().await {
                if let Some(phase) = event
                    .data
                    .as_ref()
                    .and_then(|data| data.pointer("/turn/phase"))
                    .and_then(serde_json::Value::as_str)
                {
                    phases.push(phase.to_string());
                }
            }
            let completion = control.handle.completion.await.expect("Claude completion");
            (phases, completion.status)
        });
        assert!(completion_status.success());
        assert_eq!(phases, ["exchange_opened", "completed"]);
        let args = fs::read_to_string(args_path).expect("read Claude arguments");
        let args = args.lines().collect::<Vec<_>>();
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--permission-mode", "bypassPermissions"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--output-format", "stream-json"]));
        assert_eq!(
            args.iter()
                .filter(|argument| **argument == "real Claude Start prompt")
                .count(),
            1,
            "the direct Claude adapter must submit the caller prompt exactly once"
        );
    }

    #[cfg(unix)]
    #[test]
    fn codex_resume_one_turn_uses_exact_id_once_and_preserves_typed_lifecycle() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::TempDir::new().expect("tempdir");
        let script = temp.path().join("fake-codex-resume.sh");
        let args_path = temp.path().join("args.txt");
        let stdin_path = temp.path().join("stdin.txt");
        fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\ncat > '{}'\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"codex-resume-session\"}}\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"codex-resume-session\",\"turn_id\":\"turn-resume-1\"}}\\n'\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"codex-resume-session\",\"turn_id\":\"turn-resume-1\"}}\\n'\n",
                args_path.display(),
                stdin_path.display(),
            ),
        )
        .expect("write fake Codex executable");
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).unwrap();

        let bridge = PromptFulfillmentBridge::for_descriptor(&test_descriptor(
            AgentRuntimeBackendKind::Codex,
            script,
        ))
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let lifecycle = runtime.block_on(async {
            let control = bridge
                .run_authority_managed_resume_one_turn_control(
                    "codex-resume-session",
                    AgentWrapperRunRequest {
                        prompt: "real Codex ResumeOneTurn prompt".to_string(),
                        working_dir: Some(temp.path().to_path_buf()),
                        timeout: Some(Duration::from_secs(5)),
                        env: BTreeMap::from([("RESUME_TEST_ENV".to_string(), "exact".to_string())]),
                        extensions: exact_resume_extensions("codex-resume-session"),
                    },
                )
                .await
                .expect("launch typed Codex ResumeOneTurn");
            let mut events = control.handle.events;
            let mut lifecycle = Vec::new();
            while let Some(event) = events.next().await {
                if let Some(fields) = lifecycle_fields(&event) {
                    lifecycle.push(fields);
                }
            }
            let completion = control.handle.completion.await.expect("Codex completion");
            assert!(completion.status.success());
            lifecycle
        });

        assert_eq!(
            lifecycle,
            [
                (
                    "exchange_opened".to_string(),
                    "thread.resumed".to_string(),
                    "codex-resume-session".to_string(),
                    "codex-resume-session".to_string(),
                ),
                (
                    "started".to_string(),
                    "turn.started".to_string(),
                    "codex-resume-session".to_string(),
                    "turn-resume-1".to_string(),
                ),
                (
                    "completed".to_string(),
                    "turn.completed".to_string(),
                    "codex-resume-session".to_string(),
                    "turn-resume-1".to_string(),
                ),
            ]
        );
        let args = fs::read_to_string(args_path).expect("read Codex arguments");
        let args = args.lines().collect::<Vec<_>>();
        assert!(args
            .windows(2)
            .any(|pair| pair == ["resume", "codex-resume-session"]));
        assert_eq!(
            args.iter()
                .filter(|argument| **argument == "codex-resume-session")
                .count(),
            1,
            "the exact authenticated Codex session ID must be selected once"
        );
        assert_eq!(
            fs::read_to_string(stdin_path).expect("read Codex stdin"),
            "real Codex ResumeOneTurn prompt\n",
            "the real prompt must be submitted exactly once over native resume stdin"
        );
    }

    #[test]
    fn codex_resume_one_turn_preserves_native_failed_terminal_evidence() {
        let event: codex::ThreadEvent = serde_json::from_value(serde_json::json!({
            "type": "turn.failed",
            "thread_id": "codex-resume-session",
            "turn_id": "turn-resume-failed",
            "error": { "message": "native failure" },
        }))
        .expect("parse native Codex failure");

        let mapped = codex_resume_event(event);
        assert_eq!(
            lifecycle_fields(&mapped),
            Some((
                "failed".to_string(),
                "turn.failed".to_string(),
                "codex-resume-session".to_string(),
                "turn-resume-failed".to_string(),
            ))
        );
        assert_eq!(mapped.kind, AgentWrapperEventKind::Error);
        assert_eq!(mapped.message.as_deref(), Some("native failure"));
    }

    #[cfg(unix)]
    #[test]
    fn claude_resume_one_turn_uses_exact_id_once_and_preserves_typed_lifecycle() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::TempDir::new().expect("tempdir");
        let script = temp.path().join("fake-claude-resume.sh");
        let args_path = temp.path().join("args.txt");
        fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nprintf '{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"claude-resume-session\"}}\\n'\nprintf '{{\"type\":\"result\",\"subtype\":\"success\",\"session_id\":\"claude-resume-session\",\"is_error\":false}}\\n'\n",
                args_path.display(),
            ),
        )
        .expect("write fake Claude executable");
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).unwrap();

        let bridge = PromptFulfillmentBridge::for_descriptor(&test_descriptor(
            AgentRuntimeBackendKind::ClaudeCode,
            script,
        ))
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let lifecycle = runtime.block_on(async {
            let control = bridge
                .run_authority_managed_resume_one_turn_control(
                    "claude-resume-session",
                    AgentWrapperRunRequest {
                        prompt: "real Claude ResumeOneTurn prompt".to_string(),
                        working_dir: Some(temp.path().to_path_buf()),
                        timeout: Some(Duration::from_secs(5)),
                        env: BTreeMap::new(),
                        extensions: exact_resume_extensions("claude-resume-session"),
                    },
                )
                .await
                .expect("launch typed Claude ResumeOneTurn");
            let mut events = control.handle.events;
            let mut lifecycle = Vec::new();
            while let Some(event) = events.next().await {
                if let Some(fields) = lifecycle_fields(&event) {
                    lifecycle.push(fields);
                }
            }
            let completion = control.handle.completion.await.expect("Claude completion");
            assert!(completion.status.success());
            lifecycle
        });

        assert_eq!(
            lifecycle,
            [
                (
                    "exchange_opened".to_string(),
                    "system.init".to_string(),
                    "claude-resume-session".to_string(),
                    "claude-resume-session".to_string(),
                ),
                (
                    "completed".to_string(),
                    "result.success".to_string(),
                    "claude-resume-session".to_string(),
                    "claude-resume-session".to_string(),
                ),
            ]
        );
        let args = fs::read_to_string(args_path).expect("read Claude arguments");
        let args = args.lines().collect::<Vec<_>>();
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--resume", "claude-resume-session"]));
        assert_eq!(
            args.iter()
                .filter(|argument| **argument == "claude-resume-session")
                .count(),
            1,
            "the exact authenticated Claude session ID must be selected once"
        );
        assert_eq!(
            args.iter()
                .filter(|argument| **argument == "real Claude ResumeOneTurn prompt")
                .count(),
            1,
            "the real Claude prompt must be submitted exactly once"
        );
    }

    #[test]
    fn claude_resume_one_turn_preserves_native_failed_terminal_evidence() {
        let raw = serde_json::json!({
            "type": "result",
            "subtype": "error_during_execution",
            "session_id": "claude-resume-session",
            "is_error": true,
        });
        let mapped = claude_resume_event(claude_code::ClaudeStreamJsonEvent::ResultError {
            session_id: "claude-resume-session".to_string(),
            raw,
        });
        assert_eq!(
            lifecycle_fields(&mapped),
            Some((
                "failed".to_string(),
                "result.error".to_string(),
                "claude-resume-session".to_string(),
                "claude-resume-session".to_string(),
            ))
        );
        assert_eq!(mapped.kind, AgentWrapperEventKind::Error);
    }

    #[cfg(unix)]
    #[test]
    fn authority_managed_resume_rejects_inexact_extensions_before_provider_spawn() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::TempDir::new().expect("tempdir");
        let script = temp.path().join("must-not-spawn.sh");
        let spawn_marker = temp.path().join("spawned");
        fs::write(
            &script,
            format!("#!/bin/sh\nprintf spawned > '{}'\n", spawn_marker.display()),
        )
        .expect("write provider guard executable");
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).unwrap();
        let bridge = PromptFulfillmentBridge::for_descriptor(&test_descriptor(
            AgentRuntimeBackendKind::Codex,
            script,
        ))
        .unwrap();
        let invalid_extensions = [
            BTreeMap::new(),
            BTreeMap::from([(
                "agent_api.session.resume.v1".to_string(),
                serde_json::json!("not-an-object"),
            )]),
            BTreeMap::from([(
                "agent_api.session.resume.v1".to_string(),
                serde_json::json!({ "selector": "last" }),
            )]),
            exact_resume_extensions("substituted-session"),
            BTreeMap::from([(
                "agent_api.session.resume.v1".to_string(),
                serde_json::json!({
                    "selector": "id",
                    "id": "authenticated-session",
                    "extra": true,
                }),
            )]),
            BTreeMap::from([
                (
                    "agent_api.session.resume.v1".to_string(),
                    serde_json::json!({
                        "selector": "id",
                        "id": "authenticated-session",
                    }),
                ),
                ("unexpected.extension".to_string(), serde_json::json!({})),
            ]),
        ];
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            for extensions in invalid_extensions {
                let error = match bridge
                    .run_authority_managed_resume_one_turn_control(
                        "authenticated-session",
                        AgentWrapperRunRequest {
                            prompt: "must not be submitted".to_string(),
                            working_dir: Some(temp.path().to_path_buf()),
                            timeout: Some(Duration::from_secs(1)),
                            env: BTreeMap::new(),
                            extensions,
                        },
                    )
                    .await
                {
                    Ok(_) => panic!("inexact resume extension must fail closed"),
                    Err(error) => error,
                };
                assert!(matches!(error, AgentWrapperError::InvalidRequest { .. }));
            }
        });
        assert!(
            !spawn_marker.exists(),
            "resume validation must complete before provider execution"
        );
    }
}
