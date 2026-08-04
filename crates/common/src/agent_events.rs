use std::fmt;
use std::io;
use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;

use crate::authority_commitment::HostTransitionWorkCorrelationV1;
use crate::identity::{
    normalize_identity_tuple_client_id, validate_identity_tuple_and_placement_posture,
    IdentityTuple, PlacementExecution, PlacementPosture,
};

pub const AGENT_EVENT_CHANNEL_MAX_BYTES: usize = 64;
const PURE_AGENT_ROUTER: &str = "agent_hub";
// Substrate-local normalized protocol-family id for current pure-agent records.
// This label is not, by itself, a claim of upstream UAA wire/API compatibility.
const PURE_AGENT_PROTOCOL: &str = "substrate.agent.session";

pub const RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1: u32 = 1;

/// Producer-assigned identity shared by every frame in one runtime stream.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RuntimeFrameIdentityV1Def")]
pub struct RuntimeFrameIdentityV1 {
    pub schema_version: u32,
    pub stream_id: String,
    pub frame_sequence: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeFrameIdentityV1Def {
    schema_version: u32,
    stream_id: String,
    frame_sequence: u64,
}

impl RuntimeFrameIdentityV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1 {
            return Err(format!(
                "unsupported runtime frame identity schema_version: {} (expected {})",
                self.schema_version, RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1
            ));
        }
        validate_runtime_identity_id("stream_id", &self.stream_id)?;
        validate_runtime_identity_sequence("frame_sequence", self.frame_sequence)
    }
}

impl TryFrom<RuntimeFrameIdentityV1Def> for RuntimeFrameIdentityV1 {
    type Error = String;

    fn try_from(value: RuntimeFrameIdentityV1Def) -> Result<Self, Self::Error> {
        let identity = Self {
            schema_version: value.schema_version,
            stream_id: value.stream_id,
            frame_sequence: value.frame_sequence,
        };
        identity.validate()?;
        Ok(identity)
    }
}

/// Producer-assigned identity for one semantic event in a runtime stream.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RuntimeEventIdentityV1Def")]
pub struct RuntimeEventIdentityV1 {
    pub event_id: String,
    pub event_sequence: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeEventIdentityV1Def {
    event_id: String,
    event_sequence: u64,
}

impl RuntimeEventIdentityV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_runtime_identity_id("event_id", &self.event_id)?;
        validate_runtime_identity_sequence("event_sequence", self.event_sequence)
    }
}

impl TryFrom<RuntimeEventIdentityV1Def> for RuntimeEventIdentityV1 {
    type Error = String;

    fn try_from(value: RuntimeEventIdentityV1Def) -> Result<Self, Self::Error> {
        let identity = Self {
            event_id: value.event_id,
            event_sequence: value.event_sequence,
        };
        identity.validate()?;
        Ok(identity)
    }
}

/// Exact semantic event identity named by the terminal runtime frame.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RuntimeTerminalIdentityV1Def")]
pub struct RuntimeTerminalIdentityV1 {
    pub terminal_event_id: String,
    pub terminal_event_sequence: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeTerminalIdentityV1Def {
    terminal_event_id: String,
    terminal_event_sequence: u64,
}

impl RuntimeTerminalIdentityV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_runtime_identity_id("terminal_event_id", &self.terminal_event_id)?;
        validate_runtime_identity_sequence("terminal_event_sequence", self.terminal_event_sequence)
    }

    pub fn matches_event(&self, event: &RuntimeEventIdentityV1) -> bool {
        self.terminal_event_id == event.event_id
            && self.terminal_event_sequence == event.event_sequence
    }
}

impl From<&RuntimeEventIdentityV1> for RuntimeTerminalIdentityV1 {
    fn from(event: &RuntimeEventIdentityV1) -> Self {
        Self {
            terminal_event_id: event.event_id.clone(),
            terminal_event_sequence: event.event_sequence,
        }
    }
}

impl TryFrom<RuntimeTerminalIdentityV1Def> for RuntimeTerminalIdentityV1 {
    type Error = String;

