//! Shared request/response models and error types for the Agent API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
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
    #[serde(default)]
    pub net_allowed: Vec<String>,
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
        snapshot.net_allowed = canonicalize_net_allowed(&snapshot.net_allowed);

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

pub fn canonicalize_net_allowed(entries: &[String]) -> Vec<String> {
    let mut canonical = Vec::with_capacity(entries.len());

    for raw in entries {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut normalized = trimmed.to_ascii_lowercase();
        normalized.truncate(normalized.trim_end_matches('.').len());
        if normalized.is_empty() {
            continue;
        }

        if normalized.starts_with('[') && normalized.ends_with(']') {
            let inner = &normalized[1..normalized.len() - 1];
            if matches!(inner.parse::<IpAddr>(), Ok(IpAddr::V6(_))) {
                normalized = inner.to_string();
            }
        }

        if normalized == "*" {
            return vec!["*".to_string()];
        }

        if !canonical.contains(&normalized) {
            canonical.push(normalized);
        }
    }

    canonical
}

pub fn validate_net_allowed_for_enforcement(entries: &[String]) -> Result<(), String> {
    let canonical = canonicalize_net_allowed(entries);

    for entry in &canonical {
        validate_net_allowed_entry_for_enforcement(entry)?;
    }

    Ok(())
}

fn validate_net_allowed_entry_for_enforcement(entry: &str) -> Result<(), String> {
    if entry == "*" {
        return Ok(());
    }

    if !entry.is_ascii() {
        return Err(
            "net_allowed entries must be ASCII; use punycode A-labels for IDNs".to_string(),
        );
    }

    if entry.contains("://") {
        return Err("net_allowed entries must not include URL schemes".to_string());
    }
    if entry.contains('/') {
        return Err("net_allowed entries must not include paths".to_string());
    }
    if entry.contains('?') {
        return Err("net_allowed entries must not include query strings".to_string());
    }
    if entry.contains('#') {
        return Err("net_allowed entries must not include URL fragments".to_string());
    }
    if entry.contains(['*', '[', ']']) {
        return Err("net_allowed wildcard forms other than '*' are not supported".to_string());
    }

    if entry.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    if entry.contains(':') {
        return Err(
            "net_allowed entries must be hostnames or IP literals without ports".to_string(),
        );
    }

    validate_hostname(entry)
}

