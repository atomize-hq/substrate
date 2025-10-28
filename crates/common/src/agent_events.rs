use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub kind: AgentEventKind,
    pub data: serde_json::Value,
}

impl AgentEvent {
    /// Build a message-style event with the provided payload text.
    pub fn message(
        agent_id: impl Into<String>,
        kind: AgentEventKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            ts: Utc::now(),
            agent_id: agent_id.into(),
            project: None,
            kind,
            data: serde_json::json!({ "message": message.into() }),
        }
    }

    /// Convenience helper for stdout/stderr stream chunks.
    pub fn stream_chunk(
        agent_id: impl Into<String>,
        is_stderr: bool,
        chunk: impl Into<String>,
    ) -> Self {
        let kind = AgentEventKind::PtyData;
        Self {
            ts: Utc::now(),
            agent_id: agent_id.into(),
            project: None,
            kind,
            data: serde_json::json!({
                "stream": if is_stderr { "stderr" } else { "stdout" },
                "chunk": chunk.into(),
            }),
        }
    }
}
