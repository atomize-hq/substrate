# Substrate: Command Tracing Implementation Guide

## Project Overview

A production-ready Rust implementation for tracing all command execution (binaries, built-ins, pipelines) with zero shell recursion and minimal overhead. Designed for comprehensive command observability and monitoring.

## Architecture

### Core Components

```
substrate/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── shim/               # Core shim binary (Phase 1)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs     # Thin binary entry point
│   │       ├── lib.rs      # Core shim logic
│   │       ├── resolver.rs # Clean path resolution
│   │       ├── logger.rs   # JSONL structured logging
│   │       └── exec.rs     # Cross-platform execution
│   ├── supervisor/         # Launch manager (Phase 2)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       └── env.rs      # Environment preparation
│   └── shell/              # Custom REPL (Phase 3)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── lib.rs
│           ├── pty.rs      # PTY management
│           ├── builtins.rs # Built-in commands
│           └── session.rs  # Session management
├── scripts/
│   ├── stage_shims.sh      # Shim deployment
│   ├── rollback.sh         # Emergency rollback
│   └── create_bashenv.sh
└── docs/
    └── ops.md              # Operations guide
```

## Phase 1: Shim Implementation

### Core Shim Logic (`crates/shim/src/lib.rs`)

```rust
use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::RwLock;
use std::time::{Instant, SystemTime};

pub const SHIM_ACTIVE_VAR: &str = "SHIM_ACTIVE";
pub const SHIM_DEPTH_VAR: &str = "SHIM_DEPTH";
pub const SHIM_SESSION_VAR: &str = "SHIM_SESSION_ID";
pub const ORIGINAL_PATH_VAR: &str = "ORIGINAL_PATH";
pub const TRACE_LOG_VAR: &str = "TRACE_LOG_FILE";
pub const CACHE_BUST_VAR: &str = "SHIM_CACHE_BUST";

// Resolution cache to avoid repeated stat() calls
static RESOLUTION_CACHE: Lazy<RwLock<HashMap<String, Option<PathBuf>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
pub struct ShimContext {
    pub command_name: String,
    pub shim_dir: PathBuf,
    pub search_paths: Vec<PathBuf>,
    pub log_file: Option<PathBuf>,
}

impl ShimContext {
    pub fn from_current_exe() -> Result<Self> {
        let exe = env::current_exe()
            .context("Failed to get current executable path")?;

        let shim_dir = exe.parent()
            .ok_or_else(|| anyhow!("Executable has no parent directory"))?
            .to_path_buf();

        let command_name = exe.file_name()
            .ok_or_else(|| anyhow!("Executable has no filename"))?
            .to_string_lossy()
            .to_string();

        let original_path = env::var(ORIGINAL_PATH_VAR).ok();
        let search_paths = build_clean_search_path(&shim_dir, original_path)?;

        let log_file = env::var(TRACE_LOG_VAR)
            .ok()
            .map(PathBuf::from);

        Ok(Self {
            command_name,
            shim_dir,
            search_paths,
            log_file,
        })
    }
}

// Binary integrity fingerprint for forensics and compliance
static SHIM_FINGERPRINT: Lazy<String> = Lazy::new(|| {
    env::current_exe()
        .ok()
        .and_then(|path| std::fs::read(path).ok())
        .map(|bytes| {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&bytes);
            let hash = hasher.finalize();
            format!("sha256:{:x}", hash)
        })
        .unwrap_or_else(|| "sha256:unknown".to_string())
});

pub fn run_shim() -> Result<i32> {
    // Early escape hatch for debugging and sensitive sessions
    if env::var("SHIM_BYPASS").as_deref() == Ok("1") {
        let ctx = ShimContext::from_current_exe()?;
        let args: Vec<_> = env::args_os().skip(1).collect();

        // Resolve the real binary (same logic as normal execution)
        let real_binary = if ctx.command_name.contains(std::path::MAIN_SEPARATOR) {
            // Explicit path - don't search PATH
            let path = PathBuf::from(&ctx.command_name);
            if is_executable(&path) {
                path
            } else {
                return Err(anyhow!("SHIM_BYPASS: Command '{}' not executable", ctx.command_name));
            }
        } else {
            // Search PATH
            resolve_real_binary(&ctx.command_name, &ctx.search_paths)
                .ok_or_else(|| anyhow!("SHIM_BYPASS: Command '{}' not found in PATH", ctx.command_name))?
        };

        // Direct execution without logging
        let status = Command::new(&real_binary)
            .arg0(&ctx.command_name)  // Preserve argv[0] semantics
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

        return Ok(status.code().unwrap_or(1));
    }

    // Track execution depth and session for hierarchical traceability
    let shim_depth = env::var(SHIM_DEPTH_VAR)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    // Generate or inherit session ID for command chain correlation
    let session_id = env::var(SHIM_SESSION_VAR)
        .unwrap_or_else(|_| uuid::Uuid::now_v7().to_string());

    env::set_var(SHIM_SESSION_VAR, &session_id);
    env::set_var(SHIM_DEPTH_VAR, (shim_depth + 1).to_string());

    // Keep legacy SHIM_ACTIVE for backward compatibility
    env::set_var(SHIM_ACTIVE_VAR, "1");

    let ctx = ShimContext::from_current_exe()?;

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
    }.ok_or_else(|| anyhow!("Command '{}' not found", ctx.command_name))?;

    // Prepare execution context
    let args: Vec<_> = env::args_os().skip(1).collect();
    let start_time = Instant::now();
    let timestamp = SystemTime::now();

    // Environment variables already set above

    // Execute the real command with spawn failure telemetry
    let status = match execute_command(&real_binary, &args, &ctx.command_name) {
        Ok(status) => status,
        Err(e) => {
            // Log spawn failure with detailed error information
            if let Some(log_path) = &ctx.log_file {
                let spawn_error = e.downcast_ref::<std::io::Error>();
                let mut error_entry = json!({
                    "ts": format_timestamp(timestamp),
                    "command": ctx.command_name,
                    "resolved_path": real_binary.display().to_string(),
                    "error": "spawn_failed",
                    "depth": shim_depth,
                    "session_id": session_id,
                    "shim_fingerprint": SHIM_FINGERPRINT.as_str()
                });

                if let Some(io_err) = spawn_error {
                    error_entry["spawn_error_kind"] = json!(format!("{:?}", io_err.kind()));
                    if let Some(errno) = io_err.raw_os_error() {
                        error_entry["spawn_errno"] = json!(errno);
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
        if let Err(e) = log_execution(log_path, &ctx, &args, &status, duration, timestamp, shim_depth, &session_id, &real_binary) {
            eprintln!("Warning: Failed to log execution: {}", e);
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

fn build_clean_search_path(shim_dir: &Path, original_path: Option<String>) -> Result<Vec<PathBuf>> {
    let path_str = original_path
        .or_else(|| env::var("PATH").ok())
        .ok_or_else(|| anyhow!("No PATH or ORIGINAL_PATH found"))?;

    let separator = if cfg!(windows) { ';' } else { ':' };

    // Helper to validate PATH entries
    fn is_good_dir(p: &str) -> bool {
        let pb = std::path::Path::new(p);
        pb.is_absolute() && pb.is_dir()
    }

    // Deduplicate PATH entries for predictable resolution and fewer stats
    let mut seen = std::collections::HashSet::new();
    let paths: Vec<PathBuf> = path_str
        .split(separator)
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches('/'))
        .filter(|p| !Path::new(p).starts_with(shim_dir))
        .filter(|p| is_good_dir(p))  // Validate paths
        .filter(|p| seen.insert(p.to_string()))
        .map(PathBuf::from)
        .collect();

    if paths.is_empty() {
        return Err(anyhow!("No valid search paths found after filtering"));
    }

    Ok(paths)
}

fn resolve_real_binary(command: &str, search_paths: &[PathBuf]) -> Option<PathBuf> {
    // Check cache first (unless cache busting is enabled)
    if env::var(CACHE_BUST_VAR).is_err() {
        let cache_key = build_cache_key(command, search_paths);

        if let Ok(cache) = RESOLUTION_CACHE.read() {
            if let Some(cached_result) = cache.get(&cache_key) {
                return cached_result.clone();
            }
        }
    }

    // Perform resolution
    let result = resolve_binary_uncached(command, search_paths);

    // Cache the result (unless cache busting is enabled)
    if env::var(CACHE_BUST_VAR).is_err() {
        let cache_key = build_cache_key(command, search_paths);

        if let Ok(mut cache) = RESOLUTION_CACHE.write() {
            cache.insert(cache_key, result.clone());
        }
    }

    result
}

fn build_cache_key(command: &str, search_paths: &[PathBuf]) -> String {
    // Cache key based only on command name and search paths
    // PATH resolution doesn't depend on CWD, so including it would reduce cache hit rate

    // Normalize paths by trimming trailing slashes
    let normalized_paths: Vec<String> = search_paths.iter()
        .map(|p| p.display().to_string().trim_end_matches('/').to_string())
        .collect();

    format!("{}:{}", command, normalized_paths.join(":"))
}

fn resolve_binary_uncached(command: &str, search_paths: &[PathBuf]) -> Option<PathBuf> {
    for dir in search_paths {
        let candidate = dir.join(command);

        // On Windows, try with common executable extensions
        #[cfg(windows)]
        {
            let extensions = env::var("PATHEXT")
                .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());

            for ext in extensions.split(';') {
                if !ext.is_empty() {
                    let mut path_with_ext = candidate.clone();
                    path_with_ext.set_extension(&ext[1..]); // Remove leading dot
                    if is_executable(&path_with_ext) {
                        return Some(path_with_ext);
                    }
                }
            }
        }

        // Unix or Windows without extension
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let mode = metadata.permissions().mode();
            return metadata.is_file() && (mode & 0o111) != 0;
        }
        false
    }
    #[cfg(windows)]
    {
        path.is_file()
    }
}

fn execute_command(binary: &Path, args: &[std::ffi::OsString], command_name: &str) -> Result<ExitStatus> {
    let status = Command::new(binary)
        .arg0(command_name)  // Preserve argv[0] semantics for tools that check invocation name
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute {}", binary.display()))?;

    Ok(status)
}

fn log_execution(
    log_path: &Path,
    ctx: &ShimContext,
    args: &[std::ffi::OsString],
    status: &ExitStatus,
    duration: std::time::Duration,
    timestamp: SystemTime,
    depth: u32,
    session_id: &str,
    resolved_path: &Path,
) -> Result<()> {
    let cwd = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("/unknown"));

    let pid = std::process::id();
    let hostname = gethostname::gethostname()
        .to_string_lossy()
        .to_string();

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
    let ppid = nix::unistd::getppid().as_raw();
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
        "ppid": ppid,
        "user": env::var("USER").or_else(|_| env::var("USERNAME")).unwrap_or_else(|_| "unknown".to_string()),
        "host": hostname,
        "platform": env::consts::OS,
        "component": "shim",
        "depth": depth,
        "session_id": session_id,
        "resolved_path": resolved_path.display().to_string(),
        "isatty_stdin": atty::is(atty::Stream::Stdin),
        "isatty_stdout": atty::is(atty::Stream::Stdout),
        "isatty_stderr": atty::is(atty::Stream::Stderr),
        "shim_fingerprint": SHIM_FINGERPRINT.as_str()
    });

    // Add build version if available
    if let Ok(build) = env::var("SHIM_BUILD") {
        log_entry["build"] = json!(build);
    }

    // Add signal information if process was terminated by signal
    if let Some(signal) = term_signal {
        log_entry["term_signal"] = json!(signal);
    }

    // Ensure log directory exists
    if let Some(dir) = log_path.parent() {
        std::fs::create_dir_all(dir).ok();
    }

    // Serialize to string and write atomically
    let log_line = format!("{}\n", log_entry);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("Failed to open log file: {}", log_path.display()))?;

    // Set user-only permissions on first creation (Unix)
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(log_path, Permissions::from_mode(0o600));
    }

    file.write_all(log_line.as_bytes())
        .context("Failed to write log entry")?;

    Ok(())
}

// Helper function for writing log entries with optional fsync
fn write_log_entry(log_path: &Path, entry: &serde_json::Value) -> Result<()> {
    // Ensure log directory exists
    if let Some(dir) = log_path.parent() {
        std::fs::create_dir_all(dir).ok();
    }

    // Ensure single-line JSON by escaping newlines
    let mut line = entry.to_string();
    if line.contains('\n') {
        line = line.replace('\n', "\\n");
    }
    line.push('\n');

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("Failed to open log file: {}", log_path.display()))?;

    // Set user-only permissions on Unix
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(log_path, Permissions::from_mode(0o600));
    }

    file.write_all(line.as_bytes())
        .context("Failed to write log entry")?;

    // Optional fsync for maximum durability
    if env::var("SHIM_FSYNC").as_deref() == Ok("1") {
        use std::io::Write as _;
        file.flush().ok();
        let _ = file.sync_all();
    }

    Ok(())
}

fn redact_sensitive(arg: &str) -> String {
    // Skip redaction if SHIM_LOG_OPTS=raw is set
    if env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        return arg.to_string();
    }

    // Enhanced redaction for key=value patterns
    if arg.contains("token=") || arg.contains("password=") || arg.contains("secret=")
        || arg.contains("key=") || arg.contains("TOKEN=") || arg.contains("PASSWORD=")
        || arg.contains("SECRET=") || arg.contains("KEY=") || arg.contains("apikey=")
        || arg.contains("access-key=") || arg.contains("secret-key=") {
        if let Some(eq_pos) = arg.find('=') {
            return format!("{}=***", &arg[..eq_pos]);
        }
    }

    // Flag-value pattern redaction for common CLI tools
    const SENSITIVE_FLAGS: &[&str] = &[
        "--token", "--password", "--secret", "-p", "--apikey",
        "--access-key", "--secret-key", "--auth-token",
        "--bearer-token", "--api-token", "-H", "--header",
        "--data-raw", "--data-binary", "--form", "-u", "--user"
    ];

    for flag in SENSITIVE_FLAGS {
        if arg.eq_ignore_ascii_case(flag) {
            return "***".to_string();
        }
    }

    arg.to_string()
}

fn redact_header_value(header_value: &str) -> String {
    // Skip redaction if SHIM_LOG_OPTS=raw is set
    if env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        return header_value.to_string();
    }

    // Handle key:value header format
    if let Some((key, _value)) = header_value.split_once(':') {
        let key_lower = key.trim().to_ascii_lowercase();
        const SENSITIVE_HEADER_KEYS: &[&str] = &[
            "authorization", "x-api-key", "x-auth-token", "x-access-token",
            "cookie", "set-cookie", "x-csrf-token", "x-session-token"
        ];

        if SENSITIVE_HEADER_KEYS.iter().any(|&sensitive| key_lower == sensitive) {
            return format!("{}: ***", key.trim());
        }
    }

    // Redact common Bearer token patterns
    let lower_value = header_value.to_ascii_lowercase();
    if lower_value.contains("authorization:") && lower_value.contains("bearer ") {
        return "Authorization: Bearer ***".to_string();
    }

    // Redact other token patterns in headers
    if lower_value.contains("token") || lower_value.contains("key") || lower_value.contains("secret") {
        // Simple heuristic: if it looks like a credential header, redact the value part
        if let Some((key, _)) = header_value.split_once(':') {
            return format!("{}: ***", key.trim());
        }
    }

    header_value.to_string()
}

fn redact_sensitive_argv(argv: &[std::ffi::OsString]) -> Vec<String> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < argv.len() {
        let arg = argv[i].to_string_lossy();
        let redacted_arg = redact_sensitive(&arg);
        result.push(redacted_arg.clone());

        // If this argument was a sensitive flag, redact the next argument too
        if redacted_arg == "***" && i + 1 < argv.len() {
            let next_arg = argv[i + 1].to_string_lossy();

            // Special handling for different flag types
            if arg.eq_ignore_ascii_case("-H") || arg.eq_ignore_ascii_case("--header") {
                result.push(redact_header_value(&next_arg));
            } else if arg.eq_ignore_ascii_case("-u") || arg.eq_ignore_ascii_case("--user") {
                // Redact entire user:pass
                result.push("***".to_string());
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

fn format_timestamp(timestamp: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Utc> = timestamp.into();
    dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
```

