use std::{sync::Arc, time::Duration};

use tokio::{
    io::AsyncWriteExt,
    sync::{oneshot, Mutex},
    time,
};

use crate::{ClaudeCodeError, CommandOutput};

use super::{process::SetupTokenProcess, url::extract_oauth_url};

#[cfg(unix)]
use super::pty::portable_exit_status_to_std;

const CLEANUP_GRACE: Duration = Duration::from_secs(2);

async fn join_or_abort<T>(mut handle: tokio::task::JoinHandle<T>, grace: Duration) -> Option<T> {
    if grace.is_zero() {
        handle.abort();
        let _ = handle.await;
        return None;
    }

    tokio::select! {
        output = &mut handle => output.ok(),
        _ = time::sleep(grace) => {
            handle.abort();
            let _ = handle.await;
            None
        }
    }
}

pub struct ClaudeSetupTokenSession {
    url: String,
    url_rx: Option<oneshot::Receiver<String>>,
    process: Option<SetupTokenProcess>,
    stdout_buf: Arc<Mutex<Vec<u8>>>,
    stderr_buf: Arc<Mutex<Vec<u8>>>,
    stdout_task: Option<tokio::task::JoinHandle<Result<(), ClaudeCodeError>>>,
    stderr_task: Option<tokio::task::JoinHandle<Result<(), ClaudeCodeError>>>,
    timeout: Option<Duration>,
}

