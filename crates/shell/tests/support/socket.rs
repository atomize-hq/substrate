#![cfg(unix)]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::json;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

/// Behavior for socket stubs spawned during tests.
#[derive(Clone)]
pub enum SocketResponse {
    /// Responds to `/v1/capabilities` requests with a JSON payload that
    /// advertises socket activation mode.
    Capabilities,
    /// Handles capabilities and execute calls with canned payloads.
    CapabilitiesAndExecute {
        stdout: String,
        stderr: String,
        exit: i32,
        scopes: Vec<String>,
    },
    /// Accepts connections but never returns a response (simulates a stuck
    /// systemd-managed socket where the service failed to start).
    Silent,
}

/// Minimal Unix socket server used to simulate socket-activated world-agent
/// listeners.
pub struct AgentSocket {
    path: PathBuf,
    shutdown: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl AgentSocket {
    /// Spawn a new stub server bound to the provided path.
    pub fn start(path: &Path, response: SocketResponse) -> Self {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create socket parent");
        }
        let _ = fs::remove_file(path);
        let listener = UnixListener::bind(path).expect("failed to bind stub socket");
        listener
            .set_nonblocking(true)
            .expect("failed to configure stub socket");

        let socket_path = path.to_path_buf();
        let cleanup_path = socket_path.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();

        let handle = thread::spawn(move || {
            while !shutdown_flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _addr)) => {
                        let mut buf = [0u8; 4096];
                        let read = stream.read(&mut buf).unwrap_or(0);
                        let request = String::from_utf8_lossy(&buf[..read]);
                        let first_line = request.lines().next().unwrap_or("");
                        match response {
                            SocketResponse::Capabilities => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                                }
                            }
                            SocketResponse::CapabilitiesAndExecute {
                                ref stdout,
                                ref stderr,
                                exit,
                                ref scopes,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute") {
                                    let payload = json!({
                                        "exit": exit,
                                        "span_id": "agent-span",
                                        "stdout_b64": BASE64.encode(stdout.as_bytes()),
                                        "stderr_b64": BASE64.encode(stderr.as_bytes()),
                                        "scopes_used": scopes,
                                        "fs_diff": serde_json::Value::Null
                                    })
                                    .to_string();
                                    write_response(&mut stream, &payload);
                                } else {
                                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                                }
                            }
                            SocketResponse::Silent => {
                                // Read and drop the request to simulate a hung service.
                                let mut discard = [0u8; 512];
                                let _ = stream.read(&mut discard);
                            }
                        };
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }

            let _ = fs::remove_file(&socket_path);
        });

        Self {
            path: cleanup_path,
            shutdown,
            handle: Some(handle),
        }
    }

    /// Return the on-disk socket path for the stub.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for AgentSocket {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = UnixStream::connect(&self.path);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn write_capabilities(stream: &mut UnixStream) {
    let body = json!({
        "version": "v1",
        "features": ["execute"],
        "backend": "world-agent",
        "platform": "linux",
        "listener_mode": "socket_activation"
    })
    .to_string();
    write_response(stream, &body);
}

fn write_world_doctor_report(stream: &mut UnixStream) {
    let body = json!({
        "schema_version": 1,
        "ok": true,
        "collected_at_utc": "2026-01-08T00:00:00Z",
        "landlock": {
            "supported": true,
            "abi": 3,
            "reason": null
        },
        "world_fs_strategy": {
            "primary": "overlay",
            "fallback": "fuse",
            "probe": {
                "id": "enumeration_v1",
                "probe_file": ".substrate_enum_probe",
                "result": "pass",
                "failure_reason": null
            }
        }
    })
    .to_string();
    write_response(stream, &body);
}

fn write_response(stream: &mut UnixStream, body: &str) {
    let reply = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(reply.as_bytes());
}