### Shim Binary (`crates/shim/src/main.rs`)

```rust
//! Substrate command shim - intercepts and logs command execution
//!
//! This binary is copied to multiple names in ~/.cmdshim_rust/ to intercept
//! different commands. It resolves the real binary from a clean PATH and
//! executes it while logging the invocation.

use anyhow::Result;
use substrate_shim::run_shim;

fn main() -> Result<()> {
    let exit_code = run_shim()?;
    std::process::exit(exit_code);
}
```

### Shim Cargo.toml (`crates/shim/Cargo.toml`)

```toml
[package]
name = "substrate-shim"
version = "0.1.1"
edition = "2021"
rust-version = "1.74"
authors = ["Your Name <you@example.com>"]
description = "Command execution shim for tracing"
license = "MIT OR Apache-2.0"

[[bin]]
name = "shim"
path = "src/main.rs"

[dependencies]
anyhow = "1.0"
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
gethostname = "0.4"
once_cell = "1.19"
uuid = { version = "1.10", features = ["v7"] }
atty = "0.2"
sha2 = "0.10"

[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["fs"] }

[dev-dependencies]
tempfile = "3.0"
which = "6"
```

## Phase 2: Supervisor Implementation

### Supervisor Library (`crates/supervisor/src/lib.rs`)

