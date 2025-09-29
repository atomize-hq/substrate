//! Host proxy server binary.

use anyhow::Result;

#[cfg(unix)]
use {
    agent_api_core::build_router,
    anyhow::Context,
    host_proxy::{
        cleanup_socket, ensure_socket_dir, AgentTransportConfig, HostProxyService, ProxyConfig,
    },
    std::path::PathBuf,
    std::sync::Arc,
    tower::ServiceBuilder,
    tower::ServiceExt,
    tower_http::limit::RequestBodyLimitLayer,
    tracing::info,
    tracing_subscriber::prelude::*,
};

#[cfg(not(unix))]
use anyhow::anyhow;

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting host-proxy server");

    let config = load_config()?;
    info!("Configuration loaded: {:?}", config);

    ensure_socket_dir(&config.host_socket).await?;
    cleanup_socket(&config.host_socket).await?;

    let service =
        Arc::new(HostProxyService::new(config.clone()).context("Failed to create proxy service")?);

    let api_router = build_router(service);

    let app = api_router
        .route(
            "/health",
            axum::routing::get(host_proxy::middleware::health_check),
        )
        .layer(axum::middleware::from_fn(
            host_proxy::middleware::logging_middleware,
        ))
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(config.max_body_size))
                .into_inner(),
        );

    let socket_path = config.host_socket.clone();
    info!("Binding to Unix socket: {:?}", socket_path);

    let listener =
        tokio::net::UnixListener::bind(&socket_path).context("Failed to bind to Unix socket")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o666))
            .context("Failed to set socket permissions")?;
    }

    info!("Host proxy listening on: {:?}", socket_path);
    info!("Ready to forward requests to world-agent");

    loop {
        let (stream, _addr) = listener
            .accept()
            .await
            .context("Failed to accept connection")?;

        let app = app.clone();

        tokio::spawn(async move {
            let io = hyper_util::rt::TokioIo::new(stream);
            let hyper_service =
                hyper::service::service_fn(move |request| app.clone().oneshot(request));

            if let Err(err) =
                hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                    .serve_connection_with_upgrades(io, hyper_service)
                    .await
            {
                tracing::error!("Failed to serve connection: {}", err);
            }
        });
    }
}

#[cfg(not(unix))]
fn main() -> Result<()> {
    Err(anyhow!(
        "host-proxy binary is not supported on this platform"
    ))
}

#[cfg(unix)]
fn load_config() -> Result<ProxyConfig> {
    let mut config = ProxyConfig::default();

    if let Ok(host_socket) = std::env::var("HOST_PROXY_SOCKET") {
        config.host_socket = PathBuf::from(host_socket);
    }

    if let Some(agent_transport) = agent_transport_from_env()? {
        config.agent = agent_transport;
    }

    if let Ok(max_body) = std::env::var("MAX_BODY_SIZE") {
        config.max_body_size = max_body.parse().unwrap_or(config.max_body_size);
    }

    if let Ok(timeout) = std::env::var("REQUEST_TIMEOUT") {
        config.request_timeout = timeout.parse().unwrap_or(config.request_timeout);
    }

    if let Ok(rpm) = std::env::var("RATE_LIMIT_RPM") {
        config.rate_limits.requests_per_minute = rpm.parse().unwrap_or(60);
    }

    if let Ok(max_concurrent) = std::env::var("RATE_LIMIT_CONCURRENT") {
        config.rate_limits.max_concurrent = max_concurrent.parse().unwrap_or(5);
    }

    if let Ok(auth_enabled) = std::env::var("AUTH_ENABLED") {
        config.auth.enabled = auth_enabled.parse().unwrap_or(false);
    }

    if let Ok(token_file) = std::env::var("AUTH_TOKEN_FILE") {
        config.auth.token_file = Some(PathBuf::from(token_file));
    }

    Ok(config)
}

#[cfg(unix)]
fn agent_transport_from_env() -> Result<Option<AgentTransportConfig>> {
    if let Ok(value) = std::env::var("SUBSTRATE_AGENT_TRANSPORT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(AgentTransportConfig::from_uri(trimmed)?));
        }
    }

    if let Ok(value) = std::env::var("AGENT_TRANSPORT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            if let Ok(parsed) = AgentTransportConfig::from_uri(trimmed) {
                return Ok(Some(parsed));
            }

            match trimmed.to_ascii_lowercase().as_str() {
                "tcp" => {
                    let host =
                        std::env::var("AGENT_TCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
                    let port = std::env::var("AGENT_TCP_PORT")
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(17788);
                    return Ok(Some(AgentTransportConfig::Tcp { host, port }));
                }
                "unix" | "uds" => {
                    if let Ok(agent_socket) = std::env::var("AGENT_SOCKET") {
                        return Ok(Some(AgentTransportConfig::Unix {
                            path: PathBuf::from(agent_socket),
                        }));
                    }
                }
                _ => {}
            }
        }
    }

    if let Ok(agent_socket) = std::env::var("AGENT_SOCKET") {
        return Ok(Some(AgentTransportConfig::Unix {
            path: PathBuf::from(agent_socket),
        }));
    }

    Ok(None)
}
