use crate::config::BridgeTarget;
use anyhow::Context;
use std::collections::BTreeMap;
use std::io;
use std::pin::Pin;
use std::process::ExitStatus;
use std::task::{Context as TaskContext, Poll};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::task::JoinHandle;

const BRIDGE_SCRIPT: &str = r#"
import os
import selectors
import socket
import sys
import time

DEFAULT_MODE = "uds"
DEFAULT_UDS = "/run/substrate.sock"
DEFAULT_TCP_HOST = "127.0.0.1"
DEFAULT_TCP_PORT = 61337
DEFAULT_CONNECT_TIMEOUT_S = 2.0
DEFAULT_CONNECT_DEADLINE_S = 10.0
DEFAULT_IDLE_AFTER_STDIN_CLOSE_S = 2.0


def _float_env(name, default):
    raw = os.environ.get(name)
    if raw is None:
        return default
    raw = raw.strip()
    if not raw:
        return default
    try:
        return float(raw)
    except ValueError:
        return default


def connect():
    mode = os.environ.get("SUBSTRATE_FORWARDER_TARGET_MODE", DEFAULT_MODE).lower()
    connect_timeout_s = _float_env("SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S", DEFAULT_CONNECT_TIMEOUT_S)
    connect_deadline_s = _float_env("SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S", DEFAULT_CONNECT_DEADLINE_S)
    deadline = time.monotonic() + connect_deadline_s

    while True:
        try:
            if mode == "tcp":
                host = os.environ.get("SUBSTRATE_FORWARDER_TARGET_HOST", DEFAULT_TCP_HOST)
                port_value = os.environ.get("SUBSTRATE_FORWARDER_TARGET_PORT")
                try:
                    port = int(port_value) if port_value else DEFAULT_TCP_PORT
                except ValueError as exc:
                    raise RuntimeError(f"invalid tcp port: {port_value!r}") from exc
                sock = socket.create_connection((host, port), timeout=connect_timeout_s)
            else:
                path = os.environ.get("SUBSTRATE_FORWARDER_TARGET_ENDPOINT", DEFAULT_UDS)
                sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                sock.settimeout(connect_timeout_s)
                sock.connect(path)

            sock.setblocking(False)
            return sock
        except Exception as exc:
            if time.monotonic() >= deadline:
                raise RuntimeError(
                    f"timed out connecting to target mode={mode!r} after {connect_deadline_s:.1f}s: {exc}"
                ) from exc
            time.sleep(0.25)


def idle_after_stdin_close_s():
    return _float_env(
        "SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S",
        DEFAULT_IDLE_AFTER_STDIN_CLOSE_S,
    )


def main():
    sock = connect()

    stdin_fd = sys.stdin.buffer.fileno()
    stdout = sys.stdout.buffer

    selector = selectors.DefaultSelector()
    selector.register(sock, selectors.EVENT_READ)
    selector.register(stdin_fd, selectors.EVENT_READ)

    stdin_closed = False
    last_activity = time.monotonic()
    idle_deadline = None
    idle_timeout_s = idle_after_stdin_close_s()

    while True:
        events = selector.select(timeout=1.0)
        now = time.monotonic()

        if stdin_closed and idle_deadline is not None and now >= idle_deadline:
            try:
                sock.close()
            except Exception:
                pass
            return

        if not events:
            continue

        last_activity = now
        for key, _ in events:
            if key.fileobj is sock:
                data = sock.recv(65536)
                if not data:
                    return
                stdout.write(data)
                stdout.flush()
            else:
                try:
                    data = os.read(stdin_fd, 65536)
                except BlockingIOError:
                    continue
                if not data:
                    stdin_closed = True
                    try:
                        selector.unregister(stdin_fd)
                    except Exception:
                        pass
                    try:
                        sock.shutdown(socket.SHUT_WR)
                    except OSError:
                        pass
                    idle_deadline = last_activity + idle_timeout_s
                else:
                    try:
                        sock.sendall(data)
                    except BrokenPipeError:
                        return


if __name__ == "__main__":
    main()
"#;

