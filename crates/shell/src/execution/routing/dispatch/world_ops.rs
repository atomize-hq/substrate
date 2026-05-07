//! World and agent routing helpers.

use super::shim_ops::build_world_env_map_for_cwd;
use crate::execution::agent_events::publish_agent_event;
use crate::execution::agent_events::ShellCommandEventContext;
#[cfg(target_os = "windows")]
use crate::execution::policy_snapshot::world_spec_for_network_policy;
use crate::execution::policy_snapshot::{
    request_world_network_routing, resolve_world_network_policy_for_cwd,
};
#[cfg(target_os = "macos")]
use crate::execution::pw;
#[cfg(target_os = "linux")]
use crate::execution::routing::{get_term_size, RawModeGuard};
#[cfg(all(test, any(target_os = "linux", target_os = "windows")))]
use crate::execution::world_env_guard;
#[cfg(target_os = "linux")]
use crate::execution::{policy_snapshot::bootstrap_world_spec, socket_activation};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use agent_api_client::AgentClient;
#[cfg(not(target_os = "windows"))]
use agent_api_types::ExecuteCancelRequestV1;
use agent_api_types::{
    ExecuteRequest, ExecuteStreamFrame, MemberDispatchRequestV1, MemberRuntimeBackendKindV1,
    ProcessTelemetry, ResolvedMemberRuntimeDescriptorV1, WorldFsMode,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::env;
use std::io;
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
use world_api::WorldBackend;

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
const WORLD_PROJECT_DIR_OVERRIDE_ENV: &str = "SUBSTRATE_WORLD_PROJECT_DIR";
const SUBSTRATE_PARENT_SPAN_ENV: &str = "SUBSTRATE_PARENT_SPAN_ID";
const RESERVED_WORLD_REQUEST_PROFILES: &[&str] = &["world-deps-provision", "world-deps-probe"];

fn inject_process_trace_env(
    env_map: &mut std::collections::HashMap<String, String>,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
) {
    if let Ok(session_id) = std::env::var("SHIM_SESSION_ID") {
        if !session_id.is_empty() {
            env_map.insert("SHIM_SESSION_ID".to_string(), session_id);
        }
    }
    if let Some(span_id) = parent_span_id {
        env_map.insert(SUBSTRATE_PARENT_SPAN_ENV.to_string(), span_id.to_string());
    }
    if let Some(cmd_id) = parent_cmd_id {
        env_map.insert("SHIM_PARENT_CMD_ID".to_string(), cmd_id.to_string());
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn normalize_env_for_linux_guest(
    env_map: &mut std::collections::HashMap<String, String>,
) {
    // macOS host PATH often contains directories that are mounted into the guest (e.g. /Users/...),
    // which can lead to confusing behavior where `which node` points at a macOS binary that cannot
    // run inside the Linux VM. Prefer a stable Linux guest PATH.
    const GUEST_BASE_PATH: &str =
        "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games";
    const WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    let world_deps_bin = env_map
        .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| WORLD_DEPS_BIN.to_string());
    env_map.insert(
        "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
        world_deps_bin.clone(),
    );
    let world_deps_bin_str = world_deps_bin.as_str();
    let current_path = env_map.get("PATH").map(String::as_str).unwrap_or("");
    // If the caller already provided a Linux-ish PATH (common in tests/fixtures and advanced
    // setups), don't clobber it; just ensure the world-deps bin is present.
    if current_path.contains(GUEST_BASE_PATH) {
        let has_world_deps_bin = current_path
            .split(':')
            .any(|segment| segment.trim_end_matches('/') == world_deps_bin_str);
        if !has_world_deps_bin {
            if current_path.trim().is_empty() {
                env_map.insert("PATH".to_string(), world_deps_bin.clone());
            } else {
                env_map.insert(
                    "PATH".to_string(),
                    format!("{world_deps_bin}:{current_path}"),
                );
            }
        }
    } else {
        env_map.insert(
            "PATH".to_string(),
            format!("{world_deps_bin_str}:{GUEST_BASE_PATH}"),
        );
    }

    // Avoid leaking host HOME into the Linux guest. This both reduces accidental use of host
    // toolchains and keeps guest-only state in a predictable location.
    if env_map.get("HOME").is_none_or(|home| {
        home.is_empty()
            || !home.starts_with('/')
            || (cfg!(target_os = "macos") && home.starts_with("/Users/"))
    }) {
        env_map.insert("HOME".to_string(), "/root".to_string());
    }

    // Note: SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR is set above and may be overridden by tests/fixtures
    // that use a host-exec world-agent stub.
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn ensure_world_deps_bin_on_path(env_map: &mut std::collections::HashMap<String, String>) {
    const DEFAULT_WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    let bin = env_map
        .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_WORLD_DEPS_BIN.to_string());

    env_map.insert(
        "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
        bin.clone(),
    );

    let current = env_map.get("PATH").map(String::as_str).unwrap_or("");
    let bin_norm = bin.trim_end_matches('/');
    let has = current
        .split(':')
        .any(|segment| segment.trim_end_matches('/') == bin_norm);
    if has {
        return;
    }
    if current.trim().is_empty() {
        env_map.insert("PATH".to_string(), bin);
    } else {
        env_map.insert("PATH".to_string(), format!("{bin}:{current}"));
    }
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
            shared_binding: None,
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

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) struct PtyWorldOutcome {
    pub(crate) exit_code: i32,
    pub(crate) fs_strategy: Option<WorldFsStrategyTraceMeta>,
    pub(crate) process_telemetry: ProcessTelemetry,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct MemberDispatchTransportRequest {
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    pub parent_participant_id: Option<String>,
    pub resumed_from_participant_id: Option<String>,
    pub backend_id: String,
    pub protocol: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub initial_prompt: Option<String>,
    pub backend_kind: MemberRuntimeBackendKindV1,
    pub binary_path: String,
}

fn build_execute_request(input: ExecuteRequestInput) -> ExecuteRequest {
    ExecuteRequest {
        profile: input.profile,
        cmd: input.cmd,
        cwd: Some(input.cwd),
        env: Some(input.env_map),
        pty: false,
        agent_id: input.agent_id,
        budget: None,
        policy_snapshot: input.policy_snapshot,
        shared_world: None,
        world_network: Some(input.world_network),
        world_fs_mode: Some(input.world_fs_mode),
        member_dispatch: input.member_dispatch,
    }
}

struct ExecuteRequestInput {
    profile: Option<String>,
    cmd: String,
    cwd: String,
    env_map: std::collections::HashMap<String, String>,
    agent_id: String,
    policy_snapshot: agent_api_types::PolicySnapshotV3,
    world_network: agent_api_types::WorldNetworkRoutingV1,
    world_fs_mode: WorldFsMode,
    member_dispatch: Option<MemberDispatchRequestV1>,
}

#[allow(dead_code)]
fn build_member_dispatch_payload(
    request: &MemberDispatchTransportRequest,
) -> MemberDispatchRequestV1 {
    MemberDispatchRequestV1 {
        schema_version: 1,
        orchestration_session_id: request.orchestration_session_id.clone(),
        participant_id: request.participant_id.clone(),
        orchestrator_participant_id: request.orchestrator_participant_id.clone(),
        parent_participant_id: request.parent_participant_id.clone(),
        resumed_from_participant_id: request.resumed_from_participant_id.clone(),
        backend_id: request.backend_id.clone(),
        protocol: request.protocol.clone(),
        run_id: request.run_id.clone(),
        world_id: request.world_id.clone(),
        world_generation: request.world_generation,
        initial_prompt: request.initial_prompt.clone(),
        resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
            backend_kind: request.backend_kind,
            binary_path: request.binary_path.clone(),
        },
    }
}

#[cfg(target_os = "linux")]
pub(super) fn execute_world_pty_over_ws(
    cmd: &str,
    span_id: &str,
    parent_cmd_id: Option<&str>,
) -> anyhow::Result<PtyWorldOutcome> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use futures::{SinkExt, StreamExt};

    // Ensure agent is ready
    ensure_world_agent_ready()?;

    // Connect UDS and do WS handshake
    let rt = tokio::runtime::Runtime::new()?;
    let outcome = rt.block_on(async move {
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
        let policy_snapshot = resolve_world_network_policy_for_cwd(&cwd)?.snapshot;
        let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd)?;
        if inherit_from_host {
            eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
        }
        crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
            &policy_snapshot,
            &mut env_map,
        )?;
        ensure_world_deps_bin_on_path(&mut env_map);
        inject_process_trace_env(&mut env_map, Some(span_id), parent_cmd_id);
        #[cfg(unix)]
        let (cols, rows) = get_term_size();
        #[cfg(not(target_os = "linux"))]
        let (cols, rows) = (80u16, 24u16);
        let start = serde_json::json!({
            "type": "start",
            "cmd": cmd_sanitized,
            "cwd": cwd,
            "env": env_map,
            "span_id": span_id,
            "policy_snapshot": policy_snapshot,
            "cols": cols,
            "rows": rows,
        });
        sink.lock()
            .await
            .send(tungs::tungstenite::Message::Text(start.to_string()))
            .await
            .map_err(|e| anyhow::anyhow!("ws send start: {}", e))?;

        // Enter raw mode on the local terminal and ensure restoration
        #[cfg(unix)]
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
        #[cfg(unix)]
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
        #[cfg(unix)]
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
        let mut fs_strategy: Option<WorldFsStrategyTraceMeta> = None;
        let mut process_telemetry = ProcessTelemetry::default();
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
                            process_telemetry = extract_process_telemetry_from_ws_exit(&v);
                            if let (Some(primary), Some(final_strategy), Some(reason)) = (
                                v.get("world_fs_strategy_primary")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(substrate_common::WorldFsStrategy::parse),
                                v.get("world_fs_strategy_final")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(substrate_common::WorldFsStrategy::parse),
                                v.get("world_fs_strategy_fallback_reason")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(
                                        substrate_common::WorldFsStrategyFallbackReason::parse,
                                    ),
                            ) {
                                fs_strategy = Some(WorldFsStrategyTraceMeta {
                                    primary,
                                    final_strategy,
                                    fallback_reason: reason,
                                });
                            }
                            break;
                        }
                        Some("error") => {
                            if let Some(message) = v.get("message").and_then(|m| m.as_str()) {
                                if message.contains("WORLD_FS_STRATEGY_UNAVAILABLE") {
                                    return Err(anyhow::Error::new(
                                        WorldFsStrategyUnavailableError {
                                            raw_message: message.to_string(),
                                            fallback_reason:
                                                parse_world_fs_strategy_unavailable_reason(message),
                                        },
                                    ));
                                }
                                return Err(anyhow::anyhow!("world-agent error: {}", message));
                            }
                            return Err(anyhow::anyhow!("world-agent error"));
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
        #[cfg(unix)]
        {
            resize_task.abort();
            for t in signal_tasks {
                t.abort();
            }
        }
        Ok::<PtyWorldOutcome, anyhow::Error>(PtyWorldOutcome {
            exit_code,
            fs_strategy,
            process_telemetry,
        })
    })?;
    Ok(outcome)
}

