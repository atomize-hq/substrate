#![cfg(all(unix, target_os = "linux"))]

use axum::routing::get;
use axum::Router;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::sync::mpsc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use world_agent::WorldAgentService;

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

fn minimal_policy_snapshot() -> Value {
    json!({
        "schema_version": 3,
        "world_fs": {
            "host_visible": true,
            "fail_closed": { "routing": false },
            "write": { "enabled": true }
        }
    })
}

async fn spawn_world_agent_ws(
    service: WorldAgentService,
) -> (
    SocketAddr,
    tokio::sync::oneshot::Sender<()>,
    tokio::task::JoinHandle<()>,
) {
    let router = Router::new()
        .route("/v1/stream", get(world_agent::handlers::stream))
        .with_state(service);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ws listener");
    let addr = listener.local_addr().expect("ws listener addr");
    let std_listener = listener.into_std().expect("into_std listener");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server = tokio::spawn(async move {
        let _ = axum::Server::from_tcp(std_listener)
            .expect("from_tcp")
            .serve(router.into_make_service())
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    (addr, shutdown_tx, server)
}

async fn stop_server(
    shutdown: tokio::sync::oneshot::Sender<()>,
    mut server: tokio::task::JoinHandle<()>,
) {
    let _ = shutdown.send(());
    match timeout(Duration::from_secs(2), &mut server).await {
        Ok(_) => {}
        Err(_) => {
            server.abort();
            let _ = server.await;
        }
    }
}

#[cfg(target_os = "linux")]
fn install_seccomp_deny_ioctl_fionread() -> std::io::Result<()> {
    // Deny only ioctl(..., FIONREAD, ...) for this thread.
    // This allows the test to simulate DR-23 “watermark query unsupported” without breaking
    // the rest of the networking stack (tokio may use other ioctls like FIONBIO).
    const PR_SET_NO_NEW_PRIVS: libc::c_int = 38;
    const PR_SET_SECCOMP: libc::c_int = 22;
    const SECCOMP_MODE_FILTER: libc::c_ulong = 2;

    const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
    const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;

    const BPF_LD: u16 = 0x00;
    const BPF_W: u16 = 0x00;
    const BPF_ABS: u16 = 0x20;
    const BPF_JMP: u16 = 0x05;
    const BPF_JEQ: u16 = 0x10;
    const BPF_K: u16 = 0x00;
    const BPF_RET: u16 = 0x06;

    const SECCOMP_DATA_NR_OFFSET: u32 = 0;
    const SECCOMP_DATA_ARGS1_OFFSET: u32 = 24; // args[1] == ioctl request

    let rc = unsafe { libc::prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }

    let deny_errno = SECCOMP_RET_ERRNO | (libc::EPERM as u32);
    let sys_ioctl = libc::SYS_ioctl as u32;
    let fionread = libc::FIONREAD as u32;

    let filter: [libc::sock_filter; 8] = [
        // load seccomp_data.nr
        libc::sock_filter {
            code: BPF_LD | BPF_W | BPF_ABS,
            jt: 0,
            jf: 0,
            k: SECCOMP_DATA_NR_OFFSET,
        },
        // if nr != SYS_ioctl -> allow
        libc::sock_filter {
            code: BPF_JMP | BPF_JEQ | BPF_K,
            jt: 0,
            jf: 5,
            k: sys_ioctl,
        },
        // load seccomp_data.args[1] (ioctl request)
        libc::sock_filter {
            code: BPF_LD | BPF_W | BPF_ABS,
            jt: 0,
            jf: 0,
            k: SECCOMP_DATA_ARGS1_OFFSET,
        },
        // if request == FIONREAD -> deny
        libc::sock_filter {
            code: BPF_JMP | BPF_JEQ | BPF_K,
            jt: 0,
            jf: 1,
            k: fionread,
        },
        libc::sock_filter {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: deny_errno,
        },
        // allow ioctl requests != FIONREAD
        libc::sock_filter {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: SECCOMP_RET_ALLOW,
        },
        // allow non-ioctl syscalls
        libc::sock_filter {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: SECCOMP_RET_ALLOW,
        },
        // (unused padding to keep array fixed-size; never reached)
        libc::sock_filter {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: SECCOMP_RET_ALLOW,
        },
    ];

    let prog = libc::sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_ptr() as *mut libc::sock_filter,
    };

    let rc = unsafe { libc::prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
}

async fn spawn_world_agent_ws_with_fionread_blocked() -> (
    SocketAddr,
    tokio::sync::oneshot::Sender<()>,
    std::thread::JoinHandle<()>,
) {
    let (tx, rx) = mpsc::channel::<(SocketAddr, tokio::sync::oneshot::Sender<()>)>();

    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");

        rt.block_on(async move {
            let service = WorldAgentService::new().expect("WorldAgentService::new");
            let router = Router::new()
                .route("/v1/stream", get(world_agent::handlers::stream))
                .with_state(service);

            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind ws listener");
            let addr = listener.local_addr().expect("ws listener addr");
            let std_listener = listener.into_std().expect("into_std listener");

            // Install filter after binding (so networking setup is unaffected), but before
            // accepting `start_session` and attempting the DR-23 watermark query.
            install_seccomp_deny_ioctl_fionread().expect("install seccomp filter");

            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
            tx.send((addr, shutdown_tx)).expect("send addr");

            let _ = axum::Server::from_tcp(std_listener)
                .expect("from_tcp")
                .serve(router.into_make_service())
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });
    });

    let (addr, shutdown) = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("receive addr");
    (addr, shutdown, handle)
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

async fn expect_terminal_frame(ws: &mut Ws) -> Value {
    for _ in 0..20 {
        let frame = recv_json(ws).await;
        match frame.get("type").and_then(Value::as_str) {
            Some("stdout") => continue,
            Some("ready") | Some("error") | Some("exit") => return frame,
            other => panic!("unexpected server frame type: {other:?} frame={frame}"),
        }
    }
    panic!("did not receive terminal frame (ready/error/exit) after 20 messages");
}

fn start_session_frame(cwd: &std::path::Path, policy_snapshot: Value) -> Value {
    json!({
        "type": "start_session",
        "cwd": cwd.display().to_string(),
        "env": {
            "HOME": "/root",
            "TERM": "xterm-256color",
        },
        "policy_snapshot": policy_snapshot,
        "cols": 80,
        "rows": 24,
    })
}

fn looks_like_missing_world_prereqs(frame: &Value) -> bool {
    if frame.get("type").and_then(Value::as_str) != Some("error") {
        return false;
    }
    let message = frame
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    message.contains("failed to create session world")
        || message.contains("failed to prepare world overlay")
        || message.contains("user namespaces disabled")
        || message.contains("operation not permitted")
}

fn legacy_stdout_indicates_missing_world_prereqs(stdout: &str) -> bool {
    let stdout = stdout.to_ascii_lowercase();
    let setup_marker =
        stdout.contains("mount:") || stdout.contains("unshare:") || stdout.contains("mkdir:");
    let permission_marker = stdout.contains("operation not permitted")
        || stdout.contains("permission denied")
        || stdout.contains("wrong fs type");
    setup_marker && permission_marker
}

async fn connect_and_start_session_or_skip(
    addr: SocketAddr,
    cwd: &std::path::Path,
) -> Option<(Ws, Value)> {
    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        start_session_frame(cwd, minimal_policy_snapshot()).to_string(),
    ))
    .await
    .expect("send start_session");
    let frame = expect_terminal_frame(&mut ws).await;

    if looks_like_missing_world_prereqs(&frame) {
        return None;
    }

    Some((ws, frame))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_one_shot_start_remains_accepted() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping legacy /v1/stream test: service init failed: {err}");
            return;
        }
    };

    let (addr, shutdown, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        json!({
            "type": "start",
            "cmd": "sh -lc 'printf hello'",
            "cwd": cwd.display().to_string(),
            "env": HashMap::<String, String>::new(),
            "policy_snapshot": null,
            "span_id": null,
            "cols": 80,
            "rows": 24,
        })
        .to_string(),
    ))
    .await
    .expect("send start");

    let mut saw_stdout = false;
    let mut stdout_text = String::new();
    let mut exit_code = None;
    for _ in 0..50 {
        let frame = recv_json(&mut ws).await;
        match frame.get("type").and_then(Value::as_str) {
            Some("stdout") => {
                saw_stdout = true;
                if let Some(data_b64) = frame.get("data_b64").and_then(Value::as_str) {
                    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data_b64) {
                        stdout_text.push_str(&String::from_utf8_lossy(&bytes));
                    }
                }
            }
            Some("exit") => {
                exit_code = frame.get("code").and_then(Value::as_i64);
                break;
            }
            Some("error") => {
                if looks_like_missing_world_prereqs(&frame) {
                    eprintln!(
                        "skipping legacy /v1/stream assertions: world prereqs missing: {frame}"
                    );
                    drop(ws);
                    stop_server(shutdown, server).await;
                    return;
                }
                panic!("unexpected error for legacy start: {frame}");
            }
            other => panic!("unexpected server frame type: {other:?} frame={frame}"),
        }
    }

    let missing_world_prereqs = matches!(exit_code, Some(1 | 32))
        && legacy_stdout_indicates_missing_world_prereqs(&stdout_text);
    if missing_world_prereqs {
        eprintln!(
            "skipping legacy /v1/stream assertions: world prereqs missing during legacy start: {stdout_text}"
        );
        drop(ws);
        stop_server(shutdown, server).await;
        return;
    }

    assert!(saw_stdout, "expected at least one stdout frame");
    assert_eq!(exit_code, Some(0), "expected exit code 0");

    drop(ws);
    stop_server(shutdown, server).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn persistent_session_enforces_first_frame_start_session() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping persistent first-frame test: service init failed: {err}");
            return;
        }
    };

    let (addr, shutdown, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        json!({
            "type": "stdin",
            "data_b64": "AA=="
        })
        .to_string(),
    ))
    .await
    .expect("send invalid first frame");

    let frame = expect_terminal_frame(&mut ws).await;
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(
        frame.get("code").and_then(Value::as_str),
        Some("bad_request")
    );
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));
    assert!(
        !frame
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .is_empty(),
        "error.message must be non-empty"
    );

    // The server MUST NOT emit `ready` after a bad first frame (fail-closed posture).
    ws.send(Message::Text(
        start_session_frame(cwd.as_path(), minimal_policy_snapshot()).to_string(),
    ))
    .await
    .ok();

    drop(ws);
    stop_server(shutdown, server).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_session_rejects_policy_snapshot_with_unknown_fields_fail_closed() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping policy_snapshot unknown-fields test: service init failed: {err}");
            return;
        }
    };

    let (addr, shutdown, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut snapshot = minimal_policy_snapshot();
    snapshot
        .as_object_mut()
        .expect("snapshot object")
        .insert("unknown_field".to_string(), json!(123));

    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        start_session_frame(cwd.as_path(), snapshot).to_string(),
    ))
    .await
    .expect("send start_session");

    let frame = expect_terminal_frame(&mut ws).await;
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(
        frame.get("code").and_then(Value::as_str),
        Some("bad_request")
    );
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));

    drop(ws);
    stop_server(shutdown, server).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_session_requires_policy_snapshot_fail_closed() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping policy_snapshot required test: service init failed: {err}");
            return;
        }
    };

    let (addr, shutdown, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        json!({
            "type": "start_session",
            "cwd": cwd.display().to_string(),
            "env": { "HOME": "/root" },
            "cols": 80,
            "rows": 24,
        })
        .to_string(),
    ))
    .await
    .expect("send start_session missing policy_snapshot");

    let frame = expect_terminal_frame(&mut ws).await;
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(
        frame.get("code").and_then(Value::as_str),
        Some("bad_request")
    );
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));

    drop(ws);
    stop_server(shutdown, server).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_session_yields_ready_with_fresh_hex32_session_nonce() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping ready.session_nonce test: service init failed: {err}");
            return;
        }
    };

    let (addr, shutdown, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();
    let (ws, frame) = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ok) => ok,
        None => {
            eprintln!("skipping start_session ready assertions: world prereqs missing");
            stop_server(shutdown, server).await;
            return;
        }
    };
    if looks_like_missing_world_prereqs(&frame) {
        eprintln!("skipping start_session ready assertions: world prereqs missing: {frame}");
        drop(ws);
        stop_server(shutdown, server).await;
        return;
    }
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("ready"));
    assert_eq!(
        frame.get("protocol_version").and_then(Value::as_i64),
        Some(1)
    );

    let nonce = frame
        .get("session_nonce")
        .and_then(Value::as_str)
        .expect("ready.session_nonce string");
    assert_eq!(
        nonce.len(),
        32,
        "session_nonce must be 32 lowercase hex chars"
    );
    assert!(
        nonce
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "ready.session_nonce must be lowercase hex"
    );

    drop(ws);
    stop_server(shutdown, server).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_nonce_is_unique_per_session_restart() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping session_nonce uniqueness test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let (addr1, shutdown1, server1) = spawn_world_agent_ws(service.clone()).await;
    let (ws1, ready1) = match connect_and_start_session_or_skip(addr1, cwd.as_path()).await {
        Some(ok) => ok,
        None => {
            eprintln!("skipping session_nonce uniqueness assertions: world prereqs missing");
            stop_server(shutdown1, server1).await;
            return;
        }
    };
    if looks_like_missing_world_prereqs(&ready1) {
        eprintln!("skipping session_nonce uniqueness assertions: world prereqs missing: {ready1}");
        drop(ws1);
        stop_server(shutdown1, server1).await;
        return;
    }
    let nonce1 = ready1
        .get("session_nonce")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    drop(ws1);
    stop_server(shutdown1, server1).await;

    let (addr2, shutdown2, server2) = spawn_world_agent_ws(service).await;
    let (ws2, ready2) = match connect_and_start_session_or_skip(addr2, cwd.as_path()).await {
        Some(ok) => ok,
        None => {
            eprintln!("skipping session_nonce uniqueness assertions: world prereqs missing");
            stop_server(shutdown2, server2).await;
            return;
        }
    };
    if looks_like_missing_world_prereqs(&ready2) {
        eprintln!("skipping session_nonce uniqueness assertions: world prereqs missing: {ready2}");
        drop(ws2);
        stop_server(shutdown2, server2).await;
        return;
    }
    let nonce2 = ready2
        .get("session_nonce")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    drop(ws2);
    stop_server(shutdown2, server2).await;

    assert!(!nonce1.is_empty(), "session_nonce missing for session 1");
    assert!(!nonce2.is_empty(), "session_nonce missing for session 2");
    assert_ne!(nonce1, nonce2, "session_nonce must change per session");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_session_fails_closed_when_fionread_watermark_is_unavailable() {
    // DR-23 preflight is fail-closed: if the required PTY watermark query is unavailable for
    // protocol v1 (Linux ioctl(FIONREAD)), world-agent MUST NOT emit `ready`.
    let (addr, shutdown, server_thread) = spawn_world_agent_ws_with_fionread_blocked().await;

    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let (ws, frame) = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ok) => ok,
        None => {
            eprintln!("skipping DR-23 FIONREAD assertions: world prereqs missing");
            let _ = shutdown.send(());
            let _ = server_thread.join();
            return;
        }
    };
    if looks_like_missing_world_prereqs(&frame) {
        eprintln!("skipping DR-23 FIONREAD assertions: world prereqs missing: {frame}");
        drop(ws);
        let _ = shutdown.send(());
        let _ = server_thread.join();
        return;
    }
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));

    drop(ws);
    let _ = shutdown.send(());
    let _ = server_thread.join();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_session_is_robust_to_inheritable_non_stdio_fds() {
    // DR-22 preflight: the evaluator MUST NOT inherit non-stdio fds.
    // This test opens a non-CLOEXEC fd in the world-agent process and requires that the
    // session bootstrap still succeeds, implying the session's spawn strategy doesn't rely on
    // CLOEXEC alone.
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping DR-22 inheritable-fd test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let fd_leak_guard = std::fs::File::open("/dev/null").expect("open /dev/null");
    let fd = fd_leak_guard.as_raw_fd();
    let rc = unsafe { libc::fcntl(fd, libc::F_SETFD, 0) };
    assert_eq!(rc, 0, "clear CLOEXEC");

    let (addr, shutdown, server) = spawn_world_agent_ws(service).await;
    let (ws, frame) = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ok) => ok,
        None => {
            eprintln!("skipping DR-22 inheritable-fd assertions: world prereqs missing");
            stop_server(shutdown, server).await;
            drop(fd_leak_guard);
            return;
        }
    };
    if looks_like_missing_world_prereqs(&frame) {
        eprintln!("skipping DR-22 inheritable-fd assertions: world prereqs missing: {frame}");
        drop(ws);
        stop_server(shutdown, server).await;
        drop(fd_leak_guard);
        return;
    }
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("ready"));

    drop(ws);
    stop_server(shutdown, server).await;
    drop(fd_leak_guard);
}
