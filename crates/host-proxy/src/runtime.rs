use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use agent_api_client::AgentClient;
use agent_api_core::AgentService;
use agent_api_types::{ApiError, ExecuteRequest, ExecuteResponse};
use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::{body::Body, response::Response};
use http_body_util::BodyExt;
use tokio::sync::RwLock;
use tracing::debug;

use crate::auth::AuthService;
use crate::config::ProxyConfig;
#[cfg(unix)]
use crate::middleware;
use crate::rate_limit::RateLimiter;
use crate::transport::AgentTransportConfig;
#[cfg(unix)]
use crate::transport::DEFAULT_AGENT_TCP_PORT;

/// Host proxy service that forwards requests to world-agent.
#[derive(Clone)]
pub struct HostProxyService {
    client: Arc<AgentClient>,
    config: ProxyConfig,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    auth_service: Arc<AuthService>,
    stats: Arc<RwLock<ProxyStats>>,
}

impl HostProxyService {
    /// Create a new host proxy service.
    pub fn new(config: ProxyConfig) -> Result<Self> {
        let client = match &config.agent {
            AgentTransportConfig::Unix { path } => AgentClient::unix_socket(path),
            AgentTransportConfig::Tcp { host, port } => AgentClient::tcp(host, *port),
            #[cfg(target_os = "windows")]
            AgentTransportConfig::NamedPipe { path } => AgentClient::named_pipe(path),
        }
        .context("Failed to create agent client")?;
        let client = Arc::new(client);
        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(&config.rate_limits)));
        let auth_service = Arc::new(AuthService::new(&config.auth)?);
        let stats = Arc::new(RwLock::new(ProxyStats::default()));

        Ok(Self {
            client,
            config,
            rate_limiter,
            auth_service,
            stats,
        })
    }

    /// Check rate limits for an agent.
    async fn check_rate_limit(&self, agent_id: &str) -> Result<(), ApiError> {
        let mut limiter = self.rate_limiter.write().await;
        limiter
            .check_and_update(agent_id)
            .map_err(|e| ApiError::RateLimited(format!("Rate limit exceeded: {}", e)))
    }

    /// Update statistics.
    async fn update_stats(&self, agent_id: &str, duration: Duration, success: bool) {
        let mut stats = self.stats.write().await;
        stats.record_request(agent_id, duration, success);
    }
}

#[async_trait]
impl AgentService for HostProxyService {
    async fn execute(&self, req: ExecuteRequest) -> Result<ExecuteResponse, ApiError> {
        let start = Instant::now();
        let agent_id = req.agent_id.clone();

        if self.config.auth.enabled {
            self.auth_service.verify_agent(&agent_id).await?;
        }

        self.check_rate_limit(&agent_id).await?;

        debug!("Forwarding execute request for agent: {}", agent_id);

        let result = tokio::time::timeout(
            Duration::from_secs(self.config.request_timeout),
            self.client.execute(req),
        )
        .await
        .map_err(|_| ApiError::Internal("Request timeout".to_string()))?
        .map_err(|e| ApiError::Internal(format!("Failed to forward request: {}", e)))?;

        let duration = start.elapsed();
        self.update_stats(&agent_id, duration, true).await;

        Ok(result)
    }

    async fn execute_stream(&self, req: ExecuteRequest) -> Result<Response, ApiError> {
        let start = Instant::now();
        let agent_id = req.agent_id.clone();

        if self.config.auth.enabled {
            self.auth_service.verify_agent(&agent_id).await?;
        }

        self.check_rate_limit(&agent_id).await?;

        debug!(
            "Forwarding streaming execute request for agent: {}",
            agent_id
        );

        let response = tokio::time::timeout(
            Duration::from_secs(self.config.request_timeout),
            self.client.execute_stream(req),
        )
        .await
        .map_err(|_| ApiError::Internal("Request timeout".to_string()))?
        .map_err(|e| ApiError::Internal(format!("Failed to forward request: {}", e)))?;

        let (parts, body) = response.into_parts();
        let stream = body.into_data_stream();
        let body = Body::from_stream(stream);
        let response = Response::from_parts(parts, body);

        let duration = start.elapsed();
        self.update_stats(&agent_id, duration, true).await;

        Ok(response)
    }

    async fn get_trace(&self, span_id: String) -> Result<serde_json::Value, ApiError> {
        self.client
            .get_trace(&span_id)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to get trace: {}", e)))
    }
}

/// Proxy statistics.
#[derive(Debug, Default)]
struct ProxyStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_duration_ms: u64,
    per_agent: std::collections::HashMap<String, AgentStats>,
}

impl ProxyStats {
    fn record_request(&mut self, agent_id: &str, duration: Duration, success: bool) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        self.total_duration_ms += duration.as_millis() as u64;

        let agent_stats = self.per_agent.entry(agent_id.to_string()).or_default();
        agent_stats.total_requests += 1;
        if success {
            agent_stats.successful_requests += 1;
        } else {
            agent_stats.failed_requests += 1;
        }
        agent_stats.total_duration_ms += duration.as_millis() as u64;
    }
}

#[derive(Debug, Default)]
struct AgentStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_duration_ms: u64,
}

/// Create the directory for the Unix socket if it doesn't exist.
pub async fn ensure_socket_dir(socket_path: &Path) -> Result<()> {
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create socket directory")?;
    }
    Ok(())
}

/// Clean up an existing Unix socket file.
pub async fn cleanup_socket(socket_path: &Path) -> Result<()> {
    if socket_path.exists() {
        tokio::fs::remove_file(socket_path)
            .await
            .context("Failed to remove existing socket")?;
    }
    Ok(())
}

#[cfg(unix)]
pub async fn run_host_proxy() -> Result<()> {
    use agent_api_core::build_router;
    use axum::routing::get;
    use tower::ServiceBuilder;
    use tower::ServiceExt;
    use tower_http::limit::RequestBodyLimitLayer;
    use tracing::info;
    use tracing_subscriber::prelude::*;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting host-proxy server");

    let config = load_config_from_env()?;
    info!("Configuration loaded: {:?}", config);

    ensure_socket_dir(&config.host_socket).await?;
    cleanup_socket(&config.host_socket).await?;

    let service =
        Arc::new(HostProxyService::new(config.clone()).context("Failed to create proxy service")?);

    let api_router = build_router(service);

    let app = api_router
        .route("/health", get(middleware::health_check))
        .layer(axum::middleware::from_fn(middleware::logging_middleware))
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

#[cfg_attr(not(unix), allow(dead_code))]
#[cfg(not(unix))]
pub async fn run_host_proxy() -> Result<()> {
    Err(anyhow::anyhow!(
        "host-proxy binary is not supported on this platform"
    ))
}

#[cfg(unix)]
pub fn load_config_from_env() -> Result<ProxyConfig> {
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
                        .unwrap_or(DEFAULT_AGENT_TCP_PORT);
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
