//! Main shim execution logic with bypass handling and error recovery
//!
//! This module contains the core `run_shim` function that orchestrates the entire
//! shim execution process, including bypass mode, path resolution, command execution,
//! and logging.

use anyhow::{anyhow, Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::time::{Instant, SystemTime};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::context::{
    build_clean_search_path, ShimContext, ORIGINAL_PATH_VAR, SHIM_CALLER_VAR, SHIM_CALL_STACK_VAR,
    SHIM_DEPTH_VAR, SHIM_PARENT_CMD_VAR,
};
use crate::logger::{log_execution, write_log_entry};
use crate::resolver::resolve_real_binary;

/// Main shim execution function
pub fn run_shim() -> Result<i32> {
    // Early escape hatch for debugging and sensitive sessions
    if ShimContext::is_bypass_enabled() {
        return handle_bypass_mode();
    }

    let ctx = ShimContext::from_current_exe()?;

    // If SHIM_ACTIVE is set, this is a nested shim call (e.g., npm -> node)
    // Bypass shim logic and execute the real binary directly
    if ctx.should_skip_shimming() {
        return execute_real_binary_bypass(&ctx);
    }

    // Set up environment for execution
    ctx.setup_execution_env();

    // Handle explicit paths (containing '/') differently
    let real_binary = if ctx.command_name.contains(std::path::MAIN_SEPARATOR) {
        // Explicit path - don't search PATH
        let path = PathBuf::from(&ctx.command_name);
        if is_executable(&path) {
            Some(path)
        } else {
            None
        }
    } else {
        resolve_real_binary(&ctx.command_name, &ctx.search_paths)
    }
    .ok_or_else(|| anyhow!("Command '{}' not found", ctx.command_name))?;

    // Prepare execution context
    let args: Vec<_> = env::args_os().skip(1).collect();
    let start_time = Instant::now();
    let timestamp = SystemTime::now();

    // Execute the real command with spawn failure telemetry
    let status = match execute_command(&real_binary, &args, &ctx.command_name) {
        Ok(status) => status,
        Err(e) => {
            // Log spawn failure with detailed error information
            if let Some(log_path) = &ctx.log_file {
                let spawn_error = e.downcast_ref::<std::io::Error>();
                let mut error_entry = serde_json::json!({
                    "ts": crate::logger::format_timestamp(timestamp),
                    "command": ctx.command_name,
                    "resolved_path": real_binary.display().to_string(),
                    "error": "spawn_failed",
                    "depth": ctx.depth,
                    "session_id": ctx.session_id,
                    "shim_fingerprint": crate::logger::get_shim_fingerprint()
                });

                if let Some(io_err) = spawn_error {
                    error_entry["spawn_error_kind"] =
                        serde_json::json!(format!("{:?}", io_err.kind()));
                    if let Some(errno) = io_err.raw_os_error() {
                        error_entry["spawn_errno"] = serde_json::json!(errno);
                    }
                }

                let _ = write_log_entry(log_path, &error_entry);
            }
            return Err(e);
        }
    };

    let duration = start_time.elapsed();

    // Always log execution with depth and session correlation
    if let Some(log_path) = &ctx.log_file {
        if let Err(e) = log_execution(
            log_path,
            &ctx,
            &args,
            &status,
            duration,
            timestamp,
            &real_binary,
        ) {
            eprintln!("Warning: Failed to log execution: {e}");
        }
    }

    // Unix signal exit status parity - return 128 + signal for terminated processes
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ok(128 + signal);
        }
    }

    Ok(status.code().unwrap_or(1))
}

/// Handle bypass mode execution
fn handle_bypass_mode() -> Result<i32> {
    let ctx = ShimContext::from_current_exe()?;
    let args: Vec<_> = env::args_os().skip(1).collect();

    // Resolve the real binary (same logic as normal execution)
    let real_binary = if ctx.command_name.contains(std::path::MAIN_SEPARATOR) {
        // Explicit path - don't search PATH
        let path = PathBuf::from(&ctx.command_name);
        if is_executable(&path) {
            path
        } else {
            return Err(anyhow!(
                "SHIM_BYPASS: Command '{}' not executable",
                ctx.command_name
            ));
        }
    } else {
        // Search PATH
        resolve_real_binary(&ctx.command_name, &ctx.search_paths).ok_or_else(|| {
            anyhow!(
                "SHIM_BYPASS: Command '{}' not found in PATH",
                ctx.command_name
            )
        })?
    };

    // Direct execution without logging
    let mut cmd = Command::new(&real_binary);

    #[cfg(unix)]
    cmd.arg0(&ctx.command_name); // Preserve argv[0] semantics on Unix

    let status = cmd
        .args(&args)
        .status()
        .with_context(|| format!("SHIM_BYPASS exec failed: {}", real_binary.display()))?;

    // Unix signal exit status parity
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ok(128 + signal);
        }
    }

    Ok(status.code().unwrap_or(1))
}

