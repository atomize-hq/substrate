use codex::ThreadEvent;
use serde_json::Value;

use crate::{AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperKind};

use super::TOOLS_FACET_SCHEMA;

pub(super) fn map_thread_event(event: &ThreadEvent) -> AgentWrapperEvent {
    match event {
        ThreadEvent::ThreadStarted(_) => status_event(None),
        ThreadEvent::TurnStarted(_) => status_event(None),
        ThreadEvent::TurnCompleted(_) => status_event(None),
        ThreadEvent::TurnFailed(_) => status_event(Some("turn failed".to_string())),
        ThreadEvent::Error(err) => error_event(err.message.clone()),
        ThreadEvent::ItemStarted(envelope) => map_item_snapshot_event(envelope, ToolPhase::Start),
        ThreadEvent::ItemCompleted(envelope) => {
            map_item_snapshot_event(envelope, ToolPhase::Complete)
        }
        ThreadEvent::ItemDelta(delta) => map_item_delta_event(delta),
        ThreadEvent::ItemFailed(envelope) => map_item_failed_event(envelope),
    }
}

pub(super) fn status_event(message: Option<String>) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind("codex".to_string()),
        kind: AgentWrapperEventKind::Status,
        channel: Some("status".to_string()),
        text: None,
        message,
        data: None,
    }
}

pub(super) fn error_event(message: String) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind("codex".to_string()),
        kind: AgentWrapperEventKind::Error,
        channel: Some("error".to_string()),
        text: None,
        message: Some(message),
        data: None,
    }
}

#[derive(Copy, Clone, Debug)]
enum ToolPhase {
    Start,
    Delta,
    Complete,
    Fail,
}

#[derive(Copy, Clone, Debug, Default)]
struct ToolBytes {
    stdout: usize,
    stderr: usize,
    diff: usize,
    result: usize,
}

fn is_toolish_item_type(item_type: &str) -> bool {
    matches!(
        item_type,
        "command_execution" | "file_change" | "mcp_tool_call" | "web_search"
    )
}

#[allow(clippy::too_many_arguments)]
fn tools_facet_data(
    backend_item_id: Option<&str>,
    thread_id: Option<&str>,
    turn_id: Option<&str>,
    kind: &str,
    phase: ToolPhase,
    status: &str,
    exit_code: Option<i32>,
    bytes: ToolBytes,
) -> Value {
    let phase = match phase {
        ToolPhase::Start => "start",
        ToolPhase::Delta => "delta",
        ToolPhase::Complete => "complete",
        ToolPhase::Fail => "fail",
    };

    serde_json::json!({
        "schema": TOOLS_FACET_SCHEMA,
        "tool": {
            "backend_item_id": backend_item_id,
            "thread_id": thread_id,
            "turn_id": turn_id,
            "kind": kind,
            "phase": phase,
            "status": status,
            "exit_code": exit_code,
            "bytes": {
                "stdout": bytes.stdout,
                "stderr": bytes.stderr,
                "diff": bytes.diff,
                "result": bytes.result
            },
            "tool_name": null,
            "tool_use_id": null
        }
    })
}

fn tool_result_bytes(value: &Option<Value>) -> usize {
    let Some(value) = value else {
        return 0;
    };
    serde_json::to_vec(value).map(|v| v.len()).unwrap_or(0)
}

fn map_item_snapshot_event(
    envelope: &codex::ItemEnvelope<codex::ItemSnapshot>,
    phase: ToolPhase,
) -> AgentWrapperEvent {
    match &envelope.item.payload {
        codex::ItemPayload::AgentMessage(content) | codex::ItemPayload::Reasoning(content) => {
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(content.text.clone()),
                message: None,
                data: None,
            }
        }
        codex::ItemPayload::CommandExecution(state) => {
            let (status, event_kind) = match phase {
                ToolPhase::Complete => ("completed", AgentWrapperEventKind::ToolResult),
                ToolPhase::Start | ToolPhase::Delta | ToolPhase::Fail => {
                    ("running", AgentWrapperEventKind::ToolCall)
                }
            };
            let bytes = ToolBytes {
                stdout: state.stdout.len(),
                stderr: state.stderr.len(),
                diff: 0,
                result: 0,
            };
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: event_kind,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    Some(envelope.item.item_id.as_str()),
                    Some(envelope.thread_id.as_str()),
                    Some(envelope.turn_id.as_str()),
                    "command_execution",
                    phase,
                    status,
                    state.exit_code,
                    bytes,
                )),
            }
        }
        codex::ItemPayload::FileChange(state) => {
            let (status, event_kind) = match phase {
                ToolPhase::Complete => ("completed", AgentWrapperEventKind::ToolResult),
                ToolPhase::Start | ToolPhase::Delta | ToolPhase::Fail => {
                    ("running", AgentWrapperEventKind::ToolCall)
                }
            };
            let bytes = ToolBytes {
                stdout: state.stdout.len(),
                stderr: state.stderr.len(),
                diff: state.diff.as_ref().map(|s| s.len()).unwrap_or(0),
                result: 0,
            };
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: event_kind,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    Some(envelope.item.item_id.as_str()),
                    Some(envelope.thread_id.as_str()),
                    Some(envelope.turn_id.as_str()),
                    "file_change",
                    phase,
                    status,
                    state.exit_code,
                    bytes,
                )),
            }
        }
        codex::ItemPayload::McpToolCall(state) => {
            let (status, event_kind) = match phase {
                ToolPhase::Complete => ("completed", AgentWrapperEventKind::ToolResult),
                ToolPhase::Start | ToolPhase::Delta | ToolPhase::Fail => {
                    ("running", AgentWrapperEventKind::ToolCall)
                }
            };
            let bytes = ToolBytes {
                stdout: 0,
                stderr: 0,
                diff: 0,
                // Count only MCP tool output (`result`), never `arguments`.
                result: tool_result_bytes(&state.result),
            };
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: event_kind,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    Some(envelope.item.item_id.as_str()),
                    Some(envelope.thread_id.as_str()),
                    Some(envelope.turn_id.as_str()),
                    "mcp_tool_call",
                    phase,
                    status,
                    None,
                    bytes,
                )),
            }
        }
        codex::ItemPayload::WebSearch(state) => {
            let (status, event_kind) = match phase {
                ToolPhase::Complete => ("completed", AgentWrapperEventKind::ToolResult),
                ToolPhase::Start | ToolPhase::Delta | ToolPhase::Fail => {
                    ("running", AgentWrapperEventKind::ToolCall)
                }
            };
            let bytes = ToolBytes {
                stdout: 0,
                stderr: 0,
                diff: 0,
                // Count only web search output (`results`), never `query`.
                result: tool_result_bytes(&state.results),
            };
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: event_kind,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    Some(envelope.item.item_id.as_str()),
                    Some(envelope.thread_id.as_str()),
                    Some(envelope.turn_id.as_str()),
                    "web_search",
                    phase,
                    status,
                    None,
                    bytes,
                )),
            }
        }
        codex::ItemPayload::TodoList(_) => status_event(None),
        codex::ItemPayload::Error(err) => error_event(err.message.clone()),
    }
}

