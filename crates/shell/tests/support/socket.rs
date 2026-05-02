#![cfg(unix)]
#![allow(dead_code)]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

/// Behavior for socket stubs spawned during tests.
#[derive(Clone)]
pub enum SocketResponse {
    /// Responds to `/v1/capabilities` requests with a JSON payload that
    /// advertises socket activation mode.
    Capabilities,
    /// Responds to `/v1/capabilities` and `/v1/doctor/world` using a
    /// caller-supplied world doctor payload.
    CapabilitiesAndDoctorWorld { report: JsonValue },
    /// Responds to `/v1/capabilities`, `/v1/doctor/world`, and `/v1/pending_diff`
    /// with a JSON payload describing the session's pending diff record.
    ///
    /// This is a test stub for `workspace sync --dry-run` flows (WS1+).
    CapabilitiesAndPendingDiff {
        features: Vec<String>,
        pending_diff: JsonValue,
    },
    /// Responds to pending diff discovery plus a best-effort clear endpoint.
    ///
    /// The clear endpoint is intentionally permissive about the exact path so the
    /// WS2 tests can exercise "clear by diff_id" semantics.
    ///
    /// Supported routes:
    /// - `POST /v1/pending_diff/clear`
    /// - `POST /v1/workspace/pending_diff/clear`
    ///
    /// Request body must include a string `diff_id` field (or `diffId`).
    ///
    /// Response body matches `PendingDiffClearResponseV1` (`{ schema_version, cleared }`).
    CapabilitiesPendingDiffAndAck {
        state: Arc<Mutex<PendingDiffAckState>>,
    },
    /// Responds to `/v1/capabilities` and `/v1/doctor/world`, but returns a
    /// non-2xx response for `/v1/pending_diff`.
    ///
    /// Used to simulate a backend that is reachable for capabilities but fails
    /// on pending diff retrieval.
    CapabilitiesAndPendingDiffHttpError {
        features: Vec<String>,
        status: u16,
        body: String,
    },
    /// Handles capabilities and execute calls with canned payloads.
    CapabilitiesAndExecute {
        stdout: String,
        stderr: String,
        exit: i32,
        scopes: Vec<String>,
    },
    /// Like `CapabilitiesAndExecute`, but also records each `/v1/execute` and
    /// `/v1/execute/stream` request JSON payload.
    CapabilitiesAndExecuteRecord {
        stdout: String,
        stderr: String,
        exit: i32,
        scopes: Vec<String>,
        records: Arc<Mutex<Vec<JsonValue>>>,
    },
    /// Like `CapabilitiesAndExecute`, but includes explicit process-event
    /// diagnostics and optional batched `world_process_*` records.
    CapabilitiesAndExecuteWithProcessEvents {
        stdout: String,
        stderr: String,
        exit: i32,
        scopes: Vec<String>,
        process_events: Vec<JsonValue>,
        process_events_status: String,
        process_events_reason: Option<String>,
        process_events_dropped: Option<u64>,
    },
    /// Executes `/v1/execute` and `/v1/execute/stream` requests on the host, using
    /// the request's `cwd` and `env` for a lightweight world-agent simulation.
    CapabilitiesAndHostExecute { scopes: Vec<String> },
    /// Like `CapabilitiesAndHostExecute`, but also records each `/v1/execute` and
    /// `/v1/execute/stream` request JSON payload.
    CapabilitiesAndHostExecuteRecord {
        scopes: Vec<String>,
        records: Arc<Mutex<Vec<JsonValue>>>,
    },
    /// Responds to the typed gateway lifecycle routes.
    GatewayLifecycle {
        status: JsonValue,
        sync: JsonValue,
        restart: JsonValue,
    },
    /// Responds to the typed gateway lifecycle routes with an HTTP error.
    GatewayLifecycleHttpError { status: u16, body: String },
    /// Accepts connections but never returns a response (simulates a stuck
    /// systemd-managed socket where the service failed to start).
    Silent,
    /// Responds to `/v1/capabilities` and `/v1/doctor/world`, and returns an
    /// `execute/stream` error frame with the provided message.
    CapabilitiesAndExecuteStreamError { message: String },
    /// Handles non-PTY execute calls plus pending-diff discovery/clear routes.
    ///
    /// This is a test stub for WS3 auto-sync flows: run a successful non-PTY
    /// command (`POST /v1/execute`), then apply `workspace sync` automatically
    /// by consuming `pending_diff_v1` and `pending_diff_clear_v1`.
    CapabilitiesExecutePendingDiffAndAck {
        stdout: String,
        stderr: String,
        exit: i32,
        scopes: Vec<String>,
        state: Arc<Mutex<PendingDiffAckState>>,
    },
}

