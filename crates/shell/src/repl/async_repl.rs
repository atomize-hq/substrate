use std::collections::{BTreeMap, HashMap};
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use agent_api_client::AgentClient;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use agent_api_types::{
    ExecuteCancelRequestV1, ExecuteStreamFrame, MemberRuntimeBackendKindV1,
    MemberTurnSubmitRequestV1,
};
use anyhow::{anyhow, Context, Result};
use futures::{pin_mut, FutureExt, StreamExt};
use reedline::{ExternalPrinter, Prompt, Reedline, Signal};
use serde::Deserialize;
use tokio::runtime::Builder as TokioRuntimeBuilder;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task;
use uuid::Uuid;

use crate::execution::agent_events::{
    clear_agent_event_sender, format_event_line, init_event_channel, publish_agent_event,
    publish_command_completion, schedule_demo_burst, schedule_demo_events,
    ShellCommandEventContext, ShellEventEmissionContext,
};
use crate::execution::agent_inventory::{load_effective_agent_inventory, AgentInventoryEntryV1};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::execution::agent_runtime::control::spawn_remote_private_prompt_owner;
use crate::execution::agent_runtime::control::{
    apply_runtime_stop_closeout, build_session_resume_extension,
    invalidate_stale_world_members_after_binding, mark_orchestration_session_failed,
    mark_runtime_startup_failed, note_runtime_stop_requested, persist_runtime_snapshots,
    persist_world_binding_authority, private_prompt_request_channel, private_stop_request_channel,
    prompt_runtime_from_parts, register_private_prompt_transport, register_private_stop_transport,
    runtime_controls_parent_session, runtime_is_terminal, runtime_stop_transport_ids,
    spawn_local_private_prompt_owner, spawn_local_private_stop_owner, submit_host_prompt_turn,
    HiddenOwnerHelperLaunchPlan, OwnerHelperMode, PersistedWorldBinding, PrivatePromptTransport,
    PrivateStopOutcome, PrivateStopRequestReceiver, PrivateStopTransport, PublicPromptAction,
    PublicPromptEnvelope, PublicSessionPosture, ResolvedRuntimeDescriptor,
    SubmittedPromptStreamEvent, AGENT_API_SESSION_RESUME_V1, AGENT_API_TURN_LIFECYCLE_V1,
};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::execution::agent_runtime::mapping::AgentRuntimeBackendKind;
use crate::execution::agent_runtime::orchestration_session::{
    OrchestrationSessionPosture, StartupPromptStreamState,
};
use crate::execution::agent_runtime::session::AgentRuntimeReplacementParticipantInit;
use crate::execution::agent_runtime::state_store::valid_detached_host_continuity_posture;
use crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor;
use crate::execution::agent_runtime::validator::{
    exact_backend_selection_error_exit_code, validate_exact_backend_selection,
};
#[cfg(any(test, target_os = "linux", target_os = "macos"))]
use crate::execution::agent_runtime::validator::{
    member_selection_error_exit_code, validate_member_selection,
};
#[cfg(any(test, target_os = "linux", target_os = "macos"))]
use crate::execution::agent_runtime::AgentRuntimeParticipantWorldBinding;
use crate::execution::agent_runtime::{
    backend_allowed, build_gateway_for_descriptor, runtime_realizability_error_exit_code,
    validate_orchestrator_selection, validate_runtime_realizability, AgentRuntimeParticipantRecord,
    AgentRuntimeSessionManifest, AgentRuntimeSessionState, AgentRuntimeStateStore,
    OrchestrationSessionRecord, OrchestrationSessionState, MEMBER_ROLE, ORCHESTRATOR_ROLE,
    PURE_AGENT_PROTOCOL, SESSION_HANDLE_SCHEMA_V1,
};
#[cfg(unix)]
use crate::execution::get_terminal_size;
use crate::execution::ReplSessionTelemetry;
use crate::execution::WorldRootSettings;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::execution::{
    build_agent_client_and_member_dispatch_request, build_agent_client_and_pending_diff_request,
    MemberDispatchTransportRequest,
};
use crate::execution::{
    canonicalize_or, enforce_caged_destination, execute_command, find_workspace_root,
    is_shell_stream_event, needs_pty, policy_snapshot, resolve_world_root, setup_signal_handlers,
    MinimalTerminalGuard, ReplPersistentSessionClient, ReplSessionStartParams, ReplStdinMode,
    ShellConfig, PTY_ACTIVE,
};
use crate::repl::editor;
#[cfg(any(test, target_os = "linux", target_os = "macos"))]
use substrate_broker::Policy;
use substrate_broker::{detect_profile, world_fs_policy};
use substrate_common::agent_events::{AgentEvent, MessageEventKind};
use substrate_common::paths as substrate_paths;
use substrate_common::WorldRootMode;

#[derive(Clone)]
enum ReplPrinter {
    Reedline(ExternalPrinter<String>),
    Stdout,
}

impl ReplPrinter {
    fn print(&self, line: impl Into<String>) {
        match self {
            ReplPrinter::Reedline(printer) => {
                let _ = printer.print(line.into());
            }
            ReplPrinter::Stdout => {
                write_locked_stdout_line(&line.into());
            }
        }
    }
}

fn write_locked_stdout(bytes: &[u8]) {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    let _ = lock.write_all(bytes);
    let _ = lock.flush();
}

fn write_locked_stdout_line(line: &str) {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    let _ = lock.write_all(line.as_bytes());
    let _ = lock.write_all(b"\n");
    let _ = lock.flush();
}

fn write_best_effort_stdout_line(line: &str) {
    write_best_effort_stdout(format!("{line}\n").as_bytes());
}

fn write_best_effort_stderr_line(line: &str) {
    write_best_effort_stderr(format!("{line}\n").as_bytes());
}

#[cfg(unix)]
fn write_best_effort_unix<F>(bytes: &[u8], mut write_once: F) -> usize
where
    F: FnMut(&[u8]) -> io::Result<usize>,
{
    let mut offset = 0usize;
    while offset < bytes.len() {
        match write_once(&bytes[offset..]) {
            Ok(0) => break,
            Ok(written) => {
                offset += written.min(bytes.len() - offset);
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    offset
}

#[cfg(unix)]
fn write_best_effort_fd(fd: libc::c_int, bytes: &[u8]) {
    let _ = write_best_effort_unix(bytes, |remaining| {
        let written = unsafe { libc::write(fd, remaining.as_ptr().cast(), remaining.len()) };
        if written < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(written as usize)
        }
    });
}

#[cfg(unix)]
fn write_best_effort_stdout(bytes: &[u8]) {
    write_best_effort_fd(libc::STDOUT_FILENO, bytes);
}

#[cfg(not(unix))]
fn write_best_effort_stdout(bytes: &[u8]) {
    let _ = io::stdout().write_all(bytes);
    let _ = io::stdout().flush();
}

#[cfg(unix)]
fn write_best_effort_stderr(bytes: &[u8]) {
    write_best_effort_fd(libc::STDERR_FILENO, bytes);
}

#[cfg(not(unix))]
fn write_best_effort_stderr(bytes: &[u8]) {
    let _ = io::stderr().write_all(bytes);
    let _ = io::stderr().flush();
}

fn is_cursor_position_timeout_error(err: &anyhow::Error) -> bool {
    err.to_string()
        .contains("cursor position could not be read within a normal duration")
}

fn is_terminal_loss_io_error(err: &std::io::Error) -> bool {
    if matches!(
        err.kind(),
        std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::NotConnected
            | std::io::ErrorKind::UnexpectedEof
    ) {
        return true;
    }

    #[cfg(unix)]
    {
        if let Some(code) = err.raw_os_error() {
            return matches!(
                code,
                libc::ENOTTY | libc::EIO | libc::EBADF | libc::ENXIO | libc::ENODEV
            );
        }
    }

    false
}

fn is_terminal_loss_error(err: &anyhow::Error) -> bool {
    for cause in err.chain() {
        if let Some(io_err) = cause.downcast_ref::<std::io::Error>() {
            if is_terminal_loss_io_error(io_err) {
                return true;
            }
        }
    }

    let message = err.to_string().to_lowercase();
    message.contains("broken pipe")
        || message.contains("not connected")
        || message.contains("bad file descriptor")
        || message.contains("inappropriate ioctl for device")
        || message.contains("input/output error")
        || message.contains("terminal invalid")
        || message.contains("end of file")
        || message.contains("unexpected eof")
}

fn detect_terminal_loss_while_prompting() -> Option<anyhow::Error> {
    if !io::stdin().is_terminal() {
        return Some(anyhow!("controlling terminal is no longer a TTY"));
    }

    #[cfg(unix)]
    {
        let mut termios = std::mem::MaybeUninit::<libc::termios>::uninit();
        let rc = unsafe { libc::tcgetattr(libc::STDIN_FILENO, termios.as_mut_ptr()) };
        if rc == 0 {
            #[cfg(target_os = "macos")]
            {
                match std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("/dev/tty")
                {
                    Ok(file) => {
                        use std::os::fd::AsRawFd;

                        let rc = unsafe { libc::tcgetattr(file.as_raw_fd(), termios.as_mut_ptr()) };
                        if rc == 0 {
                            return None;
                        }

                        let err = std::io::Error::last_os_error();
                        if is_terminal_loss_io_error(&err) {
                            return Some(
                                anyhow!(err).context("controlling terminal became invalid"),
                            );
                        }
                    }
                    Err(err) if is_terminal_loss_io_error(&err) => {
                        return Some(anyhow!(err).context("controlling terminal became invalid"));
                    }
                    Err(_) => {}
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                return None;
            }
        }

        let err = std::io::Error::last_os_error();
        if is_terminal_loss_io_error(&err) {
            return Some(anyhow!(err).context("controlling terminal became invalid"));
        }
    }

    None
}

fn emit_best_effort_terminal_loss_diagnostic(err: &anyhow::Error) {
    let message = format!("substrate: error: abnormal terminal loss: {err:#}\n");
    write_best_effort_stderr(message.as_bytes());
}

fn spawn_prompt_terminal_loss_monitor(
    prompt_active: Arc<AtomicBool>,
    terminal_loss_detected: Arc<AtomicBool>,
    terminal_loss_message: Arc<Mutex<Option<String>>>,
) {
    thread::spawn(move || {
        while prompt_active.load(Ordering::SeqCst) {
            if let Some(err) = detect_terminal_loss_while_prompting() {
                let mut guard = terminal_loss_message
                    .lock()
                    .expect("terminal loss message mutex poisoned");
                *guard = Some(format!("{err:#}"));
                terminal_loss_detected.store(true, Ordering::SeqCst);
                #[cfg(unix)]
                unsafe {
                    // Best effort: force any blocked prompt read off stdin once the TTY is known
                    // to be invalid so Reedline can surface an error instead of spinning forever.
                    let _ = libc::close(libc::STDIN_FILENO);
                }
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
}

const REEDLINE_CTRLD_TERMINAL_LOSS_RECHECKS: usize = 6;
const REEDLINE_CTRLD_TERMINAL_LOSS_RECHECK_DELAY: Duration = Duration::from_millis(25);

fn take_detected_terminal_loss_message(
    terminal_loss_detected: &AtomicBool,
    terminal_loss_message: &Mutex<Option<String>>,
) -> Option<anyhow::Error> {
    if terminal_loss_detected.load(Ordering::SeqCst) {
        let message = terminal_loss_message
            .lock()
            .expect("terminal loss message mutex poisoned")
            .take()
            .unwrap_or_else(|| "controlling terminal became invalid".to_string());
        return Some(anyhow!(message));
    }

    None
}

fn resolve_reedline_ctrl_d_terminal_loss(
    terminal_loss_detected: &AtomicBool,
    terminal_loss_message: &Mutex<Option<String>>,
) -> Option<anyhow::Error> {
    resolve_reedline_ctrl_d_terminal_loss_with(
        terminal_loss_detected,
        terminal_loss_message,
        detect_terminal_loss_while_prompting,
        || thread::sleep(REEDLINE_CTRLD_TERMINAL_LOSS_RECHECK_DELAY),
    )
}

fn resolve_reedline_ctrl_d_terminal_loss_with<D, S>(
    terminal_loss_detected: &AtomicBool,
    terminal_loss_message: &Mutex<Option<String>>,
    mut detect_terminal_loss: D,
    mut sleep_between_rechecks: S,
) -> Option<anyhow::Error>
where
    D: FnMut() -> Option<anyhow::Error>,
    S: FnMut(),
{
    if let Some(err) =
        take_detected_terminal_loss_message(terminal_loss_detected, terminal_loss_message)
    {
        return Some(err);
    }

    for attempt in 0..REEDLINE_CTRLD_TERMINAL_LOSS_RECHECKS {
        if let Some(err) = detect_terminal_loss() {
            return Some(err);
        }

        if let Some(err) =
            take_detected_terminal_loss_message(terminal_loss_detected, terminal_loss_message)
        {
            return Some(err);
        }

        if attempt + 1 < REEDLINE_CTRLD_TERMINAL_LOSS_RECHECKS {
            sleep_between_rechecks();
        }
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplTerminationCause {
    NormalExit,
    AbnormalTerminalLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptWorkerShutdownDisposition {
    Graceful,
    Abandon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptWorkerErrorDisposition {
    FallbackToStdio,
    AbnormalTerminalLoss,
    GenericError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostRuntimeShutdownMode {
    Stop,
    ParkIfResumable,
}

fn classify_prompt_worker_error(
    is_reedline: bool,
    err: &anyhow::Error,
) -> PromptWorkerErrorDisposition {
    if is_reedline && is_cursor_position_timeout_error(err) {
        PromptWorkerErrorDisposition::FallbackToStdio
    } else if is_reedline && is_terminal_loss_error(err) {
        PromptWorkerErrorDisposition::AbnormalTerminalLoss
    } else {
        PromptWorkerErrorDisposition::GenericError
    }
}

fn shutdown_disposition_for_termination_cause(
    cause: ReplTerminationCause,
) -> PromptWorkerShutdownDisposition {
    match cause {
        ReplTerminationCause::NormalExit => PromptWorkerShutdownDisposition::Graceful,
        ReplTerminationCause::AbnormalTerminalLoss => PromptWorkerShutdownDisposition::Abandon,
    }
}

struct ReplPreflight {
    entered_cwd: String,
    exit_cwd: crate::execution::config_model::ReplExitCwdMode,
    max_pty_buffered_lines: usize,
    max_pty_buffered_lines_clamp: Option<crate::execution::config_model::I64ClampInfo>,
}

pub(crate) fn run_async_repl(config: &ShellConfig) -> Result<i32> {
    let preflight = preflight_caging_required(config)?;
    write_best_effort_stdout_line(&format!("Substrate v{}", env!("CARGO_PKG_VERSION")));
    write_best_effort_stdout_line(&format!("Session ID: {}", config.session_id));
    write_best_effort_stdout_line(&format!("Logging to: {}", config.trace_log_file.display()));

    let running_child_pid = Arc::new(AtomicI32::new(0));
    setup_signal_handlers(running_child_pid.clone())?;

    let rt = TokioRuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize async REPL runtime")?;

    let shared_config = Arc::new(config.clone());

    let entered_cwd = preflight.entered_cwd.clone();
    let repl_exit_cwd = preflight.exit_cwd;
    let max_pty_buffered_lines = preflight.max_pty_buffered_lines;
    let max_pty_buffered_lines_clamp = preflight.max_pty_buffered_lines_clamp;

    let exit_code = rt.block_on(async move {
        let mut telemetry = ReplSessionTelemetry::new(shared_config.clone(), "async");
        if let Some(clamp) = max_pty_buffered_lines_clamp {
            telemetry.persist_warning_config_value_clamped(
                "repl.max_pty_buffered_lines",
                clamp.provided,
                clamp.effective,
                clamp.min,
                clamp.max,
            );
        }
        let mut prompt_worker = PromptWorker::spawn(shared_config.clone())
            .context("failed to start Reedline worker")?;
        let mut agent_printer = prompt_worker.printer_handle();
        let stdout_printer = prompt_worker.external_printer_handle();
        let mut prompt_responses = prompt_worker.take_response_receiver();
        let mut agent_rx = init_event_channel();

        let host_escape_enabled = shared_config.repl_host_escape;
        let mut host_state = HostState::new().context("failed to initialize host state")?;

        let (resize_tx, mut resize_rx) = mpsc::unbounded_channel::<(u16, u16)>();
        spawn_resize_task(resize_tx);

        let (sigint_tx, mut sigint_rx) = mpsc::unbounded_channel::<()>();
        spawn_sigint_task(sigint_tx);

        let prompt_active = Arc::new(AtomicBool::new(false));
        let stdout_cb = make_world_stdout_callback(prompt_active.clone(), stdout_printer);
        let resolved_host_bootstrap = match resolve_host_orchestrator_bootstrap(&shared_config) {
            Ok(resolved) => resolved,
            Err(failure) => {
                agent_printer.print(failure.message.clone());
                write_best_effort_stderr_line(&failure.message);
                tokio::time::sleep(Duration::from_millis(100)).await;
                return Ok(failure.exit_code);
            }
        };
        let (prepared_runtime, mut dormant_host_bootstrap) = match resolved_host_bootstrap {
            Some(resolved) if should_eager_bootstrap_host_orchestrator(&resolved) => {
                let prepared = match prepare_host_orchestrator_runtime_from_resolved(resolved) {
                    Ok(prepared) => prepared,
                    Err(failure) => {
                        agent_printer.print(failure.message.clone());
                        write_best_effort_stderr_line(&failure.message);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        return Ok(failure.exit_code);
                    }
                };
                (Some(prepared), None)
            }
            Some(resolved) => (None, Some(resolved)),
            None => (None, None),
        };
        let mut startup_context = prepared_runtime
            .as_ref()
            .map(|prepared| prepared.startup_context.clone());

        let mut world_session = if !shared_config.no_world {
            let requested = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .display()
                .to_string();
            match start_world_session(
                requested,
                startup_context.as_ref(),
                stdout_cb.clone(),
                &agent_printer,
                &mut telemetry,
            )
            .await
            {
                Ok(session) => Some(session),
                Err(err) => {
                    if let Some(startup_context) = startup_context.as_ref() {
                        mark_orchestration_session_failed(
                            &startup_context.store,
                            &startup_context.orchestration_session,
                            format!("failed to start persistent world session: {err:#}"),
                        );
                    }
                    let exit_code = if is_world_restart_required_error(&err) {
                        3
                    } else {
                        1
                    };
                    let message = if is_world_restart_required_error(&err) {
                        err.to_string()
                    } else {
                        format!(
                            "substrate: error: failed to start persistent world session: {err:#}"
                        )
                    };
                    agent_printer.print(message.clone());
                    write_best_effort_stderr_line(&message);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    return Ok(exit_code);
                }
            }
        } else {
            None
        };
        let initial_world_binding = world_session.as_ref().map(|session| PersistedWorldBinding {
            world_id: session.world_id.clone(),
            world_generation: session.world_generation,
        });
        let mut agent_runtime = match start_host_orchestrator_runtime_with_prepared(
            prepared_runtime,
            initial_world_binding.as_ref(),
            &agent_printer,
            &mut telemetry,
        )
        .await
        {
            Ok(runtime) => runtime,
            Err(failure) => {
                finalize_runtime_startup_failure(
                    startup_context.as_ref(),
                    &mut world_session,
                    &failure.message,
                )
                .await;
                agent_printer.print(failure.message.clone());
                write_best_effort_stderr_line(&failure.message);
                tokio::time::sleep(Duration::from_millis(100)).await;
                return Ok(failure.exit_code);
            }
        };
        let mut member_runtimes = RetainedMemberRuntimeMap::new();
        let mut pending_member_replacements = PendingMemberReplacementMap::new();

        let mut should_exit = false;
        let mut termination_cause = ReplTerminationCause::NormalExit;
        let mut fatal_runtime_error: Option<anyhow::Error> = None;
        'repl_loop: while !should_exit {
            prompt_active.store(true, Ordering::SeqCst);
            if let Err(err) = prompt_worker.request_prompt() {
                termination_cause = ReplTerminationCause::AbnormalTerminalLoss;
                should_exit = true;
                emit_best_effort_terminal_loss_diagnostic(
                    &anyhow!("failed to request prompt: {err:#}"),
                );
                prompt_active.store(false, Ordering::SeqCst);
                continue;
            }

            let terminal_loss_detected = Arc::new(AtomicBool::new(false));
            let terminal_loss_message = Arc::new(Mutex::new(None::<String>));
            if prompt_worker.is_reedline() {
                spawn_prompt_terminal_loss_monitor(
                    prompt_active.clone(),
                    terminal_loss_detected.clone(),
                    terminal_loss_message.clone(),
                );
            }
            let mut prompt_health_check = tokio::time::interval(Duration::from_millis(100));
            prompt_health_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            let prompt_response = loop {
                tokio::select! {
                    resp = prompt_responses.recv() => {
                        match resp {
                            Some(resp) => break resp,
                            None => {
                                termination_cause = ReplTerminationCause::AbnormalTerminalLoss;
                                break PromptWorkerResponse::Error(anyhow!(
                                    "prompt worker closed unexpectedly"
                                ));
                            }
                        }
                    }
                    maybe_event = agent_rx.recv() => {
                        if let Some(event) = maybe_event {
                            handle_agent_event(event, &mut telemetry, &agent_printer);
                        }
                    }
                    maybe_resize = resize_rx.recv() => {
                        if let Some((cols, rows)) = maybe_resize {
                            if let Some(session) = world_session.as_ref() {
                                let _ = session.client.send_resize(cols, rows).await;
                            }
                        }
                    }
                    _maybe_sigint = sigint_rx.recv() => {
                        // In Idle, Reedline handles Ctrl+C; ignore host-originated SIGINT here.
                    }
                    _ = prompt_health_check.tick(), if prompt_worker.is_reedline() => {
                        if terminal_loss_detected.load(Ordering::SeqCst) {
                            let message = terminal_loss_message
                                .lock()
                                .expect("terminal loss message mutex poisoned")
                                .take()
                                .unwrap_or_else(|| "controlling terminal became invalid".to_string());
                            termination_cause = ReplTerminationCause::AbnormalTerminalLoss;
                            break PromptWorkerResponse::AbnormalTerminalLoss(anyhow!(message));
                        }
                    }
                }
            };
            let prompt_response = match prompt_response {
                PromptWorkerResponse::CtrlD if prompt_worker.is_reedline() => {
                    if let Some(err) = resolve_reedline_ctrl_d_terminal_loss(
                        terminal_loss_detected.as_ref(),
                        terminal_loss_message.as_ref(),
                    ) {
                        termination_cause = ReplTerminationCause::AbnormalTerminalLoss;
                        PromptWorkerResponse::AbnormalTerminalLoss(err)
                    } else {
                        PromptWorkerResponse::CtrlD
                    }
                }
                other => other,
            };
            // Keep the monitor alive through the Reedline Ctrl-D recheck window so delayed TTY
            // invalidation can still be observed before we classify the prompt result.
            prompt_active.store(false, Ordering::SeqCst);

            match prompt_response {
                PromptWorkerResponse::Line(command) => {
                    loop {
                        match agent_rx.try_recv() {
                            Ok(event) => handle_agent_event(event, &mut telemetry, &agent_printer),
                            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                        }
                    }

                    let trimmed = command.trim();

                    if trimmed.is_empty() {
                        continue;
                    }

                    if is_exit_directive(trimmed) && !has_embedded_newlines(&command) {
                        should_exit = true;
                        continue;
                    }

                    if trimmed == ":demo-agent" {
                        schedule_demo_events();
                        continue;
                    }

                    if let Some((agents, events, delay_ms)) = parse_demo_burst(trimmed) {
                        schedule_demo_burst(agents, events, Duration::from_millis(delay_ms));
                        write_best_effort_stdout_line(&format!(
                            "[demo] scheduled burst: agents={}, events_per_agent={}, delay_ms={}",
                            agents, events, delay_ms
                        ));
                        continue;
                    }

                    let trimmed_owned = trimmed.to_string();
                    telemetry.record_input_event();

                    let cmd_id = Uuid::now_v7().to_string();

                    if trimmed.starts_with("::") {
                        let Some(targeted_turn) = parse_targeted_turn(trimmed) else {
                            agent_printer.print(
                                "substrate: error: targeted follow-up turns require exact syntax '::<backend_id> <prompt>' on a single line",
                            );
                            continue;
                        };

                        let result = match dispatch_targeted_follow_up_turn(
                            targeted_turn,
                            TargetedTurnDispatchContext {
                                startup_context: &mut startup_context,
                                dormant_host_bootstrap: &mut dormant_host_bootstrap,
                                agent_runtime: &mut agent_runtime,
                                world_session: &mut world_session,
                                member_runtimes: &mut member_runtimes,
                                pending_member_replacements: &mut pending_member_replacements,
                                agent_printer: &agent_printer,
                                telemetry: &mut telemetry,
                            },
                        )
                        .await
                        {
                            Ok(TargetedTurnDispatchStatus::Submitted) => Ok(()),
                            Ok(TargetedTurnDispatchStatus::Rejected(failure)) => {
                                agent_printer
                                    .print(format!("substrate: error: {}", failure.message));
                                continue;
                            }
                            Err(err) => Err(err),
                        };

                        if let Err(err) = result {
                            fatal_runtime_error = Some(err);
                            should_exit = true;
                            continue 'repl_loop;
                        }
                        continue;
                    }

                    if !has_embedded_newlines(&command) {
                        if trimmed == ":host" {
                            agent_printer.print("substrate: error: :host requires a command");
                            continue;
                        }
                        if trimmed == ":pty" {
                            agent_printer.print("substrate: error: :pty requires a command");
                            continue;
                        }

                        if let Some(rest) = command.strip_prefix(":host ") {
                            let host_cmd = rest.trim_start();
                            if host_cmd.is_empty() {
                                agent_printer.print("substrate: error: :host requires a command");
                                continue;
                            }
                            if !host_escape_enabled {
                                agent_printer.print("substrate: error: host escape not enabled (use --repl-host-escape or SUBSTRATE_REPL_HOST_ESCAPE=1)");
                                continue;
                            }

                            let mut io_ctx = ReplIo {
                                agent_rx: &mut agent_rx,
                                resize_rx: &mut resize_rx,
                                sigint_rx: &mut sigint_rx,
                                telemetry: &mut telemetry,
                                agent_printer: &agent_printer,
                                max_pty_buffered_lines,
                            };
                            let command_event_context = build_repl_shell_command_event_context(
                                startup_context.as_ref(),
                                agent_runtime.as_ref(),
                                world_session.as_ref(),
                                &cmd_id,
                            );
                            let exit_code = exec_host_line(
                                shared_config.as_ref(),
                                &mut host_state,
                                host_cmd,
                                &cmd_id,
                                running_child_pid.clone(),
                                world_session.as_ref().map(|s| &s.client),
                                &mut io_ctx,
                            )
                            .await;
                            let exit_code = match exit_code {
                                Ok(exit_code) => exit_code,
                                Err(err) => {
                                    fatal_runtime_error = Some(err);
                                    should_exit = true;
                                    continue 'repl_loop;
                                }
                            };
                            let status = exit_status_from_code(exit_code);
                            report_nonzero_status(&status);
                            publish_command_completion(
                                command_event_context.as_ref(),
                                &trimmed_owned,
                                &status,
                            );
                            telemetry.record_command();
                            continue;
                        }

                        if let Some(rest) = command.strip_prefix(":pty ") {
                            let pty_cmd = rest.trim_start();
                            if pty_cmd.is_empty() {
                                agent_printer.print("substrate: error: :pty requires a command");
                                continue;
                            }

                            if world_session.is_some() {
                                let drift_check = ensure_no_policy_drift(
                                    &mut world_session,
                                    startup_context.as_ref(),
                                    &agent_printer,
                                    &mut telemetry,
                                )
                                .await;
                                if let Err(err) = drift_check {
                                    fatal_runtime_error = Some(err);
                                    should_exit = true;
                                    continue 'repl_loop;
                                }
                                if let Err(err) = reconcile_member_runtime_generation(
                                    world_session.as_ref(),
                                    &mut member_runtimes,
                                    &mut pending_member_replacements,
                                    &agent_printer,
                                    &mut telemetry,
                                )
                                .await
                                {
                                    fatal_runtime_error = Some(err);
                                    should_exit = true;
                                    continue 'repl_loop;
                                }
                                if let Err(err) = ensure_member_runtime_ready(
                                    startup_context.as_ref(),
                                    world_session.as_ref(),
                                    &mut member_runtimes,
                                    &mut pending_member_replacements,
                                    &agent_printer,
                                    &mut telemetry,
                                )
                                .await
                                {
                                    fatal_runtime_error = Some(err);
                                    should_exit = true;
                                    continue 'repl_loop;
                                }
                                let command_event_context =
                                    build_repl_shell_command_event_context(
                                        startup_context.as_ref(),
                                        agent_runtime.as_ref(),
                                        world_session.as_ref(),
                                        &cmd_id,
                                    );
                                let exit_code = {
                                    let session = world_session
                                        .as_mut()
                                        .expect("world_session present after ensure_no_policy_drift");
                                    let mut io_ctx = ReplIo {
                                        agent_rx: &mut agent_rx,
                                        resize_rx: &mut resize_rx,
                                        sigint_rx: &mut sigint_rx,
                                        telemetry: &mut telemetry,
                                        agent_printer: &agent_printer,
                                        max_pty_buffered_lines,
                                    };
                                    match exec_world_pty(session, pty_cmd, &cmd_id, &mut io_ctx)
                                        .await
                                    {
                                        Ok(exit_code) => exit_code,
                                        Err(err) => {
                                            fatal_runtime_error = Some(err);
                                            should_exit = true;
                                            continue 'repl_loop;
                                        }
                                    }
                                };
                                let drift_check = ensure_no_policy_drift(
                                    &mut world_session,
                                    startup_context.as_ref(),
                                    &agent_printer,
                                    &mut telemetry,
                                )
                                .await;
                                if let Err(err) = drift_check {
                                    fatal_runtime_error = Some(err);
                                    should_exit = true;
                                    continue 'repl_loop;
                                }
                                if let Err(err) = reconcile_member_runtime_generation(
                                    world_session.as_ref(),
                                    &mut member_runtimes,
                                    &mut pending_member_replacements,
                                    &agent_printer,
                                    &mut telemetry,
                                )
                                .await
                                {
                                    fatal_runtime_error = Some(err);
                                    should_exit = true;
                                    continue 'repl_loop;
                                }
                                let status = exit_status_from_code(exit_code);
                                report_nonzero_status(&status);
                                publish_command_completion(
                                    command_event_context.as_ref(),
                                    &trimmed_owned,
                                    &status,
                                );
                                telemetry.record_command();
                                continue;
                            }
                        }
                    }

                    if world_session.is_some() {
                        let drift_check = ensure_no_policy_drift(
                            &mut world_session,
                            startup_context.as_ref(),
                            &agent_printer,
                            &mut telemetry,
                        )
                        .await;
                        if let Err(err) = drift_check {
                            fatal_runtime_error = Some(err);
                            should_exit = true;
                            continue 'repl_loop;
                        }
                        if let Err(err) = reconcile_member_runtime_generation(
                            world_session.as_ref(),
                            &mut member_runtimes,
                            &mut pending_member_replacements,
                            &agent_printer,
                            &mut telemetry,
                        )
                        .await
                        {
                            fatal_runtime_error = Some(err);
                            should_exit = true;
                            continue 'repl_loop;
                        }
                        if let Err(err) = ensure_member_runtime_ready(
                            startup_context.as_ref(),
                            world_session.as_ref(),
                            &mut member_runtimes,
                            &mut pending_member_replacements,
                            &agent_printer,
                            &mut telemetry,
                        )
                        .await
                        {
                            fatal_runtime_error = Some(err);
                            should_exit = true;
                            continue 'repl_loop;
                        }
                        let command_event_context = build_repl_shell_command_event_context(
                            startup_context.as_ref(),
                            agent_runtime.as_ref(),
                            world_session.as_ref(),
                            &cmd_id,
                        );
                        let pty = needs_pty(trimmed);
                        let exit_code = {
                            let session = world_session
                                .as_mut()
                                .expect("world_session present after ensure_no_policy_drift");
                            let mut io_ctx = ReplIo {
                                agent_rx: &mut agent_rx,
                                resize_rx: &mut resize_rx,
                                sigint_rx: &mut sigint_rx,
                                telemetry: &mut telemetry,
                                agent_printer: &agent_printer,
                                max_pty_buffered_lines,
                            };
                            if pty {
                                match exec_world_pty(session, &command, &cmd_id, &mut io_ctx)
                                    .await
                                {
                                    Ok(exit_code) => exit_code,
                                    Err(err) => {
                                        fatal_runtime_error = Some(err);
                                        should_exit = true;
                                        continue 'repl_loop;
                                    }
                                }
                            } else {
                                match exec_world_line(session, &command, &cmd_id, &mut io_ctx)
                                    .await
                                {
                                    Ok(exit_code) => exit_code,
                                    Err(err) => {
                                        fatal_runtime_error = Some(err);
                                        should_exit = true;
                                        continue 'repl_loop;
                                    }
                                }
                            }
                        };
                        let drift_check = ensure_no_policy_drift(
                            &mut world_session,
                            startup_context.as_ref(),
                            &agent_printer,
                            &mut telemetry,
                        )
                        .await;
                        if let Err(err) = drift_check {
                            fatal_runtime_error = Some(err);
                            should_exit = true;
                            continue 'repl_loop;
                        }
                        if let Err(err) = reconcile_member_runtime_generation(
                            world_session.as_ref(),
                            &mut member_runtimes,
                            &mut pending_member_replacements,
                            &agent_printer,
                            &mut telemetry,
                        )
                        .await
                        {
                            fatal_runtime_error = Some(err);
                            should_exit = true;
                            continue 'repl_loop;
                        }
                        let status = exit_status_from_code(exit_code);
                        report_nonzero_status(&status);
                        publish_command_completion(
                            command_event_context.as_ref(),
                            &trimmed_owned,
                            &status,
                        );
                        telemetry.record_command();
                        continue;
                    }

                    // Host-only mode (explicit --no-world)
                    let host_pty_passthrough = trimmed.starts_with(":pty ") || needs_pty(trimmed);
                    let config_clone = (*shared_config).clone();
                    let running_clone = running_child_pid.clone();
                    let command_for_exec = command.clone();
                    let cmd_id_for_exec = cmd_id.clone();
                    let command_event_context = build_repl_shell_command_event_context(
                        startup_context.as_ref(),
                        agent_runtime.as_ref(),
                        world_session.as_ref(),
                        &cmd_id,
                    );
                    let command_event_context_for_exec = command_event_context.clone();
                    let command_fut = task::spawn_blocking(move || {
                        execute_command(
                            &config_clone,
                            &command_for_exec,
                            &cmd_id_for_exec,
                            running_clone,
                            command_event_context_for_exec,
                        )
                    })
                    .map(|res: Result<Result<ExitStatus, anyhow::Error>, tokio::task::JoinError>| {
                        match res {
                            Ok(inner) => inner,
                            Err(err) => Err(anyhow!(err)),
                        }
                    });
                    pin_mut!(command_fut);

                    let mut buffered_structured_lines = Vec::<String>::new();
                    let mut dropped_structured_event_lines: u64 = 0;

                    let status = loop {
                        tokio::select! {
                            res = &mut command_fut => {
                                match res {
                                    Ok(status) => break Some(status),
                                    Err(err) => {
                                        fatal_runtime_error = Some(err);
                                        break None;
                                    }
                                }
                            }
                            maybe_event = agent_rx.recv() => {
                                if let Some(event) = maybe_event {
                                    if is_shell_stream_event(&event) {
                                        continue;
                                    }

                                    telemetry.persist_agent_event(&event);
                                    telemetry.record_agent_event();

                                    if host_pty_passthrough {
                                        if buffered_structured_lines.len() < max_pty_buffered_lines {
                                            buffered_structured_lines.push(format_event_line(&event));
                                        } else {
                                            dropped_structured_event_lines =
                                                dropped_structured_event_lines.saturating_add(1);
                                        }
                                    } else {
                                        agent_printer.print(format_event_line(&event));
                                    }
                                }
                            }
                            _maybe_resize = resize_rx.recv() => {}
                            _maybe_sigint = sigint_rx.recv() => {}
                        }
                    };
                    let Some(status) = status else {
                        should_exit = true;
                        continue 'repl_loop;
                    };

                    for line in buffered_structured_lines {
                        agent_printer.print(line);
                    }
                    if dropped_structured_event_lines > 0 {
                        telemetry.persist_warning_pty_structured_event_drops(
                            dropped_structured_event_lines,
                            max_pty_buffered_lines,
                            Some(&cmd_id),
                        );
                        agent_printer.print(format!(
                            "substrate: warning: dropped {dropped_structured_event_lines} structured agent event line(s) during PTY passthrough (cap={max_pty_buffered_lines})"
                        ));
                    }

                    report_nonzero_status(&status);
                    publish_command_completion(
                        command_event_context.as_ref(),
                        &trimmed_owned,
                        &status,
                    );
                    telemetry.record_command();
                }
                PromptWorkerResponse::CtrlC => {
                    write_best_effort_stdout_line("^C");
                }
                PromptWorkerResponse::CtrlD => {
                    write_best_effort_stdout_line("^D");
                    should_exit = true;
                }
                PromptWorkerResponse::AbnormalTerminalLoss(err) => {
                    termination_cause = ReplTerminationCause::AbnormalTerminalLoss;
                    should_exit = true;
                    emit_best_effort_terminal_loss_diagnostic(&err);
                }
                PromptWorkerResponse::Error(err) => {
                    match classify_prompt_worker_error(prompt_worker.is_reedline(), &err) {
                        PromptWorkerErrorDisposition::FallbackToStdio => {
                            write_best_effort_stderr_line(
                                "substrate: warning: prompt backend degraded (cursor query timeout); falling back to plain stdin reader"
                            );
                            prompt_worker = match PromptWorker::spawn_stdio(shared_config.clone())
                                .context("failed to start plain prompt worker")
                            {
                                Ok(worker) => worker,
                                Err(err) => {
                                    fatal_runtime_error = Some(err);
                                    should_exit = true;
                                    continue 'repl_loop;
                                }
                            };
                            agent_printer = prompt_worker.printer_handle();
                            prompt_responses = prompt_worker.take_response_receiver();
                            continue;
                        }
                        PromptWorkerErrorDisposition::AbnormalTerminalLoss => {
                            termination_cause = ReplTerminationCause::AbnormalTerminalLoss;
                            should_exit = true;
                            emit_best_effort_terminal_loss_diagnostic(&err);
                        }
                        PromptWorkerErrorDisposition::GenericError => {
                            write_best_effort_stderr_line(&format!("prompt error: {err}"));
                            should_exit = true;
                        }
                    }
                }
            }
        }

        let note_lines = {
            let last_world_cwd = world_session
                .as_ref()
                .map(|s| s.world_cwd.clone())
                .unwrap_or_else(|| entered_cwd.clone());
            if last_world_cwd != entered_cwd {
                let (exit_target, fallback_reason) = match repl_exit_cwd {
                    crate::execution::config_model::ReplExitCwdMode::Entered => {
                        (entered_cwd.clone(), None)
                    }
                    crate::execution::config_model::ReplExitCwdMode::LastWorld => {
                        let reason = if has_embedded_newlines(&last_world_cwd) {
                            Some(
                                "last world cwd is not representable (embedded newlines)"
                                    .to_string(),
                            )
                        } else if !Path::new(&last_world_cwd).is_absolute() {
                            Some(
                                "last world cwd is not representable (not an absolute path)"
                                    .to_string(),
                            )
                        } else if !Path::new(&last_world_cwd).is_dir() {
                            Some(
                                "last world cwd does not exist as a directory on the host at exit"
                                    .to_string(),
                            )
                        } else {
                            None
                        };
                        if let Some(reason) = reason {
                            (entered_cwd.clone(), Some(reason))
                        } else {
                            (last_world_cwd, None)
                        }
                    }
                };

                let mut lines = vec![format!(
                    "substrate: note: returning to host cwd: {}",
                    exit_target
                )];
                if let Some(reason) = fallback_reason {
                    lines.push(format!(
                        "substrate: note: repl.exit_cwd=last_world fallback to entered cwd ({})",
                        reason
                    ));
                }
                Some(lines)
            } else {
                None
            }
        };

        shutdown_all_member_runtimes(&mut member_runtimes, &agent_printer, &mut telemetry).await;
        if let Some(runtime) = agent_runtime.take() {
            let shutdown_mode = if termination_cause == ReplTerminationCause::AbnormalTerminalLoss
            {
                HostRuntimeShutdownMode::ParkIfResumable
            } else {
                HostRuntimeShutdownMode::Stop
            };
            shutdown_host_orchestrator_runtime_with_mode(
                runtime,
                shutdown_mode,
                &agent_printer,
                &mut telemetry,
            )
            .await;
        }
        drain_pending_agent_events(&mut agent_rx, &mut telemetry, &agent_printer);
        prompt_worker.shutdown_with_disposition(shutdown_disposition_for_termination_cause(
            termination_cause,
        ));
        clear_agent_event_sender();

        let auto_sync_exit_code: i32 = {
            let cwd_for_profile = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let cli_world_enabled = if shared_config.cli_world {
                Some(true)
            } else if shared_config.cli_no_world {
                Some(false)
            } else {
                None
            };

            let effective_config = crate::execution::config_model::resolve_effective_config(
                &cwd_for_profile,
                &crate::execution::config_model::CliConfigOverrides {
                    world_enabled: cli_world_enabled,
                    anchor_mode: shared_config.cli_anchor_mode,
                    anchor_path: shared_config
                        .cli_anchor_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string()),
                    caged: shared_config.cli_caged,
                },
            )?;

            if shared_config.no_world || !effective_config.world.enabled {
                0
            } else {
                let cfg = (*shared_config).clone();
                let effective = effective_config.clone();
                task::spawn_blocking(move || crate::execution::run_auto_sync_if_enabled(&cfg, &effective))
                    .await
                    .map_err(|e| anyhow!(e))??
            }
        };

        if let Some(lines) = note_lines {
            for line in lines {
                write_best_effort_stdout_line(&line);
            }
        }

        if termination_cause == ReplTerminationCause::NormalExit {
            io::stdout().flush().ok();
        }

        if let Some(session) = world_session.take() {
            let _ = session.client.close().await;
        }

        if let Some(err) = fatal_runtime_error.as_ref() {
            let (exit_code, message) = runtime_loop_exit(err);
            agent_printer.print(message.clone());
            write_best_effort_stderr_line(&message);
            tokio::time::sleep(Duration::from_millis(100)).await;
            return Ok(exit_code);
        }

        let exit_code = match termination_cause {
            ReplTerminationCause::NormalExit => auto_sync_exit_code,
            ReplTerminationCause::AbnormalTerminalLoss => 1,
        };

        Ok::<_, anyhow::Error>(exit_code)
    })?;

    Ok(exit_code)
}

struct PromptWorker {
    command_tx: UnboundedSender<PromptWorkerCommand>,
    join_handle: Option<thread::JoinHandle<()>>,
    response_rx: Option<UnboundedReceiver<PromptWorkerResponse>>,
    printer: ReplPrinter,
    reedline_printer: Option<ExternalPrinter<String>>,
    kind: PromptWorkerKind,
}

impl PromptWorker {
    fn spawn(config: Arc<ShellConfig>) -> Result<Self> {
        // Tests that intentionally exercise Reedline can force that path even when they also skip
        // shims for bootstrap simplicity. We still require a real terminal below.
        let force_reedline = std::env::var_os("SUBSTRATE_FORCE_REEDLINE").is_some();
        // CI runners often drive Substrate through PTY harnesses like `script` where Reedline's
        // cursor position query can consume the piped input stream. Prefer a plain stdin-backed
        // prompt in CI to keep smoke runs deterministic. Do the same when shims are explicitly
        // skipped, since that mode is primarily used by tests and diagnostic harnesses that
        // emulate a TTY but do not always satisfy Reedline's terminal capability probes in time.
        if config.ci_mode
            || (config.skip_shims && !force_reedline)
            || std::env::var_os("CI").is_some()
            || std::env::var_os("GITHUB_ACTIONS").is_some()
        {
            return Self::spawn_stdio(config);
        }
        if !io::stdin().is_terminal() {
            return Self::spawn_stdio(config);
        }

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (response_tx, response_rx) = mpsc::unbounded_channel();

        let editor::EditorSetup {
            line_editor,
            printer,
        } = editor::build_editor(&config)?;
        let prompt = editor::make_prompt(config.ci_mode);

        let join_handle = thread::spawn(move || {
            run_prompt_worker(line_editor, prompt, command_rx, response_tx);
        });

        Ok(Self {
            command_tx,
            join_handle: Some(join_handle),
            response_rx: Some(response_rx),
            printer: ReplPrinter::Reedline(printer.clone()),
            reedline_printer: Some(printer),
            kind: PromptWorkerKind::Reedline,
        })
    }

    fn spawn_stdio(config: Arc<ShellConfig>) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (response_tx, response_rx) = mpsc::unbounded_channel();
        let prompt = editor::make_prompt(config.ci_mode);

        let join_handle = thread::spawn(move || {
            run_prompt_worker_stdio(prompt, command_rx, response_tx);
        });

        Ok(Self {
            command_tx,
            join_handle: Some(join_handle),
            response_rx: Some(response_rx),
            printer: ReplPrinter::Stdout,
            reedline_printer: None,
            kind: PromptWorkerKind::Stdio,
        })
    }

    fn request_prompt(&self) -> Result<()> {
        self.command_tx
            .send(PromptWorkerCommand::StartPrompt)
            .map_err(|_| anyhow!("prompt worker stopped"))
    }

    fn shutdown(&mut self) {
        self.shutdown_with_disposition(PromptWorkerShutdownDisposition::Graceful);
    }

    fn shutdown_with_disposition(&mut self, disposition: PromptWorkerShutdownDisposition) {
        let _ = self.command_tx.send(PromptWorkerCommand::Shutdown);
        match disposition {
            PromptWorkerShutdownDisposition::Graceful => {
                if let Some(handle) = self.join_handle.take() {
                    let _ = handle.join();
                }
            }
            PromptWorkerShutdownDisposition::Abandon => {
                let _ = self.join_handle.take();
            }
        }
    }

    fn printer_handle(&self) -> ReplPrinter {
        self.printer.clone()
    }

    fn external_printer_handle(&self) -> Option<ExternalPrinter<String>> {
        self.reedline_printer.clone()
    }

    fn is_reedline(&self) -> bool {
        matches!(self.kind, PromptWorkerKind::Reedline)
    }

    fn take_response_receiver(&mut self) -> UnboundedReceiver<PromptWorkerResponse> {
        // UnboundedReceiver doesn't implement Clone, so we move it out by replacing with an empty channel.
        self.response_rx
            .take()
            .expect("response receiver already taken")
    }
}

#[derive(Clone, Copy, Debug)]
enum PromptWorkerKind {
    Reedline,
    Stdio,
}

impl Drop for PromptWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run_prompt_worker(
    mut line_editor: Reedline,
    prompt: editor::SubstratePrompt,
    mut command_rx: UnboundedReceiver<PromptWorkerCommand>,
    response_tx: UnboundedSender<PromptWorkerResponse>,
) {
    while let Some(cmd) = command_rx.blocking_recv() {
        match cmd {
            PromptWorkerCommand::StartPrompt => {
                let resp = match line_editor.read_line(&prompt) {
                    Ok(Signal::Success(line)) => PromptWorkerResponse::Line(line),
                    Ok(Signal::CtrlC) => PromptWorkerResponse::CtrlC,
                    Ok(Signal::CtrlD) => PromptWorkerResponse::CtrlD,
                    Err(err) => PromptWorkerResponse::Error(err.into()),
                };
                if response_tx.send(resp).is_err() {
                    break;
                }
            }
            PromptWorkerCommand::Shutdown => break,
        }
    }
}

fn run_prompt_worker_stdio(
    prompt: editor::SubstratePrompt,
    mut command_rx: UnboundedReceiver<PromptWorkerCommand>,
    response_tx: UnboundedSender<PromptWorkerResponse>,
) {
    #[cfg(unix)]
    struct StdinEchoGuard {
        fd: std::os::fd::RawFd,
        original: libc::termios,
        active: bool,
    }

    #[cfg(unix)]
    impl StdinEchoGuard {
        fn new() -> Self {
            use std::os::fd::AsRawFd;
            let fd = io::stdin().as_raw_fd();

            let mut original: libc::termios = unsafe { std::mem::zeroed() };
            // SAFETY: tcgetattr expects a valid fd and termios pointer.
            let ok = unsafe { libc::tcgetattr(fd, &mut original as *mut libc::termios) } == 0;
            if !ok {
                return Self {
                    fd,
                    original,
                    active: false,
                };
            }

            let mut next = original;
            next.c_lflag &= !(libc::ECHO | libc::ECHONL);
            // SAFETY: tcsetattr expects a valid fd and termios pointer.
            let set_ok =
                unsafe { libc::tcsetattr(fd, libc::TCSANOW, &next as *const libc::termios) } == 0;

            Self {
                fd,
                original,
                active: set_ok,
            }
        }
    }

    #[cfg(unix)]
    impl Drop for StdinEchoGuard {
        fn drop(&mut self) {
            if !self.active {
                return;
            }
            // SAFETY: tcsetattr expects a valid fd and termios pointer.
            let _ = unsafe {
                libc::tcsetattr(
                    self.fd,
                    libc::TCSANOW,
                    &self.original as *const libc::termios,
                )
            };
        }
    }

    #[cfg(unix)]
    let _echo_guard = StdinEchoGuard::new();

    let stdin = io::stdin();

    while let Some(cmd) = command_rx.blocking_recv() {
        match cmd {
            PromptWorkerCommand::StartPrompt => {
                write_locked_stdout(prompt.render_prompt_left().as_bytes());

                let mut line = String::new();
                let read = stdin.read_line(&mut line);
                let resp = match read {
                    Ok(0) => PromptWorkerResponse::CtrlD,
                    Ok(_) => {
                        let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
                        PromptWorkerResponse::Line(trimmed)
                    }
                    Err(err) => PromptWorkerResponse::Error(anyhow!(err)),
                };

                if response_tx.send(resp).is_err() {
                    break;
                }
            }
            PromptWorkerCommand::Shutdown => break,
        }
    }
}

#[derive(Debug)]
enum PromptWorkerCommand {
    StartPrompt,
    Shutdown,
}

#[derive(Debug)]
enum PromptWorkerResponse {
    Line(String),
    CtrlC,
    CtrlD,
    AbnormalTerminalLoss(anyhow::Error),
    Error(anyhow::Error),
}

fn handle_agent_event(
    event: AgentEvent,
    telemetry: &mut ReplSessionTelemetry,
    agent_printer: &ReplPrinter,
) {
    if is_shell_stream_event(&event) {
        return;
    }

    telemetry.persist_agent_event(&event);
    telemetry.record_agent_event();
    agent_printer.print(format_event_line(&event));
}

fn drain_pending_agent_events(
    agent_rx: &mut tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    telemetry: &mut ReplSessionTelemetry,
    agent_printer: &ReplPrinter,
) {
    while let Ok(event) = agent_rx.try_recv() {
        handle_agent_event(event, telemetry, agent_printer);
    }
}

#[derive(Debug)]
struct RuntimeBootstrapFailure {
    exit_code: i32,
    message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TargetedTurn<'a> {
    backend_id: &'a str,
    prompt: &'a str,
}

enum TargetedTurnRoute {
    Host,
    World(RuntimeSelectionDescriptor),
}

enum TargetedTurnDispatchStatus {
    Submitted,
    Rejected(RuntimeBootstrapFailure),
}

struct TargetedTurnDispatchContext<'a> {
    startup_context: &'a mut Option<RuntimeOrchestrationContext>,
    dormant_host_bootstrap: &'a mut Option<ResolvedHostOrchestratorBootstrap>,
    agent_runtime: &'a mut Option<AsyncReplAgentRuntime>,
    world_session: &'a mut Option<WorldSession>,
    member_runtimes: &'a mut RetainedMemberRuntimeMap,
    pending_member_replacements: &'a mut PendingMemberReplacementMap,
    agent_printer: &'a ReplPrinter,
    telemetry: &'a mut ReplSessionTelemetry,
}

#[derive(Clone)]
struct RuntimeOrchestrationContext {
    store: AgentRuntimeStateStore,
    orchestration_session: Arc<Mutex<OrchestrationSessionRecord>>,
    effective_config: crate::execution::config_model::SubstrateConfig,
    #[cfg(any(test, target_os = "linux", target_os = "macos"))]
    base_policy: Policy,
    inventory: BTreeMap<String, AgentInventoryEntryV1>,
}

impl RuntimeOrchestrationContext {
    fn orchestration_session_id(&self) -> String {
        self.orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned")
            .orchestration_session_id
            .clone()
    }

    fn snapshot(&self) -> OrchestrationSessionRecord {
        self.orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned")
            .clone()
    }
}

struct PreparedAgentRuntime {
    descriptor: RuntimeSelectionDescriptor,
    gateway: agent_api::AgentWrapperGateway,
    agent_kind: agent_api::AgentWrapperKind,
    startup_context: RuntimeOrchestrationContext,
    manifest: Arc<Mutex<AgentRuntimeSessionManifest>>,
    run_id: String,
    startup_extensions: BTreeMap<String, serde_json::Value>,
}

struct ResolvedHostOrchestratorBootstrap {
    cwd: PathBuf,
    shell_session_id: String,
    descriptor: RuntimeSelectionDescriptor,
    gateway: agent_api::AgentWrapperGateway,
    agent_kind: agent_api::AgentWrapperKind,
    state_store: AgentRuntimeStateStore,
    effective_config: crate::execution::config_model::SubstrateConfig,
    #[cfg(any(test, target_os = "linux", target_os = "macos"))]
    base_policy: Policy,
    inventory: BTreeMap<String, AgentInventoryEntryV1>,
}

enum RuntimeStartupSignal {
    Running,
    Failed(String),
}

#[derive(Clone, Debug)]
enum InitialExecPromptPlan {
    Replace(String),
    StartupPrompt {
        prompt: String,
        stream_path: PathBuf,
    },
}

#[derive(Clone)]
struct StartupPromptBackchannel {
    envelope_tx: UnboundedSender<PublicPromptEnvelope>,
    terminal_sent: Arc<AtomicBool>,
}

impl StartupPromptBackchannel {
    fn send(&self, envelope: PublicPromptEnvelope) {
        if self.terminal_sent.load(Ordering::SeqCst) {
            return;
        }
        let terminal = matches!(
            envelope,
            PublicPromptEnvelope::Completed { .. } | PublicPromptEnvelope::Failed { .. }
        );
        if terminal {
            self.terminal_sent.store(true, Ordering::SeqCst);
        }
        let _ = self.envelope_tx.send(envelope);
    }

    fn send_event(&self, event: &AgentEvent) {
        self.send(PublicPromptEnvelope::Event {
            version: 1,
            event_kind: "message".to_string(),
            data: serde_json::to_value(event).unwrap_or_default(),
        });
    }
}

const AGENT_API_NO_TURN_SESSION_START_V1: &str = "agent_api.session.start.no_turn.v1";

fn runtime_supports_no_turn_session_start(
    gateway: &agent_api::AgentWrapperGateway,
    agent_kind: &agent_api::AgentWrapperKind,
) -> bool {
    gateway
        .backend(agent_kind)
        .map(|backend| {
            backend
                .capabilities()
                .contains(AGENT_API_NO_TURN_SESSION_START_V1)
        })
        .unwrap_or(false)
}

fn should_eager_bootstrap_host_orchestrator(resolved: &ResolvedHostOrchestratorBootstrap) -> bool {
    runtime_supports_no_turn_session_start(&resolved.gateway, &resolved.agent_kind)
}

// The REPL retains live UAA runtime ownership via the cancel handle plus the two
// long-lived tasks that own the non-clonable `run_control.handle` facets. A manifest
// may only advertise a live orchestrator session while all three remain retained.
struct LocalRetainedRunControl {
    cancel: agent_api::AgentWrapperCancelHandle,
    event_task: Option<tokio::task::JoinHandle<()>>,
    completion_task: Option<tokio::task::JoinHandle<()>>,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct RemoteRetainedRunControl {
    client: Arc<AgentClient>,
    span_id: String,
    observe_task: Option<tokio::task::JoinHandle<()>>,
}

enum RetainedRunControl {
    Local(LocalRetainedRunControl),
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Remote(RemoteRetainedRunControl),
}

const LOCAL_RETAINED_STOP_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);

struct AsyncReplAgentRuntime {
    descriptor: RuntimeSelectionDescriptor,
    orchestration_session: Arc<Mutex<OrchestrationSessionRecord>>,
    manifest: Arc<Mutex<AgentRuntimeSessionManifest>>,
    store: AgentRuntimeStateStore,
    uaa_session_handle_id: String,
    retained_control: RetainedRunControl,
    shutdown_requested: Arc<AtomicBool>,
    private_stop_rx: Option<PrivateStopRequestReceiver>,
    stop_transport: Option<PrivateStopTransport>,
    stop_owner_task: Option<tokio::task::JoinHandle<()>>,
    prompt_transport: Option<PrivatePromptTransport>,
    prompt_owner_task: Option<tokio::task::JoinHandle<()>>,
    heartbeat_stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
    heartbeat_task: Option<tokio::task::JoinHandle<()>>,
}

type RetainedMemberRuntimeMap = BTreeMap<String, AsyncReplAgentRuntime>;
type PendingMemberReplacementMap = BTreeMap<String, AgentRuntimeParticipantRecord>;

fn runtime_registered_message(role: &str) -> &'static str {
    if role == MEMBER_ROLE {
        "world-scoped member session handle allocated"
    } else {
        "shell-owned orchestrator session handle allocated"
    }
}

fn runtime_task_start_message(role: &str) -> &'static str {
    if role == MEMBER_ROLE {
        "starting long-lived world-scoped member control turn"
    } else {
        "starting long-lived shell-owned orchestrator control turn"
    }
}

fn runtime_bootstrap_prompt(role: &str) -> &'static str {
    if role == MEMBER_ROLE {
        "Enter persistent Substrate world-scoped member mode. Keep this control session attached for the lifetime of the parent REPL session and do not exit until the client cancels the run."
    } else {
        "Enter persistent Substrate host orchestrator mode. Keep this control session attached for the lifetime of the host REPL and do not exit until the client cancels the run."
    }
}

fn runtime_ready_message(role: &str) -> &'static str {
    if role == MEMBER_ROLE {
        "world-scoped member session is ready via retained attached control ownership"
    } else {
        "shell-owned orchestrator session is ready via retained attached control ownership"
    }
}

#[derive(Debug, Deserialize)]
struct TurnLifecycleEventV1 {
    schema: String,
    turn: TurnLifecyclePayloadV1,
}

#[derive(Debug, Deserialize)]
struct TurnLifecyclePayloadV1 {
    thread_id: String,
    turn_id: String,
    phase: String,
}

fn turn_lifecycle_phase(data: Option<&serde_json::Value>) -> Option<&str> {
    let value = data?;
    let parsed: TurnLifecycleEventV1 = serde_json::from_value(value.clone()).ok()?;
    if parsed.schema != AGENT_API_TURN_LIFECYCLE_V1 {
        return None;
    }
    let _ = (&parsed.turn.thread_id, &parsed.turn.turn_id);
    Some(match parsed.turn.phase.as_str() {
        "completed" => "completed",
        "failed" => "failed",
        _ => return None,
    })
}

fn infer_startup_turn_phase_fallback(
    event: &agent_api::AgentWrapperEvent,
    saw_substantive_output: bool,
) -> Option<&'static str> {
    match event.kind {
        agent_api::AgentWrapperEventKind::Status => {
            match event
                .message
                .as_deref()
                .map(str::trim)
                .filter(|message| !message.is_empty())
            {
                Some("turn failed") => Some("failed"),
                Some(_) => None,
                None if event.data.is_none() && saw_substantive_output => Some("completed"),
                None => None,
            }
        }
        _ => None,
    }
}

#[cfg(unix)]
async fn connect_startup_prompt_backchannel(
    stream_path: &Path,
) -> std::result::Result<StartupPromptBackchannel, RuntimeBootstrapFailure> {
    use tokio::io::AsyncWriteExt;
    use tokio::net::UnixStream;

    let stream = UnixStream::connect(stream_path)
        .await
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 4,
            message: format!(
                "failed to connect hidden owner-helper startup prompt stream {}: {err}",
                stream_path.display()
            ),
        })?;
    let (envelope_tx, mut envelope_rx) = mpsc::unbounded_channel::<PublicPromptEnvelope>();
    let terminal_sent = Arc::new(AtomicBool::new(false));
    let terminal_sent_for_task = Arc::clone(&terminal_sent);
    tokio::spawn(async move {
        let mut stream = stream;
        while let Some(envelope) = envelope_rx.recv().await {
            if stream
                .write_all(
                    format!(
                        "{}\n",
                        serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string())
                    )
                    .as_bytes(),
                )
                .await
                .is_err()
            {
                break;
            }
            let _ = stream.flush().await;
            if matches!(
                envelope,
                PublicPromptEnvelope::Completed { .. } | PublicPromptEnvelope::Failed { .. }
            ) {
                break;
            }
        }
        terminal_sent_for_task.store(true, Ordering::SeqCst);
    });

    Ok(StartupPromptBackchannel {
        envelope_tx,
        terminal_sent,
    })
}

