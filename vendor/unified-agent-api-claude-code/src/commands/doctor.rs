use std::time::Duration;

use super::command::ClaudeCommandRequest;

#[derive(Debug, Clone)]
pub struct ClaudeDoctorRequest {
    press_enter: bool,
    timeout: Option<Duration>,
}

impl ClaudeDoctorRequest {
    pub fn new() -> Self {
        Self {
            press_enter: true,
            timeout: None,
        }
    }

    /// Some `claude doctor` versions wait for `Enter` before exiting. When enabled (default),
    /// the wrapper sends a single newline on stdin so the command can terminate cleanly.
    pub fn press_enter(mut self, press_enter: bool) -> Self {
        self.press_enter = press_enter;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        let mut cmd = ClaudeCommandRequest::new(["doctor"]);
        if self.press_enter {
            cmd = cmd.stdin_bytes(b"\n".to_vec());
        }
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd
    }
}

impl Default for ClaudeDoctorRequest {
    fn default() -> Self {
        Self::new()
    }
}
