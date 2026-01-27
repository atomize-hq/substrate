use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use futures::{pin_mut, FutureExt};
use reedline::{ExternalPrinter, Reedline, Signal};
use tokio::runtime::Builder as TokioRuntimeBuilder;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task;
use uuid::Uuid;

use crate::execution::agent_events::{
    clear_agent_event_sender, format_event_line, init_event_channel, publish_command_completion,
    schedule_demo_burst, schedule_demo_events,
};
use crate::execution::ReplSessionTelemetry;
use crate::execution::{
    execute_command, find_workspace_root, get_terminal_size, is_shell_stream_event, needs_pty,
    policy_snapshot, setup_signal_handlers, MinimalTerminalGuard, ReplPersistentSessionClient,
    ReplSessionStartParams, ReplStdinMode, ShellConfig, PTY_ACTIVE,
};
use crate::repl::editor;
use substrate_common::agent_events::AgentEvent;

pub(crate) fn run_async_repl(config: &ShellConfig) -> Result<i32> {
    println!("Substrate v{}", env!("CARGO_PKG_VERSION"));
    println!("Session ID: {}", config.session_id);
    println!("Logging to: {}", config.trace_log_file.display());

    let running_child_pid = Arc::new(AtomicI32::new(0));
    setup_signal_handlers(running_child_pid.clone())?;

    let rt = TokioRuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize async REPL runtime")?;

    let shared_config = Arc::new(config.clone());

    rt.block_on(async move {
        let mut telemetry = ReplSessionTelemetry::new(shared_config.clone(), "async");
        let mut prompt_worker = PromptWorker::spawn(shared_config.clone())
            .context("failed to start Reedline worker")?;
        let agent_printer = prompt_worker.printer_handle();
        let mut prompt_responses = prompt_worker.take_response_receiver();
        let mut agent_rx = init_event_channel();

        let host_escape_enabled = shared_config.repl_host_escape;
        let mut host_state = HostState::new().context("failed to initialize host state")?;

        let (resize_tx, mut resize_rx) = mpsc::unbounded_channel::<(u16, u16)>();
        spawn_resize_task(resize_tx);

        let (sigint_tx, mut sigint_rx) = mpsc::unbounded_channel::<()>();
        spawn_sigint_task(sigint_tx);

        let stdout_cb = make_world_stdout_callback();
        let mut world_session = if !shared_config.no_world {
            let requested = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .display()
                .to_string();
            Some(
                start_world_session(requested, stdout_cb.clone(), &agent_printer)
                    .await
                    .context("failed to start persistent world session")?,
            )
        } else {
            None
        };

        let mut should_exit = false;
        while !should_exit {
            prompt_worker
                .request_prompt()
                .context("failed to request prompt")?;

            let prompt_response = loop {
                tokio::select! {
                    resp = prompt_responses.recv() => {
                        match resp {
                            Some(resp) => break resp,
                            None => return Err(anyhow!("prompt worker closed")),
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
                }
            };

            match prompt_response {
                PromptWorkerResponse::Line(command) => {
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
                        println!(
                            "[demo] scheduled burst: agents={}, events_per_agent={}, delay_ms={}",
                            agents, events, delay_ms
                        );
                        continue;
                    }

                    let trimmed_owned = trimmed.to_string();
                    telemetry.record_input_event();

                    let cmd_id = Uuid::now_v7().to_string();

                    if !has_embedded_newlines(&command) {
                        if trimmed == ":host" {
                            let _ = agent_printer.print(
                                "substrate: error: :host requires a command".to_string(),
                            );
                            continue;
                        }
                        if trimmed == ":pty" {
                            let _ =
                                agent_printer.print("substrate: error: :pty requires a command".to_string());
                            continue;
                        }

                        if let Some(rest) = command.strip_prefix(":host ") {
                            let host_cmd = rest.trim_start();
                            if host_cmd.is_empty() {
                                let _ = agent_printer.print(
                                    "substrate: error: :host requires a command".to_string(),
                                );
                                continue;
                            }
                            if !host_escape_enabled {
                                let _ = agent_printer.print(
                                    "substrate: error: host escape not enabled (use --repl-host-escape or SUBSTRATE_REPL_HOST_ESCAPE=1)".to_string(),
                                );
                                continue;
                            }

                            let mut io_ctx = ReplIo {
                                agent_rx: &mut agent_rx,
                                resize_rx: &mut resize_rx,
                                sigint_rx: &mut sigint_rx,
                                telemetry: &mut telemetry,
                                agent_printer: &agent_printer,
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
                            publish_command_completion(&trimmed_owned, &status);
                            telemetry.record_command();
                            continue;
                        }

                        if let Some(rest) = command.strip_prefix(":pty ") {
                            let pty_cmd = rest.trim_start();
                            if pty_cmd.is_empty() {
                                let _ = agent_printer.print(
                                    "substrate: error: :pty requires a command".to_string(),
                                );
                                continue;
                            }

                            if world_session.is_some() {
                                ensure_no_policy_drift(&mut world_session, &agent_printer).await?;
                                let session = world_session
                                    .as_mut()
                                    .expect("world_session present after ensure_no_policy_drift");
                                let mut io_ctx = ReplIo {
                                    agent_rx: &mut agent_rx,
                                    resize_rx: &mut resize_rx,
                                    sigint_rx: &mut sigint_rx,
                                    telemetry: &mut telemetry,
                                    agent_printer: &agent_printer,
                                };
                                let exit_code =
                                    exec_world_pty(session, pty_cmd, &cmd_id, &mut io_ctx).await?;
                                let status = exit_status_from_code(exit_code);
                                report_nonzero_status(&status);
                                publish_command_completion(&trimmed_owned, &status);
                                telemetry.record_command();
                                continue;
                            }
                        }
                    }

                    if world_session.is_some() {
                        ensure_no_policy_drift(&mut world_session, &agent_printer).await?;
                        let session = world_session
                            .as_mut()
                            .expect("world_session present after ensure_no_policy_drift");
                        let pty = needs_pty(trimmed);
                        let mut io_ctx = ReplIo {
                            agent_rx: &mut agent_rx,
                            resize_rx: &mut resize_rx,
                            sigint_rx: &mut sigint_rx,
                            telemetry: &mut telemetry,
                            agent_printer: &agent_printer,
                        };
                        let exit_code = if pty {
                            exec_world_pty(session, &command, &cmd_id, &mut io_ctx).await?
                        } else {
                            exec_world_line(session, &command, &cmd_id, &mut io_ctx).await?
                        };
                        let status = exit_status_from_code(exit_code);
                        report_nonzero_status(&status);
                        publish_command_completion(&trimmed_owned, &status);
                        telemetry.record_command();
                        continue;
                    }

                    // Host-only mode (explicit --no-world)
                    let config_clone = (*shared_config).clone();
                    let running_clone = running_child_pid.clone();
                    let command_for_exec = command.clone();
                    let command_fut = task::spawn_blocking(move || {
                        execute_command(&config_clone, &command_for_exec, &cmd_id, running_clone)
                    })
                    .map(|res: Result<Result<ExitStatus, anyhow::Error>, tokio::task::JoinError>| {
                        match res {
                            Ok(inner) => inner,
                            Err(err) => Err(anyhow!(err)),
                        }
                    });
                    pin_mut!(command_fut);

                    let status = loop {
                        tokio::select! {
                            res = &mut command_fut => break res?,
                            maybe_event = agent_rx.recv() => {
                                if let Some(event) = maybe_event {
                                    handle_agent_event(event, &mut telemetry, &agent_printer);
                                }
                            }
                            _maybe_resize = resize_rx.recv() => {}
                            _maybe_sigint = sigint_rx.recv() => {}
                        }
                    };

                    report_nonzero_status(&status);
                    publish_command_completion(&trimmed_owned, &status);
                    telemetry.record_command();
                }
                PromptWorkerResponse::CtrlC => {
                    println!("^C");
                }
                PromptWorkerResponse::CtrlD => {
                    println!("^D");
                    should_exit = true;
                }
                PromptWorkerResponse::Error(err) => {
                    eprintln!("prompt error: {err}");
                    should_exit = true;
                }
            }
        }

        if let Some(session) = world_session.take() {
            let _ = session.client.close().await;
        }

        prompt_worker.shutdown();
        clear_agent_event_sender();

        io::stdout().flush().ok();

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(0)
}

struct PromptWorker {
    command_tx: UnboundedSender<PromptWorkerCommand>,
    join_handle: Option<thread::JoinHandle<()>>,
    response_rx: Option<UnboundedReceiver<PromptWorkerResponse>>,
    printer: ExternalPrinter<String>,
}

impl PromptWorker {
    fn spawn(config: Arc<ShellConfig>) -> Result<Self> {
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
            printer,
        })
    }

    fn request_prompt(&self) -> Result<()> {
        self.command_tx
            .send(PromptWorkerCommand::StartPrompt)
            .map_err(|_| anyhow!("prompt worker stopped"))
    }

    fn shutdown(&mut self) {
        let _ = self.command_tx.send(PromptWorkerCommand::Shutdown);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }

    fn printer_handle(&self) -> ExternalPrinter<String> {
        self.printer.clone()
    }

    fn take_response_receiver(&mut self) -> UnboundedReceiver<PromptWorkerResponse> {
        // UnboundedReceiver doesn't implement Clone, so we move it out by replacing with an empty channel.
        self.response_rx
            .take()
            .expect("response receiver already taken")
    }
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
    Error(anyhow::Error),
}

fn handle_agent_event(
    event: AgentEvent,
    telemetry: &mut ReplSessionTelemetry,
    agent_printer: &ExternalPrinter<String>,
) {
    if is_shell_stream_event(&event) {
        return;
    }

    telemetry.record_agent_event();
    let _ = agent_printer.print(format_event_line(&event));
}

fn report_nonzero_status(status: &ExitStatus) {
    if status.success() {
        return;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            eprintln!("Command terminated by signal {sig}");
            return;
        }
    }

    eprintln!(
        "Command failed with status: {}",
        status.code().unwrap_or(-1)
    );
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
    world_cwd: String,
    snapshot_hash: String,
    workspace_root: Option<PathBuf>,
    on_stdout: StdoutCallback,
}

struct ReplIo<'a> {
    agent_rx: &'a mut UnboundedReceiver<AgentEvent>,
    resize_rx: &'a mut UnboundedReceiver<(u16, u16)>,
    sigint_rx: &'a mut UnboundedReceiver<()>,
    telemetry: &'a mut ReplSessionTelemetry,
    agent_printer: &'a ExternalPrinter<String>,
}

