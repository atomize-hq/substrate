use super::agent_events::publish_agent_event;
use super::cli::*;
use super::invocation::{
    needs_shell, run_interactive_shell, run_pipe_mode, run_script_mode, run_wrap_mode, ShellConfig,
    ShellMode,
};
use super::pty;
mod builtin;
mod path_env;
use super::shim_deploy::{DeploymentStatus, ShimDeployer};
use super::{
    configure_child_shell_env, configure_manager_init, log_manager_init_event,
    write_manager_env_script,
};
use crate::builtins as commands;
use crate::repl::async_repl;
use crate::scripts::write_bash_preexec_script;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, IsTerminal};
// (avoid unused: import Read/Write locally where needed)
use agent_api_types::ExecuteStreamFrame;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use substrate_broker::{detect_profile, evaluate, set_global_broker, BrokerHandle, Decision};
use substrate_common::{
    agent_events::{AgentEvent, AgentEventKind},
    log_schema, paths as substrate_paths, redact_sensitive, WorldRootMode,
};
use substrate_trace::{
    append_to_trace, create_span_builder, init_trace, set_global_trace_context, PolicyDecision,
    TraceContext, TransportMeta,
};
use tracing::{info, warn};
use uuid::Uuid;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use agent_api_client::AgentClient;

// Reedline imports
#[cfg_attr(target_os = "windows", allow(unused_imports))]
use std::thread;
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(target_os = "linux")]
use tokio::signal::unix::{signal, SignalKind};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use tokio_tungstenite as tungs;
// use nu_ansi_term::{Color, Style}; // Unused for now
#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::pw;
pub(crate) use builtin::handle_builtin;
#[cfg(target_os = "linux")]
use nix::sys::termios::{
    self, ControlFlags, InputFlags, LocalFlags, OutputFlags, SetArg, SpecialCharacterIndices,
    Termios,
};
pub(crate) use path_env::{canonicalize_or, world_deps_manifest_base_path};
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
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

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

/// Collect filesystem diff and network scopes from world backend
#[allow(unused_variables)]
fn collect_world_telemetry(
    _span_id: &str,
) -> (Vec<String>, Option<substrate_common::fs_diff::FsDiff>) {
    // Try to get world handle from environment
    let world_id = match env::var("SUBSTRATE_WORLD_ID") {
        Ok(id) => id,
        Err(_) => {
            // No world ID, return empty telemetry
            return (vec![], None);
        }
    };

    // Create world backend and collect telemetry
    #[cfg(target_os = "linux")]
    {
        use world::LinuxLocalBackend;
        use world_api::WorldBackend;

        let backend = LinuxLocalBackend::new();
        let handle = world_api::WorldHandle {
            id: world_id.clone(),
        };

        // Try to get filesystem diff
        let fs_diff = backend.fs_diff(&handle, _span_id).ok(); // PTY sessions may run in a separate process; missing cache is expected

        // For now, scopes are tracked in the session world's execute method
        // and would need to be retrieved from there
        let scopes_used = vec![];

        (scopes_used, fs_diff)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // World backend only available on Linux for now
        (vec![], None)
    }
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

/// Check if it's sudo that needs PTY for password prompt
fn sudo_wants_pty(cmd_lower: &str, tokens: &[String]) -> bool {
    if cmd_lower != "sudo" {
        return false;
    }

    // No PTY if -n/-S/-A or their long forms
    !tokens.iter().any(|t| {
        matches!(
            t.as_str(),
            "-n" | "--non-interactive" | "-S" | "--stdin" | "-A" | "--askpass"
        )
    })
}

/// Check if it's an interactive shell
fn is_interactive_shell(cmd_lower: &str, tokens: &[String]) -> bool {
    let is_shell = matches!(cmd_lower, "bash" | "zsh" | "sh" | "fish" | "dash" | "ksh");
    if !is_shell {
        return false;
    }

    // No PTY if executing command with -c
    let has_command = tokens.iter().any(|t| t == "-c");

    // Explicit interactive flag
    let has_interactive = tokens.iter().any(|t| t == "-i" || t == "--interactive");

    // It's interactive if: no -c flag OR explicit -i flag
    !has_command || has_interactive
}

/// Check if interpreter command looks like interactive REPL
fn looks_like_repl(cmd_lower: &str, tokens: &[String]) -> bool {
    let is_interp = matches!(
        cmd_lower,
        "python" | "python3" | "ipython" | "bpython" | "node" | "irb" | "pry"
    );
    if !is_interp {
        return false;
    }

    // Force interactive if -i/--interactive present, regardless of script/inline code
    let has_i = tokens.iter().any(|t| t == "-i" || t == "--interactive");
    if has_i {
        return true;
    }

    // Check for script file (any non-option argument after the command)
    let has_script = tokens.iter().skip(1).any(|t| !t.starts_with('-'));

    // Check for inline code execution flags
    let has_inline = tokens.iter().any(|t| {
        matches!(
            t.as_str(),
            "-c" |                                    // python
            "-e" | "--eval" | "-p" | "--print" // node
        )
    });

    // REPL when no script AND not inline
    !has_script && !has_inline
}

/// Check if it's a container/k8s command that needs PTY
fn container_wants_pty(cmd_lower: &str, tokens: &[String]) -> bool {
    // Check for "docker compose" (space-separated form)
    let is_docker_compose = cmd_lower == "docker"
        && tokens
            .get(1)
            .map(|s| s.as_str() == "compose")
            .unwrap_or(false);

    // Docker/Podman/docker-compose run|exec: only scan flags up to image/container name
    if matches!(cmd_lower, "docker" | "podman" | "docker-compose") || is_docker_compose {
        if let Some(subcmd_idx) = tokens.iter().position(|t| t == "run" || t == "exec") {
            let mut has_i = false;
            let mut has_t = false;

            for token in tokens.iter().skip(subcmd_idx + 1) {
                if token == "--" {
                    break;
                }
                if let Some(stripped) = token.strip_prefix('-') {
                    if token == "-it" || token == "-ti" {
                        return true;
                    }
                    if token == "-i" || token == "--interactive" || token == "--stdin" {
                        has_i = true;
                    }
                    if token == "-t" || token == "--tty" {
                        has_t = true;
                    }
                    if !token.starts_with("--") && !stripped.is_empty() {
                        let chars: Vec<char> = stripped.chars().collect();
                        if chars.contains(&'i') {
                            has_i = true;
                        }
                        if chars.contains(&'t') {
                            has_t = true;
                        }
                    }
                } else {
                    // First non-option = image (run) or container (exec)
                    break; // stop scanning; rest belongs to the in-container command
                }
            }
            // Need both -i and -t for interactive container session
            return has_i && has_t;
        }
    }

    // kubectl exec with proper flag detection (scoped to after exec, stop at --)
    if cmd_lower == "kubectl" {
        if let Some(exec_idx) = tokens.iter().position(|t| t == "exec") {
            let mut has_i = false;
            let mut has_t = false;

            // Only check flags after exec and before --
            for token in tokens.iter().skip(exec_idx + 1) {
                // Stop scanning at -- (rest are remote command args)
                if token == "--" {
                    break;
                }

                if token == "-it" || token == "-ti" {
                    return true;
                }
                if token == "-i" || token == "--stdin" {
                    has_i = true;
                }
                if token == "-t" || token == "--tty" {
                    has_t = true;
                }
                // Check for flags in clusters
                if token.starts_with("-") && !token.starts_with("--") && token.len() > 1 {
                    let chars: Vec<char> = token[1..].chars().collect();
                    if chars.contains(&'i') {
                        has_i = true;
                    }
                    if chars.contains(&'t') {
                        has_t = true;
                    }
                }
            }
            // kubectl: need both -i and -t for interactive exec
            return has_i && has_t;
        }
    }

    false
}

/// Check if command is launching an interactive debugger
fn wants_debugger_pty(cmd_lower: &str, tokens: &[String]) -> bool {
    // Python debuggers: python -m pdb/ipdb
    if cmd_lower == "python" || cmd_lower == "python3" {
        if let Some(i) = tokens.iter().position(|t| t == "-m") {
            if let Some(modname) = tokens.get(i + 1) {
                if modname == "pdb" || modname == "ipdb" {
                    return true;
                }
            }
        }
    }

    // Node debuggers: node inspect or node --inspect-brk
    if cmd_lower == "node"
        && tokens
            .iter()
            .any(|t| t == "inspect" || t == "--inspect" || t == "--inspect-brk")
    {
        return true;
    }

    false
}

/// Check if git command needs interactive PTY
fn git_wants_pty(tokens: &[String]) -> bool {
    // Skip "git"
    let mut i = 1;

    // Git global options that may appear before the subcommand.
    // Options that consume a value: -C <path>, -c <name=val>, --git-dir <path>, --work-tree <path>, --namespace <ns>
    while i < tokens.len() {
        let t = tokens[i].as_str();
        match t {
            "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace" => {
                i += 2; // skip option + value
            }
            _ if t.starts_with("--git-dir=")
                || t.starts_with("--work-tree=")
                || t.starts_with("--namespace=") =>
            {
                i += 1;
            }
            // First non-option token is the subcommand
            _ if !t.starts_with('-') => break,
            // Unknown global flag without value (safe to skip)
            _ => i += 1,
        }
    }

    if i >= tokens.len() {
        return false;
    }
    let sub = tokens[i].as_str();

    match sub {
        "add" => tokens.iter().any(|t| t == "-p" || t == "-i"),
        "rebase" => tokens.iter().any(|t| t == "-i"),
        "commit" => {
            // Scan all flags - -e/--edit can override -m/-F to open editor
            let mut no_editor = false;
            let mut force_editor = false;
            for t in tokens.iter().skip(i + 1) {
                if t == "-e" || t == "--edit" {
                    force_editor = true;
                }
                if t == "-m"
                    || t == "--message"
                    || t.starts_with("-m")
                    || t.starts_with("--message=")
                {
                    no_editor = true;
                }
                if t == "-F" || t == "--file" || t.starts_with("--file=") {
                    no_editor = true;
                }
                if t == "--no-edit" {
                    no_editor = true;
                    force_editor = false; // --no-edit overrides -e
                }
            }
            // Editor opens if forced OR if no message provided
            force_editor || !no_editor
        }
        _ => false,
    }
}

/// Check for shell metacharacters at top-level (not inside quotes, subshells, or backticks)
fn has_top_level_shell_meta(s: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    let mut in_backticks = false;
    let mut escape = false;
    let mut subshell_depth = 0;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if escape {
            escape = false;
            continue;
        }

        // Check for $( subshell start
        if ch == '$' && !in_single && !in_backticks && chars.peek() == Some(&'(') {
            chars.next(); // consume '('
            subshell_depth += 1;
            continue;
        }

        match ch {
            '\\' if !in_single => {
                escape = true;
            }
            '`' if !in_single && !in_double && subshell_depth == 0 => {
                in_backticks = !in_backticks;
            }
            '\'' if !in_double && !in_backticks && subshell_depth == 0 => {
                in_single = !in_single;
            }
            '"' if !in_single && !in_backticks && subshell_depth == 0 => {
                in_double = !in_double;
            }
            '(' if !in_single && !in_double && !in_backticks && subshell_depth > 0 => {
                subshell_depth += 1;
            }
            ')' if !in_single && !in_double && !in_backticks && subshell_depth > 0 => {
                subshell_depth -= 1;
            }
            '|' | '>' | '<' | '&' | ';'
                if !in_single && !in_double && !in_backticks && subshell_depth == 0 =>
            {
                return true
            }
            _ => {}
        }
    }
    false
}

