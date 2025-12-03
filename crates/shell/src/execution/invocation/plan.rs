//! Shell invocation planning and environment preparation.

use crate::builtins as commands;
use crate::execution::cli::*;
use crate::execution::settings::{self, apply_world_root_env, resolve_world_root};
use crate::execution::shim_deploy::{DeploymentStatus, ShimDeployer};
#[cfg(target_os = "linux")]
use crate::execution::socket_activation;
use crate::execution::{
    handle_config_command, handle_graph_command, handle_health_command, handle_replay_command,
    handle_shim_command, handle_trace_command, handle_world_command, update_world_env,
};
use anyhow::{Context, Result};
use clap::Parser;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use substrate_common::{dedupe_path, paths as substrate_paths, WorldRootMode};
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
                    "agent_socket": shim_status_socket_json(),
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
                    "agent_socket": shim_status_socket_json(),
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
                    "agent_socket": shim_status_socket_json(),
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
                    "agent_socket": shim_status_socket_json(),
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
                print_socket_activation_summary();
                println!("Status: Skipped");
                std::process::exit(0);
            }

            let shims_dir = substrate_common::paths::shims_dir()?;
            let version_file = substrate_common::paths::version_file()?;

            if !shims_dir.exists() {
                println!("Shims: Not deployed");
                print_socket_activation_summary();
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
                print_socket_activation_summary();

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
                print_socket_activation_summary();
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
        if let Some(span_id) = cli.replay.clone() {
            if cli.replay_verbose {
                env::set_var("SUBSTRATE_REPLAY_VERBOSE", "1");
            }
            handle_replay_command(&span_id, &cli)?;
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
                SubCommands::Config(config_cmd) => {
                    handle_config_command(config_cmd)?;
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
        let config_path = substrate_paths::config_file()?;
        let install_config = commands::world_enable::load_install_config(&config_path)?;
        if !install_config.exists() {
            eprintln!(
                "substrate: info: no config file at {}; run `substrate config init` to create defaults",
                config_path.display()
            );
        }
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
            use crate::execution::platform_world::windows;

            if let Err(e) = windows::ensure_world_ready(&cli) {
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

#[cfg(target_os = "linux")]
fn shim_status_socket_json() -> serde_json::Value {
    let report = socket_activation::socket_activation_report();
    json!({
        "mode": report.mode.as_str(),
        "path": report.socket_path.as_str(),
        "socket_path": report.socket_path.as_str(),
        "socket_exists": report.socket_exists,
        "systemd_error": report.systemd_error,
        "systemd_socket": report.socket_unit.as_ref().map(|unit| json!({
            "name": unit.name,
            "active_state": unit.active_state,
            "unit_file_state": unit.unit_file_state,
            "listens": unit.listens,
        })),
        "systemd_service": report.service_unit.as_ref().map(|unit| json!({
            "name": unit.name,
            "active_state": unit.active_state,
            "unit_file_state": unit.unit_file_state,
            "listens": unit.listens,
        })),
    })
}

#[cfg(not(target_os = "linux"))]
fn shim_status_socket_json() -> serde_json::Value {
    serde_json::Value::Null
}

#[cfg(target_os = "linux")]
fn print_socket_activation_summary() {
    let report = socket_activation::socket_activation_report();
    if report.is_socket_activated() {
        println!(
            "World socket: socket activation ({} {})",
            report
                .socket_unit
                .as_ref()
                .map(|u| u.name)
                .unwrap_or("substrate-world-agent.socket"),
            report
                .socket_unit
                .as_ref()
                .map(|u| u.active_state.as_str())
                .unwrap_or("unknown")
        );
    } else if report.socket_unit.is_some() {
        println!(
            "World socket: socket activation inactive ({} state: {})",
            report
                .socket_unit
                .as_ref()
                .map(|u| u.name)
                .unwrap_or("substrate-world-agent.socket"),
            report
                .socket_unit
                .as_ref()
                .map(|u| u.active_state.as_str())
                .unwrap_or("unknown")
        );
    } else if report.socket_exists {
        println!("World socket: manual listener present");
    } else {
        println!("World socket: listener missing; run `substrate world enable`");
    }
    println!("Socket path: {}", report.socket_path);
}

#[cfg(not(target_os = "linux"))]
fn print_socket_activation_summary() {}
