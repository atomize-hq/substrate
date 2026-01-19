//! Shared request/response models and error types for the Agent API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use substrate_common::agent_events::AgentEvent;
pub use substrate_common::{FsDiff, WorldFsMode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicySnapshotWorldFsIsolationV1 {
    Workspace,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsV1 {
    pub mode: WorldFsMode,
    pub isolation: PolicySnapshotWorldFsIsolationV1,
    pub require_world: bool,
    pub read_allowlist: Vec<String>,
    pub write_allowlist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotLimitsV1 {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<u32>,
    pub max_runtime_ms: Option<u64>,
    pub max_egress_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotV1 {
    pub schema_version: u32,
    pub world_fs: PolicySnapshotWorldFsV1,
    pub net_allowed: Vec<String>,
    pub limits: PolicySnapshotLimitsV1,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_snapshot: Option<PolicySnapshotV1>,
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
    pub landlock: WorldDoctorLandlockV1,
    pub world_fs_strategy: WorldDoctorWorldFsStrategyV1,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldDoctorWorldFsStrategyProbeResultV1 {
    Pass,
    Fail,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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
        let req = ExecuteRequest {
            profile: None,
            cmd: "echo hi".into(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: None,
            world_fs_mode: Some(WorldFsMode::ReadOnly),
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(
            json.contains("read_only"),
            "expected world_fs_mode to serialize"
        );
        assert!(
            !json.contains("policy_snapshot"),
            "expected policy_snapshot to be omitted when None"
        );
        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        assert_eq!(back.world_fs_mode, Some(WorldFsMode::ReadOnly));
        assert!(back.policy_snapshot.is_none());
    }

    #[test]
    fn execute_request_policy_snapshot_round_trip() {
        let req = ExecuteRequest {
            profile: None,
            cmd: "echo hi".into(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: Some(PolicySnapshotV1 {
                schema_version: 1,
                world_fs: PolicySnapshotWorldFsV1 {
                    mode: WorldFsMode::Writable,
                    isolation: PolicySnapshotWorldFsIsolationV1::Workspace,
                    require_world: false,
                    read_allowlist: vec!["*".to_string()],
                    write_allowlist: vec!["dist/**".to_string()],
                },
                net_allowed: vec!["github.com".to_string()],
                limits: PolicySnapshotLimitsV1 {
                    max_memory_mb: Some(4096),
                    max_cpu_percent: Some(80),
                    max_runtime_ms: Some(600_000),
                    max_egress_bytes: Some(1_073_741_824),
                },
            }),
            world_fs_mode: None,
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(
            json.contains("policy_snapshot"),
            "expected policy_snapshot to serialize"
        );
        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        let snapshot = back.policy_snapshot.expect("snapshot missing after deserialize");
        assert_eq!(snapshot.schema_version, 1);
        assert_eq!(snapshot.world_fs.mode, WorldFsMode::Writable);
        assert_eq!(
            snapshot.world_fs.isolation,
            PolicySnapshotWorldFsIsolationV1::Workspace
        );
        assert_eq!(snapshot.world_fs.read_allowlist, vec!["*".to_string()]);
        assert_eq!(
            snapshot.world_fs.write_allowlist,
            vec!["dist/**".to_string()]
        );
        assert_eq!(snapshot.net_allowed, vec!["github.com".to_string()]);
        assert_eq!(snapshot.limits.max_memory_mb, Some(4096));
        assert_eq!(snapshot.limits.max_cpu_percent, Some(80));
        assert_eq!(snapshot.limits.max_runtime_ms, Some(600_000));
        assert_eq!(snapshot.limits.max_egress_bytes, Some(1_073_741_824));
    }

    #[test]
    fn world_doctor_report_v1_schema_round_trip() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct WorldDoctorReportV1 {
            schema_version: u32,
            ok: bool,
            collected_at_utc: String,
            landlock: LandlockReportV1,
            world_fs_strategy: WorldFsStrategyReportV1,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct LandlockReportV1 {
            supported: bool,
            abi: Option<u32>,
            reason: Option<String>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct WorldFsStrategyReportV1 {
            primary: String,
            fallback: String,
            probe: WorldFsStrategyProbeV1,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct WorldFsStrategyProbeV1 {
            id: String,
            probe_file: String,
            result: String,
            failure_reason: Option<String>,
        }

        let report = WorldDoctorReportV1 {
            schema_version: 1,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            landlock: LandlockReportV1 {
                supported: true,
                abi: Some(3),
                reason: None,
            },
            world_fs_strategy: WorldFsStrategyReportV1 {
                primary: "overlay".to_string(),
                fallback: "fuse".to_string(),
                probe: WorldFsStrategyProbeV1 {
                    id: "enumeration_v1".to_string(),
                    probe_file: ".substrate_enum_probe".to_string(),
                    result: "pass".to_string(),
                    failure_reason: None,
                },
            },
        };

        let json = serde_json::to_string(&report).expect("serialize report");
        let back: WorldDoctorReportV1 = serde_json::from_str(&json).expect("deserialize report");
        assert_eq!(back, report);
    }
}
