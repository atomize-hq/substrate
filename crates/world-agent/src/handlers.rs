//! HTTP handlers for the world agent API.

use crate::service::WorldAgentService;
use agent_api_types::{
    ApiError, ExecuteRequest, ExecuteResponse, PendingDiffClearRequestV1,
    PendingDiffClearResponseV1, PendingDiffReconcileRequestV1, PendingDiffReconcileResponseV1,
    PendingDiffRecordV1, PendingDiffRequestV1, WorldDoctorLandlockV1, WorldDoctorReportV1,
    WorldDoctorWorldFsStrategyKindV1, WorldDoctorWorldFsStrategyProbeResultV1,
    WorldDoctorWorldFsStrategyProbeV1, WorldDoctorWorldFsStrategyV1, WorldFsReadRequestV1,
    WorldFsReadResponseV1,
};
use axum::{
    body::Bytes,
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson, Response},
};
use chrono::SecondsFormat;
use serde_json::{json, Value};
use substrate_common::{WorldFsMode, WorldFsStrategyProbeResult};

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

        let message = match &self.0 {
            ApiError::BadRequest(message) => message.as_str(),
            ApiError::NotFound(message) => message.as_str(),
            ApiError::RateLimited(message) => message.as_str(),
            ApiError::Internal(message) => message.as_str(),
        };

        let body = json!({
            "error": message,
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
            "policy_snapshot_v3",
            "pty_streaming",
            "pending_diff_v1",
            "pending_diff_clear_v1",
            "pending_diff_reconcile_v1",
            "world_fs_read_v1",
            "trace_retrieval",
            "scope_requests"
        ],
        "backend": "world-agent",
        "platform": std::env::consts::OS
    })))
}

/// Get agent-reported world enforcement readiness.
pub async fn doctor_world(
    State(service): State<WorldAgentService>,
) -> Result<ResponseJson<WorldDoctorReportV1>, ApiErrorResponse> {
    let collected_at_utc = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    #[cfg(target_os = "linux")]
    let (landlock, probe) = {
        let support = world::landlock::detect_support();

        let probe_root = std::env::temp_dir();
        let probe_raw = match world::overlayfs::select_strategy(
            "doctor_world",
            &probe_root,
            WorldFsMode::Writable,
        ) {
            Ok(selection) => selection.probe,
            Err(err) => {
                let mut probe = world::overlayfs::run_enumeration_probe(
                    "doctor_world",
                    substrate_common::WorldFsStrategy::Overlay,
                    &probe_root,
                );
                if probe.failure_reason.is_none() {
                    probe.failure_reason = Some(err.to_string());
                }
                probe
            }
        };

        let probe_result = match probe_raw.result {
            WorldFsStrategyProbeResult::Pass => WorldDoctorWorldFsStrategyProbeResultV1::Pass,
            WorldFsStrategyProbeResult::Fail => WorldDoctorWorldFsStrategyProbeResultV1::Fail,
        };

        (
            WorldDoctorLandlockV1 {
                supported: support.supported,
                abi: support.abi,
                reason: support.reason,
            },
            WorldDoctorWorldFsStrategyProbeV1 {
                id: probe_raw.id,
                probe_file: probe_raw.probe_file,
                result: probe_result,
                failure_reason: probe_raw.failure_reason,
            },
        )
    };

    #[cfg(not(target_os = "linux"))]
    let (landlock, probe) = {
        (
            WorldDoctorLandlockV1 {
                supported: false,
                abi: None,
                reason: Some("landlock only supported on Linux".to_string()),
            },
            WorldDoctorWorldFsStrategyProbeV1 {
                id: "enumeration_v1".to_string(),
                probe_file: ".substrate_enum_probe".to_string(),
                result: WorldDoctorWorldFsStrategyProbeResultV1::Fail,
                failure_reason: Some("world fs probe unsupported on this platform".to_string()),
            },
        )
    };

    let ok =
        landlock.supported && matches!(probe.result, WorldDoctorWorldFsStrategyProbeResultV1::Pass);
    let report = WorldDoctorReportV1 {
        schema_version: 2,
        ok,
        collected_at_utc,
        policy_snapshot_v1_supported: service.policy_snapshot_v1_supported(),
        policy_resolution_mode: service.last_policy_resolution_mode(),
        landlock,
        world_fs_strategy: WorldDoctorWorldFsStrategyV1 {
            primary: WorldDoctorWorldFsStrategyKindV1::Overlay,
            fallback: WorldDoctorWorldFsStrategyKindV1::Fuse,
            probe,
        },
    };

    Ok(ResponseJson(report))
}

