//! Structured logging with JSONL format and session correlation
//!
//! This module handles all logging functionality including structured JSONL output,
//! credential redaction, and session correlation for command chains.

use anyhow::Result;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::path::Path;
use std::process::ExitStatus;
use std::time::{Duration, SystemTime};

use crate::context::ShimContext;
use substrate_trace::{append_to_trace, init_trace};

/// Log a command execution with full context
pub fn log_execution(
    log_path: &Path,
    ctx: &ShimContext,
    args: &[std::ffi::OsString],
    status: &ExitStatus,
    duration: Duration,
    timestamp: SystemTime,
    resolved_path: &Path,
    manager_hint: Option<&Value>,
) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/unknown"));

    let pid = std::process::id();
    let hostname = gethostname::gethostname().to_string_lossy().to_string();

    // Redact sensitive arguments with flag-value awareness
    let argv: Vec<String> = std::iter::once(ctx.command_name.clone())
        .chain(redact_sensitive_argv(args))
        .collect();

    // Capture signal information on Unix systems
    #[cfg(unix)]
    let (exit_code, term_signal) = {
        use std::os::unix::process::ExitStatusExt;
        (status.code(), status.signal())
    };
    #[cfg(not(unix))]
    let (exit_code, term_signal) = (status.code(), None::<i32>);

    // Enhanced execution context for debugging
    #[cfg(unix)]
    let ppid = {
        use nix::unistd::getppid;
        Some(getppid().as_raw())
    };
    #[cfg(not(unix))]
    let ppid = None::<i32>;

    let mut log_entry = json!({
        "ts": format_timestamp(timestamp),
        "command": ctx.command_name,
        "argv": argv,
        "cwd": cwd.to_string_lossy(),
        "exit_code": exit_code.unwrap_or(-1),
        "duration_ms": duration.as_millis(),
        "pid": pid,
        "hostname": hostname,
        "platform": get_platform_info(),
        "component": "shim",
        "depth": ctx.depth,
        "session_id": ctx.session_id,
        "resolved_path": resolved_path.display().to_string(),
        "caller": env::var("SHIM_CALLER").ok(),
        "call_stack": env::var("SHIM_CALL_STACK").ok(),
        "parent_cmd_id": env::var("SHIM_PARENT_CMD_ID").ok(),
        "isatty_stdin": atty::is(atty::Stream::Stdin),
        "isatty_stdout": atty::is(atty::Stream::Stdout),
        "isatty_stderr": atty::is(atty::Stream::Stderr),
        "shim_fingerprint": get_shim_fingerprint(),
        "user": env::var("USER").or_else(|_| env::var("USERNAME")).unwrap_or_else(|_| "unknown".to_string()),
    });

    // Add build version if available
    if let Ok(build) = env::var("SHIM_BUILD") {
        log_entry["build"] = json!(build);
    }

    // Add parent process ID if available
    if let Some(ppid) = ppid {
        log_entry["ppid"] = json!(ppid);
    }

    // Add signal information if process was terminated by signal
    if let Some(signal) = term_signal {
        log_entry["term_signal"] = json!(signal);
    }

    if let Some(hint) = manager_hint {
        log_entry["manager_hint"] = hint.clone();
    }

    write_log_entry(log_path, &log_entry)
}

/// Helper function for writing log entries with optional fsync
pub fn write_log_entry(_log_path: &Path, entry: &Value) -> Result<()> {
    // Initialize trace if not already set up (no-op if already initialized)
    let _ = init_trace(None);
    // Ensure single-line JSON (append_to_trace expects a single Value and handles flushing/rotation)
    append_to_trace(entry)
}

/// Redact sensitive command-line arguments
pub fn redact_sensitive_argv(argv: &[std::ffi::OsString]) -> Vec<String> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < argv.len() {
        let arg = argv[i].to_string_lossy();
        let redacted_arg = redact_sensitive(&arg);
        result.push(redacted_arg.clone());

        // If this argument was a sensitive flag, redact the next argument too
        if redacted_arg == "***" && i + 1 < argv.len() {
            let next_arg = argv[i + 1].to_string_lossy();

            // Special handling for header flags - apply header-specific redaction
            if arg.eq_ignore_ascii_case("-H") || arg.eq_ignore_ascii_case("--header") {
                result.push(redact_header_value(&next_arg));
            } else {
                result.push("***".to_string());
            }
            i += 2; // Skip the next argument
        } else {
            i += 1;
        }
    }

    result
}

/// Redact sensitive information from individual arguments
fn redact_sensitive(arg: &str) -> String {
    // Skip redaction if SHIM_LOG_OPTS=raw is set
    if env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        return arg.to_string();
    }

    // Enhanced redaction for key=value patterns
    if arg.contains("token=")
        || arg.contains("password=")
        || arg.contains("secret=")
        || arg.contains("key=")
        || arg.contains("TOKEN=")
        || arg.contains("PASSWORD=")
        || arg.contains("SECRET=")
        || arg.contains("KEY=")
        || arg.contains("apikey=")
        || arg.contains("access-key=")
        || arg.contains("secret-key=")
    {
        if let Some(eq_pos) = arg.find('=') {
            return format!("{}=***", &arg[..eq_pos]);
        }
    }

    // Flag-value pattern redaction for common CLI tools
    const SENSITIVE_FLAGS: &[&str] = &[
        "--token",
        "--password",
        "--secret",
        "-p",
        "--apikey",
        "--access-key",
        "--secret-key",
        "--auth-token",
        "--bearer-token",
        "--api-token",
        "-H",
        "--header",
    ];

    for flag in SENSITIVE_FLAGS {
        if arg.eq_ignore_ascii_case(flag) {
            return "***".to_string();
        }
    }

    arg.to_string()
}

