//! World agent library for execution inside worlds/VMs.

pub mod gc;
pub mod handlers;
pub mod pty;
pub mod service;
#[cfg(unix)]
mod socket_activation;

pub use service::WorldAgentService;

#[cfg(unix)]
use crate::socket_activation::{
    collect_socket_activation, InheritedUnixListener, SocketActivation,
};
use anyhow::{Context, Result};
use axum::routing::{get, post};
use axum::Router;
use futures_util::future::{try_join_all, BoxFuture};
use futures_util::FutureExt;
use hyper::server::accept::from_stream;
use std::net::SocketAddr;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use substrate_broker::{set_global_broker, BrokerHandle};
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio_stream::wrappers::TcpListenerStream;
#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

const SOCKET_PATH: &str = "/run/substrate.sock";
const TCP_ENV_VAR: &str = "SUBSTRATE_AGENT_TCP_PORT";

pub async fn run_world_agent() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let _ = set_global_broker(BrokerHandle::new());

    info!("Starting Substrate World Agent");

    let socket_path = PathBuf::from(SOCKET_PATH);
    #[cfg(unix)]
    let mut socket_activation = collect_socket_activation()?;
    #[cfg(unix)]
    log_listener_mode(socket_activation.as_ref());
    #[cfg(not(unix))]
    info!(
        component = "world-agent",
        event = "listener_state",
        mode = "direct_bind",
        listen_fds = 0,
        uds_inherited = 0,
        tcp_inherited = 0,
        "Socket activation is unavailable; binding listeners directly"
    );

    #[cfg(unix)]
    let inherited_uds = socket_activation
        .as_mut()
        .and_then(|activation| activation.unix_listeners.pop());
    #[cfg(unix)]
    {
        if let Some(ref mut activation) = socket_activation {
            if !activation.unix_listeners.is_empty() && inherited_uds.is_some() {
                warn!(
                    remaining = activation.unix_listeners.len(),
                    "Multiple inherited Unix sockets detected; only the first will be consumed"
                );
            }
        }
    }
    #[cfg(unix)]
    let uds_cleanup_required = inherited_uds.is_none();
    #[cfg(unix)]
    if socket_activation.is_some() && inherited_uds.is_none() {
        warn!("LISTEN_FDS was set but no Unix stream sockets were provided; falling back to direct bind");
    }
    #[cfg(unix)]
    if uds_cleanup_required {
        prepare_socket_path(&socket_path)?;
    }

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

    let mut tcp_handles: Vec<TcpListenerHandle> = Vec::new();
    #[cfg(unix)]
    if let Some(ref mut activation) = socket_activation {
        for listener in activation.tcp_listeners.drain(..) {
            tcp_handles.push(TcpListenerHandle {
                listener: listener.listener,
                source: TcpListenerSource::SocketActivation {
                    fd: listener.fd,
                    name: listener.name,
                },
            });
        }
    }

    let tcp_port = read_tcp_port()?;
    if let Some(port) = tcp_port {
        if tcp_handles.is_empty() {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            let listener = TcpListener::bind(addr)
                .await
                .with_context(|| format!("Failed to bind TCP listener on {addr}"))?;
            info!(
                component = "world-agent",
                event = "listener_state",
                listener_kind = "tcp",
                listener_mode = "direct_bind",
                tcp_port = port,
                "Loopback TCP listener enabled via environment configuration"
            );
            tcp_handles.push(TcpListenerHandle {
                listener,
                source: TcpListenerSource::Manual,
            });
        } else {
            info!(
                component = "world-agent",
                event = "listener_state",
                listener_kind = "tcp",
                listener_mode = "socket_activation",
                tcp_listeners = tcp_handles.len(),
                env_override = port,
                "SUBSTRATE_AGENT_TCP_PORT ignored because TCP listeners were inherited"
            );
        }
    } else if tcp_handles.is_empty() {
        info!(
            component = "world-agent",
            event = "listener_state",
            listener_kind = "tcp",
            listener_mode = "disabled",
            "Loopback TCP listener disabled"
        );
    } else {
        info!(
            component = "world-agent",
            event = "listener_state",
            listener_kind = "tcp",
            listener_mode = "socket_activation",
            tcp_listeners = tcp_handles.len(),
            "Loopback TCP listener(s) provided via socket activation"
        );
    }

    let shutdown = CancellationToken::new();
    tokio::spawn(watch_for_shutdown(shutdown.clone()));

    let mut server_tasks: Vec<BoxFuture<'static, Result<()>>> = Vec::new();
    #[cfg(unix)]
    {
        let uds_future = run_uds_server(
            router.clone(),
            socket_path.clone(),
            inherited_uds,
            shutdown.clone(),
        );
        server_tasks.push(uds_future.boxed());
    }
    #[cfg(not(unix))]
    {
        server_tasks
            .push(run_uds_server(router.clone(), socket_path.clone(), shutdown.clone()).boxed());
    }

    for handle in tcp_handles {
        let router_clone = router.clone();
        let shutdown_clone = shutdown.clone();
        server_tasks.push(run_tcp_server(router_clone, handle, shutdown_clone).boxed());
    }

    let result = try_join_all(server_tasks).await.map(|_| ());

    shutdown.cancel();
    #[cfg(unix)]
    if uds_cleanup_required && socket_path.exists() {
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
fn log_listener_mode(activation: Option<&SocketActivation>) {
    if let Some(activation) = activation {
        info!(
            component = "world-agent",
            event = "listener_state",
            mode = "socket_activation",
            listen_fds = activation.total_fds,
            uds_inherited = activation.unix_listeners.len(),
            tcp_inherited = activation.tcp_listeners.len(),
            "Using inherited listeners via LISTEN_FDS"
        );
    } else {
        info!(
            component = "world-agent",
            event = "listener_state",
            mode = "direct_bind",
            listen_fds = 0,
            uds_inherited = 0,
            tcp_inherited = 0,
            "No inherited listeners detected; binding sockets directly"
        );
    }
}

#[cfg(unix)]
async fn run_uds_server(
    router: Router,
    socket_path: PathBuf,
    inherited: Option<InheritedUnixListener>,
    shutdown: CancellationToken,
) -> Result<()> {
    let (listener, mode, fd, name) = match inherited {
        Some(inherited) => (
            inherited.listener,
            "socket_activation",
            Some(inherited.fd),
            inherited.name,
        ),
        None => {
            let uds = UnixListener::bind(&socket_path)
                .with_context(|| format!("Failed to bind Unix socket {}", socket_path.display()))?;
            if let Ok(meta) = std::fs::metadata(&socket_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o666);
                let _ = std::fs::set_permissions(&socket_path, perms);
            }
            (uds, "direct_bind", None, None)
        }
    };

    let stream = UnixListenerStream::new(listener);
    let inherited_name = name.as_deref().unwrap_or("");
    info!(
        component = "world-agent",
        event = "listener_ready",
        listener_kind = "uds",
        listener_mode = mode,
        inherited_fd = fd,
        inherited_name,
        socket = %socket_path.display(),
        "World agent listening on Unix socket"
    );

    hyper::Server::builder(from_stream(stream))
        .serve(router.into_make_service())
        .with_graceful_shutdown(async move {
            shutdown.cancelled().await;
        })
        .await
        .context("Unix socket server failed")
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

struct TcpListenerHandle {
    listener: TcpListener,
    source: TcpListenerSource,
}

enum TcpListenerSource {
    Manual,
    #[cfg(unix)]
    SocketActivation {
        fd: i32,
        name: Option<String>,
    },
}

impl TcpListenerSource {
    fn mode(&self) -> &'static str {
        match self {
            TcpListenerSource::Manual => "direct_bind",
            #[cfg(unix)]
            TcpListenerSource::SocketActivation { .. } => "socket_activation",
        }
    }

    #[cfg(unix)]
    fn fd(&self) -> Option<i32> {
        match self {
            TcpListenerSource::Manual => None,
            TcpListenerSource::SocketActivation { fd, .. } => Some(*fd),
        }
    }

    #[cfg(not(unix))]
    fn fd(&self) -> Option<i32> {
        let _ = self;
        None
    }

    #[cfg(unix)]
    fn name(&self) -> Option<&str> {
        match self {
            TcpListenerSource::Manual => None,
            TcpListenerSource::SocketActivation { name, .. } => name.as_deref(),
        }
    }

    #[cfg(not(unix))]
    fn name(&self) -> Option<&str> {
        let _ = self;
        None
    }
}

async fn run_tcp_server(
    router: Router,
    handle: TcpListenerHandle,
    shutdown: CancellationToken,
) -> Result<()> {
    let TcpListenerHandle { listener, source } = handle;
    let addr = listener
        .local_addr()
        .context("Failed to read TCP listener address")?;
    let inherited_fd = source.fd();
    let inherited_name = source.name().unwrap_or("");
    info!(
        component = "world-agent",
        event = "listener_ready",
        listener_kind = "tcp",
        listener_mode = source.mode(),
        tcp_port = addr.port(),
        inherited_fd,
        inherited_name,
        "World agent listening on loopback TCP"
    );

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