/// Strip known wrapper commands to find the actual command being run
fn peel_wrappers(tokens: &[String]) -> Vec<String> {
    if tokens.is_empty() {
        return tokens.to_vec();
    }

    let i = 0;
    if i < tokens.len() {
        let cmd = tokens[i].as_str();

        // Get base command name (strip path)
        let base_cmd = std::path::Path::new(cmd)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(cmd);

        match base_cmd {
            // Wrappers that take 1 argument
            "sshpass" => {
                // sshpass -p pass cmd... or sshpass -f file cmd...
                if i + 1 < tokens.len()
                    && (tokens[i + 1] == "-p" || tokens[i + 1] == "-f")
                    && i + 3 < tokens.len()
                {
                    return tokens[i + 3..].to_vec(); // Skip sshpass -p pass
                }
                return tokens[i + 1..].to_vec(); // Skip just sshpass
            }
            "timeout" => {
                // timeout [opts] duration command...
                let mut j = i + 1;
                // Skip options
                while j < tokens.len() && tokens[j].starts_with('-') {
                    j += if tokens[j] == "-s" || tokens[j] == "--signal" {
                        2
                    } else {
                        1
                    };
                }
                // Skip duration
                if j < tokens.len() && !tokens[j].starts_with('-') {
                    j += 1;
                }
                if j < tokens.len() {
                    return tokens[j..].to_vec();
                }
                return vec![];
            }
            "env" => {
                // env [-i] [-u NAME]... [VAR=val]... command...
                let mut j = i + 1;
                while j < tokens.len() {
                    let t = tokens[j].as_str();
                    match t {
                        "-i" => j += 1,                    // clear environment
                        "-u" => j += 2,                    // unset NAME
                        _ if t.starts_with('-') => j += 1, // other env flags
                        _ if t.contains('=') => j += 1,    // VAR=val
                        _ => break,                        // first real command
                    }
                }
                return tokens.get(j..).map(|s| s.to_vec()).unwrap_or_else(Vec::new);
            }
            "stdbuf" => {
                // stdbuf -oL|-eL|-iL command...
                let mut j = i + 1;
                while j < tokens.len() && tokens[j].starts_with('-') {
                    j += 1;
                }
                if j < tokens.len() {
                    return tokens[j..].to_vec();
                }
                return vec![];
            }
            "nice" | "ionice" => {
                // nice [-n priority] command...
                let mut j = i + 1;
                if j < tokens.len() && tokens[j] == "-n" {
                    j += 2; // Skip -n and value
                }
                if j < tokens.len() {
                    return tokens[j..].to_vec();
                }
                return vec![];
            }
            "doas" => {
                // doas [-u user] command... (sudo alternative)
                let mut j = i + 1;
                if j < tokens.len() && tokens[j] == "-u" {
                    j += 2; // Skip -u and user
                }
                if j < tokens.len() {
                    return tokens[j..].to_vec();
                }
                return vec![];
            }
            _ => return tokens.to_vec(), // Not a wrapper
        }
    }

    tokens.to_vec()
}

