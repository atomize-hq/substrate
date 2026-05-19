//! World backend abstraction for cross-platform execution environments.
//!
//! This crate provides the foundational traits and types for world backends,
//! allowing the same policy enforcement logic to work across Linux, macOS, and
//! future platforms like Windows.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Reuse semantics for a world allocation request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorldReuseMode {
    /// Use the legacy compatibility lookup rules for generic reusable worlds.
    #[default]
    GenericCompatible,
    /// Use the explicit shared-world ownership contract.
    SharedOrchestration(SharedWorldOwnerSpec),
}

impl WorldReuseMode {
    pub fn shared_owner(&self) -> Option<&SharedWorldOwnerSpec> {
        match self {
            Self::GenericCompatible => None,
            Self::SharedOrchestration(owner) => Some(owner),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SharedWorldOwnerSpec {
    pub orchestration_session_id: String,
    pub action: SharedWorldOwnerAction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SharedWorldOwnerAction {
    AttachOrCreate,
    ReplaceExpectedGeneration {
        expected_generation: u64,
        reason: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SharedWorldBindingState {
    Active,
    Replacing,
    Replaced,
    Abandoned,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SharedWorldBindingSnapshot {
    pub orchestration_session_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub binding_state: SharedWorldBindingState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRuntimeBackendKindV1 {
    Codex,
    ClaudeCode,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedMemberRuntimeDescriptorV1 {
    pub backend_kind: MemberRuntimeBackendKindV1,
    pub binary_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemberDispatchRequestV1 {
    pub schema_version: u32,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_participant_id: Option<String>,
    pub backend_id: String,
    pub protocol: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    pub resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
}

/// Configuration for a world execution environment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSpec {
    /// Whether to reuse an existing session world.
    pub reuse_session: bool,
    /// Whether to use generic compatibility reuse or explicit shared-world ownership.
    #[serde(default)]
    pub reuse_mode: WorldReuseMode,
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
            reuse_mode: WorldReuseMode::GenericCompatible,
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
    /// Authoritative shared-world binding proof when explicit owner mode is active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_binding: Option<SharedWorldBindingSnapshot>,
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
    /// Optional shared-world ownership intent for orchestration-aware backends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_world: Option<SharedWorldOwnerSpec>,
    /// Optional member dispatch payload for orchestration-aware backends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_dispatch: Option<MemberDispatchRequestV1>,
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
    /// Process telemetry returned by the world backend (ADR-0028/WPEP1).
    #[serde(flatten, default)]
    pub process_telemetry: ProcessTelemetry,
}

// Re-export shared types from substrate_common
pub use substrate_common::{
    FsDiff, ProcessEvent, ProcessEventType, ProcessEventsStatus, ProcessTelemetry, WorldFsMode,
    WorldFsStrategy, WorldFsStrategyFallbackReason, WorldFsStrategyProbe,
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
        assert_eq!(spec.reuse_mode, WorldReuseMode::GenericCompatible);
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

    #[test]
    fn shared_world_contract_round_trips_with_canonical_shape() {
        let mode = WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::ReplaceExpectedGeneration {
                expected_generation: 7,
                reason: "restart".into(),
            },
        });
        let json = serde_json::to_string(&mode).expect("serialize reuse mode");
        assert_eq!(
            json,
            r#"{"shared_orchestration":{"orchestration_session_id":"orch_123","action":{"replace_expected_generation":{"expected_generation":7,"reason":"restart"}}}}"#
        );

        let binding = SharedWorldBindingSnapshot {
            orchestration_session_id: "orch_123".into(),
            world_id: "wld_123".into(),
            world_generation: 8,
            binding_state: SharedWorldBindingState::Active,
        };
        let binding_json = serde_json::to_string(&binding).expect("serialize binding");
        assert_eq!(
            binding_json,
            r#"{"orchestration_session_id":"orch_123","world_id":"wld_123","world_generation":8,"binding_state":"active"}"#
        );

        let spec = WorldSpec {
            reuse_mode: mode.clone(),
            ..WorldSpec::default()
        };
        let decoded: WorldSpec =
            serde_json::from_str(&serde_json::to_string(&spec).expect("serialize spec"))
                .expect("deserialize spec");
        assert_eq!(decoded.reuse_mode, mode);

        let handle = WorldHandle {
            id: "wld_123".into(),
            shared_binding: Some(binding.clone()),
        };
        let handle_back: WorldHandle =
            serde_json::from_str(&serde_json::to_string(&handle).expect("serialize handle"))
                .expect("deserialize handle");
        assert_eq!(
            handle_back
                .shared_binding
                .expect("shared binding should deserialize"),
            binding
        );

        let exec_request = ExecRequest {
            cmd: "echo hi".into(),
            cwd: PathBuf::from("/tmp"),
            env: HashMap::new(),
            pty: false,
            span_id: Some("spn_123".into()),
            shared_world: Some(SharedWorldOwnerSpec {
                orchestration_session_id: "orch_123".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
            member_dispatch: Some(MemberDispatchRequestV1 {
                schema_version: 1,
                orchestration_session_id: "orch_123".into(),
                participant_id: "participant_123".into(),
                orchestrator_participant_id: "participant_root".into(),
                parent_participant_id: Some("participant_parent".into()),
                resumed_from_participant_id: None,
                backend_id: "backend_123".into(),
                protocol: "stdio".into(),
                run_id: "run_123".into(),
                world_id: "wld_123".into(),
                world_generation: 8,
                initial_prompt: Some("Continue".into()),
                resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path: "/usr/bin/env".into(),
                },
            }),
        };
        let exec_back: ExecRequest = serde_json::from_str(
            &serde_json::to_string(&exec_request).expect("serialize exec request"),
        )
        .expect("deserialize exec request");
        assert_eq!(
            exec_back
                .shared_world
                .expect("shared world should deserialize"),
            SharedWorldOwnerSpec {
                orchestration_session_id: "orch_123".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }
        );
        assert_eq!(
            exec_back
                .member_dispatch
                .expect("member dispatch should deserialize"),
            exec_request
                .member_dispatch
                .expect("original member dispatch should exist")
        );
    }
}
