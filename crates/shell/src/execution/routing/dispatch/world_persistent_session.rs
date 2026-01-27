//! Host-side persistent REPL session client for world-agent `/v1/stream` (PROTOCOL v1).

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[allow(dead_code)]
mod imp {
    use super::super::shim_ops::build_world_env_map;
    use crate::execution::policy_snapshot;
    use crate::execution::pty;
    #[cfg(target_os = "macos")]
    use crate::execution::pw;
    use anyhow::{anyhow, Context, Result};
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use futures::{SinkExt, StreamExt};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{Mutex, OnceCell};
    use tokio_tungstenite as tungs;
    use uuid::Uuid;

    #[cfg(unix)]
    use tokio::net::UnixStream;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ReplStdinMode {
        Eof,
        Passthrough,
    }

    impl ReplStdinMode {
        fn as_str(self) -> &'static str {
            match self {
                ReplStdinMode::Eof => "eof",
                ReplStdinMode::Passthrough => "passthrough",
            }
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct ReplCommandComplete {
        pub(crate) exit: i32,
        pub(crate) cwd: String,
    }

    pub(crate) struct ReplPersistentSessionClient {
        sink: Arc<
            Mutex<
                futures::stream::SplitSink<
                    tungs::WebSocketStream<WsIo>,
                    tungs::tungstenite::Message,
                >,
            >,
        >,
        state: Arc<Mutex<SessionState>>,
        ready: OnceCell<ReadyFrame>,
        read_task: tokio::task::JoinHandle<Result<()>>,
    }

    #[derive(Debug)]
    enum SessionState {
        Starting {
            ready_tx: tokio::sync::oneshot::Sender<ReadyFrame>,
        },
        Ready {
            next_seq: u64,
        },
        InFlight {
            expected_seq: u64,
            expected_token: String,
            complete_tx: tokio::sync::oneshot::Sender<ReplCommandComplete>,
        },
        Closing,
        Closed,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct ReadyFrame {
        pub(crate) session_nonce: String,
        pub(crate) cwd: String,
        pub(crate) protocol_version: u32,
    }

    type StdoutCallback = Arc<dyn Fn(&[u8]) + Send + Sync>;

    impl ReplPersistentSessionClient {
        pub(crate) async fn start(on_stdout: StdoutCallback) -> Result<Self> {
            let (ws, start_frame) = build_ws_and_start_session_frame().await?;
            let (sink, stream) = ws.split();
            let sink = Arc::new(Mutex::new(sink));

            let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
            let state = Arc::new(Mutex::new(SessionState::Starting { ready_tx }));
            let ready_cell = OnceCell::new();

            sink.lock()
                .await
                .send(tungs::tungstenite::Message::Text(start_frame))
                .await
                .context("world session ws send start_session")?;

            let read_state = state.clone();
            let read_sink = sink.clone();
            let read_task =
                tokio::spawn(
                    async move { read_loop(stream, read_sink, read_state, on_stdout).await },
                );

            let ready = ready_rx
                .await
                .map_err(|_| anyhow!("world session failed before ready"))?;
            if ready.protocol_version != 1 {
                read_task.abort();
                let _ = read_task.await;
                return Err(anyhow!(
                    "unsupported persistent session protocol_version={} (expected 1)",
                    ready.protocol_version
                ));
            }
            if !is_hex_lower(&ready.session_nonce) || ready.session_nonce.len() != 32 {
                read_task.abort();
                let _ = read_task.await;
                return Err(anyhow!(
                    "protocol error: invalid ready.session_nonce (expected 32 lowercase hex chars)"
                ));
            }
            if !ready.cwd.starts_with('/') {
                read_task.abort();
                let _ = read_task.await;
                return Err(anyhow!(
                    "protocol error: ready.cwd must be an absolute world path: {}",
                    ready.cwd
                ));
            }

            let _ = ready_cell.set(ready.clone());

            Ok(Self {
                sink,
                state,
                ready: ready_cell,
                read_task,
            })
        }

        pub(crate) fn ready(&self) -> &ReadyFrame {
            self.ready.get().expect("ready set during start()")
        }

