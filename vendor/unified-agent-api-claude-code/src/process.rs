use std::{
    collections::BTreeMap,
    io::{self, Write},
    path::Path,
    process::ExitStatus,
    time::Duration,
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
    task, time,
};

use crate::ClaudeCodeError;

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

#[derive(Clone, Copy)]
pub(crate) enum ConsoleTarget {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub(crate) async fn tee_stream<R>(
    mut reader: R,
    target: ConsoleTarget,
    mirror_console: bool,
) -> Result<Vec<u8>, io::Error>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = reader.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        if mirror_console {
            task::block_in_place(|| match target {
                ConsoleTarget::Stdout => {
                    let mut out = io::stdout();
                    out.write_all(&chunk[..n])?;
                    out.flush()
                }
                ConsoleTarget::Stderr => {
                    let mut out = io::stderr();
                    out.write_all(&chunk[..n])?;
                    out.flush()
                }
            })?;
        }
        buffer.extend_from_slice(&chunk[..n]);
    }
    Ok(buffer)
}

pub(crate) fn spawn_with_retry(
    command: &mut Command,
    binary: &Path,
) -> Result<tokio::process::Child, ClaudeCodeError> {
    let mut backoff = Duration::from_millis(2);
    for attempt in 0..5 {
        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(source) => {
                let is_busy = matches!(source.kind(), std::io::ErrorKind::ExecutableFileBusy)
                    || source.raw_os_error() == Some(26);
                if is_busy && attempt < 4 {
                    std::thread::sleep(backoff);
                    backoff = std::cmp::min(backoff * 2, Duration::from_millis(50));
                    continue;
                }
                return Err(ClaudeCodeError::Spawn {
                    binary: binary.to_path_buf(),
                    source,
                });
            }
        }
    }

    unreachable!("spawn_with_retry should return before exhausting retries")
}

pub(crate) async fn run_command(
    mut command: Command,
    binary: &Path,
    stdin_bytes: Option<&[u8]>,
    timeout: Option<Duration>,
    mirror_stdout: bool,
    mirror_stderr: bool,
) -> Result<CommandOutput, ClaudeCodeError> {
    command.stdin(if stdin_bytes.is_some() {
        std::process::Stdio::piped()
    } else {
        std::process::Stdio::null()
    });
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    command.kill_on_drop(true);

    let mut child = spawn_with_retry(&mut command, binary)?;

    if let Some(bytes) = stdin_bytes {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(bytes)
                .await
                .map_err(ClaudeCodeError::StdinWrite)?;
        }
    }

    let stdout = child.stdout.take().ok_or(ClaudeCodeError::MissingStdout)?;
    let stderr = child.stderr.take().ok_or(ClaudeCodeError::MissingStderr)?;

    let stdout_task = tokio::spawn(tee_stream(stdout, ConsoleTarget::Stdout, mirror_stdout));
    let stderr_task = tokio::spawn(tee_stream(stderr, ConsoleTarget::Stderr, mirror_stderr));

    let status = if let Some(dur) = timeout {
        match time::timeout(dur, child.wait()).await {
            Ok(Ok(status)) => status,
            Ok(Err(source)) => {
                let _ = child.start_kill();
                let deadline = time::Instant::now() + CLEANUP_GRACE;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = time::timeout(remaining, child.wait()).await;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = join_or_abort(stdout_task, remaining).await;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = join_or_abort(stderr_task, remaining).await;
                return Err(ClaudeCodeError::Wait(source));
            }
            Err(_) => {
                let _ = child.start_kill();
                let deadline = time::Instant::now() + CLEANUP_GRACE;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = time::timeout(remaining, child.wait()).await;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = join_or_abort(stdout_task, remaining).await;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = join_or_abort(stderr_task, remaining).await;
                return Err(ClaudeCodeError::Timeout { timeout: dur });
            }
        }
    } else {
        match child.wait().await {
            Ok(status) => status,
            Err(source) => {
                let _ = child.start_kill();
                let deadline = time::Instant::now() + CLEANUP_GRACE;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = time::timeout(remaining, child.wait()).await;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = join_or_abort(stdout_task, remaining).await;
                let remaining = deadline.saturating_duration_since(time::Instant::now());
                let _ = join_or_abort(stderr_task, remaining).await;
                return Err(ClaudeCodeError::Wait(source));
            }
        }
    };

    let stdout = stdout_task
        .await
        .map_err(|e| ClaudeCodeError::Join(e.to_string()))?
        .map_err(ClaudeCodeError::StdoutRead)?;
    let stderr = stderr_task
        .await
        .map_err(|e| ClaudeCodeError::Join(e.to_string()))?
        .map_err(ClaudeCodeError::StderrRead)?;

    Ok(CommandOutput {
        status,
        stdout,
        stderr,
    })
}

pub(crate) fn apply_env(command: &mut Command, env: &BTreeMap<String, String>) {
    for (k, v) in env {
        command.env(k, v);
    }
}

/// Only the selected help-preflight path uses this additional retained owner.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn run_command_with_custody(
    mut command: Command,
    binary: &Path,
    input: Option<Vec<u8>>,
    timeout: Option<Duration>,
    mirror_stdout: bool,
    mirror_stderr: bool,
    custody: &crate::ClaudeProcessCustody,
) -> Result<CommandOutput, ClaudeCodeError> {
    command.stdin(if input.is_some() {
        std::process::Stdio::piped()
    } else {
        std::process::Stdio::null()
    });
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (child, owner) = custody.spawn_admitted(&mut command, binary).await?;
    let (ready_tx, _ready_rx) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(crate::process_custody::drive_child(
        child,
        owner,
        input,
        false,
        timeout,
        crate::process_custody::StderrMode::Capture {
            mirror: mirror_stderr,
        },
        ready_tx,
        move |stdout| async move { tee_stream(stdout, ConsoleTarget::Stdout, mirror_stdout).await },
    ));
    let mut cancelled_on_drop = custody.cancel_on_drop();
    let output = task
        .await
        .map_err(|error| ClaudeCodeError::Join(error.to_string()))?;
    cancelled_on_drop.disarm();
    let status = output.status.map_err(ClaudeCodeError::Wait)?;
    if output.settlement_failed {
        return Err(ClaudeCodeError::Join(
            "help process settlement failed".to_string(),
        ));
    }
    if output.timed_out {
        return Err(ClaudeCodeError::Timeout {
            timeout: timeout.unwrap_or_default(),
        });
    }
    output.stdin.map_err(ClaudeCodeError::StdinWrite)?;
    let stdout = output
        .stdout
        .map_err(ClaudeCodeError::StdoutRead)?
        .ok_or_else(|| {
            ClaudeCodeError::StdoutRead(io::Error::new(
                io::ErrorKind::Interrupted,
                "help stdout disposed",
            ))
        })?;
    let stderr = output
        .stderr
        .map_err(ClaudeCodeError::StderrRead)?
        .unwrap_or_default();
    Ok(CommandOutput {
        status,
        stdout,
        stderr,
    })
}
