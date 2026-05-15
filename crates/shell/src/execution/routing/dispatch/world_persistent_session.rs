//! Host-side persistent REPL session client for world-agent `/v1/stream` (PROTOCOL v1).

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[allow(dead_code)]
mod imp {
    use super::super::shim_ops::build_world_env_map_for_cwd;
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
    use std::path::Path;
    use std::sync::Arc;
    use tokio::sync::{Mutex, OnceCell};
    use tokio_tungstenite as tungs;

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
        fatal: tokio::sync::watch::Receiver<Option<String>>,
        read_task: tokio::task::JoinHandle<Result<()>>,
    }

    #[derive(Debug)]
    enum SessionState {
        Starting {
            ready_tx: tokio::sync::oneshot::Sender<ReadyFrame>,
            requested_shared_world: Option<agent_api_types::SharedWorldOwnerSpec>,
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
        pub(crate) world_id: String,
        pub(crate) cwd: String,
        pub(crate) protocol_version: u32,
        pub(crate) shared_world: Option<agent_api_types::SharedWorldBindingSnapshot>,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct ReplSessionStartParams {
        pub(crate) cwd: String,
        pub(crate) env: HashMap<String, String>,
        pub(crate) policy_snapshot: agent_api_types::PolicySnapshotV3,
        pub(crate) shared_world: Option<agent_api_types::SharedWorldOwnerSpec>,
        pub(crate) world_network: agent_api_types::WorldNetworkRoutingV1,
        pub(crate) cols: u16,
        pub(crate) rows: u16,
    }

    impl ReplSessionStartParams {
        pub(crate) fn for_cwd_and_snapshot(
            cwd: String,
            cwd_path: &Path,
            policy_snapshot: agent_api_types::PolicySnapshotV3,
            world_network: agent_api_types::WorldNetworkRoutingV1,
        ) -> Result<(Self, bool)> {
            let (env, inherit_from_host) = build_world_env_map_for_cwd(cwd_path)?;
            let (cols, rows) = terminal_size_or_default();
            Ok((
                Self {
                    cwd,
                    env,
                    policy_snapshot,
                    shared_world: None,
                    world_network,
                    cols,
                    rows,
                },
                inherit_from_host,
            ))
        }
    }

    type StdoutCallback = Arc<dyn Fn(&[u8]) + Send + Sync>;

    impl ReplPersistentSessionClient {
        pub(crate) async fn start_with(
            start: ReplSessionStartParams,
            on_stdout: StdoutCallback,
        ) -> Result<Self> {
            let requested_shared_world = start.shared_world.clone();
            let (ws, start_frame) = build_ws_and_start_session_frame(start).await?;
            let (sink, stream) = ws.split();
            let sink = Arc::new(Mutex::new(sink));

            let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
            let state = Arc::new(Mutex::new(SessionState::Starting {
                ready_tx,
                requested_shared_world,
            }));
            let ready_cell = OnceCell::new();
            let (fatal_tx, fatal_rx) = tokio::sync::watch::channel::<Option<String>>(None);

            sink.lock()
                .await
                .send(tungs::tungstenite::Message::Text(start_frame))
                .await
                .context("world session ws send start_session")?;

            let read_state = state.clone();
            let read_sink = sink.clone();
            let read_task = tokio::spawn(async move {
                read_loop(stream, read_sink, read_state, on_stdout, fatal_tx).await
            });

            let ready = ready_rx.await.map_err(|_| {
                fatal_rx
                    .borrow()
                    .clone()
                    .map(anyhow::Error::msg)
                    .unwrap_or_else(|| anyhow!("world session failed before ready"))
            })?;
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
            if ready.world_id.trim().is_empty() {
                read_task.abort();
                let _ = read_task.await;
                return Err(anyhow!(
                    "protocol error: ready.world_id must be a non-empty string"
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
                fatal: fatal_rx,
                read_task,
            })
        }

        pub(crate) async fn start(on_stdout: StdoutCallback) -> Result<Self> {
            let cwd_path =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let cwd = cwd_path.display().to_string();
            #[cfg(target_os = "macos")]
            let (env_map, inherit_from_host) = {
                let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
                super::super::world_ops::normalize_env_for_linux_guest(&mut env_map);
                (env_map, inherit_from_host)
            };
            #[cfg(not(target_os = "macos"))]
            let (env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
            if inherit_from_host {
                eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
            }
            let network_policy = policy_snapshot::resolve_world_network_policy_for_cwd(&cwd_path)?;
            let world_network = policy_snapshot::request_world_network_routing(&network_policy);
            let (cols, rows) = terminal_size_or_default();

            Self::start_with(
                ReplSessionStartParams {
                    cwd,
                    env: env_map,
                    policy_snapshot: network_policy.snapshot,
                    shared_world: None,
                    world_network,
                    cols,
                    rows,
                },
                on_stdout,
            )
            .await
        }

        pub(crate) fn ready(&self) -> &ReadyFrame {
            self.ready.get().expect("ready set during start()")
        }

        pub(crate) async fn exec(
            &self,
            program_utf8: &str,
            stdin_mode: ReplStdinMode,
            cmd_id: &str,
        ) -> Result<ReplCommandComplete> {
            let (seq, token_hex, complete_rx, exec_payload) = {
                let mut guard = self.state.lock().await;
                match &*guard {
                    SessionState::Ready { next_seq } => {
                        let seq = *next_seq;
                        let token_hex = generate_token_hex();
                        let program_b64 = BASE64.encode(program_utf8.as_bytes());
                        let frame = ClientFrame::Exec {
                            seq,
                            token_hex: token_hex.clone(),
                            cmd_id: cmd_id.to_string(),
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
                let fatal = self.fatal.borrow().clone();
                anyhow!(
                    "world session terminated while awaiting command_complete (seq={}, token={}){}",
                    seq,
                    redact_token(&token_hex),
                    fatal
                        .as_ref()
                        .map(|s| format!("; cause: {s}"))
                        .unwrap_or_default()
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
                sig: signal.to_string(),
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
            policy_snapshot: Box<agent_api_types::PolicySnapshotV3>,
            #[serde(skip_serializing_if = "Option::is_none")]
            shared_world: Option<agent_api_types::SharedWorldOwnerSpec>,
            world_network: agent_api_types::WorldNetworkRoutingV1,
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
            sig: String,
        },
        Close,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum ServerFrame {
        Ready {
            session_nonce: String,
            world_id: String,
            cwd: String,
            protocol_version: u32,
            #[serde(default)]
            shared_world: Option<agent_api_types::SharedWorldBindingSnapshot>,
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
        fatal_tx: tokio::sync::watch::Sender<Option<String>>,
    ) -> Result<()> {
        use tungs::tungstenite::Message;

        let publish_fatal = |msg: String| {
            let _ = fatal_tx.send(Some(msg));
        };

        while let Some(item) = stream.next().await {
            let msg = match item {
                Ok(m) => m,
                Err(err) => {
                    let err = anyhow!(err).context("world session ws read");
                    publish_fatal(err.to_string());
                    fail_closed(&state).await;
                    return Err(err);
                }
            };
            match msg {
                Message::Text(text) => {
                    let frame: ServerFrame = match serde_json::from_str(&text) {
                        Ok(f) => f,
                        Err(err) => {
                            let err = anyhow!(err).context("protocol error: invalid JSON frame");
                            publish_fatal(err.to_string());
                            fail_closed(&state).await;
                            return Err(err);
                        }
                    };
                    if let Err(err) = handle_server_frame(frame, &state, &on_stdout).await {
                        publish_fatal(err.to_string());
                        fail_closed(&state).await;
                        return Err(err);
                    }
                }
                Message::Ping(payload) => {
                    if let Err(err) = sink.lock().await.send(Message::Pong(payload)).await {
                        let err = anyhow!(err).context("world session ws pong");
                        publish_fatal(err.to_string());
                        fail_closed(&state).await;
                        return Err(err);
                    }
                }
                Message::Pong(_) => {}
                Message::Close(_) => {
                    let closing = matches!(*state.lock().await, SessionState::Closing);
                    if !closing {
                        publish_fatal(
                            "world session closed unexpectedly (protocol fail-closed)".to_string(),
                        );
                        fail_closed(&state).await;
                        return Err(anyhow!(
                            "world session closed unexpectedly (protocol fail-closed)"
                        ));
                    }
                    return Ok(());
                }
                other => {
                    publish_fatal(
                        anyhow!(
                            "protocol error: unexpected websocket message type: {:?}",
                            other
                        )
                        .to_string(),
                    );
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
            publish_fatal(
                "world session stream ended unexpectedly (protocol fail-closed)".to_string(),
            );
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
                world_id,
                cwd,
                protocol_version,
                shared_world,
            } => {
                if world_id.trim().is_empty() {
                    return Err(anyhow!(
                        "protocol error: ready.world_id must be a non-empty string"
                    ));
                }
                let mut guard = state.lock().await;
                let requested_shared_world = match &*guard {
                    SessionState::Starting {
                        requested_shared_world,
                        ..
                    } => requested_shared_world.clone(),
                    _ => {
                        return Err(anyhow!(
                            "protocol error: unexpected ready frame after session start"
                        ))
                    }
                };
                let validated_shared_world =
                    crate::execution::repl_persistent_session::validate_shared_world_echo(
                        requested_shared_world.as_ref(),
                        shared_world.as_ref(),
                        "ready.shared_world",
                        Some(world_id.as_str()),
                    )
                    .map_err(|message| anyhow!("protocol error: {message}"))?;
                let ready = ReadyFrame {
                    session_nonce,
                    world_id,
                    cwd,
                    protocol_version,
                    shared_world: validated_shared_world,
                };
                match std::mem::replace(&mut *guard, SessionState::Closed) {
                    SessionState::Starting { ready_tx, .. } => {
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
    async fn build_ws_and_start_session_frame(
        start: ReplSessionStartParams,
    ) -> Result<(tungs::WebSocketStream<WsIo>, String)> {
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

        let ReplSessionStartParams {
            cwd,
            env,
            policy_snapshot,
            shared_world,
            world_network,
            cols,
            rows,
        } = start;
        let start = ClientFrame::StartSession {
            cwd,
            env,
            policy_snapshot: Box::new(policy_snapshot),
            shared_world,
            world_network,
            cols,
            rows,
        };
        let payload = serde_json::to_string(&start).context("serialize start_session")?;
        Ok((ws, payload))
    }

    #[cfg(target_os = "macos")]
    async fn build_ws_and_start_session_frame(
        start: ReplSessionStartParams,
    ) -> Result<(tungs::WebSocketStream<WsIo>, String)> {
        let ReplSessionStartParams {
            cwd,
            mut env,
            policy_snapshot,
            shared_world,
            world_network,
            cols,
            rows,
        } = start;
        pw::reject_non_linux_shared_owner_request(
            shared_world.as_ref(),
            "persistent world session bootstrap",
        )?;
        super::super::world_ops::normalize_env_for_linux_guest(&mut env);

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
            let start = ClientFrame::StartSession {
                cwd,
                env,
                policy_snapshot: Box::new(policy_snapshot),
                shared_world: shared_world.clone(),
                world_network: world_network.clone(),
                cols,
                rows,
            };
            let payload = serde_json::to_string(&start).context("serialize start_session")?;
            return Ok((ws, payload));
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
        pw::ensure_persistent_session_ready_async(&ctx).await?;

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

        let start = ClientFrame::StartSession {
            cwd,
            env,
            policy_snapshot: Box::new(policy_snapshot),
            shared_world,
            world_network,
            cols,
            rows,
        };
        let payload = serde_json::to_string(&start).context("serialize start_session")?;
        Ok((ws, payload))
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
            SessionState::Starting { ready_tx, .. } => {
                drop(ready_tx);
            }
            SessionState::InFlight { complete_tx, .. } => {
                drop(complete_tx);
            }
            _ => {}
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[cfg(target_os = "macos")]
        use anyhow::Result;
        #[cfg(target_os = "macos")]
        use futures::{SinkExt, StreamExt};
        #[cfg(target_os = "macos")]
        use serial_test::serial;
        #[cfg(target_os = "macos")]
        use std::path::{Path, PathBuf};
        #[cfg(target_os = "macos")]
        use std::sync::atomic::{AtomicUsize, Ordering};
        #[cfg(target_os = "macos")]
        use std::sync::{Arc, Mutex, OnceLock};
        #[cfg(target_os = "macos")]
        use tempfile::tempdir;
        #[cfg(target_os = "macos")]
        use tokio::net::UnixListener;
        #[cfg(target_os = "macos")]
        use tokio_tungstenite::accept_async;
        #[cfg(target_os = "macos")]
        use world_api::{ExecRequest, ExecResult, FsDiff, WorldBackend, WorldHandle, WorldSpec};

        #[test]
        fn signal_frame_serializes_sig_field() {
            let frame = ClientFrame::Signal {
                sig: "INT".to_string(),
            };

            let value = serde_json::to_value(&frame).expect("serialize signal frame");
            assert_eq!(value["type"], "signal");
            assert_eq!(value["sig"], "INT");
            assert!(value.get("signal").is_none());
        }

        #[test]
        fn start_session_serializes_world_network_routing() {
            let frame = ClientFrame::StartSession {
                cwd: "/tmp/project".to_string(),
                env: HashMap::new(),
                policy_snapshot: Box::new(agent_api_types::PolicySnapshotV3 {
                    schema_version: 3,
                    net_allowed: vec!["example.com".to_string()],
                    world_fs: agent_api_types::PolicySnapshotWorldFsV3 {
                        host_visible: true,
                        fail_closed: agent_api_types::PolicySnapshotWorldFsFailClosedV3 {
                            routing: false,
                        },
                        deny_enforcement: None,
                        caged_required: false,
                        discover: Some(agent_api_types::PolicySnapshotWorldFsDimensionV3 {
                            allow_list: vec![".".to_string()],
                            deny_list: Vec::new(),
                        }),
                        read: Some(agent_api_types::PolicySnapshotWorldFsDimensionV3 {
                            allow_list: vec![".".to_string()],
                            deny_list: Vec::new(),
                        }),
                        write: agent_api_types::PolicySnapshotWorldFsWriteV3 {
                            enabled: true,
                            allow_list: vec![".".to_string()],
                            deny_list: Vec::new(),
                        },
                    },
                }),
                shared_world: Some(agent_api_types::SharedWorldOwnerSpec {
                    orchestration_session_id: "orch-test".to_string(),
                    action: agent_api_types::SharedWorldOwnerAction::AttachOrCreate,
                }),
                world_network: agent_api_types::WorldNetworkRoutingV1 {
                    isolate_network: true,
                    allowed_domains: vec!["example.com".to_string()],
                },
                cols: 80,
                rows: 24,
            };

            let value = serde_json::to_value(&frame).expect("serialize start session frame");
            assert_eq!(value["world_network"]["isolate_network"], true);
            assert_eq!(
                value["world_network"]["allowed_domains"],
                serde_json::json!(["example.com"])
            );
            assert_eq!(
                value["shared_world"],
                serde_json::json!({
                    "orchestration_session_id": "orch-test",
                    "action": "attach_or_create",
                })
            );
        }

        #[cfg(target_os = "macos")]
        static TEST_SYNC_READY_CALLS: AtomicUsize = AtomicUsize::new(0);
        #[cfg(target_os = "macos")]
        static TEST_ASYNC_READY_CALLS: AtomicUsize = AtomicUsize::new(0);

        #[cfg(target_os = "macos")]
        fn test_env_lock() -> &'static Mutex<()> {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            LOCK.get_or_init(|| Mutex::new(()))
        }

        #[cfg(target_os = "macos")]
        #[derive(Clone)]
        struct StubWorldBackend;

        #[cfg(target_os = "macos")]
        impl WorldBackend for StubWorldBackend {
            fn ensure_session(&self, _spec: &WorldSpec) -> Result<WorldHandle> {
                Ok(WorldHandle {
                    id: "wld_test".to_string(),
                    shared_binding: None,
                })
            }

            fn exec(&self, _world: &WorldHandle, _req: ExecRequest) -> Result<ExecResult> {
                anyhow::bail!("exec not implemented in test backend")
            }

            fn fs_diff(&self, _world: &WorldHandle, _span_id: &str) -> Result<FsDiff> {
                Ok(FsDiff::default())
            }

            fn apply_policy(&self, _world: &WorldHandle, _spec: &WorldSpec) -> Result<()> {
                Ok(())
            }
        }

        #[cfg(target_os = "macos")]
        fn test_start_params() -> ReplSessionStartParams {
            ReplSessionStartParams {
                cwd: "/tmp/project".to_string(),
                env: HashMap::new(),
                policy_snapshot: agent_api_types::PolicySnapshotV3 {
                    schema_version: 3,
                    net_allowed: Vec::new(),
                    world_fs: agent_api_types::PolicySnapshotWorldFsV3 {
                        host_visible: true,
                        fail_closed: agent_api_types::PolicySnapshotWorldFsFailClosedV3 {
                            routing: false,
                        },
                        deny_enforcement: None,
                        caged_required: false,
                        discover: None,
                        read: None,
                        write: agent_api_types::PolicySnapshotWorldFsWriteV3 {
                            enabled: true,
                            allow_list: vec![".".to_string()],
                            deny_list: Vec::new(),
                        },
                    },
                },
                shared_world: None,
                world_network: agent_api_types::WorldNetworkRoutingV1 {
                    isolate_network: false,
                    allowed_domains: Vec::new(),
                },
                cols: 80,
                rows: 24,
            }
        }

        #[cfg(target_os = "macos")]
        async fn spawn_ready_server(socket_path: &Path) -> tokio::task::JoinHandle<Result<()>> {
            if let Some(parent) = socket_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .expect("create socket parent");
            }
            let _ = tokio::fs::remove_file(socket_path).await;
            let listener = UnixListener::bind(socket_path).expect("bind ready socket");
            tokio::spawn(async move {
                let (stream, _) = listener.accept().await.expect("accept start_session");
                let mut ws = accept_async(stream).await.expect("accept websocket");
                let message = ws.next().await.expect("start frame").expect("frame");
                let text = message.into_text().expect("text frame");
                assert!(
                    text.contains("\"type\":\"start_session\""),
                    "expected start_session frame, got: {text}"
                );
                let ready = serde_json::json!({
                    "type": "ready",
                    "session_nonce": "0123456789abcdef0123456789abcdef",
                    "world_id": "wld_test",
                    "cwd": "/tmp/project",
                    "protocol_version": 1,
                });
                ws.send(tungs::tungstenite::Message::Text(ready.to_string().into()))
                    .await
                    .expect("send ready");
                Ok(())
            })
        }

        #[cfg(target_os = "macos")]
        fn install_test_platform_context_once(socket_path: PathBuf) {
            if pw::get_context().is_some() {
                return;
            }

            let backend: Arc<dyn WorldBackend> = Arc::new(StubWorldBackend);
            let transport = pw::WorldTransport::Unix(socket_path.clone());
            let ensure_ready = Box::new(|| {
                TEST_SYNC_READY_CALLS.fetch_add(1, Ordering::SeqCst);
                tokio::task::block_in_place(|| Ok(()))
            });
            let ensure_persistent_session_ready_async = Box::new(|| {
                TEST_ASYNC_READY_CALLS.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Ok(()) }) as pw::PersistentSessionReadyFuture
            });

            pw::store_context_globally(pw::PlatformWorldContext {
                backend,
                transport,
                socket_path,
                ensure_ready,
                ensure_persistent_session_ready_async,
            });
        }

        #[cfg(target_os = "macos")]
        #[test]
        #[serial]
        fn macos_no_override_current_thread_start_uses_async_readiness_without_panic() {
            let _guard = test_env_lock().lock().expect("env lock");
            TEST_SYNC_READY_CALLS.store(0, Ordering::SeqCst);
            TEST_ASYNC_READY_CALLS.store(0, Ordering::SeqCst);
            std::env::remove_var("SUBSTRATE_WORLD_SOCKET");

            let temp = tempdir().expect("tempdir");
            let socket_path = temp.path().join("world.sock");
            install_test_platform_context_once(socket_path.clone());

            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("current-thread runtime");

            runtime.block_on(async {
                let server = spawn_ready_server(&socket_path).await;
                let client =
                    ReplPersistentSessionClient::start_with(test_start_params(), Arc::new(|_| {}))
                        .await
                        .expect("start persistent session");
                client.close().await.expect("close client");
                server.await.expect("server join").expect("server result");
            });

            assert_eq!(TEST_SYNC_READY_CALLS.load(Ordering::SeqCst), 0);
            assert_eq!(TEST_ASYNC_READY_CALLS.load(Ordering::SeqCst), 1);
        }

        #[cfg(target_os = "macos")]
        #[test]
        #[serial]
        fn macos_socket_override_bypasses_platform_async_readiness() {
            let _guard = test_env_lock().lock().expect("env lock");
            TEST_SYNC_READY_CALLS.store(0, Ordering::SeqCst);
            TEST_ASYNC_READY_CALLS.store(0, Ordering::SeqCst);

            let temp = tempdir().expect("tempdir");
            let socket_path = temp.path().join("override.sock");

            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("current-thread runtime");

            runtime.block_on(async {
                let server = spawn_ready_server(&socket_path).await;
                std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket_path);
                let client =
                    ReplPersistentSessionClient::start_with(test_start_params(), Arc::new(|_| {}))
                        .await
                        .expect("start persistent session with override");
                std::env::remove_var("SUBSTRATE_WORLD_SOCKET");
                client.close().await.expect("close client");
                server.await.expect("server join").expect("server result");
            });

            assert_eq!(TEST_SYNC_READY_CALLS.load(Ordering::SeqCst), 0);
            assert_eq!(TEST_ASYNC_READY_CALLS.load(Ordering::SeqCst), 0);
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[allow(dead_code)]
mod imp {
    use anyhow::{anyhow, Result};
    use std::collections::HashMap;
    use std::path::Path;
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
        pub(crate) world_id: String,
        pub(crate) cwd: String,
        pub(crate) protocol_version: u32,
        pub(crate) shared_world: Option<agent_api_types::SharedWorldBindingSnapshot>,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct ReplSessionStartParams {
        pub(crate) cwd: String,
        pub(crate) env: HashMap<String, String>,
        pub(crate) policy_snapshot: agent_api_types::PolicySnapshotV3,
        pub(crate) shared_world: Option<agent_api_types::SharedWorldOwnerSpec>,
        pub(crate) world_network: agent_api_types::WorldNetworkRoutingV1,
        pub(crate) cols: u16,
        pub(crate) rows: u16,
    }

    impl ReplSessionStartParams {
        pub(crate) fn for_cwd_and_snapshot(
            cwd: String,
            _cwd_path: &Path,
            policy_snapshot: agent_api_types::PolicySnapshotV3,
            world_network: agent_api_types::WorldNetworkRoutingV1,
        ) -> Result<(Self, bool)> {
            Ok((
                Self {
                    cwd,
                    env: HashMap::new(),
                    policy_snapshot,
                    shared_world: None,
                    world_network,
                    cols: 80,
                    rows: 24,
                },
                false,
            ))
        }
    }

    #[derive(Debug)]
    pub(crate) struct ReplPersistentSessionClient {
        ready: ReadyFrame,
    }

    impl ReplPersistentSessionClient {
        pub(crate) async fn start_with(
            _start: ReplSessionStartParams,
            _on_stdout: StdoutCallback,
        ) -> Result<Self> {
            Err(anyhow!(
                "persistent world PTY sessions are unsupported on this platform"
            ))
        }

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
            _cmd_id: &str,
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
