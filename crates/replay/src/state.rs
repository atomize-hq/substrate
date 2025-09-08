//! State reconstruction from trace spans

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::replay::ExecutionState;
use crate::SpanFilter;

/// A span loaded from the trace file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub ts: DateTime<Utc>,
    pub event_type: String,
    pub span_id: String,
    pub session_id: String,
    pub component: String,
    pub cmd: String,
    pub cwd: Option<PathBuf>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub policy_decision: Option<PolicyDecision>,
    pub fs_diff: Option<substrate_common::FsDiff>,
    pub scopes_used: Option<Vec<String>>,
    pub replay_context: Option<ReplayContext>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub env_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub action: String,
    pub reason: Option<String>,
    pub restrictions: Option<Vec<String>>,
}

/// Context needed to replay a command deterministically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayContext {
    pub path: Option<String>,
    pub env_hash: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub shell: Option<String>,
    pub term: Option<String>,
    pub world_image: Option<String>,
}

/// Load a specific span from the trace file
pub async fn load_span_from_trace(trace_file: &Path, span_id: &str) -> Result<TraceSpan> {
    let file = tokio::fs::File::open(trace_file)
        .await
        .context("Failed to open trace file")?;
    
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            continue;
        }
        
        let value: Value = serde_json::from_str(&line)
            .context("Failed to parse trace line as JSON")?;
        
        // Check if this is the span we're looking for
        if let Some(sid) = value.get("span_id").and_then(|v| v.as_str()) {
            if sid == span_id {
                // Parse into our TraceSpan structure
                let span: TraceSpan = serde_json::from_value(value)
                    .context("Failed to deserialize trace span")?;
                
                // Only return completed spans
                if span.event_type == "command_complete" {
                    return Ok(span);
                }
            }
        }
    }
    
    anyhow::bail!("Span {} not found in trace file", span_id)
}

/// Reconstruct execution state from a trace span
pub fn reconstruct_state(
    span: &TraceSpan,
    env_overrides: &HashMap<String, String>,
) -> Result<ExecutionState> {
    // Parse command into binary and args
    let (command, args) = crate::replay::parse_command(&span.cmd);
    
    // Reconstruct working directory
    let cwd = span.cwd.clone().unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"))
    });
    
    // Reconstruct environment
    let mut env = HashMap::new();
    
    // Start with replay context environment if available
    if let Some(ctx) = &span.replay_context {
        if let Some(path) = &ctx.path {
            env.insert("PATH".to_string(), path.clone());
        }
        if let Some(user) = &ctx.user {
            env.insert("USER".to_string(), user.clone());
        }
        if let Some(shell) = &ctx.shell {
            env.insert("SHELL".to_string(), shell.clone());
        }
        if let Some(term) = &ctx.term {
            env.insert("TERM".to_string(), term.clone());
        }
        if let Some(hostname) = &ctx.hostname {
            env.insert("HOSTNAME".to_string(), hostname.clone());
        }
    }
    
    // Apply any overrides
    for (key, value) in env_overrides {
        env.insert(key.clone(), value.clone());
    }
    
    // Add substrate-specific environment
    env.insert("SUBSTRATE_REPLAY".to_string(), "1".to_string());
    env.insert("SHIM_SESSION_ID".to_string(), span.session_id.clone());
    env.insert("SHIM_PARENT_SPAN".to_string(), span.span_id.clone());
    
    Ok(ExecutionState {
        raw_cmd: span.cmd.clone(),
        command,
        args,
        cwd,
        env,
        stdin: None, // TODO: Support stdin replay if captured
        session_id: span.session_id.clone(),
        span_id: span.span_id.clone(),
    })
}

/// Filter spans from trace file based on criteria
pub async fn filter_spans_from_trace(
    trace_file: &Path,
    filter: SpanFilter,
) -> Result<Vec<String>> {
    let file = tokio::fs::File::open(trace_file)
        .await
        .context("Failed to open trace file")?;
    
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut matching_spans = Vec::new();
    
    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            continue;
        }
        
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue, // Skip malformed lines
        };
        
        // Only process completed spans
        if value.get("event_type").and_then(|v| v.as_str()) != Some("command_complete") {
            continue;
        }
        
        let span: TraceSpan = match serde_json::from_value(value) {
            Ok(s) => s,
            Err(_) => continue,
        };
        
        // Apply filters
        if !filter.command_patterns.is_empty() {
            let matches = filter.command_patterns.iter().any(|pattern| {
                span.cmd.contains(pattern)
            });
            if !matches {
                continue;
            }
        }
        
        if let Some((start, end)) = &filter.time_range {
            if span.ts < *start || span.ts > *end {
                continue;
            }
        }
        
        if let Some(exit_codes) = &filter.exit_codes {
            if let Some(exit) = span.exit_code {
                if !exit_codes.contains(&exit) {
                    continue;
                }
            }
        }
        
        if let Some(component) = &filter.component {
            if span.component != *component {
                continue;
            }
        }
        
        matching_spans.push(span.span_id);
    }
    
    Ok(matching_spans)
}