    fn try_from(value: RuntimeTerminalIdentityV1Def) -> Result<Self, Self::Error> {
        let identity = Self {
            terminal_event_id: value.terminal_event_id,
            terminal_event_sequence: value.terminal_event_sequence,
        };
        identity.validate()?;
        Ok(identity)
    }
}

fn validate_runtime_identity_id(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("runtime identity {field} must be non-empty"));
    }
    Ok(())
}

fn validate_runtime_identity_sequence(field: &str, value: u64) -> Result<(), String> {
    if value == 0 {
        return Err(format!("runtime identity {field} must be positive"));
    }
    Ok(())
}

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldWorkerEventClassV1 {
    Reply,
    ProgressUpdate,
    ControlAck,
    FollowUpQuestion,
    ApprovalRequest,
    Blocked,
    AttentionRequired,
    ForkRequest,
    ForkRecommendation,
    Result,
    Failure,
}

impl WorldWorkerEventClassV1 {
    pub fn attention_required_by_default(self) -> bool {
        matches!(
            self,
            Self::FollowUpQuestion
                | Self::ApprovalRequest
                | Self::Blocked
                | Self::AttentionRequired
                | Self::ForkRequest
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "NormalizedWorldWorkerEventFacetV1Def")]
pub struct NormalizedWorldWorkerEventFacetV1 {
    pub schema_version: u32,
    pub thread_id: String,
    pub event_class: WorldWorkerEventClassV1,
    pub attention_required: bool,
    pub causation_message_id: String,
    pub causation_request_id: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NormalizedWorldWorkerEventFacetV1Def {
    schema_version: u32,
    thread_id: String,
    event_class: WorldWorkerEventClassV1,
    attention_required: bool,
    causation_message_id: String,
    causation_request_id: String,
    payload: serde_json::Value,
}

impl NormalizedWorldWorkerEventFacetV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported worker_event_facet.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        validate_required_trimmed("worker_event_facet.thread_id", &self.thread_id)?;
        validate_prefixed_uuid_v7(
            "worker_event_facet.causation_message_id",
            &self.causation_message_id,
            "wwm_",
        )?;
        validate_required_trimmed(
            "worker_event_facet.causation_request_id",
            &self.causation_request_id,
        )?;
        if self.event_class.attention_required_by_default() && !self.attention_required {
            return Err(format!(
                "worker_event_facet.event_class {:?} requires attention_required=true",
                self.event_class
            ));
        }
        Ok(())
    }
}

impl TryFrom<NormalizedWorldWorkerEventFacetV1Def> for NormalizedWorldWorkerEventFacetV1 {
    type Error = String;

    fn try_from(value: NormalizedWorldWorkerEventFacetV1Def) -> Result<Self, Self::Error> {
        let facet = Self {
            schema_version: value.schema_version,
            thread_id: value.thread_id,
            event_class: value.event_class,
            attention_required: value.attention_required,
            causation_message_id: value.causation_message_id,
            causation_request_id: value.causation_request_id,
            payload: value.payload,
        };
        facet.validate()?;
        Ok(facet)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "WorldWorkerEventV1Def")]
