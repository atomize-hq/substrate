use crate::config::BridgeTarget;
use anyhow::Context;
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

DEFAULT_MODE = "uds"
DEFAULT_UDS = "/run/substrate.sock"
DEFAULT_TCP_HOST = "127.0.0.1"
DEFAULT_TCP_PORT = 61337


def connect():
    mode = os.environ.get("SUBSTRATE_FORWARDER_TARGET_MODE", DEFAULT_MODE).lower()
    if mode == "tcp":
        host = os.environ.get("SUBSTRATE_FORWARDER_TARGET_HOST", DEFAULT_TCP_HOST)
        port_value = os.environ.get("SUBSTRATE_FORWARDER_TARGET_PORT")
        try:
            port = int(port_value) if port_value else DEFAULT_TCP_PORT
        except ValueError as exc:
            raise RuntimeError(f"invalid tcp port: {port_value!r}") from exc
        sock = socket.create_connection((host, port))
    else:
        path = os.environ.get("SUBSTRATE_FORWARDER_TARGET_ENDPOINT", DEFAULT_UDS)
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(path)
    sock.setblocking(False)
    return sock


def main():
    sock = connect()

    stdin_fd = sys.stdin.buffer.fileno()
    stdout = sys.stdout.buffer

    selector = selectors.DefaultSelector()
    selector.register(sock, selectors.EVENT_READ)
    selector.register(stdin_fd, selectors.EVENT_READ)

    while True:
        events = selector.select()
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
                    try:
                        selector.unregister(stdin_fd)
                    except Exception:
                        pass
                    try:
                        sock.shutdown(socket.SHUT_WR)
                    except OSError:
                        pass
                else:
                    sock.sendall(data)


if __name__ == "__main__":
    main()
"#;

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

pub async fn spawn(distro: &str, target: &BridgeTarget) -> anyhow::Result<WslStreamBundle> {
    let mut cmd = Command::new("wsl");
    cmd.arg("-d").arg(distro);
    cmd.arg("--");
    cmd.arg("python3");
    cmd.arg("-c").arg(BRIDGE_SCRIPT);
    cmd.env("PYTHONUNBUFFERED", "1");

    match target {
        BridgeTarget::Uds { path } => {
            cmd.env("SUBSTRATE_FORWARDER_TARGET_MODE", "uds");
            cmd.env("SUBSTRATE_FORWARDER_TARGET_ENDPOINT", path);
        }
        BridgeTarget::Tcp { addr } => {
            cmd.env("SUBSTRATE_FORWARDER_TARGET_MODE", "tcp");
            cmd.env("SUBSTRATE_FORWARDER_TARGET_HOST", addr.ip().to_string());
            cmd.env("SUBSTRATE_FORWARDER_TARGET_PORT", addr.port().to_string());
            cmd.env("SUBSTRATE_FORWARDER_TARGET_ENDPOINT", addr.to_string());
        }
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to start wsl -d {distro}"))?;
    let stdin = child.stdin.take().context("missing child stdin")?;
    let stdout = child.stdout.take().context("missing child stdout")?;
    let stderr = child.stderr.take();

    let stderr_task = stderr.map(|stderr| {
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
                            tracing::debug!(target = "forwarder::wsl", "stderr: {trimmed}");
                        }
                    }
                    Err(err) => {
                        tracing::debug!(target = "forwarder::wsl", "stderr read error: {err}");
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
