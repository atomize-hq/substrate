use std::collections::HashMap;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use futures::{pin_mut, FutureExt};
use reedline::{ExternalPrinter, Prompt, Reedline, Signal};
use tokio::runtime::Builder as TokioRuntimeBuilder;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task;
use uuid::Uuid;

use crate::execution::agent_events::{
    clear_agent_event_sender, format_event_line, init_event_channel, orchestration_session_id,
    publish_agent_event, publish_command_completion, schedule_demo_burst, schedule_demo_events,
};
#[cfg(unix)]
use crate::execution::get_terminal_size;
use crate::execution::ReplSessionTelemetry;
use crate::execution::WorldRootSettings;
use crate::execution::{
    canonicalize_or, enforce_caged_destination, execute_command, find_workspace_root,
    is_shell_stream_event, needs_pty, policy_snapshot, resolve_world_root, setup_signal_handlers,
    MinimalTerminalGuard, ReplPersistentSessionClient, ReplSessionStartParams, ReplStdinMode,
    ShellConfig, PTY_ACTIVE,
};
use crate::repl::editor;
use substrate_broker::{detect_profile, world_fs_policy};
use substrate_common::agent_events::AgentEvent;

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
            return matches!(code, libc::ENOTTY | libc::EIO | libc::EBADF);
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
            return None;
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
        let mut world_session = if !shared_config.no_world {
            let requested = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .display()
                .to_string();
            match start_world_session(requested, stdout_cb.clone(), &agent_printer, &mut telemetry)
                .await
            {
                Ok(session) => Some(session),
                Err(err) => {
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

        let mut should_exit = false;
        let mut termination_cause = ReplTerminationCause::NormalExit;
        while !should_exit {
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
            prompt_active.store(false, Ordering::SeqCst);
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
                            let exit_code = exec_host_line(
                                shared_config.as_ref(),
                                &mut host_state,
                                host_cmd,
                                &cmd_id,
                                running_child_pid.clone(),
                                world_session.as_ref().map(|s| &s.client),
                                &mut io_ctx,
                            )
                            .await?;
                            let status = exit_status_from_code(exit_code);
                            report_nonzero_status(&status);
                            publish_command_completion(&trimmed_owned, &cmd_id, &status);
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
                                ensure_no_policy_drift(
                                    &mut world_session,
                                    &agent_printer,
                                    &mut telemetry,
                                )
                                .await?;
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
                                    exec_world_pty(session, pty_cmd, &cmd_id, &mut io_ctx).await?
                                };
                                ensure_no_policy_drift(
                                    &mut world_session,
                                    &agent_printer,
                                    &mut telemetry,
                                )
                                .await?;
                                let status = exit_status_from_code(exit_code);
                                report_nonzero_status(&status);
                                publish_command_completion(&trimmed_owned, &cmd_id, &status);
                                telemetry.record_command();
                                continue;
                            }
                        }
                    }

                    if world_session.is_some() {
                        ensure_no_policy_drift(
                            &mut world_session,
                            &agent_printer,
                            &mut telemetry,
                        )
                        .await?;
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
                                exec_world_pty(session, &command, &cmd_id, &mut io_ctx).await?
                            } else {
                                exec_world_line(session, &command, &cmd_id, &mut io_ctx).await?
                            }
                        };
                        ensure_no_policy_drift(
                            &mut world_session,
                            &agent_printer,
                            &mut telemetry,
                        )
                        .await?;
                        let status = exit_status_from_code(exit_code);
                        report_nonzero_status(&status);
                        publish_command_completion(&trimmed_owned, &cmd_id, &status);
                        telemetry.record_command();
                        continue;
                    }

                    // Host-only mode (explicit --no-world)
                    let host_pty_passthrough = trimmed.starts_with(":pty ") || needs_pty(trimmed);
                    let config_clone = (*shared_config).clone();
                    let running_clone = running_child_pid.clone();
                    let command_for_exec = command.clone();
                    let cmd_id_for_exec = cmd_id.clone();
                    let command_fut = task::spawn_blocking(move || {
                        execute_command(
                            &config_clone,
                            &command_for_exec,
                            &cmd_id_for_exec,
                            running_clone,
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
                            res = &mut command_fut => break res?,
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
                    publish_command_completion(&trimmed_owned, &cmd_id, &status);
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
                            prompt_worker = PromptWorker::spawn_stdio(shared_config.clone())
                                .context("failed to start plain prompt worker")?;
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
        // CI runners often drive Substrate through PTY harnesses like `script` where Reedline's
        // cursor position query can consume the piped input stream. Prefer a plain stdin-backed
        // prompt in CI to keep smoke runs deterministic.
        if config.ci_mode
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
    previous_world_id: &str,
    previous_world_generation: u64,
    new_world_id: &str,
    new_world_generation: u64,
    reason: WorldRestartReason,
) {
    let mut event = AgentEvent::alert(
        "shell",
        orchestration_session_id(),
        Uuid::now_v7().to_string(),
        "world_restarted",
        reason.message(),
    );
    event.role = Some("orchestrator".to_string());
    event.backend_id = Some("shell:repl".to_string());
    event.world_id = Some(previous_world_id.to_string());

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
    current_world_id: &str,
    current_world_generation: u64,
    reason: WorldRestartReason,
) -> AgentEvent {
    let mut event = AgentEvent::alert(
        "shell",
        orchestration_session_id(),
        Uuid::now_v7().to_string(),
        "world_restart_required",
        reason.restart_required_message(),
    );
    event.role = Some("orchestrator".to_string());
    event.backend_id = Some("shell:repl".to_string());
    event.world_id = Some(current_world_id.to_string());

    if let Some(data) = event.data.as_object_mut() {
        data.insert("reason".to_string(), serde_json::json!(reason.code()));
        data.insert(
            "required_action".to_string(),
            serde_json::json!("restart_world"),
        );
        data.insert("on_drift".to_string(), serde_json::json!("fail_closed"));
        data.insert("world_id".to_string(), serde_json::json!(current_world_id));
        data.insert(
            "world_generation".to_string(),
            serde_json::json!(current_world_generation),
        );
    }

    event
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
                requested_cwd,
                policy_snapshot,
                snapshot_hash,
                workspace_root,
                agent_printer,
                reason,
            )
            .await
        }
        crate::execution::config_model::WorldRestartOnDriftMode::FailClosed => {
            let alert = build_world_restart_required_alert(
                &old_session.world_id,
                old_session.world_generation,
                reason,
            );
            telemetry.persist_agent_event(&alert);
            telemetry.record_agent_event();
            agent_printer.print(format_event_line(&alert));
            let _ = old_session.client.close().await;
            Err(anyhow!(WorldRestartRequiredError::new(format!(
                "substrate: error: world restart required before continuing ({}, world_id={}, generation={})",
                reason.code(),
                old_session.world_id,
                old_session.world_generation,
            ))))
        }
    }
}

