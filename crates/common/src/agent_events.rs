use std::fmt;
use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

pub const AGENT_EVENT_CHANNEL_MAX_BYTES: usize = 64;

/// Canonical set of agent event categories.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentEventKind {
    Registered,
    Status,
    TaskStart,
    TaskProgress,
    TaskEnd,
    PtyData,
    Alert,
}

impl fmt::Display for AgentEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            AgentEventKind::Registered => "registered",
            AgentEventKind::Status => "status",
            AgentEventKind::TaskStart => "task_start",
            AgentEventKind::TaskProgress => "task_progress",
            AgentEventKind::TaskEnd => "task_end",
            AgentEventKind::PtyData => "pty_data",
            AgentEventKind::Alert => "alert",
        };
        f.write_str(label)
    }
}

/// Structured envelope for asynchronous agent updates.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentEvent {
    pub ts: DateTime<Utc>,
    pub kind: AgentEventKind,
    pub data: serde_json::Value,

    // Attribution + correlation (required)
    pub agent_id: String,
    pub orchestration_session_id: String,
    pub run_id: String,

    // Attribution + correlation (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmd_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,

    // Routing hint (optional; secrets-safe)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_sanitized_channel"
    )]
    pub channel: Option<String>,

    // Tuple-compatible metadata (optional; semantics delegated to later ADRs)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub router: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_authority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    // Legacy field (v1 producers should omit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

impl AgentEvent {
    pub fn sanitize_channel(raw: Option<String>) -> Option<String> {
        let value = raw?;
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed != value {
            return None;
        }
        if trimmed.len() > AGENT_EVENT_CHANNEL_MAX_BYTES {
            return None;
        }
        if !agent_event_channel_pattern().is_match(trimmed) {
            return None;
        }
        Some(trimmed.to_string())
    }

    pub fn set_channel(&mut self, raw: Option<String>) {
        self.channel = Self::sanitize_channel(raw);
    }

    /// Build a message-style event with the provided payload text.
    pub fn message(
        agent_id: impl Into<String>,
        orchestration_session_id: impl Into<String>,
        run_id: impl Into<String>,
        kind: AgentEventKind,
        message: impl Into<String>,
    ) -> Self {
        let mut event = Self {
            ts: Utc::now(),
            agent_id: agent_id.into(),
            kind,
            orchestration_session_id: orchestration_session_id.into(),
            run_id: run_id.into(),
            data: serde_json::json!({ "message": message.into() }),
            backend_id: None,
            thread_id: None,
            role: None,
            world_id: None,
            cmd_id: None,
            span_id: None,
            channel: None,
            client: None,
            router: None,
            provider: None,
            auth_authority: None,
            protocol: None,
            project: None,
        };
        // Ensure any producer-provided value is secrets-safe.
        let channel = event.channel.take();
        event.set_channel(channel);
        event
    }

    /// Convenience helper for stdout/stderr stream chunks.
    pub fn stream_chunk(
        agent_id: impl Into<String>,
        orchestration_session_id: impl Into<String>,
        run_id: impl Into<String>,
        is_stderr: bool,
        chunk: impl Into<String>,
    ) -> Self {
        let kind = AgentEventKind::PtyData;
        let mut event = Self {
            ts: Utc::now(),
            agent_id: agent_id.into(),
            kind,
            orchestration_session_id: orchestration_session_id.into(),
            run_id: run_id.into(),
            data: serde_json::json!({
                "stream": if is_stderr { "stderr" } else { "stdout" },
                "chunk": chunk.into(),
            }),
            backend_id: None,
            thread_id: None,
            role: None,
            world_id: None,
            cmd_id: None,
            span_id: None,
            channel: None,
            client: None,
            router: None,
            provider: None,
            auth_authority: None,
            protocol: None,
            project: None,
        };
        let channel = event.channel.take();
        event.set_channel(channel);
        event
    }
}

fn agent_event_channel_pattern() -> &'static Regex {
    static CHANNEL_RE: OnceLock<Regex> = OnceLock::new();
    CHANNEL_RE.get_or_init(|| {
        // Conservative, deterministic allowlist to avoid leaking secrets via channel.
        // - No whitespace, quotes, or '='
        // - ASCII-safe tokens only
        Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9._:/-]{0,63}$").expect("channel regex is valid")
    })
}

fn deserialize_sanitized_channel<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(AgentEvent::sanitize_channel(raw))
}
