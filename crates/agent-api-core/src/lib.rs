//! Agent API core: service trait and router builder shared by host-proxy and world-agent.

use std::sync::Arc;

use agent_api_types::{ApiError, ErrorResponse, ExecuteRequest, ExecuteResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

// Service trait implemented by both host-proxy (delegating) and world-agent (executing)
#[async_trait::async_trait]
pub trait AgentService: Send + Sync + 'static {
    async fn execute(&self, req: ExecuteRequest) -> Result<ExecuteResponse, ApiError>;
    async fn get_trace(&self, span_id: String) -> Result<serde_json::Value, ApiError>;
}

// Wrapper type for ApiError to implement IntoResponse
pub struct ApiErrorResponse(pub ApiError);

impl From<ApiError> for ApiErrorResponse {
    fn from(error: ApiError) -> Self {
        ApiErrorResponse(error)
    }
}

// Axum error mapping for ApiError → HTTP JSON
impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let (status, error) = match &self.0 {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            ApiError::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        let msg = self.0.to_string();
        let body = Json(ErrorResponse {
            error: error.to_string(),
            message: msg,
        });
        (status, body).into_response()
    }
}

#[derive(Clone)]
struct AppState<S: AgentService> {
    svc: Arc<S>,
}

pub fn build_router<S>(svc: Arc<S>) -> Router
where
    S: AgentService + Clone,
{
    let state = AppState { svc };
    Router::new()
        .route("/v1/execute", post(handle_execute::<S>))
        .route("/v1/trace/:span_id", get(handle_get_trace::<S>))
        .with_state(state)
}

async fn handle_execute<S: AgentService>(
    State(state): State<AppState<S>>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, ApiErrorResponse> {
    let resp = state.svc.execute(req).await.map_err(ApiErrorResponse)?;
    Ok(Json(resp))
}

async fn handle_get_trace<S: AgentService>(
    State(state): State<AppState<S>>,
    Path(span_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
    let trace = state.svc.get_trace(span_id).await.map_err(ApiErrorResponse)?;
    Ok(Json(trace))
}
