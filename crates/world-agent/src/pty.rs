//! PTY WebSocket handler for world-agent implementing JSON frame protocol

use crate::service::WorldAgentService;
use axum::extract::ws::{Message, WebSocket};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{SinkExt, StreamExt};
use portable_pty::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
// no atomic imports needed here

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

#[cfg(not(unix))]
fn parse_signal(_sig: &str) -> Option<i32> { None }

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
                let _ = tx
                    .lock()
                    .await
                    .send(Message::Text(
                        serde_json::to_string(&ServerMessage::Error {
                            message: "Expected start message".to_string(),
                        })
                        .unwrap(),
                    ))
                    .await;
                return;
            }
            Err(e) => {
                let _ = tx
                    .lock()
                    .await
                    .send(Message::Text(
                        serde_json::to_string(&ServerMessage::Error {
                            message: format!("Invalid JSON: {}", e),
                        })
                        .unwrap(),
                    ))
                    .await;
                return;
            }
        },
        Some(Ok(_)) => {
            let _ = tx
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: "Expected text message".to_string(),
                    })
                    .unwrap(),
                ))
                .await;
            return;
        }
        Some(Err(e)) => {
            let _ = tx
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: format!("WebSocket error: {}", e),
                    })
                    .unwrap(),
                ))
                .await;
            return;
        }
        None => return, // Connection closed
    };

    let (cmd, cwd, env, _span_id, cols, rows) = start_msg;

    // Prepare in-world session context (best-effort)
    let mut world_id_for_logs: String = "-".to_string();
    let mut ns_name_opt: Option<String> = None;
    let mut cgroup_path_opt: Option<std::path::PathBuf> = None;
    let mut in_world = false;
    #[cfg(target_os = "linux")]
    {
        use world_api::{ResourceLimits, WorldSpec};
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: substrate_broker::allowed_domains(),
            project_dir: cwd.clone(),
            always_isolate: false, // Default: use heuristic-based isolation
        };
        if let Ok(world) = service.ensure_session_world(&spec) {
            world_id_for_logs = world.id.clone();
            let ns_name = format!("substrate-{}", world.id);
            let ns_path = format!("/var/run/netns/{}", ns_name);
            if std::path::Path::new(&ns_path).exists() {
                ns_name_opt = Some(ns_name);
            }
            let cg = std::path::PathBuf::from("/sys/fs/cgroup/substrate").join(&world.id);
            cgroup_path_opt = Some(cg);
            in_world = true;
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
            let _ = tx
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: format!("Failed to create PTY: {}", e),
                    })
                    .unwrap(),
                ))
                .await;
            return;
        }
    };

    // Spawn command (under netns when available)
    let mut cmd_builder = CommandBuilder::new("sh");
    #[cfg(target_os = "linux")]
    if let Some(ref ns_name) = ns_name_opt {
        cmd_builder = CommandBuilder::new("ip");
        cmd_builder.args(["netns", "exec", ns_name, "sh", "-lc", &cmd]);
    } else {
        cmd_builder.args(["-lc", &cmd]);
    }
    #[cfg(not(target_os = "linux"))]
    {
        cmd_builder.args(["-lc", &cmd]);
    }
    cmd_builder.cwd(cwd);
    for (key, value) in env {
        cmd_builder.env(key, value);
    }

    let mut child = match pair.slave.spawn_command(cmd_builder) {
        Ok(child) => child,
        Err(e) => {
            let _ = tx
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: format!("Failed to spawn command: {}", e),
                    })
                    .unwrap(),
                ))
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
    info!(
        world_id = %world_id_for_logs,
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
            let _ = tx
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: format!("Failed to clone PTY reader: {}", e),
                    })
                    .unwrap(),
                ))
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
                    if tx_clone
                        .lock()
                        .await
                        .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                        .await
                        .is_err()
                    {
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
            let _ = tx
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: format!("Failed to take PTY writer: {}", e),
                    })
                    .unwrap(),
                ))
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
                            // Forward signal to child process if available
                            if let Some(pid) = child_pid {
                                if let Some(signo) = parse_signal(&sig) {
                                    #[cfg(unix)]
                                    {
                                        // Safety: libc::kill is async-signal-safe; called on background task
                                        unsafe { libc::kill(pid as libc::pid_t, signo) };
                                        // best-effort: no response frame, just log on server side
                                        info!("ws_pty: forwarded signal {} to pid {}", sig, pid);
                                    }
                                }
                            }
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
    let _ = tx
        .lock()
        .await
        .send(Message::Text(serde_json::to_string(&exit_msg).unwrap()))
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