/// Execute real binary when in bypass mode (nested shim call)
fn execute_real_binary_bypass(ctx: &ShimContext) -> Result<i32> {
    // Get clean PATH without shim directory
    let original_path = env::var(ORIGINAL_PATH_VAR)
        .or_else(|_| env::var("PATH"))
        .unwrap_or_default();

    // Build clean search paths
    let search_paths = build_clean_search_path(&ctx.shim_dir, Some(original_path))?;

    // Resolve the real binary
    let real_binary = resolve_real_binary(&ctx.command_name, &search_paths)
        .ok_or_else(|| anyhow!("Command '{}' not found in bypass mode", ctx.command_name))?;

    // Get command arguments
    let args: Vec<_> = env::args_os().skip(1).collect();

    // Increment depth for observability (but keep SHIM_ACTIVE set)
    let depth = env::var(SHIM_DEPTH_VAR)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    env::set_var(SHIM_DEPTH_VAR, (depth + 1).to_string());

    // Log the bypass execution for observability
    let start_time = Instant::now();
    let timestamp = SystemTime::now();

    // Execute the real command
    let mut cmd = Command::new(&real_binary);

    #[cfg(unix)]
    cmd.arg0(&ctx.command_name); // Preserve argv[0] semantics

    let status = cmd
        .args(&args)
        .status()
        .with_context(|| format!("Failed to execute {} in bypass mode", real_binary.display()))?;

    // Log the bypass execution
    let exit_code = status.code().unwrap_or(1);
    if let Some(log_path) = &ctx.log_file {
        // Log with a bypass marker in the entry
        let mut log_entry = serde_json::json!({
            "ts": crate::logger::format_timestamp(timestamp),
            "command": ctx.command_name,
            "argv": std::iter::once(ctx.command_name.clone())
                .chain(crate::logger::redact_sensitive_argv(&args))
                .collect::<Vec<_>>(),
            "resolved_path": real_binary.display().to_string(),
            "exit_code": exit_code,
            "duration_ms": start_time.elapsed().as_millis(),
            "component": "shim",
            "depth": depth + 1,
            "session_id": ctx.session_id,
            "bypass": true,  // Mark this as a bypass execution
            "caller": env::var(SHIM_CALLER_VAR).ok(),
            "call_stack": env::var(SHIM_CALL_STACK_VAR).ok(),
            "parent_cmd_id": env::var(SHIM_PARENT_CMD_VAR).ok(),
            "cwd": env::current_dir().unwrap_or_else(|_| PathBuf::from("/unknown")).to_string_lossy(),
            "pid": std::process::id(),
            "hostname": gethostname::gethostname().to_string_lossy().to_string(),
            "platform": if cfg!(target_os = "macos") { "macos" } else if cfg!(target_os = "linux") { "linux" } else { "other" },
            "shim_fingerprint": crate::logger::get_shim_fingerprint(),
            "user": env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
        });

        // Add TTY information
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            log_entry["isatty_stdin"] = serde_json::json!(nix::unistd::isatty(
                std::io::stdin().as_raw_fd()
            )
            .unwrap_or(false));
            log_entry["isatty_stdout"] = serde_json::json!(nix::unistd::isatty(
                std::io::stdout().as_raw_fd()
            )
            .unwrap_or(false));
            log_entry["isatty_stderr"] = serde_json::json!(nix::unistd::isatty(
                std::io::stderr().as_raw_fd()
            )
            .unwrap_or(false));

            // Add parent process ID
            log_entry["ppid"] = serde_json::json!(nix::unistd::getppid().as_raw());
        }

        let _ = write_log_entry(log_path, &log_entry);
    }

    // Unix signal exit status parity
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ok(128 + signal);
        }
    }

    Ok(exit_code)
}

/// Execute command with preserved argv[0] semantics
fn execute_command(
    binary: &PathBuf,
    args: &[std::ffi::OsString],
    command_name: &str,
) -> Result<ExitStatus> {
    let mut cmd = Command::new(binary);

    #[cfg(unix)]
    cmd.arg0(command_name); // Preserve argv[0] semantics for tools that check invocation name

    let status = cmd
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute {}", binary.display()))?;

    Ok(status)
}

/// Check if a path is executable (cross-platform)
fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0)
        } else {
            false
        }
    }

    #[cfg(windows)]
    {
        std::fs::metadata(path)
            .map(|m| m.is_file())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_executable_bit_check() {
        let temp = TempDir::new().unwrap();
        let non_executable = temp.path().join("not_exec");
        fs::write(&non_executable, "content").unwrap();

        // Should not be considered executable
        assert!(!is_executable(&non_executable));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let executable = temp.path().join("exec");
            fs::write(&executable, "#!/bin/bash\necho test").unwrap();
            let mut perms = fs::metadata(&executable).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&executable, perms).unwrap();

            assert!(is_executable(&executable));
        }
    }

    #[test]
    fn test_spawn_failure_handling() {
        // Test that spawn failures are properly logged
        use std::ffi::OsString;

        // This should fail to spawn
        let result = execute_command(
            &PathBuf::from("/nonexistent/command"),
            &[OsString::from("arg1")],
            "nonexistent",
        );

        assert!(result.is_err());

        // The error should be an io::Error that we can inspect
        if let Err(e) = result {
            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
            }
        }
    }
}
