use std::io::{self, Write};
use std::process::ExitStatus;
use std::sync::atomic::AtomicI32;
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
    execute_command, is_shell_stream_event, setup_signal_handlers, ShellConfig,
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

        let mut should_exit = false;
        while !should_exit {
            prompt_worker
                .request_prompt()
                .context("failed to request prompt")?;

            let prompt_response = await_prompt_response(
                &mut prompt_responses,
                &mut agent_rx,
                &mut telemetry,
                &agent_printer,
            )
            .await?;

            match prompt_response {
                PromptWorkerResponse::Line(command) => {
                    let trimmed = command.trim();

                    if trimmed.is_empty() {
                        continue;
                    }

                    if matches!(trimmed, "exit" | "quit") {
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

                    let config_clone = (*shared_config).clone();
                    let cmd_id = Uuid::now_v7().to_string();
                    let running_clone = running_child_pid.clone();
                    let command_for_exec = command.clone();

                    let command_fut = task::spawn_blocking(move || {
                        execute_command(&config_clone, &command_for_exec, &cmd_id, running_clone)
                    })
                    .map(
                        |res: Result<Result<ExitStatus, anyhow::Error>, tokio::task::JoinError>| {
                            match res {
                                Ok(inner) => inner,
                                Err(err) => Err(anyhow!(err)),
                            }
                        },
                    );

                    let status = await_with_agent(
                        command_fut,
                        &mut agent_rx,
                        &mut telemetry,
                        &agent_printer,
                    )
                    .await?;

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

async fn await_prompt_response(
    prompt_responses: &mut UnboundedReceiver<PromptWorkerResponse>,
    agent_rx: &mut UnboundedReceiver<AgentEvent>,
    telemetry: &mut ReplSessionTelemetry,
    agent_printer: &ExternalPrinter<String>,
) -> Result<PromptWorkerResponse> {
    loop {
        tokio::select! {
            resp = prompt_responses.recv() => {
                if let Some(resp) = resp {
                    return Ok(resp);
                } else {
                    return Err(anyhow!("prompt worker closed"));
                }
            }
            maybe_event = agent_rx.recv() => {
                if let Some(event) = maybe_event {
                    handle_agent_event(event, telemetry, agent_printer);
                } else {
                    // Channel closed; continue waiting for prompt response.
                }
            }
        }
    }
}

async fn await_with_agent<F, T>(
    future: F,
    agent_rx: &mut UnboundedReceiver<AgentEvent>,
    telemetry: &mut ReplSessionTelemetry,
    agent_printer: &ExternalPrinter<String>,
) -> Result<T>
where
    F: std::future::Future<Output = Result<T>> + Send,
{
    pin_mut!(future);
    loop {
        tokio::select! {
            res = &mut future => {
                return res;
            }
            maybe_event = agent_rx.recv() => {
                if let Some(event) = maybe_event {
                    handle_agent_event(event, telemetry, agent_printer);
                } else {
                    // Agent channel closed, keep waiting for command future.
                }
            }
        }
    }
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