#[cfg(not(unix))]
async fn connect_startup_prompt_backchannel(
    _stream_path: &Path,
) -> std::result::Result<StartupPromptBackchannel, RuntimeBootstrapFailure> {
    Err(RuntimeBootstrapFailure {
        exit_code: 4,
        message: "hidden owner-helper startup prompt streaming requires Unix transports"
            .to_string(),
    })
}

fn startup_prompt_state_label(state: &AgentRuntimeSessionState) -> &'static str {
    match state {
        AgentRuntimeSessionState::Allocating
        | AgentRuntimeSessionState::Ready
        | AgentRuntimeSessionState::Running
        | AgentRuntimeSessionState::Restarting
        | AgentRuntimeSessionState::Stopping => "active",
        AgentRuntimeSessionState::Stopped => "stopped",
        AgentRuntimeSessionState::Failed => "failed",
        AgentRuntimeSessionState::Invalidated => "invalidated",
    }
}

fn startup_prompt_posture(
    session: &OrchestrationSessionRecord,
    manifest: &AgentRuntimeSessionManifest,
) -> PublicSessionPosture {
    if session.state == OrchestrationSessionState::Active
        && manifest.is_authoritative_live()
        && session.attached_participant_id() == Some(manifest.handle.participant_id.as_str())
    {
        PublicSessionPosture::Active
    } else {
        PublicSessionPosture::Terminal
    }
}

fn startup_prompt_completed_envelope(
    session: &OrchestrationSessionRecord,
    manifest: &AgentRuntimeSessionManifest,
) -> PublicPromptEnvelope {
    PublicPromptEnvelope::Completed {
        version: 1,
        action: PublicPromptAction::Start,
        orchestration_session_id: session.orchestration_session_id.clone(),
        backend_id: manifest.handle.backend_id.clone(),
        participant_id: Some(manifest.handle.participant_id.clone()),
        turn_outcome: "success".to_string(),
        session_posture: startup_prompt_posture(session, manifest),
        state: startup_prompt_state_label(&manifest.handle.state).to_string(),
        warnings: Vec::new(),
    }
}

fn startup_prompt_failed_envelope(message: impl Into<String>) -> PublicPromptEnvelope {
    PublicPromptEnvelope::Failed {
        version: 1,
        terminal: true,
        stage: "runtime".to_string(),
        error_code: "owner_unreachable".to_string(),
        message: message.into(),
    }
}

fn runtime_stopping_message(role: &str, uaa_session_handle_id: &str) -> String {
    if role == MEMBER_ROLE {
        format!(
            "world-scoped member session stopping (uaa_session_handle_id={uaa_session_handle_id})"
        )
    } else {
        format!("shell-owned orchestrator session stopping (uaa_session_handle_id={uaa_session_handle_id})")
    }
}

fn runtime_stopped_message(role: &str) -> &'static str {
    if role == MEMBER_ROLE {
        "world-scoped member session stopped"
    } else {
        "shell-owned orchestrator session stopped"
    }
}

fn runtime_detached_message(role: &str) -> &'static str {
    if role == MEMBER_ROLE {
        "world-scoped member session detached cleanly"
    } else {
        "shell-owned orchestrator detached cleanly; session parked for resume"
    }
}

fn runtime_stream_closed_alert_code(role: &str) -> &'static str {
    if role == MEMBER_ROLE {
        "member_runtime_stream_closed"
    } else {
        "orchestrator_runtime_stream_closed"
    }
}

fn runtime_invalidated_alert_code(role: &str) -> &'static str {
    if role == MEMBER_ROLE {
        "member_runtime_invalidated"
    } else {
        "orchestrator_runtime_invalidated"
    }
}

fn build_repl_shell_event_emission_context(
    startup_context: Option<&RuntimeOrchestrationContext>,
    agent_runtime: Option<&AsyncReplAgentRuntime>,
    world_session: Option<&WorldSession>,
) -> Option<ShellEventEmissionContext> {
    let startup_context = startup_context?;
    let agent_runtime = agent_runtime?;
    let orchestration_snapshot = startup_context.snapshot();
    let manifest_snapshot = agent_runtime
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .clone();

    Some(ShellEventEmissionContext {
        orchestration_session_id: orchestration_snapshot.orchestration_session_id,
        agent_id: "shell".to_string(),
        role: Some("orchestrator".to_string()),
        backend_id: Some("shell:repl".to_string()),
        participant_id: Some(manifest_snapshot.handle.participant_id),
        parent_participant_id: manifest_snapshot.handle.parent_participant_id,
        resumed_from_participant_id: manifest_snapshot.handle.resumed_from_participant_id,
        world_id: world_session
            .map(|session| session.world_id.clone())
            .or(orchestration_snapshot.world_id),
        world_generation: world_session
            .map(|session| session.world_generation)
            .or(orchestration_snapshot.world_generation),
    })
}

fn build_repl_shell_command_event_context(
    startup_context: Option<&RuntimeOrchestrationContext>,
    agent_runtime: Option<&AsyncReplAgentRuntime>,
    world_session: Option<&WorldSession>,
    cmd_id: &str,
) -> Option<ShellCommandEventContext> {
    let emission =
        build_repl_shell_event_emission_context(startup_context, agent_runtime, world_session)?;
    Some(ShellCommandEventContext::new(
        emission,
        cmd_id,
        Some(cmd_id.to_string()),
        None,
    ))
}

#[allow(dead_code)]
async fn start_host_orchestrator_runtime(
    config: &Arc<ShellConfig>,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> std::result::Result<Option<AsyncReplAgentRuntime>, RuntimeBootstrapFailure> {
    let prepared = prepare_host_orchestrator_runtime_startup(config)?;
    start_host_orchestrator_runtime_with_prepared(prepared, None, agent_printer, telemetry).await
}

fn prepare_host_orchestrator_runtime_startup(
    config: &Arc<ShellConfig>,
) -> std::result::Result<Option<PreparedAgentRuntime>, RuntimeBootstrapFailure> {
    let Some(resolved) = resolve_host_orchestrator_bootstrap(config)? else {
        return Ok(None);
    };
    Ok(Some(prepare_host_orchestrator_runtime_from_resolved(
        resolved,
    )?))
}

fn prepare_host_orchestrator_runtime_from_resolved(
    resolved: ResolvedHostOrchestratorBootstrap,
) -> std::result::Result<PreparedAgentRuntime, RuntimeBootstrapFailure> {
    let ResolvedHostOrchestratorBootstrap {
        cwd,
        shell_session_id,
        descriptor,
        gateway,
        agent_kind,
        state_store,
        effective_config,
        #[cfg(any(test, target_os = "linux", target_os = "macos"))]
        base_policy,
        inventory,
    } = resolved;

    let participant_id = format!("ash_{}", Uuid::now_v7());
    let lease_token = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let orchestration_session_id = Uuid::now_v7().to_string();
    let mut manifest = AgentRuntimeSessionManifest::new(
        &descriptor,
        orchestration_session_id.clone(),
        participant_id,
        lease_token,
    );
    manifest.internal.latest_run_id = Some(run_id.clone());
    let orchestration_session = Arc::new(Mutex::new(OrchestrationSessionRecord::new(
        orchestration_session_id,
        shell_session_id,
        cwd.display().to_string(),
        &manifest,
    )));
    state_store
        .persist_orchestration_session(
            &orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned"),
        )
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to persist orchestration session record: {err:#}"),
        })?;

    Ok(PreparedAgentRuntime {
        descriptor,
        gateway,
        agent_kind,
        startup_context: RuntimeOrchestrationContext {
            store: state_store,
            orchestration_session,
            effective_config,
            #[cfg(any(test, target_os = "linux", target_os = "macos"))]
            base_policy,
            inventory,
        },
        manifest: Arc::new(Mutex::new(manifest)),
        run_id,
        startup_extensions: BTreeMap::new(),
    })
}

