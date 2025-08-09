use anyhow::{anyhow, Result};
use std::env;
use substrate_supervisor::{launch_supervised, SupervisorConfig};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} echo hello world", args[0]);
        eprintln!("  {} bash -c 'git status'", args[0]);
        eprintln!("  {} code  # Launch VS Code with tracing", args[0]);
        eprintln!();
        eprintln!("Environment variables:");
        eprintln!("  SHIM_TRACE_LOG - Set log file location (default: no logging)");
        eprintln!("  SHIM_SESSION_ID - Set session ID (default: auto-generated)");
        return Err(anyhow!("No command specified"));
    }

    let target_command = args[1..].iter().map(|s| s.to_string()).collect();

    // Create supervisor config
    let mut config = SupervisorConfig::new(target_command)?;

    // Check for trace log file in environment
    if let Ok(log_file) = env::var("SHIM_TRACE_LOG") {
        config = config.with_log_file(log_file);
    } else {
        // Default to home directory log file
        if let Ok(home) = env::var("HOME") {
            let default_log = format!("{}/.trace_shell.jsonl", home);
            config = config.with_log_file(default_log);
        }
    }

    // Launch the supervised application
    launch_supervised(config)
}
