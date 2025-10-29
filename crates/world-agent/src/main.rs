//! World agent binary - runs inside worlds/VMs to provide execution API.

use anyhow::{Context, Result};
use axum::routing::{get, post};
use axum::Router;
use hyper::server::accept::from_stream;
#[cfg(unix)]
use hyperlocal::UnixServerExt;
use std::net::SocketAddr;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

mod gc;
mod handlers;
mod pty;
mod service;

use service::WorldAgentService;

const SOCKET_PATH: &str = "/run/substrate.sock";
const TCP_ENV_VAR: &str = "SUBSTRATE_AGENT_TCP_PORT";

#[tokio::main]
async fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    info!("Starting Substrate World Agent");

    let socket_path = PathBuf::from(SOCKET_PATH);
    prepare_socket_path(&socket_path)?;

    info!("Running initial netns GC sweep");
    let ttl = get_env_u64("SUBSTRATE_NETNS_GC_TTL_SECS", 0)
        .filter(|&t| t > 0)
        .map(std::time::Duration::from_secs);

    match gc::sweep(ttl).await {
        Ok(report) => {
            info!(
                "Initial GC sweep complete: removed={}, kept={}, errors={}",
                report.removed.len(),
                report.kept.len(),
                report.errors.len()
            );
        }
        Err(e) => {
            warn!("Initial GC sweep failed: {}", e);
        }
    }

    let service = WorldAgentService::new()?;
    let router = build_router(service.clone());

    info!("Routes registered, starting listeners");

    let gc_interval_secs = get_env_u64("SUBSTRATE_NETNS_GC_INTERVAL_SECS", 600).unwrap_or(600);
    if gc_interval_secs > 0 {
        info!(
            "Starting periodic GC sweep every {} seconds",
            gc_interval_secs
        );
        spawn_periodic_gc(gc_interval_secs);
    } else {
        info!("Periodic GC sweep disabled");
    }

    let tcp_port = read_tcp_port()?;
    if let Some(port) = tcp_port {
        info!(tcp_port = port, "Loopback TCP listener enabled");
    } else {
        info!("Loopback TCP listener disabled");
    }

    let shutdown = CancellationToken::new();
    tokio::spawn(watch_for_shutdown(shutdown.clone()));

    let uds_task = run_uds_server(router.clone(), socket_path.clone(), shutdown.clone());
    let result = if let Some(port) = tcp_port {
        let tcp_task = run_tcp_server(router, port, shutdown.clone());
        tokio::try_join!(uds_task, tcp_task).map(|_| ())
    } else {
        uds_task.await
    };

    shutdown.cancel();
    if socket_path.exists() {
        if let Err(err) = std::fs::remove_file(&socket_path) {
            warn!(error = %err, "Failed to remove socket on shutdown");
        }
    }

    result
}

fn prepare_socket_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    if path.exists() {
        std::fs::remove_file(path).context("Failed to remove existing socket")?;
    }

    Ok(())
}

fn build_router(service: WorldAgentService) -> Router {
    Router::new()
        .route("/v1/capabilities", get(handlers::capabilities))
        .route("/v1/execute", post(handlers::execute))
        .route("/v1/execute/stream", post(handlers::execute_stream))
        .route("/v1/stream", get(handlers::stream))
        .route("/v1/trace/:span_id", get(handlers::get_trace))
        .route("/v1/request_scopes", post(handlers::request_scopes))
        .route("/v1/gc", post(handlers::gc))
        .with_state(service)
}

#[cfg(unix)]
async fn run_uds_server(
    router: Router,
    socket_path: PathBuf,
    shutdown: CancellationToken,
) -> Result<()> {
    let make_service = router.into_make_service();
    let server = hyper::Server::bind_unix(&socket_path)
        .context("Failed to bind Unix socket listener")?
        .serve(make_service)
        .with_graceful_shutdown(async move {
            shutdown.cancelled().await;
        });

    #[cfg(unix)]
    {
        if let Ok(meta) = std::fs::metadata(&socket_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o666);
            let _ = std::fs::set_permissions(&socket_path, perms);
        }
    }

    info!(socket = %socket_path.display(), "World agent listening on Unix socket");
    server.await.context("Unix socket server failed")
}

