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

#[cfg(all(test, target_os = "linux"))]
mod native_caller_identity_help_tests;
mod native_terminal;
pub(crate) use native_terminal::{
    finish_after_native_ack, NativeAttemptCustody, NativeTerminalObservation,
    PendingProviderTerminal,
};
use native_terminal::{retain_native_completion, retain_native_startup};

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
        custody: NativeAttemptCustody,
        cancel_requested: Arc<AtomicBool>,
    },
    ClaudeNative {
        handle: claude_code::ClaudeTerminationHandle,
        custody: NativeAttemptCustody,
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
                custody,
                cancel_requested,
            } => {
                cancel_requested.store(true, std::sync::atomic::Ordering::SeqCst);
                custody.request_termination();
                handle.request_termination();
            }
            Self::ClaudeNative {
                handle,
                custody,
                cancel_requested,
            } => {
                cancel_requested.store(true, std::sync::atomic::Ordering::SeqCst);
                custody.request_termination();
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
    pub(crate) terminal: NativeTerminalObservation,
}

pub(crate) struct PromptFulfillmentBridge {
    runtime: GatewayAdapterRuntime,
    backend_kind: AgentRuntimeBackendKind,
    binary_path: PathBuf,
    #[cfg(target_os = "linux")]
    ordinary: Option<
        crate::execution::agent_runtime::native_launch_context::OrdinaryNativePromptContextV1,
    >,
}