/// Minimal Unix socket server used to simulate socket-activated world-agent
/// listeners.
pub struct AgentSocket {
    path: PathBuf,
    shutdown: Arc<AtomicBool>,
    connections: Arc<AtomicUsize>,
    execute_requests: Arc<AtomicUsize>,
    reconcile_requests: Arc<AtomicUsize>,
    last_reconcile_discard_paths: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

/// Mutable state for [`SocketResponse::CapabilitiesPendingDiffAndAck`].
#[derive(Debug, Clone)]
pub struct PendingDiffAckState {
    pub features: Vec<String>,
    pub current_pending_diff: JsonValue,
    pub cleared_pending_diff: JsonValue,
    pub flip_after_first_pending_diff: Option<JsonValue>,
    pub ack_error: Option<PendingDiffAckError>,
    pub pending_diff_calls: usize,
    pub ack_calls: usize,
}

#[derive(Debug, Clone)]
pub struct PendingDiffAckError {
    pub status: u16,
    pub body: String,
}

impl AgentSocket {
    /// Spawn a new stub server bound to the provided path.
    pub fn start(path: &Path, response: SocketResponse) -> Self {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create socket parent");
        }
        let _ = fs::remove_file(path);
        let listener = UnixListener::bind(path).expect("failed to bind stub socket");
        listener
            .set_nonblocking(true)
            .expect("failed to configure stub socket");

        let socket_path = path.to_path_buf();
        let cleanup_path = socket_path.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let connections = Arc::new(AtomicUsize::new(0));
        let connections_for_thread = connections.clone();
        let execute_requests = Arc::new(AtomicUsize::new(0));
        let execute_requests_for_thread = execute_requests.clone();
        let reconcile_requests = Arc::new(AtomicUsize::new(0));
        let reconcile_requests_for_thread = reconcile_requests.clone();
        let last_reconcile_discard_paths: Arc<Mutex<Vec<String>>> =
            Arc::new(Mutex::new(Vec::new()));
        let last_reconcile_discard_paths_for_thread = last_reconcile_discard_paths.clone();

