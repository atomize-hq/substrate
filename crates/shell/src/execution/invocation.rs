use super::agent_events::{
    clear_agent_event_sender, format_event_line, init_event_channel, publish_command_completion,
    schedule_demo_burst, schedule_demo_events,
};
use super::cli::*;
use super::settings::{self, apply_world_root_env, resolve_world_root};
use super::shim_deploy::{DeploymentStatus, ShimDeployer};
use super::{
    configure_child_shell_env, execute_command, handle_graph_command, handle_health_command,
    handle_replay_command, handle_shim_command, handle_trace_command, handle_world_command,
    is_shell_stream_event, log_command_event, parse_demo_burst_command, setup_signal_handlers,
    update_world_env, ReplSessionTelemetry,
};
use crate::builtins as commands;
use crate::repl::editor;
use anyhow::{Context, Result};
use clap::Parser;
use reedline::Signal;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, IsTerminal};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use substrate_common::{dedupe_path, log_schema, paths as substrate_paths, WorldRootMode};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ShellMode {
    Interactive { use_pty: bool }, // Full REPL with optional PTY
    Wrap(String),                  // Single command execution (-c "cmd")
    Script(PathBuf),               // Script file execution (-f script.sh)
    Pipe,                          // Read commands from stdin
}

#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub mode: ShellMode,
    pub session_id: String,
    pub trace_log_file: PathBuf,
    pub original_path: String,
    pub shim_dir: PathBuf,
    pub shell_path: String,
    pub ci_mode: bool,
    pub no_exit_on_error: bool,
    pub skip_shims: bool,
    pub no_world: bool,
    pub world_root: settings::WorldRootSettings,
    pub async_repl: bool,
    pub env_vars: HashMap<String, String>,
    pub manager_init_path: PathBuf,
    pub manager_env_path: PathBuf,
    pub shimmed_path: Option<String>,
    pub host_bash_env: Option<String>,
    pub bash_preexec_path: PathBuf,
    pub preexec_available: bool,
}

impl ShellConfig {
    pub fn from_args() -> Result<Self> {
        Self::from_cli(Cli::parse())
    }

