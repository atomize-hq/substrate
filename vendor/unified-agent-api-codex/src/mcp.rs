//! Launch and interact with Codex MCP + app servers using stored runtime definitions.
//!
//! - Spawn `codex mcp-server`, call `codex/codex` or `codex/codex-reply`, and stream
//!   `codex/event` notifications (task completion, approvals, cancellations, errors).
//! - Start `codex app-server` threads/turns and surface item/task_complete notifications.
//! - Manage `[mcp_servers]` and `[app_runtimes]` config entries, resolve them into launch-ready
//!   runtimes, and expose read-only APIs (including pooled app runtimes) without mutating stored
//!   config or thread metadata.
//! - Requests may be cancelled via the JSON-RPC `$ /cancelRequest` flow.
//!
//! The MCP server exposes two tool entrypoints:
//! - `codex/codex`: start a new Codex session with a prompt.
//! - `codex/codex-reply`: continue an existing session by conversation ID.
//!
//! The app-server supports `thread/start`, `thread/resume`, `turn/start`, and `turn/interrupt`
//! requests. Runtime and pool helpers keep resume hints/metadata intact while starting,
//! reusing, and stopping app-server instances.

mod protocol;
pub use protocol::*;

mod config;
pub use config::*;

mod runtime;
pub use runtime::*;
mod app;
pub use app::*;
mod custody;
mod jsonrpc;
pub use custody::{McpProcessCustody, McpSettlementError};

mod client;
pub use client::*;

#[cfg(all(test, unix))]
mod test_support;
#[cfg(all(test, unix))]
mod tests_core;
#[cfg(all(test, unix))]
mod tests_runtime_app;
