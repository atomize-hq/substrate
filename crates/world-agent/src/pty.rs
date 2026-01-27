//! PTY WebSocket handler for world-agent implementing JSON frame protocol

use crate::service::WorldAgentService;
#[cfg(target_os = "linux")]
use crate::service::{
    is_full_isolation, resolve_landlock_allowlist_paths, resolve_project_dir,
    resolve_project_write_allowlist_prefixes, WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV,
    WORLD_FS_LANDLOCK_WRITE_ALLOWLIST_ENV, WORLD_FS_MODE_ENV, WORLD_FS_WRITE_ALLOWLIST_ENV,
};
use agent_api_types::PolicySnapshotV1;
#[cfg(target_os = "linux")]
use agent_api_types::{PolicyResolutionModeV1, PolicySnapshotWorldFsIsolationV1};
use axum::extract::ws::{Message, WebSocket};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
#[cfg(target_os = "linux")]
use once_cell::sync::OnceCell;
use portable_pty::*;
#[cfg(target_os = "linux")]
use rand::rngs::OsRng;
#[cfg(target_os = "linux")]
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
#[cfg(target_os = "linux")]
use tracing::warn;
use tracing::{error, info};
// no atomic imports needed here
#[cfg(target_os = "linux")]
use world::exec::PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT;
#[cfg(target_os = "linux")]
use world::guard::{should_guard_anchor, wrap_with_anchor_guard};
#[cfg(target_os = "linux")]
use world_api::WorldFsMode;

fn ensure_xdg_dirs(env: &mut HashMap<String, String>) {
    // Some minimal images don't ship with pre-created XDG dirs (e.g. /root/.local/share),
    // and TUIs like `nano` expect them to exist.
    let home = env
        .get("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    let data_home = env
        .get("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local").join("share"));

    if std::fs::create_dir_all(&data_home).is_err() {
        // If the caller provided a HOME/XDG path that doesn't exist in-world (e.g. host HOME),
        // fall back to a stable, writable location.
        let fallback = PathBuf::from("/tmp/substrate-xdg");
        let _ = std::fs::create_dir_all(&fallback);
        env.insert(
            "XDG_DATA_HOME".to_string(),
            fallback.to_string_lossy().to_string(),
        );
    }
}

#[cfg(unix)]
fn parse_signal(sig: &str) -> Option<i32> {
    match sig {
        "INT" | "SIGINT" => Some(libc::SIGINT),
        "TERM" | "SIGTERM" => Some(libc::SIGTERM),
        "QUIT" | "SIGQUIT" => Some(libc::SIGQUIT),
        "HUP" | "SIGHUP" => Some(libc::SIGHUP),
        _ => None,
    }
}

#[cfg(unix)]
fn forward_signal(child_pid: Option<i32>, sig: &str) {
    if let (Some(pid), Some(signo)) = (child_pid, parse_signal(sig)) {
        // Safety: libc::kill is async-signal-safe and we are not accessing shared data
        unsafe { libc::kill(pid as libc::pid_t, signo) };
        info!("ws_pty: forwarded signal {} to pid {}", sig, pid);
    }
}

#[cfg(not(unix))]
fn forward_signal(_child_pid: Option<i32>, _sig: &str) {}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "start")]
    Start {
        cmd: String,
        cwd: PathBuf,
        env: HashMap<String, String>,
        #[serde(default)]
        policy_snapshot: Box<Option<PolicySnapshotV1>>,
        span_id: Option<String>,
        cols: u16,
        rows: u16,
    },
    #[serde(rename = "stdin")]
    Stdin { data_b64: String },
    #[serde(rename = "resize")]
    Resize { cols: u16, rows: u16 },
    #[serde(rename = "signal")]
    Signal { sig: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "stdout")]
    Stdout { data_b64: String },
    #[serde(rename = "exit")]
    Exit { code: i32 },
    #[serde(rename = "error")]
    Error { message: String },
}

type WsSender = SplitSink<WebSocket, Message>;
type WsReceiver = SplitStream<WebSocket>;

async fn send_ws_message(tx: &Arc<Mutex<WsSender>>, msg: &ServerMessage) -> Result<(), ()> {
    let text = serde_json::to_string(msg).map_err(|err| {
        error!(%err, "ws_pty: failed to serialize server message");
    })?;

    tx.lock()
        .await
        .send(Message::Text(text))
        .await
        .map_err(|err| {
            error!(%err, "ws_pty: failed to send server message");
        })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg(target_os = "linux")]