        let handle = thread::spawn(move || {
            while !shutdown_flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _addr)) => {
                        // `UnixListener` is configured as non-blocking; on some platforms the
                        // accepted stream inherits this flag. Switch back to blocking IO so the
                        // request reader doesn't spuriously drop connections with `WouldBlock`.
                        let _ = stream.set_nonblocking(false);
                        connections_for_thread.fetch_add(1, Ordering::SeqCst);
                        let request = match read_http_request(&mut stream) {
                            Ok(req) => req,
                            Err(_) => continue,
                        };
                        let first_line = request.header.lines().next().unwrap_or("");
                        if first_line.starts_with("POST /v1/execute")
                            || first_line.starts_with("POST /v1/execute/stream")
                        {
                            execute_requests_for_thread.fetch_add(1, Ordering::SeqCst);
                        } else if is_pending_diff_reconcile_route(first_line) {
                            reconcile_requests_for_thread.fetch_add(1, Ordering::SeqCst);
                            let discard_paths = extract_discard_paths(&request.body);
                            if let Ok(mut guard) = last_reconcile_discard_paths_for_thread.lock() {
                                *guard = discard_paths;
                            }
                        }
                        match &response {
                            SocketResponse::Capabilities => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndDoctorWorld { report } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report_with_body(&mut stream, report);
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndPendingDiff {
                                features,
                                pending_diff,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities_with_features(&mut stream, features);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if is_pending_diff_discovery_route(first_line) {
                                    write_response(&mut stream, &pending_diff.to_string());
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesPendingDiffAndAck { state } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    let features = state
                                        .lock()
                                        .ok()
                                        .map(|s| s.features.clone())
                                        .unwrap_or_else(|| vec!["execute".to_string()]);
                                    write_capabilities_with_features(&mut stream, &features);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if is_pending_diff_reconcile_route(first_line) {
                                    let mut guard =
                                        state.lock().expect("lock pending diff ack state");
                                    let req_id = match extract_diff_id(&request.body) {
                                        Some(id) => id,
                                        None => {
                                            write_status_response(
                                                &mut stream,
                                                400,
                                                "Bad Request",
                                                "{\"error\":\"missing diff_id\"}",
                                            );
                                            continue;
                                        }
                                    };
                                    let cur_id = guard
                                        .current_pending_diff
                                        .get("diff_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    if cur_id != req_id {
                                        write_response(
                                            &mut stream,
                                            "{\"schema_version\":1,\"reconciled\":false,\"discarded\":0}",
                                        );
                                        continue;
                                    }

                                    let discard_paths = extract_discard_paths(&request.body);
                                    let discarded = discard_paths
                                        .iter()
                                        .map(|p| {
                                            discard_from_pending_diff(
                                                &mut guard.current_pending_diff,
                                                p,
                                            )
                                        })
                                        .sum::<u32>();
                                    write_response(
                                        &mut stream,
                                        &format!(
                                            "{{\"schema_version\":1,\"reconciled\":true,\"discarded\":{discarded}}}"
                                        ),
                                    );
                                } else if is_pending_diff_ack_route(first_line) {
                                    let mut guard =
                                        state.lock().expect("lock pending diff ack state");
                                    guard.ack_calls += 1;
                                    if let Some(err) = guard.ack_error.clone() {
                                        write_status_response(
                                            &mut stream,
                                            err.status,
                                            http_reason_phrase(err.status),
                                            &err.body,
                                        );
                                        continue;
                                    }

                                    let req_id = match extract_diff_id(&request.body) {
                                        Some(id) => id,
                                        None => {
                                            write_status_response(
                                                &mut stream,
                                                400,
                                                "Bad Request",
                                                "{\"error\":\"missing diff_id\"}",
                                            );
                                            continue;
                                        }
                                    };
                                    let cur_id = guard
                                        .current_pending_diff
                                        .get("diff_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    if cur_id != req_id {
                                        write_response(
                                            &mut stream,
                                            "{\"schema_version\":1,\"cleared\":false}",
                                        );
                                        continue;
                                    }

                                    guard.current_pending_diff = guard.cleared_pending_diff.clone();
                                    write_response(
                                        &mut stream,
                                        "{\"schema_version\":1,\"cleared\":true}",
                                    );
                                } else if is_pending_diff_discovery_route(first_line) {
                                    let mut guard =
                                        state.lock().expect("lock pending diff ack state");
                                    let payload = guard.current_pending_diff.to_string();
                                    write_response(&mut stream, &payload);
                                    guard.pending_diff_calls += 1;
                                    if guard.pending_diff_calls == 1 {
                                        if let Some(next) =
                                            guard.flip_after_first_pending_diff.take()
                                        {
                                            guard.current_pending_diff = next;
                                        }
                                    }
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndPendingDiffHttpError {
                                features,
                                status,
                                body,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities_with_features(&mut stream, features);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if is_pending_diff_discovery_route(first_line) {
                                    write_status_response(
                                        &mut stream,
                                        *status,
                                        http_reason_phrase(*status),
                                        body,
                                    );
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndExecute {
                                stdout,
                                stderr,
                                exit,
                                scopes,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    let payload =
                                        build_stream_payload(*exit, stdout, stderr, scopes);
                                    write_stream_response(&mut stream, &payload);
                                } else if first_line.starts_with("POST /v1/execute") {
                                    let mut payload = json!({
                                        "exit": exit,
                                        "span_id": "agent-span",
                                        "stdout_b64": BASE64.encode(stdout.as_bytes()),
                                        "stderr_b64": BASE64.encode(stderr.as_bytes()),
                                        "scopes_used": scopes,
                                        "fs_diff": serde_json::Value::Null
                                    });
                                    apply_default_process_telemetry_fields(&mut payload);
                                    let payload = payload.to_string();
                                    write_response(&mut stream, &payload);
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndExecuteRecord {
                                stdout,
                                stderr,
                                exit,
                                scopes,
                                records,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    record_execute_request(records, &request);
                                    let payload =
                                        build_stream_payload(*exit, stdout, stderr, scopes);
                                    write_stream_response(&mut stream, &payload);
                                } else if first_line.starts_with("POST /v1/execute") {
                                    record_execute_request(records, &request);
                                    let mut payload = json!({
                                        "exit": exit,
                                        "span_id": "agent-span",
                                        "stdout_b64": BASE64.encode(stdout.as_bytes()),
                                        "stderr_b64": BASE64.encode(stderr.as_bytes()),
                                        "scopes_used": scopes,
                                        "fs_diff": serde_json::Value::Null
                                    });
                                    apply_default_process_telemetry_fields(&mut payload);
                                    let payload = payload.to_string();
                                    write_response(&mut stream, &payload);
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndExecuteWithProcessEvents {
                                stdout,
                                stderr,
                                exit,
                                scopes,
                                process_events,
                                process_events_status,
                                process_events_reason,
                                process_events_dropped,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    let payload = build_stream_payload_with_process_events(
                                        *exit,
                                        stdout,
                                        stderr,
                                        scopes,
                                        process_events,
                                        process_events_status,
                                        process_events_reason.as_deref(),
                                        *process_events_dropped,
                                    );
                                    write_stream_response(&mut stream, &payload);
                                } else if first_line.starts_with("POST /v1/execute") {
                                    let mut payload = json!({
                                        "exit": exit,
                                        "span_id": "agent-span",
                                        "stdout_b64": BASE64.encode(stdout.as_bytes()),
                                        "stderr_b64": BASE64.encode(stderr.as_bytes()),
                                        "scopes_used": scopes,
                                        "fs_diff": serde_json::Value::Null,
                                        "process_events": process_events,
                                        "process_events_status": process_events_status,
                                        "process_events_max": serde_json::Value::Null,
                                        "process_events_backend": serde_json::Value::Null,
                                        "process_events_error": serde_json::Value::Null,
                                    });
                                    if let Some(reason) = process_events_reason {
                                        payload["process_events_reason"] = json!(reason);
                                    }
                                    if let Some(dropped) = process_events_dropped {
                                        payload["process_events_dropped"] = json!(dropped);
                                    }
                                    write_response(&mut stream, &payload.to_string());
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndHostExecute { scopes } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    match handle_host_execute_stream(&request, scopes) {
                                        Ok(payload) => write_stream_response(&mut stream, &payload),
                                        Err(err) => {
                                            let payload = json!({
                                                "type": "error",
                                                "message": format!("{:#}", err)
                                            })
                                            .to_string();
                                            write_stream_response(&mut stream, &(payload + "\n"));
                                        }
                                    }
                                } else if first_line.starts_with("POST /v1/execute") {
                                    match handle_host_execute(&request, scopes) {
                                        Ok(payload) => write_response(&mut stream, &payload),
                                        Err(err) => {
                                            let payload = json!({
                                                "error": "internal",
                                                "message": format!("{:#}", err)
                                            })
                                            .to_string();
                                            let _ = stream.write_all(
                                                format!(
                                                    "HTTP/1.1 500 Internal Server Error\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                                    payload.len(),
                                                    payload
                                                )
                                                .as_bytes(),
                                            );
                                        }
                                    }
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndHostExecuteRecord {
                                scopes,
                                records,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    record_execute_request(records, &request);
                                    match handle_host_execute_stream(&request, scopes) {
                                        Ok(payload) => write_stream_response(&mut stream, &payload),
                                        Err(err) => {
                                            let payload = json!({
                                                "type": "error",
                                                "message": format!("{:#}", err)
                                            })
                                            .to_string();
                                            write_stream_response(&mut stream, &(payload + "\n"));
                                        }
                                    }
                                } else if first_line.starts_with("POST /v1/execute") {
                                    record_execute_request(records, &request);
                                    match handle_host_execute(&request, scopes) {
                                        Ok(payload) => write_response(&mut stream, &payload),
                                        Err(err) => {
                                            let payload = json!({
                                                "error": "internal",
                                                "message": format!("{:#}", err)
                                            })
                                            .to_string();
                                            let _ = stream.write_all(
                                                format!(
                                                    "HTTP/1.1 500 Internal Server Error\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                                    payload.len(),
                                                    payload
                                                )
                                                .as_bytes(),
                                            );
                                        }
                                    }
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::GatewayLifecycle {
                                status,
                                sync,
                                restart,
                            } => {
                                if first_line.starts_with("POST /v1/gateway/status") {
                                    write_response(&mut stream, &status.to_string());
                                } else if first_line.starts_with("POST /v1/gateway/sync") {
                                    write_response(&mut stream, &sync.to_string());
                                } else if first_line.starts_with("POST /v1/gateway/restart") {
                                    write_response(&mut stream, &restart.to_string());
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::GatewayLifecycleHttpError { status, body } => {
                                if first_line.starts_with("POST /v1/gateway/status")
                                    || first_line.starts_with("POST /v1/gateway/sync")
                                    || first_line.starts_with("POST /v1/gateway/restart")
                                {
                                    write_status_response(
                                        &mut stream,
                                        *status,
                                        http_reason_phrase(*status),
                                        body,
                                    );
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::Silent => {
                                // Read and drop the request to simulate a hung service.
                                let mut discard = [0u8; 512];
                                let _ = stream.read(&mut discard);
                            }
                            SocketResponse::CapabilitiesAndExecuteStreamError { message } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    let payload = build_stream_error_payload(message);
                                    write_stream_response(&mut stream, &payload);
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesExecutePendingDiffAndAck {
                                stdout,
                                stderr,
                                exit,
                                scopes,
                                state,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    let features = state
                                        .lock()
                                        .ok()
                                        .map(|s| s.features.clone())
                                        .unwrap_or_else(|| vec!["execute".to_string()]);
                                    write_capabilities_with_features(&mut stream, &features);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    let payload =
                                        build_stream_payload(*exit, stdout, stderr, scopes);
                                    write_stream_response(&mut stream, &payload);
                                } else if first_line.starts_with("POST /v1/execute") {
                                    let mut payload = json!({
                                        "exit": exit,
                                        "span_id": "agent-span",
                                        "stdout_b64": BASE64.encode(stdout.as_bytes()),
                                        "stderr_b64": BASE64.encode(stderr.as_bytes()),
                                        "scopes_used": scopes,
                                        "fs_diff": serde_json::Value::Null
                                    });
                                    apply_default_process_telemetry_fields(&mut payload);
                                    let payload = payload.to_string();
                                    write_response(&mut stream, &payload);
                                } else if is_pending_diff_reconcile_route(first_line) {
                                    let mut guard =
                                        state.lock().expect("lock pending diff ack state");
                                    let req_id = match extract_diff_id(&request.body) {
                                        Some(id) => id,
                                        None => {
                                            write_status_response(
                                                &mut stream,
                                                400,
                                                "Bad Request",
                                                "{\"error\":\"missing diff_id\"}",
                                            );
                                            continue;
                                        }
                                    };
                                    let cur_id = guard
                                        .current_pending_diff
                                        .get("diff_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    if cur_id != req_id {
                                        write_response(
                                            &mut stream,
                                            "{\"schema_version\":1,\"reconciled\":false,\"discarded\":0}",
                                        );
                                        continue;
                                    }

                                    let discard_paths = extract_discard_paths(&request.body);
                                    let discarded = discard_paths
                                        .iter()
                                        .map(|p| {
                                            discard_from_pending_diff(
                                                &mut guard.current_pending_diff,
                                                p,
                                            )
                                        })
                                        .sum::<u32>();
                                    write_response(
                                        &mut stream,
                                        &format!(
                                            "{{\"schema_version\":1,\"reconciled\":true,\"discarded\":{discarded}}}"
                                        ),
                                    );
                                } else if is_pending_diff_ack_route(first_line) {
                                    let mut guard =
                                        state.lock().expect("lock pending diff ack state");
                                    guard.ack_calls += 1;
                                    if let Some(err) = guard.ack_error.clone() {
                                        write_status_response(
                                            &mut stream,
                                            err.status,
                                            http_reason_phrase(err.status),
                                            &err.body,
                                        );
                                        continue;
                                    }

                                    let req_id = match extract_diff_id(&request.body) {
                                        Some(id) => id,
                                        None => {
                                            write_status_response(
                                                &mut stream,
                                                400,
                                                "Bad Request",
                                                "{\"error\":\"missing diff_id\"}",
                                            );
                                            continue;
                                        }
                                    };
                                    let cur_id = guard
                                        .current_pending_diff
                                        .get("diff_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    if cur_id != req_id {
                                        write_response(
                                            &mut stream,
                                            "{\"schema_version\":1,\"cleared\":false}",
                                        );
                                        continue;
                                    }

                                    guard.current_pending_diff = guard.cleared_pending_diff.clone();
                                    write_response(
                                        &mut stream,
                                        "{\"schema_version\":1,\"cleared\":true}",
                                    );
                                } else if is_pending_diff_discovery_route(first_line) {
                                    let mut guard =
                                        state.lock().expect("lock pending diff ack state");
                                    let payload = guard.current_pending_diff.to_string();
                                    write_response(&mut stream, &payload);
                                    guard.pending_diff_calls += 1;
                                    if guard.pending_diff_calls == 1 {
                                        if let Some(next) =
                                            guard.flip_after_first_pending_diff.take()
                                        {
                                            guard.current_pending_diff = next;
                                        }
                                    }
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                                }
                            }
                        };
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }

            let _ = fs::remove_file(&socket_path);
        });

        Self {
            path: cleanup_path,
            shutdown,
            connections,
            execute_requests,
            reconcile_requests,
            last_reconcile_discard_paths,
            handle: Some(handle),
        }
    }

    /// Return the on-disk socket path for the stub.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the number of accepted connections.
    pub fn connection_count(&self) -> usize {
        self.connections.load(Ordering::SeqCst)
    }

    /// Return the number of accepted `/v1/execute*` requests.
    pub fn execute_request_count(&self) -> usize {
        self.execute_requests.load(Ordering::SeqCst)
    }

    /// Return the number of accepted `/v1/pending_diff/reconcile` requests.
    pub fn reconcile_request_count(&self) -> usize {
        self.reconcile_requests.load(Ordering::SeqCst)
    }

    /// Return the last observed `discard_paths` payload from a reconcile request.
    pub fn last_reconcile_discard_paths(&self) -> Vec<String> {
        self.last_reconcile_discard_paths
            .lock()
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}

impl Drop for AgentSocket {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = UnixStream::connect(&self.path);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn is_pending_diff_ack_route(first_line: &str) -> bool {
    first_line.starts_with("POST /v1/pending_diff/ack ")
        || first_line.starts_with("POST /v1/pending_diff/clear ")
        || first_line.starts_with("POST /v1/workspace/pending_diff/ack ")
        || first_line.starts_with("POST /v1/workspace/pending_diff/clear ")
}

fn is_pending_diff_reconcile_route(first_line: &str) -> bool {
    first_line.starts_with("POST /v1/pending_diff/reconcile ")
        || first_line.starts_with("POST /v1/workspace/pending_diff/reconcile ")
}

fn is_pending_diff_discovery_route(first_line: &str) -> bool {
    first_line.starts_with("GET /v1/pending_diff ")
        || first_line.starts_with("POST /v1/pending_diff ")
        || first_line.starts_with("GET /v1/workspace/pending_diff ")
        || first_line.starts_with("POST /v1/workspace/pending_diff ")
}

fn extract_discard_paths(body: &[u8]) -> Vec<String> {
    let value = serde_json::from_slice::<JsonValue>(body).ok();
    let Some(value) = value else {
        return Vec::new();
    };
    value
        .get("discard_paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn discard_from_pending_diff(pending_diff: &mut JsonValue, path: &str) -> u32 {
    fn remove_from_bucket(bucket: &mut JsonValue, key: &str, path: &str) -> u32 {
        let Some(arr) = bucket.get_mut(key).and_then(|v| v.as_array_mut()) else {
            return 0;
        };
        let before = arr.len();
        arr.retain(|v| v.as_str() != Some(path));
        (before.saturating_sub(arr.len())) as u32
    }

    let mut removed = 0u32;
    if let Some(non_pty) = pending_diff.get_mut("non_pty") {
        removed = removed.saturating_add(remove_from_bucket(non_pty, "writes", path));
        removed = removed.saturating_add(remove_from_bucket(non_pty, "mods", path));
        removed = removed.saturating_add(remove_from_bucket(non_pty, "deletes", path));
    }
    if let Some(pty) = pending_diff.get_mut("pty") {
        removed = removed.saturating_add(remove_from_bucket(pty, "writes", path));
        removed = removed.saturating_add(remove_from_bucket(pty, "mods", path));
        removed = removed.saturating_add(remove_from_bucket(pty, "deletes", path));
    }
    removed
}

fn extract_diff_id(body: &[u8]) -> Option<String> {
    let value = serde_json::from_slice::<JsonValue>(body).ok()?;
    value
        .get("diff_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            value
                .get("diffId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

fn write_capabilities(stream: &mut UnixStream) {
    write_capabilities_with_features(stream, &["execute".to_string()]);
}

fn write_capabilities_with_features(stream: &mut UnixStream, features: &[String]) {
    let body = json!({
        "version": "v1",
        "features": features,
        "backend": "world-agent",
        "platform": "linux",
        "listener_mode": "socket_activation"
    })
    .to_string();
    write_response(stream, &body);
}

fn write_world_doctor_report(stream: &mut UnixStream) {
    let body = json!({
        "schema_version": 2,
        "ok": true,
        "collected_at_utc": "2026-01-08T00:00:00Z",
        "policy_snapshot_v1_supported": true,
        "policy_resolution_mode": null,
        "netfilter_status": {
            "requested": false,
            "enabled": false,
            "world_netfilter_enable_present": false,
            "last_failure_reason": null
        },
        "landlock": {
            "supported": true,
            "abi": 3,
            "reason": null
        },
        "world_fs_strategy": {
            "primary": "overlay",
            "fallback": "fuse",
            "probe": {
                "id": "enumeration_v1",
                "probe_file": ".substrate_enum_probe",
                "result": "pass",
                "failure_reason": null
            }
        }
    });
    write_world_doctor_report_with_body(stream, &body);
}

fn write_world_doctor_report_with_body(stream: &mut UnixStream, body: &JsonValue) {
    write_response(stream, &body.to_string());
}

fn write_response(stream: &mut UnixStream, body: &str) {
    let reply = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(reply.as_bytes());
}

fn http_reason_phrase(status: u16) -> &'static str {
    match status {
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

fn write_status_response(stream: &mut UnixStream, status: u16, reason: &str, body: &str) {
    let reply = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        reason,
        body.len(),
        body
    );
    let _ = stream.write_all(reply.as_bytes());
}

fn write_stream_response(stream: &mut UnixStream, body: &str) {
    let reply = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(reply.as_bytes());
}

struct HttpRequest {
    header: String,
    body: Vec<u8>,
}

fn record_execute_request(records: &Arc<Mutex<Vec<JsonValue>>>, request: &HttpRequest) {
    if let Ok(value) = serde_json::from_slice::<JsonValue>(&request.body) {
        if let Ok(mut guard) = records.lock() {
            guard.push(value);
        }
    }
}

pub fn decode_recorded_execute_requests(
    records: &Arc<Mutex<Vec<JsonValue>>>,
) -> anyhow::Result<Vec<agent_api_types::ExecuteRequest>> {
    let guard = records
        .lock()
        .map_err(|_| anyhow::anyhow!("recorded execute request mutex poisoned"))?;
    guard
        .iter()
        .cloned()
        .map(serde_json::from_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow::anyhow!("invalid recorded execute request: {err}"))
}

fn read_http_request(stream: &mut UnixStream) -> std::io::Result<HttpRequest> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let mut header_end = None;
    let mut expected_len = None;
    let mut chunked = false;

    loop {
        let read = stream.read(&mut tmp)?;
        if read == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..read]);

        if header_end.is_none() {
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                let header = String::from_utf8_lossy(&buf[..pos + 4]).to_string();
                expected_len = Some(parse_content_length(&header));
                chunked = header.lines().any(|line| {
                    line.to_ascii_lowercase().starts_with("transfer-encoding:")
                        && line.to_ascii_lowercase().contains("chunked")
                });
            }
        }

        if let Some(h_end) = header_end {
            if chunked {
                if decode_chunked_body(&buf[h_end..]).is_some() {
                    break;
                }
            } else if let Some(len) = expected_len {
                if buf.len() >= h_end + len {
                    break;
                }
            }
        }
    }

    let header_end = header_end.unwrap_or(buf.len());
    let header = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let body = if chunked {
        decode_chunked_body(&buf[header_end..]).unwrap_or_default()
    } else {
        let len = expected_len.unwrap_or_else(|| parse_content_length(&header));
        let body_start = header_end;
        let body_end = std::cmp::min(body_start + len, buf.len());
        buf[body_start..body_end].to_vec()
    };

    Ok(HttpRequest { header, body })
}

fn parse_content_length(header: &str) -> usize {
    header
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            if key.eq_ignore_ascii_case("content-length") {
                Some(value.trim().parse::<usize>().ok()?)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn decode_chunked_body(buf: &[u8]) -> Option<Vec<u8>> {
    let mut pos = 0usize;
    let mut out = Vec::new();

    loop {
        let line_end = buf[pos..].windows(2).position(|w| w == b"\r\n")? + pos;
        let line = &buf[pos..line_end];
        let line_str = std::str::from_utf8(line).ok()?;
        let size_str = line_str.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_str, 16).ok()?;
        pos = line_end + 2;
        if size == 0 {
            // Expect trailing CRLF after the 0-size chunk payload (no trailers).
            // Without this, we can treat a partial `0\r\n` read as completion and respond early,
            // causing clients to see broken pipes while still streaming the request body.
            if buf.len() < pos + 2 {
                return None;
            }
            if &buf[pos..pos + 2] != b"\r\n" {
                return None;
            }
            return Some(out);
        }
        if buf.len() < pos + size + 2 {
            return None;
        }
        out.extend_from_slice(&buf[pos..pos + size]);
        pos += size;
        if &buf[pos..pos + 2] != b"\r\n" {
            return None;
        }
        pos += 2;
    }
}

#[derive(Debug, Deserialize)]
struct ExecuteRequestStub {
    cmd: String,
    cwd: Option<String>,
    env: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    member_dispatch: Option<agent_api_types::MemberDispatchRequestV1>,
}

fn handle_host_execute(request: &HttpRequest, scopes: &[String]) -> anyhow::Result<String> {
    let parsed: ExecuteRequestStub = serde_json::from_slice(&request.body)?;
    let output = run_host_command(&parsed)?;
    let mut payload = json!({
        "exit": output.exit,
        "span_id": "agent-span",
        "stdout_b64": BASE64.encode(&output.stdout),
        "stderr_b64": BASE64.encode(&output.stderr),
        "scopes_used": scopes,
        "fs_diff": serde_json::Value::Null,
    });
    apply_default_process_telemetry_fields(&mut payload);
    Ok(payload.to_string())
}

fn handle_host_execute_stream(request: &HttpRequest, scopes: &[String]) -> anyhow::Result<String> {
    let parsed: ExecuteRequestStub = serde_json::from_slice(&request.body)?;
    let output = run_host_command(&parsed)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(build_stream_payload(
        output.exit,
        stdout.as_ref(),
        stderr.as_ref(),
        scopes,
    ))
}

struct HostCommandOutput {
    exit: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_host_command(request: &ExecuteRequestStub) -> anyhow::Result<HostCommandOutput> {
    use std::process::Command;

    if request.member_dispatch.is_some() {
        return Ok(HostCommandOutput {
            exit: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
        });
    }

    let mut cmd = Command::new("bash");
    cmd.arg("-c").arg(&request.cmd);
    if let Some(cwd) = &request.cwd {
        cmd.current_dir(cwd);
    }
    if let Some(env) = &request.env {
        cmd.envs(env);
    }
    cmd.env_remove("BASH_ENV");

    let output = cmd.output()?;
    Ok(HostCommandOutput {
        exit: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn build_stream_payload(exit: i32, stdout: &str, stderr: &str, scopes: &[String]) -> String {
    build_stream_payload_with_process_events(
        exit,
        stdout,
        stderr,
        scopes,
        &[],
        "unavailable",
        Some("backend_disabled"),
        None,
    )
}

fn apply_default_process_telemetry_fields(payload: &mut JsonValue) {
    payload["process_events"] = json!([]);
    payload["process_events_status"] = json!("unavailable");
    payload["process_events_reason"] = json!("backend_disabled");
    payload["process_events_dropped"] = JsonValue::Null;
    payload["process_events_max"] = JsonValue::Null;
    payload["process_events_backend"] = JsonValue::Null;
    payload["process_events_error"] = JsonValue::Null;
}

#[allow(clippy::too_many_arguments)]
fn build_stream_payload_with_process_events(
    exit: i32,
    stdout: &str,
    stderr: &str,
    scopes: &[String],
    process_events: &[JsonValue],
    process_events_status: &str,
    process_events_reason: Option<&str>,
    process_events_dropped: Option<u64>,
) -> String {
    let mut frames = String::new();
    frames.push_str(
        &json!({
            "type": "start",
            "span_id": "agent-span"
        })
        .to_string(),
    );
    frames.push('\n');
    if !stdout.is_empty() {
        frames.push_str(
            &json!({
                "type": "stdout",
                "chunk_b64": BASE64.encode(stdout.as_bytes())
            })
            .to_string(),
        );
        frames.push('\n');
    }
    if !stderr.is_empty() {
        frames.push_str(
            &json!({
                "type": "stderr",
                "chunk_b64": BASE64.encode(stderr.as_bytes())
            })
            .to_string(),
        );
        frames.push('\n');
    }
    let mut exit_frame = json!({
        "type": "exit",
        "exit": exit,
        "span_id": "agent-span",
        "scopes_used": scopes,
        "fs_diff": serde_json::Value::Null,
        "process_events": process_events,
        "process_events_status": process_events_status,
    });
    if let Some(reason) = process_events_reason {
        exit_frame["process_events_reason"] = json!(reason);
    }
    if let Some(dropped) = process_events_dropped {
        exit_frame["process_events_dropped"] = json!(dropped);
    }
    frames.push_str(&exit_frame.to_string());
    frames.push('\n');
    frames
}

fn build_stream_error_payload(message: &str) -> String {
    let mut frames = String::new();
    frames.push_str(
        &json!({
            "type": "start",
            "span_id": "agent-span"
        })
        .to_string(),
    );
    frames.push('\n');
    frames.push_str(
        &json!({
            "type": "error",
            "message": message
        })
        .to_string(),
    );
    frames.push('\n');
    frames
}
