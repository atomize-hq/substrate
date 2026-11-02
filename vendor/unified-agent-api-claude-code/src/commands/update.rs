use std::time::Duration;

use super::command::ClaudeCommandRequest;

#[derive(Debug, Clone)]
pub struct ClaudeUpdateRequest {
    timeout: Option<Duration>,
}

impl ClaudeUpdateRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["update"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for ClaudeUpdateRequest {
    fn default() -> Self {
        Self::new()
    }
}
