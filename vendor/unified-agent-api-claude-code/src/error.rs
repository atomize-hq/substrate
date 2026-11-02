use std::{path::PathBuf, process::ExitStatus, time::Duration};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClaudeCodeError {
    #[error("claude binary not found")]
    MissingBinary,
    #[error("failed to spawn claude process (binary={binary:?}): {source}")]
    Spawn {
        binary: PathBuf,
        source: std::io::Error,
    },
    #[error("claude process timed out after {timeout:?}")]
    Timeout { timeout: Duration },
    #[error("failed waiting for claude process: {0}")]
    Wait(std::io::Error),
    #[error("failed reading stdout: {0}")]
    StdoutRead(std::io::Error),
    #[error("failed reading stderr: {0}")]
    StderrRead(std::io::Error),
    #[error("failed writing stdin: {0}")]
    StdinWrite(std::io::Error),
    #[error("internal error: missing stdout pipe")]
    MissingStdout,
    #[error("internal error: missing stderr pipe")]
    MissingStderr,
    #[error("internal error: join failure: {0}")]
    Join(String),
    #[error("request is invalid: {0}")]
    InvalidRequest(String),
    #[error("claude returned non-zero exit status: {status}")]
    NonZeroExit { status: ExitStatus },
    #[error("failed to parse JSON output: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("failed to prepare CLAUDE_HOME directory `{path}`: {source}")]
    PrepareClaudeHome {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed seeding Claude home (io) `{path}`: {source}")]
    ClaudeHomeSeedIo {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed seeding Claude home: copy `{from}` -> `{to}`: {error}")]
    ClaudeHomeSeedCopy {
        from: PathBuf,
        to: PathBuf,
        error: std::io::Error,
    },
    #[error("claude home prepare failed: {0}")]
    ClaudeHomePrepareFailed(String),
    #[error("claude home seed failed: {0}")]
    ClaudeHomeSeedFailed(String),
}

#[derive(Debug, Error, Clone)]
#[error("stream-json line {line_number}: {message}")]
pub struct StreamJsonLineError {
    pub line_number: usize,
    pub message: String,
}
