use std::{
    collections::HashMap,
    ffi::OsString,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use super::custody::{self, McpProcessCustody};
use serde::Deserialize;
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStderr, ChildStdin, ChildStdout, Command},
    sync::{mpsc, oneshot, watch, Mutex},
    time,
};
use tracing::{debug, warn};

use super::{
    AppNotification, ApprovalKind, ApprovalRequest, CodexEvent, EventStream, InitializeParams,
    McpError, RequestId, StdioServerConfig, METHOD_CANCEL, METHOD_CODEX_EVENT, METHOD_EXIT,
    METHOD_INITIALIZE, METHOD_SHUTDOWN,
};

#[derive(Clone)]
enum NotificationHook {
    Codex {
        sinks: Arc<Mutex<Vec<mpsc::UnboundedSender<CodexEvent>>>>,
    },
    App {
        sinks: Arc<Mutex<Vec<mpsc::UnboundedSender<AppNotification>>>>,
    },
}

type PendingRequests = Arc<Mutex<HashMap<RequestId, oneshot::Sender<Result<Value, McpError>>>>>;

/// Internal transport that handles stdio JSON-RPC.
pub(super) struct JsonRpcTransport {
    writer: mpsc::UnboundedSender<String>,
    pending: PendingRequests,
    notification_hook: NotificationHook,
    next_id: AtomicU64,
    custody: McpProcessCustody,
    startup_timeout: Duration,
    child_pid: Option<u32>,
}

impl JsonRpcTransport {
    pub(super) async fn spawn_mcp(config: StdioServerConfig) -> Result<Self, McpError> {
        let hook = NotificationHook::Codex {
            sinks: Arc::new(Mutex::new(Vec::new())),
        };
        Self::spawn_with_subcommand(config, "mcp-server", hook, McpProcessCustody::new()).await
    }

    pub(super) async fn spawn_app(
        config: StdioServerConfig,
        custody: McpProcessCustody,
    ) -> Result<Self, McpError> {
        let hook = NotificationHook::App {
            sinks: Arc::new(Mutex::new(Vec::new())),
        };
        Self::spawn_with_subcommand(config, "app-server", hook, custody).await
    }