#[cfg(target_os = "linux")]
pub(super) fn ensure_world_agent_ready() -> anyhow::Result<()> {
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

    // Clean up stale socket if present (no responding server). Only do this when we're
    // confident the socket isn't systemd-managed; if systemd probing fails, keep the
    // path intact to avoid breaking socket activation.
    if matches!(
        activation_report.mode,
        socket_activation::SocketActivationMode::Manual
    ) && Path::new(&socket_path).exists()
    {
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

            let spec = bootstrap_world_spec(
                crate::execution::settings::world_root_from_env().path,
                world_fs_mode(),
            );
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
pub(super) fn execute_world_pty_over_ws_macos(
    cmd: &str,
    span_id: &str,
    _parent_cmd_id: Option<&str>,
) -> anyhow::Result<PtyWorldOutcome> {
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
        ) -> anyhow::Result<PtyWorldOutcome>
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
            let policy_snapshot = resolve_world_network_policy_for_cwd(&cwd)?.snapshot;
            let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd)?;
            if inherit_from_host {
                eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
            }
            normalize_env_for_linux_guest(&mut env_map);
            crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
                &policy_snapshot,
                &mut env_map,
            )?;
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
                "policy_snapshot": policy_snapshot,
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
            let mut fs_strategy: Option<WorldFsStrategyTraceMeta> = None;
            let mut process_telemetry = ProcessTelemetry::default();
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
                                process_telemetry = extract_process_telemetry_from_ws_exit(&v);
                                if let (Some(primary), Some(final_strategy), Some(reason)) = (
                                    v.get("world_fs_strategy_primary")
                                        .and_then(serde_json::Value::as_str)
                                        .and_then(substrate_common::WorldFsStrategy::parse),
                                    v.get("world_fs_strategy_final")
                                        .and_then(serde_json::Value::as_str)
                                        .and_then(substrate_common::WorldFsStrategy::parse),
                                    v.get("world_fs_strategy_fallback_reason")
                                        .and_then(serde_json::Value::as_str)
                                        .and_then(
                                            substrate_common::WorldFsStrategyFallbackReason::parse,
                                        ),
                                ) {
                                    fs_strategy = Some(WorldFsStrategyTraceMeta {
                                        primary,
                                        final_strategy,
                                        fallback_reason: reason,
                                    });
                                }
                                break;
                            }
                            Some("error") => {
                                if let Some(message) = v.get("message").and_then(|m| m.as_str()) {
                                    if message.contains("WORLD_FS_STRATEGY_UNAVAILABLE") {
                                        return Err(anyhow::Error::new(
                                            WorldFsStrategyUnavailableError {
                                                raw_message: message.to_string(),
                                                fallback_reason:
                                                    parse_world_fs_strategy_unavailable_reason(
                                                        message,
                                                    ),
                                            },
                                        ));
                                    }
                                    return Err(anyhow::anyhow!(
                                        "world-agent error: {}",
                                        message
                                    ));
                                }
                                return Err(anyhow::anyhow!("world-agent error"));
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
            Ok::<PtyWorldOutcome, anyhow::Error>(PtyWorldOutcome {
                exit_code,
                fs_strategy,
                process_telemetry,
            })
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
    pub(crate) fs_strategy: Option<WorldFsStrategyTraceMeta>,
    pub(crate) process_telemetry: ProcessTelemetry,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WorldFsStrategyTraceMeta {
    pub(crate) primary: substrate_common::WorldFsStrategy,
    pub(crate) final_strategy: substrate_common::WorldFsStrategy,
    pub(crate) fallback_reason: substrate_common::WorldFsStrategyFallbackReason,
}

#[derive(Debug)]
pub(crate) struct WorldFsStrategyUnavailableError {
    pub(crate) raw_message: String,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    pub(crate) fallback_reason: Option<substrate_common::WorldFsStrategyFallbackReason>,
}

impl std::fmt::Display for WorldFsStrategyUnavailableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw_message)
    }
}

