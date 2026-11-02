use tokio::{process::Command, time};

use crate::{
    process::{spawn_with_retry, tee_stream, ConsoleTarget},
    CodexClient, CodexError, SandboxCommandRequest, SandboxPlatform, SandboxRun, StdioToUdsRequest,
};

impl CodexClient {
    /// Spawns `codex stdio-to-uds <SOCKET_PATH>` with piped stdio for manual relays.
    ///
    /// Returns the child process so callers can write to stdin/read from stdout (e.g., to bridge a
    /// JSON-RPC transport over a Unix domain socket). Fails fast on empty socket paths and inherits
    /// the builder working directory when none is provided on the request.
    pub fn stdio_to_uds(
        &self,
        request: StdioToUdsRequest,
    ) -> Result<tokio::process::Child, CodexError> {
        let StdioToUdsRequest {
            socket_path,
            working_dir,
        } = request;

        if socket_path.as_os_str().is_empty() {
            return Err(CodexError::EmptySocketPath);
        }

        let mut command = Command::new(self.command_env.binary_path());
        command
            .arg("stdio-to-uds")
            .arg(&socket_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .current_dir(self.sandbox_working_dir(working_dir)?);

        self.command_env.apply(&mut command)?;

        spawn_with_retry(&mut command, self.command_env.binary_path())
    }

    /// Runs `codex sandbox <platform> [--full-auto|--log-denials] [--config/--enable/--disable] -- <COMMAND...>`.
    ///
    /// Captures stdout/stderr and mirrors them according to the builder (`mirror_stdout` / `quiet`). Unlike
    /// `apply`/`diff`, non-zero exit codes are returned in [`SandboxRun::status`] without being wrapped in
    /// [`CodexError::NonZeroExit`]. macOS denial logging is enabled via [`SandboxCommandRequest::log_denials`]
    /// and ignored on other platforms. Linux uses the bundled `codex-linux-sandbox` helper; Windows sandboxing
    /// is experimental and relies on the upstream helper. The wrapper does not gate availability—unsupported
    /// installs will surface as non-zero statuses.
    pub async fn run_sandbox(
        &self,
        request: SandboxCommandRequest,
    ) -> Result<SandboxRun, CodexError> {
        if request.command.is_empty() {
            return Err(CodexError::EmptySandboxCommand);
        }

        let SandboxCommandRequest {
            platform,
            command,
            full_auto,
            log_denials,
            allow_unix_socket,
            include_managed_config,
            permissions_profile,
            config_overrides,
            feature_toggles,
            working_dir,
        } = request;

        let working_dir = self.sandbox_working_dir(working_dir)?;

        let mut process = Command::new(self.command_env.binary_path());
        process
            .arg("sandbox")
            .arg(platform.subcommand())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .current_dir(&working_dir);

        if full_auto {
            process.arg("--full-auto");
        }

        if log_denials && matches!(platform, SandboxPlatform::Macos) {
            process.arg("--log-denials");
        }

        if allow_unix_socket && matches!(platform, SandboxPlatform::Macos) {
            process.arg("--allow-unix-socket");
        }

        if include_managed_config {
            process.arg("--include-managed-config");
        }

        if let Some(permissions_profile) = permissions_profile {
            process
                .arg("--permissions-profile")
                .arg(permissions_profile);
        }

        for override_ in config_overrides {
            process.arg("--config");
            process.arg(format!("{}={}", override_.key, override_.value));
        }

        for feature in feature_toggles.enable {
            process.arg("--enable");
            process.arg(feature);
        }

        for feature in feature_toggles.disable {
            process.arg("--disable");
            process.arg(feature);
        }

        process.arg("--");
        process.args(&command);

        self.command_env.apply(&mut process)?;

        let mut child = spawn_with_retry(&mut process, self.command_env.binary_path())?;

        let stdout = child.stdout.take().ok_or(CodexError::StdoutUnavailable)?;
        let stderr = child.stderr.take().ok_or(CodexError::StderrUnavailable)?;

        let stdout_task = tokio::spawn(tee_stream(
            stdout,
            ConsoleTarget::Stdout,
            self.mirror_stdout,
        ));
        let stderr_task = tokio::spawn(tee_stream(stderr, ConsoleTarget::Stderr, !self.quiet));

        let wait_task = async move {
            let status = child
                .wait()
                .await
                .map_err(|source| CodexError::Wait { source })?;
            let stdout_bytes = stdout_task
                .await
                .map_err(CodexError::Join)?
                .map_err(CodexError::CaptureIo)?;
            let stderr_bytes = stderr_task
                .await
                .map_err(CodexError::Join)?
                .map_err(CodexError::CaptureIo)?;
            Ok::<_, CodexError>((status, stdout_bytes, stderr_bytes))
        };

        let (status, stdout_bytes, stderr_bytes) = if self.timeout.is_zero() {
            wait_task.await?
        } else {
            match time::timeout(self.timeout, wait_task).await {
                Ok(result) => result?,
                Err(_) => {
                    return Err(CodexError::Timeout {
                        timeout: self.timeout,
                    });
                }
            }
        };

        Ok(SandboxRun {
            status,
            stdout: String::from_utf8(stdout_bytes)?,
            stderr: String::from_utf8(stderr_bytes)?,
        })
    }
}