    async fn spawn_with_subcommand(
        config: StdioServerConfig,
        subcommand: &str,
        notification_hook: NotificationHook,
        custody: McpProcessCustody,
    ) -> Result<Self, McpError> {
        let mut command = Command::new(&config.binary);
        command
            .arg(subcommand)
            .args(
                (subcommand == "app-server" && config.app_server_analytics_default_enabled)
                    .then_some(OsString::from("--analytics-default-enabled")),
            )
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        if let Some(dir) = &config.current_dir {
            command.current_dir(dir);
        }

        if let Some(code_home) = &config.code_home {
            command.env("CODEX_HOME", code_home);
        }

        for (key, value) in &config.env {
            command.env(key, value);
        }

        let command_debug = format!("{command:?}");
        let mut backoff = Duration::from_millis(2);
        let (mut child, owner) = {
            let mut child = None;
            for attempt in 0..5 {
                match custody.spawn(&mut command) {
                    Ok(spawned) => {
                        child = Some(spawned);
                        break;
                    }
                    Err(source) => {
                        let is_busy =
                            matches!(source.kind(), std::io::ErrorKind::ExecutableFileBusy)
                                || source.raw_os_error() == Some(26);
                        if is_busy && attempt < 4 {
                            tokio::time::sleep(backoff).await;
                            backoff = std::cmp::min(backoff * 2, Duration::from_millis(50));
                            continue;
                        }
                        return Err(McpError::Spawn {
                            command: command_debug,
                            source,
                        });
                    }
                }
            }
            child.expect("spawn loop should return or set child")
        };

        let child_pid = child.id();
        let stdout = child.stdout.take();
        let stdin = child.stdin.take();
        let stderr = child.stderr.take();
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let (writer_tx, writer_rx) = mpsc::unbounded_channel();
        let (stop_tx, stop_rx) = watch::channel(false);
        let (writer_stop_tx, writer_stop_rx) = watch::channel(false);
        let mut tasks = Vec::new();
        let mut setup_failures = 0;
        if let Some(stdin) = stdin {
            tasks.push(custody::pipe_task(
                custody::stdin_category(),
                writer_task(stdin, writer_rx, writer_stop_rx, child_pid),
                custody.clone(),
                false,
            ));
        } else {
            setup_failures |= custody::SETUP_FAILED;
        }
        if let Some(stdout) = stdout {
            tasks.push(custody::pipe_task(
                custody::stdout_category(),
                reader_task(
                    stdout,
                    pending.clone(),
                    notification_hook.clone(),
                    config.mirror_stdio,
                    stop_rx.clone(),
                ),
                custody.clone(),
                true,
            ));
        } else {
            setup_failures |= custody::SETUP_FAILED;
        }
        if let Some(stderr) = stderr {
            tasks.push(custody::pipe_task(
                custody::stderr_category(),
                stderr_task(stderr, config.mirror_stdio, stop_rx),
                custody.clone(),
                false,
            ));
        } else {
            setup_failures |= custody::SETUP_FAILED;
        }
        // The actual Child and every pipe owner are transferred without an await.
        custody::supervise(child, owner, tasks, stop_tx, writer_stop_tx, setup_failures);
        tracing::debug!(target: "codex::mcp::safe_lifecycle", child_pid,
            phase = "child_registered", "MCP startup timing");
        if setup_failures != 0 {
            return Err(McpError::Transport("child stdio unavailable".into()));
        }

        Ok(Self {
            writer: writer_tx,
            pending,
            notification_hook,
            next_id: AtomicU64::new(1),
            custody,
            startup_timeout: config.startup_timeout,
            child_pid,
        })
    }

    pub(super) async fn initialize(
        &self,
        params: InitializeParams,
        timeout: Duration,
    ) -> Result<Value, McpError> {
        let (_, rx) = self
            .request(METHOD_INITIALIZE, serde_json::to_value(params)?)
            .await?;
        tracing::debug!(target: "codex::mcp::safe_lifecycle", child_pid = self.child_pid,
            phase = "initialize_queued", "MCP startup timing");
        recv_with_timeout(rx, timeout, self.child_pid, "initialize").await
    }

    pub(super) async fn request(
        &self,
        method: &str,
        params: Value,
    ) -> Result<(RequestId, oneshot::Receiver<Result<Value, McpError>>), McpError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        let serialized = serde_json::to_string(&message)?;
        let (tx, rx) = oneshot::channel();

        {
            let mut guard = self.pending.lock().await;
            guard.insert(id, tx);
        }

        if self.writer.send(serialized).is_err() {
            let mut guard = self.pending.lock().await;
            guard.remove(&id);
            return Err(McpError::ChannelClosed);
        }

