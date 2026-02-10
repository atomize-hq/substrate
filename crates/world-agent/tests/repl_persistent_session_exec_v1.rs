#![cfg(all(unix, target_os = "linux"))]

use axum::routing::get;
use axum::Router;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::{sleep, timeout};
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

fn start_session_frame(
    cwd: &std::path::Path,
    policy_snapshot: Value,
    env: HashMap<String, String>,
) -> Value {
    json!({
        "type": "start_session",
        "cwd": cwd.display().to_string(),
        "env": env,
        "policy_snapshot": policy_snapshot,
        "cols": 80,
        "rows": 24,
    })
}

fn token_hex_for_seq(seq: u64) -> String {
    format!("{seq:032x}")
}

fn exec_frame(seq: u64, stdin_mode: &str, program_bytes: &[u8]) -> Value {
    json!({
        "type": "exec",
        "seq": seq,
        "token_hex": token_hex_for_seq(seq),
        "cmd_id": uuid::Uuid::now_v7().to_string(),
        "stdin_mode": stdin_mode,
        "program_b64": BASE64.encode(program_bytes),
    })
}

fn exec_frame_with_raw_program_b64(seq: u64, stdin_mode: &str, program_b64: &str) -> Value {
    json!({
        "type": "exec",
        "seq": seq,
        "token_hex": token_hex_for_seq(seq),
        "cmd_id": uuid::Uuid::now_v7().to_string(),
        "stdin_mode": stdin_mode,
        "program_b64": program_b64,
    })
}

async fn connect_and_start_session_or_skip(addr: SocketAddr, cwd: &std::path::Path) -> Option<Ws> {
    let mut env = HashMap::<String, String>::new();
    env.insert("HOME".to_string(), "/root".to_string());
    env.insert("TERM".to_string(), "xterm-256color".to_string());

    connect_and_start_session_with_env_or_skip(addr, cwd, env).await
}

async fn connect_and_start_session_with_env_or_skip(
    addr: SocketAddr,
    cwd: &std::path::Path,
    env: HashMap<String, String>,
) -> Option<Ws> {
    let mut ws = ws_connect(addr).await;
    ws.send(Message::Text(
        start_session_frame(cwd, minimal_policy_snapshot(), env.clone()).to_string(),
    ))
    .await
    .expect("send start_session");
    let frame = recv_json(&mut ws).await;

    if looks_like_missing_world_prereqs(&frame) {
        eprintln!("skipping persistent exec tests: world prereqs missing: {frame}");
        return None;
    }
    if frame.get("type").and_then(Value::as_str) == Some("ready") {
        return Some(ws);
    }

    panic!("unexpected terminal frame for start_session: {frame}");
}

async fn collect_until_completion(ws: &mut Ws) -> (Vec<u8>, Value) {
    let mut stdout = Vec::<u8>::new();
    for _ in 0..200 {
        let frame = recv_json(ws).await;
        match frame.get("type").and_then(Value::as_str) {
            Some("stdout") => {
                let b64 = frame
                    .get("data_b64")
                    .and_then(Value::as_str)
                    .expect("stdout frame has data_b64");
                let bytes = BASE64.decode(b64).expect("stdout.data_b64 decodes");
                stdout.extend_from_slice(&bytes);
            }
            Some("command_complete") => return (stdout, frame),
            Some("error") => panic!("unexpected error while awaiting completion: {frame}"),
            Some(other) => panic!("unexpected server frame type: {other:?} frame={frame}"),
            None => panic!("server frame missing type: {frame}"),
        }
    }
    panic!("did not observe command_complete within 200 frames");
}

async fn recv_for_duration(ws: &mut Ws, duration: Duration) -> Vec<Value> {
    let deadline = tokio::time::Instant::now() + duration;
    let mut frames = Vec::new();
    loop {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            break;
        }
        let remaining = deadline - now;
        let msg = match timeout(remaining, ws.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(_))) => break,
            Ok(None) => break,
            Err(_) => break,
        };
        if let Message::Text(text) = msg {
            if let Ok(frame) = serde_json::from_str::<Value>(&text) {
                frames.push(frame);
            }
        }
    }
    frames
}

