use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures_util::{stream, StreamExt};
use tokio::sync::{oneshot, OnceCell};

use super::{
    mapping::{
        error_event, extract_assistant_message_final_text, map_stream_json_event,
        session_handle_facet, status_event,
    },
    util::{
        build_fresh_run_print_request, json_contains_add_dirs_runtime_rejection_signal,
        json_contains_model_runtime_rejection_signal, json_contains_not_found_signal, parse_bool,
        preflight_allow_flag_support, render_backend_error_message,
        resolve_claude_effective_working_dir, resolve_completion_messages, startup_failure_spawn,
        ADD_DIRS_RUNTIME_REJECTION_MESSAGE, PINNED_MODEL_RUNTIME_REJECTION,
    },
    ClaudeCodeBackendConfig, AGENT_KIND, CLAUDE_EXEC_POLICY_PREFIX, EXT_ADD_DIRS_V1,
    EXT_EXTERNAL_SANDBOX_V1, EXT_NON_INTERACTIVE, PINNED_EXTERNAL_SANDBOX_WARNING,
    SESSION_HANDLE_ID_BOUND_BYTES, SESSION_HANDLE_OVERSIZE_WARNING,
    SUPPORTED_EXTENSION_KEYS_DEFAULT, SUPPORTED_EXTENSION_KEYS_EXTERNAL_SANDBOX_OPT_IN,
};
use crate::{
    backend_harness::{
        normalize_add_dirs_v1, BackendHarnessAdapter, BackendHarnessErrorPhase, BackendSpawn,
        DynBackendEventStream, NormalizedRequest,
    },
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperKind, AgentWrapperRunRequest,
};

use super::super::session_selectors::{
    parse_session_fork_v1, parse_session_resume_v1, validate_resume_fork_mutual_exclusion,
    SessionSelectorV1, EXT_SESSION_FORK_V1, EXT_SESSION_RESUME_V1,
};

#[derive(Clone, Debug)]
pub(super) struct ClaudeHarnessAdapter {
    custody: claude_code::ClaudeProcessCustody,
    config: ClaudeCodeBackendConfig,
    run_start_cwd: Option<PathBuf>,
    termination: Option<
        Arc<super::super::termination::TerminationState<claude_code::ClaudeTerminationHandle>>,
    >,
    handle_state: Arc<Mutex<ClaudeHandleFacetState>>,
    allow_flag_preflight: Arc<OnceCell<bool>>,
}

#[derive(Clone, Debug)]
pub(super) struct ClaudeExecPolicy {
    pub(super) non_interactive: bool,
    pub(super) external_sandbox: bool,
    pub(super) resume: Option<SessionSelectorV1>,
    pub(super) fork: Option<SessionSelectorV1>,
    pub(super) resolved_working_dir: Option<PathBuf>,
    pub(super) add_dirs: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub(super) struct ClaudeBackendCompletion {
    pub(super) status: std::process::ExitStatus,
    pub(super) final_text: Option<String>,
    pub(super) backend_error_message: Option<String>,
}

#[derive(Debug, Default)]
struct ClaudeStreamState {
    last_assistant_text: Option<String>,
    saw_assistant_message: bool,
    saw_stream_error: bool,
    saw_not_found_signal: bool,
    backend_error_message: Option<String>,
}

#[derive(Debug)]
pub(super) enum ClaudeBackendEvent {
    ExternalSandboxWarning,
    Stream(claude_code::ClaudeStreamJsonEvent),
    TerminalError { message: String },
}

#[derive(Debug, Default)]
struct ClaudeHandleFacetState {
    session_id: Option<String>,
    handle_facet_emitted: bool,
    oversize_warning_emitted: bool,
}

#[derive(Debug)]
pub(super) enum ClaudeBackendError {
    Spawn(claude_code::ClaudeCodeError),
    StreamParse(claude_code::ClaudeStreamJsonParseError),
    Completion(claude_code::ClaudeCodeError),
    ExternalSandboxPreflight { message: String },
}

pub(super) fn new_harness_adapter(
    config: ClaudeCodeBackendConfig,
    run_start_cwd: Option<PathBuf>,
    termination: Option<
        Arc<super::super::termination::TerminationState<claude_code::ClaudeTerminationHandle>>,
    >,
    allow_flag_preflight: Arc<OnceCell<bool>>,
) -> ClaudeHarnessAdapter {
    ClaudeHarnessAdapter {
        custody: claude_code::ClaudeProcessCustody::new(),
        config,
        run_start_cwd,
        termination,
        handle_state: Arc::new(Mutex::new(ClaudeHandleFacetState::default())),
        allow_flag_preflight,
    }
}

#[cfg(test)]
pub(super) fn new_test_adapter(config: ClaudeCodeBackendConfig) -> ClaudeHarnessAdapter {
    new_harness_adapter(config, None, None, Arc::new(OnceCell::new()))
}

#[cfg(test)]
pub(super) fn new_test_adapter_with_run_start_cwd(
    config: ClaudeCodeBackendConfig,
    run_start_cwd: Option<PathBuf>,
) -> ClaudeHarnessAdapter {
    new_harness_adapter(config, run_start_cwd, None, Arc::new(OnceCell::new()))
}

impl BackendHarnessAdapter for ClaudeHarnessAdapter {
    fn kind(&self) -> AgentWrapperKind {
        AgentWrapperKind(AGENT_KIND.to_string())
    }