/// Determines if a command needs PTY allocation for proper terminal control
fn needs_pty(cmd: &str) -> bool {
    // For unit tests, skip actual TTY detection
    let is_test_mode = std::env::var("TEST_MODE").is_ok();

    // If parent stdio isn't a TTY, never use PTY (skip in test mode)
    if !is_test_mode {
        let stdin_is_tty = io::stdin().is_terminal();
        let stdout_is_tty = io::stdout().is_terminal();
        if !(stdin_is_tty && stdout_is_tty) {
            return false;
        }
    }

    // Optional: Enable pipeline-last TUI detection
    let enable_pipeline_last = std::env::var("SUBSTRATE_PTY_PIPELINE_LAST").is_ok();

    // Check for shell metacharacters at top-level (not inside quotes)
    if has_top_level_shell_meta(cmd) {
        // If pipeline-last is enabled, check if last segment needs PTY
        if enable_pipeline_last && cmd.contains('|') {
            // Simple heuristic: split by top-level pipes and check last segment
            // This is simplified - a full implementation would parse properly
            if let Some(last_segment) = cmd.rsplit('|').next() {
                // Check if output is redirected (>, <, >>, 1>, 2>, 2>&1, etc.)
                let has_redirect = last_segment.chars().any(|c| c == '>' || c == '<')
                    || last_segment.contains("&>");
                if !has_redirect {
                    // Recursively check if last segment needs PTY
                    return needs_pty(last_segment.trim());
                }
            }
        }
        return false;
    }

    // Conservative allowlist for known TUIs that definitely need PTY
    const KNOWN_TUIS: &[&str] = &[
        "vim", "vi", "nvim", "neovim", "nano", "emacs", // editors
        "less", "more", "most", // pagers
        "top", "htop", "btop", "glances", // monitors
        "telnet", "ftp", "sftp", // network tools
        "claude", "codex", "gemini", "atomize", // AI tools
        "tmux", "screen", "zellij", // multiplexers
        "fzf", "lazygit", "gitui", "tig", // git/file tools
        "ranger", "yazi", "k9s", "nmtui", // additional TUIs
        "ipython", "bpython", // interactive pythons
        "sqlite3", "psql",
        "mysql", // database CLIs
                 // Note: python, node, git, ssh handled by special logic
                 // 🔥 PRODUCTION FIX: Removed ssh from list since dedicated logic is comprehensive
    ];

    // Parse command properly using shell_words for quoted argument handling
    let tokens = match shell_words::split(cmd) {
        Ok(tokens) => tokens,
        Err(_) => {
            // Fallback: on Windows, accept bare paths like C:\Foo\bar.exe
            #[cfg(windows)]
            {
                vec![cmd.to_string()]
            }
            #[cfg(not(windows))]
            {
                return false; // Malformed command, don't use PTY
            }
        }
    };

    // Peel off wrapper commands to find the actual command
    let peeled_tokens = peel_wrappers(&tokens);

    // Use peeled tokens if available, otherwise original
    let working_tokens = if !peeled_tokens.is_empty() {
        &peeled_tokens
    } else {
        &tokens
    };

    // Windows-safe program extraction: prefer the program component from the original string
    #[cfg(windows)]
    let first_raw = {
        // Try to extract <...>.exe from the original string regardless of spaces
        let lower = cmd.to_ascii_lowercase();
        if let Some(pos) = lower.find(".exe") {
            &cmd[..pos + 4]
        } else {
            working_tokens.first().map(|s| s.as_str()).unwrap_or("")
        }
    };
    #[cfg(not(windows))]
    let first_raw = working_tokens.first().map(|s| s.as_str()).unwrap_or("");
    let first_token = Path::new(first_raw)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // 🔥 EXPERT FIX: Convert to lowercase FIRST, then strip Windows extensions
    let lower = first_token.to_ascii_lowercase();
    let cmd_lower = if cfg!(windows) {
        lower
            .trim_end_matches(".exe")
            .trim_end_matches(".cmd")
            .trim_end_matches(".bat")
            .to_string()
    } else {
        lower
    };

    // Check for sudo (needs PTY for password prompt)
    if sudo_wants_pty(&cmd_lower, working_tokens) {
        return true;
    }

    // Check if it's an interactive shell
    if is_interactive_shell(&cmd_lower, working_tokens) {
        return true;
    }

    // Check if it's an interactive REPL
    if looks_like_repl(&cmd_lower, working_tokens) {
        return true;
    }

    // Check if it's launching a debugger
    if wants_debugger_pty(&cmd_lower, working_tokens) {
        return true;
    }

    // Check for container/k8s commands
    if container_wants_pty(&cmd_lower, working_tokens) {
        return true;
    }

    // Check if it's an interactive git command
    if cmd_lower == "git" && git_wants_pty(working_tokens) {
        return true;
    }

    // Special SSH handling for -t/-T flags and remote commands
    if cmd_lower == "ssh" {
        // If no args at all, assume interactive client
        if working_tokens.len() == 1 {
            return true;
        }
        // Create lowercase versions for case-insensitive option checking
        let tokens_lc: Vec<String> = working_tokens
            .iter()
            .map(|t| t.to_ascii_lowercase())
            .collect();

        // Check for explicit -t or -tt flag (force PTY)
        let has_t = tokens_lc.iter().any(|arg| arg == "-t" || arg == "-tt");

        // Check for explicit -T flag (no PTY) - uppercase T
        if working_tokens.iter().any(|arg| arg == "-T") {
            return false;
        }

        // Check for -N flag (no remote command, typically for port forwarding)
        // Only deny PTY if -t/-tt not present
        if working_tokens.iter().any(|arg| arg == "-N") && !has_t {
            return false;
        }

        // Check for -O control operations (check|exit|stop|forward|cancel)
        if working_tokens.iter().any(|arg| arg == "-O") && !has_t {
            return false;
        }

        // Check for -W (stdio forwarding) - never PTY unless -t is explicit
        if tokens_lc.iter().any(|arg| arg == "-w") && !has_t {
            return false;
        }

        // If -t or -tt was present, force PTY
        if has_t {
            return true;
        }

        // Check for BatchMode=yes (case-insensitive, no PTY)
        // First check inline form: -oBatchMode=yes
        for arg in &tokens_lc {
            if let Some(val) = arg.strip_prefix("-obatchmode=") {
                if val == "yes" {
                    return false;
                }
            }
        }
        // Check spaced form: -o BatchMode=yes or -o BatchMode = yes
        for (i, arg) in tokens_lc.iter().enumerate() {
            if arg == "-o" && i + 1 < tokens_lc.len() {
                // Handle: -o BatchMode=yes
                if tokens_lc[i + 1] == "batchmode=yes" {
                    return false;
                }
                // Handle: -o BatchMode = yes (with spaces)
                if tokens_lc[i + 1] == "batchmode"
                    && i + 3 < tokens_lc.len()
                    && tokens_lc[i + 2] == "="
                    && tokens_lc[i + 3] == "yes"
                {
                    return false;
                }
            }
        }

        // Check for RequestTTY option (case-insensitive, ssh_config style)
        // First check spaced form: -o RequestTTY=value or -o RequestTTY = value
        for (i, arg) in tokens_lc.iter().enumerate() {
            if arg == "-o" && i + 1 < tokens_lc.len() {
                // Handle: -o RequestTTY=value
                if let Some(val) = tokens_lc[i + 1].strip_prefix("requesttty=") {
                    match val {
                        "yes" | "force" => return true,
                        "no" => return false,
                        _ => {}
                    }
                }
                // Handle: -o RequestTTY = value (with spaces)
                if tokens_lc[i + 1] == "requesttty"
                    && i + 3 < tokens_lc.len()
                    && tokens_lc[i + 2] == "="
                {
                    match tokens_lc[i + 3].as_str() {
                        "yes" | "force" => return true,
                        "no" => return false,
                        _ => {}
                    }
                }
            }
        }

        // Check inline form: -oRequestTTY=value
        for arg in &tokens_lc {
            if let Some(val) = arg.strip_prefix("-orequesttty=") {
                match val {
                    "yes" | "force" => return true,
                    "no" => return false,
                    _ => {}
                }
            }
        }

        // Handle all 2-arg SSH options (not just -l)
        // 🔥 EXPERT FIX: Skip ALL 2-arg options to correctly identify host
        let mut skip_next = false;
        let mut host_idx = None;
        // Start from index 1 to skip "ssh" itself
        for i in 1..working_tokens.len() {
            let arg = &working_tokens[i];
            if skip_next {
                skip_next = false;
                continue;
            }
            // Skip all 2-arg SSH options: -p -l -i -F -J -b -c -D -L -R -S -E -B -o
            if matches!(
                arg.as_str(),
                "-p" | "-l"
                    | "-i"
                    | "-F"
                    | "-J"
                    | "-b"
                    | "-c"
                    | "-D"
                    | "-L"
                    | "-R"
                    | "-S"
                    | "-E"
                    | "-B"
            ) {
                skip_next = true;
                continue;
            }
            // Handle -o option (can be -o key=val or -okey=val)
            if arg == "-o" {
                skip_next = true;
                continue;
            }
            // Stop at -- delimiter
            if arg == "--" {
                if i + 1 < working_tokens.len() {
                    host_idx = Some(i + 1);
                }
                break;
            }
            // First non-option is the host
            if !arg.starts_with('-') && !arg.contains('=') {
                host_idx = Some(i);
                break;
            }
        }

        // Check if there's a remote command after the host
        if let Some(idx) = host_idx {
            if idx + 1 < working_tokens.len() {
                // There's a remote command, no explicit -t, so no PTY
                return false;
            }
        }

        // 🔥 CRITICAL FIX: No -T/-W/BatchMode, no remote command => interactive login
        return true;
    }

    // Check if it's a known TUI
    KNOWN_TUIS.iter().any(|&tui| cmd_lower == tui)
}

/// Force PTY for specific command (user override)
fn is_force_pty_command(cmd: &str) -> bool {
    cmd.starts_with(":pty ") || std::env::var("SUBSTRATE_FORCE_PTY").is_ok()
}

/// Check if PTY is disabled globally
fn is_pty_disabled() -> bool {
    std::env::var("SUBSTRATE_DISABLE_PTY").is_ok()
}

