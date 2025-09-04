//! Replay module for deterministic trace replay and regression testing
//!
//! This module provides functionality to replay recorded command sequences from trace.jsonl files,
//! enabling regression testing and debugging by reproducing exact command execution patterns.

pub mod compare;
pub mod regression;
pub mod replay;
pub mod state;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use substrate_common::FsDiff;

/// Result of replaying a traced command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// The original span ID that was replayed
    pub span_id: String,
    /// Exit code from the replayed command
    pub exit_code: i32,
    /// Captured stdout
    pub stdout: Vec<u8>,
    /// Captured stderr
    pub stderr: Vec<u8>,
    /// Filesystem differences detected
    pub fs_diff: Option<FsDiff>,
    /// Network scopes that were accessed
    pub scopes_used: Vec<String>,
    /// Whether the replay matched the original execution
    pub matched: bool,
    /// Details about any divergence from original
    pub divergence: Option<DivergenceReport>,
    /// Warnings about context drift (env changes, etc.)
    pub warnings: Vec<String>,
}

/// Report of how replay diverged from original execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceReport {
    /// Type of divergence detected
    pub divergence_type: DivergenceType,
    /// Human-readable description
    pub description: String,
    /// Expected value from original trace
    pub expected: String,
    /// Actual value from replay
    pub actual: String,
    /// Severity of the divergence
    pub severity: DivergenceSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DivergenceType {
    ExitCode,
    StdoutMismatch,
    StderrMismatch,
    FilesystemDiff,
    NetworkScope,
    TimingDrift,
    EnvironmentChange,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DivergenceSeverity {
    Critical, // Command failed differently
    High,     // Output significantly different
    Medium,   // Minor output differences
    Low,      // Timing or non-deterministic differences
}

/// Configuration for replay execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    /// Path to trace.jsonl file
    pub trace_file: PathBuf,
    /// Whether to use strict matching (fail on any divergence)
    pub strict: bool,
    /// Timeout for each command replay (seconds)
    pub timeout: u64,
    /// Whether to use a fresh world for each replay
    pub fresh_world: bool,
    /// Environment variables to override
    pub env_overrides: std::collections::HashMap<String, String>,
    /// Whether to ignore timing differences
    pub ignore_timing: bool,
    /// Maximum output size to compare (bytes)
    pub max_output_compare: usize,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            trace_file: PathBuf::from("~/.substrate/trace.jsonl"),
            strict: false,
            timeout: 300, // 5 minutes default
            fresh_world: true,
            env_overrides: std::collections::HashMap::new(),
            ignore_timing: true,
            max_output_compare: 1024 * 1024, // 1MB
        }
    }
}

/// Main entry point for replaying a span
pub async fn replay_span(span_id: &str, config: &ReplayConfig) -> Result<ReplayResult> {
    // Load the original span from trace
    let original_span = state::load_span_from_trace(&config.trace_file, span_id).await?;
    
    // Reconstruct the execution state
    let exec_state = state::reconstruct_state(&original_span, &config.env_overrides)?;
    
    // Execute in a fresh world if configured
    let execution_result = if config.fresh_world {
        replay::execute_in_world(&exec_state, config.timeout).await?
    } else {
        replay::execute_direct(&exec_state, config.timeout).await?
    };
    
    // Compare results with original
    let comparison = compare::compare_execution(&original_span, &execution_result, config)?;
    
    // Build final result
    Ok(ReplayResult {
        span_id: span_id.to_string(),
        exit_code: execution_result.exit_code,
        stdout: execution_result.stdout,
        stderr: execution_result.stderr,
        fs_diff: execution_result.fs_diff,
        scopes_used: execution_result.scopes_used,
        matched: comparison.is_match(),
        divergence: comparison.divergence,
        warnings: comparison.warnings,
    })
}

/// Replay multiple spans and generate a regression report
pub async fn replay_batch(
    span_ids: &[String],
    config: &ReplayConfig,
) -> Result<regression::RegressionReport> {
    let mut results = Vec::new();
    
    for span_id in span_ids {
        match replay_span(span_id, config).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::warn!("Failed to replay span {}: {}", span_id, e);
                // Continue with other spans
            }
        }
    }
    
    regression::analyze_results(results)
}

/// Find all spans in a trace file that match certain criteria
pub async fn find_spans_to_replay(
    trace_file: &std::path::Path,
    filter: SpanFilter,
) -> Result<Vec<String>> {
    state::filter_spans_from_trace(trace_file, filter).await
}

/// Filter criteria for selecting spans to replay
#[derive(Debug, Clone, Default)]
pub struct SpanFilter {
    /// Only replay spans with these commands
    pub command_patterns: Vec<String>,
    /// Only replay spans from this time range
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    /// Only replay spans with specific exit codes
    pub exit_codes: Option<Vec<i32>>,
    /// Only replay spans from specific component
    pub component: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_replay_config_default() {
        let config = ReplayConfig::default();
        assert_eq!(config.timeout, 300);
        assert!(config.fresh_world);
        assert!(config.ignore_timing);
    }
}