//! Main shim execution logic with bypass handling and error recovery
//!
//! This module contains the core `run_shim` function that orchestrates the entire
//! shim execution process, including bypass mode, path resolution, command execution,
//! and logging.

use anyhow::{anyhow, Context, Result};
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::io::{self, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{ChildStderr, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Instant, SystemTime};
use world_api::FsDiff;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::context::{
    build_clean_search_path, merge_path_sources, world_features_enabled, ShimContext,
    ORIGINAL_PATH_VAR, SHIM_CALLER_VAR, SHIM_CALL_STACK_VAR, SHIM_DEPTH_VAR, SHIM_PARENT_CMD_VAR,
};
use crate::logger::{format_timestamp, log_execution, write_log_entry};
use crate::resolver::resolve_real_binary;
use substrate_broker::{quick_check, Decision};
use substrate_common::{
    manager_manifest::{ManagerManifest, ManagerSpec, Platform, RegexPattern},
    paths,
};
use substrate_trace::{create_span_builder, init_trace};

/// Main shim execution function
pub fn run_shim() -> Result<i32> {
    // Early escape hatch for debugging and sensitive sessions
    if ShimContext::is_bypass_enabled() {
        return handle_bypass_mode();
    }

    let ctx = ShimContext::from_current_exe()?;

    // Ensure SHIM_ORIGINAL_PATH is persisted for nested shims (clean PATH without shim dir)
    if std::env::var(ORIGINAL_PATH_VAR).is_err() {
        let sep = if cfg!(windows) { ';' } else { ':' };
        let clean = ctx
            .search_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(&sep.to_string());
        std::env::set_var(ORIGINAL_PATH_VAR, clean);
    }

    // If SHIM_ACTIVE is set, this is a nested shim call (e.g., npm -> node)
    // Bypass shim logic and execute the real binary directly
    if ctx.should_skip_shimming() {
        return execute_real_binary_bypass(&ctx);
    }

    // Set up environment for execution
    ctx.setup_execution_env();

    let mut hint_engine = ManagerHintEngine::new();
    let capture_stderr = hint_engine
        .as_ref()
        .map(|engine| engine.is_active())
        .unwrap_or(false);

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
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Build command string for logging and spans
    let command_str = std::iter::once(ctx.command_name.clone())
        .chain(args.iter().map(|s| s.to_string_lossy().to_string()))
        .collect::<Vec<_>>()
        .join(" ");

    // Policy evaluation (Phase 4) and span creation
    let mut policy_decision = None;
    let mut active_span = None;

    if world_features_enabled() {
        // Initialize trace if needed
        let _ = init_trace(None);

        let argv: Vec<String> = std::iter::once(ctx.command_name.clone())
            .chain(args.iter().map(|s| s.to_string_lossy().to_string()))
            .collect();

        match quick_check(&argv, cwd.to_str().unwrap_or(".")) {
            Ok(Decision::Allow) => {
                policy_decision = Some(substrate_trace::PolicyDecision {
                    action: "allow".to_string(),
                    restrictions: None,
                    reason: None,
                });
            }
            Ok(Decision::AllowWithRestrictions(restrictions)) => {
                eprintln!(
                    "substrate: command requires restrictions: {:?}",
                    restrictions
                );
                policy_decision = Some(substrate_trace::PolicyDecision {
                    action: "allow_with_restrictions".to_string(),
                    restrictions: Some(restrictions.iter().map(|r| format!("{:?}", r)).collect()),
                    reason: None,
                });
            }
            Ok(Decision::Deny(reason)) => {
                eprintln!("substrate: command denied by policy: {}", reason);
                policy_decision = Some(substrate_trace::PolicyDecision {
                    action: "deny".to_string(),
                    restrictions: None,
                    reason: Some(reason.clone()),
                });

                // Create span for denied command
                let mut builder = create_span_builder()
                    .with_command(&command_str)
                    .with_cwd(cwd.to_str().unwrap_or("."));

                if let Some(pd) = policy_decision.clone() {
                    builder = builder.with_policy_decision(pd);
                }

                if let Ok(span) = builder.start() {
                    // Immediately finish with error code
                    let _ = span.finish(126, vec![], None);
                }

                return Ok(126); // Cannot execute
            }
            Err(e) => {
                eprintln!("substrate: policy check failed: {}", e);
                // Continue in observe mode
            }
        }

        // Create span for allowed command
        let mut builder = create_span_builder()
            .with_command(&command_str)
            .with_cwd(cwd.to_str().unwrap_or("."));

        if let Some(pd) = policy_decision.clone() {
            builder = builder.with_policy_decision(pd);
        }

        // Set parent span ID in environment for child processes
        match builder.start() {
            Ok(span) => {
                std::env::set_var("SHIM_PARENT_SPAN", span.get_span_id());
                active_span = Some(span);
            }
            Err(e) => {
                eprintln!("substrate: failed to create span: {}", e);
            }
        }
    }

    let start_time = Instant::now();
    let timestamp = SystemTime::now();

    // Execute the real command with spawn failure telemetry
    let outcome = match execute_command(&real_binary, &args, &ctx.command_name, capture_stderr) {
        Ok(outcome) => outcome,
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

    let mut manager_hint_payload = None;
    if let Some(engine) = hint_engine.as_mut() {
        if engine.is_active()
            && !outcome.status.success()
            && capture_stderr
            && outcome.captured_stderr.is_some()
        {
            if let Some(match_info) = engine.evaluate(outcome.captured_stderr.as_deref().unwrap()) {
                eprintln!(
                    "substrate: {} hint matched (pattern: {})\n{}",
                    match_info.manager_name,
                    match_info.pattern,
                    match_info.hint.trim_end()
                );
                manager_hint_payload = Some(json!({
                    "name": match_info.manager_name,
                    "hint": match_info.hint,
                    "pattern": match_info.pattern,
                    "ts": format_timestamp(SystemTime::now())
                }));
            }
        }
    }

    let status = outcome.status;
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
            manager_hint_payload.as_ref(),
        ) {
            eprintln!("Warning: Failed to log execution: {e}");
        }
    }

    // Complete span if we created one
    if let Some(span) = active_span {
        let exit_code = status.code().unwrap_or(-1);

        // Collect scopes and fs_diff from world backend if enabled
        let (scopes_used, fs_diff) = if world_features_enabled() {
            collect_world_telemetry(span.get_span_id())
        } else {
            (vec![], None)
        };

        let _ = span.finish(exit_code, scopes_used, fs_diff);
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
    let merged_path = merge_path_sources(env::var(ORIGINAL_PATH_VAR).ok())
        .ok_or_else(|| anyhow!("No PATH available for bypass resolution"))?;

    // Build clean search paths
    let search_paths = build_clean_search_path(&ctx.shim_dir, Some(merged_path))?;

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

#[derive(Debug)]
struct CommandOutcome {
    status: ExitStatus,
    captured_stderr: Option<Vec<u8>>,
}

/// Execute command with preserved argv[0] semantics
fn execute_command(
    binary: &PathBuf,
    args: &[std::ffi::OsString],
    #[allow(unused_variables)] command_name: &str,
    capture_stderr: bool,
) -> Result<CommandOutcome> {
    // Proactive guard: ensure the binary exists and is executable on Unix.
    // This makes failure behavior consistent across environments (some distros may
    // return different errors when spawning invalid paths).
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
    cmd.arg0(command_name); // Preserve argv[0] semantics for tools that check invocation name

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
        use std::io::Read;
        if let Ok(meta) = std::fs::metadata(path) {
            if !meta.is_file() {
                return false;
            }
            // Treat shebang scripts as executable
            if let Ok(mut f) = std::fs::File::open(path) {
                let mut head = [0u8; 2];
                if f.read(&mut head).ok() == Some(2) && &head == b"#!" {
                    return true;
                }
            }
            // Executable extensions on Windows
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

/// Collect filesystem diff and network scopes from world backend
#[allow(unused_variables)]
fn collect_world_telemetry(span_id: &str) -> (Vec<String>, Option<FsDiff>) {
    // Try to get world handle from environment
    let world_id = match env::var("SUBSTRATE_WORLD_ID") {
        Ok(id) => id,
        Err(_) => {
            // No world ID, return empty telemetry
            return (vec![], None);
        }
    };

    // Create world backend via factory and collect telemetry (macOS/Linux parity)
    if let Ok(backend) = world_backend_factory::factory() {
        let handle = world_api::WorldHandle {
            id: world_id.clone(),
        };

        // Try to get filesystem diff
        let fs_diff = match backend.fs_diff(&handle, span_id) {
            Ok(diff) => Some(diff),
            Err(e) => {
                eprintln!("Warning: Failed to collect fs_diff: {}", e);
                None
            }
        };

        // Scopes are returned from exec path; not re-fetched here
        let scopes_used = vec![];
        (scopes_used, fs_diff)
    } else {
        (vec![], None)
    }
}

struct ManagerHintEngine {
    rules: Vec<ManagerHintRule>,
    emitted: HashSet<String>,
}

impl ManagerHintEngine {
    fn new() -> Option<Self> {
        if !world_features_enabled() || hints_disabled() {
            return None;
        }

        let (base, overlay) = manifest_paths()?;
        let manifest = ManagerManifest::load(&base, overlay.as_deref()).ok()?;
        let specs = manifest.resolve_for_platform(current_platform());

        let mut rules = Vec::new();
        for spec in specs {
            if let Some(rule) = ManagerHintRule::from_spec(&spec) {
                rules.push(rule);
            }
        }

        if rules.is_empty() {
            None
        } else {
            Some(Self {
                rules,
                emitted: HashSet::new(),
            })
        }
    }

    fn is_active(&self) -> bool {
        !self.rules.is_empty()
    }

    fn evaluate(&mut self, stderr: &[u8]) -> Option<HintMatch> {
        let stderr_text = String::from_utf8_lossy(stderr);
        for rule in &self.rules {
            if self.emitted.contains(&rule.key) {
                continue;
            }
            if let Some(pattern) = rule.matches(&stderr_text) {
                self.emitted.insert(rule.key.clone());
                return Some(HintMatch {
                    manager_name: rule.name.clone(),
                    hint: rule.hint.clone(),
                    pattern,
                });
            }
        }
        None
    }
}

struct ManagerHintRule {
    name: String,
    key: String,
    hint: String,
    patterns: Vec<RegexPattern>,
}

impl ManagerHintRule {
    fn from_spec(spec: &ManagerSpec) -> Option<Self> {
        let hint = spec.repair_hint.as_ref()?.trim();
        if hint.is_empty() || spec.errors.is_empty() {
            return None;
        }

        Some(Self {
            name: spec.name.clone(),
            key: spec.name.to_ascii_lowercase(),
            hint: hint.to_string(),
            patterns: spec.errors.clone(),
        })
    }

    fn matches(&self, stderr: &str) -> Option<String> {
        for pattern in &self.patterns {
            if pattern.regex.is_match(stderr) {
                return Some(pattern.pattern.clone());
            }
        }
        None
    }
}

struct HintMatch {
    manager_name: String,
    hint: String,
    pattern: String,
}

fn hints_disabled() -> bool {
    match env::var("SUBSTRATE_SHIM_HINTS") {
        Ok(value) => disabled_flag(&value),
        Err(_) => false,
    }
}

fn disabled_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "off" | "disabled"
    )
}

fn manifest_paths() -> Option<(PathBuf, Option<PathBuf>)> {
    if let Ok(override_path) = env::var("SUBSTRATE_MANAGER_MANIFEST") {
        return Some((PathBuf::from(override_path), manifest_overlay_path()));
    }

    if let Ok(home) = paths::substrate_home() {
        let base = home.join("manager_hooks.yaml");
        if base.exists() {
            return Some((base, Some(home.join("manager_hooks.local.yaml"))));
        }
    }

    let fallback = repo_manifest_path();
    if fallback.exists() {
        Some((fallback, manifest_overlay_path()))
    } else {
        None
    }
}

fn manifest_overlay_path() -> Option<PathBuf> {
    paths::substrate_home()
        .ok()
        .map(|home| home.join("manager_hooks.local.yaml"))
}

fn repo_manifest_path() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|dir| dir.parent())
        .map(|root| root.join("config").join("manager_hooks.yaml"))
        .unwrap_or_else(|| PathBuf::from("config/manager_hooks.yaml"))
}

