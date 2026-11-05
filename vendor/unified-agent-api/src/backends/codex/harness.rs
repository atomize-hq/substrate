use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    process::ExitStatus,
    sync::{Arc, Mutex},
    time::Duration,
};

use codex::{CodexError, ExecStreamError, ThreadEvent};
use futures_util::{stream, StreamExt};

use super::{
    mapping::{error_event, map_thread_event, status_event},
    validate_and_extract_exec_policy, CodexBackendConfig, CAP_SESSION_HANDLE_V1, EXT_ADD_DIRS_V1,
    PINNED_ADD_DIRS_UNSUPPORTED_FOR_FORK, PINNED_EXTERNAL_SANDBOX_WARNING, PINNED_NO_SESSION_FOUND,
    PINNED_SESSION_NOT_FOUND, PINNED_TIMEOUT, SESSION_HANDLE_ID_BOUND_BYTES,
    SESSION_HANDLE_OVERSIZE_WARNING_MARKER, SUPPORTED_EXTENSION_KEYS_DEFAULT,
    SUPPORTED_EXTENSION_KEYS_EXTERNAL_SANDBOX_OPT_IN,
};
use crate::{
    backend_harness::{
        accepted_model_override_v1, normalize_add_dirs_v1, BackendHarnessAdapter,
        BackendHarnessErrorPhase, BackendSpawn, DynBackendEventStream, NormalizedRequest,
    },
    backends::spawn_path::resolve_effective_working_dir,
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperKind, AgentWrapperRunRequest,
};

use super::super::session_selectors::{
    parse_session_resume_v1, validate_resume_fork_mutual_exclusion, EXT_SESSION_RESUME_V1,
};

pub(super) enum CodexTerminationHandle {
    Exec(codex::ExecTerminationHandle),
    AppServerTurn(super::fork::AppServerTurnCancelHandle),
}

impl std::fmt::Debug for CodexTerminationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodexTerminationHandle::Exec(_) => f.debug_tuple("Exec").field(&"<handle>").finish(),
            CodexTerminationHandle::AppServerTurn(_) => {
                f.debug_tuple("AppServerTurn").field(&"<handle>").finish()
            }
        }
    }
}

