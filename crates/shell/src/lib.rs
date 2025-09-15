pub mod lock;
mod pty_exec;
pub mod shim_deploy; // Made public for integration tests

use pty_exec::execute_with_pty;
use shim_deploy::{DeploymentStatus, ShimDeployer};

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use lazy_static::lazy_static;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use substrate_broker::{detect_profile, evaluate, Decision};
use substrate_common::{dedupe_path, log_schema, redact_sensitive};
use substrate_trace::{append_to_trace, create_span_builder, init_trace, PolicyDecision};
use uuid::Uuid;

// Reedline imports
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Completer, DefaultValidator, Emacs,
    ExampleHighlighter, FileBackedHistory, KeyCode, KeyModifiers, MenuBuilder, Prompt,
    PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, Reedline, ReedlineEvent,
    ReedlineMenu, Signal, Span, Suggestion,
};
use tokio_tungstenite as tungs;
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(target_os = "linux")]
use tokio::signal::unix::{signal, SignalKind};
use futures::StreamExt as _; // for next()
use futures::SinkExt as _;   // for send()
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
// use nu_ansi_term::{Color, Style}; // Unused for now
use std::borrow::Cow;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;
#[cfg(target_os = "linux")]
use nix::sys::termios::{self, ControlFlags, InputFlags, LocalFlags, OutputFlags, SetArg, SpecialCharacterIndices, Termios};

#[cfg(target_os = "linux")]
fn get_term_size() -> (u16, u16) {
    // Try to read the current terminal size; fall back to 80x24
    let fd = std::io::stdout().as_raw_fd();
    let mut ws: libc::winsize = libc::winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
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
        let file = std::fs::OpenOptions::new().read(true).write(true).open("/dev/tty")?;
        let mut tio = termios::tcgetattr(&file)
            .map_err(|e| anyhow::anyhow!("tcgetattr: {}", e))?;
        let orig = tio.clone();
        // Configure raw mode (manual equivalent of cfmakeraw)
        tio.input_flags.remove(
            InputFlags::BRKINT | InputFlags::ICRNL | InputFlags::INPCK | InputFlags::ISTRIP | InputFlags::IXON,
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
        if !atty::is(atty::Stream::Stdin) {
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

// Global flag to prevent double SIGINT handling - must be pub(crate) for pty_exec access
pub(crate) static PTY_ACTIVE: AtomicBool = AtomicBool::new(false);

// Type alias to simplify complex PTY type
type CurrentPtyType = Arc<Mutex<Option<Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>>>>;

// Global SIGWINCH handler state - must be pub(crate) for pty_exec access
lazy_static! {
    // Store the current PTY as a mutex-wrapped Box<dyn MasterPty + Send>
    // Note: Using nested Mutex to satisfy Sync requirement for global static, since
    // portable_pty::MasterPty is not Sync. The outer mutex protects Option swapping,
    // the inner mutex protects the MasterPty itself. This could be simplified if
    // portable_pty adds Sync to MasterPty trait in the future.
    pub(crate) static ref CURRENT_PTY: CurrentPtyType = Arc::new(Mutex::new(None));
}

// Forward declaration for pty_exec module
#[cfg(unix)]
pub(crate) fn initialize_global_sigwinch_handler() {
    pty_exec::initialize_global_sigwinch_handler_impl();
}

#[cfg(not(unix))]
pub(crate) fn initialize_global_sigwinch_handler() {
    // No-op on non-Unix platforms
}

const BASH_PREEXEC_SCRIPT: &str = r#"# Substrate PTY command logging
# Source user's bashrc ONLY in interactive shells
[[ $- == *i* ]] && [[ -f ~/.bashrc ]] && source ~/.bashrc

__substrate_preexec() {
    [[ -z "$SHIM_TRACE_LOG" ]] && return 0
    [[ "$BASH_COMMAND" == __substrate_preexec* ]] && return 0
    [[ -n "$COMP_LINE" ]] && return 0
    printf '{"ts":"%s","event_type":"builtin_command","command":%q,"session_id":%q,"component":"shell","pty":true}\n' \
        "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)" \
        "$BASH_COMMAND" \
        "${SHIM_SESSION_ID:-unknown}" >> "$SHIM_TRACE_LOG" 2>/dev/null || true
}
trap '__substrate_preexec' DEBUG
"#;

#[derive(Parser, Debug)]
#[command(name = "substrate")]
#[command(version, about = "Substrate shell wrapper with comprehensive tracing", long_about = None)]
pub struct Cli {
    /// Execute a single command
    #[arg(
        short = 'c',
        long = "command",
        value_name = "CMD",
        conflicts_with = "script"
    )]
    pub command: Option<String>,

    /// Execute a script file
    #[arg(
        short = 'f',
        long = "file",
        value_name = "SCRIPT",
        conflicts_with = "command"
    )]
    pub script: Option<PathBuf>,

    /// Enable CI mode with strict error handling
    #[arg(long = "ci")]
    pub ci_mode: bool,

    /// Continue execution after errors (overrides CI mode behavior)
    #[arg(long = "no-exit-on-error")]
    pub no_exit_on_error: bool,

    /// Use PTY for full terminal emulation in interactive mode (Unix only)
    #[cfg_attr(not(unix), arg(hide = true))]
    #[arg(long = "pty")]
    pub use_pty: bool,

    /// Specify shell to use (defaults to $SHELL or /bin/bash)
    #[arg(long = "shell", value_name = "PATH")]
    pub shell: Option<String>,

    /// Output version information as JSON
    #[arg(long = "version-json", conflicts_with_all = &["command", "script"])]
    pub version_json: bool,

    /// Show shim deployment status
    #[arg(long = "shim-status", conflicts_with_all = &["command", "script", "shim_deploy", "shim_remove"])]
    pub shim_status: bool,

    /// Show shim deployment status as JSON (CI-friendly)
    #[arg(long = "shim-status-json", conflicts_with_all = &["command", "script", "shim_deploy", "shim_remove"])]
    pub shim_status_json: bool,

    /// Skip shim deployment check
    #[arg(long = "shim-skip")]
    pub shim_skip: bool,

    /// Force deployment of command shims
    #[arg(long = "shim-deploy", conflicts_with_all = &["command", "script", "shim_remove", "shim_status"])]
    pub shim_deploy: bool,

    /// Remove all deployed shims
    #[arg(long = "shim-remove", conflicts_with_all = &["command", "script", "shim_deploy", "shim_status"])]
    pub shim_remove: bool,

    /// Show trace information for a span ID
    #[arg(long = "trace", value_name = "SPAN_ID", conflicts_with_all = &["command", "script", "shim_deploy", "shim_status", "shim_remove", "replay"])]
    pub trace: Option<String>,

    /// Replay a traced command by span ID
    #[arg(long = "replay", value_name = "SPAN_ID", conflicts_with_all = &["command", "script", "shim_deploy", "shim_status", "shim_remove", "trace"])]
    pub replay: Option<String>,

    /// Verbose output during replay (prints command, cwd, mode, and capability warnings)
    #[arg(long = "replay-verbose", requires = "replay")]
    pub replay_verbose: bool,

    /// Disable world isolation (Linux only)
    #[arg(long = "no-world")]
    pub no_world: bool,

    /// Graph commands (ingest/status/what-changed)
    #[command(subcommand)]
    pub sub: Option<SubCommands>,
}

