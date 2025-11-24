//! Invocation execution helpers.

use super::plan::{ShellConfig, ShellMode};
use crate::execution::agent_events::{
    clear_agent_event_sender, format_event_line, init_event_channel, publish_command_completion,
    schedule_demo_burst, schedule_demo_events,
};
use crate::execution::{
    configure_child_shell_env, execute_command, is_shell_stream_event, log_command_event,
    parse_demo_burst_command, setup_signal_handlers, ReplSessionTelemetry,
};
use crate::repl::editor;
use anyhow::{Context, Result};
use reedline::Signal;
use serde_json::json;
use std::io::{self, BufRead};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use substrate_common::log_schema;
use uuid::Uuid;

pub(crate) fn run_interactive_shell(config: &ShellConfig) -> Result<i32> {
    println!("Substrate v{}", env!("CARGO_PKG_VERSION"));
    println!("Session ID: {}", config.session_id);
    println!("Logging to: {}", config.trace_log_file.display());

    let mut telemetry = ReplSessionTelemetry::new(Arc::new(config.clone()), "sync");

    let prompt = editor::make_prompt(config.ci_mode);

    let editor::EditorSetup {
        mut line_editor,
        printer,
    } = editor::build_editor(config)?;

    let mut agent_rx = init_event_channel();

    let renderer_handle = thread::spawn(move || {
        let printer = printer;
        while let Some(event) = agent_rx.blocking_recv() {
            if is_shell_stream_event(&event) {
                continue;
            }
            let line = format_event_line(&event);
            if printer.print(line).is_err() {
                break;
            }
        }
    });

    // Set up the host command decider for PTY commands

    // Signal handling setup
    let running_child_pid = Arc::new(AtomicI32::new(0));
    setup_signal_handlers(running_child_pid.clone())?;

    // Main REPL loop
    loop {
        let sig = line_editor.read_line(&prompt);

        match sig {
            Ok(Signal::Success(line)) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                // Check for exit commands
                if matches!(trimmed, "exit" | "quit") {
                    break;
                }

                if trimmed == ":demo-agent" {
                    schedule_demo_events();
                    continue;
                }

                if let Some((agents, events, delay_ms)) = parse_demo_burst_command(trimmed) {
                    schedule_demo_burst(agents, events, std::time::Duration::from_millis(delay_ms));
                    println!(
                        "[demo] scheduled burst: agents={}, events_per_agent={}, delay_ms={}",
                        agents, events, delay_ms
                    );
                    continue;
                }

                let cmd_id = Uuid::now_v7().to_string();

                match execute_command(config, &line, &cmd_id, running_child_pid.clone()) {
                    Ok(status) => {
                        if !status.success() {
                            #[cfg(unix)]
                            if let Some(sig) = status.signal() {
                                eprintln!("Command terminated by signal {sig}");
                            } else {
                                eprintln!(
                                    "Command failed with status: {}",
                                    status.code().unwrap_or(-1)
                                );
                            }
                            #[cfg(not(unix))]
                            eprintln!(
                                "Command failed with status: {}",
                                status.code().unwrap_or(-1)
                            );
                        }

                        publish_command_completion(trimmed, &status);
                        telemetry.record_command();
                    }
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            Ok(Signal::CtrlC) => {
                println!("^C");
                // Reedline handles this better than rustyline
            }
            Ok(Signal::CtrlD) => {
                println!("^D");
                break;
            }
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        }
    }

    // Save history before exit
    if let Err(e) = line_editor.sync_history() {
        log::warn!("Failed to save history: {e}");
    }

    clear_agent_event_sender();
    let _ = renderer_handle.join();

    Ok(0)
}

pub(crate) fn run_wrap_mode(config: &ShellConfig, command: &str) -> Result<i32> {
    let cmd_id = Uuid::now_v7().to_string();
    let running_child_pid = Arc::new(AtomicI32::new(0));

    // Set up signal handlers for wrap mode to properly handle signals like SIGTERM
    setup_signal_handlers(running_child_pid.clone())?;

    let status = execute_command(config, command, &cmd_id, running_child_pid)?;
    Ok(exit_code(status))
}

#[cfg(unix)]
pub(crate) fn exit_code(status: ExitStatus) -> i32 {
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(1))
}