enum PersistentClientMessage {
    StartSession {
        #[serde(default)]
        protocol_version: Option<u32>,
        cwd: PathBuf,
        env: HashMap<String, String>,
        policy_snapshot: PolicySnapshotV1,
        cols: u16,
        rows: u16,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PersistentServerMessage {
    Ready {
        session_nonce: String,
        cwd: PathBuf,
        protocol_version: u32,
    },
    Exit {
        code: i32,
    },
    Error {
        code: String,
        message: String,
        fatal: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        seq: Option<u64>,
    },
}

async fn send_persistent_ws_message(
    tx: &Arc<Mutex<WsSender>>,
    msg: &PersistentServerMessage,
) -> Result<(), ()> {
    let text = serde_json::to_string(msg).map_err(|err| {
        error!(%err, "ws_pty: failed to serialize persistent server message");
    })?;

    tx.lock()
        .await
        .send(Message::Text(text))
        .await
        .map_err(|err| {
            error!(%err, "ws_pty: failed to send persistent server message");
        })
}

#[cfg(target_os = "linux")]
fn hex32(bytes: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 64];
    for (i, b) in bytes.iter().enumerate() {
        out[i * 2] = HEX[(b >> 4) as usize];
        out[i * 2 + 1] = HEX[(b & 0x0f) as usize];
    }
    // Safety: HEX table is ASCII.
    std::str::from_utf8(&out)
        .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000")
        .to_string()
}

#[cfg(target_os = "linux")]
fn generate_session_nonce() -> String {
    let mut raw = [0u8; 32];
    OsRng.fill_bytes(&mut raw);
    hex32(raw)
}

#[cfg(target_os = "linux")]
fn sanitize_session_env(env: &mut HashMap<String, String>) {
    for key in [
        "SHIM_ACTIVE",
        "SHIM_CALLER",
        "SHIM_CALL_STACK",
        "SHIM_DEPTH",
    ] {
        env.remove(key);
    }

    // Suppress in-world prompts; the REPL prompt is host-side.
    env.insert("PS1".to_string(), "".to_string());
    env.insert("PS2".to_string(), "".to_string());
    env.insert("PROMPT_COMMAND".to_string(), "".to_string());
}

#[cfg(target_os = "linux")]
fn validate_pty_watermark_query_supported(master: &dyn MasterPty) -> Result<(), std::io::Error> {
    let fd = master.as_raw_fd().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "PTY master does not expose a raw fd",
        )
    })?;
    let mut bytes_readable: libc::c_int = 0;
    // Safety: FIONREAD expects a pointer to int.
    let rc = unsafe { libc::ioctl(fd, libc::FIONREAD, &mut bytes_readable) };
    if rc == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn validate_control_plane_privacy_precondition() -> Result<(), std::io::Error> {
    static PREFLIGHT: OnceCell<Result<(), String>> = OnceCell::new();
    match PREFLIGHT.get_or_init(control_plane_privacy_preflight_impl) {
        Ok(()) => Ok(()),
        Err(message) => Err(std::io::Error::other(message.clone())),
    }
}

