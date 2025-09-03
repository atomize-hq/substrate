//! Host-side API gateway for Agent API.
//!
//! This crate provides the host-side HTTP/WebSocket server that:
//! - Binds to ~/.substrate/sock/agent.sock
//! - Forwards requests to world-agent via agent-api-client
//! - Adds middleware for auth, rate limiting, and budgets
//! - Uses the same schema from agent-api-types

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use agent_api_client::AgentClient;
use agent_api_core::AgentService;
use agent_api_types::{ApiError, ExecuteRequest, ExecuteResponse};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::debug;

pub mod auth;
pub mod middleware;
pub mod rate_limit;

use auth::AuthService;
use rate_limit::RateLimiter;

/// Configuration for the host proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Path to the host-side Unix socket.
    pub host_socket: PathBuf,
    /// Path to the world-agent Unix socket.
    pub agent_socket: PathBuf,
    /// Maximum request body size in bytes.
    pub max_body_size: usize,
    /// Rate limiting configuration.
    pub rate_limits: RateLimitConfig,
    /// Authentication configuration.
    pub auth: AuthConfig,
    /// Request timeout in seconds.
    pub request_timeout: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Self {
            host_socket: PathBuf::from(format!("{}/.substrate/sock/agent.sock", home)),
            agent_socket: PathBuf::from("/run/substrate.sock"),
            max_body_size: 10 * 1024 * 1024, // 10MB
            rate_limits: RateLimitConfig::default(),
            auth: AuthConfig::default(),
            request_timeout: 30,
        }
    }
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute per agent.
    pub requests_per_minute: u32,
    /// Maximum concurrent executions per agent.
    pub max_concurrent: u32,
    /// Enable burst allowance.
    pub burst_enabled: bool,
    /// Burst multiplier.
    pub burst_multiplier: f32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            max_concurrent: 5,
            burst_enabled: true,
            burst_multiplier: 1.5,
        }
    }
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication.
    pub enabled: bool,
    /// Path to token file.
    pub token_file: Option<PathBuf>,
    /// Allow unauthenticated requests (for local development).
    pub allow_anonymous: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token_file: None,
            allow_anonymous: true,
        }
    }
}

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
        let client = Arc::new(AgentClient::unix_socket(&config.agent_socket));
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

        // Check authentication
        if self.config.auth.enabled {
            self.auth_service.verify_agent(&agent_id).await?;
        }

        // Check rate limits
        self.check_rate_limit(&agent_id).await?;

        debug!("Forwarding execute request for agent: {}", agent_id);

        // Forward to world-agent
        let result = tokio::time::timeout(
            Duration::from_secs(self.config.request_timeout),
            self.client.execute(req),
        )
        .await
        .map_err(|_| ApiError::Internal("Request timeout".to_string()))?
        .map_err(|e| ApiError::Internal(format!("Failed to forward request: {}", e)))?;

        // Update statistics
        let duration = start.elapsed();
        self.update_stats(&agent_id, duration, true).await;

        Ok(result)
    }

    async fn get_trace(&self, span_id: String) -> Result<serde_json::Value, ApiError> {
        // Forward to world-agent
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