        pub(crate) async fn exec(
            &self,
            program_utf8: &str,
            stdin_mode: ReplStdinMode,
        ) -> Result<ReplCommandComplete> {
            let (seq, token_hex, complete_rx, exec_payload) = {
                let mut guard = self.state.lock().await;
                match &*guard {
                    SessionState::Ready { next_seq } => {
                        let seq = *next_seq;
                        let token_hex = generate_token_hex();
                        let cmd_id = Uuid::now_v7().to_string();
                        let program_b64 = BASE64.encode(program_utf8.as_bytes());
                        let frame = ClientFrame::Exec {
                            seq,
                            token_hex: token_hex.clone(),
                            cmd_id,
                            stdin_mode: stdin_mode.as_str().to_string(),
                            program_b64,
                        };
                        let payload =
                            serde_json::to_string(&frame).context("serialize exec frame")?;
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        *guard = SessionState::InFlight {
                            expected_seq: seq,
                            expected_token: token_hex.clone(),
                            complete_tx: tx,
                        };
                        (seq, token_hex, rx, payload)
                    }
                    SessionState::InFlight { .. } => {
                        return Err(anyhow!("protocol error: attempted exec while in-flight"));
                    }
                    SessionState::Starting { .. } => {
                        return Err(anyhow!("protocol error: attempted exec before ready"));
                    }
                    SessionState::Closing | SessionState::Closed => {
                        return Err(anyhow!("protocol error: attempted exec after close"));
                    }
                }
            };

            self.sink
                .lock()
                .await
                .send(tungs::tungstenite::Message::Text(exec_payload))
                .await
                .with_context(|| format!("world session ws send exec seq={seq}"))?;

            let complete = complete_rx.await.map_err(|_| {
                anyhow!(
                    "world session terminated while awaiting command_complete (seq={}, token={})",
                    seq,
                    redact_token(&token_hex)
                )
            })?;
            Ok(complete)
        }

        pub(crate) async fn send_stdin(&self, bytes: &[u8]) -> Result<()> {
            let frame = ClientFrame::Stdin {
                data_b64: BASE64.encode(bytes),
            };
            let payload = serde_json::to_string(&frame).context("serialize stdin frame")?;
            self.sink
                .lock()
                .await
                .send(tungs::tungstenite::Message::Text(payload))
                .await
                .context("world session ws send stdin")?;
            Ok(())
        }

        pub(crate) async fn send_resize(&self, cols: u16, rows: u16) -> Result<()> {
            let frame = ClientFrame::Resize { cols, rows };
            let payload = serde_json::to_string(&frame).context("serialize resize frame")?;
            self.sink
                .lock()
                .await
                .send(tungs::tungstenite::Message::Text(payload))
                .await
                .context("world session ws send resize")?;
            Ok(())
        }

        pub(crate) async fn send_signal(&self, signal: &str) -> Result<()> {
            let frame = ClientFrame::Signal {
                signal: signal.to_string(),
            };
            let payload = serde_json::to_string(&frame).context("serialize signal frame")?;
            self.sink
                .lock()
                .await
                .send(tungs::tungstenite::Message::Text(payload))
                .await
                .context("world session ws send signal")?;
            Ok(())
        }

        pub(crate) async fn close(self) -> Result<()> {
            {
                let mut guard = self.state.lock().await;
                *guard = SessionState::Closing;
            }
            let payload =
                serde_json::to_string(&ClientFrame::Close).context("serialize close frame")?;
            let _ = self
                .sink
                .lock()
                .await
                .send(tungs::tungstenite::Message::Text(payload))
                .await;

            // Let the read loop observe `exit` / close if it arrives; then stop it.
            self.read_task.abort();
            let _ = self.read_task.await;
            Ok(())
        }
    }

    trait WsStreamIo: tokio::io::AsyncRead + tokio::io::AsyncWrite {}

    impl<T> WsStreamIo for T where T: tokio::io::AsyncRead + tokio::io::AsyncWrite {}

    type WsIo = Box<dyn WsStreamIo + Unpin + Send>;

    #[derive(Debug, Serialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum ClientFrame {
        StartSession {
            cwd: String,
            env: HashMap<String, String>,
            policy_snapshot: agent_api_types::PolicySnapshotV1,
            cols: u16,
            rows: u16,
        },
        Exec {
            seq: u64,
            token_hex: String,
            cmd_id: String,
            stdin_mode: String,
            program_b64: String,
        },
        Stdin {
            data_b64: String,
        },
        Resize {
            cols: u16,
            rows: u16,
        },
        Signal {
            signal: String,
        },
        Close,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum ServerFrame {
        Ready {
            session_nonce: String,
            cwd: String,
            protocol_version: u32,
        },
        Stdout {
            data_b64: String,
        },
        CommandComplete {
            seq: u64,
            token_hex: String,
            exit: i32,
            cwd: String,
        },
        Exit {
            code: i32,
        },
        Error {
            code: String,
            message: String,
            fatal: bool,
            #[serde(default)]
            seq: Option<u64>,
        },
    }

