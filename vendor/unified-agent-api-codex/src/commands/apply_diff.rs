use std::{env, ffi::OsString};

use tokio::{process::Command, time};

use crate::{
    builder::{apply_cli_overrides, resolve_cli_overrides},
    process::{spawn_with_retry, tee_stream, ConsoleTarget},
    ApplyDiffArtifacts, CliOverridesPatch, CodexClient, CodexError,
};

impl CodexClient {
    /// Applies a Codex diff by invoking `codex apply <TASK_ID>`.
    ///
    /// Stdout mirrors to the console when `mirror_stdout` is enabled; stderr mirrors unless `quiet`
    /// is set. Output and exit status are always captured and returned, and `RUST_LOG=error` is
    /// injected for the child process when the environment variable is unset.
    ///
    /// Convenience behavior: if `CODEX_TASK_ID` is set, it is appended as `<TASK_ID>`. When the
    /// environment variable is missing, the subprocess is still spawned and will typically exit
    /// non-zero with a "missing TASK_ID" error from the CLI.
    pub async fn apply(&self) -> Result<ApplyDiffArtifacts, CodexError> {
        let task_id = env::var_os("CODEX_TASK_ID")
            .and_then(|v| crate::normalize_non_empty(&v.to_string_lossy()).map(OsString::from));
        self.apply_task_inner(task_id).await
    }

    /// Applies a Codex diff by task id via `codex apply <TASK_ID>`.
    pub async fn apply_task(
        &self,
        task_id: impl AsRef<str>,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let task_id = task_id.as_ref().trim();
        if task_id.is_empty() {
            return Err(CodexError::EmptyTaskId);
        }
        self.apply_task_inner(Some(OsString::from(task_id))).await
    }

    /// Shows a Codex Cloud task diff by invoking `codex cloud diff <TASK_ID>`.
    ///
    /// Mirrors stdout/stderr using the same `mirror_stdout`/`quiet` defaults as `apply`, but always
    /// returns the captured output alongside the child exit status. Applies the same `RUST_LOG`
    /// defaulting behavior when the variable is unset.
    ///
    /// Convenience behavior: if `CODEX_TASK_ID` is set, it is appended as `<TASK_ID>`. When the
    /// environment variable is missing, the subprocess is still spawned and will typically exit
    /// non-zero with a "missing TASK_ID" error from the CLI.
    pub async fn diff(&self) -> Result<ApplyDiffArtifacts, CodexError> {
        let task_id = env::var_os("CODEX_TASK_ID")
            .and_then(|v| crate::normalize_non_empty(&v.to_string_lossy()).map(OsString::from));
        self.cloud_diff_task_inner(task_id).await
    }

    /// Shows a Codex Cloud task diff by task id via `codex cloud diff <TASK_ID>`.
    pub async fn cloud_diff_task(
        &self,
        task_id: impl AsRef<str>,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let task_id = task_id.as_ref().trim();
        if task_id.is_empty() {
            return Err(CodexError::EmptyTaskId);
        }
        self.cloud_diff_task_inner(Some(OsString::from(task_id)))
            .await
    }

    async fn apply_task_inner(
        &self,
        task_id: Option<OsString>,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let mut args = vec![OsString::from("apply")];
        if let Some(task_id) = task_id {
            args.push(task_id);
        }
        self.capture_codex_command(args, false).await
    }

    async fn cloud_diff_task_inner(
        &self,
        task_id: Option<OsString>,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let mut args = vec![OsString::from("cloud"), OsString::from("diff")];
        if let Some(task_id) = task_id {
            args.push(task_id);
        }
        self.capture_codex_command(args, false).await
    }

    async fn capture_codex_command(
        &self,
        args: Vec<OsString>,
        include_search: bool,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let dir_ctx = self.directory_context()?;
        let resolved_overrides = resolve_cli_overrides(
            &self.cli_overrides,
            &CliOverridesPatch::default(),
            self.model.as_deref(),
        );

        let mut command = Command::new(self.command_env.binary_path());
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .current_dir(dir_ctx.path());

        apply_cli_overrides(&mut command, &resolved_overrides, include_search);
        command.args(&args);
        self.command_env.apply(&mut command)?;

        let mut child = spawn_with_retry(&mut command, self.command_env.binary_path())?;

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

        Ok(ApplyDiffArtifacts {
            status,
            stdout: String::from_utf8(stdout_bytes)?,
            stderr: String::from_utf8(stderr_bytes)?,
        })
    }
}