#[cfg(target_os = "linux")]
fn control_plane_privacy_preflight_impl() -> Result<(), String> {
    let dev_null = std::ffi::CString::new("/dev/null").map_err(|e| e.to_string())?;
    // Safety: `dev_null` is a valid C string. We close this fd before returning.
    let mut leak_fd = unsafe { libc::open(dev_null.as_ptr(), libc::O_RDONLY) };
    if leak_fd < 0 {
        return Err(format!(
            "open(/dev/null) failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    if leak_fd < 3 {
        // Safety: fcntl calls with valid fd; best-effort cleanup on failure.
        let dup_fd = unsafe { libc::fcntl(leak_fd, libc::F_DUPFD, 3) };
        if dup_fd < 0 {
            unsafe { libc::close(leak_fd) };
            return Err(format!(
                "fcntl(F_DUPFD) failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        unsafe { libc::close(leak_fd) };
        leak_fd = dup_fd;
    }

    // Safety: fcntl calls with valid fd.
    let flags = unsafe { libc::fcntl(leak_fd, libc::F_GETFD) };
    if flags < 0 {
        // Safety: best-effort cleanup.
        unsafe { libc::close(leak_fd) };
        return Err(format!(
            "fcntl(F_GETFD) failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let rc = unsafe { libc::fcntl(leak_fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) };
    if rc < 0 {
        // Safety: best-effort cleanup.
        unsafe { libc::close(leak_fd) };
        return Err(format!(
            "fcntl(F_SETFD) failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(
        r#"if [ ! -d /proc/self/fd ]; then exit 2; fi
if [ -e "/proc/self/fd/$SUBSTRATE_PREFLIGHT_LEAK_FD" ]; then exit 3; fi
printf '%s' ok"#,
    );
    cmd.env("SUBSTRATE_PREFLIGHT_LEAK_FD", leak_fd.to_string());
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                let open_max = libc::sysconf(libc::_SC_OPEN_MAX);
                let max_fd = if open_max > 0 && open_max <= i64::from(i32::MAX) {
                    open_max as i32
                } else {
                    1024
                };

                for fd in 3..max_fd {
                    libc::close(fd);
                }

                Ok(())
            });
        }
    }

    let output = cmd
        .output()
        .map_err(|e| format!("preflight spawn failed: {e}"))?;

    // Safety: best-effort cleanup.
    unsafe { libc::close(leak_fd) };

    if !output.status.success() {
        return Err(format!(
            "preflight failed (exit={:?}, stdout={}, stderr={})",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout != "ok" {
        return Err(format!("preflight failed (unexpected stdout={stdout:?})"));
    }

    Ok(())
}

pub async fn handle_ws_pty(
    #[cfg_attr(not(target_os = "linux"), allow(unused_variables))] service: WorldAgentService,
    ws: WebSocket,
) {
    info!("ws_pty: client connected");
    let (tx, mut rx) = ws.split();
    let tx = Arc::new(Mutex::new(tx));

    // Wait for initial message: either legacy `start` or persistent `start_session`.
    let first_text = match rx.next().await {
        Some(Ok(Message::Text(text))) => text,
        Some(Ok(_)) => {
            let _ = send_persistent_ws_message(
                &tx,
                &PersistentServerMessage::Error {
                    code: "bad_request".to_string(),
                    message: "Expected text message".to_string(),
                    fatal: true,
                    seq: None,
                },
            )
            .await;
            return;
        }
        Some(Err(e)) => {
            let _ = send_persistent_ws_message(
                &tx,
                &PersistentServerMessage::Error {
                    code: "internal_error".to_string(),
                    message: format!("WebSocket error: {e}"),
                    fatal: true,
                    seq: None,
                },
            )
            .await;
            return;
        }
        None => return,
    };

    let msg_type = match serde_json::from_str::<serde_json::Value>(&first_text)
        .ok()
        .and_then(|v| {
            v.get("type")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        }) {
        Some(t) => t,
        None => {
            let _ = send_persistent_ws_message(
                &tx,
                &PersistentServerMessage::Error {
                    code: "bad_request".to_string(),
                    message: "Invalid JSON: missing or non-string `type`".to_string(),
                    fatal: true,
                    seq: None,
                },
            )
            .await;
            return;
        }
    };

    match msg_type.as_str() {
        "start" => {
            handle_legacy_start(service, tx, rx, first_text).await;
        }
        "start_session" => {
            handle_persistent_session(service, tx, rx, first_text).await;
        }
        _ => {
            let _ = send_persistent_ws_message(
                &tx,
                &PersistentServerMessage::Error {
                    code: "bad_request".to_string(),
                    message: "First frame must be `start` or `start_session`".to_string(),
                    fatal: true,
                    seq: None,
                },
            )
            .await;
        }
    }
}

#[cfg(not(target_os = "linux"))]
async fn handle_persistent_session(
    _service: WorldAgentService,
    tx: Arc<Mutex<WsSender>>,
    _rx: WsReceiver,
    _first_text: String,
) {
    let _ = send_persistent_ws_message(
        &tx,
        &PersistentServerMessage::Error {
            code: "internal_error".to_string(),
            message: "Persistent sessions are only supported on Linux world-agent".to_string(),
            fatal: true,
            seq: None,
        },
    )
    .await;
}

#[cfg(target_os = "linux")]
async fn handle_persistent_session(
    _service: WorldAgentService,
    tx: Arc<Mutex<WsSender>>,
    mut rx: WsReceiver,
    first_text: String,
) {
    let start = match serde_json::from_str::<PersistentClientMessage>(&first_text) {
        Ok(PersistentClientMessage::StartSession {
            protocol_version,
            cwd,
            env,
            policy_snapshot,
            cols,
            rows,
        }) => (
            protocol_version.unwrap_or(1),
            cwd,
            env,
            policy_snapshot,
            cols,
            rows,
        ),
        Err(e) => {
            let _ = send_persistent_ws_message(
                &tx,
                &PersistentServerMessage::Error {
                    code: "bad_request".to_string(),
                    message: format!("Invalid JSON: {e}"),
                    fatal: true,
                    seq: None,
                },
            )
            .await;
            return;
        }
    };

    let (requested_protocol_version, requested_cwd, mut session_env, policy_snapshot, cols, rows) =
        start;

    if requested_protocol_version != 1 {
        let _ = send_persistent_ws_message(
            &tx,
            &PersistentServerMessage::Error {
                code: "unsupported_protocol_version".to_string(),
                message: format!(
                    "Unsupported protocol_version: {requested_protocol_version} (expected 1)"
                ),
                fatal: true,
                seq: None,
            },
        )
        .await;
        return;
    }

    if policy_snapshot.schema_version != 1 {
        let _ = send_persistent_ws_message(
            &tx,
            &PersistentServerMessage::Error {
                code: "bad_request".to_string(),
                message: format!(
                    "Invalid policy_snapshot.schema_version: {}",
                    policy_snapshot.schema_version
                ),
                fatal: true,
                seq: None,
            },
        )
        .await;
        return;
    }

    sanitize_session_env(&mut session_env);
    ensure_xdg_dirs(&mut session_env);

    // DR-23 preflight: watermark-query capability for Session PTY (v1 requires FIONREAD).
    let pty_system = native_pty_system();
    let pair = match pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(pair) => pair,
        Err(e) => {
            let _ = send_persistent_ws_message(
                &tx,
                &PersistentServerMessage::Error {
                    code: "internal_error".to_string(),
                    message: format!("Failed to create session PTY: {e}"),
                    fatal: true,
                    seq: None,
                },
            )
            .await;
            return;
        }
    };

    if let Err(e) = validate_pty_watermark_query_supported(&*pair.master) {
        let _ = send_persistent_ws_message(
            &tx,
            &PersistentServerMessage::Error {
                code: "internal_error".to_string(),
                message: format!("PTY watermark query unsupported: {e}"),
                fatal: true,
                seq: None,
            },
        )
        .await;
        return;
    }

    // DR-22 preflight: ensure we can enforce a minimal evaluator FD table.
    if let Err(e) = validate_control_plane_privacy_precondition() {
        let _ = send_persistent_ws_message(
            &tx,
            &PersistentServerMessage::Error {
                code: "internal_error".to_string(),
                message: format!("Control-plane privacy precondition failed: {e}"),
                fatal: true,
                seq: None,
            },
        )
        .await;
        return;
    }

    let ready_cwd = match resolve_ready_cwd(&session_env, requested_cwd.as_path(), &policy_snapshot)
    {
        Ok(cwd) => cwd,
        Err(message) => {
            let _ = send_persistent_ws_message(
                &tx,
                &PersistentServerMessage::Error {
                    code: "internal_error".to_string(),
                    message,
                    fatal: true,
                    seq: None,
                },
            )
            .await;
            return;
        }
    };
    let session_nonce = generate_session_nonce();

    let _ = send_persistent_ws_message(
        &tx,
        &PersistentServerMessage::Ready {
            session_nonce,
            cwd: ready_cwd,
            protocol_version: 1,
        },
    )
    .await;

    // C0 scope: accept close and keep the session alive; exec/command_complete is C1.
    loop {
        let Some(msg) = rx.next().await else { break };
        match msg {
            Ok(Message::Text(text)) => {
                let msg_type = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| {
                        v.get("type")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string())
                    });

                match msg_type.as_deref() {
                    Some("close") => {
                        let _ = send_persistent_ws_message(
                            &tx,
                            &PersistentServerMessage::Exit { code: 0 },
                        )
                        .await;
                        break;
                    }
                    Some("resize") | Some("stdin") | Some("signal") => {
                        continue;
                    }
                    Some(_) => {
                        let _ = send_persistent_ws_message(
                            &tx,
                            &PersistentServerMessage::Error {
                                code: "protocol_violation".to_string(),
                                message: "Unsupported frame for C0 persistent session".to_string(),
                                fatal: true,
                                seq: None,
                            },
                        )
                        .await;
                        break;
                    }
                    None => {
                        let _ = send_persistent_ws_message(
                            &tx,
                            &PersistentServerMessage::Error {
                                code: "bad_request".to_string(),
                                message: "Invalid JSON frame".to_string(),
                                fatal: true,
                                seq: None,
                            },
                        )
                        .await;
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(_) => break,
        }
    }

    drop(pair);
    info!("ws_pty: persistent session closed");
}

#[cfg(target_os = "linux")]
fn resolve_ready_cwd(
    env: &HashMap<String, String>,
    requested_cwd: &std::path::Path,
    _policy_snapshot: &PolicySnapshotV1,
) -> Result<PathBuf, String> {
    let project_dir =
        resolve_project_dir(Some(env), Some(requested_cwd)).map_err(|e| e.to_string())?;

    if requested_cwd.starts_with(&project_dir)
        && requested_cwd.is_absolute()
        && requested_cwd.is_dir()
    {
        Ok(requested_cwd.to_path_buf())
    } else {
        Ok(project_dir)
    }
}

async fn handle_legacy_start(
    #[cfg_attr(not(target_os = "linux"), allow(unused_variables))] service: WorldAgentService,
    tx: Arc<Mutex<WsSender>>,
    mut rx: WsReceiver,
    first_text: String,
) {
    let start_msg = match serde_json::from_str::<ClientMessage>(&first_text) {
        Ok(ClientMessage::Start {
            cmd,
            cwd,
            env,
            policy_snapshot,
            span_id,
            cols,
            rows,
        }) => {
            let mut env = env;
            ensure_xdg_dirs(&mut env);
            let policy_snapshot = *policy_snapshot;
            info!(
                %cmd,
                cwd = %cwd.display(),
                span_id = span_id.as_deref().unwrap_or("-"),
                cols = cols,
                rows = rows,
                "ws_pty: start"
            );
            (cmd, cwd, env, policy_snapshot, span_id, cols, rows)
        }
        Ok(_) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: "Expected start message".to_string(),
                },
            )
            .await;
            return;
        }
        Err(e) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: format!("Invalid JSON: {}", e),
                },
            )
            .await;
            return;
        }
    };

    let (cmd, cwd, env_map, policy_snapshot, _span_id, cols, rows) = start_msg;
    #[cfg(not(target_os = "linux"))]
    let _policy_snapshot = policy_snapshot;
    #[cfg(target_os = "linux")]
    let mut env = env_map;
    #[cfg(not(target_os = "linux"))]
    let env = env_map;
    #[cfg(target_os = "linux")]
    let mut inner_cmd = cmd.clone();
    #[cfg(target_os = "linux")]
    let command_to_run: String;
    #[cfg(not(target_os = "linux"))]
    let command_to_run = cmd.clone();
    #[cfg(target_os = "linux")]
    let cwd_for_child: PathBuf;
    #[cfg(not(target_os = "linux"))]
    let cwd_for_child = cwd.clone();
    #[cfg(target_os = "linux")]
    let anchor_root: PathBuf;

    // Prepare in-world session context (best-effort)
    #[cfg(target_os = "linux")]
    let world_id_for_logs: Option<String>;
    #[cfg(not(target_os = "linux"))]
    let world_id_for_logs: Option<String> = None;

    #[cfg(target_os = "linux")]
    let mut ns_name_opt: Option<String> = None;
    #[cfg(not(target_os = "linux"))]
    let ns_name_opt: Option<String> = None;

    #[cfg(target_os = "linux")]
    let cgroup_path_opt: Option<std::path::PathBuf>;
    #[cfg(not(target_os = "linux"))]
    let cgroup_path_opt: Option<std::path::PathBuf> = None;

    #[cfg(target_os = "linux")]
    let in_world: bool;
    #[cfg(not(target_os = "linux"))]
    let in_world = false;
    #[cfg(target_os = "linux")]
    let fs_strategy_meta: Option<world::overlayfs::WorldFsStrategyMeta>;
    #[cfg(target_os = "linux")]
    {
        use world_api::{ResourceLimits, WorldSpec};
        let project_dir = match resolve_project_dir(Some(&env), Some(&cwd)) {
            Ok(dir) => dir,
            Err(err) => {
                let _ = send_ws_message(
                    &tx,
                    &ServerMessage::Error {
                        message: format!("Failed to resolve world root: {}", err),
                    },
                )
                .await;
                return;
            }
        };
        let (policy_resolution_mode, isolation_full, fs_mode, allowed_domains) =
            if let Some(snapshot) = policy_snapshot.as_ref() {
                if snapshot.schema_version != 1 {
                    let _ = send_ws_message(
                        &tx,
                        &ServerMessage::Error {
                            message: format!(
                                "Invalid policy_snapshot.schema_version: {}",
                                snapshot.schema_version
                            ),
                        },
                    )
                    .await;
                    return;
                }

                let isolation_full = matches!(
                    snapshot.world_fs.isolation,
                    PolicySnapshotWorldFsIsolationV1::Full
                );
                (
                    PolicyResolutionModeV1::SnapshotV1,
                    isolation_full,
                    snapshot.world_fs.mode,
                    snapshot.net_allowed.clone(),
                )
            } else {
                let isolation_full = is_full_isolation(Some(&env));
                let fs_mode = env
                    .get(WORLD_FS_MODE_ENV)
                    .and_then(|value| WorldFsMode::parse(value))
                    .unwrap_or(WorldFsMode::Writable);
                if let Err(e) = substrate_broker::detect_profile(&cwd) {
                    warn!(
                        error = %e,
                        cwd = %cwd.display(),
                        "world-agent: failed to detect policy profile for PTY request"
                    );
                }
                (
                    PolicyResolutionModeV1::LegacyLocal,
                    isolation_full,
                    fs_mode,
                    substrate_broker::allowed_domains(),
                )
            };

        service.set_last_policy_resolution_mode(policy_resolution_mode);

        if isolation_full {
            let (prefixes, landlock_read_paths, landlock_write_paths) =
                if let Some(snapshot) = policy_snapshot.as_ref() {
                    (
                        resolve_project_write_allowlist_prefixes(
                            &project_dir,
                            &snapshot.world_fs.write_allowlist,
                        ),
                        resolve_landlock_allowlist_paths(
                            &project_dir,
                            &snapshot.world_fs.read_allowlist,
                        ),
                        resolve_landlock_allowlist_paths(
                            &project_dir,
                            &snapshot.world_fs.write_allowlist,
                        ),
                    )
                } else {
                    let world_fs = substrate_broker::world_fs_policy();
                    (
                        resolve_project_write_allowlist_prefixes(
                            &project_dir,
                            &world_fs.write_allowlist,
                        ),
                        resolve_landlock_allowlist_paths(&project_dir, &world_fs.read_allowlist),
                        resolve_landlock_allowlist_paths(&project_dir, &world_fs.write_allowlist),
                    )
                };

            if !prefixes.is_empty() {
                env.insert(
                    WORLD_FS_WRITE_ALLOWLIST_ENV.to_string(),
                    prefixes.join("\n"),
                );
            }
            let landlock_supported = world::landlock::detect_support().supported;
            let landlock_env_needed = landlock_supported
                && (!landlock_read_paths.is_empty() || !landlock_write_paths.is_empty());
            if landlock_env_needed {
                if !landlock_read_paths.is_empty() {
                    env.insert(
                        WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV.to_string(),
                        landlock_read_paths.join("\n"),
                    );
                }
                if !landlock_write_paths.is_empty() {
                    env.insert(
                        WORLD_FS_LANDLOCK_WRITE_ALLOWLIST_ENV.to_string(),
                        landlock_write_paths.join("\n"),
                    );
                }
                if let Ok(exe) = std::env::current_exe() {
                    env.entry("SUBSTRATE_LANDLOCK_HELPER_SRC".to_string())
                        .or_insert_with(|| exe.display().to_string());
                }
            }
        }

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains,
            project_dir: project_dir.clone(),
            always_isolate: true,
            fs_mode,
        };

        let merged_dir: PathBuf;
        match service.ensure_session_overlay_root(&spec) {
            Ok((world, merged)) => {
                world_id_for_logs = Some(world.id.clone());
                fs_strategy_meta = world::overlayfs::world_fs_strategy_meta(&world.id);
                let ns_name = format!("substrate-{}", world.id);
                let ns_path = format!("/var/run/netns/{}", ns_name);
                if std::path::Path::new(&ns_path).exists() {
                    ns_name_opt = Some(ns_name);
                }
                let cg = std::path::PathBuf::from("/sys/fs/cgroup/substrate").join(&world.id);
                cgroup_path_opt = Some(cg);
                in_world = true;
                merged_dir = merged;
            }
            Err(err) => {
                let _ = send_ws_message(
                    &tx,
                    &ServerMessage::Error {
                        message: format!("Failed to prepare world overlay: {}", err),
                    },
                )
                .await;
                return;
            }
        }

        // Avoid inode escapes: don't start the PTY child inside the project dir before the bind mount.
        cwd_for_child = PathBuf::from("/");
        anchor_root = project_dir.clone();

        let desired_cwd = if cwd.starts_with(&project_dir) {
            cwd.clone()
        } else {
            project_dir.clone()
        };
        env.insert(
            "SUBSTRATE_MOUNT_MERGED_DIR".to_string(),
            merged_dir.display().to_string(),
        );
        env.insert(
            "SUBSTRATE_MOUNT_PROJECT_DIR".to_string(),
            project_dir.display().to_string(),
        );
        env.insert(
            "SUBSTRATE_MOUNT_CWD".to_string(),
            desired_cwd.display().to_string(),
        );
        env.insert(
            "SUBSTRATE_MOUNT_FS_MODE".to_string(),
            fs_mode.as_str().to_string(),
        );

        if should_guard_anchor(&env) {
            inner_cmd = wrap_with_anchor_guard(&inner_cmd, &anchor_root);
        }
        env.insert("SUBSTRATE_INNER_CMD".to_string(), inner_cmd);
        env.insert("SUBSTRATE_INNER_LOGIN_SHELL".to_string(), "1".to_string());

        env.insert("HOME".to_string(), "/tmp/substrate-home".to_string());
        env.insert(
            "XDG_CACHE_HOME".to_string(),
            "/tmp/substrate-xdg/cache".to_string(),
        );
        env.insert(
            "XDG_CONFIG_HOME".to_string(),
            "/tmp/substrate-xdg/config".to_string(),
        );
        env.insert(
            "XDG_DATA_HOME".to_string(),
            "/tmp/substrate-xdg/data".to_string(),
        );

        command_to_run = PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT.to_string();
    }

    // Create PTY
    let pty_system = native_pty_system();
    let pair = match pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(pair) => pair,
        Err(e) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: format!("Failed to create PTY: {}", e),
                },
            )
            .await;
            return;
        }
    };

    #[cfg(target_os = "linux")]
    let mut cmd_builder = {
        let needs_userns = unsafe { libc::geteuid() != 0 };

        if let Some(ref ns_name) = ns_name_opt {
            let mut builder = CommandBuilder::new("ip");
            builder.arg("netns");
            builder.arg("exec");
            builder.arg(ns_name);
            builder.arg("unshare");
            builder.arg("--mount");
            builder.arg("--fork");
            if needs_userns {
                builder.arg("--user");
                builder.arg("--map-root-user");
            }
            builder.arg("--");
            builder.arg("sh");
            builder.arg("-c");
            builder.arg(&command_to_run);
            builder
        } else {
            let mut builder = CommandBuilder::new("unshare");
            builder.arg("--mount");
            builder.arg("--fork");
            if needs_userns {
                builder.arg("--user");
                builder.arg("--map-root-user");
            }
            builder.arg("--");
            builder.arg("sh");
            builder.arg("-c");
            builder.arg(&command_to_run);
            builder
        }
    };
    #[cfg(not(target_os = "linux"))]
    let mut cmd_builder = {
        let mut builder = CommandBuilder::new("sh");
        builder.args(["-lc", &command_to_run]);
        builder
    };
    cmd_builder.cwd(&cwd_for_child);
    for (key, value) in env {
        cmd_builder.env(key, value);
    }

    let mut child = match pair.slave.spawn_command(cmd_builder) {
        Ok(child) => child,
        Err(e) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: format!("Failed to spawn command: {}", e),
                },
            )
            .await;
            return;
        }
    };

    // Cache child PID for signal handling
    let child_pid: Option<i32> = child.process_id().map(|p| p as i32);

    // Attach child to world cgroup (best-effort)
    #[cfg(target_os = "linux")]
    if let (Some(pid), Some(ref cg)) = (child.process_id(), cgroup_path_opt.as_ref()) {
        let _ = std::fs::create_dir_all(cg);
        let _ = std::fs::write(cg.join("cgroup.procs"), pid.to_string());
    }

    drop(pair.slave);

    // Log start with in-world context
    let world_id_log = world_id_for_logs.as_deref().unwrap_or("-");
    info!(
        world_id = %world_id_log,
        ns = %ns_name_opt.clone().unwrap_or_else(|| "-".into()),
        cgroup = %cgroup_path_opt
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".into()),
        cols = cols,
        rows = rows,
        in_world = in_world,
        "ws_pty: start"
    );

    // Reader task: forward PTY output to WebSocket
    let reader = match pair.master.try_clone_reader() {
        Ok(reader) => reader,
        Err(e) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: format!("Failed to clone PTY reader: {}", e),
                },
            )
            .await;
            return;
        }
    };
    let tx_clone = tx.clone();
    let reader_task = tokio::spawn(async move {
        let mut reader = reader;
        let mut buf = [0u8; 8192];
        loop {
            let read_result = tokio::task::spawn_blocking(move || {
                let result = reader.read(&mut buf);
                (reader, buf, result)
            })
            .await;

            match read_result {
                Ok((r, b, Ok(n))) if n > 0 => {
                    reader = r;
                    buf = b;
                    let data_b64 = BASE64.encode(&buf[..n]);
                    let msg = ServerMessage::Stdout { data_b64 };
                    if send_ws_message(&tx_clone, &msg).await.is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    });

    // Get writer once using take_writer
    let writer = match pair.master.take_writer() {
        Ok(writer) => writer,
        Err(e) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: format!("Failed to take PTY writer: {}", e),
                },
            )
            .await;
            return;
        }
    };

    // Keep master for resize operations
    let master = Arc::new(Mutex::new(pair.master));
    let master_clone = master.clone();

    // Writer task: handle WebSocket messages
    let input_task = tokio::spawn(async move {
        let mut writer = writer;
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(ClientMessage::Stdin { data_b64 }) => {
                            match BASE64.decode(&data_b64) {
                                Ok(data) => {
                                    let write_result = tokio::task::spawn_blocking(move || {
                                        let result = writer.write_all(&data);
                                        (writer, result)
                                    })
                                    .await;

                                    match write_result {
                                        Ok((w, Ok(_))) => {
                                            writer = w;
                                        }
                                        _ => break,
                                    }
                                }
                                Err(_) => continue, // Ignore invalid base64
                            }
                        }
                        Ok(ClientMessage::Resize { cols, rows }) => {
                            let size = PtySize {
                                rows,
                                cols,
                                pixel_width: 0,
                                pixel_height: 0,
                            };
                            let _ = master_clone.lock().await.resize(size);
                        }
                        Ok(ClientMessage::Signal { sig }) => {
                            // Forward signal to child process if available (platform-specific)
                            forward_signal(child_pid, &sig);
                        }
                        _ => {} // Ignore other message types
                    }
                }
                Ok(Message::Close(_)) => break,
                _ => {} // Ignore other message types
            }
        }
    });

    // Wait for child process to exit
    let status = tokio::task::spawn_blocking(move || child.wait())
        .await
        .unwrap_or_else(|_| Ok(portable_pty::ExitStatus::with_exit_code(1)));

    // Send exit message
    let exit_code = status.map(|s| s.exit_code() as i32).unwrap_or(1);
    info!(exit_code, "ws_pty: exit");
    #[cfg(target_os = "linux")]
    let (primary, final_strategy, reason) = match fs_strategy_meta.as_ref() {
        Some(meta) => (
            Some(meta.primary.as_str().to_string()),
            Some(meta.final_strategy.as_str().to_string()),
            Some(meta.fallback_reason.as_str().to_string()),
        ),
        None => (None, None, None),
    };
    #[cfg(not(target_os = "linux"))]
    let (primary, final_strategy, reason): (Option<String>, Option<String>, Option<String>) =
        (None, None, None);
    let exit_payload = serde_json::json!({
        "type": "exit",
        "code": exit_code,
        "world_fs_strategy_primary": primary,
        "world_fs_strategy_final": final_strategy,
        "world_fs_strategy_fallback_reason": reason,
    });
    let _ = tx
        .lock()
        .await
        .send(Message::Text(exit_payload.to_string()))
        .await;

    // Clean up tasks
    reader_task.abort();
    input_task.abort();
    info!("ws_pty: session closed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_xdg_dirs_creates_default_data_home_under_home() {
        let tmp = tempfile::tempdir().unwrap();
        let mut env = HashMap::new();
        env.insert("HOME".to_string(), tmp.path().display().to_string());

        ensure_xdg_dirs(&mut env);

        assert!(tmp.path().join(".local/share").is_dir());
    }

    #[test]
    fn ensure_xdg_dirs_creates_explicit_xdg_data_home() {
        let tmp = tempfile::tempdir().unwrap();
        let data_home = tmp.path().join("xdg-data");
        let mut env = HashMap::new();
        env.insert("HOME".to_string(), tmp.path().display().to_string());
        env.insert("XDG_DATA_HOME".to_string(), data_home.display().to_string());

        ensure_xdg_dirs(&mut env);

        assert!(data_home.is_dir());
    }

    #[test]
    fn ensure_xdg_dirs_falls_back_when_data_home_uncreatable() {
        let tmp = tempfile::tempdir().unwrap();
        let mut env = HashMap::new();

        let home = tmp.path().join("home-as-file");
        std::fs::write(&home, "not a dir").unwrap();
        env.insert("HOME".to_string(), home.display().to_string());

        ensure_xdg_dirs(&mut env);

        assert_eq!(
            env.get("XDG_DATA_HOME").map(String::as_str),
            Some("/tmp/substrate-xdg")
        );
    }

    #[test]
    fn test_client_message_start_serialization() {
        let msg = ClientMessage::Start {
            cmd: "echo hi".into(),
            cwd: std::env::current_dir().unwrap(),
            env: HashMap::new(),
            policy_snapshot: Box::new(None),
            span_id: Some("spn_test".into()),
            cols: 80,
            rows: 24,
        };
        let js = serde_json::to_string(&msg).unwrap();
        assert!(js.contains("\"start\""));
        assert!(js.contains("echo hi"));
    }

    #[test]
    fn test_server_message_stdout_serialization() {
        let msg = ServerMessage::Stdout {
            data_b64: BASE64.encode(b"hello"),
        };
        let js = serde_json::to_string(&msg).unwrap();
        assert!(js.contains("\"stdout\""));
        assert!(js.contains("aGVsbG8"));
    }
}
