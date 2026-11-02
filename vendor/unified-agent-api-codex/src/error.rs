use std::{path::PathBuf, process::ExitStatus, time::Duration};

use thiserror::Error;

/// Errors that may occur while invoking the Codex CLI.
#[derive(Debug, Error)]
pub enum CodexError {
    #[error("codex binary `{binary}` could not be spawned: {source}")]
    Spawn {
        binary: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to wait for codex process: {source}")]
    Wait {
        #[source]
        source: std::io::Error,
    },
    #[error("codex exceeded timeout of {timeout:?}")]
    Timeout { timeout: Duration },
    #[error("codex exited with {status:?}: {stderr}")]
    NonZeroExit { status: ExitStatus, stderr: String },
    #[error("codex output was not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("failed to parse {context} JSON output: {source}")]
    JsonParse {
        context: &'static str,
        stdout: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to parse execpolicy JSON output: {source}")]
    ExecPolicyParse {
        stdout: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to parse features list output: {reason}")]
    FeatureListParse { reason: String, stdout: String },
    #[error("failed to read responses-api-proxy server info from `{path}`: {source}")]
    ResponsesApiProxyInfoRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse responses-api-proxy server info from `{path}`: {source}")]
    ResponsesApiProxyInfoParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("prompt must not be empty")]
    EmptyPrompt,
    #[error("sandbox command must not be empty")]
    EmptySandboxCommand,
    #[error("execpolicy command must not be empty")]
    EmptyExecPolicyCommand,
    #[error("API key must not be empty")]
    EmptyApiKey,
    #[error("task id must not be empty")]
    EmptyTaskId,
    #[error("environment id must not be empty")]
    EmptyEnvId,
    #[error("MCP server name must not be empty")]
    EmptyMcpServerName,
    #[error("MCP server command must not be empty")]
    EmptyMcpCommand,
    #[error("MCP server URL must not be empty")]
    EmptyMcpUrl,
    #[error("socket path must not be empty")]
    EmptySocketPath,
    #[error("failed to create temporary working directory: {0}")]
    TempDir(#[source] std::io::Error),
    #[error("failed to resolve working directory: {source}")]
    WorkingDirectory {
        #[source]
        source: std::io::Error,
    },
    #[error("failed to prepare app-server output directory `{path}`: {source}")]
    PrepareOutputDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to prepare CODEX_HOME at `{path}`: {source}")]
    PrepareCodexHome {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("codex stdout unavailable")]
    StdoutUnavailable,
    #[error("codex stderr unavailable")]
    StderrUnavailable,
    #[error("codex stdin unavailable")]
    StdinUnavailable,
    #[error("failed to capture codex output: {0}")]
    CaptureIo(#[from] std::io::Error),
    #[error("failed to write prompt to codex stdin: {0}")]
    StdinWrite(#[source] std::io::Error),
    #[error("failed to join codex output task: {0}")]
    Join(#[from] tokio::task::JoinError),
}
