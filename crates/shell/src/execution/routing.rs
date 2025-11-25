use super::cli::*;
use super::invocation::{
    run_interactive_shell, run_pipe_mode, run_script_mode, run_wrap_mode, ShellConfig, ShellMode,
};
mod builtin;
mod dispatch;
mod path_env;
#[cfg(test)]
mod test_utils;
use super::shim_deploy::{DeploymentStatus, ShimDeployer};
use super::{configure_manager_init, log_manager_init_event, write_manager_env_script};
use crate::builtins as commands;
use crate::repl::async_repl;
use crate::scripts::write_bash_preexec_script;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, IsTerminal};
// (avoid unused: import Read/Write locally where needed)
#[cfg(test)]
use agent_api_types::ExecuteStreamFrame;
#[cfg(test)]
use base64::engine::general_purpose::STANDARD as BASE64;
#[cfg(test)]
use base64::Engine;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use substrate_broker::{set_global_broker, BrokerHandle};
#[cfg(test)]
use substrate_common::WorldRootMode;
use substrate_common::{
    agent_events::{AgentEvent, AgentEventKind},
    log_schema,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use substrate_trace::TransportMeta;
use substrate_trace::{append_to_trace, init_trace, set_global_trace_context, TraceContext};
use tracing::{info, warn};
use uuid::Uuid;

// Reedline imports
#[cfg_attr(target_os = "windows", allow(unused_imports))]
use std::thread;
// use nu_ansi_term::{Color, Style}; // Unused for now
#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::pw;
#[cfg(test)]
pub(crate) use builtin::handle_builtin;
#[cfg(target_os = "linux")]
pub(crate) use dispatch::init_linux_world;
pub(crate) use dispatch::{
    build_agent_client_and_request, execute_command, parse_demo_burst_command,
    stream_non_pty_via_agent,
};
#[cfg(test)]
pub(crate) use dispatch::{
    consume_agent_stream_buffer, container_wants_pty, git_wants_pty, has_top_level_shell_meta,
    is_force_pty_command, is_interactive_shell, is_pty_disabled, looks_like_repl, needs_pty,
    peel_wrappers, sudo_wants_pty, wants_debugger_pty,
};
#[cfg(all(test, target_os = "linux"))]
pub(crate) use dispatch::{init_linux_world_with_probe, LinuxWorldInit};
#[cfg(target_os = "linux")]
use nix::sys::termios::{
    self, ControlFlags, InputFlags, LocalFlags, OutputFlags, SetArg, SpecialCharacterIndices,
    Termios,
};
pub(crate) use path_env::world_deps_manifest_base_path;
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn world_transport_to_meta(transport: &pw::WorldTransport) -> TransportMeta {
    match transport {
        pw::WorldTransport::Unix(path) => TransportMeta {
            mode: "unix".to_string(),
            endpoint: Some(path.display().to_string()),
        },
        pw::WorldTransport::Tcp { host, port } => TransportMeta {
            mode: "tcp".to_string(),
            endpoint: Some(format!("{}:{}", host, port)),
        },
        pw::WorldTransport::Vsock { port } => TransportMeta {
            mode: "vsock".to_string(),
            endpoint: Some(format!("{}", port)),
        },
        #[cfg(target_os = "windows")]
        pw::WorldTransport::NamedPipe(path) => TransportMeta {
            mode: "named_pipe".to_string(),
            endpoint: Some(path.display().to_string()),
        },
    }
}
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;
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

const SHELL_AGENT_ID: &str = "shell";

pub(crate) fn is_shell_stream_event(event: &AgentEvent) -> bool {
    event.agent_id == SHELL_AGENT_ID && matches!(event.kind, AgentEventKind::PtyData)
}

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

    // Default-on world initialization (Linux only)
    #[cfg(target_os = "windows")]
    {
        let world_disabled = env::var("SUBSTRATE_WORLD")
            .map(|v| v == "disabled")
            .unwrap_or(false)
            || config.no_world;
        if !world_disabled {
            match pw::detect() {
                Ok(ctx) => {
                    if let Err(e) = (ctx.ensure_ready)() {
                        eprintln!("substrate: windows world ensure_ready failed: {:#}", e);
                    } else {
                        std::env::set_var("SUBSTRATE_WORLD", "enabled");
                        if let Ok(handle) = ctx.backend.ensure_session(&pw::windows::world_spec()) {
                            std::env::set_var("SUBSTRATE_WORLD_ID", handle.id);
                        }
                    }
                    pw::store_context_globally(ctx);
                }
                Err(e) => {
                    eprintln!("substrate: windows world detection failed: {}", e);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Default ON (parity with Linux); allow disabling via env/CLI
        let world_disabled = env::var("SUBSTRATE_WORLD")
            .map(|v| v == "disabled")
            .unwrap_or(false)
            || config.no_world;
        if !world_disabled {
            // Auto-detect mac world context and ensure readiness
            match pw::detect() {
                Ok(ctx) => {
                    if let Err(e) = (ctx.ensure_ready)() {
                        // Degrade silently if ensure_ready fails
                        eprintln!("substrate: mac world ensure_ready failed: {}", e);
                    } else {
                        // Set parity with Linux: world enabled + ID only
                        std::env::set_var("SUBSTRATE_WORLD", "enabled");

                        // Attempt to retrieve world id
                        use world_api::{ResourceLimits, WorldSpec};
                        let spec = WorldSpec {
                            reuse_session: true,
                            isolate_network: true,
                            limits: ResourceLimits::default(),
                            enable_preload: false,
                            allowed_domains: substrate_broker::allowed_domains(),
                            project_dir: config.world_root.effective_root(),
                            always_isolate: false,
                        };
                        if let Ok(handle) = ctx.backend.ensure_session(&spec) {
                            std::env::set_var("SUBSTRATE_WORLD_ID", handle.id);
                        }
                    }
                    pw::store_context_globally(ctx);
                }
                Err(e) => {
                    // Degrade silently on mac as well
                    eprintln!("substrate: mac world detection failed: {}", e);
                }
            }
        }
    }

    // Default-on world initialization (Linux only)
    #[cfg(target_os = "linux")]
    {
        let world_disabled = env::var("SUBSTRATE_WORLD")
            .map(|v| v == "disabled")
            .unwrap_or(false)
            || config.no_world;

        let _ = init_linux_world(world_disabled);
    }

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

pub(crate) fn handle_trace_command(span_id: &str) -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    // Get trace file location
    let trace_file = env::var("SHIM_TRACE_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("Cannot determine home directory")
                .join(".substrate/trace.jsonl")
        });

    if !trace_file.exists() {
        eprintln!("Trace file not found: {}", trace_file.display());
        eprintln!("Make sure tracing is enabled with SUBSTRATE_WORLD=enabled");
        std::process::exit(1);
    }

    // Read trace file and find the span
    let file = File::open(&trace_file)?;
    let reader = BufReader::new(file);
    let mut found: Option<serde_json::Value> = None;

    for line in reader.lines() {
        let line = line?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(id) = json.get("span_id").and_then(|v| v.as_str()) {
                if id == span_id {
                    // Prefer command_complete if multiple entries exist
                    let is_complete =
                        json.get("event_type").and_then(|v| v.as_str()) == Some("command_complete");
                    match &found {
                        None => found = Some(json),
                        Some(current) => {
                            let current_is_complete =
                                current.get("event_type").and_then(|v| v.as_str())
                                    == Some("command_complete");
                            if is_complete && !current_is_complete {
                                found = Some(json);
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(json) = found {
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        eprintln!("Span ID not found: {}", span_id);
        std::process::exit(1);
    }

    Ok(())
}

/// Handle replay command - replay a traced command by span ID
pub(crate) fn handle_replay_command(span_id: &str) -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    // Get trace file location
    let trace_file = env::var("SHIM_TRACE_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("Cannot determine home directory")
                .join(".substrate/trace.jsonl")
        });

    if !trace_file.exists() {
        eprintln!("Trace file not found: {}", trace_file.display());
        eprintln!("Make sure tracing is enabled with SUBSTRATE_WORLD=enabled");
        std::process::exit(1);
    }

    // Load the trace for the span
    let file = File::open(&trace_file)?;
    let reader = BufReader::new(file);
    let mut trace_entry = None;

    for line in reader.lines() {
        let line = line?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(id) = json.get("span_id").and_then(|v| v.as_str()) {
                if id == span_id {
                    trace_entry = Some(json);
                    break;
                }
            }
        }
    }

    let entry = trace_entry.ok_or_else(|| anyhow::anyhow!("Span ID not found: {}", span_id))?;

    // Extract command information from the trace
    let command = entry
        .get("cmd")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("No command found in trace"))?;

    let cwd = entry
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap());

    let session_id = entry
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    // Verbose header
    if env::var("SUBSTRATE_REPLAY_VERBOSE").unwrap_or_default() == "1"
        || env::args().any(|a| a == "--replay-verbose")
    {
        eprintln!("[replay] span_id: {}", span_id);
        eprintln!("[replay] command: {}", command);
        eprintln!("[replay] cwd: {}", cwd.display());
        eprintln!("[replay] mode: bash -lc");
    }

    // Parse command into command and args
    let parts: Vec<&str> = command.split_whitespace().collect();
    let (cmd, args) = if !parts.is_empty() {
        (
            parts[0].to_string(),
            parts[1..].iter().map(|s| s.to_string()).collect(),
        )
    } else {
        (command.to_string(), Vec::new())
    };

    // Create execution state
    let state = substrate_replay::replay::ExecutionState {
        raw_cmd: command.to_string(),
        command: cmd,
        args,
        cwd,
        env: HashMap::new(), // Could extract from trace if captured
        stdin: None,
        session_id,
        span_id: span_id.to_string(),
    };

    // Execute with replay (choose world isolation; default enabled, allow opt-out)
    let runtime = tokio::runtime::Runtime::new()?;
    // Respect --no-world flag, then environment variable override, else default enabled
    let no_world_flag = env::args().any(|a| a == "--no-world");
    let use_world = if no_world_flag {
        false
    } else {
        match env::var("SUBSTRATE_REPLAY_USE_WORLD") {
            Ok(val) => val != "0" && val != "disabled",
            Err(_) => true,
        }
    };
    // Best-effort capability warnings when world isolation requested but not available
    if cfg!(target_os = "linux") && use_world {
        // cgroup v2
        if !PathBuf::from("/sys/fs/cgroup/cgroup.controllers").exists() {
            eprintln!("[replay] warn: cgroup v2 not mounted; world cgroups will not activate");
        }
        // overlayfs
        let overlay_ok = std::fs::read_to_string("/proc/filesystems")
            .ok()
            .map(|s| s.contains("overlay"))
            .unwrap_or(false);
        if !overlay_ok {
            eprintln!("[replay] warn: overlayfs not present; fs_diff will be unavailable");
        }
        // nftables
        let nft_ok = std::process::Command::new("nft")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()
            .map(|s| s.success())
            .unwrap_or(false);
        if !nft_ok {
            eprintln!("[replay] warn: nft not available; netfilter scoping/logging disabled");
        }
        // dmesg restrict
        if let Ok(out) = std::process::Command::new("sh")
            .arg("-lc")
            .arg("sysctl -n kernel.dmesg_restrict 2>/dev/null || echo n/a")
            .output()
        {
            if let Ok(s) = String::from_utf8(out.stdout) {
                if s.trim() == "1" {
                    eprintln!(
                        "[replay] warn: kernel.dmesg_restrict=1; LOG lines may not be visible"
                    );
                }
            }
        }
    }

    let result = if use_world {
        runtime.block_on(async { substrate_replay::replay::execute_in_world(&state, 60).await })?
    } else {
        runtime.block_on(async { substrate_replay::replay::execute_direct(&state, 60).await })?
    };

    // Display results
    println!("Exit code: {}", result.exit_code);
    if !result.stdout.is_empty() {
        println!("\nStdout:");
        println!("{}", String::from_utf8_lossy(&result.stdout));
    }
    if !result.stderr.is_empty() {
        println!("\nStderr:");
        println!("{}", String::from_utf8_lossy(&result.stderr));
    }

    if let Some(fs_diff) = result.fs_diff {
        if !fs_diff.is_empty() {
            println!("\nFilesystem changes:");
            for write in &fs_diff.writes {
                println!("  + {}", write.display());
            }
            for modify in &fs_diff.mods {
                println!("  ~ {}", modify.display());
            }
            for delete in &fs_diff.deletes {
                println!("  - {}", delete.display());
            }
        }
    }

    std::process::exit(result.exit_code);
}

#[cfg(test)]
mod fs_diff_parse_tests {
    #[test]
    fn test_parse_fs_diff_from_agent_json() {
        let sample = r#"{
            "exit":0,
            "span_id":"spn_x",
            "stdout_b64":"",
            "stderr_b64":"",
            "scopes_used":["tcp:example.com:443"],
            "fs_diff":{
                "writes":["/tmp/t/a.txt"],
                "mods":[],
                "deletes":[],
                "truncated":false
            }
        }"#;
        let v: serde_json::Value = serde_json::from_str(sample).unwrap();
        let fd_val = v.get("fs_diff").cloned().unwrap();
        let diff: substrate_common::FsDiff = serde_json::from_value(fd_val).unwrap();
        assert_eq!(diff.writes.len(), 1);
        assert_eq!(diff.writes[0], std::path::PathBuf::from("/tmp/t/a.txt"));
        assert!(diff.mods.is_empty());
        assert!(diff.deletes.is_empty());
        assert!(!diff.truncated);
    }
}

#[cfg(all(test, target_os = "linux"))]
mod linux_world_tests {
    use super::*;
    use anyhow::anyhow;
    use serial_test::serial;

    fn clear_env() {
        env::remove_var("SUBSTRATE_WORLD");
        env::remove_var("SUBSTRATE_WORLD_ID");
        env::remove_var("SUBSTRATE_TEST_LOCAL_WORLD_ID");
    }

    #[test]
    #[serial]
    fn agent_probe_enables_world() {
        clear_env();
        let outcome = init_linux_world_with_probe(false, || Ok(()));
        assert_eq!(outcome, LinuxWorldInit::Agent);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert!(env::var("SUBSTRATE_WORLD_ID").is_err());
    }

    #[test]
    #[serial]
    fn fallback_uses_local_backend_stub() {
        clear_env();
        env::set_var("SUBSTRATE_TEST_LOCAL_WORLD_ID", "wld_test_stub");
        let outcome = init_linux_world_with_probe(false, || Err(anyhow!("no agent")));
        assert_eq!(outcome, LinuxWorldInit::LocalBackend);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ID").unwrap(), "wld_test_stub");
    }

    #[test]
    #[serial]
    fn disabled_skips_initialization() {
        clear_env();
        let outcome = init_linux_world_with_probe(true, || Ok(()));
        assert_eq!(outcome, LinuxWorldInit::Disabled);
        assert!(env::var("SUBSTRATE_WORLD").is_err());
        assert!(env::var("SUBSTRATE_WORLD_ID").is_err());
    }
}

#[cfg(test)]
mod streaming_tests {
    use super::*;
    use crate::execution::agent_events::{self, clear_agent_event_sender, init_event_channel};
    use substrate_common::agent_events::AgentEventKind;
    use tokio::runtime::Runtime;

    #[test]
    fn parse_demo_burst_command_defaults() {
        assert_eq!(parse_demo_burst_command(":demo-burst"), Some((4, 400, 0)));
        assert_eq!(
            parse_demo_burst_command(":demo-burst 3 10 5"),
            Some((3, 10, 5))
        );
        assert!(parse_demo_burst_command(":other").is_none());
    }

    #[test]
    fn consume_agent_stream_buffer_emits_agent_events() {
        let _guard = agent_events::acquire_event_test_guard();
        let rt = Runtime::new().expect("runtime");
        rt.block_on(async {
            let mut rx = init_event_channel();

            let frames = [
                ExecuteStreamFrame::Stdout {
                    chunk_b64: BASE64.encode("hello"),
                },
                ExecuteStreamFrame::Stderr {
                    chunk_b64: BASE64.encode("oops"),
                },
                ExecuteStreamFrame::Exit {
                    exit: 0,
                    span_id: "spn_test".into(),
                    scopes_used: vec!["scope:a".into()],
                    fs_diff: None,
                },
            ];

            let mut buffer = Vec::new();
            for frame in frames {
                let mut line = serde_json::to_vec(&frame).expect("serialize frame");
                line.push(b'\n');
                buffer.extend(line);
            }

            let mut exit_code = None;
            let mut scopes_used = Vec::new();
            let mut fs_diff = None;

            consume_agent_stream_buffer(
                "tester",
                &mut buffer,
                &mut exit_code,
                &mut scopes_used,
                &mut fs_diff,
            )
            .expect("consume stream");

            let stdout_event = rx.recv().await.expect("stdout event");
            assert_eq!(stdout_event.kind, AgentEventKind::PtyData);
            assert_eq!(stdout_event.data["chunk"], "hello");
            assert_eq!(stdout_event.data["stream"], "stdout");

            let stderr_event = rx.recv().await.expect("stderr event");
            assert_eq!(stderr_event.kind, AgentEventKind::PtyData);
            assert_eq!(stderr_event.data["chunk"], "oops");
            assert_eq!(stderr_event.data["stream"], "stderr");

            assert_eq!(exit_code, Some(0));
            assert_eq!(scopes_used, vec!["scope:a".to_string()]);
            assert!(fs_diff.is_none());
        });
        clear_agent_event_sender();
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct ReplMetrics {
    input_events: u64,
    agent_events: u64,
    commands_executed: u64,
}

pub(crate) struct ReplSessionTelemetry {
    config: Arc<ShellConfig>,
    mode: &'static str,
    metrics: ReplMetrics,
}

impl ReplSessionTelemetry {
    pub(crate) fn new(config: Arc<ShellConfig>, mode: &'static str) -> Self {
        let telemetry = Self {
            config,
            mode,
            metrics: ReplMetrics::default(),
        };

        if let Err(err) = log_repl_event(&telemetry.config, mode, "start", None) {
            warn!(target = "substrate::shell", error = %err, "failed to append REPL start event");
        }

        telemetry
    }

    pub(crate) fn record_input_event(&mut self) {
        self.metrics.input_events = self.metrics.input_events.saturating_add(1);
    }

    pub(crate) fn record_agent_event(&mut self) {
        self.metrics.agent_events = self.metrics.agent_events.saturating_add(1);
    }

    pub(crate) fn record_command(&mut self) {
        self.metrics.commands_executed = self.metrics.commands_executed.saturating_add(1);
    }
}

impl Drop for ReplSessionTelemetry {
    fn drop(&mut self) {
        if let Err(err) = log_repl_event(&self.config, self.mode, "stop", Some(&self.metrics)) {
            warn!(target = "substrate::shell", error = %err, "failed to append REPL stop event");
        }
    }
}

fn log_repl_event(
    config: &ShellConfig,
    mode: &str,
    stage: &str,
    metrics: Option<&ReplMetrics>,
) -> Result<()> {
    let mut entry = json!({
        log_schema::TIMESTAMP: Utc::now().to_rfc3339(),
        log_schema::EVENT_TYPE: "repl_status",
        log_schema::SESSION_ID: config.session_id,
        log_schema::COMPONENT: "shell",
        "stage": stage,
        "repl_mode": mode,
        "ci_mode": config.ci_mode,
        "async_enabled": config.async_repl,
        "no_world": config.no_world,
        "shell": config.shell_path,
    });

    if let ShellMode::Interactive { use_pty } = &config.mode {
        entry["interactive_use_pty"] = json!(*use_pty);
    }

    let (input_events, agent_events, commands_executed) = metrics
        .map(|m| (m.input_events, m.agent_events, m.commands_executed))
        .unwrap_or((0, 0, 0));

    if metrics.is_some() {
        entry["metrics"] = json!({
            "input_events": input_events,
            "agent_events": agent_events,
            "commands_executed": commands_executed,
        });
    }

    let _ = init_trace(None);
    append_to_trace(&entry)?;

    info!(
        target = "substrate::shell",
        repl_mode = mode,
        stage,
        input_events,
        agent_events,
        commands_executed,
        "repl_status"
    );

    Ok(())
}

pub(crate) fn log_command_event(
    config: &ShellConfig,
    event_type: &str,
    command: &str,
    cmd_id: &str,
    extra: Option<serde_json::Value>,
) -> Result<()> {
    let stdin_is_tty = io::stdin().is_terminal();
    let stdout_is_tty = io::stdout().is_terminal();
    let stderr_is_tty = io::stderr().is_terminal();

    let mut log_entry = json!({
        log_schema::TIMESTAMP: Utc::now().to_rfc3339(),
        log_schema::EVENT_TYPE: event_type,
        log_schema::SESSION_ID: config.session_id,
        log_schema::COMMAND_ID: cmd_id,
        "command": command,
        log_schema::COMPONENT: "shell",
        "mode": match &config.mode {
            ShellMode::Interactive { .. } => "interactive",
            ShellMode::Wrap(_) => "wrap",
            ShellMode::Script(_) => "script",
            ShellMode::Pipe => "pipe",
        },
        "cwd": env::current_dir()?.display().to_string(),
        "host": gethostname::gethostname().to_string_lossy().to_string(),
        "shell": config.shell_path,
        "isatty_stdin": stdin_is_tty,
        "isatty_stdout": stdout_is_tty,
        "isatty_stderr": stderr_is_tty,
        "pty": matches!(&config.mode, ShellMode::Interactive { use_pty: true }),
    });

    if matches!(&config.mode, ShellMode::Interactive { .. }) {
        log_entry["repl_mode"] = json!(if config.async_repl { "async" } else { "sync" });
    }

    // Add build version if available
    if let Ok(build) = env::var("SHIM_BUILD") {
        log_entry["build"] = json!(build);
    }

    // Add ppid on Unix
    #[cfg(unix)]
    {
        log_entry["ppid"] = json!(nix::unistd::getppid().as_raw());
    }

    // Merge extra data
    if let Some(extra_data) = extra {
        if let Some(obj) = log_entry.as_object_mut() {
            if let Some(extra_obj) = extra_data.as_object() {
                for (k, v) in extra_obj {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    }

    // Route all event writes through unified trace; writer handles rotation
    // Ensure trace is initialized first when world is enabled
    let _ = init_trace(None);
    append_to_trace(&log_entry)?;

    Ok(())
}

#[cfg(test)]
mod manager_init_wiring_tests {
    use super::*;
    use crate::execution::routing::path_env::enforce_caged_destination;
    use serial_test::serial;
    use std::{collections::HashMap, env, fs, path::PathBuf};
    use tempfile::{tempdir, TempDir};

    fn test_shell_config(temp: &TempDir) -> ShellConfig {
        let trace_log_file = temp.path().join("trace.jsonl");
        env::set_var("SHIM_TRACE_LOG", &trace_log_file);
        let _ = set_global_trace_context(TraceContext::default());
        let _ = init_trace(Some(trace_log_file.clone()));

        ShellConfig {
            mode: ShellMode::Interactive { use_pty: false },
            session_id: "test-session".to_string(),
            trace_log_file,
            original_path: env::var("PATH").unwrap_or_default(),
            shim_dir: temp.path().join("shims"),
            shell_path: if cfg!(windows) {
                "cmd.exe".to_string()
            } else {
                "/bin/sh".to_string()
            },
            ci_mode: false,
            no_exit_on_error: false,
            skip_shims: false,
            no_world: false,
            world_root: crate::execution::settings::WorldRootSettings {
                mode: WorldRootMode::Project,
                path: temp.path().to_path_buf(),
                caged: true,
            },
            async_repl: false,
            env_vars: HashMap::new(),
            manager_init_path: temp.path().join("manager_init.sh"),
            manager_env_path: temp.path().join("manager_env.sh"),
            shimmed_path: Some(temp.path().join("shims").display().to_string()),
            host_bash_env: None,
            bash_preexec_path: temp.path().join(".substrate_preexec"),
            preexec_available: true,
        }
    }

    fn set_env(key: &str, value: &str) -> Option<String> {
        let previous = env::var(key).ok();
        env::set_var(key, value);
        previous
    }

    fn restore_env(key: &str, previous: Option<String>) {
        if let Some(value) = previous {
            env::set_var(key, value);
        } else {
            env::remove_var(key);
        }
    }

    struct DirGuard {
        original: PathBuf,
    }

    impl DirGuard {
        fn new() -> Self {
            let original = env::current_dir().expect("capture cwd");
            Self { original }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original);
        }
    }

    #[test]
    #[serial]
    fn export_builtin_sets_plain_pairs() {
        let temp = tempdir().unwrap();
        let config = test_shell_config(&temp);

        let prev_token = set_env("API_TOKEN", "old");
        let prev_plain = set_env("PLAIN_VALUE", "unset");

        let status = handle_builtin(
            &config,
            "export API_TOKEN=new-secret PLAIN_VALUE=fresh",
            "parent",
        )
        .expect("builtin export should succeed");
        assert!(status.is_some());
        assert_eq!(env::var("API_TOKEN").unwrap(), "new-secret");
        assert_eq!(env::var("PLAIN_VALUE").unwrap(), "fresh");

        restore_env("PLAIN_VALUE", prev_plain);
        restore_env("API_TOKEN", prev_token);
    }

    #[test]
    #[serial]
    fn export_builtin_defers_when_value_needs_shell() {
        let temp = tempdir().unwrap();
        let config = test_shell_config(&temp);

        env::remove_var("EXPORT_COMPLEX");
        let status =
            handle_builtin(&config, "export EXPORT_COMPLEX=\"$SHOULD_SKIP\"", "parent").unwrap();
        assert!(status.is_none());
        assert!(env::var("EXPORT_COMPLEX").is_err());
    }

    #[test]
    #[serial]
    fn unset_builtin_clears_variables() {
        let temp = tempdir().unwrap();
        let config = test_shell_config(&temp);

        let prev = set_env("UNSET_ME", "present");
        let status = handle_builtin(&config, "unset UNSET_ME", "parent").unwrap();
        assert!(status.is_some());
        assert!(env::var("UNSET_ME").is_err());

        restore_env("UNSET_ME", prev);
    }

    #[test]
    #[serial]
    fn world_flag_overrides_disabled_config_and_env() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        fs::create_dir_all(substrate_home.join("shims")).unwrap();
        fs::write(
            substrate_home.join("config.toml"),
            "[install]\nworld_enabled = false\n",
        )
        .unwrap();

        let prev_home = set_env("HOME", &home.display().to_string());
        let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
        let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
        let prev_root_mode = set_env("SUBSTRATE_WORLD_ROOT_MODE", "project");
        let prev_root_path = set_env("SUBSTRATE_WORLD_ROOT_PATH", "/env/root");
        let prev_caged = set_env("SUBSTRATE_CAGED", "1");
        let prev_anchor_mode = env::var("SUBSTRATE_ANCHOR_MODE").ok();
        let prev_anchor_path = env::var("SUBSTRATE_ANCHOR_PATH").ok();
        let prev_manager_env = env::var("SUBSTRATE_MANAGER_ENV").ok();
        let prev_manager_init = env::var("SUBSTRATE_MANAGER_INIT").ok();
        let _dir_guard = DirGuard::new();
        fs::create_dir_all(&home).unwrap();
        env::set_current_dir(&home).unwrap();

        let cli = Cli::parse_from(["substrate", "--world"]);
        let config = ShellConfig::from_cli(cli).expect("parse config with world override");
        assert!(!config.no_world);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "1");

        restore_env("SUBSTRATE_WORLD_ROOT_MODE", prev_root_mode);
        restore_env("SUBSTRATE_WORLD_ROOT_PATH", prev_root_path);
        restore_env("SUBSTRATE_CAGED", prev_caged);
        restore_env("SUBSTRATE_ANCHOR_MODE", prev_anchor_mode);
        restore_env("SUBSTRATE_ANCHOR_PATH", prev_anchor_path);
        restore_env("SUBSTRATE_MANAGER_ENV", prev_manager_env);
        restore_env("SUBSTRATE_MANAGER_INIT", prev_manager_init);
        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }

    #[test]
    #[serial]
    fn world_flag_honors_directory_world_root_settings() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        let workdir = temp.path().join("workspace");
        let custom_root = workdir.join("nested-root");
        fs::create_dir_all(substrate_home.join("shims")).unwrap();
        fs::create_dir_all(workdir.join(".substrate")).unwrap();
        fs::create_dir_all(&custom_root).unwrap();
        fs::write(
            substrate_home.join("config.toml"),
            "[install]\nworld_enabled = false\n[world]\nroot_mode = \"project\"\nroot_path = \"\"\ncaged = true\n",
        )
        .unwrap();
        let settings_body = format!(
            "[world]\nroot_mode = \"custom\"\nroot_path = \"{}\"\ncaged = false\n",
            custom_root.display().to_string().replace('\\', "\\\\")
        );
        fs::write(workdir.join(".substrate/settings.toml"), settings_body).unwrap();

        let prev_home = set_env("HOME", &home.display().to_string());
        let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
        let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
        let prev_root_mode = set_env("SUBSTRATE_WORLD_ROOT_MODE", "project");
        let prev_root_path = set_env("SUBSTRATE_WORLD_ROOT_PATH", "/env/root");
        let prev_caged = set_env("SUBSTRATE_CAGED", "1");
        let prev_anchor_mode = env::var("SUBSTRATE_ANCHOR_MODE").ok();
        let prev_anchor_path = env::var("SUBSTRATE_ANCHOR_PATH").ok();
        let _dir_guard = DirGuard::new();
        env::set_current_dir(&workdir).unwrap();

        let cli = Cli::parse_from(["substrate", "--world"]);
        let config = ShellConfig::from_cli(cli).expect("parse config with directory world root");
        assert!(!config.no_world);
        assert_eq!(config.world_root.mode, WorldRootMode::Custom);
        assert_eq!(config.world_root.path, custom_root);
        assert!(!config.world_root.caged);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "1");
        assert_eq!(env::var("SUBSTRATE_WORLD_ROOT_MODE").unwrap(), "custom");
        assert_eq!(
            env::var("SUBSTRATE_WORLD_ROOT_PATH").unwrap(),
            custom_root.display().to_string()
        );
        assert_eq!(env::var("SUBSTRATE_CAGED").unwrap(), "0");

        restore_env("SUBSTRATE_WORLD_ROOT_MODE", prev_root_mode);
        restore_env("SUBSTRATE_WORLD_ROOT_PATH", prev_root_path);
        restore_env("SUBSTRATE_CAGED", prev_caged);
        restore_env("SUBSTRATE_ANCHOR_MODE", prev_anchor_mode);
        restore_env("SUBSTRATE_ANCHOR_PATH", prev_anchor_path);
        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }

    #[test]
    #[serial]
    fn anchor_flags_override_configs_and_export_legacy_env() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        let workdir = temp.path().join("workspace");
        let cli_anchor = workdir.join("cli-anchor");
        let dir_anchor = workdir.join("dir-anchor");
        fs::create_dir_all(substrate_home.join("shims")).unwrap();
        fs::create_dir_all(workdir.join(".substrate")).unwrap();
        fs::create_dir_all(&cli_anchor).unwrap();
        fs::create_dir_all(&dir_anchor).unwrap();
        fs::write(
            substrate_home.join("config.toml"),
            "[world]\nanchor_mode = \"project\"\nanchor_path = \"/config/root\"\ncaged = false\n",
        )
        .unwrap();
        let settings_body = format!(
            "[world]\nanchor_mode = \"custom\"\nanchor_path = \"{}\"\ncaged = false\n",
            dir_anchor.display().to_string().replace('\\', "\\\\")
        );
        fs::write(workdir.join(".substrate/settings.toml"), settings_body).unwrap();

        let prev_home = set_env("HOME", &home.display().to_string());
        let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
        let prev_anchor_mode = set_env("SUBSTRATE_ANCHOR_MODE", "follow-cwd");
        let prev_anchor_path = set_env("SUBSTRATE_ANCHOR_PATH", "/env/anchor");
        let prev_root_mode = set_env("SUBSTRATE_WORLD_ROOT_MODE", "follow-cwd");
        let prev_root_path = set_env("SUBSTRATE_WORLD_ROOT_PATH", "/env/root");
        let prev_caged = set_env("SUBSTRATE_CAGED", "0");
        let _dir_guard = DirGuard::new();
        env::set_current_dir(&workdir).unwrap();

        let cli_anchor_path = cli_anchor.display().to_string();
        let cli = Cli::parse_from([
            "substrate",
            "--anchor-mode",
            "custom",
            "--anchor-path",
            &cli_anchor_path,
            "--caged",
        ]);
        let config = ShellConfig::from_cli(cli).expect("parse config with anchor flags");

        assert_eq!(config.world_root.mode, WorldRootMode::Custom);
        assert_eq!(config.world_root.path, cli_anchor);
        assert!(config.world_root.caged);
        assert_eq!(env::var("SUBSTRATE_ANCHOR_MODE").unwrap(), "custom");
        assert_eq!(env::var("SUBSTRATE_WORLD_ROOT_MODE").unwrap(), "custom");
        assert_eq!(
            env::var("SUBSTRATE_ANCHOR_PATH").unwrap(),
            cli_anchor.display().to_string()
        );
        assert_eq!(
            env::var("SUBSTRATE_WORLD_ROOT_PATH").unwrap(),
            cli_anchor.display().to_string()
        );
        assert_eq!(env::var("SUBSTRATE_CAGED").unwrap(), "1");

        restore_env("SUBSTRATE_CAGED", prev_caged);
        restore_env("SUBSTRATE_WORLD_ROOT_PATH", prev_root_path);
        restore_env("SUBSTRATE_WORLD_ROOT_MODE", prev_root_mode);
        restore_env("SUBSTRATE_ANCHOR_PATH", prev_anchor_path);
        restore_env("SUBSTRATE_ANCHOR_MODE", prev_anchor_mode);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }

    #[test]
    #[serial]
    fn no_world_flag_disables_world_and_sets_root_exports() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        let workdir = temp.path().join("workspace");
        fs::create_dir_all(substrate_home.join("shims")).unwrap();
        fs::create_dir_all(&workdir).unwrap();
        fs::write(
            substrate_home.join("config.toml"),
            "[install]\nworld_enabled = true\n[world]\nroot_mode = \"project\"\nroot_path = \"\"\ncaged = true\n",
        )
        .unwrap();

        let prev_home = set_env("HOME", &home.display().to_string());
        let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
        let prev_world = set_env("SUBSTRATE_WORLD", "enabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "1");
        let prev_root_mode = set_env("SUBSTRATE_WORLD_ROOT_MODE", "project");
        let prev_root_path = set_env("SUBSTRATE_WORLD_ROOT_PATH", "/env/root");
        let prev_caged = set_env("SUBSTRATE_CAGED", "1");
        let prev_anchor_mode = env::var("SUBSTRATE_ANCHOR_MODE").ok();
        let prev_anchor_path = env::var("SUBSTRATE_ANCHOR_PATH").ok();
        let _dir_guard = DirGuard::new();
        env::set_current_dir(&workdir).unwrap();

        let cli = Cli::parse_from([
            "substrate",
            "--no-world",
            "--world-root-mode",
            "follow-cwd",
            "--uncaged",
        ]);
        let config = ShellConfig::from_cli(cli).expect("parse config with no-world flag");
        assert!(config.no_world);
        assert_eq!(config.world_root.mode, WorldRootMode::FollowCwd);
        let expected_workdir = fs::canonicalize(&workdir).unwrap_or_else(|_| workdir.clone());
        let actual_workdir =
            fs::canonicalize(&config.world_root.path).unwrap_or(config.world_root.path);
        assert_eq!(actual_workdir, expected_workdir);
        assert!(!config.world_root.caged);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "disabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "0");
        assert_eq!(env::var("SUBSTRATE_WORLD_ROOT_MODE").unwrap(), "follow-cwd");
        let env_root_path = PathBuf::from(env::var("SUBSTRATE_WORLD_ROOT_PATH").unwrap());
        let env_root_canon = fs::canonicalize(&env_root_path).unwrap_or(env_root_path);
        assert_eq!(env_root_canon, expected_workdir);
        assert_eq!(env::var("SUBSTRATE_CAGED").unwrap(), "0");

        restore_env("SUBSTRATE_WORLD_ROOT_MODE", prev_root_mode);
        restore_env("SUBSTRATE_WORLD_ROOT_PATH", prev_root_path);
        restore_env("SUBSTRATE_CAGED", prev_caged);
        restore_env("SUBSTRATE_ANCHOR_MODE", prev_anchor_mode);
        restore_env("SUBSTRATE_ANCHOR_PATH", prev_anchor_path);
        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }

    #[test]
    fn enforce_caged_destination_bounces_outside_anchor() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let settings = crate::execution::settings::WorldRootSettings {
            mode: WorldRootMode::Project,
            path: fs::canonicalize(&root).unwrap(),
            caged: true,
        };
        let requested = fs::canonicalize(&outside).unwrap();

        let (destination, warning) =
            enforce_caged_destination(&settings, &settings.path, requested);
        assert_eq!(destination, settings.path);
        let message = warning.expect("expected caged warning");
        assert!(message.contains("caged root guard"));
        assert!(message.contains(settings.path.to_str().unwrap()));
    }

    #[test]
    #[serial]
    fn cd_bounces_when_caged_without_world() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let inside = root.join("inside");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&inside).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let mut config = test_shell_config(&temp);
        config.world_root.path = fs::canonicalize(&root).unwrap();
        config.world_root.caged = true;
        config.no_world = true;

        let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
        let prev_pwd = env::var("PWD").ok();
        let prev_oldpwd = env::var("OLDPWD").ok();
        let _guard = DirGuard::new();
        let inside_canon = fs::canonicalize(&inside).unwrap();
        env::set_current_dir(&inside_canon).unwrap();

        let status = handle_builtin(&config, "cd ../../outside", "test-cmd").unwrap();
        assert!(status.is_some());

        assert_eq!(env::current_dir().unwrap(), config.world_root.path);
        assert_eq!(
            env::var("OLDPWD").unwrap(),
            inside_canon.display().to_string()
        );

        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("PWD", prev_pwd);
        restore_env("OLDPWD", prev_oldpwd);
    }

    #[test]
    #[serial]
    fn cd_bounces_when_caged_with_world_enabled() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let inside = root.join("inside");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&inside).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let mut config = test_shell_config(&temp);
        config.world_root.path = fs::canonicalize(&root).unwrap();
        config.world_root.caged = true;
        config.no_world = false;

        let prev_world = set_env("SUBSTRATE_WORLD", "enabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "1");
        let prev_pwd = env::var("PWD").ok();
        let prev_oldpwd = env::var("OLDPWD").ok();
        let _guard = DirGuard::new();
        let inside_canon = fs::canonicalize(&inside).unwrap();
        env::set_current_dir(&inside_canon).unwrap();

        let status = handle_builtin(&config, "cd ../../outside", "test-cmd").unwrap();
        assert!(status.is_some());

        assert_eq!(env::current_dir().unwrap(), config.world_root.path);
        assert_eq!(
            env::var("OLDPWD").unwrap(),
            inside_canon.display().to_string()
        );

        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("PWD", prev_pwd);
        restore_env("OLDPWD", prev_oldpwd);
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn anchor_guard_bounces_chained_cd_when_world_disabled() {
        #[allow(unused_imports)]
        use crate::execution::routing::dispatch::wrap_with_anchor_guard;
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let inside = root.join("inside");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&inside).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let mut config = test_shell_config(&temp);
        config.world_root.path = fs::canonicalize(&root).unwrap();
        config.world_root.caged = true;
        config.no_world = true;

        let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
        let _guard = DirGuard::new();
        let inside_canon = fs::canonicalize(&inside).unwrap();
        env::set_current_dir(&inside_canon).unwrap();

        let wrapped = wrap_with_anchor_guard("cd .. && cd ../outside && pwd", &config);
        let output = std::process::Command::new(&config.shell_path)
            .arg("-c")
            .arg(&wrapped)
            .current_dir(&inside_canon)
            .output()
            .expect("execute guarded command");

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            config.world_root.path.display().to_string()
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("caged root guard"),
            "stderr missing guard warning: {}",
            stderr
        );

        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn anchor_guard_bounces_chained_cd_when_world_enabled() {
        #[allow(unused_imports)]
        use crate::execution::routing::dispatch::wrap_with_anchor_guard;
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let inside = root.join("inside");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&inside).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let mut config = test_shell_config(&temp);
        config.world_root.path = fs::canonicalize(&root).unwrap();
        config.world_root.caged = true;
        config.no_world = false;

        let prev_world = set_env("SUBSTRATE_WORLD", "enabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "1");
        let _guard = DirGuard::new();
        let inside_canon = fs::canonicalize(&inside).unwrap();
        env::set_current_dir(&inside_canon).unwrap();

        let wrapped = wrap_with_anchor_guard("cd .. && cd ../outside && pwd", &config);
        let output = std::process::Command::new(&config.shell_path)
            .arg("-c")
            .arg(&wrapped)
            .current_dir(&inside_canon)
            .output()
            .expect("execute guarded command");

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            config.world_root.path.display().to_string()
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("caged root guard"),
            "stderr missing guard warning: {}",
            stderr
        );

        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    }

    #[test]
    #[serial]
    fn cd_allows_uncaged_escape_from_anchor() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let inside = root.join("inside");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&inside).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let mut config = test_shell_config(&temp);
        config.world_root.path = fs::canonicalize(&root).unwrap();
        config.world_root.caged = false;
        config.no_world = true;

        let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
        let prev_pwd = env::var("PWD").ok();
        let prev_oldpwd = env::var("OLDPWD").ok();
        let _guard = DirGuard::new();
        let inside_canon = fs::canonicalize(&inside).unwrap();
        env::set_current_dir(&inside_canon).unwrap();

        let status = handle_builtin(&config, "cd ../../outside", "test-cmd").unwrap();
        assert!(status.is_some());

        let outside_canon = fs::canonicalize(&outside).unwrap();
        assert_eq!(env::current_dir().unwrap(), outside_canon);

        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("PWD", prev_pwd);
        restore_env("OLDPWD", prev_oldpwd);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Global mutex to ensure tests that modify environment run sequentially
    static TEST_ENV_MUTEX: Mutex<()> = Mutex::new(());

    // Helper to run tests with TEST_MODE set
    fn with_test_mode<F: FnOnce()>(f: F) {
        // Lock the mutex to ensure exclusive access to environment
        let _guard = TEST_ENV_MUTEX.lock().unwrap();

        // Save original value if it exists
        let original = env::var("TEST_MODE").ok();

        env::set_var("TEST_MODE", "1");
        f();

        // Restore original value or remove
        match original {
            Some(val) => env::set_var("TEST_MODE", val),
            None => env::remove_var("TEST_MODE"),
        }
    }

    #[test]
    fn test_sudo_wants_pty() {
        // sudo without flags should want PTY
        assert!(sudo_wants_pty(
            "sudo",
            &["sudo".to_string(), "apt".to_string()]
        ));

        // sudo with -n should not want PTY
        assert!(!sudo_wants_pty(
            "sudo",
            &["sudo".to_string(), "-n".to_string(), "apt".to_string()]
        ));
        assert!(!sudo_wants_pty(
            "sudo",
            &["sudo".to_string(), "--non-interactive".to_string()]
        ));

        // sudo with -S should not want PTY
        assert!(!sudo_wants_pty(
            "sudo",
            &["sudo".to_string(), "-S".to_string()]
        ));
        assert!(!sudo_wants_pty(
            "sudo",
            &["sudo".to_string(), "--stdin".to_string()]
        ));

        // Not sudo
        assert!(!sudo_wants_pty(
            "apt",
            &["apt".to_string(), "update".to_string()]
        ));
    }

    #[test]
    fn test_is_interactive_shell() {
        // Plain shell invocation is interactive
        assert!(is_interactive_shell("bash", &["bash".to_string()]));
        assert!(is_interactive_shell("zsh", &["zsh".to_string()]));
        assert!(is_interactive_shell("sh", &["sh".to_string()]));

        // Shell with -c is not interactive (unless -i is also present)
        assert!(!is_interactive_shell(
            "bash",
            &[
                "bash".to_string(),
                "-c".to_string(),
                "echo hello".to_string()
            ]
        ));
        assert!(is_interactive_shell(
            "bash",
            &[
                "bash".to_string(),
                "-i".to_string(),
                "-c".to_string(),
                "echo hello".to_string()
            ]
        ));

        // Explicit interactive flag
        assert!(is_interactive_shell(
            "bash",
            &["bash".to_string(), "-i".to_string()]
        ));
        assert!(is_interactive_shell(
            "bash",
            &["bash".to_string(), "--interactive".to_string()]
        ));

        // Not a shell
        assert!(!is_interactive_shell("python", &["python".to_string()]));
    }

    #[test]
    fn test_looks_like_repl() {
        // Plain interpreter invocation is REPL
        assert!(looks_like_repl("python", &["python".to_string()]));
        assert!(looks_like_repl("python3", &["python3".to_string()]));
        assert!(looks_like_repl("node", &["node".to_string()]));
        assert!(looks_like_repl("irb", &["irb".to_string()]));

        // With script file is not REPL
        assert!(!looks_like_repl(
            "python",
            &["python".to_string(), "script.py".to_string()]
        ));
        assert!(!looks_like_repl(
            "node",
            &["node".to_string(), "app.js".to_string()]
        ));

        // With inline code is not REPL
        assert!(!looks_like_repl(
            "python",
            &[
                "python".to_string(),
                "-c".to_string(),
                "print('hello')".to_string()
            ]
        ));
        assert!(!looks_like_repl(
            "node",
            &[
                "node".to_string(),
                "-e".to_string(),
                "console.log('hello')".to_string()
            ]
        ));

        // Force interactive with -i is REPL even with script
        assert!(looks_like_repl(
            "python",
            &[
                "python".to_string(),
                "-i".to_string(),
                "script.py".to_string()
            ]
        ));
        assert!(looks_like_repl(
            "python",
            &[
                "python".to_string(),
                "--interactive".to_string(),
                "-c".to_string(),
                "print()".to_string()
            ]
        ));

        // Not an interpreter
        assert!(!looks_like_repl("bash", &["bash".to_string()]));
    }

    #[test]
    fn test_container_wants_pty() {
        // docker run -it
        assert!(container_wants_pty(
            "docker",
            &[
                "docker".to_string(),
                "run".to_string(),
                "-it".to_string(),
                "ubuntu".to_string()
            ]
        ));
        assert!(container_wants_pty(
            "docker",
            &[
                "docker".to_string(),
                "run".to_string(),
                "-ti".to_string(),
                "ubuntu".to_string()
            ]
        ));

        // docker run with separate -i and -t
        assert!(container_wants_pty(
            "docker",
            &[
                "docker".to_string(),
                "run".to_string(),
                "-i".to_string(),
                "-t".to_string(),
                "ubuntu".to_string()
            ]
        ));

        // docker exec -it
        assert!(container_wants_pty(
            "docker",
            &[
                "docker".to_string(),
                "exec".to_string(),
                "-it".to_string(),
                "container1".to_string(),
                "bash".to_string()
            ]
        ));

        // Only -i or only -t is not enough
        assert!(!container_wants_pty(
            "docker",
            &[
                "docker".to_string(),
                "run".to_string(),
                "-i".to_string(),
                "ubuntu".to_string()
            ]
        ));
        assert!(!container_wants_pty(
            "docker",
            &[
                "docker".to_string(),
                "run".to_string(),
                "-t".to_string(),
                "ubuntu".to_string()
            ]
        ));

        // kubectl exec -it
        assert!(container_wants_pty(
            "kubectl",
            &[
                "kubectl".to_string(),
                "exec".to_string(),
                "-it".to_string(),
                "pod1".to_string(),
                "--".to_string(),
                "bash".to_string()
            ]
        ));

        // kubectl exec with separate flags
        assert!(container_wants_pty(
            "kubectl",
            &[
                "kubectl".to_string(),
                "exec".to_string(),
                "-i".to_string(),
                "-t".to_string(),
                "pod1".to_string()
            ]
        ));

        // docker-compose run/exec
        assert!(container_wants_pty(
            "docker-compose",
            &[
                "docker-compose".to_string(),
                "run".to_string(),
                "-it".to_string(),
                "service1".to_string()
            ]
        ));

        // docker compose (space form)
        assert!(container_wants_pty(
            "docker",
            &[
                "docker".to_string(),
                "compose".to_string(),
                "run".to_string(),
                "-it".to_string(),
                "service1".to_string()
            ]
        ));
    }

    #[test]
    fn test_wants_debugger_pty() {
        // Python debuggers
        assert!(wants_debugger_pty(
            "python",
            &[
                "python".to_string(),
                "-m".to_string(),
                "pdb".to_string(),
                "script.py".to_string()
            ]
        ));
        assert!(wants_debugger_pty(
            "python3",
            &["python3".to_string(), "-m".to_string(), "ipdb".to_string()]
        ));

        // Node debuggers
        assert!(wants_debugger_pty(
            "node",
            &[
                "node".to_string(),
                "inspect".to_string(),
                "app.js".to_string()
            ]
        ));
        assert!(wants_debugger_pty(
            "node",
            &[
                "node".to_string(),
                "--inspect".to_string(),
                "app.js".to_string()
            ]
        ));
        assert!(wants_debugger_pty(
            "node",
            &[
                "node".to_string(),
                "--inspect-brk".to_string(),
                "app.js".to_string()
            ]
        ));

        // Not debuggers
        assert!(!wants_debugger_pty(
            "python",
            &["python".to_string(), "script.py".to_string()]
        ));
        assert!(!wants_debugger_pty(
            "node",
            &["node".to_string(), "app.js".to_string()]
        ));
    }

    #[test]
    fn test_git_wants_pty() {
        // git add -p needs PTY
        assert!(git_wants_pty(&[
            "git".to_string(),
            "add".to_string(),
            "-p".to_string()
        ]));
        assert!(git_wants_pty(&[
            "git".to_string(),
            "add".to_string(),
            "-i".to_string()
        ]));

        // git rebase -i needs PTY
        assert!(git_wants_pty(&[
            "git".to_string(),
            "rebase".to_string(),
            "-i".to_string(),
            "HEAD~3".to_string()
        ]));

        // git commit without message needs PTY (opens editor)
        assert!(git_wants_pty(&["git".to_string(), "commit".to_string()]));

        // git commit with -e forces editor even with -m
        assert!(git_wants_pty(&[
            "git".to_string(),
            "commit".to_string(),
            "-m".to_string(),
            "msg".to_string(),
            "-e".to_string()
        ]));
        assert!(git_wants_pty(&[
            "git".to_string(),
            "commit".to_string(),
            "-m".to_string(),
            "msg".to_string(),
            "--edit".to_string()
        ]));

        // git commit with message doesn't need PTY
        assert!(!git_wants_pty(&[
            "git".to_string(),
            "commit".to_string(),
            "-m".to_string(),
            "message".to_string()
        ]));
        assert!(!git_wants_pty(&[
            "git".to_string(),
            "commit".to_string(),
            "--message=message".to_string()
        ]));

        // git commit with --no-edit doesn't need PTY
        assert!(!git_wants_pty(&[
            "git".to_string(),
            "commit".to_string(),
            "--no-edit".to_string()
        ]));

        // Regular git commands don't need PTY
        assert!(!git_wants_pty(&["git".to_string(), "status".to_string()]));
        assert!(!git_wants_pty(&["git".to_string(), "push".to_string()]));
        assert!(!git_wants_pty(&["git".to_string(), "pull".to_string()]));
    }

    #[test]
    fn test_has_top_level_shell_meta() {
        // Top-level metacharacters
        assert!(has_top_level_shell_meta("echo hello | grep h"));
        assert!(has_top_level_shell_meta("ls > file.txt"));
        assert!(has_top_level_shell_meta("cat < input.txt"));
        assert!(has_top_level_shell_meta("cmd1 && cmd2"));
        assert!(has_top_level_shell_meta("cmd1; cmd2"));

        // Metacharacters inside quotes don't count
        assert!(!has_top_level_shell_meta("echo 'hello | world'"));
        assert!(!has_top_level_shell_meta("echo \"hello > world\""));

        // Metacharacters inside subshells don't count
        assert!(!has_top_level_shell_meta("echo $(ls | grep txt)"));
        assert!(!has_top_level_shell_meta("echo `ls | grep txt`"));

        // No metacharacters
        assert!(!has_top_level_shell_meta("echo hello world"));
        assert!(!has_top_level_shell_meta("ls -la"));
    }

    #[test]
    fn test_peel_wrappers() {
        // sshpass wrapper
        assert_eq!(
            peel_wrappers(&[
                "sshpass".to_string(),
                "-p".to_string(),
                "pass".to_string(),
                "ssh".to_string(),
                "host".to_string()
            ]),
            vec!["ssh".to_string(), "host".to_string()]
        );

        // timeout wrapper
        assert_eq!(
            peel_wrappers(&[
                "timeout".to_string(),
                "10".to_string(),
                "command".to_string()
            ]),
            vec!["command".to_string()]
        );
        assert_eq!(
            peel_wrappers(&[
                "timeout".to_string(),
                "-s".to_string(),
                "KILL".to_string(),
                "10".to_string(),
                "command".to_string()
            ]),
            vec!["command".to_string()]
        );

        // env wrapper
        assert_eq!(
            peel_wrappers(&[
                "env".to_string(),
                "VAR=val".to_string(),
                "command".to_string()
            ]),
            vec!["command".to_string()]
        );
        assert_eq!(
            peel_wrappers(&["env".to_string(), "-i".to_string(), "command".to_string()]),
            vec!["command".to_string()]
        );

        // stdbuf wrapper
        assert_eq!(
            peel_wrappers(&[
                "stdbuf".to_string(),
                "-oL".to_string(),
                "command".to_string()
            ]),
            vec!["command".to_string()]
        );

        // nice wrapper
        assert_eq!(
            peel_wrappers(&[
                "nice".to_string(),
                "-n".to_string(),
                "10".to_string(),
                "command".to_string()
            ]),
            vec!["command".to_string()]
        );

        // doas wrapper
        assert_eq!(
            peel_wrappers(&[
                "doas".to_string(),
                "-u".to_string(),
                "user".to_string(),
                "command".to_string()
            ]),
            vec!["command".to_string()]
        );

        // Not a wrapper
        assert_eq!(
            peel_wrappers(&["ls".to_string(), "-la".to_string()]),
            vec!["ls".to_string(), "-la".to_string()]
        );
    }

    #[test]
    fn test_needs_pty_ssh() {
        with_test_mode(|| {
            // SSH without remote command needs PTY
            assert!(needs_pty("ssh host"), "ssh host should need PTY");

            // SSH with -t forces PTY
            assert!(needs_pty("ssh -t host"), "ssh -t host should need PTY");
            assert!(needs_pty("ssh -tt host"), "ssh -tt host should need PTY");
            assert!(
                needs_pty("ssh -t host ls"),
                "ssh -t host ls should need PTY"
            );

            // SSH with -T disables PTY
            assert!(!needs_pty("ssh -T host"), "ssh -T host should not need PTY");
            assert!(
                !needs_pty("ssh -T host ls"),
                "ssh -T host ls should not need PTY"
            );

            // SSH with remote command doesn't need PTY
            assert!(!needs_pty("ssh host ls"), "ssh host ls should not need PTY");
            assert!(!needs_pty("ssh host 'echo hello'"));

            // SSH with BatchMode=yes doesn't need PTY
            assert!(!needs_pty("ssh -o BatchMode=yes host"));
            assert!(!needs_pty("ssh -oBatchMode=yes host"));

            // SSH with RequestTTY options
            assert!(needs_pty("ssh -o RequestTTY=yes host"));
            assert!(needs_pty("ssh -oRequestTTY=force host"));
            assert!(!needs_pty("ssh -o RequestTTY=no host"));

            // SSH with -N (no remote command for port forwarding)
            assert!(!needs_pty("ssh -N -L 8080:localhost:80 host"));

            // SSH with -W (stdio forwarding)
            assert!(!needs_pty("ssh -W localhost:80 host"));
        });
    }

    #[test]
    fn test_needs_pty_known_tuis() {
        with_test_mode(|| {
            // Known TUI editors
            assert!(needs_pty("vim"));
            assert!(needs_pty("vi"));
            assert!(needs_pty("nano"));
            assert!(needs_pty("emacs"));

            // Known TUI pagers
            assert!(needs_pty("less"));
            assert!(needs_pty("more"));

            // Known TUI monitors
            assert!(needs_pty("top"));
            assert!(needs_pty("htop"));
            assert!(needs_pty("btop"));

            // AI tools
            assert!(needs_pty("claude"));

            // Not TUIs
            assert!(!needs_pty("ls"));
            assert!(!needs_pty("cat"));
            assert!(!needs_pty("echo hello"));
        });
    }

    #[test]
    fn test_needs_pty_shell_meta() {
        with_test_mode(|| {
            // Commands with pipes don't need PTY by default
            assert!(!needs_pty("ls | grep txt"));
            assert!(!needs_pty("echo hello > file.txt"));

            // Commands with && or ; don't need PTY
            assert!(!needs_pty("cmd1 && cmd2"));
            assert!(!needs_pty("cmd1; cmd2"));
        });
    }

    #[test]
    fn test_is_force_pty_command() {
        // Save and remove SUBSTRATE_FORCE_PTY if it exists
        let old_force = std::env::var("SUBSTRATE_FORCE_PTY").ok();
        std::env::remove_var("SUBSTRATE_FORCE_PTY");

        // :pty prefix forces PTY
        assert!(is_force_pty_command(":pty ls"));
        assert!(is_force_pty_command(":pty echo hello"));

        // Regular commands without SUBSTRATE_FORCE_PTY
        assert!(!is_force_pty_command("ls"));
        assert!(!is_force_pty_command("echo hello"));

        // Test with SUBSTRATE_FORCE_PTY set
        std::env::set_var("SUBSTRATE_FORCE_PTY", "1");
        assert!(is_force_pty_command("ls"));
        assert!(is_force_pty_command("echo hello"));

        // Restore original state
        match old_force {
            Some(val) => std::env::set_var("SUBSTRATE_FORCE_PTY", val),
            None => std::env::remove_var("SUBSTRATE_FORCE_PTY"),
        }
    }

    #[test]
    fn test_is_pty_disabled() {
        // Test with env var not set
        env::remove_var("SUBSTRATE_DISABLE_PTY");
        assert!(!is_pty_disabled());

        // Test with env var set
        env::set_var("SUBSTRATE_DISABLE_PTY", "1");
        assert!(is_pty_disabled());
        env::remove_var("SUBSTRATE_DISABLE_PTY");
    }

    #[test]
    #[cfg(unix)]
    fn test_stdin_nonblock_roundtrip() {
        // Test that O_NONBLOCK can be set and restored correctly
        // This verifies the save/restore behavior that TerminalGuard relies on
        use std::io;
        use std::os::unix::io::AsRawFd;

        unsafe {
            let fd = io::stdin().as_raw_fd();

            // Get current flags
            let flags_before = libc::fcntl(fd, libc::F_GETFL);
            assert!(flags_before != -1, "Failed to get stdin flags");

            // Mimic TerminalGuard behavior: set O_NONBLOCK
            let result = libc::fcntl(fd, libc::F_SETFL, flags_before | libc::O_NONBLOCK);
            assert!(result != -1, "Failed to set O_NONBLOCK");

            // Verify O_NONBLOCK is set
            let flags_after = libc::fcntl(fd, libc::F_GETFL);
            assert!(
                flags_after != -1,
                "Failed to get flags after setting O_NONBLOCK"
            );
            assert!(
                flags_after & libc::O_NONBLOCK != 0,
                "O_NONBLOCK should be set"
            );

            // Restore original flags
            let result = libc::fcntl(fd, libc::F_SETFL, flags_before);
            assert!(result != -1, "Failed to restore original flags");

            // Verify restoration
            let flags_restored = libc::fcntl(fd, libc::F_GETFL);
            assert!(flags_restored != -1, "Failed to get restored flags");
            assert_eq!(
                flags_restored & libc::O_NONBLOCK,
                flags_before & libc::O_NONBLOCK,
                "O_NONBLOCK state should be restored to original"
            );
        }
    }

    #[test]
    fn test_needs_pty_integration() {
        with_test_mode(|| {
            // Interactive shells need PTY
            assert!(needs_pty("bash"));
            assert!(needs_pty("zsh"));

            // Shell with command doesn't need PTY
            assert!(!needs_pty("bash -c 'echo hello'"));

            // Python REPL needs PTY
            assert!(needs_pty("python"));
            assert!(needs_pty("python3"));

            // Python with script doesn't need PTY
            assert!(!needs_pty("python script.py"));

            // Docker run -it needs PTY
            assert!(needs_pty("docker run -it ubuntu"));

            // Git interactive commands need PTY
            assert!(needs_pty("git add -p"));
            assert!(needs_pty("git commit"));

            // Sudo needs PTY for password
            assert!(needs_pty("sudo apt update"));
            assert!(!needs_pty("sudo -n apt update"));
        });
    }

    #[test]
    fn test_needs_pty_ssh_variations() {
        with_test_mode(|| {
            // SSH with -T flag should not get PTY
            assert!(!needs_pty("ssh -T host 'cmd'"));

            // SSH with -t flag should get PTY
            assert!(needs_pty("ssh -t host"));
            assert!(needs_pty("ssh -tt host"));

            // SSH with remote command (no -t) should not get PTY
            assert!(!needs_pty("ssh host ls"));
            assert!(!needs_pty("ssh host 'echo hello'"));

            // SSH with -l user form
            assert!(needs_pty("ssh -l user host"));
            assert!(!needs_pty("ssh -l user host ls"));

            // SSH with -- delimiter
            assert!(needs_pty("ssh -o SomeOption -- host"));
            assert!(!needs_pty("ssh -o SomeOption -- host ls"));

            // SSH with BatchMode should not get PTY
            assert!(!needs_pty("ssh -o BatchMode=yes host"));

            // SSH with RequestTTY option
            assert!(needs_pty("ssh -o RequestTTY=yes host"));
            assert!(needs_pty("ssh -o RequestTTY=force host"));
            assert!(!needs_pty("ssh -o RequestTTY=no host"));

            // SSH RequestTTY=auto behavior
            assert!(needs_pty("ssh -o RequestTTY=auto host")); // interactive login
            assert!(!needs_pty("ssh -o RequestTTY=auto host id")); // remote cmd, no -t

            // Test case-insensitive options
            assert!(needs_pty("ssh -o RequestTTY=YES host"));
            assert!(needs_pty("ssh -o RequestTTY=Force host"));
            assert!(!needs_pty("ssh -o RequestTTY=NO host"));
            assert!(!needs_pty("ssh -o BatchMode=YES host"));

            // Test inline -o format
            assert!(needs_pty("ssh -oRequestTTY=yes host"));
            assert!(needs_pty("ssh -oRequestTTY=force host"));
            assert!(!needs_pty("ssh -oRequestTTY=no host"));
            assert!(!needs_pty("ssh -oBatchMode=yes host"));

            // Test case-insensitive inline options
            assert!(needs_pty("ssh -oRequestTTY=Yes host"));
            assert!(!needs_pty("ssh -oRequestTTY=No host"));
            assert!(!needs_pty("ssh -oBatchMode=YES host"));

            // SSH with -W should not get PTY unless -t is explicit
            assert!(!needs_pty("ssh -W host:port jumphost"));
            assert!(needs_pty("ssh -t -W host:port jumphost"));

            // SSH with 2-arg options that could confuse host detection
            assert!(needs_pty("ssh -p 2222 host"));
            assert!(needs_pty("ssh -o StrictHostKeyChecking=no host"));
            assert!(!needs_pty("ssh -p 2222 host echo ok"));
            assert!(needs_pty("ssh -J jumphost host"));
            assert!(!needs_pty("ssh -J jumphost host 'id'"));

            // Plain SSH interactive login
            assert!(needs_pty("ssh host"));
            assert!(needs_pty("ssh -l user host"));
            assert!(needs_pty("ssh user@host"));

            // SSH -N flag (no remote command, typically for port forwarding)
            assert!(!needs_pty("ssh -N host"));
            assert!(!needs_pty("ssh -N -L 8080:localhost:80 host"));
            assert!(needs_pty("ssh -t -N host")); // -t forces PTY

            // SSH -O control operations
            assert!(!needs_pty("ssh -O check host"));
            assert!(!needs_pty("ssh -O exit host"));
            assert!(!needs_pty("ssh -O stop host"));
            assert!(needs_pty("ssh -t -O check host")); // -t forces PTY
        });
    }

    #[test]
    fn test_needs_pty_quoted_args() {
        with_test_mode(|| {
            // Quoted filename with spaces
            assert!(needs_pty("vim 'a b.txt'"));
            assert!(needs_pty("vim \"file with spaces.txt\""));

            // Complex quoted arguments
            assert!(needs_pty("vim 'file (1).txt'"));
        });
    }

    #[test]
    fn test_needs_pty_pipes_redirects() {
        with_test_mode(|| {
            // Pipes should prevent PTY
            assert!(!needs_pty("ls | less"));
            assert!(!needs_pty("vim file.txt | grep pattern"));

            // Redirects should prevent PTY
            assert!(!needs_pty("vim > output.txt"));
            assert!(!needs_pty("less < input.txt"));

            // Background jobs should prevent PTY
            assert!(!needs_pty("vim &"));

            // Command sequences should prevent PTY
            assert!(!needs_pty("vim; ls"));
        });
    }

    #[test]
    fn test_repl_heuristic() {
        with_test_mode(|| {
            // Interactive REPL (no args) should get PTY
            assert!(needs_pty("python"));
            assert!(needs_pty("python3"));
            assert!(needs_pty("node"));

            // Script execution should NOT get PTY
            assert!(!needs_pty("python script.py"));
            assert!(!needs_pty("python3 /path/to/script.py"));
            assert!(!needs_pty("node app.js"));

            // Inline code should NOT get PTY
            assert!(!needs_pty("python -c 'print(1)'"));
            assert!(!needs_pty("node -e 'console.log(1)'"));
            assert!(!needs_pty("node -p '1+1'"));
            assert!(!needs_pty("node --print '1+1'"));
            assert!(!needs_pty("node --eval 'console.log(1)'"));

            // Explicit interactive flag should get PTY even with script
            assert!(needs_pty("python -i script.py"));
            assert!(needs_pty("python -i -c 'print(1)'"));
        });
    }

    #[test]
    fn test_debugger_pty() {
        with_test_mode(|| {
            // Debuggers should get PTY
            assert!(needs_pty("python -m pdb script.py"));
            assert!(needs_pty("python3 -m ipdb script.py"));
            assert!(needs_pty("node inspect app.js"));
            assert!(needs_pty("node --inspect-brk app.js"));
            assert!(needs_pty("node --inspect script.js"));
        });
    }

    #[test]
    fn test_windows_exe_handling() {
        with_test_mode(|| {
            // Windows-style paths with .exe should work
            if cfg!(windows) {
                assert!(needs_pty(r#"C:\Python\python.exe"#));
                assert!(needs_pty(r#"C:\Program Files\Git\usr\bin\ssh.exe"#));
                assert!(needs_pty(r#"vim.exe file.txt"#));
            }
        });
    }

    #[test]
    fn test_container_k8s_pty() {
        with_test_mode(|| {
            // Docker/Podman commands with -it should get PTY
            assert!(needs_pty("docker run -it ubuntu bash"));
            assert!(needs_pty("docker exec -it container bash"));
            assert!(needs_pty("podman run -it alpine sh"));
            assert!(!needs_pty("docker run ubuntu echo hello"));

            // Only -t without -i should NOT get PTY (not fully interactive)
            assert!(!needs_pty("podman run -t alpine sh"));
            assert!(!needs_pty("docker run -t ubuntu bash"));

            // kubectl exec with -it should get PTY
            assert!(needs_pty("kubectl exec -it pod -- sh"));
            assert!(needs_pty("kubectl exec --stdin --tty pod -- bash"));
            assert!(!needs_pty("kubectl exec pod -- ls"));
            assert!(!needs_pty("kubectl exec --tty pod -- bash")); // Only -t, not -i

            // Container false positives and split flags
            assert!(!needs_pty("docker run --timeout=5s ubuntu echo hi"));
            assert!(needs_pty("docker exec -ti c bash"));
            assert!(needs_pty("kubectl exec --stdin --tty pod -- sh"));
            assert!(needs_pty("docker exec -i -t c bash"));
            assert!(needs_pty("docker exec -t -i c bash"));

            // Docker/Podman should NOT detect flags in the in-container command
            assert!(!needs_pty("docker run ubuntu bash -lc \"echo -t\""));
            assert!(!needs_pty("docker exec c sh -c 'echo -it'"));

            // Docker -- separator sanity test
            assert!(needs_pty("docker run -it -- ubuntu bash"));

            // docker-compose support (both forms)
            assert!(needs_pty("docker-compose exec -it web sh"));
            assert!(needs_pty("docker compose exec -it web sh"));
            assert!(needs_pty("docker compose run --rm -it web sh"));
            assert!(!needs_pty("docker compose exec web ls"));
        });
    }

    #[test]
    fn test_sudo_pty() {
        with_test_mode(|| {
            // sudo should get PTY for password prompt
            assert!(needs_pty("sudo ls"));
            assert!(needs_pty("sudo apt update"));

            // sudo with -n or -S should NOT get PTY
            assert!(!needs_pty("sudo -n ls"));
            assert!(!needs_pty("sudo --non-interactive command"));
            assert!(!needs_pty("sudo -S ls"));

            // sudo -S (stdin password)
            assert!(!needs_pty("sudo -S true"));

            // sudo with -A (askpass) doesn't get PTY
            assert!(!needs_pty("sudo -A true"));
            assert!(!needs_pty("sudo --askpass true"));
        });
    }

    #[test]
    fn test_interactive_shells() {
        with_test_mode(|| {
            // Interactive shells should get PTY
            assert!(needs_pty("bash"));
            assert!(needs_pty("zsh"));
            assert!(needs_pty("sh"));
            assert!(needs_pty("fish"));
            assert!(needs_pty("bash -i"));
            assert!(needs_pty("zsh --interactive"));

            // Shells with -c should NOT get PTY (unless -i is also present)
            assert!(!needs_pty("bash -c 'echo ok'"));
            assert!(!needs_pty("sh -c 'ls'"));
            assert!(needs_pty("bash -i -c 'echo ok'")); // -i overrides

            // bash --interactive
            assert!(needs_pty("bash --interactive"));
        });
    }

    #[test]
    fn test_git_selective_pty() {
        with_test_mode(|| {
            // Interactive git commands should get PTY
            assert!(needs_pty("git add -p"));
            assert!(needs_pty("git add -i"));
            assert!(needs_pty("git rebase -i"));
            assert!(needs_pty("git commit")); // No -m, will open editor

            // Non-interactive git commands should NOT get PTY
            assert!(!needs_pty("git status"));
            assert!(!needs_pty("git log"));
            assert!(!needs_pty("git diff"));
            assert!(!needs_pty("git commit -m 'message'"));
            assert!(!needs_pty("git add file.txt"));
            assert!(!needs_pty("git push"));

            // git commit with --no-edit and -F should not get PTY
            assert!(!needs_pty("git commit --no-edit"));
            assert!(!needs_pty("git commit -F msg.txt"));
            assert!(!needs_pty("git commit --file=msg.txt"));

            // git with global options before subcommand
            assert!(needs_pty("git -c core.editor=vim commit"));
            assert!(needs_pty("git -C repo commit"));
            assert!(!needs_pty(
                "git --git-dir=.git --work-tree=. commit -m 'msg'"
            ));
        });
    }

    #[test]
    fn test_wrapper_commands() {
        with_test_mode(|| {
            // sshpass wrapper
            assert!(needs_pty("sshpass -p x ssh host"));
            assert!(!needs_pty("sshpass -p x ssh host ls"));

            // env wrapper with -i and -u flags
            assert!(needs_pty("env -i vim file"));
            assert!(needs_pty("env -u PATH bash"));
            assert!(needs_pty("env FOO=1 -i bash"));
            assert!(needs_pty("env FOO=1 ssh -t host"));
            assert!(needs_pty("env FOO=1 BAR=2 vim file.txt"));

            // timeout wrapper
            assert!(needs_pty("timeout 10s ssh host"));
            assert!(!needs_pty("timeout 10s ssh host ls"));

            // stdbuf wrapper
            assert!(needs_pty("stdbuf -oL less README.md"));
            assert!(needs_pty("stdbuf -oL vim file.txt"));

            // nice/ionice wrappers
            assert!(needs_pty("nice -n 10 vim file.txt"));
            assert!(needs_pty("ionice -n 5 less README.md"));

            // doas wrapper (sudo alternative)
            assert!(needs_pty("doas vim /etc/hosts"));
            assert!(needs_pty("doas -u user ssh host"));
        });
    }

    #[test]
    fn test_pipeline_last_tui() {
        with_test_mode(|| {
            // This test requires SUBSTRATE_PTY_PIPELINE_LAST to be set
            let old_pipeline = std::env::var("SUBSTRATE_PTY_PIPELINE_LAST").ok();
            std::env::set_var("SUBSTRATE_PTY_PIPELINE_LAST", "1");

            // Pipeline with TUI at the end should get PTY
            assert!(needs_pty("ls | less"));
            assert!(needs_pty("git ls-files | fzf"));

            // Pipeline with redirect should NOT get PTY
            assert!(!needs_pty("ls | less > out.txt"));
            assert!(!needs_pty("git diff | head > changes.txt"));
            assert!(!needs_pty("ls | less 2>err.log"));
            assert!(!needs_pty("cmd | less < input.txt"));
            assert!(!needs_pty("ls | less >> append.txt"));
            assert!(!needs_pty("ls | less 2>&1"));

            // Restore SUBSTRATE_PTY_PIPELINE_LAST
            match old_pipeline {
                Some(val) => std::env::set_var("SUBSTRATE_PTY_PIPELINE_LAST", val),
                None => std::env::remove_var("SUBSTRATE_PTY_PIPELINE_LAST"),
            }
        });
    }

    #[test]
    fn test_ssh_spacing_edge_cases() {
        with_test_mode(|| {
            // SSH with spaces around = in options (OpenSSH accepts this)
            assert!(needs_pty("ssh -o RequestTTY = yes host"));
            assert!(needs_pty("ssh -o RequestTTY = force host"));
            assert!(!needs_pty("ssh -o RequestTTY = no host"));
            assert!(!needs_pty("ssh -o BatchMode = yes host"));

            // -E and -B options with 2 args
            assert!(needs_pty("ssh -E logfile.txt host"));
            assert!(needs_pty("ssh -B 192.168.1.1 host"));
            assert!(!needs_pty("ssh -E log.txt host ls"));
        });
    }

    #[test]
    fn test_force_vs_disable_precedence() {
        // Test that force overrides disable at the execute_command level
        let old_disable = std::env::var("SUBSTRATE_DISABLE_PTY").ok();
        let old_force = std::env::var("SUBSTRATE_FORCE_PTY").ok();

        // Test with both set - force should override
        std::env::set_var("SUBSTRATE_DISABLE_PTY", "1");
        std::env::set_var("SUBSTRATE_FORCE_PTY", "1");

        // is_force_pty_command should return true when SUBSTRATE_FORCE_PTY is set
        assert!(is_force_pty_command("echo hello"));
        assert!(is_force_pty_command("ls -l"));

        // :pty prefix should also force
        assert!(is_force_pty_command(":pty echo hello"));

        // is_pty_disabled should still return true
        assert!(is_pty_disabled());

        // Restore environment variables
        match old_disable {
            Some(val) => std::env::set_var("SUBSTRATE_DISABLE_PTY", val),
            None => std::env::remove_var("SUBSTRATE_DISABLE_PTY"),
        }
        match old_force {
            Some(val) => std::env::set_var("SUBSTRATE_FORCE_PTY", val),
            None => std::env::remove_var("SUBSTRATE_FORCE_PTY"),
        }
    }

    #[test]
    fn test_git_commit_edit_flag() {
        with_test_mode(|| {
            // git commit -e can override -m to open editor
            assert!(needs_pty("git commit -m 'msg' -e"));
            assert!(needs_pty("git commit -m 'msg' --edit"));

            // --no-edit overrides -e
            assert!(!needs_pty("git commit -e --no-edit"));
            assert!(!needs_pty("git commit --edit --no-edit"));
        });
    }
    #[cfg(target_os = "windows")]
    #[test]
    fn transport_meta_named_pipe_mode() {
        let meta = world_transport_to_meta(&pw::WorldTransport::NamedPipe(PathBuf::from(
            r"\\.\pipe\substrate-agent",
        )));
        assert_eq!(meta.mode, "named_pipe");
        assert_eq!(meta.endpoint.as_deref(), Some(r"\\.\pipe\substrate-agent"));
    }
}
