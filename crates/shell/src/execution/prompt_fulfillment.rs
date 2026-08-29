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
    mapping::AgentRuntimeBackendKind, tool_invocation_contract::host_tool_contracts_v1,
    validator::RuntimeSelectionDescriptor,
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
    CodexStart {
        handle: codex::ExecTerminationHandle,
        cancel_requested: Arc<AtomicBool>,
    },
    ClaudeStart {
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
            Self::CodexStart {
                handle,
                cancel_requested,
            } => {
                cancel_requested.store(true, std::sync::atomic::Ordering::SeqCst);
                handle.request_termination();
            }
            Self::ClaudeStart {
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
- Mixed follow-ups: inspect_world_worker and cancel_world_work require exactly one exact handle: task_run_id for an active task or participant_id for a retained worker.
- Do not provide runtime-owned fields. Substrate injects request_id, idempotency_key, orchestration_session_id, caller_participant_id, world_id, and world_generation.
- Never provide both task_run_id and participant_id in the same follow-up call; reuse the exact receipt handle returned by Substrate.

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
            cancel: PromptFulfillmentCancelHandle::CodexStart {
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
            cancel: PromptFulfillmentCancelHandle::ClaudeStart {
                handle: control.termination,
                cancel_requested,
            },
        })
    }
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
                "thread.started",
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
            Some("Claude reported an inaugural Start result error".to_string()),
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
}
