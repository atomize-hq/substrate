use crate::bridge;
use crate::config::ForwarderConfig;
use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

pub async fn serve(
    addr: SocketAddr,
    config: Arc<ForwarderConfig>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind tcp listener on {addr}"))?;
    let local_addr = listener.local_addr().unwrap_or(addr);
    tracing::info!(address = %local_addr, "listening on TCP bridge");

    let mut sessions: JoinSet<anyhow::Result<()>> = JoinSet::new();
    let mut counter: u64 = 0;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("tcp listener shutting down");
                break;
            }
            maybe_res = sessions.join_next() => {
                if let Some(res) = maybe_res {
                    match res {
                        Ok(Ok(())) => {}
                        Ok(Err(err)) => tracing::warn!(error = %err, "tcp session ended with error"),
                        Err(join_err) => tracing::warn!("tcp session panicked: {join_err}"),
                    }
                }
            }
            accept_res = listener.accept() => {
                match accept_res {
                    Ok((stream, peer)) => {
                        counter = counter.wrapping_add(1);
                        let cfg = config.clone();
                        sessions.spawn(async move {
                            bridge::run_tcp_session(stream, peer, cfg, counter).await
                        });
                    }
                    Err(err) => {
                        tracing::error!(error = %err, "failed to accept tcp connection");
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }
            }
        }
    }

    while let Some(res) = sessions.join_next().await {
        match res {
            Ok(Ok(())) => {}
            Ok(Err(err)) => tracing::warn!(error = %err, "tcp session ended with error"),
            Err(join_err) => tracing::warn!("tcp session panicked: {join_err}"),
        }
    }

    Ok(())
}
