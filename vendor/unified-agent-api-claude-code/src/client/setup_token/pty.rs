use std::{collections::BTreeMap, path::Path, sync::Arc};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tokio::sync::{oneshot, Mutex};

use crate::{process::ConsoleTarget, ClaudeCodeError};

use super::{capture::spawn_pty_capture_task, process::SetupTokenProcess, url::UrlCapture};

pub(super) fn portable_exit_status_to_std(
    status: portable_pty::ExitStatus,
) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;

    let code = std::cmp::min(status.exit_code(), 255) as i32;
    // POSIX encodes the exit code in the high byte.
    std::process::ExitStatus::from_raw(code << 8)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_setup_token_pty(
    binary: &Path,
    argv: &[String],
    working_dir: Option<&Path>,
    env: &BTreeMap<String, String>,
    mirror_stdout: bool,
    mirror_stderr: bool,
    out: Arc<Mutex<Vec<u8>>>,
    url_state: Arc<Mutex<UrlCapture>>,
    url_tx: Arc<Mutex<Option<oneshot::Sender<String>>>>,
) -> Result<
    (
        SetupTokenProcess,
        tokio::task::JoinHandle<Result<(), ClaudeCodeError>>,
    ),
    ClaudeCodeError,
> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| ClaudeCodeError::InvalidRequest(format!("failed to open PTY: {e}")))?;

    // Note: we don't attempt to alter termios/echo settings here. The underlying CLI
    // may toggle raw mode itself; this wrapper only needs a PTY to avoid Ink errors.

    let mut cmd = CommandBuilder::new(binary.to_string_lossy().to_string());
    for arg in argv {
        cmd.arg(arg);
    }
    if let Some(dir) = working_dir {
        cmd.cwd(dir);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }

    let child = pair.slave.spawn_command(cmd).map_err(|e| {
        ClaudeCodeError::InvalidRequest(format!("failed to spawn PTY command: {e}"))
    })?;
    drop(pair.slave);

    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| ClaudeCodeError::InvalidRequest(format!("failed to clone PTY reader: {e}")))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| ClaudeCodeError::InvalidRequest(format!("failed to take PTY writer: {e}")))?;

    let mirror_console = mirror_stdout || mirror_stderr;
    let mirror_target = if mirror_stdout {
        ConsoleTarget::Stdout
    } else {
        ConsoleTarget::Stderr
    };

    let stdout_task = spawn_pty_capture_task(
        reader,
        mirror_target,
        mirror_console,
        out,
        url_state,
        url_tx,
    );

    Ok((
        SetupTokenProcess::Pty {
            child,
            writer: Some(writer),
        },
        stdout_task,
    ))
}