/// Redact sensitive header values
fn redact_header_value(header_value: &str) -> String {
    // Skip redaction if SHIM_LOG_OPTS=raw is set
    if env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        return header_value.to_string();
    }

    // Handle key:value header format
    if let Some((key, _value)) = header_value.split_once(':') {
        let key_lower = key.trim().to_ascii_lowercase();
        const SENSITIVE_HEADER_KEYS: &[&str] = &[
            "authorization",
            "x-api-key",
            "x-auth-token",
            "x-access-token",
            "cookie",
            "set-cookie",
            "x-csrf-token",
            "x-session-token",
        ];

        if SENSITIVE_HEADER_KEYS
            .iter()
            .any(|&sensitive| key_lower == sensitive)
        {
            return format!("{}: ***", key.trim());
        }
    }

    // Redact common Bearer token patterns
    let lower_value = header_value.to_ascii_lowercase();
    if lower_value.contains("authorization:") && lower_value.contains("bearer ") {
        return "Authorization: ***".to_string();
    }

    // Redact other token patterns in headers
    if lower_value.contains("token")
        || lower_value.contains("key")
        || lower_value.contains("secret")
    {
        // Simple heuristic: if it looks like a credential header, redact the value part
        if let Some((key, _)) = header_value.split_once(':') {
            return format!("{}: ***", key.trim());
        }
    }

    header_value.to_string()
}

/// Format timestamp as RFC3339 with milliseconds
pub fn format_timestamp(timestamp: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Utc> = timestamp.into();
    dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Get platform information
fn get_platform_info() -> String {
    std::env::consts::OS.to_string()
}

/// Get shim binary fingerprint for integrity verification  
pub fn get_shim_fingerprint() -> String {
    use once_cell::sync::Lazy;

    static SHIM_FINGERPRINT: Lazy<String> = Lazy::new(|| {
        env::current_exe()
            .and_then(|exe| std::fs::read(&exe))
            .map(|bytes| {
                let hash = Sha256::digest(&bytes);
                format!("sha256:{hash:x}")
            })
            .unwrap_or_else(|_| "sha256:unknown".to_string())
    });

    SHIM_FINGERPRINT.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::ffi::OsString;

    #[test]
    #[serial]
    fn test_sensitive_arg_redaction() {
        // Ensure clean test environment
        env::remove_var("SHIM_LOG_OPTS");

        assert_eq!(redact_sensitive("normal_arg"), "normal_arg");
        assert_eq!(redact_sensitive("token=secret123"), "token=***");
        assert_eq!(redact_sensitive("password=mypass"), "password=***");
        assert_eq!(redact_sensitive("SECRET=topsecret"), "SECRET=***");
        assert_eq!(redact_sensitive("--token"), "***");
        assert_eq!(redact_sensitive("--password"), "***");
        assert_eq!(redact_sensitive("-p"), "***");

        // Test with SHIM_LOG_OPTS=raw
        env::set_var("SHIM_LOG_OPTS", "raw");
        assert_eq!(redact_sensitive("token=secret123"), "token=secret123");
        env::remove_var("SHIM_LOG_OPTS");
    }

    #[test]
    fn test_flag_value_redaction() {
        let args = vec![
            OsString::from("--token"),
            OsString::from("secret123"),
            OsString::from("--url"),
            OsString::from("https://example.com"),
        ];

        let redacted = redact_sensitive_argv(&args);
        assert_eq!(redacted, vec!["***", "***", "--url", "https://example.com"]);
    }

    #[test]
    #[serial]
    fn test_header_value_redaction() {
        // Ensure clean test environment
        env::remove_var("SHIM_LOG_OPTS");

        assert_eq!(
            redact_header_value("Content-Type: application/json"),
            "Content-Type: application/json"
        );
        assert_eq!(
            redact_header_value("Authorization: Bearer token123"),
            "Authorization: ***"
        );
        assert_eq!(
            redact_header_value("X-API-Key: secret123"),
            "X-API-Key: ***"
        );
        assert_eq!(redact_header_value("Cookie: session=abc123"), "Cookie: ***");

        // Test with SHIM_LOG_OPTS=raw
        env::set_var("SHIM_LOG_OPTS", "raw");
        assert_eq!(
            redact_header_value("Authorization: Bearer token123"),
            "Authorization: Bearer token123"
        );
        env::remove_var("SHIM_LOG_OPTS");
    }

    #[test]
    fn test_header_flag_redaction() {
        let args = vec![
            OsString::from("-H"),
            OsString::from("Authorization: Bearer secret123"),
            OsString::from("--header"),
            OsString::from("X-API-Key: mykey456"),
            OsString::from("--url"),
            OsString::from("https://api.example.com"),
        ];

        let redacted = redact_sensitive_argv(&args);
        assert_eq!(
            redacted,
            vec![
                "***",
                "Authorization: ***",
                "***",
                "X-API-Key: ***",
                "--url",
                "https://api.example.com"
            ]
        );
    }

    #[test]
    fn test_binary_fingerprint() {
        // Test that fingerprint is generated and has correct format
        let fingerprint = get_shim_fingerprint();
        assert!(fingerprint.starts_with("sha256:"));

        // Should be sha256: + 64 hex characters
        if fingerprint != "sha256:unknown" {
            assert_eq!(fingerprint.len(), 71); // "sha256:" + 64 chars
            assert!(fingerprint.chars().skip(7).all(|c| c.is_ascii_hexdigit()));
        }
    }
}
