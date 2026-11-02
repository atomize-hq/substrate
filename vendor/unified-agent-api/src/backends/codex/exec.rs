use std::{
    collections::BTreeMap,
    path::PathBuf,
    pin::Pin,
    process::ExitStatus,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};

use codex::{CodexError, ExecStreamError, ExecStreamRequest, ThreadEvent};
use futures_core::Stream;
use futures_util::StreamExt;
use tokio::sync::{mpsc, oneshot};

use crate::{
    backend_harness::{BackendSpawn, EventObservabilitySignal, DEFAULT_EVENT_CHANNEL_CAPACITY},
    backends::spawn_path::resolve_effective_working_dir,
};

pub(super) struct ExecFlowRequest {
    pub(super) custody: codex::ExecProcessCustody,
    pub(super) config: super::CodexBackendConfig,
    pub(super) run_start_cwd: Option<PathBuf>,
    pub(super) termination:
        Option<Arc<super::super::termination::TerminationState<super::CodexTerminationHandle>>>,
    pub(super) add_dirs: Vec<PathBuf>,
    pub(super) non_interactive: bool,
    pub(super) external_sandbox: bool,
    pub(super) approval_policy: Option<super::CodexApprovalPolicy>,
    pub(super) sandbox_mode: super::CodexSandboxMode,
    pub(super) resume: Option<super::SessionSelectorV1>,
    pub(super) prompt: String,
    pub(super) model_id: Option<String>,
    pub(super) working_dir: Option<PathBuf>,
    pub(super) effective_timeout: Option<Duration>,
    pub(super) env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default)]
struct CodexStreamState {
    saw_thread_id: bool,
    saw_stream_error: bool,
    backend_error_message: Option<String>,
    last_transport_error_code: Option<String>,
    last_transport_error_message: Option<String>,
}

#[derive(Debug)]
enum CodexTailEvent {
    NonZeroExit { status: ExitStatus },
    TerminalError { message: String },
}

fn snapshot_backend_error_message(stream_state: &Arc<Mutex<CodexStreamState>>) -> Option<String> {
    stream_state
        .lock()
        .ok()
        .and_then(|snapshot| snapshot.backend_error_message.clone())
}

async fn forward_backend_event(
    event_tx: &mpsc::Sender<Result<super::CodexBackendEvent, super::CodexBackendError>>,
    forward: &mut bool,
    event: Result<super::CodexBackendEvent, super::CodexBackendError>,
) {
    if !*forward {
        return;
    }

    if event_tx.send(event).await.is_err() {
        *forward = false;
    }
}

async fn wait_for_event_processing(
    events_done_rx: &mut Option<oneshot::Receiver<()>>,
    should_wait: bool,
) {
    if !should_wait {
        return;
    }

    let Some(rx) = events_done_rx.take() else {
        return;
    };

    // Consumer observability can end before the event task drains a buffered tail of suppressed
    // terminal errors. Only `events_done_rx` proves the drainer finished classification.
    let _ = rx.await;
}

struct ObservabilityEventStream {
    rx: mpsc::Receiver<Result<super::CodexBackendEvent, super::CodexBackendError>>,
    events_observability: Option<EventObservabilitySignal>,
}

impl ObservabilityEventStream {
    fn signal_observability(&self) {
        if let Some(signal) = self.events_observability.as_ref() {
            signal.signal();
        }
    }
}

impl Stream for ObservabilityEventStream {
    type Item = Result<super::CodexBackendEvent, super::CodexBackendError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = Pin::new(&mut self.rx).poll_recv(cx);
        if matches!(poll, Poll::Ready(None)) {
            self.signal_observability();
        }
        poll
    }
}

impl Drop for ObservabilityEventStream {
    fn drop(&mut self) {
        self.signal_observability();
    }
}

pub(super) fn is_model_runtime_rejection_signal(code: Option<&str>) -> bool {
    code == Some("model_runtime_rejection")
}

