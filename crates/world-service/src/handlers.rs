//! HTTP handlers for the world agent API.

#[cfg(target_os = "linux")]
use crate::member_turn_join::MemberTurnJoinError;
use crate::service::WorldService;
use axum::{
    body::Bytes,
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson, Response},
};
use chrono::SecondsFormat;
use serde_json::{json, Value};
#[cfg(target_os = "linux")]
use substrate_common::{WorldFsMode, WorldFsStrategyProbeResult};
use transport_api_types::{
    ApiError, ExecuteCancelRequestV1, ExecuteCancelResponseV1, ExecuteRequest, ExecuteResponse,
    ExecuteStreamReplayRequestV1, GatewayLifecycleRequestV1, GatewayLifecycleResponseV1,
    MemberTurnSubmitRequestV1, PendingDiffClearRequestV1, PendingDiffClearResponseV1,
    PendingDiffReconcileRequestV1, PendingDiffReconcileResponseV1, PendingDiffRecordV1,
    PendingDiffRequestV1, WorldDoctorLandlockV1, WorldDoctorNetfilterStatusV1, WorldDoctorReportV1,
    WorldDoctorWorldFsStrategyKindV1, WorldDoctorWorldFsStrategyProbeResultV1,
    WorldDoctorWorldFsStrategyProbeV1, WorldDoctorWorldFsStrategyV1, WorldFsReadRequestV1,
    WorldFsReadResponseV1,
};

#[cfg(target_os = "linux")]
fn doctor_world_netfilter_enable_present() -> bool {
    world::netfilter::world_netfilter_enable_present()
}

#[cfg(not(target_os = "linux"))]
fn doctor_world_netfilter_enable_present() -> bool {
    false
}

/// Wrapper type to implement IntoResponse for ApiError
#[derive(Debug)]
pub struct ApiErrorResponse(ApiError);

#[derive(Debug)]
pub struct MemberTurnSubmitErrorResponse {
    status: StatusCode,
    error_code: String,
}

impl MemberTurnSubmitErrorResponse {
    fn new(status: StatusCode, error_code: impl Into<String>) -> Self {
        Self {
            status,
            error_code: error_code.into(),
        }
    }
}

impl IntoResponse for MemberTurnSubmitErrorResponse {
    fn into_response(self) -> Response {
        (
            self.status,
            ResponseJson(json!({ "error": self.error_code })),
        )
            .into_response()
    }
}

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
        "backend": "world-service",
        "platform": std::env::consts::OS
    })))
}

/// Get agent-reported world enforcement readiness.
pub async fn doctor_world(
    State(service): State<WorldService>,
) -> Result<ResponseJson<WorldDoctorReportV1>, ApiErrorResponse> {
    #[cfg(target_os = "linux")]
    let _privileged_child_exclusion_lease = service
        .acquire_non_e3_helper_operation()
        .map_err(|error| ApiErrorResponse(ApiError::Internal(error.to_string())))?;
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
    let requested = service.last_netfilter_requested();
    let world_netfilter_enable_present = doctor_world_netfilter_enable_present();
    let last_failure_reason = service.last_netfilter_failure_reason();
    let report = WorldDoctorReportV1 {
        schema_version: 2,
        ok,
        collected_at_utc,
        selected_host_prefix: None,
        host_context_commitment: None,
        policy_snapshot_v1_supported: service.policy_snapshot_v1_supported(),
        policy_resolution_mode: service.last_policy_resolution_mode(),
        netfilter_status: Some(WorldDoctorNetfilterStatusV1 {
            requested,
            enabled: requested && world_netfilter_enable_present,
            world_netfilter_enable_present,
            last_failure_reason,
        }),
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
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<ResponseJson<ExecuteResponse>, ApiErrorResponse> {
    let req: ExecuteRequest = serde_json::from_slice(&body)
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
    State(service): State<WorldService>,
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
    State(service): State<WorldService>,
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
    State(service): State<WorldService>,
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
    State(service): State<WorldService>,
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

/// Return the typed gateway lifecycle/status surface.
pub async fn gateway_status(
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<ResponseJson<GatewayLifecycleResponseV1>, ApiErrorResponse> {
    let req: GatewayLifecycleRequestV1 = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service
        .gateway_status(req)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(response))
}

/// Return the typed gateway lifecycle sync surface.
pub async fn gateway_sync(
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<ResponseJson<GatewayLifecycleResponseV1>, ApiErrorResponse> {
    let req: GatewayLifecycleRequestV1 = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service
        .gateway_sync(req)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(response))
}

/// Return the typed gateway lifecycle restart surface.
pub async fn gateway_restart(
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<ResponseJson<GatewayLifecycleResponseV1>, ApiErrorResponse> {
    let req: GatewayLifecycleRequestV1 = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service
        .gateway_restart(req)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(response))
}

/// Execute a command and stream incremental output.
pub async fn execute_stream(
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<Response, ApiErrorResponse> {
    let req: ExecuteRequest = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    if req.ordinary_start_selection.is_some() {
        return Err(ApiErrorResponse(ApiError::BadRequest(
            "ordinary initial input requires authenticated ordinary route".into(),
        )));
    }
    service.execute_stream(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            ApiErrorResponse(ApiError::BadRequest(bad.message().to_string()))
        } else {
            ApiErrorResponse(ApiError::Internal(e.to_string()))
        }
    })
}

/// Resume one exact accepted execution stream from its durable host cursor.
pub async fn execute_stream_replay(
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<Response, ApiErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let req: ExecuteStreamReplayRequestV1 = serde_json::from_value(payload)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    req.validate()
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(e)))?;
    service.replay_execute_stream(req).await.map_err(|e| {
        ApiErrorResponse(ApiError::NotFound(format!(
            "exact runtime replay stream is unavailable: {e}"
        )))
    })
}