const HOST_CONTEXT_COMMITMENT_ENV: &str = "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT";
const WSLENV_EXPORTS: &[&str] = &[
    "SUBSTRATE_FORWARDER_TARGET_MODE",
    "SUBSTRATE_FORWARDER_TARGET_HOST",
    "SUBSTRATE_FORWARDER_TARGET_PORT",
    "SUBSTRATE_FORWARDER_TARGET_ENDPOINT",
    HOST_CONTEXT_COMMITMENT_ENV,
    "SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S",
    "SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S",
    "SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S",
];
const CONNECT_TIMEOUT_ENV: &str = "SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S";
const CONNECT_TIMEOUT_VALUE: &str = "2";
const CONNECT_DEADLINE_ENV: &str = "SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S";
const CONNECT_DEADLINE_VALUE: &str = "10";
const IDLE_AFTER_STDIN_CLOSE_ENV: &str = "SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S";
const IDLE_AFTER_STDIN_CLOSE_VALUE: &str = "2";

#[derive(Debug, PartialEq, Eq)]
struct SpawnSpec {
    program: String,
    args: Vec<String>,
    env_remove: Vec<&'static str>,
    env_set: BTreeMap<&'static str, String>,
}

fn trusted_wsl_program() -> anyhow::Result<String> {
    #[cfg(test)]
    {
        Ok(r"C:\Windows\System32\wsl.exe".to_string())
    }

    #[cfg(not(test))]
    {
        unsafe extern "system" {
            fn GetSystemDirectoryW(lpbuffer: *mut u16, usize: u32) -> u32;
        }

        let mut buffer = vec![0_u16; 32768];
        let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };
        if length == 0 || length as usize >= buffer.len() {
            anyhow::bail!("trusted Windows system directory is unavailable");
        }
        let system_directory = String::from_utf16(&buffer[..length as usize])
            .context("trusted Windows system directory is malformed")?;
        Ok(format!(r"{}\wsl.exe", system_directory))
    }
}

fn build_spawn_spec(
    distro: &str,
    target: &BridgeTarget,
    host_context_commitment: Option<&str>,
) -> anyhow::Result<SpawnSpec> {
    if let Some(host_context_commitment) = host_context_commitment {
        if host_context_commitment.len() != 64
            || !host_context_commitment
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            anyhow::bail!("forwarder WSL spawn requires a canonical lowercase host commitment");
        }
    }

    let mut env_set = BTreeMap::new();
    env_set.insert("PYTHONUNBUFFERED", "1".to_string());
    env_set.insert(
        "WSLENV",
        WSLENV_EXPORTS.iter().copied().collect::<Vec<_>>().join(":"),
    );
    env_set.insert(CONNECT_TIMEOUT_ENV, CONNECT_TIMEOUT_VALUE.to_string());
    env_set.insert(CONNECT_DEADLINE_ENV, CONNECT_DEADLINE_VALUE.to_string());
    env_set.insert(
        IDLE_AFTER_STDIN_CLOSE_ENV,
        IDLE_AFTER_STDIN_CLOSE_VALUE.to_string(),
    );
    if let Some(host_context_commitment) = host_context_commitment {
        env_set.insert(
            HOST_CONTEXT_COMMITMENT_ENV,
            host_context_commitment.to_string(),
        );
    }

    match target {
        BridgeTarget::Uds { path } => {
            env_set.insert("SUBSTRATE_FORWARDER_TARGET_MODE", "uds".to_string());
            env_set.insert("SUBSTRATE_FORWARDER_TARGET_ENDPOINT", path.clone());
        }
        BridgeTarget::Tcp { addr } => {
            env_set.insert("SUBSTRATE_FORWARDER_TARGET_MODE", "tcp".to_string());
            env_set.insert("SUBSTRATE_FORWARDER_TARGET_HOST", addr.ip().to_string());
            env_set.insert("SUBSTRATE_FORWARDER_TARGET_PORT", addr.port().to_string());
            env_set.insert("SUBSTRATE_FORWARDER_TARGET_ENDPOINT", addr.to_string());
        }
    }

    Ok(SpawnSpec {
        program: trusted_wsl_program()?,
        args: vec![
            "-d".to_string(),
            distro.to_string(),
            "--".to_string(),
            "python3".to_string(),
            "-c".to_string(),
            BRIDGE_SCRIPT.to_string(),
        ],
        env_remove: vec![
            "WSLENV",
            "SUBSTRATE_FORWARDER_TARGET",
            "SUBSTRATE_FORWARDER_TARGET_MODE",
            "SUBSTRATE_FORWARDER_TARGET_HOST",
            "SUBSTRATE_FORWARDER_TARGET_PORT",
            "SUBSTRATE_FORWARDER_TARGET_ENDPOINT",
            HOST_CONTEXT_COMMITMENT_ENV,
            "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1",
            "SUBSTRATE_FORWARDER_PIPE",
            "SUBSTRATE_FORWARDER_TCP",
            "SUBSTRATE_FORWARDER_TCP_ADDR",
            "SUBSTRATE_FORWARDER_TCP_HOST",
            "SUBSTRATE_FORWARDER_TCP_PORT",
            CONNECT_TIMEOUT_ENV,
            CONNECT_DEADLINE_ENV,
            IDLE_AFTER_STDIN_CLOSE_ENV,
        ],
        env_set,
    })
}