fn map_item_delta_event(delta: &codex::ItemDelta) -> AgentWrapperEvent {
    match &delta.delta {
        codex::ItemDeltaPayload::AgentMessage(content)
        | codex::ItemDeltaPayload::Reasoning(content) => AgentWrapperEvent {
            agent_kind: AgentWrapperKind("codex".to_string()),
            kind: AgentWrapperEventKind::TextOutput,
            channel: Some("assistant".to_string()),
            text: Some(content.text_delta.clone()),
            message: None,
            data: None,
        },
        codex::ItemDeltaPayload::CommandExecution(state) => {
            let bytes = ToolBytes {
                stdout: state.stdout.len(),
                stderr: state.stderr.len(),
                diff: 0,
                result: 0,
            };
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::ToolCall,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    Some(delta.item_id.as_str()),
                    Some(delta.thread_id.as_str()),
                    Some(delta.turn_id.as_str()),
                    "command_execution",
                    ToolPhase::Delta,
                    "running",
                    state.exit_code,
                    bytes,
                )),
            }
        }
        codex::ItemDeltaPayload::FileChange(state) => {
            let bytes = ToolBytes {
                stdout: state.stdout.len(),
                stderr: state.stderr.len(),
                diff: state.diff.as_ref().map(|s| s.len()).unwrap_or(0),
                result: 0,
            };
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::ToolCall,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    Some(delta.item_id.as_str()),
                    Some(delta.thread_id.as_str()),
                    Some(delta.turn_id.as_str()),
                    "file_change",
                    ToolPhase::Delta,
                    "running",
                    state.exit_code,
                    bytes,
                )),
            }
        }
        codex::ItemDeltaPayload::McpToolCall(state) => {
            let bytes = ToolBytes {
                stdout: 0,
                stderr: 0,
                diff: 0,
                // Count only MCP tool output (`result`), never `arguments`.
                result: tool_result_bytes(&state.result),
            };
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::ToolCall,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    Some(delta.item_id.as_str()),
                    Some(delta.thread_id.as_str()),
                    Some(delta.turn_id.as_str()),
                    "mcp_tool_call",
                    ToolPhase::Delta,
                    "running",
                    None,
                    bytes,
                )),
            }
        }
        codex::ItemDeltaPayload::WebSearch(state) => {
            let bytes = ToolBytes {
                stdout: 0,
                stderr: 0,
                diff: 0,
                // Count only web search output (`results`), never `query`.
                result: tool_result_bytes(&state.results),
            };
            AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::ToolCall,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    Some(delta.item_id.as_str()),
                    Some(delta.thread_id.as_str()),
                    Some(delta.turn_id.as_str()),
                    "web_search",
                    ToolPhase::Delta,
                    "running",
                    None,
                    bytes,
                )),
            }
        }
        codex::ItemDeltaPayload::TodoList(_) => status_event(None),
        codex::ItemDeltaPayload::Error(err) => error_event(err.message.clone()),
    }
}

fn map_item_failed_event(envelope: &codex::ItemEnvelope<codex::ItemFailure>) -> AgentWrapperEvent {
    // IMPORTANT: `ItemFailure.extra["item_type"]` is populated from a *top-level* `item_type` field
    // on the `item.failed` JSON object, not from a nested `{ "extra": { ... } }` object.
    let item_type = envelope.item.extra.get("item_type").and_then(Value::as_str);
    let Some(item_type) = item_type else {
        return error_event(envelope.item.error.message.clone());
    };
    if !is_toolish_item_type(item_type) {
        return error_event(envelope.item.error.message.clone());
    }

    AgentWrapperEvent {
        agent_kind: AgentWrapperKind("codex".to_string()),
        kind: AgentWrapperEventKind::ToolResult,
        channel: Some("tool".to_string()),
        text: None,
        message: None,
        data: Some(tools_facet_data(
            Some(envelope.item.item_id.as_str()),
            Some(envelope.thread_id.as_str()),
            Some(envelope.turn_id.as_str()),
            // Failed ToolResult kind is derived from the deterministically-attributable item_type.
            item_type,
            ToolPhase::Fail,
            "failed",
            // Failure attribution is metadata-only: no exit_code, and all byte counts are zero.
            None,
            ToolBytes::default(),
        )),
    }
}
