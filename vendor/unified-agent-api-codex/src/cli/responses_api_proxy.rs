use crate::CodexError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf};
use tokio::fs;

/// Request for `codex responses-api-proxy`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponsesApiProxyRequest {
    /// API key to write to stdin on startup.
    pub api_key: String,
    /// Optional port to bind; falls back to an OS-assigned ephemeral port when omitted.
    pub port: Option<u16>,
    /// Optional path passed to `--server-info` for `{port,pid}` JSON output.
    pub server_info_path: Option<PathBuf>,
    /// Enables the HTTP shutdown endpoint (`GET /shutdown`).
    pub http_shutdown: bool,
    /// Optional upstream URL passed to `--upstream-url` (defaults to `https://api.openai.com/v1/responses`).
    pub upstream_url: Option<String>,
}

impl ResponsesApiProxyRequest {
    /// Creates a request with the API key provided via stdin.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            port: None,
            server_info_path: None,
            http_shutdown: false,
            upstream_url: None,
        }
    }

    /// Sets the listening port (`--port`).
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Writes `{port,pid}` JSON to the provided path via `--server-info`.
    pub fn server_info(mut self, path: impl Into<PathBuf>) -> Self {
        self.server_info_path = Some(path.into());
        self
    }

    /// Enables the `--http-shutdown` flag (GET /shutdown).
    pub fn http_shutdown(mut self, enable: bool) -> Self {
        self.http_shutdown = enable;
        self
    }

    /// Overrides the upstream responses endpoint URL.
    pub fn upstream_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        self.upstream_url = (!url.trim().is_empty()).then_some(url);
        self
    }
}

/// Running responses proxy process and metadata.
#[derive(Debug)]
pub struct ResponsesApiProxyHandle {
    /// Spawned `codex responses-api-proxy` child (inherits kill-on-drop).
    pub child: tokio::process::Child,
    /// Optional `--server-info` path that may contain `{port,pid}` JSON.
    pub server_info_path: Option<PathBuf>,
}

impl ResponsesApiProxyHandle {
    /// Reads and parses the `{port,pid}` JSON written by `--server-info`.
    ///
    /// Returns `Ok(None)` when no server info path was configured.
    pub async fn read_server_info(&self) -> Result<Option<ResponsesApiProxyInfo>, CodexError> {
        let Some(path) = &self.server_info_path else {
            return Ok(None);
        };

        const MAX_ATTEMPTS: usize = 10;
        const BACKOFF_MS: u64 = 25;

        for attempt in 0..MAX_ATTEMPTS {
            match fs::read_to_string(path).await {
                Ok(contents) => match serde_json::from_str::<ResponsesApiProxyInfo>(&contents) {
                    Ok(info) => return Ok(Some(info)),
                    Err(source) => {
                        if attempt + 1 == MAX_ATTEMPTS {
                            return Err(CodexError::ResponsesApiProxyInfoParse {
                                path: path.clone(),
                                source,
                            });
                        }
                    }
                },
                Err(source) => {
                    let is_missing = source.kind() == std::io::ErrorKind::NotFound;
                    if !is_missing || attempt + 1 == MAX_ATTEMPTS {
                        return Err(CodexError::ResponsesApiProxyInfoRead {
                            path: path.clone(),
                            source,
                        });
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(BACKOFF_MS)).await;
        }

        unreachable!("read_server_info loop must return by MAX_ATTEMPTS")
    }
}

/// Parsed `{port,pid}` emitted by `codex responses-api-proxy --server-info`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResponsesApiProxyInfo {
    pub port: u16,
    pub pid: u32,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}
