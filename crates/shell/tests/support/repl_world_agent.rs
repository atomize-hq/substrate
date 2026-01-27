#![cfg(unix)]
#![allow(dead_code)]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;

#[derive(Debug, Clone)]
pub struct PersistentStartSessionRecord {
    pub cwd: String,
    pub env: HashMap<String, String>,
    pub policy_snapshot: JsonValue,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone)]
pub struct PersistentExecRecord {
    pub seq: u64,
    pub token_hex: String,
    pub cmd_id: String,
    pub stdin_mode: String,
    pub program_utf8: String,
}

#[derive(Debug, Clone)]
pub struct LegacyPtyStartRecord {
    pub cmd: String,
    pub cwd: String,
    pub span_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Default)]
pub struct ReplWorldAgentRecords {
    pub persistent_start_sessions: Vec<PersistentStartSessionRecord>,
    pub persistent_execs: Vec<PersistentExecRecord>,
    pub persistent_stdin: Vec<Vec<u8>>,
    pub persistent_signals: Vec<String>,
    pub legacy_pty_starts: Vec<LegacyPtyStartRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamBehavior {
    /// Accept websocket, but close immediately without sending `ready`.
    CloseBeforeReady,
    /// Behave normally: respond to `start_session`, then accept `exec`.
    Normal,
}

#[derive(Debug, Clone)]
pub struct PersistentExecStdoutOverride {
    pub marker: String,
    pub bytes: Vec<u8>,
    pub suffix_bytes: Option<Vec<u8>>,
    pub delay_before_suffix_ms: Option<u64>,
    pub out_of_band_after_complete: Option<(u64, Vec<u8>)>,
}

/// Test-only UDS server that can answer:
/// - `GET /v1/capabilities` (readiness probe)
/// - `GET /v1/stream` websocket (both legacy PTY `/v1/stream` and persistent-session REPL)
pub struct ReplWorldAgentStub {
    path: PathBuf,
    shutdown: Arc<AtomicBool>,
    connections: Arc<AtomicUsize>,
    records: Arc<Mutex<ReplWorldAgentRecords>>,
    persistent_exec_stdout_override: Option<PersistentExecStdoutOverride>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ReplWorldAgentStub {
    pub fn start(path: &Path, behavior: StreamBehavior) -> Self {
        Self::start_with_override(path, behavior, None)
    }

    pub fn start_with_persistent_exec_stdout_override(
        path: &Path,
        marker: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Self {
        Self::start_with_override(
            path,
            StreamBehavior::Normal,
            Some(PersistentExecStdoutOverride {
                marker: marker.into(),
                bytes,
                suffix_bytes: None,
                delay_before_suffix_ms: None,
                out_of_band_after_complete: None,
            }),
        )
    }

    pub fn start_with_persistent_exec_script(
        path: &Path,
        script: PersistentExecStdoutOverride,
    ) -> Self {
        Self::start_with_override(path, StreamBehavior::Normal, Some(script))
    }

    fn start_with_override(
        path: &Path,
        behavior: StreamBehavior,
        persistent_exec_stdout_override: Option<PersistentExecStdoutOverride>,
    ) -> Self {
        let _ = std::fs::remove_file(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create socket parent");
        }

        let path_buf = path.to_path_buf();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_for_thread = shutdown.clone();
        let connections = Arc::new(AtomicUsize::new(0));
        let connections_for_thread = connections.clone();
        let records = Arc::new(Mutex::new(ReplWorldAgentRecords::default()));
        let records_for_thread = records.clone();
        let persistent_exec_stdout_override_for_thread = persistent_exec_stdout_override.clone();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");

            rt.block_on(async move {
                use std::pin::Pin;
                use std::task::{Context, Poll};
                use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
                use tokio::net::UnixListener;
                use tokio_tungstenite as tungs;
                use tungs::tungstenite::protocol::Message;

                #[derive(serde::Deserialize)]
                struct ExecuteRequestStub {
                    cmd: String,
                    cwd: Option<String>,
                    env: Option<HashMap<String, String>>,
                }

                async fn read_http_request(
                    stream: &mut tokio::net::UnixStream,
                ) -> Option<(String, Vec<u8>)> {
                    let mut buf = Vec::new();
                    let mut header_end = None;
                    let mut expected_len = None;

                    for _ in 0..64 {
                        let mut tmp = [0u8; 1024];
                        let n = tokio::time::timeout(
                            std::time::Duration::from_millis(250),
                            stream.read(&mut tmp),
                        )
                            .await
                            .ok()?
                            .ok()?;
                        if n == 0 {
                            break;
                        }
                        buf.extend_from_slice(&tmp[..n]);

                        if header_end.is_none() {
                            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                                header_end = Some(pos + 4);
                                let header = String::from_utf8_lossy(&buf[..pos + 4]).to_string();
                                expected_len = header
                                    .lines()
                                    .find_map(|line| {
                                        let (k, v) = line.split_once(':')?;
                                        if k.eq_ignore_ascii_case("content-length") {
                                            Some(v.trim().parse::<usize>().ok()?)
                                        } else {
                                            None
                                        }
                                    });
                            }
                        }

                        if let (Some(h_end), Some(len)) = (header_end, expected_len) {
                            if buf.len() >= h_end + len {
                                let header = String::from_utf8_lossy(&buf[..h_end]).to_string();
                                let body = buf[h_end..h_end + len].to_vec();
                                return Some((header, body));
                            }
                        } else if let Some(h_end) = header_end {
                            // No Content-Length; treat as header-only request.
                            let header = String::from_utf8_lossy(&buf[..h_end]).to_string();
                            return Some((header, Vec::new()));
                        }
                    }
                    None
                }

                struct ReplayStream {
                    prefix: std::io::Cursor<Vec<u8>>,
                    inner: tokio::net::UnixStream,
                }

                impl AsyncRead for ReplayStream {
                    fn poll_read(
                        mut self: Pin<&mut Self>,
                        cx: &mut Context<'_>,
                        buf: &mut ReadBuf<'_>,
                    ) -> Poll<std::io::Result<()>> {
                        if (self.prefix.position() as usize) < self.prefix.get_ref().len() {
                            let remaining = self.prefix.get_ref().len() - self.prefix.position() as usize;
                            if remaining > 0 && buf.remaining() > 0 {
                                let to_copy = std::cmp::min(remaining, buf.remaining());
                                let pos = self.prefix.position() as usize;
                                buf.put_slice(&self.prefix.get_ref()[pos..pos + to_copy]);
                                self.prefix.set_position((pos + to_copy) as u64);
                                return Poll::Ready(Ok(()));
                            }
                        }
                        Pin::new(&mut self.inner).poll_read(cx, buf)
                    }
                }

                impl AsyncWrite for ReplayStream {
                    fn poll_write(
                        mut self: Pin<&mut Self>,
                        cx: &mut Context<'_>,
                        data: &[u8],
                    ) -> Poll<std::io::Result<usize>> {
                        Pin::new(&mut self.inner).poll_write(cx, data)
                    }

                    fn poll_flush(
                        mut self: Pin<&mut Self>,
                        cx: &mut Context<'_>,
                    ) -> Poll<std::io::Result<()>> {
                        Pin::new(&mut self.inner).poll_flush(cx)
                    }

                    fn poll_shutdown(
                        mut self: Pin<&mut Self>,
                        cx: &mut Context<'_>,
                    ) -> Poll<std::io::Result<()>> {
                        Pin::new(&mut self.inner).poll_shutdown(cx)
                    }
                }

                async fn write_http_json(
                    stream: &mut tokio::net::UnixStream,
                    status_line: &str,
                    body: &str,
                ) {
                    let resp = format!(
                        "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes()).await;
                    let _ = stream.shutdown().await;
                }

                let listener = UnixListener::bind(&path_buf).expect("bind stub socket");

                while !shutdown_for_thread.load(Ordering::SeqCst) {
                    let accept = tokio::time::timeout(
                        std::time::Duration::from_millis(50),
                        listener.accept(),
                    )
                    .await;

                    let Ok(Ok((mut stream, _addr))) = accept else {
                        continue;
                    };

                    connections_for_thread.fetch_add(1, Ordering::SeqCst);

                    let Some((header, body)) = read_http_request(&mut stream).await else {
                        continue;
                    };
                    let first_line = header.lines().next().unwrap_or("");

                    if first_line.starts_with("GET /v1/capabilities ") {
                        let body = r#"{"schema_version":1,"policy_snapshot_v1_supported":true}"#;
                        write_http_json(&mut stream, "200 OK", body).await;
                        continue;
                    }

                    if first_line.starts_with("POST /v1/execute/stream ") {
                        let parsed: ExecuteRequestStub = match serde_json::from_slice(&body) {
                            Ok(p) => p,
                            Err(_) => {
                                write_http_json(&mut stream, "400 Bad Request", r#"{"error":"bad_request","message":"invalid json"}"#).await;
                                continue;
                            }
                        };

                        let mut cmd = std::process::Command::new("bash");
                        cmd.arg("-lc").arg(&parsed.cmd);
                        if let Some(cwd) = &parsed.cwd {
                            cmd.current_dir(cwd);
                        }
                        if let Some(env) = &parsed.env {
                            cmd.envs(env);
                        }
                        let output = cmd.output().expect("run host command");
                        let stdout_b64 = BASE64.encode(&output.stdout);
                        let stderr_b64 = BASE64.encode(&output.stderr);

                        let mut frames = String::new();
                        frames.push_str(
                            &serde_json::to_string(&agent_api_types::ExecuteStreamFrame::Start {
                                span_id: "agent-span".to_string(),
                            })
                            .expect("serialize start"),
                        );
                        frames.push('\n');
                        if !output.stdout.is_empty() {
                            frames.push_str(
                                &serde_json::to_string(
                                    &agent_api_types::ExecuteStreamFrame::Stdout {
                                        chunk_b64: stdout_b64,
                                    },
                                )
                                .expect("serialize stdout"),
                            );
                            frames.push('\n');
                        }
                        if !output.stderr.is_empty() {
                            frames.push_str(
                                &serde_json::to_string(
                                    &agent_api_types::ExecuteStreamFrame::Stderr {
                                        chunk_b64: stderr_b64,
                                    },
                                )
                                .expect("serialize stderr"),
                            );
                            frames.push('\n');
                        }
                        frames.push_str(
                            &serde_json::to_string(&agent_api_types::ExecuteStreamFrame::Exit {
                                exit: output.status.code().unwrap_or(-1),
                                span_id: "agent-span".to_string(),
                                scopes_used: Vec::new(),
                                fs_diff: None,
                            })
                            .expect("serialize exit"),
                        );
                        frames.push('\n');

                        write_http_json(&mut stream, "200 OK", &frames).await;
                        continue;
                    }

                    if first_line.starts_with("POST /v1/execute ") {
                        let parsed: ExecuteRequestStub = match serde_json::from_slice(&body) {
                            Ok(p) => p,
                            Err(_) => {
                                write_http_json(&mut stream, "400 Bad Request", r#"{"error":"bad_request","message":"invalid json"}"#).await;
                                continue;
                            }
                        };

                        let mut cmd = std::process::Command::new("bash");
                        cmd.arg("-lc").arg(&parsed.cmd);
                        if let Some(cwd) = &parsed.cwd {
                            cmd.current_dir(cwd);
                        }
                        if let Some(env) = &parsed.env {
                            cmd.envs(env);
                        }
                        let output = cmd.output().expect("run host command");
                        let resp = serde_json::json!({
                            "exit": output.status.code().unwrap_or(-1),
                            "span_id": "agent-span",
                            "stdout_b64": BASE64.encode(&output.stdout),
                            "stderr_b64": BASE64.encode(&output.stderr),
                            "scopes_used": [],
                            "fs_diff": serde_json::Value::Null,
                        })
                        .to_string();
                        write_http_json(&mut stream, "200 OK", &resp).await;
                        continue;
                    }

                    if !first_line.starts_with("GET /v1/stream ") {
                        // Already read the request; just close.
                        continue;
                    }

                    // The request bytes were already read. Replay them into the websocket acceptor.
                    let raw = header.into_bytes();
                    let replay = ReplayStream {
                        prefix: std::io::Cursor::new(raw),
                        inner: stream,
                    };

                    let ws = match tungs::accept_async(replay).await {
                        Ok(ws) => ws,
                        Err(_) => continue,
                    };
                    let (mut sink, mut ws_stream) = ws.split();

                    let first = ws_stream.next().await;
                    let Some(Ok(Message::Text(first_text))) = first else {
                        continue;
                    };

                    let Ok(first_json) = serde_json::from_str::<JsonValue>(&first_text) else {
                        let _ = sink.send(Message::Close(None)).await;
                        continue;
                    };

                    let ty = first_json.get("type").and_then(|v| v.as_str()).unwrap_or("");

                    if ty == "start" {
                        // Legacy per-command PTY protocol.
                        let cmd = first_json
                            .get("cmd")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let cwd = first_json
                            .get("cwd")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let span_id = first_json
                            .get("span_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let cols = first_json.get("cols").and_then(|v| v.as_u64()).unwrap_or(80) as u16;
                        let rows = first_json.get("rows").and_then(|v| v.as_u64()).unwrap_or(24) as u16;

                        if let Ok(mut guard) = records_for_thread.lock() {
                            guard.legacy_pty_starts.push(LegacyPtyStartRecord {
                                cmd: cmd.clone(),
                                cwd,
                                span_id,
                                cols,
                                rows,
                            });
                        }

                        // Minimal happy-path: produce one stdout frame and then exit.
                        let stdout = format!("__LEGACY_PTY_STUB__ {cmd}\n");
                        let out = serde_json::json!({
                            "type": "stdout",
                            "data_b64": BASE64.encode(stdout.as_bytes()),
                        })
                        .to_string();
                        let _ = sink.send(Message::Text(out)).await;

                        let exit = serde_json::json!({
                            "type": "exit",
                            "code": 0,
                            "world_fs_strategy_primary": "overlay",
                            "world_fs_strategy_final": "overlay",
                            "world_fs_strategy_fallback_reason": "none",
                        })
                        .to_string();
                        let _ = sink.send(Message::Text(exit)).await;
                        let _ = sink.send(Message::Close(None)).await;
                        continue;
                    }

                    if ty != "start_session" {
                        let _ = sink.send(Message::Close(None)).await;
                        continue;
                    }

                    if behavior == StreamBehavior::CloseBeforeReady {
                        let _ = sink.send(Message::Close(None)).await;
                        continue;
                    }

                    // Persistent session protocol v1.
                    let cwd = first_json
                        .get("cwd")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let env = first_json
                        .get("env")
                        .and_then(|v| v.as_object())
                        .map(|m| {
                            m.iter()
                                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                                .collect::<HashMap<String, String>>()
                        })
                        .unwrap_or_default();
                    // Avoid leaking host secrets into test failure output: record env keys but redact values.
                    let env = env
                        .into_iter()
                        .map(|(k, _v)| (k, "<redacted>".to_string()))
                        .collect::<HashMap<String, String>>();
                    let policy_snapshot = first_json
                        .get("policy_snapshot")
                        .cloned()
                        .unwrap_or(JsonValue::Null);
                    let cols = first_json.get("cols").and_then(|v| v.as_u64()).unwrap_or(80) as u16;
                    let rows = first_json.get("rows").and_then(|v| v.as_u64()).unwrap_or(24) as u16;

                    if let Ok(mut guard) = records_for_thread.lock() {
                        guard.persistent_start_sessions.push(PersistentStartSessionRecord {
                            cwd,
                            env,
                            policy_snapshot,
                            cols,
                            rows,
                        });
                    }

                    // Respond with a deterministic ready.
                    let ready = serde_json::json!({
                        "type": "ready",
                        "session_nonce": "0123456789abcdef0123456789abcdef",
                        "cwd": "/",
                        "protocol_version": 1,
                    })
                    .to_string();
                    let _ = sink.send(Message::Text(ready)).await;

                    while let Some(next) = ws_stream.next().await {
                        let Ok(msg) = next else {
                            break;
                        };
                        let Message::Text(text) = msg else {
                            if msg.is_close() {
                                break;
                            }
                            continue;
                        };

                        let Ok(frame) = serde_json::from_str::<JsonValue>(&text) else {
                            break;
                        };
                        let fty = frame.get("type").and_then(|v| v.as_str()).unwrap_or("");

                        match fty {
                            "exec" => {
                                let seq = frame.get("seq").and_then(|v| v.as_u64()).unwrap_or(0);
                                let token_hex = frame
                                    .get("token_hex")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let cmd_id = frame
                                    .get("cmd_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let stdin_mode = frame
                                    .get("stdin_mode")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let program_b64 = frame
                                    .get("program_b64")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let program_utf8 = BASE64
                                    .decode(program_b64)
                                    .ok()
                                    .and_then(|b| String::from_utf8(b).ok())
                                    .unwrap_or_default();

                                if let Ok(mut guard) = records_for_thread.lock() {
                                    guard.persistent_execs.push(PersistentExecRecord {
                                        seq,
                                        token_hex: token_hex.clone(),
                                        cmd_id,
                                        stdin_mode: stdin_mode.clone(),
                                        program_utf8: program_utf8.clone(),
                                    });
                                }

                                // Minimal stdout for observability (optionally overridden for tests).
                                let mut stdout_bytes = None;
                                if let Some(ov) = &persistent_exec_stdout_override_for_thread {
                                    if !ov.marker.is_empty() && program_utf8.contains(&ov.marker) {
                                        stdout_bytes = Some(ov.bytes.clone());
                                    }
                                }

                                let stdout_bytes = stdout_bytes.unwrap_or_else(|| {
                                    format!(
                                        "__PERSISTENT_EXEC_STUB__ {stdin_mode} {program_utf8}\n"
                                    )
                                    .into_bytes()
                                });

                                // Allow testing output buffering by splitting the stream.
                                if stdout_bytes.len() > 3 {
                                    let split_at = stdout_bytes.len() / 2;
                                    for chunk in [&stdout_bytes[..split_at], &stdout_bytes[split_at..]]
                                    {
                                        let stdout = serde_json::json!({
                                            "type": "stdout",
                                            "data_b64": BASE64.encode(chunk),
                                        })
                                        .to_string();
                                        let _ = sink.send(Message::Text(stdout)).await;
                                    }
                                } else {
                                    let stdout = serde_json::json!({
                                        "type": "stdout",
                                        "data_b64": BASE64.encode(&stdout_bytes),
                                    })
                                    .to_string();
                                    let _ = sink.send(Message::Text(stdout)).await;
                                }

                                if let Some(ov) = &persistent_exec_stdout_override_for_thread {
                                    if ov.marker.is_empty() || !program_utf8.contains(&ov.marker) {
                                        // no-op
                                    } else {
                                        if let (Some(delay_ms), Some(suffix)) =
                                            (ov.delay_before_suffix_ms, ov.suffix_bytes.as_ref())
                                        {
                                            tokio::time::sleep(std::time::Duration::from_millis(
                                                delay_ms,
                                            ))
                                            .await;
                                            let stdout = serde_json::json!({
                                                "type": "stdout",
                                                "data_b64": BASE64.encode(suffix),
                                            })
                                            .to_string();
                                            let _ = sink.send(Message::Text(stdout)).await;
                                        }
                                    }
                                }

                                let complete = serde_json::json!({
                                    "type": "command_complete",
                                    "seq": seq,
                                    "token_hex": token_hex,
                                    "exit": 0,
                                    "cwd": "/",
                                })
                                .to_string();
                                let _ = sink.send(Message::Text(complete)).await;

                                if let Some(ov) = &persistent_exec_stdout_override_for_thread {
                                    if ov.marker.is_empty() || !program_utf8.contains(&ov.marker) {
                                        // no-op
                                    } else if let Some((delay_ms, bytes)) =
                                        ov.out_of_band_after_complete.as_ref()
                                    {
                                        tokio::time::sleep(std::time::Duration::from_millis(
                                            *delay_ms,
                                        ))
                                        .await;
                                        let stdout = serde_json::json!({
                                            "type": "stdout",
                                            "data_b64": BASE64.encode(bytes),
                                        })
                                        .to_string();
                                        let _ = sink.send(Message::Text(stdout)).await;
                                    }
                                }
                            }
                            "stdin" => {
                                let data_b64 = frame
                                    .get("data_b64")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                if let Ok(bytes) = BASE64.decode(data_b64.as_bytes()) {
                                    if let Ok(mut guard) = records_for_thread.lock() {
                                        guard.persistent_stdin.push(bytes);
                                    }
                                }
                            }
                            "signal" => {
                                let signal = frame
                                    .get("signal")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                if let Ok(mut guard) = records_for_thread.lock() {
                                    guard.persistent_signals.push(signal);
                                }
                            }
                            "close" => {
                                let exit = serde_json::json!({ "type": "exit", "code": 0 }).to_string();
                                let _ = sink.send(Message::Text(exit)).await;
                                let _ = sink.send(Message::Close(None)).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            });
        });

        Self {
            path: path.to_path_buf(),
            shutdown,
            connections,
            records,
            persistent_exec_stdout_override,
            handle: Some(handle),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn connections(&self) -> usize {
        self.connections.load(Ordering::SeqCst)
    }

    pub fn records(&self) -> Arc<Mutex<ReplWorldAgentRecords>> {
        self.records.clone()
    }
}

impl Drop for ReplWorldAgentStub {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = std::fs::remove_file(&self.path);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