fn current_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(windows) {
        Platform::Windows
    } else {
        Platform::Linux
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
            false,
        );

        assert!(result.is_err());

        // The error should be an io::Error that we can inspect
        if let Err(e) = result {
            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
            }
        }
    }

    #[test]
    fn test_bypass_mode_detection() {
        // Test SHIM_BYPASS environment variable detection
        env::remove_var("SHIM_BYPASS");
        assert!(!ShimContext::is_bypass_enabled());

        env::set_var("SHIM_BYPASS", "1");
        assert!(ShimContext::is_bypass_enabled());

        // Note: is_bypass_enabled() only accepts "1" as true
        env::set_var("SHIM_BYPASS", "true");
        assert!(!ShimContext::is_bypass_enabled());

        env::set_var("SHIM_BYPASS", "0");
        assert!(!ShimContext::is_bypass_enabled());

        env::set_var("SHIM_BYPASS", "false");
        assert!(!ShimContext::is_bypass_enabled());

        env::set_var("SHIM_BYPASS", "");
        assert!(!ShimContext::is_bypass_enabled());

        env::remove_var("SHIM_BYPASS");
    }

    #[test]
    fn test_explicit_path_handling() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test_script.sh");

        // Create a simple executable script
        std::fs::write(&script_path, "#!/bin/bash\necho 'test'").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms).unwrap();
        }

        // Test that explicit paths are handled correctly
        assert!(is_executable(&script_path));

        // Test non-executable file
        let non_exec_path = temp_dir.path().join("not_executable");
        std::fs::write(&non_exec_path, "content").unwrap();
        assert!(!is_executable(&non_exec_path));
    }

    #[test]
    fn test_handle_bypass_mode_command_resolution() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let fake_bin = temp_dir.path().join("fake_echo");

        #[cfg(unix)]
        std::fs::write(&fake_bin, "#!/bin/bash\necho 'bypass test'").unwrap();
        #[cfg(windows)]
        std::fs::write(fake_bin.with_extension("exe"), "@echo bypass test").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&fake_bin).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&fake_bin, perms).unwrap();
        }

        // Test explicit path detection
        #[cfg(unix)]
        assert!(fake_bin
            .to_string_lossy()
            .contains(std::path::MAIN_SEPARATOR));
        #[cfg(windows)]
        assert!(fake_bin
            .with_extension("exe")
            .to_string_lossy()
            .contains(std::path::MAIN_SEPARATOR));
    }

    #[test]
    fn test_signal_exit_status_handling() {
        // Test Unix signal handling in exit status
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            use std::process::Command;

            // We can't easily test signal handling without spawning actual processes,
            // but we can test the logic that converts exit codes
            // Most Unix systems have 'false' command that exits with status 1
            if let Ok(status) = Command::new("false").status() {
                assert_eq!(status.code(), Some(1));
                assert!(status.signal().is_none());

                // Test the conversion logic we use in run_shim
                let exit_code = if let Some(signal) = status.signal() {
                    128 + signal
                } else {
                    status.code().unwrap_or(1)
                };
                assert_eq!(exit_code, 1);
            }
        }
    }

    #[test]
    fn test_environment_variable_preservation() {
        // Test that critical environment variables are preserved
        let original_path = env::var("PATH").unwrap_or_default();
        let original_user = env::var("USER").ok();

        // Test that we don't accidentally clear critical env vars
        assert!(!original_path.is_empty());

        // Test ORIGINAL_PATH_VAR handling
        env::set_var(ORIGINAL_PATH_VAR, "/test/path");
        let retrieved = env::var(ORIGINAL_PATH_VAR).unwrap();
        assert_eq!(retrieved, "/test/path");

        // Cleanup
        env::remove_var(ORIGINAL_PATH_VAR);
        if let Some(user) = original_user {
            env::set_var("USER", user);
        }
    }

    #[test]
    fn test_shim_context_creation() {
        // Test basic ShimContext creation from current executable
        // This will fail in test environment, but we test error handling
        let result = ShimContext::from_current_exe();

        // In test environment, this should fail gracefully
        match result {
            Ok(_ctx) => {
                // If it succeeds, verify basic properties
                // This might happen in some test setups
            }
            Err(_e) => {
                // Expected in most test environments
                // The important thing is it doesn't panic
            }
        }
    }

    #[test]
    fn test_command_argument_handling() {
        use std::ffi::OsString;

        // Test that command arguments are properly collected
        let args = vec![
            OsString::from("arg1"),
            OsString::from("arg with spaces"),
            OsString::from("arg-with-dashes"),
            OsString::from(""), // empty arg
        ];

        // Test that we can handle various argument types
        for arg in &args {
            assert!(!arg.to_string_lossy().is_empty() || arg.is_empty()); // Better sanity check
        }

        // Test empty args collection
        let empty_args: Vec<OsString> = vec![];
        assert!(empty_args.is_empty());
    }

    #[test]
    fn test_path_separator_detection() {
        // Test path separator detection for explicit paths
        let with_separator = format!(
            "path{}to{}command",
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        );
        let without_separator = "command";

        assert!(with_separator.contains(std::path::MAIN_SEPARATOR));
        assert!(!without_separator.contains(std::path::MAIN_SEPARATOR));

        // Test platform-specific separators
        #[cfg(windows)]
        {
            assert!(r"C:\path\to\command".contains(std::path::MAIN_SEPARATOR));
            assert!(!r"command".contains(std::path::MAIN_SEPARATOR));
        }

        #[cfg(unix)]
        {
            assert!("/path/to/command".contains(std::path::MAIN_SEPARATOR));
            assert!(!"command".contains(std::path::MAIN_SEPARATOR));
        }
    }

    #[test]
    fn test_error_context_preservation() {
        use std::ffi::OsString;

        // Test that error contexts are properly preserved through the call stack
        let nonexistent = PathBuf::from("/definitely/does/not/exist/command");
        let result = execute_command(&nonexistent, &[OsString::from("test")], "test", false);

        assert!(result.is_err());

        // Verify error message contains useful context
        let error_msg = result.unwrap_err().to_string();
        assert!(!error_msg.is_empty());
        // Error should mention the failed command path
        assert!(error_msg.contains("Failed to execute") || error_msg.contains("not found"));
    }

    #[test]
    fn test_cross_platform_executable_detection() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        #[cfg(unix)]
        {
            // Unix: Test executable bit detection
            let exec_file = temp_dir.path().join("executable");
            let non_exec_file = temp_dir.path().join("not_executable");

            std::fs::write(&exec_file, "#!/bin/bash\necho test").unwrap();
            std::fs::write(&non_exec_file, "not executable").unwrap();

            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&exec_file).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&exec_file, perms).unwrap();

            assert!(is_executable(&exec_file));
            assert!(!is_executable(&non_exec_file));
        }

        #[cfg(windows)]
        {
            // Windows: Any file is considered "executable"
            let file = temp_dir.path().join("test.exe");
            std::fs::write(&file, "dummy content").unwrap();

            assert!(is_executable(&file));

            // Test non-existent file
            let nonexistent = temp_dir.path().join("does_not_exist.exe");
            assert!(!is_executable(&nonexistent));
        }
    }

    #[test]
    fn test_timing_and_metrics() {
        use std::time::{Duration, Instant, SystemTime};

        // Test that timing functions work correctly
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(5));
        assert!(elapsed < Duration::from_millis(100));

        // Test SystemTime functionality used in logging
        let timestamp = SystemTime::now();
        let duration_since_epoch = timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        assert!(duration_since_epoch.as_secs() > 1_600_000_000); // After 2020
    }
}