pub struct WorldWorkerEventV1 {
    pub schema_version: u32,
    pub acceptance_record_id: String,
    pub stream_id: String,
    pub frame_sequence: u64,
    pub event_id: String,
    pub event_sequence: u64,
    pub request_id: String,
    pub active_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    pub causation_message_id: String,
    pub causation_request_id: String,
    pub orchestration_session_id: String,
    pub source_participant_id: String,
    pub target_participant_id: String,
    pub source_backend_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub thread_id: String,
    pub event_class: WorldWorkerEventClassV1,
    pub attention_required: bool,
    pub payload: serde_json::Value,
    pub emitted_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorldWorkerEventV1Def {
    schema_version: u32,
    acceptance_record_id: String,
    stream_id: String,
    frame_sequence: u64,
    event_id: String,
    event_sequence: u64,
    request_id: String,
    active_run_id: String,
    #[serde(default)]
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    causation_message_id: String,
    causation_request_id: String,
    orchestration_session_id: String,
    source_participant_id: String,
    target_participant_id: String,
    source_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    thread_id: String,
    event_class: WorldWorkerEventClassV1,
    attention_required: bool,
    payload: serde_json::Value,
    emitted_at: DateTime<Utc>,
}

impl WorldWorkerEventV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported worker_event.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        validate_prefixed_uuid_v7(
            "worker_event.acceptance_record_id",
            &self.acceptance_record_id,
            "wwa_",
        )?;
        RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: self.stream_id.clone(),
            frame_sequence: self.frame_sequence,
        }
        .validate()?;
        RuntimeEventIdentityV1 {
            event_id: self.event_id.clone(),
            event_sequence: self.event_sequence,
        }
        .validate()?;
        for (field, value) in [
            ("worker_event.request_id", self.request_id.as_str()),
            ("worker_event.active_run_id", self.active_run_id.as_str()),
            (
                "worker_event.causation_request_id",
                self.causation_request_id.as_str(),
            ),
            (
                "worker_event.orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            (
                "worker_event.source_participant_id",
                self.source_participant_id.as_str(),
            ),
            (
                "worker_event.target_participant_id",
                self.target_participant_id.as_str(),
            ),
            (
                "worker_event.source_backend_id",
                self.source_backend_id.as_str(),
            ),
            (
                "worker_event.target_backend_id",
                self.target_backend_id.as_str(),
            ),
            ("worker_event.world_id", self.world_id.as_str()),
            ("worker_event.thread_id", self.thread_id.as_str()),
        ] {
            validate_required_trimmed(field, value)?;
        }
        validate_prefixed_uuid_v7(
            "worker_event.causation_message_id",
            &self.causation_message_id,
            "wwm_",
        )?;
        if self.request_id != self.active_run_id {
            return Err(
                "worker_event.request_id must equal worker_event.active_run_id".to_string(),
            );
        }
        if self.request_id != self.causation_request_id {
            return Err(
                "worker_event.causation_request_id must equal worker_event.request_id".to_string(),
            );
        }
        if self.source_participant_id == self.target_participant_id {
            return Err(
                "worker_event.target_participant_id must not equal worker_event.source_participant_id"
                    .to_string(),
            );
        }
        if self.event_class.attention_required_by_default() && !self.attention_required {
            return Err(format!(
                "worker_event.event_class {:?} requires attention_required=true",
                self.event_class
            ));
        }
        if let Some(correlation) = self.host_transition_correlation.as_ref() {
            correlation.validate()?;
            if correlation.orchestration_session_id != self.orchestration_session_id {
                return Err(
                    "worker_event.host_transition_correlation.orchestration_session_id must match worker_event.orchestration_session_id"
                        .to_string(),
                );
            }
            if correlation.authoritative_participant_id != self.target_participant_id {
                return Err(
                    "worker_event.host_transition_correlation.authoritative_participant_id must match worker_event.target_participant_id"
                        .to_string(),
                );
            }
        }
        Ok(())
    }
}

impl TryFrom<WorldWorkerEventV1Def> for WorldWorkerEventV1 {
    type Error = String;