pub struct WslStream {
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl WslStream {
    pub async fn shutdown(&mut self) -> io::Result<()> {
        self.stdin.shutdown().await
    }
}

impl AsyncRead for WslStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        unsafe { self.map_unchecked_mut(|this| &mut this.stdout) }.poll_read(cx, buf)
    }
}

impl AsyncWrite for WslStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        data: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        unsafe { self.map_unchecked_mut(|this| &mut this.stdin) }.poll_write(cx, data)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<io::Result<()>> {
        unsafe { self.map_unchecked_mut(|this| &mut this.stdin) }.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<io::Result<()>> {
        unsafe { self.map_unchecked_mut(|this| &mut this.stdin) }.poll_shutdown(cx)
    }
}

pub struct WslStreamBundle {
    stream: Option<WslStream>,
    child: Child,
    stderr_task: Option<JoinHandle<()>>,
}

impl WslStreamBundle {
    pub fn stream_mut(&mut self) -> &mut WslStream {
        self.stream.as_mut().expect("stream not available")
    }

    pub async fn wait(mut self) -> io::Result<ExitStatus> {
        self.stream.take();
        if let Some(handle) = self.stderr_task.take() {
            if let Err(err) = handle.await {
                tracing::debug!("stderr task join error: {err}");
            }
        }
        self.child.wait().await
    }
}