    async fn read_loop(
        mut stream: futures::stream::SplitStream<tungs::WebSocketStream<WsIo>>,
        sink: Arc<
            Mutex<
                futures::stream::SplitSink<
                    tungs::WebSocketStream<WsIo>,
                    tungs::tungstenite::Message,
                >,
            >,
        >,
        state: Arc<Mutex<SessionState>>,
        on_stdout: StdoutCallback,
    ) -> Result<()> {
        use tungs::tungstenite::Message;

        while let Some(item) = stream.next().await {
            let msg = match item {
                Ok(m) => m,
                Err(err) => {
                    fail_closed(&state).await;
                    return Err(anyhow!(err).context("world session ws read"));
                }
            };
            match msg {
                Message::Text(text) => {
                    let frame: ServerFrame = match serde_json::from_str(&text) {
                        Ok(f) => f,
                        Err(err) => {
                            fail_closed(&state).await;
                            return Err(anyhow!(err).context("protocol error: invalid JSON frame"));
                        }
                    };
                    if let Err(err) = handle_server_frame(frame, &state, &on_stdout).await {
                        fail_closed(&state).await;
                        return Err(err);
                    }
                }
                Message::Ping(payload) => {
                    if let Err(err) = sink.lock().await.send(Message::Pong(payload)).await {
                        fail_closed(&state).await;
                        return Err(anyhow!(err).context("world session ws pong"));
                    }
                }
                Message::Pong(_) => {}
                Message::Close(_) => {
                    let closing = matches!(*state.lock().await, SessionState::Closing);
                    if !closing {
                        fail_closed(&state).await;
                        return Err(anyhow!(
                            "world session closed unexpectedly (protocol fail-closed)"
                        ));
                    }
                    return Ok(());
                }
                other => {
                    fail_closed(&state).await;
                    return Err(anyhow!(
                        "protocol error: unexpected websocket message type: {:?}",
                        other
                    ));
                }
            }
        }

        let closing = matches!(*state.lock().await, SessionState::Closing);
        if closing {
            Ok(())
        } else {
            fail_closed(&state).await;
            Err(anyhow!(
                "world session stream ended unexpectedly (protocol fail-closed)"
            ))
        }
    }

    async fn handle_server_frame(
        frame: ServerFrame,
        state: &Arc<Mutex<SessionState>>,
        on_stdout: &StdoutCallback,
    ) -> Result<()> {
        match frame {
            ServerFrame::Ready {
                session_nonce,
                cwd,
                protocol_version,
            } => {
                let ready = ReadyFrame {
                    session_nonce,
                    cwd,
                    protocol_version,
                };
                let mut guard = state.lock().await;
                match std::mem::replace(&mut *guard, SessionState::Closed) {
                    SessionState::Starting { ready_tx } => {
                        *guard = SessionState::Ready { next_seq: 1 };
                        let _ = ready_tx.send(ready);
                        Ok(())
                    }
                    other => {
                        *guard = other;
                        Err(anyhow!(
                            "protocol error: unexpected ready frame after session start"
                        ))
                    }
                }
            }
            ServerFrame::Stdout { data_b64 } => {
                let bytes = BASE64
                    .decode(data_b64.as_bytes())
                    .context("protocol error: stdout.data_b64 invalid base64")?;
                (on_stdout)(&bytes);
                Ok(())
            }
            ServerFrame::CommandComplete {
                seq,
                token_hex,
                exit,
                cwd,
            } => {
                if !cwd.starts_with('/') {
                    return Err(anyhow!(
                    "protocol error: command_complete.cwd must be an absolute world path: {cwd}"
                ));
                }
                if !is_hex_lower(&token_hex) || token_hex.len() != 32 {
                    return Err(anyhow!(
                        "protocol error: invalid command_complete.token_hex (expected 32 lowercase hex chars)"
                    ));
                }
                let mut guard = state.lock().await;
                match std::mem::replace(&mut *guard, SessionState::Closed) {
                    SessionState::InFlight {
                        expected_seq,
                        expected_token,
                        complete_tx,
                    } => {
                        if seq != expected_seq || token_hex != expected_token {
                            return Err(anyhow!(
                            "protocol error: command_complete mismatch (expected seq={}, token={} got seq={}, token={})",
                            expected_seq,
                            redact_token(&expected_token),
                            seq,
                            redact_token(&token_hex)
                        ));
                        }
                        *guard = SessionState::Ready {
                            next_seq: expected_seq
                                .checked_add(1)
                                .ok_or_else(|| anyhow!("seq overflow"))?,
                        };
                        let _ = complete_tx.send(ReplCommandComplete { exit, cwd });
                        Ok(())
                    }
                    other => {
                        *guard = other;
                        Err(anyhow!(
                            "protocol error: command_complete received with no command in flight"
                        ))
                    }
                }
            }
            ServerFrame::Exit { code } => {
                let mut guard = state.lock().await;
                if matches!(*guard, SessionState::Closing) {
                    *guard = SessionState::Closed;
                    Ok(())
                } else {
                    Err(anyhow!(
                        "world session exited unexpectedly with code={} (protocol fail-closed)",
                        code
                    ))
                }
            }
            ServerFrame::Error {
                code,
                message,
                fatal,
                seq,
            } => {
                if !fatal {
                    return Err(anyhow!(
                        "protocol error: error.fatal=false is invalid for protocol_version=1"
                    ));
                }
                let seq_note = seq.map(|s| format!(" seq={s}")).unwrap_or_default();
                Err(anyhow!("world-agent error ({code}{seq_note}): {message}"))
            }
        }
    }

