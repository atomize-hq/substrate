use crate::config::{BridgeTarget, ForwarderConfig};
use crate::wsl;
use anyhow::Context;
use std::io;
use std::net::SocketAddr;
use std::os::windows::io::AsRawHandle;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio::net::TcpStream;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::FlushFileBuffers;

pub async fn run_pipe_session(
    mut pipe: NamedPipeServer,
    config: Arc<ForwarderConfig>,
    session_id: u64,
) -> anyhow::Result<()> {
    let target = config.target().clone();
    tracing::info!(
        session = session_id,
        kind = "pipe",
        target_mode = target.mode(),
        target = %target,
        "client connected"
    );

    let mut wsl_bundle = spawn_bridge(&config, &target, session_id, "pipe").await?;
    {
        let mut bridge_stream = wsl_bundle.stream_mut();
        bridge_copy(session_id, "pipe", &target, &mut pipe, &mut bridge_stream).await;

        match pipe.flush().await {
            Ok(()) => tracing::trace!(session = session_id, "pipe async flush complete"),
            Err(err) => tracing::debug!(session = session_id, "pipe async flush error: {err}"),
        }
        match flush_pipe_buffers(&pipe) {
            Ok(()) => tracing::trace!(session = session_id, "FlushFileBuffers complete"),
            Err(err) => tracing::debug!(session = session_id, "FlushFileBuffers error: {err}"),
        }
        match pipe.disconnect() {
            Ok(()) => tracing::trace!(session = session_id, "pipe disconnect complete"),
            Err(err) => tracing::debug!(session = session_id, "pipe disconnect error: {err}"),
        }

        if let Err(err) = bridge_stream.shutdown().await {
            tracing::debug!(
                session = session_id,
                target_mode = target.mode(),
                target = %target,
                "WSL stream shutdown error: {err}"
            );
        }
    }

    finalize_bridge(wsl_bundle, session_id, &target).await
}

pub async fn run_tcp_session(
    mut stream: TcpStream,
    peer: SocketAddr,
    config: Arc<ForwarderConfig>,
    session_id: u64,
) -> anyhow::Result<()> {
    let target = config.target().clone();
    let label = format!("tcp:{peer}");
    tracing::info!(
        session = session_id,
        kind = "tcp",
        peer = %peer,
        target_mode = target.mode(),
        target = %target,
        "client connected"
    );

    let mut wsl_bundle = spawn_bridge(&config, &target, session_id, &label).await?;
    {
        let mut bridge_stream = wsl_bundle.stream_mut();
        bridge_copy(session_id, &label, &target, &mut stream, &mut bridge_stream).await;

        if let Err(err) = stream.shutdown().await {
            tracing::debug!(session = session_id, "tcp client shutdown error: {err}");
        }
        if let Err(err) = bridge_stream.shutdown().await {
            tracing::debug!(
                session = session_id,
                target_mode = target.mode(),
                target = %target,
                "WSL stream shutdown error: {err}"
            );
        }
    }

    finalize_bridge(wsl_bundle, session_id, &target).await
}

async fn spawn_bridge(
    config: &ForwarderConfig,
    target: &BridgeTarget,
    session_id: u64,
    label: &str,
) -> anyhow::Result<wsl::WslStreamBundle> {
    wsl::spawn(&config.distro, target, session_id, label)
        .await
        .with_context(|| {
            format!("session {session_id}: failed to spawn WSL bridge for {label} via {target}")
        })
}

async fn finalize_bridge(
    bundle: wsl::WslStreamBundle,
    session_id: u64,
    target: &BridgeTarget,
) -> anyhow::Result<()> {
    let status = bundle
        .wait()
        .await
        .with_context(|| format!("session {session_id}: failed to wait for WSL process"))?;
    if !status.success() {
        tracing::warn!(
            session = session_id,
            target_mode = target.mode(),
            target = %target,
            exit = ?status.code(),
            "WSL bridge exited with failure"
        );
    }

    Ok(())
}

async fn bridge_copy<C, W>(
    session_id: u64,
    label: &str,
    target: &BridgeTarget,
    client: &mut C,
    bridge: &mut W,
) where
    C: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    W: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    match tokio::io::copy_bidirectional(client, bridge).await {
        Ok((c2w, w2c)) => {
            tracing::debug!(
                session = session_id,
                kind = label,
                target_mode = target.mode(),
                target = %target,
                client_to_wsl = c2w,
                wsl_to_client = w2c,
                "stream closed"
            );
        }
        Err(err) => {
            tracing::warn!(
                session = session_id,
                kind = label,
                target_mode = target.mode(),
                target = %target,
                error = %err,
                "bidirectional copy failed"
            );
        }
    }
}

fn flush_pipe_buffers(pipe: &NamedPipeServer) -> io::Result<()> {
    let handle = pipe.as_raw_handle();
    unsafe {
        FlushFileBuffers(HANDLE(handle as *mut _))
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
    }
    Ok(())
}