fn has_embedded_newlines(s: &str) -> bool {
    s.contains('\n') || s.contains('\r')
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

fn make_world_stdout_callback() -> StdoutCallback {
    Arc::new(|bytes: &[u8]| {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(bytes);
        let _ = stdout.flush();
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
    agent_printer: &ExternalPrinter<String>,
) -> Result<WorldSession> {
    let requested_path = Path::new(&requested_cwd);
    let resolved_start = policy_snapshot::resolve_policy_snapshot_for_cwd(requested_path)
        .context("policy snapshot (start)")?;
    let start_hash = resolved_start.snapshot_hash.clone();
    let start_workspace_root = find_workspace_root(requested_path);

    let start_params = ReplSessionStartParams::for_cwd_and_snapshot(
        requested_cwd.clone(),
        resolved_start.snapshot,
    );
    let client = ReplPersistentSessionClient::start_with(start_params, on_stdout.clone()).await?;
    let ready = client.ready().clone();

    if ready.cwd != requested_cwd {
        let _ = agent_printer.print(format!(
            "substrate: note: world session started in {} (requested {})",
            ready.cwd, requested_cwd
        ));
    }

    let ready_path = Path::new(&ready.cwd);
    let resolved_ready = policy_snapshot::resolve_policy_snapshot_for_cwd(ready_path)
        .context("policy snapshot (ready.cwd)")?;
    let ready_hash = resolved_ready.snapshot_hash.clone();
    let ready_workspace_root = find_workspace_root(ready_path);

    if ready_hash != start_hash || ready_workspace_root != start_workspace_root {
        let _ = agent_printer.print(
            "substrate: note: world session restarting due to snapshot/workspace drift before first command".to_string(),
        );
        client.close().await?;

        let restart_params = ReplSessionStartParams::for_cwd_and_snapshot(
            ready.cwd.clone(),
            resolved_ready.snapshot,
        );
        let client =
            ReplPersistentSessionClient::start_with(restart_params, on_stdout.clone()).await?;
        let ready2 = client.ready().clone();

        if ready2.cwd != ready.cwd {
            let _ = agent_printer.print(format!(
                "substrate: note: world session restarted in {} (requested {})",
                ready2.cwd, ready.cwd
            ));
        }

        return Ok(WorldSession {
            client,
            world_cwd: ready2.cwd,
            snapshot_hash: ready_hash,
            workspace_root: ready_workspace_root,
            on_stdout,
        });
    }

    Ok(WorldSession {
        client,
        world_cwd: ready.cwd,
        snapshot_hash: start_hash,
        workspace_root: start_workspace_root,
        on_stdout,
    })
}

async fn ensure_no_policy_drift(
    world_session: &mut Option<WorldSession>,
    agent_printer: &ExternalPrinter<String>,
) -> Result<()> {
    let Some(session) = world_session.as_ref() else {
        return Ok(());
    };

    let path = Path::new(&session.world_cwd);
    let resolved = policy_snapshot::resolve_policy_snapshot_for_cwd(path)
        .context("policy snapshot (drift)")?;
    let workspace_root = find_workspace_root(path);

    if resolved.snapshot_hash == session.snapshot_hash && workspace_root == session.workspace_root {
        return Ok(());
    }

    let _ = agent_printer.print(
        "substrate: note: world session restarting due to snapshot/workspace drift".to_string(),
    );

    let old = world_session
        .take()
        .expect("world_session present if session was Some above");
    let requested = old.world_cwd.clone();
    let on_stdout = old.on_stdout.clone();
    old.client.close().await?;

    *world_session = Some(start_world_session(requested, on_stdout, agent_printer).await?);
    Ok(())
}

async fn exec_world_line(
    session: &mut WorldSession,
    program: &str,
    cmd_id: &str,
    io: &mut ReplIo<'_>,
) -> Result<i32> {
    let fut = session
        .client
        .exec(program, ReplStdinMode::Eof, cmd_id)
        .map(|res| res.map(|c| (c.exit, c.cwd)));
    pin_mut!(fut);

    loop {
        tokio::select! {
            res = &mut fut => {
                let (exit, cwd) = res?;
                session.world_cwd = cwd;
                return Ok(exit);
            }
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

    let fut = session
        .client
        .exec(program, ReplStdinMode::Passthrough, cmd_id)
        .map(|res| res.map(|c| (c.exit, c.cwd)));
    pin_mut!(fut);

    let (exit, cwd) = loop {
        tokio::select! {
            res = &mut fut => break res?,
            maybe_bytes = stdin_rx.recv() => {
                if let Some(bytes) = maybe_bytes {
                    let _ = session.client.send_stdin(&bytes).await;
                }
            }
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
    };

    stdin_done.store(true, Ordering::Relaxed);
    let _ = stdin_thread.join();

    session.world_cwd = cwd;
    Ok(exit)
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
    if let Some(code) = try_run_host_builtin(host_state, line, io.agent_printer)? {
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
    host_state: &mut HostState,
    line: &str,
    agent_printer: &ExternalPrinter<String>,
) -> Result<Option<i32>> {
    let tokens = shell_words::split(line)
        .unwrap_or_else(|_| line.split_whitespace().map(|s| s.to_string()).collect());
    if tokens.is_empty() {
        return Ok(Some(0));
    }

    match tokens[0].as_str() {
        "pwd" => {
            let _ = agent_printer.print(format!("{}", host_state.cwd.display()));
            Ok(Some(0))
        }
        "cd" => {
            let target = tokens.get(1).map(String::as_str).unwrap_or("~");
            let expanded = shellexpand::tilde(target).to_string();
            let next = PathBuf::from(expanded);
            let next = if next.is_absolute() {
                next
            } else {
                host_state.cwd.join(next)
            };

            if !next.is_dir() {
                let _ = agent_printer.print(format!(
                    "substrate: error: :host cd: not a directory: {}",
                    next.display()
                ));
                return Ok(Some(1));
            }

            host_state.cwd = next;
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
