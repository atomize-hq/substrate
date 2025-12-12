//! PTY WebSocket handler for world-agent implementing JSON frame protocol

use crate::service::WorldAgentService;
#[cfg(target_os = "linux")]
use crate::service::{resolve_project_dir, WORLD_FS_MODE_ENV};
use axum::extract::ws::{Message, WebSocket};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use portable_pty::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
// no atomic imports needed here
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

pub async fn handle_ws_pty(
    #[cfg_attr(not(target_os = "linux"), allow(unused_variables))] service: WorldAgentService,
    ws: WebSocket,
) {
    info!("ws_pty: client connected");
    let (tx, mut rx) = ws.split();
    let tx = Arc::new(Mutex::new(tx));

    // Wait for start message
    let start_msg = match rx.next().await {
        Some(Ok(Message::Text(text))) => match serde_json::from_str::<ClientMessage>(&text) {
            Ok(ClientMessage::Start {
                cmd,
                cwd,
                env,
                span_id,
                cols,
                rows,
            }) => {
                let mut env = env;
                ensure_xdg_dirs(&mut env);
                info!(
                    %cmd,
                    cwd = %cwd.display(),
                    span_id = span_id.as_deref().unwrap_or("-"),
                    cols = cols,
                    rows = rows,
                    "ws_pty: start"
                );
                (cmd, cwd, env, span_id, cols, rows)
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
        },
        Some(Ok(_)) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: "Expected text message".to_string(),
                },
            )
            .await;
            return;
        }
        Some(Err(e)) => {
            let _ = send_ws_message(
                &tx,
                &ServerMessage::Error {
                    message: format!("WebSocket error: {}", e),
                },
            )
            .await;
            return;
        }
        None => return, // Connection closed
    };

    let (cmd, cwd, env, _span_id, cols, rows) = start_msg;
    #[cfg(target_os = "linux")]
    let mut command_to_run = cmd.clone();
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
        let fs_mode = env
            .get(WORLD_FS_MODE_ENV)
            .and_then(|value| WorldFsMode::parse(value))
            .unwrap_or(WorldFsMode::Writable);
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: substrate_broker::allowed_domains(),
            project_dir: project_dir.clone(),
            always_isolate: true,
            fs_mode,
        };

        match service.ensure_session_overlay_root(&spec) {
            Ok((world, merged_dir)) => {
                world_id_for_logs = Some(world.id.clone());
                let ns_name = format!("substrate-{}", world.id);
                let ns_path = format!("/var/run/netns/{}", ns_name);
                if std::path::Path::new(&ns_path).exists() {
                    ns_name_opt = Some(ns_name);
                }
                let cg = std::path::PathBuf::from("/sys/fs/cgroup/substrate").join(&world.id);
                cgroup_path_opt = Some(cg);
                in_world = true;

                let rel = if cwd.starts_with(&project_dir) {
                    cwd.strip_prefix(&project_dir)
                        .unwrap_or_else(|_| Path::new("."))
                        .to_path_buf()
                } else {
                    PathBuf::from(".")
                };
                cwd_for_child = merged_dir.join(rel);
                anchor_root = merged_dir;
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

        if should_guard_anchor(&env) {
            command_to_run = wrap_with_anchor_guard(&command_to_run, &anchor_root);
        }
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

    // Spawn command (under netns when available)
    let mut cmd_builder = CommandBuilder::new("sh");
    #[cfg(target_os = "linux")]
    if let Some(ref ns_name) = ns_name_opt {
        cmd_builder = CommandBuilder::new("ip");
        cmd_builder.args(["netns", "exec", ns_name, "sh", "-lc", &command_to_run]);
    } else {
        cmd_builder.args(["-lc", &command_to_run]);
    }
    #[cfg(not(target_os = "linux"))]
    {
        cmd_builder.args(["-lc", &command_to_run]);
    }
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
    let exit_msg = ServerMessage::Exit { code: exit_code };
    let _ = send_ws_message(&tx, &exit_msg).await;

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
