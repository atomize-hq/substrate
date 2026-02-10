#![cfg(all(unix, target_os = "linux"))]

use axum::routing::{get, post};
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use hyper::{Body, Request, StatusCode};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;
use world_agent::WorldAgentService;

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

fn minimal_policy_snapshot_v3_full_isolation() -> Value {
    json!({
        "schema_version": 3,
        "world_fs": {
            "host_visible": false,
            "fail_closed": { "routing": false },
            "deny_enforcement": "weak",
            "caged_required": false,
            "discover": { "allow_list": ["."], "deny_list": [] },
            "read": { "allow_list": ["."], "deny_list": [] },
            "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
        }
    })
}

fn minimal_execute_request_with_snapshot(policy_snapshot: Value) -> Value {
    json!({
        "cmd": "echo ok",
        "cwd": "/tmp",
        "env": { "HOME": "/root" },
        "pty": false,
        "agent_id": "wfgad1-test",
        "policy_snapshot": policy_snapshot,
    })
}

async fn post_execute_json(service: WorldAgentService, payload: Value) -> (StatusCode, Value) {
    let app = Router::new()
        .route("/v1/execute", post(world_agent::handlers::execute))
        .with_state(service);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/execute")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&payload).expect("serialize execute request JSON"),
        ))
        .expect("build request");

    let resp = app.oneshot(req).await.expect("execute request");
    let status = resp.status();
    let body = hyper::body::to_bytes(resp.into_body())
        .await
        .expect("read response body");

    let json: Value = serde_json::from_slice(&body).unwrap_or_else(|e| {
        panic!(
            "expected JSON body for /v1/execute response: {e}; body={}",
            String::from_utf8_lossy(&body)
        )
    });

    (status, json)
}

fn assert_http_400_error_shape(body: &Value) -> &str {
    let obj = body
        .as_object()
        .expect("response body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "HTTP 400 rejection must have only an `error` field: {body}"
    );
    let error = obj
        .get("error")
        .and_then(Value::as_str)
        .expect("error string");
    assert!(
        !error.trim().is_empty(),
        "error field must be non-empty: {body}"
    );
    error
}

#[tokio::test(flavor = "current_thread")]
async fn http_execute_rejects_policy_snapshot_schema_version_1_and_2() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping /v1/execute rejection test: service init failed: {err}");
            return;
        }
    };

    for legacy in [1u32, 2u32] {
        let mut snapshot = minimal_policy_snapshot_v3_full_isolation();
        snapshot["schema_version"] = json!(legacy);

        let (status, body) = post_execute_json(
            service.clone(),
            minimal_execute_request_with_snapshot(snapshot),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let error = assert_http_400_error_shape(&body);
        assert!(
            error.to_ascii_lowercase().contains("schema_version"),
            "expected schema_version diagnostic for schema_version={legacy}, got: {error}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn http_execute_rejects_policy_snapshot_unknown_fields() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping /v1/execute unknown-fields test: service init failed: {err}");
            return;
        }
    };

    let mut snapshot = minimal_policy_snapshot_v3_full_isolation();
    snapshot
        .as_object_mut()
        .expect("snapshot object")
        .insert("unknown_field".to_string(), json!(123));

    let (status, body) =
        post_execute_json(service, minimal_execute_request_with_snapshot(snapshot)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error = assert_http_400_error_shape(&body);
    assert!(
        error.to_ascii_lowercase().contains("unknown"),
        "expected unknown-field diagnostic, got: {error}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn http_execute_rejects_missing_policy_snapshot() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!(
                "skipping /v1/execute missing-policy_snapshot test: service init failed: {err}"
            );
            return;
        }
    };

    let payload = json!({
        "cmd": "echo ok",
        "cwd": "/tmp",
        "env": { "HOME": "/root" },
        "pty": false,
        "agent_id": "wfgad1-test",
    });

    let (status, body) = post_execute_json(service, payload).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error = assert_http_400_error_shape(&body);
    assert!(
        error.to_ascii_lowercase().contains("policy_snapshot"),
        "expected policy_snapshot diagnostic, got: {error}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn http_execute_rejects_policy_snapshot_invalid_allow_list_patterns() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping /v1/execute invalid-pattern test: service init failed: {err}");
            return;
        }
    };

    let mut snapshot = minimal_policy_snapshot_v3_full_isolation();
    snapshot["world_fs"]["read"]["allow_list"] = json!(["src/**"]);
    // In V3, wildcards are forbidden in allow_list.

    let (status, body) =
        post_execute_json(service, minimal_execute_request_with_snapshot(snapshot)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error = assert_http_400_error_shape(&body);
    assert!(
        error.to_ascii_lowercase().contains("allow_list")
            || error.to_ascii_lowercase().contains("wildcard")
            || error.contains('*'),
        "expected allow_list/wildcard diagnostic, got: {error}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn http_execute_rejects_write_disabled_without_fail_closed_routing() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping /v1/execute routing-invariant test: service init failed: {err}");
            return;
        }
    };

    let mut snapshot = minimal_policy_snapshot_v3_full_isolation();
    snapshot["world_fs"]["write"]["enabled"] = json!(false);
    snapshot["world_fs"]["fail_closed"]["routing"] = json!(false);
    // In V3, write.enabled=false requires fail_closed.routing=true.

    let (status, body) =
        post_execute_json(service, minimal_execute_request_with_snapshot(snapshot)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error = assert_http_400_error_shape(&body);
    assert!(
        error.to_ascii_lowercase().contains("write")
            || error.to_ascii_lowercase().contains("fail_closed")
            || error.to_ascii_lowercase().contains("routing"),
        "expected write/fail_closed.routing diagnostic, got: {error}"
    );
}