pub(crate) fn execute_command(
    config: &ShellConfig,
    command: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
) -> Result<ExitStatus> {
    let trimmed = command.trim();

    // Prepare redacted command once (used for span + logging)
    let redacted_for_logging = if std::env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        trimmed.to_string()
    } else {
        let toks = shell_words::split(trimmed)
            .unwrap_or_else(|_| trimmed.split_whitespace().map(|s| s.to_string()).collect());
        let mut out = Vec::new();
        let mut i = 0;

        while i < toks.len() {
            let t = &toks[i];
            let lt = t.to_lowercase();

            if lt == "-u" || lt == "--user" || lt == "--password" || lt == "--token" || lt == "-p" {
                out.push("***".into());
                if i + 1 < toks.len() {
                    out.push("***".into());
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }

            if t == "-H" || lt == "--header" {
                out.push(t.clone());
                if i + 1 < toks.len() {
                    let hv = &toks[i + 1];
                    let lower = hv.to_lowercase();
                    let redacted = if lower.starts_with("authorization:")
                        || lower.starts_with("x-api-key:")
                        || lower.starts_with("x-auth-token:")
                        || lower.starts_with("cookie:")
                    {
                        format!(
                            "{}: ***",
                            hv.split(':').next().unwrap_or("").trim_end_matches(':')
                        )
                    } else {
                        hv.clone()
                    };
                    out.push(redacted);
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }

            if t.contains('=') {
                let (k, _) = t.split_once('=').unwrap();
                let kl = k.to_lowercase();
                if kl.contains("token")
                    || kl.contains("password")
                    || kl.contains("secret")
                    || kl.contains("apikey")
                    || kl.contains("api_key")
                {
                    out.push(format!("{k}=***"));
                    i += 1;
                    continue;
                }
            }

            out.push(redact_sensitive(t));
            i += 1;
        }
        out.join(" ")
    };

    // Start span for command execution
    let policy_decision;
    let mut span = if std::env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled" {
        // Policy evaluation (Phase 4)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Detect and load .substrate-profile if present
        let _ = detect_profile(&cwd);

        let decision = evaluate(trimmed, cwd.to_str().unwrap_or("."), None);

        // Convert broker Decision to trace PolicyDecision
        policy_decision = match &decision {
            Ok(Decision::Allow) => Some(PolicyDecision {
                action: "allow".to_string(),
                reason: None,
                restrictions: None,
            }),
            Ok(Decision::AllowWithRestrictions(restrictions)) => {
                eprintln!(
                    "substrate: command requires restrictions: {:?}",
                    restrictions
                );
                // Convert Restriction objects to strings for trace
                let restriction_strings: Vec<String> = restrictions
                    .iter()
                    .map(|r| format!("{:?}:{}", r.type_, r.value))
                    .collect();
                Some(PolicyDecision {
                    action: "allow_with_restrictions".to_string(),
                    reason: None,
                    restrictions: Some(restriction_strings),
                })
            }
            Ok(Decision::Deny(reason)) => {
                eprintln!("substrate: command denied by policy: {}", reason);
                Some(PolicyDecision {
                    action: "deny".to_string(),
                    reason: Some(reason.clone()),
                    restrictions: None,
                })
            }
            Err(e) => {
                eprintln!("substrate: policy check failed: {}", e);
                None
            }
        };

        // Handle denial
        if let Ok(Decision::Deny(_)) = decision {
            // Return failure status
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                return Ok(ExitStatus::from_raw(126 << 8)); // Cannot execute
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::ExitStatusExt;
                return Ok(ExitStatus::from_raw(126));
            }
        }

        // Create span with policy decision
        if let Ok(mut builder) = create_span_builder() {
            builder = builder
                .with_command(&redacted_for_logging)
                .with_cwd(cwd.to_str().unwrap_or("."));

            if let Some(pd) = policy_decision.clone() {
                builder = builder.with_policy_decision(pd);
            }

            // Set parent span ID in environment for child processes
            match builder.start() {
                Ok(span) => {
                    std::env::set_var("SHIM_PARENT_SPAN", span.get_span_id());
                    Some(span)
                }
                Err(e) => {
                    eprintln!("substrate: failed to create span: {}", e);
                    None
                }
            }
        } else {
            eprintln!("substrate: failed to create span builder");
            None
        }
    } else {
        None
    };

    // Check if PTY should be used (force overrides disable)
    let disabled = is_pty_disabled();
    let forced = is_force_pty_command(trimmed);
    let should_use_pty = forced || (!disabled && needs_pty(trimmed));

    if should_use_pty {
        // Attempt world-agent PTY WS route on Linux when world is enabled or agent socket exists
        #[cfg(target_os = "linux")]
        {
            let world_enabled = std::env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled";
            let uds_exists = std::path::Path::new("/run/substrate.sock").exists();
            if world_enabled || uds_exists {
                // Use span id if we have a span, otherwise fall back to cmd_id as a correlation hint
                if let Some(active_span) = span.as_mut() {
                    active_span.set_transport(TransportMeta {
                        mode: "unix".to_string(),
                        endpoint: Some("/run/substrate.sock".to_string()),
                    });
                }
                let span_id_for_ws = span
                    .as_ref()
                    .map(|s| s.get_span_id().to_string())
                    .unwrap_or_else(|| cmd_id.to_string());
                match execute_world_pty_over_ws(trimmed, &span_id_for_ws) {
                    Ok(code) => {
                        if let Some(active_span) = span.take() {
                            let (scopes_used, fs_diff) =
                                collect_world_telemetry(active_span.get_span_id());
                            let _ = active_span.finish(code, scopes_used, fs_diff);
                        }
                        #[cfg(unix)]
                        {
                            use std::os::unix::process::ExitStatusExt;
                            return Ok(ExitStatus::from_raw((code & 0xff) << 8));
                        }
                        #[cfg(windows)]
                        {
                            use std::os::windows::process::ExitStatusExt;
                            return Ok(ExitStatus::from_raw(code as u32));
                        }
                    }
                    Err(e) => {
                        eprintln!("substrate: warn: world PTY over WS failed, falling back to host PTY: {}", e);
                        // fall through to host PTY
                    }
                }
            }
        }

        // Attempt world-agent PTY WS route on mac when world is enabled
        #[cfg(target_os = "macos")]
        {
            let world_enabled = std::env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled";
            let uds_exists = pw::get_context()
                .map(|c| match &c.transport {
                    pw::WorldTransport::Unix(p) => p.exists(),
                    _ => false,
                })
                .unwrap_or(false);
            if world_enabled || uds_exists {
                let span_id_for_ws = span
                    .as_ref()
                    .map(|s| s.get_span_id().to_string())
                    .unwrap_or_else(|| cmd_id.to_string());
                match execute_world_pty_over_ws_macos(trimmed, &span_id_for_ws) {
                    Ok(code) => {
                        if let Some(active_span) = span.take() {
                            let (scopes_used, fs_diff) =
                                collect_world_telemetry(active_span.get_span_id());
                            let _ = active_span.finish(code, scopes_used, fs_diff);
                        }
                        #[cfg(unix)]
                        {
                            use std::os::unix::process::ExitStatusExt;
                            return Ok(ExitStatus::from_raw((code & 0xff) << 8));
                        }
                        #[cfg(windows)]
                        {
                            use std::os::windows::process::ExitStatusExt;
                            return Ok(ExitStatus::from_raw(code as u32));
                        }
                    }
                    Err(e) => {
                        static WARNED: std::sync::Once = std::sync::Once::new();
                        WARNED.call_once(|| {
                            eprintln!("substrate: warn: world PTY over WS failed on mac, falling back to host PTY: {}", e);
                        });
                        // fall through to host PTY
                    }
                }
            }
        }

        // Use host PTY execution path as fallback
        let pty_status = pty::execute_with_pty(config, trimmed, cmd_id, running_child_pid)?;

        // Finish span if we created one (PTY path)
        if let Some(active_span) = span {
            let exit_code = pty_status
                .code()
                .or_else(|| pty_status.success().then_some(0))
                .unwrap_or(-1);
            // Collect scopes and fs_diff from world backend if enabled
            let (scopes_used, fs_diff) =
                if env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled" {
                    collect_world_telemetry(active_span.get_span_id())
                } else {
                    (vec![], None)
                };
            let _ = active_span.finish(exit_code, scopes_used, fs_diff);
        }

        // Convert PtyExitStatus to std::process::ExitStatus for compatibility
        // NOTE: This is a documented compatibility shim using from_raw
        // The actual exit information is preserved in logs
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(signal) = pty_status.signal() {
                // Terminated by signal: set low 7 bits to the signal number
                // This makes status.signal() work correctly
                return Ok(ExitStatus::from_raw(signal & 0x7f));
            } else if let Some(code) = pty_status.code() {
                // Normal exit: code in bits 8-15
                return Ok(ExitStatus::from_raw((code & 0xff) << 8));
            } else {
                return Ok(ExitStatus::from_raw(0));
            }
        }

        #[cfg(windows)]
        {
            // Ensure Windows builds exercise signal() accessor even though it always returns None.
            let _ = pty_status.signal();
            // 🔥 EXPERT FIX: Don't shift bits on Windows - use raw code directly
            use std::os::windows::process::ExitStatusExt;
            let code = pty_status.code().unwrap_or(0) as u32;
            return Ok(ExitStatus::from_raw(code));
        }
    }

    // Continue with existing implementation for non-PTY commands...
    // Compute resolved path from raw command before redaction
    let resolved = first_command_path(trimmed);

    // Redact sensitive information before logging (token-aware)
    let redacted_command = redacted_for_logging.clone();

    // Log command start with redacted command and resolved path
    let start_extra = resolved.map(|p| json!({ "resolved_path": p }));
    log_command_event(
        config,
        "command_start",
        &redacted_command,
        cmd_id,
        start_extra,
    )?;
    let start_time = std::time::Instant::now();

    // Attempt to route non-PTY commands through the world agent when available
    let mut agent_result: Option<AgentStreamOutcome> = None;

    #[cfg(target_os = "macos")]
    {
        let world_enabled = std::env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled";
        let uds_exists = pw::get_context()
            .map(|c| matches!(&c.transport, pw::WorldTransport::Unix(path) if path.exists()))
            .unwrap_or(false);
        if world_enabled || uds_exists {
            if let Some(active_span) = span.as_mut() {
                if let Some(ctx) = pw::get_context() {
                    active_span.set_transport(world_transport_to_meta(&ctx.transport));
                }
            }
            let mut agent_command = trimmed.to_string();
            if config.ci_mode && !config.no_exit_on_error {
                let shell_name = Path::new(&config.shell_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if shell_name == "bash" || shell_name == "bash.exe" {
                    agent_command = format!("set -euo pipefail; {trimmed}");
                }
            }
            match stream_non_pty_via_agent(&agent_command) {
                Ok(outcome) => agent_result = Some(outcome),
                Err(e) => {
                    static WARN_ONCE: std::sync::Once = std::sync::Once::new();
                    WARN_ONCE.call_once(|| {
                        eprintln!(
                            "substrate: warn: mac world-agent exec failed, running direct: {}",
                            e
                        );
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let world_env = std::env::var("SUBSTRATE_WORLD").unwrap_or_default();
        let context = pw::get_context();
        if (world_env == "enabled" || context.is_some()) && !config.no_world {
            if let Some(active_span) = span.as_mut() {
                if let Some(ctx) = context.clone() {
                    active_span.set_transport(world_transport_to_meta(&ctx.transport));
                }
            }
            let mut agent_command = trimmed.to_string();
            if config.ci_mode && !config.no_exit_on_error {
                let shell_name = Path::new(&config.shell_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if shell_name == "bash" || shell_name == "bash.exe" {
                    agent_command = format!("set -euo pipefail; {trimmed}");
                }
            }
            match stream_non_pty_via_agent(&agent_command) {
                Ok(outcome) => agent_result = Some(outcome),
                Err(e) => {
                    static WARN_ONCE: std::sync::Once = std::sync::Once::new();
                    WARN_ONCE.call_once(|| {
                        eprintln!(
                            "substrate: warn: windows world-agent exec failed, running direct: {}",
                            e
                        );
                    });
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let world_env = env::var("SUBSTRATE_WORLD").unwrap_or_default();
        let world_enabled = world_env == "enabled";
        let world_disabled = world_env == "disabled" || config.no_world;
        let uds_exists = std::path::Path::new("/run/substrate.sock").exists();
        if world_enabled || (!world_disabled && uds_exists) {
            if let Some(active_span) = span.as_mut() {
                active_span.set_transport(TransportMeta {
                    mode: "unix".to_string(),
                    endpoint: Some("/run/substrate.sock".to_string()),
                });
            }
            match stream_non_pty_via_agent(trimmed) {
                Ok(outcome) => agent_result = Some(outcome),
                Err(e) => {
                    eprintln!(
                        "substrate: warn: shell world-agent exec failed, running direct: {}",
                        e
                    );
                }
            }
        }
    }

    if let Some(outcome) = agent_result {
        if let Some(active_span) = span {
            let _ = active_span.finish(
                outcome.exit_code,
                outcome.scopes_used.clone(),
                outcome.fs_diff.clone(),
            );
        }
        let completion_extra = json!({
            log_schema::EXIT_CODE: outcome.exit_code,
            log_schema::DURATION_MS: start_time.elapsed().as_millis()
        });
        log_command_event(
            config,
            "command_complete",
            &redacted_command,
            cmd_id,
            Some(completion_extra),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            return Ok(ExitStatus::from_raw((outcome.exit_code & 0xff) << 8));
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            return Ok(ExitStatus::from_raw(outcome.exit_code as u32));
        }
    }

    // Check for built-in commands only in interactive mode or for simple commands
    // Complex commands with shell operators must be handled by the external shell
    let status = if !needs_shell(trimmed) {
        if let Some(status) = handle_builtin(config, trimmed, cmd_id)? {
            status
        } else {
            execute_external(config, trimmed, running_child_pid, cmd_id)?
        }
    } else {
        // Execute external command through shell for complex commands
        execute_external(config, trimmed, running_child_pid, cmd_id)?
    };

    // Log command completion with redacted command
    let duration = start_time.elapsed();
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
        &redacted_command,
        cmd_id,
        Some(extra),
    )?;

    // Finish span if we created one
    if let Some(active_span) = span {
        let exit_code = status.code().unwrap_or(-1);
        // Collect scopes and fs_diff from world backend if enabled
        let (scopes_used, fs_diff) = if env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled"
        {
            collect_world_telemetry(active_span.get_span_id())
        } else {
            (vec![], None)
        };
        let _ = active_span.finish(exit_code, scopes_used, fs_diff);
    }

    Ok(status)
}

#[cfg(target_os = "linux")]
fn execute_world_pty_over_ws(cmd: &str, span_id: &str) -> anyhow::Result<i32> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use futures::{SinkExt, StreamExt};

    // Ensure agent is ready
    ensure_world_agent_ready()?;

    // Connect UDS and do WS handshake
    let rt = tokio::runtime::Runtime::new()?;
    let code = rt.block_on(async move {
        let stream = UnixStream::connect("/run/substrate.sock")
            .await
            .map_err(|e| anyhow::anyhow!("connect UDS: {}", e))?;
        let url = url::Url::parse("ws://localhost/v1/stream").unwrap();
        let (ws, _resp) = tungs::client_async(url, stream)
            .await
            .map_err(|e| anyhow::anyhow!("ws handshake: {}", e))?;
        let (sink, mut stream) = ws.split();
        let sink = std::sync::Arc::new(tokio::sync::Mutex::new(sink));

        if std::env::var("SUBSTRATE_WS_DEBUG").ok().as_deref() == Some("1") {
            eprintln!("using world-agent PTY WS");
        }

        // Prepare start frame (strip optional ":pty " prefix used in REPL to force PTY)
        let cmd_sanitized = if let Some(rest) = cmd.strip_prefix(":pty ") {
            rest
        } else {
            cmd
        };
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let env_map: std::collections::HashMap<String, String> = std::env::vars().collect();
        #[cfg(target_os = "linux")]
        let (cols, rows) = get_term_size();
        #[cfg(not(target_os = "linux"))]
        let (cols, rows) = (80u16, 24u16);
        let start = serde_json::json!({
            "type": "start",
            "cmd": cmd_sanitized,
            "cwd": cwd,
            "env": env_map,
            "span_id": span_id,
            "cols": cols,
            "rows": rows,
        });
        sink.lock()
            .await
            .send(tungs::tungstenite::Message::Text(start.to_string()))
            .await
            .map_err(|e| anyhow::anyhow!("ws send start: {}", e))?;

        // Enter raw mode on the local terminal and ensure restoration
        #[cfg(target_os = "linux")]
        let _raw_guard = RawModeGuard::for_stdin_if_tty()?;

        // Spawn stdin forwarder (raw bytes)
        let sink_in = sink.clone();
        let stdin_task = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut stdin = tokio::io::stdin();
            let mut buf = [0u8; 8192];
            loop {
                match stdin.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let b64 = STANDARD.encode(&buf[..n]);
                        let frame = serde_json::json!({"type":"stdin", "data_b64": b64});
                        if sink_in
                            .lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Spawn resize watcher (SIGWINCH)
        #[cfg(target_os = "linux")]
        let resize_task = {
            let sink_resize = sink.clone();
            let mut sig = signal(SignalKind::window_change())
                .map_err(|e| anyhow::anyhow!("sigwinch subscribe: {}", e))?;
            tokio::spawn(async move {
                while sig.recv().await.is_some() {
                    let (c, r) = get_term_size();
                    let frame = serde_json::json!({"type":"resize", "cols": c, "rows": r});
                    if sink_resize
                        .lock()
                        .await
                        .send(tungs::tungstenite::Message::Text(frame.to_string()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            })
        };

        // Spawn Unix signal forwarders (INT, TERM, HUP, QUIT) → WS Signal frames
        #[cfg(target_os = "linux")]
        let signal_tasks = {
            let mut tasks = Vec::new();

            // SIGINT
            if let Ok(mut sig) = signal(SignalKind::interrupt()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "INT"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGTERM
            if let Ok(mut sig) = signal(SignalKind::terminate()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "TERM"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGHUP
            if let Ok(mut sig) = signal(SignalKind::hangup()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "HUP"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGQUIT
            if let Ok(mut sig) = signal(SignalKind::quit()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "QUIT"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }

            tasks
        };

        let mut exit_code: i32 = 0;
        while let Some(msg) = stream.next().await {
            let msg = msg.map_err(|e| anyhow::anyhow!("ws recv: {}", e))?;
            if msg.is_text() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    match v.get("type").and_then(|t| t.as_str()) {
                        Some("stdout") => {
                            if let Some(b64) = v.get("data_b64").and_then(|x| x.as_str()) {
                                if let Ok(bytes) = STANDARD.decode(b64) {
                                    use std::io::Write;
                                    let _ = std::io::stdout().write_all(&bytes);
                                    let _ = std::io::stdout().flush();
                                }
                            }
                        }
                        Some("exit") => {
                            exit_code = v.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                            break;
                        }
                        Some("error") => {
                            if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                                eprintln!("world-agent error: {}", msg);
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            } else if msg.is_close() {
                break;
            }
        }

        // Cleanup background tasks
        stdin_task.abort();
        #[cfg(target_os = "linux")]
        {
            resize_task.abort();
            for t in signal_tasks {
                t.abort();
            }
        }
        Ok::<i32, anyhow::Error>(exit_code)
    })?;
    Ok(code)
}

#[cfg(target_os = "linux")]
fn ensure_world_agent_ready() -> anyhow::Result<()> {
    use std::path::Path;
    const SOCK: &str = "/run/substrate.sock";

    // Helper: quick readiness probe via HTTP-over-UDS
    fn probe_caps() -> bool {
        use std::io::{Read, Write};
        match std::os::unix::net::UnixStream::connect(SOCK) {
            Ok(mut s) => {
                let _ = s.set_read_timeout(Some(std::time::Duration::from_millis(150)));
                let _ = s.set_write_timeout(Some(std::time::Duration::from_millis(150)));
                let req = b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
                if s.write_all(req).is_ok() {
                    let mut buf = [0u8; 512];
                    if let Ok(n) = s.read(&mut buf) {
                        return n > 0
                            && std::str::from_utf8(&buf[..n])
                                .unwrap_or("")
                                .contains(" 200 ");
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    // Fast path: already ready
    if probe_caps() {
        return Ok(());
    }

    // Clean up stale socket if present (no responding server)
    if Path::new(SOCK).exists() {
        let _ = std::fs::remove_file(SOCK);
    }

    // Try to spawn agent
    let candidate_bins = [
        std::env::var("SUBSTRATE_WORLD_AGENT_BIN").ok(),
        which::which("substrate-world-agent")
            .ok()
            .map(|p| p.display().to_string()),
        Some("target/debug/world-agent".to_string()),
    ];
    let bin = candidate_bins
        .into_iter()
        .flatten()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or_else(|| anyhow::anyhow!("world-agent binary not found"))?;

    std::process::Command::new(&bin)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("spawn world-agent: {}", e))?;

    // Wait up to ~1s for readiness
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1000);
    while std::time::Instant::now() < deadline {
        if probe_caps() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    anyhow::bail!("world-agent readiness probe failed")
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
enum LinuxWorldInit {
    Disabled,
    Agent,
    LocalBackend,
    LocalBackendFailed,
}

#[cfg(target_os = "linux")]
fn init_linux_world(world_disabled: bool) -> LinuxWorldInit {
    init_linux_world_with_probe(world_disabled, ensure_world_agent_ready)
}

#[cfg(target_os = "linux")]
fn init_linux_world_with_probe<F>(world_disabled: bool, agent_probe: F) -> LinuxWorldInit
where
    F: Fn() -> anyhow::Result<()>,
{
    use world::LinuxLocalBackend;
    use world_api::{ResourceLimits, WorldBackend, WorldSpec};

    if world_disabled {
        return LinuxWorldInit::Disabled;
    }

    match agent_probe() {
        Ok(()) => {
            env::set_var("SUBSTRATE_WORLD", "enabled");
            env::remove_var("SUBSTRATE_WORLD_ID");
            LinuxWorldInit::Agent
        }
        Err(agent_err) => {
            #[cfg(test)]
            if let Ok(mock_id) = env::var("SUBSTRATE_TEST_LOCAL_WORLD_ID") {
                env::set_var("SUBSTRATE_WORLD", "enabled");
                env::set_var("SUBSTRATE_WORLD_ID", mock_id);
                return LinuxWorldInit::LocalBackend;
            }

            let spec = WorldSpec {
                reuse_session: true,
                isolate_network: true,
                limits: ResourceLimits::default(),
                enable_preload: false,
                allowed_domains: substrate_broker::allowed_domains(),
                project_dir: crate::execution::settings::world_root_from_env().path,
                always_isolate: false,
            };
            let backend = LinuxLocalBackend::new();
            match backend.ensure_session(&spec) {
                Ok(handle) => {
                    env::set_var("SUBSTRATE_WORLD", "enabled");
                    env::set_var("SUBSTRATE_WORLD_ID", &handle.id);
                    LinuxWorldInit::LocalBackend
                }
                Err(local_err) => {
                    eprintln!(
                        "substrate: linux world fallback failed (agent error: {agent_err:#}; local error: {local_err:#})"
                    );
                    LinuxWorldInit::LocalBackendFailed
                }
            }
        }
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

#[cfg(target_os = "macos")]
fn execute_world_pty_over_ws_macos(cmd: &str, span_id: &str) -> anyhow::Result<i32> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use futures::StreamExt;
    use tungs::tungstenite::Message;

    let ctx = pw::get_context().ok_or_else(|| anyhow::anyhow!("no platform world context"))?;
    let rt = tokio::runtime::Runtime::new()?;
    let code = rt.block_on(async move {
        async fn handle_ws<S>(
            ws: tungs::WebSocketStream<S>,
            cmd: &str,
            span_id: &str,
        ) -> anyhow::Result<i32>
        where
            S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
        {
            use futures::SinkExt;
            let (mut sink, mut stream) = ws.split();

            let cmd_sanitized = if let Some(rest) = cmd.strip_prefix(":pty ") {
                rest
            } else {
                cmd
            };
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let env_map: std::collections::HashMap<String, String> = std::env::vars().collect();
            let (cols, rows) = (80u16, 24u16);
            let start = serde_json::json!({
                "type": "start",
                "cmd": cmd_sanitized,
                "cwd": cwd,
                "env": env_map,
                "span_id": span_id,
                "cols": cols,
                "rows": rows,
            });
            sink.send(Message::Text(start.to_string()))
                .await
                .map_err(|e| anyhow::anyhow!("ws send start: {}", e))?;

            // stdin forwarder
            let mut stdin = tokio::io::stdin();
            let stdin_task = tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                let mut buf = [0u8; 8192];
                loop {
                    match stdin.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&buf[..n]);
                            let frame = serde_json::json!({"type":"stdin", "data_b64": b64});
                            if sink.send(Message::Text(frame.to_string())).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            let mut exit_code: i32 = 0;
            while let Some(msg) = stream.next().await {
                let msg = msg.map_err(|e| anyhow::anyhow!("ws recv: {}", e))?;
                if msg.is_text() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                        match v.get("type").and_then(|t| t.as_str()) {
                            Some("stdout") => {
                                if let Some(b64) = v.get("data_b64").and_then(|x| x.as_str()) {
                                    if let Ok(bytes) = STANDARD.decode(b64) {
                                        use std::io::Write;
                                        let _ = std::io::stdout().write_all(&bytes);
                                        let _ = std::io::stdout().flush();
                                    }
                                }
                            }
                            Some("exit") => {
                                exit_code =
                                    v.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                                break;
                            }
                            Some("error") => {
                                if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                                    eprintln!("world-agent error: {}", msg);
                                }
                                break;
                            }
                            _ => {}
                        }
                    }
                } else if msg.is_close() {
                    break;
                }
            }

            stdin_task.abort();
            Ok::<i32, anyhow::Error>(exit_code)
        }

        // Connect according to transport and delegate to generic handler
        let url = url::Url::parse("ws://localhost/v1/stream").unwrap();
        match &ctx.transport {
            pw::WorldTransport::Unix(path) => {
                let stream = UnixStream::connect(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("connect UDS: {}", e))?;
                let (ws, _resp) = tungs::client_async(url, stream)
                    .await
                    .map_err(|e| anyhow::anyhow!("ws handshake: {}", e))?;
                handle_ws(ws, cmd, span_id).await
            }
            pw::WorldTransport::Tcp { host, port } => {
                let ws_url = format!("ws://{}:{}/v1/stream", host, port);
                let (ws, _resp) = tungs::connect_async(&ws_url)
                    .await
                    .map_err(|e| anyhow::anyhow!("ws connect: {}", e))?;
                handle_ws(ws, cmd, span_id).await
            }
            pw::WorldTransport::Vsock { port } => {
                let ws_url = format!("ws://127.0.0.1:{}/v1/stream", port);
                let (ws, _resp) = tungs::connect_async(&ws_url)
                    .await
                    .map_err(|e| anyhow::anyhow!("ws connect: {}", e))?;
                handle_ws(ws, cmd, span_id).await
            }
        }
    })?;

    Ok(code)
}

pub(crate) struct AgentStreamOutcome {
    pub(crate) exit_code: i32,
    pub(crate) scopes_used: Vec<String>,
    pub(crate) fs_diff: Option<substrate_common::FsDiff>,
}

pub(crate) fn stream_non_pty_via_agent(command: &str) -> anyhow::Result<AgentStreamOutcome> {
    let (client, request, agent_id) = build_agent_client_and_request(command)?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        use agent_api_types::ApiError;
        use http_body_util::BodyExt;

        let response = client.execute_stream(request).await?;
        if !response.status().is_success() {
            let status = response.status();
            let body_bytes = response
                .into_body()
                .collect()
                .await
                .map_err(|e| anyhow::anyhow!("stream read failed: {}", e))?
                .to_bytes();
            if let Ok(api_error) = serde_json::from_slice::<ApiError>(&body_bytes) {
                anyhow::bail!("API error: {}", api_error);
            }
            let text = String::from_utf8_lossy(&body_bytes);
            anyhow::bail!("HTTP {} error: {}", status, text);
        }

        process_agent_stream(response.into_body(), agent_id).await
    })
}

async fn process_agent_stream(
    mut body: hyper::body::Incoming,
    agent_label: String,
) -> anyhow::Result<AgentStreamOutcome> {
    use http_body_util::BodyExt;

    let mut buffer = Vec::new();
    let mut exit_code = None;
    let mut scopes_used = Vec::new();
    let mut fs_diff = None;

    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|e| anyhow::anyhow!("stream frame error: {}", e))?;
        if let Some(data) = frame.data_ref() {
            buffer.extend_from_slice(data);
            consume_agent_stream_buffer(
                &agent_label,
                &mut buffer,
                &mut exit_code,
                &mut scopes_used,
                &mut fs_diff,
            )?;
        }
    }

    if !buffer.is_empty() {
        consume_agent_stream_buffer(
            &agent_label,
            &mut buffer,
            &mut exit_code,
            &mut scopes_used,
            &mut fs_diff,
        )?;
    }

    let exit_code =
        exit_code.ok_or_else(|| anyhow::anyhow!("agent stream completed without exit frame"))?;

    Ok(AgentStreamOutcome {
        exit_code,
        scopes_used,
        fs_diff,
    })
}

fn consume_agent_stream_buffer(
    agent_label: &str,
    buffer: &mut Vec<u8>,
    exit_code: &mut Option<i32>,
    scopes_used: &mut Vec<String>,
    fs_diff: &mut Option<substrate_common::FsDiff>,
) -> anyhow::Result<()> {
    use anyhow::Context as _;

    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
        let line: Vec<u8> = buffer.drain(..=pos).collect();
        if line.len() <= 1 {
            continue;
        }
        let payload = &line[..line.len() - 1];
        if payload.is_empty() {
            continue;
        }

        let frame: ExecuteStreamFrame = serde_json::from_slice(payload).with_context(|| {
            format!(
                "invalid agent stream frame: {}",
                String::from_utf8_lossy(payload)
            )
        })?;

        match frame {
            ExecuteStreamFrame::Start { .. } => {}
            ExecuteStreamFrame::Stdout { chunk_b64 } => {
                let bytes = BASE64
                    .decode(chunk_b64.as_bytes())
                    .map_err(|e| anyhow::anyhow!("invalid stdout chunk: {}", e))?;
                emit_stream_chunk(agent_label, &bytes, false);
            }
            ExecuteStreamFrame::Stderr { chunk_b64 } => {
                let bytes = BASE64
                    .decode(chunk_b64.as_bytes())
                    .map_err(|e| anyhow::anyhow!("invalid stderr chunk: {}", e))?;
                emit_stream_chunk(agent_label, &bytes, true);
            }
            ExecuteStreamFrame::Event { event } => {
                let _ = publish_agent_event(event);
            }
            ExecuteStreamFrame::Exit {
                exit,
                scopes_used: scopes,
                fs_diff: diff,
                ..
            } => {
                *exit_code = Some(exit);
                *scopes_used = scopes;
                *fs_diff = diff;
            }
            ExecuteStreamFrame::Error { message } => {
                eprintln!("world-agent error: {}", message);
                anyhow::bail!(message);
            }
        }
    }

    Ok(())
}

fn emit_stream_chunk(agent_label: &str, data: &[u8], is_stderr: bool) {
    use std::io::Write;

    if is_stderr {
        let mut stderr = io::stderr();
        let _ = stderr.write_all(data);
        let _ = stderr.flush();
    } else {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(data);
        let _ = stdout.flush();
    }

    let text = String::from_utf8_lossy(data);
    let _ = publish_agent_event(AgentEvent::stream_chunk(
        agent_label,
        is_stderr,
        text.to_string(),
    ));
}

pub(crate) fn parse_demo_burst_command(input: &str) -> Option<(usize, usize, u64)> {
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

pub(crate) fn build_agent_client_and_request(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    build_agent_client_and_request_impl(cmd)
}

#[cfg(target_os = "linux")]
fn build_agent_client_and_request_impl(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    use agent_api_types::ExecuteRequest;

    ensure_world_agent_ready()?;

    let client = AgentClient::unix_socket("/run/substrate.sock")?;
    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .display()
        .to_string();
    let env_map = build_world_env_map();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());

    let request = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd),
        env: Some(env_map),
        pty: false,
        agent_id: agent_id.clone(),
        budget: None,
    };

    Ok((client, request, agent_id))
}

fn build_world_env_map() -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;

    let mut env_map: HashMap<String, String> = std::env::vars().collect();

    if let Ok(original_path) = std::env::var("SHIM_ORIGINAL_PATH") {
        env_map.insert("PATH".to_string(), original_path.clone());
        #[cfg(windows)]
        {
            env_map.insert("Path".to_string(), original_path);
        }
    } else if let Ok(shim_dir) = substrate_paths::shims_dir() {
        if let Some(current_path) = env_map.get("PATH").cloned() {
            let separator = if cfg!(windows) { ';' } else { ':' };
            let shim_str = shim_dir.to_string_lossy();
            let filtered: String = current_path
                .split(separator)
                .filter(|segment| segment != &shim_str)
                .filter(|segment| !segment.is_empty())
                .collect::<Vec<&str>>()
                .join(&separator.to_string());
            env_map.insert("PATH".to_string(), filtered);
        }
    }

    for key in [
        "SHIM_ACTIVE",
        "SHIM_CALLER",
        "SHIM_CALL_STACK",
        "SHIM_DEPTH",
    ] {
        env_map.remove(key);
    }

    env_map
}

#[cfg(target_os = "macos")]
fn build_agent_client_and_request_impl(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    use agent_api_types::ExecuteRequest;

    let ctx = pw::get_context().ok_or_else(|| anyhow::anyhow!("no platform world context"))?;
    (ctx.ensure_ready.as_ref())()?;

    let client = match &ctx.transport {
        pw::WorldTransport::Unix(path) => AgentClient::unix_socket(path),
        pw::WorldTransport::Tcp { host, port } => AgentClient::tcp(host, *port),
        pw::WorldTransport::Vsock { port } => AgentClient::tcp("127.0.0.1", *port),
    }?;

    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .display()
        .to_string();
    let env_map = build_world_env_map();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());

    let request = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd),
        env: Some(env_map),
        pty: false,
        agent_id: agent_id.clone(),
        budget: None,
    };

    Ok((client, request, agent_id))
}

#[cfg(target_os = "windows")]
fn build_agent_client_and_request_impl(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    use crate::execution::platform_world::windows;
    use agent_api_types::ExecuteRequest;

    let backend = windows::get_backend()?;
    let handle = backend.ensure_session(&windows::world_spec())?;
    std::env::set_var("SUBSTRATE_WORLD", "enabled");
    std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);

    let client = windows::build_agent_client()?;
    let cwd = windows::current_dir_wsl()?;
    let env_map = build_world_env_map();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());

    let request = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd),
        env: Some(env_map),
        pty: false,
        agent_id: agent_id.clone(),
        budget: None,
    };

    Ok((client, request, agent_id))
}

fn execute_external(
    config: &ShellConfig,
    command: &str,
    running_child_pid: Arc<AtomicI32>,
    cmd_id: &str,
) -> Result<ExitStatus> {
    let shell = &config.shell_path;
    let mut command = command.to_string();

    // Verify shell exists
    if which::which(shell).is_err() && !Path::new(shell).exists() {
        return Err(anyhow::anyhow!("Shell not found: {}", shell));
    }

    let mut cmd = Command::new(shell);

    // Shell-specific command execution
    let shell_name = Path::new(shell)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let is_pwsh = shell_name == "pwsh.exe" || shell_name == "pwsh";
    let is_powershell = shell_name == "powershell.exe" || shell_name == "powershell";
    let is_cmd = shell_name == "cmd.exe" || shell_name == "cmd";
    let is_bash = shell_name == "bash" || shell_name == "bash.exe";
    let should_guard_anchor = config.world_root.caged
        && config.world_root.mode != WorldRootMode::FollowCwd
        && !cfg!(windows)
        && needs_shell(&command);

    if should_guard_anchor {
        command = wrap_with_anchor_guard(&command, config);
    }
    let mut command_for_shell = command.clone();

    if is_pwsh || is_powershell {
        // PowerShell
        if config.ci_mode && !config.no_exit_on_error {
            command_for_shell = format!("$ErrorActionPreference='Stop'; {command_for_shell}");
            cmd.arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg(command_for_shell);
        } else {
            cmd.arg("-NoProfile").arg("-Command").arg(command_for_shell);
        }
    } else if is_cmd {
        // Windows CMD
        cmd.arg("/C").arg(command_for_shell);
    } else {
        // Unix shells (bash, sh, zsh, etc.)
        if config.ci_mode && !config.no_exit_on_error && is_bash {
            command_for_shell = format!("set -euo pipefail; {command_for_shell}");
        }
        cmd.arg("-c").arg(command_for_shell);
    }

    // Propagate environment
    cmd.env("SHIM_SESSION_ID", &config.session_id);
    cmd.env("SHIM_TRACE_LOG", &config.trace_log_file);
    cmd.env("SHIM_PARENT_CMD_ID", cmd_id); // Pass cmd_id for shim correlation
    cmd.env_remove("SHIM_ACTIVE"); // Clear to allow shims to work
    cmd.env_remove("SHIM_CALLER"); // Clear caller chain for fresh command
    cmd.env_remove("SHIM_CALL_STACK"); // Clear call stack for fresh command
                                       // Keep PATH as-is with shims - the env_remove("SHIM_ACTIVE") should be sufficient

    configure_child_shell_env(
        &mut cmd,
        config,
        is_bash,
        matches!(config.mode, ShellMode::Script(_)),
    );

    // Handle I/O based on mode - always inherit stdin for better compatibility and stream output ourselves
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

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

    // Spawn and track child PID for signal handling
    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to execute: {command}"))?;

    let stdout_thread = child
        .stdout
        .take()
        .map(|pipe| spawn_host_stream_thread(pipe, false, SHELL_AGENT_ID.to_string()));
    let stderr_thread = child
        .stderr
        .take()
        .map(|pipe| spawn_host_stream_thread(pipe, true, SHELL_AGENT_ID.to_string()));

    let child_pid = child.id() as i32;
    running_child_pid.store(child_pid, Ordering::SeqCst);

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait for command: {command}"))?;

    running_child_pid.store(0, Ordering::SeqCst);

    if let Some(handle) = stdout_thread {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("substrate: warn: stdout stream error: {}", e),
            Err(e) => eprintln!("substrate: warn: stdout stream thread panicked: {:?}", e),
        }
    }
    if let Some(handle) = stderr_thread {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("substrate: warn: stderr stream error: {}", e),
            Err(e) => eprintln!("substrate: warn: stderr stream thread panicked: {:?}", e),
        }
    }

    Ok(status)
}

fn wrap_with_anchor_guard(command: &str, config: &ShellConfig) -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let anchor = canonicalize_or(&config.world_root.anchor_root(&current_dir));
    let anchor_escaped = shell_escape_for_shell(&anchor);
    let mut guarded = format!(
        "__substrate_anchor_root={anchor}; substrate_anchor_cd() {{ command cd \"$@\" || return $?; dest=$(pwd -P); case \"$dest\" in \"$__substrate_anchor_root\"|\"$__substrate_anchor_root\"/*) ;; *) printf 'substrate: info: caged root guard: returning to %s\\n' \"$__substrate_anchor_root\" >&2; command cd \"$__substrate_anchor_root\" || return $?;; esac; unset dest; }}; cd() {{ substrate_anchor_cd \"$@\"; }}; substrate_anchor_cd .; ",
        anchor = anchor_escaped,
    );
    guarded.push_str(command);
    guarded
}

