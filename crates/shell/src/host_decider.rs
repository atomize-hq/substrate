use reedline::{ExecDecision, HostCommandDecider};
use std::sync::Arc;

pub struct SubstrateHostDecider {
    // Could add config/allowlists here if needed
}

impl SubstrateHostDecider {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}

impl HostCommandDecider for SubstrateHostDecider {
    fn decide(&self, line: &str) -> ExecDecision {
        // Skip empty lines
        if line.trim().is_empty() {
            return ExecDecision::Success(line.to_string());
        }
        
        // Check if PTY is disabled
        if crate::is_pty_disabled() {
            return ExecDecision::Success(line.to_string());
        }
        
        // Check if it's a forced PTY command
        if crate::is_force_pty_command(line) {
            log::debug!("Forcing PTY for command: {}", line);
            return ExecDecision::ExecuteHostCommand(line.to_string());
        }
        
        // Check if the command needs PTY
        if crate::needs_pty(line) {
            log::debug!("Command needs PTY: {}", line);
            return ExecDecision::ExecuteHostCommand(line.to_string());
        }
        
        // Default: normal REPL submission
        ExecDecision::Success(line.to_string())
    }
}