async fn spawn_world_agent_ws(
    service: WorldAgentService,
) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let router = Router::new()
        .route("/v1/stream", get(world_agent::handlers::stream))
        .with_state(service);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ws listener");
    let addr = listener.local_addr().expect("ws listener addr");
    let std_listener = listener.into_std().expect("into_std listener");

    let server = tokio::spawn(async move {
        let _ = axum::Server::from_tcp(std_listener)
            .expect("from_tcp")
            .serve(router.into_make_service())
            .await;
    });

    (addr, server)
}

async fn ws_connect(addr: SocketAddr) -> Ws {
    let (ws, _) = connect_async(format!("ws://{addr}/v1/stream"))
        .await
        .expect("connect ws");
    ws
}

async fn recv_json(ws: &mut Ws) -> Value {
    let msg = timeout(Duration::from_secs(2), ws.next())
        .await
        .expect("timed out waiting for ws message")
        .expect("ws closed without a message")
        .expect("ws read error");

    let Message::Text(text) = msg else {
        panic!("expected text ws message, got: {msg:?}");
    };
    serde_json::from_str(&text).expect("server ws message is valid JSON")
}

fn assert_ws_fatal_start_session_error(frame: &Value) -> &str {
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    let code = frame.get("code").and_then(Value::as_str);
    assert!(
        matches!(code, Some("bad_request" | "unsupported_protocol_version")),
        "expected code=bad_request or code=unsupported_protocol_version for start_session schema violations: {frame}"
    );
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));
    assert!(
        frame.get("seq").is_none(),
        "start_session failures MUST omit seq: {frame}"
    );
    let message = frame
        .get("message")
        .and_then(Value::as_str)
        .expect("error.message string");
    assert!(
        !message.trim().is_empty(),
        "error.message must be non-empty: {frame}"
    );
    message
}