/// Submit a follow-up turn to a retained world member and stream the response.
pub async fn member_turn_stream(
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<Response, MemberTurnSubmitErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|_| MemberTurnSubmitErrorResponse::new(StatusCode::BAD_REQUEST, "invalid_json"))?;
    let req: MemberTurnSubmitRequestV1 = serde_json::from_value(payload)
        .map_err(|_| MemberTurnSubmitErrorResponse::new(StatusCode::BAD_REQUEST, "invalid_json"))?;
    service.submit_member_turn_stream(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            return MemberTurnSubmitErrorResponse::new(
                StatusCode::BAD_REQUEST,
                bad.message().to_string(),
            );
        }
        #[cfg(target_os = "linux")]
        if let Some(join) = e.downcast_ref::<MemberTurnJoinError>() {
            return match join {
                MemberTurnJoinError::RequestConflict => MemberTurnSubmitErrorResponse::new(
                    StatusCode::CONFLICT,
                    "member_turn_request_conflict_v1",
                ),
                MemberTurnJoinError::LaunchIndeterminate => MemberTurnSubmitErrorResponse::new(
                    StatusCode::CONFLICT,
                    "member_turn_launch_indeterminate_v1",
                ),
                MemberTurnJoinError::UnsupportedLegacyState => MemberTurnSubmitErrorResponse::new(
                    StatusCode::CONFLICT,
                    "unsupported_legacy_state",
                ),
                MemberTurnJoinError::CapacityExhausted => MemberTurnSubmitErrorResponse::new(
                    StatusCode::TOO_MANY_REQUESTS,
                    "member_turn_join_capacity_exhausted_v1",
                ),
                MemberTurnJoinError::FailedBeforeLaunch {
                    error_code,
                    http_status,
                } => MemberTurnSubmitErrorResponse::new(
                    StatusCode::from_u16(*http_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    error_code.clone(),
                ),
                MemberTurnJoinError::Unsafe(_) => MemberTurnSubmitErrorResponse::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "member_turn_join_unsafe_v1",
                ),
            };
        }
        MemberTurnSubmitErrorResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "member_turn_internal_error",
        )
    })
}

/// Submit an ordinary retained turn. Decoding grants no configured authority.
pub(crate) async fn ordinary_member_turn_stream(
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<Response, MemberTurnSubmitErrorResponse> {
    // Decode the received bytes directly: no Value roundtrip can erase duplicate
    // fields in the request or nested carrier before its strict typed codec.
    let req: transport_api_types::OrdinaryMemberTurnSubmitRequestV1 = serde_json::from_slice(&body)
        .map_err(|_| MemberTurnSubmitErrorResponse::new(StatusCode::BAD_REQUEST, "invalid_json"))?;
    service
        .submit_ordinary_member_turn_stream(req)
        .await
        .map_err(|e| {
            if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
                return MemberTurnSubmitErrorResponse::new(
                    StatusCode::BAD_REQUEST,
                    bad.message().to_string(),
                );
            }
            #[cfg(target_os = "linux")]
            if let Some(join) = e.downcast_ref::<MemberTurnJoinError>() {
                return match join {
                    MemberTurnJoinError::RequestConflict => MemberTurnSubmitErrorResponse::new(
                        StatusCode::CONFLICT,
                        "member_turn_request_conflict_v1",
                    ),
                    MemberTurnJoinError::LaunchIndeterminate => MemberTurnSubmitErrorResponse::new(
                        StatusCode::CONFLICT,
                        "member_turn_launch_indeterminate_v1",
                    ),
                    MemberTurnJoinError::UnsupportedLegacyState => {
                        MemberTurnSubmitErrorResponse::new(
                            StatusCode::CONFLICT,
                            "unsupported_legacy_state",
                        )
                    }
                    MemberTurnJoinError::CapacityExhausted => MemberTurnSubmitErrorResponse::new(
                        StatusCode::TOO_MANY_REQUESTS,
                        "member_turn_join_capacity_exhausted_v1",
                    ),
                    MemberTurnJoinError::FailedBeforeLaunch {
                        error_code,
                        http_status,
                    } => MemberTurnSubmitErrorResponse::new(
                        StatusCode::from_u16(*http_status)
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        error_code.clone(),
                    ),
                    MemberTurnJoinError::Unsafe(_) => MemberTurnSubmitErrorResponse::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "member_turn_join_unsafe_v1",
                    ),
                };
            }
            MemberTurnSubmitErrorResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "member_turn_internal_error",
            )
        })
}

