use std::fmt;
use std::io;
use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;

use crate::identity::{
    validate_identity_tuple_and_placement_posture, IdentityTuple, PlacementExecution,
    PlacementPosture,
};

pub const AGENT_EVENT_CHANNEL_MAX_BYTES: usize = 64;
const PURE_AGENT_ROUTER: &str = "agent_hub";
const PURE_AGENT_PROTOCOL: &str = "uaa.agent.session";

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

/// Non-alert event kinds that use the shared `{ "message": ... }` payload shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageEventKind {
    Registered,
    Status,
    TaskStart,
    TaskProgress,
    TaskEnd,
}

impl From<MessageEventKind> for AgentEventKind {
    fn from(kind: MessageEventKind) -> Self {
        match kind {
            MessageEventKind::Registered => AgentEventKind::Registered,
            MessageEventKind::Status => AgentEventKind::Status,
            MessageEventKind::TaskStart => AgentEventKind::TaskStart,
            MessageEventKind::TaskProgress => AgentEventKind::TaskProgress,
            MessageEventKind::TaskEnd => AgentEventKind::TaskEnd,
        }
    }
}

/// Structured envelope for asynchronous agent updates.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "AgentEventDef")]
pub struct AgentEvent {
    pub ts: DateTime<Utc>,
    pub kind: AgentEventKind,
    pub data: serde_json::Value,

    // Attribution + correlation (required)
    pub agent_id: String,
    pub orchestration_session_id: String,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,

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
    pub world_generation: Option<u64>,
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

    // Tuple-compatible metadata (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_tuple: Option<IdentityTuple>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_posture: Option<PlacementPosture>,

    // Legacy field (v1 producers should omit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct AgentEventDef {
    ts: DateTime<Utc>,
    kind: AgentEventKind,
    data: serde_json::Value,
    agent_id: String,
    orchestration_session_id: String,
    run_id: String,
    #[serde(default)]
    parent_run_id: Option<String>,
    #[serde(default)]
    backend_id: Option<String>,
    #[serde(default)]
    thread_id: Option<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    world_id: Option<String>,
    #[serde(default)]
    world_generation: Option<u64>,
    #[serde(default)]
    cmd_id: Option<String>,
    #[serde(default)]
    span_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_sanitized_channel")]
    channel: Option<String>,
    #[serde(default)]
    client: Option<String>,
    #[serde(default)]
    router: Option<String>,
    #[serde(default)]
    protocol: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    auth_authority: Option<String>,
    #[serde(default)]
    identity_tuple: Option<IdentityTuple>,
    #[serde(default)]
    placement_posture: Option<PlacementPosture>,
    #[serde(default)]
    project: Option<String>,
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

    fn new(
        agent_id: impl Into<String>,
        orchestration_session_id: impl Into<String>,
        run_id: impl Into<String>,
        kind: AgentEventKind,
        data: serde_json::Value,
    ) -> Self {
        let mut event = Self {
            ts: Utc::now(),
            agent_id: agent_id.into(),
            kind,
            orchestration_session_id: orchestration_session_id.into(),
            run_id: run_id.into(),
            parent_run_id: None,
            data,
            backend_id: None,
            thread_id: None,
            role: None,
            world_id: None,
            world_generation: None,
            cmd_id: None,
            span_id: None,
            channel: None,
            identity_tuple: None,
            placement_posture: None,
            project: None,
        };
        let channel = event.channel.take();
        event.set_channel(channel);
        event
    }

