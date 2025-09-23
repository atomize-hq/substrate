use crate::config::ForwarderConfig;
use crate::wsl;
use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

pub async fn run_pipe_session(
    mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    config: Arc<ForwarderConfig>,
    session_id: u64,
) -> anyhow::Result<()> {
    tracing::info!(session = session_id, kind = "pipe", "client connected");
    run_session(&mut pipe, SessionKind::Pipe, config, session_id).await
}

pub async fn run_tcp_session(
    mut stream: tokio::net::TcpStream,
    peer: SocketAddr,
    config: Arc<ForwarderConfig>,
    session_id: u64,
) -> anyhow::Result<()> {
    tracing::info!(session = session_id, kind = "tcp", peer = %peer, "client connected");
    run_session(&mut stream, SessionKind::Tcp { peer }, config, session_id).await
}

enum SessionKind {
    Pipe,
    Tcp { peer: SocketAddr },
}

async fn run_session<S>(
    client: &mut S,
    kind: SessionKind,
    config: Arc<ForwarderConfig>,
    session_id: u64,
) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let mut wsl_bundle = wsl::spawn(&config.distro)
        .await
        .with_context(|| format!("session {session_id}: failed to spawn WSL bridge"))?;

    let mut stream = wsl_bundle.stream_mut();
    match tokio::io::copy_bidirectional(client, &mut stream).await {
        Ok((c2w, w2c)) => {
            tracing::debug!(session = session_id, kind = %session_label(&kind), client_to_wsl = c2w, wsl_to_client = w2c, "stream closed");
        }
        Err(err) => {
            tracing::warn!(session = session_id, kind = %session_label(&kind), error = %err, "bidirectional copy failed");
        }
    }

    if let Err(err) = client.shutdown().await {
        tracing::debug!(session = session_id, "client shutdown error: {err}");
    }
    if let Err(err) = stream.shutdown().await {
        tracing::debug!(session = session_id, "WSL stream shutdown error: {err}");
    }

    let status = wsl_bundle
        .wait()
        .await
        .with_context(|| format!("session {session_id}: failed to wait for WSL process"))?;
    if !status.success() {
        tracing::warn!(session = session_id, exit = ?status.code(), "WSL bridge exited with failure");
    }

    Ok(())
}

fn session_label(kind: &SessionKind) -> String {
    match kind {
        SessionKind::Pipe => "pipe".to_string(),
        SessionKind::Tcp { peer } => format!("tcp:{peer}"),
    }
}
