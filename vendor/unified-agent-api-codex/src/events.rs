use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Single JSONL event emitted by `codex exec --json`.
///
/// Each line on stdout maps to a [`ThreadEvent`] with lifecycle edges:
/// - `thread.started` is emitted once per invocation.
/// - `turn.started` begins the turn associated with the provided prompt.
/// - one or more `item.*` events stream output and tool activity.
/// - `turn.completed` or `turn.failed` closes the stream; `error` captures transport-level failures.
///
/// Item variants mirror the upstream `item_type` field: `agent_message`, `reasoning`,
/// `command_execution`, `file_change`, `mcp_tool_call`, `web_search`, `todo_list`, and `error`.
/// Unknown or future fields are preserved in `extra` maps to keep the parser forward-compatible.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ThreadEvent {
    #[serde(rename = "thread.started", alias = "thread.resumed")]
    ThreadStarted(ThreadStarted),
    #[serde(rename = "turn.started")]
    TurnStarted(TurnStarted),
    #[serde(rename = "turn.completed")]
    TurnCompleted(TurnCompleted),
    #[serde(rename = "turn.failed")]
    TurnFailed(TurnFailed),
    #[serde(rename = "item.started", alias = "item.created")]
    ItemStarted(ItemEnvelope<ItemSnapshot>),
    #[serde(rename = "item.delta", alias = "item.updated")]
    ItemDelta(ItemDelta),
    #[serde(rename = "item.completed")]
    ItemCompleted(ItemEnvelope<ItemSnapshot>),
    #[serde(rename = "item.failed")]
    ItemFailed(ItemEnvelope<ItemFailure>),
    #[serde(rename = "error")]
    Error(EventError),
}

impl ThreadEvent {
    pub fn thread_id(&self) -> Option<&str> {
        match self {
            ThreadEvent::ThreadStarted(event) => Some(event.thread_id.as_str()),
            ThreadEvent::TurnStarted(event) => Some(event.thread_id.as_str()),
            ThreadEvent::TurnCompleted(event) => Some(event.thread_id.as_str()),
            ThreadEvent::TurnFailed(event) => Some(event.thread_id.as_str()),
            ThreadEvent::ItemStarted(event) => Some(event.thread_id.as_str()),
            ThreadEvent::ItemDelta(event) => Some(event.thread_id.as_str()),
            ThreadEvent::ItemCompleted(event) => Some(event.thread_id.as_str()),
            ThreadEvent::ItemFailed(event) => Some(event.thread_id.as_str()),
            ThreadEvent::Error(_) => None,
        }
    }
}

/// Marks the start of a new thread.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThreadStarted {
    pub thread_id: String,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Indicates the CLI accepted a new turn within a thread.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TurnStarted {
    pub thread_id: String,
    pub turn_id: String,
    /// Original input text when upstream echoes it; may be omitted for security reasons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_text: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Reports a completed turn.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TurnCompleted {
    pub thread_id: String,
    pub turn_id: String,
    /// Identifier of the last output item when provided by the CLI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_item_id: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Indicates a turn-level failure.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TurnFailed {
    pub thread_id: String,
    pub turn_id: String,
    pub error: EventError,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Shared wrapper for item events that always include thread/turn context.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemEnvelope<T> {
    pub thread_id: String,
    pub turn_id: String,
    #[serde(flatten)]
    pub item: T,
}

/// Snapshot of an item at start/completion time.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemSnapshot {
    #[serde(rename = "item_id", alias = "id")]
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(default)]
    pub status: ItemStatus,
    #[serde(flatten)]
    pub payload: ItemPayload,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Streaming delta describing the next piece of an item.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemDelta {
    pub thread_id: String,
    pub turn_id: String,
    #[serde(rename = "item_id", alias = "id")]
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(flatten)]
    pub delta: ItemDeltaPayload,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Terminal item failure event.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemFailure {
    #[serde(rename = "item_id", alias = "id")]
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    pub error: EventError,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Fully-typed item payload for start/completed events.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "item_type", content = "content", rename_all = "snake_case")]
pub enum ItemPayload {
    AgentMessage(TextContent),
    Reasoning(TextContent),
    CommandExecution(CommandExecutionState),
    FileChange(FileChangeState),
    McpToolCall(McpToolCallState),
    WebSearch(WebSearchState),
    TodoList(TodoListState),
    Error(EventError),
}

/// Delta form of an item payload. Each delta should be applied in order to reconstruct the item.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "item_type", content = "delta", rename_all = "snake_case")]
pub enum ItemDeltaPayload {
    AgentMessage(TextDelta),
    Reasoning(TextDelta),
    CommandExecution(CommandExecutionDelta),
    FileChange(FileChangeDelta),
    McpToolCall(McpToolCallDelta),
    WebSearch(WebSearchDelta),
    TodoList(TodoListDelta),
    Error(EventError),
}

/// Item status supplied by the CLI for bookkeeping.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    #[default]
    InProgress,
    Completed,
    Failed,
    #[serde(other)]
    Unknown,
}

/// Human-readable content emitted by the agent.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextContent {
    pub text: String,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Incremental content fragment for streaming items.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextDelta {
    #[serde(rename = "text_delta", alias = "text")]
    pub text_delta: String,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Snapshot of a command execution, including accumulated stdout/stderr.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommandExecutionState {
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "aggregated_output",
        alias = "output"
    )]
    pub stdout: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "error_output",
        alias = "err"
    )]
    pub stderr: String,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Streaming delta for command execution.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommandExecutionDelta {
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "aggregated_output",
        alias = "output"
    )]
    pub stdout: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "error_output",
        alias = "err"
    )]
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// File change or diff applied by the agent.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileChangeState {
    #[serde(alias = "file_path")]
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<FileChangeKind>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "patch")]
    pub diff: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "aggregated_output",
        alias = "output"
    )]
    pub stdout: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "error_output",
        alias = "err"
    )]
    pub stderr: String,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Streaming delta describing a file change.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileChangeDelta {
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "patch")]
    pub diff: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "aggregated_output",
        alias = "output"
    )]
    pub stdout: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "error_output",
        alias = "err"
    )]
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Type of file operation being reported.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeKind {
    Apply,
    Diff,
    #[serde(other)]
    Unknown,
}

/// State of an MCP tool call.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McpToolCallState {
    #[serde(alias = "server")]
    pub server_name: String,
    #[serde(alias = "tool")]
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default)]
    pub status: ToolCallStatus,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Streaming delta for MCP tool call output.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McpToolCallDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default)]
    pub status: ToolCallStatus,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Lifecycle state for a tool call.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    #[serde(other)]
    Unknown,
}

/// Details of a web search step.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebSearchState {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<Value>,
    #[serde(default)]
    pub status: WebSearchStatus,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Streaming delta for search results.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebSearchDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<Value>,
    #[serde(default)]
    pub status: WebSearchStatus,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Search progress indicator.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    #[serde(other)]
    Unknown,
}

/// Checklist maintained by the agent.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TodoListState {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<TodoItem>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Streaming delta for todo list mutations.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TodoListDelta {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<TodoItem>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Single todo item.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TodoItem {
    pub title: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Error payload shared by turn/item failures.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventError {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}