    fn supported_extension_keys(&self) -> &'static [&'static str] {
        if self.config.allow_external_sandbox_exec {
            SUPPORTED_EXTENSION_KEYS_EXTERNAL_SANDBOX_OPT_IN
        } else {
            SUPPORTED_EXTENSION_KEYS_DEFAULT
        }
    }

    type Policy = ClaudeExecPolicy;

    fn validate_and_extract_policy(
        &self,
        request: &AgentWrapperRunRequest,
    ) -> Result<Self::Policy, AgentWrapperError> {
        let non_interactive_requested: Option<bool> = request
            .extensions
            .get(EXT_NON_INTERACTIVE)
            .map(|value| parse_bool(value, EXT_NON_INTERACTIVE))
            .transpose()?;
        let non_interactive = non_interactive_requested.unwrap_or(true);

        let external_sandbox = request
            .extensions
            .get(EXT_EXTERNAL_SANDBOX_V1)
            .map(|value| parse_bool(value, EXT_EXTERNAL_SANDBOX_V1))
            .transpose()?
            .unwrap_or(false);

        if external_sandbox {
            if non_interactive_requested == Some(false) {
                return Err(AgentWrapperError::InvalidRequest {
                    message: format!(
                        "{EXT_EXTERNAL_SANDBOX_V1}=true must not be combined with {EXT_NON_INTERACTIVE}=false"
                    ),
                });
            }

            if request
                .extensions
                .keys()
                .any(|key| key.starts_with(CLAUDE_EXEC_POLICY_PREFIX))
            {
                return Err(AgentWrapperError::InvalidRequest {
                    message: format!(
                        "{EXT_EXTERNAL_SANDBOX_V1}=true must not be combined with backend.claude_code.exec.* keys"
                    ),
                });
            }
        }

        let resume = request
            .extensions
            .get(EXT_SESSION_RESUME_V1)
            .map(parse_session_resume_v1)
            .transpose()?;

        let fork = request
            .extensions
            .get(EXT_SESSION_FORK_V1)
            .map(parse_session_fork_v1)
            .transpose()?;

        let resolved_working_dir = resolve_claude_effective_working_dir(
            &self.config,
            self.run_start_cwd.as_deref(),
            request.working_dir.as_deref(),
        )?;

        validate_resume_fork_mutual_exclusion(&request.extensions)?;

        let add_dirs = match request.extensions.get(EXT_ADD_DIRS_V1) {
            Some(raw) => normalize_add_dirs_v1(Some(raw), resolved_working_dir.as_deref())?,
            None => Vec::new(),
        };

        Ok(ClaudeExecPolicy {
            non_interactive,
            external_sandbox,
            resume,
            fork,
            resolved_working_dir,
            add_dirs,
        })
    }

    type BackendEvent = ClaudeBackendEvent;
    type BackendCompletion = ClaudeBackendCompletion;
    type BackendError = ClaudeBackendError;

