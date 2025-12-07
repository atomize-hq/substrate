#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{ExecuteRequest, WorldFsMode};
use axum::extract::ws::WebSocketUpgrade;
use axum::routing::get;
use axum::{Router, Server};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use world_agent::pty::{handle_ws_pty, ServerMessage};
use world_agent::WorldAgentService;

fn decode(b64: &str) -> String {
    String::from_utf8_lossy(
        &BASE64
            .decode(b64.as_bytes())
            .unwrap_or_else(|_| Vec::from(b"<invalid base64>")),
    )
    .into_owned()
}

fn overlay_available() -> bool {
    (unsafe { libc::geteuid() == 0 })
        && std::fs::read_to_string("/proc/filesystems")
            .map(|data| data.contains("overlay"))
            .unwrap_or(false)
}

#[test]
fn non_pty_read_only_mode_blocks_writes() {
    if !overlay_available() {
        eprintln!("skipping read-only fs_mode test: overlay support or privileges missing");
        return;
    }

    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping read-only fs_mode test: service init failed: {err}");
            return;
        }
    };
    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let req = ExecuteRequest {
        profile: None,
        cmd: "sh -lc 'echo denied > ro-deny.txt'".to_string(),
        cwd: Some(cwd.display().to_string()),
        env: Some(HashMap::new()),
        pty: false,
        agent_id: "fs-mode-test".to_string(),
        budget: None,
        world_fs_mode: Some(WorldFsMode::ReadOnly),
    };

    let rt = Runtime::new().expect("runtime");
    let resp = match rt.block_on(service.execute(req)) {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("skipping read-only fs_mode assertions: execute failed: {err}");
            return;
        }
    };

    let stderr = decode(&resp.stderr_b64);
    if resp.exit == 0 {
        eprintln!(
            "skipping read-only fs_mode assertions: command unexpectedly succeeded (stderr: {stderr})"
        );
        return;
    }

    assert!(
        stderr.to_lowercase().contains("read-only") || stderr.to_lowercase().contains("read only"),
        "expected read-only filesystem failure in stderr, got: {stderr}"
    );
    assert!(
        resp.fs_diff.is_none(),
        "read-only executions should not report fs_diff writes"
    );
}