```rust
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct SupervisorConfig {
    pub shim_dir: PathBuf,
    pub original_path: String,
    pub bash_env_file: Option<PathBuf>,
    pub target_command: Vec<String>,
    pub environment: HashMap<String, String>,
}

impl SupervisorConfig {
    pub fn new(target_command: Vec<String>) -> Result<Self> {
        let home = env::var("HOME")
            .context("HOME environment variable not set")?;

        let shim_dir = PathBuf::from(home).join(".cmdshim_rust");

        // Build clean original path (common macOS/Linux paths)
        let original_path = build_default_path(&home)?;

        let bash_env_file = Some(PathBuf::from(home).join(".substrate_bashenv"));

        Ok(Self {
            shim_dir,
            original_path,
            bash_env_file,
            target_command,
            environment: HashMap::new(),
        })
    }

    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }
}

pub fn launch_supervised(config: SupervisorConfig) -> Result<()> {
    // Prepare environment
    let mut cmd = Command::new(&config.target_command[0]);

    if config.target_command.len() > 1 {
        cmd.args(&config.target_command[1..]);
    }

    // Set up clean environment with session seeding
    let session_id = env::var("SHIM_SESSION_ID")
        .unwrap_or_else(|_| uuid::Uuid::now_v7().to_string());
    cmd.env("SHIM_SESSION_ID", &session_id);
    cmd.env("SHIM_BUILD", env!("CARGO_PKG_VERSION"));
    cmd.env("ORIGINAL_PATH", &config.original_path);

    // Build shimmed PATH with deduplication
    let shimmed_path = format!("{}:{}",
        config.shim_dir.display(),
        config.original_path
    );
    cmd.env("PATH", dedupe_path(&shimmed_path));

    // Set BASH_ENV for non-interactive shells
    if let Some(bash_env) = &config.bash_env_file {
        if bash_env.exists() {
            cmd.env("BASH_ENV", bash_env);
        }
    }

    // Apply additional environment variables
    for (key, value) in &config.environment {
        cmd.env(key, value);
    }

    // Inherit stdio for interactive use
    cmd.stdin(Stdio::inherit())
       .stdout(Stdio::inherit())
       .stderr(Stdio::inherit());

    let mut child = cmd.spawn()
        .context("Failed to spawn target command")?;

    let status = child.wait()
        .context("Failed to wait for target command")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn build_default_path(home: &str) -> Result<String> {
    // Start with parent PATH if available, otherwise use common paths
    if let Ok(parent_path) = env::var("PATH") {
        let shim_dir = format!("{}/.cmdshim_rust", home);
        Ok(strip_shim_dir_from_path(&parent_path, &shim_dir))
    } else {
        // Fallback to common paths for macOS/Linux development environments
        let paths = vec![
            format!("{}/.nvm/versions/node/v22.16.0/bin", home),
            "/opt/homebrew/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
            "/usr/sbin".to_string(),
            "/sbin".to_string(),
            format!("{}/.bun/bin", home),
            format!("{}/.cargo/bin", home),
        ];
        Ok(paths.join(":"))
    }
}

fn strip_shim_dir_from_path(path: &str, shim_dir: &str) -> String {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let shim_dir_normalized = shim_dir.trim_end_matches('/');

    // Helper to validate PATH entries
    fn is_good_dir(p: &str) -> bool {
        let pb = std::path::Path::new(p);
        pb.is_absolute() && pb.is_dir()
    }

    path.split(separator)
        .filter(|s| !s.is_empty())
        .filter(|p| p.trim_end_matches('/') != shim_dir_normalized)
        .filter(|p| is_good_dir(p))  // Validate paths
        .collect::<Vec<_>>()
        .join(&separator.to_string())
}

fn dedupe_path(path: &str) -> String {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::new();

    for component in path.split(separator) {
        if !component.is_empty() {
            let canonical = component.trim_end_matches('/');
            if seen.insert(canonical.to_string()) {
                deduped.push(component);
            }
        }
    }

    deduped.join(&separator.to_string())
}
```
