//! Shim execution context and environment detection
//!
//! This module handles environment setup, command detection, and context
//! management for shim execution.

use anyhow::{anyhow, Context, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Environment variable names used by the shim system
pub const SHIM_ACTIVE_VAR: &str = "SHIM_ACTIVE"; // Signals nested shim call (bypass mode)
pub const SHIM_DEPTH_VAR: &str = "SHIM_DEPTH"; // Tracks nesting depth (0-based)
pub const SHIM_SESSION_VAR: &str = "SHIM_SESSION_ID"; // UUIDv7 for command chain correlation
pub const ORIGINAL_PATH_VAR: &str = "SHIM_ORIGINAL_PATH"; // Clean PATH without shim directory
pub const TRACE_LOG_VAR: &str = "SHIM_TRACE_LOG"; // Path to JSONL trace log
pub const CACHE_BUST_VAR: &str = "SHIM_CACHE_BUST"; // Forces cache invalidation
pub const SHIM_CALLER_VAR: &str = "SHIM_CALLER"; // First shim in the call chain
pub const SHIM_CALL_STACK_VAR: &str = "SHIM_CALL_STACK"; // Comma-separated chain (capped at 8)
pub const SHIM_PARENT_CMD_VAR: &str = "SHIM_PARENT_CMD_ID"; // Links to substrate shell cmd_id

/// Execution context for a shim invocation
#[derive(Debug)]
pub struct ShimContext {
    /// The command name this shim was invoked as (e.g., "git", "npm")
    pub command_name: String,
    /// Directory containing shim binaries
    pub shim_dir: PathBuf,
    /// Clean search paths (excluding shim directory)
    pub search_paths: Vec<PathBuf>,
    /// Optional log file path
    pub log_file: Option<PathBuf>,
    /// Session ID for command chain correlation
    pub session_id: String,
    /// Execution depth for nested commands
    pub depth: u32,
}

impl ShimContext {
    /// Create context from current executable and environment
    pub fn from_current_exe() -> Result<Self> {
        let exe = env::current_exe().context("Failed to get current executable path")?;

        let shim_dir = exe
            .parent()
            .ok_or_else(|| anyhow!("Executable has no parent directory"))?
            .to_path_buf();

        let command_name = exe
            .file_name()
            .ok_or_else(|| anyhow!("Executable has no filename"))?
            .to_string_lossy()
            .to_string();

        let original_path = env::var(ORIGINAL_PATH_VAR).ok();
        let search_paths = build_clean_search_path(&shim_dir, original_path)?;

        let log_file = env::var(TRACE_LOG_VAR).ok().map(PathBuf::from);

        // Track execution depth for nested commands
        let depth = env::var(SHIM_DEPTH_VAR)
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        // Generate or inherit session ID for command chain correlation
        let session_id =
            env::var(SHIM_SESSION_VAR).unwrap_or_else(|_| uuid::Uuid::now_v7().to_string());

        Ok(Self {
            command_name,
            shim_dir,
            search_paths,
            log_file,
            session_id,
            depth,
        })
    }

    /// Check if we should skip execution (already shimmed)
    pub fn should_skip_shimming(&self) -> bool {
        env::var(SHIM_ACTIVE_VAR).is_ok()
    }

    /// Check if bypass mode is enabled
    /// When SHIM_BYPASS=1, the shim will skip all logging and tracing,
    /// executing the real binary directly without any instrumentation.
    /// This provides a true "no-trace" escape hatch for debugging.
    pub fn is_bypass_enabled() -> bool {
        env::var("SHIM_BYPASS").as_deref() == Ok("1")
    }

    /// Set up environment for command execution (idempotent)
    pub fn setup_execution_env(&self) {
        env::set_var(SHIM_SESSION_VAR, &self.session_id);

        // Track the caller chain for debugging
        if env::var(SHIM_CALLER_VAR).is_err() {
            // First shim in the chain
            env::set_var(SHIM_CALLER_VAR, &self.command_name);
            env::set_var(SHIM_CALL_STACK_VAR, &self.command_name);
        } else {
            // Append to call stack with safety limits
            let current_stack = env::var(SHIM_CALL_STACK_VAR).unwrap_or_default();
            let new_stack = build_safe_call_stack(&current_stack, &self.command_name);
            env::set_var(SHIM_CALL_STACK_VAR, new_stack);
        }

        // Only set SHIM_ACTIVE if not already set (idempotent)
        if env::var(SHIM_ACTIVE_VAR).is_err() {
            env::set_var(SHIM_ACTIVE_VAR, "1");
        }

        // Always increment depth for observability
        let current_depth = env::var(SHIM_DEPTH_VAR)
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        env::set_var(SHIM_DEPTH_VAR, (current_depth + 1).to_string());
    }
}

/// Build safe call stack with limits to prevent loops and memory issues
fn build_safe_call_stack(current: &str, new_cmd: &str) -> String {
    const MAX_STACK_ITEMS: usize = 8;

    if current.is_empty() {
        return new_cmd.to_string();
    }

    let mut items: Vec<&str> = current.split(',').collect();

    // Don't add consecutive duplicates
    if items.last() == Some(&new_cmd) {
        return current.to_string();
    }

    items.push(new_cmd);

    // Cap at MAX_STACK_ITEMS and add ellipsis if truncated
    if items.len() > MAX_STACK_ITEMS {
        items = items[items.len() - MAX_STACK_ITEMS + 1..].to_vec();
        items.insert(0, "...");
    }

    items.join(",")
}

/// Build clean search path excluding shim directory
pub fn build_clean_search_path(
    shim_dir: &Path,
    original_path: Option<String>,
) -> Result<Vec<PathBuf>> {
    let path_str = original_path
        .or_else(|| env::var("PATH").ok())
        .ok_or_else(|| anyhow!("No PATH or SHIM_ORIGINAL_PATH found"))?;

    let separator = if cfg!(windows) { ';' } else { ':' };

    // Helper to validate PATH entries
    fn is_good_dir(p: &str) -> bool {
        let pb = std::path::Path::new(p);
        pb.is_absolute() && pb.is_dir()
    }

    // Deduplicate PATH entries for predictable resolution
    let mut seen = std::collections::HashSet::new();
    let paths: Vec<PathBuf> = path_str
        .split(separator)
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches('/'))
        .filter(|p| !Path::new(p).starts_with(shim_dir))
        .filter(|p| is_good_dir(p)) // Validate paths
        .filter(|p| seen.insert(p.to_string()))
        .map(PathBuf::from)
        .collect();

    if paths.is_empty() {
        return Err(anyhow!("No valid search paths found after filtering"));
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_clean_search_path_filters_shim_dir() {
        let temp = TempDir::new().unwrap();
        let shim_dir = temp.path().join("shims");
        fs::create_dir(&shim_dir).unwrap();

        let original_path = format!("/usr/bin:{}:/bin", shim_dir.display());
        let paths = build_clean_search_path(&shim_dir, Some(original_path)).unwrap();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("/usr/bin"));
        assert_eq!(paths[1], PathBuf::from("/bin"));
    }

    #[test]
    fn test_path_deduplication() {
        let temp = TempDir::new().unwrap();
        let shim_dir = temp.path().join("shims");
        fs::create_dir(&shim_dir).unwrap();

        // PATH with duplicates
        let original_path = "/usr/bin:/bin:/usr/bin:/usr/local/bin:/bin".to_string();
        let paths = build_clean_search_path(&shim_dir, Some(original_path)).unwrap();

        // Should be deduplicated
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], PathBuf::from("/usr/bin"));
        assert_eq!(paths[1], PathBuf::from("/bin"));
        assert_eq!(paths[2], PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn test_safe_call_stack() {
        // Test basic append
        assert_eq!(build_safe_call_stack("", "npm"), "npm");
        assert_eq!(build_safe_call_stack("npm", "node"), "npm,node");

        // Test consecutive deduplication
        assert_eq!(build_safe_call_stack("npm,node", "node"), "npm,node");
        assert_eq!(build_safe_call_stack("npm", "npm"), "npm");

        // Test A→B→A→B pattern (no consecutive dups)
        let stack = build_safe_call_stack("", "A");
        let stack = build_safe_call_stack(&stack, "B");
        let stack = build_safe_call_stack(&stack, "A");
        let stack = build_safe_call_stack(&stack, "B");
        assert_eq!(stack, "A,B,A,B");

        // Test capping at 8 items
        let mut stack = String::new();
        for i in 1..=10 {
            stack = build_safe_call_stack(&stack, &format!("cmd{i}"));
        }
        // Should be capped: "...,cmd3,cmd4,cmd5,cmd6,cmd7,cmd8,cmd9,cmd10"
        assert!(stack.starts_with("..."));
        assert!(stack.contains("cmd10"));
        let parts: Vec<&str> = stack.split(',').collect();
        assert_eq!(parts.len(), 8);
    }
}