/// Check for environment drift between original and current context
pub fn detect_context_drift(original: &ReplayContext) -> Vec<String> {
    let mut warnings = Vec::new();
    
    // Check if PATH has changed significantly
    if let Some(orig_path) = &original.path {
        if let Ok(current_path) = std::env::var("PATH") {
            if orig_path != &current_path {
                // Simple check - could be more sophisticated
                let orig_count = orig_path.split(':').count();
                let curr_count = current_path.split(':').count();
                
                if (orig_count as i32 - curr_count as i32).abs() > 5 {
                    warnings.push(format!(
                        "PATH has changed significantly ({} vs {} entries)",
                        orig_count, curr_count
                    ));
                }
            }
        }
    }
    
    // Check hostname
    if let Some(orig_hostname) = &original.hostname {
        if let Ok(current_hostname) = hostname::get() {
            let current = current_hostname.to_string_lossy();
            if orig_hostname != &current {
                warnings.push(format!(
                    "Hostname changed: {} -> {}",
                    orig_hostname, current
                ));
            }
        }
    }
    
    // Check user
    if let Some(orig_user) = &original.user {
        if let Ok(current_user) = std::env::var("USER") {
            if orig_user != &current_user {
                warnings.push(format!(
                    "User changed: {} -> {}",
                    orig_user, current_user
                ));
            }
        }
    }
    
    warnings
}

/// Calculate a hash of environment variables for comparison
pub fn hash_env_vars() -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    let mut env_pairs: Vec<(String, String)> = std::env::vars().collect();
    env_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    
    for (key, value) in env_pairs {
        // Skip volatile environment variables
        if key.starts_with("SHIM_") || 
           key.starts_with("SUBSTRATE_") ||
           key == "PWD" || 
           key == "OLDPWD" ||
           key == "SHLVL" {
            continue;
        }
        
        hasher.update(format!("{}={}", key, value).as_bytes());
    }
    
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;
    
    #[tokio::test]
    async fn test_load_span_from_trace() {
        let temp_file = NamedTempFile::new().unwrap();
        let trace_content = r#"
{"ts":"2024-01-01T00:00:00Z","event_type":"command_start","span_id":"test-span-1","session_id":"session-1","component":"shell","cmd":"echo test"}
{"ts":"2024-01-01T00:00:01Z","event_type":"command_complete","span_id":"test-span-1","session_id":"session-1","component":"shell","cmd":"echo test","exit_code":0}
{"ts":"2024-01-01T00:00:02Z","event_type":"command_complete","span_id":"test-span-2","session_id":"session-1","component":"shell","cmd":"ls","exit_code":0}
"#;
        
        let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
        file.write_all(trace_content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        drop(file);
        
        let span = load_span_from_trace(temp_file.path(), "test-span-1").await.unwrap();
        assert_eq!(span.span_id, "test-span-1");
        assert_eq!(span.cmd, "echo test");
        assert_eq!(span.exit_code, Some(0));
    }
    
    #[test]
    fn test_reconstruct_state() {
        let span = TraceSpan {
            ts: Utc::now(),
            event_type: "command_complete".to_string(),
            span_id: "test-span".to_string(),
            session_id: "test-session".to_string(),
            component: "shell".to_string(),
            cmd: "echo hello world".to_string(),
            cwd: Some(PathBuf::from("/tmp")),
            exit_code: Some(0),
            duration_ms: Some(100),
            policy_decision: None,
            fs_diff: None,
            scopes_used: None,
            replay_context: Some(ReplayContext {
                path: Some("/usr/bin:/bin".to_string()),
                env_hash: "abc123".to_string(),
                hostname: Some("test-host".to_string()),
                user: Some("testuser".to_string()),
                shell: Some("/bin/bash".to_string()),
                term: Some("xterm-256color".to_string()),
                world_image: None,
            }),
            stdout: None,
            stderr: None,
            env_hash: None,
        };
        
        let state = reconstruct_state(&span, &HashMap::new()).unwrap();
        assert_eq!(state.command, "echo");
        assert_eq!(state.args, vec!["hello", "world"]);
        assert_eq!(state.cwd, PathBuf::from("/tmp"));
        assert_eq!(state.env.get("PATH"), Some(&"/usr/bin:/bin".to_string()));
        assert_eq!(state.env.get("USER"), Some(&"testuser".to_string()));
    }
}