impl std::error::Error for WorldFsStrategyUnavailableError {}

fn parse_world_fs_strategy_unavailable_reason(
    message: &str,
) -> Option<substrate_common::WorldFsStrategyFallbackReason> {
    if !message.contains("WORLD_FS_STRATEGY_UNAVAILABLE") {
        return None;
    }
    for token in message.split_whitespace() {
        if let Some(value) = token.strip_prefix("fallback_reason=") {
            return substrate_common::WorldFsStrategyFallbackReason::parse(value);
        }
    }
    None
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn extract_process_telemetry_from_ws_exit(value: &serde_json::Value) -> ProcessTelemetry {
    let process_events = value
        .get("process_events")
        .cloned()
        .and_then(|raw| serde_json::from_value(raw).ok())
        .unwrap_or_default();
    let process_events_status = value
        .get("process_events_status")
        .and_then(serde_json::Value::as_str)
        .and_then(substrate_common::ProcessEventsStatus::parse)
        .unwrap_or(substrate_common::ProcessEventsStatus::Unavailable);

    ProcessTelemetry {
        process_events,
        process_events_status,
        process_events_reason: value
            .get("process_events_reason")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| {
                (process_events_status == substrate_common::ProcessEventsStatus::Unavailable)
                    .then(|| "backend_disabled".to_string())
            }),
        process_events_dropped: value
            .get("process_events_dropped")
            .and_then(serde_json::Value::as_u64),
        process_events_max: value
            .get("process_events_max")
            .and_then(serde_json::Value::as_u64),
        process_events_backend: value
            .get("process_events_backend")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        process_events_error: value
            .get("process_events_error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
    }
}

pub(crate) fn build_agent_client_and_request(
    cmd: &str,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    build_agent_client_and_request_with_trace_metadata(cmd, None, None)
}

pub(crate) fn build_agent_client_and_request_with_trace_metadata(
    cmd: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    build_agent_client_and_request_impl(cmd, parent_span_id, parent_cmd_id)
}

#[allow(dead_code)]
pub(crate) fn build_agent_client_and_member_dispatch_request(
    request: &MemberDispatchTransportRequest,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    build_agent_client_and_member_dispatch_request_impl(request)
}

pub(crate) fn build_agent_client_and_pending_diff_request() -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::PendingDiffRequestV1,
    String,
)> {
    build_agent_client_and_pending_diff_request_impl()
}

fn current_world_fs_mode() -> WorldFsMode {
    std::env::var("SUBSTRATE_WORLD_FS_MODE")
        .ok()
        .and_then(|value| WorldFsMode::parse(&value))
        .unwrap_or_else(world_fs_mode)
}

fn current_world_request_profile() -> Option<String> {
    std::env::var("SUBSTRATE_WORLD_REQUEST_PROFILE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .filter(|value| !RESERVED_WORLD_REQUEST_PROFILES.contains(&value.as_str()))
}

#[cfg(target_os = "windows")]
fn validate_execute_response_shared_world(
    requested: Option<&agent_api_types::SharedWorldOwnerSpec>,
    response: &agent_api_types::ExecuteResponse,
) -> anyhow::Result<()> {
    crate::execution::repl_persistent_session::validate_shared_world_echo(
        requested,
        response.shared_world.as_ref(),
        "execute_response.shared_world",
        None,
    )
    .map(|_| ())
    .map_err(|message| anyhow::anyhow!("protocol error: {message}"))
}

#[cfg(target_os = "linux")]
fn build_agent_client_and_request_impl(
    cmd: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
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
    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    ensure_world_deps_bin_on_path(&mut env_map);
    preserve_world_project_dir_override(&mut env_map);
    inject_process_trace_env(&mut env_map, parent_span_id, parent_cmd_id);

    let request = build_execute_request(ExecuteRequestInput {
        profile: current_world_request_profile(),
        cmd: cmd.to_string(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: None,
    });

    Ok((client, request, agent_id))
}

#[allow(dead_code)]
#[cfg(target_os = "linux")]
fn build_agent_client_and_member_dispatch_request_impl(
    dispatch: &MemberDispatchTransportRequest,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));

    let client = AgentClient::unix_socket(&socket_path)?;
    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    ensure_world_deps_bin_on_path(&mut env_map);
    preserve_world_project_dir_override(&mut env_map);
    let request = build_execute_request(ExecuteRequestInput {
        profile: current_world_request_profile(),
        cmd: String::new(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: Some(build_member_dispatch_payload(dispatch)),
    });

    Ok((client, request, agent_id))
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn preserve_world_project_dir_override(env_map: &mut std::collections::HashMap<String, String>) {
    let project_dir = crate::execution::settings::world_root_from_env().path;
    env_map.insert(
        WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
        project_dir.display().to_string(),
    );
}

#[cfg(target_os = "linux")]
fn build_agent_client_and_pending_diff_request_impl() -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::PendingDiffRequestV1,
    String,
)> {
    ensure_world_agent_ready()?;

    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));

    let client = AgentClient::unix_socket(&socket_path)?;
    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    let (mut env_map, _inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    ensure_world_deps_bin_on_path(&mut env_map);

    let request = agent_api_types::PendingDiffRequestV1 {
        profile: current_world_request_profile(),
        cwd: Some(cwd),
        env: Some(env_map),
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network: Some(world_network),
    };

    Ok((client, request, agent_id))
}