/// Send a signal to an active streamed execution.
pub async fn execute_cancel(
    State(service): State<WorldService>,
    body: Bytes,
) -> Result<ResponseJson<ExecuteCancelResponseV1>, ApiErrorResponse> {
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let req: ExecuteCancelRequestV1 = serde_json::from_value(payload)
        .map_err(|e| ApiErrorResponse(ApiError::BadRequest(format!("Invalid JSON: {e}"))))?;
    let response = service.execute_cancel(req).await.map_err(|e| {
        if let Some(bad) = e.downcast_ref::<crate::service::BadRequestError>() {
            ApiErrorResponse(ApiError::BadRequest(bad.message().to_string()))
        } else {
            ApiErrorResponse(ApiError::Internal(e.to_string()))
        }
    })?;

    Ok(ResponseJson(response))
}

/// Handle WebSocket upgrade for PTY streaming.
pub async fn stream(
    State(service): State<WorldService>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| async move {
        crate::pty::handle_ws_pty(service, socket).await;
    })
}

/// Get trace information for a span.
pub async fn get_trace(
    Path(span_id): Path<String>,
    State(service): State<WorldService>,
) -> Result<ResponseJson<Value>, ApiErrorResponse> {
    let trace = service
        .get_trace(&span_id)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(trace))
}

