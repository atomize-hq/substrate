use super::cli::*;
use super::invocation::{
    run_interactive_shell, run_pipe_mode, run_script_mode, run_wrap_mode, ShellConfig, ShellMode,
};
mod builtin;
mod dispatch;
mod path_env;
mod replay;
mod telemetry;
#[cfg(test)]
mod test_utils;
mod world;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod world_env;
use super::shim_deploy::{DeploymentStatus, ShimDeployer};
use super::{configure_manager_init, log_manager_init_event, write_manager_env_script};
use crate::builtins as commands;
use crate::repl::async_repl;
use crate::scripts::write_bash_preexec_script;

use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;
use std::io::{self, IsTerminal};
// (avoid unused: import Read/Write locally where needed)
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use substrate_broker::{set_global_broker, BrokerHandle};
use substrate_trace::{init_trace, set_global_trace_context, TraceContext};
use tracing::warn;

// Reedline imports
#[cfg_attr(target_os = "windows", allow(unused_imports))]
use std::thread;
// use nu_ansi_term::{Color, Style}; // Unused for now
pub(crate) use self::world::initialize_world;
pub(crate) use dispatch::{
    build_agent_client_and_request, execute_command, parse_demo_burst_command,
    stream_non_pty_via_agent,
};
#[cfg(target_os = "linux")]
use nix::sys::termios::{
    self, ControlFlags, InputFlags, LocalFlags, OutputFlags, SetArg, SpecialCharacterIndices,
    Termios,
};
pub(crate) use path_env::world_deps_manifest_base_path;
pub(crate) use replay::{handle_replay_command, handle_trace_command};
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;
pub(crate) use telemetry::{
    is_shell_stream_event, log_command_event, ReplSessionTelemetry, SHELL_AGENT_ID,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use world_env::world_transport_to_meta;
#[cfg(target_os = "linux")]
fn get_term_size() -> (u16, u16) {
    // Try to read the current terminal size; fall back to 80x24
    let fd = std::io::stdout().as_raw_fd();
    let mut ws: libc::winsize = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        if libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) == 0 {
            let cols = if ws.ws_col == 0 { 80 } else { ws.ws_col };
            let rows = if ws.ws_row == 0 { 24 } else { ws.ws_row };
            return (cols, rows);
        }
    }
    (80, 24)
}

#[cfg(target_os = "linux")]
struct RawModeGuard {
    file: std::fs::File,
    orig: Termios,
}

#[cfg(target_os = "linux")]
impl RawModeGuard {
    fn new_for_tty() -> anyhow::Result<Self> {
        // Open controlling terminal
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?;
        let mut tio = termios::tcgetattr(&file).map_err(|e| anyhow::anyhow!("tcgetattr: {}", e))?;
        let orig = tio.clone();
        // Configure raw mode (manual equivalent of cfmakeraw)
        tio.input_flags.remove(
            InputFlags::BRKINT
                | InputFlags::ICRNL
                | InputFlags::INPCK
                | InputFlags::ISTRIP
                | InputFlags::IXON,
        );
        tio.control_flags.insert(ControlFlags::CS8);
        tio.local_flags
            .remove(LocalFlags::ECHO | LocalFlags::ICANON | LocalFlags::IEXTEN | LocalFlags::ISIG);
        tio.output_flags.remove(OutputFlags::OPOST);
        // Set read to return as soon as 1 byte is available
        let vmin = SpecialCharacterIndices::VMIN as usize;
        let vtime = SpecialCharacterIndices::VTIME as usize;
        tio.control_chars[vmin] = 1;
        tio.control_chars[vtime] = 0;
        termios::tcsetattr(&file, SetArg::TCSANOW, &tio)
            .map_err(|e| anyhow::anyhow!("tcsetattr: {}", e))?;
        Ok(Self { file, orig })
    }

