//! Host-side API gateway for Agent API.
//!
//! This crate provides the host-side HTTP/WebSocket server that:
//! - Binds to ~/.substrate/sock/agent.sock
//! - Forwards requests to world-agent via agent-api-client
//! - Adds middleware for auth, rate limiting, and budgets
//! - Uses the same schema from agent-api-types

pub mod auth;
pub mod middleware;
pub mod rate_limit;

mod config;
mod runtime;
mod transport;

pub use config::{AuthConfig, ProxyConfig, RateLimitConfig};
pub use runtime::{cleanup_socket, ensure_socket_dir, HostProxyService};
#[cfg(unix)]
pub use runtime::{load_config_from_env, run_host_proxy};
pub use transport::AgentTransportConfig;