#[cfg(target_os = "macos")]
fn build_agent_client_and_request_impl(
    cmd: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
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
        let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cwd = cwd_path.display().to_string();
        let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
        if inherit_from_host {
            eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
        }
        normalize_env_for_linux_guest(&mut env_map);
        ensure_world_deps_bin_on_path(&mut env_map);
        let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
        let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
        let world_network = request_world_network_routing(&network_policy);
        let policy_snapshot = network_policy.snapshot;
        crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
            &policy_snapshot,
            &mut env_map,
        )?;
        inject_process_trace_env(&mut env_map, parent_span_id, parent_cmd_id);

        let request = build_execute_request(ExecuteRequestInput {
            profile: current_world_request_profile(),
            cmd: cmd.to_string(),
            cwd,
            env_map,
            agent_id: agent_id.clone(),
            policy_snapshot,
            world_network,
            world_fs_mode: current_world_fs_mode(),
            member_dispatch: None,
        });

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

    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    normalize_env_for_linux_guest(&mut env_map);
    ensure_world_deps_bin_on_path(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    inject_process_trace_env(&mut env_map, parent_span_id, parent_cmd_id);

    let request = build_execute_request(ExecuteRequestInput {
        profile: current_world_request_profile(),
        cmd: cmd.to_string(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: None,
    });

    Ok((client, request, agent_id))
}

#[allow(dead_code)]
#[cfg(target_os = "macos")]
fn build_agent_client_and_member_dispatch_request_impl(
    dispatch: &MemberDispatchTransportRequest,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        let socket_path = std::path::PathBuf::from(socket_path);
        let client = AgentClient::unix_socket(&socket_path)?;
        let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cwd = cwd_path.display().to_string();
        let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
        if inherit_from_host {
            eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
        }
        normalize_env_for_linux_guest(&mut env_map);
        ensure_world_deps_bin_on_path(&mut env_map);
        let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
        let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
        let world_network = request_world_network_routing(&network_policy);
        let policy_snapshot = network_policy.snapshot;
        crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
            &policy_snapshot,
            &mut env_map,
        )?;
        let request = build_execute_request(ExecuteRequestInput {
            profile: current_world_request_profile(),
            cmd: String::new(),
            cwd,
            env_map,
            agent_id: agent_id.clone(),
            policy_snapshot,
            world_network,
            world_fs_mode: current_world_fs_mode(),
            member_dispatch: Some(build_member_dispatch_payload(dispatch)),
        });

        return Ok((client, request, agent_id));
    }

    let ctx = match pw::get_context() {
        Some(ctx) => ctx,
        None => {
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

    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    normalize_env_for_linux_guest(&mut env_map);
    ensure_world_deps_bin_on_path(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    let request = build_execute_request(ExecuteRequestInput {
        profile: current_world_request_profile(),
        cmd: String::new(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: Some(build_member_dispatch_payload(dispatch)),
    });

    Ok((client, request, agent_id))
}

#[cfg(target_os = "macos")]
fn build_agent_client_and_pending_diff_request_impl() -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::PendingDiffRequestV1,
    String,
)> {
    // Allow explicit socket overrides (used by tests/fixtures and advanced setups).
    // When set, we bypass Lima detection/startup and connect directly.
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        let socket_path = std::path::PathBuf::from(socket_path);
        let client = AgentClient::unix_socket(&socket_path)?;
        let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cwd = cwd_path.display().to_string();
        let (mut env_map, _inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
        normalize_env_for_linux_guest(&mut env_map);
        ensure_world_deps_bin_on_path(&mut env_map);
        let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
        let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
        let world_network = request_world_network_routing(&network_policy);
        let policy_snapshot = network_policy.snapshot;
        crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
            &policy_snapshot,
            &mut env_map,
        )?;

        let request = agent_api_types::PendingDiffRequestV1 {
            profile: current_world_request_profile(),
            cwd: Some(cwd),
            env: Some(env_map),
            agent_id: agent_id.clone(),
            policy_snapshot,
            world_network: Some(world_network),
        };

        return Ok((client, request, agent_id));
    }

    let ctx = match pw::get_context() {
        Some(ctx) => ctx,
        None => {
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

    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let (mut env_map, _inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    normalize_env_for_linux_guest(&mut env_map);
    ensure_world_deps_bin_on_path(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;

    let request = agent_api_types::PendingDiffRequestV1 {
        profile: current_world_request_profile(),
        cwd: Some(cwd),
        env: Some(env_map),
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network: Some(world_network),
    };

    Ok((client, request, agent_id))
}

#[cfg(target_os = "windows")]
fn build_agent_client_and_request_impl(
    cmd: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    use crate::execution::platform_world::windows;
    let backend = windows::get_backend()?;
    #[cfg(test)]
    let _env_guard = world_env_guard();

    let client = windows::build_agent_client()?;
    let cwd = windows::current_dir_wsl()?;
    let host_cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let network_policy = resolve_world_network_policy_for_cwd(&host_cwd)?;
    let spec = world_spec_for_network_policy(
        crate::execution::settings::world_root_from_env().path,
        world_fs_mode(),
        &network_policy,
    );
    let handle = backend.ensure_session(&spec)?;

    std::env::set_var("SUBSTRATE_WORLD", "enabled");
    std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);

    let profile = current_world_request_profile();
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&host_cwd)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    normalize_env_for_linux_guest(&mut env_map);
    if profile.as_deref() == Some("world-deps-provision") {
        env_map.retain(|k, _| {
            k == "PATH"
                || k == "HOME"
                || k.starts_with("SUBSTRATE_")
                || k.starts_with("WORLD_")
                || k.starts_with("SHIM_")
        });
    }
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    inject_process_trace_env(&mut env_map, parent_span_id, parent_cmd_id);

    let request = build_execute_request(ExecuteRequestInput {
        profile,
        cmd: cmd.to_string(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: None,
    });

    Ok((client, request, agent_id))
}

#[allow(dead_code)]
#[cfg(target_os = "windows")]
fn build_agent_client_and_member_dispatch_request_impl(
    dispatch: &MemberDispatchTransportRequest,
) -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::ExecuteRequest,
    String,
)> {
    use crate::execution::platform_world::windows;
    let backend = windows::get_backend()?;
    #[cfg(test)]
    let _env_guard = world_env_guard();

    let client = windows::build_agent_client()?;
    let cwd = windows::current_dir_wsl()?;
    let host_cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let network_policy = resolve_world_network_policy_for_cwd(&host_cwd)?;
    let spec = world_spec_for_network_policy(
        crate::execution::settings::world_root_from_env().path,
        world_fs_mode(),
        &network_policy,
    );
    let handle = backend.ensure_session(&spec)?;

    std::env::set_var("SUBSTRATE_WORLD", "enabled");
    std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);

    let profile = current_world_request_profile();
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&host_cwd)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    normalize_env_for_linux_guest(&mut env_map);
    if profile.as_deref() == Some("world-deps-provision") {
        env_map.retain(|k, _| {
            k == "PATH"
                || k == "HOME"
                || k.starts_with("SUBSTRATE_")
                || k.starts_with("WORLD_")
                || k.starts_with("SHIM_")
        });
    }
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    let request = build_execute_request(ExecuteRequestInput {
        profile,
        cmd: String::new(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: Some(build_member_dispatch_payload(dispatch)),
    });

    Ok((client, request, agent_id))
}

#[cfg(target_os = "windows")]
fn build_agent_client_and_pending_diff_request_impl() -> anyhow::Result<(
    agent_api_client::AgentClient,
    agent_api_types::PendingDiffRequestV1,
    String,
)> {
    use crate::execution::platform_world::windows;
    let backend = windows::get_backend()?;
    let handle = backend.ensure_session(&windows::bootstrap_world_spec())?;

    #[cfg(test)]
    let _env_guard = world_env_guard();

    std::env::set_var("SUBSTRATE_WORLD", "enabled");
    std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);

    let client = windows::build_agent_client()?;
    let cwd = windows::current_dir_wsl()?;
    let host_cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let profile = current_world_request_profile();
    let (mut env_map, _inherit_from_host) = build_world_env_map_for_cwd(&host_cwd)?;
    normalize_env_for_linux_guest(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&host_cwd)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;

    let request = agent_api_types::PendingDiffRequestV1 {
        profile,
        cwd: Some(cwd),
        env: Some(env_map),
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network: Some(world_network),
    };

    Ok((client, request, agent_id))
}

pub(crate) fn stream_non_pty_via_agent(
    command: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
    command_event_context: Option<ShellCommandEventContext>,
) -> anyhow::Result<AgentStreamOutcome> {
    let (client, request, agent_id) =
        build_agent_client_and_request_with_trace_metadata(command, parent_span_id, parent_cmd_id)?;

    let host_visible = request.policy_snapshot.world_fs.host_visible;
    let empty_env: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let env_map = request.env.as_ref().unwrap_or(&empty_env);
    let cwd = request
        .cwd
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("."));
    if let Some(deny) =
        substrate_common::world_exec_guard::check_command(&request.cmd, cwd, env_map, host_visible)
    {
        let message = substrate_common::world_exec_guard::deny_message(&deny);
        emit_stream_chunk_with_context(
            &agent_id,
            command_event_context.as_ref(),
            None,
            message.as_bytes(),
            true,
        );
        return Ok(AgentStreamOutcome {
            exit_code: 5,
            scopes_used: Vec::new(),
            fs_diff: None,
            fs_strategy: None,
            process_telemetry: ProcessTelemetry::default(),
        });
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        #[cfg(target_os = "windows")]
        {
            use anyhow::Context as _;

            fn parse_timeout_ms(var: &str) -> Option<std::time::Duration> {
                std::env::var(var)
                    .ok()
                    .and_then(|v| v.trim().parse::<u64>().ok())
                    .map(std::time::Duration::from_millis)
            }

            let timeout = parse_timeout_ms("SUBSTRATE_WSL_AGENT_EXEC_TIMEOUT_MS")
                .unwrap_or_else(|| std::time::Duration::from_secs(120));
            let requested_shared_world = request.shared_world.clone();

            let response = tokio::time::timeout(timeout, async {
                client
                    .execute(request)
                    .await
                    .context("world-agent /v1/execute request failed")
            })
            .await
            .with_context(|| {
                format!(
                    "Timed out after {}s waiting for world-agent /v1/execute (transport: {}).\nHint: ensure the Windows named-pipe forwarder is running (\\\\.\\pipe\\substrate-agent) and the WSL agent is healthy (try `pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl`).",
                    timeout.as_secs(),
                    client.transport().description()
                )
            })??;
            validate_execute_response_shared_world(requested_shared_world.as_ref(), &response)?;
            let stdout = BASE64
                .decode(response.stdout_b64.as_bytes())
                .unwrap_or_else(|_| response.stdout_b64.clone().into_bytes());
            let stderr = BASE64
                .decode(response.stderr_b64.as_bytes())
                .unwrap_or_else(|_| response.stderr_b64.clone().into_bytes());
            emit_stream_chunk_with_context(
                &agent_id,
                command_event_context.as_ref(),
                None,
                &stdout,
                false,
            );
            emit_stream_chunk_with_context(
                &agent_id,
                command_event_context.as_ref(),
                None,
                &stderr,
                true,
            );

            Ok(AgentStreamOutcome {
                exit_code: response.exit,
                scopes_used: response.scopes_used,
                fs_diff: response.fs_diff,
                fs_strategy: None,
                process_telemetry: ProcessTelemetry::not_supported_platform(),
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
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

            let (sigint_tx, mut sigint_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
            let sigint_task = tokio::spawn(async move {
                loop {
                    if tokio::signal::ctrl_c().await.is_err() {
                        break;
                    }
                    if sigint_tx.send(()).is_err() {
                        break;
                    }
                }
            });

            let result = process_agent_stream(
                response.into_body(),
                agent_id,
                command_event_context,
                &mut sigint_rx,
                |span_id, sig| async {
                    client
                        .cancel_execute(ExecuteCancelRequestV1 { span_id, sig })
                        .await
                        .map(|_| ())
                },
            )
            .await;
            sigint_task.abort();
            result
        }
    })
}

#[cfg(not(target_os = "windows"))]
async fn process_agent_stream<Fut>(
    body: hyper::body::Incoming,
    agent_label: String,
    command_event_context: Option<ShellCommandEventContext>,
    sigint_rx: &mut tokio::sync::mpsc::UnboundedReceiver<()>,
    cancel: impl FnMut(String, String) -> Fut,
) -> anyhow::Result<AgentStreamOutcome>
where
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    process_agent_stream_body(
        body,
        agent_label,
        command_event_context.as_ref(),
        sigint_rx,
        cancel,
    )
    .await
}

#[cfg(not(target_os = "windows"))]
async fn process_agent_stream_body<B, Fut>(
    body: B,
    agent_label: String,
    command_event_context: Option<&ShellCommandEventContext>,
    sigint_rx: &mut tokio::sync::mpsc::UnboundedReceiver<()>,
    mut cancel: impl FnMut(String, String) -> Fut,
) -> anyhow::Result<AgentStreamOutcome>
where
    B: hyper::body::Body<Data = hyper::body::Bytes>,
    B::Error: std::fmt::Display,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    use http_body_util::BodyExt;
    let mut body = std::pin::pin!(body);

    let mut buffer = Vec::new();
    let mut exit_code = None;
    let mut scopes_used = Vec::new();
    let mut fs_diff = None;
    let mut fs_strategy = None;
    let mut process_telemetry = ProcessTelemetry::default();
    let mut active_span_id: Option<String> = None;

    loop {
        while sigint_rx.try_recv().is_ok() {
            if let Some(span_id) = active_span_id.as_deref() {
                let cancel_span_id: String = span_id.to_owned();
                if let Err(err) = cancel(cancel_span_id, "INT".to_string()).await {
                    eprintln!("substrate: warn: failed to interrupt world command: {err:#}");
                }
            }
        }

        let frame = match tokio::time::timeout(
            std::time::Duration::from_millis(100),
            body.as_mut().frame(),
        )
        .await
        {
            Ok(frame) => frame,
            Err(_) => continue,
        };

        let Some(frame) = frame else {
            break;
        };
        let frame = frame.map_err(|e| anyhow::anyhow!("stream frame error: {}", e))?;
        if let Some(data) = frame.data_ref() {
            buffer.extend_from_slice(data);
            consume_agent_stream_buffer_with_context(
                &agent_label,
                command_event_context,
                &mut buffer,
                &mut active_span_id,
                &mut exit_code,
                &mut scopes_used,
                &mut fs_diff,
                &mut fs_strategy,
                &mut process_telemetry,
            )?;
            if exit_code.is_some() {
                break;
            }
        }
    }

    if exit_code.is_none() && !buffer.is_empty() {
        consume_agent_stream_buffer_with_context(
            &agent_label,
            command_event_context,
            &mut buffer,
            &mut active_span_id,
            &mut exit_code,
            &mut scopes_used,
            &mut fs_diff,
            &mut fs_strategy,
            &mut process_telemetry,
        )?;
    }

    let exit_code =
        exit_code.ok_or_else(|| anyhow::anyhow!("agent stream completed without exit frame"))?;

    Ok(AgentStreamOutcome {
        exit_code,
        scopes_used,
        fs_diff,
        fs_strategy,
        process_telemetry,
    })
}

