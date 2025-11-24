use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::transport::AgentTransportConfig;

/// Configuration for the host proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Path to the host-side Unix socket.
    pub host_socket: PathBuf,
    /// Transport used to reach world-agent.
    #[serde(default)]
    pub agent: AgentTransportConfig,
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
            agent: AgentTransportConfig::default(),
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
