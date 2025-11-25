//! PTY execution entrypoint wiring together manager and reader flows.

use super::super::control::ActivePtyGuard;
#[cfg(windows)]
use super::manager::initialize_windows_input_forwarder;
use super::manager::start_pty_manager;
use super::reader::handle_pty_io;
#[cfg(any(unix, test))]
use super::types::verify_process_group;
use super::types::{get_terminal_size, MinimalTerminalGuard, PtyActiveGuard, PtyExitStatus};
use crate::execution::{
    configure_child_shell_env, log_command_event, ShellConfig, ShellMode, PTY_ACTIVE,
};
use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder};
use serde_json::json;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

/// Execute a command with full PTY support.
pub fn execute_with_pty(
    config: &ShellConfig,
    command: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
) -> Result<PtyExitStatus> {
    // Initialize global handlers once
    #[cfg(unix)]
    super::super::control::initialize_global_sigwinch_handler_impl();

    #[cfg(windows)]
    initialize_windows_input_forwarder();

    // Set PTY active flag to prevent double SIGINT handling
    PTY_ACTIVE.store(true, Ordering::SeqCst);

    // Ensure flag is cleared on exit (RAII guard for panic safety)
    let _pty_guard = PtyActiveGuard;

    // Create minimal terminal guard - ONLY for stdin raw mode
    // This allows proper input forwarding without display interference
    let _terminal_guard = MinimalTerminalGuard::new()?;

    // Get current terminal size
    let pty_size = get_terminal_size()?;

    // Log the detected terminal size (debug only)
    log::info!(
        "PTY: Detected terminal size: {}x{} (rows x cols)",
        pty_size.rows,
        pty_size.cols
    );

    // Create PTY system
    let pty_system = native_pty_system();

    // Create a new PTY pair with graceful error on older Windows
    let pair = pty_system.openpty(pty_size).map_err(|e| {
        #[cfg(windows)]
        {
            // ConPTY requires Windows 10 1809+
            anyhow::anyhow!(
                "PTY creation failed. ConPTY requires Windows 10 version 1809 or later. Error: {}",
                e
            )
        }
        #[cfg(not(windows))]
        {
            anyhow::anyhow!("Failed to create PTY: {}", e)
        }
    })?;

    // Prepare command - handle :pty prefix if present
    let actual_command = if let Some(stripped) = command.strip_prefix(":pty ") {
        stripped
    } else {
        command
    };

    let mut cmd = CommandBuilder::new(&config.shell_path);
    let shell_name = Path::new(&config.shell_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let is_bash = shell_name == "bash" || shell_name == "bash.exe";
    cmd.arg("-c");
    cmd.arg(actual_command);
    cmd.cwd(std::env::current_dir()?);

    // CRITICAL: Preserve tracing environment variables needed for logging
    cmd.env("SHIM_SESSION_ID", &config.session_id);
    cmd.env("SHIM_TRACE_LOG", &config.trace_log_file);
    cmd.env("SHIM_PARENT_CMD_ID", cmd_id);

    // Clear SHIM_ACTIVE/CALLER/CALL_STACK to allow shims to work inside PTY
    cmd.env_remove("SHIM_ACTIVE");
    cmd.env_remove("SHIM_CALLER");
    cmd.env_remove("SHIM_CALL_STACK");

    configure_child_shell_env(
        &mut cmd,
        config,
        is_bash,
        matches!(config.mode, ShellMode::Script(_)),
    );

    // Preserve existing TERM or set a default
    // Many TUIs like claude need the correct TERM to function properly
    match std::env::var("TERM") {
        Ok(term) => cmd.env("TERM", term),
        Err(_) => cmd.env("TERM", "xterm-256color"),
    };

    // Set COLUMNS/LINES for TUIs that read them (only if valid)
    if pty_size.cols > 0 && pty_size.rows > 0 {
        cmd.env("COLUMNS", pty_size.cols.to_string());
        cmd.env("LINES", pty_size.rows.to_string());
    }

    // Log command start with pty flag and initial size
    let start_extra = json!({
        "pty": true,
        "pty_rows": pty_size.rows,
        "pty_cols": pty_size.cols
    });

    // Add debug logging if requested
    if std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
        log::debug!(
            "[{}] PTY allocated: {}x{}",
            cmd_id,
            pty_size.cols,
            pty_size.rows
        );
    }

    log_command_event(
        config,
        "command_start",
        actual_command,
        cmd_id,
        Some(start_extra),
    )?;
    let start_time = std::time::Instant::now();

    // Spawn the child process
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context(format!("Failed to spawn PTY command: {actual_command}"))?;

    // Store child PID for signal handling
    if let Some(pid) = child.process_id() {
        running_child_pid.store(pid as i32, Ordering::SeqCst);
    }

    // Verify process group setup (Unix only, debug mode)
    #[cfg(unix)]
    if std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
        if let Some(pid) = child.process_id() {
            verify_process_group(Some(pid));
        }
    }

    let (pty_control, reader, manager_handle) = start_pty_manager(pair.master)?;
    let _active_pty_guard = ActivePtyGuard::register(pty_control.clone());

    let exit_status = handle_pty_io(pty_control, reader, &mut child, cmd_id, manager_handle)?;

    // Clear the running PID BEFORE logging completion
    running_child_pid.store(0, Ordering::SeqCst);

    // Log command completion with pty flag
    let duration = start_time.elapsed();
    let mut extra = json!({
        "duration_ms": duration.as_millis(),
        "pty": true
    });

    if let Some(code) = exit_status.code {
        extra["exit_code"] = json!(code);
    }
    if let Some(signal) = exit_status.signal {
        extra["term_signal"] = json!(signal);
    }

    log_command_event(
        config,
        "command_complete",
        actual_command,
        cmd_id,
        Some(extra),
    )?;

    Ok(exit_status)
}