    fn try_from(value: WorldWorkerEventV1Def) -> Result<Self, Self::Error> {
        let worker_event = Self {
            schema_version: value.schema_version,
            acceptance_record_id: value.acceptance_record_id,
            stream_id: value.stream_id,
            frame_sequence: value.frame_sequence,
            event_id: value.event_id,
            event_sequence: value.event_sequence,
            request_id: value.request_id,
            active_run_id: value.active_run_id,
            host_transition_correlation: value.host_transition_correlation,
            causation_message_id: value.causation_message_id,
            causation_request_id: value.causation_request_id,
            orchestration_session_id: value.orchestration_session_id,
            source_participant_id: value.source_participant_id,
            target_participant_id: value.target_participant_id,
            source_backend_id: value.source_backend_id,
            target_backend_id: value.target_backend_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            thread_id: value.thread_id,
            event_class: value.event_class,
            attention_required: value.attention_required,
            payload: value.payload,
            emitted_at: value.emitted_at,
        };
        worker_event.validate()?;
        Ok(worker_event)
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_participant_id: Option<String>,

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

    // Producer-assigned semantic runtime identity. Standalone legacy events may omit it;
    // runtime Event frames require it at the transport boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_identity: Option<RuntimeEventIdentityV1>,

    // Typed retained worker-to-host semantic envelope. Legacy or out-of-scope
    // events may omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_event: Option<WorldWorkerEventV1>,

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
    participant_id: Option<String>,
    #[serde(default)]
    parent_participant_id: Option<String>,
    #[serde(default)]
    resumed_from_participant_id: Option<String>,
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
    #[serde(default)]
    event_identity: Option<RuntimeEventIdentityV1>,
    #[serde(default)]
    worker_event: Option<WorldWorkerEventV1>,
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
            participant_id: None,
            parent_participant_id: None,
            resumed_from_participant_id: None,
            data,
            backend_id: None,
            thread_id: None,
            role: None,
            world_id: None,
            world_generation: None,
            cmd_id: None,
            span_id: None,
            event_identity: None,
            worker_event: None,
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
        if let Some(event_identity) = self.event_identity.as_ref() {
            event_identity.validate()?;
        }
        if let Some(worker_event) = self.worker_event.as_ref() {
            worker_event.validate()?;
            let event_identity = self
                .event_identity
                .as_ref()
                .ok_or_else(|| "worker_event requires top-level event_identity".to_string())?;
            if event_identity.event_id != worker_event.event_id
                || event_identity.event_sequence != worker_event.event_sequence
            {
                return Err("worker_event must match top-level event_identity exactly".to_string());
            }
            validate_top_level_worker_event_match(
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
                worker_event.orchestration_session_id.as_str(),
            )?;
            validate_top_level_worker_event_match(
                "run_id",
                self.run_id.as_str(),
                worker_event.active_run_id.as_str(),
            )?;
            validate_top_level_worker_event_option_match(
                "participant_id",
                self.participant_id.as_deref(),
                worker_event.source_participant_id.as_str(),
            )?;
            validate_top_level_worker_event_option_match(
                "backend_id",
                self.backend_id.as_deref(),
                worker_event.source_backend_id.as_str(),
            )?;
            validate_top_level_worker_event_option_match(
                "thread_id",
                self.thread_id.as_deref(),
                worker_event.thread_id.as_str(),
            )?;
            validate_top_level_worker_event_option_match(
                "world_id",
                self.world_id.as_deref(),
                worker_event.world_id.as_str(),
            )?;
            let world_generation = self
                .world_generation
                .ok_or_else(|| "worker_event requires top-level world_generation".to_string())?;
            if world_generation != worker_event.world_generation {
                return Err(format!(
                    "worker_event.world_generation {} must match top-level world_generation {}",
                    worker_event.world_generation, world_generation
                ));
            }
        }
        validate_identity_tuple_and_placement_posture(
            self.identity_tuple.as_ref(),
            self.placement_posture.as_ref(),
        )
    }