        Ok((id, rx))
    }

    pub(super) async fn register_codex_listener(&self) -> EventStream<CodexEvent> {
        match &self.notification_hook {
            NotificationHook::Codex { sinks } => {
                let (tx, rx) = mpsc::unbounded_channel();
                let mut guard = sinks.lock().await;
                guard.push(tx);
                rx
            }
            _ => {
                let (_tx, rx) = mpsc::unbounded_channel();
                rx
            }
        }
    }

    pub(super) async fn register_app_listener(&self) -> EventStream<AppNotification> {
        match &self.notification_hook {
            NotificationHook::App { sinks } => {
                let (tx, rx) = mpsc::unbounded_channel();
                let mut guard = sinks.lock().await;
                guard.push(tx);
                rx
            }
            _ => {
                let (_tx, rx) = mpsc::unbounded_channel();
                rx
            }
        }
    }

    pub(super) fn cancel(&self, request_id: RequestId) -> Result<(), McpError> {
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": METHOD_CANCEL,
            "params": { "id": request_id }
        });
        let serialized = serde_json::to_string(&message)?;
        self.writer
            .send(serialized)
            .map_err(|_| McpError::ChannelClosed)
    }

    pub(super) async fn shutdown(&self) -> Result<(), McpError> {
        if let Ok((_, rx)) = self.request(METHOD_SHUTDOWN, Value::Null).await {
            let _ = recv_with_timeout(rx, Duration::from_secs(5), self.child_pid, "shutdown").await;
        }

        let exit_message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": METHOD_EXIT,
            "params": Value::Null
        });

        let _ = self
            .writer
            .send(serde_json::to_string(&exit_message).unwrap_or_default());

        self.custody.request_termination();
        self.custody
            .settle()
            .await
            .map_err(|error| McpError::Transport(error.to_string()))
    }

    pub(super) async fn terminate_and_settle(&self) -> Result<(), McpError> {
        self.custody.request_termination();
        self.custody
            .settle()
            .await
            .map_err(|error| McpError::Transport(error.to_string()))
    }

    pub(super) fn startup_timeout(&self) -> Duration {
        self.startup_timeout
    }
}

impl Drop for JsonRpcTransport {
    fn drop(&mut self) {
        // The independent owner retains Child and joins all stdio tasks.
        self.custody.request_termination();
    }
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    id: Value,
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcNotification {
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

async fn writer_task(
    mut stdin: ChildStdin,
    mut rx: mpsc::UnboundedReceiver<String>,
    mut stop: watch::Receiver<bool>,
    child_pid: Option<u32>,
) -> io::Result<()> {
    tokio::select! {
        biased;
        _ = custody::stopped(&mut stop) => Ok(()),
        result = async {
            let mut first_write = true;
            while let Some(message) = rx.recv().await {
                if first_write {
                    tracing::debug!(target: "codex::mcp::safe_lifecycle", child_pid,
                        phase = "stdin_first_write_begin", "MCP startup timing");
                }
                let write = async {
                    stdin.write_all(message.as_bytes()).await?;
                    stdin.write_all(b"\n").await?;
                    stdin.flush().await
                }.await;
                if first_write {
                    tracing::debug!(target: "codex::mcp::safe_lifecycle", child_pid,
                        phase = "stdin_first_write_finished", write_checked = write.is_ok(),
                        "MCP startup timing");
                    first_write = false;
                }
                if let Err(error) = write {
                    // The best-effort exit notification may race an already exiting
                    // peer after shutdown. Ordinary request writes remain checked.
                    let exit = serde_json::json!({"jsonrpc":"2.0", "method":METHOD_EXIT, "params":Value::Null}).to_string();
                    if error.kind() != io::ErrorKind::BrokenPipe || message != exit {
                        return Err(error);
                    }
                    return Ok(());
                }
            }
            stdin.shutdown().await
        } => result,
    }
}

async fn reader_task(
    stdout: ChildStdout,
    pending: PendingRequests,
    notification_hook: NotificationHook,
    mirror_stdio: bool,
    mut stop: watch::Receiver<bool>,
) -> io::Result<()> {
    let mut lines = BufReader::new(stdout).lines();
    let result = tokio::select! {
        biased;
        _ = custody::stopped(&mut stop) => Ok(()),
        result = async {
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(err) => {
                return Err(err);
            }
        };

        if mirror_stdio {
            eprintln!("[mcp stdout] {line}");
        }

        if line.trim().is_empty() {
            continue;
        }

        let value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(err) => {
                warn!("failed to parse MCP stdout as JSON: {err}");
                continue;
            }
        };

        match decode_message(value) {
            Some(Incoming::Response(response)) => {
                handle_response(response, &pending).await;
            }
            Some(Incoming::Notification(notification)) => match &notification_hook {
                NotificationHook::Codex { sinks } => {
                    if notification.method == METHOD_CODEX_EVENT {
                        let params = notification.params.unwrap_or(Value::Null);
                        let event = parse_codex_event(&params).unwrap_or(CodexEvent::Raw {
                            method: METHOD_CODEX_EVENT.to_string(),
                            params,
                        });
                        broadcast_codex_event(event, sinks).await;
                    }
                }
                NotificationHook::App { sinks } => {
                    let params = notification.params.unwrap_or(Value::Null);
                    let event = parse_app_notification(&notification.method, &params);
                    broadcast_app_event(event, sinks).await;
                }
            },
            None => {
                warn!("received malformed MCP message");
            }
        }
    }

    Ok(())
        } => result,
    };
    let mut guard = pending.lock().await;
    for (_, tx) in guard.drain() {
        let _ = tx.send(Err(McpError::ChannelClosed));
    }
    drop(guard);
    match notification_hook {
        NotificationHook::Codex { sinks } => sinks.lock().await.clear(),
        NotificationHook::App { sinks } => sinks.lock().await.clear(),
    }
    result
}

