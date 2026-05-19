use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;
use tracing_subscriber::EnvFilter;

mod auth;
mod cli;
mod core;
mod launch;
mod message_tracing;
mod models;
mod pid;
mod providers;
mod router;
mod server;
mod structured_events;

use launch::GatewayLaunchContract;

const PROCESS_TRANSITION_GRACE_MS: u64 = 500;

async fn stop_service(pid: u32) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
            .map_err(|e| anyhow::anyhow!("Failed to stop service: {}", e))?;
    }
    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute taskkill: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to stop process: {}", stderr));
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(
        PROCESS_TRANSITION_GRACE_MS,
    ))
    .await;
    Ok(())
}

async fn start_foreground(
    config: cli::AppConfig,
    launch: GatewayLaunchContract,
) -> anyhow::Result<()> {
    let integrated_auth = server::IntegratedGatewayAuthContext::from_launch_mode(launch.mode)?;

    // Write PID file
    if let Err(e) = pid::write_pid() {
        eprintln!("Warning: Failed to write PID file: {}", e);
    }

    tracing::info!("Starting Substrate Gateway on port {}", config.server.port);
    println!("🚀 Substrate Gateway v{}", env!("CARGO_PKG_VERSION"));
    println!(
        "📡 Starting server on {}:{}",
        config.server.host, config.server.port
    );
    println!();

    // Display gateway routing settings
    println!("🔀 Gateway Routing Settings:");
    println!("   Default route: {}", config.router.default);
    if let Some(ref bg) = config.router.background {
        println!("   Background route: {}", bg);
    }
    if let Some(ref think) = config.router.think {
        println!("   Think route: {}", think);
    }
    if let Some(ref ws) = config.router.websearch {
        println!("   Web search route: {}", ws);
    }
    println!();
    println!("Press Ctrl+C to stop");

    let result = server::start_server(config, launch, integrated_auth).await;
    let _ = pid::cleanup_pid();
    result
}

fn spawn_background_service(port: Option<u16>, config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?;
    let mut cmd = Command::new(&exe_path);
    cmd.arg("start");

    if let Some(port) = port {
        cmd.arg("--port").arg(port.to_string());
    }
    if let Some(config_path) = config_path {
        cmd.arg("--config").arg(config_path);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                nix::libc::setsid();
                Ok(())
            });
        }
    }

    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    cmd.spawn()?;
    Ok(())
}

fn resolve_launch_and_config(
    cli_config: Option<PathBuf>,
) -> anyhow::Result<(GatewayLaunchContract, cli::AppConfig)> {
    let launch = GatewayLaunchContract::resolve(
        cli_config,
        cli::AppConfig::default_path,
        auth::token_store::TokenStore::default_path,
    )?;
    let config = cli::AppConfig::parse_file_without_env_resolution(&launch.config_path)?;
    Ok((launch, config))
}

