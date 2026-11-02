use tokio::{io::AsyncWriteExt, process::Command};

use crate::{
    process::spawn_with_retry, CodexClient, CodexError, ResponsesApiProxyHandle,
    ResponsesApiProxyRequest,
};

impl CodexClient {
    /// Starts the `codex responses-api-proxy` helper with a supplied API key.
    ///
    /// Forwards optional `--port`, `--server-info`, `--http-shutdown`, and `--upstream-url` flags.
    /// The API key is written to stdin immediately after spawn, stdout/stderr remain piped for callers
    /// to drain, and the returned handle owns the child process plus any `--server-info` path used.
    pub async fn start_responses_api_proxy(
        &self,
        request: ResponsesApiProxyRequest,
    ) -> Result<ResponsesApiProxyHandle, CodexError> {
        let ResponsesApiProxyRequest {
            api_key,
            port,
            server_info_path,
            http_shutdown,
            upstream_url,
        } = request;

        let api_key = api_key.trim().to_string();
        if api_key.is_empty() {
            return Err(CodexError::EmptyApiKey);
        }

        let working_dir = self.sandbox_working_dir(None)?;

        let mut command = Command::new(self.command_env.binary_path());
        command
            .arg("responses-api-proxy")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .current_dir(&working_dir);

        if let Some(port) = port {
            command.arg("--port").arg(port.to_string());
        }

        if let Some(path) = server_info_path.as_ref() {
            command.arg("--server-info").arg(path);
        }

        if http_shutdown {
            command.arg("--http-shutdown");
        }

        if let Some(url) = upstream_url.as_ref() {
            if !url.trim().is_empty() {
                command.arg("--upstream-url").arg(url);
            }
        }

        self.command_env.apply(&mut command)?;

        let mut child = spawn_with_retry(&mut command, self.command_env.binary_path())?;

        let mut stdin = child.stdin.take().ok_or(CodexError::StdinUnavailable)?;
        stdin
            .write_all(api_key.as_bytes())
            .await
            .map_err(CodexError::StdinWrite)?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(CodexError::StdinWrite)?;
        stdin.shutdown().await.map_err(CodexError::StdinWrite)?;

        Ok(ResponsesApiProxyHandle {
            child,
            server_info_path,
        })
    }
}