#[cfg(not(unix))]
async fn run_uds_server(
    _router: Router,
    _socket_path: PathBuf,
    _shutdown: CancellationToken,
) -> Result<()> {
    // UDS server is not available on non-Unix platforms; forwarder uses TCP inside WSL.
    Ok(())
}

async fn run_tcp_server(router: Router, port: u16, shutdown: CancellationToken) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind TCP listener on {addr}"))?;
    info!(address = %addr, "World agent listening on loopback TCP");

    let stream = TcpListenerStream::new(listener);
    hyper::Server::builder(from_stream(stream))
        .serve(router.into_make_service())
        .with_graceful_shutdown(async move {
            shutdown.cancelled().await;
        })
        .await
        .context("TCP server failed")
}

fn read_tcp_port() -> Result<Option<u16>> {
    match std::env::var(TCP_ENV_VAR) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                anyhow::bail!(
                    "{TCP_ENV_VAR} is set but empty; unset it or provide a valid port number"
                );
            }
            let port: u16 = trimmed
                .parse()
                .with_context(|| format!("Failed to parse {TCP_ENV_VAR}={trimmed}"))?;
            if port == 0 {
                anyhow::bail!("{TCP_ENV_VAR} must be between 1 and 65535 (got 0)");
            }
            Ok(Some(port))
        }
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            anyhow::bail!("{TCP_ENV_VAR} contains non-Unicode data")
        }
    }
}

async fn watch_for_shutdown(token: CancellationToken) {
    shutdown_signal().await;
    info!("Shutdown signal received; closing listeners");
    token.cancel();
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut terminate = match signal(SignalKind::terminate()) {
            Ok(signal) => signal,
            Err(err) => {
                warn!(error = %err, "Failed to install SIGTERM handler");
                if let Err(ctrl_err) = tokio::signal::ctrl_c().await {
                    warn!(error = %ctrl_err, "ctrl_c handler error");
                }
                return;
            }
        };

        tokio::select! {
            res = tokio::signal::ctrl_c() => {
                if let Err(err) = res {
                    warn!(error = %err, "ctrl_c handler error");
                }
            }
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        if let Err(err) = tokio::signal::ctrl_c().await {
            warn!(error = %err, "ctrl_c handler error");
        }
    }
}
fn get_env_u64(key: &str, default: u64) -> Option<u64> {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .or(Some(default))
}

fn spawn_periodic_gc(interval_secs: u64) {
    use tokio::time::{interval, Duration};

    tokio::spawn(async move {
        let base_interval = Duration::from_secs(interval_secs);
        let jitter_range = (interval_secs as f64 * 0.1) as u64;

        let mut interval = interval(base_interval);
        interval.tick().await; // Skip first immediate tick

        loop {
            interval.tick().await;

            let jitter = if jitter_range > 0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                Duration::from_secs(rng.gen_range(0..jitter_range))
            } else {
                Duration::ZERO
            };

            tokio::time::sleep(jitter).await;

            let ttl = get_env_u64("SUBSTRATE_NETNS_GC_TTL_SECS", 0)
                .filter(|&t| t > 0)
                .map(Duration::from_secs);

            info!("Starting periodic GC sweep");
            match gc::sweep(ttl).await {
                Ok(report) => {
                    info!(
                        "Periodic GC sweep complete: removed={}, kept={}, errors={}",
                        report.removed.len(),
                        report.kept.len(),
                        report.errors.len()
                    );
                }
                Err(e) => {
                    warn!("Periodic GC sweep failed: {}", e);
                }
            }
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn reset_env() {
        std::env::remove_var(TCP_ENV_VAR);
    }

    #[test]
    fn read_tcp_port_returns_none_when_unset() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        assert!(read_tcp_port().unwrap().is_none());
    }

    #[test]
    fn read_tcp_port_parses_value() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        std::env::set_var(TCP_ENV_VAR, "55001");
        assert_eq!(read_tcp_port().unwrap(), Some(55001));
        reset_env();
    }

    #[test]
    fn read_tcp_port_rejects_invalid() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        std::env::set_var(TCP_ENV_VAR, "not-a-number");
        let err = read_tcp_port().unwrap_err();
        assert!(err.to_string().contains("Failed to parse"));
        reset_env();
    }
}
