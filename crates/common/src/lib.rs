//! Shared utilities for substrate components

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod agent_events;
pub mod agent_identity;
pub mod fs_diff;
pub mod gateway_auth_bundle;
pub mod identity;
pub mod manager_manifest;
pub mod paths;
pub mod settings;
pub mod world_exec_guard;

pub use agent_events::{AgentEvent, AgentEventKind};
pub use agent_identity::derive_agent_backend_id;
pub use fs_diff::FsDiff;
pub use gateway_auth_bundle::{
    allowed_gateway_auth_fields, gateway_auth_bundle_schema_version, required_gateway_auth_fields,
    validate_gateway_auth_bundle, GatewayAuthBundleV1, API_OPENAI_GATEWAY_AUTH_ALLOWED_FIELDS,
    API_OPENAI_GATEWAY_AUTH_REQUIRED_FIELDS, CLI_CODEX_GATEWAY_AUTH_ALLOWED_FIELDS,
    CLI_CODEX_GATEWAY_AUTH_REQUIRED_FIELDS, GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI,
    GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CODEX, GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION,
    SUBSTRATE_LLM_AUTH_BUNDLE_FD, SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY,
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
};
pub use identity::{
    validate_identity_tuple_and_placement_posture, IdentityTuple, PlacementExecution,
    PlacementPosture,
};
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
    pub use crate::gateway_auth_bundle::{
        allowed_gateway_auth_fields, gateway_auth_bundle_schema_version,
        required_gateway_auth_fields, validate_gateway_auth_bundle, GatewayAuthBundleV1,
        API_OPENAI_GATEWAY_AUTH_ALLOWED_FIELDS, API_OPENAI_GATEWAY_AUTH_REQUIRED_FIELDS,
        CLI_CODEX_GATEWAY_AUTH_ALLOWED_FIELDS, CLI_CODEX_GATEWAY_AUTH_REQUIRED_FIELDS,
        GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI, GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CODEX,
        GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION, SUBSTRATE_LLM_AUTH_BUNDLE_FD,
        SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY,
        SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
        SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
    };
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

pub fn process_env_key_allowlisted(key: &str) -> bool {
    let key_upper = key.trim().to_ascii_uppercase();
    matches!(
        key_upper.as_str(),
        "PATH" | "HOME" | "USER" | "SHELL" | "LANG" | "TERM" | "NO_PROXY" | "ALL_PROXY"
    ) || key_upper.starts_with("LC_")
        || key_upper.starts_with("SHIM_")
        || key_upper.starts_with("SUBSTRATE_")
        || (key_upper.starts_with("HTTP") && key_upper.ends_with("_PROXY"))
}

pub fn truncate_utf8_bytes(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }

    let mut count = 0usize;
    let mut out = String::new();
    for ch in value.chars() {
        let ch_len = ch.len_utf8();
        if count + ch_len > max_bytes {
            break;
        }
        out.push(ch);
        count += ch_len;
    }
    out
}

fn redact_url_credentials(value: &str) -> String {
    let Some(scheme_idx) = value.find("://") else {
        return value.to_string();
    };
    let after_scheme = scheme_idx + 3;
    let Some(at_idx) = value[after_scheme..]
        .find('@')
        .map(|idx| idx + after_scheme)
    else {
        return value.to_string();
    };
    let slash_idx = value[after_scheme..]
        .find('/')
        .map(|idx| idx + after_scheme)
        .unwrap_or(value.len());
    if at_idx > slash_idx {
        return value.to_string();
    }

    let userinfo = &value[after_scheme..at_idx];
    if userinfo.is_empty() {
        return value.to_string();
    }

    format!("{}***@{}", &value[..after_scheme], &value[at_idx + 1..])
}

fn redact_bearer_tokens(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let Some(bearer_idx) = lower.find("bearer ") else {
        return value.to_string();
    };
    let token_start = bearer_idx + "bearer ".len();
    let token_end = value[token_start..]
        .find(|c: char| c.is_whitespace())
        .map(|idx| idx + token_start)
        .unwrap_or(value.len());
    if token_start >= token_end {
        return value.to_string();
    }
    format!("{}***{}", &value[..token_start], &value[token_end..])
}