impl PromptFulfillmentBridge {
    pub(crate) fn native_attempt(&self) -> NativeAttemptCustody {
        match self.backend_kind {
            AgentRuntimeBackendKind::Codex => NativeAttemptCustody::codex(),
            AgentRuntimeBackendKind::ClaudeCode => NativeAttemptCustody::claude(),
        }
    }

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
            #[cfg(target_os = "linux")]
            ordinary: None,
        })
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn for_ordinary_descriptor(
        descriptor: &RuntimeSelectionDescriptor,
        selection: Arc<crate::execution::agent_runtime::native_launch_context::AuthenticatedOrdinarySelectionV1>,
        runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
    ) -> anyhow::Result<Self> {
        let ordinary = crate::execution::agent_runtime::native_launch_context::OrdinaryNativePromptContextV1::new(selection, runtime, descriptor)?;
        let mut bridge = Self::for_descriptor(descriptor)?;
        bridge.ordinary = Some(ordinary);
        Ok(bridge)
    }

    #[cfg(target_os = "linux")]
    async fn prepare_ordinary_native_launch(
        &self,
        request: &mut AgentWrapperRunRequest,
        attempt: &NativeAttemptCustody,
    ) -> Result<
        Option<
            crate::execution::agent_runtime::native_launch_context::PreparedOrdinaryNativePromptV1,
        >,
        AgentWrapperError,
    > {
        let Some(ordinary) = self.ordinary.as_ref() else {
            return Ok(None);
        };
        let acquisition = attempt.begin_host_acquisition()?;
        let prepared = tokio::select! {
            biased;
            _ = attempt.cancelled() => Err(AgentWrapperError::Backend {
                message: "ordinary native Host preparation cancelled".into(),
            }),
            prepared = ordinary.prepare(Arc::clone(&acquisition)) => prepared.map_err(|error| AgentWrapperError::Backend {
                message: format!("ordinary native managed route refused: {error}"),
            }),
        };
        let prepared = finish_host_preparation_v1(
            prepared.and_then(|prepared| {
                prepared
                    .revalidate()
                    .map_err(|error| AgentWrapperError::Backend {
                        message: error.to_string(),
                    })?;
                Ok(prepared)
            }),
            &acquisition,
        )
        .await?;
        request.working_dir = Some(PathBuf::from(prepared.workspace_root()));
        prepare_ordinary_native_env(
            &mut request.env,
            &prepared.home,
            self.backend_kind,
            &prepared.model,
            &prepared.anthropic_base_url,
            prepared.local_route_capability(),
        );
        Ok(Some(prepared))
    }

    pub(crate) async fn run_control(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        #[cfg(target_os = "linux")]
        if self.ordinary.is_some() {
            return Err(AgentWrapperError::InvalidRequest {
                message: "ordinary selected native bridge requires explicit Start or exact ResumeOneTurn controls".into(),
            });
        }
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let control = self.runtime.run_control(request).await?;
        let mut handle = control.handle;
        let terminate = control.cancel.clone();
        let (completion, terminal) = retain_native_completion(
            handle.completion,
            control.cancel.settlement(),
            (),
            move || terminate.cancel(),
            None,
        );
        handle.completion = completion;
        Ok(PromptFulfillmentRunControl {
            handle,
            cancel: PromptFulfillmentCancelHandle::Gateway {
                handle: control.cancel,
                cancel_requested,
            },
            terminal,
        })
    }

    /// Runs only the inaugural durable public Start exchange. Unlike the
    /// compatibility gateway event mapping, this path retains the provider's
    /// typed lifecycle discriminators and exact native identities.
    pub(crate) async fn run_durable_start_control(
        &self,
        request: AgentWrapperRunRequest,
        attempt: &NativeAttemptCustody,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        match self.backend_kind {
            AgentRuntimeBackendKind::Codex => self.run_codex_durable_start(request, attempt).await,
            AgentRuntimeBackendKind::ClaudeCode => {
                self.run_claude_durable_start(request, attempt).await
            }
        }
    }

    /// Runs only a previously authenticated HSA ResumeOneTurn successor. The
    /// expected session ID comes from durable authority, not from the request;
    /// the request extension must exact-match it before the provider is spawned.
    pub(crate) async fn run_authority_managed_resume_one_turn_control(
        &self,
        expected_session_id: &str,
        request: AgentWrapperRunRequest,
        attempt: &NativeAttemptCustody,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        validate_exact_resume_extension(&request.extensions, expected_session_id)?;
        match self.backend_kind {
            AgentRuntimeBackendKind::Codex => {
                self.run_codex_authority_managed_resume(expected_session_id, request, attempt)
                    .await
            }
            AgentRuntimeBackendKind::ClaudeCode => {
                self.run_claude_authority_managed_resume(expected_session_id, request, attempt)
                    .await
            }
        }
    }

    async fn run_codex_durable_start(
        &self,
        request: AgentWrapperRunRequest,
        attempt: &NativeAttemptCustody,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let NativeAttemptCustody::Codex {
            process: process_custody,
            ..
        } = attempt
        else {
            return Err(AgentWrapperError::InvalidRequest {
                message: "native custody backend mismatch".into(),
            });
        };
        let process_custody = process_custody.clone();
        if !request.extensions.is_empty() {
            return Err(AgentWrapperError::InvalidRequest {
                message: "durable Codex Start does not accept continuation extensions".to_string(),
            });
        }
        #[cfg(target_os = "linux")]
        let mut request = request;
        #[cfg(target_os = "linux")]
        let ordinary_native = self
            .prepare_ordinary_native_launch(&mut request, attempt)
            .await?;
        #[cfg(target_os = "linux")]
        if let Some(prepared) = ordinary_native.as_ref() {
            attempt.bind_host_route(prepared.gateway_lease()).await?;
        }
        #[cfg(target_os = "linux")]
        let ordinary_timeout = request.timeout;
        let mut builder = codex::CodexClient::builder()
            .binary(self.binary_path.clone())
            .json(true)
            .mirror_stdout(false)
            .quiet(true)
            .color_mode(codex::ColorMode::Never)
            .dangerously_bypass_approvals_and_sandbox(true)
            .timeout(request.timeout.unwrap_or(Duration::ZERO));
        #[cfg(target_os = "linux")]
        if let Some(prepared) = ordinary_native.as_ref() {
            builder = configure_selected_codex_builder(
                builder,
                &prepared.model,
                &prepared.home,
                &prepared.openai_base_url,
            )
            .map_err(|error| AgentWrapperError::InvalidRequest {
                message: error.to_string(),
            })?
            .timeout(Duration::ZERO);
        }
        if let Some(working_dir) = request.working_dir {
            builder = builder.working_dir(working_dir);
        }
        let client = builder.build();
        let native_request = codex::ExecStreamRequest {
            prompt: request.prompt,
            ephemeral: false,
            ignore_rules: false,
            ignore_user_config: false,
            idle_timeout: None,
            output_last_message: None,
            output_schema: None,
            json_event_log: None,
        };
        let env = request.env;
        #[cfg(target_os = "linux")]
        let resources = ordinary_native;
        #[cfg(not(target_os = "linux"))]
        let resources = ();
        let (control, resources) = retain_native_startup(
            async move {
                client
                    .stream_exec_with_env_overrides_custody(native_request, &env, &process_custody)
                    .await
                    .map_err(|error| AgentWrapperError::Backend {
                        message: format!("Codex durable Start failed: {error}"),
                    })
            },
            attempt.clone(),
            resources,
        )
        .await?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let events = Box::pin(control.events.map(|event| match event {
            Ok(event) => codex_start_event(event),
            Err(error) => {
                observe_codex_native_stream_error("start", &error);
                typed_adapter_error("codex", format!("Codex event error: {error}"))
            }
        }));
        let completion: agent_api::DynAgentWrapperCompletion = Box::pin(async move {
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
        #[cfg(target_os = "linux")]
        let timeout = resources.with_resource(|resources| resources.as_ref().and(ordinary_timeout));
        #[cfg(not(target_os = "linux"))]
        let timeout = None;
        let termination = control.termination.clone();
        let attempt_termination = attempt.clone();
        let (completion, terminal) = resources.retain_completion(
            completion,
            Some(attempt.settlement()),
            move || {
                attempt_termination.request_termination();
                termination.request_termination();
            },
            timeout,
        );
        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::CodexNative {
                handle: control.termination,
                custody: attempt.clone(),
                cancel_requested,
            },
            terminal,
        })
    }

    async fn run_codex_authority_managed_resume(
        &self,
        expected_session_id: &str,
        request: AgentWrapperRunRequest,
        attempt: &NativeAttemptCustody,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let NativeAttemptCustody::Codex {
            process: process_custody,
            ..
        } = attempt
        else {
            return Err(AgentWrapperError::InvalidRequest {
                message: "native custody backend mismatch".into(),
            });
        };
        let process_custody = process_custody.clone();
        #[cfg(target_os = "linux")]
        let mut request = request;
        #[cfg(target_os = "linux")]
        let ordinary_native = self
            .prepare_ordinary_native_launch(&mut request, attempt)
            .await?;
        #[cfg(target_os = "linux")]
        if let Some(prepared) = ordinary_native.as_ref() {
            attempt.bind_host_route(prepared.gateway_lease()).await?;
        }
        #[cfg(target_os = "linux")]
        let ordinary_timeout = request.timeout;
        let mut builder = codex::CodexClient::builder()
            .binary(self.binary_path.clone())
            .json(true)
            .mirror_stdout(false)
            .quiet(true)
            .color_mode(codex::ColorMode::Never)
            .dangerously_bypass_approvals_and_sandbox(true)
            .timeout(request.timeout.unwrap_or(Duration::ZERO));
        #[cfg(target_os = "linux")]
        if let Some(prepared) = ordinary_native.as_ref() {
            builder = configure_selected_codex_builder(
                builder,
                &prepared.model,
                &prepared.home,
                &prepared.openai_base_url,
            )
            .map_err(|error| AgentWrapperError::InvalidRequest {
                message: error.to_string(),
            })?
            .timeout(Duration::ZERO);
        }
        if let Some(working_dir) = request.working_dir {
            builder = builder.working_dir(working_dir);
        }
        let client = builder.build();
        let native_request =
            codex::ResumeRequest::with_id(expected_session_id).prompt(request.prompt);
        let env = request.env;
        #[cfg(target_os = "linux")]
        let resources = ordinary_native;
        #[cfg(not(target_os = "linux"))]
        let resources = ();
        let (control, resources) = retain_native_startup(
            async move {
                client
                    .stream_resume_with_env_overrides_custody(
                        native_request,
                        &env,
                        &process_custody,
                    )
                    .await
                    .map_err(|error| AgentWrapperError::Backend {
                        message: format!("Codex authority-managed ResumeOneTurn failed: {error}"),
                    })
            },
            attempt.clone(),
            resources,
        )
        .await?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let events = Box::pin(control.events.map(|event| match event {
            Ok(event) => codex_resume_event(event),
            Err(error) => {
                observe_codex_native_stream_error("resume", &error);
                typed_adapter_error("codex", format!("Codex ResumeOneTurn event error: {error}"))
            }
        }));
        let completion: agent_api::DynAgentWrapperCompletion = Box::pin(async move {
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
        #[cfg(target_os = "linux")]
        let timeout = resources.with_resource(|resources| resources.as_ref().and(ordinary_timeout));
        #[cfg(not(target_os = "linux"))]
        let timeout = None;
        let termination = control.termination.clone();
        let attempt_termination = attempt.clone();
        let (completion, terminal) = resources.retain_completion(
            completion,
            Some(attempt.settlement()),
            move || {
                attempt_termination.request_termination();
                termination.request_termination();
            },
            timeout,
        );
        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::CodexNative {
                handle: control.termination,
                custody: attempt.clone(),
                cancel_requested,
            },
            terminal,
        })
    }

    async fn run_claude_durable_start(
        &self,
        request: AgentWrapperRunRequest,
        attempt: &NativeAttemptCustody,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let NativeAttemptCustody::Claude {
            process: process_custody,
            ..
        } = attempt
        else {
            return Err(AgentWrapperError::InvalidRequest {
                message: "native custody backend mismatch".into(),
            });
        };
        let process_custody = process_custody.clone();
        if !request.extensions.is_empty() {
            return Err(AgentWrapperError::InvalidRequest {
                message: "durable Claude Start does not accept continuation extensions".to_string(),
            });
        }
        #[cfg(target_os = "linux")]
        let mut request = request;
        #[cfg(target_os = "linux")]
        let ordinary_native = self
            .prepare_ordinary_native_launch(&mut request, attempt)
            .await?;
        #[cfg(target_os = "linux")]
        if let Some(prepared) = ordinary_native.as_ref() {
            attempt.bind_host_route(prepared.gateway_lease()).await?;
        }
        #[cfg(target_os = "linux")]
        let ordinary_timeout = request.timeout;
        let mut builder = claude_code::ClaudeClient::builder()
            .binary(self.binary_path.clone())
            .mirror_stdout(false)
            .mirror_stderr(false);
        #[cfg(target_os = "linux")]
        if ordinary_native.is_some() {
            builder = builder.timeout(None);
        }
        if let Some(working_dir) = request.working_dir {
            builder = builder.working_dir(working_dir);
        }
        if let Some(timeout) = request.timeout {
            #[cfg(target_os = "linux")]
            if ordinary_native.is_none() {
                builder = builder.timeout(Some(timeout));
            }
            #[cfg(not(target_os = "linux"))]
            {
                builder = builder.timeout(Some(timeout));
            }
        }
        for (key, value) in request.env {
            builder = builder.env(key, value);
        }
        let native_request = claude_code::ClaudePrintRequest::new(request.prompt)
            .output_format(claude_code::ClaudeOutputFormat::StreamJson)
            .permission_mode("bypassPermissions");
        #[cfg(target_os = "linux")]
        let native_request = match ordinary_native.as_ref() {
            Some(prepared) => native_request.model(prepared.model.clone()),
            None => native_request,
        };
        let client = builder.build();
        #[cfg(target_os = "linux")]
        let resources = ordinary_native;
        #[cfg(not(target_os = "linux"))]
        let resources = ();
        let (control, resources) = retain_native_startup(
            async move {
                client
                    .print_stream_json_control_with_custody(native_request, &process_custody)
                    .await
                    .map_err(|error| AgentWrapperError::Backend {
                        message: format!("Claude durable Start failed: {error}"),
                    })
            },
            attempt.clone(),
            resources,
        )
        .await?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let events = Box::pin(control.events.map(|event| match event {
            Ok(event) => claude_start_event(event),
            Err(error) => {
                typed_adapter_error("claude_code", format!("Claude event error: {error}"))
            }
        }));
        let completion: agent_api::DynAgentWrapperCompletion = Box::pin(async move {
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
        #[cfg(target_os = "linux")]
        let timeout = resources.with_resource(|resources| resources.as_ref().and(ordinary_timeout));
        #[cfg(not(target_os = "linux"))]
        let timeout = None;
        let termination = control.termination.clone();
        let attempt_termination = attempt.clone();
        let (completion, terminal) = resources.retain_completion(
            completion,
            Some(attempt.settlement()),
            move || {
                attempt_termination.request_termination();
                termination.request_termination();
            },
            timeout,
        );
        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::ClaudeNative {
                handle: control.termination,
                custody: attempt.clone(),
                cancel_requested,
            },
            terminal,
        })
    }

    async fn run_claude_authority_managed_resume(
        &self,
        expected_session_id: &str,
        request: AgentWrapperRunRequest,
        attempt: &NativeAttemptCustody,
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let NativeAttemptCustody::Claude {
            process: process_custody,
            ..
        } = attempt
        else {
            return Err(AgentWrapperError::InvalidRequest {
                message: "native custody backend mismatch".into(),
            });
        };
        let process_custody = process_custody.clone();
        #[cfg(target_os = "linux")]
        let mut request = request;
        #[cfg(target_os = "linux")]
        let ordinary_native = self
            .prepare_ordinary_native_launch(&mut request, attempt)
            .await?;
        #[cfg(target_os = "linux")]
        if let Some(prepared) = ordinary_native.as_ref() {
            attempt.bind_host_route(prepared.gateway_lease()).await?;
        }
        #[cfg(target_os = "linux")]
        let ordinary_timeout = request.timeout;
        let mut builder = claude_code::ClaudeClient::builder()
            .binary(self.binary_path.clone())
            .mirror_stdout(false)
            .mirror_stderr(false);
        #[cfg(target_os = "linux")]
        if ordinary_native.is_some() {
            builder = builder.timeout(None);
        }
        if let Some(working_dir) = request.working_dir {
            builder = builder.working_dir(working_dir);
        }
        if let Some(timeout) = request.timeout {
            #[cfg(target_os = "linux")]
            if ordinary_native.is_none() {
                builder = builder.timeout(Some(timeout));
            }
            #[cfg(not(target_os = "linux"))]
            {
                builder = builder.timeout(Some(timeout));
            }
        }
        for (key, value) in request.env {
            builder = builder.env(key, value);
        }
        let native_request = claude_code::ClaudePrintRequest::new(request.prompt)
            .output_format(claude_code::ClaudeOutputFormat::StreamJson)
            .permission_mode("bypassPermissions")
            .resume_value(expected_session_id);
        #[cfg(target_os = "linux")]
        let native_request = match ordinary_native.as_ref() {
            Some(prepared) => native_request.model(prepared.model.clone()),
            None => native_request,
        };
        let client = builder.build();
        #[cfg(target_os = "linux")]
        let resources = ordinary_native;
        #[cfg(not(target_os = "linux"))]
        let resources = ();
        let (control, resources) = retain_native_startup(
            async move {
                client
                    .print_stream_json_control_with_custody(native_request, &process_custody)
                    .await
                    .map_err(|error| AgentWrapperError::Backend {
                        message: format!("Claude authority-managed ResumeOneTurn failed: {error}"),
                    })
            },
            attempt.clone(),
            resources,
        )
        .await?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let events = Box::pin(control.events.map(|event| match event {
            Ok(event) => claude_resume_event(event),
            Err(error) => typed_adapter_error(
                "claude_code",
                format!("Claude ResumeOneTurn event error: {error}"),
            ),
        }));
        let completion: agent_api::DynAgentWrapperCompletion = Box::pin(async move {
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
        #[cfg(target_os = "linux")]
        let timeout = resources.with_resource(|resources| resources.as_ref().and(ordinary_timeout));
        #[cfg(not(target_os = "linux"))]
        let timeout = None;
        let termination = control.termination.clone();
        let attempt_termination = attempt.clone();
        let (completion, terminal) = resources.retain_completion(
            completion,
            Some(attempt.settlement()),
            move || {
                attempt_termination.request_termination();
                termination.request_termination();
            },
            timeout,
        );
        Ok(PromptFulfillmentRunControl {
            handle: AgentWrapperRunHandle { events, completion },
            cancel: PromptFulfillmentCancelHandle::ClaudeNative {
                handle: control.termination,
                custody: attempt.clone(),
                cancel_requested,
            },
            terminal,
        })
    }
}

// Private transient transport slot; the existing sensitive-prefix scrub clears
// inherited/request values before the held turn capability is installed.
#[cfg(target_os = "linux")]
const ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV: &str = "SUBSTRATE_INTERNAL_CODEX_LOCAL_ROUTE_CAPABILITY";

#[cfg(target_os = "linux")]
fn configure_selected_codex_builder(
    builder: codex::CodexClientBuilder,
    model: &str,
    home: &std::path::Path,
    endpoint: &str,
) -> anyhow::Result<codex::CodexClientBuilder> {
    crate::execution::agent_runtime::native_launch_context::validate_native_loopback_endpoint(
        endpoint,
    )?;
    let quote = |value: &str| toml::Value::String(value.into()).to_string();
    let provider = format!("{{ substrate-managed-gateway = {{ name = \"Substrate managed gateway\", base_url = {}, wire_api = \"responses\", env_key = {}, requires_openai_auth = false, supports_websockets = false }} }}", quote(&format!("{endpoint}/v1")), quote(ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV));
    Ok(builder
        .model(model)
        .codex_home(home)
        .create_home_dirs(false)
        .config_override("model_provider", quote("substrate-managed-gateway"))
        .config_override("model_providers", provider)
        .config_override("check_for_update_on_startup", "false")
        .config_override("cli_auth_credentials_store", quote("ephemeral")))
}

#[cfg(target_os = "linux")]
fn prepare_ordinary_native_env(
    env: &mut BTreeMap<String, String>,
    home: &std::path::Path,
    backend: AgentRuntimeBackendKind,
    model: &str,
    anthropic_endpoint: &str,
    local_route_capability: &str,
) {
    let sensitive = |key: &str| {
        [
            "OPENAI_",
            "ANTHROPIC_",
            "CODEX_",
            "CLAUDE_",
            "CHATGPT_",
            "SUBSTRATE_LLM_",
            "SUBSTRATE_INTERNAL_CODEX_",
        ]
        .iter()
        .any(|prefix| key.starts_with(prefix))
            || [
                "HTTP_PROXY",
                "HTTPS_PROXY",
                "ALL_PROXY",
                "NO_PROXY",
                "http_proxy",
                "https_proxy",
                "all_proxy",
                "no_proxy",
            ]
            .contains(&key)
    };
    env.retain(|key, _| !sensitive(key));
    // The pinned SDK exposes string overrides, not env_remove. These empty
    // inherited provider fields contain no credential or upstream route.
    for (key, _) in std::env::vars_os() {
        if let Some(key) = key.to_str().filter(|key| sensitive(key)) {
            env.insert(key.into(), String::new());
        }
    }
    for key in [
        "HOME",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_STATE_HOME",
        "XDG_CACHE_HOME",
    ] {
        env.insert(key.into(), home.display().to_string());
    }
    env.insert("NO_PROXY".into(), "127.0.0.1,::1,localhost".into());
    match backend {
        AgentRuntimeBackendKind::Codex => {
            env.insert("CODEX_HOME".into(), home.display().to_string());
            env.insert(
                ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV.into(),
                local_route_capability.into(),
            );
        }
        AgentRuntimeBackendKind::ClaudeCode => {
            // This short-lived local capability authenticates the exact managed
            // Host route. It is neither upstream auth nor original HSA authority.
            env.insert("CLAUDE_CONFIG_DIR".into(), home.display().to_string());
            env.insert("ANTHROPIC_BASE_URL".into(), anthropic_endpoint.into());
            env.insert("ANTHROPIC_AUTH_TOKEN".into(), local_route_capability.into());
            env.insert("ANTHROPIC_MODEL".into(), model.into());
        }
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

fn observe_codex_native_stream_error(operation: &'static str, error: &codex::ExecStreamError) {
    use codex::{CodexError as C, ExecStreamError as E};
    let (stream_error_variant, codex_error_variant) = match error {
        E::Codex(error) => (
            "codex",
            match error {
                C::Spawn { .. } => "spawn",
                C::Wait { .. } => "wait",
                C::Timeout { .. } => "timeout",
                C::NonZeroExit { .. } => "non_zero_exit",
                C::InvalidUtf8(_) => "invalid_utf8",
                C::JsonParse { .. } => "json_parse",
                C::ExecPolicyParse { .. } => "exec_policy_parse",
                C::FeatureListParse { .. } => "feature_list_parse",
                C::ResponsesApiProxyInfoRead { .. } => "responses_api_proxy_info_read",
                C::ResponsesApiProxyInfoParse { .. } => "responses_api_proxy_info_parse",
                C::EmptyPrompt => "empty_prompt",
                C::EmptySandboxCommand => "empty_sandbox_command",
                C::EmptyExecPolicyCommand => "empty_exec_policy_command",
                C::EmptyApiKey => "empty_api_key",
                C::EmptyTaskId => "empty_task_id",
                C::EmptyEnvId => "empty_env_id",
                C::EmptyMcpServerName => "empty_mcp_server_name",
                C::EmptyMcpCommand => "empty_mcp_command",
                C::EmptyMcpUrl => "empty_mcp_url",
                C::EmptySocketPath => "empty_socket_path",
                C::TempDir(_) => "temp_dir",
                C::WorkingDirectory { .. } => "working_directory",
                C::PrepareOutputDirectory { .. } => "prepare_output_directory",
                C::PrepareCodexHome { .. } => "prepare_codex_home",
                C::StdoutUnavailable => "stdout_unavailable",
                C::StderrUnavailable => "stderr_unavailable",
                C::StdinUnavailable => "stdin_unavailable",
                C::CaptureIo(_) => "capture_io",
                C::StdinWrite(_) => "stdin_write",
                C::Join(_) => "join",
            },
        ),
        E::Parse { .. } => ("parse", "none"),
        E::Normalize { .. } => ("normalize", "none"),
        E::IdleTimeout { .. } => ("idle_timeout", "none"),
        E::ChannelClosed => ("channel_closed", "none"),
    };
    tracing::debug!(target: "substrate::native_stream_error",
        operation, stream_error_variant, codex_error_variant,
        "native Codex stream failed");
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
                .run_durable_start_control(
                    AgentWrapperRunRequest {
                        prompt: "real Claude Start prompt".to_string(),
                        working_dir: Some(temp.path().to_path_buf()),
                        timeout: Some(Duration::from_secs(5)),
                        env: BTreeMap::new(),
                        extensions: BTreeMap::new(),
                    },
                    &bridge.native_attempt(),
                )
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
                    &bridge.native_attempt(),
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
                    &bridge.native_attempt(),
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
                        &bridge.native_attempt(),
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

    #[cfg(target_os = "linux")]
    #[test]
    fn test_selected_native_env_excludes_inherited_provider_custody() {
        let _env = crate::execution::AuthorityEnvTestGuard::preserve();
        std::env::set_var("OPENAI_API_KEY", "synthetic-parent-provider-secret");
        std::env::set_var(
            "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
            "synthetic-parent-account-secret",
        );
        let previous_cap = std::env::var_os(ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV);
        std::env::set_var(
            ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV,
            "synthetic-parent-stale-cap",
        );
        let mut env = BTreeMap::from([
            (
                ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV.into(),
                "synthetic-request-stale-cap".into(),
            ),
            (
                "SUBSTRATE_LLM_HOST_CLEANUP_TOKEN".into(),
                "synthetic-request-cleanup-secret".into(),
            ),
            (
                "ANTHROPIC_API_KEY".into(),
                "synthetic-request-secret".into(),
            ),
            ("CODEX_HOME".into(), "/untrusted/home".into()),
            (
                SUBSTRATE_AGENT_TOOLBOX_ENDPOINT_ENV.into(),
                "unix:///synthetic/toolbox".into(),
            ),
        ]);
        prepare_ordinary_native_env(
            &mut env,
            std::path::Path::new("/synthetic/held-home"),
            AgentRuntimeBackendKind::Codex,
            "synthetic-model",
            "http://127.0.0.1:4123",
            "synthetic-held-route-capability",
        );
        match previous_cap {
            Some(value) => std::env::set_var(ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV, value),
            None => std::env::remove_var(ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV),
        }
        assert_eq!(
            env[ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV],
            "synthetic-held-route-capability"
        );
        assert_eq!(
            env.values()
                .filter(|value| *value == "synthetic-held-route-capability")
                .count(),
            1
        );
        assert!(
            !env.contains_key("SUBSTRATE_LLM_HOST_CLEANUP_TOKEN")
                || env["SUBSTRATE_LLM_HOST_CLEANUP_TOKEN"].is_empty()
        );
        assert_eq!(env["OPENAI_API_KEY"], "");
        assert_eq!(env["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN"], "");
        assert!(!env.contains_key("ANTHROPIC_API_KEY") || env["ANTHROPIC_API_KEY"].is_empty());
        assert_eq!(env["CODEX_HOME"], "/synthetic/held-home");
        assert_eq!(
            env[SUBSTRATE_AGENT_TOOLBOX_ENDPOINT_ENV],
            "unix:///synthetic/toolbox"
        );
        assert!(
            !env.values()
                .any(|value| value.contains("synthetic-parent")
                    || value.contains("synthetic-request"))
        );
        prepare_ordinary_native_env(
            &mut env,
            std::path::Path::new("/synthetic/held-home"),
            AgentRuntimeBackendKind::ClaudeCode,
            "claude-sonnet-4.5",
            "http://127.0.0.1:4123",
            "synthetic-held-claude-capability",
        );
        assert!(
            !env.contains_key(ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV)
                || env[ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV].is_empty()
        );
        assert!(!env
            .values()
            .any(|value| value == "synthetic-held-route-capability"));
        assert_eq!(
            env.values()
                .filter(|value| *value == "synthetic-held-claude-capability")
                .count(),
            1
        );
        assert_eq!(env["ANTHROPIC_BASE_URL"], "http://127.0.0.1:4123");
        assert_eq!(
            env["ANTHROPIC_AUTH_TOKEN"],
            "synthetic-held-claude-capability"
        );
        assert_eq!(env["ANTHROPIC_MODEL"], "claude-sonnet-4.5");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_selected_codex_builder_pins_managed_route_and_model() {
        use std::os::unix::fs::PermissionsExt;
        let _env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempfile::tempdir().unwrap();
        let script = temp.path().join("synthetic-codex");
        let args = temp.path().join("args");
        std::fs::write(&script, format!(
            "#!/bin/sh\n[ \"$SUBSTRATE_INTERNAL_CODEX_LOCAL_ROUTE_CAPABILITY\" = synthetic-held-route-capability ] || exit 91\nprintf '%s\\n' \"$@\" >> '{}'\n/bin/cat >/dev/null\nprintf '{{\"type\":\"thread.started\",\"thread_id\":\"component-history\"}}\\n'\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"component-history\",\"turn_id\":\"component-turn\"}}\\n'\n", args.display()
        )).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut native_env = BTreeMap::new();
        prepare_ordinary_native_env(
            &mut native_env,
            temp.path(),
            AgentRuntimeBackendKind::Codex,
            "synthetic-pinned-model",
            "http://127.0.0.1:4123",
            "synthetic-held-route-capability",
        );
        runtime.block_on(async {
            let builder = || {
                configure_selected_codex_builder(
                    codex::CodexClient::builder()
                        .binary(&script)
                        .json(true)
                        .quiet(true)
                        .mirror_stdout(false)
                        .working_dir(temp.path()),
                    "synthetic-pinned-model",
                    temp.path(),
                    "http://127.0.0.1:4123",
                )
                .unwrap()
                .build()
            };
            let control = builder()
                .stream_exec_with_env_overrides_control(
                    codex::ExecStreamRequest {
                        prompt: "component Start".into(),
                        ephemeral: false,
                        ignore_rules: false,
                        ignore_user_config: false,
                        idle_timeout: None,
                        output_last_message: None,
                        output_schema: None,
                        json_event_log: None,
                    },
                    &native_env,
                )
                .await
                .unwrap();
            let mut events = control.events;
            while events.next().await.is_some() {}
            control.completion.await.unwrap();
            for prompt in ["component Resume one", "component Resume two"] {
                let control = builder()
                    .stream_resume_with_env_overrides_control(
                        codex::ResumeRequest::with_id("component-history").prompt(prompt),
                        &native_env,
                    )
                    .await
                    .unwrap();
                let mut events = control.events;
                while events.next().await.is_some() {}
                control.completion.await.unwrap();
            }
        });
        let args = std::fs::read_to_string(args).unwrap();
        assert_eq!(
            args.lines()
                .filter(|line| *line == "synthetic-pinned-model")
                .count(),
            3
        );
        assert_eq!(
            args.lines()
                .filter(|line| *line == "component-history")
                .count(),
            2
        );
        assert_eq!(
            args.lines()
                .filter(|line| line.contains("base_url = \"http://127.0.0.1:4123/v1\""))
                .count(),
            3
        );
        assert!(!args.contains("access_token"));
        assert!(!args.contains("synthetic-held-route-capability"));
        let providers: Vec<_> = args
            .lines()
            .filter(|line| line.starts_with("model_providers="))
            .map(|line| toml::from_str::<toml::Value>(line).unwrap())
            .collect();
        assert_eq!(
            providers.len(),
            3,
            "Start and both exact Resumes pin the provider"
        );
        for config in providers {
            let providers = config["model_providers"].as_table().unwrap();
            assert_eq!(providers.len(), 1);
            let provider = providers["substrate-managed-gateway"].as_table().unwrap();
            assert_eq!(
                provider["env_key"].as_str(),
                Some(ORDINARY_CODEX_LOCAL_ROUTE_CAP_ENV)
            );
            let mut keys: Vec<_> = provider.keys().map(String::as_str).collect();
            keys.sort_unstable();
            assert_eq!(
                keys,
                vec![
                    "base_url",
                    "env_key",
                    "name",
                    "requires_openai_auth",
                    "supports_websockets",
                    "wire_api"
                ]
            );
        }
        for endpoint in [
            "https://upstream.invalid",
            "http://localhost:4123",
            "http://127.0.0.1:0",
            "http://127.0.0.1:4123/other",
        ] {
            assert!(configure_selected_codex_builder(
                codex::CodexClient::builder(),
                "synthetic-model",
                temp.path(),
                endpoint
            )
            .is_err());
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_selected_completion_drop_and_cancel_retains_home_until_child_terminal() {
        use std::sync::atomic::Ordering;
        struct Custody {
            _home: std::fs::File,
            terminal: Arc<AtomicBool>,
            premature: Arc<AtomicBool>,
            released: Arc<tokio::sync::Notify>,
        }
        impl Drop for Custody {
            fn drop(&mut self) {
                self.premature
                    .store(!self.terminal.load(Ordering::SeqCst), Ordering::SeqCst);
                self.released.notify_one();
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            for mode in ["drop", "cancel", "timeout", "normal"] {
                let terminal = Arc::new(AtomicBool::new(false));
                let premature = Arc::new(AtomicBool::new(false));
                let released = Arc::new(tokio::sync::Notify::new());
                let custody = Custody { _home: std::fs::File::open(temp.path()).unwrap(),
                    terminal: Arc::clone(&terminal), premature: Arc::clone(&premature), released: Arc::clone(&released) };
                let mut child = tokio::process::Command::new("/bin/sleep")
                    .arg(if mode == "normal" { "0" } else { "10" }).kill_on_drop(true).spawn().unwrap();
                let (stop, mut stopped) = tokio::sync::oneshot::channel::<()>();
                let stop = Arc::new(std::sync::Mutex::new(Some(stop)));
                let terminal_after_wait = Arc::clone(&terminal);
                let completion: agent_api::DynAgentWrapperCompletion = Box::pin(async move {
                    let status = tokio::select! {
                        status = child.wait() => status.unwrap(),
                        _ = &mut stopped => { let _ = child.start_kill(); child.wait().await.unwrap() },
                    };
                    terminal_after_wait.store(true, Ordering::SeqCst);
                    Ok(AgentWrapperCompletion { status, final_text: None, data: None })
                });
                let private_stop = Arc::clone(&stop);
                let (completion, terminal_observation) = retain_native_completion(completion, Some(Box::pin(async { Ok(()) })), custody,
                    move || { if let Some(stop) = private_stop.lock().unwrap().take() { let _ = stop.send(()); } },
                    (mode == "timeout").then_some(Duration::from_millis(10)));
                if mode == "drop" { drop(completion); }
                else {
                    if mode == "cancel" { let _ = stop.lock().unwrap().take().unwrap().send(()); }
                    completion.await.unwrap();
                }
                tokio::time::timeout(Duration::from_secs(5), released.notified()).await.unwrap();
                assert!(terminal.load(Ordering::SeqCst), "{mode}");
                assert!(!premature.load(Ordering::SeqCst), "{mode}");
                assert!(terminal_observation.wait().await.require_checked().is_ok());
            }
        });
    }
}

#[cfg(target_os = "linux")]
async fn finish_host_preparation_v1<T>(
    result: Result<T, AgentWrapperError>,
    acquisition: &crate::execution::agent_runtime::native_launch_context::HostGatewayAcquisitionCustodyV1,
) -> Result<T, AgentWrapperError> {
    match result {
        Ok(prepared) => Ok(prepared),
        Err(error) => match acquisition.close_and_wait().await {
            Ok(_) => Err(error),
            Err(cleanup) => Err(AgentWrapperError::Backend {
                message: format!("{error}; Host cleanup: {cleanup}"),
            }),
        },
    }
}

#[cfg(all(test, target_os = "linux"))]
mod pending_host_preparation_tests {
    use super::*;
    use crate::execution::agent_runtime::native_launch_context::HostGatewayCleanupOutcomeV1;
    #[tokio::test]
    async fn test_pending_host_preparation_joins_checked_cleanup() {
        // Finite pending-owner fixtures, without a fabricated lease/admission,
        // service exchange, native process or provider effect. Exercise the same
        // production error join for cancellation, unknown delivery and an early
        // post-acquisition validation failure before bind_host_route.
        for error in [
            "preparation cancelled",
            "submission_outcome_unknown: response lost",
            "post-acquisition revalidation failed",
        ] {
            for custody in [
                NativeAttemptCustody::codex(),
                NativeAttemptCustody::claude(),
            ] {
                let acquisition = custody.begin_host_acquisition().unwrap();
                let completion = acquisition.begin().unwrap();
                if error == "preparation cancelled" {
                    custody.request_termination();
                }
                let terminal = finish_host_preparation_v1::<()>(
                    Err(AgentWrapperError::Backend {
                        message: error.into(),
                    }),
                    &acquisition,
                );
                tokio::pin!(terminal);
                let settlement = custody.settlement();
                tokio::pin!(settlement);
                assert!(
                    futures::poll!(&mut terminal).is_pending(),
                    "public error cannot bypass cleanup"
                );
                assert!(
                    futures::poll!(&mut settlement).is_pending(),
                    "empty native custody cannot bypass pending Host ownership"
                );
                assert!(
                    custody.clone().begin_host_acquisition().is_err(),
                    "uncertainty cannot be rebound to a second acquisition"
                );
                // Explicit checked fixture acknowledgment, not process absence.
                completion
                    .send(Some(HostGatewayCleanupOutcomeV1::Released))
                    .unwrap();
                assert!(terminal.await.unwrap_err().to_string().contains(error));
                settlement.await.unwrap();
            }
        }
        let custody = NativeAttemptCustody::codex();
        let acquisition = custody.begin_host_acquisition().unwrap();
        let completion = acquisition.begin().unwrap();
        let mut terminal = Box::pin(finish_host_preparation_v1::<()>(
            Err(AgentWrapperError::Backend {
                message: "observer lost".into(),
            }),
            &acquisition,
        ));
        assert!(futures::poll!(&mut terminal).is_pending());
        // Dropping the response/error awaiter does not detach pending custody.
        drop(terminal);
        let settlement = custody.settlement();
        tokio::pin!(settlement);
        assert!(futures::poll!(&mut settlement).is_pending());
        completion
            .send(Some(HostGatewayCleanupOutcomeV1::Unknown))
            .unwrap();
        assert!(settlement
            .await
            .unwrap_err()
            .to_string()
            .contains("cleanup unacknowledged"));
        assert!(custody.begin_host_acquisition().is_err());
        let abandoned = NativeAttemptCustody::codex();
        let acquisition = abandoned.begin_host_acquisition().unwrap();
        let completion = acquisition.begin().unwrap();
        drop(completion);
        assert!(abandoned
            .settlement()
            .await
            .unwrap_err()
            .to_string()
            .contains("owner unavailable"));
        assert!(abandoned.begin_host_acquisition().is_err());
        let no_effect = NativeAttemptCustody::claude();
        let acquisition = no_effect.begin_host_acquisition().unwrap();
        assert_eq!(
            acquisition.close_and_wait().await.unwrap(),
            HostGatewayCleanupOutcomeV1::NoAcquireSent
        );
        assert!(acquisition.begin().is_err());
        no_effect.settlement().await.unwrap();
        assert!(no_effect.begin_host_acquisition().is_err());
    }
}