async fn stderr_task(
    stderr: ChildStderr,
    mirror_stdio: bool,
    mut stop: watch::Receiver<bool>,
) -> io::Result<()> {
    tokio::select! {
        biased;
        _ = custody::stopped(&mut stop) => Ok(()),
        result = async {
    let mut lines = BufReader::new(stderr).lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(err) => {
                return Err(err);
            }
        };

        if mirror_stdio {
            eprintln!("[mcp stderr] {line}");
        } else {
            debug!("mcp stderr: {line}");
        }
    }
    Ok(())
        } => result,
    }
}

pub(super) fn map_response<T: for<'a> Deserialize<'a> + Send + 'static>(
    rx: oneshot::Receiver<Result<Value, McpError>>,
) -> oneshot::Receiver<Result<T, McpError>> {
    let (tx, mapped_rx) = oneshot::channel();
    tokio::spawn(async move {
        let mapped = match rx.await {
            Ok(Ok(value)) => serde_json::from_value::<T>(value).map_err(McpError::from),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(McpError::ChannelClosed),
        };
        let _ = tx.send(mapped);
    });
    mapped_rx
}

async fn recv_with_timeout(
    rx: oneshot::Receiver<Result<Value, McpError>>,
    timeout: Duration,
    child_pid: Option<u32>,
    operation: &'static str,
) -> Result<Value, McpError> {
    let receive = time::timeout(timeout, rx);
    tracing::debug!(target: "codex::mcp::safe_lifecycle", child_pid, operation,
        phase = "receive_timer_armed", timeout_ms = timeout.as_millis() as u64,
        "MCP startup timing");
    let outcome = receive.await;
    tracing::debug!(target: "codex::mcp::safe_lifecycle", child_pid, operation,
        phase = "receive_timer_resolved", timed_out = outcome.is_err(),
        "MCP startup timing");
    match outcome {
        Ok(Ok(Ok(value))) => Ok(value),
        Ok(Ok(Err(err))) => Err(err),
        Ok(Err(_)) => Err(McpError::ChannelClosed),
        Err(_) => Err(McpError::Timeout(timeout)),
    }
}

#[derive(Debug)]
enum Incoming {
    Response(RpcResponse),
    Notification(RpcNotification),
}

fn decode_message(value: Value) -> Option<Incoming> {
    let is_notification = value.get("id").is_none() && value.get("method").is_some();
    if is_notification {
        let notification: RpcNotification = serde_json::from_value(value).ok()?;
        return Some(Incoming::Notification(notification));
    }

    let is_response = value.get("id").is_some();
    if is_response {
        let response: RpcResponse = serde_json::from_value(value).ok()?;
        return Some(Incoming::Response(response));
    }

    None
}

