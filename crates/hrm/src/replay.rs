//! Replay engine for re-executing traced commands

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use substrate_common::FsDiff;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// State required to execute a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
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

/// Execute a command in an isolated world
pub async fn execute_in_world(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    // Check if world isolation is available
    if world_isolation_available() {
        execute_with_world_isolation(state, timeout_secs).await
    } else {
        // PR#12 Phase 1: Direct execution while world backend API stabilizes
        // This aligns with Phase 4's parallel development strategy
        tracing::info!(
            "HRM replay using direct execution (world integration pending PR#9-10 stabilization)"
        );
        execute_direct(state, timeout_secs).await
    }
}

/// Check if world isolation backend is available
fn world_isolation_available() -> bool {
    // TODO: Implement actual check once world backend API is stable
    // For now, return false to use direct execution
    std::env::var("SUBSTRATE_HRM_USE_WORLD").unwrap_or_default() == "1"
}

/// Execute with full world isolation (future implementation)
async fn execute_with_world_isolation(
    _state: &ExecutionState,
    _timeout_secs: u64,
) -> Result<ExecutionResult> {
    // TODO: Implement once world backend API stabilizes
    // This will use SessionWorld::ensure_started() and related methods
    // Structure matches planning doc's expectation:
    // - Create WorldSpec with replay-specific settings
    // - Execute in isolated world
    // - Collect telemetry (fs_diff, network scopes)
    // - Clean up world
    
    anyhow::bail!("World isolation not yet implemented - use SUBSTRATE_HRM_USE_WORLD=0")
}

/// Execute a command directly (without world isolation)
pub async fn execute_direct(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    // Set up command
    let mut cmd = Command::new(&state.command);
    cmd.args(&state.args);
    cmd.current_dir(&state.cwd);
    cmd.envs(&state.env);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    // Add substrate environment variables for correlation
    cmd.env("SHIM_SESSION_ID", &state.session_id);
    cmd.env("SHIM_PARENT_SPAN", &state.span_id);
    cmd.env("SUBSTRATE_REPLAY", "1");
    
    if state.stdin.is_some() {
        cmd.stdin(Stdio::piped());
    }
    
    // Execute with timeout
    let start = std::time::Instant::now();
    let result = match timeout(Duration::from_secs(timeout_secs), async {
        let mut child = cmd.spawn().context("Failed to spawn command")?;
        
        // Provide stdin if needed
        if let Some(stdin_data) = &state.stdin {
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                stdin.write_all(stdin_data).await
                    .context("Failed to write stdin")?;
            }
        }
        
        Ok::<_, anyhow::Error>(child.wait_with_output().await?)
    })
    .await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return Err(e),
        Err(_) => anyhow::bail!("Command execution timed out"),
    };
    
    let duration_ms = start.elapsed().as_millis() as u64;
    
    Ok(ExecutionResult {
        exit_code: result.status.code().unwrap_or(-1),
        stdout: result.stdout,
        stderr: result.stderr,
        fs_diff: None, // No isolation means no diff tracking
        scopes_used: Vec::new(),
        duration_ms,
    })
}

/// Parse command string into command and args
pub fn parse_command(cmd_str: &str) -> (String, Vec<String>) {
    // Simple parsing - in production would use shell_words or similar
    let parts: Vec<String> = cmd_str.split_whitespace().map(String::from).collect();
    
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }
    
    let command = parts[0].clone();
    let args = parts[1..].to_vec();
    
    (command, args)
}

/// Replay a command sequence (multiple related commands)
pub async fn replay_sequence(
    states: Vec<ExecutionState>,
    timeout_secs: u64,
    use_world: bool,
) -> Result<Vec<ExecutionResult>> {
    let mut results = Vec::new();
    
    for state in states {
        let result = if use_world {
            execute_in_world(&state, timeout_secs).await?
        } else {
            execute_direct(&state, timeout_secs).await?
        };
        
        // Check if we should continue after failure
        if result.exit_code != 0 {
            tracing::warn!(
                "Command failed with exit code {}: {}",
                result.exit_code, state.command
            );
        }
        
        results.push(result);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_command() {
        let (cmd, args) = parse_command("echo hello world");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello", "world"]);
        
        let (cmd, args) = parse_command("ls");
        assert_eq!(cmd, "ls");
        assert!(args.is_empty());
    }
    
    #[tokio::test]
    async fn test_execute_direct_simple() {
        let state = ExecutionState {
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
}