//! Shared request/response models and error types for the Agent API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use substrate_common::agent_events::AgentEvent;
pub use substrate_common::{FsDiff, WorldFsMode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicySnapshotWorldFsIsolationV2 {
    Workspace,
    Full,
}

fn default_true() -> bool {
    true
}

fn default_allow_list_dot() -> Vec<String> {
    vec![".".to_string()]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsEnforcementV2 {
    Strict,
    BestEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsDimensionV2 {
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsV2 {
    pub mode: WorldFsMode,
    pub isolation: PolicySnapshotWorldFsIsolationV2,
    pub require_world: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<WorldFsEnforcementV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discover: Option<PolicySnapshotWorldFsDimensionV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<PolicySnapshotWorldFsDimensionV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write: Option<PolicySnapshotWorldFsDimensionV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotLimitsV2 {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u32,
    pub max_runtime_ms: u64,
    pub max_egress_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotV2 {
    pub schema_version: u32,
    pub world_fs: PolicySnapshotWorldFsV2,
    pub net_allowed: Vec<String>,
    pub limits: PolicySnapshotLimitsV2,
}

impl PolicySnapshotV2 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 2 {
            return Err(format!(
                "unsupported policy_snapshot.schema_version: {} (expected 2)",
                self.schema_version
            ));
        }
        validate_world_fs_snapshot(&self.world_fs)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsDenyEnforcementV3 {
    Strict,
    PreferStrict,
    Weak,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsFailClosedV3 {
    #[serde(default)]
    pub routing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsDimensionV3 {
    #[serde(default = "default_allow_list_dot")]
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsWriteV3 {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_allow_list_dot")]
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

impl Default for PolicySnapshotWorldFsWriteV3 {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            allow_list: default_allow_list_dot(),
            deny_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsV3 {
    #[serde(default = "default_true")]
    pub host_visible: bool,
    #[serde(default)]
    pub fail_closed: PolicySnapshotWorldFsFailClosedV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deny_enforcement: Option<WorldFsDenyEnforcementV3>,
    #[serde(default)]
    pub caged_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discover: Option<PolicySnapshotWorldFsDimensionV3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<PolicySnapshotWorldFsDimensionV3>,
    #[serde(default)]
    pub write: PolicySnapshotWorldFsWriteV3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotV3 {
    pub schema_version: u32,
    pub world_fs: PolicySnapshotWorldFsV3,
}

impl PolicySnapshotV3 {
    pub fn canonicalize(&self) -> Result<Self, String> {
        if self.schema_version != 3 {
            return Err(format!(
                "unsupported policy_snapshot.schema_version: {} (expected 3)",
                self.schema_version
            ));
        }

        let mut snapshot = self.clone();

        if snapshot.world_fs.read.is_none() {
            snapshot.world_fs.read = Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: default_allow_list_dot(),
                deny_list: Vec::new(),
            });
        }

        let read_clone =
            snapshot
                .world_fs
                .read
                .clone()
                .unwrap_or(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: default_allow_list_dot(),
                    deny_list: Vec::new(),
                });

        if snapshot.world_fs.discover.is_none() {
            snapshot.world_fs.discover = Some(read_clone.clone());
        }

        if snapshot.world_fs.write.allow_list.is_empty() {
            snapshot.world_fs.write.allow_list = default_allow_list_dot();
        }

        normalize_and_validate_world_fs_snapshot_v3(&mut snapshot.world_fs)?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> Result<(), String> {
        let _ = self.canonicalize()?;
        Ok(())
    }
}

fn normalize_project_pattern(raw: &str) -> Result<String, String> {
    let mut pattern = raw.trim();
    if pattern.is_empty() {
        return Err("pattern must be non-empty".to_string());
    }
    if pattern.starts_with('/') {
        return Err("absolute paths are not allowed".to_string());
    }

    while let Some(stripped) = pattern.strip_prefix("./") {
        pattern = stripped;
    }

    let mut normalized = pattern.trim_end_matches('/').to_string();
    if normalized.is_empty() {
        normalized = ".".to_string();
    }

    if normalized.split('/').any(|segment| segment == "..") {
        return Err("path segments must not be '..'".to_string());
    }

    Ok(normalized)
}

fn contains_any_glob_metacharacters(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[') || value.contains(']')
}

fn contains_unsupported_deny_metacharacters(value: &str) -> bool {
    value.contains('?') || value.contains('[') || value.contains(']')
}

fn validate_deny_wildcards(pattern: &str) -> Result<(), String> {
    let mut run = 0usize;
    for ch in pattern.chars() {
        if ch == '*' {
            run += 1;
            continue;
        }
        if run > 0 && run != 1 && run != 2 {
            return Err("deny_list wildcard runs must be '*' or '**' (no '***' or longer)".into());
        }
        run = 0;
    }
    if run > 0 && run != 1 && run != 2 {
        return Err("deny_list wildcard runs must be '*' or '**' (no '***' or longer)".into());
    }
    Ok(())
}

fn validate_dimension(prefix: &str, dim: &PolicySnapshotWorldFsDimensionV2) -> Result<(), String> {
    if dim.allow_list.is_empty() {
        return Err(format!("{prefix}.allow_list must be non-empty"));
    }

    for raw in &dim.allow_list {
        let normalized =
            normalize_project_pattern(raw).map_err(|e| format!("{prefix}.allow_list: {e}"))?;
        if contains_any_glob_metacharacters(&normalized) {
            return Err(format!(
                "{prefix}.allow_list contains glob metacharacters; wildcards are not supported in allow_list"
            ));
        }
    }

    for raw in &dim.deny_list {
        let normalized =
            normalize_project_pattern(raw).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        if contains_unsupported_deny_metacharacters(&normalized) {
            return Err(format!(
                "{prefix}.deny_list contains unsupported glob metacharacters ('?' or character classes)"
            ));
        }
        validate_deny_wildcards(&normalized).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
    }

    Ok(())
}

fn validate_world_fs_snapshot(world_fs: &PolicySnapshotWorldFsV2) -> Result<(), String> {
    match world_fs.isolation {
        PolicySnapshotWorldFsIsolationV2::Workspace => {
            if world_fs.enforcement.is_some() {
                return Err(
                    "world_fs.enforcement must be omitted when world_fs.isolation=workspace"
                        .to_string(),
                );
            }
            if world_fs.discover.is_some() {
                return Err(
                    "world_fs.discover must be omitted when world_fs.isolation=workspace"
                        .to_string(),
                );
            }
            if world_fs.read.is_some() {
                return Err(
                    "world_fs.read must be omitted when world_fs.isolation=workspace".to_string(),
                );
            }
            if world_fs.write.is_some() {
                return Err(
                    "world_fs.write must be omitted when world_fs.isolation=workspace".to_string(),
                );
            }
            Ok(())
        }
        PolicySnapshotWorldFsIsolationV2::Full => {
            let read = world_fs.read.as_ref().ok_or_else(|| {
                "world_fs.read must be present when world_fs.isolation=full".to_string()
            })?;
            validate_dimension("world_fs.read", read)?;

            if let Some(discover) = world_fs.discover.as_ref() {
                validate_dimension("world_fs.discover", discover)?;
            }

            match world_fs.mode {
                WorldFsMode::ReadOnly => {
                    if world_fs.write.is_some() {
                        return Err(
                            "world_fs.write must be omitted when world_fs.mode=read_only"
                                .to_string(),
                        );
                    }
                }
                WorldFsMode::Writable => {
                    let write = world_fs.write.as_ref().ok_or_else(|| {
                        "world_fs.write must be present when world_fs.mode=writable".to_string()
                    })?;
                    validate_dimension("world_fs.write", write)?;
                }
            }

            let any_deny = world_fs
                .read
                .as_ref()
                .is_some_and(|d| !d.deny_list.is_empty())
                || world_fs
                    .discover
                    .as_ref()
                    .is_some_and(|d| !d.deny_list.is_empty())
                || world_fs
                    .write
                    .as_ref()
                    .is_some_and(|d| !d.deny_list.is_empty());

            if any_deny {
                if world_fs.enforcement.is_none() {
                    return Err(
                        "world_fs.enforcement must be present when any deny_list is non-empty"
                            .to_string(),
                    );
                }
                if !world_fs.require_world {
                    return Err("deny_list requires world_fs.require_world=true".to_string());
                }
            } else if world_fs.enforcement.is_some() {
                return Err(
                    "world_fs.enforcement is only valid when at least one deny_list is non-empty"
                        .to_string(),
                );
            }

            Ok(())
        }
    }
}

fn normalize_project_pattern_v3(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("pattern must not be empty".to_string());
    }

    if raw.starts_with('/') {
        return Err("absolute patterns are invalid".to_string());
    }

    let mut segments: Vec<&str> = Vec::new();
    for seg in raw.split('/') {
        let seg = seg.trim();
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err("pattern must not contain '..' segments".to_string());
        }
        segments.push(seg);
    }

    if segments.is_empty() {
        Ok(".".to_string())
    } else {
        Ok(segments.join("/"))
    }
}

fn validate_deny_wildcards_v3(pattern: &str) -> Result<(), String> {
    let mut run = 0usize;
    for ch in pattern.chars() {
        if ch == '*' {
            run += 1;
            if run > 2 {
                return Err(
                    "deny_list wildcard runs must be '*' or '**' (no '***' or longer)".to_string(),
                );
            }
        } else {
            run = 0;
        }
    }
    Ok(())
}

fn validate_dimension_v3(
    prefix: &str,
    dim: &mut PolicySnapshotWorldFsDimensionV3,
) -> Result<(), String> {
    if dim.allow_list.is_empty() {
        return Err(format!("{prefix}.allow_list must be non-empty"));
    }

    let mut allow_out = Vec::with_capacity(dim.allow_list.len());
    for raw in &dim.allow_list {
        let normalized =
            normalize_project_pattern_v3(raw).map_err(|e| format!("{prefix}.allow_list: {e}"))?;
        if normalized.contains(['*', '?', '[', ']']) {
            return Err(format!(
                "{prefix}.allow_list contains glob metacharacters; wildcards are not supported in allow_list"
            ));
        }
        allow_out.push(normalized);
    }
    dim.allow_list = allow_out;

    let mut deny_out = Vec::with_capacity(dim.deny_list.len());
    for raw in &dim.deny_list {
        let normalized =
            normalize_project_pattern_v3(raw).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        if normalized.contains(['?', '[', ']']) {
            return Err(format!(
                "{prefix}.deny_list contains unsupported glob metacharacters ('?' or character classes)"
            ));
        }
        validate_deny_wildcards_v3(&normalized).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        deny_out.push(normalized);
    }
    dim.deny_list = deny_out;

    Ok(())
}

fn normalize_and_validate_world_fs_snapshot_v3(
    world_fs: &mut PolicySnapshotWorldFsV3,
) -> Result<(), String> {
    if !world_fs.write.enabled && !world_fs.fail_closed.routing {
        return Err(
            "world_fs.write.enabled=false requires world_fs.fail_closed.routing=true".to_string(),
        );
    }

    let mut read = world_fs
        .read
        .clone()
        .unwrap_or(PolicySnapshotWorldFsDimensionV3 {
            allow_list: default_allow_list_dot(),
            deny_list: Vec::new(),
        });
    validate_dimension_v3("world_fs.read", &mut read)?;
    world_fs.read = Some(read.clone());

    let mut discover = world_fs.discover.clone().unwrap_or_else(|| read.clone());
    validate_dimension_v3("world_fs.discover", &mut discover)?;
    world_fs.discover = Some(discover.clone());

    let mut write_dim = PolicySnapshotWorldFsDimensionV3 {
        allow_list: world_fs.write.allow_list.clone(),
        deny_list: world_fs.write.deny_list.clone(),
    };
    validate_dimension_v3("world_fs.write", &mut write_dim)?;
    world_fs.write.allow_list = write_dim.allow_list;
    world_fs.write.deny_list = write_dim.deny_list;

    let any_deny = !read.deny_list.is_empty()
        || !discover.deny_list.is_empty()
        || !world_fs.write.deny_list.is_empty();

    if world_fs.host_visible && any_deny {
        return Err("deny_list usage requires world_fs.host_visible=false".to_string());
    }

    if any_deny && world_fs.deny_enforcement.is_none() {
        return Err(
            "world_fs.deny_enforcement must be present when any deny_list is non-empty".to_string(),
        );
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub max_execs: Option<u32>,
    pub max_runtime_ms: Option<u64>,
    pub max_egress_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub profile: Option<String>,
    pub cmd: String,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub pty: bool,
    pub agent_id: String,
    pub budget: Option<Budget>,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_fs_mode: Option<WorldFsMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub exit: i32,
    pub span_id: String,
    pub stdout_b64: String,
    pub stderr_b64: String,
    pub scopes_used: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs_diff: Option<FsDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingDiffBucketV1 {
    pub writes: Vec<String>,
    pub mods: Vec<String>,
    pub deletes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffRecordV1 {
    pub schema_version: u32,
    pub session_started_at: String,
    pub diff_id: String,
    pub non_pty: PendingDiffBucketV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pty: Option<PendingDiffBucketV1>,
}

/// Streaming frame describing incremental execution output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecuteStreamFrame {
    /// Initial handshake announcing the span identifier for this execution.
    Start { span_id: String },
    /// Incremental stdout data (base64 encoded for transport safety).
    Stdout { chunk_b64: String },
    /// Incremental stderr data (base64 encoded for transport safety).
    Stderr { chunk_b64: String },
    /// Optional higher-level agent event forwarded from the world.
    Event { event: AgentEvent },
    /// Terminal frame with exit metadata and optional filesystem diff.
    Exit {
        exit: i32,
        span_id: String,
        scopes_used: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fs_diff: Option<FsDiff>,
    },
    /// Error reported while attempting to execute the command.
    Error { message: String },
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("bad_request: {0}")]
    BadRequest(String),
    #[error("not_found: {0}")]
    NotFound(String),
    #[error("rate_limited: {0}")]
    RateLimited(String),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Agent-reported world enforcement readiness (world scope).
///
/// This response is produced by `GET /v1/doctor/world`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorReportV1 {
    pub schema_version: u32,
    pub ok: bool,
    pub collected_at_utc: String,
    /// Whether the connected world-agent supports ingesting `PolicySnapshotV1` on execution requests.
    #[serde(default)]
    pub policy_snapshot_v1_supported: bool,
    /// The policy resolution mode most recently used by the world-agent (when known).
    #[serde(default)]
    pub policy_resolution_mode: Option<PolicyResolutionModeV1>,
    pub landlock: WorldDoctorLandlockV1,
    pub world_fs_strategy: WorldDoctorWorldFsStrategyV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyResolutionModeV1 {
    SnapshotV3,
    LegacyLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorLandlockV1 {
    pub supported: bool,
    pub abi: Option<u32>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorWorldFsStrategyV1 {
    pub primary: WorldDoctorWorldFsStrategyKindV1,
    pub fallback: WorldDoctorWorldFsStrategyKindV1,
    pub probe: WorldDoctorWorldFsStrategyProbeV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldDoctorWorldFsStrategyKindV1 {
    Overlay,
    Fuse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorWorldFsStrategyProbeV1 {
    pub id: String,
    pub probe_file: String,
    pub result: WorldDoctorWorldFsStrategyProbeResultV1,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldDoctorWorldFsStrategyProbeResultV1 {
    Pass,
    Fail,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_stream_frame_roundtrip() {
        let frame = ExecuteStreamFrame::Exit {
            exit: 0,
            span_id: "spn_test".into(),
            scopes_used: vec!["tcp:example.com:443".into()],
            fs_diff: None,
        };

        let json = serde_json::to_string(&frame).expect("serialize");
        let back: ExecuteStreamFrame = serde_json::from_str(&json).expect("deserialize");

        match back {
            ExecuteStreamFrame::Exit {
                exit,
                span_id,
                scopes_used,
                fs_diff,
            } => {
                assert_eq!(exit, 0);
                assert_eq!(span_id, "spn_test");
                assert_eq!(scopes_used, vec!["tcp:example.com:443".to_string()]);
                assert!(fs_diff.is_none());
            }
            other => panic!("unexpected frame: {:?}", other),
        }
    }

    #[test]
    fn execute_request_world_fs_mode_round_trip() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let req = ExecuteRequest {
            profile: None,
            cmd: "echo hi".into(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: snapshot,
            world_fs_mode: Some(WorldFsMode::ReadOnly),
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(
            json.contains("read_only"),
            "expected world_fs_mode to serialize"
        );
        assert!(
            json.contains("policy_snapshot"),
            "expected policy_snapshot to serialize"
        );
        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        assert_eq!(back.world_fs_mode, Some(WorldFsMode::ReadOnly));
        assert_eq!(back.policy_snapshot.schema_version, 3);
    }

    #[test]
    fn execute_request_policy_snapshot_round_trip() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: Some(WorldFsDenyEnforcementV3::Strict),
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                },
            },
        };

        let req = ExecuteRequest {
            profile: None,
            cmd: "echo hi".into(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: snapshot,
            world_fs_mode: None,
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(
            json.contains("policy_snapshot"),
            "expected policy_snapshot to serialize"
        );
        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        let snapshot = back.policy_snapshot;
        assert_eq!(snapshot.schema_version, 3);
        assert!(!snapshot.world_fs.host_visible);
        assert_eq!(
            snapshot.world_fs.deny_enforcement,
            Some(WorldFsDenyEnforcementV3::Strict)
        );
        assert_eq!(
            snapshot.world_fs.read.as_ref().expect("read").allow_list,
            vec!["src".to_string()]
        );
    }

    #[test]
    fn world_doctor_report_v1_schema_round_trip() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            landlock: super::WorldDoctorLandlockV1 {
                supported: true,
                abi: Some(3),
                reason: None,
            },
            world_fs_strategy: super::WorldDoctorWorldFsStrategyV1 {
                primary: super::WorldDoctorWorldFsStrategyKindV1::Overlay,
                fallback: super::WorldDoctorWorldFsStrategyKindV1::Fuse,
                probe: super::WorldDoctorWorldFsStrategyProbeV1 {
                    id: "enumeration_v1".to_string(),
                    probe_file: ".substrate_enum_probe".to_string(),
                    result: super::WorldDoctorWorldFsStrategyProbeResultV1::Pass,
                    failure_reason: None,
                },
            },
        };

        let json = serde_json::to_string(&report).expect("serialize report");
        let back: super::WorldDoctorReportV1 =
            serde_json::from_str(&json).expect("deserialize report");
        assert_eq!(back.schema_version, report.schema_version);
        assert_eq!(back.ok, report.ok);
        assert_eq!(back.collected_at_utc, report.collected_at_utc);
        assert_eq!(
            back.policy_snapshot_v1_supported,
            report.policy_snapshot_v1_supported
        );
        assert_eq!(back.policy_resolution_mode, report.policy_resolution_mode);
        assert_eq!(back.landlock.supported, report.landlock.supported);
        assert_eq!(back.landlock.abi, report.landlock.abi);
        assert_eq!(back.landlock.reason, report.landlock.reason);
        assert_eq!(
            back.world_fs_strategy.primary,
            report.world_fs_strategy.primary
        );
        assert_eq!(
            back.world_fs_strategy.fallback,
            report.world_fs_strategy.fallback
        );
        assert_eq!(
            back.world_fs_strategy.probe.id,
            report.world_fs_strategy.probe.id
        );
        assert_eq!(
            back.world_fs_strategy.probe.probe_file,
            report.world_fs_strategy.probe.probe_file
        );
        assert_eq!(
            back.world_fs_strategy.probe.result,
            report.world_fs_strategy.probe.result
        );
        assert_eq!(
            back.world_fs_strategy.probe.failure_reason,
            report.world_fs_strategy.probe.failure_reason
        );
    }

    #[test]
    fn world_doctor_report_v1_defaults_snapshot_fields_when_missing() {
        // Legacy world-agents may omit snapshot fields; the client schema must default safely.
        let json = r#"{
            "schema_version": 1,
            "ok": true,
            "collected_at_utc": "2026-01-08T00:00:00Z",
            "landlock": { "supported": true, "abi": 3, "reason": null },
            "world_fs_strategy": {
                "primary": "overlay",
                "fallback": "fuse",
                "probe": {
                    "id": "enumeration_v1",
                    "probe_file": ".substrate_enum_probe",
                    "result": "pass",
                    "failure_reason": null
                }
            }
        }"#;

        let report: super::WorldDoctorReportV1 = serde_json::from_str(json).expect("deserialize");
        assert!(report.ok);
        assert!(!report.policy_snapshot_v1_supported);
        assert!(report.policy_resolution_mode.is_none());
    }
}