#[derive(clap::Subcommand, Debug)]
pub enum SubCommands {
    Graph(GraphCmd),
    World(WorldCmd),
}

#[derive(clap::Args, Debug)]
pub struct GraphCmd {
    #[command(subcommand)]
    pub action: GraphAction,
}

#[derive(clap::Subcommand, Debug)]
pub enum GraphAction {
    Ingest {
        file: std::path::PathBuf,
    },
    Status,
    WhatChanged {
        span_id: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
}

#[derive(clap::Args, Debug)]
pub struct WorldCmd {
    #[command(subcommand)]
    pub action: WorldAction,
}

#[derive(clap::Subcommand, Debug)]
pub enum WorldAction {
    Doctor {
        /// Output machine-readable JSON for CI
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone)]
pub enum ShellMode {
    Interactive { use_pty: bool }, // Full REPL with optional PTY
    Wrap(String),                  // Single command execution (-c "cmd")
    Script(PathBuf),               // Script file execution (-f script.sh)
    Pipe,                          // Read commands from stdin
}

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
    pub env_vars: HashMap<String, String>,
}

impl ShellConfig {
    pub fn from_args() -> Result<Self> {
        let cli = Cli::parse();

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
                        use std::path::PathBuf;
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
                        use std::path::PathBuf;
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
                    handle_world_command(world_cmd)?;
                    std::process::exit(0);
                }
            }
        }

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

        // Determine shell to use
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
        let mode = if let Some(cmd) = cli.command {
            ShellMode::Wrap(cmd)
        } else if let Some(script) = cli.script {
            ShellMode::Script(script)
        } else if !atty::is(atty::Stream::Stdin) {
            ShellMode::Pipe
        } else {
            // Ensure PTY is only used on Unix systems
            let use_pty = cli.use_pty && cfg!(unix);
            ShellMode::Interactive { use_pty }
        };

        Ok(ShellConfig {
            mode,
            session_id,
            trace_log_file,
            original_path,
            shim_dir,
            shell_path,
            ci_mode: cli.ci_mode,
            no_exit_on_error: cli.no_exit_on_error,
            skip_shims: cli.shim_skip,
            no_world: cli.no_world,
            env_vars: HashMap::new(),
        })
    }
}

