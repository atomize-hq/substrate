//! Replay engine for re-executing traced commands

mod executor;
mod helpers;
mod planner;

pub use executor::execute_direct;
pub use helpers::{parse_command, world_isolation_available};
pub use planner::{execute_in_world, replay_sequence};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use substrate_common::FsDiff;

/// State required to execute a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    /// The original raw command string as captured in the span
    pub raw_cmd: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub stdin: Option<Vec<u8>>,
    pub session_id: String,
    pub span_id: String,
}

/// Result of executing a command
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub fs_diff: Option<FsDiff>,
    pub scopes_used: Vec<String>,
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use tempfile::tempdir;

    #[test]
    fn test_parse_command() {
        let (cmd, args) = parse_command("echo hello world");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello", "world"]);

        let (cmd, args) = parse_command("ls");
        assert_eq!(cmd, "ls");
        assert!(args.is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_execute_direct_simple() {
        let state = ExecutionState {
            raw_cmd: "echo test".to_string(),
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            cwd: std::env::current_dir().unwrap(),
            env: HashMap::new(),
            stdin: None,
            session_id: "test-session".to_string(),
            span_id: "test-span".to_string(),
        };

        let result = execute_direct(&state, 10).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "test");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_execute_direct_with_redirection() {
        let dir = tempdir().unwrap();
        let cwd = dir.path().to_path_buf();
        let state = ExecutionState {
            raw_cmd: "echo hello > out.txt".to_string(),
            command: "echo".to_string(),
            args: vec!["hello".to_string(), ">".to_string(), "out.txt".to_string()],
            cwd: cwd.clone(),
            env: HashMap::new(),
            stdin: None,
            session_id: "s".to_string(),
            span_id: "sp".to_string(),
        };
        let res = execute_direct(&state, 10).await.unwrap();
        assert_eq!(res.exit_code, 0);
        let content = std::fs::read_to_string(cwd.join("out.txt")).unwrap();
        assert_eq!(content.trim(), "hello");
    }
}
