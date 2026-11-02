use crate::{AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperKind};

use super::{
    AGENT_KIND, CAP_SESSION_HANDLE_V1, CAP_TOOLS_STRUCTURED_V1, CHANNEL_ASSISTANT, CHANNEL_TOOL,
};

pub(super) fn map_stream_json_event(
    ev: claude_code::ClaudeStreamJsonEvent,
) -> Vec<AgentWrapperEvent> {
    match ev {
        claude_code::ClaudeStreamJsonEvent::SystemInit { .. } => {
            vec![status_event(Some("system init".to_string()))]
        }
        claude_code::ClaudeStreamJsonEvent::SystemOther { subtype, .. } => {
            vec![status_event(Some(format!("system {subtype}")))]
        }
        claude_code::ClaudeStreamJsonEvent::ResultError { .. } => {
            vec![error_event("result error".to_string())]
        }
        claude_code::ClaudeStreamJsonEvent::ResultSuccess { .. } => {
            vec![status_event(Some("result success".to_string()))]
        }
        claude_code::ClaudeStreamJsonEvent::AssistantMessage { raw, .. } => {
            map_assistant_message(&raw)
        }
        claude_code::ClaudeStreamJsonEvent::StreamEvent { stream, .. } => {
            map_stream_event(&stream.raw)
        }
        claude_code::ClaudeStreamJsonEvent::UserMessage { .. } => vec![status_event(None)],
        claude_code::ClaudeStreamJsonEvent::Unknown { .. } => vec![unknown_event()],
    }
}

pub(super) fn extract_assistant_message_final_text(raw: &serde_json::Value) -> Option<String> {
    let blocks = raw
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())?;

    let mut texts = Vec::new();
    for block in blocks {
        let Some(obj) = block.as_object() else {
            continue;
        };
        if obj.get("type").and_then(|v| v.as_str()) != Some("text") {
            continue;
        }
        let Some(text) = obj.get("text").and_then(|v| v.as_str()) else {
            continue;
        };
        texts.push(text);
    }

    if texts.is_empty() {
        None
    } else {
        Some(texts.join("\n"))
    }
}

pub(super) fn map_assistant_message(raw: &serde_json::Value) -> Vec<AgentWrapperEvent> {
    let Some(blocks) = raw
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
    else {
        return vec![unknown_event()];
    };

    let mut out = Vec::new();
    for block in blocks {
        let Some(obj) = block.as_object() else {
            out.push(unknown_event());
            continue;
        };
        let block_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match block_type {
            "text" => {
                if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                    out.extend(text_output_events(text, Some(CHANNEL_ASSISTANT)));
                } else {
                    out.push(unknown_event());
                }
            }
            "tool_use" => {
                let tool_name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());
                let tool_use_id = obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());
                out.push(tool_call_start_event(tool_name, tool_use_id));
            }
            "tool_result" => {
                let tool_use_id = obj
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());
                out.push(tool_result_complete_event(tool_use_id));
            }
            _ => out.push(unknown_event()),
        }
    }
    out
}

pub(super) fn map_stream_event(raw: &serde_json::Value) -> Vec<AgentWrapperEvent> {
    let Some(obj) = raw.as_object() else {
        return vec![unknown_event()];
    };
    let event_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match event_type {
        "content_block_start" => {
            let Some(content_block) = obj.get("content_block").and_then(|v| v.as_object()) else {
                return vec![unknown_event()];
            };
            let block_type = content_block
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match block_type {
                "tool_use" => {
                    let tool_name = content_block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let tool_use_id = content_block
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    vec![tool_call_start_event(tool_name, tool_use_id)]
                }
                "tool_result" => {
                    let tool_use_id = content_block
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    vec![tool_result_complete_event(tool_use_id)]
                }
                _ => vec![unknown_event()],
            }
        }
        "content_block_delta" => {
            let Some(delta) = obj.get("delta").and_then(|v| v.as_object()) else {
                return vec![unknown_event()];
            };
            let delta_type = delta.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match delta_type {
                "text_delta" => {
                    let Some(text) = delta.get("text").and_then(|v| v.as_str()) else {
                        return vec![unknown_event()];
                    };
                    text_output_events(text, Some(CHANNEL_ASSISTANT))
                }
                "input_json_delta" => vec![tool_call_delta_event()],
                _ => vec![unknown_event()],
            }
        }
        _ => vec![unknown_event()],
    }
}

fn tool_call_start_event(
    tool_name: Option<String>,
    tool_use_id: Option<String>,
) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::ToolCall,
        channel: Some(CHANNEL_TOOL.to_string()),
        text: None,
        message: None,
        data: Some(tool_facet(
            "tool_use",
            "start",
            "running",
            tool_name,
            tool_use_id,
        )),
    }
}

fn tool_call_delta_event() -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::ToolCall,
        channel: Some(CHANNEL_TOOL.to_string()),
        text: None,
        message: None,
        data: Some(tool_facet("tool_use", "delta", "running", None, None)),
    }
}

fn tool_result_complete_event(tool_use_id: Option<String>) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::ToolResult,
        channel: Some(CHANNEL_TOOL.to_string()),
        text: None,
        message: None,
        data: Some(tool_facet(
            "tool_result",
            "complete",
            "completed",
            None,
            tool_use_id,
        )),
    }
}

fn tool_facet(
    kind: &'static str,
    phase: &'static str,
    status: &'static str,
    tool_name: Option<String>,
    tool_use_id: Option<String>,
) -> serde_json::Value {
    serde_json::json!({
        "schema": CAP_TOOLS_STRUCTURED_V1,
        "tool": {
            "backend_item_id": null,
            "thread_id": null,
            "turn_id": null,
            "kind": kind,
            "phase": phase,
            "status": status,
            "exit_code": null,
            "bytes": { "stdout": 0, "stderr": 0, "diff": 0, "result": 0 },
            "tool_name": tool_name,
            "tool_use_id": tool_use_id,
        },
    })
}

pub(super) fn session_handle_facet(session_id: &str) -> serde_json::Value {
    serde_json::json!({
        "schema": CAP_SESSION_HANDLE_V1,
        "session": { "id": session_id },
    })
}

pub(super) fn status_event(message: Option<String>) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::Status,
        channel: Some("status".to_string()),
        text: None,
        message,
        data: None,
    }
}

pub(super) fn error_event(message: String) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::Error,
        channel: Some("error".to_string()),
        text: None,
        message: Some(message),
        data: None,
    }
}

fn unknown_event() -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::Unknown,
        channel: None,
        text: None,
        message: None,
        data: None,
    }
}

fn text_output_events(text: &str, channel: Option<&str>) -> Vec<AgentWrapperEvent> {
    vec![AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::TextOutput,
        channel: channel.map(|c| c.to_string()),
        text: Some(text.to_string()),
        message: None,
        data: None,
    }]
}
