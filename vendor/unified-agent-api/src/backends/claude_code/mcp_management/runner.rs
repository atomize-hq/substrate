use std::{
    env,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    time::Duration,
};

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    task::JoinHandle,
};

use crate::{
    bounds::{enforce_mcp_output_bound, MCP_STDERR_BOUND_BYTES, MCP_STDOUT_BOUND_BYTES},
    mcp::AgentWrapperMcpCommandOutput,
    AgentWrapperError,
};

use super::{
    backend_error, PINNED_CAPTURE_FAILURE, PINNED_MCP_RUNTIME_CONFLICT,
    PINNED_PREPARE_CLAUDE_HOME_FAILURE, PINNED_SPAWN_FAILURE, PINNED_TIMEOUT_FAILURE,
    PINNED_WAIT_FAILURE,
};

// Keep the captured bytes separate from final bounded strings so drift classification
// can evolve without changing the runner entrypoint.
#[derive(Debug)]
pub(super) struct CapturedClaudeMcpCommandOutput {
    pub(super) status: ExitStatus,
    pub(super) stdout_bytes: Vec<u8>,
    pub(super) stdout_saw_more: bool,
    pub(super) stderr_bytes: Vec<u8>,
    pub(super) stderr_saw_more: bool,
}

pub(super) async fn capture_claude_mcp_output(
    resolved: &super::resolve::ResolvedClaudeMcpCommand,
    argv: &[OsString],
) -> Result<CapturedClaudeMcpCommandOutput, AgentWrapperError> {
    if resolved.timeout == Some(Duration::ZERO) {
        return Err(backend_error(PINNED_TIMEOUT_FAILURE));
    }

    if let Some(layout) = resolved.materialize_claude_home.as_ref() {
        layout
            .materialize(true)
            .map_err(|_| backend_error(PINNED_PREPARE_CLAUDE_HOME_FAILURE))?;
    }

    let mut command = Command::new(&resolved.binary_path);
    command
        .args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .env_clear()
        .envs(&resolved.env)
        .current_dir(spawn_current_dir(resolved.working_dir.as_deref()));

    let mut child = command
        .spawn()
        .map_err(|_| backend_error(PINNED_SPAWN_FAILURE))?;

    let Some(stdout) = child.stdout.take() else {
        cleanup_child(&mut child).await;
        return Err(backend_error(PINNED_CAPTURE_FAILURE));
    };
    let Some(stderr) = child.stderr.take() else {
        cleanup_child(&mut child).await;
        return Err(backend_error(PINNED_CAPTURE_FAILURE));
    };

    let stdout_task = tokio::spawn(capture_bounded(stdout, MCP_STDOUT_BOUND_BYTES));
    let stderr_task = tokio::spawn(capture_bounded(stderr, MCP_STDERR_BOUND_BYTES));

    let status = match wait_for_exit(&mut child, resolved.timeout).await {
        Ok(status) => status,
        Err(err) => {
            stdout_task.abort();
            stderr_task.abort();
            return Err(err);
        }
    };

    let (stdout_bytes, stdout_saw_more) = join_capture_task(stdout_task).await?;
    let (stderr_bytes, stderr_saw_more) = join_capture_task(stderr_task).await?;

    Ok(CapturedClaudeMcpCommandOutput {
        status,
        stdout_bytes,
        stdout_saw_more,
        stderr_bytes,
        stderr_saw_more,
    })
}

fn spawn_current_dir(working_dir: Option<&Path>) -> PathBuf {
    working_dir
        .map(Path::to_path_buf)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(env::temp_dir)
}