struct OpenWorldSessionRequest<'a> {
    requested_cwd: String,
    requested_path: &'a Path,
    resolved_policy_snapshot: agent_api_types::PolicySnapshotV3,
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
    let client = ReplPersistentSessionClient::start_with(start_params, on_stdout.clone()).await?;
    let ready = client.ready().clone();

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
    requested_cwd: String,
    policy_snapshot: agent_api_types::PolicySnapshotV3,
    snapshot_hash: String,
    workspace_root: Option<PathBuf>,
    agent_printer: &ReplPrinter,
    reason: WorldRestartReason,
) -> Result<WorldSession> {
    let requested_path = PathBuf::from(&requested_cwd);
    let on_stdout = old_session.on_stdout.clone();
    let previous_world_id = old_session.world_id.clone();
    let previous_world_generation = old_session.world_generation;

    old_session.client.close().await?;

    let new_session = open_world_session(OpenWorldSessionRequest {
        requested_cwd,
        requested_path: requested_path.as_path(),
        resolved_policy_snapshot: policy_snapshot,
        snapshot_hash,
        workspace_root,
        on_stdout,
        agent_printer,
        world_generation: previous_world_generation.saturating_add(1),
        restarted: true,
    })
    .await?;

    emit_world_restarted_alert(
        &previous_world_id,
        previous_world_generation,
        &new_session.world_id,
        new_session.world_generation,
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
    on_stdout: StdoutCallback,
    agent_printer: &ReplPrinter,
    telemetry: &mut ReplSessionTelemetry,
) -> Result<WorldSession> {
    let requested_path = Path::new(&requested_cwd);
    let resolved_start = policy_snapshot::resolve_policy_snapshot_for_cwd(requested_path)
        .context("policy snapshot (start)")?;
    let start_hash = resolved_start.snapshot_hash.clone();
    let start_workspace_root = find_workspace_root(requested_path);
    let session = open_world_session(OpenWorldSessionRequest {
        requested_cwd: requested_cwd.clone(),
        requested_path,
        resolved_policy_snapshot: resolved_start.snapshot,
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
    use std::cell::Cell;
    #[cfg(unix)]
    use std::cell::RefCell;

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