fn init_tracing(config: &cli::AppConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.server.log_level));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[derive(Parser)]
#[command(name = "substrate-gateway")]
#[command(about = "Substrate Gateway - single public gateway identity built in Rust", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to configuration file (standalone-local default: ~/.substrate-gateway/config.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the router service
    Start {
        /// Port to listen on
        #[arg(short, long)]
        port: Option<u16>,
        /// Run in detached/background mode
        #[arg(short = 'd', long)]
        detach: bool,
    },
    /// Stop the router service
    Stop,
    /// Restart the router service
    Restart {
        /// Run in detached/background mode
        #[arg(short = 'd', long)]
        detach: bool,
    },
    /// Check service status
    Status,
    /// Manage models and providers
    Model,
    /// Install statusline script for Claude Code
    InstallStatusline,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { port, detach } => {
            let (launch, config) = resolve_launch_and_config(cli.config.clone())?;
            init_tracing(&config);

            // If detached, spawn as background process
            if detach {
                println!("Starting Substrate Gateway in background...");

                // Stop existing service if running
                if let Ok(pid) = pid::read_pid() {
                    if pid::is_process_running(pid) {
                        println!("Stopping existing service...");
                        if let Err(e) = stop_service(pid).await {
                            eprintln!("Warning: Failed to stop existing service: {}", e);
                        }
                    }
                }
                let _ = pid::cleanup_pid();

                // Start in background
                spawn_background_service(port, cli.config.clone())?;
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    PROCESS_TRANSITION_GRACE_MS,
                ))
                .await;

                if let Ok(pid) = pid::read_pid() {
                    println!("✅ Substrate Gateway started in background (PID: {})", pid);
                } else {
                    println!("✅ Substrate Gateway started in background");
                }
                println!("📡 Running on port {}", port.unwrap_or(config.server.port));
                return Ok(());
            }

            // Foreground mode
            let mut config = config;

            // Override port if specified
            if let Some(port) = port {
                config.server.port = port;
            }

            // Check if already running
            if let Ok(existing_pid) = pid::read_pid() {
                if pid::is_process_running(existing_pid) {
                    eprintln!(
                        "❌ Error: Service is already running (PID: {})",
                        existing_pid
                    );
                    eprintln!("Use 'substrate-gateway stop' to stop it first, or use 'substrate-gateway start -d' to restart it");
                    return Ok(());
                }
                // Stale PID file, clean it up
                let _ = pid::cleanup_pid();
            }

            start_foreground(config, launch.clone()).await?;
        }
        Commands::Stop => {
            println!("Stopping Substrate Gateway...");
            match pid::read_pid() {
                Ok(pid) if pid::is_process_running(pid) => match stop_service(pid).await {
                    Ok(_) => {
                        println!("✅ Service stopped successfully");
                        let _ = pid::cleanup_pid();
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to stop service (PID: {}): {}", pid, e);
                    }
                },
                _ => {
                    println!("Service is not running");
                    let _ = pid::cleanup_pid();
                }
            }
        }
        Commands::Restart { detach } => {
            let (launch, config) = resolve_launch_and_config(cli.config.clone())?;
            init_tracing(&config);

            // Stop the existing service
            let was_running = match pid::read_pid() {
                Ok(pid) => {
                    if pid::is_process_running(pid) {
                        println!("Stopping existing service...");
                        match stop_service(pid).await {
                            Ok(_) => true,
                            Err(e) => {
                                eprintln!("Warning: Failed to stop existing service: {}", e);
                                false
                            }
                        }
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };
            let _ = pid::cleanup_pid();

            if detach {
                // Background mode
                println!("Starting service in background...");
                let port_from_config = Some(config.server.port);
                spawn_background_service(port_from_config, cli.config.clone())?;
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    PROCESS_TRANSITION_GRACE_MS,
                ))
                .await;

                let verb = if was_running { "restarted" } else { "started" };
                if let Ok(pid) = pid::read_pid() {
                    println!("✅ Service {} successfully (PID: {})", verb, pid);
                } else {
                    println!("✅ Service {} successfully", verb);
                }
            } else {
                // Foreground mode
                start_foreground(config, launch.clone()).await?;
            }
        }
        Commands::Status => {
            println!("Checking service status...");
            match pid::read_pid() {
                Ok(pid) => {
                    if pid::is_process_running(pid) {
                        println!("✅ Service is running (PID: {})", pid);
                    } else {
                        println!("❌ Service is not running (stale PID file)");
                        let _ = pid::cleanup_pid();
                    }
                }
                Err(_) => {
                    println!("❌ Service is not running");
                }
            }
        }
        Commands::Model => {
            let (_launch, config) = resolve_launch_and_config(cli.config.clone())?;
            init_tracing(&config);

            println!("📊 Model Configuration");
            println!();
            println!("Configured Models:");
            println!("  • Default: {}", config.router.default);
            if let Some(ref think) = config.router.think {
                println!("  • Think: {}", think);
            }
            if let Some(ref ws) = config.router.websearch {
                println!("  • WebSearch: {}", ws);
            }
            if let Some(ref bg) = config.router.background {
                println!("  • Background: {}", bg);
            }
            println!();
            println!("Providers:");
            for provider in &config.providers {
                if provider.enabled.unwrap_or(false) {
                    println!("  • {} ({})", provider.name, provider.provider_type);
                }
            }
        }
        Commands::InstallStatusline => {
            println!("📊 Installing Claude Code Statusline Script");
            println!();

            // Get home directory and create .substrate-gateway directory
            let home =
                dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
            let gateway_dir = home.join(".substrate-gateway");
            std::fs::create_dir_all(&gateway_dir)?;

            // Write statusline script
            let script_path = gateway_dir.join("statusline.sh");
            let script_content = include_str!("../statusline.sh");
            std::fs::write(&script_path, script_content)?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&script_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&script_path, perms)?;
            }

            println!(
                "✅ Statusline script installed to: {}",
                script_path.display()
            );
            println!();
            println!(
                "🧪 Install this before smoke; the script reads ~/.substrate-gateway/last_routing.json"
            );
            println!();
            println!("📝 To use it, add this to ~/.claude/settings.json:");
            println!();
            println!("   {{");
            println!("     \"statusLine\": {{");
            println!("       \"type\": \"command\",");
            println!("       \"command\": \"{}\",", script_path.display());
            println!("       \"padding\": 0");
            println!("     }}");
            println!("   }}");
            println!();
            println!("📊 The statusline will show: model@provider (route-type) HH:MM:SS");
            println!("   Example: minimax-m2@minimax (default) 14:23:45");
        }
    }

    Ok(())
}