fn validate_hostname(entry: &str) -> Result<(), String> {
    if entry.starts_with('.') || entry.ends_with('.') {
        return Err("net_allowed hostnames must not start or end with '.'".to_string());
    }

    for label in entry.split('.') {
        if label.is_empty() {
            return Err("net_allowed hostnames must not contain empty labels".to_string());
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err("net_allowed hostname labels must not start or end with '-'".to_string());
        }
        if !label
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        {
            return Err(
                "net_allowed hostnames may contain only ASCII letters, digits, '-' and '.'"
                    .to_string(),
            );
        }
    }

    Ok(())
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldNetworkRoutingV1 {
    pub isolate_network: bool,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
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
    pub world_network: Option<WorldNetworkRoutingV1>,
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
    #[serde(default)]
    pub process_events: Vec<WorldProcessEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_events_status: Option<ProcessEventsStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_events_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_events_dropped: Option<u64>,
}

pub type WorldProcessEvent = serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessEventsStatus {
    Ok,
    Truncated,
    Unavailable,
    Error,
}

impl ProcessEventsStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Truncated => "truncated",
            Self::Unavailable => "unavailable",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecuteCancelRequestV1 {
    pub span_id: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecuteCancelResponseV1 {
    #[serde(default = "execute_cancel_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub delivered: bool,
}

fn execute_cancel_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingDiffBucketV1 {
    pub writes: Vec<String>,
    pub mods: Vec<String>,
    pub deletes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffRecordV1 {
    #[serde(default = "pending_diff_record_v1_default_schema_version")]
    pub schema_version: u32,
    pub session_started_at: String,
    pub diff_id: String,
    pub non_pty: PendingDiffBucketV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pty: Option<PendingDiffBucketV1>,
}

fn pending_diff_record_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffClearRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub diff_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffReconcileRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub diff_id: String,
    pub discard_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffReconcileResponseV1 {
    #[serde(default = "pending_diff_reconcile_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub reconciled: bool,
    pub discarded: u32,
}

fn pending_diff_reconcile_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffClearResponseV1 {
    #[serde(default = "pending_diff_clear_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub cleared: bool,
}

fn pending_diff_clear_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsEntryTypeV1 {
    RegularFile,
    Directory,
    Symlink,
    Socket,
    Fifo,
    BlockDevice,
    CharDevice,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFsReadRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub path: String,
    #[serde(default)]
    pub include_contents: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFsReadResponseV1 {
    #[serde(default = "world_fs_read_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub path: String,
    pub entry_type: WorldFsEntryTypeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contents_b64: Option<String>,
}

fn world_fs_read_response_v1_default_schema_version() -> u32 {
    1
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
        #[serde(default)]
        process_events: Vec<WorldProcessEvent>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        process_events_status: Option<ProcessEventsStatus>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        process_events_reason: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        process_events_dropped: Option<u64>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netfilter_status: Option<WorldDoctorNetfilterStatusV1>,
    pub landlock: WorldDoctorLandlockV1,
    pub world_fs_strategy: WorldDoctorWorldFsStrategyV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldDoctorNetfilterStatusV1 {
    pub requested: bool,
    pub enabled: bool,
    pub world_netfilter_enable_present: bool,
    #[serde(default)]
    pub last_failure_reason: Option<String>,
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
            process_events: vec![serde_json::json!({
                "event_type": "world_process_start",
                "pid": 42,
            })],
            process_events_status: Some(ProcessEventsStatus::Truncated),
            process_events_reason: Some("capture_overflow".into()),
            process_events_dropped: Some(3),
        };

        let json = serde_json::to_string(&frame).expect("serialize");
        let back: ExecuteStreamFrame = serde_json::from_str(&json).expect("deserialize");

        match back {
            ExecuteStreamFrame::Exit {
                exit,
                span_id,
                scopes_used,
                fs_diff,
                process_events,
                process_events_status,
                process_events_reason,
                process_events_dropped,
            } => {
                assert_eq!(exit, 0);
                assert_eq!(span_id, "spn_test");
                assert_eq!(scopes_used, vec!["tcp:example.com:443".to_string()]);
                assert!(fs_diff.is_none());
                assert_eq!(process_events.len(), 1);
                assert_eq!(process_events_status, Some(ProcessEventsStatus::Truncated));
                assert_eq!(process_events_reason.as_deref(), Some("capture_overflow"));
                assert_eq!(process_events_dropped, Some(3));
            }
            other => panic!("unexpected frame: {:?}", other),
        }
    }

    #[test]
    fn execute_request_world_fs_mode_round_trip() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["Github.COM.".to_string()],
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
            world_network: Some(WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["github.com".to_string()],
            }),
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
        assert_eq!(
            back.policy_snapshot.net_allowed,
            vec!["Github.COM.".to_string()]
        );
        assert_eq!(
            back.world_network,
            Some(WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["github.com".to_string()],
            })
        );
    }

    #[test]
    fn execute_request_policy_snapshot_round_trip() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["github.com".to_string(), "crates.io".to_string()],
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
            world_network: None,
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
        assert_eq!(
            snapshot.net_allowed,
            vec!["github.com".to_string(), "crates.io".to_string()]
        );
    }

    #[test]
    fn execute_cancel_request_round_trip() {
        let req = ExecuteCancelRequestV1 {
            span_id: "spn_cancel".to_string(),
            sig: "INT".to_string(),
        };

        let json = serde_json::to_string(&req).expect("serialize cancel request");
        let back: ExecuteCancelRequestV1 =
            serde_json::from_str(&json).expect("deserialize cancel request");
        assert_eq!(back, req);
    }

    #[test]
    fn execute_cancel_response_defaults_schema_version() {
        let response: ExecuteCancelResponseV1 = serde_json::from_value(serde_json::json!({
            "delivered": true
        }))
        .expect("deserialize cancel response");

        assert_eq!(response.schema_version, 1);
        assert!(response.delivered);
    }

    #[test]
    fn policy_snapshot_v3_missing_net_allowed_defaults_to_empty() {
        let snapshot: PolicySnapshotV3 = serde_json::from_value(serde_json::json!({
            "schema_version": 3,
            "world_fs": {
                "host_visible": true,
                "fail_closed": { "routing": false },
                "caged_required": false,
                "discover": { "allow_list": ["."], "deny_list": [] },
                "read": { "allow_list": ["."], "deny_list": [] },
                "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
            }
        }))
        .expect("deserialize snapshot");

        assert!(snapshot.net_allowed.is_empty());
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_canonicalizes_trim_case_trailing_dot_and_dedupe() {
        let canonical = canonicalize_net_allowed(&[
            " GitHub.COM. ".to_string(),
            "github.com".to_string(),
            "CRATES.IO".to_string(),
            "".to_string(),
            "   ".to_string(),
            "crates.io.".to_string(),
        ]);

        assert_eq!(
            canonical,
            vec!["github.com".to_string(), "crates.io".to_string()]
        );
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_collapses_star_to_singleton() {
        let canonical = canonicalize_net_allowed(&[
            "github.com".to_string(),
            " * ".to_string(),
            "crates.io".to_string(),
        ]);

        assert_eq!(canonical, vec!["*".to_string()]);
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_canonicalizes_bracketed_ipv6() {
        let canonical = canonicalize_net_allowed(&[" [2001:DB8::1] ".to_string()]);
        assert_eq!(canonical, vec!["2001:db8::1".to_string()]);
    }

    #[test]
    fn policy_snapshot_v3_validate_does_not_reject_enforcement_only_net_allowed_shapes() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec![
                "*.example.com".to_string(),
                "https://example.com".to_string(),
            ],
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

        snapshot.validate().expect("snapshot validates");
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_enforcement_validator_rejects_invalid_shapes() {
        for entries in [
            vec!["*.example.com".to_string()],
            vec!["https://example.com".to_string()],
            vec!["example.com:443".to_string()],
            vec!["example.com/path".to_string()],
            vec!["example.com?query".to_string()],
            vec!["example.com#fragment".to_string()],
            vec!["bücher.example".to_string()],
        ] {
            assert!(
                validate_net_allowed_for_enforcement(&entries).is_err(),
                "expected invalid net_allowed entries to fail: {entries:?}"
            );
        }
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_enforcement_validator_accepts_punycode_and_ips() {
        validate_net_allowed_for_enforcement(&[
            "XN--BCHER-KVA.EXAMPLE.".to_string(),
            "1.2.3.4".to_string(),
            "[2001:db8::1]".to_string(),
        ])
        .expect("valid entries");
    }

    #[test]
    fn world_doctor_report_v1_schema_round_trip() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: true,
                world_netfilter_enable_present: true,
                last_failure_reason: Some(
                    "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules"
                        .to_string(),
                ),
            }),
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
        assert_eq!(back.netfilter_status, report.netfilter_status);
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
    fn world_doctor_report_v1_serializes_null_last_failure_reason_when_absent() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: false,
                world_netfilter_enable_present: false,
                last_failure_reason: None,
            }),
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

        let value = serde_json::to_value(&report).expect("serialize report");
        assert_eq!(
            value["netfilter_status"]["last_failure_reason"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn world_doctor_report_v1_serializes_exact_netfilter_status_field_names() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: false,
                world_netfilter_enable_present: false,
                last_failure_reason: None,
            }),
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

        let value = serde_json::to_value(&report).expect("serialize report");
        let netfilter_status = value["netfilter_status"]
            .as_object()
            .expect("netfilter_status should serialize as object");
        assert_eq!(netfilter_status.len(), 4);
        assert!(netfilter_status.contains_key("requested"));
        assert!(netfilter_status.contains_key("enabled"));
        assert!(netfilter_status.contains_key("world_netfilter_enable_present"));
        assert!(netfilter_status.contains_key("last_failure_reason"));
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
        assert!(report.netfilter_status.is_none());
    }

    #[test]
    fn world_doctor_report_v1_defaults_last_failure_reason_when_missing() {
        let json = r#"{
            "schema_version": 2,
            "ok": true,
            "collected_at_utc": "2026-01-08T00:00:00Z",
            "policy_snapshot_v1_supported": true,
            "policy_resolution_mode": "snapshot_v3",
            "netfilter_status": {
                "requested": true,
                "enabled": false,
                "world_netfilter_enable_present": false
            },
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
        let status = report.netfilter_status.expect("netfilter status");
        assert!(status.requested);
        assert!(!status.enabled);
        assert!(!status.world_netfilter_enable_present);
        assert!(status.last_failure_reason.is_none());
    }
}
