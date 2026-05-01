//! Invocation execution helpers.

use super::plan::{ShellConfig, ShellMode};
use crate::execution::config_model;
use crate::execution::config_model::CliConfigOverrides;
use crate::execution::{
    configure_child_shell_env, execute_command, export_runtime_config_env, log_command_event,
    setup_signal_handlers,
};
use anyhow::{Context, Result};
use serde_json::json;
use std::io::{self, BufRead};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use substrate_common::log_schema;
use substrate_common::{WorldFsStrategy, WorldFsStrategyFallbackReason};
use substrate_trace::create_span_builder;
use uuid::Uuid;

fn preflight_caging_required(config: &ShellConfig) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cli_world_enabled = if config.cli_world {
        Some(true)
    } else if config.cli_no_world {
        Some(false)
    } else {
        None
    };
    let effective_config = config_model::resolve_effective_config(
        &cwd,
        &CliConfigOverrides {
            world_enabled: cli_world_enabled,
            anchor_mode: config.cli_anchor_mode,
            anchor_path: config
                .cli_anchor_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            caged: config.cli_caged,
        },
    )?;

    let policy_mode = effective_config.policy.mode;
    std::env::set_var("SUBSTRATE_POLICY_MODE", policy_mode.as_str());
    export_runtime_config_env(&effective_config);
    substrate_broker::set_policy_mode(match policy_mode {
        config_model::PolicyMode::Disabled => substrate_broker::PolicyMode::Disabled,
        config_model::PolicyMode::Observe => substrate_broker::PolicyMode::Observe,
        config_model::PolicyMode::Enforce => substrate_broker::PolicyMode::Enforce,
    });

    substrate_broker::detect_profile(&cwd)
        .with_context(|| format!("failed to load Substrate profile for cwd {}", cwd.display()))
        .map_err(|err| config_model::user_error(format!("{:#}", err)))?;

    let world_fs = substrate_broker::world_fs_policy();
    if world_fs.caged_required {
        if !effective_config.world.caged {
            return Err(config_model::user_error(
                "world_fs.caged_required=true requires world.caged=true (uncaged mode is a hard error)",
            ));
        }
        if effective_config.world.anchor_mode == substrate_common::WorldRootMode::FollowCwd {
            return Err(config_model::user_error(
                "world_fs.caged_required=true is incompatible with world.anchor_mode=follow-cwd (hard error)",
            ));
        }
    }

    Ok(())
}

pub(crate) fn run_wrap_mode(config: &ShellConfig, command: &str) -> Result<i32> {
    let cmd_id = Uuid::now_v7().to_string();
    let running_child_pid = Arc::new(AtomicI32::new(0));

    // Set up signal handlers for wrap mode to properly handle signals like SIGTERM
    setup_signal_handlers(running_child_pid.clone())?;

    let status = execute_command(config, command, &cmd_id, running_child_pid, None)?;
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

    preflight_caging_required(config)?;

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
    let mut span = if let Ok(mut builder) = create_span_builder() {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());
        builder = builder.with_command(&script_cmd).with_cwd(&cwd);
        builder.start().ok()
    } else {
        None
    };
    let span_id_for_cmd_events = span.as_ref().map(|s| s.get_span_id().to_string());

    cmd.env("SHIM_PARENT_CMD_ID", &cmd_id);
    if let Some(span_id) = span_id_for_cmd_events.as_ref() {
        cmd.env("SHIM_PARENT_SPAN", span_id);
    }

    let start_extra = span_id_for_cmd_events
        .as_ref()
        .map(|span_id| json!({ "span_id": span_id }));
    log_command_event(config, "command_start", &script_cmd, &cmd_id, start_extra)?;
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
    let mut extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });

    #[cfg(unix)]
    {
        if let Some(sig) = status.signal() {
            extra["term_signal"] = json!(sig);
        }
    }

    extra["world_fs_strategy_primary"] = json!(WorldFsStrategy::Overlay.as_str());
    extra["world_fs_strategy_final"] = json!(WorldFsStrategy::Host.as_str());
    extra["world_fs_strategy_fallback_reason"] =
        json!(WorldFsStrategyFallbackReason::None.as_str());
    if let Some(span_id) = span_id_for_cmd_events.as_ref() {
        extra["span_id"] = json!(span_id);
    }

    log_command_event(
        config,
        "command_complete",
        &script_cmd,
        &cmd_id,
        Some(extra),
    )?;

    if let Some(active_span) = span.take() {
        let _ = active_span.finish(exit_code(status), vec![], None);
    }

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
        match execute_command(config, &line, &cmd_id, no_signal_handler.clone(), None) {
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