/// Request additional scopes.
pub async fn request_scopes(
    State(service): State<WorldService>,
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
    State(service): State<WorldService>,
) -> Result<ResponseJson<Value>, ApiErrorResponse> {
    // Read TTL from environment
    let ttl = std::env::var("SUBSTRATE_NETNS_GC_TTL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|&ttl| ttl > 0)
        .map(std::time::Duration::from_secs);

    #[cfg(target_os = "linux")]
    let admission = service
        .acquire_non_e3_helper_operation()
        .map(crate::gc::E3GcSweepAdmissionV1::NonE3Lease)
        .map_err(|error| ApiErrorResponse(ApiError::Internal(error.to_string())))?;
    #[cfg(not(target_os = "linux"))]
    let admission = crate::gc::E3GcSweepAdmissionV1::StartupRecovery;
    let report = crate::gc::sweep(ttl, admission)
        .await
        .map_err(|e| ApiErrorResponse(ApiError::Internal(e.to_string())))?;

    Ok(ResponseJson(
        serde_json::to_value(report).unwrap_or_else(|_| json!({})),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use crate::service::WorldService;
    #[cfg(target_os = "linux")]
    use tokio::sync::Mutex;

    #[cfg(target_os = "linux")]
    static ENV_LOCK: Mutex<()> = Mutex::const_new(());

    #[cfg(target_os = "linux")]
    struct EnvGuard {
        entries: Vec<(String, Option<String>)>,
    }

    #[cfg(target_os = "linux")]
    impl EnvGuard {
        fn set(entries: &[(&str, Option<&str>)]) -> Self {
            let previous = entries
                .iter()
                .map(|(key, value)| {
                    let previous = std::env::var(key).ok();
                    match value {
                        Some(value) => unsafe { std::env::set_var(key, value) },
                        None => unsafe { std::env::remove_var(key) },
                    }
                    ((*key).to_string(), previous)
                })
                .collect();

            Self { entries: previous }
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.entries.iter().rev() {
                match value {
                    Some(value) => unsafe { std::env::set_var(key, value) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
        }
    }

    #[tokio::test]
    async fn test_capabilities_handler() {
        let result = capabilities().await.unwrap();
        let value = result.0;

        assert_eq!(value["version"], "v1");
        assert!(value["features"].is_array());
        assert_eq!(value["backend"], "world-service");
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn doctor_world_defaults_netfilter_status_when_no_request_seen() {
        let _lock = ENV_LOCK.lock().await;
        let _guard = EnvGuard::set(&[("WORLD_NETFILTER_ENABLE", None)]);
        let service = WorldService::new().expect("service");

        let result = doctor_world(State(service)).await.expect("doctor");
        let report = result.0;
        assert!(report.selected_host_prefix.is_none());
        assert!(report.host_context_commitment.is_none());
        let value = serde_json::to_value(&report).expect("serialize world-service doctor report");
        assert!(value.get("selected_host_prefix").is_none());
        assert!(value.get("host_context_commitment").is_none());
        let status = report.netfilter_status.expect("netfilter status");

        assert!(!status.requested);
        assert!(!status.enabled);
        assert!(!status.world_netfilter_enable_present);
        assert!(status.last_failure_reason.is_none());
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn doctor_world_reports_requested_and_guard_presence() {
        let _lock = ENV_LOCK.lock().await;
        let _guard = EnvGuard::set(&[("WORLD_NETFILTER_ENABLE", Some("true"))]);
        let service = WorldService::new().expect("service");
        service.set_last_netfilter_requested(true);

        let result = doctor_world(State(service)).await.expect("doctor");
        let report = result.0;
        let status = report.netfilter_status.expect("netfilter status");

        assert!(status.requested);
        assert!(status.enabled);
        assert!(status.world_netfilter_enable_present);
        assert!(status.last_failure_reason.is_none());
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn doctor_world_reports_requested_without_guard_as_disabled() {
        let _lock = ENV_LOCK.lock().await;
        let _guard = EnvGuard::set(&[("WORLD_NETFILTER_ENABLE", None)]);
        let service = WorldService::new().expect("service");
        service.set_last_netfilter_requested(true);

        let result = doctor_world(State(service)).await.expect("doctor");
        let report = result.0;
        let status = report.netfilter_status.expect("netfilter status");

        assert!(status.requested);
        assert!(!status.enabled);
        assert!(!status.world_netfilter_enable_present);
        assert!(status.last_failure_reason.is_none());
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn doctor_world_surfaces_last_netfilter_failure_reason() {
        let _lock = ENV_LOCK.lock().await;
        let _guard = EnvGuard::set(&[("WORLD_NETFILTER_ENABLE", None)]);
        let service = WorldService::new().expect("service");
        service.set_last_netfilter_requested(true);
        service.set_last_netfilter_failure_reason(Some(
            "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules"
                .to_string(),
        ));

        let result = doctor_world(State(service)).await.expect("doctor");
        let report = result.0;
        let status = report.netfilter_status.expect("netfilter status");

        assert_eq!(
            status.last_failure_reason.as_deref(),
            Some(
                "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules"
            )
        );
    }
}

/// E3 admission requires the acceptor's kernel-authenticated connection before body polling.
#[cfg(target_os = "linux")]
pub(crate) async fn e3_config_projection_prepare(
    State(service): State<WorldService>,
    request: axum::http::Request<hyper::Body>,
) -> Response {
    use crate::e3_local_transport::{
        E3AuthenticatedLinuxUdsListenerV1, E3AuthenticatedLinuxUdsPeerV1,
    };
    use crate::socket_activation::InheritedUnixListener;
    use std::sync::Arc;
    let authenticated = (|| {
        let peer = request
            .extensions()
            .get::<E3AuthenticatedLinuxUdsPeerV1>()?;
        let inherited = request.extensions().get::<Arc<InheritedUnixListener>>()?;
        let configured = service.accepted_home_authority.as_ref()?;
        let listener =
            E3AuthenticatedLinuxUdsListenerV1::from_inherited(inherited, 1, Arc::clone(configured))
                .ok()?;
        if peer.peer_uid != listener.expected_peer_uid
            || peer.kernel_boot_id != listener.kernel_boot_id
            || peer.listener_descriptor_device_id != listener.descriptor_device_id
            || peer.listener_descriptor_inode != listener.descriptor_inode
            || peer.peer_pid == 0
            || peer.peer_pid_start_time_ticks == 0
            || libc::gid_t::try_from(peer.peer_gid).is_err()
        {
            return None;
        }
        Some(())
    })();
    if authenticated.is_none() {
        return (
            StatusCode::FORBIDDEN,
            ResponseJson(json!({"error":"WrongBinding"})),
        )
            .into_response();
    }
    let result = service
        .e3_config_projection_prepare(request.into_body())
        .await;
    match result {
        Ok(response) => ResponseJson(response).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            ResponseJson(json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

/// E3 admission requires the acceptor's kernel-authenticated connection before body polling.
#[cfg(target_os = "linux")]
pub(crate) async fn e3_config_projection_cancel(
    State(service): State<WorldService>,
    request: axum::http::Request<hyper::Body>,
) -> Response {
    use crate::e3_local_transport::{
        E3AuthenticatedLinuxUdsListenerV1, E3AuthenticatedLinuxUdsPeerV1,
    };
    use crate::socket_activation::InheritedUnixListener;
    use std::sync::Arc;
    let authenticated = (|| {
        let peer = request
            .extensions()
            .get::<E3AuthenticatedLinuxUdsPeerV1>()?;
        let inherited = request.extensions().get::<Arc<InheritedUnixListener>>()?;
        let configured = service.accepted_home_authority.as_ref()?;
        let listener =
            E3AuthenticatedLinuxUdsListenerV1::from_inherited(inherited, 1, Arc::clone(configured))
                .ok()?;
        if peer.peer_uid != listener.expected_peer_uid
            || peer.kernel_boot_id != listener.kernel_boot_id
            || peer.listener_descriptor_device_id != listener.descriptor_device_id
            || peer.listener_descriptor_inode != listener.descriptor_inode
            || peer.peer_pid == 0
            || peer.peer_pid_start_time_ticks == 0
            || libc::gid_t::try_from(peer.peer_gid).is_err()
        {
            return None;
        }
        Some(())
    })();
    if authenticated.is_none() {
        return (
            StatusCode::FORBIDDEN,
            ResponseJson(json!({"error":"WrongBinding"})),
        )
            .into_response();
    }
    use hyper::body::HttpBody;
    let mut body = request.into_body();
    let mut bytes = Vec::with_capacity(65_536);
    while let Some(chunk) = body.data().await {
        let Ok(chunk) = chunk else {
            return (
                StatusCode::BAD_REQUEST,
                ResponseJson(json!({"error":"Malformed"})),
            )
                .into_response();
        };
        if chunk.len() > 65_536 - bytes.len() {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                ResponseJson(json!({"error":"Malformed"})),
            )
                .into_response();
        }
        bytes.extend_from_slice(&chunk);
    }
    let result = serde_json::from_slice(&bytes)
        .map_err(|_| config_projection::ConfigProjectionFailureV1::Malformed)
        .and_then(|request| service.e3_config_projection_cancel(request));
    match result {
        Ok(response) => ResponseJson(response).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            ResponseJson(json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

#[cfg(all(test, target_os = "linux"))]
mod e3_ingress_tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    #[tokio::test]
    async fn e3_handlers_reject_absent_peer_without_polling_secret_body() {
        let root = tempfile::tempdir().unwrap();
        let service = WorldService::new_with_member_turn_state_root_for_test(root.path()).unwrap();
        for prepare in [true, false] {
            let polled = Arc::new(AtomicBool::new(false));
            let observed = Arc::clone(&polled);
            let stream = futures_util::stream::once(async move {
                observed.store(true, Ordering::SeqCst);
                Ok::<_, std::io::Error>(Bytes::from_static(b"synthetic-secret-invalid-json"))
            });
            let request = axum::http::Request::new(hyper::Body::wrap_stream(stream));
            let response = if prepare {
                e3_config_projection_prepare(State(service.clone()), request).await
            } else {
                e3_config_projection_cancel(State(service.clone()), request).await
            };
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert!(!polled.load(Ordering::SeqCst));
            assert_eq!(
                hyper::body::to_bytes(response.into_body()).await.unwrap(),
                r#"{"error":"WrongBinding"}"#
            );
        }
    }
}

/// Authenticate the inherited connection before polling the secret-bearing body.
#[cfg(target_os = "linux")]
pub(crate) async fn gateway_host_turn(
    State(service): State<WorldService>,
    request: axum::http::Request<hyper::Body>,
) -> Response {
    use crate::e3_local_transport::{
        E3AuthenticatedLinuxUdsListenerV1, E3AuthenticatedLinuxUdsPeerV1,
    };
    use crate::socket_activation::InheritedUnixListener;
    use hyper::body::HttpBody;
    use std::sync::Arc;
    let principal = (|| {
        let peer = request
            .extensions()
            .get::<E3AuthenticatedLinuxUdsPeerV1>()?;
        let inherited = request.extensions().get::<Arc<InheritedUnixListener>>()?;
        let configured = service.accepted_home_authority.as_ref()?;
        let listener =
            E3AuthenticatedLinuxUdsListenerV1::from_inherited(inherited, 1, Arc::clone(configured))
                .ok()?;
        if peer.peer_uid != listener.expected_peer_uid
            || peer.kernel_boot_id != listener.kernel_boot_id
            || peer.listener_descriptor_device_id != listener.descriptor_device_id
            || peer.listener_descriptor_inode != listener.descriptor_inode
            || peer.peer_pid == 0
            || peer.peer_pid_start_time_ticks == 0
            || libc::gid_t::try_from(peer.peer_gid).is_err()
        {
            return None;
        }
        u32::try_from(peer.peer_uid).ok().filter(|uid| *uid != 0)
    })();
    let Some(principal) = principal else {
        return (
            StatusCode::FORBIDDEN,
            ResponseJson(json!({"error":"host_gateway_turn_wrong_binding"})),
        )
            .into_response();
    };
    let mut body = request.into_body();
    let mut bytes = Vec::new();
    while let Some(chunk) = body.data().await {
        let Ok(chunk) = chunk else {
            return (
                StatusCode::BAD_REQUEST,
                ResponseJson(json!({"error":"host_gateway_turn_invalid_request"})),
            )
                .into_response();
        };
        if chunk.len() > (64 * 1024usize).saturating_sub(bytes.len()) {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                ResponseJson(json!({"error":"host_gateway_turn_invalid_request"})),
            )
                .into_response();
        }
        bytes.extend_from_slice(&chunk);
    }
    // Decode directly from raw bytes: no Value round trip can erase duplicates.
    let request: transport_api_types::HostGatewayInstanceRequestV3 =
        match serde_json::from_slice(&bytes) {
            Ok(request) => request,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    ResponseJson(json!({"error":"host_gateway_turn_invalid_request"})),
                )
                    .into_response()
            }
        };
    match service.host_gateway_turn(principal, request).await {
        Ok(response) => ResponseJson(response).into_response(),
        // Never expose a cleanup credential, account, HSA detail or native error.
        Err(_) => (
            StatusCode::CONFLICT,
            ResponseJson(json!({"error":"host_gateway_turn_refused_or_unknown"})),
        )
            .into_response(),
    }
}

#[cfg(target_os = "linux")]
pub(crate) async fn workspace_current_v1(
    State(service): State<WorldService>,
    peer: Option<axum::Extension<substrate_shell::WorkspaceNativePeerV1>>,
    request: axum::http::Request<hyper::Body>,
) -> ResponseJson<transport_api_types::WorkspaceCurrentResponseV1> {
    use hyper::body::HttpBody;
    use transport_api_types::{WorkspaceCurrentRefusalV1 as F, WorkspaceCurrentResponseV1 as R};
    let deny = || ResponseJson(R::Refused { reason: F::Denied });
    let Some(axum::Extension(peer)) = peer else {
        return deny();
    };
    let mut body = request.into_body();
    let mut bytes = Vec::new();
    while let Some(chunk) = body.data().await {
        let Ok(chunk) = chunk else {
            return deny();
        };
        if bytes.len() + chunk.len() > 32768 {
            return deny();
        }
        bytes.extend_from_slice(&chunk);
    }
    let Ok(request) =
        serde_json::from_slice::<transport_api_types::WorkspaceCurrentRequestV1>(&bytes)
    else {
        return deny();
    };
    if request.validate().is_err() {
        return deny();
    }
    let publish = matches!(
        request.operation,
        transport_api_types::WorkspaceCurrentOperationV1::PublishAssociation
    );
    match tokio::task::spawn_blocking(move || service.workspace_current_v1(&peer, &request)).await {
        Ok(value) => ResponseJson(value),
        Err(_) if publish => ResponseJson(R::AssociationPublicationFailed {
            reason: F::Changed,
            effects: transport_api_types::WorkspaceAssociationEffectsV1::unknown(),
        }),
        Err(_) => deny(),
    }
}

#[cfg(target_os = "linux")]
async fn workspace_bounded_body_v1(mut body: hyper::Body) -> Option<Vec<u8>> {
    use hyper::body::HttpBody;
    let mut bytes = Vec::new();
    while let Some(chunk) = body.data().await {
        let chunk = chunk.ok()?;
        if bytes.len() + chunk.len() > 32768 {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    Some(bytes)
}

#[cfg(target_os = "linux")]
pub(crate) async fn workspace_preflight_v1(
    State(service): State<WorldService>,
    peer: Option<axum::Extension<substrate_shell::WorkspaceNativePeerV1>>,
    request: axum::http::Request<hyper::Body>,
) -> ResponseJson<transport_api_types::WorkspacePreflightResponseV1> {
    use transport_api_types::{WorkspaceCurrentRefusalV1 as F, WorkspacePreflightResponseV1 as R};
    let deny = || ResponseJson(R::Refused { reason: F::Denied });
    let Some(axum::Extension(peer)) = peer else {
        return deny();
    };
    let Some(bytes) = workspace_bounded_body_v1(request.into_body()).await else {
        return deny();
    };
    let Ok(request) =
        serde_json::from_slice::<transport_api_types::WorkspaceCurrentRequestV1>(&bytes)
    else {
        return deny();
    };
    if request.validate().is_err()
        || !matches!(
            request.operation,
            transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead { .. }
        )
    {
        return deny();
    }
    match tokio::task::spawn_blocking(move || service.workspace_preflight_v1(&peer, &request)).await
    {
        Ok(value) => ResponseJson(value),
        Err(_) => deny(),
    }
}

#[cfg(target_os = "linux")]
pub(crate) async fn workspace_association_update_v1(
    State(service): State<WorldService>,
    peer: Option<axum::Extension<substrate_shell::WorkspaceNativePeerV1>>,
    request: axum::http::Request<hyper::Body>,
) -> ResponseJson<transport_api_types::WorkspaceAssociationUpdateResponseV1> {
    use transport_api_types::{
        WorkspaceAssociationUpdateResponseV1 as R, WorkspaceCurrentRefusalV1 as F,
    };
    let deny = || ResponseJson(R::Refused { reason: F::Denied });
    let Some(axum::Extension(peer)) = peer else {
        return deny();
    };
    let Some(bytes) = workspace_bounded_body_v1(request.into_body()).await else {
        return deny();
    };
    let Ok(request) =
        serde_json::from_slice::<transport_api_types::WorkspaceAssociationUpdateV1>(&bytes)
    else {
        return deny();
    };
    if request.current.validate().is_err()
        || !matches!(
            request.current.operation,
            transport_api_types::WorkspaceCurrentOperationV1::PublishAssociation
        )
    {
        return deny();
    }
    match tokio::task::spawn_blocking(move || {
        service.workspace_association_update_v1(&peer, &request)
    })
    .await
    {
        Ok(value) => ResponseJson(value),
        Err(_) => ResponseJson(R::AssociationPublicationFailed {
            reason: F::Changed,
            effects: transport_api_types::WorkspaceAssociationUpdateEffectsV1::unknown(),
        }),
    }
}

/// Retire only through the original actual member and admitted Stop. The
/// inherited acceptor authenticated this peer before HTTP/body polling.
#[cfg(target_os = "linux")]
pub(crate) async fn ordinary_retained_worker_stop(
    State(service): State<WorldService>,
    request: axum::http::Request<hyper::Body>,
) -> Response {
    use crate::e3_local_transport::{
        E3AuthenticatedLinuxUdsListenerV1, E3AuthenticatedLinuxUdsPeerV1,
    };
    use crate::socket_activation::InheritedUnixListener;
    use hyper::body::HttpBody;
    use std::sync::Arc;
    let principal = (|| {
        let peer = request
            .extensions()
            .get::<E3AuthenticatedLinuxUdsPeerV1>()?;
        let inherited = request.extensions().get::<Arc<InheritedUnixListener>>()?;
        let configured = service.accepted_home_authority.as_ref()?;
        let listener =
            E3AuthenticatedLinuxUdsListenerV1::from_inherited(inherited, 1, Arc::clone(configured))
                .ok()?;
        if peer.peer_uid != listener.expected_peer_uid
            || peer.kernel_boot_id != listener.kernel_boot_id
            || peer.listener_descriptor_device_id != listener.descriptor_device_id
            || peer.listener_descriptor_inode != listener.descriptor_inode
            || peer.peer_pid == 0
            || peer.peer_pid_start_time_ticks == 0
            || libc::gid_t::try_from(peer.peer_gid).is_err()
        {
            return None;
        }
        u32::try_from(peer.peer_uid).ok().filter(|uid| *uid != 0)
    })();
    let Some(principal) = principal else {
        return (
            StatusCode::FORBIDDEN,
            ResponseJson(json!({"error":"ordinary_retained_stop_wrong_binding"})),
        )
            .into_response();
    };
    let mut body = request.into_body();
    let mut bytes = Vec::new();
    while let Some(chunk) = body.data().await {
        let Ok(chunk) = chunk else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        if bytes.len().saturating_add(chunk.len()) > 32768 {
            return StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }
        bytes.extend_from_slice(&chunk);
    }
    let Ok(request) =
        serde_json::from_slice::<transport_api_types::OrdinaryRetainedWorkerStopRequestV1>(&bytes)
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match service.retire_ordinary_retained_worker(&request, principal) {
        Ok(response) => ResponseJson(response).into_response(),
        Err(_) => (
            StatusCode::CONFLICT,
            ResponseJson(json!({"error":"ordinary_retained_stop_unresolved"})),
        )
            .into_response(),
    }
}

#[cfg(target_os = "linux")]
pub(crate) async fn ordinary_accepted_work_cancel(
    State(service): State<WorldService>,
    request: axum::http::Request<hyper::Body>,
) -> Response {
    use crate::e3_local_transport::{
        E3AuthenticatedLinuxUdsListenerV1, E3AuthenticatedLinuxUdsPeerV1,
    };
    use crate::socket_activation::InheritedUnixListener;
    use hyper::body::HttpBody;
    use std::sync::Arc;
    let principal = (|| {
        let peer = request
            .extensions()
            .get::<E3AuthenticatedLinuxUdsPeerV1>()?;
        let inherited = request.extensions().get::<Arc<InheritedUnixListener>>()?;
        let configured = service.accepted_home_authority.as_ref()?;
        let listener =
            E3AuthenticatedLinuxUdsListenerV1::from_inherited(inherited, 1, Arc::clone(configured))
                .ok()?;
        if peer.peer_uid != listener.expected_peer_uid
            || peer.kernel_boot_id != listener.kernel_boot_id
            || peer.listener_descriptor_device_id != listener.descriptor_device_id
            || peer.listener_descriptor_inode != listener.descriptor_inode
            || peer.peer_pid == 0
            || peer.peer_pid_start_time_ticks == 0
            || libc::gid_t::try_from(peer.peer_gid).is_err()
        {
            return None;
        }
        u32::try_from(peer.peer_uid).ok().filter(|uid| *uid != 0)
    })();
    let Some(principal) = principal else {
        return (
            StatusCode::FORBIDDEN,
            ResponseJson(json!({"error":"ordinary_accepted_cancel_wrong_binding"})),
        )
            .into_response();
    };
    let mut body = request.into_body();
    let mut bytes = Vec::new();
    while let Some(chunk) = body.data().await {
        let Ok(chunk) = chunk else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        if bytes.len().saturating_add(chunk.len()) > 32768 {
            return StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }
        bytes.extend_from_slice(&chunk);
    }
    let Ok(request) =
        serde_json::from_slice::<transport_api_types::OrdinaryAcceptedWorkCancelRequestV1>(&bytes)
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match service.cancel_ordinary_accepted_work(&request, principal) {
        Ok(delivered) => ResponseJson(transport_api_types::OrdinaryAcceptedWorkCancelResponseV1 {
            schema_version: 1,
            delivered,
        })
        .into_response(),
        Err(_) => (
            StatusCode::CONFLICT,
            ResponseJson(json!({"error":"ordinary_accepted_cancel_unresolved"})),
        )
            .into_response(),
    }
}

/// Initial control uses the authenticated carrying connection and the actual
/// reserved/bootstrap owner. A response is delivery data, never terminal proof.
#[cfg(target_os = "linux")]
pub(crate) async fn ordinary_initial_bootstrap_control(
    State(service): State<WorldService>,
    request: axum::http::Request<hyper::Body>,
) -> Response {
    use crate::e3_local_transport::{
        E3AuthenticatedLinuxUdsListenerV1, E3AuthenticatedLinuxUdsPeerV1,
    };
    use crate::socket_activation::InheritedUnixListener;
    use axum::extract::FromRequest;
    use std::sync::Arc;
    let Some(peer) = request
        .extensions()
        .get::<E3AuthenticatedLinuxUdsPeerV1>()
        .cloned()
    else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let Some(listener) = request
        .extensions()
        .get::<Arc<InheritedUnixListener>>()
        .cloned()
    else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let Some(configured) = service.accepted_home_authority.as_ref().cloned() else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let verify_peer = || {
        let current = E3AuthenticatedLinuxUdsListenerV1::from_inherited(
            &listener,
            1,
            Arc::clone(&configured),
        )
        .ok()?;
        if peer.peer_uid != current.expected_peer_uid
            || peer.kernel_boot_id != current.kernel_boot_id
            || peer.listener_descriptor_device_id != current.descriptor_device_id
            || peer.listener_descriptor_inode != current.descriptor_inode
            || peer.peer_pid == 0
            || peer.peer_pid_start_time_ticks == 0
            || libc::gid_t::try_from(peer.peer_gid).is_err()
        {
            return None;
        }
        u32::try_from(peer.peer_uid).ok().filter(|uid| *uid != 0)
    };
    if verify_peer().is_none() {
        return StatusCode::FORBIDDEN.into_response();
    }
    let bytes = match Bytes::from_request(request, &service).await {
        Ok(bytes) => bytes,
        Err(error) => return error.into_response(),
    };
    let Some(uid) = verify_peer() else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let Ok(request) = serde_json::from_slice::<
        transport_api_types::OrdinaryInitialBootstrapControlRequestV1,
    >(&bytes) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match service.control_ordinary_initial_bootstrap(&request, uid) {
        Ok(delivery) => ResponseJson(
            transport_api_types::OrdinaryInitialBootstrapControlResponseV1 {
                schema_version: 1,
                delivery,
            },
        )
        .into_response(),
        Err(_) => (
            StatusCode::CONFLICT,
            ResponseJson(json!({"error":"ordinary_initial_control_unresolved"})),
        )
            .into_response(),
    }
}

/// Initial ordinary input requires the real authenticated carrying connection.
/// Validate before polling any body and again after body collection. Parsing
/// uses the existing Axum Bytes extractor's configured/default body limit.
#[cfg(target_os = "linux")]
pub(crate) async fn ordinary_initial_execute_stream(
    State(service): State<WorldService>,
    request: axum::http::Request<hyper::Body>,
) -> Response {
    use crate::e3_local_transport::{
        E3AuthenticatedLinuxUdsListenerV1, E3AuthenticatedLinuxUdsPeerV1,
    };
    use crate::socket_activation::InheritedUnixListener;
    use axum::extract::FromRequest;
    use std::sync::Arc;
    let Some(peer) = request
        .extensions()
        .get::<E3AuthenticatedLinuxUdsPeerV1>()
        .cloned()
    else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let Some(listener) = request
        .extensions()
        .get::<Arc<InheritedUnixListener>>()
        .cloned()
    else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let Some(configured) = service.accepted_home_authority.as_ref().cloned() else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let verify_peer = || {
        let current = E3AuthenticatedLinuxUdsListenerV1::from_inherited(
            &listener,
            1,
            Arc::clone(&configured),
        )
        .ok()?;
        if peer.peer_uid != current.expected_peer_uid
            || peer.kernel_boot_id != current.kernel_boot_id
            || peer.listener_descriptor_device_id != current.descriptor_device_id
            || peer.listener_descriptor_inode != current.descriptor_inode
            || peer.peer_pid == 0
            || peer.peer_pid_start_time_ticks == 0
            || libc::gid_t::try_from(peer.peer_gid).is_err()
        {
            return None;
        }
        u32::try_from(peer.peer_uid).ok().filter(|uid| *uid != 0)
    };
    if verify_peer().is_none() {
        return StatusCode::FORBIDDEN.into_response();
    }
    let bytes = match Bytes::from_request(request, &service).await {
        Ok(bytes) => bytes,
        Err(error) => return error.into_response(),
    };
    if verify_peer().is_none() {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(request) = serde_json::from_slice::<ExecuteRequest>(&bytes) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if request.schema_version != 2
        || request.ordinary_start_selection.is_none()
        || request.member_dispatch.is_none()
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match service.execute_stream(request).await {
        Ok(response) => response,
        Err(_) => (
            StatusCode::CONFLICT,
            ResponseJson(json!({"error":"ordinary_initial_execute_refused_or_unknown"})),
        )
            .into_response(),
    }
}