fn shell_escape_for_shell(path: &Path) -> String {
    let raw = path.to_string_lossy();
    if raw.contains('\'') {
        format!("'{}'", raw.replace('\'', "'\"'\"'"))
    } else {
        format!("'{raw}'")
    }
}

fn spawn_host_stream_thread<R>(
    mut reader: R,
    is_stderr: bool,
    agent_label: String,
) -> std::thread::JoinHandle<anyhow::Result<()>>
where
    R: std::io::Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match std::io::Read::read(&mut reader, &mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    emit_stream_chunk(&agent_label, &buf[..n], is_stderr);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(anyhow::anyhow!("pipe read failed: {}", e)),
            }
        }
        Ok(())
    })
}

fn first_command_path(cmd: &str) -> Option<String> {
    // Skip resolution unless SHIM_LOG_OPTS=resolve is set (performance optimization)
    if env::var("SHIM_LOG_OPTS").as_deref() != Ok("resolve") {
        return None;
    }

    // Use shell_words for proper tokenization, fall back to whitespace split
    let tokens = shell_words::split(cmd)
        .unwrap_or_else(|_| cmd.split_whitespace().map(|s| s.to_string()).collect());

    let first = tokens.first()?;
    let p = std::path::Path::new(first);
    if p.is_absolute() {
        return Some(first.to_string());
    }
    // Best effort PATH lookup
    which::which(first).ok().map(|pb| pb.display().to_string())
}

// Note: removed unused maybe_rotate_log helper to avoid dead_code warnings.

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
    #[cfg(unix)]
    use std::process::Command;
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
        let output = Command::new(&config.shell_path)
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
        let output = Command::new(&config.shell_path)
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