#[allow(dead_code)]
pub(crate) fn consume_agent_stream_buffer(
    agent_label: &str,
    buffer: &mut Vec<u8>,
    exit_code: &mut Option<i32>,
    scopes_used: &mut Vec<String>,
    fs_diff: &mut Option<substrate_common::FsDiff>,
) -> anyhow::Result<()> {
    let mut ignored = None;
    let mut process_telemetry = ProcessTelemetry::default();
    consume_agent_stream_buffer_with_meta(
        agent_label,
        buffer,
        &mut None,
        exit_code,
        scopes_used,
        fs_diff,
        &mut ignored,
        &mut process_telemetry,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn consume_agent_stream_buffer_with_meta(
    agent_label: &str,
    buffer: &mut Vec<u8>,
    active_span_id: &mut Option<String>,
    exit_code: &mut Option<i32>,
    scopes_used: &mut Vec<String>,
    fs_diff: &mut Option<substrate_common::FsDiff>,
    fs_strategy: &mut Option<WorldFsStrategyTraceMeta>,
    process_telemetry: &mut ProcessTelemetry,
) -> anyhow::Result<()> {
    consume_agent_stream_buffer_with_context(
        agent_label,
        None,
        buffer,
        active_span_id,
        exit_code,
        scopes_used,
        fs_diff,
        fs_strategy,
        process_telemetry,
    )
}

#[allow(clippy::too_many_arguments)]
fn consume_agent_stream_buffer_with_context(
    agent_label: &str,
    command_event_context: Option<&ShellCommandEventContext>,
    buffer: &mut Vec<u8>,
    active_span_id: &mut Option<String>,
    exit_code: &mut Option<i32>,
    scopes_used: &mut Vec<String>,
    fs_diff: &mut Option<substrate_common::FsDiff>,
    fs_strategy: &mut Option<WorldFsStrategyTraceMeta>,
    process_telemetry: &mut ProcessTelemetry,
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
            ExecuteStreamFrame::Start { span_id } => {
                *active_span_id = Some(span_id);
            }
            ExecuteStreamFrame::Stdout { chunk_b64 } => {
                let bytes = BASE64
                    .decode(chunk_b64.as_bytes())
                    .map_err(|e| anyhow::anyhow!("invalid stdout chunk: {}", e))?;
                emit_stream_chunk_with_context(
                    agent_label,
                    command_event_context,
                    active_span_id.as_deref(),
                    &bytes,
                    false,
                );
            }
            ExecuteStreamFrame::Stderr { chunk_b64 } => {
                let bytes = BASE64
                    .decode(chunk_b64.as_bytes())
                    .map_err(|e| anyhow::anyhow!("invalid stderr chunk: {}", e))?;
                emit_stream_chunk_with_context(
                    agent_label,
                    command_event_context,
                    active_span_id.as_deref(),
                    &bytes,
                    true,
                );
            }
            ExecuteStreamFrame::Event { event } => {
                if let (Some(primary), Some(final_strategy), Some(reason)) = (
                    event
                        .data
                        .get("world_fs_strategy_primary")
                        .and_then(serde_json::Value::as_str)
                        .and_then(substrate_common::WorldFsStrategy::parse),
                    event
                        .data
                        .get("world_fs_strategy_final")
                        .and_then(serde_json::Value::as_str)
                        .and_then(substrate_common::WorldFsStrategy::parse),
                    event
                        .data
                        .get("world_fs_strategy_fallback_reason")
                        .and_then(serde_json::Value::as_str)
                        .and_then(substrate_common::WorldFsStrategyFallbackReason::parse),
                ) {
                    *fs_strategy = Some(WorldFsStrategyTraceMeta {
                        primary,
                        final_strategy,
                        fallback_reason: reason,
                    });
                }
                let _ = publish_agent_event(event);
            }
            ExecuteStreamFrame::Exit {
                exit,
                scopes_used: scopes,
                fs_diff: diff,
                process_telemetry: exit_process_telemetry,
                ..
            } => {
                *exit_code = Some(exit);
                *scopes_used = scopes;
                *fs_diff = diff;
                *process_telemetry = exit_process_telemetry;
            }
            ExecuteStreamFrame::Error { message } => {
                if message.contains("WORLD_FS_STRATEGY_UNAVAILABLE") {
                    return Err(anyhow::Error::new(WorldFsStrategyUnavailableError {
                        raw_message: message.clone(),
                        fallback_reason: parse_world_fs_strategy_unavailable_reason(&message),
                    }));
                }
                eprintln!("world-agent error: {}", message);
                anyhow::bail!(message);
            }
        }
    }

    Ok(())
}

