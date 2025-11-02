use std::io::{self, Write};
use std::process::ExitStatus;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event as CtEvent, EventStream, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use futures::StreamExt;
use reedline::{EditCommand, ExternalPrinter, Reedline, Signal, SuspendGuard};
use tokio::runtime::Builder as TokioRuntimeBuilder;
use tokio::task;
use uuid::Uuid;

use super::is_shell_stream_event;
use crate::agent_events::{
    clear_agent_event_sender, format_event_line, init_event_channel, publish_command_completion,
    schedule_demo_burst, schedule_demo_events,
};
use crate::editor;
use crate::ReplSessionTelemetry;

use super::{execute_command, setup_signal_handlers, ShellConfig};

pub(super) fn run_async_repl(config: &ShellConfig) -> Result<i32> {
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
        let mut terminal_guard = RawTerminalGuard::new()
            .context("failed to prepare terminal for async REPL")?;
        let mut telemetry = ReplSessionTelemetry::new(shared_config.clone(), "async");

        let mut adapter = AsyncReedlineAdapter::new(&shared_config)
            .context("failed to initialize async Reedline adapter")?;
        adapter
            .begin_session()
            .context("failed to prepare async Reedline session")?;

        let mut agent_rx = init_event_channel();
        let agent_printer = adapter.printer_handle();

        let mut input_stream = EventStream::new().fuse();
        let mut should_exit = false;

        while !should_exit {
            tokio::select! {
                maybe_event = input_stream.next() => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            if let CtEvent::Key(ref key_event) = event {
                                if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                                    continue;
                                }
                                telemetry.record_input_event();
                            }

                            match adapter.handle_event(event)? {
                                AdapterAction::Continue => {}
                                AdapterAction::Submit(command) => {
                                                                        let trimmed = command.trim();

                                    if trimmed.is_empty() {
                                        adapter.render_prompt().context("failed to redraw prompt")?;
                                        continue;
                                    }

                                    if matches!(trimmed, "exit" | "quit") {
                                        should_exit = true;
                                        continue;
                                    }

                                    if trimmed == ":demo-agent" {
                                        schedule_demo_events();
                                        adapter.render_prompt().context("failed to redraw prompt after demo agent")?;
                                        continue;
                                    }

                                    if let Some((agents, events, delay_ms)) = parse_demo_burst(trimmed) {
                                        schedule_demo_burst(
                                            agents,
                                            events,
                                            Duration::from_millis(delay_ms),
                                        );
                                        println!(
                                            "[demo] scheduled burst: agents={}, events_per_agent={}, delay_ms={}",
                                            agents, events, delay_ms
                                        );
                                        adapter.render_prompt().context("failed to redraw prompt after demo burst")?;
                                        continue;
                                    }

                                    let trimmed_owned = trimmed.to_string();

                                    adapter.end_session();
                                    let reedline_guard = adapter.suspend_for_command();
                                    terminal_guard.pause()?;

                                    let config_clone = (*shared_config).clone();
                                    let cmd_id = Uuid::now_v7().to_string();
                                    let running_clone = running_child_pid.clone();
                                    let command_for_exec = command.clone();

                                    let status = task::spawn_blocking(move || {
                                        execute_command(&config_clone, &command_for_exec, &cmd_id, running_clone)
                                    })
                                    .await
                                    .context("command execution task failed")??;

                                    report_nonzero_status(&status);

                                    terminal_guard.resume()?;
                                    drop(reedline_guard);
                                    adapter
                                        .begin_session()
                                        .context("failed to resume async Reedline session")?;
                                    adapter
                                        .begin_session()
                                        .context("failed to resume async Reedline session")?;

                                    publish_command_completion(&trimmed_owned, &status);
                                    telemetry.record_command();
                                    adapter.render_prompt().context("failed to redraw prompt after command")?;
                                    let _ = adapter.sync_history();
                                }
                                AdapterAction::Interrupt => {
                                    println!("^C");
                                    adapter.render_prompt().context("failed to redraw prompt after interrupt")?;
                                }
                                AdapterAction::Exit => {
                                    should_exit = true;
                                }
                            }
                        }
                        Some(Err(e)) => {
                            eprintln!("stdin event error: {e}");
                            break;
                        }
                        None => break,
                    }
                }
                Some(event) = agent_rx.recv() => {
                    if is_shell_stream_event(&event) {
                        continue;
                    }
                    telemetry.record_agent_event();
                    let _ = agent_printer.print(format_event_line(&event));
                    adapter
                        .flush_external_messages()
                        .context("failed to flush agent output")?;
                }
            }
        }

        let _ = adapter.sync_history();
        adapter.end_session();
        clear_agent_event_sender();
        terminal_guard.pause()?;

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(0)
}