fn resolve_host_orchestrator_bootstrap(
    config: &Arc<ShellConfig>,
) -> std::result::Result<Option<ResolvedHostOrchestratorBootstrap>, RuntimeBootstrapFailure> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let effective_config = crate::execution::config_model::resolve_effective_config(
        &cwd,
        &crate::execution::config_model::CliConfigOverrides {
            world_enabled: if config.cli_world {
                Some(true)
            } else if config.cli_no_world {
                Some(false)
            } else {
                None
            },
            anchor_mode: config.cli_anchor_mode,
            anchor_path: config
                .cli_anchor_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            caged: config.cli_caged,
        },
    )
    .map_err(runtime_bootstrap_failure_from_anyhow)?;
    if !effective_config.agents.enabled {
        return Ok(None);
    }

    let (base_policy, _) = substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 2,
            message: err.to_string(),
        })?;
    let inventory = load_effective_agent_inventory(&cwd, &base_policy)
        .map_err(runtime_bootstrap_failure_from_anyhow)?;
    let orchestrator =
        validate_orchestrator_selection(&effective_config, &inventory).map_err(|reason| {
            RuntimeBootstrapFailure {
                exit_code: 2,
                message: reason,
            }
        })?;

    if !backend_allowed(&base_policy, &orchestrator.derived_backend_id()) {
        return Err(RuntimeBootstrapFailure {
            exit_code: 5,
            message: format!(
                "selected orchestrator backend '{}' is not allowlisted by effective policy agents.allowed_backends",
                orchestrator.derived_backend_id()
            ),
        });
    }

    let descriptor =
        validate_runtime_realizability(orchestrator, &effective_config).map_err(|err| {
            RuntimeBootstrapFailure {
                exit_code: runtime_realizability_error_exit_code(&err),
                message: err.reason,
            }
        })?;
    let state_store = AgentRuntimeStateStore::new().map_err(|err| RuntimeBootstrapFailure {
        exit_code: 1,
        message: format!("failed to initialize shell runtime state store: {err:#}"),
    })?;
    let gateway =
        build_gateway_for_descriptor(&descriptor).map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to build shell-owned UAA runtime registry: {err:#}"),
        })?;
    let agent_kind = agent_api::AgentWrapperKind::new(descriptor.backend_kind.as_agent_kind_str())
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 2,
            message: format!("failed to resolve runtime backend kind: {err}"),
        })?;

    Ok(Some(ResolvedHostOrchestratorBootstrap {
        cwd,
        shell_session_id: config.session_id.clone(),
        descriptor,
        gateway,
        agent_kind,
        state_store,
        effective_config,
        #[cfg(any(test, target_os = "linux", target_os = "macos"))]
        base_policy,
        inventory,
    }))
}

fn owner_helper_shell_config(plan: &HiddenOwnerHelperLaunchPlan) -> Result<ShellConfig> {
    let cwd =
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(&plan.session.workspace_root));
    let substrate_home = substrate_paths::substrate_home()?;
    let trace_log_file = env::var("SHIM_TRACE_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| substrate_home.join("trace.jsonl"));

    Ok(ShellConfig {
        mode: crate::execution::ShellMode::Interactive { use_pty: false },
        session_id: plan.session.shell_trace_session_id.clone(),
        trace_log_file,
        original_path: env::var("PATH").unwrap_or_default(),
        shim_dir: substrate_home.join("shim-bin"),
        shell_path: env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
        ci_mode: false,
        no_exit_on_error: false,
        skip_shims: true,
        no_world: false,
        cli_world: false,
        cli_no_world: false,
        cli_anchor_mode: None,
        cli_anchor_path: None,
        cli_caged: Some(true),
        world_root: WorldRootSettings {
            mode: WorldRootMode::Project,
            path: cwd,
            caged: true,
        },
        async_repl: false,
        repl_host_escape: false,
        env_vars: HashMap::new(),
        manager_init_path: substrate_home.join("manager_init.sh"),
        manager_env_path: substrate_home.join("manager_env.sh"),
        shimmed_path: None,
        host_bash_env: None,
        bash_preexec_path: substrate_home.join(".substrate_preexec"),
        preexec_available: false,
    })
}

fn owner_helper_runtime_descriptor(
    descriptor: &ResolvedRuntimeDescriptor,
) -> std::result::Result<RuntimeSelectionDescriptor, RuntimeBootstrapFailure> {
    descriptor
        .try_into()
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to decode hidden owner-helper runtime descriptor: {err:#}"),
        })
}

fn owner_helper_startup_extensions(
    plan: &HiddenOwnerHelperLaunchPlan,
) -> std::result::Result<BTreeMap<String, serde_json::Value>, RuntimeBootstrapFailure> {
    match plan.mode {
        OwnerHelperMode::Start => Ok(BTreeMap::new()),
        OwnerHelperMode::Resume | OwnerHelperMode::Fork => {
            let session_id = plan
                .participant
                .internal_uaa_session_id
                .as_deref()
                .ok_or_else(|| RuntimeBootstrapFailure {
                    exit_code: 1,
                    message: format!(
                        "hidden owner-helper {} requires internal.uaa_session_id",
                        plan.mode.as_str()
                    ),
                })?;
            Ok(BTreeMap::from([(
                AGENT_API_SESSION_RESUME_V1.to_string(),
                build_session_resume_extension(session_id),
            )]))
        }
    }
}

fn owner_helper_manifest(
    descriptor: &RuntimeSelectionDescriptor,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> std::result::Result<AgentRuntimeSessionManifest, RuntimeBootstrapFailure> {
    let participant_id = plan.participant.participant_id.clone();
    let lease_token = plan.participant.lease_token.clone();
    let manifest = match plan.mode {
        OwnerHelperMode::Start => AgentRuntimeSessionManifest::new_orchestrator_participant(
            descriptor,
            plan.session.orchestration_session_id.clone(),
            participant_id,
            lease_token,
        ),
        OwnerHelperMode::Resume | OwnerHelperMode::Fork => {
            let resumed_from_participant_id = plan
                .participant
                .resumed_from_participant_id
                .clone()
                .ok_or_else(|| RuntimeBootstrapFailure {
                    exit_code: 1,
                    message: format!(
                        "hidden owner-helper {} requires resumed_from_participant_id",
                        plan.mode.as_str()
                    ),
                })?;
            AgentRuntimeSessionManifest::new_replacement_participant(
                descriptor,
                AgentRuntimeReplacementParticipantInit {
                    orchestration_session_id: plan.session.orchestration_session_id.clone(),
                    participant_id,
                    role: ORCHESTRATOR_ROLE.to_string(),
                    orchestrator_participant_id: None,
                    parent_participant_id: None,
                    resumed_from_participant_id,
                    world: None,
                    lease_token,
                },
            )
        }
    }
    .map_err(|err| RuntimeBootstrapFailure {
        exit_code: 1,
        message: format!("failed to construct hidden owner-helper manifest: {err:#}"),
    })?;

    let mut manifest = manifest;
    manifest.internal.latest_run_id = Some(plan.participant.run_id.clone());
    Ok(manifest)
}

fn owner_helper_orchestration_session(
    plan: &HiddenOwnerHelperLaunchPlan,
    manifest: &AgentRuntimeSessionManifest,
) -> OrchestrationSessionRecord {
    let mut session = OrchestrationSessionRecord::new(
        plan.session.orchestration_session_id.clone(),
        plan.session.shell_trace_session_id.clone(),
        plan.session.workspace_root.clone(),
        manifest,
    );
    session.latest_run_id = Some(plan.participant.run_id.clone());
    session.active_session_handle_id = None;
    session.shell_owner_pid = std::process::id();
    if let (Some(world_id), Some(world_generation)) =
        (&plan.session.world_id, plan.session.world_generation)
    {
        session.world_id = Some(world_id.clone());
        session.world_generation = Some(world_generation);
    }
    if plan.startup_prompt.is_some() {
        session.initialize_startup_prompt(manifest.handle.participant_id.clone());
    }
    session
}

fn prepare_hidden_owner_helper_runtime(
    config: &Arc<ShellConfig>,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> std::result::Result<PreparedAgentRuntime, RuntimeBootstrapFailure> {
    let Some(resolved) = resolve_host_orchestrator_bootstrap(config)? else {
        return Err(RuntimeBootstrapFailure {
            exit_code: 2,
            message: "hidden owner-helper requires agents.enabled=true".to_string(),
        });
    };
    let ResolvedHostOrchestratorBootstrap {
        cwd: _cwd,
        shell_session_id: _shell_session_id,
        descriptor: validated_descriptor,
        gateway: _validated_gateway,
        agent_kind: _validated_agent_kind,
        state_store,
        effective_config,
        #[cfg(any(test, target_os = "linux", target_os = "macos"))]
        base_policy,
        inventory,
    } = resolved;
    let descriptor = owner_helper_runtime_descriptor(&plan.descriptor)?;
    if validated_descriptor.backend_id != descriptor.backend_id {
        return Err(RuntimeBootstrapFailure {
            exit_code: 2,
            message: format!(
                "hidden owner-helper descriptor mismatch: validated backend '{}' does not match resolved backend '{}'",
                validated_descriptor.backend_id, descriptor.backend_id
            ),
        });
    }

    let manifest = owner_helper_manifest(&descriptor, plan)?;
    let orchestration_session = Arc::new(Mutex::new(owner_helper_orchestration_session(
        plan, &manifest,
    )));
    state_store
        .persist_orchestration_session(
            &orchestration_session
                .lock()
                .expect("hidden owner-helper orchestration session mutex poisoned"),
        )
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to persist hidden owner-helper session record: {err:#}"),
        })?;

    let gateway =
        build_gateway_for_descriptor(&descriptor).map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to build hidden owner-helper gateway: {err:#}"),
        })?;
    let agent_kind = agent_api::AgentWrapperKind::new(descriptor.backend_kind.as_agent_kind_str())
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 2,
            message: format!("failed to resolve hidden owner-helper backend kind: {err}"),
        })?;
    Ok(PreparedAgentRuntime {
        descriptor,
        gateway,
        agent_kind,
        startup_context: RuntimeOrchestrationContext {
            store: state_store,
            orchestration_session,
            effective_config,
            #[cfg(any(test, target_os = "linux", target_os = "macos"))]
            base_policy,
            inventory,
        },
        manifest: Arc::new(Mutex::new(manifest)),
        run_id: plan.participant.run_id.clone(),
        startup_extensions: owner_helper_startup_extensions(plan)?,
    })
}

async fn wait_for_hidden_owner_helper_completion(runtime: AsyncReplAgentRuntime) -> Result<i32> {
    let AsyncReplAgentRuntime {
        descriptor: _,
        orchestration_session,
        manifest,
        store,
        uaa_session_handle_id: _,
        mut retained_control,
        shutdown_requested,
        private_stop_rx,
        mut stop_transport,
        mut stop_owner_task,
        mut prompt_transport,
        mut prompt_owner_task,
        mut heartbeat_stop_tx,
        mut heartbeat_task,
    } = runtime;

    let mut join_failed = false;

    match &mut retained_control {
        RetainedRunControl::Local(retained_control) => {
            if let Some(stop_rx) = private_stop_rx {
                join_failed |= wait_for_hidden_owner_helper_local_runtime(
                    &store,
                    &orchestration_session,
                    &manifest,
                    &shutdown_requested,
                    retained_control,
                    stop_rx,
                    &mut stop_transport,
                    &mut prompt_transport,
                    &mut prompt_owner_task,
                    &mut heartbeat_stop_tx,
                    &mut heartbeat_task,
                )
                .await;
            } else {
                if let Some(task) = retained_control.completion_task.take() {
                    join_failed |= task.await.is_err();
                }
                if let Some(task) = retained_control.event_task.take() {
                    let _ = task.await;
                }
            }
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        RetainedRunControl::Remote(retained_control) => {
            if let Some(task) = retained_control.observe_task.take() {
                join_failed |= task.await.is_err();
            }
        }
    }

    if let Some(mut stop_transport) = stop_transport.take() {
        stop_transport.close().await;
    }
    if let Some(task) = stop_owner_task.take() {
        let _ = task.await;
    }
    if let Some(mut prompt_transport) = prompt_transport.take() {
        prompt_transport.close().await;
    }
    if let Some(task) = prompt_owner_task.take() {
        let _ = task.await;
    }
    if let Some(stop_tx) = heartbeat_stop_tx.take() {
        let _ = stop_tx.send(());
    }
    if let Some(task) = heartbeat_task.take() {
        let _ = task.await;
    }

    let manifest = manifest
        .lock()
        .expect("hidden owner-helper manifest mutex poisoned")
        .clone();
    Ok(
        if join_failed || manifest.handle.state == AgentRuntimeSessionState::Failed {
            1
        } else {
            0
        },
    )
}

async fn wait_for_hidden_owner_helper_local_runtime(
    store: &AgentRuntimeStateStore,
    orchestration_session: &Arc<Mutex<OrchestrationSessionRecord>>,
    manifest: &Arc<Mutex<AgentRuntimeSessionManifest>>,
    shutdown_requested: &Arc<AtomicBool>,
    retained_control: &mut LocalRetainedRunControl,
    mut stop_rx: PrivateStopRequestReceiver,
    stop_transport: &mut Option<PrivateStopTransport>,
    prompt_transport: &mut Option<PrivatePromptTransport>,
    prompt_owner_task: &mut Option<tokio::task::JoinHandle<()>>,
    heartbeat_stop_tx: &mut Option<tokio::sync::oneshot::Sender<()>>,
    heartbeat_task: &mut Option<tokio::task::JoinHandle<()>>,
) -> bool {
    let mut join_failed = false;
    let Some(mut completion_task) = retained_control.completion_task.take() else {
        return true;
    };

    let accepted_stop = tokio::select! {
        result = &mut completion_task => {
            join_failed |= result.is_err();
            false
        }
        maybe_request = stop_rx.recv() => {
            let Some(request) = maybe_request else {
                join_failed |= completion_task.await.is_err();
                return join_failed;
            };

            if runtime_is_terminal(manifest) {
                let _ = request.response_tx.send(PrivateStopOutcome::AlreadyTerminal);
                join_failed |= completion_task.await.is_err();
                return join_failed;
            }

            if let Err(err) = note_runtime_stop_requested(store, orchestration_session, manifest) {
                persist_hidden_owner_helper_stop_failure(
                    store,
                    orchestration_session,
                    manifest,
                    format!("failed to persist stop request before shutdown: {err:#}"),
                );
                let _ = request.response_tx.send(PrivateStopOutcome::ProtocolError);
                shutdown_requested.store(true, Ordering::SeqCst);
                retained_control.cancel.cancel();
                join_failed |= completion_task.await.is_err();
                return join_failed;
            }

            let _ = request.response_tx.send(PrivateStopOutcome::Accepted);
            true
        }
    };

    if accepted_stop {
        if let Some(mut owned_stop_transport) = stop_transport.take() {
            owned_stop_transport.close().await;
        }
        if let Some(mut owned_prompt_transport) = prompt_transport.take() {
            owned_prompt_transport.close().await;
        }
        if let Some(task) = prompt_owner_task.take() {
            let _ = task.await;
        }
        shutdown_requested.store(true, Ordering::SeqCst);
        if let Some(stop_tx) = heartbeat_stop_tx.take() {
            let _ = stop_tx.send(());
        }
        retained_control.cancel.cancel();

        let mut completion_observed = false;
        let mut stop_failed = false;
        tokio::select! {
            result = &mut completion_task => {
                completion_observed = true;
                join_failed |= result.is_err();
                stop_failed |= result.is_err();
            }
            _ = tokio::time::sleep(LOCAL_RETAINED_STOP_COMPLETION_TIMEOUT) => {
                completion_task.abort();
                let _ = completion_task.await;
                stop_failed = true;
            }
        }

        if let Some(task) = retained_control.event_task.take() {
            match tokio::time::timeout(LOCAL_RETAINED_STOP_COMPLETION_TIMEOUT, task).await {
                Ok(_) => {}
                Err(_) => join_failed = true,
            }
        }
        if let Some(task) = heartbeat_task.take() {
            let _ = task.await;
        }

        if stop_failed || !completion_observed {
            persist_hidden_owner_helper_stop_failure(
                store,
                orchestration_session,
                manifest,
                "hidden owner-helper stop did not produce authoritative terminal completion"
                    .to_string(),
            );
        }
    } else {
        if let Some(task) = retained_control.event_task.take() {
            let _ = task.await;
        }
    }

    join_failed
}

fn persist_hidden_owner_helper_stop_failure(
    store: &AgentRuntimeStateStore,
    orchestration_session: &Arc<Mutex<OrchestrationSessionRecord>>,
    manifest: &Arc<Mutex<AgentRuntimeSessionManifest>>,
    reason: String,
) {
    let (orchestration_snapshot, manifest_snapshot) = {
        let mut orchestration_guard = orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        let mut manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");

        if !manifest_guard.handle.state.is_live() {
            return;
        }

        manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
        manifest_guard.mark_terminal_state(reason.clone());
        manifest_guard.internal.last_error_bucket = Some("runtime_shutdown".to_string());
        manifest_guard.internal.last_error_message = Some(reason.clone());
        if runtime_controls_parent_session(&manifest_guard.handle.role) {
            orchestration_guard.transition_state(OrchestrationSessionState::Failed);
            orchestration_guard.mark_terminal(reason);
        } else {
            orchestration_guard.touch_active();
        }
        (orchestration_guard.clone(), manifest_guard.clone())
    };

    let _ = persist_runtime_snapshots(store, &orchestration_snapshot, &manifest_snapshot);
}

pub(crate) fn run_hidden_owner_helper(plan: HiddenOwnerHelperLaunchPlan) -> Result<i32> {
    let config = Arc::new(owner_helper_shell_config(&plan)?);
    let running_child_pid = Arc::new(AtomicI32::new(0));
    setup_signal_handlers(running_child_pid)?;
    let rt = TokioRuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize hidden owner-helper runtime")?;

    rt.block_on(async move {
        let mut telemetry = ReplSessionTelemetry::new(config.clone(), "agent_owner_helper");
        let prepared = prepare_hidden_owner_helper_runtime(&config, &plan)
            .map_err(|failure| anyhow!(failure.message))?;
        let initial_prompt = plan.startup_prompt.as_ref().map(|startup_prompt| {
            InitialExecPromptPlan::StartupPrompt {
                prompt: startup_prompt.prompt_text.clone(),
                stream_path: startup_prompt.stream_path.clone(),
            }
        });
        let runtime = start_host_orchestrator_runtime_with_prepared_prompt(
            Some(prepared),
            None,
            initial_prompt,
            true,
            &ReplPrinter::Stdout,
            &mut telemetry,
        )
        .await
        .map_err(|failure| anyhow!(failure.message))?;
        let Some(runtime) = runtime else {
            return Ok(0);
        };
        wait_for_hidden_owner_helper_completion(runtime).await
    })
}

async fn start_host_orchestrator_runtime_with_prepared(
    prepared: Option<PreparedAgentRuntime>,
    initial_world_binding: Option<&PersistedWorldBinding>,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> std::result::Result<Option<AsyncReplAgentRuntime>, RuntimeBootstrapFailure> {
    start_host_orchestrator_runtime_with_prepared_prompt(
        prepared,
        initial_world_binding,
        None,
        false,
        agent_printer,
        telemetry,
    )
    .await
}

async fn start_host_orchestrator_runtime_with_prepared_prompt(
    prepared: Option<PreparedAgentRuntime>,
    initial_world_binding: Option<&PersistedWorldBinding>,
    initial_prompt: Option<InitialExecPromptPlan>,
    runtime_owns_private_stop: bool,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> std::result::Result<Option<AsyncReplAgentRuntime>, RuntimeBootstrapFailure> {
    let Some(prepared) = prepared else {
        return Ok(None);
    };
    let PreparedAgentRuntime {
        descriptor,
        gateway,
        agent_kind,
        startup_context,
        manifest,
        run_id,
        startup_extensions,
    } = prepared;
    let runtime_role = {
        manifest
            .lock()
            .expect("runtime manifest mutex poisoned")
            .handle
            .role
            .clone()
    };
    let controls_parent_session = runtime_controls_parent_session(&runtime_role);
    if let Err(err) = persist_world_binding_authority(
        &startup_context.store,
        &startup_context.orchestration_session,
        initial_world_binding,
    ) {
        mark_orchestration_session_failed(
            &startup_context.store,
            &startup_context.orchestration_session,
            format!("failed to persist startup world binding: {err:#}"),
        );
        return Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to persist startup world binding: {err:#}"),
        });
    }
    let persist_participant_result = {
        let manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
        startup_context.store.persist_participant(&manifest_guard)
    };
    if let Err(err) = persist_participant_result {
        mark_orchestration_session_failed(
            &startup_context.store,
            &startup_context.orchestration_session,
            format!("failed to persist agent runtime participant record: {err:#}"),
        );
        return Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to persist agent runtime participant record: {err:#}"),
        });
    }
    let orchestration_snapshot = startup_context
        .orchestration_session
        .lock()
        .expect("orchestration session mutex poisoned")
        .clone();
    let manifest_snapshot = manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .clone();
    emit_runtime_event(
        build_runtime_message_event(
            &manifest_snapshot,
            &orchestration_snapshot,
            run_id.clone(),
            MessageEventKind::Registered,
            runtime_registered_message(&runtime_role),
        ),
        telemetry,
        agent_printer,
    );
    emit_runtime_event(
        build_runtime_message_event(
            &manifest_snapshot,
            &orchestration_snapshot,
            run_id.clone(),
            MessageEventKind::TaskStart,
            runtime_task_start_message(&runtime_role),
        ),
        telemetry,
        agent_printer,
    );

    let request = agent_api::AgentWrapperRunRequest {
        prompt: match initial_prompt.as_ref() {
            Some(InitialExecPromptPlan::Replace(prompt)) => prompt.clone(),
            Some(InitialExecPromptPlan::StartupPrompt { prompt, .. }) => prompt.clone(),
            None => runtime_bootstrap_prompt(&runtime_role).to_string(),
        },
        working_dir: Some(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        timeout: None,
        env: BTreeMap::new(),
        extensions: startup_extensions,
    };
    let startup_backchannel = match initial_prompt.as_ref() {
        Some(InitialExecPromptPlan::StartupPrompt { stream_path, .. }) => {
            Some(connect_startup_prompt_backchannel(stream_path).await?)
        }
        _ => None,
    };
    if let Some(backchannel) = startup_backchannel.as_ref() {
        let (orchestration_snapshot, manifest_snapshot) = {
            let mut orchestration_guard = startup_context
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            let manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
            orchestration_guard
                .mark_startup_prompt_accepted(manifest_guard.handle.participant_id.as_str());
            (orchestration_guard.clone(), manifest_guard.clone())
        };
        persist_runtime_snapshots(
            &startup_context.store,
            &orchestration_snapshot,
            &manifest_snapshot,
        )
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to persist startup prompt acceptance: {err:#}"),
        })?;
        backchannel.send(PublicPromptEnvelope::Accepted {
            version: 1,
            action: PublicPromptAction::Start,
            orchestration_session_id: orchestration_snapshot.orchestration_session_id.clone(),
            backend_id: manifest_snapshot.handle.backend_id.clone(),
            participant_id: Some(manifest_snapshot.handle.participant_id.clone()),
            scope: "host".to_string(),
        });
    }
    let control = match gateway.run_control(&agent_kind, request).await {
        Ok(control) => control,
        Err(err) => {
            let failure = runtime_bootstrap_failure_from_wrapper_error(err);
            if let Some(backchannel) = startup_backchannel.as_ref() {
                let (orchestration_snapshot, manifest_snapshot) = {
                    let mut orchestration_guard = startup_context
                        .orchestration_session
                        .lock()
                        .expect("orchestration session mutex poisoned");
                    let manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
                    orchestration_guard.mark_startup_prompt_failed(
                        manifest_guard.handle.participant_id.as_str(),
                        failure.message.clone(),
                    );
                    (orchestration_guard.clone(), manifest_guard.clone())
                };
                let _ = persist_runtime_snapshots(
                    &startup_context.store,
                    &orchestration_snapshot,
                    &manifest_snapshot,
                );
                backchannel.send(startup_prompt_failed_envelope(failure.message.clone()));
            }
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &failure.message,
            );
            return Err(failure);
        }
    };
    let agent_api::AgentWrapperRunControl { handle, cancel } = control;
    let agent_api::AgentWrapperRunHandle { events, completion } = handle;
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let (startup_tx, startup_rx) = tokio::sync::oneshot::channel::<RuntimeStartupSignal>();
    let startup_signal = Arc::new(Mutex::new(Some(startup_tx)));
    let agent_id = {
        manifest
            .lock()
            .expect("runtime manifest mutex poisoned")
            .handle
            .agent_id
            .clone()
    };
    let run_id_for_tasks = run_id.clone();
    let event_store = startup_context.store.clone();
    let event_orchestration_session = Arc::clone(&startup_context.orchestration_session);
    let event_manifest = Arc::clone(&manifest);
    let startup_signal_for_events = Arc::clone(&startup_signal);
    let startup_backchannel_for_events = startup_backchannel.clone();
    let shutdown_for_events = Arc::clone(&shutdown_requested);
    let runtime_role_for_events = runtime_role.clone();
    let mut events = events;
    let event_task = tokio::spawn(async move {
        let controls_parent_session = runtime_controls_parent_session(&runtime_role_for_events);
        let mut startup_saw_substantive_output = false;
        while let Some(wrapper_event) = events.next().await {
            let startup_turn_phase = turn_lifecycle_phase(wrapper_event.data.as_ref())
                .map(ToOwned::to_owned)
                .or_else(|| {
                    infer_startup_turn_phase_fallback(
                        &wrapper_event,
                        startup_saw_substantive_output,
                    )
                    .map(ToOwned::to_owned)
                });
            let startup_failure_message = wrapper_event.message.clone();
            if matches!(
                wrapper_event.kind,
                agent_api::AgentWrapperEventKind::TextOutput
                    | agent_api::AgentWrapperEventKind::ToolCall
                    | agent_api::AgentWrapperEventKind::ToolResult
                    | agent_api::AgentWrapperEventKind::Error
            ) {
                startup_saw_substantive_output = true;
            }
            let mut startup_became_live = false;
            let (orchestration_snapshot, manifest_snapshot, event) = {
                let mut orchestration_guard = event_orchestration_session
                    .lock()
                    .expect("orchestration session mutex poisoned");
                let mut manifest_guard = event_manifest
                    .lock()
                    .expect("runtime manifest mutex poisoned");
                if let Some(session_id) = extract_session_handle_id(wrapper_event.data.as_ref()) {
                    if manifest_guard.internal.uaa_session_id.as_deref() != Some(session_id) {
                        manifest_guard.set_uaa_session_id(session_id.to_string());
                    }
                    if manifest_guard.handle.state == AgentRuntimeSessionState::Allocating
                        && manifest_guard.can_advertise_live()
                    {
                        manifest_guard.transition_state(AgentRuntimeSessionState::Ready);
                        manifest_guard.touch_heartbeat();
                        if controls_parent_session {
                            orchestration_guard.bind_active_session_handle(
                                manifest_guard.handle.participant_id.clone(),
                            );
                            orchestration_guard.transition_state(OrchestrationSessionState::Active);
                        } else if orchestration_guard.state == OrchestrationSessionState::Active {
                            orchestration_guard.touch_active();
                        }
                        startup_became_live = true;
                    }
                } else if manifest_guard.handle.state == AgentRuntimeSessionState::Ready
                    && manifest_guard.can_advertise_live()
                {
                    manifest_guard.transition_state(AgentRuntimeSessionState::Running);
                    orchestration_guard.touch_active();
                }
                let event = translate_wrapper_event(
                    &manifest_guard,
                    &orchestration_guard,
                    &run_id_for_tasks,
                    wrapper_event,
                );
                manifest_guard.touch_event(event.ts);
                if orchestration_guard.state == OrchestrationSessionState::Active {
                    orchestration_guard.touch_active();
                }
                if manifest_guard.handle.state == AgentRuntimeSessionState::Ready
                    && manifest_guard.can_advertise_live()
                    && !startup_became_live
                {
                    manifest_guard.transition_state(AgentRuntimeSessionState::Running);
                    orchestration_guard.touch_active();
                }
                if let Some(phase) = startup_turn_phase.as_deref() {
                    match phase {
                        "completed" => orchestration_guard.mark_startup_prompt_completed(
                            manifest_guard.handle.participant_id.as_str(),
                            "success",
                        ),
                        "failed" => orchestration_guard.mark_startup_prompt_failed(
                            manifest_guard.handle.participant_id.as_str(),
                            startup_failure_message
                                .clone()
                                .unwrap_or_else(|| "startup prompt turn failed".to_string()),
                        ),
                        _ => {}
                    }
                }
                (orchestration_guard.clone(), manifest_guard.clone(), event)
            };
            let _ = persist_runtime_snapshots(
                &event_store,
                &orchestration_snapshot,
                &manifest_snapshot,
            );
            if let Some(backchannel) = startup_backchannel_for_events.as_ref() {
                backchannel.send_event(&event);
                if let Some(phase) = startup_turn_phase.as_deref() {
                    match phase {
                        "completed" => {
                            backchannel.send(startup_prompt_completed_envelope(
                                &orchestration_snapshot,
                                &manifest_snapshot,
                            ));
                        }
                        "failed" => backchannel.send(startup_prompt_failed_envelope(
                            startup_failure_message
                                .clone()
                                .unwrap_or_else(|| "startup prompt turn failed".to_string()),
                        )),
                        _ => {}
                    }
                }
            }
            let _ = publish_agent_event(event);

            if startup_became_live {
                let _ = publish_agent_event(build_runtime_message_event(
                    &manifest_snapshot,
                    &orchestration_snapshot,
                    run_id_for_tasks.clone(),
                    MessageEventKind::Status,
                    runtime_ready_message(&runtime_role_for_events),
                ));
                signal_runtime_startup(&startup_signal_for_events, RuntimeStartupSignal::Running);
            }
        }

        let mut publish_events = Vec::new();
        let mut startup_failure: Option<String> = None;
        let (orchestration_snapshot, manifest_snapshot) = {
            let mut orchestration_guard = event_orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            let mut manifest_guard = event_manifest
                .lock()
                .expect("runtime manifest mutex poisoned");
            let was_allocating =
                manifest_guard.handle.state == AgentRuntimeSessionState::Allocating;
            let was_live = manifest_guard.is_authoritative_live();
            manifest_guard.set_event_stream_active(false);
            if was_allocating {
                let reason =
                    "attached control turn ended before ownership could be established".to_string();
                manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                manifest_guard.mark_terminal_state(reason.clone());
                manifest_guard.internal.last_error_bucket = Some("bootstrap_run".to_string());
                manifest_guard.internal.last_error_message = Some(reason.clone());
                if controls_parent_session {
                    orchestration_guard.transition_state(OrchestrationSessionState::Failed);
                    orchestration_guard.mark_terminal(reason.clone());
                } else {
                    orchestration_guard.touch_active();
                }
                startup_failure = Some(reason);
            } else if !shutdown_for_events.load(Ordering::SeqCst) && was_live {
                if can_park_host_runtime_after_detach(
                    &event_store,
                    controls_parent_session,
                    &orchestration_guard,
                    &manifest_guard,
                    runtime_owns_private_stop,
                    false,
                ) {
                    apply_parked_host_runtime_snapshots(
                        &mut orchestration_guard,
                        &mut manifest_guard,
                        "owner detached cleanly",
                    )
                    .expect("prevalidated host runtime parking should satisfy continuity");
                    publish_events.push(build_runtime_message_event(
                        &manifest_guard,
                        &orchestration_guard,
                        run_id_for_tasks.clone(),
                        MessageEventKind::Status,
                        runtime_detached_message(&runtime_role_for_events),
                    ));
                } else if runtime_owns_private_stop && controls_parent_session {
                    // Hidden owner-helper lifetimes are resolved by the retained completion
                    // observer at helper-exit handoff. Losing the event stream first is not,
                    // by itself, an invalidation condition for the durable parent session.
                    orchestration_guard.touch_active();
                } else {
                    let reason = if runtime_role_for_events == MEMBER_ROLE {
                        "world-scoped member control stream ended before completion observation"
                    } else {
                        "shell-owned orchestrator control stream ended before completion observation"
                    }
                    .to_string();
                    manifest_guard.transition_state(AgentRuntimeSessionState::Invalidated);
                    manifest_guard.mark_terminal_state(reason.clone());
                    manifest_guard.internal.last_error_bucket =
                        Some("runtime_lifecycle".to_string());
                    manifest_guard.internal.last_error_message = Some(reason.clone());
                    if controls_parent_session {
                        orchestration_guard
                            .transition_state(OrchestrationSessionState::Invalidated);
                        orchestration_guard.mark_terminal(reason.clone());
                    } else {
                        orchestration_guard.touch_active();
                    }
                    let mut event = AgentEvent::alert(
                        manifest_guard.handle.agent_id.clone(),
                        manifest_guard.handle.orchestration_session_id.clone(),
                        run_id_for_tasks.clone(),
                        runtime_stream_closed_alert_code(&runtime_role_for_events),
                        reason,
                    );
                    apply_runtime_participant_lineage(&mut event, &manifest_guard);
                    publish_events.push(event);
                }
            }
            (orchestration_guard.clone(), manifest_guard.clone())
        };
        let _ =
            persist_runtime_snapshots(&event_store, &orchestration_snapshot, &manifest_snapshot);
        for event in publish_events {
            let _ = publish_agent_event(event);
        }
        if let Some(backchannel) = startup_backchannel_for_events.as_ref() {
            let startup_state = orchestration_snapshot.startup_prompt_state();
            if matches!(
                startup_state,
                Some(StartupPromptStreamState::PendingAcceptance)
            ) {
                backchannel.send(startup_prompt_failed_envelope(
                    startup_failure.clone().unwrap_or_else(|| {
                        "startup prompt stream closed before acceptance".to_string()
                    }),
                ));
            } else if !matches!(
                startup_state,
                Some(StartupPromptStreamState::Completed | StartupPromptStreamState::Failed)
            ) {
                backchannel.send(startup_prompt_failed_envelope(
                    startup_failure.clone().unwrap_or_else(|| {
                        "startup prompt stream ended before terminal completion".to_string()
                    }),
                ));
            }
        }
        if let Some(message) = startup_failure {
            signal_runtime_startup(
                &startup_signal_for_events,
                RuntimeStartupSignal::Failed(message),
            );
        }
    });

    let completion_store = startup_context.store.clone();
    let completion_orchestration_session = Arc::clone(&startup_context.orchestration_session);
    let completion_manifest = Arc::clone(&manifest);
    let startup_signal_for_completion = Arc::clone(&startup_signal);
    let startup_backchannel_for_completion = startup_backchannel.clone();
    let shutdown_for_completion = Arc::clone(&shutdown_requested);
    let run_id_for_completion = run_id.clone();
    let runtime_role_for_completion = runtime_role.clone();
    let completion_task = tokio::spawn(async move {
        let controls_parent_session = runtime_controls_parent_session(&runtime_role_for_completion);
        let completion = completion.await;
        let shutdown_requested = shutdown_for_completion.load(Ordering::SeqCst);
        let mut startup_failure: Option<String> = None;
        let mut publish_events = Vec::new();

        let (orchestration_snapshot, manifest_snapshot) = {
            let mut orchestration_guard = completion_orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            let mut manifest_guard = completion_manifest
                .lock()
                .expect("runtime manifest mutex poisoned");

            match completion {
                Ok(completion) => {
                    if manifest_guard.internal.uaa_session_id.is_none() {
                        if let Some(session_id) =
                            extract_session_handle_id(completion.data.as_ref())
                        {
                            manifest_guard.set_uaa_session_id(session_id.to_string());
                        }
                    }

                    if manifest_guard.handle.state == AgentRuntimeSessionState::Allocating {
                        let reason = if completion.status.success() {
                            "attached control turn ended before ownership could be established"
                                .to_string()
                        } else {
                            format!(
                                "attached control turn exited with status {} before ownership was established",
                                completion.status.code().unwrap_or(-1)
                            )
                        };
                        manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                        manifest_guard.mark_terminal_state(reason.clone());
                        manifest_guard.internal.last_error_bucket =
                            Some("bootstrap_run".to_string());
                        manifest_guard.internal.last_error_message = Some(reason.clone());
                        if controls_parent_session {
                            orchestration_guard.transition_state(OrchestrationSessionState::Failed);
                            orchestration_guard.mark_terminal(reason.clone());
                        } else {
                            orchestration_guard.touch_active();
                        }
                        startup_failure = Some(reason);
                    } else if shutdown_requested
                        && manifest_guard.handle.state == AgentRuntimeSessionState::Stopping
                    {
                        apply_runtime_stop_closeout(&mut orchestration_guard, &mut manifest_guard);
                    } else if shutdown_requested
                        && manifest_guard.handle.state == AgentRuntimeSessionState::Failed
                    {
                        // Preserve bootstrap failure when startup cleanup observes completion.
                    } else if manifest_guard.handle.state == AgentRuntimeSessionState::Failed {
                        // Preserve bootstrap failure even if completion wins the race before
                        // startup cleanup flips the shutdown_requested flag.
                    } else if manifest_guard.handle.state == AgentRuntimeSessionState::Invalidated {
                        // Preserve invalidation emitted when the attached control boundary ended.
                    } else if manifest_guard.handle.state == AgentRuntimeSessionState::Stopping {
                        apply_runtime_stop_closeout(&mut orchestration_guard, &mut manifest_guard);
                    } else if can_park_host_runtime_after_detach(
                        &completion_store,
                        controls_parent_session,
                        &orchestration_guard,
                        &manifest_guard,
                        runtime_owns_private_stop,
                        true,
                    ) {
                        apply_parked_host_runtime_snapshots(
                            &mut orchestration_guard,
                            &mut manifest_guard,
                            "owner detached cleanly",
                        )
                        .expect("prevalidated host runtime parking should satisfy continuity");
                        publish_events.push(build_runtime_message_event(
                            &manifest_guard,
                            &orchestration_guard,
                            run_id_for_completion.clone(),
                            MessageEventKind::Status,
                            runtime_detached_message(&runtime_role_for_completion),
                        ));
                    } else {
                        let reason = format!(
                            "attached control turn exited with status {}",
                            completion.status.code().unwrap_or(-1)
                        );
                        manifest_guard.transition_state(AgentRuntimeSessionState::Invalidated);
                        manifest_guard.mark_terminal_state(reason.clone());
                        manifest_guard.internal.last_error_bucket =
                            Some("runtime_lifecycle".to_string());
                        manifest_guard.internal.last_error_message = Some(reason.clone());
                        if controls_parent_session {
                            orchestration_guard
                                .transition_state(OrchestrationSessionState::Invalidated);
                            orchestration_guard.mark_terminal(reason.clone());
                        } else {
                            orchestration_guard.touch_active();
                        }
                        let mut event = AgentEvent::alert(
                            manifest_guard.handle.agent_id.clone(),
                            manifest_guard.handle.orchestration_session_id.clone(),
                            run_id_for_completion.clone(),
                            runtime_invalidated_alert_code(&runtime_role_for_completion),
                            reason,
                        );
                        apply_runtime_participant_lineage(&mut event, &manifest_guard);
                        publish_events.push(event);
                    }
                }
                Err(err) => {
                    let reason = match &err {
                        agent_api::AgentWrapperError::Backend { message } => message.clone(),
                        other => other.to_string(),
                    };
                    if manifest_guard.handle.state == AgentRuntimeSessionState::Allocating {
                        let reason =
                            format!("failed to establish attached control ownership: {reason}");
                        manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                        manifest_guard.mark_terminal_state(reason.clone());
                        manifest_guard.internal.last_error_bucket =
                            Some("bootstrap_run".to_string());
                        manifest_guard.internal.last_error_message = Some(reason.clone());
                        if controls_parent_session {
                            orchestration_guard.transition_state(OrchestrationSessionState::Failed);
                            orchestration_guard.mark_terminal(reason.clone());
                        } else {
                            orchestration_guard.touch_active();
                        }
                        startup_failure = Some(reason);
                    } else if shutdown_requested
                        && manifest_guard.handle.state == AgentRuntimeSessionState::Stopping
                        && reason == "cancelled"
                    {
                        apply_runtime_stop_closeout(&mut orchestration_guard, &mut manifest_guard);
                    } else if shutdown_requested
                        && manifest_guard.handle.state == AgentRuntimeSessionState::Failed
                    {
                        // Preserve bootstrap failure when startup cleanup observes completion.
                    } else if manifest_guard.handle.state == AgentRuntimeSessionState::Failed {
                        // Preserve bootstrap failure even if completion wins the race before
                        // startup cleanup flips the shutdown_requested flag.
                    } else if manifest_guard.handle.state == AgentRuntimeSessionState::Invalidated {
                        // Preserve invalidation emitted when the attached control boundary ended.
                    } else if shutdown_requested {
                        let reason = format!("failed to stop attached control: {reason}");
                        manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                        manifest_guard.mark_terminal_state(reason.clone());
                        manifest_guard.internal.last_error_bucket =
                            Some("runtime_shutdown".to_string());
                        manifest_guard.internal.last_error_message = Some(reason.clone());
                        if controls_parent_session {
                            orchestration_guard.transition_state(OrchestrationSessionState::Failed);
                            orchestration_guard.mark_terminal(reason);
                        } else {
                            orchestration_guard.touch_active();
                        }
                    } else if can_park_host_runtime_after_detach(
                        &completion_store,
                        controls_parent_session,
                        &orchestration_guard,
                        &manifest_guard,
                        runtime_owns_private_stop,
                        true,
                    ) {
                        apply_parked_host_runtime_snapshots(
                            &mut orchestration_guard,
                            &mut manifest_guard,
                            "owner detached cleanly",
                        )
                        .expect("prevalidated host runtime parking should satisfy continuity");
                        publish_events.push(build_runtime_message_event(
                            &manifest_guard,
                            &orchestration_guard,
                            run_id_for_completion.clone(),
                            MessageEventKind::Status,
                            runtime_detached_message(&runtime_role_for_completion),
                        ));
                    } else {
                        let reason = format!("attached control turn ended unexpectedly: {reason}");
                        manifest_guard.transition_state(AgentRuntimeSessionState::Invalidated);
                        manifest_guard.mark_terminal_state(reason.clone());
                        manifest_guard.internal.last_error_bucket =
                            Some("runtime_lifecycle".to_string());
                        manifest_guard.internal.last_error_message = Some(reason.clone());
                        if controls_parent_session {
                            orchestration_guard
                                .transition_state(OrchestrationSessionState::Invalidated);
                            orchestration_guard.mark_terminal(reason.clone());
                        } else {
                            orchestration_guard.touch_active();
                        }
                        let mut event = AgentEvent::alert(
                            manifest_guard.handle.agent_id.clone(),
                            manifest_guard.handle.orchestration_session_id.clone(),
                            run_id_for_completion.clone(),
                            runtime_invalidated_alert_code(&runtime_role_for_completion),
                            reason,
                        );
                        apply_runtime_participant_lineage(&mut event, &manifest_guard);
                        publish_events.push(event);
                    }
                }
            }

            (orchestration_guard.clone(), manifest_guard.clone())
        };

        let _ = persist_runtime_snapshots(
            &completion_store,
            &orchestration_snapshot,
            &manifest_snapshot,
        );
        for event in publish_events {
            let _ = publish_agent_event(event);
        }
        if let Some(backchannel) = startup_backchannel_for_completion.as_ref() {
            let startup_state = orchestration_snapshot.startup_prompt_state();
            if !matches!(
                startup_state,
                Some(StartupPromptStreamState::Completed | StartupPromptStreamState::Failed)
            ) {
                backchannel.send(startup_prompt_failed_envelope(
                    startup_failure.clone().unwrap_or_else(|| {
                        "startup prompt completion did not reach a terminal turn outcome"
                            .to_string()
                    }),
                ));
            }
        }
        if let Some(message) = startup_failure {
            signal_runtime_startup(
                &startup_signal_for_completion,
                RuntimeStartupSignal::Failed(message),
            );
        }
    });

    let (heartbeat_stop_tx, mut heartbeat_stop_rx) = tokio::sync::oneshot::channel();
    let heartbeat_store = startup_context.store.clone();
    let heartbeat_orchestration_session = Arc::clone(&startup_context.orchestration_session);
    let heartbeat_manifest = Arc::clone(&manifest);
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let next = {
                        let mut orchestration_guard = heartbeat_orchestration_session
                            .lock()
                            .expect("orchestration session mutex poisoned");
                        let mut manifest_guard = heartbeat_manifest
                            .lock()
                            .expect("runtime manifest mutex poisoned");
                        if !manifest_guard.is_authoritative_live() {
                            None
                        } else {
                            manifest_guard.touch_heartbeat();
                            if orchestration_guard.state == OrchestrationSessionState::Active {
                                orchestration_guard.touch_active();
                            }
                            Some((orchestration_guard.clone(), manifest_guard.clone()))
                        }
                    };
                    let Some(next) = next else {
                        break;
                    };
                    let _ = persist_runtime_snapshots(
                        &heartbeat_store,
                        &next.0,
                        &next.1,
                    );
                }
                _ = &mut heartbeat_stop_rx => break,
            }
        }
    });
    let (retained_orchestration_snapshot, retained_manifest_snapshot) = {
        let mut orchestration_guard = startup_context
            .orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        let mut manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
        manifest_guard.mark_runtime_ownership_retained();
        if orchestration_guard.state == OrchestrationSessionState::Active || controls_parent_session
        {
            orchestration_guard.touch_active();
        }
        (orchestration_guard.clone(), manifest_guard.clone())
    };
    persist_runtime_snapshots(
        &startup_context.store,
        &retained_orchestration_snapshot,
        &retained_manifest_snapshot,
    )
    .map_err(|err| RuntimeBootstrapFailure {
        exit_code: 1,
        message: format!("failed to persist retained runtime ownership: {err:#}"),
    })?;
    let mut retained_control = LocalRetainedRunControl {
        cancel,
        event_task: Some(event_task),
        completion_task: Some(completion_task),
    };
    let mut heartbeat_stop_tx = Some(heartbeat_stop_tx);
    let mut heartbeat_task = Some(heartbeat_task);

    match tokio::time::timeout(Duration::from_secs(10), startup_rx).await {
        Ok(Ok(RuntimeStartupSignal::Running)) => {}
        Ok(Ok(RuntimeStartupSignal::Failed(message))) => {
            abort_bootstrap_runtime(
                &shutdown_requested,
                &mut retained_control,
                &mut heartbeat_stop_tx,
                &mut heartbeat_task,
            )
            .await;
            return Err(RuntimeBootstrapFailure {
                exit_code: 4,
                message,
            });
        }
        Ok(Err(_)) => {
            abort_bootstrap_runtime(
                &shutdown_requested,
                &mut retained_control,
                &mut heartbeat_stop_tx,
                &mut heartbeat_task,
            )
            .await;
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                "failed to establish attached control ownership",
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 1,
                message: "failed to establish attached control ownership".to_string(),
            });
        }
        Err(_) => {
            abort_bootstrap_runtime(
                &shutdown_requested,
                &mut retained_control,
                &mut heartbeat_stop_tx,
                &mut heartbeat_task,
            )
            .await;
            let message = format!(
                "timed out waiting for {} control ownership: {agent_id}",
                if runtime_role == MEMBER_ROLE {
                    "world-scoped member"
                } else {
                    "shell-owned orchestrator"
                }
            );
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &message,
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 4,
                message,
            });
        }
    }
    let surfaced_uaa_session_handle_id = {
        manifest
            .lock()
            .expect("runtime manifest mutex poisoned")
            .internal
            .uaa_session_id
            .clone()
    };
    let uaa_session_handle_id = match surfaced_uaa_session_handle_id {
        Some(session_handle_id) => session_handle_id,
        None => {
            abort_bootstrap_runtime(
                &shutdown_requested,
                &mut retained_control,
                &mut heartbeat_stop_tx,
                &mut heartbeat_task,
            )
            .await;
            let message =
                "runtime startup signalled ready without a surfaced UAA session handle".to_string();
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &message,
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 1,
                message,
            });
        }
    };
    let (stop_tx, stop_rx) = private_stop_request_channel();
    let (stop_orchestration_session_id, stop_participant_id) =
        runtime_stop_transport_ids(&manifest);
    let stop_transport = match register_private_stop_transport(
        &startup_context.store,
        &stop_orchestration_session_id,
        &stop_participant_id,
        stop_tx,
    )
    .await
    {
        Ok(stop_transport) => stop_transport,
        Err(err) => {
            abort_bootstrap_runtime(
                &shutdown_requested,
                &mut retained_control,
                &mut heartbeat_stop_tx,
                &mut heartbeat_task,
            )
            .await;
            let message = format!("failed to register private stop transport: {err:#}");
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &message,
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 1,
                message,
            });
        }
    };
    let (private_stop_rx, stop_owner_task) = if runtime_owns_private_stop {
        (Some(stop_rx), None)
    } else {
        (
            None,
            Some(spawn_local_private_stop_owner(
                startup_context.store.clone(),
                Arc::clone(&startup_context.orchestration_session),
                Arc::clone(&manifest),
                Arc::clone(&shutdown_requested),
                retained_control.cancel.clone(),
                stop_rx,
            )),
        )
    };
    let (prompt_tx, prompt_rx) = private_prompt_request_channel();
    let prompt_transport = match register_private_prompt_transport(
        &startup_context.store,
        &stop_orchestration_session_id,
        &stop_participant_id,
        prompt_tx,
    )
    .await
    {
        Ok(prompt_transport) => prompt_transport,
        Err(err) => {
            abort_bootstrap_runtime(
                &shutdown_requested,
                &mut retained_control,
                &mut heartbeat_stop_tx,
                &mut heartbeat_task,
            )
            .await;
            let message = format!("failed to register private prompt transport: {err:#}");
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &message,
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 1,
                message,
            });
        }
    };
    let prompt_owner_task = spawn_local_private_prompt_owner(
        prompt_runtime_from_parts(
            descriptor.clone(),
            Arc::clone(&startup_context.orchestration_session),
            Arc::clone(&manifest),
            startup_context.store.clone(),
            uaa_session_handle_id.clone(),
        ),
        prompt_rx,
    );

    Ok(Some(AsyncReplAgentRuntime {
        descriptor,
        orchestration_session: startup_context.orchestration_session,
        manifest,
        store: startup_context.store,
        uaa_session_handle_id,
        retained_control: RetainedRunControl::Local(retained_control),
        shutdown_requested,
        private_stop_rx,
        stop_transport: Some(stop_transport),
        stop_owner_task,
        prompt_transport: Some(prompt_transport),
        prompt_owner_task: Some(prompt_owner_task),
        heartbeat_stop_tx,
        heartbeat_task,
    }))
}

