use std::collections::VecDeque;
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
        drain_cursor_position_reports();

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
                                    drain_cursor_position_reports();
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
                                    drain_cursor_position_reports();
                                    drop(reedline_guard);
                                    adapter
                                        .begin_session()
                                        .context("failed to resume async Reedline session")?;
                                    drain_cursor_position_reports();

                                    publish_command_completion(&trimmed_owned, &status);
                                    telemetry.record_command();
                                    adapter
                                        .render_prompt()
                                        .context("failed to redraw prompt after command")?;
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
        drain_cursor_position_reports();
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
                let buffer = self.editor.current_buffer_contents();
                if let Some(backspaces) = continuation_backspaces(buffer) {
                    for _ in 0..backspaces {
                        self.editor.run_edit_commands(&[EditCommand::Backspace]);
                    }
                    self.editor.run_edit_commands(&[EditCommand::InsertNewline]);
                    self.render_prompt()?;
                    return Ok(AdapterAction::Continue);
                }

                if heredoc_requires_continuation(buffer) {
                    self.editor.run_edit_commands(&[EditCommand::InsertNewline]);
                    self.render_prompt()?;
                    return Ok(AdapterAction::Continue);
                }

                if command_requires_continuation(buffer) {
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
    let trimmed = buffer.trim_end_matches([' ', '\t']);
    let trailing_ws = buffer.len() - trimmed.len();
    if trimmed.ends_with('\\') {
        Some(trailing_ws + 1)
    } else {
        None
    }
}

#[derive(Debug, Clone)]
struct HeredocSpec {
    delimiter: String,
    strip_tabs: bool,
}

fn heredoc_requires_continuation(buffer: &str) -> bool {
    let mut pending: VecDeque<HeredocSpec> = VecDeque::new();

    for line in buffer.lines() {
        if pending.is_empty() {
            detect_heredocs_in_line(line, &mut pending);
        } else {
            consume_heredoc_line(line, &mut pending);
        }
    }

    !pending.is_empty()
}

fn detect_heredocs_in_line(line: &str, pending: &mut VecDeque<HeredocSpec>) {
    let mut rest = line;

    while let Some(idx) = rest.find("<<") {
        let after = &rest[idx + 2..];

        // Skip here-strings (<<<)
        if let Some(after) = after.strip_prefix('<') {
            rest = after;
            continue;
        }

        let (strip_tabs, remainder) = if let Some(remainder) = after.strip_prefix('-') {
            (true, remainder)
        } else {
            (false, after)
        };

        let remainder = remainder.trim_start();
        if remainder.is_empty() {
            break;
        }

        let (delimiter, consumed) = parse_heredoc_delimiter(remainder);
        if let Some(delimiter) = delimiter {
            pending.push_back(HeredocSpec {
                delimiter,
                strip_tabs,
            });
        }

        rest = &remainder[consumed..];
    }
}

fn consume_heredoc_line(line: &str, pending: &mut VecDeque<HeredocSpec>) {
    if let Some(spec) = pending.front() {
        let candidate = if spec.strip_tabs {
            line.trim_start_matches('\t')
        } else {
            line
        };

        if candidate == spec.delimiter {
            pending.pop_front();
        }
    }
}

fn parse_heredoc_delimiter(input: &str) -> (Option<String>, usize) {
    let mut chars = input.char_indices();

    match chars.next() {
        Some((_, '\'')) => parse_quoted_delimiter(input, '\''),
        Some((_, '"')) => parse_quoted_delimiter(input, '"'),
        Some((start, _)) => {
            let mut end = start;
            for (idx, ch) in input.char_indices().skip(1) {
                if ch.is_whitespace() || matches!(ch, ';' | '|' | '&' | '<' | '>' | '(' | ')') {
                    break;
                }
                end = idx;
            }

            let end = if end < input.len() {
                end + 1
            } else {
                input.len()
            };
            let delimiter = input[..end].trim_end();
            (Some(delimiter.to_string()), end)
        }
        None => (None, input.len()),
    }
}

fn parse_quoted_delimiter(input: &str, quote: char) -> (Option<String>, usize) {
    let mut chars = input.char_indices();
    let mut end = None;

    // Skip opening quote
    chars.next();

    for (idx, ch) in chars {
        if ch == quote {
            end = Some(idx);
            break;
        }
    }

    match end {
        Some(end_idx) => {
            let delimiter = input[1..end_idx].to_string();
            (Some(delimiter), end_idx + 1)
        }
        None => (None, input.len()),
    }
}

