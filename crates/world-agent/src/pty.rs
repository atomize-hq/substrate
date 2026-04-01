//! PTY WebSocket handler for world-agent implementing JSON frame protocol

#[cfg(target_os = "linux")]
use crate::enforcement_plan;
#[cfg(target_os = "linux")]
use crate::request_routing::resolve_snapshot_routing;
use crate::service::WorldAgentService;
#[cfg(target_os = "linux")]
use crate::service::{
    apply_full_isolation_helper_env, is_full_isolation, resolve_landlock_allowlist_paths,
    resolve_project_dir, resolve_project_write_allowlist_prefixes, WORLD_FS_MODE_ENV,
    WORLD_FS_WRITE_ALLOWLIST_ENV,
};
#[cfg(target_os = "linux")]
use agent_api_types::PolicyResolutionModeV1;
use agent_api_types::{PolicySnapshotV3, WorldNetworkRoutingV1};
use axum::extract::ws::{Message, WebSocket};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
#[cfg(target_os = "linux")]
use once_cell::sync::OnceCell;
#[cfg(target_os = "linux")]
use rand::rngs::OsRng;
#[cfg(target_os = "linux")]
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
#[cfg(target_os = "linux")]
use tracing::warn;
use tracing::{error, info};
#[cfg(target_os = "linux")]
use world::exec::PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT;
#[cfg(target_os = "linux")]
use world::guard::{should_guard_anchor, wrap_with_anchor_guard};
#[cfg(target_os = "linux")]
use world_api::WorldFsMode;

#[cfg(target_os = "linux")]
fn record_doctor_request_context_for_pty(
    service: &WorldAgentService,
    policy_resolution_mode: PolicyResolutionModeV1,
    isolate_network: bool,
) {
    service.record_doctor_request_context(policy_resolution_mode, isolate_network);
}

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
        policy_snapshot: Box<Option<PolicySnapshotV3>>,
        #[serde(default)]
        world_network: Option<WorldNetworkRoutingV1>,
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
        policy_snapshot: Box<PolicySnapshotV3>,
        #[serde(default)]
        world_network: Option<WorldNetworkRoutingV1>,
        cols: u16,
        rows: u16,
    },
    Exec {
        seq: u64,
        token_hex: String,
        cmd_id: String,
        stdin_mode: PersistentStdinMode,
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

#[cfg(target_os = "linux")]
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum PersistentStdinMode {
    Eof,
    Passthrough,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PersistentServerMessage {
    #[cfg(target_os = "linux")]
    Ready {
        session_nonce: String,
        cwd: PathBuf,
        protocol_version: u32,
    },
    #[cfg(target_os = "linux")]
    Stdout { data_b64: String },
    #[cfg(target_os = "linux")]
    CommandComplete {
        seq: u64,
        token_hex: String,
        exit: i32,
        cwd: PathBuf,
    },
    #[cfg(target_os = "linux")]
    Exit { code: i32 },
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
async fn close_ws_connection(tx: &Arc<Mutex<WsSender>>) {
    let _ = tx.lock().await.send(Message::Close(None)).await;
}

#[cfg(target_os = "linux")]
fn hex32(bytes: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 32];
    for (i, b) in bytes.iter().enumerate() {
        out[i * 2] = HEX[(b >> 4) as usize];
        out[i * 2 + 1] = HEX[(b & 0x0f) as usize];
    }
    // Safety: HEX table is ASCII.
    std::str::from_utf8(&out)
        .unwrap_or("00000000000000000000000000000000")
        .to_string()
}

#[cfg(target_os = "linux")]
fn generate_session_nonce() -> String {
    let mut raw = [0u8; 16];
    OsRng.fill_bytes(&mut raw);
    hex32(raw)
}

#[cfg(all(test, target_os = "linux"))]
mod session_nonce_tests {
    use super::{generate_session_nonce, hex32};

    #[test]
    fn hex32_renders_32_lowercase_hex_chars() {
        assert_eq!(hex32([0u8; 16]), "00000000000000000000000000000000");
    }

    #[test]
    fn generate_session_nonce_is_hex32_lower() {
        let nonce = generate_session_nonce();
        assert_eq!(nonce.len(), 32);
        assert!(
            nonce
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "nonce must be [0-9a-f]; got: {nonce}"
        );
    }
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
fn validate_pty_watermark_query_supported(master_fd: libc::c_int) -> Result<(), std::io::Error> {
    let mut bytes_readable: libc::c_int = 0;
    // Safety: FIONREAD expects a pointer to int.
    let rc = unsafe { libc::ioctl(master_fd, libc::FIONREAD, &mut bytes_readable) };
    if rc == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
struct RawPty {
    master: std::os::fd::OwnedFd,
    slave: std::os::fd::OwnedFd,
}

#[cfg(target_os = "linux")]
fn open_raw_pty(rows: u16, cols: u16) -> Result<RawPty, std::io::Error> {
    let mut master_fd: libc::c_int = -1;
    let mut slave_fd: libc::c_int = -1;
    let size = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    // Safety: openpty initializes master_fd/slave_fd on success.
    let rc = unsafe {
        libc::openpty(
            &mut master_fd,
            &mut slave_fd,
            std::ptr::null_mut(),
            std::ptr::null(),
            &size,
        )
    };
    if rc == -1 {
        return Err(std::io::Error::last_os_error());
    }

    let master = unsafe { std::os::fd::OwnedFd::from_raw_fd(master_fd) };
    let slave = unsafe { std::os::fd::OwnedFd::from_raw_fd(slave_fd) };

    // Best-effort: set master non-blocking so the PTY reader thread can poll+drain without
    // risking an indefinite block mid-drain.
    let flags = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_GETFL) };
    if flags >= 0 {
        let _ = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) };
    }

    Ok(RawPty {
        // Safety: openpty returned valid fds on success.
        master,
        slave,
    })
}

#[cfg(target_os = "linux")]
fn pty_resize(master_fd: libc::c_int, rows: u16, cols: u16) -> Result<(), std::io::Error> {
    let size = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // Safety: TIOCSWINSZ expects winsize pointer.
    let rc = unsafe { libc::ioctl(master_fd, libc::TIOCSWINSZ as _, &size as *const _) };
    if rc == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn pty_fionread(master_fd: libc::c_int) -> Result<usize, std::io::Error> {
    let mut bytes_readable: libc::c_int = 0;
    // Safety: FIONREAD expects a pointer to int.
    let rc = unsafe { libc::ioctl(master_fd, libc::FIONREAD, &mut bytes_readable) };
    if rc == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(bytes_readable.max(0) as usize)
}

#[cfg(target_os = "linux")]
fn exit_code_from_raw_wait_status(status: libc::c_int) -> i32 {
    if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status) as i32
    } else if libc::WIFSIGNALED(status) {
        128 + libc::WTERMSIG(status) as i32
    } else {
        1
    }
}

#[cfg(target_os = "linux")]
fn spawn_legacy_ws_exec(
    pty: &RawPty,
    command_to_run: &str,
    cwd: &std::path::Path,
    env: HashMap<String, String>,
    netns_name: Option<String>,
    cgroup_path: Option<std::path::PathBuf>,
    process_capture: world::exec::ProcessCaptureSpec,
) -> Result<
    (
        std::sync::mpsc::Receiver<Result<world::exec::TracedProcessTreeResult, String>>,
        libc::pid_t,
    ),
    String,
