//! Host proxy server binary.

use std::path::PathBuf;
use std::sync::Arc;

use agent_api_core::build_router;
use anyhow::{Context, Result};
use host_proxy::{cleanup_socket, ensure_socket_dir, HostProxyService, ProxyConfig};
use tower::ServiceExt;
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::info;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting host-proxy server");

    // Load configuration
    let config = load_config()?;
    info!("Configuration loaded: {:?}", config);

    // Ensure socket directory exists and clean up old socket
    ensure_socket_dir(&config.host_socket).await?;
    cleanup_socket(&config.host_socket).await?;

    // Create the proxy service
    let service = Arc::new(
        HostProxyService::new(config.clone()).context("Failed to create proxy service")?,
    );

    // Build the router with agent API routes
    let api_router = build_router(service);

    // Add middleware and additional routes
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

    // Bind to Unix socket
    let socket_path = config.host_socket.clone();
    info!("Binding to Unix socket: {:?}", socket_path);

    let listener = tokio::net::UnixListener::bind(&socket_path)
        .context("Failed to bind to Unix socket")?;

    // Set socket permissions to be accessible
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o666))
            .context("Failed to set socket permissions")?;
    }

    info!("Host proxy listening on: {:?}", socket_path);
    info!("Ready to forward requests to world-agent");

    // Accept loop for Unix socket connections
    loop {
        let (stream, _addr) = listener
            .accept()
            .await
            .context("Failed to accept connection")?;

        let app = app.clone();

        tokio::spawn(async move {
            let io = hyper_util::rt::TokioIo::new(stream);
            let hyper_service = hyper::service::service_fn(move |request| {
                app.clone().oneshot(request)
            });

            if let Err(err) = hyper_util::server::conn::auto::Builder::new(
                hyper_util::rt::TokioExecutor::new(),
            )
            .serve_connection_with_upgrades(io, hyper_service)
            .await
            {
                tracing::error!("Failed to serve connection: {}", err);
            }
        });
    }
}

/// Load configuration from environment or defaults.
fn load_config() -> Result<ProxyConfig> {
    // Try to load from environment variables
    let mut config = ProxyConfig::default();

    if let Ok(host_socket) = std::env::var("HOST_PROXY_SOCKET") {
        config.host_socket = PathBuf::from(host_socket);
    }

    if let Ok(agent_socket) = std::env::var("AGENT_SOCKET") {
        config.agent_socket = PathBuf::from(agent_socket);
    }

    if let Ok(max_body) = std::env::var("MAX_BODY_SIZE") {
        config.max_body_size = max_body.parse().unwrap_or(config.max_body_size);
    }

    if let Ok(timeout) = std::env::var("REQUEST_TIMEOUT") {
        config.request_timeout = timeout.parse().unwrap_or(config.request_timeout);
    }

    // Rate limiting from env
    if let Ok(rpm) = std::env::var("RATE_LIMIT_RPM") {
        config.rate_limits.requests_per_minute = rpm.parse().unwrap_or(60);
    }

    if let Ok(max_concurrent) = std::env::var("RATE_LIMIT_CONCURRENT") {
        config.rate_limits.max_concurrent = max_concurrent.parse().unwrap_or(5);
    }

    // Auth from env
    if let Ok(auth_enabled) = std::env::var("AUTH_ENABLED") {
        config.auth.enabled = auth_enabled.parse().unwrap_or(false);
    }

    if let Ok(token_file) = std::env::var("AUTH_TOKEN_FILE") {
        config.auth.token_file = Some(PathBuf::from(token_file));
    }

    Ok(config)
}