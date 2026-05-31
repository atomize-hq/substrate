use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    CodexRolloutJsonl,
    CodexWrapperJsonl,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CompactionKind {
    UserMessage,
    AssistantMessage,
    DeveloperMessage,
    SystemMessage,
    Reasoning,
    ToolCall,
    ToolOutput,
    Status,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UserMessageRole {
    Prompt,
    Steer,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactionRow {
    pub source_file: Utf8PathBuf,
    pub source_kind: SourceKind,
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub event_index: usize,
    pub line_number: usize,
    pub row_ordinal: usize,
    #[serde(with = "time::serde::rfc3339::option")]
    pub timestamp: Option<OffsetDateTime>,
    pub kind: CompactionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_message_role: Option<UserMessageRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedupe_identity: Option<String>,
    pub text: String,
    #[serde(default)]
    pub canonical_text: String,
    pub text_hash_hex: String,
}