fn command_requires_continuation(buffer: &str) -> bool {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut single_quote = false;
    let mut double_quote = false;
    let mut escaped = false;
    let mut comment = false;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut square_depth = 0i32;
    let mut prev_char: Option<char> = None;

    let mut chars = buffer.char_indices().peekable();

    let finalize_token = |tok: &mut String, tokens: &mut Vec<String>| {
        if !tok.is_empty() {
            tokens.push(tok.clone());
            tok.clear();
        }
    };

    while let Some((_, ch)) = chars.next() {
        let next_char = chars.peek().map(|&(_, c)| c);

        if comment {
            if ch == '\n' {
                comment = false;
                finalize_token(&mut token, &mut tokens);
            }
            prev_char = Some(ch);
            continue;
        }

        if escaped {
            token.push(ch);
            escaped = false;
            prev_char = Some(ch);
            continue;
        }

        if single_quote {
            if ch == '\'' {
                single_quote = false;
                finalize_token(&mut token, &mut tokens);
            } else {
                token.push(ch);
            }
            prev_char = Some(ch);
            continue;
        }

        if double_quote {
            match ch {
                '"' => {
                    double_quote = false;
                    finalize_token(&mut token, &mut tokens);
                }
                '\\' => {
                    escaped = true;
                }
                _ => token.push(ch),
            }
            prev_char = Some(ch);
            continue;
        }

        // Not inside quotes from here down
        match ch {
            '\\' => {
                escaped = true;
            }
            '\'' => {
                finalize_token(&mut token, &mut tokens);
                single_quote = true;
            }
            '"' => {
                finalize_token(&mut token, &mut tokens);
                double_quote = true;
            }
            '#' => {
                if token.is_empty() && prev_char.is_none_or(|c| c.is_whitespace()) {
                    comment = true;
                    finalize_token(&mut token, &mut tokens);
                } else {
                    token.push(ch);
                }
            }
            '(' => {
                paren_depth += 1;
                finalize_token(&mut token, &mut tokens);
                tokens.push("(".to_string());
            }
            ')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
                finalize_token(&mut token, &mut tokens);
                tokens.push(")".to_string());
            }
            '{' => {
                brace_depth += 1;
                finalize_token(&mut token, &mut tokens);
                tokens.push("{".to_string());
            }
            '}' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                }
                finalize_token(&mut token, &mut tokens);
                tokens.push("}".to_string());
            }
            '[' => {
                square_depth += 1;
                finalize_token(&mut token, &mut tokens);
                tokens.push("[".to_string());
            }
            ']' => {
                if square_depth > 0 {
                    square_depth -= 1;
                }
                finalize_token(&mut token, &mut tokens);
                tokens.push("]".to_string());
            }
            ';' => {
                finalize_token(&mut token, &mut tokens);
                if next_char == Some(';') {
                    chars.next();
                    tokens.push(";;".to_string());
                } else {
                    tokens.push(";".to_string());
                }
            }
            '&' | '|' => {
                finalize_token(&mut token, &mut tokens);
                tokens.push(ch.to_string());
            }
            '<' => {
                finalize_token(&mut token, &mut tokens);
                if next_char == Some('<') {
                    chars.next();
                    if let Some('-') = chars.peek().map(|&(_, c)| c) {
                        chars.next();
                        tokens.push("<<-".to_string());
                    } else {
                        tokens.push("<<".to_string());
                    }
                } else {
                    tokens.push("<".to_string());
                }
            }
            '>' => {
                finalize_token(&mut token, &mut tokens);
                if next_char == Some('>') {
                    chars.next();
                    tokens.push(">>".to_string());
                } else {
                    tokens.push(">".to_string());
                }
            }
            '\n' | '\r' | '\t' | ' ' => {
                finalize_token(&mut token, &mut tokens);
            }
            _ => {
                token.push(ch);
            }
        }

        prev_char = Some(ch);
    }

    finalize_token(&mut token, &mut tokens);

    if single_quote || double_quote || escaped {
        return true;
    }

    if paren_depth > 0 || brace_depth > 0 || square_depth > 0 {
        return true;
    }

    let mut if_stack = 0i32;
    let mut loop_stack = 0i32;
    let mut pending_do = 0i32;
    let mut case_stack = 0i32;
    let mut brace_stack = 0i32;

    for token in tokens.iter().map(|s| s.as_str()) {
        match token {
            "if" => {
                if_stack += 1;
            }
            "fi" => {
                if if_stack > 0 {
                    if_stack -= 1;
                }
            }
            "while" | "until" | "select" | "for" => {
                pending_do += 1;
            }
            "do" => {
                if pending_do > 0 {
                    pending_do -= 1;
                }
                loop_stack += 1;
            }
            "done" => {
                if loop_stack > 0 {
                    loop_stack -= 1;
                }
            }
            "case" => {
                case_stack += 1;
            }
            "esac" => {
                if case_stack > 0 {
                    case_stack -= 1;
                }
            }
            "{" => {
                brace_stack += 1;
            }
            "}" => {
                if brace_stack > 0 {
                    brace_stack -= 1;
                }
            }
            _ => {}
        }
    }

    if if_stack > 0 || loop_stack > 0 || pending_do > 0 || case_stack > 0 || brace_stack > 0 {
        return true;
    }

    false
}

#[cfg(unix)]
fn drain_cursor_position_reports() {
    use libc::{fcntl, ioctl, read, FIONREAD, F_GETFL, F_SETFL, O_NONBLOCK};
    use std::io;
    use std::os::unix::io::AsRawFd;

    let stdin = io::stdin();
    let fd = stdin.as_raw_fd();
    unsafe {
        let original = fcntl(fd, F_GETFL);
        if original == -1 {
            return;
        }
        if fcntl(fd, F_SETFL, original | O_NONBLOCK) == -1 {
            return;
        }

        let mut buf = [0u8; 256];

        loop {
            let mut available: libc::c_int = 0;
            if ioctl(fd, FIONREAD, &mut available) == -1 {
                break;
            }

            if available <= 0 {
                break;
            }

            let to_read = available.clamp(1, buf.len() as libc::c_int) as usize;
            let read_bytes = read(fd, buf.as_mut_ptr() as *mut _, to_read);
            if read_bytes <= 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock
                    || err.kind() == io::ErrorKind::Interrupted
                {
                    continue;
                }
                break;
            }
        }

        let _ = fcntl(fd, F_SETFL, original);
    }
}

#[cfg(not(unix))]
fn drain_cursor_position_reports() {}