#[tokio::test(flavor = "current_thread")]
async fn exec_rejects_invalid_base64_fail_closed() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping exec invalid-base64 test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    ws.send(Message::Text(
        exec_frame_with_raw_program_b64(1, "eof", "not-base64!!!").to_string(),
    ))
    .await
    .expect("send exec");

    let frame = recv_json(&mut ws).await;
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(
        frame.get("code").and_then(Value::as_str),
        Some("bad_request")
    );
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));
    assert_eq!(frame.get("seq").and_then(Value::as_u64), Some(1));

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn exec_rejects_invalid_utf8_fail_closed() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping exec invalid-utf8 test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    let invalid = [0xffu8, 0xfeu8, 0xfdu8];
    ws.send(Message::Text(exec_frame(1, "eof", &invalid).to_string()))
        .await
        .expect("send exec");

    let frame = recv_json(&mut ws).await;
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(
        frame.get("code").and_then(Value::as_str),
        Some("program_invalid_utf8")
    );
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));
    assert_eq!(frame.get("seq").and_then(Value::as_u64), Some(1));

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn exec_rejects_nul_fail_closed() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping exec NUL test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    let program = b"echo ok\0echo bad";
    ws.send(Message::Text(exec_frame(1, "eof", program).to_string()))
        .await
        .expect("send exec");

    let frame = recv_json(&mut ws).await;
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(
        frame.get("code").and_then(Value::as_str),
        Some("program_contains_nul")
    );
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));
    assert_eq!(frame.get("seq").and_then(Value::as_u64), Some(1));

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn exec_while_busy_is_fatal_protocol_error() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping exec-while-busy test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    ws.send(Message::Text(
        exec_frame(1, "passthrough", br#"read -r line; echo "GOT:$line""#).to_string(),
    ))
    .await
    .expect("send exec 1");

    sleep(Duration::from_millis(100)).await;

    ws.send(Message::Text(exec_frame(2, "eof", b"echo ok").to_string()))
        .await
        .expect("send exec 2 while exec 1 in-flight");

    let frame = recv_json(&mut ws).await;
    assert_eq!(frame.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(
        frame.get("code").and_then(Value::as_str),
        Some("exec_while_busy")
    );
    assert_eq!(frame.get("fatal").and_then(Value::as_bool), Some(true));
    assert_eq!(frame.get("seq").and_then(Value::as_u64), Some(2));

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn stdout_is_drained_before_command_complete() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping output ordering test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    let marker = format!("ENDMARK-{}\n", uuid::Uuid::now_v7());
    let program = format!(r#"for i in $(seq 1 5000); do printf X; done; printf "{marker}""#);
    ws.send(Message::Text(
        exec_frame(1, "eof", program.as_bytes()).to_string(),
    ))
    .await
    .expect("send exec");

    let (stdout, complete) = collect_until_completion(&mut ws).await;
    assert_eq!(
        complete.get("type").and_then(Value::as_str),
        Some("command_complete")
    );
    assert_eq!(complete.get("seq").and_then(Value::as_u64), Some(1));
    assert_eq!(
        complete.get("token_hex").and_then(Value::as_str),
        Some(token_hex_for_seq(1).as_str())
    );
    assert_eq!(complete.get("exit").and_then(Value::as_i64), Some(0));
    assert!(
        complete
            .get("cwd")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .starts_with('/'),
        "command_complete.cwd must be absolute: {complete}"
    );

    let stdout_str = String::from_utf8_lossy(&stdout);
    assert!(
        stdout_str.contains(&marker),
        "expected marker in stdout before command_complete; marker={marker:?}"
    );

    let post_frames = recv_for_duration(&mut ws, Duration::from_millis(200)).await;
    for frame in post_frames {
        if frame.get("type").and_then(Value::as_str) != Some("stdout") {
            continue;
        }
        let b64 = frame
            .get("data_b64")
            .and_then(Value::as_str)
            .expect("stdout frame has data_b64");
        let bytes = BASE64.decode(b64).expect("stdout.data_b64 decodes");
        let s = String::from_utf8_lossy(&bytes);
        assert!(
            !s.contains(&marker),
            "saw marker bytes after command_complete (DR-23 violation): frame={frame}"
        );
    }

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn stdin_is_dropped_unless_passthrough_and_never_leaks_across_commands() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping stdin gating test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    // While idle, stdin MUST be dropped.
    ws.send(Message::Text(
        json!({"type":"stdin","data_b64": BASE64.encode(b"idle-leak\n")}).to_string(),
    ))
    .await
    .expect("send idle stdin");

    ws.send(Message::Text(
        exec_frame(
            1,
            "passthrough",
            br#"read -t 0.2 -r line && echo "LEAK:$line" || echo "NOLEAK_IDLE""#,
        )
        .to_string(),
    ))
    .await
    .expect("send exec 1");
    let (stdout1, _) = collect_until_completion(&mut ws).await;
    let s1 = String::from_utf8_lossy(&stdout1);
    assert!(
        s1.contains("NOLEAK_IDLE"),
        "expected idle stdin to be dropped, got stdout: {s1:?}"
    );

    // After completion, stdin MUST be dropped until a new passthrough command begins.
    ws.send(Message::Text(
        json!({"type":"stdin","data_b64": BASE64.encode(b"late-leak\n")}).to_string(),
    ))
    .await
    .expect("send late stdin");

    ws.send(Message::Text(
        exec_frame(
            2,
            "passthrough",
            br#"read -t 0.2 -r line && echo "LEAK2:$line" || echo "NOLEAK_LATE""#,
        )
        .to_string(),
    ))
    .await
    .expect("send exec 2");
    let (stdout2, _) = collect_until_completion(&mut ws).await;
    let s2 = String::from_utf8_lossy(&stdout2);
    assert!(
        s2.contains("NOLEAK_LATE"),
        "expected post-completion stdin to be dropped, got stdout: {s2:?}"
    );

    // While stdin_mode=eof, stdin MUST be dropped (and not buffered into the PTY for later).
    ws.send(Message::Text(
        exec_frame(3, "eof", b"sleep 0.2; echo DONE_EOF").to_string(),
    ))
    .await
    .expect("send exec 3");
    sleep(Duration::from_millis(50)).await;
    ws.send(Message::Text(
        json!({"type":"stdin","data_b64": BASE64.encode(b"eof-leak\n")}).to_string(),
    ))
    .await
    .expect("send stdin during eof");
    let (stdout3, _) = collect_until_completion(&mut ws).await;
    let s3 = String::from_utf8_lossy(&stdout3);
    assert!(s3.contains("DONE_EOF"), "expected exec to complete");

    ws.send(Message::Text(
        exec_frame(
            4,
            "passthrough",
            br#"read -t 0.2 -r line && echo "LEAK3:$line" || echo "NOLEAK_EOF""#,
        )
        .to_string(),
    ))
    .await
    .expect("send exec 4");
    let (stdout4, _) = collect_until_completion(&mut ws).await;
    let s4 = String::from_utf8_lossy(&stdout4);
    assert!(
        s4.contains("NOLEAK_EOF"),
        "expected stdin during eof to be dropped (no buffering), got stdout: {s4:?}"
    );

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn signal_targets_foreground_process_group_and_session_survives() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping signal targeting test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    ws.send(Message::Text(exec_frame(1, "eof", b"sleep 10").to_string()))
        .await
        .expect("send exec");
    sleep(Duration::from_millis(150)).await;

    ws.send(Message::Text(
        json!({"type":"signal","sig":"INT"}).to_string(),
    ))
    .await
    .expect("send signal INT");

    let (_stdout, complete) = collect_until_completion(&mut ws).await;
    assert_eq!(complete.get("seq").and_then(Value::as_u64), Some(1));
    assert_eq!(
        complete.get("token_hex").and_then(Value::as_str),
        Some(token_hex_for_seq(1).as_str())
    );
    assert_eq!(
        complete.get("exit").and_then(Value::as_i64),
        Some(130),
        "SIGINT exit should follow bash conventions (128+2)"
    );

    ws.send(Message::Text(
        exec_frame(2, "eof", b"echo STILL_OK").to_string(),
    ))
    .await
    .expect("send exec 2");
    let (stdout2, complete2) = collect_until_completion(&mut ws).await;
    assert_eq!(complete2.get("seq").and_then(Value::as_u64), Some(2));
    assert!(
        String::from_utf8_lossy(&stdout2).contains("STILL_OK"),
        "expected session to remain usable after SIGINT"
    );

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn persists_physical_cwd_and_exported_env_across_execs() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping persistence test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut env = HashMap::<String, String>::new();
    env.insert("HOME".to_string(), "/root".to_string());
    env.insert("TERM".to_string(), "xterm-256color".to_string());
    // Explicitly exercise uncaged traversal: this test depends on being able to persist a cwd
    // outside the project anchor between commands.
    env.insert("SUBSTRATE_CAGED".to_string(), "0".to_string());
    env.insert("SUBSTRATE_ANCHOR_MODE".to_string(), "workspace".to_string());

    let mut ws = match connect_and_start_session_with_env_or_skip(addr, cwd.as_path(), env).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    let id = uuid::Uuid::now_v7().to_string();
    let real = format!("/tmp/substrate-c1-real-{id}");
    let link = format!("/tmp/substrate-c1-link-{id}");
    let val = format!("bar-{id}");
    let program1 = format!(
        r#"set -euo pipefail
rm -rf "{real}" "{link}"
mkdir -p "{real}"
ln -s "{real}" "{link}"
cd "{link}"
export FOO="{val}"
echo SET_OK"#
    );

    ws.send(Message::Text(
        exec_frame(1, "eof", program1.as_bytes()).to_string(),
    ))
    .await
    .expect("send exec 1");
    let (stdout1, complete1) = collect_until_completion(&mut ws).await;
    assert!(
        String::from_utf8_lossy(&stdout1).contains("SET_OK"),
        "expected exec 1 to run"
    );

    let cwd1 = complete1
        .get("cwd")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert_eq!(
        cwd1, real,
        "command_complete.cwd must be physical (pwd -P): got={cwd1:?} expected={real:?}"
    );

    let program2 = r#"pwd -P; echo "$FOO""#;
    ws.send(Message::Text(
        exec_frame(2, "eof", program2.as_bytes()).to_string(),
    ))
    .await
    .expect("send exec 2");
    let (stdout2, complete2) = collect_until_completion(&mut ws).await;
    let s2 = String::from_utf8_lossy(&stdout2);
    assert!(s2.contains(&real), "expected persisted pwd -P, got: {s2:?}");
    assert!(s2.contains(&val), "expected persisted FOO, got: {s2:?}");

    let cwd2 = complete2
        .get("cwd")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert_eq!(cwd2, real, "expected persisted physical cwd");

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn caged_session_prevents_escape_from_anchor() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping caged-escape test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut env = HashMap::<String, String>::new();
    env.insert("HOME".to_string(), "/root".to_string());
    env.insert("TERM".to_string(), "xterm-256color".to_string());
    env.insert("SUBSTRATE_CAGED".to_string(), "1".to_string());
    env.insert("SUBSTRATE_ANCHOR_MODE".to_string(), "workspace".to_string());

    let mut ws = match connect_and_start_session_with_env_or_skip(addr, cwd.as_path(), env).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    ws.send(Message::Text(
        exec_frame(1, "eof", b"cd ..; pwd -P").to_string(),
    ))
    .await
    .expect("send exec 1");
    let (stdout1, complete1) = collect_until_completion(&mut ws).await;

    let out = String::from_utf8_lossy(&stdout1);
    let cwd1 = complete1
        .get("cwd")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    assert!(
        cwd1.starts_with(cwd.to_string_lossy().as_ref()),
        "expected caged session to stay under anchor; got command_complete.cwd={cwd1:?} anchor={:?} stdout={out:?}",
        cwd.display()
    );

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn evaluator_is_bash_noprofile_norc_and_prompts_suppressed() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping evaluator shell test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    let program = r#"set -euo pipefail
ps -o args= -p $$
test -z "${PS1-}" && echo PS1_EMPTY
test -z "${PS2-}" && echo PS2_EMPTY
test -z "${PROMPT_COMMAND-}" && echo PROMPT_COMMAND_EMPTY"#;

    ws.send(Message::Text(
        exec_frame(1, "eof", program.as_bytes()).to_string(),
    ))
    .await
    .expect("send exec");
    let (stdout, _complete) = collect_until_completion(&mut ws).await;
    let s = String::from_utf8_lossy(&stdout);
    assert!(
        s.contains("/bin/bash") && s.contains("--noprofile") && s.contains("--norc"),
        "expected evaluator shell to be /bin/bash --noprofile --norc, got: {s:?}"
    );
    assert!(
        s.contains("PS1_EMPTY"),
        "expected prompts suppressed: {s:?}"
    );
    assert!(
        s.contains("PS2_EMPTY"),
        "expected prompts suppressed: {s:?}"
    );
    assert!(
        s.contains("PROMPT_COMMAND_EMPTY"),
        "expected prompts suppressed: {s:?}"
    );

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn incomplete_construct_does_not_hang_and_session_returns_to_idle() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping PS2 continuation test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    ws.send(Message::Text(
        exec_frame(1, "eof", b"if true; then echo hi").to_string(),
    ))
    .await
    .expect("send exec");
    let (_stdout, complete) = collect_until_completion(&mut ws).await;
    assert_ne!(
        complete.get("exit").and_then(Value::as_i64),
        Some(0),
        "incomplete construct must fail as a bounded submission"
    );

    ws.send(Message::Text(exec_frame(2, "eof", b"echo OK2").to_string()))
        .await
        .expect("send exec 2");
    let (stdout2, complete2) = collect_until_completion(&mut ws).await;
    assert_eq!(complete2.get("exit").and_then(Value::as_i64), Some(0));
    assert!(
        String::from_utf8_lossy(&stdout2).contains("OK2"),
        "expected session to return to idle after syntax error"
    );

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn program_text_sent_over_stdin_is_not_executed() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping command/control separation test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    ws.send(Message::Text(
        json!({"type":"stdin","data_b64": BASE64.encode(b"echo SHOULD_NOT_RUN\n")}).to_string(),
    ))
    .await
    .expect("send stdin");

    let frames = recv_for_duration(&mut ws, Duration::from_millis(200)).await;
    for frame in frames {
        if frame.get("type").and_then(Value::as_str) != Some("stdout") {
            continue;
        }
        let b64 = frame
            .get("data_b64")
            .and_then(Value::as_str)
            .expect("stdout frame has data_b64");
        let bytes = BASE64.decode(b64).expect("stdout.data_b64 decodes");
        let s = String::from_utf8_lossy(&bytes);
        assert!(
            !s.contains("SHOULD_NOT_RUN"),
            "stdin bytes were misinterpreted as program text: frame={frame}"
        );
    }

    ws.send(Message::Text(exec_frame(1, "eof", b"echo OK").to_string()))
        .await
        .expect("send exec");
    let (stdout, _complete) = collect_until_completion(&mut ws).await;
    let s = String::from_utf8_lossy(&stdout);
    assert!(s.contains("OK"), "expected exec output, got: {s:?}");
    assert!(
        !s.contains("SHOULD_NOT_RUN"),
        "unexpected output from stdin-as-program: {s:?}"
    );

    server.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn evaluator_cannot_see_inherited_socket_fds_dr22_smoke() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping DR-22 adversarial fd test: service init failed: {err}");
            return;
        }
    };

    let (addr, server) = spawn_world_agent_ws(service).await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut ws = match connect_and_start_session_or_skip(addr, cwd.as_path()).await {
        Some(ws) => ws,
        None => {
            server.abort();
            return;
        }
    };

    let program = r#"set -euo pipefail
for fd in /proc/self/fd/*; do
  echo "$(basename "$fd")=$(readlink "$fd")"
done"#;
    ws.send(Message::Text(
        exec_frame(1, "eof", program.as_bytes()).to_string(),
    ))
    .await
    .expect("send exec");
    let (stdout, _complete) = collect_until_completion(&mut ws).await;
    let s = String::from_utf8_lossy(&stdout);
    assert!(
        !s.contains("socket:"),
        "evaluator must not inherit control-plane socket fds: {s:?}"
    );

    server.abort();
}