fn is_unknown_bypass_flag_stderr(stderr: &str) -> bool {
    let stderr = stderr.to_ascii_lowercase();
    if !stderr.contains("dangerously-bypass-approvals-and-sandbox") {
        return false;
    }

    const UNKNOWN_SIGNALS: &[&str] = &[
        "unknown",
        "unrecognized",
        "unexpected",
        "unknown option",
        "unknown flag",
        "unrecognized option",
        "unrecognized flag",
        "unexpected argument",
        "found argument",
        "invalid option",
        "invalid flag",
    ];

    UNKNOWN_SIGNALS.iter().any(|signal| stderr.contains(signal))
}

fn map_approval_policy(policy: &super::CodexApprovalPolicy) -> codex::ApprovalPolicy {
    match policy {
        super::CodexApprovalPolicy::Untrusted => codex::ApprovalPolicy::Untrusted,
        super::CodexApprovalPolicy::OnFailure => codex::ApprovalPolicy::OnFailure,
        super::CodexApprovalPolicy::OnRequest => codex::ApprovalPolicy::OnRequest,
        super::CodexApprovalPolicy::Never => codex::ApprovalPolicy::Never,
    }
}

fn map_sandbox_mode(mode: &super::CodexSandboxMode) -> codex::SandboxMode {
    match mode {
        super::CodexSandboxMode::ReadOnly => codex::SandboxMode::ReadOnly,
        super::CodexSandboxMode::WorkspaceWrite => codex::SandboxMode::WorkspaceWrite,
        super::CodexSandboxMode::DangerFullAccess => codex::SandboxMode::DangerFullAccess,
    }
}

pub(super) async fn spawn_exec_or_resume_flow(
    req: ExecFlowRequest,
) -> Result<
    BackendSpawn<super::CodexBackendEvent, super::CodexBackendCompletion, super::CodexBackendError>,
    super::CodexBackendError,
