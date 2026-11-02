use std::time::Duration;

use super::command::ClaudeCommandRequest;

#[derive(Debug, Clone)]
pub struct ClaudeSetupTokenRequest {
    pub(crate) timeout: Option<Duration>,
}

impl ClaudeSetupTokenRequest {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    /// Timeout for the overall `claude setup-token` flow (including waiting for user input).
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["setup-token"]);
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for ClaudeSetupTokenRequest {
    fn default() -> Self {
        Self::new()
    }
}
