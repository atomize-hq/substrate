use crate::{
    ClaudeCodeError, ClaudeProcessCustody, ClaudeStreamJsonEvent, ClaudeStreamJsonParseError,
    ClaudeStreamJsonParser,
};
use std::{io, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::ChildStdout,
    sync::mpsc,
};

pub(super) async fn forward_events(
    stdout: ChildStdout,
    tx: mpsc::Sender<Result<ClaudeStreamJsonEvent, ClaudeStreamJsonParseError>>,
    mirror: bool,
    custody: ClaudeProcessCustody,
) -> io::Result<()> {
    let mut lines = BufReader::new(stdout).lines();
    let mut parser = ClaudeStreamJsonParser::new();
    let mut console = tokio::io::stdout();
    loop {
        // Receiver drop is cancellation even when the child is quiet and no send is attempted.
        let next = tokio::select! {
            biased;
            _ = tx.closed() => { custody.request_termination(); return Ok(()); }
            next = lines.next_line() => next?,
        };
        let Some(line) = next else {
            return Ok(());
        };
        if line.trim().is_empty() {
            continue;
        }
        if mirror {
            console.write_all(line.as_bytes()).await?;
            console.write_all(b"\n").await?;
            console.flush().await?;
        }
        let event = match parser.parse_line(&line) {
            Ok(Some(event)) => Ok(event),
            Ok(None) => continue,
            Err(error) => Err(error),
        };
        let sent = tokio::select! {
            biased;
            _ = tx.closed() => false,
            sent = tx.send(event) => sent.is_ok(),
        };
        if !sent {
            custody.request_termination();
            return Ok(());
        }
    }
}

pub(super) fn completion(
    result: crate::process_custody::ChildOutcome<()>,
    timeout: Option<Duration>,
) -> Result<std::process::ExitStatus, ClaudeCodeError> {
    let status = result.status.map_err(ClaudeCodeError::Wait)?;
    result.stdout.map_err(ClaudeCodeError::StdoutRead)?;
    result.stderr.map_err(ClaudeCodeError::StderrRead)?;
    if result.settlement_failed {
        return Err(ClaudeCodeError::Join(
            "process settlement failed".to_string(),
        ));
    }
    if result.timed_out {
        return Err(ClaudeCodeError::Timeout {
            timeout: timeout.unwrap_or_default(),
        });
    }
    // Cancellation can intentionally dispose stdin. Its acknowledgment is recorded separately.
    if let Err(error) = result.stdin {
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(ClaudeCodeError::StdinWrite(error));
        }
    }
    Ok(status)
}