fn runtime_manifest_snapshot(runtime: &AsyncReplAgentRuntime) -> AgentRuntimeParticipantRecord {
    runtime
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .clone()
}

fn runtime_backend_id(runtime: &AsyncReplAgentRuntime) -> String {
    runtime
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .handle
        .backend_id
        .clone()
}

fn runtime_prompt_submit_runtime(
    runtime: &AsyncReplAgentRuntime,
) -> crate::execution::agent_runtime::control::PromptSubmitRuntime {
    prompt_runtime_from_parts(
        runtime.descriptor.clone(),
        Arc::clone(&runtime.orchestration_session),
        Arc::clone(&runtime.manifest),
        runtime.store.clone(),
        runtime.uaa_session_handle_id.clone(),
    )
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn select_member_runtime_descriptor(
    startup_context: &RuntimeOrchestrationContext,
) -> std::result::Result<Option<RuntimeSelectionDescriptor>, RuntimeBootstrapFailure> {
    validate_member_selection(
        &startup_context.effective_config,
        &startup_context.inventory,
    )
    .map_err(|err| RuntimeBootstrapFailure {
        exit_code: member_selection_error_exit_code(&err),
        message: err.reason,
    })
}

fn select_exact_runtime_descriptor(
    startup_context: &RuntimeOrchestrationContext,
    scope: crate::execution::config_model::AgentExecutionScope,
    backend_id: &str,
) -> std::result::Result<Option<RuntimeSelectionDescriptor>, RuntimeBootstrapFailure> {
    validate_exact_backend_selection(
        &startup_context.effective_config,
        &startup_context.inventory,
        scope,
        backend_id,
    )
    .map_err(|err| RuntimeBootstrapFailure {
        exit_code: exact_backend_selection_error_exit_code(&err),
        message: err.reason,
    })
}

fn select_member_runtime_descriptor_for_backend(
    startup_context: &RuntimeOrchestrationContext,
    backend_id: &str,
) -> std::result::Result<Option<RuntimeSelectionDescriptor>, RuntimeBootstrapFailure> {
    select_exact_runtime_descriptor(
        startup_context,
        crate::execution::config_model::AgentExecutionScope::World,
        backend_id,
    )
}

fn select_host_runtime_descriptor_for_backend(
    startup_context: &RuntimeOrchestrationContext,
    backend_id: &str,
) -> std::result::Result<Option<RuntimeSelectionDescriptor>, RuntimeBootstrapFailure> {
    select_exact_runtime_descriptor(
        startup_context,
        crate::execution::config_model::AgentExecutionScope::Host,
        backend_id,
    )
}

fn active_orchestrator_backend_id(runtime: &AsyncReplAgentRuntime) -> String {
    runtime_backend_id(runtime)
}

fn resolve_targeted_turn_route(
    startup_context: Option<&RuntimeOrchestrationContext>,
    dormant_host_bootstrap: Option<&ResolvedHostOrchestratorBootstrap>,
    agent_runtime: Option<&AsyncReplAgentRuntime>,
    backend_id: &str,
) -> std::result::Result<TargetedTurnRoute, RuntimeBootstrapFailure> {
    if let Some(runtime) = agent_runtime {
        let active_backend_id = active_orchestrator_backend_id(runtime);
        if active_backend_id == backend_id {
            return Ok(TargetedTurnRoute::Host);
        }
    }

    if let Some(dormant_host_bootstrap) = dormant_host_bootstrap {
        if dormant_host_bootstrap.descriptor.backend_id == backend_id {
            return Ok(TargetedTurnRoute::Host);
        }
    }

    if let Some(startup_context) = startup_context {
        if select_host_runtime_descriptor_for_backend(startup_context, backend_id)?.is_some() {
            let expected = agent_runtime
                .map(active_orchestrator_backend_id)
                .or_else(|| {
                    dormant_host_bootstrap.map(|bootstrap| bootstrap.descriptor.backend_id.clone())
                })
                .unwrap_or_else(|| "<none>".to_string());
            return Err(RuntimeBootstrapFailure {
                exit_code: 2,
                message: format!(
                    "targeted host follow-up turns may only target the active orchestrator backend for this REPL session (expected '{}', got '{}')",
                    expected, backend_id
                ),
            });
        }

        if let Some(descriptor) =
            select_member_runtime_descriptor_for_backend(startup_context, backend_id)?
        {
            return Ok(TargetedTurnRoute::World(descriptor));
        }
    }

    Err(RuntimeBootstrapFailure {
        exit_code: 2,
        message: format!("no exact targeted-turn backend match for '{backend_id}'"),
    })
}

async fn dispatch_targeted_follow_up_turn(
    targeted_turn: TargetedTurn<'_>,
    context: TargetedTurnDispatchContext<'_>,
) -> std::result::Result<TargetedTurnDispatchStatus, anyhow::Error> {
    let TargetedTurnDispatchContext {
        startup_context,
        dormant_host_bootstrap,
        agent_runtime,
        world_session,
        member_runtimes,
        pending_member_replacements,
        agent_printer,
        telemetry,
    } = context;

    let route = match resolve_targeted_turn_route(
        startup_context.as_ref(),
        dormant_host_bootstrap.as_ref(),
        agent_runtime.as_ref(),
        targeted_turn.backend_id,
    ) {
        Ok(route) => route,
        Err(failure) => return Ok(TargetedTurnDispatchStatus::Rejected(failure)),
    };

    match route {
        TargetedTurnRoute::Host => {
            if agent_runtime.is_none() {
                let resolved = dormant_host_bootstrap.take().ok_or_else(|| {
                    anyhow!(
                        "substrate: error: no active or dormant orchestrator runtime is available for targeted follow-up turns"
                    )
                })?;
                let prepared = prepare_host_orchestrator_runtime_from_resolved(resolved)
                    .map_err(|failure| anyhow!("substrate: error: {}", failure.message))?;
                let prepared_startup_context = prepared.startup_context.clone();
                let initial_world_binding =
                    world_session.as_ref().map(|session| PersistedWorldBinding {
                        world_id: session.world_id.clone(),
                        world_generation: session.world_generation,
                    });
                let runtime = match start_host_orchestrator_runtime_with_prepared_prompt(
                    Some(prepared),
                    initial_world_binding.as_ref(),
                    Some(InitialExecPromptPlan::Replace(
                        targeted_turn.prompt.to_string(),
                    )),
                    false,
                    agent_printer,
                    telemetry,
                )
                .await
                {
                    Ok(Some(runtime)) => runtime,
                    Ok(None) => {
                        return Err(anyhow!(
                            "substrate: error: targeted orchestrator launch did not produce a runtime"
                        ));
                    }
                    Err(failure) => {
                        finalize_runtime_startup_failure(
                            Some(&prepared_startup_context),
                            world_session,
                            &failure.message,
                        )
                        .await;
                        return Err(anyhow!("substrate: error: {}", failure.message));
                    }
                };
                *startup_context = Some(prepared_startup_context);
                *agent_runtime = Some(runtime);
            } else {
                let runtime = agent_runtime.as_mut().ok_or_else(|| {
                    anyhow!(
                        "substrate: error: no active orchestrator runtime is available for targeted follow-up turns"
                    )
                })?;
                submit_host_targeted_turn(runtime, targeted_turn.prompt, agent_printer, telemetry)
                    .await?;
            }
        }
        TargetedTurnRoute::World(descriptor) => {
            ensure_no_policy_drift(
                world_session,
                startup_context.as_ref(),
                agent_printer,
                telemetry,
            )
            .await?;
            reconcile_member_runtime_generation(
                world_session.as_ref(),
                member_runtimes,
                pending_member_replacements,
                agent_printer,
                telemetry,
            )
            .await?;
            let launched_from_targeted_prompt = ensure_member_runtime_ready_for_descriptor(
                EnsureMemberRuntimeReadyContext {
                    startup_context: startup_context.as_ref(),
                    world_session: world_session.as_ref(),
                    agent_printer,
                    telemetry,
                },
                &descriptor,
                Some(targeted_turn.prompt),
                member_runtimes,
                pending_member_replacements,
            )
            .await?;
            if launched_from_targeted_prompt {
                return Ok(TargetedTurnDispatchStatus::Submitted);
            }
            let runtime = member_runtimes
                .get_mut(targeted_turn.backend_id)
                .ok_or_else(|| {
                    anyhow!(
                        "substrate: error: world-scoped member runtime is unavailable for targeted follow-up turns for backend '{}'",
                        targeted_turn.backend_id
                    )
                })?;
            submit_world_targeted_turn(runtime, targeted_turn.prompt, agent_printer, telemetry)
                .await?;
        }
    }

    Ok(TargetedTurnDispatchStatus::Submitted)
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn ensure_member_backend_allowed(
    startup_context: &RuntimeOrchestrationContext,
    descriptor: &RuntimeSelectionDescriptor,
) -> std::result::Result<(), RuntimeBootstrapFailure> {
    if backend_allowed(&startup_context.base_policy, &descriptor.backend_id) {
        Ok(())
    } else {
        Err(RuntimeBootstrapFailure {
            exit_code: 5,
            message: format!(
                "required world-scoped member backend '{}' is not allowlisted by effective policy agents.allowed_backends",
                descriptor.backend_id
            ),
        })
    }
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn resolve_live_member_parent(
    startup_context: &RuntimeOrchestrationContext,
) -> std::result::Result<AgentRuntimeParticipantRecord, RuntimeBootstrapFailure> {
    let orchestration_snapshot = startup_context.snapshot();
    let (parent_session, parent_participant) = startup_context
        .store
        .resolve_live_orchestrator_participant(&orchestration_snapshot.orchestrator_agent_id)
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!(
                "failed to resolve the live orchestrator parent required for member launch: {err:#}"
            ),
        })?
        .ok_or_else(|| RuntimeBootstrapFailure {
            exit_code: 1,
            message:
                "member launch requires exactly one live orchestrator parent, but none is active"
                    .to_string(),
        })?;
    if parent_session.orchestration_session_id != orchestration_snapshot.orchestration_session_id {
        return Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!(
                "member launch resolved a live orchestrator parent from a different orchestration session (expected {}, got {})",
                orchestration_snapshot.orchestration_session_id,
                parent_session.orchestration_session_id
            ),
        });
    }
    Ok(parent_participant)
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn authoritative_member_world_binding(
    startup_context: &RuntimeOrchestrationContext,
    world_binding: &PersistedWorldBinding,
) -> std::result::Result<AgentRuntimeParticipantWorldBinding, RuntimeBootstrapFailure> {
    let orchestration_snapshot = startup_context.snapshot();
    let Some(authoritative_world_id) = orchestration_snapshot.world_id.clone() else {
        return Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message:
                "member launch requires an authoritative world binding, but world_id is missing"
                    .to_string(),
        });
    };
    let Some(authoritative_world_generation) = orchestration_snapshot.world_generation else {
        return Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message:
                "member launch requires an authoritative world binding, but world_generation is missing"
                    .to_string(),
        });
    };
    if authoritative_world_id != world_binding.world_id
        || authoritative_world_generation != world_binding.world_generation
    {
        return Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!(
                "member launch requires the authoritative parent world binding to match the active world session (authoritative={},{} active={},{} )",
                authoritative_world_id,
                authoritative_world_generation,
                world_binding.world_id,
                world_binding.world_generation,
            ),
        });
    }
    Ok(AgentRuntimeParticipantWorldBinding {
        world_id: authoritative_world_id,
        world_generation: authoritative_world_generation,
    })
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn live_member_for_generation(
    startup_context: &RuntimeOrchestrationContext,
    descriptor: &RuntimeSelectionDescriptor,
    world_generation: u64,
) -> std::result::Result<Option<AgentRuntimeParticipantRecord>, RuntimeBootstrapFailure> {
    let orchestration_session_id = startup_context.orchestration_session_id();
    let mut live_members = startup_context
        .store
        .list_live_participants_for_session(&orchestration_session_id)
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!(
                "failed to inspect live member participants for the current session: {err:#}"
            ),
        })?
        .into_iter()
        .filter(|participant| {
            participant.handle.role == MEMBER_ROLE
                && participant.handle.backend_id == descriptor.backend_id
                && participant.handle.execution.scope
                    == crate::execution::config_model::AgentExecutionScope::World
                && participant.handle.world_generation == Some(world_generation)
        })
        .collect::<Vec<_>>();
    match live_members.len() {
        0 => Ok(None),
        1 => Ok(live_members.pop()),
        _ => Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!(
                "member launch found multiple live world-scoped member participants for backend '{}' in world generation {} within the current orchestration session",
                descriptor.backend_id, world_generation
            ),
        }),
    }
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn prepare_member_runtime_startup_for_descriptor(
    startup_context: &RuntimeOrchestrationContext,
    descriptor: RuntimeSelectionDescriptor,
    world_binding: &PersistedWorldBinding,
    resumed_from: Option<&AgentRuntimeParticipantRecord>,
) -> std::result::Result<PreparedAgentRuntime, RuntimeBootstrapFailure> {
    ensure_member_backend_allowed(startup_context, &descriptor)?;
    let parent_participant = resolve_live_member_parent(startup_context)?;
    let authoritative_world = authoritative_member_world_binding(startup_context, world_binding)?;

    if live_member_for_generation(
        startup_context,
        &descriptor,
        authoritative_world.world_generation,
    )?
    .is_some()
    {
        return Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!(
                "member launch found an existing authoritative-live world-scoped member participant for backend '{}' in world generation {} without a retained runtime handle in the current REPL",
                descriptor.backend_id, authoritative_world.world_generation
            ),
        });
    }

    let participant_id = format!("ash_{}", Uuid::now_v7());
    let lease_token = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let manifest = if let Some(previous_participant) = resumed_from {
        AgentRuntimeSessionManifest::new_replacement_participant(
            &descriptor,
            AgentRuntimeReplacementParticipantInit {
                orchestration_session_id: startup_context.orchestration_session_id(),
                participant_id,
                role: MEMBER_ROLE.to_string(),
                orchestrator_participant_id: Some(parent_participant.handle.participant_id.clone()),
                parent_participant_id: None,
                resumed_from_participant_id: previous_participant.handle.participant_id.clone(),
                world: Some(authoritative_world),
                lease_token,
            },
        )
    } else {
        AgentRuntimeSessionManifest::new_member_participant(
            &descriptor,
            startup_context.orchestration_session_id(),
            participant_id,
            parent_participant.handle.participant_id.clone(),
            None,
            Some(authoritative_world),
            lease_token,
        )
    }
    .map_err(|err| RuntimeBootstrapFailure {
        exit_code: 1,
        message: format!("failed to construct world-scoped member participant state: {err:#}"),
    })?;
    let mut manifest = manifest;
    manifest.internal.latest_run_id = Some(run_id.clone());

    let gateway =
        build_gateway_for_descriptor(&descriptor).map_err(|err| RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to build world-scoped member UAA runtime registry: {err:#}"),
        })?;
    let agent_kind = agent_api::AgentWrapperKind::new(descriptor.backend_kind.as_agent_kind_str())
        .map_err(|err| RuntimeBootstrapFailure {
            exit_code: 2,
            message: format!("failed to resolve member runtime backend kind: {err}"),
        })?;

    Ok(PreparedAgentRuntime {
        descriptor,
        gateway,
        agent_kind,
        startup_context: startup_context.clone(),
        manifest: Arc::new(Mutex::new(manifest)),
        run_id,
        startup_extensions: BTreeMap::new(),
    })
}

