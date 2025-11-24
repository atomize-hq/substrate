use crate::context::{
    build_clean_search_path, merge_path_sources, ShimContext, ORIGINAL_PATH_VAR, SHIM_CALLER_VAR,
    SHIM_CALL_STACK_VAR, SHIM_DEPTH_VAR, SHIM_PARENT_CMD_VAR,
};
use crate::resolver::resolve_real_binary;
use anyhow::{anyhow, Context, Result};
use std::env;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Instant, SystemTime};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[derive(Debug)]
pub(crate) struct CommandOutcome {
    pub(crate) status: ExitStatus,
    pub(crate) captured_stderr: Option<Vec<u8>>,
}

pub(crate) fn persist_original_path(ctx: &ShimContext) {
    if env::var(ORIGINAL_PATH_VAR).is_err() {
        let sep = if cfg!(windows) { ';' } else { ':' };
        let clean = ctx
            .search_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(&sep.to_string());
        env::set_var(ORIGINAL_PATH_VAR, clean);
    }
}

pub(crate) fn resolve_command_binary(ctx: &ShimContext) -> Result<PathBuf> {
    if ctx.command_name.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(&ctx.command_name);
        if is_executable(&path) {
            Ok(path)
        } else {
            Err(anyhow!("Command '{}' not found", ctx.command_name))
        }
    } else {
        resolve_real_binary(&ctx.command_name, &ctx.search_paths)
            .ok_or_else(|| anyhow!("Command '{}' not found", ctx.command_name))
    }
}

pub(crate) fn handle_bypass_mode() -> Result<i32> {
    let ctx = ShimContext::from_current_exe()?;
    let args: Vec<_> = env::args_os().skip(1).collect();

    let real_binary = if ctx.command_name.contains(std::path::MAIN_SEPARATOR) {
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
        resolve_real_binary(&ctx.command_name, &ctx.search_paths).ok_or_else(|| {
            anyhow!(
                "SHIM_BYPASS: Command '{}' not found in PATH",
                ctx.command_name
            )
        })?
    };

    let mut cmd = Command::new(&real_binary);

    #[cfg(unix)]
    cmd.arg0(&ctx.command_name);

    let status = cmd
        .args(&args)
        .status()
        .with_context(|| format!("SHIM_BYPASS exec failed: {}", real_binary.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ok(128 + signal);
        }
    }

    Ok(status.code().unwrap_or(1))
}

pub(crate) fn execute_real_binary_bypass(ctx: &ShimContext) -> Result<i32> {
    let merged_path = merge_path_sources(env::var(ORIGINAL_PATH_VAR).ok())
        .ok_or_else(|| anyhow!("No PATH available for bypass resolution"))?;

    let search_paths = build_clean_search_path(&ctx.shim_dir, Some(merged_path))?;
    let real_binary = resolve_real_binary(&ctx.command_name, &search_paths)
        .ok_or_else(|| anyhow!("Command '{}' not found in bypass mode", ctx.command_name))?;

    let args: Vec<_> = env::args_os().skip(1).collect();
    let depth = env::var(SHIM_DEPTH_VAR)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    env::set_var(SHIM_DEPTH_VAR, (depth + 1).to_string());

    let start_time = Instant::now();
    let timestamp = SystemTime::now();

    let mut cmd = Command::new(&real_binary);

    #[cfg(unix)]
    cmd.arg0(&ctx.command_name);

    let status = cmd
        .args(&args)
        .status()
        .with_context(|| format!("Failed to execute {} in bypass mode", real_binary.display()))?;

    let exit_code = status.code().unwrap_or(1);
    if let Some(log_path) = &ctx.log_file {
        #[allow(unused_mut)]
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
            "bypass": true,
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

            log_entry["ppid"] = serde_json::json!(nix::unistd::getppid().as_raw());
        }

        let _ = crate::logger::write_log_entry(log_path, &log_entry);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ok(128 + signal);
        }
    }

    Ok(exit_code)
}

