use std::io::{self, Write};
use std::process::ExitStatus;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::thread;

use anyhow::{Context, Result};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    Event as CtEvent, EventStream, KeyCode as CtKeyCode, KeyEvent, KeyEventKind,
    KeyModifiers as CtKeyModifiers,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use futures::StreamExt;
use serde_json::Value;
use std::time::Duration;
use tokio::runtime::Builder as TokioRuntimeBuilder;
use tokio::task;
use uuid::Uuid;

use crate::agent_events::{
    agent_event_sender, clear_agent_event_sender, init_event_channel, publish_agent_event,
};
use substrate_common::agent_events::{AgentEvent, AgentEventKind};

use super::{execute_command, setup_signal_handlers, ShellConfig};

const CLEAR_LINE: &str = "\r\x1b[K";

pub(super) fn run_async_repl(config: &ShellConfig) -> Result<i32> {
    println!("Substrate v{}", env!("CARGO_PKG_VERSION"));
    println!("Session ID: {}", config.session_id);
    println!("Logging to: {}", config.trace_log_file.display());

    let running_child_pid = Arc::new(AtomicI32::new(0));
    setup_signal_handlers(running_child_pid.clone())?;

    let prompt = if config.ci_mode { "> " } else { "substrate> " };
    let prompt = prompt.to_string();

    let rt = TokioRuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize async REPL runtime")?;

    let shared_config = Arc::new(config.clone());

    rt.block_on(async move {
        let mut terminal_guard = RawTerminalGuard::new()
            .context("failed to prepare terminal for async REPL")?;

        let mut stdout = io::stdout();
        redraw_prompt(&mut stdout, &prompt, "")?;

        let mut agent_rx = init_event_channel();

        let mut input_stream = EventStream::new().fuse();
        let mut current_input = String::new();
        let mut should_exit = false;

        while !should_exit {
            tokio::select! {
                maybe_event = input_stream.next() => {
                    match maybe_event {
                        Some(Ok(CtEvent::Key(key_event))) => {
                            if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                                continue;
                            }

                            if handle_control_key(
                                &key_event,
                                &mut stdout,
                                &prompt,
                                &mut current_input,
                                &mut should_exit,
                            )? {
                                continue;
                            }

                            match key_event.code {
                                CtKeyCode::Char(c) => {
                                    current_input.push(c);
                                    write!(stdout, "{c}")?;
                                    stdout.flush().ok();
                                }
                                CtKeyCode::Backspace => {
                                    if current_input.pop().is_some() {
                                        write!(stdout, "\u{8}\x1b[K")?;
                                        stdout.flush().ok();
                                    }
                                }
                                CtKeyCode::Tab => {
                                    current_input.push('\t');
                                    write!(stdout, "\t")?;
                                    stdout.flush().ok();
                                }
                                CtKeyCode::Enter => {
                                    let command = std::mem::take(&mut current_input);
                                    write!(stdout, "\r\n")?;
                                    stdout.flush().ok();

                                    let trimmed = command.trim();
                                    if trimmed.is_empty() {
                                        redraw_prompt(&mut stdout, &prompt, &current_input)?;
                                        continue;
                                    }

                                    if matches!(trimmed, "exit" | "quit") {
                                        should_exit = true;
                                        continue;
                                    }

                                    if trimmed == ":demo-agent" {
                                        schedule_demo_events();
                                        redraw_prompt(&mut stdout, &prompt, &current_input)?;
                                        continue;
                                    }

                                    let trimmed_owned = trimmed.to_string();

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
                                    emit_command_event(&trimmed_owned, &status);
                                    redraw_prompt(&mut stdout, &prompt, &current_input)?;
                                    // Re-create the event stream to keep crossterm in sync after 
                                    // toggling raw mode for the executed command.
                                    input_stream = EventStream::new().fuse();
                                }
                                CtKeyCode::Esc => {
                                    current_input.clear();
                                    redraw_prompt(&mut stdout, &prompt, &current_input)?;
                                }
                                _ => {}
                            }
                        }
                        Some(Ok(CtEvent::Resize(_, _))) => {
                            redraw_prompt(&mut stdout, &prompt, &current_input)?;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            eprintln!("stdin event error: {e}");
                            break;
                        }
                        None => break,
                    }
                }
                Some(event) = agent_rx.recv() => {
                    render_agent_event(&mut stdout, &prompt, &current_input, &event)?;
                }
            }
        }

        clear_agent_event_sender();
        terminal_guard.pause()?;

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(0)
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

fn redraw_prompt(stdout: &mut io::Stdout, prompt: &str, buffer: &str) -> io::Result<()> {
    write!(stdout, "{CLEAR_LINE}{prompt}{buffer}")?;
    stdout.flush()
}

fn render_agent_event(
    stdout: &mut io::Stdout,
    prompt: &str,
    buffer: &str,
    event: &AgentEvent,
) -> io::Result<()> {
    let agent = if event.agent_id.is_empty() {
        "agent"
    } else {
        event.agent_id.as_str()
    };
    let message = extract_event_message(&event.kind, &event.data);
    write!(stdout, "{CLEAR_LINE}[{agent}] {message}\r\n")?;
    redraw_prompt(stdout, prompt, buffer)
}

fn handle_control_key(
    key_event: &KeyEvent,
    stdout: &mut io::Stdout,
    prompt: &str,
    current_input: &mut String,
    should_exit: &mut bool,
) -> Result<bool> {
    if !key_event.modifiers.contains(CtKeyModifiers::CONTROL) {
        return Ok(false);
    }

    match key_event.code {
        CtKeyCode::Char('c') | CtKeyCode::Char('C') => {
            current_input.clear();
            write!(stdout, "\r\n")?;
            redraw_prompt(stdout, prompt, current_input)?;
            return Ok(true);
        }
        CtKeyCode::Char('d') | CtKeyCode::Char('D') => {
            if current_input.is_empty() {
                *should_exit = true;
                write!(stdout, "\r\n")?;
            }
            return Ok(true);
        }
        _ => Ok(false),
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

fn emit_command_event(command: &str, status: &ExitStatus) {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if status.signal().is_some() {
            return;
        }
    }

    let event = if status.success() {
        AgentEvent::message(
            "shell",
            AgentEventKind::TaskEnd,
            format!("Command `{command}` completed successfully"),
        )
    } else {
        let code = status.code().unwrap_or(-1);
        AgentEvent::message(
            "shell",
            AgentEventKind::Alert,
            format!("Command `{command}` exited with status {code}"),
        )
    };

    let _ = publish_agent_event(event);
}

fn schedule_demo_events() {
    if agent_event_sender().is_none() {
        return;
    }

    let events = vec![
        (
            Duration::from_millis(300),
            "Demo agent event #1".to_string(),
        ),
        (
            Duration::from_millis(820),
            "Demo agent event #2".to_string(),
        ),
        (
            Duration::from_millis(1350),
            "Demo agent event #3".to_string(),
        ),
    ];

    thread::spawn(move || {
        for (delay, message) in events {
            thread::sleep(delay);
            let _ = publish_agent_event(AgentEvent::message(
                "demo",
                AgentEventKind::TaskProgress,
                message,
            ));
        }
    });
}

fn extract_event_message(kind: &AgentEventKind, data: &Value) -> String {
    if let Some(msg) = data.get("message").and_then(Value::as_str) {
        return msg.to_string();
    }

    if let Some(chunk) = data.get("chunk").and_then(Value::as_str) {
        let stream = data
            .get("stream")
            .and_then(Value::as_str)
            .unwrap_or("stdout");
        return format!("{}: {}", stream, chunk);
    }

    if data.is_null() {
        kind.to_string()
    } else {
        data.to_string()
    }
}
