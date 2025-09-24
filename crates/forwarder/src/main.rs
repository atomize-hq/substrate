#![cfg(target_os = "windows")]

mod bridge;
mod config;
mod logging;
mod pipe;
mod tcp;
mod wsl;

use crate::config::ForwarderConfig;
use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

#[derive(Parser, Debug)]
#[command(
    name = "substrate-forwarder",
    about = "Bridge Windows named pipes to WSL world agent",
    version,
    disable_help_subcommand = true
)]
struct Cli {
    /// WSL distribution name that hosts the substrate agent
    #[arg(long, default_value = "substrate-wsl")]
    distro: String,

    /// Windows named pipe path exposed to host processes
    #[arg(long, default_value = r"\\.\pipe\substrate-agent")]
    pipe: String,

    /// Optional TCP address for compatibility fallback (e.g. 127.0.0.1:17788)
    #[arg(long)]
    tcp_bridge: Option<std::net::SocketAddr>,

    /// Directory for structured logs (defaults to %LOCALAPPDATA%\Substrate\logs)
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Optional path to the forwarder configuration file (defaults to %LOCALAPPDATA%\Substrate\forwarder.toml)
    #[arg(long)]
    config: Option<PathBuf>,

    /// Run without console output (service-friendly)
    #[arg(long)]
    run_as_service: bool,
}

impl Cli {
    fn resolve_log_dir(&self) -> anyhow::Result<PathBuf> {
        if let Some(dir) = &self.log_dir {
            return Ok(dir.clone());
        }
        let base = std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA not set")?;
        let mut path = PathBuf::from(base);
        path.push("Substrate");
        path.push("logs");
        Ok(path)
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_dir = cli.resolve_log_dir()?;
    let _guard = logging::init(&log_dir, cli.run_as_service)
        .with_context(|| format!("failed to initialize logging at {}", log_dir.display()))?;

    let forwarder_config = ForwarderConfig::load(
        cli.distro.clone(),
        cli.pipe.clone(),
        cli.tcp_bridge,
        cli.config.clone(),
    )
    .with_context(|| "failed to load forwarder configuration")?;
    let config = Arc::new(forwarder_config);

    let config_path_display = cli.config.as_ref().map(|path| path.display().to_string());

    tracing::info!(
        distro = %config.distro,
        pipe = %config.pipe_path,
        host_tcp_bridge = ?config.host_tcp_bridge,
        target_mode = config.target_mode(),
        target = %config.target(),
        config_path = config_path_display.as_deref(),
        "starting substrate-forwarder"
    );

    let cancel = CancellationToken::new();
    install_ctrlc_handler(cancel.clone());

    let mut join_set = JoinSet::new();
    join_set.spawn(pipe::serve(config.clone(), cancel.clone()));
    if let Some(addr) = config.host_tcp_bridge {
        join_set.spawn(tcp::serve(addr, config.clone(), cancel.clone()));
    }

    tokio::select! {
        _ = cancel.cancelled() => {
            tracing::info!("shutdown requested");
        }
        Some(res) = join_set.join_next() => {
            match res {
                Ok(Ok(())) => {
                    tracing::info!("listener task exited cleanly");
                }
                Ok(Err(err)) => {
                    tracing::error!(error = %err, "listener task failed" );
                }
                Err(join_err) => {
                    tracing::error!("listener task panicked: {join_err}");
                }
            }
            cancel.cancel();
        }
    }

    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(Ok(())) => {}
            Ok(Err(err)) => tracing::warn!(error = %err, "background task finished with error"),
            Err(join_err) => tracing::warn!("background task panicked: {join_err}"),
        }
    }

    tracing::info!("forwarder shutdown complete");
    Ok(())
}

fn install_ctrlc_handler(cancel: CancellationToken) {
    if let Err(err) = ctrlc::set_handler(move || {
        tracing::warn!("CTRL+C received, initiating shutdown");
        cancel.cancel();
    }) {
        tracing::warn!("failed to install ctrl-c handler: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_defaults() {
        let cli = Cli::parse_from(["substrate-forwarder"]);
        assert_eq!(cli.distro, "substrate-wsl");
        assert_eq!(cli.pipe, r"\\.\pipe\substrate-agent");
        assert!(cli.tcp_bridge.is_none());
        assert!(cli.config.is_none());
    }

    #[test]
    fn cli_overrides() {
        let cli = Cli::parse_from([
            "substrate-forwarder",
            "--distro",
            "alt",
            "--pipe",
            r"\\.\pipe\custom",
            "--tcp-bridge",
            "127.0.0.1:5000",
            "--log-dir",
            "C:/tmp/logs",
            "--config",
            "C:/tmp/forwarder.toml",
            "--run-as-service",
        ]);
        assert_eq!(cli.distro, "alt");
        assert_eq!(cli.pipe, r"\\.\pipe\custom");
        assert_eq!(cli.tcp_bridge.unwrap().port(), 5000);
        assert!(cli.run_as_service);
        assert_eq!(cli.log_dir.as_ref().unwrap(), &PathBuf::from("C:/tmp/logs"));
        assert_eq!(
            cli.config.as_ref().unwrap(),
            &PathBuf::from("C:/tmp/forwarder.toml")
        );
    }
}