async fn handle_response(response: RpcResponse, pending: &PendingRequests) {
    let Some(id) = parse_request_id(&response.id) else {
        warn!("received response without numeric id");
        return;
    };

    let sender = { pending.lock().await.remove(&id) };
    let Some(tx) = sender else {
        return;
    };

    if let Some(err) = response.error {
        let mapped = if err.code == -32800 {
            McpError::Cancelled
        } else {
            McpError::Rpc {
                code: err.code,
                message: err.message,
                data: err.data,
            }
        };
        let _ = tx.send(Err(mapped));
    } else if let Some(result) = response.result {
        let _ = tx.send(Ok(result));
    } else {
        let _ = tx.send(Err(McpError::Transport(
            "response missing result and error".into(),
        )));
    }
}

fn parse_request_id(value: &Value) -> Option<RequestId> {
    if let Some(num) = value.as_u64() {
        return Some(num);
    }

    value.as_str().and_then(|s| s.parse::<RequestId>().ok())
}

fn parse_codex_event(value: &Value) -> Option<CodexEvent> {
    let payload = value.get("msg").unwrap_or(value);
    let event_type = payload.get("type")?.as_str()?;
    let conversation_id = payload
        .get("thread_id")
        .or_else(|| payload.get("threadId"))
        .or_else(|| payload.get("session_id"))
        .or_else(|| payload.get("conversation_id"))
        .or_else(|| payload.get("conversationId"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    match event_type {
        "task_complete" => Some(CodexEvent::TaskComplete {
            conversation_id,
            result: payload
                .get("result")
                .cloned()
                .unwrap_or_else(|| payload.clone()),
        }),
        "approval_required" | "approval" => {
            let approval_id = payload
                .get("approval_id")
                .or_else(|| payload.get("id"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let raw_kind = payload
                .get("kind")
                .or_else(|| payload.get("approval_kind"))
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();

            let kind = match raw_kind.to_lowercase().as_str() {
                "exec" => ApprovalKind::Exec,
                "apply" => ApprovalKind::Apply,
                other => ApprovalKind::Unknown(other.to_string()),
            };

            Some(CodexEvent::ApprovalRequired(ApprovalRequest {
                approval_id,
                kind,
                payload: payload.clone(),
            }))
        }
        "cancelled" | "canceled" => Some(CodexEvent::Cancelled {
            conversation_id: payload
                .get("conversation_id")
                .and_then(Value::as_str)
                .map(|s| s.to_string()),
            reason: payload
                .get("reason")
                .and_then(Value::as_str)
                .map(|s| s.to_string()),
        }),
        "error" => Some(CodexEvent::Error {
            message: payload
                .get("message")
                .or_else(|| payload.get("error"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            data: payload.get("data").cloned(),
        }),
        _ => None,
    }
}

async fn broadcast_codex_event(
    event: CodexEvent,
    sinks: &Arc<Mutex<Vec<mpsc::UnboundedSender<CodexEvent>>>>,
) {
    let mut guard = sinks.lock().await;
    guard.retain(|tx| tx.send(event.clone()).is_ok());
}

fn parse_app_notification(method: &str, value: &Value) -> AppNotification {
    let notification_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase();

    let thread_id = extract_string(value, &["thread_id", "threadId"]).unwrap_or_default();
    let turn_id = extract_string(value, &["turn_id", "turnId"]);

    match notification_type.as_str() {
        "task_complete" | "taskcomplete" => AppNotification::TaskComplete {
            thread_id,
            turn_id,
            result: value.get("result").cloned().unwrap_or(Value::Null),
        },
        "item" => AppNotification::Item {
            thread_id,
            turn_id,
            item: value.get("item").cloned().unwrap_or_else(|| value.clone()),
        },
        "error" => AppNotification::Error {
            message: value
                .get("message")
                .or_else(|| value.get("error"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            data: value.get("data").cloned(),
        },
        _ => AppNotification::Raw {
            method: method.to_string(),
            params: value.clone(),
        },
    }
}

fn extract_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
        .map(|s| s.to_string())
}

async fn broadcast_app_event(
    event: AppNotification,
    sinks: &Arc<Mutex<Vec<mpsc::UnboundedSender<AppNotification>>>>,
) {
    let mut guard = sinks.lock().await;
    guard.retain(|tx| tx.send(event.clone()).is_ok());
}