pub async fn spawn(
    distro: &str,
    target: &BridgeTarget,
    host_context_commitment: Option<&str>,
    session_id: u64,
    label: &str,
) -> anyhow::Result<WslStreamBundle> {
    let spec = build_spawn_spec(distro, target, host_context_commitment)?;
    let mut cmd = Command::new(&spec.program);
    for arg in &spec.args {
        cmd.arg(arg);
    }
    for key in &spec.env_remove {
        cmd.env_remove(key);
    }
    for (key, value) in &spec.env_set {
        cmd.env(key, value);
    }

    use std::process::Stdio;
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start wsl -d {distro}"))?;
    let stdin = child.stdin.take().context("missing child stdin")?;
    let stdout = child.stdout.take().context("missing child stdout")?;
    let stderr = child.stderr.take();

    let lbl = label.to_string();
    let sid = session_id;
    let stderr_task = stderr.map(move |stderr| {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let trimmed = line.trim_end();
                        if !trimmed.is_empty() {
                            tracing::debug!(
                                target = "forwarder::wsl",
                                session = sid,
                                kind = lbl,
                                "stderr: {trimmed}"
                            );
                        }
                    }
                    Err(err) => {
                        tracing::debug!(
                            target = "forwarder::wsl",
                            session = sid,
                            kind = lbl,
                            "stderr read error: {err}"
                        );
                        break;
                    }
                }
            }
        })
    });

    Ok(WslStreamBundle {
        stream: Some(WslStream { stdin, stdout }),
        child,
        stderr_task,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddr};

    #[test]
    fn spawn_spec_uses_exact_registered_distro_and_product_uds_target() {
        let spec = build_spawn_spec(
            "Substrate-WSL",
            &BridgeTarget::Uds {
                path: "/run/substrate.sock".to_string(),
            },
            Some("3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7"),
        )
        .unwrap();

        assert_eq!(spec.program, r"C:\Windows\System32\wsl.exe");
        assert_eq!(spec.args[0], "-d");
        assert_eq!(spec.args[1], "Substrate-WSL");
        assert_eq!(spec.args[2], "--");
        assert_eq!(spec.env_set["SUBSTRATE_FORWARDER_TARGET_MODE"], "uds");
        assert_eq!(
            spec.env_set["SUBSTRATE_FORWARDER_TARGET_ENDPOINT"],
            "/run/substrate.sock"
        );
        assert_eq!(
            spec.env_set[HOST_CONTEXT_COMMITMENT_ENV],
            "3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7"
        );
        assert_eq!(spec.env_set[CONNECT_TIMEOUT_ENV], CONNECT_TIMEOUT_VALUE);
        assert_eq!(spec.env_set[CONNECT_DEADLINE_ENV], CONNECT_DEADLINE_VALUE);
        assert_eq!(
            spec.env_set[IDLE_AFTER_STDIN_CLOSE_ENV],
            IDLE_AFTER_STDIN_CLOSE_VALUE
        );
        assert_eq!(
            spec.env_set["WSLENV"],
            "SUBSTRATE_FORWARDER_TARGET_MODE:SUBSTRATE_FORWARDER_TARGET_HOST:SUBSTRATE_FORWARDER_TARGET_PORT:SUBSTRATE_FORWARDER_TARGET_ENDPOINT:SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT:SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S:SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S:SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S"
        );
        assert!(spec.env_remove.contains(&"WSLENV"));
        assert!(spec.env_remove.contains(&"SUBSTRATE_FORWARDER_TARGET"));
        assert!(spec
            .env_remove
            .contains(&"SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1"));
        assert!(spec.env_remove.contains(&CONNECT_TIMEOUT_ENV));
        assert!(spec.env_remove.contains(&CONNECT_DEADLINE_ENV));
        assert!(spec.env_remove.contains(&IDLE_AFTER_STDIN_CLOSE_ENV));
    }

    #[test]
    fn spawn_spec_replaces_inherited_target_projection_for_tcp_diagnostics() {
        let spec = build_spawn_spec(
            "Substrate-WSL",
            &BridgeTarget::Tcp {
                addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 61337)),
            },
            Some("3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7"),
        )
        .unwrap();

        assert_eq!(spec.env_set["SUBSTRATE_FORWARDER_TARGET_MODE"], "tcp");
        assert_eq!(spec.env_set["SUBSTRATE_FORWARDER_TARGET_HOST"], "127.0.0.1");
        assert_eq!(spec.env_set["SUBSTRATE_FORWARDER_TARGET_PORT"], "61337");
        assert_eq!(
            spec.env_set["SUBSTRATE_FORWARDER_TARGET_ENDPOINT"],
            "127.0.0.1:61337"
        );
        assert!(!spec.env_set.contains_key("SUBSTRATE_FORWARDER_TARGET"));
        assert!(spec.env_remove.contains(&"SUBSTRATE_FORWARDER_PIPE"));
        assert!(spec.env_remove.contains(&"SUBSTRATE_FORWARDER_TCP_ADDR"));
    }

    #[test]
    fn spawn_spec_rejects_noncanonical_host_commitment() {
        let err = build_spawn_spec(
            "Substrate-WSL",
            &BridgeTarget::Uds {
                path: "/run/substrate.sock".to_string(),
            },
            Some("INVALID"),
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("requires a canonical lowercase host commitment"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn spawn_spec_scrubs_commitment_for_legacy_forwarder_mode() {
        let spec = build_spawn_spec(
            "Substrate-WSL",
            &BridgeTarget::Uds {
                path: "/run/substrate.sock".to_string(),
            },
            None,
        )
        .unwrap();

        assert!(!spec.env_set.contains_key(HOST_CONTEXT_COMMITMENT_ENV));
        assert!(spec.env_remove.contains(&HOST_CONTEXT_COMMITMENT_ENV));
        assert_eq!(spec.program, r"C:\Windows\System32\wsl.exe");
    }
}
