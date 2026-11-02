use crate::{AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperKind};

use super::{AGENT_KIND, CHANNEL_ASSISTANT, CHANNEL_TOOL};

pub(super) fn map_stream_json_event(
    event: gemini_cli::GeminiStreamJsonEvent,
) -> Vec<AgentWrapperEvent> {
    match event {
        gemini_cli::GeminiStreamJsonEvent::Init {
            session_id,
            model,
            raw,
        } => vec![status_event(
            Some("init".to_string()),
            Some(serde_json::json!({
                "session": { "id": session_id },
                "model": model,
                "raw": raw,
            })),
        )],
        gemini_cli::GeminiStreamJsonEvent::Message {
            role,
            content,
            delta,
            raw,
        } => vec![text_event(
            role.clone(),
            content,
            Some(serde_json::json!({
                "role": role,
                "delta": delta,
                "raw": raw,
            })),
        )],
        gemini_cli::GeminiStreamJsonEvent::ToolUse {
            tool_name,
            tool_id,
            parameters,
            raw,
        } => vec![AgentWrapperEvent {
            agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
            kind: AgentWrapperEventKind::ToolCall,
            channel: Some(CHANNEL_TOOL.to_string()),
            text: None,
            message: None,
            data: Some(serde_json::json!({
                "tool_name": tool_name,
                "tool_id": tool_id,
                "parameters": parameters,
                "raw": raw,
            })),
        }],
        gemini_cli::GeminiStreamJsonEvent::ToolResult {
            tool_id,
            status,
            output,
            error,
            raw,
        } => vec![AgentWrapperEvent {
            agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
            kind: AgentWrapperEventKind::ToolResult,
            channel: Some(CHANNEL_TOOL.to_string()),
            text: output,
            message: None,
            data: Some(serde_json::json!({
                "tool_id": tool_id,
                "status": status,
                "error": error,
                "raw": raw,
            })),
        }],
        gemini_cli::GeminiStreamJsonEvent::Error {
            severity,
            message,
            raw,
        } => vec![error_event(
            message,
            Some(serde_json::json!({
                "severity": severity,
                "raw": raw,
            })),
        )],
        gemini_cli::GeminiStreamJsonEvent::Result { payload } => {
            if payload.status == "error" {
                vec![error_event(
                    payload
                        .error_message
                        .unwrap_or_else(|| "result error".to_string()),
                    Some(serde_json::json!({
                        "status": payload.status,
                        "error_type": payload.error_type,
                        "stats": payload.stats,
                        "raw": payload.raw,
                    })),
                )]
            } else {
                vec![status_event(
                    Some("result success".to_string()),
                    Some(serde_json::json!({
                        "status": payload.status,
                        "stats": payload.stats,
                        "raw": payload.raw,
                    })),
                )]
            }
        }
        gemini_cli::GeminiStreamJsonEvent::Unknown { event_type, raw } => vec![AgentWrapperEvent {
            agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
            kind: AgentWrapperEventKind::Unknown,
            channel: None,
            text: None,
            message: None,
            data: Some(serde_json::json!({
                "event_type": event_type,
                "raw": raw,
            })),
        }],
    }
}

fn status_event(message: Option<String>, data: Option<serde_json::Value>) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::Status,
        channel: Some("status".to_string()),
        text: None,
        message,
        data,
    }
}

fn text_event(role: String, text: String, data: Option<serde_json::Value>) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::TextOutput,
        channel: Some(match role.as_str() {
            "assistant" => CHANNEL_ASSISTANT.to_string(),
            other => other.to_string(),
        }),
        text: Some(text),
        message: None,
        data,
    }
}

fn error_event(message: String, data: Option<serde_json::Value>) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::Error,
        channel: Some("error".to_string()),
        text: None,
        message: Some(message),
        data,
    }
}