pub(super) fn finalize_claude_mcp_output(
    argv: &[OsString],
    captured: CapturedClaudeMcpCommandOutput,
) -> Result<AgentWrapperMcpCommandOutput, AgentWrapperError> {
    if !captured.status.success()
        && is_manifest_runtime_conflict(argv, &captured.stdout_bytes, &captured.stderr_bytes)
    {
        return Err(backend_error(PINNED_MCP_RUNTIME_CONFLICT));
    }

    let (stdout, stdout_truncated) = enforce_mcp_output_bound(
        &captured.stdout_bytes,
        captured.stdout_saw_more,
        MCP_STDOUT_BOUND_BYTES,
    );
    let (stderr, stderr_truncated) = enforce_mcp_output_bound(
        &captured.stderr_bytes,
        captured.stderr_saw_more,
        MCP_STDERR_BOUND_BYTES,
    );

    Ok(AgentWrapperMcpCommandOutput {
        status: captured.status,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

fn is_manifest_runtime_conflict(argv: &[OsString], stdout: &[u8], stderr: &[u8]) -> bool {
    let stderr = String::from_utf8_lossy(stderr);
    let stdout = String::from_utf8_lossy(stdout);
    classify_manifest_runtime_conflict_text(argv, &format!("{stderr}\n{stdout}"))
}

pub(super) fn classify_manifest_runtime_conflict_text(argv: &[OsString], text: &str) -> bool {
    let text = text.to_ascii_lowercase();

    let unknown_signal = [
        "unknown",
        "unrecognized",
        "unexpected",
        "invalid",
        "no such",
        "not recognized",
    ]
    .iter()
    .any(|signal| text.contains(signal));
    if !unknown_signal {
        return false;
    }

    let syntax_context = ["command", "subcommand", "argument", "option", "flag"]
        .iter()
        .any(|signal| text.contains(signal));
    if !syntax_context {
        return false;
    }

    if is_add_shape_conflict(argv, &text) {
        return true;
    }

    manifest_conflict_tokens(argv)
        .into_iter()
        .any(|token| text.contains(token))
}

fn is_add_shape_conflict(argv: &[OsString], text: &str) -> bool {
    matches!(argv.get(1).and_then(|arg| arg.to_str()), Some("add"))
        && ["--transport", "--env"]
            .iter()
            .any(|token| text.contains(token))
}

fn manifest_conflict_tokens(argv: &[OsString]) -> Vec<&'static str> {
    let mut tokens = vec!["mcp"];
    match argv.get(1).and_then(|arg| arg.to_str()) {
        Some("list") => tokens.push("list"),
        Some("get") => tokens.push("get"),
        Some("add") => tokens.push("add"),
        Some("remove") => tokens.push("remove"),
        _ => {}
    }
    tokens
}

async fn wait_for_exit(
    child: &mut Child,
    timeout: Option<Duration>,
) -> Result<ExitStatus, AgentWrapperError> {
    match timeout {
        Some(timeout) => {
            debug_assert_ne!(timeout, Duration::ZERO);
            match tokio::time::timeout(timeout, child.wait()).await {
                Ok(Ok(status)) => Ok(status),
                Ok(Err(_)) => Err(backend_error(PINNED_WAIT_FAILURE)),
                Err(_) => {
                    cleanup_child(child).await;
                    Err(backend_error(PINNED_TIMEOUT_FAILURE))
                }
            }
        }
        None => child
            .wait()
            .await
            .map_err(|_| backend_error(PINNED_WAIT_FAILURE)),
    }
}

async fn cleanup_child(child: &mut Child) {
    let _ = child.kill().await;
    let _ = child.wait().await;
}

async fn join_capture_task(
    task: JoinHandle<io::Result<(Vec<u8>, bool)>>,
) -> Result<(Vec<u8>, bool), AgentWrapperError> {
    task.await
        .map_err(|_| backend_error(PINNED_CAPTURE_FAILURE))?
        .map_err(|_| backend_error(PINNED_CAPTURE_FAILURE))
}

pub(super) async fn capture_bounded<R>(
    mut reader: R,
    bound_bytes: usize,
) -> io::Result<(Vec<u8>, bool)>
where
    R: AsyncRead + Unpin,
{
    let retain_bound = bound_bytes.saturating_add(1);
    let mut retained = Vec::with_capacity(retain_bound.min(4096));
    let mut saw_more = false;
    let mut chunk = [0u8; 4096];

    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }

        if retained.len() < retain_bound {
            let remaining = retain_bound - retained.len();
            let to_copy = remaining.min(read);
            retained.extend_from_slice(&chunk[..to_copy]);
            if to_copy < read {
                saw_more = true;
            }
        } else {
            saw_more = true;
        }
    }

    if retained.len() > bound_bytes {
        retained.truncate(bound_bytes);
        saw_more = true;
    }

    Ok((retained, saw_more))
}
