//! Shared utilities for substrate components

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod agent_events;
pub mod fs_diff;
pub mod manager_manifest;
pub mod paths;
pub mod settings;
pub mod world_exec_guard;

pub use agent_events::{AgentEvent, AgentEventKind};
pub use fs_diff::FsDiff;
pub use manager_manifest::{
    DetectSpec, GuestSpec, InitSpec, InstallClass, InstallSpec, ManagerManifest, ManagerSpec,
    Platform, RegexPattern, SystemPackagesSpec, MANAGER_MANIFEST_VERSION,
};
pub use settings::{WorldFsMode, WorldRootMode};
pub use settings::{
    WorldFsStrategy, WorldFsStrategyFallbackReason, WorldFsStrategyProbe,
    WorldFsStrategyProbeResult,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessEventType {
    WorldProcessStart,
    WorldProcessExit,
}

impl ProcessEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WorldProcessStart => "world_process_start",
            Self::WorldProcessExit => "world_process_exit",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessEvent {
    pub event_type: ProcessEventType,
    pub ts: String,
    pub ts_unix_ns: u64,
    pub session_id: String,
    pub world_id: String,
    pub pid: u32,
    pub ppid: u32,
    pub cwd: String,
    pub parent_span: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_cmd_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub argv: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub argv_omitted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exe: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessEventsStatus {
    Ok,
    Unavailable,
    Truncated,
    Error,
}

impl ProcessEventsStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Unavailable => "unavailable",
            Self::Truncated => "truncated",
            Self::Error => "error",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "unavailable" => Some(Self::Unavailable),
            "truncated" => Some(Self::Truncated),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessTelemetry {
    #[serde(default)]
    pub process_events: Vec<ProcessEvent>,
    pub process_events_status: ProcessEventsStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_events_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_events_dropped: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_events_max: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_events_backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_events_error: Option<String>,
}

impl ProcessTelemetry {
    pub fn ok(process_events: Vec<ProcessEvent>) -> Self {
        Self {
            process_events,
            process_events_status: ProcessEventsStatus::Ok,
            process_events_reason: None,
            process_events_dropped: None,
            process_events_max: None,
            process_events_backend: None,
            process_events_error: None,
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            process_events: Vec::new(),
            process_events_status: ProcessEventsStatus::Unavailable,
            process_events_reason: Some(reason.into()),
            process_events_dropped: None,
            process_events_max: None,
            process_events_backend: None,
            process_events_error: None,
        }
    }

    pub fn backend_disabled() -> Self {
        Self::unavailable("backend_disabled")
    }

    pub fn not_supported_platform() -> Self {
        Self::unavailable("not_supported_platform")
    }
}

impl Default for ProcessTelemetry {
    fn default() -> Self {
        Self::backend_disabled()
    }
}

/// Convenience re-exports for consumers that need the common substrate types.
///
/// ```
/// use substrate_common::prelude::*;
///
/// let mut diff = FsDiff::default();
/// assert!(diff.is_empty());
///
/// let redacted = redact_sensitive("token=secret");
/// assert_eq!(redacted, "token=***");
///
/// let sample = if cfg!(windows) { r"C:\\bin;C:\\bin" } else { "/bin:/bin" };
/// let deduped = dedupe_path(sample);
/// assert_eq!(deduped, dedupe_path(&deduped));
/// ```
pub mod prelude {
    pub use crate::agent_events::{AgentEvent, AgentEventKind};
    pub use crate::fs_diff::FsDiff;
    pub use crate::log_schema;
    pub use crate::manager_manifest::{
        DetectSpec, GuestSpec, InitSpec, InstallClass, InstallSpec, ManagerManifest, ManagerSpec,
        Platform, RegexPattern, SystemPackagesSpec, MANAGER_MANIFEST_VERSION,
    };
    pub use crate::paths;
    pub use crate::settings::{WorldFsMode, WorldRootMode};
    pub use crate::settings::{
        WorldFsStrategy, WorldFsStrategyFallbackReason, WorldFsStrategyProbe,
        WorldFsStrategyProbeResult,
    };
    pub use crate::{dedupe_path, redact_sensitive};
    pub use crate::{ProcessEvent, ProcessEventType, ProcessEventsStatus, ProcessTelemetry};
}

/// Deduplicate PATH-like strings while preserving order
pub fn dedupe_path(path: &str) -> String {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for component in path.split(separator) {
        if !component.is_empty() {
            let canonical = component.trim_end_matches('/').trim_end_matches('\\');
            if seen.insert(canonical.to_string()) {
                deduped.push(component);
            }
        }
    }

    deduped.join(&separator.to_string())
}

/// Standard log schema constants
pub mod log_schema {
    pub const EVENT_TYPE: &str = "event_type";
    pub const SESSION_ID: &str = "session_id";
    pub const COMMAND_ID: &str = "cmd_id";
    pub const TIMESTAMP: &str = "ts";
    pub const COMPONENT: &str = "component";
    pub const EXIT_CODE: &str = "exit_code";
    pub const DURATION_MS: &str = "duration_ms";
    pub const PROCESS_EVENTS_STATUS: &str = "process_events_status";
    pub const PROCESS_EVENTS_REASON: &str = "process_events_reason";
    pub const PROCESS_EVENTS_DROPPED: &str = "process_events_dropped";
}

/// Redact sensitive information from command arguments
pub fn redact_sensitive(arg: &str) -> String {
    if std::env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        return arg.to_string();
    }

    // Token/password patterns
    if arg.contains("token=") || arg.contains("password=") || arg.contains("SECRET=") {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        if parts.len() == 2 {
            return format!("{}=***", parts[0]);
        }
    }

    // Flag-based redaction
    match arg {
        "--token" | "--password" | "-p" | "-H" | "--header" => "***".to_string(),
        _ => arg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::string::string_regex;

    #[test]
    fn test_dedupe_path() {
        if cfg!(windows) {
            let path = r"C:\bin;C:\Windows;C:\bin;C:\Tools;C:\Windows";
            let result = dedupe_path(path);
            assert_eq!(result, r"C:\bin;C:\Windows;C:\Tools");
        } else {
            let path = "/usr/bin:/bin:/usr/bin:/usr/local/bin:/bin";
            let result = dedupe_path(path);
            assert_eq!(result, "/usr/bin:/bin:/usr/local/bin");
        }
    }

    #[test]
    fn test_redact_sensitive() {
        assert_eq!(redact_sensitive("normal_arg"), "normal_arg");
        assert_eq!(redact_sensitive("token=secret123"), "token=***");
        assert_eq!(redact_sensitive("--password"), "***");
    }

    proptest! {
        #[test]
        fn dedupe_path_is_idempotent(segments in proptest::collection::vec(
            string_regex(r"[A-Za-z0-9_./\\:-]{1,12}").unwrap(),
            1..6
        )) {
            let separator = if cfg!(windows) { ";" } else { ":" };
            let path = segments.join(separator);
            let once = dedupe_path(&path);
            let twice = dedupe_path(&once);

            prop_assert_eq!(once.clone(), twice);
            if !once.is_empty() {
                prop_assert!(once.split(separator).all(|part| !part.is_empty()));
            }
        }
    }
}