    #[cfg(target_os = "linux")]
    async fn build_ws_and_start_session_frame() -> Result<(tungs::WebSocketStream<WsIo>, String)> {
        super::super::world_ops::ensure_world_agent_ready()
            .context("world backend unavailable: ensure world-agent ready")?;

        let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));
        let stream = UnixStream::connect(&socket_path)
            .await
            .with_context(|| format!("connect world-agent UDS ({})", socket_path.display()))?;

        let url = url::Url::parse("ws://localhost/v1/stream").expect("static ws URL");
        let io: WsIo = Box::new(stream);
        let (ws, _resp) = tungs::client_async(url, io)
            .await
            .context("ws handshake /v1/stream")?;

        let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cwd = cwd_path.display().to_string();
        let env_map = build_world_env_map();
        let policy_snapshot = policy_snapshot::resolve_policy_snapshot_for_cwd(&cwd_path)?.snapshot;

        let (cols, rows) = terminal_size_or_default();
        let start = ClientFrame::StartSession {
            cwd,
            env: env_map,
            policy_snapshot,
            cols,
            rows,
        };
        let payload = serde_json::to_string(&start).context("serialize start_session")?;
        Ok((ws, payload))
    }

    #[cfg(target_os = "macos")]
    async fn build_ws_and_start_session_frame() -> Result<(tungs::WebSocketStream<WsIo>, String)> {
        // Allow explicit socket overrides (used by tests/fixtures and advanced setups).
        if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
            let socket_path = std::path::PathBuf::from(socket_path);
            let stream = UnixStream::connect(&socket_path)
                .await
                .with_context(|| format!("connect world-agent UDS ({})", socket_path.display()))?;
            let url = url::Url::parse("ws://localhost/v1/stream").expect("static ws URL");
            let io: WsIo = Box::new(stream);
            let (ws, _resp) = tungs::client_async(url, io)
                .await
                .context("ws handshake /v1/stream")?;
            return Ok((ws, build_start_frame()?));
        }

        let ctx = match pw::get_context() {
            Some(ctx) => ctx,
            None => {
                let detected = pw::detect()
                    .map_err(|e| anyhow::anyhow!("platform world detect failed: {e:#}"))?;
                pw::store_context_globally(detected);
                pw::get_context().ok_or_else(|| anyhow!("no platform world context"))?
            }
        };
        (ctx.ensure_ready.as_ref())()?;

        let (ws, _resp) = match &ctx.transport {
            pw::WorldTransport::Unix(path) => {
                let stream = UnixStream::connect(path)
                    .await
                    .with_context(|| format!("connect world-agent UDS ({})", path.display()))?;
                let url = url::Url::parse("ws://localhost/v1/stream").expect("static ws URL");
                let io: WsIo = Box::new(stream);
                tungs::client_async(url, io).await?
            }
            pw::WorldTransport::Tcp { host, port } => {
                let ws_url = format!("ws://{}:{}/v1/stream", host, port);
                let url = url::Url::parse(&ws_url).context("invalid ws URL")?;
                let tcp = tokio::net::TcpStream::connect(format!("{host}:{port}"))
                    .await
                    .with_context(|| format!("connect world-agent TCP ({host}:{port})"))?;
                let io: WsIo = Box::new(tcp);
                tungs::client_async(url, io).await?
            }
            pw::WorldTransport::Vsock { port } => {
                let host = "127.0.0.1";
                let ws_url = format!("ws://{host}:{port}/v1/stream");
                let url = url::Url::parse(&ws_url).context("invalid ws URL")?;
                let tcp = tokio::net::TcpStream::connect(format!("{host}:{port}"))
                    .await
                    .with_context(|| {
                        format!("connect world-agent VSock proxy TCP ({host}:{port})")
                    })?;
                let io: WsIo = Box::new(tcp);
                tungs::client_async(url, io).await?
            }
        };