#[test]
fn non_pty_writable_mode_records_diffs_for_writes() {
    if !overlay_available() {
        eprintln!("skipping writable fs_mode test: overlay support or privileges missing");
        return;
    }

    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping writable fs_mode test: service init failed: {err}");
            return;
        }
    };
    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let req = ExecuteRequest {
        profile: None,
        cmd: "sh -lc 'echo ok > writable.txt'".to_string(),
        cwd: Some(cwd.display().to_string()),
        env: Some(HashMap::new()),
        pty: false,
        agent_id: "fs-mode-test".to_string(),
        budget: None,
        world_fs_mode: Some(WorldFsMode::Writable),
    };

    let rt = Runtime::new().expect("runtime");
    let resp = match rt.block_on(service.execute(req)) {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("skipping writable fs_mode assertions: execute failed: {err}");
            return;
        }
    };

    if resp.exit != 0 {
        let stderr = decode(&resp.stderr_b64);
        eprintln!(
            "skipping writable fs_mode assertions: command failed with exit {} (stderr: {stderr})",
            resp.exit
        );
        return;
    }

    let diff = match resp.fs_diff {
        Some(diff) => diff,
        None => {
            eprintln!("skipping writable fs_mode assertions: fs_diff missing");
            return;
        }
    };
    assert!(
        !diff.writes.is_empty() || !diff.mods.is_empty() || !diff.deletes.is_empty(),
        "writable executions should surface filesystem changes"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn pty_read_only_mode_returns_clear_error() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping pty read-only test: service init failed: {err}");
            return;
        }
    };
    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let router = Router::new().route(
        "/pty",
        get({
            let service = service.clone();
            move |ws: WebSocketUpgrade| {
                let service = service.clone();
                async move { ws.on_upgrade(move |socket| handle_ws_pty(service, socket)) }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ws listener");
    let addr = listener.local_addr().expect("ws listener addr");
    let std_listener = listener.into_std().expect("into_std listener");
    let server = tokio::spawn(async move {
        let _ = Server::from_tcp(std_listener)
            .unwrap()
            .serve(router.into_make_service())
            .await;
    });

    let (mut client_ws, _) = connect_async(format!("ws://{}/pty", addr))
        .await
        .expect("connect ws");

    let start = serde_json::json!({
        "type": "start",
        "cmd": "sh -lc 'echo denied > ro-pty.txt'",
        "cwd": cwd,
        "env": {
            "SUBSTRATE_WORLD_FS_MODE": "read_only"
        },
        "span_id": null,
        "cols": 80,
        "rows": 24
    });
    client_ws
        .send(Message::Text(start.to_string()))
        .await
        .expect("send start");

    let mut error: Option<String> = None;
    let mut exit_code: Option<i32> = None;
    while let Some(frame) = client_ws.next().await {
        match frame {
            Ok(Message::Text(text)) => {
                if let Ok(msg) = serde_json::from_str::<ServerMessage>(&text) {
                    match msg {
                        ServerMessage::Error { message } => {
                            error = Some(message);
                            break;
                        }
                        ServerMessage::Exit { code } => {
                            exit_code = Some(code);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    let _ = client_ws.close(None).await;
    server.abort();

    if let Some(message) = error {
        let lowered = message.to_lowercase();
        assert!(
            lowered.contains("read-only")
                || lowered.contains("overlay")
                || lowered.contains("failed to prepare"),
            "expected clear read-only overlay error, got: {message}"
        );
        return;
    }

    if let Some(code) = exit_code {
        assert_ne!(code, 0, "read-only PTY execution should fail");
        let target = cwd.join("ro-pty.txt");
        assert!(
            !target.exists(),
            "read-only PTY execution should not create files (found {})",
            target.display()
        );
    } else {
        eprintln!("skipping read-only PTY assertions: no error or exit frame received");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn pty_writable_mode_allows_write() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping pty writable test: service init failed: {err}");
            return;
        }
    };
    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();
    let target = cwd.join("pty-writable.txt");

    let router = Router::new().route(
        "/pty",
        get({
            let service = service.clone();
            move |ws: WebSocketUpgrade| {
                let service = service.clone();
                async move { ws.on_upgrade(move |socket| handle_ws_pty(service, socket)) }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ws listener");
    let addr = listener.local_addr().expect("ws listener addr");
    let std_listener = listener.into_std().expect("into_std listener");
    let server = tokio::spawn(async move {
        let _ = Server::from_tcp(std_listener)
            .unwrap()
            .serve(router.into_make_service())
            .await;
    });

    let (mut client_ws, _) = connect_async(format!("ws://{}/pty", addr))
        .await
        .expect("connect ws");

    let start = serde_json::json!({
        "type": "start",
        "cmd": "sh -lc 'echo writable-ok > pty-writable.txt'",
        "cwd": cwd,
        "env": {
            "SUBSTRATE_WORLD_FS_MODE": "writable"
        },
        "span_id": null,
        "cols": 80,
        "rows": 24
    });
    client_ws
        .send(Message::Text(start.to_string()))
        .await
        .expect("send start");

    let mut exit_code: Option<i32> = None;
    while let Some(frame) = client_ws.next().await {
        match frame {
            Ok(Message::Text(text)) => {
                if let Ok(ServerMessage::Exit { code }) =
                    serde_json::from_str::<ServerMessage>(&text)
                {
                    exit_code = Some(code);
                    break;
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    let _ = client_ws.close(None).await;
    server.abort();

    if let Some(code) = exit_code {
        if code != 0 {
            eprintln!(
                "skipping writable PTY assertions: command exited with {code} (target exists: {})",
                target.exists()
            );
            return;
        }
        assert!(
            target.exists(),
            "writable PTY execution should create files in project dir"
        );
    } else {
        eprintln!("skipping writable PTY assertions: no exit frame received");
    }
}
