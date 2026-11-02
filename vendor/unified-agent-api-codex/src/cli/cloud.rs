use crate::CliOverridesPatch;
use serde_json::Value;
use std::process::ExitStatus;

/// Request for `codex cloud` (overview/help).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudOverviewRequest {
    pub overrides: CliOverridesPatch,
}

impl CloudOverviewRequest {
    pub fn new() -> Self {
        Self {
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for CloudOverviewRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex cloud list`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudListRequest {
    pub json: bool,
    pub env_id: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub overrides: CliOverridesPatch,
}

impl CloudListRequest {
    pub fn new() -> Self {
        Self {
            json: false,
            env_id: None,
            limit: None,
            cursor: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn json(mut self, enable: bool) -> Self {
        self.json = enable;
        self
    }

    pub fn env_id(mut self, env_id: impl Into<String>) -> Self {
        let env_id = env_id.into();
        self.env_id = (!env_id.trim().is_empty()).then_some(env_id);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        let cursor = cursor.into();
        self.cursor = (!cursor.trim().is_empty()).then_some(cursor);
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for CloudListRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from `codex cloud list`.
#[derive(Clone, Debug, PartialEq)]
pub struct CloudListOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    /// Parsed JSON output when `--json` was requested.
    pub json: Option<Value>,
}

/// Request for `codex cloud status <TASK_ID>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStatusRequest {
    pub task_id: String,
    pub overrides: CliOverridesPatch,
}

impl CloudStatusRequest {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex cloud exec`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudExecRequest {
    pub env_id: String,
    pub query: Option<String>,
    pub attempts: Option<u32>,
    pub branch: Option<String>,
    pub overrides: CliOverridesPatch,
}

impl CloudExecRequest {
    pub fn new(env_id: impl Into<String>) -> Self {
        Self {
            env_id: env_id.into(),
            query: None,
            attempts: None,
            branch: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        let query = query.into();
        self.query = (!query.trim().is_empty()).then_some(query);
        self
    }

    pub fn attempts(mut self, attempts: u32) -> Self {
        self.attempts = Some(attempts);
        self
    }

    pub fn branch(mut self, branch: impl Into<String>) -> Self {
        let branch = branch.into();
        self.branch = (!branch.trim().is_empty()).then_some(branch);
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}