        Ok((ws, build_start_frame()?))
    }

    #[cfg(target_os = "macos")]
    fn build_start_frame() -> Result<String> {
        let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cwd = cwd_path.display().to_string();
        let mut env_map = build_world_env_map();
        super::super::world_ops::normalize_env_for_linux_guest(&mut env_map);
        let policy_snapshot = policy_snapshot::resolve_policy_snapshot_for_cwd(&cwd_path)?.snapshot;
        let (cols, rows) = terminal_size_or_default();
        let start = ClientFrame::StartSession {
            cwd,
            env: env_map,
            policy_snapshot,
            cols,
            rows,
        };
        serde_json::to_string(&start).context("serialize start_session")
    }

    fn terminal_size_or_default() -> (u16, u16) {
        match pty::get_terminal_size() {
            Ok(sz) if sz.cols > 0 && sz.rows > 0 => (sz.cols, sz.rows),
            _ => (80, 24),
        }
    }

    fn generate_token_hex() -> String {
        use rand::RngCore;
        let mut raw = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut raw);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = [0u8; 32];
        for (i, b) in raw.iter().enumerate() {
            out[i * 2] = HEX[(b >> 4) as usize];
            out[i * 2 + 1] = HEX[(b & 0x0f) as usize];
        }
        std::str::from_utf8(&out)
            .expect("hex token is valid utf8")
            .to_string()
    }

    fn redact_token(token: &str) -> String {
        if token.len() <= 8 {
            return "***".to_string();
        }
        format!("{}…{}", &token[..4], &token[token.len() - 4..])
    }

    fn is_hex_lower(s: &str) -> bool {
        s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
    }

    async fn fail_closed(state: &Arc<Mutex<SessionState>>) {
        let mut guard = state.lock().await;
        let old = std::mem::replace(&mut *guard, SessionState::Closed);
        match old {
            SessionState::Starting { ready_tx } => {
                drop(ready_tx);
            }
            SessionState::InFlight { complete_tx, .. } => {
                drop(complete_tx);
            }
            _ => {}
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[allow(dead_code)]
mod imp {
    use anyhow::{anyhow, Result};
    use std::sync::Arc;

    type StdoutCallback = Arc<dyn Fn(&[u8]) + Send + Sync>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ReplStdinMode {
        Eof,
        Passthrough,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct ReplCommandComplete {
        pub(crate) exit: i32,
        pub(crate) cwd: String,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct ReadyFrame {
        pub(crate) session_nonce: String,
        pub(crate) cwd: String,
        pub(crate) protocol_version: u32,
    }

    #[derive(Debug)]
    pub(crate) struct ReplPersistentSessionClient {
        ready: ReadyFrame,
    }

    impl ReplPersistentSessionClient {
        pub(crate) async fn start(_on_stdout: StdoutCallback) -> Result<Self> {
            Err(anyhow!(
                "persistent world PTY sessions are unsupported on this platform"
            ))
        }

        pub(crate) fn ready(&self) -> &ReadyFrame {
            &self.ready
        }

        pub(crate) async fn exec(
            &self,
            _program_utf8: &str,
            _stdin_mode: ReplStdinMode,
        ) -> Result<ReplCommandComplete> {
            Err(anyhow!(
                "persistent world PTY sessions are unsupported on this platform"
            ))
        }

        pub(crate) async fn send_stdin(&self, _bytes: &[u8]) -> Result<()> {
            Err(anyhow!(
                "persistent world PTY sessions are unsupported on this platform"
            ))
        }

        pub(crate) async fn send_resize(&self, _cols: u16, _rows: u16) -> Result<()> {
            Err(anyhow!(
                "persistent world PTY sessions are unsupported on this platform"
            ))
        }

        pub(crate) async fn send_signal(&self, _signal: &str) -> Result<()> {
            Err(anyhow!(
                "persistent world PTY sessions are unsupported on this platform"
            ))
        }

        pub(crate) async fn close(self) -> Result<()> {
            Ok(())
        }
    }
}

#[allow(unused_imports)]
pub(crate) use imp::*;