    fn for_stdin_if_tty() -> anyhow::Result<Option<Self>> {
        if !io::stdin().is_terminal() {
            return Ok(None);
        }
        match Self::new_for_tty() {
            Ok(g) => Ok(Some(g)),
            Err(e) => Err(e),
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = termios::tcsetattr(&self.file, SetArg::TCSANOW, &self.orig);
    }
}

// Global flag to prevent double SIGINT handling - must be pub(crate) for pty access
pub(crate) static PTY_ACTIVE: AtomicBool = AtomicBool::new(false);

pub(crate) fn handle_graph_command(cmd: &GraphCmd) -> Result<()> {
    use substrate_graph::{connect_mock, GraphConfig, GraphService};
    let cfg = GraphConfig {
        backend: "mock".into(),
        db_path: substrate_graph::default_graph_path()?,
    };
    let mut svc = connect_mock(cfg).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    svc.ensure_schema()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    match &cmd.action {
        GraphAction::Status => {
            let s = svc.status().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            println!("graph status: {}", s);
        }
        GraphAction::Ingest { file } => {
            use std::io::{BufRead, BufReader};
            let f = std::fs::File::open(file)?;
            let reader = BufReader::new(f);
            let mut n = 0usize;
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(span) = serde_json::from_str::<substrate_trace::Span>(&line) {
                    svc.ingest_span(&span)
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                    n += 1;
                }
            }
            println!("ingested {} spans (mock)", n);
        }
        GraphAction::WhatChanged { span_id, limit } => {
            let items = svc
                .what_changed(span_id, *limit)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            if items.is_empty() {
                println!("no changes recorded for span {}", span_id);
            } else {
                for fc in items {
                    println!("{}\t{}", fc.change, fc.path);
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn handle_shim_command(cmd: &ShimCmd, cli: &Cli) -> ! {
    match &cmd.action {
        ShimAction::Doctor { json } => {
            let exit = match commands::shim_doctor::run_doctor(*json, cli.no_world, cli.world) {
                Ok(_) => 0,
                Err(err) => {
                    eprintln!("substrate shim doctor failed: {:#}", err);
                    1
                }
            };
            std::process::exit(exit);
        }
        ShimAction::Repair { manager, yes } => {
            let exit = match commands::shim_doctor::run_repair(manager, *yes) {
                Ok(commands::shim_doctor::RepairOutcome::Applied {
                    manager,
                    bashenv_path,
                    backup_path,
                }) => {
                    if let Some(backup) = &backup_path {
                        println!(
                            "Updated {} with repair snippet for `{}` (backup at {})",
                            bashenv_path.display(),
                            manager,
                            backup.display()
                        );
                    } else {
                        println!(
                            "Created {} with repair snippet for `{}`",
                            bashenv_path.display(),
                            manager
                        );
                    }
                    0
                }
                Ok(commands::shim_doctor::RepairOutcome::Skipped { manager, reason }) => {
                    println!("No changes applied for `{}`: {}", manager, reason);
                    0
                }
                Err(err) => {
                    eprintln!("substrate shim repair failed: {:#}", err);
                    1
                }
            };
            std::process::exit(exit);
        }
    }
}

pub fn run_shell() -> Result<i32> {
    run_shell_with_cli(Cli::parse())
}

pub fn run_shell_with_cli(cli: Cli) -> Result<i32> {
    let mut config = ShellConfig::from_cli(cli)?;

    if matches!(config.mode, ShellMode::Interactive { .. }) {
        let stdin_is_tty = io::stdin().is_terminal();
        let stdout_is_tty = io::stdout().is_terminal();
        if !(stdin_is_tty && stdout_is_tty) {
            eprintln!(
                "substrate: no interactive TTY detected on stdin/stdout; exiting. Use -c, --script, or pipe commands instead."
            );
            return Ok(0);
        }
    }

    let _ = set_global_broker(BrokerHandle::new());
    let _ = set_global_trace_context(TraceContext::default());

    // Initialize trace
    if let Err(e) = init_trace(None) {
        eprintln!("substrate: warning: failed to initialize trace: {}", e);
    }

    let manager_init_result = if config.no_world {
        None
    } else {
        configure_manager_init(&config)
    };

    if let Some(result) = &manager_init_result {
        if let Err(err) = log_manager_init_event(&config, result) {
            warn!(
                target = "substrate::shell",
                error = %err,
                "failed to record manager init telemetry"
            );
        }
    }

    if config.no_world {
        env::remove_var("SUBSTRATE_MANAGER_INIT");
        env::remove_var("SUBSTRATE_MANAGER_ENV");
    } else {
        if let Err(err) = write_manager_env_script(&config) {
            warn!(
                target = "substrate::shell",
                error = %err,
                "failed to write manager_env.sh"
            );
        }
        if let Some(parent) = config.bash_preexec_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match write_bash_preexec_script(&config.bash_preexec_path) {
            Ok(()) => config.preexec_available = true,
            Err(err) => {
                config.preexec_available = false;
                warn!(
                    target = "substrate::shell",
                    error = %err,
                    "failed to write BASH preexec script"
                );
            }
        }
        env::set_var("SUBSTRATE_MANAGER_ENV", &config.manager_env_path);
    }

    initialize_world(&config);

    // Deploy shims if needed (non-blocking, continues on error)
    // Skip if either the CLI flag is set or the environment variable is set
    let skip_shims = config.skip_shims;
    match ShimDeployer::with_skip(skip_shims)?.ensure_deployed() {
        Ok(DeploymentStatus::Deployed) => {
            // Shims were deployed, no additional action needed
        }
        Ok(DeploymentStatus::Failed(msg)) => {
            eprintln!("Warning: Shim deployment failed: {msg}");
            // Continue without shims
        }
        Ok(_) => {
            // Current, Skipped - no action needed
        }
        Err(e) => {
            eprintln!("Warning: Error checking shim deployment: {e}");
            // Continue without shims
        }
    }

    // Set up environment for child processes
    env::set_var("SHIM_SESSION_ID", &config.session_id);
    env::set_var("SHIM_ORIGINAL_PATH", &config.original_path);
    env::set_var("SHIM_TRACE_LOG", &config.trace_log_file);

    // Clear SHIM_ACTIVE to allow shims to work properly
    // The substrate shell itself should not be considered "active" shimming
    env::remove_var("SHIM_ACTIVE");

    match &config.mode {
        ShellMode::Interactive { use_pty: _ } => {
            if config.async_repl {
                async_repl::run_async_repl(&config)
            } else {
                // PTY mode is now handled within run_interactive_shell on a per-command basis
                run_interactive_shell(&config)
            }
        }
        ShellMode::Wrap(cmd) => run_wrap_mode(&config, cmd),
        ShellMode::Script(path) => run_script_mode(&config, path),
        ShellMode::Pipe => run_pipe_mode(&config),
    }
}

// Helper function to setup signal handlers
pub(crate) fn setup_signal_handlers(running_child_pid: Arc<AtomicI32>) -> Result<()> {
    // Set up Ctrl-C handler with PTY awareness
    {
        let running = running_child_pid.clone();
        ctrlc::set_handler(move || {
            // Check if PTY is active - if so, let PTY handle the signal
            if PTY_ACTIVE.load(Ordering::Relaxed) {
                // No-op: PTY is handling signals
                return;
            }

            let pid = running.load(Ordering::SeqCst);
            if pid > 0 {
                // Forward signal to entire process group
                #[cfg(unix)]
                {
                    use nix::sys::signal::{killpg, Signal};
                    use nix::unistd::{getpgid, Pid};
                    if let Ok(pgid) = getpgid(Some(Pid::from_raw(pid))) {
                        let _ = killpg(pgid, Signal::SIGINT);
                    }
                }
            }
            // If no child is running, the signal is dropped and REPL continues
        })?;
    }

    // Set up additional signal forwarding for non-PTY path (SIGTERM, SIGQUIT, SIGHUP)
    #[cfg(unix)]
    {
        use signal_hook::{
            consts::{SIGHUP, SIGQUIT, SIGTERM},
            iterator::Signals,
        };

        let running = running_child_pid.clone();
        thread::spawn(move || {
            let mut signals = match Signals::new([SIGTERM, SIGQUIT, SIGHUP]) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Failed to register additional signal handlers: {e}");
                    return;
                }
            };

            for sig in signals.forever() {
                // Only forward if PTY is not active (PTY gets kernel-side job control)
                if !PTY_ACTIVE.load(Ordering::Relaxed) {
                    let pid = running.load(Ordering::SeqCst);
                    if pid > 0 {
                        use nix::sys::signal::{killpg, Signal};
                        use nix::unistd::{getpgid, Pid};

                        let signal = match sig {
                            SIGTERM => Signal::SIGTERM,
                            SIGQUIT => Signal::SIGQUIT,
                            SIGHUP => Signal::SIGHUP,
                            _ => continue,
                        };

                        if let Ok(pgid) = getpgid(Some(Pid::from_raw(pid))) {
                            let _ = killpg(pgid, signal);
                        }
                    }
                }

                if sig == SIGTERM || sig == SIGQUIT {
                    std::process::exit(128 + sig);
                }
            }
        });
    }

    Ok(())
}