fn handle_graph_command(cmd: &GraphCmd) -> Result<()> {
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

fn handle_world_command(cmd: &WorldCmd) -> Result<()> {
    match &cmd.action {
        WorldAction::Doctor { json } => {
            let code = world_doctor_main(*json);
            std::process::exit(code);
        }
    }
}

fn world_doctor_main(json_mode: bool) -> i32 {
    #[cfg(not(target_os = "linux"))]
    {
        if json_mode {
            let out = json!({
                "platform": std::env::consts::OS,
                "overlay_present": serde_json::Value::Null,
                "fuse": {"dev": serde_json::Value::Null, "bin": serde_json::Value::Null},
                "cgroup_v2": serde_json::Value::Null,
                "nft_present": serde_json::Value::Null,
                "dmesg_restrict": serde_json::Value::Null,
                "overlay_root": serde_json::Value::Null,
                "copydiff_root": serde_json::Value::Null,
                "ok": true
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            eprintln!("substrate world doctor is focused on Linux environments");
            println!("overlay: N/A");
            println!("fuse-overlayfs: N/A");
            println!("cgroup v2: N/A");
            println!("nft: N/A");
            println!("dmesg_restrict: N/A");
        }
        0
    }

    #[cfg(target_os = "linux")]
    {
        use std::path::Path;
        use std::path::PathBuf;

        // Helpers
        fn pass(msg: &str) {
            println!("PASS  | {}", msg);
        }
        fn warn(msg: &str) {
            println!("WARN  | {}", msg);
        }
        // fn fail(msg: &str) { println!("FAIL  | {}", msg); }

        fn overlay_present() -> bool {
            std::fs::read_to_string("/proc/filesystems")
                .ok()
                .map(|s| s.contains("overlay"))
                .unwrap_or(false)
        }

        fn try_modprobe_overlay_if_root() {
            let is_root = unsafe { libc::geteuid() } == 0;
            if !is_root {
                return;
            }
            let _ = Command::new("modprobe").arg("overlay").status();
        }

        fn fuse_dev_present() -> bool {
            Path::new("/dev/fuse").exists()
        }
        fn fuse_bin_present() -> bool {
            which::which("fuse-overlayfs").is_ok()
        }
        fn cgroup_v2_present() -> bool {
            Path::new("/sys/fs/cgroup/cgroup.controllers").exists()
        }
        fn nft_present() -> bool {
            Command::new("nft")
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .ok()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        fn dmesg_restrict() -> Option<String> {
            Command::new("sh")
                .arg("-lc")
                .arg("sysctl -n kernel.dmesg_restrict 2>/dev/null || echo n/a")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        }
        fn overlay_root() -> PathBuf {
            let uid = unsafe { libc::geteuid() } as u32;
            if uid == 0 {
                return PathBuf::from("/var/lib/substrate/overlay");
            }
            if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
                if !xdg.is_empty() {
                    return PathBuf::from(xdg).join("substrate/overlay");
                }
            }
            let run = PathBuf::from(format!("/run/user/{}/substrate/overlay", uid));
            if run.parent().unwrap_or(Path::new("/run")).exists() {
                return run;
            }
            PathBuf::from(format!("/tmp/substrate-{}-overlay", uid))
        }
        fn copydiff_root() -> PathBuf {
            let uid = unsafe { libc::geteuid() } as u32;
            if uid == 0 {
                return PathBuf::from("/var/lib/substrate/copydiff");
            }
            if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
                if !xdg.is_empty() {
                    return PathBuf::from(xdg).join("substrate/copydiff");
                }
            }
            let run = PathBuf::from(format!("/run/user/{}/substrate/copydiff", uid));
            if run.parent().unwrap_or(Path::new("/run")).exists() {
                return run;
            }
            PathBuf::from(format!("/tmp/substrate-{}-copydiff", uid))
        }

        // overlay
        let mut overlay_ok = overlay_present();
        if !json_mode {
            println!("== substrate world doctor ==");
            if overlay_ok {
                pass("overlayfs: present");
            } else {
                warn("overlayfs: not present; attempting modprobe overlay (root only)");
                try_modprobe_overlay_if_root();
                overlay_ok = overlay_present();
                if overlay_ok {
                    pass("overlayfs: present after modprobe");
                } else {
                    warn("overlayfs: unavailable");
                }
            }
        } else {
            // still try modprobe if root to improve signal
            if !overlay_ok {
                try_modprobe_overlay_if_root();
                overlay_ok = overlay_present();
            }
        }

        // fuse
        let fuse_dev = fuse_dev_present();
        let fuse_bin = fuse_bin_present();
        if !json_mode {
            if fuse_dev && fuse_bin {
                pass("fuse-overlayfs: /dev/fuse present and binary found");
            } else if fuse_dev || fuse_bin {
                warn(&format!(
                    "fuse-overlayfs: partial ({}, {})",
                    if fuse_dev {
                        "/dev/fuse"
                    } else {
                        "missing /dev/fuse"
                    },
                    if fuse_bin {
                        "binary found"
                    } else {
                        "missing binary"
                    }
                ));
            } else {
                warn("fuse-overlayfs: not available");
            }
        }

        let cgv2 = cgroup_v2_present();
        let nft = nft_present();
        let dmsg = dmesg_restrict().unwrap_or_else(|| "n/a".to_string());
        let o_root = overlay_root();
        let c_root = copydiff_root();

        if !json_mode {
            if cgv2 {
                pass("cgroup v2: present");
            } else {
                warn("cgroup v2: missing");
            }
            if nft {
                pass("nft: present");
            } else {
                warn("nft: missing");
            }
            println!("INFO  | dmesg_restrict={}", dmsg);
            println!("INFO  | overlay_root: {}", o_root.display());
            println!("INFO  | copydiff_root: {}", c_root.display());
        } else {
            let ok = overlay_ok || (fuse_dev && fuse_bin);
            let out = json!({
                "platform": std::env::consts::OS,
                "overlay_present": overlay_ok,
                "fuse": {"dev": fuse_dev, "bin": fuse_bin},
                "cgroup_v2": cgv2,
                "nft_present": nft,
                "dmesg_restrict": dmsg,
                "overlay_root": o_root,
                "copydiff_root": c_root,
                "ok": ok,
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }

        // Exit code policy
        if overlay_ok || (fuse_dev && fuse_bin) {
            0
        } else {
            2
        }
    }
}

pub fn run_shell() -> Result<i32> {
    let config = ShellConfig::from_args()?;

    // Initialize trace
    if let Err(e) = init_trace(None) {
        eprintln!("substrate: warning: failed to initialize trace: {}", e);
    }

    // Default-on world initialization (Linux only)
    #[cfg(target_os = "linux")]
    {
        use world::LinuxLocalBackend;
        use world_api::{WorldBackend, WorldSpec, ResourceLimits};
        let world_disabled = env::var("SUBSTRATE_WORLD").map(|v| v == "disabled").unwrap_or(false)
            || config.no_world;
        if !world_disabled {
            let spec = WorldSpec {
                reuse_session: true,
                isolate_network: true,
                limits: ResourceLimits::default(),
                enable_preload: false,
                allowed_domains: substrate_broker::allowed_domains(),
                project_dir: env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
                always_isolate: false,  // Default: use heuristic-based isolation
            };
            let backend = LinuxLocalBackend::new();
            match backend.ensure_session(&spec) {
                Ok(handle) => {
                    env::set_var("SUBSTRATE_WORLD", "enabled");
                    env::set_var("SUBSTRATE_WORLD_ID", &handle.id);
                }
                Err(_e) => {
                    // Degrade silently: world may be unavailable in this environment.
                }
            }
        }
    }

    // Deploy shims if needed (non-blocking, continues on error)
    // Skip if either the CLI flag is set or the environment variable is set
    let skip_shims = config.skip_shims || env::var("SUBSTRATE_NO_SHIMS").is_ok();
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

    // Ensure shim directory is in PATH with deduplication (use OS-specific separator)
    let sep = if cfg!(windows) { ';' } else { ':' };
    let path_with_shims = format!(
        "{}{}{}",
        config.shim_dir.display(),
        sep,
        config.original_path
    );
    env::set_var("PATH", dedupe_path(&path_with_shims));

    match &config.mode {
        ShellMode::Interactive { use_pty: _ } => {
            // PTY mode is now handled within run_interactive_shell on a per-command basis
            run_interactive_shell(&config)
        }
        ShellMode::Wrap(cmd) => run_wrap_mode(&config, cmd),
        ShellMode::Script(path) => run_script_mode(&config, path),
        ShellMode::Pipe => run_pipe_mode(&config),
    }
}

fn run_interactive_shell(config: &ShellConfig) -> Result<i32> {
    println!("Substrate v{}", env!("CARGO_PKG_VERSION"));
    println!("Session ID: {}", config.session_id);
    println!("Logging to: {}", config.trace_log_file.display());

    // Set up history file with proper initialization
    let hist_path = dirs::home_dir()
        .map(|p| p.join(".substrate_history"))
        .unwrap_or_else(|| PathBuf::from(".substrate_history"));

    // Ensure history file exists and is accessible
    if !hist_path.exists() {
        std::fs::File::create(&hist_path)?;
    }

    let history = Box::new(
        FileBackedHistory::with_file(100_000, hist_path.clone())
            .expect("Error configuring history file"),
    );

    // Create custom prompt
    let prompt = SubstratePrompt::new(config.ci_mode);

    // Configure keybindings
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::Menu("completion_menu".to_string()),
    );
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('l'),
        ReedlineEvent::ClearScreen,
    );

    // Create completer
    let completer = Box::new(SubstrateCompleter::new(config));

    // Create the line editor
    let edit_mode = Box::new(Emacs::new(keybindings));

    // Create a simple transient prompt for after command execution
    let transient_prompt = SubstratePrompt::new(config.ci_mode);

    let mut line_editor = Reedline::create()
        .with_history(history)
        .with_edit_mode(edit_mode)
        .with_completer(completer)
        .with_highlighter(Box::new(ExampleHighlighter::default()))
        .with_validator(Box::new(DefaultValidator))
        .with_transient_prompt(Box::new(transient_prompt))
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(
            ColumnarMenu::default().with_name("completion_menu"),
        )));

    // Set up the host command decider for PTY commands

    // Signal handling setup
    let running_child_pid = Arc::new(AtomicI32::new(0));
    setup_signal_handlers(running_child_pid.clone())?;

    // Main REPL loop
    loop {
        let sig = line_editor.read_line(&prompt);

        match sig {
            Ok(Signal::Success(line)) => {
                if line.trim().is_empty() {
                    continue;
                }

                // Check for exit commands
                if matches!(line.trim(), "exit" | "quit") {
                    break;
                }

                // Determine if this needs PTY
                let disabled = is_pty_disabled();
                let forced = is_force_pty_command(&line);
                let needs_pty_exec = forced || (!disabled && needs_pty(&line));

                let cmd_id = Uuid::now_v7().to_string();

                // Use suspend guard for PTY commands and route via execute_command (WS PTY when enabled)
                if needs_pty_exec {
                    // Suspend Reedline while PTY command runs
                    let _guard = line_editor.suspend_guard();
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
                        }
                        Err(e) => eprintln!("Error: {e}"),
                    }
                    // Guard automatically drops and resumes here
                } else {
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
                        }
                        Err(e) => eprintln!("Error: {e}"),
                    }
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

    Ok(0)
}

