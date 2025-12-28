//! World and agent routing helpers.

use super::shim_ops::build_world_env_map;
use crate::execution::agent_events::publish_agent_event;
#[cfg(target_os = "macos")]
use crate::execution::pw;
#[cfg(all(test, any(target_os = "linux", target_os = "windows")))]
use crate::execution::world_env_guard;
#[cfg(target_os = "linux")]
use crate::execution::{
    routing::{get_term_size, RawModeGuard},
    socket_activation,
};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use agent_api_client::AgentClient;
use agent_api_types::{ExecuteRequest, ExecuteStreamFrame, WorldFsMode};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::env;
use std::io;
#[cfg(target_os = "linux")]
use substrate_broker::allowed_domains;
use substrate_broker::world_fs_mode;
use substrate_common::agent_events::AgentEvent;
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(target_os = "linux")]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use tokio_tungstenite as tungs;
#[cfg(target_os = "linux")]
use world::LinuxLocalBackend;
#[cfg(target_os = "linux")]
use world_api::{ResourceLimits, WorldBackend, WorldSpec};

#[cfg(target_os = "macos")]
fn normalize_env_for_linux_guest(env_map: &mut std::collections::HashMap<String, String>) {
    // macOS host PATH often contains directories that are mounted into the guest (e.g. /Users/...),
    // which can lead to confusing behavior where `which node` points at a macOS binary that cannot
    // run inside the Linux VM. Prefer a stable Linux guest PATH.
    const GUEST_BASE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
    const WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    env_map.insert(
        "PATH".to_string(),
        format!("{WORLD_DEPS_BIN}:{GUEST_BASE_PATH}"),
    );

    // Avoid leaking macOS host HOME into the Linux guest. This both reduces
    // accidental use of macOS toolchains (nvm/pyenv) and keeps guest-only state
    // in a predictable location.
    if env_map
        .get("HOME")
        .is_none_or(|home| home.is_empty() || home.starts_with("/Users/"))
    {
        env_map.insert("HOME".to_string(), "/root".to_string());
    }

    env_map
        .entry("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string())
        .or_insert_with(|| WORLD_DEPS_BIN.to_string());
}

/// Collect filesystem diff and network scopes from world backend
#[allow(unused_variables)]
pub(super) fn collect_world_telemetry(
    _span_id: &str,
) -> (Vec<String>, Option<substrate_common::fs_diff::FsDiff>) {
    // Try to get world handle from environment
    let world_id = match env::var("SUBSTRATE_WORLD_ID") {
        Ok(id) => id,
        Err(_) => {
            // No world ID, return empty telemetry
            return (vec![], None);
        }
    };

    // Create world backend and collect telemetry
    #[cfg(target_os = "linux")]
    {
        let backend = LinuxLocalBackend::new();
        let handle = world_api::WorldHandle {
            id: world_id.clone(),
        };

        // Try to get filesystem diff
        let fs_diff = backend.fs_diff(&handle, _span_id).ok(); // PTY sessions may run in a separate process; missing cache is expected

        // For now, scopes are tracked in the session world's execute method
        // and would need to be retrieved from there
        let scopes_used = vec![];

        (scopes_used, fs_diff)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // World backend only available on Linux for now
        (vec![], None)
    }
}

#[cfg(target_os = "linux")]
pub(super) fn execute_world_pty_over_ws(cmd: &str, span_id: &str) -> anyhow::Result<i32> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use futures::{SinkExt, StreamExt};

    // Ensure agent is ready
    ensure_world_agent_ready()?;

    // Connect UDS and do WS handshake
    let rt = tokio::runtime::Runtime::new()?;
    let code = rt.block_on(async move {
        let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));
        let stream = UnixStream::connect(&socket_path)
            .await
            .map_err(|e| anyhow::anyhow!("connect UDS ({}): {}", socket_path.display(), e))?;
        let url = url::Url::parse("ws://localhost/v1/stream").unwrap();
        let (ws, _resp) = tungs::client_async(url, stream)
            .await
            .map_err(|e| anyhow::anyhow!("ws handshake: {}", e))?;
        let (sink, mut stream) = ws.split();
        let sink = std::sync::Arc::new(tokio::sync::Mutex::new(sink));

        if std::env::var("SUBSTRATE_WS_DEBUG").ok().as_deref() == Some("1") {
            eprintln!("using world-agent PTY WS");
        }

        // Prepare start frame (strip optional ":pty " prefix used in REPL to force PTY)
        let cmd_sanitized = if let Some(rest) = cmd.strip_prefix(":pty ") {
            rest
        } else {
            cmd
        };
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let env_map: std::collections::HashMap<String, String> = std::env::vars().collect();
        #[cfg(target_os = "linux")]
        let (cols, rows) = get_term_size();
        #[cfg(not(target_os = "linux"))]
        let (cols, rows) = (80u16, 24u16);
        let start = serde_json::json!({
            "type": "start",
            "cmd": cmd_sanitized,
            "cwd": cwd,
            "env": env_map,
            "span_id": span_id,
            "cols": cols,
            "rows": rows,
        });
        sink.lock()
            .await
            .send(tungs::tungstenite::Message::Text(start.to_string()))
            .await
            .map_err(|e| anyhow::anyhow!("ws send start: {}", e))?;

        // Enter raw mode on the local terminal and ensure restoration
        #[cfg(target_os = "linux")]
        let _raw_guard = RawModeGuard::for_stdin_if_tty()?;

        // Spawn stdin forwarder (raw bytes)
        let sink_in = sink.clone();
        let stdin_task = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut stdin = tokio::io::stdin();
            let mut buf = [0u8; 8192];
            loop {
                match stdin.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let b64 = STANDARD.encode(&buf[..n]);
                        let frame = serde_json::json!({"type":"stdin", "data_b64": b64});
                        if sink_in
                            .lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Spawn resize watcher (SIGWINCH)
        #[cfg(target_os = "linux")]
        let resize_task = {
            let sink_resize = sink.clone();
            let mut sig = signal(SignalKind::window_change())
                .map_err(|e| anyhow::anyhow!("sigwinch subscribe: {}", e))?;
            tokio::spawn(async move {
                while sig.recv().await.is_some() {
                    let (c, r) = get_term_size();
                    let frame = serde_json::json!({"type":"resize", "cols": c, "rows": r});
                    if sink_resize
                        .lock()
                        .await
                        .send(tungs::tungstenite::Message::Text(frame.to_string()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            })
        };

        // Spawn Unix signal forwarders (INT, TERM, HUP, QUIT) → WS Signal frames
        #[cfg(target_os = "linux")]
        let signal_tasks = {
            let mut tasks = Vec::new();

            // SIGINT
            if let Ok(mut sig) = signal(SignalKind::interrupt()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "INT"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGTERM
            if let Ok(mut sig) = signal(SignalKind::terminate()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "TERM"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGHUP
            if let Ok(mut sig) = signal(SignalKind::hangup()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "HUP"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGQUIT
            if let Ok(mut sig) = signal(SignalKind::quit()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "QUIT"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }

            tasks
        };

        let mut exit_code: i32 = 0;
        while let Some(msg) = stream.next().await {
            let msg = msg.map_err(|e| anyhow::anyhow!("ws recv: {}", e))?;
            if msg.is_text() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    match v.get("type").and_then(|t| t.as_str()) {
                        Some("stdout") => {
                            if let Some(b64) = v.get("data_b64").and_then(|x| x.as_str()) {
                                if let Ok(bytes) = STANDARD.decode(b64) {
                                    use std::io::Write;
                                    let _ = std::io::stdout().write_all(&bytes);
                                    let _ = std::io::stdout().flush();
                                }
                            }
                        }
                        Some("exit") => {
                            exit_code = v.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                            break;
                        }
                        Some("error") => {
                            if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                                eprintln!("world-agent error: {}", msg);
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            } else if msg.is_close() {
                break;
            }
        }

        // Cleanup background tasks
        stdin_task.abort();
        #[cfg(target_os = "linux")]
        {
            resize_task.abort();
            for t in signal_tasks {
                t.abort();
            }
        }
        Ok::<i32, anyhow::Error>(exit_code)
    })?;
    Ok(code)
}

#[cfg(target_os = "linux")]
fn ensure_world_agent_ready() -> anyhow::Result<()> {
    use std::path::Path;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{Duration, Instant};
    const ACTIVATION_WAIT_MS: u64 = 2_000;
    const DEFAULT_SOCKET_PATH: &str = "/run/substrate.sock";

    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET_PATH));

    let socket_override_active = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(|p| p != std::ffi::OsStr::new(DEFAULT_SOCKET_PATH))
        .unwrap_or(false);

    // Helper: quick readiness probe via HTTP-over-UDS
    fn probe_caps(sock: &Path) -> bool {
        use std::io::{Read, Write};
        match std::os::unix::net::UnixStream::connect(sock) {
            Ok(mut s) => {
                let _ = s.set_read_timeout(Some(std::time::Duration::from_millis(150)));
                let _ = s.set_write_timeout(Some(std::time::Duration::from_millis(150)));
                let req = b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
                if s.write_all(req).is_ok() {
                    let mut buf = [0u8; 512];
                    if let Ok(n) = s.read(&mut buf) {
                        return n > 0
                            && std::str::from_utf8(&buf[..n])
                                .unwrap_or("")
                                .contains(" 200 ");
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    // Fast path: already ready
    if probe_caps(&socket_path) {
        return Ok(());
    }

    let activation_report = socket_activation::socket_activation_report();

    if activation_report.is_socket_activated() {
        let deadline = Instant::now() + Duration::from_millis(ACTIVATION_WAIT_MS);
        while Instant::now() < deadline {
            if probe_caps(&socket_path) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }
        anyhow::bail!(
            "world-agent socket activation detected but {} did not respond. \
             Run 'systemctl status substrate-world-agent.socket' for details.",
            socket_path.display()
        );
    }

    // Clean up stale socket if present (no responding server)
    if !activation_report.is_socket_activated() && Path::new(&socket_path).exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    if socket_override_active {
        anyhow::bail!(
            "world backend unavailable (SUBSTRATE_WORLD_SOCKET override): {} did not respond",
            socket_path.display()
        );
    }

    // Try to spawn agent
    let candidate_bins = [
        std::env::var("SUBSTRATE_WORLD_AGENT_BIN").ok(),
        which::which("substrate-world-agent")
            .ok()
            .map(|p| p.display().to_string()),
        Some("target/release/world-agent".to_string()),
        Some("target/debug/world-agent".to_string()),
    ];
    let bin = candidate_bins
        .into_iter()
        .flatten()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or_else(|| anyhow::anyhow!("world-agent binary not found"))?;

    std::process::Command::new(&bin)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("spawn world-agent: {}", e))?;

    // Wait up to ~1s for readiness
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1000);
    while std::time::Instant::now() < deadline {
        if probe_caps(&socket_path) {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    anyhow::bail!("world-agent readiness probe failed")
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LinuxWorldInit {
    Disabled,
    Agent,
    LocalBackend,
    LocalBackendFailed,
}

#[cfg(target_os = "linux")]
pub(crate) fn init_linux_world(world_disabled: bool) -> LinuxWorldInit {
    init_linux_world_with_probe(world_disabled, ensure_world_agent_ready)
}

#[cfg(target_os = "linux")]
pub(crate) fn init_linux_world_with_probe<F>(world_disabled: bool, agent_probe: F) -> LinuxWorldInit
where
    F: Fn() -> anyhow::Result<()>,
{
    if world_disabled {
        return LinuxWorldInit::Disabled;
    }

    #[cfg(test)]
    let _env_guard = world_env_guard();

    match agent_probe() {
        Ok(()) => {
            env::set_var("SUBSTRATE_WORLD", "enabled");
            env::remove_var("SUBSTRATE_WORLD_ID");
            LinuxWorldInit::Agent
        }
        Err(_agent_err) => {
            #[cfg(test)]
            if let Ok(mock_id) = env::var("SUBSTRATE_TEST_LOCAL_WORLD_ID") {
                env::set_var("SUBSTRATE_WORLD", "enabled");
                env::set_var("SUBSTRATE_WORLD_ID", mock_id);
                return LinuxWorldInit::LocalBackend;
            }

            let spec = WorldSpec {
                reuse_session: true,
                isolate_network: true,
                limits: ResourceLimits::default(),
                enable_preload: false,
                allowed_domains: allowed_domains(),
                project_dir: crate::execution::settings::world_root_from_env().path,
                always_isolate: false,
                fs_mode: world_fs_mode(),
            };
            let backend = LinuxLocalBackend::new();
            match backend.ensure_session(&spec) {
                Ok(handle) => {
                    env::set_var("SUBSTRATE_WORLD", "enabled");
                    env::set_var("SUBSTRATE_WORLD_ID", &handle.id);
                    LinuxWorldInit::LocalBackend
                }
                Err(_local_err) => LinuxWorldInit::LocalBackendFailed,
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub(super) fn execute_world_pty_over_ws_macos(cmd: &str, span_id: &str) -> anyhow::Result<i32> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use futures::StreamExt;
    use tungs::tungstenite::Message;

    let ctx = pw::get_context().ok_or_else(|| anyhow::anyhow!("no platform world context"))?;

    // Put the host terminal into raw mode so interactive programs (nano/vim/top)
    // receive keystrokes immediately (not line-buffered until Enter).
    let _terminal_guard = crate::execution::pty::MinimalTerminalGuard::new()?;

    let rt = tokio::runtime::Runtime::new()?;
    let code = rt.block_on(async move {
        async fn handle_ws<S>(
            ws: tungs::WebSocketStream<S>,
            cmd: &str,
            span_id: &str,
        ) -> anyhow::Result<i32>
        where
            S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
        {
            use futures::SinkExt;
            use std::sync::Arc;
            use tokio::sync::Mutex;
            let (sink, mut stream) = ws.split();
            let sink = Arc::new(Mutex::new(sink));

            let cmd_sanitized = if let Some(rest) = cmd.strip_prefix(":pty ") {
                rest
            } else {
                cmd
            };
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let mut env_map = build_world_env_map();
            normalize_env_for_linux_guest(&mut env_map);
            env_map
                .entry("XDG_DATA_HOME".to_string())
                .or_insert_with(|| "/root/.local/share".to_string());

            // Ensure a few common XDG dirs exist to avoid noisy TUI warnings (e.g. nano history).
            // This is best-effort and does not fail the session in read-only modes.
            let cmd_sanitized = format!(
                "mkdir -p \"${{XDG_DATA_HOME:-$HOME/.local/share}}\" >/dev/null 2>&1 || true; {cmd_sanitized}"
            );
            let (cols, rows) = match crate::execution::pty::get_terminal_size() {
                Ok(sz) => (sz.cols, sz.rows),
                Err(_) => (80u16, 24u16),
            };
            let start = serde_json::json!({
                "type": "start",
                "cmd": cmd_sanitized,
                "cwd": cwd,
                "env": env_map,
                "span_id": span_id,
                "cols": cols,
                "rows": rows,
            });
            sink.lock()
                .await
                .send(Message::Text(start.to_string()))
                .await
                .map_err(|e| anyhow::anyhow!("ws send start: {}", e))?;

            // stdin forwarder
            let mut stdin = tokio::io::stdin();
            let sink_for_stdin = sink.clone();
            let stdin_task = tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                let mut buf = [0u8; 8192];
                loop {
                    match stdin.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&buf[..n]);
                            let frame = serde_json::json!({"type":"stdin", "data_b64": b64});
                            if sink_for_stdin
                                .lock()
                                .await
                                .send(Message::Text(frame.to_string()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            // Terminal resize forwarder (SIGWINCH => WS "resize" frame).
            let sink_for_resize = sink.clone();
            let resize_task = tokio::spawn(async move {
                #[cfg(unix)]
                {
                    use tokio::signal::unix::{signal, SignalKind};
                    if let Ok(mut sigwinch) = signal(SignalKind::window_change()) {
                        while sigwinch.recv().await.is_some() {
                            let (cols, rows) = match crate::execution::pty::get_terminal_size() {
                                Ok(sz) => (sz.cols, sz.rows),
                                Err(_) => continue,
                            };
                            let frame =
                                serde_json::json!({"type":"resize", "cols": cols, "rows": rows});
                            if sink_for_resize
                                .lock()
                                .await
                                .send(Message::Text(frame.to_string()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
            });

            let mut exit_code: i32 = 0;
            while let Some(msg) = stream.next().await {
                let msg = msg.map_err(|e| anyhow::anyhow!("ws recv: {}", e))?;
                if msg.is_text() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                        match v.get("type").and_then(|t| t.as_str()) {
                            Some("stdout") => {
                                if let Some(b64) = v.get("data_b64").and_then(|x| x.as_str()) {
                                    if let Ok(bytes) = STANDARD.decode(b64) {
                                        use std::io::Write;
                                        let _ = std::io::stdout().write_all(&bytes);
                                        let _ = std::io::stdout().flush();
                                    }
                                }
                            }
                            Some("exit") => {
                                exit_code =
                                    v.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                                break;
                            }
                            Some("error") => {
                                if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                                    eprintln!("world-agent error: {}", msg);
                                }
                                break;
                            }
                            _ => {}
                        }
                    }
                } else if msg.is_close() {
                    break;
                }
            }

            stdin_task.abort();
            resize_task.abort();
            Ok::<i32, anyhow::Error>(exit_code)
        }

        // Connect according to transport and delegate to generic handler
        let url = url::Url::parse("ws://localhost/v1/stream").unwrap();
        match &ctx.transport {
            pw::WorldTransport::Unix(path) => {
                let stream = UnixStream::connect(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("connect UDS: {}", e))?;
                let (ws, _resp) = tungs::client_async(url, stream)
                    .await
                    .map_err(|e| anyhow::anyhow!("ws handshake: {}", e))?;
                handle_ws(ws, cmd, span_id).await
            }
            pw::WorldTransport::Tcp { host, port } => {
                let ws_url = format!("ws://{}:{}/v1/stream", host, port);
                let (ws, _resp) = tungs::connect_async(&ws_url)
                    .await
                    .map_err(|e| anyhow::anyhow!("ws connect: {}", e))?;
                handle_ws(ws, cmd, span_id).await
            }
            pw::WorldTransport::Vsock { port } => {
                let ws_url = format!("ws://127.0.0.1:{}/v1/stream", port);
                let (ws, _resp) = tungs::connect_async(&ws_url)
                    .await
                    .map_err(|e| anyhow::anyhow!("ws connect: {}", e))?;
                handle_ws(ws, cmd, span_id).await
            }
        }
    })?;

    Ok(code)
}

pub(crate) struct AgentStreamOutcome {
    pub(crate) exit_code: i32,
    pub(crate) scopes_used: Vec<String>,
    pub(crate) fs_diff: Option<substrate_common::FsDiff>,
}

pub(crate) fn build_agent_client_and_request(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    build_agent_client_and_request_impl(cmd)
}

fn current_world_fs_mode() -> WorldFsMode {
    std::env::var("SUBSTRATE_WORLD_FS_MODE")
        .ok()
        .and_then(|value| WorldFsMode::parse(&value))
        .unwrap_or_else(world_fs_mode)
}

#[cfg(target_os = "linux")]
fn build_agent_client_and_request_impl(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    ensure_world_agent_ready()?;

    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));

    let client = AgentClient::unix_socket(&socket_path)?;
    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .display()
        .to_string();
    let env_map = build_world_env_map();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());

    let request = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd),
        env: Some(env_map),
        pty: false,
        agent_id: agent_id.clone(),
        budget: None,
        world_fs_mode: Some(current_world_fs_mode()),
    };

    Ok((client, request, agent_id))
}

#[cfg(target_os = "macos")]
fn build_agent_client_and_request_impl(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    // Allow explicit socket overrides (used by tests/fixtures and advanced setups).
    // When set, we bypass Lima detection/startup and connect directly.
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        let socket_path = std::path::PathBuf::from(socket_path);
        let client = AgentClient::unix_socket(&socket_path)?;
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .display()
            .to_string();
        let mut env_map = build_world_env_map();
        normalize_env_for_linux_guest(&mut env_map);
        let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());

        let request = ExecuteRequest {
            profile: None,
            cmd: cmd.to_string(),
            cwd: Some(cwd),
            env: Some(env_map),
            pty: false,
            agent_id: agent_id.clone(),
            budget: None,
            world_fs_mode: Some(current_world_fs_mode()),
        };

        return Ok((client, request, agent_id));
    }

    let ctx = match pw::get_context() {
        Some(ctx) => ctx,
        None => {
            // Subcommands like `substrate health` may execute without going through the full shell
            // initialization path, so the platform world context might not be populated yet.
            let detected =
                pw::detect().map_err(|e| anyhow::anyhow!("platform world detect failed: {e:#}"))?;
            pw::store_context_globally(detected);
            pw::get_context().ok_or_else(|| anyhow::anyhow!("no platform world context"))?
        }
    };
    (ctx.ensure_ready.as_ref())()?;

    let client = match &ctx.transport {
        pw::WorldTransport::Unix(path) => AgentClient::unix_socket(path),
        pw::WorldTransport::Tcp { host, port } => AgentClient::tcp(host, *port),
        pw::WorldTransport::Vsock { port } => AgentClient::tcp("127.0.0.1", *port),
    }?;

    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .display()
        .to_string();
    let mut env_map = build_world_env_map();
    normalize_env_for_linux_guest(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());

    let request = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd),
        env: Some(env_map),
        pty: false,
        agent_id: agent_id.clone(),
        budget: None,
        world_fs_mode: Some(current_world_fs_mode()),
    };

    Ok((client, request, agent_id))
}

#[cfg(target_os = "windows")]
fn build_agent_client_and_request_impl(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    use crate::execution::platform_world::windows;
    let backend = windows::get_backend()?;
    let handle = backend.ensure_session(&windows::world_spec())?;

    #[cfg(test)]
    let _env_guard = world_env_guard();

    std::env::set_var("SUBSTRATE_WORLD", "enabled");
    std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);

    let client = windows::build_agent_client()?;
    let cwd = windows::current_dir_wsl()?;
    let env_map = build_world_env_map();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());

    let request = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd),
        env: Some(env_map),
        pty: false,
        agent_id: agent_id.clone(),
        budget: None,
        world_fs_mode: Some(current_world_fs_mode()),
    };

    Ok((client, request, agent_id))
}

pub(crate) fn stream_non_pty_via_agent(command: &str) -> anyhow::Result<AgentStreamOutcome> {
    let (client, request, agent_id) = build_agent_client_and_request(command)?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        use agent_api_types::ApiError;
        use http_body_util::BodyExt;

        let response = client.execute_stream(request).await?;
        if !response.status().is_success() {
            let status = response.status();
            let body_bytes = response
                .into_body()
                .collect()
                .await
                .map_err(|e| anyhow::anyhow!("stream read failed: {}", e))?
                .to_bytes();
            if let Ok(api_error) = serde_json::from_slice::<ApiError>(&body_bytes) {
                anyhow::bail!("API error: {}", api_error);
            }
            let text = String::from_utf8_lossy(&body_bytes);
            anyhow::bail!("HTTP {} error: {}", status, text);
        }

        process_agent_stream(response.into_body(), agent_id).await
    })
}

async fn process_agent_stream(
    mut body: hyper::body::Incoming,
    agent_label: String,
) -> anyhow::Result<AgentStreamOutcome> {
    use http_body_util::BodyExt;

    let mut buffer = Vec::new();
    let mut exit_code = None;
    let mut scopes_used = Vec::new();
    let mut fs_diff = None;

    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|e| anyhow::anyhow!("stream frame error: {}", e))?;
        if let Some(data) = frame.data_ref() {
            buffer.extend_from_slice(data);
            consume_agent_stream_buffer(
                &agent_label,
                &mut buffer,
                &mut exit_code,
                &mut scopes_used,
                &mut fs_diff,
            )?;
        }
    }

    if !buffer.is_empty() {
        consume_agent_stream_buffer(
            &agent_label,
            &mut buffer,
            &mut exit_code,
            &mut scopes_used,
            &mut fs_diff,
        )?;
    }

    let exit_code =
        exit_code.ok_or_else(|| anyhow::anyhow!("agent stream completed without exit frame"))?;

    Ok(AgentStreamOutcome {
        exit_code,
        scopes_used,
        fs_diff,
    })
}

