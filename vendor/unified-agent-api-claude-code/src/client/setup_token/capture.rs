use std::sync::Arc;

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    sync::{oneshot, Mutex},
    task,
};

use crate::{process::ConsoleTarget, ClaudeCodeError};

use super::url::UrlCapture;

async fn record_and_capture(
    bytes: &[u8],
    out: &Arc<Mutex<Vec<u8>>>,
    url_state: &Arc<Mutex<UrlCapture>>,
    url_tx: &Arc<Mutex<Option<oneshot::Sender<String>>>>,
) {
    out.lock().await.extend_from_slice(bytes);

    let text = String::from_utf8_lossy(bytes);
    if let Some(url) = url_state.lock().await.push_text(&text) {
        if let Some(tx) = url_tx.lock().await.take() {
            let _ = tx.send(url);
        }
    }
}

pub(super) fn spawn_capture_task<R>(
    reader: R,
    target: ConsoleTarget,
    mirror_console: bool,
    out: Arc<Mutex<Vec<u8>>>,
    url_state: Arc<Mutex<UrlCapture>>,
    url_tx: Arc<Mutex<Option<oneshot::Sender<String>>>>,
) -> tokio::task::JoinHandle<Result<(), ClaudeCodeError>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut reader = reader;
        let mut chunk = [0u8; 4096];
        loop {
            let n = reader.read(&mut chunk).await.map_err(|e| match target {
                ConsoleTarget::Stdout => ClaudeCodeError::StdoutRead(e),
                ConsoleTarget::Stderr => ClaudeCodeError::StderrRead(e),
            })?;
            if n == 0 {
                break;
            }

            if mirror_console {
                task::block_in_place(|| {
                    let mut w: Box<dyn std::io::Write> = match target {
                        ConsoleTarget::Stdout => Box::new(std::io::stdout()),
                        ConsoleTarget::Stderr => Box::new(std::io::stderr()),
                    };
                    w.write_all(&chunk[..n])?;
                    w.flush()
                })
                .map_err(|e| match target {
                    ConsoleTarget::Stdout => ClaudeCodeError::StdoutRead(e),
                    ConsoleTarget::Stderr => ClaudeCodeError::StderrRead(e),
                })?;
            }

            record_and_capture(&chunk[..n], &out, &url_state, &url_tx).await;
        }
        Ok(())
    })
}

#[cfg(unix)]
pub(super) fn spawn_pty_capture_task(
    mut reader: Box<dyn std::io::Read + Send>,
    target: ConsoleTarget,
    mirror_console: bool,
    out: Arc<Mutex<Vec<u8>>>,
    url_state: Arc<Mutex<UrlCapture>>,
    url_tx: Arc<Mutex<Option<oneshot::Sender<String>>>>,
) -> tokio::task::JoinHandle<Result<(), ClaudeCodeError>> {
    tokio::spawn(async move {
        use tokio::sync::mpsc;

        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let read_task = tokio::task::spawn_blocking(move || -> Result<(), std::io::Error> {
            let mut chunk = [0u8; 4096];
            loop {
                let n = reader.read(&mut chunk)?;
                if n == 0 {
                    break;
                }
                if tx.send(chunk[..n].to_vec()).is_err() {
                    break;
                }
            }
            Ok(())
        });

        while let Some(bytes) = rx.recv().await {
            if mirror_console {
                task::block_in_place(|| {
                    let mut w: Box<dyn std::io::Write> = match target {
                        ConsoleTarget::Stdout => Box::new(std::io::stdout()),
                        ConsoleTarget::Stderr => Box::new(std::io::stderr()),
                    };
                    w.write_all(&bytes)?;
                    w.flush()
                })
                .map_err(|e| match target {
                    ConsoleTarget::Stdout => ClaudeCodeError::StdoutRead(e),
                    ConsoleTarget::Stderr => ClaudeCodeError::StderrRead(e),
                })?;
            }

            record_and_capture(&bytes, &out, &url_state, &url_tx).await;
        }

        read_task
            .await
            .map_err(|e| ClaudeCodeError::Join(e.to_string()))?
            .map_err(|e| match target {
                ConsoleTarget::Stdout => ClaudeCodeError::StdoutRead(e),
                ConsoleTarget::Stderr => ClaudeCodeError::StderrRead(e),
            })?;

        Ok(())
    })
}
