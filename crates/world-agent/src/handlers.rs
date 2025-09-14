//! HTTP handlers for the world agent API.

use crate::service::WorldAgentService;
use agent_api_types::{ApiError, ExecuteRequest, ExecuteResponse};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde_json::{json, Value};

/// Wrapper type to implement IntoResponse for ApiError
#[derive(Debug)]
pub struct ApiErrorResponse(ApiError);

impl From<ApiError> for ApiErrorResponse {
    fn from(err: ApiError) -> Self {
        ApiErrorResponse(err)
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.0 {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = json!({
            "error": self.0.to_string(),
        });

        (status, ResponseJson(body)).into_response()
    }
}

/// Get agent capabilities.
pub async fn capabilities() -> Result<ResponseJson<Value>, ApiErrorResponse> {
    Ok(ResponseJson(json!({
        "version": "v1",
        "features": [
            "execute",
            "pty_streaming",
            "trace_retrieval",
            "scope_requests"
        ],
        "backend": "world-agent",
        "platform": std::env::consts::OS
    })))
}

/// Execute a command in the world.
pub async fn execute(
    State(service): State<WorldAgentService>,
    Json(req): Json<ExecuteRequest>,
) -> Result<ResponseJson<ExecuteResponse>, ApiErrorResponse> {
    let response = service
        .execute(req)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(response))
}

/// Handle WebSocket upgrade for PTY streaming.
pub async fn stream(
    State(service): State<WorldAgentService>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| async move {
        crate::pty::handle_ws_pty(service, socket).await;
    })
}

/// Get trace information for a span.
pub async fn get_trace(
    Path(span_id): Path<String>,
    State(service): State<WorldAgentService>,
) -> Result<ResponseJson<Value>, ApiErrorResponse> {
    let trace = service
        .get_trace(&span_id)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(trace))
}

/// Request additional scopes.
pub async fn request_scopes(
    State(service): State<WorldAgentService>,
    Json(scopes): Json<Vec<String>>,
) -> Result<ResponseJson<Value>, ApiErrorResponse> {
    let response = service
        .request_scopes(scopes)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capabilities_handler() {
        let result = capabilities().await.unwrap();
        let value = result.0;

        assert_eq!(value["version"], "v1");
        assert!(value["features"].is_array());
        assert_eq!(value["backend"], "world-agent");
    }
}
