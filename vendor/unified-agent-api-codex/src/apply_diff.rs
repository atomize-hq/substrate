use std::process::ExitStatus;

use super::CliOverridesPatch;

/// Captured output from task-oriented subcommands such as `codex apply <TASK_ID>` or
/// `codex cloud diff <TASK_ID>`.
#[derive(Clone, Debug)]
pub struct ApplyDiffArtifacts {
    /// Exit status returned by the subcommand.
    pub status: ExitStatus,
    /// Captured stdout (mirrored to the console when `mirror_stdout` is true).
    pub stdout: String,
    /// Captured stderr (mirrored unless `quiet` is set).
    pub stderr: String,
}

/// Request for `codex cloud diff [--attempt N] <TASK_ID>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudDiffRequest {
    pub task_id: String,
    pub attempt: Option<u32>,
    pub overrides: CliOverridesPatch,
}

impl CloudDiffRequest {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            attempt: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn attempt(mut self, attempt: u32) -> Self {
        self.attempt = Some(attempt);
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex cloud apply [--attempt N] <TASK_ID>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudApplyRequest {
    pub task_id: String,
    pub attempt: Option<u32>,
    pub overrides: CliOverridesPatch,
}

impl CloudApplyRequest {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            attempt: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn attempt(mut self, attempt: u32) -> Self {
        self.attempt = Some(attempt);
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}
