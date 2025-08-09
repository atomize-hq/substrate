use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use uuid::Uuid;
use substrate_common::{dedupe_path, log_schema, redact_sensitive};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

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
    #[arg(short = 'c', long = "command", value_name = "CMD", conflicts_with = "script")]
    pub command: Option<String>,

    /// Execute a script file
    #[arg(short = 'f', long = "file", value_name = "SCRIPT", conflicts_with = "command")]
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
}

#[derive(Debug, Clone)]
pub enum ShellMode {
    Interactive { use_pty: bool },  // Full REPL with optional PTY
    Wrap(String),                   // Single command execution (-c "cmd")
    Script(PathBuf),                // Script file execution (-f script.sh)
    Pipe,                           // Read commands from stdin
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
        
        let session_id = env::var("SHIM_SESSION_ID")
            .unwrap_or_else(|_| Uuid::now_v7().to_string());

        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))  // Windows support
            .context("HOME/USERPROFILE not set")?;

        let trace_log_file = env::var("SHIM_TRACE_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&home).join(".trace_shell.jsonl"));

        let original_path = env::var("SHIM_ORIGINAL_PATH")
            .or_else(|_| env::var("PATH"))
            .context("No PATH found")?;

        let shim_dir = PathBuf::from(&home).join(".cmdshim_rust");

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
            env_vars: HashMap::new(),
        })
    }
}

pub fn run_shell() -> Result<i32> {
    let config = ShellConfig::from_args()?;

    // Set up environment for child processes
    env::set_var("SHIM_SESSION_ID", &config.session_id);
    env::set_var("SHIM_ORIGINAL_PATH", &config.original_path);
    env::set_var("SHIM_TRACE_LOG", &config.trace_log_file);
    
    // Clear SHIM_ACTIVE to allow shims to work properly
    // The substrate shell itself should not be considered "active" shimming
    env::remove_var("SHIM_ACTIVE");

    // Ensure shim directory is in PATH with deduplication (use OS-specific separator)
    let sep = if cfg!(windows) { ';' } else { ':' };
    let path_with_shims = format!("{}{}{}",
        config.shim_dir.display(),
        sep,
        config.original_path
    );
    env::set_var("PATH", dedupe_path(&path_with_shims));

    match &config.mode {
        ShellMode::Interactive { use_pty } => {
            if *use_pty && cfg!(unix) {
                run_interactive_pty(&config)
            } else {
                run_interactive_shell(&config)
            }
        }
        ShellMode::Wrap(cmd) => run_wrap_mode(&config, cmd),
        ShellMode::Script(path) => run_script_mode(&config, path),
        ShellMode::Pipe => run_pipe_mode(&config),
    }
}

#[cfg(unix)]
fn run_interactive_pty(config: &ShellConfig) -> Result<i32> {
    use crate::pty::{spawn_pty_shell, handle_pty_io};
    
    println!("Substrate v{} (PTY mode)", env!("CARGO_PKG_VERSION"));
    println!("Session ID: {}", config.session_id);
    println!("Logging to: {}", config.trace_log_file.display());
    println!("Shell: {}", config.shell_path);
    
    // Log PTY session start
    let session_cmd_id = Uuid::now_v7().to_string();
    log_command_event(config, "pty_session_start", &config.shell_path, &session_cmd_id, None)?;
    
    // Spawn shell in PTY
    let pty_session = spawn_pty_shell(&config.shell_path)?;
    
    // Handle I/O between terminal and PTY
    let result = handle_pty_io(pty_session, config);
    
    // Log PTY session end
    let exit_extra = json!({
        log_schema::EXIT_CODE: if result.is_ok() { 0 } else { 1 }
    });
    log_command_event(config, "pty_session_end", &config.shell_path, &session_cmd_id, Some(exit_extra))?;
    
    result?;
    Ok(0)
}

#[cfg(not(unix))]
fn run_interactive_pty(_config: &ShellConfig) -> Result<i32> {
    anyhow::bail!("PTY mode is not supported on this platform")
}