    fn attempt_custody(
        &self,
        _req: &NormalizedRequest<Self::Policy>,
    ) -> Option<crate::backend_harness::BackendAttemptCustody> {
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
                            message: "claude process settlement failed".to_string(),
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
        let termination = self.termination.clone();
        let allow_flag_preflight = Arc::clone(&self.allow_flag_preflight);
        let NormalizedRequest {
            prompt,
            model_id,
            working_dir: _raw_working_dir,
            effective_timeout,
            env,
            policy,
            ..
        } = req;
        let ClaudeExecPolicy {
            non_interactive,
            external_sandbox,
            resume,
            fork,
            resolved_working_dir,
            add_dirs,
        } = policy;
        Box::pin(async move {
            let mut builder = claude_code::ClaudeClient::builder();
            if let Some(binary) = config.binary.as_ref() {
                builder = builder.binary(binary.clone());
            }
            if let Some(claude_home) = config.claude_home.as_ref() {
                builder = builder.claude_home(claude_home.clone());
            }

            if let Some(dir) = resolved_working_dir {
                builder = builder.working_dir(dir);
            }

            let timeout = match effective_timeout {
                Some(t) if t == Duration::ZERO => None,
                other => other,
            };
            builder = builder.timeout(timeout);

            for (k, v) in &env {
                builder = builder.env(k.clone(), v.clone());
            }

            let client = builder.build();

            let mut allow_dangerously_skip_permissions = false;
            if external_sandbox {
                allow_dangerously_skip_permissions =
                    match preflight_allow_flag_support(allow_flag_preflight.as_ref(), || {
                        client.help_with_custody(&custody)
                    })
                    .await
                    {
                        Ok(supported) => supported,
                        Err(message) => {
                            return Ok(startup_failure_spawn(
                                ClaudeBackendError::ExternalSandboxPreflight { message },
                                true,
                            ));
                        }
                    };
            }

            let mut print_req = build_fresh_run_print_request(
                prompt,
                non_interactive,
                external_sandbox,
                allow_dangerously_skip_permissions,
                &add_dirs,
            );

            let has_model_id = model_id.is_some();
            if let Some(model_id) = model_id {
                print_req = print_req.model(model_id);
            }

            if let Some(resume) = resume.as_ref() {
                match resume {
                    SessionSelectorV1::Last => {
                        print_req = print_req.continue_session(true);
                    }
                    SessionSelectorV1::Id { id } => {
                        print_req = print_req.resume_value(id.clone());
                    }
                }
            }

            if let Some(fork) = fork.as_ref() {
                print_req = print_req.fork_session(true);
                match fork {
                    SessionSelectorV1::Last => {
                        print_req = print_req.continue_session(true);
                    }
                    SessionSelectorV1::Id { id } => {
                        print_req = print_req.resume_value(id.clone());
                    }
                }
            }

            let handle = match client
                .print_stream_json_control_with_custody(print_req, &custody)
                .await
            {
                Ok(handle) => handle,
                Err(err) if external_sandbox => {
                    return Ok(startup_failure_spawn(ClaudeBackendError::Spawn(err), true));
                }
                Err(err) => return Err(ClaudeBackendError::Spawn(err)),
            };

            if let Some(state) = termination.as_ref() {
                state.set_handle(handle.termination.clone());
            }

            let selection_selector = resume.clone().or(fork.clone());
            let has_add_dirs = !add_dirs.is_empty();
            let stream_state: Arc<Mutex<ClaudeStreamState>> =
                Arc::new(Mutex::new(ClaudeStreamState::default()));
            let (events_done_tx, events_done_rx) = oneshot::channel::<()>();

            let monitor_backend_error_tail =
                selection_selector.is_some() || has_add_dirs || has_model_id;
            let (tail_tx, tail_rx) = if monitor_backend_error_tail {
                let (tx, rx) = oneshot::channel::<Option<String>>();
                (Some(tx), Some(rx))
            } else {
                (None, None)
            };

            let events: DynBackendEventStream<ClaudeBackendEvent, ClaudeBackendError> =
                Box::pin(stream::unfold(
                    (
                        handle.events,
                        Arc::clone(&stream_state),
                        Some(events_done_tx),
                        tail_rx,
                        selection_selector.clone(),
                        has_add_dirs,
                        has_model_id,
                        false,
                    ),
                    |(
                        mut events,
                        stream_state,
                        mut events_done_tx,
                        mut tail_rx,
                        selection_selector,
                        has_add_dirs,
                        has_model_id,
                        tail_emitted,
                    )| async move {
                        loop {
                            match events.next().await {
                                Some(Ok(ev)) => {
                                    if let claude_code::ClaudeStreamJsonEvent::AssistantMessage {
                                        raw,
                                        ..
                                    } = &ev
                                    {
                                        if let Ok(mut state) = stream_state.lock() {
                                            state.saw_assistant_message = true;
                                            if let Some(text) =
                                                extract_assistant_message_final_text(raw)
                                            {
                                                state.last_assistant_text = Some(text);
                                            }
                                        }
                                    }

                                    if has_add_dirs
                                        && matches!(
                                            ev,
                                            claude_code::ClaudeStreamJsonEvent::ResultError { .. }
                                        )
                                    {
                                        if let claude_code::ClaudeStreamJsonEvent::ResultError {
                                            raw,
                                            ..
                                        } = &ev
                                        {
                                            if json_contains_add_dirs_runtime_rejection_signal(raw)
                                            {
                                                if let Ok(mut state) = stream_state.lock() {
                                                    state.backend_error_message = Some(
                                                        ADD_DIRS_RUNTIME_REJECTION_MESSAGE
                                                            .to_string(),
                                                    );
                                                }
                                                continue;
                                            }
                                        }
                                    }

                                    if has_model_id
                                        && matches!(
                                            ev,
                                            claude_code::ClaudeStreamJsonEvent::ResultError { .. }
                                        )
                                    {
                                        if let claude_code::ClaudeStreamJsonEvent::ResultError {
                                            raw,
                                            ..
                                        } = &ev
                                        {
                                            if json_contains_model_runtime_rejection_signal(raw) {
                                                if let Ok(mut state) = stream_state.lock() {
                                                    state.backend_error_message = Some(
                                                        PINNED_MODEL_RUNTIME_REJECTION.to_string(),
                                                    );
                                                }
                                                continue;
                                            }
                                        }
                                    }

                                    if selection_selector.is_some()
                                        && matches!(
                                            ev,
                                            claude_code::ClaudeStreamJsonEvent::ResultError { .. }
                                        )
                                    {
                                        if let claude_code::ClaudeStreamJsonEvent::ResultError {
                                            raw,
                                            ..
                                        } = &ev
                                        {
                                            if json_contains_not_found_signal(raw) {
                                                if let Ok(mut state) = stream_state.lock() {
                                                    state.saw_not_found_signal = true;
                                                }
                                                continue;
                                            }
                                        }
                                    }

                                    if selection_selector.is_some()
                                        && matches!(
                                            ev,
                                            claude_code::ClaudeStreamJsonEvent::ResultError { .. }
                                        )
                                    {
                                        continue;
                                    }

                                    return Some((
                                        Ok(ClaudeBackendEvent::Stream(ev)),
                                        (
                                            events,
                                            stream_state,
                                            events_done_tx,
                                            tail_rx,
                                            selection_selector,
                                            has_add_dirs,
                                            has_model_id,
                                            tail_emitted,
                                        ),
                                    ));
                                }
                                Some(Err(err)) => {
                                    if let Ok(mut state) = stream_state.lock() {
                                        state.saw_stream_error = true;
                                    }

                                    return Some((
                                        Err(ClaudeBackendError::StreamParse(err)),
                                        (
                                            events,
                                            stream_state,
                                            events_done_tx,
                                            tail_rx,
                                            selection_selector,
                                            has_add_dirs,
                                            has_model_id,
                                            tail_emitted,
                                        ),
                                    ));
                                }
                                None => {
                                    if let Some(tx) = events_done_tx.take() {
                                        let _ = tx.send(());
                                    }

                                    if tail_emitted {
                                        return None;
                                    }

                                    let rx = tail_rx.take()?;
                                    let message = rx.await.ok().flatten()?;

                                    return Some((
                                        Ok(ClaudeBackendEvent::TerminalError { message }),
                                        (
                                            events,
                                            stream_state,
                                            events_done_tx,
                                            tail_rx,
                                            selection_selector,
                                            has_add_dirs,
                                            has_model_id,
                                            true,
                                        ),
                                    ));
                                }
                            }
                        }
                    },
                ));

            let events = if external_sandbox {
                Box::pin(
                    stream::once(async move { Ok(ClaudeBackendEvent::ExternalSandboxWarning) })
                        .chain(events),
                )
            } else {
                events
            };

            let completion = Box::pin(async move {
                let _ = events_done_rx.await;

                let status = handle
                    .completion
                    .await
                    .map_err(ClaudeBackendError::Completion)?;

                let (
                    final_text,
                    saw_stream_error,
                    saw_not_found_signal,
                    runtime_backend_error_message,
                ) = stream_state
                    .lock()
                    .map(|guard| {
                        (
                            guard.last_assistant_text.clone(),
                            guard.saw_stream_error,
                            guard.saw_not_found_signal,
                            guard.backend_error_message.clone(),
                        )
                    })
                    .unwrap_or((None, true, false, None));

                let (backend_error_message, terminal_error_event_message) =
                    resolve_completion_messages(
                        &status,
                        selection_selector.as_ref(),
                        saw_stream_error,
                        saw_not_found_signal,
                        runtime_backend_error_message,
                    );

                if let Some(tx) = tail_tx {
                    let _ = tx.send(terminal_error_event_message.clone());
                }

                Ok(ClaudeBackendCompletion {
                    status,
                    final_text,
                    backend_error_message,
                })
            });

            Ok(BackendSpawn {
                events,
                completion,
                events_observability: None,
            })
        })
    }