fn emit_stream_chunk_with_context(
    agent_label: &str,
    command_event_context: Option<&ShellCommandEventContext>,
    span_id: Option<&str>,
    data: &[u8],
    is_stderr: bool,
) {
    let (orchestration_session_id, run_id, effective_span_id) = match command_event_context {
        Some(context) => match context.run_id.as_deref() {
            Some(run_id) => (
                Some(context.emission.orchestration_session_id.as_str()),
                run_id,
                span_id.or(context.span_id.as_deref()),
            ),
            None => (None, "", None),
        },
        None => (None, "", None),
    };

    emit_stream_chunk(
        agent_label,
        orchestration_session_id,
        run_id,
        effective_span_id,
        data,
        is_stderr,
    );
}

pub(super) fn emit_stream_chunk(
    agent_label: &str,
    orchestration_session_id: Option<&str>,
    run_id: &str,
    span_id: Option<&str>,
    data: &[u8],
    is_stderr: bool,
) {
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

    let Some(orchestration_session_id) = orchestration_session_id else {
        return;
    };
    let text = String::from_utf8_lossy(data);
    let mut event = AgentEvent::stream_chunk(
        agent_label,
        orchestration_session_id.to_string(),
        run_id.to_string(),
        is_stderr,
        text.to_string(),
    );
    event.span_id = span_id.map(|s| s.to_string());
    let _ = publish_agent_event(event);
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod tests {
    use super::{
        build_execute_request, build_member_dispatch_payload, current_world_request_profile,
        emit_stream_chunk, ensure_world_deps_bin_on_path, extract_process_telemetry_from_ws_exit,
        preserve_world_project_dir_override, process_agent_stream_body, ExecuteRequestInput,
        MemberDispatchTransportRequest, BASE64, WORLD_PROJECT_DIR_OVERRIDE_ENV,
    };
    use crate::execution::agent_events::{
        acquire_event_test_guard, clear_agent_event_sender, init_event_channel,
        ShellCommandEventContext, ShellEventEmissionContext,
    };
    use agent_api_types::{
        ExecuteStreamFrame, MemberRuntimeBackendKindV1, PolicySnapshotV3,
        PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
        ResolvedMemberRuntimeDescriptorV1, WorldFsMode, WorldNetworkRoutingV1,
    };
    use base64::Engine;
    use futures::stream;
    use http_body_util::StreamBody;
    use serde_json::json;
    use std::convert::Infallible;
    use std::sync::{Arc, Mutex, OnceLock};
    use std::time::Duration;
    use substrate_common::agent_events::AgentEventKind;

    fn with_env_var<T>(key: &str, value: &str, f: impl FnOnce() -> T) -> T {
        let _guard = test_env_lock().lock().expect("test env mutex poisoned");
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        let result = f();
        match previous {
            Some(previous) => std::env::set_var(key, previous),
            None => std::env::remove_var(key),
        }
        result
    }

    fn test_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn encode_stream_frame(frame: ExecuteStreamFrame) -> hyper::body::Bytes {
        let mut payload = serde_json::to_vec(&frame).expect("serialize frame");
        payload.push(b'\n');
        hyper::body::Bytes::from(payload)
    }

    #[test]
    fn ensure_world_deps_bin_sets_default_and_prepends_path() {
        let mut env_map = std::collections::HashMap::<String, String>::new();
        env_map.insert("PATH".to_string(), "/usr/bin:/bin".to_string());

        ensure_world_deps_bin_on_path(&mut env_map);

        assert_eq!(
            env_map
                .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
                .map(String::as_str),
            Some("/var/lib/substrate/world-deps/bin")
        );
        assert_eq!(
            env_map.get("PATH").map(String::as_str),
            Some("/var/lib/substrate/world-deps/bin:/usr/bin:/bin")
        );
    }

    #[test]
    fn ensure_world_deps_bin_respects_override_and_avoids_duplicates() {
        let mut env_map = std::collections::HashMap::<String, String>::new();
        env_map.insert(
            "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
            "/tmp/custom-bin/".to_string(),
        );
        env_map.insert(
            "PATH".to_string(),
            "/tmp/custom-bin:/usr/local/bin:/usr/bin".to_string(),
        );

        ensure_world_deps_bin_on_path(&mut env_map);

        assert_eq!(
            env_map
                .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
                .map(String::as_str),
            Some("/tmp/custom-bin/")
        );
        assert_eq!(
            env_map.get("PATH").map(String::as_str),
            Some("/tmp/custom-bin:/usr/local/bin:/usr/bin")
        );
    }

    #[test]
    fn preserve_world_project_dir_override_records_logical_root() {
        let prev_mode = std::env::var("SUBSTRATE_ANCHOR_MODE").ok();
        let prev_path = std::env::var("SUBSTRATE_ANCHOR_PATH").ok();
        let prev_caged = std::env::var("SUBSTRATE_CAGED").ok();

        std::env::set_var("SUBSTRATE_ANCHOR_MODE", "custom");
        std::env::set_var("SUBSTRATE_ANCHOR_PATH", "/tmp/substrate-world-root");
        std::env::set_var("SUBSTRATE_CAGED", "1");

        let mut env_map = std::collections::HashMap::<String, String>::new();
        preserve_world_project_dir_override(&mut env_map);

        assert_eq!(
            env_map
                .get(WORLD_PROJECT_DIR_OVERRIDE_ENV)
                .map(String::as_str),
            Some("/tmp/substrate-world-root")
        );

        match prev_mode {
            Some(value) => std::env::set_var("SUBSTRATE_ANCHOR_MODE", value),
            None => std::env::remove_var("SUBSTRATE_ANCHOR_MODE"),
        }
        match prev_path {
            Some(value) => std::env::set_var("SUBSTRATE_ANCHOR_PATH", value),
            None => std::env::remove_var("SUBSTRATE_ANCHOR_PATH"),
        }
        match prev_caged {
            Some(value) => std::env::set_var("SUBSTRATE_CAGED", value),
            None => std::env::remove_var("SUBSTRATE_CAGED"),
        }
    }

    #[test]
    fn current_world_request_profile_accepts_non_reserved_values() {
        with_env_var(
            "SUBSTRATE_WORLD_REQUEST_PROFILE",
            "wdap-smoke-profile",
            || {
                assert_eq!(
                    current_world_request_profile().as_deref(),
                    Some("wdap-smoke-profile")
                );
            },
        );
    }

    #[test]
    fn member_dispatch_payload_preserves_frozen_lineage_and_world_fields() {
        let payload = build_member_dispatch_payload(&MemberDispatchTransportRequest {
            orchestration_session_id: "orch_123".to_string(),
            participant_id: "ash_member_123".to_string(),
            orchestrator_participant_id: "ash_orch_123".to_string(),
            parent_participant_id: Some("ash_parent_123".to_string()),
            resumed_from_participant_id: Some("ash_prev_123".to_string()),
            backend_id: "cli:codex".to_string(),
            protocol: "uaa.agent.session".to_string(),
            run_id: "run_123".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 9,
            initial_prompt: Some("first turn".to_string()),
            backend_kind: MemberRuntimeBackendKindV1::Codex,
            binary_path: "/usr/bin/codex".to_string(),
        });

        assert_eq!(payload.schema_version, 1);
        assert_eq!(payload.orchestration_session_id, "orch_123");
        assert_eq!(payload.participant_id, "ash_member_123");
        assert_eq!(payload.orchestrator_participant_id, "ash_orch_123");
        assert_eq!(
            payload.parent_participant_id.as_deref(),
            Some("ash_parent_123")
        );
        assert_eq!(
            payload.resumed_from_participant_id.as_deref(),
            Some("ash_prev_123")
        );
        assert_eq!(payload.backend_id, "cli:codex");
        assert_eq!(payload.protocol, "uaa.agent.session");
        assert_eq!(payload.run_id, "run_123");
        assert_eq!(payload.world_id, "world_123");
        assert_eq!(payload.world_generation, 9);
        assert_eq!(payload.initial_prompt.as_deref(), Some("first turn"));
        assert_eq!(
            payload.resolved_runtime,
            ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: "/usr/bin/codex".to_string(),
            }
        );
    }

    #[test]
    fn build_execute_request_supports_typed_member_dispatch_shape() {
        let request = build_execute_request(ExecuteRequestInput {
            profile: Some("world-member-dispatch".to_string()),
            cmd: String::new(),
            cwd: "/tmp/worktree".to_string(),
            env_map: std::collections::HashMap::new(),
            agent_id: "tester".to_string(),
            policy_snapshot: PolicySnapshotV3 {
                schema_version: 3,
                net_allowed: Vec::new(),
                world_fs: PolicySnapshotWorldFsV3 {
                    host_visible: true,
                    fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                    deny_enforcement: None,
                    caged_required: false,
                    discover: None,
                    read: None,
                    write: PolicySnapshotWorldFsWriteV3 {
                        enabled: true,
                        allow_list: vec![".".to_string()],
                        deny_list: Vec::new(),
                    },
                },
            },
            world_network: WorldNetworkRoutingV1 {
                isolate_network: false,
                allowed_domains: Vec::new(),
            },
            world_fs_mode: WorldFsMode::Writable,
            member_dispatch: Some(build_member_dispatch_payload(
                &MemberDispatchTransportRequest {
                    orchestration_session_id: "orch_123".to_string(),
                    participant_id: "ash_member_123".to_string(),
                    orchestrator_participant_id: "ash_orch_123".to_string(),
                    parent_participant_id: None,
                    resumed_from_participant_id: None,
                    backend_id: "cli:codex".to_string(),
                    protocol: "uaa.agent.session".to_string(),
                    run_id: "run_123".to_string(),
                    world_id: "world_123".to_string(),
                    world_generation: 9,
                    initial_prompt: None,
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path: "/usr/bin/codex".to_string(),
                },
            )),
        });

        assert!(request.cmd.is_empty());
        assert!(!request.pty);
        assert_eq!(request.agent_id, "tester");
        assert_eq!(
            request
                .member_dispatch
                .as_ref()
                .map(|dispatch| dispatch.run_id.as_str()),
            Some("run_123")
        );
        assert_eq!(
            request
                .member_dispatch
                .as_ref()
                .map(|dispatch| dispatch.resolved_runtime.binary_path.as_str()),
            Some("/usr/bin/codex")
        );
        request
            .validate()
            .expect("typed member dispatch request validates");
    }

    #[test]
    fn current_world_request_profile_rejects_reserved_world_deps_profiles() {
        for reserved in ["world-deps-provision", "world-deps-probe"] {
            with_env_var("SUBSTRATE_WORLD_REQUEST_PROFILE", reserved, || {
                assert_eq!(
                    current_world_request_profile(),
                    None,
                    "reserved internal profile should not be forwarded from env: {reserved}"
                );
            });
        }
    }

    #[test]
    fn process_agent_stream_requests_cancel_after_start_frame() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let frames = vec![
                encode_stream_frame(ExecuteStreamFrame::Start {
                    span_id: "spn_interrupt".to_string(),
                }),
                encode_stream_frame(ExecuteStreamFrame::Exit {
                    exit: 130,
                    span_id: "spn_interrupt".to_string(),
                    scopes_used: Vec::new(),
                    fs_diff: None,
                    process_telemetry: agent_api_types::ProcessTelemetry::default(),
                }),
            ];

            let stream = stream::unfold((0usize, frames), |(idx, frames)| async move {
                match idx {
                    0 => Some((
                        Ok::<_, Infallible>(hyper::body::Frame::data(frames[0].clone())),
                        (1, frames),
                    )),
                    1 => {
                        tokio::time::sleep(Duration::from_millis(250)).await;
                        Some((
                            Ok::<_, Infallible>(hyper::body::Frame::data(frames[1].clone())),
                            (2, frames),
                        ))
                    }
                    _ => None,
                }
            });
            let body = StreamBody::new(stream);

            let (sigint_tx, mut sigint_rx) = tokio::sync::mpsc::unbounded_channel();
            let cancels = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
            let cancels_for_cancel = Arc::clone(&cancels);

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(30)).await;
                let _ = sigint_tx.send(());
            });

            let outcome = process_agent_stream_body(
                body,
                "agent".to_string(),
                None,
                &mut sigint_rx,
                move |span_id, sig| {
                    let cancels = Arc::clone(&cancels_for_cancel);
                    async move {
                        cancels.lock().expect("cancel lock").push((span_id, sig));
                        Ok(())
                    }
                },
            )
            .await
            .expect("process stream");

            assert_eq!(outcome.exit_code, 130);
            assert_eq!(
                cancels.lock().expect("cancel lock").as_slice(),
                &[("spn_interrupt".to_string(), "INT".to_string())]
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn emit_stream_chunk_some_emits_orchestration_scoped_agent_event() {
        let _guard = acquire_event_test_guard();
        let mut rx = init_event_channel();

        emit_stream_chunk(
            "agent",
            Some("orch-live"),
            "run-1",
            Some("spn-1"),
            b"hello stdout",
            false,
        );

        let event = rx.try_recv().expect("stream chunk event");
        assert_eq!(event.kind, AgentEventKind::PtyData);
        assert_eq!(event.orchestration_session_id, "orch-live");
        assert_eq!(event.run_id, "run-1");
        assert_eq!(event.span_id.as_deref(), Some("spn-1"));
        clear_agent_event_sender();
    }

    #[test]
    #[serial_test::serial]
    fn emit_stream_chunk_none_emits_no_orchestration_scoped_agent_event() {
        let _guard = acquire_event_test_guard();
        let mut rx = init_event_channel();

        emit_stream_chunk("agent", None, "run-1", Some("spn-1"), b"hello stderr", true);

        assert!(
            rx.try_recv().is_err(),
            "stream chunk without orchestration context must not emit an agent event"
        );
        clear_agent_event_sender();
    }

    #[test]
    #[serial_test::serial]
    fn process_agent_stream_body_uses_launch_owned_run_id_for_stream_rows() {
        let _guard = acquire_event_test_guard();
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let mut rx = init_event_channel();
            let frames = vec![
                encode_stream_frame(ExecuteStreamFrame::Start {
                    span_id: "spn-world".to_string(),
                }),
                encode_stream_frame(ExecuteStreamFrame::Stdout {
                    chunk_b64: BASE64.encode(b"hello world"),
                }),
                encode_stream_frame(ExecuteStreamFrame::Exit {
                    exit: 0,
                    span_id: "spn-world".to_string(),
                    scopes_used: Vec::new(),
                    fs_diff: None,
                    process_telemetry: agent_api_types::ProcessTelemetry::default(),
                }),
            ];
            let body = StreamBody::new(stream::iter(
                frames
                    .into_iter()
                    .map(|frame| Ok::<_, Infallible>(hyper::body::Frame::data(frame))),
            ));
            let mut sigint_rx = tokio::sync::mpsc::unbounded_channel().1;
            let context = ShellCommandEventContext::new(
                ShellEventEmissionContext {
                    orchestration_session_id: "orch-live".to_string(),
                    agent_id: "shell".to_string(),
                    role: Some("orchestrator".to_string()),
                    backend_id: Some("shell:repl".to_string()),
                    participant_id: Some("participant-1".to_string()),
                    parent_participant_id: None,
                    resumed_from_participant_id: None,
                    world_id: Some("world-1".to_string()),
                    world_generation: Some(4),
                },
                "cmd-123",
                Some("cmd-123".to_string()),
                Some("spn-shell".to_string()),
            );

            let outcome = process_agent_stream_body(
                body,
                "agent".to_string(),
                Some(&context),
                &mut sigint_rx,
                |_span_id, _sig| async { Ok(()) },
            )
            .await
            .expect("process stream");

            assert_eq!(outcome.exit_code, 0);
            let event = rx.try_recv().expect("stream chunk event");
            assert_eq!(event.kind, AgentEventKind::PtyData);
            assert_eq!(event.orchestration_session_id, "orch-live");
            assert_eq!(event.run_id, "cmd-123");
            assert_eq!(event.span_id.as_deref(), Some("spn-world"));
        });
        clear_agent_event_sender();
    }

    #[test]
    fn extract_process_telemetry_from_ws_exit_preserves_ptrace_not_permitted_reason() {
        let exit = json!({
            "type": "exit",
            "exit": 0,
            "span_id": "spn_ptrace_denied",
            "scopes_used": [],
            "process_events": [],
            "process_events_status": "unavailable",
            "process_events_reason": "ptrace_not_permitted"
        });

        let process_telemetry = extract_process_telemetry_from_ws_exit(&exit);

        assert_eq!(
            process_telemetry.process_events_status,
            substrate_common::ProcessEventsStatus::Unavailable
        );
        assert_eq!(
            process_telemetry.process_events_reason.as_deref(),
            Some("ptrace_not_permitted")
        );
        assert!(process_telemetry.process_events.is_empty());
        assert!(process_telemetry.process_events_dropped.is_none());
    }

    #[test]
    fn extract_process_telemetry_from_ws_exit_preserves_linux_process_event_fields() {
        let exit = json!({
            "type": "exit",
            "exit": 0,
            "span_id": "spn_parent",
            "scopes_used": [],
            "process_events": [
                {
                    "ts": "2026-04-01T00:00:00Z",
                    "ts_unix_ns": 1_743_465_600_000_000_000u64,
                    "event_type": "world_process_start",
                    "session_id": "ses_linux",
                    "world_id": "wld_linux",
                    "pid": 42,
                    "ppid": 1,
                    "cwd": "/project",
                    "parent_span": "spn_parent",
                    "parent_cmd_id": "cmd_parent",
                    "argv_omitted": true
                },
                {
                    "ts": "2026-04-01T00:00:01Z",
                    "ts_unix_ns": 1_743_465_601_000_000_000u64,
                    "event_type": "world_process_exit",
                    "session_id": "ses_linux",
                    "world_id": "wld_linux",
                    "pid": 42,
                    "ppid": 1,
                    "cwd": "/project",
                    "parent_span": "spn_parent",
                    "parent_cmd_id": "cmd_parent",
                    "argv_omitted": true,
                    "exit_code": 0,
                    "duration_ms": 11
                }
            ],
            "process_events_status": "truncated",
            "process_events_reason": "capture_overflow",
            "process_events_dropped": 7,
            "process_events_max": 10000,
            "process_events_backend": "ptrace"
        });

        let process_telemetry = extract_process_telemetry_from_ws_exit(&exit);

        assert_eq!(
            process_telemetry.process_events_status,
            substrate_common::ProcessEventsStatus::Truncated
        );
        assert_eq!(
            process_telemetry.process_events_reason.as_deref(),
            Some("capture_overflow")
        );
        assert_eq!(process_telemetry.process_events_dropped, Some(7));
        assert_eq!(process_telemetry.process_events_max, Some(10_000));
        assert_eq!(
            process_telemetry.process_events_backend.as_deref(),
            Some("ptrace")
        );
        assert_eq!(process_telemetry.process_events.len(), 2);
        assert_eq!(process_telemetry.process_events[0].argv_omitted, Some(true));
        assert_eq!(
            process_telemetry.process_events[0].parent_span,
            "spn_parent"
        );
        assert_eq!(
            process_telemetry.process_events[0].parent_cmd_id.as_deref(),
            Some("cmd_parent")
        );
        assert_eq!(process_telemetry.process_events[1].exit_code, Some(0));
        assert_eq!(process_telemetry.process_events[1].duration_ms, Some(11));
    }
}
