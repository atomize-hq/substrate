use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::PathBuf,
    process::{ExitStatus, Stdio},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{process::Command, time};

use super::{
    apply_cli_overrides, resolve_cli_overrides, spawn_with_retry, tee_stream, CliOverridesPatch,
    CodexClient, CodexError, ConfigOverride, ConsoleTarget, FlagState,
};

/// Decision returned by execpolicy evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecPolicyDecision {
    Allow,
    Prompt,
    Forbidden,
}

/// Matched rule entry returned by `codex execpolicy check`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecPolicyRuleMatch {
    /// Optional rule name/identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Human-readable description when provided by the policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Decision attached to the rule. Defaults to [`ExecPolicyDecision::Allow`] when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<ExecPolicyDecision>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Matched execpolicy summary with the merged decision and contributing rules.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecPolicyMatch {
    pub decision: ExecPolicyDecision,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<ExecPolicyRuleMatch>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Response returned when no rules matched.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecPolicyNoMatch {
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Parsed output from `codex execpolicy check`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecPolicyEvaluation {
    #[serde(rename = "match", default, skip_serializing_if = "Option::is_none")]
    pub match_result: Option<ExecPolicyMatch>,
    #[serde(rename = "noMatch", default, skip_serializing_if = "Option::is_none")]
    pub no_match: Option<ExecPolicyNoMatch>,
}

impl ExecPolicyEvaluation {
    /// Returns the top-level decision when a policy matched.
    pub fn decision(&self) -> Option<ExecPolicyDecision> {
        self.match_result.as_ref().map(|result| result.decision)
    }
}

/// Captured output from `codex execpolicy check`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecPolicyCheckResult {
    /// Exit status returned by the subcommand.
    pub status: ExitStatus,
    /// Captured stdout (mirrored to the console when `mirror_stdout` is true).
    pub stdout: String,
    /// Captured stderr (mirrored unless `quiet` is set).
    pub stderr: String,
    /// Parsed decision JSON.
    pub evaluation: ExecPolicyEvaluation,
}

impl ExecPolicyCheckResult {
    /// Convenience accessor for the matched decision (if any).
    pub fn decision(&self) -> Option<ExecPolicyDecision> {
        self.evaluation.decision()
    }
}

/// Request to evaluate a command against Starlark execpolicy files.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecPolicyCheckRequest {
    /// One or more `.codexpolicy` files to merge with repeatable `--policy` flags.
    pub policies: Vec<PathBuf>,
    /// Pretty-print JSON output (`--pretty`).
    pub pretty: bool,
    /// Command argv forwarded after `--`. Must not be empty.
    pub command: Vec<OsString>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl ExecPolicyCheckRequest {
    pub fn new<I, S>(command: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self {
            policies: Vec::new(),
            pretty: false,
            command: command.into_iter().map(Into::into).collect(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Adds a single `--policy` path.
    pub fn policy(mut self, policy: impl Into<PathBuf>) -> Self {
        self.policies.push(policy.into());
        self
    }

    /// Adds multiple `--policy` paths.
    pub fn policies<I, P>(mut self, policies: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.policies
            .extend(policies.into_iter().map(|policy| policy.into()));
        self
    }

    /// Controls whether `--pretty` is forwarded.
    pub fn pretty(mut self, enable: bool) -> Self {
        self.pretty = enable;
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }

    /// Adds a `--config key=value` override for this request.
    pub fn config_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::new(key, value));
        self
    }

    /// Adds a raw `--config key=value` override without validation.
    pub fn config_override_raw(mut self, raw: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::from_raw(raw));
        self
    }

    /// Sets the config profile (`--profile`) for this request.
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        let profile = profile.into();
        self.overrides.profile = (!profile.trim().is_empty()).then_some(profile);
        self
    }

    /// Requests the CLI `--oss` flag for this call.
    pub fn oss(mut self, enable: bool) -> Self {
        self.overrides.oss = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }

    /// Adds a `--enable <feature>` toggle for this call.
    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.enable.push(name.into());
        self
    }

    /// Adds a `--disable <feature>` toggle for this call.
    pub fn disable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.disable.push(name.into());
        self
    }

    /// Controls whether `--search` is passed through to Codex.
    pub fn search(mut self, enable: bool) -> Self {
        self.overrides.search = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }
}

impl CodexClient {
    /// Evaluates a command against Starlark execpolicy files via `codex execpolicy check`.
    ///
    /// Forwards repeatable `--policy` paths, optional `--pretty`, and builder/request CLI overrides
    /// (config/profile/approval/sandbox/local-provider/cd/search). Captures stdout/stderr according to the
    /// builder, returns parsed JSON, and surfaces non-zero exits as [`CodexError::NonZeroExit`].
    /// Empty command argv returns [`CodexError::EmptyExecPolicyCommand`].
    pub async fn check_execpolicy(
        &self,
        request: ExecPolicyCheckRequest,
    ) -> Result<ExecPolicyCheckResult, CodexError> {
        if request.command.is_empty() {
            return Err(CodexError::EmptyExecPolicyCommand);
        }

        let ExecPolicyCheckRequest {
            policies,
            pretty,
            command,
            overrides,
        } = request;

        let dir_ctx = self.directory_context()?;
        let resolved_overrides =
            resolve_cli_overrides(&self.cli_overrides, &overrides, self.model.as_deref());

        let mut process = Command::new(self.command_env.binary_path());
        process
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .current_dir(dir_ctx.path());

        apply_cli_overrides(&mut process, &resolved_overrides, true);
        process.arg("execpolicy").arg("check");

        for policy in policies {
            process.arg("--policy").arg(policy);
        }

        if pretty {
            process.arg("--pretty");
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

        let stdout_string = String::from_utf8(stdout_bytes)?;
        let stderr_string = String::from_utf8(stderr_bytes)?;

        if !status.success() {
            return Err(CodexError::NonZeroExit {
                status,
                stderr: stderr_string,
            });
        }

        let evaluation: ExecPolicyEvaluation =
            serde_json::from_str(&stdout_string).map_err(|source| CodexError::ExecPolicyParse {
                stdout: stdout_string.clone(),
                source,
            })?;

        Ok(ExecPolicyCheckResult {
            status,
            stdout: stdout_string,
            stderr: stderr_string,
            evaluation,
        })
    }
}