#[derive(Debug)]
enum AdapterAction {
    Continue,
    Submit(String),
    Interrupt,
    Exit,
}

struct AsyncReedlineAdapter {
    editor: Reedline,
    prompt: editor::SubstratePrompt,
    printer: ExternalPrinter<String>,
}

impl AsyncReedlineAdapter {
    fn new(config: &ShellConfig) -> Result<Self> {
        let editor::EditorSetup {
            line_editor,
            printer,
        } = editor::build_editor(config)?;
        Ok(Self {
            editor: line_editor,
            prompt: editor::make_prompt(config.ci_mode),
            printer,
        })
    }

    fn begin_session(&mut self) -> Result<()> {
        self.editor
            .begin_nonblocking_session(&self.prompt)
            .context("begin_nonblocking_session")?;
        let _ = self.editor.flush_external_messages(&self.prompt)?;
        Ok(())
    }

    fn end_session(&mut self) {
        self.editor.end_nonblocking_session();
    }

    fn handle_event(&mut self, event: CtEvent) -> Result<AdapterAction> {
        if let CtEvent::Key(ref key_event) = event {
            if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                return Ok(AdapterAction::Continue);
            }
        }
        if let CtEvent::Key(ref key_event) = event {
            if matches!(key_event.code, crossterm::event::KeyCode::Enter)
                && key_event.modifiers.is_empty()
            {
                if let Some(backspaces) =
                    continuation_backspaces(self.editor.current_buffer_contents())
                {
                    for _ in 0..backspaces {
                        self.editor.run_edit_commands(&[EditCommand::Backspace]);
                    }
                    self.editor.run_edit_commands(&[EditCommand::InsertNewline]);
                    self.render_prompt()?;
                    return Ok(AdapterAction::Continue);
                }
            }
        }

        if let Some(signal) = self.editor.process_events(&self.prompt, vec![event])? {
            return Ok(Self::signal_to_action(signal));
        }

        Ok(AdapterAction::Continue)
    }

    fn flush_external_messages(&mut self) -> Result<()> {
        let _ = self.editor.flush_external_messages(&self.prompt)?;
        Ok(())
    }

    fn suspend_for_command(&mut self) -> SuspendGuard<'_> {
        self.editor.suspend_guard()
    }

    fn render_prompt(&mut self) -> Result<()> {
        self.editor.force_repaint(&self.prompt)?;
        Ok(())
    }

    fn sync_history(&mut self) -> io::Result<()> {
        self.editor.sync_history()
    }

    fn printer_handle(&self) -> ExternalPrinter<String> {
        self.printer.clone()
    }

    fn signal_to_action(signal: Signal) -> AdapterAction {
        match signal {
            Signal::Success(command) => AdapterAction::Submit(command),
            Signal::CtrlC => AdapterAction::Interrupt,
            Signal::CtrlD => AdapterAction::Exit,
        }
    }
}

struct RawTerminalGuard {
    active: bool,
}

impl RawTerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode().context("enable raw mode")?;
        io::stdout().execute(Hide).context("hide cursor")?;
        Ok(Self { active: true })
    }

    fn pause(&mut self) -> Result<()> {
        if self.active {
            io::stdout().execute(Show).context("show cursor")?;
            disable_raw_mode().context("disable raw mode")?;
            self.active = false;
        }
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        if !self.active {
            enable_raw_mode().context("enable raw mode")?;
            io::stdout().execute(Hide).context("hide cursor")?;
            self.active = true;
        }
        Ok(())
    }
}

impl Drop for RawTerminalGuard {
    fn drop(&mut self) {
        let _ = self.pause();
        let _ = io::stdout().flush();
    }
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

fn continuation_backspaces(buffer: &str) -> Option<usize> {
    let trimmed = buffer.trim_end_matches(|c: char| c == ' ' || c == '\t');
    let trailing_ws = buffer.len() - trimmed.len();
    if trimmed.ends_with('\\') {
        Some(trailing_ws + 1)
    } else {
        None
    }
}