    pub fn set_pure_agent_telemetry_identity(&mut self, client: impl Into<String>) {
        if self.identity_tuple.is_none() {
            let client = client.into();
            self.identity_tuple = Some(IdentityTuple {
                client: normalize_identity_tuple_client_id(&client)
                    .unwrap_or_else(|| "human".to_string()),
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
            participant_id: value.participant_id,
            parent_participant_id: value.parent_participant_id,
            resumed_from_participant_id: value.resumed_from_participant_id,
            backend_id: value.backend_id,
            thread_id: value.thread_id,
            role: value.role,
            world_id: value.world_id,
            world_generation: value.world_generation,
            cmd_id: value.cmd_id,
            span_id: value.span_id,
            event_identity: value.event_identity,
            worker_event: value.worker_event,
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

fn lowercase_uuid_v7_pattern() -> &'static Regex {
    static UUID_V7_RE: OnceLock<Regex> = OnceLock::new();
    UUID_V7_RE.get_or_init(|| {
        Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
            .expect("uuid v7 regex is valid")
    })
}

fn deserialize_sanitized_channel<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(AgentEvent::sanitize_channel(raw))
}

fn validate_required_trimmed(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() || value.trim() != value {
        return Err(format!("{field} must be non-empty and trimmed"));
    }
    Ok(())
}

fn validate_prefixed_uuid_v7(field: &str, value: &str, prefix: &str) -> Result<(), String> {
    validate_required_trimmed(field, value)?;
    let Some(suffix) = value.strip_prefix(prefix) else {
        return Err(format!("{field} must start with {prefix}"));
    };
    if !lowercase_uuid_v7_pattern().is_match(suffix) {
        return Err(format!(
            "{field} must contain a lowercase UUIDv7 after {prefix}"
        ));
    }
    Ok(())
}

fn validate_top_level_worker_event_match(
    field: &str,
    top_level: &str,
    worker_event: &str,
) -> Result<(), String> {
    if top_level != worker_event {
        return Err(format!(
            "worker_event.{field} {worker_event} must match top-level {field} {top_level}"
        ));
    }
    Ok(())
}

fn validate_top_level_worker_event_option_match(
    field: &str,
    top_level: Option<&str>,
    worker_event: &str,
) -> Result<(), String> {
    let top_level = top_level.ok_or_else(|| format!("worker_event requires top-level {field}"))?;
    validate_top_level_worker_event_match(field, top_level, worker_event)
}

#[cfg(test)]
mod tests {
    use super::{
        AgentEvent, MessageEventKind, NormalizedWorldWorkerEventFacetV1, RuntimeEventIdentityV1,
        WorldWorkerEventClassV1, WorldWorkerEventV1,
    };
    use crate::authority_commitment::{
        HostTransitionWorkCorrelationV1, OpaqueAuthorityCommitmentV1,
    };
    use serde_json::json;

    fn sample_worker_event() -> WorldWorkerEventV1 {
        WorldWorkerEventV1 {
            schema_version: 1,
            acceptance_record_id: "wwa_018f0f3a-9b2c-7def-8abc-0123456789ab".to_string(),
            stream_id: "rts_worker_event".to_string(),
            frame_sequence: 2,
            event_id: "evt_worker_event_1".to_string(),
            event_sequence: 1,
            request_id: "req_worker_event".to_string(),
            active_run_id: "req_worker_event".to_string(),
            host_transition_correlation: Some(HostTransitionWorkCorrelationV1 {
                schema_version: 1,
                authority_store_id: "authority_store_1".to_string(),
                orchestration_session_id: "session_worker".to_string(),
                authoritative_participant_id: "orch_participant".to_string(),
                transition_intent_id: "intent_1".to_string(),
                transition_intent_revision_observed: 3,
                transition_run_id: "transition_run_1".to_string(),
                transition_payload_commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                        .to_string(),
                },
                authority_revision_observed: 7,
            }),
            causation_message_id: "wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string(),
            causation_request_id: "req_worker_event".to_string(),
            orchestration_session_id: "session_worker".to_string(),
            source_participant_id: "worker_participant".to_string(),
            target_participant_id: "orch_participant".to_string(),
            source_backend_id: "cli:codex-world".to_string(),
            target_backend_id: "cli:codex-host".to_string(),
            world_id: "world_worker".to_string(),
            world_generation: 2,
            thread_id: "thread_worker".to_string(),
            event_class: WorldWorkerEventClassV1::FollowUpQuestion,
            attention_required: true,
            payload: json!({
                "message": "need confirmation"
            }),
            emitted_at: chrono::Utc::now(),
        }
    }

    fn sample_agent_event() -> AgentEvent {
        let worker_event = sample_worker_event();
        let mut event = AgentEvent::message(
            "agent",
            worker_event.orchestration_session_id.clone(),
            worker_event.active_run_id.clone(),
            MessageEventKind::TaskProgress,
            "need confirmation",
        );
        event.participant_id = Some(worker_event.source_participant_id.clone());
        event.backend_id = Some(worker_event.source_backend_id.clone());
        event.thread_id = Some(worker_event.thread_id.clone());
        event.world_id = Some(worker_event.world_id.clone());
        event.world_generation = Some(worker_event.world_generation);
        event.event_identity = Some(RuntimeEventIdentityV1 {
            event_id: worker_event.event_id.clone(),
            event_sequence: worker_event.event_sequence,
        });
        event.worker_event = Some(worker_event);
        event
    }

    #[test]
    fn legacy_agent_event_omits_worker_event_when_absent() {
        let mut event = AgentEvent::message(
            "agent",
            "session",
            "run",
            MessageEventKind::Status,
            "legacy",
        );
        event.event_identity = Some(RuntimeEventIdentityV1 {
            event_id: "evt_legacy".to_string(),
            event_sequence: 1,
        });

        let json = serde_json::to_value(&event).expect("serialize legacy event");
        assert!(json.get("worker_event").is_none());
        let decoded: AgentEvent = serde_json::from_value(json).expect("decode legacy event");
        assert!(decoded.worker_event.is_none());
    }

    #[test]
    fn worker_event_round_trips_with_strict_schema() {
        let event = sample_agent_event();
        let json = serde_json::to_value(&event).expect("serialize typed worker event");
        let decoded: AgentEvent = serde_json::from_value(json.clone()).expect("decode typed event");
        assert_eq!(decoded, event);
        assert_eq!(
            json.pointer("/worker_event/causation_request_id"),
            Some(&json!("req_worker_event"))
        );
    }

    #[test]
    fn agent_event_rejects_worker_event_top_level_identity_drift() {
        let mut json = serde_json::to_value(sample_agent_event()).expect("serialize");
        json["participant_id"] = json!("other_participant");

        let err = serde_json::from_value::<AgentEvent>(json)
            .expect_err("participant drift must fail closed");
        assert!(
            err.to_string().contains("worker_event.participant_id")
                || err
                    .to_string()
                    .contains("worker_event.source_participant_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn agent_event_rejects_unknown_worker_event_fields() {
        let mut json = serde_json::to_value(sample_agent_event()).expect("serialize");
        json["worker_event"]["unexpected"] = json!(true);

        assert!(
            serde_json::from_value::<AgentEvent>(json).is_err(),
            "unknown worker_event fields must fail closed"
        );
    }

    #[test]
    fn worker_event_rejects_malformed_ids_and_correlation_scope_drift() {
        let mut json = serde_json::to_value(sample_agent_event()).expect("serialize");
        json["worker_event"]["acceptance_record_id"] = json!("wwa_not-a-uuid");
        assert!(
            serde_json::from_value::<AgentEvent>(json.clone()).is_err(),
            "malformed acceptance_record_id must fail closed"
        );

        json = serde_json::to_value(sample_agent_event()).expect("serialize");
        json["worker_event"]["host_transition_correlation"]["authoritative_participant_id"] =
            json!("other_participant");
        assert!(
            serde_json::from_value::<AgentEvent>(json).is_err(),
            "correlation participant drift must fail closed"
        );
    }

    #[test]
    fn normalized_worker_event_facet_rejects_missing_attention_truth_for_attention_class() {
        let facet = json!({
            "schema_version": 1,
            "thread_id": "thread_worker",
            "event_class": "approval_request",
            "attention_required": false,
            "causation_message_id": "wwm_018f0f3a-9b2c-7def-8abc-0123456789ac",
            "causation_request_id": "req_worker_event",
            "payload": {
                "message": "needs approval"
            }
        });

        assert!(
            serde_json::from_value::<NormalizedWorldWorkerEventFacetV1>(facet).is_err(),
            "attention-driving classes must remain explicit"
        );
    }

    #[test]
    fn control_ack_round_trips_without_attention() {
        let mut event = sample_agent_event();
        let worker_event = event
            .worker_event
            .as_mut()
            .expect("sample agent event must carry worker_event");
        worker_event.event_class = WorldWorkerEventClassV1::ControlAck;
        worker_event.attention_required = false;

        let json = serde_json::to_value(&event).expect("serialize control_ack event");
        let decoded: AgentEvent = serde_json::from_value(json).expect("decode control_ack event");
        assert_eq!(
            decoded
                .worker_event
                .as_ref()
                .map(|worker_event| worker_event.event_class),
            Some(WorldWorkerEventClassV1::ControlAck)
        );
    }
}