pub(crate) fn consume_agent_stream_buffer(
    agent_label: &str,
    buffer: &mut Vec<u8>,
    exit_code: &mut Option<i32>,
    scopes_used: &mut Vec<String>,
    fs_diff: &mut Option<substrate_common::FsDiff>,
) -> anyhow::Result<()> {
    use anyhow::Context as _;

    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
        let line: Vec<u8> = buffer.drain(..=pos).collect();
        if line.len() <= 1 {
            continue;
        }
        let payload = &line[..line.len() - 1];
        if payload.is_empty() {
            continue;
        }

        let frame: ExecuteStreamFrame = serde_json::from_slice(payload).with_context(|| {
            format!(
                "invalid agent stream frame: {}",
                String::from_utf8_lossy(payload)
            )
        })?;

        match frame {
            ExecuteStreamFrame::Start { .. } => {}
            ExecuteStreamFrame::Stdout { chunk_b64 } => {
                let bytes = BASE64
                    .decode(chunk_b64.as_bytes())
                    .map_err(|e| anyhow::anyhow!("invalid stdout chunk: {}", e))?;
                emit_stream_chunk(agent_label, &bytes, false);
            }
            ExecuteStreamFrame::Stderr { chunk_b64 } => {
                let bytes = BASE64
                    .decode(chunk_b64.as_bytes())
                    .map_err(|e| anyhow::anyhow!("invalid stderr chunk: {}", e))?;
                emit_stream_chunk(agent_label, &bytes, true);
            }
            ExecuteStreamFrame::Event { event } => {
                let _ = publish_agent_event(event);
            }
            ExecuteStreamFrame::Exit {
                exit,
                scopes_used: scopes,
                fs_diff: diff,
                ..
            } => {
                *exit_code = Some(exit);
                *scopes_used = scopes;
                *fs_diff = diff;
            }
            ExecuteStreamFrame::Error { message } => {
                eprintln!("world-agent error: {}", message);
                anyhow::bail!(message);
            }
        }
    }

    Ok(())
}

pub(super) fn emit_stream_chunk(agent_label: &str, data: &[u8], is_stderr: bool) {
    use std::io::Write;

    if is_stderr {
        let mut stderr = io::stderr();
        let _ = stderr.write_all(data);
        let _ = stderr.flush();
    } else {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(data);
        let _ = stdout.flush();
    }

    let text = String::from_utf8_lossy(data);
    let _ = publish_agent_event(AgentEvent::stream_chunk(
        agent_label,
        is_stderr,
        text.to_string(),
    ));
}
