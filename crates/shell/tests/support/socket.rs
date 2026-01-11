#![cfg(unix)]
#![allow(dead_code)]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Deserialize;
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
    /// Executes `/v1/execute` and `/v1/execute/stream` requests on the host, using
    /// the request's `cwd` and `env` for a lightweight world-agent simulation.
    CapabilitiesAndHostExecute { scopes: Vec<String> },
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
                        let request = match read_http_request(&mut stream) {
                            Ok(req) => req,
                            Err(_) => continue,
                        };
                        let first_line = request.header.lines().next().unwrap_or("");
                        match &response {
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
                                stdout,
                                stderr,
                                exit,
                                scopes,
                            } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    let payload =
                                        build_stream_payload(*exit, stdout, stderr, scopes);
                                    write_stream_response(&mut stream, &payload);
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
                            SocketResponse::CapabilitiesAndHostExecute { scopes } => {
                                if first_line.starts_with("GET /v1/capabilities") {
                                    write_capabilities(&mut stream);
                                } else if first_line.starts_with("GET /v1/doctor/world") {
                                    write_world_doctor_report(&mut stream);
                                } else if first_line.starts_with("POST /v1/execute/stream") {
                                    match handle_host_execute_stream(&request, scopes) {
                                        Ok(payload) => write_stream_response(&mut stream, &payload),
                                        Err(err) => {
                                            let payload = json!({
                                                "type": "error",
                                                "message": format!("{:#}", err)
                                            })
                                            .to_string();
                                            write_stream_response(&mut stream, &(payload + "\n"));
                                        }
                                    }
                                } else if first_line.starts_with("POST /v1/execute") {
                                    match handle_host_execute(&request, scopes) {
                                        Ok(payload) => write_response(&mut stream, &payload),
                                        Err(err) => {
                                            let payload = json!({
                                                "error": "internal",
                                                "message": format!("{:#}", err)
                                            })
                                            .to_string();
                                            let _ = stream.write_all(
                                                format!(
                                                    "HTTP/1.1 500 Internal Server Error\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                                    payload.len(),
                                                    payload
                                                )
                                                .as_bytes(),
                                            );
                                        }
                                    }
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

fn write_stream_response(stream: &mut UnixStream, body: &str) {
    let reply = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(reply.as_bytes());
}

struct HttpRequest {
    header: String,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut UnixStream) -> std::io::Result<HttpRequest> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let mut header_end = None;
    let mut expected_len = None;
    let mut chunked = false;

    loop {
        let read = stream.read(&mut tmp)?;
        if read == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..read]);

        if header_end.is_none() {
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                let header = String::from_utf8_lossy(&buf[..pos + 4]).to_string();
                expected_len = Some(parse_content_length(&header));
                chunked = header.lines().any(|line| {
                    line.to_ascii_lowercase().starts_with("transfer-encoding:")
                        && line.to_ascii_lowercase().contains("chunked")
                });
            }
        }

        if let Some(h_end) = header_end {
            if chunked {
                if decode_chunked_body(&buf[h_end..]).is_some() {
                    break;
                }
            } else if let Some(len) = expected_len {
                if buf.len() >= h_end + len {
                    break;
                }
            }
        }
    }

    let header_end = header_end.unwrap_or(buf.len());
    let header = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let body = if chunked {
        decode_chunked_body(&buf[header_end..]).unwrap_or_default()
    } else {
        let len = expected_len.unwrap_or_else(|| parse_content_length(&header));
        let body_start = header_end;
        let body_end = std::cmp::min(body_start + len, buf.len());
        buf[body_start..body_end].to_vec()
    };

    Ok(HttpRequest { header, body })
}

fn parse_content_length(header: &str) -> usize {
    header
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            if key.eq_ignore_ascii_case("content-length") {
                Some(value.trim().parse::<usize>().ok()?)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn decode_chunked_body(buf: &[u8]) -> Option<Vec<u8>> {
    let mut pos = 0usize;
    let mut out = Vec::new();

    loop {
        let line_end = buf[pos..].windows(2).position(|w| w == b"\r\n")? + pos;
        let line = &buf[pos..line_end];
        let line_str = std::str::from_utf8(line).ok()?;
        let size_str = line_str.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_str, 16).ok()?;
        pos = line_end + 2;
        if size == 0 {
            // Expect trailing CRLF after the 0-size chunk payload.
            return Some(out);
        }
        if buf.len() < pos + size + 2 {
            return None;
        }
        out.extend_from_slice(&buf[pos..pos + size]);
        pos += size;
        if &buf[pos..pos + 2] != b"\r\n" {
            return None;
        }
        pos += 2;
    }
}

#[derive(Debug, Deserialize)]
struct ExecuteRequestStub {
    cmd: String,
    cwd: Option<String>,
    env: Option<std::collections::HashMap<String, String>>,
}

fn handle_host_execute(request: &HttpRequest, scopes: &[String]) -> anyhow::Result<String> {
    let parsed: ExecuteRequestStub = serde_json::from_slice(&request.body)?;
    let output = run_host_command(&parsed)?;
    Ok(json!({
        "exit": output.exit,
        "span_id": "agent-span",
        "stdout_b64": BASE64.encode(&output.stdout),
        "stderr_b64": BASE64.encode(&output.stderr),
        "scopes_used": scopes,
        "fs_diff": serde_json::Value::Null
    })
    .to_string())
}

fn handle_host_execute_stream(request: &HttpRequest, scopes: &[String]) -> anyhow::Result<String> {
    let parsed: ExecuteRequestStub = serde_json::from_slice(&request.body)?;
    let output = run_host_command(&parsed)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(build_stream_payload(
        output.exit,
        stdout.as_ref(),
        stderr.as_ref(),
        scopes,
    ))
}

struct HostCommandOutput {
    exit: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_host_command(request: &ExecuteRequestStub) -> anyhow::Result<HostCommandOutput> {
    use std::process::Command;

    let mut cmd = Command::new("bash");
    cmd.arg("-lc").arg(&request.cmd);
    if let Some(cwd) = &request.cwd {
        cmd.current_dir(cwd);
    }
    if let Some(env) = &request.env {
        cmd.envs(env);
    }

    let output = cmd.output()?;
    Ok(HostCommandOutput {
        exit: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn build_stream_payload(exit: i32, stdout: &str, stderr: &str, scopes: &[String]) -> String {
    let mut frames = String::new();
    frames.push_str(
        &json!({
            "type": "start",
            "span_id": "agent-span"
        })
        .to_string(),
    );
    frames.push('\n');
    if !stdout.is_empty() {
        frames.push_str(
            &json!({
                "type": "stdout",
                "chunk_b64": BASE64.encode(stdout.as_bytes())
            })
            .to_string(),
        );
        frames.push('\n');
    }
    if !stderr.is_empty() {
        frames.push_str(
            &json!({
                "type": "stderr",
                "chunk_b64": BASE64.encode(stderr.as_bytes())
            })
            .to_string(),
        );
        frames.push('\n');
    }
    frames.push_str(
        &json!({
            "type": "exit",
            "exit": exit,
            "span_id": "agent-span",
            "scopes_used": scopes,
            "fs_diff": serde_json::Value::Null
        })
        .to_string(),
    );
    frames.push('\n');
    frames
}
