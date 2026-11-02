use std::{collections::BTreeMap, ffi::OsString, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

/// JSON-RPC method name used to initialize MCP servers.
pub const METHOD_INITIALIZE: &str = "initialize";
/// JSON-RPC method name used to shut down MCP servers.
pub const METHOD_SHUTDOWN: &str = "shutdown";
/// JSON-RPC method name used after shutdown to signal exit.
pub const METHOD_EXIT: &str = "exit";
/// JSON-RPC cancellation method per the spec.
pub const METHOD_CANCEL: &str = "$/cancelRequest";

/// Tool-call method exposed by the MCP server.
pub const METHOD_CODEX: &str = "tools/call";
/// Tool-call method for follow-up prompts (codex-reply).
pub const METHOD_CODEX_REPLY: &str = "tools/call";
/// Notification channel emitted by `codex mcp-server`.
pub const METHOD_CODEX_EVENT: &str = "codex/event";
/// Expected approval response hook (server-specific; confirmed during E2).
pub const METHOD_CODEX_APPROVAL: &str = "codex/approval";

/// Method names exposed by `codex app-server`.
pub const METHOD_THREAD_START: &str = "thread/start";
/// Resume an existing thread.
pub const METHOD_THREAD_RESUME: &str = "thread/resume";
/// List threads (paged).
pub const METHOD_THREAD_LIST: &str = "thread/list";
/// Fork an existing thread.
pub const METHOD_THREAD_FORK: &str = "thread/fork";
/// Start a new turn on a thread.
pub const METHOD_TURN_START: &str = "turn/start";
/// Interrupt an active turn.
pub const METHOD_TURN_INTERRUPT: &str = "turn/interrupt";

/// Unique identifier for JSON-RPC calls.
pub type RequestId = u64;

/// Stream of notifications surfaced alongside a JSON-RPC response.
pub type EventStream<T> = mpsc::UnboundedReceiver<T>;

/// Shared launch configuration for stdio MCP/app-server processes.
///
/// The Workstream A env-prep helper should populate `binary`, `code_home`, and
/// baseline environment entries. Callers can layer additional `env` entries for
/// per-call overrides (e.g., `RUST_LOG`). `mirror_stdio` controls whether raw
/// stdout/stderr should be mirrored to the host console in addition to being
/// parsed as JSON-RPC.
#[derive(Clone, Debug)]
pub struct StdioServerConfig {
    pub binary: PathBuf,
    pub code_home: Option<PathBuf>,
    pub current_dir: Option<PathBuf>,
    pub env: Vec<(OsString, OsString)>,
    /// Enables the `codex app-server --analytics-default-enabled` flag when launching app-server.
    pub app_server_analytics_default_enabled: bool,
    pub mirror_stdio: bool,
    pub startup_timeout: Duration,
}

/// Client metadata attached to the `initialize` request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Parameters for the initial `initialize` handshake.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "clientInfo")]
    pub client: ClientInfo,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: Value,
}

/// Parameters for `codex/codex` (new session).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CodexCallParams {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub config: BTreeMap<String, Value>,
}

/// Parameters for `codex/codex-reply` (continue an existing conversation).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodexReplyParams {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    pub prompt: String,
}

/// Classification for approval prompts surfaced by the MCP server.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ApprovalKind {
    Exec,
    Apply,
    Unknown(String),
}

/// Approval request emitted as part of a `codex/event` notification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub kind: ApprovalKind,
    /// Full payload from the server so callers can render UI or inspect diffs/commands.
    pub payload: Value,
}

/// Decision payload sent back to the MCP server in response to an approval prompt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approve {
        approval_id: String,
    },
    Reject {
        approval_id: String,
        reason: Option<String>,
    },
}

/// Notification emitted by `codex/event`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CodexEvent {
    TaskComplete {
        conversation_id: String,
        result: Value,
    },
    ApprovalRequired(ApprovalRequest),
    Cancelled {
        conversation_id: Option<String>,
        reason: Option<String>,
    },
    Error {
        message: String,
        data: Option<Value>,
    },
    Raw {
        method: String,
        params: Value,
    },
}

/// Final response payload for `codex/codex` or `codex/codex-reply`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodexCallResult {
    #[serde(default, rename = "conversationId", alias = "conversation_id")]
    pub conversation_id: Option<String>,
    #[serde(default, rename = "content", alias = "output")]
    pub output: Value,
}

/// Handle returned for each codex call, bundling response and notifications.
pub struct CodexCallHandle {
    pub request_id: RequestId,
    pub events: EventStream<CodexEvent>,
    pub response: oneshot::Receiver<Result<CodexCallResult, super::McpError>>,
}

/// Parameters for `thread/start`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadStartParams {
    pub thread_id: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

/// Parameters for `thread/resume`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadResumeParams {
    pub thread_id: String,
}

/// Sorting key for `thread/list`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadListSortKey {
    CreatedAt,
    UpdatedAt,
}

/// Parameters for `thread/list`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Opaque pagination cursor; MUST be explicitly `null` on the first request.
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_key: Option<ThreadListSortKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_providers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kinds: Option<Vec<String>>,
}

/// Response payload for `thread/list`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListResponse {
    pub data: Vec<ThreadSummary>,
    pub next_cursor: Option<String>,
}

/// Thread metadata summary returned by `thread/list`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadSummary {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Parameters for `thread/fork`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadForkParams {
    pub thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persist_extended_history: Option<bool>,
}

/// Response payload for `thread/fork`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadForkResponse {
    pub thread: ForkedThread,
}

/// Forked thread metadata returned by `thread/fork`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForkedThread {
    pub id: String,
}

/// Parameters for `turn/start`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnStartParams {
    pub thread_id: String,
    #[serde(rename = "input", alias = "prompt")]
    pub input: Vec<TurnInput>,
    pub model: Option<String>,
    #[serde(default)]
    pub config: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnInput {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Parameters for `turn/start` (fork-focused pinned v2 subset).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnStartParamsV2 {
    pub thread_id: String,
    pub input: Vec<UserInputV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
}

/// User input shape pinned for fork flows.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserInputV2 {
    Text {
        text: String,
        #[serde(default)]
        text_elements: Vec<Value>,
    },
}

impl UserInputV2 {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            text_elements: Vec::new(),
        }
    }
}

/// Parameters for `turn/interrupt`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnInterruptParams {
    pub thread_id: Option<String>,
    pub turn_id: String,
}

/// Notification emitted by the app-server.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppNotification {
    TaskComplete {
        thread_id: String,
        turn_id: Option<String>,
        result: Value,
    },
    Item {
        thread_id: String,
        turn_id: Option<String>,
        item: Value,
    },
    Error {
        message: String,
        data: Option<Value>,
    },
    Raw {
        method: String,
        params: Value,
    },
}

/// Handle returned for each app-server call, bundling response and notifications.
pub struct AppCallHandle {
    pub request_id: RequestId,
    pub events: EventStream<AppNotification>,
    pub response: oneshot::Receiver<Result<Value, super::McpError>>,
}