#[cfg(not(unix))]
pub(crate) fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}

pub(crate) fn run_script_mode(config: &ShellConfig, script_path: &Path) -> Result<i32> {
    // Verify script exists and is readable
    std::fs::metadata(script_path)
        .with_context(|| format!("Failed to stat script: {}", script_path.display()))?;

    let mut cmd = Command::new(&config.shell_path);
    let cmd_id = Uuid::now_v7().to_string();
    let running_child_pid = Arc::new(AtomicI32::new(0));

    // Set up signal handlers for script mode to properly handle signals like SIGTERM
    setup_signal_handlers(running_child_pid.clone())?;

    // Shell-specific script execution
    let shell_name = Path::new(&config.shell_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let is_pwsh = shell_name == "pwsh.exe" || shell_name == "pwsh";
    let is_powershell = shell_name == "powershell.exe" || shell_name == "powershell";
    let is_cmd = shell_name == "cmd.exe" || shell_name == "cmd";
    let is_bash = shell_name == "bash" || shell_name == "bash.exe";

    if cfg!(windows) && (is_pwsh || is_powershell) {
        // PowerShell
        if config.ci_mode && !config.no_exit_on_error {
            cmd.arg("-NoProfile").arg("-NonInteractive");
        } else {
            cmd.arg("-NoProfile");
        }
        cmd.arg("-File").arg(script_path);
    } else if is_cmd {
        // Windows CMD
        cmd.arg("/C").arg(script_path);
    } else {
        // POSIX shells
        if config.ci_mode && !config.no_exit_on_error && is_bash {
            cmd.arg("-o")
                .arg("errexit")
                .arg("-o")
                .arg("pipefail")
                .arg("-o")
                .arg("nounset");
        }
        cmd.arg(script_path);
    }

    // Propagate environment
    cmd.env("SHIM_SESSION_ID", &config.session_id)
        .env("SHIM_TRACE_LOG", &config.trace_log_file)
        .env_remove("SHIM_ACTIVE") // Clear to allow shims to work
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    configure_child_shell_env(
        &mut cmd,
        config,
        is_bash,
        matches!(config.mode, ShellMode::Script(_)),
    );

    // Make child process a group leader on Unix
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                // Safety: setpgid is safe when called before exec
                libc::setpgid(0, 0);
                Ok(())
            });
        }
    }

    // Log script execution start
    let script_cmd = format!("{} {}", config.shell_path, script_path.display());
    log_command_event(config, "command_start", &script_cmd, &cmd_id, None)?;
    let start_time = std::time::Instant::now();

    // Execute script as single process
    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to execute script: {}", script_path.display()))?;

    let child_pid = child.id() as i32;
    running_child_pid.store(child_pid, Ordering::SeqCst);

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait for script: {}", script_path.display()))?;

    running_child_pid.store(0, Ordering::SeqCst);

    // Log script completion
    let duration = start_time.elapsed();
    #[allow(unused_mut)]
    #[cfg(unix)]
    let mut extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });

    #[cfg(not(unix))]
    let extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });

    #[cfg(unix)]
    {
        if let Some(sig) = status.signal() {
            extra["term_signal"] = json!(sig);
        }
    }

    log_command_event(
        config,
        "command_complete",
        &script_cmd,
        &cmd_id,
        Some(extra),
    )?;

    Ok(exit_code(status))
}

pub(crate) fn run_pipe_mode(config: &ShellConfig) -> Result<i32> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut last_status = 0;

    // No signal handler for pipe mode - inherit from parent
    let no_signal_handler = Arc::new(AtomicI32::new(0));

    // Stream line by line without loading entire input
    for line in reader.lines() {
        let line = line.context("Failed to read from stdin")?;

        if line.trim().is_empty() {
            continue;
        }

        let cmd_id = Uuid::now_v7().to_string();
        match execute_command(config, &line, &cmd_id, no_signal_handler.clone()) {
            Ok(status) => {
                last_status = exit_code(status);
                if !status.success() && config.ci_mode && !config.no_exit_on_error {
                    eprintln!("Command failed: {line}");
                    return Ok(last_status);
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                if !config.no_exit_on_error {
                    return Ok(1);
                }
                last_status = 1;
            }
        }
    }

    Ok(last_status)
}