fn run_interactive_shell(config: &ShellConfig) -> Result<i32> {
    use rustyline::DefaultEditor;

    println!("Substrate v{}", env!("CARGO_PKG_VERSION"));
    println!("Session ID: {}", config.session_id);
    println!("Logging to: {}", config.trace_log_file.display());

    let mut rl = DefaultEditor::new()?;
    let prompt = if config.ci_mode { "> " } else { "substrate> " };

    // Set up signal handling for Ctrl-C
    let running_child_pid = Arc::new(AtomicI32::new(0));
    {
        let running = running_child_pid.clone();
        ctrlc::set_handler(move || {
            let pid = running.load(Ordering::SeqCst);
            if pid > 0 {
                // Forward signal to entire process group
                #[cfg(unix)]
                {
                    use nix::sys::signal::{killpg, Signal};
                    use nix::unistd::Pid;
                    let _ = killpg(Pid::from_raw(pid), Signal::SIGINT);
                }
            }
            // If no child is running, the signal is dropped and REPL continues
        })?;
    }

    // Also install SIGTERM handler for clean CI cancellation
    #[cfg(unix)]
    {
        let running_term = running_child_pid.clone();
        use nix::sys::signal::{signal, SigHandler, Signal};
        use std::sync::atomic::AtomicBool;
        
        static TERM_RECEIVED: AtomicBool = AtomicBool::new(false);
        extern "C" fn handle_term(_: i32) {
            TERM_RECEIVED.store(true, Ordering::Relaxed);
        }
        unsafe {
            signal(Signal::SIGTERM, SigHandler::Handler(handle_term))?;
        }
        
        // Spawn a thread to forward SIGTERM to child process group
        std::thread::spawn(move || {
            loop {
                if TERM_RECEIVED.load(Ordering::Relaxed) {
                    let pid = running_term.load(Ordering::SeqCst);
                    if pid > 0 {
                        use nix::sys::signal::{killpg, Signal};
                        use nix::unistd::Pid;
                        let _ = killpg(Pid::from_raw(pid), Signal::SIGTERM);
                    }
                    TERM_RECEIVED.store(false, Ordering::Relaxed);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }

    loop {
        match rl.readline(prompt) {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                rl.add_history_entry(&line)?;

                // Check for exit commands
                if matches!(line.trim(), "exit" | "quit") {
                    break;
                }

                // Execute command
                let cmd_id = Uuid::now_v7().to_string();
                match execute_command(config, &line, &cmd_id, running_child_pid.clone()) {
                    Ok(status) => {
                        if !status.success() {
                            #[cfg(unix)]
                            if let Some(sig) = status.signal() {
                                eprintln!("Command terminated by signal {}", sig);
                            } else {
                                eprintln!("Command failed with status: {}", status.code().unwrap_or(-1));
                            }
                            #[cfg(not(unix))]
                            eprintln!("Command failed with status: {}", status.code().unwrap_or(-1));
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(0)
}

fn run_wrap_mode(config: &ShellConfig, command: &str) -> Result<i32> {
    let cmd_id = Uuid::now_v7().to_string();
    let running_child_pid = Arc::new(AtomicI32::new(0));
    let status = execute_command(config, command, &cmd_id, running_child_pid)?;
    Ok(exit_code(status))
}

#[cfg(unix)]
fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or_else(|| 128 + status.signal().unwrap_or(1))
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
            cmd.arg("-o").arg("errexit")
               .arg("-o").arg("pipefail")
               .arg("-o").arg("nounset");
        }
        cmd.arg(script_path);
    }

    // Propagate environment
    cmd.env("SHIM_SESSION_ID", &config.session_id)
       .env("SHIM_TRACE_LOG", &config.trace_log_file)
       .env_remove("SHIM_ACTIVE")  // Clear to allow shims to work
       .stdin(Stdio::inherit())
       .stdout(Stdio::inherit())
       .stderr(Stdio::inherit());
    
    // Set BASH_ENV for builtin command tracking when using bash
    if is_bash {
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
    let mut child = cmd.spawn()
        .with_context(|| format!("Failed to execute script: {}", script_path.display()))?;

    let child_pid = child.id() as i32;
    running_child_pid.store(child_pid, Ordering::SeqCst);

    let status = child.wait()
        .with_context(|| format!("Failed to wait for script: {}", script_path.display()))?;

    running_child_pid.store(0, Ordering::SeqCst);

    // Log script completion
    let duration = start_time.elapsed();
    let mut extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });
    
    #[cfg(unix)]
    if let Some(sig) = status.signal() {
        extra["term_signal"] = json!(sig);
    }
    
    log_command_event(config, "command_complete", &script_cmd, &cmd_id, Some(extra))?;

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
                    eprintln!("Command failed: {}", line);
                    return Ok(last_status);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                if !config.no_exit_on_error {
                    return Ok(1);
                }
                last_status = 1;
            }
        }
    }

    Ok(last_status)
}

pub fn needs_shell(cmd: &str) -> bool {
    let Ok(tokens) = shell_words::split(cmd) else { return true; };
    let ops = ["&&","||","|",";","<<",">>","<",">","&","2>","&>"];
    tokens.iter().any(|t| {
        ops.contains(&t.as_str())
        || t.starts_with("$(") || t.starts_with('`')
        || t.contains(">&")            // 2>&1, 1>&2, etc.
        || t.chars().any(|c| "<>|&".contains(c)) && t.len() > 1 // e.g. 1>/dev/null
    })
}

fn set_bashenv_trampoline(cmd: &mut Command) {
    if let Ok(home) = std::env::var("HOME") {
        let preexec_path = format!("{}/.substrate_preexec", home);
        // Base trap file we already write:
        let _ = std::fs::write(&preexec_path, BASH_PREEXEC_SCRIPT);

        // If user had BASH_ENV, create a trampoline that sources it first.
        if let Ok(user_be) = std::env::var("BASH_ENV") {
            let tramp = format!("{}/.substrate_bashenv_trampoline", home);
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

fn needs_direct_terminal(cmd: &str) -> bool {
    // Commands that need direct terminal control
    let tokens = shell_words::split(cmd).unwrap_or_else(|_| vec![cmd.to_string()]);
    if let Some(first) = tokens.first() {
        let cmd_name = Path::new(first)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(first)
            .to_lowercase();
        
        // Interactive commands that need full terminal control
        matches!(cmd_name.as_str(), 
            "claude" | "vim" | "vi" | "nano" | "emacs" | "less" | "more" |
            "top" | "htop" | "btop" | "code" | "codex" | "cursor" | 
            "nvim" | "neovim" | "micro" | "pico" | "joe" | "ed" |
            "ssh" | "telnet" | "ftp" | "sftp" | "mysql" | "psql" |
            "python" | "python3" | "ipython" | "node" | "irb" | "pry" |
            "ghci" | "scala" | "clojure" | "julia" | "R" | "bc" | "dc"
        )
    } else {
        false
    }
}

fn execute_command(
    config: &ShellConfig,
    command: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
) -> Result<ExitStatus> {
    let trimmed = command.trim();
    
    // Check if this command needs direct terminal access
    if needs_direct_terminal(trimmed) {
        // For interactive commands, spawn directly without shell wrapper
        return execute_direct(config, trimmed, cmd_id, running_child_pid);
    }
    
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
                out.push("***".into());  // redact flag
                if i + 1 < toks.len() {
                    out.push("***".into());  // redact value
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            
            // Handle -H/--header specially to preserve header name
            // Note: -H is case-sensitive, --header is case-insensitive
            if t == "-H" || lt == "--header" {
                out.push(t.clone());  // keep flag
                if i + 1 < toks.len() {
                    let hv = &toks[i + 1];
                    let lower = hv.to_lowercase();
                    let redacted = if lower.starts_with("authorization:")
                        || lower.starts_with("x-api-key:")
                        || lower.starts_with("x-auth-token:")
                        || lower.starts_with("cookie:") {
                        format!("{}: ***", hv.split(':').next().unwrap_or("").trim_end_matches(':'))
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
                if kl.contains("token") || kl.contains("password") || kl.contains("secret") 
                    || kl.contains("apikey") || kl.contains("api_key") {
                    out.push(format!("{}=***", k));
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
    log_command_event(config, "command_start", &redacted_command, cmd_id, start_extra)?;
    let start_time = std::time::Instant::now();

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
    
    log_command_event(config, "command_complete", &redacted_command, cmd_id, Some(extra))?;

    Ok(status)
}

fn ok_status() -> Result<ExitStatus> {
    if cfg!(windows) {
        Command::new("cmd").arg("/C").arg("exit 0").status()
    } else {
        Command::new("true").status()
    }
    .context("Failed to create success status")
}

fn handle_builtin(config: &ShellConfig, command: &str, parent_cmd_id: &str) -> Result<Option<ExitStatus>> {
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
                        println!("{}", oldpwd);
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
        log_command_event(config, "builtin_command", command, &builtin_cmd_id, Some(extra))?;
    }
    
    Ok(builtin_result)
}

fn execute_external(
    config: &ShellConfig,
    command: &str,
    running_child_pid: Arc<AtomicI32>,
    cmd_id: &str,
) -> Result<ExitStatus> {
    let shell = &config.shell_path;

    // Verify shell exists
    if !which::which(shell).is_ok() && !Path::new(shell).exists() {
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
               .arg(&format!("$ErrorActionPreference='Stop'; {}", command));
        } else {
            cmd.arg("-NoProfile")
               .arg("-Command")
               .arg(command);
        }
    } else if is_cmd {
        // Windows CMD
        cmd.arg("/C").arg(command);
    } else {
        // Unix shells (bash, sh, zsh, etc.)
        if config.ci_mode && !config.no_exit_on_error && is_bash {
            cmd.arg("-o").arg("errexit")
               .arg("-o").arg("pipefail")
               .arg("-o").arg("nounset");
        }
        cmd.arg("-c").arg(command);
    }

    // Propagate environment
    cmd.env("SHIM_SESSION_ID", &config.session_id);
    cmd.env("SHIM_TRACE_LOG", &config.trace_log_file);
    cmd.env("SHIM_PARENT_CMD_ID", cmd_id);  // Pass cmd_id for shim correlation
    cmd.env_remove("SHIM_ACTIVE");  // Clear to allow shims to work
    cmd.env_remove("SHIM_CALLER");  // Clear caller chain for fresh command
    cmd.env_remove("SHIM_CALL_STACK");  // Clear call stack for fresh command
    // Keep PATH as-is with shims - the env_remove("SHIM_ACTIVE") should be sufficient
    
    // Set BASH_ENV for builtin command tracking when using bash
    if is_bash {
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
    let mut child = cmd.spawn()
        .with_context(|| format!("Failed to execute: {}", command))?;

    let child_pid = child.id() as i32;
    running_child_pid.store(child_pid, Ordering::SeqCst);

    let status = child.wait()
        .with_context(|| format!("Failed to wait for command: {}", command))?;

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

fn maybe_rotate_log(path: &Path) -> Result<()> {
    const MAX_BYTES: u64 = 50 * 1024 * 1024; // 50MB default
    
    // Check environment variable for custom limit
    let max_bytes = env::var("TRACE_LOG_MAX_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|mb| mb * 1024 * 1024)
        .unwrap_or(MAX_BYTES);
    
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > max_bytes {
            let bak = path.with_extension("jsonl.1");
            let _ = std::fs::rename(path, bak);
        }
    }
    Ok(())
}

fn log_command_event(
    config: &ShellConfig,
    event_type: &str,
    command: &str,
    cmd_id: &str,
    extra: Option<serde_json::Value>
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

    // Ensure log directory exists - CRITICAL: Don't swallow errors
    if let Some(dir) = config.trace_log_file.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create log directory: {}", dir.display()))?;
    }

    // Rotate log if it's too large
    maybe_rotate_log(&config.trace_log_file)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.trace_log_file)?;

    // Set permissions on first creation
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&config.trace_log_file,
            std::fs::Permissions::from_mode(0o600));
    }

    // Ensure single-line JSON by escaping newlines
    let mut line = log_entry.to_string();
    if line.contains('\n') {
        line = line.replace('\n', "\\n");
    }
    writeln!(file, "{}", line)
        .with_context(|| format!("Failed to write log entry to: {}", config.trace_log_file.display()))?;

    // Optional fsync for durability
    if env::var("SHIM_FSYNC").as_deref() == Ok("1") {
        file.flush()?;
        file.sync_all()?;
    }

    Ok(())
}

// Re-export pty module
#[cfg(unix)]
pub mod pty;