impl std::fmt::Debug for ClaudeSetupTokenSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudeSetupTokenSession")
            .field("url", &self.url)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl ClaudeSetupTokenSession {
    pub(super) fn new(
        process: SetupTokenProcess,
        stdout_buf: Arc<Mutex<Vec<u8>>>,
        stderr_buf: Arc<Mutex<Vec<u8>>>,
        stdout_task: tokio::task::JoinHandle<Result<(), ClaudeCodeError>>,
        stderr_task: Option<tokio::task::JoinHandle<Result<(), ClaudeCodeError>>>,
        url_rx: oneshot::Receiver<String>,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            url: String::new(),
            url_rx: Some(url_rx),
            process: Some(process),
            stdout_buf,
            stderr_buf,
            stdout_task: Some(stdout_task),
            stderr_task,
            timeout,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn wait_for_url(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<&str>, ClaudeCodeError> {
        if !self.url.is_empty() {
            return Ok(Some(self.url.as_str()));
        }

        let deadline = time::Instant::now() + timeout;
        let poll_interval = Duration::from_millis(25);

        loop {
            if !self.url.is_empty() {
                return Ok(Some(self.url.as_str()));
            }

            // Parse from the full captured output buffer (more robust than relying only on
            // incremental chunk parsing, and avoids “TTY noise” issues).
            if let Some(url) = self.extract_url_from_captured_output().await {
                self.url = url;
                self.url_rx = None;
                return Ok(Some(self.url.as_str()));
            }

            if time::Instant::now() >= deadline {
                self.url_rx = None;
                return Ok(None);
            }

            let Some(rx) = self.url_rx.as_mut() else {
                time::sleep(poll_interval).await;
                continue;
            };

            let remaining = deadline.saturating_duration_since(time::Instant::now());
            let nap = std::cmp::min(poll_interval, remaining);

            match time::timeout(nap, rx).await {
                Ok(Ok(url)) => {
                    self.url = url;
                    self.url_rx = None;
                    return Ok(Some(self.url.as_str()));
                }
                Ok(Err(_closed)) => {
                    self.url_rx = None;
                    // Keep looping: URL might still be extractable from buffered output.
                }
                Err(_timeout) => {
                    // Keep looping; we'll re-check buffer.
                }
            }
        }
    }

    async fn extract_url_from_captured_output(&self) -> Option<String> {
        let mut combined = Vec::new();
        combined.extend_from_slice(&self.stdout_buf.lock().await);
        combined.extend_from_slice(&self.stderr_buf.lock().await);
        let text = String::from_utf8_lossy(&combined);
        extract_oauth_url(&text)
    }

    pub async fn submit_code(mut self, code: &str) -> Result<CommandOutput, ClaudeCodeError> {
        let process = self
            .process
            .as_mut()
            .expect("setup-token session process present");

        match process {
            SetupTokenProcess::Pipes { stdin, .. } => {
                if let Some(mut stdin) = stdin.take() {
                    let mut bytes = code.as_bytes().to_vec();
                    if !bytes.ends_with(b"\n") {
                        bytes.push(b'\n');
                    }
                    stdin
                        .write_all(&bytes)
                        .await
                        .map_err(ClaudeCodeError::StdinWrite)?;
                }
            }
            #[cfg(unix)]
            SetupTokenProcess::Pty { writer, .. } => {
                if let Some(mut writer) = writer.take() {
                    let mut bytes = code.as_bytes().to_vec();
                    if !bytes.ends_with(b"\n") {
                        bytes.push(b'\n');
                    }

                    tokio::task::spawn_blocking(move || {
                        writer.write_all(&bytes)?;
                        writer.flush()
                    })
                    .await
                    .map_err(|e| ClaudeCodeError::Join(e.to_string()))?
                    .map_err(ClaudeCodeError::StdinWrite)?;
                }
            }
        };
        self.wait().await
    }

    #[cfg(unix)]
    async fn reap_pty_until(
        child: &mut Box<dyn portable_pty::Child + Send + Sync>,
        deadline: time::Instant,
        poll_interval: Duration,
    ) -> Option<std::process::ExitStatus> {
        loop {
            match child.try_wait() {
                Ok(Some(exit)) => return Some(portable_exit_status_to_std(exit)),
                Ok(None) => {
                    if time::Instant::now() >= deadline {
                        return None;
                    }
                    time::sleep(poll_interval).await;
                }
                Err(_) => return None,
            }
        }
    }

    pub async fn wait(mut self) -> Result<CommandOutput, ClaudeCodeError> {
        let process = self
            .process
            .take()
            .expect("setup-token session process present");

        let timeout = self.timeout;
        let mut status: Option<std::process::ExitStatus> = None;
        let mut timed_out: Option<Duration> = None;
        let mut wait_error: Option<ClaudeCodeError> = None;
        let mut cleanup_deadline: Option<time::Instant> = None;

        match process {
            SetupTokenProcess::Pipes { mut child, .. } => {
                if let Some(dur) = timeout {
                    match time::timeout(dur, child.wait()).await {
                        Ok(Ok(exit)) => {
                            status = Some(exit);
                        }
                        Ok(Err(source)) => {
                            wait_error = Some(ClaudeCodeError::Wait(source));
                        }
                        Err(_) => {
                            timed_out = Some(dur);
                        }
                    }
                } else {
                    match child.wait().await {
                        Ok(exit) => {
                            status = Some(exit);
                        }
                        Err(source) => {
                            wait_error = Some(ClaudeCodeError::Wait(source));
                        }
                    }
                }

                if timed_out.is_some() || wait_error.is_some() {
                    let deadline =
                        cleanup_deadline.get_or_insert(time::Instant::now() + CLEANUP_GRACE);
                    let _ = child.start_kill();
                    let remaining = deadline.saturating_duration_since(time::Instant::now());
                    let _ = time::timeout(remaining, child.wait()).await;
                }
            }
            #[cfg(unix)]
            SetupTokenProcess::Pty { mut child, .. } => {
                let poll_interval = Duration::from_millis(50);
                let user_deadline = timeout.map(|dur| time::Instant::now() + dur);
                loop {
                    match child.try_wait() {
                        Ok(Some(exit)) => {
                            status = Some(portable_exit_status_to_std(exit));
                            break;
                        }
                        Ok(None) => {
                            if let Some(deadline) = user_deadline {
                                if time::Instant::now() >= deadline {
                                    timed_out = timeout;
                                    let deadline = cleanup_deadline
                                        .get_or_insert(time::Instant::now() + CLEANUP_GRACE);
                                    let _ = child.kill();
                                    if let Some(exit) =
                                        Self::reap_pty_until(&mut child, *deadline, poll_interval)
                                            .await
                                    {
                                        status = Some(exit);
                                    }
                                    break;
                                }
                            }
                            time::sleep(poll_interval).await;
                        }
                        Err(source) => {
                            wait_error = Some(ClaudeCodeError::Wait(source));
                            let deadline = cleanup_deadline
                                .get_or_insert(time::Instant::now() + CLEANUP_GRACE);
                            let _ = child.kill();
                            if let Some(exit) =
                                Self::reap_pty_until(&mut child, *deadline, poll_interval).await
                            {
                                status = Some(exit);
                            }
                            break;
                        }
                    }
                }
            }
        };

        let cleanup_deadline = cleanup_deadline.unwrap_or_else(time::Instant::now);
        let failure = timed_out.is_some() || wait_error.is_some();

        if failure {
            if let Some(task) = self.stdout_task.take() {
                let remaining = cleanup_deadline.saturating_duration_since(time::Instant::now());
                let _ = join_or_abort(task, remaining).await;
            }
            if let Some(task) = self.stderr_task.take() {
                let remaining = cleanup_deadline.saturating_duration_since(time::Instant::now());
                let _ = join_or_abort(task, remaining).await;
            }
        } else {
            if let Some(task) = self.stdout_task.take() {
                task.await
                    .map_err(|e| ClaudeCodeError::Join(e.to_string()))??;
            }
            if let Some(task) = self.stderr_task.take() {
                task.await
                    .map_err(|e| ClaudeCodeError::Join(e.to_string()))??;
            }
        }

        let stdout = self.stdout_buf.lock().await.clone();
        let stderr = self.stderr_buf.lock().await.clone();

        if let Some(err) = wait_error {
            return Err(err);
        }
        if let Some(dur) = timed_out {
            return Err(ClaudeCodeError::Timeout { timeout: dur });
        }

        Ok(CommandOutput {
            status: status.expect("setup-token wait should set status when no error is present"),
            stdout,
            stderr,
        })
    }
}

impl Drop for ClaudeSetupTokenSession {
    fn drop(&mut self) {
        // Best-effort cleanup; if the session is dropped before completion, avoid leaving
        // an interactive `claude setup-token` process running.
        let Some(process) = self.process.as_mut() else {
            return;
        };

        match process {
            SetupTokenProcess::Pipes { child, .. } => {
                if child.id().is_some() {
                    let _ = child.start_kill();
                }
            }
            #[cfg(unix)]
            SetupTokenProcess::Pty { child, .. } => {
                let _ = child.kill();
            }
        }
    }
}