    /// Build a message-style event with the provided payload text.
    pub fn message(
        agent_id: impl Into<String>,
        orchestration_session_id: impl Into<String>,
        run_id: impl Into<String>,
        kind: MessageEventKind,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            agent_id,
            orchestration_session_id,
            run_id,
            kind.into(),
            serde_json::json!({ "message": message.into() }),
        )
    }

    /// Build an alert event with the required schema fields.
    pub fn alert(
        agent_id: impl Into<String>,
        orchestration_session_id: impl Into<String>,
        run_id: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            agent_id,
            orchestration_session_id,
            run_id,
            AgentEventKind::Alert,
            serde_json::json!({
                "code": code.into(),
                "message": message.into(),
            }),
        )
    }

    /// Convenience helper for stdout/stderr stream chunks.
    pub fn stream_chunk(
        agent_id: impl Into<String>,
        orchestration_session_id: impl Into<String>,
        run_id: impl Into<String>,
        is_stderr: bool,
        chunk: impl Into<String>,
    ) -> Self {
        Self::new(
            agent_id,
            orchestration_session_id,
            run_id,
            AgentEventKind::PtyData,
            serde_json::json!({
                "stream": if is_stderr { "stderr" } else { "stdout" },
                "chunk": chunk.into(),
            }),
        )
    }

    pub fn validate_identity_contract(&self) -> Result<(), String> {
        validate_identity_tuple_and_placement_posture(
            self.identity_tuple.as_ref(),
            self.placement_posture.as_ref(),
        )
    }

    pub fn set_pure_agent_telemetry_identity(&mut self, client: impl Into<String>) {
        if self.identity_tuple.is_none() {
            self.identity_tuple = Some(IdentityTuple {
                client: client.into(),
                router: PURE_AGENT_ROUTER.to_string(),
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                provider: None,
                auth_authority: None,
            });
        }

        if self.placement_posture.is_none() {
            self.placement_posture = Some(PlacementPosture {
                execution: if self.world_id.is_some() {
                    PlacementExecution::InWorld
                } else {
                    PlacementExecution::HostOnly
                },
                host_to_world_bridge: None,
            });
        }
    }

    pub fn to_trace_record(&self) -> Result<serde_json::Value, serde_json::Error> {
        let mut entry = serde_json::to_value(self)?;
        let Some(obj) = entry.as_object_mut() else {
            return Err(serde_json::Error::io(io::Error::other(
                "agent event must serialize as a JSON object",
            )));
        };

        if let Some(tuple) = self.identity_tuple.as_ref() {
            obj.insert("client".to_string(), json!(tuple.client));
            obj.insert("router".to_string(), json!(tuple.router));
            obj.insert("protocol".to_string(), json!(tuple.protocol));

            match tuple.provider.as_deref() {
                Some(provider) => {
                    obj.insert("provider".to_string(), json!(provider));
                }
                None => {
                    obj.remove("provider");
                }
            }

            match tuple.auth_authority.as_deref() {
                Some(auth_authority) => {
                    obj.insert("auth_authority".to_string(), json!(auth_authority));
                }
                None => {
                    obj.remove("auth_authority");
                }
            }
        }

        Ok(entry)
    }
}

impl TryFrom<AgentEventDef> for AgentEvent {
    type Error = String;

    fn try_from(value: AgentEventDef) -> Result<Self, Self::Error> {
        let identity_tuple = value.identity_tuple.or_else(|| {
            let client = value.client?;
            let router = value.router?;
            let protocol = value.protocol?;
            Some(IdentityTuple {
                client,
                router,
                protocol,
                provider: value.provider,
                auth_authority: value.auth_authority,
            })
        });

        let event = Self {
            ts: value.ts,
            kind: value.kind,
            data: value.data,
            agent_id: value.agent_id,
            orchestration_session_id: value.orchestration_session_id,
            run_id: value.run_id,
            parent_run_id: value.parent_run_id,
            backend_id: value.backend_id,
            thread_id: value.thread_id,
            role: value.role,
            world_id: value.world_id,
            world_generation: value.world_generation,
            cmd_id: value.cmd_id,
            span_id: value.span_id,
            channel: value.channel,
            identity_tuple,
            placement_posture: value.placement_posture,
            project: value.project,
        };
        event.validate_identity_contract()?;
        Ok(event)
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