    fn map_event(&self, event: Self::BackendEvent) -> Vec<AgentWrapperEvent> {
        let event = match event {
            ClaudeBackendEvent::ExternalSandboxWarning => {
                return vec![status_event(Some(
                    PINNED_EXTERNAL_SANDBOX_WARNING.to_string(),
                ))];
            }
            ClaudeBackendEvent::Stream(event) => event,
            ClaudeBackendEvent::TerminalError { message } => {
                return vec![error_event(message)];
            }
        };

        let mut emit_oversize_warning = false;

        if let Ok(mut state) = self.handle_state.lock() {
            if state.session_id.is_none() {
                if let Some(session_id) = event.session_id() {
                    let session_id = session_id.trim();
                    if !session_id.is_empty() {
                        if session_id.len() <= SESSION_HANDLE_ID_BOUND_BYTES {
                            state.session_id = Some(session_id.to_string());
                        } else if !state.oversize_warning_emitted {
                            state.oversize_warning_emitted = true;
                            emit_oversize_warning = true;
                        }
                    }
                }
            }
        }

        let mut mapped = map_stream_json_event(event);

        let emit_handle_facet: Option<String> =
            self.handle_state.lock().ok().and_then(|mut state| {
                if state.handle_facet_emitted {
                    return None;
                }
                let session_id = state.session_id.clone()?;
                state.handle_facet_emitted = true;
                Some(session_id)
            });

        if let Some(session_id) = emit_handle_facet.as_deref() {
            let mut attached = false;
            for event in &mut mapped {
                if event.kind == AgentWrapperEventKind::Status && event.data.is_none() {
                    event.data = Some(session_handle_facet(session_id));
                    attached = true;
                    break;
                }
            }

            if !attached {
                let mut event = status_event(None);
                event.data = Some(session_handle_facet(session_id));
                mapped.push(event);
            }
        }

        if emit_oversize_warning {
            mapped.push(status_event(Some(
                SESSION_HANDLE_OVERSIZE_WARNING.to_string(),
            )));
        }

        mapped
    }

    fn map_completion(
        &self,
        completion: Self::BackendCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        if let Some(message) = completion.backend_error_message {
            return Err(AgentWrapperError::Backend { message });
        }

        let handle_facet = self
            .handle_state
            .lock()
            .ok()
            .and_then(|state| state.session_id.clone())
            .map(|session_id| session_handle_facet(&session_id));

        Ok(AgentWrapperCompletion {
            status: completion.status,
            final_text: crate::bounds::enforce_final_text_bound(completion.final_text),
            data: handle_facet,
        })
    }

    fn redact_error(&self, phase: BackendHarnessErrorPhase, err: &Self::BackendError) -> String {
        match (phase, err) {
            (BackendHarnessErrorPhase::Stream, ClaudeBackendError::StreamParse(err)) => {
                err.message.clone()
            }
            _ => render_backend_error_message(err),
        }
    }
}
