//! Shim execution context and environment detection
//!
//! This module handles environment setup, command detection, and context
//! management for shim execution.

use anyhow::{anyhow, Context, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Environment variable names used by the shim system
pub const SHIM_ACTIVE_VAR: &str = "SHIM_ACTIVE";
pub const SHIM_DEPTH_VAR: &str = "SHIM_DEPTH";
pub const SHIM_SESSION_VAR: &str = "SHIM_SESSION_ID";
pub const ORIGINAL_PATH_VAR: &str = "ORIGINAL_PATH";
pub const TRACE_LOG_VAR: &str = "TRACE_LOG_FILE";
pub const CACHE_BUST_VAR: &str = "SHIM_CACHE_BUST";

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
        let exe = env::current_exe()
            .context("Failed to get current executable path")?;

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
        let session_id = env::var(SHIM_SESSION_VAR)
            .unwrap_or_else(|_| uuid::Uuid::now_v7().to_string());

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
    pub fn is_bypass_enabled() -> bool {
        env::var("SHIM_BYPASS").as_deref() == Ok("1")
    }

    /// Set up environment for command execution (idempotent)
    pub fn setup_execution_env(&self) {
        env::set_var(SHIM_SESSION_VAR, &self.session_id);
        
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

/// Build clean search path excluding shim directory
pub fn build_clean_search_path(
    shim_dir: &Path,
    original_path: Option<String>,
) -> Result<Vec<PathBuf>> {
    let path_str = original_path
        .or_else(|| env::var("PATH").ok())
        .ok_or_else(|| anyhow!("No PATH or ORIGINAL_PATH found"))?;

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
        .filter(|p| is_good_dir(p))  // Validate paths
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
}