pub(crate) fn execute_command(
    binary: &PathBuf,
    args: &[std::ffi::OsString],
    #[allow(unused_variables)] command_name: &str,
    capture_stderr: bool,
) -> Result<CommandOutcome> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(binary) {
            Ok(meta) => {
                let mode = meta.permissions().mode();
                if !meta.is_file() || (mode & 0o111) == 0 {
                    anyhow::bail!(
                        "Failed to execute {}: not an executable file",
                        binary.display()
                    );
                }
            }
            Err(_) => {
                anyhow::bail!("Failed to execute {}: not found", binary.display());
            }
        }
    }

    let mut cmd = Command::new(binary);

    #[cfg(unix)]
    cmd.arg0(command_name);

    if capture_stderr {
        let mut child = cmd
            .args(args)
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute {}", binary.display()))?;

        let stderr_handle = child.stderr.take().map(spawn_stderr_collector);
        let status = child
            .wait()
            .with_context(|| format!("Failed to execute {}", binary.display()))?;
        let captured = match stderr_handle {
            Some(handle) => match handle.join() {
                Ok(Ok(buf)) => Some(buf),
                _ => None,
            },
            None => None,
        };

        Ok(CommandOutcome {
            status,
            captured_stderr: captured,
        })
    } else {
        let status = cmd
            .args(args)
            .status()
            .with_context(|| format!("Failed to execute {}", binary.display()))?;

        Ok(CommandOutcome {
            status,
            captured_stderr: None,
        })
    }
}

fn spawn_stderr_collector(stderr: ChildStderr) -> thread::JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut buffer = Vec::new();
        let mut stderr_writer = io::stderr();
        let mut reader = BufReader::new(stderr);
        let mut chunk = [0u8; 8192];

        loop {
            let read = reader.read(&mut chunk)?;
            if read == 0 {
                break;
            }
            stderr_writer.write_all(&chunk[..read])?;
            buffer.extend_from_slice(&chunk[..read]);
        }

        Ok(buffer)
    })
}

/// Check if a path is executable (cross-platform)
pub(crate) fn is_executable(path: &Path) -> bool {
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
        use std::io::Read;
        if let Ok(meta) = std::fs::metadata(path) {
            if !meta.is_file() {
                return false;
            }
            if let Ok(mut f) = std::fs::File::open(path) {
                let mut head = [0u8; 2];
                if f.read(&mut head).ok() == Some(2) && &head == b"#!" {
                    return true;
                }
            }
            matches!(
                path.extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_ascii_lowercase()),
                Some(ref ext) if ["exe", "bat", "cmd", "com", "ps1"].contains(&ext.as_str())
            )
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ShimContext, ORIGINAL_PATH_VAR};
    use serial_test::serial;
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use tempfile::TempDir;

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn new(key: &'static str) -> Self {
            let previous = env::var(key).ok();
            env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = self.previous.take() {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[test]
    #[serial]
    fn persist_original_path_sets_value_once() {
        let _guard = EnvGuard::new(ORIGINAL_PATH_VAR);
        let temp = TempDir::new().unwrap();
        let paths = vec![temp.path().join("one"), temp.path().join("two")];
        let ctx = ShimContext {
            command_name: "demo".to_string(),
            shim_dir: temp.path().join("shims"),
            search_paths: paths.clone(),
            log_file: None,
            session_id: "session".to_string(),
            depth: 0,
        };

        persist_original_path(&ctx);
        let stored = env::var(ORIGINAL_PATH_VAR).expect("original path to be set");
        assert!(
            stored.contains(&paths[0].display().to_string())
                && stored.contains(&paths[1].display().to_string()),
            "persisted PATH should include search paths: {stored}"
        );

        env::set_var(ORIGINAL_PATH_VAR, "custom-value");
        persist_original_path(&ctx);
        assert_eq!(
            env::var(ORIGINAL_PATH_VAR).as_deref(),
            Ok("custom-value"),
            "existing SHIM_ORIGINAL_PATH should be preserved"
        );
    }

    #[test]
    #[cfg(unix)]
    fn execute_command_captures_stderr_when_requested() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        let script = temp.path().join("stderr.sh");
        fs::write(
            &script,
            "#!/usr/bin/env bash\necho test-stdout\necho test-stderr >&2\n",
        )
        .unwrap();
        let mut perms = fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).unwrap();

        let args = [OsString::from("arg1")];
        let captured = execute_command(&script, &args, "stderr.sh", true).unwrap();
        assert!(captured.status.success());
        let stderr = String::from_utf8(captured.captured_stderr.unwrap()).unwrap();
        assert!(
            stderr.contains("test-stderr"),
            "captured stderr should include child stderr output: {stderr}"
        );

        let without_capture = execute_command(&script, &args, "stderr.sh", false).unwrap();
        assert!(without_capture.status.success());
        assert!(without_capture.captured_stderr.is_none());
    }
}
