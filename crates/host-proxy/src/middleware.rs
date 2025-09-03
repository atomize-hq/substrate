//! Middleware for the host proxy.

use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tracing::{debug, info, warn};

use crate::HostProxyService;

/// Logging middleware.
pub async fn logging_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();

    debug!("Request: {} {}", method, path);

    let response = next.run(req).await;
    let duration = start.elapsed();
    let status = response.status();

    if status.is_client_error() || status.is_server_error() {
        warn!(
            "Request failed: {} {} - {} ({:?})",
            method, path, status, duration
        );
    } else {
        info!(
            "Request completed: {} {} - {} ({:?})",
            method, path, status, duration
        );
    }

    response
}

/// Budget tracking middleware.
pub async fn budget_middleware(
    State(_service): State<Arc<HostProxyService>>,
    req: Request,
    next: Next,
) -> Response {
    // Extract agent ID from request (could be from header, query param, or body)
    let agent_id = extract_agent_id(&req)
        .await
        .unwrap_or_else(|| "anonymous".to_string());

    // Check budget (placeholder - would integrate with actual budget system)
    if !check_budget(&agent_id).await {
        return (
            StatusCode::PAYMENT_REQUIRED,
            axum::Json(json!({
                "error": "budget_exceeded",
                "message": "Agent budget has been exceeded"
            })),
        )
            .into_response();
    }

    let response = next.run(req).await;

    // Update budget usage (placeholder)
    update_budget_usage(&agent_id, 1).await;

    response
}

/// Extract agent ID from request.
async fn extract_agent_id(req: &Request) -> Option<String> {
    // Try to get from header
    if let Some(agent_id) = req.headers().get("X-Agent-ID") {
        if let Ok(id) = agent_id.to_str() {
            return Some(id.to_string());
        }
    }

    // Could also extract from JWT token, query params, or request body
    None
}

/// Check if agent has budget remaining (placeholder).
async fn check_budget(agent_id: &str) -> bool {
    // Placeholder - would integrate with actual budget tracking system
    debug!("Checking budget for agent: {}", agent_id);
    true
}

/// Update budget usage (placeholder).
async fn update_budget_usage(agent_id: &str, cost: u32) {
    // Placeholder - would integrate with actual budget tracking system
    debug!("Updating budget for agent {}: cost {}", agent_id, cost);
}

/// Health check endpoint.
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        axum::Json(json!({
            "status": "healthy",
            "service": "host-proxy"
        })),
    )
}

/// Metrics endpoint.
pub async fn metrics(State(_service): State<Arc<HostProxyService>>) -> impl IntoResponse {
    // Return basic metrics (placeholder - would integrate with actual metrics system)
    (
        StatusCode::OK,
        axum::Json(json!({
            "service": "host-proxy",
            "metrics": {
                "uptime_seconds": 0,
                "total_requests": 0,
                "active_connections": 0
            }
        })),
    )
}