#[cfg(test)]
async fn start_member_runtime_with_prepared(
    prepared: Option<PreparedAgentRuntime>,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> std::result::Result<Option<AsyncReplAgentRuntime>, RuntimeBootstrapFailure> {
    start_host_orchestrator_runtime_with_prepared(prepared, None, agent_printer, telemetry).await
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn member_runtime_backend_kind(
    descriptor: &RuntimeSelectionDescriptor,
) -> MemberRuntimeBackendKindV1 {
    match descriptor.backend_kind {
        AgentRuntimeBackendKind::Codex => MemberRuntimeBackendKindV1::Codex,
        AgentRuntimeBackendKind::ClaudeCode => MemberRuntimeBackendKindV1::ClaudeCode,
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn build_member_dispatch_transport_request(
    prepared: &PreparedAgentRuntime,
    initial_prompt: Option<String>,
) -> std::result::Result<MemberDispatchTransportRequest, RuntimeBootstrapFailure> {
    let manifest = prepared
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .clone();
    let world_id = manifest
        .handle
        .world_id
        .clone()
        .ok_or_else(|| RuntimeBootstrapFailure {
            exit_code: 1,
            message: "member dispatch requires an authoritative world_id".to_string(),
        })?;
    let world_generation =
        manifest
            .handle
            .world_generation
            .ok_or_else(|| RuntimeBootstrapFailure {
                exit_code: 1,
                message: "member dispatch requires an authoritative world_generation".to_string(),
            })?;
    let orchestrator_participant_id = manifest
        .handle
        .orchestrator_participant_id
        .clone()
        .ok_or_else(|| RuntimeBootstrapFailure {
            exit_code: 1,
            message: "member dispatch requires orchestrator_participant_id".to_string(),
        })?;

    Ok(MemberDispatchTransportRequest {
        orchestration_session_id: manifest.handle.orchestration_session_id.clone(),
        participant_id: manifest.handle.participant_id.clone(),
        orchestrator_participant_id,
        parent_participant_id: manifest.handle.parent_participant_id.clone(),
        resumed_from_participant_id: manifest.handle.resumed_from_participant_id.clone(),
        backend_id: prepared.descriptor.backend_id.clone(),
        protocol: prepared.descriptor.protocol.clone(),
        run_id: prepared.run_id.clone(),
        world_id,
        world_generation,
        initial_prompt,
        backend_kind: member_runtime_backend_kind(&prepared.descriptor),
        binary_path: prepared.descriptor.binary_path.display().to_string(),
    })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn abort_remote_member_bootstrap_runtime(
    shutdown_requested: &Arc<AtomicBool>,
    client: &AgentClient,
    span_id: &Arc<Mutex<Option<String>>>,
    observe_task: &mut Option<tokio::task::JoinHandle<()>>,
) {
    shutdown_requested.store(true, Ordering::SeqCst);
    let span_id = span_id
        .lock()
        .expect("remote member span mutex poisoned")
        .clone();
    if let Some(span_id) = span_id {
        let _ = client
            .cancel_execute(ExecuteCancelRequestV1 {
                span_id,
                sig: "INT".to_string(),
            })
            .await;
    }
    if let Some(task) = observe_task.take() {
        let _ = task.await;
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn spawn_remote_private_stop_owner(
    manifest: Arc<Mutex<AgentRuntimeSessionManifest>>,
    shutdown_requested: Arc<AtomicBool>,
    client: Arc<AgentClient>,
    span_id: String,
    mut stop_rx: PrivateStopRequestReceiver,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(request) = stop_rx.recv().await {
            let outcome = if runtime_is_terminal(&manifest) {
                PrivateStopOutcome::AlreadyTerminal
            } else {
                shutdown_requested.store(true, Ordering::SeqCst);
                match client
                    .cancel_execute(ExecuteCancelRequestV1 {
                        span_id: span_id.clone(),
                        sig: "INT".to_string(),
                    })
                    .await
                {
                    Ok(_) => PrivateStopOutcome::Accepted,
                    Err(_) => {
                        shutdown_requested.store(false, Ordering::SeqCst);
                        PrivateStopOutcome::ProtocolError
                    }
                }
            };
            let _ = request.response_tx.send(outcome);
        }
    })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn start_remote_member_runtime_with_prepared(
    prepared: Option<PreparedAgentRuntime>,
    initial_prompt: Option<String>,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> std::result::Result<Option<AsyncReplAgentRuntime>, RuntimeBootstrapFailure> {
    use http_body_util::BodyExt as _;

    let Some(prepared) = prepared else {
        return Ok(None);
    };
    let transport_request = build_member_dispatch_transport_request(&prepared, initial_prompt)?;
    let PreparedAgentRuntime {
        descriptor,
        gateway: _gateway,
        agent_kind: _agent_kind,
        startup_context,
        manifest,
        run_id,
        startup_extensions: _startup_extensions,
    } = prepared;

    let runtime_role = {
        manifest
            .lock()
            .expect("runtime manifest mutex poisoned")
            .handle
            .role
            .clone()
    };
    let persist_participant_result = {
        let manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
        startup_context.store.persist_participant(&manifest_guard)
    };
    if let Err(err) = persist_participant_result {
        mark_runtime_startup_failed(
            &startup_context.store,
            &startup_context.orchestration_session,
            &manifest,
            &format!("failed to persist agent runtime participant record: {err:#}"),
        );
        return Err(RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("failed to persist agent runtime participant record: {err:#}"),
        });
    }

    let orchestration_snapshot = startup_context
        .orchestration_session
        .lock()
        .expect("orchestration session mutex poisoned")
        .clone();
    let manifest_snapshot = manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .clone();
    emit_runtime_event(
        build_runtime_message_event(
            &manifest_snapshot,
            &orchestration_snapshot,
            run_id.clone(),
            MessageEventKind::Registered,
            runtime_registered_message(&runtime_role),
        ),
        telemetry,
        agent_printer,
    );
    emit_runtime_event(
        build_runtime_message_event(
            &manifest_snapshot,
            &orchestration_snapshot,
            run_id.clone(),
            MessageEventKind::TaskStart,
            runtime_task_start_message(&runtime_role),
        ),
        telemetry,
        agent_printer,
    );

    let (client, request, _agent_id) =
        build_agent_client_and_member_dispatch_request(&transport_request)
            .map_err(runtime_bootstrap_failure_from_anyhow)?;
    let response = client
        .execute_stream(request)
        .await
        .map_err(runtime_bootstrap_failure_from_anyhow)?;

    let (retained_orchestration_snapshot, retained_manifest_snapshot) = {
        let mut orchestration_guard = startup_context
            .orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        let mut manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
        manifest_guard.mark_runtime_ownership_retained();
        orchestration_guard.touch_active();
        (orchestration_guard.clone(), manifest_guard.clone())
    };
    persist_runtime_snapshots(
        &startup_context.store,
        &retained_orchestration_snapshot,
        &retained_manifest_snapshot,
    )
    .map_err(|err| RuntimeBootstrapFailure {
        exit_code: 1,
        message: format!("failed to persist retained runtime ownership: {err:#}"),
    })?;

    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let (startup_tx, startup_rx) = tokio::sync::oneshot::channel::<RuntimeStartupSignal>();
    let startup_signal = Arc::new(Mutex::new(Some(startup_tx)));
    let span_id = Arc::new(Mutex::new(None::<String>));

    let event_store = startup_context.store.clone();
    let event_orchestration_session = Arc::clone(&startup_context.orchestration_session);
    let event_manifest = Arc::clone(&manifest);
    let startup_signal_for_events = Arc::clone(&startup_signal);
    let shutdown_for_events = Arc::clone(&shutdown_requested);
    let runtime_role_for_events = runtime_role.clone();
    let run_id_for_events = run_id.clone();
    let span_id_for_events = Arc::clone(&span_id);

    let observe_task = tokio::spawn(async move {
        let mut body = std::pin::pin!(response.into_body());
        let mut buffer = Vec::new();
        let mut saw_terminal = false;

        while let Some(frame) = body.as_mut().frame().await {
            let Ok(frame) = frame else {
                break;
            };
            let Some(data) = frame.data_ref() else {
                continue;
            };
            buffer.extend_from_slice(data);

            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buffer.drain(..=pos).collect();
                if line.len() <= 1 {
                    continue;
                }
                let payload = &line[..line.len() - 1];
                if payload.is_empty() {
                    continue;
                }
                let Ok(frame) = serde_json::from_slice::<ExecuteStreamFrame>(payload) else {
                    continue;
                };

                match frame {
                    ExecuteStreamFrame::Start {
                        span_id: stream_span_id,
                    } => {
                        *span_id_for_events
                            .lock()
                            .expect("remote member span mutex poisoned") = Some(stream_span_id);
                    }
                    ExecuteStreamFrame::Event { event } => {
                        let mut startup_became_live = false;
                        let (orchestration_snapshot, manifest_snapshot) = {
                            let mut orchestration_guard = event_orchestration_session
                                .lock()
                                .expect("orchestration session mutex poisoned");
                            let mut manifest_guard = event_manifest
                                .lock()
                                .expect("runtime manifest mutex poisoned");
                            if let Some(session_id) = extract_session_handle_id(Some(&event.data)) {
                                if manifest_guard.internal.uaa_session_id.as_deref()
                                    != Some(session_id)
                                {
                                    manifest_guard.set_uaa_session_id(session_id.to_string());
                                }
                                if manifest_guard.handle.state
                                    == AgentRuntimeSessionState::Allocating
                                    && manifest_guard.can_advertise_live()
                                {
                                    manifest_guard
                                        .transition_state(AgentRuntimeSessionState::Ready);
                                    manifest_guard.touch_heartbeat();
                                    orchestration_guard.touch_active();
                                    startup_became_live = true;
                                }
                            }
                            manifest_guard.touch_event(event.ts);
                            orchestration_guard.touch_active();
                            (orchestration_guard.clone(), manifest_guard.clone())
                        };
                        let _ = persist_runtime_snapshots(
                            &event_store,
                            &orchestration_snapshot,
                            &manifest_snapshot,
                        );
                        let _ = publish_agent_event(event);
                        if startup_became_live {
                            let _ = publish_agent_event(build_runtime_message_event(
                                &manifest_snapshot,
                                &orchestration_snapshot,
                                run_id_for_events.clone(),
                                MessageEventKind::Status,
                                runtime_ready_message(&runtime_role_for_events),
                            ));
                            signal_runtime_startup(
                                &startup_signal_for_events,
                                RuntimeStartupSignal::Running,
                            );
                        }
                    }
                    ExecuteStreamFrame::Exit { exit, .. } => {
                        saw_terminal = true;
                        let mut startup_failure = None;
                        let (orchestration_snapshot, manifest_snapshot) = {
                            let mut orchestration_guard = event_orchestration_session
                                .lock()
                                .expect("orchestration session mutex poisoned");
                            let mut manifest_guard = event_manifest
                                .lock()
                                .expect("runtime manifest mutex poisoned");
                            if manifest_guard.handle.state == AgentRuntimeSessionState::Allocating {
                                let reason = format!(
                                    "world-scoped member runtime exited with status {exit} before ownership could be established"
                                );
                                manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                                manifest_guard.mark_terminal_state(reason.clone());
                                manifest_guard.internal.last_error_bucket =
                                    Some("bootstrap_run".to_string());
                                manifest_guard.internal.last_error_message = Some(reason.clone());
                                orchestration_guard.touch_active();
                                startup_failure = Some(reason);
                            } else if shutdown_for_events.load(Ordering::SeqCst)
                                || matches!(exit, 0 | 129 | 130 | 131 | 143)
                            {
                                if manifest_guard.handle.state
                                    == AgentRuntimeSessionState::Invalidated
                                {
                                    orchestration_guard.touch_active();
                                } else {
                                    manifest_guard
                                        .transition_state(AgentRuntimeSessionState::Stopped);
                                    manifest_guard
                                        .mark_terminal_state("world-scoped member session stopped");
                                    orchestration_guard.touch_active();
                                }
                            } else {
                                let reason = format!(
                                    "world-scoped member session exited with status {exit}"
                                );
                                manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                                manifest_guard.mark_terminal_state(reason.clone());
                                manifest_guard.internal.last_error_bucket =
                                    Some("runtime_lifecycle".to_string());
                                manifest_guard.internal.last_error_message = Some(reason.clone());
                                orchestration_guard.touch_active();
                            }
                            (orchestration_guard.clone(), manifest_guard.clone())
                        };
                        let _ = persist_runtime_snapshots(
                            &event_store,
                            &orchestration_snapshot,
                            &manifest_snapshot,
                        );
                        if let Some(reason) = startup_failure {
                            signal_runtime_startup(
                                &startup_signal_for_events,
                                RuntimeStartupSignal::Failed(reason),
                            );
                        }
                        break;
                    }
                    ExecuteStreamFrame::Error { message } => {
                        saw_terminal = true;
                        let mut startup_failure = None;
                        let (orchestration_snapshot, manifest_snapshot, alert) = {
                            let mut orchestration_guard = event_orchestration_session
                                .lock()
                                .expect("orchestration session mutex poisoned");
                            let mut manifest_guard = event_manifest
                                .lock()
                                .expect("runtime manifest mutex poisoned");
                            let reason = format!("remote member runtime failed: {message}");
                            if manifest_guard.handle.state == AgentRuntimeSessionState::Allocating {
                                manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                                startup_failure = Some(reason.clone());
                            } else {
                                manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                            }
                            manifest_guard.mark_terminal_state(reason.clone());
                            manifest_guard.internal.last_error_bucket =
                                Some("runtime_lifecycle".to_string());
                            manifest_guard.internal.last_error_message = Some(reason.clone());
                            orchestration_guard.touch_active();
                            let mut event = AgentEvent::alert(
                                manifest_guard.handle.agent_id.clone(),
                                manifest_guard.handle.orchestration_session_id.clone(),
                                run_id_for_events.clone(),
                                runtime_invalidated_alert_code(&runtime_role_for_events),
                                reason,
                            );
                            apply_runtime_participant_lineage(&mut event, &manifest_guard);
                            (orchestration_guard.clone(), manifest_guard.clone(), event)
                        };
                        let _ = persist_runtime_snapshots(
                            &event_store,
                            &orchestration_snapshot,
                            &manifest_snapshot,
                        );
                        let _ = publish_agent_event(alert);
                        if let Some(reason) = startup_failure {
                            signal_runtime_startup(
                                &startup_signal_for_events,
                                RuntimeStartupSignal::Failed(reason),
                            );
                        }
                        break;
                    }
                    ExecuteStreamFrame::Stdout { .. } | ExecuteStreamFrame::Stderr { .. } => {}
                }
            }
        }

        if saw_terminal {
            return;
        }

        let mut publish_events = Vec::new();
        let mut startup_failure = None;
        let (orchestration_snapshot, manifest_snapshot) = {
            let mut orchestration_guard = event_orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            let mut manifest_guard = event_manifest
                .lock()
                .expect("runtime manifest mutex poisoned");
            let was_allocating =
                manifest_guard.handle.state == AgentRuntimeSessionState::Allocating;
            let was_live = manifest_guard.is_authoritative_live();
            manifest_guard.set_event_stream_active(false);
            if was_allocating {
                let reason =
                    "attached control turn ended before ownership could be established".to_string();
                manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
                manifest_guard.mark_terminal_state(reason.clone());
                manifest_guard.internal.last_error_bucket = Some("bootstrap_run".to_string());
                manifest_guard.internal.last_error_message = Some(reason.clone());
                orchestration_guard.touch_active();
                startup_failure = Some(reason);
            } else if !shutdown_for_events.load(Ordering::SeqCst) && was_live {
                let reason =
                    "world-scoped member control stream ended before completion observation"
                        .to_string();
                manifest_guard.transition_state(AgentRuntimeSessionState::Invalidated);
                manifest_guard.mark_terminal_state(reason.clone());
                manifest_guard.internal.last_error_bucket = Some("runtime_lifecycle".to_string());
                manifest_guard.internal.last_error_message = Some(reason.clone());
                orchestration_guard.touch_active();
                let mut event = AgentEvent::alert(
                    manifest_guard.handle.agent_id.clone(),
                    manifest_guard.handle.orchestration_session_id.clone(),
                    run_id_for_events.clone(),
                    runtime_stream_closed_alert_code(&runtime_role_for_events),
                    reason,
                );
                apply_runtime_participant_lineage(&mut event, &manifest_guard);
                publish_events.push(event);
            }
            (orchestration_guard.clone(), manifest_guard.clone())
        };
        let _ =
            persist_runtime_snapshots(&event_store, &orchestration_snapshot, &manifest_snapshot);
        for event in publish_events {
            let _ = publish_agent_event(event);
        }
        if let Some(reason) = startup_failure {
            signal_runtime_startup(
                &startup_signal_for_events,
                RuntimeStartupSignal::Failed(reason),
            );
        }
    });

    let mut observe_task = Some(observe_task);
    match tokio::time::timeout(Duration::from_secs(10), startup_rx).await {
        Ok(Ok(RuntimeStartupSignal::Running)) => {}
        Ok(Ok(RuntimeStartupSignal::Failed(message))) => {
            abort_remote_member_bootstrap_runtime(
                &shutdown_requested,
                &client,
                &span_id,
                &mut observe_task,
            )
            .await;
            return Err(RuntimeBootstrapFailure {
                exit_code: 4,
                message,
            });
        }
        Ok(Err(_)) => {
            abort_remote_member_bootstrap_runtime(
                &shutdown_requested,
                &client,
                &span_id,
                &mut observe_task,
            )
            .await;
            let message = "failed to establish attached control ownership".to_string();
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &message,
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 1,
                message,
            });
        }
        Err(_) => {
            abort_remote_member_bootstrap_runtime(
                &shutdown_requested,
                &client,
                &span_id,
                &mut observe_task,
            )
            .await;
            let message = "timed out waiting for world-scoped member control ownership".to_string();
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &message,
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 4,
                message,
            });
        }
    }

    let surfaced_uaa_session_handle_id = {
        manifest
            .lock()
            .expect("runtime manifest mutex poisoned")
            .internal
            .uaa_session_id
            .clone()
    };
    let uaa_session_handle_id =
        surfaced_uaa_session_handle_id.ok_or_else(|| RuntimeBootstrapFailure {
            exit_code: 1,
            message: "runtime startup signalled ready without a surfaced UAA session handle"
                .to_string(),
        })?;
    let resolved_span_id = span_id
        .lock()
        .expect("remote member span mutex poisoned")
        .clone()
        .ok_or_else(|| RuntimeBootstrapFailure {
            exit_code: 1,
            message: "runtime startup signalled ready without a streamed execute span_id"
                .to_string(),
        })?;
    let (stop_tx, stop_rx) = private_stop_request_channel();
    let (stop_orchestration_session_id, stop_participant_id) =
        runtime_stop_transport_ids(&manifest);
    let stop_transport = match register_private_stop_transport(
        &startup_context.store,
        &stop_orchestration_session_id,
        &stop_participant_id,
        stop_tx,
    )
    .await
    {
        Ok(stop_transport) => stop_transport,
        Err(err) => {
            abort_remote_member_bootstrap_runtime(
                &shutdown_requested,
                &client,
                &span_id,
                &mut observe_task,
            )
            .await;
            let message = format!("failed to register private stop transport: {err:#}");
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &message,
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 1,
                message,
            });
        }
    };
    let client = Arc::new(client);
    let stop_owner_task = spawn_remote_private_stop_owner(
        Arc::clone(&manifest),
        Arc::clone(&shutdown_requested),
        Arc::clone(&client),
        resolved_span_id.clone(),
        stop_rx,
    );
    let (prompt_tx, prompt_rx) = private_prompt_request_channel();
    let prompt_transport = match register_private_prompt_transport(
        &startup_context.store,
        &stop_orchestration_session_id,
        &stop_participant_id,
        prompt_tx,
    )
    .await
    {
        Ok(prompt_transport) => prompt_transport,
        Err(err) => {
            abort_remote_member_bootstrap_runtime(
                &shutdown_requested,
                &client,
                &span_id,
                &mut observe_task,
            )
            .await;
            let message = format!("failed to register private prompt transport: {err:#}");
            mark_runtime_startup_failed(
                &startup_context.store,
                &startup_context.orchestration_session,
                &manifest,
                &message,
            );
            return Err(RuntimeBootstrapFailure {
                exit_code: 1,
                message,
            });
        }
    };
    let prompt_owner_task = spawn_remote_private_prompt_owner(
        prompt_runtime_from_parts(
            descriptor.clone(),
            Arc::clone(&startup_context.orchestration_session),
            Arc::clone(&manifest),
            startup_context.store.clone(),
            uaa_session_handle_id.clone(),
        ),
        prompt_rx,
    );

    Ok(Some(AsyncReplAgentRuntime {
        descriptor,
        orchestration_session: startup_context.orchestration_session,
        manifest,
        store: startup_context.store,
        uaa_session_handle_id,
        retained_control: RetainedRunControl::Remote(RemoteRetainedRunControl {
            client,
            span_id: resolved_span_id,
            observe_task,
        }),
        shutdown_requested,
        private_stop_rx: None,
        stop_transport: Some(stop_transport),
        stop_owner_task: Some(stop_owner_task),
        prompt_transport: Some(prompt_transport),
        prompt_owner_task: Some(prompt_owner_task),
        heartbeat_stop_tx: None,
        heartbeat_task: None,
    }))
}

