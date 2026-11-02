use std::{
    env,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use crate::{
    apply_diff::ApplyDiffArtifacts,
    builder::{apply_cli_overrides, resolve_cli_overrides, CliOverridesPatch},
    capabilities::{effective_path_env, resolve_binary_path},
    process::{spawn_with_retry, tee_stream, CommandOutput, ConsoleTarget},
    CodexClient, CodexError,
};
use tempfile::TempDir;
use tokio::{process::Command, time};

impl CodexClient {
    pub(crate) fn directory_context(&self) -> Result<DirectoryContext, CodexError> {
        if let Some(dir) = &self.working_dir {
            return Ok(DirectoryContext::Fixed(dir.clone()));
        }

        let temp = tempfile::tempdir().map_err(CodexError::TempDir)?;
        Ok(DirectoryContext::Ephemeral(temp))
    }

    pub(crate) fn sandbox_working_dir(
        &self,
        request_dir: Option<PathBuf>,
    ) -> Result<PathBuf, CodexError> {
        if let Some(dir) = request_dir {
            return Ok(dir);
        }

        if let Some(dir) = &self.working_dir {
            return Ok(dir.clone());
        }

        env::current_dir().map_err(|source| CodexError::WorkingDirectory { source })
    }

    pub(crate) async fn run_simple_command_with_overrides(
        &self,
        args: Vec<OsString>,
        overrides: CliOverridesPatch,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let dir_ctx = self.directory_context()?;
        let resolved_overrides =
            resolve_cli_overrides(&self.cli_overrides, &overrides, self.model.as_deref());

        let mut command = Command::new(self.command_env.binary_path());
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .current_dir(dir_ctx.path());

        apply_cli_overrides(&mut command, &resolved_overrides, true);
        command.args(args);

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

        let timeout = self.timeout;
        let wait_task = async move {
            let _dir_ctx = dir_ctx;
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

        let (status, stdout_bytes, stderr_bytes) = if timeout.is_zero() {
            wait_task.await?
        } else {
            match time::timeout(timeout, wait_task).await {
                Ok(result) => result?,
                Err(_) => {
                    return Err(CodexError::Timeout { timeout });
                }
            }
        };

        if !status.success() {
            return Err(CodexError::NonZeroExit {
                status,
                stderr: String::from_utf8(stderr_bytes)?,
            });
        }

        Ok(ApplyDiffArtifacts {
            status,
            stdout: String::from_utf8(stdout_bytes)?,
            stderr: String::from_utf8(stderr_bytes)?,
        })
    }

    pub(crate) async fn run_basic_command<S, I>(&self, args: I) -> Result<CommandOutput, CodexError>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
    {
        self.run_basic_command_with_env_overrides_and_current_dir(args, &[], None)
            .await
    }

    pub(crate) async fn run_basic_command_with_env_overrides_and_current_dir<S, I>(
        &self,
        args: I,
        env_overrides: &[(String, String)],
        current_dir: Option<&Path>,
    ) -> Result<CommandOutput, CodexError>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
    {
        self.run_basic_command_with_custody(args, env_overrides, current_dir, None)
            .await
    }

    pub(crate) async fn run_basic_command_with_custody<S, I>(
        &self,
        args: I,
        env_overrides: &[(String, String)],
        current_dir: Option<&Path>,
        custody: Option<&crate::ExecProcessCustody>,
    ) -> Result<CommandOutput, CodexError>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
    {
        let binary_path = resolve_binary_path(
            self.command_env.binary_path(),
            current_dir,
            effective_path_env(env_overrides),
        );
        let mut command = Command::new(&binary_path);
        command
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        if let Some(current_dir) = current_dir {
            command.current_dir(current_dir);
        }

        self.command_env.apply(&mut command)?;
        for (key, value) in env_overrides {
            command.env(key, value);
        }

        let custody = custody.cloned().unwrap_or_default();
        let (child, owner) = custody.spawn_admitted(&mut command, &binary_path).await?;
        let timeout = self.timeout;
        let (ready_tx, _ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(crate::process_custody::drive_child(
            child,
            owner,
            None,
            true,
            (!timeout.is_zero()).then_some(timeout),
            true,
            false,
            ready_tx,
            |stdout| async move { tee_stream(stdout, ConsoleTarget::Stdout, false).await },
        ));
        let mut cancelled_on_drop = custody.cancel_on_drop();
        // Dropping this await detaches only the observer; the task retains Child and all joins.
        let output = task.await.map_err(CodexError::Join)?;
        cancelled_on_drop.disarm();
        let status = output
            .status
            .map_err(|source| CodexError::Wait { source })?;
        if output.settlement_failed {
            return Err(CodexError::CaptureIo(std::io::Error::other(
                "probe process settlement failed",
            )));
        }
        if output.timed_out {
            return Err(CodexError::Timeout { timeout });
        }
        output.stdin.map_err(CodexError::StdinWrite)?;
        let stdout = output
            .stdout
            .map_err(CodexError::CaptureIo)?
            .ok_or_else(|| {
                CodexError::CaptureIo(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "probe stdout disposed",
                ))
            })?;
        let stderr = output
            .stderr
            .map_err(CodexError::CaptureIo)?
            .unwrap_or_default();
        Ok(CommandOutput {
            status,
            stdout,
            stderr,
        })
    }
}

pub(crate) enum DirectoryContext {
    Fixed(PathBuf),
    Ephemeral(TempDir),
}

impl DirectoryContext {
    pub(crate) fn path(&self) -> &Path {
        match self {
            DirectoryContext::Fixed(path) => path.as_path(),
            DirectoryContext::Ephemeral(dir) => dir.path(),
        }
    }
}