fn run_wrap_mode(config: &ShellConfig, command: &str) -> Result<i32> {
    let cmd_id = Uuid::now_v7().to_string();
    let running_child_pid = Arc::new(AtomicI32::new(0));

    // Set up signal handlers for wrap mode to properly handle signals like SIGTERM
    setup_signal_handlers(running_child_pid.clone())?;

    let status = execute_command(config, command, &cmd_id, running_child_pid)?;
    Ok(exit_code(status))
}

#[cfg(unix)]
fn exit_code(status: ExitStatus) -> i32 {
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(1))
}

#[cfg(not(unix))]
fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}

fn run_script_mode(config: &ShellConfig, script_path: &Path) -> Result<i32> {
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

    // Set BASH_ENV for builtin command tracking when using bash
    // Only in script mode where we need to track script internals
    // Skip for simple -c commands to avoid duplicate logging
    if is_bash && matches!(config.mode, ShellMode::Script(_)) {
        set_bashenv_trampoline(&mut cmd);
    }

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
    let mut extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });

    #[cfg(unix)]
    if let Some(sig) = status.signal() {
        extra["term_signal"] = json!(sig);
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

fn run_pipe_mode(config: &ShellConfig) -> Result<i32> {
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

/// Handle trace command - show trace information for a span ID
fn handle_trace_command(span_id: &str) -> Result<()> {
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
fn handle_replay_command(span_id: &str) -> Result<()> {
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
        let fs_diff = match backend.fs_diff(&handle, _span_id) {
            Ok(diff) => Some(diff),
            Err(_e) => {
                // PTY sessions may run in a different process (world-agent), so fs_diff via
                // the local backend cache can be unavailable. Suppress noisy warnings here.
                None
            }
        };

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
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_needs_shell_never_panics(input in "\\PC*") {
            // Any printable character sequence shouldn't cause panic
            let _ = needs_shell(&input);
        }

        #[test]
        fn test_simple_commands_dont_need_shell(
            cmd in "[a-zA-Z][a-zA-Z0-9_-]*",
            args in prop::collection::vec("[a-zA-Z0-9_.-]+", 0..5)
        ) {
            let command = if args.is_empty() {
                cmd
            } else {
                format!("{} {}", cmd, args.join(" "))
            };

            // Simple commands with alphanumeric args shouldn't need shell
            prop_assert!(!needs_shell(&command));
        }

        #[test]
        fn test_pipes_always_need_shell(
            left_cmd in "[a-zA-Z]+",
            right_cmd in "[a-zA-Z]+"
        ) {
            let command = format!("{} | {}", left_cmd, right_cmd);
            prop_assert!(needs_shell(&command));
        }

        #[test]
        fn test_redirections_need_shell(
            cmd in "[a-zA-Z]+",
            file in "[a-zA-Z0-9._-]+",
            redirect in prop::sample::select(vec![">", ">>", "<", "2>", "&>"])
        ) {
            let command = format!("{} {} {}", cmd, redirect, file);
            prop_assert!(needs_shell(&command));
        }

        #[test]
        fn test_logical_operators_need_shell(
            left_cmd in "[a-zA-Z]+",
            right_cmd in "[a-zA-Z]+",
            operator in prop::sample::select(vec!["&&", "||", ";"])
        ) {
            let command = format!("{} {} {}", left_cmd, operator, right_cmd);
            prop_assert!(needs_shell(&command));
        }

        #[test]
        fn test_command_substitution_needs_shell(
            outer_cmd in "[a-zA-Z]+",
            inner_cmd in "[a-zA-Z]+",
            substitution_type in prop::sample::select(vec!["$({})", "`{}`"])
        ) {
            let substitution = substitution_type.replace("{}", &inner_cmd);
            let command = format!("{} {}", outer_cmd, substitution);
            prop_assert!(needs_shell(&command));
        }

        #[test]
        fn test_background_processes_need_shell(cmd in "[a-zA-Z]+") {
            let command = format!("{} &", cmd);
            prop_assert!(needs_shell(&command));
        }

        #[test]
        fn test_stderr_redirections_need_shell(
            cmd in "[a-zA-Z]+",
            stderr_redirect in prop::sample::select(vec!["2>&1", "1>&2", "2>"])
        ) {
            let command = format!("{} {}", cmd, stderr_redirect);
            prop_assert!(needs_shell(&command));
        }
    }
}

fn set_bashenv_trampoline(cmd: &mut Command) {
    if let Ok(home) = std::env::var("HOME") {
        let preexec_path = format!("{home}/.substrate_preexec");
        // Base trap file we already write:
        let _ = std::fs::write(&preexec_path, BASH_PREEXEC_SCRIPT);

        // If user had BASH_ENV, create a trampoline that sources it first.
        if let Ok(user_be) = std::env::var("BASH_ENV") {
            let tramp = format!("{home}/.substrate_bashenv_trampoline");
            let content = format!(
                r#"#!/usr/bin/env bash
# chain user's BASH_ENV then our trap
[[ -f "{}" ]] && source "{}"
source "{}"
"#,
                shellexpand::tilde(&user_be).as_ref().replace('"', r#"\""#),
                shellexpand::tilde(&user_be).as_ref().replace('"', r#"\""#),
                preexec_path.replace('"', r#"\""#)
            );
            let _ = std::fs::write(&tramp, content);
            cmd.env("BASH_ENV", &tramp);
        } else {
            cmd.env("BASH_ENV", &preexec_path);
        }
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
    if !is_test_mode && (!atty::is(atty::Stream::Stdin) || !atty::is(atty::Stream::Stdout)) {
        return false;
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
        Err(_) => return false, // Malformed command, don't use PTY
    };

    // Peel off wrapper commands to find the actual command
    let peeled_tokens = peel_wrappers(&tokens);

    // Use peeled tokens if available, otherwise original
    let working_tokens = if !peeled_tokens.is_empty() {
        &peeled_tokens
    } else {
        &tokens
    };

    let first_token = working_tokens
        .first()
        .and_then(|s| Path::new(s).file_name())
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

fn execute_command(
    config: &ShellConfig,
    command: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
) -> Result<ExitStatus> {
    let trimmed = command.trim();

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
        let mut builder = create_span_builder()
            .with_command(trimmed)
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
                let span_id_for_ws = span
                    .as_ref()
                    .map(|s| s.get_span_id().to_string())
                    .unwrap_or_else(|| cmd_id.to_string());
                match execute_world_pty_over_ws(trimmed, &span_id_for_ws) {
                    Ok(code) => {
                        if let Some(active_span) = span.take() {
                            let (scopes_used, fs_diff) = collect_world_telemetry(active_span.get_span_id());
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

        // Use host PTY execution path as fallback
        let pty_status = pty_exec::execute_with_pty(config, trimmed, cmd_id, running_child_pid)?;

        // Finish span if we created one (PTY path)
        if let Some(active_span) = span {
            let exit_code = pty_status.code.unwrap_or(-1);
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
            if let Some(signal) = pty_status.signal {
                // Terminated by signal: set low 7 bits to the signal number
                // This makes status.signal() work correctly
                return Ok(ExitStatus::from_raw(signal & 0x7f));
            } else if let Some(code) = pty_status.code {
                // Normal exit: code in bits 8-15
                return Ok(ExitStatus::from_raw((code & 0xff) << 8));
            } else {
                return Ok(ExitStatus::from_raw(0));
            }
        }

        #[cfg(windows)]
        {
            // 🔥 EXPERT FIX: Don't shift bits on Windows - use raw code directly
            use std::os::windows::process::ExitStatusExt;
            let code = pty_status.code.unwrap_or(0) as u32;
            return Ok(ExitStatus::from_raw(code));
        }
    }

    // Continue with existing implementation for non-PTY commands...
    // Compute resolved path from raw command before redaction
    let resolved = first_command_path(trimmed);

    // Redact sensitive information before logging (token-aware)
    let redacted_command = if std::env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        trimmed.to_string()
    } else {
        let toks = shell_words::split(trimmed)
            .unwrap_or_else(|_| trimmed.split_whitespace().map(|s| s.to_string()).collect());
        let mut out = Vec::new();
        let mut i = 0;

        while i < toks.len() {
            let t = &toks[i];
            let lt = t.to_lowercase();

            // Handle -u, --user, --password, --token, -p (redact both flag and value)
            if lt == "-u" || lt == "--user" || lt == "--password" || lt == "--token" || lt == "-p" {
                out.push("***".into()); // redact flag
                if i + 1 < toks.len() {
                    out.push("***".into()); // redact value
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }

            // Handle -H/--header specially to preserve header name
            // Note: -H is case-sensitive, --header is case-insensitive
            if t == "-H" || lt == "--header" {
                out.push(t.clone()); // keep flag
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

            // Handle inline forms (k=v)
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

            // Default: use base redaction
            out.push(redact_sensitive(t));
            i += 1;
        }
        out.join(" ")
    };

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

    // Route non-PTY commands through world agent (UDS HTTP) when world is enabled (Linux only)
    #[cfg(target_os = "linux")]
    {
        let world_enabled = env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled";
        let uds_exists = std::path::Path::new("/run/substrate.sock").exists();
        if world_enabled || uds_exists {
            if let Ok((exit_code, stdout, stderr, scopes_used)) = exec_non_pty_via_agent(trimmed) {
                use std::io::{self, Write};
                let _ = io::stdout().write_all(&stdout);
                let _ = io::stderr().write_all(&stderr);
                if let Some(active_span) = span {
                    let _ = active_span.finish(exit_code, scopes_used, None);
                }
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    return Ok(std::process::ExitStatus::from_raw((exit_code & 0xff) << 8));
                }
                #[cfg(windows)]
                {
                    use std::os::windows::process::ExitStatusExt;
                    return Ok(std::process::ExitStatus::from_raw(exit_code as u32));
                }
            } else {
                eprintln!("substrate: warn: world exec failed, running direct");
            }
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
    let mut extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });

    #[cfg(unix)]
    if let Some(sig) = status.signal() {
        extra["term_signal"] = json!(sig);
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
    // Ensure agent is ready
    ensure_world_agent_ready()?;

    // Connect UDS and do WS handshake
    let rt = tokio::runtime::Runtime::new()?;
    let code = rt.block_on(async move {
        let stream = UnixStream::connect("/run/substrate.sock").await
            .map_err(|e| anyhow::anyhow!("connect UDS: {}", e))?;
        let url = url::Url::parse("ws://localhost/v1/stream").unwrap();
        let (ws, _resp) = tungs::client_async(url, stream).await
            .map_err(|e| anyhow::anyhow!("ws handshake: {}", e))?;
        let (sink, mut stream) = ws.split();
        let sink = std::sync::Arc::new(tokio::sync::Mutex::new(sink));

        if std::env::var("SUBSTRATE_WS_DEBUG").ok().as_deref() == Some("1") {
            eprintln!("using world-agent PTY WS");
        }

        // Prepare start frame (strip optional ":pty " prefix used in REPL to force PTY)
        let cmd_sanitized = if let Some(rest) = cmd.strip_prefix(":pty ") { rest } else { cmd };
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
        sink
            .lock()
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
                        let b64 = B64.encode(&buf[..n]);
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
                        if s.lock().await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err() { break; }
                    }
                }));
            }
            // SIGTERM
            if let Ok(mut sig) = signal(SignalKind::terminate()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "TERM"});
                        if s.lock().await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err() { break; }
                    }
                }));
            }
            // SIGHUP
            if let Ok(mut sig) = signal(SignalKind::hangup()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "HUP"});
                        if s.lock().await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err() { break; }
                    }
                }));
            }
            // SIGQUIT
            if let Ok(mut sig) = signal(SignalKind::quit()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "QUIT"});
                        if s.lock().await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err() { break; }
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
                                if let Ok(bytes) = B64.decode(b64) {
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
        let _ = stdin_task.abort();
        #[cfg(target_os = "linux")]
        {
            let _ = resize_task.abort();
            for t in signal_tasks { let _ = t.abort(); }
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
        match std::os::unix::net::UnixStream::connect(SOCK) {
            Ok(mut s) => {
                let _ = s.set_read_timeout(Some(std::time::Duration::from_millis(150)));
                let _ = s.set_write_timeout(Some(std::time::Duration::from_millis(150)));
                let req = b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
                if s.write_all(req).is_ok() {
                    let mut buf = [0u8; 512];
                    if let Ok(n) = s.read(&mut buf) {
                        return n > 0 && std::str::from_utf8(&buf[..n]).unwrap_or("").contains(" 200 ");
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
        which::which("substrate-world-agent").ok().map(|p| p.display().to_string()),
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

fn ok_status() -> Result<ExitStatus> {
    if cfg!(windows) {
        Command::new("cmd").arg("/C").arg("exit 0").status()
    } else {
        Command::new("true").status()
    }
    .context("Failed to create success status")
}

fn handle_builtin(
    config: &ShellConfig,
    command: &str,
    parent_cmd_id: &str,
) -> Result<Option<ExitStatus>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(None);
    }

    let builtin_result = match parts[0] {
        "cd" => {
            let target = match parts.get(1).copied() {
                None => "~".to_string(),
                Some("-") => {
                    if let Ok(oldpwd) = env::var("OLDPWD") {
                        println!("{oldpwd}");
                        oldpwd
                    } else {
                        "~".to_string()
                    }
                }
                Some(p) => p.to_string(),
            };
            let expanded = shellexpand::tilde(&target);
            let prev = env::current_dir()?;
            env::set_current_dir(expanded.as_ref())?;
            env::set_var("OLDPWD", prev);
            env::set_var("PWD", env::current_dir()?.display().to_string());
            Some(ok_status()?)
        }
        "pwd" => {
            println!("{}", env::current_dir()?.display());
            Some(ok_status()?)
        }
        "unset" => {
            for k in &parts[1..] {
                env::remove_var(k);
            }
            Some(ok_status()?)
        }
        "export" => {
            let mut handled = true;
            for part in &parts[1..] {
                if let Some((k, v)) = part.split_once('=') {
                    // Reject quotes or variable refs to avoid wrong semantics
                    if v.contains('"') || v.contains('\'') || v.contains('$') {
                        handled = false;
                        break;
                    }
                    env::set_var(k, v);
                } else {
                    handled = false;
                    break;
                }
            }
            if handled {
                Some(ok_status()?)
            } else {
                // Defer complex cases to the external shell
                None
            }
        }
        _ => None,
    };

    // Log builtin command if we handled it
    if builtin_result.is_some() {
        let builtin_cmd_id = Uuid::now_v7().to_string();
        let extra = json!({ "parent_cmd_id": parent_cmd_id });

        // Apply redaction to builtin commands
        let redacted_command = {
            let tokens = shell_words::split(command).unwrap_or_else(|_| vec![command.to_string()]);
            let mut out = Vec::new();
            let mut i = 0;

            while i < tokens.len() {
                let t = &tokens[i];

                // Check for environment variable exports with sensitive names
                if tokens.len() > 1 && tokens[0] == "export" && t.contains('=') {
                    if let Some((k, _)) = t.split_once('=') {
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
                }

                out.push(t.clone());
                i += 1;
            }
            out.join(" ")
        };

        log_command_event(
            config,
            "builtin_command",
            &redacted_command,
            &builtin_cmd_id,
            Some(extra),
        )?;
    }

    Ok(builtin_result)
}

#[cfg(target_os = "linux")]
fn exec_non_pty_via_agent(cmd: &str) -> anyhow::Result<(i32, Vec<u8>, Vec<u8>, Vec<String>)> {
    use std::os::unix::net::UnixStream;
    use std::io::{Read, Write};
    let sock_path = "/run/substrate.sock";
    let mut stream = UnixStream::connect(sock_path)
        .map_err(|e| anyhow::anyhow!("connect UDS: {}", e))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(3)))
        .ok();

    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .display()
        .to_string();
    let env_map: std::collections::HashMap<String, String> = std::env::vars().collect();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let body = serde_json::json!({
        "profile": null,
        "cmd": cmd,
        "cwd": cwd,
        "env": env_map,
        "pty": false,
        "agent_id": agent_id,
        "budget": null
    })
    .to_string();
    let req = format!(
        "POST /v1/execute HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        body.len(), body
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| anyhow::anyhow!("uds write: {}", e))?;

    let mut resp = Vec::new();
    stream
        .read_to_end(&mut resp)
        .map_err(|e| anyhow::anyhow!("uds read: {}", e))?;
    let resp_str = String::from_utf8_lossy(&resp);
    let mut parts = resp_str.splitn(2, "\r\n\r\n");
    let header = parts.next().unwrap_or("");
    let body = parts.next().unwrap_or("");
    // Parse status line
    let status_line = header.lines().next().unwrap_or("");
    if !status_line.contains("200") {
        return Err(anyhow::anyhow!(format!("agent exec failed: {}", status_line)));
    }
    let v: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| anyhow::anyhow!("parse json: {}", e))?;
    let exit_code = v.get("exit").and_then(|x| x.as_i64()).unwrap_or(1) as i32;
    let stdout_b64 = v.get("stdout_b64").and_then(|x| x.as_str()).unwrap_or("");
    let stderr_b64 = v.get("stderr_b64").and_then(|x| x.as_str()).unwrap_or("");
    let stdout = base64::engine::general_purpose::STANDARD
        .decode(stdout_b64)
        .unwrap_or_default();
    let stderr = base64::engine::general_purpose::STANDARD
        .decode(stderr_b64)
        .unwrap_or_default();
    let scopes_used = v
        .get("scopes_used")
        .and_then(|x| x.as_array())
        .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| Vec::new());
    Ok((exit_code, stdout, stderr, scopes_used))
}

fn execute_external(
    config: &ShellConfig,
    command: &str,
    running_child_pid: Arc<AtomicI32>,
    cmd_id: &str,
) -> Result<ExitStatus> {
    let shell = &config.shell_path;

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

    if is_pwsh || is_powershell {
        // PowerShell
        if config.ci_mode && !config.no_exit_on_error {
            cmd.arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg(format!("$ErrorActionPreference='Stop'; {command}"));
        } else {
            cmd.arg("-NoProfile").arg("-Command").arg(command);
        }
    } else if is_cmd {
        // Windows CMD
        cmd.arg("/C").arg(command);
    } else {
        // Unix shells (bash, sh, zsh, etc.)
        if config.ci_mode && !config.no_exit_on_error && is_bash {
            cmd.arg("-o")
                .arg("errexit")
                .arg("-o")
                .arg("pipefail")
                .arg("-o")
                .arg("nounset");
        }
        cmd.arg("-c").arg(command);
    }

    // Propagate environment
    cmd.env("SHIM_SESSION_ID", &config.session_id);
    cmd.env("SHIM_TRACE_LOG", &config.trace_log_file);
    cmd.env("SHIM_PARENT_CMD_ID", cmd_id); // Pass cmd_id for shim correlation
    cmd.env_remove("SHIM_ACTIVE"); // Clear to allow shims to work
    cmd.env_remove("SHIM_CALLER"); // Clear caller chain for fresh command
    cmd.env_remove("SHIM_CALL_STACK"); // Clear call stack for fresh command
                                       // Keep PATH as-is with shims - the env_remove("SHIM_ACTIVE") should be sufficient

    // Set BASH_ENV for builtin command tracking when using bash
    // Only in script mode where we need to track script internals
    // Skip for simple -c commands to avoid duplicate logging
    if is_bash && matches!(config.mode, ShellMode::Script(_)) {
        set_bashenv_trampoline(&mut cmd);
    }

    // Handle I/O based on mode - always inherit stdin for better compatibility
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

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

    let child_pid = child.id() as i32;
    running_child_pid.store(child_pid, Ordering::SeqCst);

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait for command: {command}"))?;

    // Clear the running PID
    running_child_pid.store(0, Ordering::SeqCst);

    Ok(status)
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

fn log_command_event(
    config: &ShellConfig,
    event_type: &str,
    command: &str,
    cmd_id: &str,
    extra: Option<serde_json::Value>,
) -> Result<()> {
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
        "isatty_stdin": atty::is(atty::Stream::Stdin),
        "isatty_stdout": atty::is(atty::Stream::Stdout),
        "isatty_stderr": atty::is(atty::Stream::Stderr),
        "pty": matches!(&config.mode, ShellMode::Interactive { use_pty: true }),
    });

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

// Helper function to setup signal handlers
fn setup_signal_handlers(running_child_pid: Arc<AtomicI32>) -> Result<()> {
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
            }
        });
    }

    Ok(())
}