async fn reconcile_member_runtime_generation(
    world_session: Option<&WorldSession>,
    member_runtimes: &mut RetainedMemberRuntimeMap,
    pending_member_replacements: &mut PendingMemberReplacementMap,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> Result<()> {
    let Some(world_session) = world_session else {
        return Ok(());
    };
    let stale_backends = member_runtimes
        .iter()
        .filter_map(|(backend_id, runtime)| {
            let manifest_snapshot = runtime_manifest_snapshot(runtime);
            (manifest_snapshot.handle.role == MEMBER_ROLE
                && manifest_snapshot.handle.world_generation
                    != Some(world_session.world_generation))
            .then_some(backend_id.clone())
        })
        .collect::<Vec<_>>();

    for backend_id in stale_backends {
        let Some(runtime) = member_runtimes.remove(&backend_id) else {
            continue;
        };
        let (orchestration_snapshot, invalidated_manifest) = {
            let mut manifest_guard = runtime
                .manifest
                .lock()
                .expect("runtime manifest mutex poisoned");
            if manifest_guard.handle.state != AgentRuntimeSessionState::Invalidated {
                let _ = manifest_guard.invalidate_for_world_generation_rollover();
            }
            let orchestration_snapshot = runtime
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned")
                .clone();
            (orchestration_snapshot, manifest_guard.clone())
        };
        persist_runtime_snapshots(
            &runtime.store,
            &orchestration_snapshot,
            &invalidated_manifest,
        )
        .with_context(|| {
            format!(
                "persist invalidated stale member runtime state after world generation rollover for backend '{}'",
                backend_id
            )
        })?;
        pending_member_replacements.insert(backend_id, invalidated_manifest.clone());
        shutdown_host_orchestrator_runtime(runtime, agent_printer, telemetry).await;
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct EnsureMemberRuntimeReadyContext<'a> {
    startup_context: Option<&'a RuntimeOrchestrationContext>,
    world_session: Option<&'a WorldSession>,
    agent_printer: &'a ReplPrinter,
    telemetry: &'a mut ReplSessionTelemetry,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn ensure_member_runtime_ready_for_descriptor(
    context: EnsureMemberRuntimeReadyContext<'_>,
    descriptor: &RuntimeSelectionDescriptor,
    initial_prompt: Option<&str>,
    member_runtimes: &mut RetainedMemberRuntimeMap,
    pending_member_replacements: &mut PendingMemberReplacementMap,
) -> Result<bool> {
    let EnsureMemberRuntimeReadyContext {
        startup_context,
        world_session,
        agent_printer,
        telemetry,
    } = context;
    let Some(startup_context) = startup_context else {
        return Ok(false);
    };
    let Some(world_session) = world_session else {
        return Ok(false);
    };
    ensure_member_backend_allowed(startup_context, descriptor)
        .map_err(|failure| anyhow!("substrate: error: {}", failure.message))?;

    let exact_live_member =
        live_member_for_generation(startup_context, descriptor, world_session.world_generation)
            .map_err(|failure| anyhow!("substrate: error: {}", failure.message))?;

    if let Some(runtime) = member_runtimes.get(&descriptor.backend_id) {
        let manifest_snapshot = runtime_manifest_snapshot(runtime);
        if manifest_snapshot.handle.role == MEMBER_ROLE
            && manifest_snapshot.handle.orchestration_session_id
                == startup_context.orchestration_session_id()
            && manifest_snapshot.handle.backend_id == descriptor.backend_id
            && manifest_snapshot.handle.world_generation == Some(world_session.world_generation)
            && manifest_snapshot.is_authoritative_live()
            && exact_live_member.as_ref().is_some_and(|participant| {
                participant.handle.participant_id == manifest_snapshot.handle.participant_id
            })
        {
            return Ok(false);
        }
    }

    if let Some(runtime) = member_runtimes.remove(&descriptor.backend_id) {
        shutdown_host_orchestrator_runtime(runtime, agent_printer, telemetry).await;
    }

    let resumed_from = pending_member_replacements
        .get(&descriptor.backend_id)
        .filter(|participant| {
            participant.handle.orchestration_session_id
                == startup_context.orchestration_session_id()
                && participant.handle.backend_id == descriptor.backend_id
        });
    let prepared = prepare_member_runtime_startup_for_descriptor(
        startup_context,
        descriptor.clone(),
        &PersistedWorldBinding {
            world_id: world_session.world_id.clone(),
            world_generation: world_session.world_generation,
        },
        resumed_from,
    )
    .map_err(|failure| anyhow!("substrate: error: {}", failure.message))?;

    let runtime = match start_remote_member_runtime_with_prepared(
        Some(prepared),
        initial_prompt.map(str::to_string),
        agent_printer,
        telemetry,
    )
    .await
    {
        Ok(runtime) => runtime,
        Err(failure) => return Err(anyhow!("substrate: error: {}", failure.message)),
    };
    if let Some(runtime) = runtime {
        pending_member_replacements.remove(&descriptor.backend_id);
        member_runtimes.insert(runtime_backend_id(&runtime), runtime);
        return Ok(true);
    }
    Ok(false)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
struct EnsureMemberRuntimeReadyContext<'a> {
    startup_context: Option<&'a RuntimeOrchestrationContext>,
    world_session: Option<&'a WorldSession>,
    agent_printer: &'a ReplPrinter,
    telemetry: &'a mut ReplSessionTelemetry,
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
async fn ensure_member_runtime_ready_for_descriptor(
    context: EnsureMemberRuntimeReadyContext<'_>,
    _descriptor: &RuntimeSelectionDescriptor,
    _initial_prompt: Option<&str>,
    _member_runtimes: &mut RetainedMemberRuntimeMap,
    _pending_member_replacements: &mut PendingMemberReplacementMap,
) -> Result<bool> {
    let EnsureMemberRuntimeReadyContext {
        startup_context,
        world_session,
        agent_printer: _agent_printer,
        telemetry: _telemetry,
    } = context;
    let Some(_startup_context) = startup_context else {
        return Ok(false);
    };
    let Some(_world_session) = world_session else {
        return Ok(false);
    };

    Err(anyhow!(
        "substrate: error: world-scoped member runtime dispatch is supported on Linux only"
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn ensure_member_runtime_ready(
    startup_context: Option<&RuntimeOrchestrationContext>,
    world_session: Option<&WorldSession>,
    member_runtimes: &mut RetainedMemberRuntimeMap,
    pending_member_replacements: &mut PendingMemberReplacementMap,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> Result<()> {
    let Some(startup_context) = startup_context else {
        return Ok(());
    };
    let Some(descriptor) = select_member_runtime_descriptor(startup_context)
        .map_err(|failure| anyhow!("substrate: error: {}", failure.message))?
    else {
        return Ok(());
    };

    ensure_member_runtime_ready_for_descriptor(
        EnsureMemberRuntimeReadyContext {
            startup_context: Some(startup_context),
            world_session,
            agent_printer,
            telemetry,
        },
        &descriptor,
        None,
        member_runtimes,
        pending_member_replacements,
    )
    .await
    .map(|_| ())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
async fn ensure_member_runtime_ready(
    startup_context: Option<&RuntimeOrchestrationContext>,
    world_session: Option<&WorldSession>,
    _member_runtimes: &mut RetainedMemberRuntimeMap,
    _pending_member_replacements: &mut PendingMemberReplacementMap,
    _agent_printer: &ReplPrinter,
    _telemetry: &mut ReplSessionTelemetry,
) -> Result<()> {
    let Some(_startup_context) = startup_context else {
        return Ok(());
    };
    let Some(_world_session) = world_session else {
        return Ok(());
    };

    Err(anyhow!(
        "substrate: error: world-scoped member runtime dispatch is supported on Linux only"
    ))
}

async fn shutdown_all_member_runtimes(
    member_runtimes: &mut RetainedMemberRuntimeMap,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) {
    while let Some((_, runtime)) = member_runtimes.pop_first() {
        shutdown_host_orchestrator_runtime(runtime, agent_printer, telemetry).await;
    }
}

fn note_submitted_turn_started(
    runtime: &AsyncReplAgentRuntime,
    run_id: &str,
    message: &str,
    telemetry: &mut ReplSessionTelemetry,
    agent_printer: &ReplPrinter,
) -> Result<()> {
    let (orchestration_snapshot, manifest_snapshot, event) = {
        let mut orchestration_guard = runtime
            .orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        let mut manifest_guard = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        manifest_guard.internal.latest_run_id = Some(run_id.to_string());
        orchestration_guard.touch_active();
        let event = build_runtime_message_event(
            &manifest_guard,
            &orchestration_guard,
            run_id.to_string(),
            MessageEventKind::TaskStart,
            message,
        );
        (orchestration_guard.clone(), manifest_guard.clone(), event)
    };
    persist_runtime_snapshots(&runtime.store, &orchestration_snapshot, &manifest_snapshot)?;
    emit_runtime_event(event, telemetry, agent_printer);
    Ok(())
}

fn note_submitted_turn_completed(
    runtime: &AsyncReplAgentRuntime,
    run_id: &str,
    message: impl Into<String>,
    telemetry: &mut ReplSessionTelemetry,
    agent_printer: &ReplPrinter,
) -> Result<()> {
    let (orchestration_snapshot, manifest_snapshot, event) = {
        let mut orchestration_guard = runtime
            .orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        let manifest_guard = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        orchestration_guard.touch_active();
        let event = build_runtime_message_event(
            &manifest_guard,
            &orchestration_guard,
            run_id.to_string(),
            MessageEventKind::Status,
            message,
        );
        (orchestration_guard.clone(), manifest_guard.clone(), event)
    };
    persist_runtime_snapshots(&runtime.store, &orchestration_snapshot, &manifest_snapshot)?;
    emit_runtime_event(event, telemetry, agent_printer);
    Ok(())
}

async fn submit_host_targeted_turn(
    runtime: &mut AsyncReplAgentRuntime,
    prompt: &str,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> Result<()> {
    let run_id = Uuid::now_v7().to_string();
    note_submitted_turn_started(
        runtime,
        &run_id,
        format!(
            "submitted targeted follow-up turn to {}",
            runtime.descriptor.backend_id
        )
        .as_str(),
        telemetry,
        agent_printer,
    )?;
    let completion = submit_host_prompt_turn(
        &runtime_prompt_submit_runtime(runtime),
        &run_id,
        prompt,
        |event| match event {
            SubmittedPromptStreamEvent::Agent(event) => {
                emit_runtime_event(*event, telemetry, agent_printer);
            }
            SubmittedPromptStreamEvent::Stdout(text) | SubmittedPromptStreamEvent::Stderr(text) => {
                agent_printer.print(text);
            }
        },
    )
    .await?;
    if let Some(message) = completion.warning.as_deref() {
        write_best_effort_stderr_line(message);
    }
    note_submitted_turn_completed(
        runtime,
        &run_id,
        if completion.exit_code == 0 {
            format!(
                "targeted follow-up turn completed for {}",
                runtime.descriptor.backend_id
            )
        } else {
            format!(
                "targeted follow-up turn exited with status {} for {}",
                completion.exit_code, runtime.descriptor.backend_id
            )
        },
        telemetry,
        agent_printer,
    )?;
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn submit_world_targeted_turn(
    runtime: &mut AsyncReplAgentRuntime,
    prompt: &str,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> Result<()> {
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use http_body_util::BodyExt as _;

    let run_id = Uuid::now_v7().to_string();
    note_submitted_turn_started(
        runtime,
        &run_id,
        format!(
            "submitted targeted follow-up turn to {}",
            runtime.descriptor.backend_id
        )
        .as_str(),
        telemetry,
        agent_printer,
    )?;
    let request = {
        let manifest_guard = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: manifest_guard.handle.orchestration_session_id.clone(),
            participant_id: manifest_guard.handle.participant_id.clone(),
            orchestrator_participant_id: manifest_guard
                .handle
                .orchestrator_participant_id
                .clone()
                .ok_or_else(|| {
                    anyhow!(
                        "substrate: error: retained world-scoped member is missing orchestrator_participant_id"
                    )
                })?,
            backend_id: runtime.descriptor.backend_id.clone(),
            run_id: run_id.clone(),
            world_id: manifest_guard.handle.world_id.clone().ok_or_else(|| {
                anyhow!("substrate: error: retained world-scoped member is missing world_id")
            })?,
            world_generation: manifest_guard.handle.world_generation.ok_or_else(|| {
                anyhow!(
                    "substrate: error: retained world-scoped member is missing world_generation"
                )
            })?,
            prompt: prompt.to_string(),
        }
    };
    let (client, _pending_diff_request, _agent_id) = build_agent_client_and_pending_diff_request()?;
    let response = client
        .submit_member_turn_stream(request)
        .await
        .map_err(|err| anyhow!("substrate: error: {err:#}"))?;

    let mut body = std::pin::pin!(response.into_body());
    let mut buffer = Vec::new();
    let mut exit_code = 0;
    while let Some(frame) = body.as_mut().frame().await {
        let frame = frame.map_err(|err| anyhow!("substrate: error: {err:#}"))?;
        let Some(data) = frame.data_ref() else {
            continue;
        };
        buffer.extend_from_slice(data);

        while let Some(pos) = buffer.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            if line.len() <= 1 {
                continue;
            }
            let payload = &line[..line.len() - 1];
            if payload.is_empty() {
                continue;
            }
            let frame = serde_json::from_slice::<ExecuteStreamFrame>(payload)
                .map_err(|err| anyhow!("substrate: error: {err:#}"))?;
            match frame {
                ExecuteStreamFrame::Start { .. } => {}
                ExecuteStreamFrame::Event { event } => {
                    handle_agent_event(event, telemetry, agent_printer);
                }
                ExecuteStreamFrame::Stdout { chunk_b64 } => {
                    let decoded = BASE64
                        .decode(chunk_b64.as_bytes())
                        .map_err(|err| anyhow!("substrate: error: {err:#}"))?;
                    agent_printer.print(String::from_utf8_lossy(&decoded).to_string());
                }
                ExecuteStreamFrame::Stderr { chunk_b64 } => {
                    let decoded = BASE64
                        .decode(chunk_b64.as_bytes())
                        .map_err(|err| anyhow!("substrate: error: {err:#}"))?;
                    agent_printer.print(String::from_utf8_lossy(&decoded).to_string());
                }
                ExecuteStreamFrame::Exit { exit, .. } => {
                    exit_code = exit;
                }
                ExecuteStreamFrame::Error { message } => {
                    return Err(anyhow!("substrate: error: {message}"));
                }
            }
        }
    }
    if exit_code != 0 {
        write_best_effort_stderr_line(&format!("Command failed with status: {exit_code}"));
    }
    note_submitted_turn_completed(
        runtime,
        &run_id,
        if exit_code == 0 {
            format!(
                "targeted follow-up turn completed for {}",
                runtime.descriptor.backend_id
            )
        } else {
            format!(
                "targeted follow-up turn exited with status {} for {}",
                exit_code, runtime.descriptor.backend_id
            )
        },
        telemetry,
        agent_printer,
    )?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
async fn submit_world_targeted_turn(
    _runtime: &mut AsyncReplAgentRuntime,
    _prompt: &str,
    _agent_printer: &ReplPrinter,
    _telemetry: &mut ReplSessionTelemetry,
) -> Result<()> {
    Err(anyhow!(
        "substrate: error: world-targeted follow-up turns are supported on Linux and macOS only"
    ))
}

async fn shutdown_host_orchestrator_runtime(
    runtime: AsyncReplAgentRuntime,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) {
    shutdown_host_orchestrator_runtime_with_mode(
        runtime,
        HostRuntimeShutdownMode::Stop,
        agent_printer,
        telemetry,
    )
    .await;
}

async fn shutdown_host_orchestrator_runtime_with_mode(
    mut runtime: AsyncReplAgentRuntime,
    mode: HostRuntimeShutdownMode,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) {
    let (run_id, should_attempt_stop, runtime_role) = {
        let guard = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        (
            guard
                .internal
                .latest_run_id
                .clone()
                .unwrap_or_else(|| Uuid::now_v7().to_string()),
            guard.internal.control_owner_retained || guard.internal.completion_observer_retained,
            guard.handle.role.clone(),
        )
    };
    let controls_parent_session = runtime_controls_parent_session(&runtime_role);
    if mode == HostRuntimeShutdownMode::ParkIfResumable
        && park_host_orchestrator_runtime(
            &mut runtime,
            controls_parent_session,
            &run_id,
            &runtime_role,
            agent_printer,
            telemetry,
        )
        .await
    {
        return;
    }

    if should_attempt_stop {
        let (orchestration_session, manifest) = {
            let mut orchestration_guard = runtime
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            let mut manifest_guard = runtime
                .manifest
                .lock()
                .expect("runtime manifest mutex poisoned");
            if manifest_guard.handle.state.is_live() {
                manifest_guard.transition_state(AgentRuntimeSessionState::Stopping);
                manifest_guard.touch_heartbeat();
            }
            if controls_parent_session
                && orchestration_guard.state == OrchestrationSessionState::Active
            {
                orchestration_guard.transition_state(OrchestrationSessionState::Stopping);
            }
            (orchestration_guard.clone(), manifest_guard.clone())
        };
        let _ = persist_runtime_snapshots(&runtime.store, &orchestration_session, &manifest);
        emit_runtime_event(
            build_runtime_message_event(
                &manifest,
                &orchestration_session,
                run_id.clone(),
                MessageEventKind::Status,
                runtime_stopping_message(&runtime_role, &runtime.uaa_session_handle_id),
            ),
            telemetry,
            agent_printer,
        );
    }

    if let Some(mut stop_transport) = runtime.stop_transport.take() {
        stop_transport.close().await;
    }
    if let Some(task) = runtime.stop_owner_task.take() {
        let _ = task.await;
    }
    if let Some(mut prompt_transport) = runtime.prompt_transport.take() {
        prompt_transport.close().await;
    }
    if let Some(task) = runtime.prompt_owner_task.take() {
        let _ = task.await;
    }
    runtime.shutdown_requested.store(true, Ordering::SeqCst);
    if let Some(stop_tx) = runtime.heartbeat_stop_tx.take() {
        let _ = stop_tx.send(());
    }
    let mut stop_failed = false;
    let mut completion_observed = false;
    match &mut runtime.retained_control {
        RetainedRunControl::Local(retained_control) => {
            retained_control.cancel.cancel();
            if let Some(task) = retained_control.completion_task.take() {
                match tokio::time::timeout(Duration::from_secs(5), task).await {
                    Ok(Ok(())) => completion_observed = true,
                    Ok(Err(_)) | Err(_) => stop_failed = true,
                }
            }
            if let Some(task) = retained_control.event_task.take() {
                let _ = tokio::time::timeout(Duration::from_secs(5), task).await;
            }
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        RetainedRunControl::Remote(retained_control) => {
            if retained_control
                .client
                .cancel_execute(ExecuteCancelRequestV1 {
                    span_id: retained_control.span_id.clone(),
                    sig: "INT".to_string(),
                })
                .await
                .is_err()
            {
                stop_failed = true;
            }
            if let Some(task) = retained_control.observe_task.take() {
                match tokio::time::timeout(Duration::from_secs(5), task).await {
                    Ok(Ok(())) => completion_observed = true,
                    Ok(Err(_)) | Err(_) => stop_failed = true,
                }
            }
        }
    }
    if let Some(task) = runtime.heartbeat_task.take() {
        let _ = task.await;
    }
    if stop_failed || (should_attempt_stop && !completion_observed) {
        let reason =
            "shell-owned orchestrator cancel did not produce authoritative terminal completion"
                .to_string();
        let (orchestration_session, manifest) = {
            let mut orchestration_guard = runtime
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            let mut manifest_guard = runtime
                .manifest
                .lock()
                .expect("runtime manifest mutex poisoned");
            manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
            manifest_guard.mark_terminal_state(reason.clone());
            manifest_guard.internal.last_error_bucket = Some("runtime_shutdown".to_string());
            manifest_guard.internal.last_error_message = Some(reason.clone());
            if controls_parent_session {
                orchestration_guard.transition_state(OrchestrationSessionState::Failed);
                orchestration_guard.mark_terminal(reason.clone());
            } else {
                orchestration_guard.touch_active();
            }
            (orchestration_guard.clone(), manifest_guard.clone())
        };
        let _ = persist_runtime_snapshots(&runtime.store, &orchestration_session, &manifest);
        let mut event = AgentEvent::alert(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id,
            runtime_invalidated_alert_code(&runtime_role),
            reason,
        );
        apply_runtime_participant_lineage(&mut event, &manifest);
        emit_runtime_event(event, telemetry, agent_printer);
        return;
    }

    let manifest = runtime
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .clone();
    if should_attempt_stop && manifest.handle.state == AgentRuntimeSessionState::Stopped {
        let orchestration_session = runtime
            .orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned")
            .clone();
        emit_runtime_event(
            build_runtime_message_event(
                &manifest,
                &orchestration_session,
                run_id,
                MessageEventKind::Status,
                runtime_stopped_message(&runtime_role),
            ),
            telemetry,
            agent_printer,
        );
    }
}

async fn park_host_orchestrator_runtime(
    runtime: &mut AsyncReplAgentRuntime,
    controls_parent_session: bool,
    run_id: &str,
    runtime_role: &str,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> bool {
    let can_park = {
        let manifest = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        controls_parent_session
            && manifest.is_host_orchestrator()
            && manifest.handle.state.is_live()
            && manifest.internal.uaa_session_id.is_some()
    };
    if !can_park {
        return false;
    }

    if let Some(mut stop_transport) = runtime.stop_transport.take() {
        stop_transport.close().await;
    }
    if let Some(task) = runtime.stop_owner_task.take() {
        task.abort();
        let _ = task.await;
    }
    if let Some(mut prompt_transport) = runtime.prompt_transport.take() {
        prompt_transport.close().await;
    }
    if let Some(task) = runtime.prompt_owner_task.take() {
        task.abort();
        let _ = task.await;
    }
    runtime.shutdown_requested.store(true, Ordering::SeqCst);
    if let Some(stop_tx) = runtime.heartbeat_stop_tx.take() {
        let _ = stop_tx.send(());
    }
    if let Some(task) = runtime.heartbeat_task.take() {
        task.abort();
        let _ = task.await;
    }

    match &mut runtime.retained_control {
        RetainedRunControl::Local(retained_control) => {
            if let Some(task) = retained_control.completion_task.take() {
                task.abort();
                let _ = task.await;
            }
            if let Some(task) = retained_control.event_task.take() {
                task.abort();
                let _ = task.await;
            }
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        RetainedRunControl::Remote(retained_control) => {
            if let Some(task) = retained_control.observe_task.take() {
                task.abort();
                let _ = task.await;
            }
        }
    }

    let reason = "owner detached cleanly".to_string();
    let Some((orchestration_session, manifest)) = ({
        let mut orchestration_guard = runtime
            .orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        let mut manifest_guard = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        apply_parked_host_runtime_snapshots(&mut orchestration_guard, &mut manifest_guard, &reason)
    }) else {
        return false;
    };
    let _ = persist_runtime_snapshots(&runtime.store, &orchestration_session, &manifest);
    emit_runtime_event(
        build_runtime_message_event(
            &manifest,
            &orchestration_session,
            run_id.to_string(),
            MessageEventKind::Status,
            runtime_detached_message(runtime_role),
        ),
        telemetry,
        agent_printer,
    );
    true
}

fn can_park_host_runtime_after_detach(
    store: &AgentRuntimeStateStore,
    controls_parent_session: bool,
    orchestration_session: &OrchestrationSessionRecord,
    manifest: &AgentRuntimeSessionManifest,
    runtime_owns_private_stop: bool,
    owner_helper_exited: bool,
) -> bool {
    if !controls_parent_session
        || !manifest.is_host_orchestrator()
        || !manifest.handle.state.is_live()
        || manifest.internal.uaa_session_id.is_none()
    {
        return false;
    }

    if !runtime_owns_private_stop {
        return true;
    }
    if !owner_helper_exited {
        return false;
    }

    let Ok(Some(record)) = store.load_session(&orchestration_session.orchestration_session_id)
    else {
        return false;
    };
    let participant_id = manifest.handle.participant_id.as_str();
    if record.session.state != OrchestrationSessionState::Active
        || record.session.posture != OrchestrationSessionPosture::ActiveAttached
        || record.session.active_participant_id() != Some(participant_id)
        || record.session.attached_participant_id() != Some(participant_id)
    {
        return false;
    }

    !record.participants.iter().any(|participant| {
        participant.participant_id() != participant_id
            && participant.matches_public_parent_linkage(&record.session)
            && participant.is_host_orchestrator()
            && participant.attached_client_present()
            && participant.is_authoritative_live()
            && shell_owner_pid_is_alive(participant.internal.shell_owner_pid)
    })
}

fn build_parked_host_runtime_snapshots(
    orchestration_guard: &OrchestrationSessionRecord,
    manifest_guard: &AgentRuntimeSessionManifest,
    reason: &str,
) -> Option<(OrchestrationSessionRecord, AgentRuntimeSessionManifest)> {
    let mut parked_orchestration = orchestration_guard.clone();
    let mut parked_manifest = manifest_guard.clone();
    parked_orchestration.shell_owner_pid = 0;
    parked_manifest.release_runtime_ownership();
    parked_manifest.mark_client_detached(reason.to_string());
    parked_manifest.internal.shell_owner_pid = 0;
    parked_manifest.touch_heartbeat();
    parked_orchestration.transition_state(OrchestrationSessionState::Active);
    match if parked_orchestration.pending_inbox_count > 0 {
        OrchestrationSessionPosture::AwaitingAttention
    } else {
        OrchestrationSessionPosture::ParkedResumable
    } {
        OrchestrationSessionPosture::ParkedResumable => {
            parked_orchestration.mark_parked_resumable(reason.to_string());
        }
        OrchestrationSessionPosture::AwaitingAttention => {
            parked_orchestration.mark_awaiting_attention();
        }
        OrchestrationSessionPosture::ActiveAttached | OrchestrationSessionPosture::Terminal => {
            unreachable!("detached host parking only normalizes to parked or attention posture")
        }
    }

    valid_detached_host_continuity_posture(&parked_orchestration, &parked_manifest, true)?;
    Some((parked_orchestration, parked_manifest))
}

fn apply_parked_host_runtime_snapshots(
    orchestration_guard: &mut OrchestrationSessionRecord,
    manifest_guard: &mut AgentRuntimeSessionManifest,
    reason: &str,
) -> Option<(OrchestrationSessionRecord, AgentRuntimeSessionManifest)> {
    let (next_orchestration, next_manifest) =
        build_parked_host_runtime_snapshots(orchestration_guard, manifest_guard, reason)?;
    *orchestration_guard = next_orchestration.clone();
    *manifest_guard = next_manifest.clone();
    Some((next_orchestration, next_manifest))
}

#[cfg(unix)]
fn shell_owner_pid_is_alive(pid: u32) -> bool {
    let pid = pid as libc::pid_t;
    if pid <= 0 {
        return false;
    }

    let rc = unsafe { libc::kill(pid, 0) };
    if rc == 0 {
        return true;
    }

    matches!(io::Error::last_os_error().raw_os_error(), Some(libc::EPERM))
}

#[cfg(not(unix))]
fn shell_owner_pid_is_alive(pid: u32) -> bool {
    pid == std::process::id()
}

async fn abort_bootstrap_runtime(
    shutdown_requested: &Arc<AtomicBool>,
    retained_control: &mut LocalRetainedRunControl,
    heartbeat_stop_tx: &mut Option<tokio::sync::oneshot::Sender<()>>,
    heartbeat_task: &mut Option<tokio::task::JoinHandle<()>>,
) {
    shutdown_requested.store(true, Ordering::SeqCst);
    if let Some(stop_tx) = heartbeat_stop_tx.take() {
        let _ = stop_tx.send(());
    }
    retained_control.cancel.cancel();
    if let Some(task) = retained_control.completion_task.take() {
        let _ = task.await;
    }
    if let Some(task) = retained_control.event_task.take() {
        let _ = task.await;
    }
    if let Some(task) = heartbeat_task.take() {
        let _ = task.await;
    }
}

fn signal_runtime_startup(
    signal: &Arc<Mutex<Option<tokio::sync::oneshot::Sender<RuntimeStartupSignal>>>>,
    value: RuntimeStartupSignal,
) {
    if let Some(tx) = signal
        .lock()
        .expect("runtime startup signal mutex poisoned")
        .take()
    {
        let _ = tx.send(value);
    }
}

async fn finalize_runtime_startup_failure(
    startup_context: Option<&RuntimeOrchestrationContext>,
    world_session: &mut Option<WorldSession>,
    failure_message: &str,
) {
    let mut close_succeeded = false;
    if let Some(session) = world_session.take() {
        close_succeeded = session.client.close().await.is_ok();
    }

    let Some(startup_context) = startup_context else {
        return;
    };

    mark_orchestration_session_failed(
        &startup_context.store,
        &startup_context.orchestration_session,
        failure_message.to_string(),
    );
    if close_succeeded {
        let _ = persist_world_binding_authority(
            &startup_context.store,
            &startup_context.orchestration_session,
            None,
        );
    }
}

fn runtime_loop_exit(err: &anyhow::Error) -> (i32, String) {
    if is_world_restart_required_error(err) {
        return (3, err.to_string());
    }

    let message = format!("{err:#}");
    if message.starts_with("substrate: error:") {
        (1, message)
    } else {
        (1, format!("substrate: error: {message}"))
    }
}

fn runtime_bootstrap_failure_from_anyhow(err: anyhow::Error) -> RuntimeBootstrapFailure {
    if crate::execution::config_model::is_user_error(&err) {
        RuntimeBootstrapFailure {
            exit_code: 2,
            message: err.to_string(),
        }
    } else {
        RuntimeBootstrapFailure {
            exit_code: 1,
            message: format!("{err:#}"),
        }
    }
}

fn runtime_bootstrap_failure_from_wrapper_error(
    err: agent_api::AgentWrapperError,
) -> RuntimeBootstrapFailure {
    match err {
        agent_api::AgentWrapperError::UnknownBackend { agent_kind }
        | agent_api::AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability: _,
        } => RuntimeBootstrapFailure {
            exit_code: 2,
            message: format!("failed to bootstrap shell-owned orchestrator runtime: {agent_kind}"),
        },
        agent_api::AgentWrapperError::InvalidAgentKind { message }
        | agent_api::AgentWrapperError::InvalidRequest { message } => RuntimeBootstrapFailure {
            exit_code: 2,
            message,
        },
        agent_api::AgentWrapperError::Backend { message } => RuntimeBootstrapFailure {
            exit_code: if message.to_ascii_lowercase().contains("timeout") {
                3
            } else {
                4
            },
            message,
        },
    }
}

fn translate_wrapper_event(
    manifest: &AgentRuntimeSessionManifest,
    orchestration_session: &OrchestrationSessionRecord,
    run_id: &str,
    wrapper_event: agent_api::AgentWrapperEvent,
) -> AgentEvent {
    let mut event = match wrapper_event.kind {
        agent_api::AgentWrapperEventKind::Status => AgentEvent::message(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            MessageEventKind::Status,
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "agent runtime status".to_string()),
        ),
        agent_api::AgentWrapperEventKind::TextOutput => AgentEvent::message(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            MessageEventKind::TaskProgress,
            wrapper_event
                .text
                .clone()
                .unwrap_or_else(|| "agent runtime output".to_string()),
        ),
        agent_api::AgentWrapperEventKind::ToolCall
        | agent_api::AgentWrapperEventKind::ToolResult => AgentEvent::message(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            MessageEventKind::TaskProgress,
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "agent runtime tool activity".to_string()),
        ),
        agent_api::AgentWrapperEventKind::Error => AgentEvent::alert(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            "agent_wrapper_error",
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "agent runtime error".to_string()),
        ),
        agent_api::AgentWrapperEventKind::Unknown => AgentEvent::message(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            MessageEventKind::TaskProgress,
            "agent runtime emitted an unknown event".to_string(),
        ),
    };

    event.role = Some(manifest.handle.role.clone());
    event.backend_id = Some(manifest.handle.backend_id.clone());
    event.set_pure_agent_telemetry_identity(manifest.handle.agent_id.clone());
    event.set_channel(wrapper_event.channel.clone());
    event.world_id = orchestration_session.world_id.clone();
    event.world_generation = orchestration_session.world_generation;
    apply_runtime_participant_lineage(&mut event, manifest);

    if let Some(data) = wrapper_event.data {
        if let Some(obj) = event.data.as_object_mut() {
            obj.insert("uaa_event".to_string(), data);
            obj.insert(
                "protocol".to_string(),
                serde_json::json!(PURE_AGENT_PROTOCOL),
            );
        }
    }

    event
}

fn apply_runtime_participant_lineage(
    event: &mut AgentEvent,
    manifest: &AgentRuntimeSessionManifest,
) {
    event.participant_id = Some(manifest.handle.participant_id.clone());
    event.parent_participant_id = manifest.handle.parent_participant_id.clone();
    event.resumed_from_participant_id = manifest.handle.resumed_from_participant_id.clone();
}

fn build_runtime_message_event(
    manifest: &AgentRuntimeSessionManifest,
    orchestration_session: &OrchestrationSessionRecord,
    run_id: String,
    kind: MessageEventKind,
    message: impl Into<String>,
) -> AgentEvent {
    let mut event = AgentEvent::message(
        manifest.handle.agent_id.clone(),
        manifest.handle.orchestration_session_id.clone(),
        run_id,
        kind,
        message.into(),
    );
    event.role = Some(manifest.handle.role.clone());
    event.backend_id = Some(manifest.handle.backend_id.clone());
    event.set_pure_agent_telemetry_identity(manifest.handle.agent_id.clone());
    event.world_id = orchestration_session.world_id.clone();
    event.world_generation = orchestration_session.world_generation;
    apply_runtime_participant_lineage(&mut event, manifest);
    event
}

fn emit_runtime_event(
    event: AgentEvent,
    telemetry: &mut ReplSessionTelemetry,
    agent_printer: &ReplPrinter,
) {
    handle_agent_event(event, telemetry, agent_printer);
}

fn extract_session_handle_id(data: Option<&serde_json::Value>) -> Option<&str> {
    let value = data?;
    if value.get("schema").and_then(serde_json::Value::as_str) == Some(SESSION_HANDLE_SCHEMA_V1) {
        return value
            .get("session")
            .and_then(serde_json::Value::as_object)
            .and_then(|session| session.get("id"))
            .and_then(serde_json::Value::as_str)
            .filter(|id| !id.trim().is_empty());
    }

    value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .filter(|event_type| matches!(*event_type, "thread.started" | "turn.started"))?;
    value
        .get("thread_id")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.trim().is_empty())
}

fn report_nonzero_status(status: &ExitStatus) {
    if status.success() {
        return;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            write_best_effort_stderr_line(&format!("Command terminated by signal {sig}"));
            return;
        }
    }

    write_best_effort_stderr_line(&format!(
        "Command failed with status: {}",
        status.code().unwrap_or(-1)
    ));
}

fn parse_targeted_turn(input: &str) -> Option<TargetedTurn<'_>> {
    if has_embedded_newlines(input) {
        return None;
    }

    let rest = input.strip_prefix("::")?;
    let (backend_id, prompt) = rest.split_once(' ')?;
    if backend_id.is_empty() || prompt.is_empty() {
        return None;
    }

    Some(TargetedTurn { backend_id, prompt })
}

fn parse_demo_burst(input: &str) -> Option<(usize, usize, u64)> {
    let rest = input.strip_prefix(":demo-burst")?.trim();
    if rest.is_empty() {
        return Some((4, 400, 0));
    }

    let mut parts = rest.split_whitespace();
    let agents = parts
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(4);
    let events = parts
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(400);
    let delay_ms = parts
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    Some((agents, events, delay_ms))
}

type StdoutCallback = Arc<dyn Fn(&[u8]) + Send + Sync>;

struct HostState {
    cwd: PathBuf,
    env: HashMap<String, String>,
}

impl HostState {
    fn new() -> Result<Self> {
        Ok(Self {
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env: std::env::vars().collect::<HashMap<String, String>>(),
        })
    }
}

struct WorldSession {
    client: ReplPersistentSessionClient,
    world_id: String,
    world_generation: u64,
    world_cwd: String,
    snapshot_hash: String,
    workspace_root: Option<PathBuf>,
    on_stdout: StdoutCallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorldRestartReason {
    PolicySnapshotChanged,
    WorkspaceRootChanged,
}

impl WorldRestartReason {
    fn code(self) -> &'static str {
        match self {
            Self::PolicySnapshotChanged => "policy_snapshot_changed",
            Self::WorkspaceRootChanged => "workspace_root_changed",
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::PolicySnapshotChanged => "world restarted due to policy snapshot drift",
            Self::WorkspaceRootChanged => "world restarted due to workspace root drift",
        }
    }

    fn restart_required_message(self) -> &'static str {
        match self {
            Self::PolicySnapshotChanged => "world restart required due to policy snapshot drift",
            Self::WorkspaceRootChanged => "world restart required due to workspace root drift",
        }
    }
}

#[derive(Debug)]
struct WorldRestartRequiredError {
    message: String,
}

impl WorldRestartRequiredError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WorldRestartRequiredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorldRestartRequiredError {}

pub(crate) fn is_world_restart_required_error(err: &anyhow::Error) -> bool {
    err.chain()
        .any(|cause| cause.is::<WorldRestartRequiredError>())
}

struct ReplIo<'a> {
    agent_rx: &'a mut UnboundedReceiver<AgentEvent>,
    resize_rx: &'a mut UnboundedReceiver<(u16, u16)>,
    sigint_rx: &'a mut UnboundedReceiver<()>,
    telemetry: &'a mut ReplSessionTelemetry,
    agent_printer: &'a ReplPrinter,
    max_pty_buffered_lines: usize,
}

fn has_embedded_newlines(s: &str) -> bool {
    // Reedline input should not include trailing line terminators, but PTY harnesses (and some
    // terminals) can surface a trailing `\r`/`\n`. We only want to treat *embedded* newlines as
    // multi-line commands that disable certain REPL directives like `:pty`/`:host`.
    let trimmed = s.trim_end_matches(['\r', '\n']);
    trimmed.contains('\n') || trimmed.contains('\r')
}

fn apply_anchor_env_for_cwd(env: &mut HashMap<String, String>, cwd: &Path) -> Result<()> {
    let resolved = resolve_world_root(None, None, None, cwd)
        .context("resolve world root settings for session env")?;
    env.insert(
        "SUBSTRATE_ANCHOR_MODE".to_string(),
        resolved.mode.as_str().to_string(),
    );
    env.insert(
        "SUBSTRATE_ANCHOR_PATH".to_string(),
        resolved.path.to_string_lossy().to_string(),
    );
    Ok(())
}

fn classify_world_restart_reason(
    previous_snapshot_hash: &str,
    previous_workspace_root: &Option<PathBuf>,
    next_snapshot_hash: &str,
    next_workspace_root: &Option<PathBuf>,
) -> Option<WorldRestartReason> {
    if previous_workspace_root != next_workspace_root {
        Some(WorldRestartReason::WorkspaceRootChanged)
    } else if previous_snapshot_hash != next_snapshot_hash {
        Some(WorldRestartReason::PolicySnapshotChanged)
    } else {
        None
    }
}

fn emit_world_restarted_alert(
    orchestration_session_id: Option<&str>,
    previous_world_id: &str,
    previous_world_generation: u64,
    new_world_id: &str,
    new_world_generation: u64,
    reason: WorldRestartReason,
) {
    let Some(orchestration_session_id) = orchestration_session_id else {
        return;
    };
    let mut event = AgentEvent::alert(
        "shell",
        orchestration_session_id.to_string(),
        Uuid::now_v7().to_string(),
        "world_restarted",
        reason.message(),
    );
    event.role = Some("orchestrator".to_string());
    event.backend_id = Some("shell:repl".to_string());
    event.world_id = Some(new_world_id.to_string());
    event.world_generation = Some(new_world_generation);
    event.set_pure_agent_telemetry_identity("shell");

    if let Some(data) = event.data.as_object_mut() {
        data.insert("reason".to_string(), serde_json::json!(reason.code()));
        data.insert("on_drift".to_string(), serde_json::json!("auto_restart"));
        data.insert(
            "previous_world_id".to_string(),
            serde_json::json!(previous_world_id),
        );
        data.insert("new_world_id".to_string(), serde_json::json!(new_world_id));
        data.insert(
            "previous_world_generation".to_string(),
            serde_json::json!(previous_world_generation),
        );
        data.insert(
            "new_world_generation".to_string(),
            serde_json::json!(new_world_generation),
        );
    }

    let _ = publish_agent_event(event);
}

fn build_world_restart_required_alert(
    orchestration_session_id: Option<&str>,
    current_world_id: &str,
    current_world_generation: u64,
    reason: WorldRestartReason,
) -> Option<AgentEvent> {
    let orchestration_session_id = orchestration_session_id?;
    let mut event = AgentEvent::alert(
        "shell",
        orchestration_session_id.to_string(),
        Uuid::now_v7().to_string(),
        "world_restart_required",
        reason.restart_required_message(),
    );
    event.role = Some("orchestrator".to_string());
    event.backend_id = Some("shell:repl".to_string());
    event.world_id = Some(current_world_id.to_string());
    event.world_generation = Some(current_world_generation);
    event.set_pure_agent_telemetry_identity("shell");

    if let Some(data) = event.data.as_object_mut() {
        data.insert("reason".to_string(), serde_json::json!(reason.code()));
        data.insert(
            "required_action".to_string(),
            serde_json::json!("restart_world"),
        );
        data.insert("on_drift".to_string(), serde_json::json!("fail_closed"));
    }

    Some(event)
}

fn resolve_world_restart_on_drift(
    cwd: &Path,
) -> Result<crate::execution::config_model::WorldRestartOnDriftMode> {
    Ok(crate::execution::config_model::resolve_effective_config(
        cwd,
        &crate::execution::config_model::CliConfigOverrides::default(),
    )?
    .agents
    .hub
    .world_restart
    .on_drift)
}

struct WorldDriftRequest<'a> {
    requested_cwd: String,
    startup_context: Option<&'a RuntimeOrchestrationContext>,
    live_runtime_established: bool,
    policy_snapshot: agent_api_types::PolicySnapshotV3,
    snapshot_hash: String,
    workspace_root: Option<PathBuf>,
    agent_printer: &'a ReplPrinter,
    telemetry: &'a mut ReplSessionTelemetry,
    reason: WorldRestartReason,
}

async fn handle_detected_world_drift(
    old_session: WorldSession,
    request: WorldDriftRequest<'_>,
) -> Result<WorldSession> {
    let WorldDriftRequest {
        requested_cwd,
        startup_context,
        live_runtime_established,
        policy_snapshot,
        snapshot_hash,
        workspace_root,
        agent_printer,
        telemetry,
        reason,
    } = request;
    let on_drift = resolve_world_restart_on_drift(Path::new(&requested_cwd))?;
    match on_drift {
        crate::execution::config_model::WorldRestartOnDriftMode::AutoRestart => {
            restart_world_session(
                old_session,
                WorldDriftRequest {
                    requested_cwd,
                    startup_context,
                    live_runtime_established,
                    policy_snapshot,
                    snapshot_hash,
                    workspace_root,
                    agent_printer,
                    telemetry,
                    reason,
                },
            )
            .await
        }
        crate::execution::config_model::WorldRestartOnDriftMode::FailClosed => {
            let persisted_snapshot = if let Some(startup_context) = startup_context {
                let current_binding = PersistedWorldBinding {
                    world_id: old_session.world_id.clone(),
                    world_generation: old_session.world_generation,
                };
                Some(persist_world_binding_authority(
                    &startup_context.store,
                    &startup_context.orchestration_session,
                    Some(&current_binding),
                )?)
            } else {
                None
            };
            let startup_orchestration_session_id =
                startup_context.map(RuntimeOrchestrationContext::orchestration_session_id);
            let orchestration_session_id = persisted_snapshot
                .as_ref()
                .map(|snapshot| snapshot.orchestration_session_id.as_str())
                .or(startup_orchestration_session_id.as_deref());
            let current_world_id = persisted_snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.world_id.as_deref())
                .unwrap_or(old_session.world_id.as_str());
            let current_world_generation = persisted_snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.world_generation)
                .unwrap_or(old_session.world_generation);
            if let Some(alert) = build_world_restart_required_alert(
                orchestration_session_id,
                current_world_id,
                current_world_generation,
                reason,
            ) {
                telemetry.persist_agent_event(&alert);
                telemetry.record_agent_event();
                // This path exits immediately after surfacing the alert. Reedline's external
                // printer can drop queued lines during that shutdown, so write directly to stdout.
                write_best_effort_stdout_line(&format_event_line(&alert));
            } else {
                write_best_effort_stdout_line(&format!(
                    "[shell] {}",
                    reason.restart_required_message()
                ));
            }
            let close_succeeded = old_session.client.close().await.is_ok();
            if !live_runtime_established && close_succeeded {
                if let Some(startup_context) = startup_context {
                    let _ = persist_world_binding_authority(
                        &startup_context.store,
                        &startup_context.orchestration_session,
                        None,
                    );
                }
            }
            Err(anyhow!(WorldRestartRequiredError::new(format!(
                "substrate: error: world restart required before continuing ({}, world_id={}, generation={})",
                reason.code(),
                current_world_id,
                current_world_generation,
            ))))
        }
    }
}

struct OpenWorldSessionRequest<'a> {
    requested_cwd: String,
    requested_path: &'a Path,
    resolved_policy_snapshot: agent_api_types::PolicySnapshotV3,
    shared_world_request: Option<agent_api_types::SharedWorldOwnerSpec>,
    snapshot_hash: String,
    workspace_root: Option<PathBuf>,
    on_stdout: StdoutCallback,
    agent_printer: &'a ReplPrinter,
    world_generation: u64,
    restarted: bool,
}

