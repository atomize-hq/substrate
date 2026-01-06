//! World backend abstraction for cross-platform execution environments.
//!
//! This crate provides the foundational traits and types for world backends,
//! allowing the same policy enforcement logic to work across Linux, macOS, and
//! future platforms like Windows.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for a world execution environment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSpec {
    /// Whether to reuse an existing session world.
    pub reuse_session: bool,
    /// Whether to isolate network access.
    pub isolate_network: bool,
    /// Resource limits for the world.
    pub limits: ResourceLimits,
    /// Whether to enable LD_PRELOAD telemetry.
    pub enable_preload: bool,
    /// Domains allowed for egress (to be resolved to IP sets).
    pub allowed_domains: Vec<String>,
    /// Host project directory to mount.
    pub project_dir: PathBuf,
    /// Whether to force isolation for all commands.
    pub always_isolate: bool,
    /// World filesystem mode (writable overlay/copy-diff vs read-only).
    #[serde(default)]
    pub fs_mode: WorldFsMode,
}

impl Default for WorldSpec {
    fn default() -> Self {
        Self {
            reuse_session: true,
            isolate_network: true,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: vec![
                "github.com".to_string(),
                "registry.npmjs.org".to_string(),
                "pypi.org".to_string(),
                "crates.io".to_string(),
            ],
            project_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
        }
    }
}

/// Resource limits for a world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit (e.g., "2" for 2 CPUs).
    pub cpu: Option<String>,
    /// Memory limit (e.g., "2Gi" for 2GB).
    pub memory: Option<String>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu: Some("2".to_string()),
            memory: Some("2Gi".to_string()),
        }
    }
}

/// Handle to an active world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldHandle {
    /// Unique identifier for this world instance.
    pub id: String,
}

/// Request to execute a command in a world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecRequest {
    /// Command to execute.
    pub cmd: String,
    /// Working directory for execution.
    pub cwd: PathBuf,
    /// Environment variables.
    pub env: HashMap<String, String>,
    /// Whether to use PTY mode.
    pub pty: bool,
    /// Optional span identifier to correlate fs_diff and telemetry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
}

/// Result of command execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecResult {
    /// Exit code of the command.
    pub exit: i32,
    /// Stdout bytes.
    pub stdout: Vec<u8>,
    /// Stderr bytes.
    pub stderr: Vec<u8>,
    /// Scopes that were accessed during execution.
    pub scopes_used: Vec<String>,
    /// Filesystem diff (when available, e.g., isolated runs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs_diff: Option<FsDiff>,
    /// Primary filesystem strategy attempted for world execution (ADR-0004).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_fs_strategy_primary: Option<WorldFsStrategy>,
    /// Final filesystem strategy used for world execution (ADR-0004).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_fs_strategy_final: Option<WorldFsStrategy>,
    /// Reason for falling back from the primary strategy (ADR-0004).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_fs_strategy_fallback_reason: Option<WorldFsStrategyFallbackReason>,
}

// Re-export shared types from substrate_common
pub use substrate_common::{
    FsDiff, WorldFsMode, WorldFsStrategy, WorldFsStrategyFallbackReason, WorldFsStrategyProbe,
    WorldFsStrategyProbeResult,
};

/// Backend implementations for different platforms.
pub enum Backend {
    /// Native Linux using namespaces/cgroups/nftables.
    LinuxLocal,
    /// macOS using Lima VM with Linux inside.
    MacLima,
    /// Optional Docker fallback for Linux.
    LinuxDocker,
    /// Optional high-isolation ephemeral (Linux-only).
    LinuxFirecracker,
    /// Deferred Windows WSL2 backend.
    WindowsWSL2,
}

impl Default for Backend {
    fn default() -> Self {
        #[cfg(target_os = "linux")]
        return Self::LinuxLocal;

        #[cfg(target_os = "macos")]
        return Self::MacLima;

        #[cfg(target_os = "windows")]
        return Self::WindowsWSL2;

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return Self::LinuxLocal;
    }
}

/// Trait for world backend implementations.
///
/// Note: WorldBackend is typically invoked from async contexts. Use
/// `tokio::task::spawn_blocking` for heavy/synchronous operations inside services.
pub trait WorldBackend: Send + Sync {
    /// Ensure a session world exists and return a handle to it.
    fn ensure_session(&self, spec: &WorldSpec) -> Result<WorldHandle>;

    /// Execute a command in the specified world.
    fn exec(&self, world: &WorldHandle, req: ExecRequest) -> Result<ExecResult>;

    /// Compute filesystem differences for a span.
    fn fs_diff(&self, world: &WorldHandle, span_id: &str) -> Result<FsDiff>;

    /// Apply policy restrictions to a world.
    fn apply_policy(&self, world: &WorldHandle, spec: &WorldSpec) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_spec_default() {
        let spec = WorldSpec::default();
        assert!(spec.reuse_session);
        assert!(spec.isolate_network);
        assert!(!spec.enable_preload);
        assert!(spec.allowed_domains.contains(&"github.com".to_string()));
        assert!(!spec.always_isolate);
        assert_eq!(spec.fs_mode, WorldFsMode::Writable);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.cpu, Some("2".to_string()));
        assert_eq!(limits.memory, Some("2Gi".to_string()));
    }

    #[test]
    fn test_backend_selection() {
        let backend = Backend::default();
        match backend {
            #[cfg(target_os = "linux")]
            Backend::LinuxLocal => (),
            #[cfg(target_os = "macos")]
            Backend::MacLima => (),
            #[cfg(target_os = "windows")]
            Backend::WindowsWSL2 => (),
            _ => panic!("Unexpected default backend"),
        }
    }
}