impl super::super::termination::TerminationHandle for CodexTerminationHandle {
    fn request_termination(&self) {
        match self {
            CodexTerminationHandle::Exec(handle) => {
                codex::ExecTerminationHandle::request_termination(handle);
            }
            CodexTerminationHandle::AppServerTurn(handle) => {
                super::super::termination::TerminationHandle::request_termination(handle);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct CodexHarnessAdapter {
    custody: codex::ExecProcessCustody,
    mcp_custody: codex::mcp::McpProcessCustody,
    config: CodexBackendConfig,
    run_start_cwd: Option<PathBuf>,
    termination: Option<Arc<super::super::termination::TerminationState<CodexTerminationHandle>>>,
    handle_state: Arc<Mutex<CodexHandleFacetState>>,
}

#[derive(Debug, Default)]
pub(super) struct CodexHandleFacetState {
    pub(super) thread_id: Option<String>,
    pub(super) handle_facet_emitted: bool,
    pub(super) oversize_warning_emitted: bool,
    pub(super) oversize_warning_len_bytes: Option<usize>,
}

#[derive(Debug)]
pub(super) enum CodexBackendEvent {
    ExternalSandboxWarning,
    Thread(Box<ThreadEvent>),
    AppServerNotification(codex::mcp::AppNotification),
    NonZeroExit { status: ExitStatus },
    TerminalError { message: String },
}

#[derive(Clone, Debug)]
pub(super) struct CodexBackendCompletion {
    pub(super) status: ExitStatus,
    pub(super) final_text: Option<String>,
    pub(super) backend_error_message: Option<String>,
    pub(super) selection_failure_message: Option<String>,
}

#[derive(Debug)]
pub(super) enum CodexBackendError {
    Exec(ExecStreamError),
    AppServer(codex::mcp::McpError),
    AppServerCleanup {
        original: Box<CodexBackendError>,
        cleanup: codex::mcp::McpError,
    },
    Timeout {
        timeout: Duration,
    },
    AddDirsRejectedByRuntime,
    ForkSelectionEmpty,
    ForkSessionNotFound,
    CompletionTaskDropped,
    WorkingDirectoryUnresolved,
}

pub(super) fn new_harness_adapter(
    config: CodexBackendConfig,
    run_start_cwd: Option<PathBuf>,
    termination: Option<Arc<super::super::termination::TerminationState<CodexTerminationHandle>>>,
) -> CodexHarnessAdapter {
    CodexHarnessAdapter {
        custody: codex::ExecProcessCustody::new(),
        mcp_custody: codex::mcp::McpProcessCustody::new(),
        config,
        run_start_cwd,
        termination,
        handle_state: Arc::new(Mutex::new(CodexHandleFacetState::default())),
    }
}

#[cfg(test)]
pub(super) fn new_test_adapter(config: CodexBackendConfig) -> CodexHarnessAdapter {
    new_harness_adapter(config, None, None)
}

#[cfg(test)]
pub(super) fn new_test_adapter_with_run_start_cwd(
    config: CodexBackendConfig,
    run_start_cwd: Option<PathBuf>,
) -> CodexHarnessAdapter {
    new_harness_adapter(config, run_start_cwd, None)
}

fn effective_working_dir_for_add_dirs(
    config: &CodexBackendConfig,
    run_start_cwd: Option<&PathBuf>,
    request: &AgentWrapperRunRequest,
) -> Result<Option<PathBuf>, AgentWrapperError> {
    if !request.extensions.contains_key(EXT_ADD_DIRS_V1) {
        return Ok(None);
    }

    Ok(resolve_effective_working_dir(
        request.working_dir.as_deref(),
        config.default_working_dir.as_deref(),
        run_start_cwd.map(PathBuf::as_path),
    ))
}

fn codex_error_kind(err: &CodexError) -> &'static str {
    match err {
        CodexError::Spawn { .. } => "spawn",
        CodexError::Wait { .. } => "wait",
        CodexError::Timeout { .. } => "timeout",
        CodexError::EmptyPrompt
        | CodexError::EmptySandboxCommand
        | CodexError::EmptyExecPolicyCommand
        | CodexError::EmptyApiKey
        | CodexError::EmptyTaskId
        | CodexError::EmptyEnvId
        | CodexError::EmptyMcpServerName
        | CodexError::EmptyMcpCommand
        | CodexError::EmptyMcpUrl
        | CodexError::EmptySocketPath => "invalid_request",
        CodexError::TempDir(_)
        | CodexError::WorkingDirectory { .. }
        | CodexError::PrepareOutputDirectory { .. }
        | CodexError::PrepareCodexHome { .. }
        | CodexError::StdoutUnavailable
        | CodexError::StderrUnavailable
        | CodexError::StdinUnavailable
        | CodexError::CaptureIo(_)
        | CodexError::StdinWrite(_)
        | CodexError::ResponsesApiProxyInfoRead { .. }
        | CodexError::Join(_) => "io",
        CodexError::NonZeroExit { .. }
        | CodexError::InvalidUtf8(_)
        | CodexError::JsonParse { .. }
        | CodexError::ExecPolicyParse { .. }
        | CodexError::FeatureListParse { .. }
        | CodexError::ResponsesApiProxyInfoParse { .. } => "other",
    }
}

pub(super) fn redact_exec_stream_error(err: &ExecStreamError) -> String {
    match err {
        ExecStreamError::Parse { source, line } => format!(
            "codex stream parse error (redacted): {source} (line_bytes={})",
            line.len()
        ),
        ExecStreamError::Normalize { message, line } => format!(
            "codex stream normalize error (redacted): {message} (line_bytes={})",
            line.len()
        ),
        ExecStreamError::IdleTimeout { idle_for } => {
            format!("codex stream idle timeout: {idle_for:?}")
        }
        ExecStreamError::ChannelClosed => "codex stream closed unexpectedly".to_string(),
        ExecStreamError::Codex(CodexError::NonZeroExit { status, .. }) => {
            format!("codex exited non-zero: {status:?} (stderr redacted)")
        }
        ExecStreamError::Codex(err) => format!(
            "codex backend error: {} (details redacted when unsafe)",
            codex_error_kind(err)
        ),
    }
}

fn render_backend_error_message(err: &CodexBackendError) -> String {
    match err {
        CodexBackendError::Exec(err) => redact_exec_stream_error(err),
        CodexBackendError::AppServer(codex::mcp::McpError::Handshake(_)) => {
            "codex app-server handshake failed".to_string()
        }
        CodexBackendError::AppServer(_) => "codex app-server rpc error".to_string(),
        CodexBackendError::AppServerCleanup { original, cleanup } => {
            // Both raw outcomes remain retained; expose only safe classification.
            let kind = match cleanup {
                codex::mcp::McpError::Transport(_) => "transport",
                _ => "protocol",
            };
            format!(
                "{}; codex app-server cleanup failed ({kind})",
                render_backend_error_message(original)
            )
        }
        CodexBackendError::Timeout { timeout: _timeout } => PINNED_TIMEOUT.to_string(),
        CodexBackendError::AddDirsRejectedByRuntime => {
            super::PINNED_ADD_DIRS_RUNTIME_REJECTION.to_string()
        }
        CodexBackendError::ForkSelectionEmpty => PINNED_NO_SESSION_FOUND.to_string(),
        CodexBackendError::ForkSessionNotFound => PINNED_SESSION_NOT_FOUND.to_string(),
        CodexBackendError::CompletionTaskDropped => "codex completion task dropped".to_string(),
        CodexBackendError::WorkingDirectoryUnresolved => {
            "codex backend failed to resolve working directory".to_string()
        }
    }
}

pub(super) fn startup_failure_spawn(
    err: CodexBackendError,
    emit_external_sandbox_warning: bool,
) -> BackendSpawn<CodexBackendEvent, CodexBackendCompletion, CodexBackendError> {
    let message = render_backend_error_message(&err);
    let events: DynBackendEventStream<CodexBackendEvent, CodexBackendError> =
        if emit_external_sandbox_warning {
            Box::pin(stream::iter(vec![
                Ok(CodexBackendEvent::ExternalSandboxWarning),
                Ok(CodexBackendEvent::TerminalError { message }),
            ]))
        } else {
            Box::pin(stream::once(async move {
                Ok(CodexBackendEvent::TerminalError { message })
            }))
        };
    let completion = Box::pin(async move { Err(err) });
    BackendSpawn {
        events,
        completion,
        events_observability: None,
    }
}

fn session_handle_facet(thread_id: &str) -> serde_json::Value {
    serde_json::json!({
        "schema": CAP_SESSION_HANDLE_V1,
        "session": { "id": thread_id },
    })
}

impl BackendHarnessAdapter for CodexHarnessAdapter {
    fn kind(&self) -> AgentWrapperKind {
        AgentWrapperKind("codex".to_string())
    }

    fn supported_extension_keys(&self) -> &'static [&'static str] {
        if self.config.allow_external_sandbox_exec {
            SUPPORTED_EXTENSION_KEYS_EXTERNAL_SANDBOX_OPT_IN
        } else {
            SUPPORTED_EXTENSION_KEYS_DEFAULT
        }
    }

    type Policy = super::CodexExecPolicy;

    fn validate_and_extract_policy(
        &self,
        request: &AgentWrapperRunRequest,
    ) -> Result<Self::Policy, AgentWrapperError> {
        let mut exec_policy = validate_and_extract_exec_policy(request)?;

        let effective_working_dir =
            effective_working_dir_for_add_dirs(&self.config, self.run_start_cwd.as_ref(), request)?;
        exec_policy.add_dirs = normalize_add_dirs_v1(
            request.extensions.get(EXT_ADD_DIRS_V1),
            effective_working_dir.as_deref(),
        )?;

        let resume = request
            .extensions
            .get(EXT_SESSION_RESUME_V1)
            .map(parse_session_resume_v1)
            .transpose()?;

        let fork = super::fork::extract_fork_selector_v1(request)?;

        validate_resume_fork_mutual_exclusion(&request.extensions)?;

        if fork.is_some() && accepted_model_override_v1(request)? {
            return Err(AgentWrapperError::Backend {
                message: super::PINNED_MODEL_OVERRIDE_UNSUPPORTED_FOR_FORK.to_string(),
            });
        }

        if fork.is_some() && !exec_policy.add_dirs.is_empty() {
            return Err(AgentWrapperError::Backend {
                message: PINNED_ADD_DIRS_UNSUPPORTED_FOR_FORK.to_string(),
            });
        }

        Ok(super::CodexExecPolicy {
            resume,
            fork,
            ..exec_policy
        })
    }

    type BackendEvent = CodexBackendEvent;
    type BackendCompletion = CodexBackendCompletion;
    type BackendError = CodexBackendError;

    fn attempt_custody(
        &self,
        req: &NormalizedRequest<Self::Policy>,
    ) -> Option<crate::backend_harness::BackendAttemptCustody> {
        if req.policy.fork.is_some() {
            let cancel = self.mcp_custody.clone();
            let settle = self.mcp_custody.clone();
            let observed = self.mcp_custody.clone();
            let admission_observed = self.mcp_custody.clone();
            return Some(crate::backend_harness::BackendAttemptCustody {
                observe_closed_before_any_spawn: Some(Arc::new(move || {
                    admission_observed.observed_closed_before_any_spawn()
                })),
                request_termination: Arc::new(move || cancel.request_termination()),
                observe_processes: Some(Arc::new(move || {
                    observed
                        .observed_process_terminations()
                        .into_iter()
                        .map(|value| crate::AgentWrapperProcessTerminationObservation {
                            process_id: value.process_id,
                            status: value.status,
                            recovery_status: value.recovery_status,
                        })
                        .collect()
                })),
                settle: Arc::new(move || {
                    let settle = settle.clone();
                    Box::pin(async move {
                        settle
                            .settle()
                            .await
                            .map_err(|_| AgentWrapperError::Backend {
                                message: "codex MCP process settlement failed".to_string(),
                            })
                    })
                }),
            });
        }
        let cancel = self.custody.clone();
        let settle = self.custody.clone();
        let observed = self.custody.clone();
        let admission_observed = self.custody.clone();
        Some(crate::backend_harness::BackendAttemptCustody {
            observe_closed_before_any_spawn: Some(Arc::new(move || {
                admission_observed.observed_closed_before_any_spawn()
            })),
            request_termination: Arc::new(move || cancel.request_termination()),
            observe_processes: Some(Arc::new(move || {
                observed
                    .observed_process_terminations()
                    .into_iter()
                    .map(|value| crate::AgentWrapperProcessTerminationObservation {
                        process_id: value.process_id,
                        status: value.status,
                        recovery_status: value.recovery_status,
                    })
                    .collect()
            })),
            settle: Arc::new(move || {
                let settle = settle.clone();
                Box::pin(async move {
                    settle
                        .settle()
                        .await
                        .map_err(|_| AgentWrapperError::Backend {
                            message: "codex process settlement failed".to_string(),
                        })
                })
            }),
        })
    }

    fn spawn(
        &self,
        req: NormalizedRequest<Self::Policy>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        BackendSpawn<
                            Self::BackendEvent,
                            Self::BackendCompletion,
                            Self::BackendError,
                        >,
                        Self::BackendError,
                    >,
                > + Send
                + 'static,
        >,
    > {
        let config = self.config.clone();
        let custody = self.custody.clone();
        let mcp_custody = self.mcp_custody.clone();
        let run_start_cwd = self.run_start_cwd.clone();
        let termination = self.termination.clone();
        let handle_state = Arc::clone(&self.handle_state);
        let super::CodexExecPolicy {
            add_dirs,
            non_interactive,
            external_sandbox,
            approval_policy,
            sandbox_mode,
            resume,
            fork,
        } = req.policy;
        let prompt = req.prompt;
        let model_id = req.model_id;
        let working_dir = req.working_dir;
        let effective_timeout = req.effective_timeout;
        let env = req.env;

        Box::pin(async move {
            let spawned = match if let Some(selector) = fork {
                super::fork::spawn_fork_v1_flow(super::fork::ForkFlowRequest {
                    custody: mcp_custody,
                    selector,
                    prompt,
                    working_dir,
                    effective_timeout,
                    env,
                    config,
                    run_start_cwd,
                    termination,
                    non_interactive,
                    external_sandbox,
                    approval_policy,
                    sandbox_mode,
                    handle_state,
                })
                .await
            } else {
                super::exec::spawn_exec_or_resume_flow(super::exec::ExecFlowRequest {
                    custody,
                    config,
                    run_start_cwd,
                    termination,
                    add_dirs,
                    non_interactive,
                    external_sandbox,
                    approval_policy,
                    sandbox_mode,
                    resume,
                    prompt,
                    model_id,
                    working_dir,
                    effective_timeout,
                    env,
                })
                .await
            } {
                Ok(spawned) => spawned,
                Err(err)
                    if external_sandbox
                        && !matches!(err, CodexBackendError::AddDirsRejectedByRuntime) =>
                {
                    return Ok(startup_failure_spawn(err, true));
                }
                Err(err) => return Err(err),
            };

            let BackendSpawn {
                events,
                completion,
                events_observability,
            } = spawned;
            let events = if external_sandbox {
                Box::pin(
                    stream::once(async move { Ok(CodexBackendEvent::ExternalSandboxWarning) })
                        .chain(events),
                )
            } else {
                events
            };

            Ok(BackendSpawn {
                events,
                completion,
                events_observability,
            })
        })
    }
    fn map_event(&self, event: Self::BackendEvent) -> Vec<AgentWrapperEvent> {
        match event {
            CodexBackendEvent::ExternalSandboxWarning => {
                vec![status_event(Some(
                    PINNED_EXTERNAL_SANDBOX_WARNING.to_string(),
                ))]
            }
            CodexBackendEvent::Thread(ev) => {
                let mut emit_oversize_warning_len: Option<usize> = None;

                if let Ok(mut state) = self.handle_state.lock() {
                    if state.thread_id.is_none() {
                        if let Some(thread_id) = ev.thread_id() {
                            if !thread_id.trim().is_empty() {
                                let len = thread_id.len();
                                if len <= SESSION_HANDLE_ID_BOUND_BYTES {
                                    state.thread_id = Some(thread_id.to_string());
                                } else if !state.oversize_warning_emitted {
                                    state.oversize_warning_emitted = true;
                                    emit_oversize_warning_len = Some(len);
                                }
                            }
                        }
                    }
                }

                let mut mapped = vec![map_thread_event(&ev)];

                let emit_handle_facet: Option<String> =
                    self.handle_state.lock().ok().and_then(|mut state| {
                        if state.handle_facet_emitted {
                            return None;
                        }
                        let thread_id = state.thread_id.clone()?;
                        state.handle_facet_emitted = true;
                        Some(thread_id)
                    });

                if let Some(thread_id) = emit_handle_facet.as_deref() {
                    let mut attached = false;
                    for event in &mut mapped {
                        if event.kind == AgentWrapperEventKind::Status && event.data.is_none() {
                            event.data = Some(session_handle_facet(thread_id));
                            attached = true;
                            break;
                        }
                    }

                    if !attached {
                        let mut event = status_event(None);
                        event.data = Some(session_handle_facet(thread_id));
                        mapped.push(event);
                    }
                }

                if let Some(len) = emit_oversize_warning_len {
                    mapped.push(status_event(Some(format!(
                        "{SESSION_HANDLE_OVERSIZE_WARNING_MARKER}: len_bytes={len}"
                    ))));
                }

                mapped
            }
            CodexBackendEvent::AppServerNotification(notification) => {
                let mut mapped = match notification {
                    codex::mcp::AppNotification::Raw { method, params } => {
                        super::fork::map_app_server_notification(&method, &params)
                            .into_iter()
                            .collect()
                    }
                    codex::mcp::AppNotification::Error { message, .. } => {
                        vec![error_event(message)]
                    }
                    _ => Vec::new(),
                };

                let emit_handle_facet: Option<String> =
                    self.handle_state.lock().ok().and_then(|mut state| {
                        if state.handle_facet_emitted {
                            return None;
                        }
                        let thread_id = state.thread_id.clone()?;
                        state.handle_facet_emitted = true;
                        Some(thread_id)
                    });

                if let Some(thread_id) = emit_handle_facet.as_deref() {
                    let mut attached = false;
                    for event in &mut mapped {
                        if event.kind == AgentWrapperEventKind::Status && event.data.is_none() {
                            event.data = Some(session_handle_facet(thread_id));
                            attached = true;
                            break;
                        }
                    }

                    if !attached {
                        let mut event = status_event(None);
                        event.data = Some(session_handle_facet(thread_id));
                        mapped.push(event);
                    }
                }

                let emit_oversize_warning_len: Option<usize> = self
                    .handle_state
                    .lock()
                    .ok()
                    .and_then(|mut state| state.oversize_warning_len_bytes.take());

                if let Some(len) = emit_oversize_warning_len {
                    mapped.push(status_event(Some(format!(
                        "{SESSION_HANDLE_OVERSIZE_WARNING_MARKER}: len_bytes={len}"
                    ))));
                }

                mapped
            }
            CodexBackendEvent::NonZeroExit { status } => {
                let mut mapped = Vec::new();
                if let Some(len) = self
                    .handle_state
                    .lock()
                    .ok()
                    .and_then(|mut state| state.oversize_warning_len_bytes.take())
                {
                    mapped.push(status_event(Some(format!(
                        "{SESSION_HANDLE_OVERSIZE_WARNING_MARKER}: len_bytes={len}"
                    ))));
                }
                mapped.push(error_event(format!(
                    "codex exited non-zero: {status:?} (stderr redacted)"
                )));
                mapped
            }
            CodexBackendEvent::TerminalError { message } => {
                let mut mapped = Vec::new();
                if let Some(len) = self
                    .handle_state
                    .lock()
                    .ok()
                    .and_then(|mut state| state.oversize_warning_len_bytes.take())
                {
                    mapped.push(status_event(Some(format!(
                        "{SESSION_HANDLE_OVERSIZE_WARNING_MARKER}: len_bytes={len}"
                    ))));
                }
                mapped.push(error_event(message));
                mapped
            }
        }
    }

    fn map_completion(
        &self,
        completion: Self::BackendCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        let CodexBackendCompletion {
            status,
            final_text,
            backend_error_message,
            selection_failure_message,
        } = completion;

        if let Some(message) = backend_error_message {
            return Err(AgentWrapperError::Backend { message });
        }

        if let Some(message) = selection_failure_message {
            return Err(AgentWrapperError::Backend { message });
        }

        let handle_facet = self
            .handle_state
            .lock()
            .ok()
            .and_then(|state| state.thread_id.clone())
            .map(|thread_id| session_handle_facet(&thread_id));

        Ok(AgentWrapperCompletion {
            status,
            final_text: crate::bounds::enforce_final_text_bound(final_text),
            data: handle_facet,
        })
    }

    fn redact_error(&self, _phase: BackendHarnessErrorPhase, err: &Self::BackendError) -> String {
        render_backend_error_message(err)
    }
}
