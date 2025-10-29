//! Helpers for executing commands with incremental streaming callbacks.

use crate::stream::{emit_chunk, StreamKind};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use tracing::warn;

/// Execute `cmd` via `sh -lc` in the provided directory/environment, pushing
/// stdout/stderr chunks through the global stream sink while accumulating the
/// full output for the caller.
pub fn execute_shell_command(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
) -> Result<Output> {
    let mut command = Command::new("sh");
    if login_shell {
        command.arg("-lc");
    } else {
        command.arg("-c");
    }
    command.arg(cmd);
    command.current_dir(cwd);
    command.envs(env);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to spawn command: {cmd}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stdout pipe"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stderr pipe"))?;

    let stdout_handle = spawn_reader(stdout, StreamKind::Stdout);
    let stderr_handle = spawn_reader(stderr, StreamKind::Stderr);

    let status = child
        .wait()
        .context("Failed to wait for child process completion")?;

    let stdout_buf = join_reader(stdout_handle, "stdout");
    let stderr_buf = join_reader(stderr_handle, "stderr");

    Ok(Output {
        status,
        stdout: stdout_buf,
        stderr: stderr_buf,
    })
}

fn spawn_reader<R>(mut reader: R, kind: StreamKind) -> thread::JoinHandle<Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    let slice = &chunk[..n];
                    buf.extend_from_slice(slice);
                    emit_chunk(kind, slice);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    return Err(anyhow!("pipe read failed: {e}"));
                }
            }
        }
        Ok(buf)
    })
}

fn join_reader(handle: thread::JoinHandle<Result<Vec<u8>>>, label: &str) -> Vec<u8> {
    match handle.join() {
        Ok(Ok(buf)) => buf,
        Ok(Err(e)) => {
            warn!(stream = label, error = %e, "stream reader error");
            Vec::new()
        }
        Err(e) => {
            warn!(stream = label, error = ?e, "stream reader panicked");
            Vec::new()
        }
    }
}