/// Execute a command in the world.
pub async fn execute(
    State(service): State<WorldAgentService>,
    body: Bytes,
) -> Result<ResponseJson<ExecuteResponse>, ApiErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let req: ExecuteRequest = serde_json::from_value(payload)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service.execute(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            ApiErrorResponse(ApiError::BadRequest(bad.message().to_string()))
        } else {
            ApiErrorResponse(ApiError::Internal(e.to_string()))
        }
    })?;

    Ok(ResponseJson(response))
}

/// Retrieve the current session's pending diff record.
pub async fn pending_diff(
    State(service): State<WorldAgentService>,
    body: Bytes,
) -> Result<ResponseJson<PendingDiffRecordV1>, ApiErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let req: PendingDiffRequestV1 = serde_json::from_value(payload)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service.pending_diff(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            ApiErrorResponse(ApiError::BadRequest(bad.message().to_string()))
        } else {
            ApiErrorResponse(ApiError::Internal(e.to_string()))
        }
    })?;

    Ok(ResponseJson(response))
}

/// Conditionally clear the current session's pending diff snapshot.
pub async fn pending_diff_clear(
    State(service): State<WorldAgentService>,
    body: Bytes,
) -> Result<ResponseJson<PendingDiffClearResponseV1>, ApiErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let req: PendingDiffClearRequestV1 = serde_json::from_value(payload)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service.pending_diff_clear(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            ApiErrorResponse(ApiError::BadRequest(bad.message().to_string()))
        } else {
            ApiErrorResponse(ApiError::Internal(e.to_string()))
        }
    })?;

    Ok(ResponseJson(response))
}

/// Conditionally reconcile pending diff paths by discarding upper/work entries.
pub async fn pending_diff_reconcile(
    State(service): State<WorldAgentService>,
    body: Bytes,
) -> Result<ResponseJson<PendingDiffReconcileResponseV1>, ApiErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let req: PendingDiffReconcileRequestV1 = serde_json::from_value(payload)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service.pending_diff_reconcile(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            ApiErrorResponse(ApiError::BadRequest(bad.message().to_string()))
        } else {
            ApiErrorResponse(ApiError::Internal(e.to_string()))
        }
    })?;

    Ok(ResponseJson(response))
}

/// Read metadata and optionally contents from the current session's overlay filesystem.
pub async fn world_fs_read(
    State(service): State<WorldAgentService>,
    body: Bytes,
) -> Result<ResponseJson<WorldFsReadResponseV1>, ApiErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let req: WorldFsReadRequestV1 = serde_json::from_value(payload)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service.world_fs_read(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            ApiErrorResponse(ApiError::BadRequest(bad.message().to_string()))
        } else {
            ApiErrorResponse(ApiError::Internal(e.to_string()))
        }
    })?;

    Ok(ResponseJson(response))
}

/// Execute a command and stream incremental output.
pub async fn execute_stream(
    State(service): State<WorldAgentService>,
    body: Bytes,
) -> Result<Response, ApiErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let req: ExecuteRequest = serde_json::from_value(payload)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    service.execute_stream(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            ApiErrorResponse(ApiError::BadRequest(bad.message().to_string()))
        } else {
            ApiErrorResponse(ApiError::Internal(e.to_string()))
        }
    })
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

/// Garbage collect orphaned network namespaces.
pub async fn gc(
    State(_service): State<WorldAgentService>,
) -> Result<ResponseJson<Value>, ApiErrorResponse> {
    // Read TTL from environment
    let ttl = std::env::var("SUBSTRATE_NETNS_GC_TTL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|&ttl| ttl > 0)
        .map(std::time::Duration::from_secs);

    let report = crate::gc::sweep(ttl)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(
        serde_json::to_value(report).unwrap_or_else(|_| json!({})),
    ))
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