async fn open_world_session(request: OpenWorldSessionRequest<'_>) -> Result<WorldSession> {
    let OpenWorldSessionRequest {
        requested_cwd,
        requested_path,
        resolved_policy_snapshot,
        shared_world_request,
        snapshot_hash,
        workspace_root,
        on_stdout,
        agent_printer,
        world_generation,
        restarted,
    } = request;
    let world_network_policy = policy_snapshot::resolve_world_network_policy_for_snapshot(
        resolved_policy_snapshot,
        requested_path,
    )
    .context("world network policy")?;
    let world_network = policy_snapshot::request_world_network_routing(&world_network_policy);
    let (mut start_params, inherit_from_host) = ReplSessionStartParams::for_cwd_and_snapshot(
        requested_cwd.clone(),
        requested_path,
        world_network_policy.snapshot,
        world_network,
    )?;
    if inherit_from_host {
        agent_printer.print("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    apply_anchor_env_for_cwd(&mut start_params.env, requested_path)?;
    start_params.shared_world = shared_world_request;
    let client = ReplPersistentSessionClient::start_with(start_params, on_stdout.clone()).await?;
    let ready = client.ready().clone();
    let world_generation = ready
        .shared_world
        .as_ref()
        .map(|binding| binding.world_generation)
        .unwrap_or(world_generation);

    if ready.cwd != requested_cwd {
        let action = if restarted { "restarted" } else { "started" };
        agent_printer.print(format!(
            "substrate: note: world session {action} in {} (requested {})",
            ready.cwd, requested_cwd
        ));
    }

    Ok(WorldSession {
        client,
        world_id: ready.world_id,
        world_generation,
        world_cwd: ready.cwd,
        snapshot_hash,
        workspace_root,
        on_stdout,
    })
}

async fn restart_world_session(
    old_session: WorldSession,
    request: WorldDriftRequest<'_>,
) -> Result<WorldSession> {
    let WorldDriftRequest {
        requested_cwd,
        startup_context,
        live_runtime_established: _live_runtime_established,
        policy_snapshot,
        snapshot_hash,
        workspace_root,
        agent_printer,
        telemetry: _telemetry,
        reason,
    } = request;
    let requested_path = PathBuf::from(&requested_cwd);
    let on_stdout = old_session.on_stdout.clone();
    let previous_world_id = old_session.world_id.clone();
    let previous_world_generation = old_session.world_generation;
    let orchestration_session_id =
        startup_context.map(RuntimeOrchestrationContext::orchestration_session_id);
    let shared_world_request =
        orchestration_session_id
            .clone()
            .map(|id| agent_api_types::SharedWorldOwnerSpec {
                orchestration_session_id: id,
                action: agent_api_types::SharedWorldOwnerAction::ReplaceExpectedGeneration {
                    expected_generation: previous_world_generation,
                    reason: reason.message().to_string(),
                },
            });

    old_session.client.close().await?;

    let new_session = open_world_session(OpenWorldSessionRequest {
        requested_cwd,
        requested_path: requested_path.as_path(),
        resolved_policy_snapshot: policy_snapshot,
        shared_world_request,
        snapshot_hash,
        workspace_root,
        on_stdout,
        agent_printer,
        world_generation: previous_world_generation.saturating_add(1),
        restarted: true,
    })
    .await?;

    let mut authoritative_world_id = new_session.world_id.clone();
    let mut authoritative_world_generation = new_session.world_generation;
    if let Some(startup_context) = startup_context {
        let next_binding = PersistedWorldBinding {
            world_id: new_session.world_id.clone(),
            world_generation: new_session.world_generation,
        };
        let persisted_snapshot = persist_world_binding_authority(
            &startup_context.store,
            &startup_context.orchestration_session,
            Some(&next_binding),
        )?;
        if let Some(world_id) = persisted_snapshot.world_id {
            authoritative_world_id = world_id;
        }
        if let Some(world_generation) = persisted_snapshot.world_generation {
            authoritative_world_generation = world_generation;
        }

        if let Err(err) = invalidate_stale_world_members_after_binding(
            &startup_context.store,
            &startup_context.orchestration_session_id(),
            authoritative_world_generation,
        ) {
            let _ = new_session.client.close().await;
            return Err(err).context(
                "failed to invalidate stale world-scoped participants after replacement binding persistence",
            );
        }
    }
    emit_world_restarted_alert(
        orchestration_session_id.as_deref(),
        &previous_world_id,
        previous_world_generation,
        &authoritative_world_id,
        authoritative_world_generation,
        reason,
    );

    Ok(new_session)
}

fn is_exit_directive(trimmed: &str) -> bool {
    if trimmed == "quit" || trimmed == "exit" {
        return true;
    }
    if let Some(rest) = trimmed.strip_prefix("exit ") {
        return rest.trim().parse::<i32>().is_ok();
    }
    false
}

fn make_world_stdout_callback(
    prompt_active: Arc<AtomicBool>,
    printer: Option<ExternalPrinter<String>>,
) -> StdoutCallback {
    Arc::new(move |bytes: &[u8]| {
        if prompt_active.load(Ordering::SeqCst) {
            if let Some(printer) = printer.as_ref() {
                if printer
                    .print(String::from_utf8_lossy(bytes).to_string())
                    .is_ok()
                {
                    return;
                }
            }
        }

        let mut stdout = io::stdout();
        let _ = stdout.write_all(bytes);
        let _ = stdout.flush();
    })
}

fn preflight_caging_required(config: &ShellConfig) -> Result<ReplPreflight> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let entered_cwd = cwd.display().to_string();
    let cli_world_enabled = if config.cli_world {
        Some(true)
    } else if config.cli_no_world {
        Some(false)
    } else {
        None
    };

    let effective_config = crate::execution::config_model::resolve_effective_config(
        &cwd,
        &crate::execution::config_model::CliConfigOverrides {
            world_enabled: cli_world_enabled,
            anchor_mode: config.cli_anchor_mode,
            anchor_path: config
                .cli_anchor_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            caged: config.cli_caged,
        },
    )?;

    let exit_cwd = effective_config.repl.exit_cwd;
    let max_pty_buffered_lines =
        usize::try_from(effective_config.repl.max_pty_buffered_lines).unwrap_or_default();
    let max_pty_buffered_lines_clamp = effective_config.repl.max_pty_buffered_lines_clamp;

    let policy_mode = effective_config.policy.mode;
    std::env::set_var("SUBSTRATE_POLICY_MODE", policy_mode.as_str());
    crate::execution::export_runtime_config_env(&effective_config);
    substrate_broker::set_policy_mode(match policy_mode {
        crate::execution::config_model::PolicyMode::Disabled => {
            substrate_broker::PolicyMode::Disabled
        }
        crate::execution::config_model::PolicyMode::Observe => {
            substrate_broker::PolicyMode::Observe
        }
        crate::execution::config_model::PolicyMode::Enforce => {
            substrate_broker::PolicyMode::Enforce
        }
    });

    detect_profile(&cwd)
        .with_context(|| format!("failed to load Substrate profile for cwd {}", cwd.display()))
        .map_err(|err| crate::execution::config_model::user_error(format!("{:#}", err)))?;

    let world_fs = world_fs_policy();
    if world_fs.caged_required {
        if !effective_config.world.caged {
            return Err(crate::execution::config_model::user_error(
                "world_fs.caged_required=true requires world.caged=true (uncaged mode is a hard error)",
            ));
        }
        if effective_config.world.anchor_mode == substrate_common::WorldRootMode::FollowCwd {
            return Err(crate::execution::config_model::user_error(
                "world_fs.caged_required=true is incompatible with world.anchor_mode=follow-cwd (hard error)",
            ));
        }
    }

    Ok(ReplPreflight {
        entered_cwd,
        exit_cwd,
        max_pty_buffered_lines,
        max_pty_buffered_lines_clamp,
    })
}

fn spawn_sigint_task(sigint_tx: UnboundedSender<()>) {
    tokio::spawn(async move {
        loop {
            if tokio::signal::ctrl_c().await.is_err() {
                break;
            }
            let _ = sigint_tx.send(());
        }
    });
}

fn spawn_resize_task(resize_tx: UnboundedSender<(u16, u16)>) {
    #[cfg(unix)]
    {
        tokio::spawn(async move {
            use tokio::signal::unix::{signal, SignalKind};

            let mut sigwinch = match signal(SignalKind::window_change()) {
                Ok(s) => s,
                Err(_) => return,
            };
            while sigwinch.recv().await.is_some() {
                let (cols, rows) = match get_terminal_size() {
                    Ok(sz) if sz.cols > 0 && sz.rows > 0 => (sz.cols, sz.rows),
                    _ => (80, 24),
                };
                let _ = resize_tx.send((cols, rows));
            }
        });
    }

    #[cfg(not(unix))]
    {
        let _ = resize_tx;
    }
}

async fn start_world_session(
    requested_cwd: String,
    startup_context: Option<&RuntimeOrchestrationContext>,
    on_stdout: StdoutCallback,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> Result<WorldSession> {
    let requested_path = Path::new(&requested_cwd);
    let resolved_start = policy_snapshot::resolve_policy_snapshot_for_cwd(requested_path)
        .context("policy snapshot (start)")?;
    let start_hash = resolved_start.snapshot_hash.clone();
    let start_workspace_root = find_workspace_root(requested_path);
    let shared_world_request =
        startup_context.map(|context| agent_api_types::SharedWorldOwnerSpec {
            orchestration_session_id: context.orchestration_session_id(),
            action: agent_api_types::SharedWorldOwnerAction::AttachOrCreate,
        });
    let session = open_world_session(OpenWorldSessionRequest {
        requested_cwd: requested_cwd.clone(),
        requested_path,
        resolved_policy_snapshot: resolved_start.snapshot,
        shared_world_request,
        snapshot_hash: start_hash.clone(),
        workspace_root: start_workspace_root.clone(),
        on_stdout,
        agent_printer,
        world_generation: 0,
        restarted: false,
    })
    .await?;

    let ready_cwd = session.world_cwd.clone();
    let ready_path = Path::new(&ready_cwd);
    let resolved_ready = policy_snapshot::resolve_policy_snapshot_for_cwd(ready_path)
        .context("policy snapshot (ready.cwd)")?;
    let ready_hash = resolved_ready.snapshot_hash.clone();
    let ready_workspace_root = find_workspace_root(ready_path);

    if let Some(reason) = classify_world_restart_reason(
        &start_hash,
        &start_workspace_root,
        &ready_hash,
        &ready_workspace_root,
    ) {
        let on_drift = resolve_world_restart_on_drift(ready_path)?;
        let note = match on_drift {
            crate::execution::config_model::WorldRestartOnDriftMode::AutoRestart => {
                "substrate: note: world session restarting due to snapshot/workspace drift before first command"
            }
            crate::execution::config_model::WorldRestartOnDriftMode::FailClosed => {
                "substrate: note: world session detected snapshot/workspace drift before first command and requires operator restart"
            }
        };
        agent_printer.print(note);
        return handle_detected_world_drift(
            session,
            WorldDriftRequest {
                requested_cwd: ready_cwd,
                startup_context,
                live_runtime_established: false,
                policy_snapshot: resolved_ready.snapshot,
                snapshot_hash: ready_hash,
                workspace_root: ready_workspace_root,
                agent_printer,
                telemetry,
                reason,
            },
        )
        .await;
    }

    Ok(session)
}

async fn ensure_no_policy_drift(
    world_session: &mut Option<WorldSession>,
    startup_context: Option<&RuntimeOrchestrationContext>,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> Result<()> {
    let Some(session) = world_session.as_ref() else {
        return Ok(());
    };

    let current_world_cwd = session.world_cwd.clone();
    let current_world_path = PathBuf::from(&current_world_cwd);
    let resolved = policy_snapshot::resolve_policy_snapshot_for_cwd(current_world_path.as_path())
        .context("policy snapshot (drift)")?;
    let workspace_root = find_workspace_root(current_world_path.as_path());

    let Some(reason) = classify_world_restart_reason(
        &session.snapshot_hash,
        &session.workspace_root,
        &resolved.snapshot_hash,
        &workspace_root,
    ) else {
        return Ok(());
    };

    let old = world_session
        .take()
        .expect("world_session present if session was Some above");
    let requested = old.world_cwd.clone();
    let on_drift = resolve_world_restart_on_drift(current_world_path.as_path())?;
    let note = match on_drift {
        crate::execution::config_model::WorldRestartOnDriftMode::AutoRestart => {
            "substrate: note: world session restarting due to snapshot/workspace drift"
        }
        crate::execution::config_model::WorldRestartOnDriftMode::FailClosed => {
            "substrate: note: world session detected snapshot/workspace drift and requires operator restart"
        }
    };
    agent_printer.print(note);

    *world_session = Some(
        handle_detected_world_drift(
            old,
            WorldDriftRequest {
                requested_cwd: requested,
                startup_context,
                live_runtime_established: true,
                policy_snapshot: resolved.snapshot,
                snapshot_hash: resolved.snapshot_hash,
                workspace_root,
                agent_printer,
                telemetry,
                reason,
            },
        )
        .await?,
    );
    Ok(())
}

fn predict_cd_next_cwd(current_cwd: &str, program: &str) -> Option<String> {
    let trimmed = program.trim();
    let rest = trimmed.strip_prefix("cd")?;
    let arg = rest.trim();
    if arg.is_empty() {
        return None;
    }

    if !current_cwd.starts_with('/') {
        return None;
    }

    let next = if arg.starts_with('/') {
        PathBuf::from(arg)
    } else {
        PathBuf::from(current_cwd).join(arg)
    };

    let mut normalized = PathBuf::new();
    for comp in next.components() {
        match comp {
            std::path::Component::RootDir => normalized.push("/"),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(seg) => {
                normalized.push(seg);
            }
            std::path::Component::Prefix(_) => {}
        }
    }
    if normalized.as_os_str().is_empty() {
        normalized.push("/");
    }

    Some(normalized.to_string_lossy().to_string())
}

fn apply_caged_predicted_cwd(
    world_root: &WorldRootSettings,
    current_cwd: &str,
    predicted: String,
) -> (String, Option<String>) {
    let current_path = Path::new(current_cwd);
    let predicted_path = PathBuf::from(&predicted);
    if !current_path.is_absolute() || !predicted_path.is_absolute() {
        return (predicted, None);
    }

    let requested = canonicalize_or(&predicted_path);
    let (destination, warning) = enforce_caged_destination(world_root, current_path, requested);
    (destination.to_string_lossy().to_string(), warning)
}

fn apply_caged_predicted_cwd_from_config(
    current_cwd: &str,
    predicted: String,
) -> (String, Option<String>) {
    let current_path = Path::new(current_cwd);
    let Ok(world_root) = resolve_world_root(None, None, None, current_path) else {
        return (predicted, None);
    };
    apply_caged_predicted_cwd(&world_root, current_cwd, predicted)
}

fn suppress_redundant_caged_prediction_warning(
    prev_cwd: &str,
    predicted_cwd: &str,
    warning: Option<String>,
) -> Option<String> {
    let message = warning?;

    let prev_path = Path::new(prev_cwd);
    let predicted_path = Path::new(predicted_cwd);

    if prev_path.is_absolute()
        && predicted_path.is_absolute()
        && canonicalize_or(prev_path) == canonicalize_or(predicted_path)
    {
        // The in-world anchor guard already prints a caged warning when it blocks `cd`.
        // When the world session reports an unchanged cwd, our cd prediction layer can
        // redundantly re-emit the same warning. Suppress the duplicate when the net
        // cwd is unchanged.
        return None;
    }

    Some(message)
}

async fn exec_world_line(
    session: &mut WorldSession,
    program: &str,
    cmd_id: &str,
    io: &mut ReplIo<'_>,
) -> Result<i32> {
    let prev_cwd = session.world_cwd.clone();
    let (exit, cwd) = {
        let fut = session
            .client
            .exec(program, ReplStdinMode::Eof, cmd_id)
            .map(|res| res.map(|c| (c.exit, c.cwd)));
        pin_mut!(fut);

        loop {
            tokio::select! {
                res = &mut fut => break res?,
                maybe_event = io.agent_rx.recv() => {
                    if let Some(event) = maybe_event {
                        handle_agent_event(event, io.telemetry, io.agent_printer);
                    }
                }
                maybe_resize = io.resize_rx.recv() => {
                    if let Some((cols, rows)) = maybe_resize {
                        let _ = session.client.send_resize(cols, rows).await;
                    }
                }
                maybe_sigint = io.sigint_rx.recv() => {
                    if maybe_sigint.is_some() {
                        let _ = session.client.send_signal("INT").await;
                    }
                }
            }
        }
    };

    let mut next_cwd = cwd;
    if exit == 0 {
        if let Some(predicted) = predict_cd_next_cwd(&prev_cwd, program) {
            if next_cwd == prev_cwd {
                let (predicted, warning) =
                    apply_caged_predicted_cwd_from_config(&prev_cwd, predicted);
                let warning =
                    suppress_redundant_caged_prediction_warning(&prev_cwd, &predicted, warning);
                if let Some(message) = warning {
                    io.agent_printer.print(message);
                }
                next_cwd = predicted;
            }
        }
    }
    session.world_cwd = next_cwd;
    Ok(exit)
}

async fn exec_world_pty(
    session: &mut WorldSession,
    program: &str,
    cmd_id: &str,
    io: &mut ReplIo<'_>,
) -> Result<i32> {
    let _pty_active_guard = PtyActiveResetGuard::new();
    let _terminal_guard = MinimalTerminalGuard::new()?;

    let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let stdin_done = Arc::new(AtomicBool::new(false));
    let stdin_thread = spawn_passthrough_stdin_thread(stdin_tx, stdin_done.clone(), cmd_id);

    let max_pty_buffered_lines = io.max_pty_buffered_lines;
    let mut buffered_structured_lines = Vec::<String>::new();
    let mut dropped_structured_event_lines: u64 = 0;

    let prev_cwd = session.world_cwd.clone();
    let (exit, cwd) = {
        let fut = session
            .client
            .exec(program, ReplStdinMode::Passthrough, cmd_id)
            .map(|res| res.map(|c| (c.exit, c.cwd)));
        pin_mut!(fut);

        loop {
            tokio::select! {
                res = &mut fut => break res?,
            maybe_bytes = stdin_rx.recv() => {
                if let Some(bytes) = maybe_bytes {
                    let _ = session.client.send_stdin(&bytes).await;
                }
            }
            maybe_event = io.agent_rx.recv() => {
                if let Some(event) = maybe_event {
                    if is_shell_stream_event(&event) {
                        continue;
                    }

                    io.telemetry.persist_agent_event(&event);
                    io.telemetry.record_agent_event();
                    if buffered_structured_lines.len() < max_pty_buffered_lines {
                        buffered_structured_lines.push(format_event_line(&event));
                    } else {
                        dropped_structured_event_lines =
                            dropped_structured_event_lines.saturating_add(1);
                    }
                }
            }
            maybe_resize = io.resize_rx.recv() => {
                if let Some((cols, rows)) = maybe_resize {
                    let _ = session.client.send_resize(cols, rows).await;
                }
            }
            maybe_sigint = io.sigint_rx.recv() => {
                if maybe_sigint.is_some() {
                    let _ = session.client.send_signal("INT").await;
                }
            }
            }
        }
    };

    stdin_done.store(true, Ordering::Relaxed);
    let _ = stdin_thread.join();

    for line in buffered_structured_lines {
        io.agent_printer.print(line);
    }
    if dropped_structured_event_lines > 0 {
        io.telemetry.persist_warning_pty_structured_event_drops(
            dropped_structured_event_lines,
            max_pty_buffered_lines,
            Some(cmd_id),
        );
        io.agent_printer.print(format!(
            "substrate: warning: dropped {dropped_structured_event_lines} structured agent event line(s) during PTY passthrough (cap={max_pty_buffered_lines})"
        ));
    }

    let mut next_cwd = cwd;
    if exit == 0 {
        if let Some(predicted) = predict_cd_next_cwd(&prev_cwd, program) {
            if next_cwd == prev_cwd {
                let (predicted, warning) =
                    apply_caged_predicted_cwd_from_config(&prev_cwd, predicted);
                let warning =
                    suppress_redundant_caged_prediction_warning(&prev_cwd, &predicted, warning);
                if let Some(message) = warning {
                    io.agent_printer.print(message);
                }
                next_cwd = predicted;
            }
        }
    }
    session.world_cwd = next_cwd;
    Ok(exit)
}

#[cfg(test)]
mod caged_prediction_tests {
    use super::*;
    use substrate_common::WorldRootMode;
    use tempfile::tempdir;

    #[test]
    fn apply_caged_predicted_cwd_bounces_outside_anchor() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();

        let settings = WorldRootSettings {
            mode: WorldRootMode::Project,
            path: std::fs::canonicalize(&root).unwrap(),
            caged: true,
        };

        let current = std::fs::canonicalize(&root).unwrap();
        let predicted = std::fs::canonicalize(&outside).unwrap();
        let current_s = current.to_string_lossy().to_string();
        let predicted_s = predicted.to_string_lossy().to_string();
        let (out, warning) = apply_caged_predicted_cwd(&settings, &current_s, predicted_s);
        assert_eq!(out, settings.path.to_string_lossy().to_string());
        assert!(warning.is_some());
    }

    #[test]
    fn apply_caged_predicted_cwd_allows_inside_anchor() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let inside = root.join("inside");
        std::fs::create_dir_all(&inside).unwrap();

        let settings = WorldRootSettings {
            mode: WorldRootMode::Project,
            path: std::fs::canonicalize(&root).unwrap(),
            caged: true,
        };

        let current = std::fs::canonicalize(&root).unwrap();
        let predicted = std::fs::canonicalize(&inside).unwrap();
        let current_s = current.to_string_lossy().to_string();
        let predicted_s = predicted.to_string_lossy().to_string();
        let (out, warning) = apply_caged_predicted_cwd(&settings, &current_s, predicted_s.clone());
        assert_eq!(out, predicted_s);
        assert!(warning.is_none());
    }

    #[test]
    fn suppress_redundant_caged_prediction_warning_drops_when_cwd_unchanged() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        std::fs::create_dir_all(&root).unwrap();
        let prev = format!("{}/.", root.display());
        let predicted = root.to_string_lossy().to_string();
        let warning = Some("substrate: info: caged root guard: blocked cd to /tmp/outside (outside /tmp/root); returning to /tmp/root".to_string());

        let out = suppress_redundant_caged_prediction_warning(&prev, &predicted, warning);
        assert!(out.is_none());
    }
}

fn spawn_passthrough_stdin_thread(
    stdin_tx: UnboundedSender<Vec<u8>>,
    done: Arc<AtomicBool>,
    cmd_id: &str,
) -> thread::JoinHandle<()> {
    #[cfg(unix)]
    {
        use nix::sys::select::{select, FdSet};
        use nix::sys::time::TimeVal;
        use std::io::Read;
        use std::os::unix::io::{AsFd, AsRawFd};

        let cmd_id = cmd_id.to_string();
        thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buffer = vec![0u8; 4096];
            while !done.load(Ordering::Relaxed) {
                let stdin_fd = stdin.as_raw_fd();
                let stdin_borrowed = stdin.as_fd();
                let mut read_fds = FdSet::new();
                read_fds.insert(stdin_borrowed);
                let mut timeout = TimeVal::new(0, 100_000);
                let result = select(
                    stdin_fd + 1,
                    Some(&mut read_fds),
                    None,
                    None,
                    Some(&mut timeout),
                );
                match result {
                    Ok(0) => continue,
                    Ok(_) if read_fds.contains(stdin_borrowed) => match stdin.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let _ = stdin_tx.send(buffer[..n].to_vec());
                        }
                        Err(e) => {
                            log::warn!("[{cmd_id}] passthrough stdin read failed: {e}");
                            break;
                        }
                    },
                    Ok(_) => continue,
                    Err(e) => {
                        if e != nix::errno::Errno::EINTR {
                            log::warn!("[{cmd_id}] passthrough select() failed: {e}");
                            break;
                        }
                    }
                }
            }
        })
    }

    #[cfg(not(unix))]
    {
        let _ = (stdin_tx, done, cmd_id);
        thread::spawn(|| {})
    }
}

async fn exec_host_line(
    config: &ShellConfig,
    host_state: &mut HostState,
    line: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
    world_client: Option<&ReplPersistentSessionClient>,
    io: &mut ReplIo<'_>,
) -> Result<i32> {
    if let Some(code) = try_run_host_builtin(config, host_state, line, io.agent_printer)? {
        return Ok(code);
    }

    let shell_path = config.shell_path.clone();
    let cwd = host_state.cwd.clone();
    let env = host_state.env.clone();
    let line = line.to_string();
    let cmd_id = cmd_id.to_string();
    let command_fut = task::spawn_blocking(move || {
        spawn_host_shell_command(&shell_path, &cwd, &env, &line, &cmd_id, running_child_pid)
    })
    .map(
        |res: Result<Result<ExitStatus, anyhow::Error>, tokio::task::JoinError>| match res {
            Ok(inner) => inner,
            Err(err) => Err(anyhow!(err)),
        },
    );
    pin_mut!(command_fut);

    let status = loop {
        tokio::select! {
            res = &mut command_fut => break res?,
            maybe_event = io.agent_rx.recv() => {
                if let Some(event) = maybe_event {
                    handle_agent_event(event, io.telemetry, io.agent_printer);
                }
            }
            maybe_resize = io.resize_rx.recv() => {
                if let Some((cols, rows)) = maybe_resize {
                    if let Some(client) = world_client {
                        let _ = client.send_resize(cols, rows).await;
                    }
                }
            }
            _maybe_sigint = io.sigint_rx.recv() => {
                // Host signals are forwarded by global handlers; drain to avoid leaking to world.
            }
        }
    };

    Ok(exit_code_from_status(status))
}

fn try_run_host_builtin(
    config: &ShellConfig,
    host_state: &mut HostState,
    line: &str,
    agent_printer: &ReplPrinter,
) -> Result<Option<i32>> {
    let tokens = shell_words::split(line)
        .unwrap_or_else(|_| line.split_whitespace().map(|s| s.to_string()).collect());
    if tokens.is_empty() {
        return Ok(Some(0));
    }

    match tokens[0].as_str() {
        "pwd" => {
            agent_printer.print(format!("{}", host_state.cwd.display()));
            Ok(Some(0))
        }
        "cd" => {
            let target = tokens.get(1).map(String::as_str).unwrap_or("~");
            let expanded = shellexpand::tilde(target).to_string();
            let prev = host_state.cwd.clone();
            let candidate = PathBuf::from(&expanded);
            let absolute = if candidate.is_absolute() {
                candidate
            } else {
                prev.join(candidate)
            };
            let requested = match std::fs::canonicalize(&absolute) {
                Ok(path) => path,
                Err(_) => {
                    agent_printer.print(format!(
                        "substrate: error: :host cd: not a directory: {}",
                        absolute.display()
                    ));
                    return Ok(Some(1));
                }
            };

            let world_root = resolve_world_root(
                config.cli_anchor_mode,
                config.cli_anchor_path.clone(),
                config.cli_caged,
                &prev,
            )?;
            let (destination, warning) = enforce_caged_destination(&world_root, &prev, requested);
            if let Some(message) = warning {
                agent_printer.print(message);
            }

            host_state.cwd = destination;
            host_state
                .env
                .insert("PWD".to_string(), host_state.cwd.display().to_string());
            Ok(Some(0))
        }
        "export" => {
            for arg in tokens.iter().skip(1) {
                if let Some((k, v)) = arg.split_once('=') {
                    host_state.env.insert(k.to_string(), v.to_string());
                } else {
                    let val = host_state
                        .env
                        .get(arg)
                        .cloned()
                        .or_else(|| std::env::var(arg).ok())
                        .unwrap_or_default();
                    host_state.env.insert(arg.to_string(), val);
                }
            }
            Ok(Some(0))
        }
        "unset" => {
            for arg in tokens.iter().skip(1) {
                host_state.env.remove(arg);
            }
            Ok(Some(0))
        }
        _ => Ok(None),
    }
}

fn spawn_host_shell_command(
    shell_path: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    line: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
) -> Result<ExitStatus> {
    let mut cmd = Command::new(shell_path);
    cmd.arg("-c")
        .arg(line)
        .current_dir(cwd)
        .env_clear()
        .envs(env)
        .env("SHIM_PARENT_CMD_ID", cmd_id)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }
    }

    let mut child = cmd.spawn().context("spawn :host command")?;
    running_child_pid.store(child.id() as i32, Ordering::SeqCst);
    let status = child.wait().context("wait :host command")?;
    running_child_pid.store(0, Ordering::SeqCst);
    Ok(status)
}

fn exit_code_from_status(status: ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(1))
    }
    #[cfg(not(unix))]
    status.code().unwrap_or(1)
}

struct PtyActiveResetGuard;

impl PtyActiveResetGuard {
    fn new() -> Self {
        PTY_ACTIVE.store(true, Ordering::SeqCst);
        Self
    }
}

impl Drop for PtyActiveResetGuard {
    fn drop(&mut self) {
        PTY_ACTIVE.store(false, Ordering::SeqCst);
    }
}

