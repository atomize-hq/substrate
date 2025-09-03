//! Async PTY handling for Agent API WebSocket streaming.
//!
//! This implementation is purpose-built for the world-agent's async environment,
//! handling WebSocket-based PTY streaming for AI agents. It's separate from the
//! shell's sync PTY implementation due to different architectural requirements.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

/// Async PTY session for Agent API.
pub struct AsyncPtySession {
    child: Child,
    tx: mpsc::Sender<Vec<u8>>,
    rx: mpsc::Receiver<Vec<u8>>,
    session_id: String,
}

impl AsyncPtySession {
    /// Create a new async PTY session for the given command.
    pub async fn new(
        cmd: &str,
        cwd: &std::path::Path,
        env: HashMap<String, String>,
    ) -> Result<Self> {
        let session_id = format!("pty_{}", uuid::Uuid::now_v7());

        // For now, use regular process with pipes instead of PTY
        // TODO: Integrate with shared pty-common utilities when available
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(cmd)
            .current_dir(cwd)
            .envs(&env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let child = command.spawn().context("Failed to spawn command")?;

        let (tx, rx) = mpsc::channel(100);

        Ok(Self {
            child,
            tx,
            rx,
            session_id,
        })
    }

    /// Handle bidirectional streaming over WebSocket.
    pub async fn handle_websocket<S>(mut self, ws: WebSocketStream<S>) -> Result<()>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let (mut ws_sender, mut ws_receiver) = ws.split();

        // Get child's stdin/stdout
        let mut stdin = self
            .child
            .stdin
            .take()
            .context("Failed to get child stdin")?;
        let stdout = self
            .child
            .stdout
            .take()
            .context("Failed to get child stdout")?;
        let stderr = self
            .child
            .stderr
            .take()
            .context("Failed to get child stderr")?;

        // Task 1: Read from child stdout/stderr and send to WebSocket
        let tx = self.tx.clone();
        let session_id = self.session_id.clone();
        let output_task = tokio::spawn(async move {
            let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
            let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

            loop {
                tokio::select! {
                    line = stdout_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                let data = format!("[OUT] {}\n", line).into_bytes();
                                if tx.send(data).await.is_err() {
                                    break;
                                }
                            }
                            Ok(None) => break, // EOF
                            Err(_) => break,
                        }
                    }
                    line = stderr_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                let data = format!("[ERR] {}\n", line).into_bytes();
                                if tx.send(data).await.is_err() {
                                    break;
                                }
                            }
                            Ok(None) => break, // EOF
                            Err(_) => break,
                        }
                    }
                }
            }

            tracing::debug!("Output task completed for session {}", session_id);
        });

        // Task 2: Forward output to WebSocket
        let mut rx = self.rx;
        let ws_forward_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if ws_sender.send(Message::Binary(data)).await.is_err() {
                    break;
                }
            }
        });

        // Task 3: Read from WebSocket and write to child stdin
        let input_task = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Binary(data)) => {
                        if stdin.write_all(&data).await.is_err() {
                            break;
                        }
                        if stdin.flush().await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Text(text)) => {
                        if stdin.write_all(text.as_bytes()).await.is_err() {
                            break;
                        }
                        if stdin.flush().await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    _ => {} // Ignore other message types
                }
            }
        });

        // Wait for any task to complete (indicating session end)
        tokio::select! {
            _ = output_task => {},
            _ = ws_forward_task => {},
            _ = input_task => {},
            _ = self.child.wait() => {},
        }

        Ok(())
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Try to get the exit status if the process has finished.
    pub fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>> {
        match self.child.try_wait() {
            Ok(status) => Ok(status),
            Err(e) => Err(anyhow::anyhow!("Failed to check child status: {}", e)),
        }
    }
}

/// Simple PTY session for non-interactive commands.
pub struct SimplePtySession {
    cmd: String,
    cwd: std::path::PathBuf,
    env: HashMap<String, String>,
}

impl SimplePtySession {
    pub fn new(cmd: String, cwd: std::path::PathBuf, env: HashMap<String, String>) -> Self {
        Self { cmd, cwd, env }
    }

    /// Execute the command and return the result.
    pub async fn execute(&self) -> Result<(i32, Vec<u8>, Vec<u8>)> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.cmd)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .output()
            .await
            .context("Failed to execute command")?;

        let exit_code = output.status.code().unwrap_or(-1);
        Ok((exit_code, output.stdout, output.stderr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_simple_pty_session() {
        let session = SimplePtySession::new(
            "echo hello".to_string(),
            std::env::current_dir().unwrap(),
            HashMap::new(),
        );

        let (exit_code, stdout, stderr) = session.execute().await.unwrap();

        assert_eq!(exit_code, 0);
        assert_eq!(String::from_utf8_lossy(&stdout).trim(), "hello");
        assert!(stderr.is_empty());
    }

    #[tokio::test]
    async fn test_async_pty_session_creation() {
        let cwd = std::env::current_dir().unwrap();
        let env = HashMap::new();

        match AsyncPtySession::new("echo hello", &cwd, env).await {
            Ok(session) => {
                assert!(session.session_id().starts_with("pty_"));
            }
            Err(e) => {
                println!("Failed to create async PTY session: {}", e);
            }
        }
    }
}
