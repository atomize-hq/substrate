use std::{
    io::{self, Write},
    path::Path,
    process::ExitStatus,
    time::Duration,
};

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    task,
};

use crate::CodexError;

#[derive(Clone, Copy)]
pub(crate) enum ConsoleTarget {
    Stdout,
    Stderr,
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
) -> Result<tokio::process::Child, CodexError> {
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
                return Err(CodexError::Spawn {
                    binary: binary.to_path_buf(),
                    source,
                });
            }
        }
    }

    unreachable!("spawn_with_retry should return before exhausting retries")
}

pub(crate) fn command_output_text(output: &CommandOutput) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let stdout = stdout.trim_end();
    let stderr = stderr.trim_end();
    if stdout.is_empty() {
        stderr.to_string()
    } else if stderr.is_empty() {
        stdout.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    }
}

pub(crate) fn preferred_output_channel(output: &CommandOutput) -> String {
    let stderr = String::from_utf8(output.stderr.clone()).unwrap_or_default();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap_or_default();
    if stderr.trim().is_empty() {
        stdout
    } else {
        stderr
    }
}

pub(crate) struct CommandOutput {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}