> {
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::time::Duration;

    let stdin_fd = unsafe { libc::dup(pty.slave.as_raw_fd()) };
    let stdout_fd = unsafe { libc::dup(pty.slave.as_raw_fd()) };
    let stderr_fd = unsafe { libc::dup(pty.slave.as_raw_fd()) };
    if stdin_fd < 0 || stdout_fd < 0 || stderr_fd < 0 {
        return Err(format!(
            "dup(pty slave) failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let stdin_file = unsafe { std::fs::File::from_raw_fd(stdin_fd) };
    let stdout_file = unsafe { std::fs::File::from_raw_fd(stdout_fd) };
    let stderr_file = unsafe { std::fs::File::from_raw_fd(stderr_fd) };

    let needs_userns = unsafe { libc::geteuid() != 0 };
    let mut cmd = if let Some(ref ns_name) = netns_name {
        let mut cmd = Command::new("ip");
        cmd.arg("netns");
        cmd.arg("exec");
        cmd.arg(ns_name);
        cmd.arg("unshare");
        cmd.arg("--mount");
        cmd.arg("--fork");
        if needs_userns {
            cmd.arg("--user");
            cmd.arg("--map-root-user");
        }
        cmd.arg("--");
        cmd.arg("sh");
        cmd.arg("-c");
        cmd.arg(command_to_run);
        cmd
    } else {
        let mut cmd = Command::new("unshare");
        cmd.arg("--mount");
        cmd.arg("--fork");
        if needs_userns {
            cmd.arg("--user");
            cmd.arg("--map-root-user");
        }
        cmd.arg("--");
        cmd.arg("sh");
        cmd.arg("-c");
        cmd.arg(command_to_run);
        cmd
    };
    cmd.current_dir(cwd);
    cmd.env_clear();
    cmd.envs(env);
    cmd.stdin(Stdio::from(stdin_file));
    cmd.stdout(Stdio::from(stdout_file));
    cmd.stderr(Stdio::from(stderr_file));

    let tracing_permitted = world::exec::process_capture_capability_available();
    unsafe {
        cmd.pre_exec(move || {
            for signo in &[
                libc::SIGCHLD,
                libc::SIGHUP,
                libc::SIGINT,
                libc::SIGQUIT,
                libc::SIGTERM,
                libc::SIGALRM,
            ] {
                libc::signal(*signo, libc::SIG_DFL);
            }

            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }

            if libc::ioctl(0, libc::TIOCSCTTY as _, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }

            let open_max = libc::sysconf(libc::_SC_OPEN_MAX);
            let max_fd = if open_max > 0 && open_max <= i64::from(i32::MAX) {
                open_max as i32
            } else {
                1024
            };
            for fd in 3..max_fd {
                libc::close(fd);
            }

            if tracing_permitted && libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }

            Ok(())
        });
    }

    let (pid_tx, pid_rx) = std::sync::mpsc::sync_channel::<Result<libc::pid_t, String>>(1);
    let (result_tx, result_rx) =
        std::sync::mpsc::channel::<Result<world::exec::TracedProcessTreeResult, String>>();

    std::thread::spawn(move || {
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(err) => {
                let _ = pid_tx.send(Err(format!("spawn failed: {err}")));
                let _ = result_tx.send(Err(format!("spawn failed: {err}")));
                return;
            }
        };

        let pid = child.id() as libc::pid_t;
        if let Some(ref cg) = cgroup_path {
            let _ = std::fs::create_dir_all(cg);
            let _ = std::fs::write(cg.join("cgroup.procs"), pid.to_string());
        }
        let _ = pid_tx.send(Ok(pid));

        let trace_result = if tracing_permitted {
            world::exec::trace_spawned_process_tree(pid, &process_capture)
        } else {
            let status = match child.wait() {
                Ok(status) => status,
                Err(err) => {
                    let _ = result_tx.send(Err(format!("wait failed: {err}")));
                    return;
                }
            };
            world::exec::TracedProcessTreeResult {
                raw_exit_status: status.into_raw(),
                process_telemetry: world::exec::unavailable_process_telemetry(
                    "ptrace_not_permitted",
                ),
            }
        };

        let _ = result_tx.send(Ok(trace_result));
    });

    let pid = match pid_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(pid)) => pid,
        Ok(Err(message)) => return Err(message),
        Err(_) => return Err("timed out waiting for child pid".to_string()),
    };

    Ok((result_rx, pid))
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
    service: WorldAgentService,
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
            world_network,
            cols,
            rows,
        }) => (
            protocol_version.unwrap_or(1),
            cwd,
            env,
            policy_snapshot,
            world_network,
            cols,
            rows,
        ),
        Ok(_) => {
            let _ = send_persistent_ws_message(
                &tx,
                &PersistentServerMessage::Error {
                    code: "bad_request".to_string(),
                    message: "First frame must be `start_session`".to_string(),
                    fatal: true,
                    seq: None,
                },
            )
            .await;
            close_ws_connection(&tx).await;
            return;
        }
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
            close_ws_connection(&tx).await;
            return;
        }
    };

    let (
        requested_protocol_version,
        requested_cwd,
        mut session_env,
        policy_snapshot,
        world_network,
        cols,
        rows,
    ) = start;

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
        close_ws_connection(&tx).await;
        return;
    }

    if let Err(err) = policy_snapshot.validate() {
        let _ = send_persistent_ws_message(
            &tx,
            &PersistentServerMessage::Error {
                code: "bad_request".to_string(),
                message: err,
                fatal: true,
                seq: None,
            },
        )
        .await;
        close_ws_connection(&tx).await;
        return;
    }

    sanitize_session_env(&mut session_env);
    ensure_xdg_dirs(&mut session_env);

    let pty = match open_raw_pty(rows, cols) {
        Ok(pty) => pty,
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

    // DR-23 preflight: watermark-query capability for Session PTY (v1 requires FIONREAD).
    if let Err(e) = validate_pty_watermark_query_supported(pty.master.as_raw_fd()) {
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

    let world = match prepare_persistent_world_context(
        &service,
        &session_env,
        &ready_cwd,
        &policy_snapshot,
        world_network.as_ref(),
    ) {
        Ok(world) => world,
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

    let (ws_write_tx, mut ws_write_rx) =
        tokio::sync::mpsc::unbounded_channel::<PersistentServerMessage>();

    let writer_task = {
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = ws_write_rx.recv().await {
                if send_persistent_ws_message(&tx, &msg).await.is_err() {
                    break;
                }
            }
        })
    };

    if ws_write_tx
        .send(PersistentServerMessage::Ready {
            session_nonce,
            cwd: ready_cwd.clone(),
            protocol_version: 1,
        })
        .is_err()
    {
        return;
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let pty_bytes_read = Arc::new(AtomicUsize::new(0));
    let (pty_bytes_tx, mut pty_bytes_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1);
    let pty_master_fd = pty.master.as_raw_fd();
    {
        let stop_flag = stop_flag.clone();
        let pty_bytes_read = pty_bytes_read.clone();
        std::thread::spawn(move || {
            read_pty_to_channel(pty_master_fd, stop_flag, pty_bytes_tx, pty_bytes_read)
        });
    }

    let mut session_cwd = ready_cwd;

    struct Running {
        seq: u64,
        token_hex: String,
        stdin_mode: PersistentStdinMode,
        pgid: libc::pid_t,
        read_baseline: usize,
        forwarded_bytes: usize,
    }

    struct Draining {
        seq: u64,
        token_hex: String,
        exit: i32,
        new_cwd: PathBuf,
        new_env: HashMap<String, String>,
        forwarded_bytes: usize,
        target_forwarded_bytes: usize,
    }

    enum ExecPhase {
        Idle,
        Running(Running),
        Draining(Draining),
    }

    let mut phase = ExecPhase::Idle;
    let mut child_events_rx: Option<tokio::sync::mpsc::Receiver<PersistentChildEvent>> = None;

    let send_fatal = |ws_write_tx: &tokio::sync::mpsc::UnboundedSender<PersistentServerMessage>,
                      code: &str,
                      message: String,
                      seq: Option<u64>| {
        let _ = ws_write_tx.send(PersistentServerMessage::Error {
            code: code.to_string(),
            message,
            fatal: true,
            seq,
        });
    };

    loop {
        tokio::select! {
            biased;

            msg = rx.next() => {
                let Some(msg) = msg else { break };
                match msg {
                    Ok(Message::Text(text)) => {
                        let frame = match serde_json::from_str::<PersistentClientMessage>(&text) {
                            Ok(f) => f,
                            Err(e) => {
                                send_fatal(&ws_write_tx, "bad_request", format!("Invalid JSON: {e}"), None);
                                break;
                            }
                        };

                        match frame {
                            PersistentClientMessage::Close => {
                                let _ = ws_write_tx.send(PersistentServerMessage::Exit { code: 0 });
                                break;
                            }
                            PersistentClientMessage::Resize { cols, rows } => {
                                if let Err(e) = pty_resize(pty.master.as_raw_fd(), rows, cols) {
                                    send_fatal(&ws_write_tx, "internal_error", format!("Failed to resize PTY: {e}"), None);
                                    break;
                                }
                            }
                            PersistentClientMessage::Stdin { data_b64 } => {
                                let ExecPhase::Running(ref running) = phase else {
                                    continue;
                                };
                                if !matches!(running.stdin_mode, PersistentStdinMode::Passthrough) {
                                    continue;
                                }
                                let data = match BASE64.decode(&data_b64) {
                                    Ok(d) => d,
                                    Err(_) => {
                                        send_fatal(&ws_write_tx, "bad_request", "Invalid base64 in stdin.data_b64".to_string(), None);
                                        break;
                                    }
                                };
                                if let Err(e) = tokio::task::spawn_blocking({
                                    let master_fd = pty.master.as_raw_fd();
                                    move || write_all_pty(master_fd, &data)
                                }).await.unwrap_or_else(|_| Err(std::io::Error::other("join error"))) {
                                    send_fatal(&ws_write_tx, "internal_error", format!("Failed to write stdin to PTY: {e}"), None);
                                    break;
                                }
                            }
                            PersistentClientMessage::Signal { sig } => {
                                let ExecPhase::Running(ref running) = phase else {
                                    continue;
                                };
                                if let Some(signo) = parse_signal(&sig) {
                                    // Safety: kill is safe; targeting foreground process group with negative pgid.
                                    unsafe { libc::kill(-running.pgid, signo) };
                                }
                            }
                            PersistentClientMessage::Exec {
                                seq,
                                token_hex,
                                cmd_id,
                                stdin_mode,
                                program_b64,
                            } => {
                                match phase {
                                    ExecPhase::Idle => {}
                                    _ => {
                                        send_fatal(&ws_write_tx, "exec_while_busy", "exec received while another command is in-flight".to_string(), Some(seq));
                                        break;
                                    }
                                }

                                let program_bytes = match BASE64.decode(&program_b64) {
                                    Ok(b) => b,
                                    Err(_) => {
                                        send_fatal(&ws_write_tx, "bad_request", "Invalid base64 in exec.program_b64".to_string(), Some(seq));
                                        break;
                                    }
                                };
                                if program_bytes.contains(&0) {
                                    send_fatal(&ws_write_tx, "program_contains_nul", "Program contains NUL byte".to_string(), Some(seq));
                                    break;
                                }
                                let program = match String::from_utf8(program_bytes) {
                                    Ok(s) => s,
                                    Err(_) => {
                                        send_fatal(&ws_write_tx, "program_invalid_utf8", "Program is not valid UTF-8".to_string(), Some(seq));
                                        break;
                                    }
                                };

                                let host_visible = !world
                                    .base_env
                                    .get("SUBSTRATE_WORLD_FS_ISOLATION")
                                    .is_some_and(|value| value.trim().eq_ignore_ascii_case("full"));

                                let mut guard_env = session_env.clone();
                                for (k, v) in world.base_env.iter() {
                                    guard_env.insert(k.clone(), v.clone());
                                }

                                let anchor_guard_enabled = should_guard_anchor(&guard_env);
                                let desired_cwd = if session_cwd.is_absolute()
                                    && (!anchor_guard_enabled
                                        || session_cwd.starts_with(&world.project_dir))
                                {
                                    session_cwd.to_path_buf()
                                } else {
                                    world.project_dir.clone()
                                };

                                if let Some(deny) = crate::world_exec_guard::check_command(
                                    &program,
                                    &desired_cwd,
                                    &guard_env,
                                    host_visible,
                                ) {
                                    let message = crate::world_exec_guard::deny_message(&deny);
                                    let data_b64 = BASE64.encode(message.as_bytes());
                                    let _ =
                                        ws_write_tx.send(PersistentServerMessage::Stdout { data_b64 });
                                    let _ =
                                        ws_write_tx.send(PersistentServerMessage::CommandComplete {
                                            seq,
                                            token_hex,
                                            exit: 5,
                                            cwd: desired_cwd.clone(),
                                        });
                                    session_cwd = desired_cwd;
                                    continue;
                                }

                                let world_handle = world_api::WorldHandle {
                                    id: world.world_id.clone(),
                                };
                                if let Err(e) =
                                    service.refresh_session_network_filter(&world_handle)
                                {
                                    service.record_last_netfilter_failure_for_error(
                                        world.isolate_network,
                                        &e,
                                    );
                                    send_fatal(
                                        &ws_write_tx,
                                        "internal_error",
                                        format!("Failed to refresh session network filter: {e}"),
                                        Some(seq),
                                    );
                                    break;
                                }
                                service.clear_last_netfilter_failure_on_success(
                                    world.isolate_network,
                                );

                                let exec_spec = PersistentExecSpec {
                                    session_env: &session_env,
                                    session_cwd: &session_cwd,
                                    cmd_id: &cmd_id,
                                    stdin_mode,
                                    program: &program,
                                };
                                let (child_events, pgid) = match spawn_persistent_exec(
                                    &world,
                                    &pty,
                                    pty_bytes_read.clone(),
                                    exec_spec,
                                ) {
                                    Ok(v) => v,
                                    Err(message) => {
                                        send_fatal(&ws_write_tx, "internal_error", message, Some(seq));
                                        break;
                                    }
                                };

                                phase = ExecPhase::Running(Running {
                                    seq,
                                    token_hex,
                                    stdin_mode,
                                    pgid,
                                    read_baseline: pty_bytes_read.load(Ordering::Relaxed),
                                    forwarded_bytes: 0,
                                });
                                child_events_rx = Some(child_events);
                            }
                            PersistentClientMessage::StartSession { .. } => {
                                send_fatal(&ws_write_tx, "protocol_violation", "Unexpected start_session after ready".to_string(), None);
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(_) => continue,
                    Err(e) => {
                        send_fatal(&ws_write_tx, "internal_error", format!("WebSocket error: {e}"), None);
                        break;
                    }
                }
            }
            maybe_bytes = pty_bytes_rx.recv() => {
                let Some(bytes) = maybe_bytes else { break };
                let data_b64 = BASE64.encode(&bytes);
                if ws_write_tx.send(PersistentServerMessage::Stdout { data_b64 }).is_err() {
                    break;
                }

                match &mut phase {
                    ExecPhase::Running(running) => {
                        running.forwarded_bytes =
                            running.forwarded_bytes.saturating_add(bytes.len());
                    }
                    ExecPhase::Draining(draining) => {
                        draining.forwarded_bytes =
                            draining.forwarded_bytes.saturating_add(bytes.len());
                    }
                    ExecPhase::Idle => {}
                }

                if matches!(
                    &phase,
                    ExecPhase::Draining(Draining {
                        forwarded_bytes,
                        target_forwarded_bytes,
                        ..
                    }) if *forwarded_bytes >= *target_forwarded_bytes
                ) {
                    // Flush any buffered PTY bytes already read (bounded channel).
                    while let Ok(extra) = pty_bytes_rx.try_recv() {
                        let data_b64 = BASE64.encode(&extra);
                        if ws_write_tx
                            .send(PersistentServerMessage::Stdout { data_b64 })
                            .is_err()
                        {
                            break;
                        }
                    }

                    let ExecPhase::Draining(draining) = std::mem::replace(&mut phase, ExecPhase::Idle) else {
                        continue;
                    };

                    let mut persisted_env = draining.new_env;
                    strip_persistent_internal_env(&mut persisted_env, &world.base_env);
                    sanitize_session_env(&mut persisted_env);
                    ensure_xdg_dirs(&mut persisted_env);

                    session_env = persisted_env;
                    session_cwd = draining.new_cwd.clone();

                    service.note_pty_pending_diff(&world.world_id);

                    if ws_write_tx
                        .send(PersistentServerMessage::CommandComplete {
                            seq: draining.seq,
                            token_hex: draining.token_hex,
                            exit: draining.exit,
                            cwd: draining.new_cwd,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }
            child_event = async {
                match child_events_rx.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => None,
                }
            }, if child_events_rx.is_some() => {
                let Some(child_event) = child_event else {
                    send_fatal(&ws_write_tx, "internal_error", "child event channel closed unexpectedly".to_string(), None);
                    break;
                };
                child_events_rx = None;
                match (child_event, std::mem::replace(&mut phase, ExecPhase::Idle)) {
                    (PersistentChildEvent::Fatal { code, message, seq }, _) => {
                        send_fatal(&ws_write_tx, &code, message, seq);
                        break;
                    }
                    (PersistentChildEvent::Finished { exit, cwd, env, watermark_bytes, bytes_read_at_exit }, ExecPhase::Running(running)) => {
                        // The exit-stop watermark only covers bytes still in the PTY buffer.
                        // Reader-task output may already have been forwarded while we were still
                        // in `Running`, so we complete the command after the total bytes
                        // forwarded for this command reaches the exit-stop byte target.
                        let target_forwarded_bytes = bytes_read_at_exit
                            .saturating_sub(running.read_baseline)
                            .saturating_add(watermark_bytes);
                        phase = ExecPhase::Draining(Draining {
                            seq: running.seq,
                            token_hex: running.token_hex,
                            exit,
                            new_cwd: cwd,
                            new_env: env,
                            forwarded_bytes: running.forwarded_bytes,
                            target_forwarded_bytes,
                        });

                        if matches!(
                            &phase,
                            ExecPhase::Draining(Draining {
                                forwarded_bytes,
                                target_forwarded_bytes,
                                ..
                            }) if *forwarded_bytes >= *target_forwarded_bytes
                        ) {
                            while let Ok(extra) = pty_bytes_rx.try_recv() {
                                let data_b64 = BASE64.encode(&extra);
                                if ws_write_tx.send(PersistentServerMessage::Stdout { data_b64 }).is_err() {
                                    break;
                                }
                            }

                            let ExecPhase::Draining(draining) = std::mem::replace(&mut phase, ExecPhase::Idle) else {
                                continue;
                            };

                            let mut persisted_env = draining.new_env;
                            strip_persistent_internal_env(&mut persisted_env, &world.base_env);
                            sanitize_session_env(&mut persisted_env);
                            ensure_xdg_dirs(&mut persisted_env);

                            session_env = persisted_env;
                            session_cwd = draining.new_cwd.clone();

                            service.note_pty_pending_diff(&world.world_id);

                            if ws_write_tx.send(PersistentServerMessage::CommandComplete {
                                seq: draining.seq,
                                token_hex: draining.token_hex,
                                exit: draining.exit,
                                cwd: draining.new_cwd,
                            }).is_err() {
                                break;
                            }
                        }
                    }
                    _ => {
                        send_fatal(&ws_write_tx, "internal_error", "unexpected child event/state transition".to_string(), None);
                        break;
                    }
                }
            }
        }
    }

    stop_flag.store(true, Ordering::Relaxed);
    drop(pty);
    drop(ws_write_tx);
    let _ = writer_task.await;
    info!(
        "ws_pty: persistent session closed (world_id={})",
        world.world_id
    );
}

#[cfg(target_os = "linux")]
fn resolve_ready_cwd(
    env: &HashMap<String, String>,
    requested_cwd: &std::path::Path,
    _policy_snapshot: &PolicySnapshotV3,
) -> Result<PathBuf, String> {
    let project_dir =
        resolve_project_dir(Some(env), Some(requested_cwd)).map_err(|e| e.to_string())?;

    if !requested_cwd.is_absolute() || !requested_cwd.is_dir() {
        return Ok(project_dir);
    }

    // When caged guards are disabled (e.g. SUBSTRATE_CAGED=0), allow the session to start anywhere
    // in the in-world filesystem, not just under the project anchor. This is required for the
    // persistent-session REPL to support `cd ..` and other uncaged traversal.
    if !should_guard_anchor(env) {
        return Ok(requested_cwd.to_path_buf());
    }

    if requested_cwd.starts_with(&project_dir) {
        Ok(requested_cwd.to_path_buf())
    } else {
        Ok(project_dir)
    }
}

#[cfg(target_os = "linux")]
struct PersistentWorldContext {
    world_id: String,
    merged_dir: PathBuf,
    project_dir: PathBuf,
    fs_mode: WorldFsMode,
    isolate_network: bool,
    netns_name: Option<String>,
    cgroup_path: Option<PathBuf>,
    base_env: HashMap<String, String>,
}

#[cfg(target_os = "linux")]
fn apply_full_isolation_env_from_snapshot(
    env: &mut HashMap<String, String>,
    project_dir: &std::path::Path,
    policy_snapshot: &PolicySnapshotV3,
) -> Result<(), String> {
    let canonical = policy_snapshot.canonicalize()?;

    let write_allowlist = canonical.world_fs.write.allow_list.as_slice();
    let write_allowlist_prefixes =
        resolve_project_write_allowlist_prefixes(project_dir, write_allowlist);
    if !write_allowlist_prefixes.is_empty() {
        env.insert(
            WORLD_FS_WRITE_ALLOWLIST_ENV.to_string(),
            write_allowlist_prefixes.join("\n"),
        );
    }

    let read_allowlist = canonical
        .world_fs
        .read
        .as_ref()
        .map(|d| d.allow_list.as_slice())
        .unwrap_or(&[]);
    let discover_allowlist = canonical
        .world_fs
        .discover
        .as_ref()
        .map(|d| d.allow_list.as_slice())
        .unwrap_or(read_allowlist);
    let landlock_read_paths = resolve_landlock_allowlist_paths(project_dir, read_allowlist);
    let landlock_discover_paths = resolve_landlock_allowlist_paths(project_dir, discover_allowlist);
    let landlock_write_paths = resolve_landlock_allowlist_paths(project_dir, write_allowlist);
    let landlock_supported = world::landlock::detect_support().supported;

    let enforcement_plan_b64 =
        enforcement_plan::maybe_encode_from_snapshot(&canonical).map_err(|err| err.to_string())?;
    apply_full_isolation_helper_env(
        env,
        landlock_supported,
        &landlock_discover_paths,
        &landlock_read_paths,
        &landlock_write_paths,
        enforcement_plan_b64.as_deref(),
    );

    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
mod wfgad2_tests {
    use super::*;
    use agent_api_types::{
        PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3,
        PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3, WorldFsDenyEnforcementV3,
    };

    #[test]
    fn pty_full_isolation_env_includes_enforcement_plan_and_helper_src() {
        let tmp = tempfile::tempdir().expect("tempdir");

        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
                deny_enforcement: Some(WorldFsDenyEnforcementV3::Strict),
                caged_required: false,
                discover: None,
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: false,
                    allow_list: vec!["src".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };
        snapshot.validate().expect("snapshot validates");

        let mut env = HashMap::new();
        apply_full_isolation_env_from_snapshot(&mut env, tmp.path(), &snapshot)
            .expect("apply full isolation env");

        let plan_b64 = env
            .get(enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV)
            .expect("enforcement plan env present");
        let bytes = BASE64.decode(plan_b64.as_bytes()).expect("plan is base64");
        let value: serde_json::Value = serde_json::from_slice(&bytes).expect("plan is JSON");

        assert_eq!(value["enforcement"], "strict");
        assert_eq!(value["read_deny"], serde_json::json!(["**/*.pem"]));
        assert_eq!(value["discover_deny"], serde_json::json!(["**/*.pem"]));

        assert!(
            env.contains_key(crate::service::LANDLOCK_HELPER_SRC_ENV),
            "helper src env must be set whenever enforcement plan is present"
        );
    }
}

#[cfg(target_os = "linux")]
fn prepare_persistent_world_context(
    service: &WorldAgentService,
    session_env: &HashMap<String, String>,
    ready_cwd: &std::path::Path,
    policy_snapshot: &PolicySnapshotV3,
    world_network: Option<&WorldNetworkRoutingV1>,
) -> Result<PersistentWorldContext, String> {
    use world_api::{ResourceLimits, WorldSpec};

    let project_dir =
        resolve_project_dir(Some(session_env), Some(ready_cwd)).map_err(|e| e.to_string())?;

    let resolved = resolve_snapshot_routing(policy_snapshot, world_network)?;
    let canonical = resolved.snapshot;
    let fs_mode = resolved.fs_mode;
    let isolate_network = resolved.world_network.isolate_network;

    record_doctor_request_context_for_pty(
        service,
        PolicyResolutionModeV1::SnapshotV3,
        isolate_network,
    );

    let spec = WorldSpec {
        reuse_session: true,
        isolate_network,
        limits: ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: resolved.world_network.allowed_domains.clone(),
        project_dir: project_dir.clone(),
        always_isolate: true,
        fs_mode,
    };

    let (world, merged_dir) = service.ensure_session_overlay_root(&spec).map_err(|e| {
        service.record_last_netfilter_failure_for_error(isolate_network, &e);
        format!("Failed to prepare world overlay: {e}")
    })?;
    service.clear_last_netfilter_failure_on_success(isolate_network);

    let ns_name = format!("substrate-{}", world.id);
    let ns_path = format!("/var/run/netns/{ns_name}");
    let netns_name = if std::path::Path::new(&ns_path).exists() {
        Some(ns_name)
    } else {
        None
    };

    let cgroup_path = Some(
        service
            .session_cgroup_path(&world)
            .map_err(|e| format!("Failed to resolve session cgroup path: {e}"))?,
    );

    let mut base_env: HashMap<String, String> = HashMap::new();
    let isolation_full = !canonical.world_fs.host_visible;
    if isolation_full {
        base_env.insert(
            "SUBSTRATE_WORLD_FS_ISOLATION".to_string(),
            "full".to_string(),
        );
        apply_full_isolation_env_from_snapshot(&mut base_env, &project_dir, &canonical)?;
    } else {
        base_env.insert(
            "SUBSTRATE_WORLD_FS_ISOLATION".to_string(),
            "workspace".to_string(),
        );
    }

    Ok(PersistentWorldContext {
        world_id: world.id,
        merged_dir,
        project_dir,
        fs_mode,
        isolate_network,
        netns_name,
        cgroup_path,
        base_env,
    })
}

#[cfg(target_os = "linux")]
fn strip_persistent_internal_env(
    env: &mut HashMap<String, String>,
    base_env: &HashMap<String, String>,
) {
    env.remove("SHIM_PARENT_CMD_ID");
    env.remove("SUBSTRATE_PROGRAM");
    env.remove("SUBSTRATE_MOUNT_MERGED_DIR");
    env.remove("SUBSTRATE_MOUNT_PROJECT_DIR");
    env.remove("SUBSTRATE_MOUNT_CWD");
    env.remove("SUBSTRATE_MOUNT_FS_MODE");
    env.remove("SUBSTRATE_INNER_CMD");
    env.remove("SUBSTRATE_INNER_LOGIN_SHELL");
    env.remove("SUBSTRATE_LANDLOCK_HELPER_PATH");

    for key in base_env.keys() {
        env.remove(key);
    }

    // Defense-in-depth: keep shim runtime variables out of the persisted env.
    for key in [
        "SHIM_ACTIVE",
        "SHIM_CALLER",
        "SHIM_CALL_STACK",
        "SHIM_DEPTH",
    ] {
        env.remove(key);
    }
}

#[cfg(target_os = "linux")]
enum PersistentChildEvent {
    Finished {
        exit: i32,
        cwd: PathBuf,
        env: HashMap<String, String>,
        watermark_bytes: usize,
        bytes_read_at_exit: usize,
    },
    Fatal {
        code: String,
        message: String,
        seq: Option<u64>,
    },
}

#[cfg(target_os = "linux")]
fn read_pty_to_channel(
    master_fd: libc::c_int,
    stop: Arc<AtomicBool>,
    tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    bytes_read: Arc<AtomicUsize>,
) {
    let mut buf = [0u8; 8192];
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let mut fds = libc::pollfd {
            fd: master_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // Safety: pollfd points to valid memory.
        let rc = unsafe { libc::poll(&mut fds as *mut libc::pollfd, 1, 100) };
        if rc < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            break;
        }
        if rc == 0 {
            continue;
        }

        if (fds.revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR)) == 0 {
            continue;
        }

        loop {
            // Safety: buf is valid for reads.
            let n = unsafe {
                libc::read(
                    master_fd,
                    buf.as_mut_ptr().cast::<libc::c_void>(),
                    buf.len(),
                )
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    break;
                }
                return;
            }
            if n == 0 {
                return;
            }
            bytes_read.fetch_add(n as usize, Ordering::Relaxed);
            let bytes = buf[..(n as usize)].to_vec();
            if tx.blocking_send(bytes).is_err() {
                return;
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn write_all_pty(master_fd: libc::c_int, data: &[u8]) -> Result<(), std::io::Error> {
    let mut offset = 0usize;
    while offset < data.len() {
        // Safety: data pointer is valid.
        let n = unsafe {
            libc::write(
                master_fd,
                data[offset..].as_ptr().cast::<libc::c_void>(),
                (data.len() - offset) as libc::size_t,
            )
        };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if err.kind() == std::io::ErrorKind::WouldBlock {
                let mut fds = libc::pollfd {
                    fd: master_fd,
                    events: libc::POLLOUT,
                    revents: 0,
                };
                // Safety: pollfd points to valid memory.
                let _ = unsafe { libc::poll(&mut fds as *mut libc::pollfd, 1, 100) };
                continue;
            }
            return Err(err);
        }
        offset += n as usize;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn parse_proc_environ(bytes: Vec<u8>) -> Result<HashMap<String, String>, String> {
    let mut env = HashMap::new();
    let mut start = 0usize;
    for i in 0..=bytes.len() {
        if i == bytes.len() || bytes[i] == 0 {
            if i > start {
                let entry = &bytes[start..i];
                let entry = String::from_utf8(entry.to_vec())
                    .map_err(|_| "process environ contains non-utf8 entry".to_string())?;
                if let Some((k, v)) = entry.split_once('=') {
                    env.insert(k.to_string(), v.to_string());
                }
            }
            start = i + 1;
        }
    }
    Ok(env)
}

#[cfg(target_os = "linux")]
struct PersistentExecSpec<'a> {
    session_env: &'a HashMap<String, String>,
    session_cwd: &'a std::path::Path,
    cmd_id: &'a str,
    stdin_mode: PersistentStdinMode,
    program: &'a str,
}

#[cfg(target_os = "linux")]
fn spawn_persistent_exec(
    world: &PersistentWorldContext,
    pty: &RawPty,
    pty_bytes_read: Arc<AtomicUsize>,
    spec: PersistentExecSpec<'_>,
) -> Result<
    (
        tokio::sync::mpsc::Receiver<PersistentChildEvent>,
        libc::pid_t,
    ),
    String,
> {
    use std::os::unix::process::CommandExt;
    use std::time::Duration;

    let mut env = spec.session_env.clone();
    for (k, v) in world.base_env.iter() {
        env.insert(k.clone(), v.clone());
    }

    let should_guard = should_guard_anchor(&env);
    let desired_cwd = if spec.session_cwd.is_absolute()
        && (!should_guard || spec.session_cwd.starts_with(&world.project_dir))
    {
        spec.session_cwd.to_path_buf()
    } else {
        world.project_dir.clone()
    };

    let program = if should_guard {
        wrap_with_anchor_guard(spec.program, &world.project_dir)
    } else {
        spec.program.to_string()
    };

    env.insert(
        "SUBSTRATE_MOUNT_MERGED_DIR".to_string(),
        world.merged_dir.display().to_string(),
    );
    env.insert(
        "SUBSTRATE_MOUNT_PROJECT_DIR".to_string(),
        world.project_dir.display().to_string(),
    );
    env.insert(
        "SUBSTRATE_MOUNT_CWD".to_string(),
        desired_cwd.display().to_string(),
    );
    env.insert(
        "SUBSTRATE_MOUNT_FS_MODE".to_string(),
        world.fs_mode.as_str().to_string(),
    );

    env.insert("SUBSTRATE_PROGRAM".to_string(), program);
    env.insert("SHIM_PARENT_CMD_ID".to_string(), spec.cmd_id.to_string());

    let inner_cmd = match spec.stdin_mode {
        PersistentStdinMode::Eof => {
            r#"exec </dev/null /bin/bash --noprofile --norc -c "$SUBSTRATE_PROGRAM""#.to_string()
        }
        PersistentStdinMode::Passthrough => {
            r#"exec /bin/bash --noprofile --norc -c "$SUBSTRATE_PROGRAM""#.to_string()
        }
    };
    env.insert("SUBSTRATE_INNER_CMD".to_string(), inner_cmd);
    env.insert("SUBSTRATE_INNER_LOGIN_SHELL".to_string(), "0".to_string());

    let needs_userns = unsafe { libc::geteuid() != 0 };

    let stdin_fd = unsafe { libc::dup(pty.slave.as_raw_fd()) };
    let stdout_fd = unsafe { libc::dup(pty.slave.as_raw_fd()) };
    let stderr_fd = unsafe { libc::dup(pty.slave.as_raw_fd()) };
    if stdin_fd < 0 || stdout_fd < 0 || stderr_fd < 0 {
        return Err(format!(
            "dup(pty slave) failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let stdin_file = unsafe { std::fs::File::from_raw_fd(stdin_fd) };
    let stdout_file = unsafe { std::fs::File::from_raw_fd(stdout_fd) };
    let stderr_file = unsafe { std::fs::File::from_raw_fd(stderr_fd) };

    let mut cmd = Command::new("unshare");
    cmd.arg("--mount");
    if needs_userns {
        cmd.arg("--user");
        cmd.arg("--map-root-user");
    }
    cmd.arg("--");
    cmd.arg("sh");
    cmd.arg("-c");
    cmd.arg(PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT);
    cmd.current_dir("/");
    cmd.env_clear();
    cmd.envs(env);
    cmd.stdin(Stdio::from(stdin_file));
    cmd.stdout(Stdio::from(stdout_file));
    cmd.stderr(Stdio::from(stderr_file));

    let netns_fd_parent = if let Some(ref ns_name) = world.netns_name {
        let ns_path = std::ffi::CString::new(format!("/var/run/netns/{ns_name}"))
            .map_err(|_| "netns path contains NUL".to_string())?;
        // Safety: libc::open called with valid C string.
        let fd = unsafe { libc::open(ns_path.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
        if fd < 0 {
            return Err(format!(
                "open(netns) failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        Some(fd)
    } else {
        None
    };
    let netns_fd_child = netns_fd_parent.map(|fd| unsafe { libc::dup(fd) });

    unsafe {
        cmd.pre_exec(move || {
            for signo in &[
                libc::SIGCHLD,
                libc::SIGHUP,
                libc::SIGINT,
                libc::SIGQUIT,
                libc::SIGTERM,
                libc::SIGALRM,
            ] {
                libc::signal(*signo, libc::SIG_DFL);
            }

            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }

            if libc::ioctl(0, libc::TIOCSCTTY as _, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }

            if let Some(fd) = netns_fd_child {
                if libc::setns(fd, libc::CLONE_NEWNET) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                libc::close(fd);
            }

            let open_max = libc::sysconf(libc::_SC_OPEN_MAX);
            let max_fd = if open_max > 0 && open_max <= i64::from(i32::MAX) {
                open_max as i32
            } else {
                1024
            };

            for fd in 3..max_fd {
                libc::close(fd);
            }

            // Request tracing by the parent so it can capture env/cwd at an exit-stop.
            // This avoids relying on PTRACE_ATTACH (which some host policies/LSMs may block)
            // while still allowing the parent to observe a deterministic exit-stop.
            if libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }

            Ok(())
        });
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<PersistentChildEvent>(1);
    let master_fd = pty.master.as_raw_fd();

    // ptrace(TRACEME) ties the tracee to the forking thread. Ensure the same OS thread
    // that spawns the child also performs all ptrace operations.
    let (pid_tx, pid_rx) = std::sync::mpsc::sync_channel::<Result<libc::pid_t, String>>(1);
    let cgroup_path = world.cgroup_path.clone();
    std::thread::spawn(move || {
        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = pid_tx.send(Err(format!("spawn failed: {e}")));
                let _ = tx.blocking_send(PersistentChildEvent::Fatal {
                    code: "internal_error".to_string(),
                    message: format!("spawn failed: {e}"),
                    seq: None,
                });
                return;
            }
        };

        let pid = child.id() as libc::pid_t;
        drop(child);

        if let Some(fd) = netns_fd_parent {
            // Safety: best-effort cleanup.
            unsafe { libc::close(fd) };
        }
        if let Some(fd) = netns_fd_child {
            // Safety: best-effort cleanup.
            unsafe { libc::close(fd) };
        }

        if let Some(ref cg) = cgroup_path {
            let _ = std::fs::create_dir_all(cg);
            let _ = std::fs::write(cg.join("cgroup.procs"), pid.to_string());
        }

        let _ = pid_tx.send(Ok(pid));
        traced_wait_loop(pid, master_fd, tx, pty_bytes_read);
    });

    let pid = match pid_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(pid)) => pid,
        Ok(Err(msg)) => return Err(msg),
        Err(_) => return Err("timed out waiting for child pid".to_string()),
    };

    Ok((rx, pid))
}

#[cfg(target_os = "linux")]
fn traced_wait_loop(
    pid: libc::pid_t,
    master_fd: libc::c_int,
    tx: tokio::sync::mpsc::Sender<PersistentChildEvent>,
    pty_bytes_read: Arc<AtomicUsize>,
) {
    let send_fatal = |code: &str, message: String| {
        let _ = tx.blocking_send(PersistentChildEvent::Fatal {
            code: code.to_string(),
            message,
            seq: None,
        });
    };

    // With PTRACE_TRACEME, the first ptrace-stop occurs on execve of the `unshare` target.
    // Wait for that stop, set TRACEEXIT, then continue.
    let mut status: libc::c_int = 0;
    // Safety: waitpid writes to status.
    let rc = unsafe { libc::waitpid(pid, &mut status as *mut libc::c_int, 0) };
    if rc < 0 {
        send_fatal(
            "internal_error",
            format!(
                "waitpid initial ptrace stop failed: {}",
                std::io::Error::last_os_error()
            ),
        );
        return;
    }

    let trace_pid = rc;
    if !libc::WIFSTOPPED(status) {
        send_fatal(
            "internal_error",
            "child did not stop as expected".to_string(),
        );
        return;
    }

    let stop_sig = libc::WSTOPSIG(status);

    // Safety: ptrace called with valid pid.
    let rc = unsafe {
        libc::ptrace(
            libc::PTRACE_SETOPTIONS,
            trace_pid,
            0,
            (libc::PTRACE_O_TRACEEXIT) as libc::c_long,
        )
    };
    if rc == -1 {
        send_fatal(
            "internal_error",
            format!(
                "ptrace SETOPTIONS failed: {} (pid={}, trace_pid={}, stop_sig={})",
                std::io::Error::last_os_error(),
                pid,
                trace_pid,
                stop_sig
            ),
        );
        return;
    }

    // Safety: ptrace called with valid pid.
    let cont_sig = if stop_sig == libc::SIGTRAP {
        0
    } else {
        stop_sig
    };
    let rc = unsafe { libc::ptrace(libc::PTRACE_CONT, trace_pid, 0, cont_sig) };
    if rc == -1 {
        send_fatal(
            "internal_error",
            format!("ptrace CONT failed: {}", std::io::Error::last_os_error()),
        );
        return;
    }

    let mut captured_env: Option<HashMap<String, String>> = None;
    let mut captured_cwd: Option<PathBuf> = None;
    let mut watermark: Option<usize> = None;
    let mut bytes_read_at_exit: Option<usize> = None;

    loop {
        status = 0;
        // Safety: waitpid writes to status.
        let rc = unsafe { libc::waitpid(trace_pid, &mut status as *mut libc::c_int, 0) };
        if rc < 0 {
            send_fatal(
                "internal_error",
                format!("waitpid failed: {}", std::io::Error::last_os_error()),
            );
            return;
        }

        if libc::WIFSTOPPED(status) {
            let sig = libc::WSTOPSIG(status);
            let event = (status >> 16) & 0xffff;
            if sig == libc::SIGTRAP && event == libc::PTRACE_EVENT_EXIT {
                let cwd =
                    std::fs::read_link(format!("/proc/{trace_pid}/cwd")).map_err(|e| e.to_string());
                let env_bytes =
                    std::fs::read(format!("/proc/{trace_pid}/environ")).map_err(|e| e.to_string());
                match (cwd, env_bytes) {
                    (Ok(cwd), Ok(env_bytes)) => match parse_proc_environ(env_bytes) {
                        Ok(env) => {
                            captured_env = Some(env);
                            captured_cwd = Some(cwd);
                        }
                        Err(e) => {
                            send_fatal(
                                "internal_error",
                                format!("Failed to parse child environ: {e}"),
                            );
                            return;
                        }
                    },
                    (Err(e), _) => {
                        send_fatal(
                            "internal_error",
                            format!("Failed to capture child cwd: {e}"),
                        );
                        return;
                    }
                    (_, Err(e)) => {
                        send_fatal(
                            "internal_error",
                            format!("Failed to capture child environ: {e}"),
                        );
                        return;
                    }
                }

                match pty_fionread(master_fd) {
                    Ok(b) => {
                        watermark = Some(b);
                        bytes_read_at_exit = Some(pty_bytes_read.load(Ordering::Relaxed));
                    }
                    Err(e) => {
                        send_fatal(
                            "internal_error",
                            format!("FIONREAD watermark query failed: {e}"),
                        );
                        return;
                    }
                }

                // Safety: ptrace called with valid pid.
                let _ = unsafe { libc::ptrace(libc::PTRACE_CONT, trace_pid, 0, 0) };
                continue;
            }

            // Pass through other stops.
            // Do not deliver SIGTRAP back to the tracee; it is used for ptrace bookkeeping.
            // Safety: ptrace called with valid pid.
            let cont_sig = if sig == libc::SIGTRAP { 0 } else { sig };
            let _ = unsafe { libc::ptrace(libc::PTRACE_CONT, trace_pid, 0, cont_sig) };
            continue;
        }

        if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
            break;
        }
    }

    let exit_code = if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status) as i32
    } else if libc::WIFSIGNALED(status) {
        let sig = libc::WTERMSIG(status) as i32;
        128 + sig
    } else {
        1
    };

    let Some(env) = captured_env else {
        send_fatal(
            "internal_error",
            "missing child env capture at exit stop".to_string(),
        );
        return;
    };
    let Some(cwd) = captured_cwd else {
        send_fatal(
            "internal_error",
            "missing child cwd capture at exit stop".to_string(),
        );
        return;
    };
    let Some(watermark_bytes) = watermark else {
        send_fatal(
            "internal_error",
            "missing PTY watermark at exit stop".to_string(),
        );
        return;
    };
    let Some(bytes_read_at_exit) = bytes_read_at_exit else {
        send_fatal(
            "internal_error",
            "missing PTY byte-count capture at exit stop".to_string(),
        );
        return;
    };

    let _ = tx.blocking_send(PersistentChildEvent::Finished {
        exit: exit_code,
        cwd,
        env,
        watermark_bytes,
        bytes_read_at_exit,
    });
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
            world_network,
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
            (
                cmd,
                cwd,
                env,
                policy_snapshot,
                world_network,
                span_id,
                cols,
                rows,
            )
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

    let (cmd, cwd, env_map, policy_snapshot, _world_network, _span_id, cols, rows) = start_msg;
    #[cfg(not(target_os = "linux"))]
    let _policy_snapshot = policy_snapshot;
    #[cfg(target_os = "linux")]
    let mut env = env_map;
    #[cfg(not(target_os = "linux"))]
    let env = env_map;
    #[cfg(target_os = "linux")]
    let mut inner_cmd = cmd.clone();
    #[cfg(target_os = "linux")]
    let world_network = _world_network;
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
        let (policy_resolution_mode, isolation_full, fs_mode, isolate_network, allowed_domains) =
            if let Some(snapshot) = policy_snapshot.as_ref() {
                let resolved = match resolve_snapshot_routing(snapshot, world_network.as_ref()) {
                    Ok(v) => v,
                    Err(err) => {
                        let _ = send_ws_message(
                            &tx,
                            &ServerMessage::Error {
                                message: format!("Invalid policy_snapshot/world_network: {err}"),
                            },
                        )
                        .await;
                        return;
                    }
                };

                let isolation_full = resolved.isolation_full;
                (
                    PolicyResolutionModeV1::SnapshotV3,
                    isolation_full,
                    resolved.fs_mode,
                    resolved.world_network.isolate_network,
                    resolved.world_network.allowed_domains,
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
                    false,
                    Vec::new(),
                )
            };

        let host_visible = !isolation_full;
        record_doctor_request_context_for_pty(&service, policy_resolution_mode, isolate_network);

        if isolation_full {
            if let Some(snapshot) = policy_snapshot.as_ref() {
                if let Err(err) =
                    apply_full_isolation_env_from_snapshot(&mut env, &project_dir, snapshot)
                {
                    let _ = send_ws_message(
                        &tx,
                        &ServerMessage::Error {
                            message: format!("Invalid policy_snapshot enforcement plan: {err}"),
                        },
                    )
                    .await;
                    return;
                }
            } else {
                let world_fs = substrate_broker::world_fs_policy();
                let prefixes = resolve_project_write_allowlist_prefixes(
                    &project_dir,
                    &world_fs.write_allowlist,
                );
                if !prefixes.is_empty() {
                    env.insert(
                        WORLD_FS_WRITE_ALLOWLIST_ENV.to_string(),
                        prefixes.join("\n"),
                    );
                }
                let landlock_read_paths =
                    resolve_landlock_allowlist_paths(&project_dir, &world_fs.read_allowlist);
                let landlock_write_paths =
                    resolve_landlock_allowlist_paths(&project_dir, &world_fs.write_allowlist);
                let landlock_discover_paths: Vec<String> = Vec::new();
                let landlock_supported = world::landlock::detect_support().supported;
                apply_full_isolation_helper_env(
                    &mut env,
                    landlock_supported,
                    &landlock_discover_paths,
                    &landlock_read_paths,
                    &landlock_write_paths,
                    None,
                );
            }
        }

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network,
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

        let should_guard = should_guard_anchor(&env);
        let desired_cwd = if cwd.is_absolute()
            && cwd.is_dir()
            && (!should_guard || cwd.starts_with(&project_dir))
        {
            cwd.clone()
        } else {
            project_dir.clone()
        };

        if let Some(deny) =
            crate::world_exec_guard::check_command(&cmd, &desired_cwd, &env, host_visible)
        {
            let message = crate::world_exec_guard::deny_message(&deny);
            let data_b64 = BASE64.encode(message.as_bytes());
            let _ = send_ws_message(&tx, &ServerMessage::Stdout { data_b64 }).await;
            let _ = send_ws_message(&tx, &ServerMessage::Exit { code: 5 }).await;
            return;
        }

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

        if should_guard {
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

    #[cfg(target_os = "linux")]
    let pty = match open_raw_pty(rows, cols) {
        Ok(pty) => pty,
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
    let process_capture = world::exec::ProcessCaptureSpec::from_env(
        world_id_for_logs.as_deref().unwrap_or("unknown"),
        &env,
        _span_id.as_deref(),
    );

    #[cfg(target_os = "linux")]
    let (trace_rx, child_pid) = match spawn_legacy_ws_exec(
        &pty,
        &command_to_run,
        &cwd_for_child,
        env,
        ns_name_opt.clone(),
        cgroup_path_opt.clone(),
        process_capture,
    ) {
        Ok((trace_rx, pid)) => (trace_rx, Some(pid)),
        Err(message) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: format!("Failed to spawn command: {message}"),
                },
            )
            .await;
            return;
        }
    };

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

    #[cfg(target_os = "linux")]
    let reader_fd = unsafe { libc::dup(pty.master.as_raw_fd()) };
    #[cfg(target_os = "linux")]
    if reader_fd < 0 {
        let _ = send_ws_message(
            &tx,
            &ServerMessage::Error {
                message: format!(
                    "Failed to duplicate PTY reader fd: {}",
                    std::io::Error::last_os_error()
                ),
            },
        )
        .await;
        return;
    }
    #[cfg(target_os = "linux")]
    let mut reader_task = {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut reader = unsafe { std::fs::File::from_raw_fd(reader_fd) };
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
                    Ok((r, b, Err(err)))
                        if matches!(
                            err.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                        ) =>
                    {
                        reader = r;
                        buf = b;
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                    _ => break,
                }
            }
        })
    };

    #[cfg(target_os = "linux")]
    let master_fd = pty.master.as_raw_fd();
    #[cfg(target_os = "linux")]
    let input_task = tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(Message::Text(text)) => match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Stdin { data_b64 }) => {
                        let Ok(data) = BASE64.decode(&data_b64) else {
                            continue;
                        };
                        if tokio::task::spawn_blocking(move || write_all_pty(master_fd, &data))
                            .await
                            .unwrap_or_else(|_| Err(std::io::Error::other("join error")))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(ClientMessage::Resize { cols, rows }) => {
                        let _ = pty_resize(master_fd, rows, cols);
                    }
                    Ok(ClientMessage::Signal { sig }) => forward_signal(child_pid, &sig),
                    _ => {}
                },
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
    });

    #[cfg(target_os = "linux")]
    let trace_result = tokio::task::spawn_blocking(move || trace_rx.recv())
        .await
        .ok()
        .and_then(Result::ok)
        .and_then(Result::ok);

    #[cfg(target_os = "linux")]
    let (exit_code, process_telemetry) = match trace_result {
        Some(result) => (
            exit_code_from_raw_wait_status(result.raw_exit_status),
            result.process_telemetry,
        ),
        None => (
            1,
            world::exec::unavailable_process_telemetry("backend_disabled"),
        ),
    };

    #[cfg(target_os = "linux")]
    input_task.abort();
    #[cfg(target_os = "linux")]
    drop(pty);
    #[cfg(target_os = "linux")]
    if tokio::time::timeout(std::time::Duration::from_millis(250), &mut reader_task)
        .await
        .is_err()
    {
        // The legacy stream contract expects buffered PTY stdout before `exit`.
        // If the reader does not quiesce promptly after we close the parent PTY handles,
        // abort it rather than hanging the websocket close path indefinitely.
        reader_task.abort();
    }

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
        "process_events": process_telemetry.process_events,
        "process_events_status": process_telemetry.process_events_status.as_str(),
        "process_events_reason": process_telemetry.process_events_reason,
        "process_events_dropped": process_telemetry.process_events_dropped,
        "process_events_max": process_telemetry.process_events_max,
        "process_events_backend": process_telemetry.process_events_backend,
        "process_events_error": process_telemetry.process_events_error,
        "world_fs_strategy_primary": primary,
        "world_fs_strategy_final": final_strategy,
        "world_fs_strategy_fallback_reason": reason,
    });
    let _ = tx
        .lock()
        .await
        .send(Message::Text(exit_payload.to_string()))
        .await;

    #[cfg(target_os = "linux")]
    if let Some(ref world_id) = world_id_for_logs {
        service.note_pty_pending_diff(world_id);
    }

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
            world_network: None,
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

    #[cfg(target_os = "linux")]
    #[test]
    fn record_doctor_request_context_for_pty_updates_requested_state() {
        let service = WorldAgentService::new().expect("service");

        record_doctor_request_context_for_pty(&service, PolicyResolutionModeV1::SnapshotV3, true);

        assert_eq!(
            service.last_policy_resolution_mode(),
            Some(PolicyResolutionModeV1::SnapshotV3)
        );
        assert!(service.last_netfilter_requested());

        record_doctor_request_context_for_pty(&service, PolicyResolutionModeV1::LegacyLocal, false);

        assert_eq!(
            service.last_policy_resolution_mode(),
            Some(PolicyResolutionModeV1::LegacyLocal)
        );
        assert!(!service.last_netfilter_requested());
    }
}
