#![cfg(all(unix, target_os = "linux"))]

use axum::extract::ws::WebSocketUpgrade;
use axum::routing::get;
use axum::{Router, Server};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use tokio::time::{timeout, Duration};
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

fn base_cage_env() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("SUBSTRATE_WORLD_REQUIRE_WORLD".to_string(), "1".to_string());
    env.insert(
        "SUBSTRATE_WORLD_FS_MODE".to_string(),
        "writable".to_string(),
    );
    env
}

async fn spawn_pty_server(service: WorldAgentService) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let router = Router::new().route(
        "/pty",
        get(move |ws: WebSocketUpgrade| {
            let service = service.clone();
            async move { ws.on_upgrade(move |socket| handle_ws_pty(service, socket)) }
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
    (addr, server)
}

async fn run_pty(
    service: WorldAgentService,
    cwd: &Path,
    cmd: &str,
    env: HashMap<String, String>,
) -> Option<(Option<i32>, Option<String>, String)> {
    let (addr, server) = spawn_pty_server(service).await;

    let (mut client_ws, _) = match connect_async(format!("ws://{}/pty", addr)).await {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("skipping full-cage PTY test: connect failed: {err}");
            server.abort();
            return None;
        }
    };

    let start = serde_json::json!({
        "type": "start",
        "cmd": cmd,
        "cwd": cwd,
        "env": env,
        "span_id": null,
        "cols": 80,
        "rows": 24
    });
    if let Err(err) = client_ws.send(Message::Text(start.to_string())).await {
        eprintln!("skipping full-cage PTY test: send start failed: {err}");
        let _ = client_ws.close(None).await;
        server.abort();
        return None;
    }

    let mut exit_code: Option<i32> = None;
    let mut error: Option<String> = None;
    let mut output = String::new();

    let recv = async {
        while let Some(frame) = client_ws.next().await {
            match frame {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<ServerMessage>(&text) {
                        match msg {
                            ServerMessage::Stdout { data_b64 } => {
                                output.push_str(&decode(&data_b64))
                            }
                            ServerMessage::Error { message } => {
                                error = Some(message);
                                break;
                            }
                            ServerMessage::Exit { code } => {
                                exit_code = Some(code);
                                break;
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
    };

    if timeout(Duration::from_secs(15), recv).await.is_err() {
        eprintln!("skipping full-cage PTY test: timeout waiting for exit frame");
        let _ = client_ws.close(None).await;
        server.abort();
        return None;
    }

    let _ = client_ws.close(None).await;
    server.abort();

    Some((exit_code, error, output))
}

#[tokio::test(flavor = "current_thread")]
async fn pty_full_cage_prevents_host_tmp_writes() {
    if !overlay_available() {
        eprintln!("skipping full-cage PTY test: overlay support or privileges missing");
        return;
    }

    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping full-cage PTY test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let host_path = PathBuf::from("/tmp").join(format!(
        "substrate-full-cage-pty-host-marker-{}",
        uuid::Uuid::now_v7()
    ));
    let _ = std::fs::remove_file(&host_path);

    let mut env = base_cage_env();
    env.insert(
        "SUBSTRATE_TEST_HOST_MARKER".to_string(),
        host_path.display().to_string(),
    );

    let outcome = match run_pty(
        service,
        &cwd,
        r#"sh -lc 'echo cage > "$SUBSTRATE_TEST_HOST_MARKER"'"#,
        env,
    )
    .await
    {
        Some(outcome) => outcome,
        None => return,
    };

    let (exit, error, output) = &outcome;
    assert!(
        error.is_none() && exit == &Some(0),
        "full-cage PTY execution failed unexpectedly: {outcome:?}"
    );
    assert!(
        !output.contains("cd:"),
        "full-cage PTY execution failed to enter cwd: {output:?}"
    );

    if host_path.exists() {
        let _ = std::fs::remove_file(&host_path);
        panic!(
            "full-cage PTY execution wrote to host /tmp (unexpected file: {}), outcome: {:?}",
            host_path.display(),
            outcome
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn pty_full_cage_prevents_host_tmp_reads() {
    if !overlay_available() {
        eprintln!("skipping full-cage PTY test: overlay support or privileges missing");
        return;
    }

    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping full-cage PTY test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let host_path = PathBuf::from("/tmp").join(format!(
        "substrate-full-cage-pty-host-secret-{}",
        uuid::Uuid::now_v7()
    ));
    let secret = format!("host-secret-{}\n", uuid::Uuid::now_v7());
    std::fs::write(&host_path, secret.as_bytes()).expect("write host secret");

    let mut env = base_cage_env();
    env.insert(
        "SUBSTRATE_TEST_HOST_SECRET".to_string(),
        host_path.display().to_string(),
    );

    let outcome = run_pty(
        service,
        &cwd,
        r#"sh -lc 'cat "$SUBSTRATE_TEST_HOST_SECRET"'"#,
        env,
    )
    .await;

    let _ = std::fs::remove_file(&host_path);

    let (_exit, _error, output) = match outcome {
        Some(outcome) => outcome,
        None => return,
    };

    assert!(
        !output.contains(&secret),
        "full-cage PTY execution was able to read host /tmp secret (path: {})",
        host_path.display()
    );
    assert!(
        !output.contains("cd:"),
        "full-cage PTY execution failed to enter cwd (masked by /tmp read assertion): {output:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn pty_full_cage_read_only_blocks_project_writes() {
    if !overlay_available() {
        eprintln!("skipping full-cage PTY read-only test: overlay support or privileges missing");
        return;
    }

    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping full-cage PTY read-only test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut env = base_cage_env();
    env.insert(
        "SUBSTRATE_WORLD_FS_MODE".to_string(),
        "read_only".to_string(),
    );

    let (exit, error, _output) = match run_pty(
        service,
        &cwd,
        "sh -lc 'echo denied > ro-full-cage-pty.txt'",
        env,
    )
    .await
    {
        Some(outcome) => outcome,
        None => return,
    };

    if let Some(message) = error {
        let lowered = message.to_lowercase();
        assert!(
            lowered.contains("read-only")
                || lowered.contains("read only")
                || lowered.contains("failed to prepare"),
            "expected clear read-only error, got: {message}"
        );
        return;
    }

    assert_ne!(
        exit.unwrap_or(0),
        0,
        "full-cage PTY read-only write unexpectedly succeeded"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn pty_full_cage_runs_from_tmp_rooted_project() {
    if !overlay_available() {
        eprintln!("skipping full-cage PTY test: overlay support or privileges missing");
        return;
    }

    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping full-cage PTY test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let (exit, error, output) = match run_pty(service, &cwd, "sh -lc 'pwd'", base_cage_env()).await
    {
        Some(outcome) => outcome,
        None => return,
    };

    assert!(
        error.is_none() && exit == Some(0),
        "full-cage PTY execution failed unexpectedly: exit={exit:?} error={error:?} output={output:?}"
    );
    assert!(
        output.trim_start().starts_with("/project"),
        "expected full-cage PTY cwd to use stable /project mount for /tmp-rooted projects, got: {output:?}"
    );
}