> {
    let ExecFlowRequest {
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
    } = req;

    let effective_model_id = model_id.clone().or_else(|| config.model.clone());

    let mut builder = codex::CodexClient::builder()
        .json(true)
        .mirror_stdout(false)
        .quiet(true)
        .color_mode(codex::ColorMode::Never)
        .sandbox_mode(map_sandbox_mode(&sandbox_mode));

    if external_sandbox {
        builder = builder.dangerously_bypass_approvals_and_sandbox(true);
    }

    if non_interactive {
        builder = builder.approval_policy(codex::ApprovalPolicy::Never);
    } else if let Some(value) = approval_policy.as_ref() {
        builder = builder.approval_policy(map_approval_policy(value));
    }

    if let Some(binary) = config.binary.as_ref() {
        builder = builder.binary(binary.clone());
    }

    if let Some(codex_home) = config.codex_home.as_ref() {
        builder = builder.codex_home(codex_home.clone());
    }

    if let Some(model) = effective_model_id.as_ref() {
        builder = builder.model(model.clone());
    }

    let working_dir = resolve_effective_working_dir(
        working_dir.as_deref(),
        config.default_working_dir.as_deref(),
        run_start_cwd.as_deref(),
    )
    .ok_or(super::CodexBackendError::WorkingDirectoryUnresolved)?;
    builder = builder.working_dir(working_dir);

    let has_add_dirs = !add_dirs.is_empty();
    if has_add_dirs {
        let probe_client = builder.clone().build();
        let capabilities = probe_client
            .probe_capabilities_with_env_overrides_custody(&env, &custody)
            .await;
        if !capabilities.guard_add_dir().is_supported() {
            return Err(super::CodexBackendError::AddDirsRejectedByRuntime);
        }
        builder = builder.capability_feature_hints(codex::CodexFeatureFlags {
            supports_add_dir: true,
            ..Default::default()
        });
    }

    // Codex crate treats `Duration::ZERO` as “no timeout”.
    builder = builder.timeout(effective_timeout.unwrap_or(Duration::ZERO));
    builder = builder.add_dirs(add_dirs);

    let client = builder.build();

    let resume_selector = resume.clone();
    let suppress_transport_errors = resume_selector.is_some();
    let stream_state = Arc::new(Mutex::new(CodexStreamState::default()));

    let handle = match resume {
        None => {
            client
                .stream_exec_with_env_overrides_custody(
                    ExecStreamRequest {
                        prompt,
                        ephemeral: false,
                        ignore_rules: false,
                        ignore_user_config: false,
                        idle_timeout: None,
                        output_last_message: None,
                        output_schema: None,
                        json_event_log: None,
                    },
                    &env,
                    &custody,
                )
                .await
        }
        Some(super::SessionSelectorV1::Last) => {
            client
                .stream_resume_with_env_overrides_custody(
                    codex::ResumeRequest::last().prompt(prompt),
                    &env,
                    &custody,
                )
                .await
        }
        Some(super::SessionSelectorV1::Id { id }) => {
            client
                .stream_resume_with_env_overrides_custody(
                    codex::ResumeRequest::with_id(id).prompt(prompt),
                    &env,
                    &custody,
                )
                .await
        }
    }
    .map_err(super::CodexBackendError::Exec)?;

    let codex::ExecStreamControl {
        events,
        completion,
        termination: termination_handle,
    } = handle;

    if let Some(state) = termination.as_ref() {
        state.set_handle(super::CodexTerminationHandle::Exec(termination_handle));
    }

    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<super::CodexBackendCompletion, super::CodexBackendError>>();
    let (tail_tx, tail_rx) = oneshot::channel::<Option<CodexTailEvent>>();
    let (events_done_tx, events_done_rx) = oneshot::channel::<()>();
    let stream_state_for_completion = Arc::clone(&stream_state);
    let has_effective_model_id = effective_model_id.is_some();
    let waits_on_event_classification =
        resume_selector.is_some() || has_add_dirs || has_effective_model_id;
    let events_observability = waits_on_event_classification.then(EventObservabilitySignal::new);
    tokio::spawn(async move {
        let mut events_done_rx = Some(events_done_rx);
        let outcome = completion.await;

        match outcome {
            Ok(exec_completion) => {
                // Completion classification depends on the event task when a suppressed terminal
                // `ThreadEvent::Error` can set backend-owned failure state, so this wait is a
                // correctness boundary rather than a latency optimization.
                wait_for_event_processing(&mut events_done_rx, waits_on_event_classification).await;
                let status = exec_completion.status;
                let backend_error_message =
                    snapshot_backend_error_message(&stream_state_for_completion);
                let tail = backend_error_message
                    .clone()
                    .map(|message| CodexTailEvent::TerminalError { message });
                let completion = super::CodexBackendCompletion {
                    status,
                    final_text: if backend_error_message.is_some() {
                        None
                    } else {
                        exec_completion.last_message
                    },
                    backend_error_message,
                    selection_failure_message: None,
                };
                let _ = completion_tx.send(Ok(completion));
                let _ = tail_tx.send(tail);
            }
            Err(ExecStreamError::Codex(CodexError::NonZeroExit { status, stderr })) => {
                wait_for_event_processing(&mut events_done_rx, waits_on_event_classification).await;

                let backend_error_message =
                    snapshot_backend_error_message(&stream_state_for_completion);
                let bypass_flag_unsupported_message =
                    if external_sandbox && is_unknown_bypass_flag_stderr(&stderr) {
                        Some(super::PINNED_EXTERNAL_SANDBOX_FLAG_UNSUPPORTED.to_string())
                    } else {
                        None
                    };

                let selection_failure_message = if backend_error_message.is_some() {
                    None
                } else {
                    bypass_flag_unsupported_message.or_else(|| {
                        resume_selector.as_ref().and_then(|selector| {
                            let snapshot = stream_state_for_completion.lock().ok()?;
                            if snapshot.saw_thread_id || snapshot.saw_stream_error {
                                return None;
                            }

                            let stderr_not_found = super::is_not_found_signal(&stderr);
                            let transport_message_not_found = snapshot
                                .last_transport_error_message
                                .as_deref()
                                .is_some_and(super::is_not_found_signal);
                            let transport_code_not_found = snapshot
                                .last_transport_error_code
                                .as_deref()
                                .is_some_and(super::is_not_found_signal);

                            if stderr_not_found
                                || transport_message_not_found
                                || transport_code_not_found
                            {
                                Some(super::pinned_selection_failure_message(selector).to_string())
                            } else {
                                None
                            }
                        })
                    })
                };

                let terminal_message = backend_error_message
                    .clone()
                    .or_else(|| selection_failure_message.clone());

                let tail = if let Some(message) = terminal_message {
                    CodexTailEvent::TerminalError { message }
                } else {
                    CodexTailEvent::NonZeroExit { status }
                };

                let completion = super::CodexBackendCompletion {
                    status,
                    final_text: None,
                    backend_error_message,
                    selection_failure_message,
                };
                let _ = completion_tx.send(Ok(completion));
                let _ = tail_tx.send(Some(tail));
            }
            Err(err) => {
                let _ = completion_tx.send(Err(super::CodexBackendError::Exec(err)));
                let _ = tail_tx.send(None);
            }
        }
    });

    let (event_tx, event_rx) = mpsc::channel::<
        Result<super::CodexBackendEvent, super::CodexBackendError>,
    >(DEFAULT_EVENT_CHANNEL_CAPACITY);

    tokio::spawn(async move {
        let mut events = events;
        let mut events_done_tx = Some(events_done_tx);
        let mut tail_rx = Some(tail_rx);
        let mut forward = true;

        while let Some(item) = events.next().await {
            match item {
                Ok(thread_ev) => {
                    let suppress_add_dirs_runtime_rejection = has_add_dirs
                        && matches!(
                            &thread_ev,
                            ThreadEvent::Error(err)
                                if super::is_add_dirs_runtime_rejection_signal(
                                    err.message.as_str()
                                )
                        );

                    let suppress_model_runtime_rejection =
                        effective_model_id.as_deref().is_some_and(|_| {
                            matches!(
                                &thread_ev,
                                ThreadEvent::Error(err)
                                    if is_model_runtime_rejection_signal(
                                        err.code.as_deref()
                                    )
                            )
                        });

                    if let Ok(mut snapshot) = stream_state.lock() {
                        if thread_ev.thread_id().is_some() {
                            snapshot.saw_thread_id = true;
                        }

                        if suppress_add_dirs_runtime_rejection {
                            snapshot.backend_error_message =
                                Some(super::PINNED_ADD_DIRS_RUNTIME_REJECTION.to_string());
                        } else if suppress_model_runtime_rejection
                            && snapshot.backend_error_message.is_none()
                        {
                            snapshot.backend_error_message =
                                Some(super::PINNED_MODEL_RUNTIME_REJECTION.to_string());
                        } else if suppress_transport_errors
                            && matches!(thread_ev, ThreadEvent::Error(_))
                        {
                            if let ThreadEvent::Error(err) = &thread_ev {
                                snapshot.last_transport_error_code = err.code.clone();
                                snapshot.last_transport_error_message = Some(err.message.clone());
                            }
                        }
                    }

                    if suppress_add_dirs_runtime_rejection
                        || suppress_model_runtime_rejection
                        || (suppress_transport_errors && matches!(thread_ev, ThreadEvent::Error(_)))
                    {
                        continue;
                    }

                    forward_backend_event(
                        &event_tx,
                        &mut forward,
                        Ok(super::CodexBackendEvent::Thread(Box::new(thread_ev))),
                    )
                    .await;
                }
                Err(err) => {
                    if let Ok(mut snapshot) = stream_state.lock() {
                        snapshot.saw_stream_error = true;
                    }

                    forward_backend_event(
                        &event_tx,
                        &mut forward,
                        Err(super::CodexBackendError::Exec(err)),
                    )
                    .await;
                }
            }
        }

        if let Some(tx) = events_done_tx.take() {
            let _ = tx.send(());
        }

        let tail = match tail_rx.take() {
            Some(rx) => rx.await.ok().flatten(),
            None => None,
        };

        if let Some(tail) = tail {
            let event = match tail {
                CodexTailEvent::NonZeroExit { status } => {
                    super::CodexBackendEvent::NonZeroExit { status }
                }
                CodexTailEvent::TerminalError { message } => {
                    super::CodexBackendEvent::TerminalError { message }
                }
            };
            forward_backend_event(&event_tx, &mut forward, Ok(event)).await;
        }
    });

    let events = Box::pin(ObservabilityEventStream {
        rx: event_rx,
        events_observability: events_observability.clone(),
    });

    let completion = Box::pin(async move {
        completion_rx
            .await
            .unwrap_or(Err(super::CodexBackendError::CompletionTaskDropped))
    });

    Ok(BackendSpawn {
        events,
        completion,
        events_observability,
    })
}
