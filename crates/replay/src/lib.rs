//! # Substrate Replay
//!
//! Deterministically replays recorded command spans from `trace.jsonl` so regressions can be
//! reproduced on demand. The crate provides helpers to select candidate spans, replay them in a
//! controlled world, and compare the results against the original execution.
//!
//! ## Example
//! ```rust
//! use anyhow::Result;
//! use substrate_replay::{find_spans_to_replay, ReplayConfig, SpanFilter};
//!
//! fn main() -> Result<()> {
//!     let rt = tokio::runtime::Runtime::new()?;
//!     let trace = tempfile::NamedTempFile::new()?;
//!
//!     std::fs::write(
//!         trace.path(),
//!         r#"{"ts":"2025-01-01T00:00:00Z","event_type":"command_complete","span_id":"spn_demo","session_id":"ses_demo","component":"shell","cmd":"echo demo","cwd":"/tmp","exit_code":0}"#,
//!     )?;
//!
//!     let spans = rt.block_on(async {
//!         find_spans_to_replay(trace.path(), SpanFilter::default()).await
//!     })?;
//!     assert_eq!(spans, vec!["spn_demo".to_string()]);
//!
//!     let mut config = ReplayConfig::default();
//!     config.trace_file = trace.path().to_path_buf();
//!     // Run a replay once a suitable backend is available:
//!     // rt.block_on(async { substrate_replay::replay_span("spn_demo", &config).await })?;
//!
//!     Ok(())
//! }
//! ```

pub mod compare;
pub mod regression;
pub mod replay;
pub mod state;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use substrate_common::{FsDiff, WorldRootMode};
use substrate_trace::ExecutionOrigin;
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1};

const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
const ANCHOR_PATH_ENV: &str = "SUBSTRATE_ANCHOR_PATH";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayPlatformBootstrapInputV1 {
    pub host_carrier: InstallBootstrapContextCarrierV1,
    pub platform_bootstrap_mapping: PlatformBootstrapMappingV1,
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
    /// Explicit authenticated platform authority for replay world backend construction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_bootstrap_mapping: Option<ReplayPlatformBootstrapInputV1>,
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
            platform_bootstrap_mapping: None,
        }
    }
}

fn is_replay_authority_failure(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        let rendered = cause.to_string();
        rendered.contains(
            "world replay on macOS requires explicit authenticated platform bootstrap input",
        ) || rendered.contains(
            "world replay on Windows requires explicit authenticated platform bootstrap input",
        ) || rendered.contains("world replay host carrier")
            || rendered.contains("world replay platform bootstrap mapping")
            || rendered.contains("world replay requires a Lima platform bootstrap mapping")
            || rendered.contains("world replay requires a WSL platform bootstrap mapping")
            || rendered.contains("world replay requires an explicit project path")
            || rendered
                .contains("world replay requires an explicit project path from the replay request")
            || rendered.contains("authenticated install bootstrap carrier")
            || rendered.contains("typed Lima")
            || rendered.contains("Windows WSL backend")
    })
}

fn apply_replay_config_to_state(
    exec_state: &mut crate::replay::ExecutionState,
    config: &ReplayConfig,
) {
    if config.fresh_world {
        exec_state.target_origin = ExecutionOrigin::World;
    }
}

