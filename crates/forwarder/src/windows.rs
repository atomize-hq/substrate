use crate::config::ForwarderConfig;
use anyhow::{anyhow, Context};
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

    /// Authenticated install bootstrap carrier for internal child mode
    #[arg(long, hide = true)]
    install_bootstrap_context_v1: Option<String>,

    /// Authenticated platform bootstrap mapping for internal child mode
    #[arg(long, hide = true)]
    platform_bootstrap_mapping_v1: Option<String>,

    /// Run without console output (service-friendly)
    #[arg(long)]
    run_as_service: bool,
}

impl Cli {
    fn resolve_log_dir(&self) -> anyhow::Result<PathBuf> {
        if let Some(dir) = &self.log_dir {
            return Ok(dir.clone());
        }
        if self.install_bootstrap_context_v1.is_some()
            || self.platform_bootstrap_mapping_v1.is_some()
        {
            anyhow::bail!("internal authenticated forwarder mode requires explicit --log-dir");
        }
        let base = std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA not set")?;
        let mut path = PathBuf::from(base);
        path.push("Substrate");
        path.push("logs");
        Ok(path)
    }
}

#[tokio::main(flavor = "multi_thread")]
pub async fn run() -> anyhow::Result<()> {
    let argv = std::env::args_os().collect::<Vec<_>>();
    let count_flag = |flag: &str| -> usize {
        argv.iter()
            .skip(1)
            .filter(|arg| {
                let arg = arg.to_string_lossy();
                arg == flag
                    || arg
                        .strip_prefix(flag)
                        .is_some_and(|suffix| suffix.starts_with('='))
            })
            .count()
    };

    let cli = Cli::try_parse_from(argv.clone())
        .map_err(|err| anyhow!("invalid substrate-forwarder arguments: {err}"))?;
    let internal_requested =
        cli.install_bootstrap_context_v1.is_some() || cli.platform_bootstrap_mapping_v1.is_some();

    if internal_requested {
        for flag in [
            "--install-bootstrap-context-v1",
            "--platform-bootstrap-mapping-v1",
            "--distro",
            "--pipe",
            "--config",
            "--log-dir",
        ] {
            if count_flag(flag) != 1 {
                return Err(anyhow!(
                    "internal authenticated forwarder mode requires exactly one {flag}"
                ));
            }
        }
        if count_flag("--tcp-bridge") > 1 {
            return Err(anyhow!(
                "internal authenticated forwarder mode requires at most one --tcp-bridge"
            ));
        }
    }

    let log_dir = cli.resolve_log_dir()?;
    let forwarder_config = if internal_requested {
        let config_path = cli
            .config
            .clone()
            .context("internal authenticated forwarder mode requires explicit --config")?;
        let install_bootstrap_context_v1 = cli.install_bootstrap_context_v1.as_deref().context(
            "internal authenticated forwarder mode requires --install-bootstrap-context-v1",
        )?;
        let platform_bootstrap_mapping_v1 = cli.platform_bootstrap_mapping_v1.as_deref().context(
            "internal authenticated forwarder mode requires --platform-bootstrap-mapping-v1",
        )?;
        ForwarderConfig::load_internal(
            cli.distro.clone(),
            cli.pipe.clone(),
            cli.tcp_bridge,
            config_path,
            log_dir.clone(),
            install_bootstrap_context_v1,
            platform_bootstrap_mapping_v1,
        )
    } else {
        ForwarderConfig::load(
            cli.distro.clone(),
            cli.pipe.clone(),
            cli.tcp_bridge,
            cli.config.clone(),
        )
    }
    .with_context(|| "failed to load forwarder configuration")?;

    let _guard = crate::logging::init(&log_dir, cli.run_as_service)
        .with_context(|| format!("failed to initialize logging at {}", log_dir.display()))?;
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
    join_set.spawn(crate::pipe::serve(config.clone(), cancel.clone()));
    if let Some(addr) = config.host_tcp_bridge {
        join_set.spawn(crate::tcp::serve(addr, config.clone(), cancel.clone()));
    }

    tokio::select! {
        _ = cancel.cancelled() => {
            tracing::info!("shutdown requested");
        }
        Some(res) = join_set.join_next() => {
            match res {
                Ok(Ok(())) => { tracing::info!("listener task exited cleanly"); }
                Ok(Err(err)) => { tracing::error!(error = %err, "listener task failed" ); return Err(err); }
                Err(join_err) => { tracing::error!("listener task panicked: {join_err}"); return Err(anyhow::anyhow!(join_err)); }
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
    use std::ffi::OsString;
    use std::sync::Mutex;
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
        WindowsForwarderScopeV1,
    };

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn load_forwarder_config_for_test(args: &[&str]) -> anyhow::Result<(PathBuf, ForwarderConfig)> {
        let argv = args.iter().map(OsString::from).collect::<Vec<_>>();
        let count_flag = |flag: &str| -> usize {
            argv.iter()
                .skip(1)
                .filter(|arg| {
                    let arg = arg.to_string_lossy();
                    arg == flag
                        || arg
                            .strip_prefix(flag)
                            .is_some_and(|suffix| suffix.starts_with('='))
                })
                .count()
        };

        let cli = Cli::try_parse_from(argv.clone())
            .map_err(|err| anyhow!("invalid substrate-forwarder arguments: {err}"))?;
        let internal_requested = cli.install_bootstrap_context_v1.is_some()
            || cli.platform_bootstrap_mapping_v1.is_some();

        if internal_requested {
            for flag in [
                "--install-bootstrap-context-v1",
                "--platform-bootstrap-mapping-v1",
                "--distro",
                "--pipe",
                "--config",
                "--log-dir",
            ] {
                if count_flag(flag) != 1 {
                    return Err(anyhow!(
                        "internal authenticated forwarder mode requires exactly one {flag}"
                    ));
                }
            }
            if count_flag("--tcp-bridge") > 1 {
                return Err(anyhow!(
                    "internal authenticated forwarder mode requires at most one --tcp-bridge"
                ));
            }
        }

        let log_dir = cli.resolve_log_dir()?;
        let forwarder_config = if internal_requested {
            let config_path = cli
                .config
                .clone()
                .context("internal authenticated forwarder mode requires explicit --config")?;
            let install_bootstrap_context_v1 =
                cli.install_bootstrap_context_v1.as_deref().context(
                    "internal authenticated forwarder mode requires --install-bootstrap-context-v1",
                )?;
            let platform_bootstrap_mapping_v1 = cli
                .platform_bootstrap_mapping_v1
                .as_deref()
                .context(
                "internal authenticated forwarder mode requires --platform-bootstrap-mapping-v1",
            )?;
            ForwarderConfig::load_internal(
                cli.distro.clone(),
                cli.pipe.clone(),
                cli.tcp_bridge,
                config_path,
                log_dir.clone(),
                install_bootstrap_context_v1,
                platform_bootstrap_mapping_v1,
            )
        } else {
            ForwarderConfig::load(
                cli.distro.clone(),
                cli.pipe.clone(),
                cli.tcp_bridge,
                cli.config.clone(),
            )
        }
        .with_context(|| "failed to load forwarder configuration")?;

        Ok((log_dir, forwarder_config))
    }

    fn sample_internal_state() -> (String, String) {
        let host_carrier = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_windows(
                r"C:\Users\Alice\AppData\Local\Substrate",
                r"ACME\Alice",
                "S-1-5-21-1000",
            )
            .unwrap(),
        )
        .unwrap();
        let scope = WindowsForwarderScopeV1::derive(
            "S-1-5-21-1000",
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            r"\\.\pipe\substrate-agent",
        )
        .unwrap();
        let mapping = PlatformBootstrapMappingV1::new_wsl(
            &host_carrier,
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            &format!(
                r"C:\Users\Alice\AppData\Local\Substrate\forwarder\{}",
                scope.0
            ),
            "/home/substrate/.substrate",
            "substrate",
            1000,
            r"\\.\pipe\substrate-agent",
            "/run/substrate.sock",
        )
        .unwrap();
        (
            host_carrier.encode().unwrap(),
            mapping.encode(&host_carrier).unwrap(),
        )
    }

    #[test]
    fn cli_defaults() {
        let cli = Cli::parse_from(["substrate-forwarder"]);
        assert_eq!(cli.distro, "substrate-wsl");
        assert_eq!(cli.pipe, r"\\.\pipe\substrate-agent");
        assert!(cli.tcp_bridge.is_none());
        assert!(cli.config.is_none());
        assert!(cli.install_bootstrap_context_v1.is_none());
        assert!(cli.platform_bootstrap_mapping_v1.is_none());
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
            "--install-bootstrap-context-v1",
            "carrier",
            "--platform-bootstrap-mapping-v1",
            "mapping",
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
        assert_eq!(cli.install_bootstrap_context_v1.as_deref(), Some("carrier"));
        assert_eq!(
            cli.platform_bootstrap_mapping_v1.as_deref(),
            Some("mapping")
        );
    }

    #[test]
    fn internal_cli_requires_explicit_log_dir() {
        let cli = Cli::parse_from([
            "substrate-forwarder",
            "--distro",
            "Substrate-WSL",
            "--pipe",
            r"\\.\pipe\substrate-agent",
            "--config",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml",
            "--install-bootstrap-context-v1",
            "carrier",
            "--platform-bootstrap-mapping-v1",
            "mapping",
        ]);
        let err = cli.resolve_log_dir().unwrap_err();
        assert!(
            err.to_string().contains("requires explicit --log-dir"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn internal_cli_uses_explicit_log_dir() {
        let cli = Cli::parse_from([
            "substrate-forwarder",
            "--distro",
            "Substrate-WSL",
            "--pipe",
            r"\\.\pipe\substrate-agent",
            "--config",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml",
            "--log-dir",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs",
            "--install-bootstrap-context-v1",
            "carrier",
            "--platform-bootstrap-mapping-v1",
            "mapping",
        ]);
        assert_eq!(
            cli.resolve_log_dir().unwrap(),
            PathBuf::from(r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs")
        );
    }

    #[test]
    fn internal_preflight_requires_complete_authenticated_surface() {
        let missing_carrier = load_forwarder_config_for_test(&[
            "substrate-forwarder",
            "--distro",
            "Substrate-WSL",
            "--pipe",
            r"\\.\pipe\substrate-agent",
            "--platform-bootstrap-mapping-v1",
            "mapping",
            "--config",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml",
            "--log-dir",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs",
        ])
        .unwrap_err();
        assert!(
            missing_carrier
                .to_string()
                .contains("requires exactly one --install-bootstrap-context-v1"),
            "unexpected error: {missing_carrier}"
        );

        let duplicate_config = load_forwarder_config_for_test(&[
            "substrate-forwarder",
            "--distro",
            "Substrate-WSL",
            "--pipe",
            r"\\.\pipe\substrate-agent",
            "--install-bootstrap-context-v1",
            "carrier",
            "--platform-bootstrap-mapping-v1",
            "mapping",
            "--config",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml",
            "--config",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\other.toml",
            "--log-dir",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs",
        ])
        .unwrap_err();
        assert!(
            duplicate_config
                .to_string()
                .contains("requires exactly one --config")
                || duplicate_config
                    .to_string()
                    .contains("argument '--config <CONFIG>' cannot be used multiple times"),
            "unexpected error: {duplicate_config}"
        );

        let duplicate_tcp = load_forwarder_config_for_test(&[
            "substrate-forwarder",
            "--distro",
            "Substrate-WSL",
            "--pipe",
            r"\\.\pipe\substrate-agent",
            "--install-bootstrap-context-v1",
            "carrier",
            "--platform-bootstrap-mapping-v1",
            "mapping",
            "--config",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml",
            "--log-dir",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs",
            "--tcp-bridge",
            "127.0.0.1:5000",
            "--tcp-bridge",
            "127.0.0.1:5001",
        ])
        .unwrap_err();
        assert!(
            duplicate_tcp
                .to_string()
                .contains("requires at most one --tcp-bridge")
                || duplicate_tcp
                    .to_string()
                    .contains("argument '--tcp-bridge <TCP_BRIDGE>' cannot be used multiple times"),
            "unexpected error: {duplicate_tcp}"
        );
    }

    #[test]
    fn internal_preflight_uses_explicit_paths_not_ambient_windows_vars() {
        let _guard = ENV_GUARD.lock().unwrap();
        std::env::set_var("LOCALAPPDATA", r"C:\Ambient\WrongLocalAppData");
        std::env::set_var("USERPROFILE", r"C:\Ambient\UserProfile");
        std::env::set_var("WSLENV", "SUBSTRATE_FORWARDER_TARGET/u");
        std::env::set_var("SUBSTRATE_FORWARDER_TARGET", "uds:/run/substrate.sock");
        crate::config::set_test_file_settings_override(Some(None));
        crate::config::set_test_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping) = sample_internal_state();

        let (log_dir, config) = load_forwarder_config_for_test(&[
            "substrate-forwarder",
            "--distro",
            "Substrate-WSL",
            "--pipe",
            r"\\.\pipe\substrate-agent",
            "--install-bootstrap-context-v1",
            &carrier,
            "--platform-bootstrap-mapping-v1",
            &mapping,
            "--config",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml",
            "--log-dir",
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs",
        ])
        .unwrap();

        assert_eq!(
            log_dir,
            PathBuf::from(r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs")
        );
        assert_eq!(config.target_mode(), "uds");
        assert_eq!(config.target().to_string(), "/run/substrate.sock");

        crate::config::set_test_current_windows_host_observation(None);
        crate::config::set_test_file_settings_override(None);
        std::env::remove_var("SUBSTRATE_FORWARDER_TARGET");
        std::env::remove_var("WSLENV");
        std::env::remove_var("USERPROFILE");
        std::env::remove_var("LOCALAPPDATA");
    }
}