// Custom prompt implementation with signal handling
struct SubstratePrompt {
    ci_mode: bool,
}

impl SubstratePrompt {
    fn new(ci_mode: bool) -> Self {
        Self { ci_mode }
    }
}

impl Prompt for SubstratePrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        if self.ci_mode {
            Cow::Borrowed("> ")
        } else {
            Cow::Borrowed("substrate> ")
        }
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("::: ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        match history_search.status {
            PromptHistorySearchStatus::Passing => Cow::Borrowed("(history search) "),
            PromptHistorySearchStatus::Failing => Cow::Borrowed("(failing search) "),
        }
    }
}

// NEW: Completer implementation (command completion is a new feature, not a port)
struct SubstrateCompleter {
    commands: Vec<String>,
}

impl SubstrateCompleter {
    fn new(config: &ShellConfig) -> Self {
        // Use PATH from config.original_path
        let commands = collect_commands_from_path(&config.original_path);
        Self { commands }
    }
}

impl Completer for SubstrateCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Extract the word being completed
        let word = extract_word_at_pos(line, pos);

        // Filter commands that start with the current word
        // Limit to first 100 suggestions for performance
        self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(word))
            .take(100)
            .map(|cmd| Suggestion {
                value: cmd.clone(),
                description: None,
                extra: None,
                span: Span::new(pos - word.len(), pos),
                append_whitespace: true,
                style: None,
            })
            .collect()
    }
}

// Helper functions for completion
fn collect_commands_from_path(path: &str) -> Vec<String> {
    let mut commands = Vec::new();
    for dir in path.split(':') {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && is_executable(&metadata) {
                        if let Some(name) = entry.file_name().to_str() {
                            commands.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    commands.sort();
    commands.dedup();
    commands
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    // On Windows, check file extensions (simplified for now)
    true
}

fn extract_word_at_pos(line: &str, pos: usize) -> &str {
    let start = line[..pos]
        .rfind(|c: char| c.is_whitespace())
        .map(|i| i + 1)
        .unwrap_or(0);
    &line[start..pos]
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
}