fn request_carried_project_dir(span: &state::TraceSpan) -> Result<(WorldRootMode, PathBuf)> {
    let request_mode = span
        .replay_context
        .as_ref()
        .and_then(|ctx| ctx.anchor_mode.as_deref())
        .and_then(WorldRootMode::parse)
        .unwrap_or(WorldRootMode::Project);
    let request_anchor_path = span
        .replay_context
        .as_ref()
        .and_then(|ctx| ctx.anchor_path.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let request_cwd = span
        .cwd
        .as_ref()
        .filter(|path| !path.as_os_str().is_empty())
        .cloned();

    let project_dir = match request_mode {
        WorldRootMode::Project => request_cwd.ok_or_else(|| {
            anyhow!(
                "Windows world replay requires an explicit project path from the replay request"
            )
        })?,
        WorldRootMode::FollowCwd => request_cwd.ok_or_else(|| {
            anyhow!(
                "Windows world replay requires an explicit project path from the replay request"
            )
        })?,
        WorldRootMode::Custom => request_anchor_path.ok_or_else(|| {
            anyhow!(
                "Windows world replay requires an explicit project path from the replay request"
            )
        })?,
    };

    if project_dir.as_os_str().is_empty() {
        return Err(anyhow!(
            "Windows world replay requires an explicit project path"
        ));
    }

    Ok((request_mode, project_dir))
}

fn apply_windows_request_project_selector(
    exec_state: &mut crate::replay::ExecutionState,
    span: &state::TraceSpan,
) -> Result<()> {
    if !cfg!(target_os = "windows")
        || exec_state.platform_bootstrap_mapping.is_none()
        || exec_state.target_origin != ExecutionOrigin::World
    {
        return Ok(());
    }

    let (mode, project_dir) = request_carried_project_dir(span)?;
    exec_state
        .env
        .insert(ANCHOR_MODE_ENV.to_string(), mode.as_str().to_string());
    match mode {
        WorldRootMode::Project | WorldRootMode::Custom => {
            exec_state.env.insert(
                ANCHOR_PATH_ENV.to_string(),
                project_dir.to_string_lossy().to_string(),
            );
        }
        WorldRootMode::FollowCwd => {
            exec_state.env.remove(ANCHOR_PATH_ENV);
        }
    }
    Ok(())
}

/// Main entry point for replaying a span
pub async fn replay_span(span_id: &str, config: &ReplayConfig) -> Result<ReplayResult> {
    // Load the original span from trace
    let original_span = state::load_span_from_trace(&config.trace_file, span_id).await?;

    // Reconstruct the execution state
    let mut exec_state = state::reconstruct_state(
        &original_span,
        &config.env_overrides,
        config.platform_bootstrap_mapping.clone(),
    )?;
    apply_replay_config_to_state(&mut exec_state, config);
    apply_windows_request_project_selector(&mut exec_state, &original_span)?;

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
                if is_replay_authority_failure(&e) {
                    return Err(e);
                }
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

/// Filter criteria for selecting spans to replay.
///
/// ```
/// use substrate_replay::{find_spans_to_replay, SpanFilter};
/// use tempfile::NamedTempFile;
/// use tokio::runtime::Runtime;
///
/// # fn main() -> anyhow::Result<()> {
/// let trace = NamedTempFile::new()?;
/// std::fs::write(
///     trace.path(),
///     r#"{"ts":"2025-01-01T00:00:00Z","event_type":"command_complete","span_id":"spn-doc","session_id":"ses-doc","component":"shell","cmd":"echo doc","exit_code":0}"#,
/// )?;
///
/// let rt = Runtime::new()?;
/// let spans = rt.block_on(async {
///     find_spans_to_replay(
///         trace.path(),
///         SpanFilter {
///             command_patterns: vec!["echo".into()],
///             ..Default::default()
///         },
///     )
///     .await
/// })?;
///
/// assert_eq!(spans, vec!["spn-doc".to_string()]);
/// # Ok(()) }
/// ```
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
    use crate::replay::ExecutionState;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use substrate_trace::TransportMeta;
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
    };

    fn replay_platform_input() -> ReplayPlatformBootstrapInputV1 {
        let host_carrier = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix("/tmp/substrate", "alice", 1000)
                .expect("host context"),
        )
        .expect("host carrier");
        let mapping = PlatformBootstrapMappingV1::new_lima(
            &host_carrier,
            "substrate",
            "0123456789abcdef0123456789abcdef",
            "/Users/alice/.lima",
            "/home/substrate/.substrate",
            "substrate",
            1000,
            "/tmp/substrate/sock/agent.sock",
            "/run/substrate.sock",
        )
        .expect("mapping");
        ReplayPlatformBootstrapInputV1 {
            host_carrier,
            platform_bootstrap_mapping: mapping,
        }
    }

    #[tokio::test]
    async fn test_replay_config_default() {
        let config = ReplayConfig::default();
        assert_eq!(config.timeout, 300);
        assert!(config.fresh_world);
        assert!(config.ignore_timing);
        assert!(config.platform_bootstrap_mapping.is_none());
    }

    #[test]
    fn replay_config_serde_omits_platform_bootstrap_mapping_when_absent() {
        let config = ReplayConfig::default();
        let json = serde_json::to_value(&config).expect("serialize ReplayConfig");

        assert!(
            json.get("platform_bootstrap_mapping").is_none(),
            "optional replay authority should stay absent for backward compatibility: {json:?}"
        );

        let restored: ReplayConfig =
            serde_json::from_value(json).expect("deserialize ReplayConfig");
        assert!(restored.platform_bootstrap_mapping.is_none());
    }

    #[test]
    fn replay_config_serde_round_trips_platform_bootstrap_mapping() {
        let config = ReplayConfig {
            platform_bootstrap_mapping: Some(replay_platform_input()),
            ..ReplayConfig::default()
        };

        let restored: ReplayConfig =
            serde_json::from_str(&serde_json::to_string(&config).expect("serialize"))
                .expect("deserialize");

        assert_eq!(
            restored.platform_bootstrap_mapping,
            config.platform_bootstrap_mapping
        );
    }

    #[test]
    fn replay_authority_failures_are_classified_for_batch_propagation() {
        let err = anyhow::anyhow!(
            "world replay on Windows requires explicit authenticated platform bootstrap input"
        );

        assert!(is_replay_authority_failure(&err));
    }

    #[test]
    fn replay_authority_classifier_catches_wrong_projection_mismatches() {
        let err = anyhow::anyhow!("world replay requires a WSL platform bootstrap mapping");

        assert!(is_replay_authority_failure(&err));
    }

    #[test]
    fn replay_authority_classifier_catches_request_project_path_failures() {
        let err = anyhow::anyhow!(
            "Windows world replay requires an explicit project path from the replay request"
        );

        assert!(is_replay_authority_failure(&err));
    }

    #[test]
    fn ordinary_replay_failures_do_not_trip_authority_classifier() {
        let err = anyhow::anyhow!("trace file missing");

        assert!(!is_replay_authority_failure(&err));
    }

    #[test]
    fn fresh_world_promotes_library_replay_target_origin_to_world() {
        let mut state = ExecutionState {
            raw_cmd: "echo hi".to_string(),
            command: "echo".to_string(),
            args: vec!["hi".to_string()],
            cwd: PathBuf::from("/tmp"),
            env: HashMap::new(),
            stdin: None,
            session_id: "session".to_string(),
            span_id: "span".to_string(),
            recorded_origin: ExecutionOrigin::Host,
            recorded_origin_source: Some("span".to_string()),
            recorded_transport: Some(TransportMeta {
                mode: "unix".to_string(),
                endpoint: Some("/tmp/substrate.sock".to_string()),
                socket_activation: None,
            }),
            target_origin: ExecutionOrigin::Host,
            origin_reason: None,
            origin_reason_code: None,
            world_disable_source: None,
            platform_bootstrap_mapping: None,
        };

        apply_replay_config_to_state(&mut state, &ReplayConfig::default());

        assert_eq!(state.target_origin, ExecutionOrigin::World);
    }
}
