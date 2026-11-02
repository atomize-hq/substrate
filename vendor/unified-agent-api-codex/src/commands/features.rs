use std::ffi::OsString;

use tokio::{process::Command, time};

use crate::{
    builder::{apply_cli_overrides, resolve_cli_overrides},
    process::{spawn_with_retry, tee_stream, ConsoleTarget},
    ApplyDiffArtifacts, CodexClient, CodexError, FeaturesCommandRequest, FeaturesDisableRequest,
    FeaturesEnableRequest, FeaturesListOutput, FeaturesListRequest,
};

impl CodexClient {
    /// Runs `codex features` and returns captured output.
    pub async fn features(
        &self,
        request: FeaturesCommandRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        self.run_simple_command_with_overrides(vec![OsString::from("features")], request.overrides)
            .await
    }

    /// Enables a CLI feature via `codex features enable <FEATURE>`.
    pub async fn features_enable(
        &self,
        request: FeaturesEnableRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let FeaturesEnableRequest { feature, overrides } = request;
        self.run_simple_command_with_overrides(
            vec![
                OsString::from("features"),
                OsString::from("enable"),
                OsString::from(feature),
            ],
            overrides,
        )
        .await
    }

    /// Disables a CLI feature via `codex features disable <FEATURE>`.
    pub async fn features_disable(
        &self,
        request: FeaturesDisableRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let FeaturesDisableRequest { feature, overrides } = request;
        self.run_simple_command_with_overrides(
            vec![
                OsString::from("features"),
                OsString::from("disable"),
                OsString::from(feature),
            ],
            overrides,
        )
        .await
    }

    /// Lists CLI features via `codex features list`.
    ///
    /// Requests JSON output when `json(true)` is set and falls back to parsing the text table when
    /// JSON is unavailable. Shared config/profile/search/approval overrides flow through via the
    /// request/builder, stdout/stderr are mirrored according to the builder, and non-zero exits
    /// surface as [`CodexError::NonZeroExit`].
    pub async fn list_features(
        &self,
        request: FeaturesListRequest,
    ) -> Result<FeaturesListOutput, CodexError> {
        let FeaturesListRequest { json, overrides } = request;

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
        command.arg("features").arg("list");

        if json {
            command.arg("--json");
        }

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

        if !status.success() {
            return Err(CodexError::NonZeroExit {
                status,
                stderr: String::from_utf8(stderr_bytes)?,
            });
        }

        let stdout_string = String::from_utf8(stdout_bytes)?;
        let stderr_string = String::from_utf8(stderr_bytes)?;
        let (features, format) = crate::version::parse_feature_list_output(&stdout_string, json)
            .map_err(|reason| CodexError::FeatureListParse {
                reason,
                stdout: stdout_string.clone(),
            })?;

        Ok(FeaturesListOutput {
            status,
            stdout: stdout_string,
            stderr: stderr_string,
            features,
            format,
        })
    }
}