fn exit_status_from_code(code: i32) -> ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        let clamped = code.clamp(0, 255);
        ExitStatus::from_raw(clamped << 8)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }
    #[cfg(not(any(unix, windows)))]
    {
        panic!("exit_status_from_code is unsupported on this platform (code={code})");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::agent_events::{
        acquire_event_test_guard, clear_agent_event_sender, init_event_channel,
    };
    use crate::execution::agent_runtime::orchestration_session::OrchestrationSessionPosture;
    use crate::execution::ShellMode;
    use crate::execution::WorldRootSettings;
    use std::cell::Cell;
    #[cfg(unix)]
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fs;
    use substrate_common::agent_events::AgentEventKind;
    use tempfile::TempDir;

    #[cfg(unix)]
    fn reedline_terminal_loss_error() -> anyhow::Error {
        anyhow!(io::Error::from_raw_os_error(libc::ENOTTY))
    }

    #[cfg(not(unix))]
    fn reedline_terminal_loss_error() -> anyhow::Error {
        anyhow!("terminal invalid")
    }

    #[cfg(unix)]
    fn common_terminal_loss_errors() -> Vec<anyhow::Error> {
        vec![
            anyhow!(io::Error::from_raw_os_error(libc::ENOTTY)),
            anyhow!(io::Error::from_raw_os_error(libc::EIO)),
            anyhow!(io::Error::from_raw_os_error(libc::EBADF)),
            anyhow!(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe")),
            anyhow!(io::Error::new(io::ErrorKind::NotConnected, "not connected")),
            anyhow!(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected eof"
            )),
        ]
    }

    #[cfg(not(unix))]
    fn common_terminal_loss_errors() -> Vec<anyhow::Error> {
        vec![
            anyhow!("terminal invalid"),
            anyhow!("input/output error"),
            anyhow!("bad file descriptor"),
            anyhow!(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe")),
            anyhow!(io::Error::new(io::ErrorKind::NotConnected, "not connected")),
            anyhow!(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected eof"
            )),
        ]
    }

    #[test]
    fn classify_prompt_worker_error_falls_back_on_cursor_timeout() {
        let err = anyhow!("cursor position could not be read within a normal duration");

        assert_eq!(
            classify_prompt_worker_error(true, &err),
            PromptWorkerErrorDisposition::FallbackToStdio
        );
    }

    #[test]
    fn classify_prompt_worker_error_treats_reedline_failures_as_abnormal_terminal_loss() {
        let err = reedline_terminal_loss_error();

        assert_eq!(
            classify_prompt_worker_error(true, &err),
            PromptWorkerErrorDisposition::AbnormalTerminalLoss
        );
    }

    #[test]
    fn classify_prompt_worker_error_treats_common_terminal_loss_errors_as_abnormal() {
        let cases = common_terminal_loss_errors();

        for err in &cases {
            assert_eq!(
                classify_prompt_worker_error(true, err),
                PromptWorkerErrorDisposition::AbnormalTerminalLoss
            );
        }
    }

    #[test]
    fn classify_prompt_worker_error_keeps_unrelated_reedline_errors_generic() {
        let err = anyhow!("completion menu rendering failed");

        assert_eq!(
            classify_prompt_worker_error(true, &err),
            PromptWorkerErrorDisposition::GenericError
        );
    }

    #[test]
    fn shutdown_disposition_tracks_termination_cause() {
        assert_eq!(
            shutdown_disposition_for_termination_cause(ReplTerminationCause::NormalExit),
            PromptWorkerShutdownDisposition::Graceful
        );
        assert_eq!(
            shutdown_disposition_for_termination_cause(ReplTerminationCause::AbnormalTerminalLoss),
            PromptWorkerShutdownDisposition::Abandon
        );
    }

    #[test]
    fn build_world_restart_required_alert_only_builds_with_orchestration_context() {
        let event = build_world_restart_required_alert(
            Some("orch-live"),
            "world-1",
            4,
            WorldRestartReason::PolicySnapshotChanged,
        )
        .expect("alert should be built");
        assert_eq!(event.kind, AgentEventKind::Alert);
        assert_eq!(event.orchestration_session_id, "orch-live");
        assert_eq!(event.world_id.as_deref(), Some("world-1"));
        assert_eq!(event.world_generation, Some(4));
        assert!(
            build_world_restart_required_alert(
                None,
                "world-1",
                4,
                WorldRestartReason::PolicySnapshotChanged,
            )
            .is_none(),
            "missing orchestration context must suppress the alert event"
        );
    }

    #[test]
    #[serial_test::serial]
    fn emit_world_restarted_alert_only_emits_with_orchestration_context() {
        let _guard = acquire_event_test_guard();
        let mut rx = init_event_channel();

        emit_world_restarted_alert(
            Some("orch-live"),
            "world-old",
            1,
            "world-new",
            2,
            WorldRestartReason::WorkspaceRootChanged,
        );

        let event = rx.try_recv().expect("world restarted event");
        assert_eq!(event.kind, AgentEventKind::Alert);
        assert_eq!(event.orchestration_session_id, "orch-live");
        assert_eq!(event.world_id.as_deref(), Some("world-new"));
        assert_eq!(event.world_generation, Some(2));

        emit_world_restarted_alert(
            None,
            "world-old",
            1,
            "world-new",
            2,
            WorldRestartReason::WorkspaceRootChanged,
        );
        assert!(
            rx.try_recv().is_err(),
            "missing orchestration context must suppress the restarted alert event"
        );
        clear_agent_event_sender();
    }

    #[test]
    fn resolve_reedline_ctrl_d_terminal_loss_returns_pre_recorded_message() {
        let detected = AtomicBool::new(true);
        let message = Mutex::new(Some("controlling terminal became invalid".to_string()));
        let detect_calls = Cell::new(0usize);

        let err = resolve_reedline_ctrl_d_terminal_loss_with(
            &detected,
            &message,
            || {
                detect_calls.set(detect_calls.get() + 1);
                None
            },
            || panic!("should not sleep when terminal loss is already recorded"),
        )
        .expect("expected recorded terminal loss");

        assert_eq!(detect_calls.get(), 0);
        assert!(err
            .to_string()
            .contains("controlling terminal became invalid"));
    }

    #[test]
    fn resolve_reedline_ctrl_d_terminal_loss_retries_detector_within_window() {
        let detected = AtomicBool::new(false);
        let message = Mutex::new(None::<String>);
        let detect_calls = Cell::new(0usize);
        let sleep_calls = Cell::new(0usize);

        let err = resolve_reedline_ctrl_d_terminal_loss_with(
            &detected,
            &message,
            || {
                let next = detect_calls.get() + 1;
                detect_calls.set(next);
                if next == 3 {
                    Some(anyhow!("delayed terminal loss"))
                } else {
                    None
                }
            },
            || sleep_calls.set(sleep_calls.get() + 1),
        )
        .expect("expected delayed terminal loss");

        assert_eq!(detect_calls.get(), 3);
        assert_eq!(sleep_calls.get(), 2);
        assert!(err.to_string().contains("delayed terminal loss"));
    }

    #[test]
    fn resolve_reedline_ctrl_d_terminal_loss_observes_monitor_update_on_retry() {
        let detected = AtomicBool::new(false);
        let message = Mutex::new(None::<String>);
        let sleep_calls = Cell::new(0usize);

        let err = resolve_reedline_ctrl_d_terminal_loss_with(
            &detected,
            &message,
            || None,
            || {
                let next = sleep_calls.get() + 1;
                sleep_calls.set(next);
                if next == 1 {
                    detected.store(true, Ordering::SeqCst);
                    *message.lock().expect("message mutex poisoned") =
                        Some("monitor reported terminal loss".to_string());
                }
            },
        )
        .expect("expected terminal loss from monitor update");

        assert_eq!(sleep_calls.get(), 1);
        assert!(err.to_string().contains("monitor reported terminal loss"));
    }

    #[test]
    fn resolve_reedline_ctrl_d_terminal_loss_returns_none_without_signal() {
        let detected = AtomicBool::new(false);
        let message = Mutex::new(None::<String>);
        let detect_calls = Cell::new(0usize);
        let sleep_calls = Cell::new(0usize);

        let err = resolve_reedline_ctrl_d_terminal_loss_with(
            &detected,
            &message,
            || {
                detect_calls.set(detect_calls.get() + 1);
                None
            },
            || sleep_calls.set(sleep_calls.get() + 1),
        );

        assert!(err.is_none());
        assert_eq!(detect_calls.get(), REEDLINE_CTRLD_TERMINAL_LOSS_RECHECKS);
        assert_eq!(sleep_calls.get(), REEDLINE_CTRLD_TERMINAL_LOSS_RECHECKS - 1);
    }

    struct CurrentDirGuard {
        original: PathBuf,
    }

    impl CurrentDirGuard {
        fn change_to(path: &Path) -> Self {
            let original = std::env::current_dir().expect("current dir should resolve");
            std::env::set_current_dir(path).expect("set current dir");
            Self { original }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    #[cfg(unix)]
    fn write_fake_codex_script(temp: &TempDir, keep_alive: bool) -> PathBuf {
        let path = temp.path().join("fake-codex.sh");
        let body = if keep_alive {
            "#!/bin/sh\ntrap 'exit 0' INT TERM\nprintf '{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}\\r\\n'\nprintf '{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}\\r\\n'\nwhile :; do sleep 1; done\n"
        } else {
            "#!/bin/sh\nprintf '{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}\\r\\n'\nprintf '{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}\\r\\n'\n"
        };
        fs::write(&path, body).expect("write fake codex script");
        let mut perms = fs::metadata(&path)
            .expect("fake codex metadata")
            .permissions();
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("set fake codex permissions");
        path
    }

    #[cfg(unix)]
    fn write_fake_codex_script_without_session_handle(temp: &TempDir) -> PathBuf {
        let path = temp.path().join("fake-codex-no-session.sh");
        let body = "#!/bin/sh\nprintf 'bootstrap-without-session-handle\\n'\n";
        fs::write(&path, body).expect("write fake codex script without session handle");
        let mut perms = fs::metadata(&path)
            .expect("fake codex metadata")
            .permissions();
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("set fake codex permissions");
        path
    }

    #[cfg(unix)]
    fn write_fake_codex_script_with_shutdown_delay(temp: &TempDir, delay_seconds: u64) -> PathBuf {
        let path = temp.path().join("fake-codex-shutdown-delay.sh");
        let body = format!(
            "#!/bin/sh\ntrap 'sleep {}; exit 0' INT TERM\nprintf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\nwhile :; do sleep 1; done\n",
            delay_seconds
        );
        fs::write(&path, body).expect("write fake codex shutdown delay script");
        let mut perms = fs::metadata(&path)
            .expect("fake codex metadata")
            .permissions();
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("set fake codex permissions");
        path
    }

    #[cfg(unix)]
    fn write_fake_codex_script_with_pid_file(temp: &TempDir, pid_file: &Path) -> PathBuf {
        let path = temp.path().join("fake-codex-with-pid.sh");
        let body = format!(
            "#!/bin/sh\ntrap 'exit 0' INT TERM\nprintf '%s\\n' $$ > {}\nprintf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\nwhile :; do sleep 1; done\n",
            pid_file.display()
        );
        fs::write(&path, body).expect("write fake codex pid script");
        let mut perms = fs::metadata(&path)
            .expect("fake codex metadata")
            .permissions();
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("set fake codex permissions");
        path
    }

    #[cfg(unix)]
    fn write_fake_codex_script_with_running_and_shutdown_delay(
        temp: &TempDir,
        running_delay_seconds: u64,
        shutdown_delay_seconds: u64,
    ) -> PathBuf {
        let path = temp.path().join("fake-codex-running-and-shutdown-delay.sh");
        let body = format!(
            "#!/bin/sh\ntrap 'sleep {}; exit 0' INT TERM\nprintf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\nsleep {}\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\nwhile :; do sleep 1; done\n",
            shutdown_delay_seconds, running_delay_seconds
        );
        fs::write(&path, body).expect("write fake codex delayed script");
        let mut perms = fs::metadata(&path)
            .expect("fake codex metadata")
            .permissions();
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("set fake codex permissions");
        path
    }

    #[cfg(unix)]
    fn participant_snapshot_path(store: &AgentRuntimeStateStore, participant_id: &str) -> PathBuf {
        store
            .participants_dir()
            .join(format!("{participant_id}.json"))
    }

    #[cfg(unix)]
    fn legacy_handle_snapshot_path(
        store: &AgentRuntimeStateStore,
        participant_id: &str,
    ) -> PathBuf {
        store.handles_dir().join(format!("{participant_id}.json"))
    }

    #[cfg(unix)]
    fn session_state_name(state: &AgentRuntimeSessionState) -> &'static str {
        match state {
            AgentRuntimeSessionState::Allocating => "allocating",
            AgentRuntimeSessionState::Ready => "ready",
            AgentRuntimeSessionState::Running => "running",
            AgentRuntimeSessionState::Restarting => "restarting",
            AgentRuntimeSessionState::Stopping => "stopping",
            AgentRuntimeSessionState::Stopped => "stopped",
            AgentRuntimeSessionState::Failed => "failed",
            AgentRuntimeSessionState::Invalidated => "invalidated",
        }
    }

    #[cfg(unix)]
    fn read_persisted_participant_snapshot(
        store: &AgentRuntimeStateStore,
        participant_id: &str,
    ) -> serde_json::Value {
        let participant_path = participant_snapshot_path(store, participant_id);
        assert!(
            participant_path.exists(),
            "participant snapshot should exist at {}",
            participant_path.display()
        );

        let legacy_handle_path = legacy_handle_snapshot_path(store, participant_id);
        assert!(
            !legacy_handle_path.exists(),
            "writer cutover must not create legacy handle snapshots at {}",
            legacy_handle_path.display()
        );

        let payload: serde_json::Value = serde_json::from_slice(
            &fs::read(&participant_path).expect("read persisted participant snapshot"),
        )
        .expect("parse persisted participant snapshot");
        assert_eq!(
            payload
                .get("participant_id")
                .and_then(serde_json::Value::as_str),
            Some(participant_id)
        );
        assert!(
            payload.get("session_handle_id").is_none(),
            "participant snapshot should not serialize legacy session_handle_id"
        );
        payload
    }

    #[cfg(unix)]
    fn assert_persisted_participant_snapshot(
        store: &AgentRuntimeStateStore,
        participant_id: &str,
        expected_state: &AgentRuntimeSessionState,
    ) -> serde_json::Value {
        let payload = read_persisted_participant_snapshot(store, participant_id);
        assert_eq!(
            payload.get("state").and_then(serde_json::Value::as_str),
            Some(session_state_name(expected_state))
        );
        payload
    }

    #[cfg(unix)]
    async fn wait_for_persisted_participant_snapshot(
        store: &AgentRuntimeStateStore,
        participant_id: &str,
        expected_state: AgentRuntimeSessionState,
    ) -> serde_json::Value {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let payload = read_persisted_participant_snapshot(store, participant_id);
                if payload.get("state").and_then(serde_json::Value::as_str)
                    == Some(session_state_name(&expected_state))
                {
                    break payload;
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        })
        .await
        .expect("timed out waiting for participant snapshot state")
    }

    #[cfg(unix)]
    fn test_shell_config(workspace_root: &Path, substrate_home: &Path) -> ShellConfig {
        ShellConfig {
            mode: ShellMode::Interactive { use_pty: false },
            session_id: Uuid::now_v7().to_string(),
            trace_log_file: substrate_home.join("trace.jsonl"),
            original_path: std::env::var("PATH").unwrap_or_default(),
            shim_dir: substrate_home.join("shims"),
            shell_path: "/bin/sh".to_string(),
            ci_mode: false,
            no_exit_on_error: false,
            skip_shims: true,
            no_world: true,
            cli_world: false,
            cli_no_world: true,
            cli_anchor_mode: None,
            cli_anchor_path: None,
            cli_caged: None,
            world_root: WorldRootSettings {
                mode: substrate_common::WorldRootMode::Project,
                path: workspace_root.to_path_buf(),
                caged: true,
            },
            async_repl: true,
            repl_host_escape: false,
            env_vars: HashMap::new(),
            manager_init_path: substrate_home.join("manager-init.sh"),
            manager_env_path: substrate_home.join("manager-env.sh"),
            shimmed_path: None,
            host_bash_env: None,
            bash_preexec_path: substrate_home.join("bash-preexec.sh"),
            preexec_available: false,
        }
    }

    #[cfg(unix)]
    fn write_runtime_inventory_with_world_member(
        substrate_home: &Path,
        orchestrator_binary: &Path,
        member_binary: &Path,
    ) {
        fs::write(
            substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: claude_code\n",
        )
        .expect("write config");
        fs::write(
            substrate_home.join("policy.yaml"),
            "agents:\n  allowed_backends:\n    - cli:claude_code\n    - cli:codex\n",
        )
        .expect("write policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("agents dir");
        fs::write(
            agents_dir.join("claude_code.yaml"),
            format!(
                "version: 1\nid: claude_code\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                orchestrator_binary.display()
            ),
        )
        .expect("write claude_code agent file");
        fs::write(
            agents_dir.join("codex.yaml"),
            format!(
                "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: world\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                member_binary.display()
            ),
        )
        .expect("write codex agent file");
    }

    #[cfg(unix)]
    fn seed_live_orchestrator_parent(
        startup_context: &RuntimeOrchestrationContext,
        manifest: &Arc<Mutex<AgentRuntimeSessionManifest>>,
        world_binding: &PersistedWorldBinding,
    ) {
        persist_world_binding_authority(
            &startup_context.store,
            &startup_context.orchestration_session,
            Some(world_binding),
        )
        .expect("persist parent world binding");
        let (orchestration_snapshot, manifest_snapshot) = {
            let mut orchestration_guard = startup_context
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            let mut manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
            manifest_guard.set_uaa_session_id("thread-parent".to_string());
            manifest_guard.mark_runtime_ownership_retained();
            manifest_guard.transition_state(AgentRuntimeSessionState::Ready);
            manifest_guard.touch_heartbeat();
            orchestration_guard
                .bind_active_session_handle(manifest_guard.handle.participant_id.clone());
            orchestration_guard.transition_state(OrchestrationSessionState::Active);
            (orchestration_guard.clone(), manifest_guard.clone())
        };
        persist_runtime_snapshots(
            &startup_context.store,
            &orchestration_snapshot,
            &manifest_snapshot,
        )
        .expect("persist live orchestrator parent");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn start_host_orchestrator_runtime_persists_participant_snapshots_across_lifecycle_states() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_codex = write_fake_codex_script_with_running_and_shutdown_delay(&temp, 1, 1);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        fs::write(
            substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n",
        )
        .expect("write config");
        fs::write(
            substrate_home.join("policy.yaml"),
            "agents:\n  allowed_backends:\n    - cli:codex\n",
        )
        .expect("write policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("agents dir");
        fs::write(
            agents_dir.join("codex.yaml"),
            format!(
                "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                fake_codex.display()
            ),
        )
        .expect("write codex agent file");

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            let runtime =
                start_host_orchestrator_runtime(&config, &ReplPrinter::Stdout, &mut telemetry)
                    .await
                    .expect("bootstrap runtime should succeed")
                    .expect("agents enabled should create a runtime");
            let store = runtime.store.clone();

            let (parent, live) = runtime
                .store
                .resolve_live_orchestrator_session("codex")
                .expect("resolve live orchestrator session")
                .expect("live orchestration session should exist");
            let participant_id = live.handle.participant_id.clone();
            assert_eq!(parent.state, OrchestrationSessionState::Active);
            assert_eq!(
                parent.active_session_handle_id.as_deref(),
                Some(participant_id.as_str())
            );
            assert_eq!(parent.shell_trace_session_id, config.session_id);
            assert_eq!(
                fs::canonicalize(&parent.workspace_root).expect("canonicalize parent workspace"),
                fs::canonicalize(&workspace_root).expect("canonicalize workspace root")
            );
            assert_eq!(
                runtime
                    .store
                    .find_active_orchestration_session_for_pid(std::process::id())
                    .expect("find active orchestration session for pid")
                    .expect("active orchestration session should exist")
                    .orchestration_session_id,
                parent.orchestration_session_id
            );
            assert_eq!(live.handle.state, AgentRuntimeSessionState::Ready);
            assert_eq!(live.internal.uaa_session_id.as_deref(), Some("thread-test"));
            assert!(live.internal.ownership_valid);
            assert!(live.internal.control_owner_retained);
            assert!(live.internal.event_stream_active);
            assert!(live.internal.completion_observer_retained);
            assert_eq!(live.handle.protocol, PURE_AGENT_PROTOCOL);
            assert_persisted_participant_snapshot(
                &store,
                &participant_id,
                &AgentRuntimeSessionState::Ready,
            );

            wait_for_persisted_participant_snapshot(
                &store,
                &participant_id,
                AgentRuntimeSessionState::Running,
            )
            .await;

            let shutdown_config = config.clone();
            let shutdown_task = tokio::spawn(async move {
                let mut shutdown_telemetry =
                    ReplSessionTelemetry::new(shutdown_config, "async-test-shutdown");
                shutdown_host_orchestrator_runtime(
                    runtime,
                    &ReplPrinter::Stdout,
                    &mut shutdown_telemetry,
                )
                .await;
            });

            wait_for_persisted_participant_snapshot(
                &store,
                &participant_id,
                AgentRuntimeSessionState::Stopping,
            )
            .await;
            let stopping_parent = store
                .load_orchestration_session(&parent.orchestration_session_id)
                .expect("load stopping orchestration session")
                .expect("stopping orchestration session should exist");
            assert_eq!(stopping_parent.state, OrchestrationSessionState::Stopping);
            assert_eq!(
                stopping_parent.active_session_handle_id.as_deref(),
                Some(participant_id.as_str())
            );

            shutdown_task
                .await
                .expect("shutdown task should complete cleanly");

            let manifests = store.list_manifests().expect("list manifests");
            let stopped = manifests
                .into_iter()
                .find(|manifest| manifest.handle.agent_id == "codex")
                .expect("stopped manifest should exist");
            assert_eq!(stopped.handle.state, AgentRuntimeSessionState::Stopped);
            assert!(!stopped.internal.ownership_valid);
            assert!(!stopped.internal.control_owner_retained);
            assert!(!stopped.internal.event_stream_active);
            assert!(!stopped.internal.completion_observer_retained);
            assert_persisted_participant_snapshot(
                &store,
                &participant_id,
                &AgentRuntimeSessionState::Stopped,
            );
            let stopped_parent = store
                .load_orchestration_session(&stopped.handle.orchestration_session_id)
                .expect("load stopped orchestration session")
                .expect("stopped orchestration session should exist");
            assert_eq!(stopped_parent.state, OrchestrationSessionState::Stopped);
            assert_eq!(
                stopped_parent.active_session_handle_id.as_deref(),
                Some(participant_id.as_str())
            );
        });
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn start_host_orchestrator_runtime_parks_when_attached_control_exits() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_codex = write_fake_codex_script(&temp, false);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        fs::write(
            substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n",
        )
        .expect("write config");
        fs::write(
            substrate_home.join("policy.yaml"),
            "agents:\n  allowed_backends:\n    - cli:codex\n",
        )
        .expect("write policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("agents dir");
        fs::write(
            agents_dir.join("codex.yaml"),
            format!(
                "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                fake_codex.display()
            ),
        )
        .expect("write codex agent file");

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            let runtime =
                start_host_orchestrator_runtime(&config, &ReplPrinter::Stdout, &mut telemetry)
                    .await
                    .expect("runtime start should still observe attached ownership briefly")
                    .expect("agents enabled should create a runtime");

            tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    let store = AgentRuntimeStateStore::new().expect("state store");
                    let manifests = store.list_manifests().expect("list manifests");
                    let manifest = manifests
                        .into_iter()
                        .find(|manifest| manifest.handle.agent_id == "codex")
                        .expect("runtime manifest should exist");
                    let parent = store
                        .load_orchestration_session(&manifest.handle.orchestration_session_id)
                        .expect("load orchestration session")
                        .expect("runtime orchestration session should exist");
                    if parent.posture == OrchestrationSessionPosture::ParkedResumable {
                        assert_eq!(parent.state, OrchestrationSessionState::Active);
                        assert_eq!(parent.attached_participant_id.as_deref(), None);
                        assert_eq!(
                            parent.active_session_handle_id.as_deref(),
                            Some(manifest.handle.participant_id.as_str())
                        );
                        assert_eq!(parent.shell_owner_pid, 0);
                        assert!(parent.closed_at.is_none());
                        assert!(!manifest.internal.ownership_valid);
                        assert!(!manifest.internal.control_owner_retained);
                        assert!(!manifest.internal.completion_observer_retained);
                        assert!(!manifest.attached_client_present());
                        assert!(manifest.is_resume_eligible());
                        assert_eq!(manifest.internal.shell_owner_pid, 0);
                        assert_eq!(
                            manifest.internal.uaa_session_id.as_deref(),
                            Some("thread-test")
                        );
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
            })
            .await
            .expect("runtime should park promptly after attached control exits");

            let live_orchestrator = AgentRuntimeStateStore::new()
                .expect("state store")
                .find_live_orchestrator("codex");
            assert!(
                match &live_orchestrator {
                    Ok(None) => true,
                    Err(err) => err.to_string().contains("references inactive participant"),
                    Ok(Some(_)) => false,
                },
                "parked detached control must disappear from authoritative live lookups: {live_orchestrator:?}"
            );
            let live_session = AgentRuntimeStateStore::new()
                .expect("state store")
                .resolve_live_orchestrator_session("codex");
            assert!(
                match &live_session {
                    Ok(None) => true,
                    Err(err) => err.to_string().contains("references inactive participant"),
                    Ok(Some(_)) => false,
                },
                "parked detached control must disappear from parent-gated live resolution: {live_session:?}"
            );
            shutdown_host_orchestrator_runtime(runtime, &ReplPrinter::Stdout, &mut telemetry).await;
        });
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn start_host_orchestrator_runtime_does_not_persist_live_manifest_without_session_handle() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_codex = write_fake_codex_script_without_session_handle(&temp);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        fs::write(
            substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n",
        )
        .expect("write config");
        fs::write(
            substrate_home.join("policy.yaml"),
            "agents:\n  allowed_backends:\n    - cli:codex\n",
        )
        .expect("write policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("agents dir");
        fs::write(
            agents_dir.join("codex.yaml"),
            format!(
                "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                fake_codex.display()
            ),
        )
        .expect("write codex agent file");

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            let failure = match start_host_orchestrator_runtime(
                &config,
                &ReplPrinter::Stdout,
                &mut telemetry,
            )
            .await
            {
                Ok(_) => panic!("bootstrap without a surfaced session handle must fail"),
                Err(failure) => failure,
            };
            assert!(
                failure
                    .message
                    .contains("failed to establish attached control ownership")
                    || failure
                        .message
                        .contains("ended before ownership could be established"),
                "bootstrap failure should explain the missing durable ownership boundary: {failure:?}"
            );

            assert!(
                AgentRuntimeStateStore::new()
                    .expect("state store")
                    .find_live_orchestrator("codex")
                    .expect("load live orchestrator")
                    .is_none(),
                "bootstrap failure before session handle ownership must not leave a live manifest"
            );
            assert!(
                AgentRuntimeStateStore::new()
                    .expect("state store")
                    .resolve_live_orchestrator_session("codex")
                    .expect("resolve live orchestrator session")
                    .is_none(),
                "bootstrap failure before session handle ownership must not resolve a live parent session"
            );

            let manifest = AgentRuntimeStateStore::new()
                .expect("state store")
                .list_manifests()
                .expect("list manifests")
                .into_iter()
                .find(|manifest| manifest.handle.agent_id == "codex")
                .expect("failed manifest should exist");
            assert_eq!(manifest.handle.state, AgentRuntimeSessionState::Failed);
            assert!(!manifest.internal.ownership_valid);
            assert_persisted_participant_snapshot(
                &AgentRuntimeStateStore::new().expect("state store"),
                &manifest.handle.participant_id,
                &AgentRuntimeSessionState::Failed,
            );
            let parent = AgentRuntimeStateStore::new()
                .expect("state store")
                .load_orchestration_session(&manifest.handle.orchestration_session_id)
                .expect("load failed orchestration session")
                .expect("failed orchestration session should exist");
            assert_eq!(parent.state, OrchestrationSessionState::Failed);
            assert!(parent.active_session_handle_id.is_none());
        });
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn shutdown_host_orchestrator_runtime_waits_for_cancel_completion_before_stopping() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_codex = write_fake_codex_script_with_shutdown_delay(&temp, 1);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        fs::write(
            substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n",
        )
        .expect("write config");
        fs::write(
            substrate_home.join("policy.yaml"),
            "agents:\n  allowed_backends:\n    - cli:codex\n",
        )
        .expect("write policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("agents dir");
        fs::write(
            agents_dir.join("codex.yaml"),
            format!(
                "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                fake_codex.display()
            ),
        )
        .expect("write codex agent file");

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            let runtime =
                start_host_orchestrator_runtime(&config, &ReplPrinter::Stdout, &mut telemetry)
                    .await
                    .expect("bootstrap runtime should succeed")
                    .expect("agents enabled should create a runtime");
            let store = runtime.store.clone();
            let shutdown_config = config.clone();
            let shutdown_task = tokio::spawn(async move {
                let mut shutdown_telemetry =
                    ReplSessionTelemetry::new(shutdown_config, "async-test-shutdown");
                shutdown_host_orchestrator_runtime(
                    runtime,
                    &ReplPrinter::Stdout,
                    &mut shutdown_telemetry,
                )
                .await;
            });

            tokio::time::sleep(Duration::from_millis(200)).await;
            assert!(
                !shutdown_task.is_finished(),
                "shutdown must stay blocked until the retained completion path resolves"
            );
            shutdown_task
                .await
                .expect("shutdown task should complete cleanly");

            let manifest = store
                .list_manifests()
                .expect("list manifests")
                .into_iter()
                .find(|manifest| manifest.handle.agent_id == "codex")
                .expect("stopped manifest should exist");
            assert_eq!(manifest.handle.state, AgentRuntimeSessionState::Stopped);
            assert_eq!(
                manifest.internal.termination_reason.as_deref(),
                Some("stopped")
            );
        });
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn hidden_owner_private_stop_fails_closed_when_completion_never_resolves() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_codex = write_fake_codex_script(&temp, true);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        fs::write(
            substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n",
        )
        .expect("write config");
        fs::write(
            substrate_home.join("policy.yaml"),
            "agents:\n  allowed_backends:\n    - cli:codex\n",
        )
        .expect("write policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("agents dir");
        fs::write(
            agents_dir.join("codex.yaml"),
            format!(
                "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                fake_codex.display()
            ),
        )
        .expect("write codex agent file");

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let prepared = prepare_host_orchestrator_runtime_startup(&config)
                .expect("prepare host runtime should succeed")
                .expect("host runtime should be configured");
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            let mut runtime = start_host_orchestrator_runtime_with_prepared_prompt(
                Some(prepared),
                None,
                None,
                true,
                &ReplPrinter::Stdout,
                &mut telemetry,
            )
            .await
            .expect("hidden helper runtime should start")
            .expect("runtime");
            let store = runtime.store.clone();
            let orchestration_session_id = runtime
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned")
                .orchestration_session_id
                .clone();
            let participant_id = runtime
                .manifest
                .lock()
                .expect("runtime manifest mutex poisoned")
                .handle
                .participant_id
                .clone();

            match &mut runtime.retained_control {
                RetainedRunControl::Local(retained_control) => {
                    if let Some(task) = retained_control.completion_task.take() {
                        task.abort();
                        let _ = task.await;
                    }
                    retained_control.completion_task =
                        Some(tokio::spawn(std::future::pending::<()>()));
                }
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                RetainedRunControl::Remote(_) => {
                    panic!("host helper runtime should stay on the local retained-control path")
                }
            }

            let helper_task = tokio::spawn(wait_for_hidden_owner_helper_completion(runtime));
            let stop_transport_path =
                crate::execution::agent_runtime::control::private_stop_transport_path(
                    &store,
                    &orchestration_session_id,
                    &participant_id,
                );
            tokio::time::timeout(Duration::from_secs(3), async {
                while !stop_transport_path.exists() {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
            })
            .await
            .expect("hidden helper stop transport should be published");

            let outcome = crate::execution::agent_runtime::control::request_private_stop(
                &stop_transport_path,
            )
            .await
            .expect("private stop request should connect");
            assert_eq!(outcome, PrivateStopOutcome::Accepted);

            let exit_code = helper_task
                .await
                .expect("hidden helper wait task should join")
                .expect("hidden helper wait should return an exit code");
            assert_eq!(exit_code, 1);

            let parent = store
                .load_orchestration_session(&orchestration_session_id)
                .expect("load orchestration session")
                .expect("terminal orchestration session should exist");
            assert_eq!(parent.state, OrchestrationSessionState::Failed);
            assert_eq!(parent.posture, OrchestrationSessionPosture::Terminal);
            assert!(parent.attached_participant_id.is_none());
            assert!(parent.closed_at.is_some());

            let manifest = store
                .list_manifests()
                .expect("list manifests")
                .into_iter()
                .find(|manifest| manifest.handle.participant_id == participant_id)
                .expect("failed manifest should exist");
            assert_eq!(manifest.handle.state, AgentRuntimeSessionState::Failed);
            assert_eq!(
                manifest.internal.last_error_bucket.as_deref(),
                Some("runtime_shutdown")
            );
        });
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn shutdown_host_orchestrator_runtime_parks_resumable_host_session_on_detach() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        let pid_file = temp.path().join("fake-codex.pid");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_codex = write_fake_codex_script_with_pid_file(&temp, &pid_file);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        fs::write(
            substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n",
        )
        .expect("write config");
        fs::write(
            substrate_home.join("policy.yaml"),
            "agents:\n  allowed_backends:\n    - cli:codex\n",
        )
        .expect("write policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("agents dir");
        fs::write(
            agents_dir.join("codex.yaml"),
            format!(
                "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                fake_codex.display()
            ),
        )
        .expect("write codex agent file");

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            let runtime =
                start_host_orchestrator_runtime(&config, &ReplPrinter::Stdout, &mut telemetry)
                    .await
                    .expect("bootstrap runtime should succeed")
                    .expect("agents enabled should create a runtime");
            let store = runtime.store.clone();
            let session_id = runtime
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned")
                .orchestration_session_id
                .clone();

            shutdown_host_orchestrator_runtime_with_mode(
                runtime,
                HostRuntimeShutdownMode::ParkIfResumable,
                &ReplPrinter::Stdout,
                &mut telemetry,
            )
            .await;

            let target = store
                .resolve_public_control_target(
                    &session_id,
                    crate::execution::agent_runtime::PublicControlAction::Resume,
                )
                .expect("parked session should stay resume-eligible");
            assert_eq!(target.session.state, OrchestrationSessionState::Active);
            assert_eq!(
                target.session.posture,
                OrchestrationSessionPosture::ParkedResumable
            );
            assert!(target.session.attached_participant_id.is_none());
            assert_eq!(
                target.session.active_session_handle_id.as_deref(),
                Some(target.active_participant.handle.participant_id.as_str())
            );
            assert!(target.active_participant.handle.state.is_live());
            assert!(!target.active_participant.attached_client_present());
            assert!(target.active_participant.is_resume_eligible());
            assert!(!target.active_participant.internal.control_owner_retained);
            assert!(
                !target
                    .active_participant
                    .internal
                    .completion_observer_retained
            );
            assert!(!target.active_participant.internal.event_stream_active);
            assert_eq!(
                target.active_participant.internal.detach_reason.as_deref(),
                Some("owner detached cleanly")
            );
        });

        let pid = fs::read_to_string(&pid_file)
            .expect("pid file")
            .trim()
            .parse::<i32>()
            .expect("pid should parse");
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn shutdown_host_orchestrator_runtime_fails_closed_when_detached_continuity_breaks() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        let pid_file = temp.path().join("fake-codex.pid");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_codex = write_fake_codex_script_with_pid_file(&temp, &pid_file);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        fs::write(
            substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n",
        )
        .expect("write config");
        fs::write(
            substrate_home.join("policy.yaml"),
            "agents:\n  allowed_backends:\n    - cli:codex\n",
        )
        .expect("write policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("agents dir");
        fs::write(
            agents_dir.join("codex.yaml"),
            format!(
                "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
                fake_codex.display()
            ),
        )
        .expect("write codex agent file");

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            let runtime =
                start_host_orchestrator_runtime(&config, &ReplPrinter::Stdout, &mut telemetry)
                    .await
                    .expect("bootstrap runtime should succeed")
                    .expect("agents enabled should create a runtime");
            let store = runtime.store.clone();
            let session_id = runtime
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned")
                .orchestration_session_id
                .clone();

            runtime
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned")
                .active_session_handle_id = Some("ash_wrong".to_string());

            shutdown_host_orchestrator_runtime_with_mode(
                runtime,
                HostRuntimeShutdownMode::ParkIfResumable,
                &ReplPrinter::Stdout,
                &mut telemetry,
            )
            .await;

            let session = store
                .load_orchestration_session(&session_id)
                .expect("load orchestration session")
                .expect("terminal orchestration session should exist");
            assert!(session.state.is_terminal());
            assert_eq!(session.posture, OrchestrationSessionPosture::Terminal);
            assert!(session.closed_at.is_some());
            assert!(
                store
                    .resolve_public_control_target(
                        &session_id,
                        crate::execution::agent_runtime::PublicControlAction::Resume,
                    )
                    .is_err(),
                "invalid detached continuity must fail closed instead of parking"
            );
        });

        let pid = fs::read_to_string(&pid_file)
            .expect("pid file")
            .trim()
            .parse::<i32>()
            .expect("pid should parse");
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn start_member_runtime_reuses_parent_session_and_persists_world_binding() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_orchestrator =
            write_fake_codex_script_with_running_and_shutdown_delay(&temp, 1, 1);
        let fake_member = write_fake_codex_script_with_running_and_shutdown_delay(&temp, 1, 1);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        write_runtime_inventory_with_world_member(
            &substrate_home,
            &fake_orchestrator,
            &fake_member,
        );

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let prepared = prepare_host_orchestrator_runtime_startup(&config)
                .expect("prepare host runtime should succeed")
                .expect("host runtime should be configured");
            let startup_context = prepared.startup_context.clone();
            let host_manifest = prepared.manifest.clone();
            let initial_world_binding = PersistedWorldBinding {
                world_id: "wld_member_test".to_string(),
                world_generation: 7,
            };
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            seed_live_orchestrator_parent(&startup_context, &host_manifest, &initial_world_binding);

            let selected_descriptor = select_member_runtime_descriptor(&startup_context)
                .expect("member selection should succeed")
                .expect("one world-scoped member should be selected");
            let member_prepared = prepare_member_runtime_startup_for_descriptor(
                &startup_context,
                selected_descriptor,
                &initial_world_binding,
                None,
            )
            .expect("member runtime prepare should succeed");
            let member_runtime = start_member_runtime_with_prepared(
                Some(member_prepared),
                &ReplPrinter::Stdout,
                &mut telemetry,
            )
            .await
            .expect("member runtime start should succeed")
            .expect("member runtime");

            let parent = startup_context.snapshot();
            let live_members = startup_context
                .store
                .list_live_participants_for_session(&parent.orchestration_session_id)
                .expect("list live participants")
                .into_iter()
                .filter(|participant| participant.handle.role == MEMBER_ROLE)
                .collect::<Vec<_>>();
            assert_eq!(
                live_members.len(),
                1,
                "expected exactly one live member runtime"
            );
            let member = &live_members[0];
            assert_eq!(member.handle.agent_id, "codex");
            assert_eq!(member.handle.world_id.as_deref(), Some("wld_member_test"));
            assert_eq!(member.handle.world_generation, Some(7));
            assert_eq!(
                parent.active_session_handle_id.as_deref(),
                Some(
                    host_manifest
                        .lock()
                        .expect("host runtime manifest")
                        .handle
                        .participant_id
                        .as_str()
                ),
                "member launch must not replace the authoritative orchestrator participant"
            );

            let member_snapshot = runtime_manifest_snapshot(&member_runtime);
            assert_eq!(member_snapshot.handle.role, MEMBER_ROLE);
            assert_eq!(member_snapshot.handle.world_generation, Some(7));
            assert!(member_snapshot.is_authoritative_live());

            shutdown_host_orchestrator_runtime(
                member_runtime,
                &ReplPrinter::Stdout,
                &mut telemetry,
            )
            .await;
        });
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn prepare_member_replacement_runtime_preserves_resumed_from_lineage() {
        let _world_env_guard = crate::execution::world_env_guard();
        let temp = TempDir::new().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&workspace_root).expect("workspace root");
        fs::create_dir_all(&substrate_home).expect("substrate home");
        let _cwd_guard = CurrentDirGuard::change_to(&workspace_root);
        let fake_orchestrator =
            write_fake_codex_script_with_running_and_shutdown_delay(&temp, 1, 1);
        let fake_member = write_fake_codex_script_with_running_and_shutdown_delay(&temp, 1, 1);

        std::env::set_var("SUBSTRATE_HOME", &substrate_home);
        write_runtime_inventory_with_world_member(
            &substrate_home,
            &fake_orchestrator,
            &fake_member,
        );

        let config = Arc::new(test_shell_config(&workspace_root, &substrate_home));
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let prepared = prepare_host_orchestrator_runtime_startup(&config)
                .expect("prepare host runtime should succeed")
                .expect("host runtime should be configured");
            let startup_context = prepared.startup_context.clone();
            let host_manifest = prepared.manifest.clone();
            let initial_world_binding = PersistedWorldBinding {
                world_id: "wld_member_old".to_string(),
                world_generation: 2,
            };
            let mut telemetry = ReplSessionTelemetry::new(config.clone(), "async-test");
            seed_live_orchestrator_parent(&startup_context, &host_manifest, &initial_world_binding);

            let selected_descriptor = select_member_runtime_descriptor(&startup_context)
                .expect("member selection should succeed")
                .expect("one world-scoped member should be selected");
            let member_prepared = prepare_member_runtime_startup_for_descriptor(
                &startup_context,
                selected_descriptor.clone(),
                &initial_world_binding,
                None,
            )
            .expect("member runtime prepare should succeed");
            let member_runtime = start_member_runtime_with_prepared(
                Some(member_prepared),
                &ReplPrinter::Stdout,
                &mut telemetry,
            )
            .await
            .expect("member runtime start should succeed")
            .expect("member runtime");

            let previous_member = runtime_manifest_snapshot(&member_runtime);
            let replacement_binding = PersistedWorldBinding {
                world_id: "wld_member_new".to_string(),
                world_generation: 3,
            };
            persist_world_binding_authority(
                &startup_context.store,
                &startup_context.orchestration_session,
                Some(&replacement_binding),
            )
            .expect("persist replacement binding");
            startup_context
                .store
                .invalidate_stale_world_members_for_session(
                    &startup_context.orchestration_session_id(),
                    replacement_binding.world_generation,
                )
                .expect("invalidate stale world members");

            let replacement_prepared = prepare_member_runtime_startup_for_descriptor(
                &startup_context,
                selected_descriptor,
                &replacement_binding,
                Some(&previous_member),
            )
            .expect("replacement prepare should succeed");
            let replacement_manifest = replacement_prepared
                .manifest
                .lock()
                .expect("replacement manifest")
                .clone();
            assert_eq!(replacement_manifest.handle.role, MEMBER_ROLE);
            assert_eq!(
                replacement_manifest
                    .handle
                    .resumed_from_participant_id
                    .as_deref(),
                Some(previous_member.handle.participant_id.as_str())
            );
            assert_ne!(
                replacement_manifest.handle.participant_id,
                previous_member.handle.participant_id
            );
            assert_eq!(
                replacement_manifest.handle.world_generation,
                Some(replacement_binding.world_generation)
            );
            assert_eq!(
                replacement_manifest.handle.world_id.as_deref(),
                Some(replacement_binding.world_id.as_str())
            );

            shutdown_host_orchestrator_runtime(
                member_runtime,
                &ReplPrinter::Stdout,
                &mut telemetry,
            )
            .await;
        });
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(unix)]
    #[test]
    fn write_best_effort_unix_completes_after_single_full_write() {
        let payload = b"abcdef";
        let calls = Cell::new(0usize);

        let written = write_best_effort_unix(payload, |remaining| {
            calls.set(calls.get() + 1);
            assert_eq!(remaining, payload);
            Ok(remaining.len())
        });

        assert_eq!(written, payload.len());
        assert_eq!(calls.get(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn write_best_effort_unix_drains_partial_writes() {
        let payload = b"abcdef";
        let calls = RefCell::new(Vec::<Vec<u8>>::new());

        let written = write_best_effort_unix(payload, |remaining| {
            calls.borrow_mut().push(remaining.to_vec());
            Ok(match remaining.len() {
                6 => 2,
                4 => 1,
                3 => 3,
                other => other,
            })
        });

        assert_eq!(written, payload.len());
        assert_eq!(
            calls.into_inner(),
            vec![b"abcdef".to_vec(), b"cdef".to_vec(), b"def".to_vec()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn write_best_effort_unix_retries_interrupted_writes() {
        let payload = b"abcdef";
        let calls = RefCell::new(Vec::<Vec<u8>>::new());
        let stage = Cell::new(0usize);

        let written = write_best_effort_unix(payload, |remaining| {
            calls.borrow_mut().push(remaining.to_vec());
            let next = stage.get();
            stage.set(next + 1);
            match next {
                0 => Err(io::Error::new(io::ErrorKind::Interrupted, "signal")),
                1 => Ok(2),
                2 => Err(io::Error::new(io::ErrorKind::Interrupted, "signal")),
                _ => Ok(remaining.len()),
            }
        });

        assert_eq!(written, payload.len());
        assert_eq!(
            calls.into_inner(),
            vec![
                b"abcdef".to_vec(),
                b"abcdef".to_vec(),
                b"cdef".to_vec(),
                b"cdef".to_vec()
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn write_best_effort_unix_stops_on_nonretryable_error() {
        let payload = b"abcdef";
        let calls = RefCell::new(Vec::<Vec<u8>>::new());

        let written = write_best_effort_unix(payload, |remaining| {
            calls.borrow_mut().push(remaining.to_vec());
            match remaining.len() {
                6 => Ok(2),
                _ => Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed")),
            }
        });

        assert_eq!(written, 2);
        assert_eq!(
            calls.into_inner(),
            vec![b"abcdef".to_vec(), b"cdef".to_vec()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn write_best_effort_unix_stops_on_zero_byte_write() {
        let payload = b"abcdef";
        let calls = RefCell::new(Vec::<Vec<u8>>::new());

        let written = write_best_effort_unix(payload, |remaining| {
            calls.borrow_mut().push(remaining.to_vec());
            match remaining.len() {
                6 => Ok(2),
                _ => Ok(0),
            }
        });

        assert_eq!(written, 2);
        assert_eq!(
            calls.into_inner(),
            vec![b"abcdef".to_vec(), b"cdef".to_vec()]
        );
    }

    #[test]
    fn parse_demo_burst_defaults() {
        assert_eq!(parse_demo_burst(":demo-burst"), Some((4, 400, 0)));
        assert_eq!(parse_demo_burst(":demo-burst   "), Some((4, 400, 0)));
    }

    #[test]
    fn parse_demo_burst_custom() {
        assert_eq!(parse_demo_burst(":demo-burst 2 5 10"), Some((2, 5, 10)));
        assert_eq!(parse_demo_burst(":demo-burst 3"), Some((3, 400, 0)));
        assert_eq!(parse_demo_burst(":demo-burst 3 9"), Some((3, 9, 0)));
    }
}