    pub fn from_cli(cli: Cli) -> Result<Self> {
        // macOS-only: apply CLI overrides to environment for platform detection precedence
        #[cfg(target_os = "macos")]
        {
            // No mac-only CLI transport overrides; maintain 1:1 parity with Linux
        }

        // Handle --version-json flag
        if cli.version_json {
            let version_info = json!({
                "version": env!("CARGO_PKG_VERSION"),
                "build": env::var("SHIM_BUILD").unwrap_or_else(|_| "unknown".to_string()),
                "rust_version": option_env!("SHIM_RUSTC_VERSION").unwrap_or("unknown"),
                "features": {
                    "pty": cfg!(unix),
                    "windows": cfg!(windows),
                },
                "platform": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
            });
            println!("{}", serde_json::to_string_pretty(&version_info)?);
            std::process::exit(0);
        }

        // Handle --shim-deploy flag
        if cli.shim_deploy {
            let deployer = ShimDeployer::with_skip(false)?;
            match deployer.ensure_deployed() {
                Ok(DeploymentStatus::Deployed) => {
                    println!("✓ Shims deployed successfully");
                    std::process::exit(0);
                }
                Ok(DeploymentStatus::Current) => {
                    println!("✓ Shims are already up to date");
                    std::process::exit(0);
                }
                Ok(DeploymentStatus::Failed(msg)) => {
                    eprintln!("✗ Shim deployment failed: {msg}");
                    std::process::exit(1);
                }
                Ok(DeploymentStatus::Skipped) => {
                    println!("Shim deployment was skipped");
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("✗ Error deploying shims: {e}");
                    std::process::exit(1);
                }
            }
        }

        // Handle --shim-remove flag
        if cli.shim_remove {
            let shims_dir = substrate_common::paths::shims_dir()?;
            if shims_dir.exists() {
                std::fs::remove_dir_all(&shims_dir)?;
                println!("✓ Removed shims from {shims_dir:?}");
            } else {
                println!("No shims found to remove");
            }
            std::process::exit(0);
        }

        // Handle --shim-status-json flag (CI)
        if cli.shim_status_json {
            if env::var("SUBSTRATE_NO_SHIMS").is_ok() {
                let out = json!({
                    "status": "disabled",
                    "deployed": false,
                    "version": serde_json::Value::Null,
                    "location": substrate_common::paths::shims_dir().ok(),
                    "commands_total": serde_json::Value::Null,
                    "commands_present": serde_json::Value::Null,
                    "missing": [],
                    "path_ok": serde_json::Value::Null,
                    "path_first": serde_json::Value::Null,
                    "exit": 0
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
                std::process::exit(0);
            }

            let shims_dir = substrate_common::paths::shims_dir()?;
            let version_file = substrate_common::paths::version_file()?;

            if !shims_dir.exists() {
                let out = json!({
                    "status": "not_deployed",
                    "deployed": false,
                    "version": serde_json::Value::Null,
                    "location": shims_dir,
                    "commands_total": 0,
                    "commands_present": 0,
                    "missing": [],
                    "path_ok": serde_json::Value::Null,
                    "path_first": env::var("PATH").ok().and_then(|p| p.split(if cfg!(windows){';'} else {':'}).next().map(|s| s.to_string())),
                    "exit": 1
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
                std::process::exit(1);
            }

            // PATH check
            let path_str = env::var("PATH").unwrap_or_default();
            let sep = if cfg!(windows) { ';' } else { ':' };
            let first_path = path_str.split(sep).next().unwrap_or("").to_string();
            let shims_dir_str = shims_dir.display().to_string();
            let path_ok = first_path == shims_dir_str;

            if let Ok(content) = std::fs::read_to_string(&version_file) {
                let info: serde_json::Value = serde_json::from_str(&content)?;
                let current_version = env!("CARGO_PKG_VERSION");
                let file_version = info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let mut total_cmds: usize = 0;
                let mut present_cmds: usize = 0;
                let mut missing_list: Vec<String> = Vec::new();
                let mut missing_any = false;
                if let Some(commands) = info.get("commands").and_then(|c| c.as_array()) {
                    let expected: Vec<String> = commands
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    total_cmds = expected.len();
                    for cmd in &expected {
                        let path = shims_dir.join(cmd);
                        if path.exists() {
                            present_cmds += 1;
                        } else {
                            missing_list.push(cmd.clone());
                        }
                    }
                    if !missing_list.is_empty() {
                        missing_any = true;
                    }
                }

                let mut exit_code = 0;
                let status_str = if missing_any {
                    exit_code = 1;
                    "needs_redeploy"
                } else if file_version != current_version {
                    exit_code = 1;
                    "update_available"
                } else {
                    "up_to_date"
                };

                let out = json!({
                    "status": status_str,
                    "deployed": true,
                    "version": file_version,
                    "deployed_at": info.get("deployed_at").cloned().unwrap_or(serde_json::Value::Null),
                    "location": shims_dir,
                    "commands_total": total_cmds,
                    "commands_present": present_cmds,
                    "missing": missing_list,
                    "path_ok": path_ok,
                    "path_first": first_path,
                    "exit": exit_code
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
                std::process::exit(exit_code);
            } else {
                let out = json!({
                    "status": "needs_redeploy",
                    "deployed": true,
                    "version": serde_json::Value::Null,
                    "deployed_at": serde_json::Value::Null,
                    "location": shims_dir,
                    "commands_total": serde_json::Value::Null,
                    "commands_present": serde_json::Value::Null,
                    "missing": [],
                    "path_ok": path_ok,
                    "path_first": first_path,
                    "exit": 1
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
                std::process::exit(1);
            }
        }

        // Handle --shim-status flag
        if cli.shim_status {
            // Respect explicit disable
            if env::var("SUBSTRATE_NO_SHIMS").is_ok() {
                println!("Shims: Deployment disabled (SUBSTRATE_NO_SHIMS=1)");
                println!("Status: Skipped");
                std::process::exit(0);
            }

            let shims_dir = substrate_common::paths::shims_dir()?;
            let version_file = substrate_common::paths::version_file()?;

            if !shims_dir.exists() {
                println!("Shims: Not deployed");
                println!("Suggestion: run `substrate` once or `substrate --shim-deploy`");
                std::process::exit(1);
            }

            // PATH check (warn only)
            let path_str = env::var("PATH").unwrap_or_default();
            let sep = if cfg!(windows) { ';' } else { ':' };
            let first_path = path_str.split(sep).next().unwrap_or("");
            let shims_dir_str = shims_dir.display().to_string();
            let path_ok = first_path == shims_dir_str;

            let mut exit_code = 0i32;
            let printed_header = false;

            if let Ok(content) = std::fs::read_to_string(&version_file) {
                let info: serde_json::Value = serde_json::from_str(&content)?;
                println!("Shims: Deployed");

                // Version
                let current_version = env!("CARGO_PKG_VERSION");
                let file_version = info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                println!("Version: {}", file_version);

                // Deployed at
                if let Some(deployed_at) = info.get("deployed_at") {
                    if let Some(secs) = deployed_at.get("secs_since_epoch").and_then(|s| s.as_i64())
                    {
                        use chrono::DateTime;
                        if let Some(datetime) = DateTime::from_timestamp(secs, 0) {
                            println!("Deployed: {}", datetime.format("%Y-%m-%d %H:%M:%S UTC"));
                        }
                    }
                }

                println!("Location: {}", shims_dir.display());

                // Commands presence check
                let mut missing_any = false;
                if let Some(commands) = info.get("commands").and_then(|c| c.as_array()) {
                    let expected: Vec<String> = commands
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    let mut missing = Vec::new();
                    for cmd in &expected {
                        let path = shims_dir.join(cmd);
                        if !path.exists() {
                            missing.push(cmd.clone());
                        }
                    }
                    if missing.is_empty() {
                        println!("Commands: {} shims", expected.len());
                    } else {
                        println!(
                            "Commands: {}/{} shims (missing: {})",
                            expected.len() - missing.len(),
                            expected.len(),
                            missing.join(", ")
                        );
                        exit_code = 1;
                        missing_any = true;
                    }
                }

                // PATH line
                if path_ok {
                    println!("PATH: OK ({} is first)", shims_dir.display());
                } else {
                    println!(
                        "PATH: WARN (shims dir is not first; current PATH begins with: {})",
                        first_path
                    );
                    #[cfg(unix)]
                    {
                        let uid = unsafe { libc::geteuid() };
                        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
                            .ok()
                            .map(PathBuf::from)
                            .filter(|p| p.is_dir())
                            .unwrap_or_else(|| PathBuf::from("/tmp"));
                        let marker = runtime_dir.join(format!("substrate_path_hint_shown_{}", uid));
                        if !marker.exists() {
                            println!("Suggestion: export PATH=\"{}:$PATH\"", shims_dir.display());
                            let _ = std::fs::write(&marker, b"1");
                        }
                    }
                }

                // Status line
                if missing_any {
                    println!("Status: Needs redeploy");
                } else if file_version != current_version {
                    println!("Status: Update available (current: {current_version})");
                    exit_code = 1;
                } else if exit_code == 0 {
                    println!("Status: Up to date");
                }

                std::process::exit(exit_code);
            } else {
                // Version file missing -> treat as needs redeploy
                if !printed_header {
                    println!("Shims: Deployed (version unknown)");
                }
                println!("Location: {}", shims_dir.display());
                if path_ok {
                    println!("PATH: OK ({} is first)", shims_dir.display());
                } else {
                    println!(
                        "PATH: WARN (shims dir is not first; current PATH begins with: {})",
                        first_path
                    );
                    #[cfg(unix)]
                    {
                        let uid = unsafe { libc::geteuid() };
                        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
                            .ok()
                            .map(PathBuf::from)
                            .filter(|p| p.is_dir())
                            .unwrap_or_else(|| PathBuf::from("/tmp"));
                        let marker = runtime_dir.join(format!("substrate_path_hint_shown_{}", uid));
                        if !marker.exists() {
                            println!("Suggestion: export PATH=\"{}:$PATH\"", shims_dir.display());
                            let _ = std::fs::write(&marker, b"1");
                        }
                    }
                }
                println!("Status: Needs redeploy (version file missing)");
                std::process::exit(1);
            }
        }

        // Handle --trace flag
        if let Some(span_id) = cli.trace {
            handle_trace_command(&span_id)?;
            std::process::exit(0);
        }

        // Handle --replay flag
        if let Some(span_id) = cli.replay {
            if cli.replay_verbose {
                env::set_var("SUBSTRATE_REPLAY_VERBOSE", "1");
            }
            handle_replay_command(&span_id)?;
            std::process::exit(0);
        }

        // Handle subcommands
        if let Some(sub) = &cli.sub {
            match sub {
                SubCommands::Graph(graph_cmd) => {
                    handle_graph_command(graph_cmd)?;
                    std::process::exit(0);
                }
                SubCommands::World(world_cmd) => {
                    handle_world_command(world_cmd, &cli)?;
                    std::process::exit(0);
                }
                SubCommands::Shim(shim_cmd) => {
                    handle_shim_command(shim_cmd, &cli);
                }
                SubCommands::Health(health_cmd) => {
                    handle_health_command(health_cmd, &cli)?;
                    std::process::exit(0);
                }
            }
        }

        let launch_cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let cli_caged = if cli.caged {
            Some(true)
        } else if cli.uncaged {
            Some(false)
        } else {
            None
        };
        let world_root_settings = resolve_world_root(
            cli.world_root_mode.map(WorldRootMode::from),
            cli.world_root_path.clone(),
            cli_caged,
            &launch_cwd,
        )?;
        apply_world_root_env(&world_root_settings);

        let session_id = env::var("SHIM_SESSION_ID").unwrap_or_else(|_| Uuid::now_v7().to_string());

        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE")) // Windows support
            .context("HOME/USERPROFILE not set")?;

        let trace_log_file = env::var("SHIM_TRACE_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&home).join(".substrate/trace.jsonl"));

        let original_path = env::var("SHIM_ORIGINAL_PATH")
            .or_else(|_| env::var("PATH"))
            .context("No PATH found")?;

        let shim_dir = substrate_common::paths::shims_dir()?;
        let substrate_home = substrate_paths::substrate_home()?;
        let install_config =
            commands::world_enable::load_install_config(&substrate_paths::config_file()?)?;
        let config_disables_world = !install_config.world_enabled;
        let env_disables_world = env::var("SUBSTRATE_WORLD")
            .map(|value| value.eq_ignore_ascii_case("disabled"))
            .unwrap_or(false)
            || env::var("SUBSTRATE_WORLD_ENABLED")
                .map(|value| value == "0")
                .unwrap_or(false);
        let final_no_world = if cli.world {
            false
        } else if cli.no_world {
            true
        } else {
            config_disables_world || env_disables_world
        };
        update_world_env(final_no_world);
        let manager_init_path = substrate_home.join("manager_init.sh");
        let manager_env_path = substrate_home.join("manager_env.sh");
        let bash_preexec_path = PathBuf::from(&home).join(".substrate_preexec");
        let host_bash_env = env::var("BASH_ENV").ok();

        let skip_shims_flag = cli.shim_skip || env::var("SUBSTRATE_NO_SHIMS").is_ok();
        let shimmed_path = if skip_shims_flag || final_no_world {
            None
        } else {
            let sep = if cfg!(windows) { ';' } else { ':' };
            let path_with_shims = format!("{}{}{}", shim_dir.display(), sep, &original_path);
            Some(dedupe_path(&path_with_shims))
        };

        // Determine shell to use
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = platform_world::windows::ensure_world_ready(&cli) {
                eprintln!(
                    "substrate: warn: windows world initialization failed: {:#}",
                    e
                );
            }
        }
        let shell_path = if let Some(shell) = cli.shell {
            shell
        } else if cfg!(windows) {
            // Windows: Check for PowerShell first, then cmd.exe
            if which::which("pwsh").is_ok() {
                "pwsh".to_string()
            } else if which::which("powershell").is_ok() {
                "powershell".to_string()
            } else {
                "cmd.exe".to_string()
            }
        } else {
            // Unix: Use $SHELL or fallback to /bin/bash
            env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        };

        // Determine mode
        let stdin_is_tty = io::stdin().is_terminal();

        let mode = if let Some(cmd) = cli.command {
            ShellMode::Wrap(cmd)
        } else if let Some(script) = cli.script {
            ShellMode::Script(script)
        } else if !stdin_is_tty {
            ShellMode::Pipe
        } else {
            // Ensure PTY is only used on Unix systems
            let use_pty = cli.use_pty && cfg!(unix);
            ShellMode::Interactive { use_pty }
        };

        let async_repl_enabled = !cli.legacy_repl;

        Ok(ShellConfig {
            mode,
            session_id,
            trace_log_file,
            original_path,
            shim_dir,
            shell_path,
            ci_mode: cli.ci_mode,
            no_exit_on_error: cli.no_exit_on_error,
            skip_shims: skip_shims_flag,
            no_world: final_no_world,
            world_root: world_root_settings,
            async_repl: async_repl_enabled,
            env_vars: HashMap::new(),
            manager_init_path,
            manager_env_path,
            shimmed_path,
            host_bash_env,
            bash_preexec_path,
            preexec_available: false,
        })
    }
}

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

/// Determines if a command requires shell interpretation.
///
/// Returns `true` if the command contains shell metacharacters that require
/// shell parsing (pipes, redirections, command substitution, etc.).
///
/// # Examples
///
/// Simple commands don't need shell interpretation:
/// ```
/// use substrate_shell::needs_shell;
///
/// assert!(!needs_shell("ls"));
/// assert!(!needs_shell("echo hello"));
/// assert!(!needs_shell("git status"));
/// ```
///
/// Commands with pipes need shell interpretation:
/// ```
/// use substrate_shell::needs_shell;
///
/// assert!(needs_shell("ls | grep txt"));
/// assert!(needs_shell("cat file.txt | head -10"));
/// ```
///
/// Commands with redirections need shell interpretation:
/// ```
/// use substrate_shell::needs_shell;
///
/// assert!(needs_shell("echo hello > file.txt"));
/// assert!(needs_shell("cat file.txt 2>/dev/null"));
/// assert!(needs_shell("command 2>&1"));
/// ```
///
/// Commands with logical operators need shell interpretation:
/// ```
/// use substrate_shell::needs_shell;
///
/// assert!(needs_shell("make && echo success"));
/// assert!(needs_shell("test -f file || echo missing"));
/// ```
///
/// Commands with command substitution need shell interpretation:
/// ```
/// use substrate_shell::needs_shell;
///
/// assert!(needs_shell("echo $(date)"));
/// assert!(needs_shell("ls `pwd`"));
/// ```
///
/// Malformed commands that can't be parsed are assumed to need shell:
/// ```
/// use substrate_shell::needs_shell;
///
/// assert!(needs_shell("echo 'unclosed quote"));
/// ```
pub fn needs_shell(cmd: &str) -> bool {
    let Ok(tokens) = shell_words::split(cmd) else {
        return true;
    };
    let ops = ["&&", "||", "|", ";", "<<", ">>", "<", ">", "&", "2>", "&>"];
    tokens.iter().any(|t| {
        ops.contains(&t.as_str())
        || t.starts_with("$(") || t.starts_with('`')
        || t.contains(">&")            // 2>&1, 1>&2, etc.
        || t.chars().any(|c| "<>|&".contains(c)) && t.len() > 1 // e.g. 1>/dev/null
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn set_env(key: &str, value: &str) -> Option<String> {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        previous
    }

    fn restore_env(key: &str, previous: Option<String>) {
        if let Some(value) = previous {
            std::env::set_var(key, value);
        } else {
            std::env::remove_var(key);
        }
    }

    #[test]
    #[serial]
    fn wrap_mode_uses_cli_shell_and_shimmed_path() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        fs::create_dir_all(substrate_home.join("shims")).unwrap();
        fs::write(
            substrate_home.join("config.toml"),
            "[install]\nworld_enabled = true\n",
        )
        .unwrap();

        let home_str = home.display().to_string();
        let substrate_home_str = substrate_home.display().to_string();
        let path_value = if cfg!(windows) {
            "C:\\bin;D:\\bin"
        } else {
            "/bin:/usr/bin"
        };
        let prev_home = set_env("HOME", &home_str);
        let prev_userprofile = set_env("USERPROFILE", &home_str);
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home_str);
        let prev_path = set_env("PATH", path_value);
        let prev_shim_original_path = std::env::var("SHIM_ORIGINAL_PATH").ok();
        let prev_world = std::env::var("SUBSTRATE_WORLD").ok();
        let prev_world_enabled = std::env::var("SUBSTRATE_WORLD_ENABLED").ok();
        let prev_no_shims = std::env::var("SUBSTRATE_NO_SHIMS").ok();
        std::env::remove_var("SHIM_ORIGINAL_PATH");
        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ENABLED");
        std::env::remove_var("SUBSTRATE_NO_SHIMS");

        let cli = Cli::parse_from(["substrate", "-c", "echo hi", "--shell", "/bin/zsh"]);
        let config = ShellConfig::from_cli(cli).expect("build shell config from CLI");

        match &config.mode {
            ShellMode::Wrap(cmd) => assert_eq!(cmd, "echo hi"),
            other => panic!("expected wrap mode, got {other:?}"),
        }
        assert_eq!(config.shell_path, "/bin/zsh");
        assert!(!config.no_world);
        assert!(!config.skip_shims);

        let sep = if cfg!(windows) { ';' } else { ':' };
        let expected = format!(
            "{}{sep}{}",
            PathBuf::from(&config.shim_dir).display(),
            std::env::var("PATH").unwrap()
        );
        let expected = substrate_common::dedupe_path(&expected);
        assert_eq!(config.shimmed_path.as_deref(), Some(expected.as_str()));

        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_NO_SHIMS", prev_no_shims);
        restore_env("SHIM_ORIGINAL_PATH", prev_shim_original_path);
        restore_env("PATH", prev_path);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }

    #[test]
    #[serial]
    fn skip_shims_and_no_world_disable_shimmed_path() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        fs::create_dir_all(substrate_home.join("shims")).unwrap();
        fs::write(
            substrate_home.join("config.toml"),
            "[install]\nworld_enabled = true\n",
        )
        .unwrap();

        let home_str = home.display().to_string();
        let substrate_home_str = substrate_home.display().to_string();
        let path_value = if cfg!(windows) {
            "C:\\bin;D:\\bin"
        } else {
            "/bin:/usr/bin"
        };
        let prev_home = set_env("HOME", &home_str);
        let prev_userprofile = set_env("USERPROFILE", &home_str);
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home_str);
        let prev_path = set_env("PATH", path_value);
        let prev_no_shims = set_env("SUBSTRATE_NO_SHIMS", "1");
        let prev_world = std::env::var("SUBSTRATE_WORLD").ok();
        let prev_world_enabled = std::env::var("SUBSTRATE_WORLD_ENABLED").ok();
        let prev_shim_original_path = std::env::var("SHIM_ORIGINAL_PATH").ok();
        std::env::remove_var("SHIM_ORIGINAL_PATH");

        let cli = Cli::parse_from(["substrate", "--no-world", "-c", "echo hi"]);
        let config = ShellConfig::from_cli(cli).expect("config honors skip flags");

        assert!(config.no_world);
        assert!(config.skip_shims);
        assert!(config.shimmed_path.is_none());
        assert_eq!(std::env::var("SUBSTRATE_WORLD").unwrap(), "disabled");
        assert_eq!(std::env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "0");

        restore_env("SHIM_ORIGINAL_PATH", prev_shim_original_path);
        restore_env("SUBSTRATE_NO_SHIMS", prev_no_shims);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("PATH", prev_path);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }
}
