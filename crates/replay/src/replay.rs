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

/// Execute a command in an isolated world
pub async fn execute_in_world(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    // Use world-api backend on Linux when enabled
    if world_isolation_available() {
        #[cfg(target_os = "linux")]
        {
            // Use overlayfs-backed execution to compute fs_diff during replay.
            // Degrade to direct execution if overlay is not available.
            let world_id = &state.span_id;
            let bash_cmd = format!("bash -lc '{}'", state.raw_cmd.replace("'", "'\\''"));
            let start = std::time::Instant::now();
            match world::overlayfs::execute_with_overlay(
                world_id,
                &bash_cmd,
                &state.cwd,
                &state.cwd,
                &state.env,
            ) {
                Ok((output, fs_diff)) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    return Ok(ExecutionResult {
                        exit_code: output.status.code().unwrap_or(-1),
                        stdout: output.stdout,
                        stderr: output.stderr,
                        fs_diff: Some(fs_diff),
                        scopes_used: Vec::new(),
                        duration_ms,
                    });
                }
                Err(e) => {
                    if std::env::var("SUBSTRATE_REPLAY_VERBOSE").unwrap_or_default() == "1" {
                        eprintln!("[replay] warn: overlay execution unavailable: {}\n[replay] falling back to direct execution (no fs_diff)", e);
                    }
                    // Fall back to direct execution
                    return execute_direct(state, timeout_secs).await;
                }
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            // Fallback to direct on non-Linux
            return execute_direct(state, timeout_secs).await;
        }
    }
    // Fallback direct execution when isolation disabled
    execute_direct(state, timeout_secs).await
}

/// Check if world isolation backend is available
fn world_isolation_available() -> bool {
    // Check if world isolation is enabled and we're on Linux
    #[cfg(target_os = "linux")]
    {
        std::env::var("SUBSTRATE_REPLAY_USE_WORLD").unwrap_or_default() == "1"
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        false // World backend only available on Linux
    }
}

/// Execute with full world isolation
// legacy world-specific execution removed in favor of world-api path

/// Execute a command directly (without world isolation)
pub async fn execute_direct(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    // Prefer running via a shell to preserve quoting, pipes, redirects, etc.
    let mut cmd = Command::new("/bin/bash");
    cmd.arg("-lc").arg(&state.raw_cmd);
    cmd.current_dir(&state.cwd);
    // Minimal environment reinjection
    cmd.envs(&state.env);
    // Ensure a reasonable default shell environment
    if std::env::var("SHELL").is_err() { cmd.env("SHELL", "/bin/bash"); }
    if std::env::var("LANG").is_err() { cmd.env("LANG", "C.UTF-8"); }
    if std::env::var("LC_ALL").is_err() { cmd.env("LC_ALL", "C.UTF-8"); }
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