fn redact_kv_like(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    for needle in [
        "token=",
        "password=",
        "secret=",
        "apikey=",
        "api_key=",
        "api-key=",
    ] {
        if let Some(start) = lower.find(needle) {
            let value_start = start + needle.len();
            let value_end = value[value_start..]
                .find(|c: char| c.is_whitespace() || c == '&' || c == ';')
                .map(|idx| idx + value_start)
                .unwrap_or(value.len());
            return format!(
                "{}{}***{}",
                &value[..start],
                &value[start..value_start],
                &value[value_end..]
            );
        }
    }
    value.to_string()
}

pub fn redact_process_env_value(_key: &str, value: &str) -> String {
    let value = redact_url_credentials(value);
    let value = redact_bearer_tokens(&value);
    redact_kv_like(&value)
}

fn is_sensitive_flag(flag: &str) -> bool {
    let flag = flag.trim();
    if !flag.starts_with('-') {
        return false;
    }

    let lower = flag.to_ascii_lowercase();
    matches!(lower.as_str(), "-p" | "--header")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("apikey")
        || lower.contains("api_key")
        || lower.contains("api-key")
        || lower.contains("auth")
        || lower.contains("bearer")
        || lower.contains("private-key")
        || lower.contains("ssh-key")
}

pub fn redact_process_argv(argv: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(argv.len());
    let mut idx = 0usize;

    while idx < argv.len() {
        let arg = &argv[idx];

        if let Some((flag, _value)) = arg.split_once('=').filter(|(k, _)| k.starts_with('-')) {
            if is_sensitive_flag(flag) {
                out.push(format!("{flag}=***"));
                idx += 1;
                continue;
            }
        }

        if arg == "-H" || arg == "--header" {
            out.push(arg.clone());
            if let Some(next) = argv.get(idx + 1) {
                out.push(redact_process_env_value("", next));
                idx += 2;
                continue;
            }
            idx += 1;
            continue;
        }

        if is_sensitive_flag(arg) {
            out.push(arg.clone());
            if idx + 1 < argv.len() {
                out.push("***".to_string());
                idx += 2;
                continue;
            }
            idx += 1;
            continue;
        }

        out.push(redact_process_env_value("", arg));
        idx += 1;
    }

    out
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

    #[test]
    fn process_env_key_allowlist_matches_adr_0028_defaults() {
        for key in [
            "PATH",
            "HOME",
            "USER",
            "SHELL",
            "LANG",
            "TERM",
            "LC_ALL",
            "LC_CTYPE",
            "SUBSTRATE_WORLD_SOCKET",
            "SHIM_SESSION_ID",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "NO_PROXY",
        ] {
            assert!(
                process_env_key_allowlisted(key),
                "expected allowlisted: {key}"
            );
        }

        for key in ["AWS_SECRET_ACCESS_KEY", "GITHUB_TOKEN", "SOME_RANDOM_VAR"] {
            assert!(
                !process_env_key_allowlisted(key),
                "expected not allowlisted: {key}"
            );
        }
    }

    #[test]
    fn truncate_utf8_bytes_preserves_utf8_boundaries() {
        let value = "αβγδε";
        let truncated = truncate_utf8_bytes(value, 5);
        assert!(value.starts_with(&truncated));
        assert!(truncated.is_char_boundary(truncated.len()));
        assert!(truncated.len() <= 5);
    }

    #[test]
    fn redact_process_env_value_redacts_url_credentials_and_bearer_tokens() {
        assert_eq!(
            redact_process_env_value("", "http://user:pass@example.com/path"),
            "http://***@example.com/path"
        );
        assert_eq!(
            redact_process_env_value("", "Authorization: Bearer abc123"),
            "Authorization: Bearer ***"
        );
        assert_eq!(redact_process_env_value("", "token=abc123"), "token=***");
    }

    #[test]
    fn redact_process_argv_redacts_flag_paired_and_equals_forms() {
        let argv = vec![
            "curl".to_string(),
            "-H".to_string(),
            "Authorization: Bearer abc123".to_string(),
            "--token".to_string(),
            "secret".to_string(),
            "--password=supersecret".to_string(),
            "http://user:pass@example.com".to_string(),
        ];
        let redacted = redact_process_argv(&argv);
        assert_eq!(redacted[2], "Authorization: Bearer ***");
        assert_eq!(redacted[4], "***");
        assert_eq!(redacted[5], "--password=***");
        assert_eq!(redacted[6], "http://***@example.com");
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