async fn assert_ws_closes_after_fatal(ws: &mut Ws) {
    let msg = timeout(Duration::from_secs(2), ws.next()).await;
    match msg {
        Ok(None) => {}
        Ok(Some(Ok(Message::Close(_)))) => {}
        Ok(Some(Ok(other))) => panic!("expected ws close after fatal error, got: {other:?}"),
        Ok(Some(Err(_))) => {
            // Some implementations drop/reset the TCP connection without a close handshake.
            // This still satisfies “close the WebSocket connection” semantics.
        }
        Err(_) => panic!("timed out waiting for ws close after fatal error"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn ws_start_session_rejects_policy_snapshot_schema_version_1_and_2_and_closes() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping ws start_session rejection test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    for legacy in [1u32, 2u32] {
        let mut snapshot = minimal_policy_snapshot_v3_full_isolation();
        snapshot["schema_version"] = json!(legacy);

        let mut ws = ws_connect(addr).await;
        ws.send(Message::Text(
            json!({
                "type": "start_session",
                "cwd": cwd.display().to_string(),
                "env": { "HOME": "/root", "TERM": "xterm-256color" },
                "policy_snapshot": snapshot,
                "cols": 80,
                "rows": 24,
            })
            .to_string(),
        ))
        .await
        .expect("send start_session");

        let frame = recv_json(&mut ws).await;
        let message = assert_ws_fatal_start_session_error(&frame);
        assert!(
            message.to_ascii_lowercase().contains("schema_version"),
            "expected schema_version diagnostic for schema_version={legacy}, got: {message}"
        );
        assert_ws_closes_after_fatal(&mut ws).await;
    }

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn ws_start_session_rejects_policy_snapshot_unknown_fields_and_closes() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping ws unknown-fields test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut snapshot = minimal_policy_snapshot_v3_full_isolation();
    snapshot
        .as_object_mut()
        .expect("snapshot object")
        .insert("unknown_field".to_string(), json!(123));

    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        json!({
            "type": "start_session",
            "cwd": cwd.display().to_string(),
            "env": { "HOME": "/root", "TERM": "xterm-256color" },
            "policy_snapshot": snapshot,
            "cols": 80,
            "rows": 24,
        })
        .to_string(),
    ))
    .await
    .expect("send start_session");

    let frame = recv_json(&mut ws).await;
    let message = assert_ws_fatal_start_session_error(&frame);
    assert!(
        message.to_ascii_lowercase().contains("unknown"),
        "expected unknown-field diagnostic, got: {message}"
    );
    assert_ws_closes_after_fatal(&mut ws).await;

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn ws_start_session_rejects_missing_policy_snapshot_and_closes() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping ws missing-policy_snapshot test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        json!({
            "type": "start_session",
            "cwd": cwd.display().to_string(),
            "env": { "HOME": "/root", "TERM": "xterm-256color" },
            "cols": 80,
            "rows": 24,
        })
        .to_string(),
    ))
    .await
    .expect("send start_session");

    let frame = recv_json(&mut ws).await;
    let message = assert_ws_fatal_start_session_error(&frame);
    assert!(
        message.to_ascii_lowercase().contains("policy_snapshot"),
        "expected policy_snapshot diagnostic, got: {message}"
    );
    assert_ws_closes_after_fatal(&mut ws).await;

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn ws_start_session_rejects_policy_snapshot_invalid_allow_list_patterns_and_closes() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping ws invalid-pattern test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut snapshot = minimal_policy_snapshot_v3_full_isolation();
    snapshot["world_fs"]["read"]["allow_list"] = json!(["src/**"]);
    // In V3, wildcards are forbidden in allow_list.

    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        json!({
            "type": "start_session",
            "cwd": cwd.display().to_string(),
            "env": { "HOME": "/root", "TERM": "xterm-256color" },
            "policy_snapshot": snapshot,
            "cols": 80,
            "rows": 24,
        })
        .to_string(),
    ))
    .await
    .expect("send start_session");

    let frame = recv_json(&mut ws).await;
    let message = assert_ws_fatal_start_session_error(&frame);
    assert!(
        message.to_ascii_lowercase().contains("allow_list")
            || message.to_ascii_lowercase().contains("wildcard")
            || message.contains('*'),
        "expected allow_list/wildcard diagnostic, got: {message}"
    );
    assert_ws_closes_after_fatal(&mut ws).await;

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn ws_start_session_rejects_write_disabled_without_fail_closed_routing_and_closes() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping ws routing-invariant test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut snapshot = minimal_policy_snapshot_v3_full_isolation();
    snapshot["world_fs"]["write"]["enabled"] = json!(false);
    snapshot["world_fs"]["fail_closed"]["routing"] = json!(false);
    // In V3, write.enabled=false requires fail_closed.routing=true.

    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        json!({
            "type": "start_session",
            "cwd": cwd.display().to_string(),
            "env": { "HOME": "/root", "TERM": "xterm-256color" },
            "policy_snapshot": snapshot,
            "cols": 80,
            "rows": 24,
        })
        .to_string(),
    ))
    .await
    .expect("send start_session");

    let frame = recv_json(&mut ws).await;
    let message = assert_ws_fatal_start_session_error(&frame);
    assert!(
        message.to_ascii_lowercase().contains("write")
            || message.to_ascii_lowercase().contains("fail_closed")
            || message.to_ascii_lowercase().contains("routing"),
        "expected write/fail_closed.routing diagnostic, got: {message}"
    );
    assert_ws_closes_after_fatal(&mut ws).await;

    server.abort();
}
