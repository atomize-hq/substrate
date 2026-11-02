use serde_json::json;

use crate::{AgentWrapperEventKind, AgentWrapperKind};

use super::super::mapping::map_run_json_event;

#[test]
fn text_event_maps_to_text_output_without_raw_payload() {
    let event = opencode::OpencodeRunJsonEvent::Text {
        session_id: Some("session-1".to_string()),
        text: "hello".to_string(),
        raw: json!({"type":"text","text":"hello","secret":"do-not-leak"}),
    };

    let mapped = map_run_json_event(event);
    assert_eq!(mapped.len(), 1);

    let mapped = &mapped[0];
    assert_eq!(mapped.agent_kind, AgentWrapperKind("opencode".to_string()));
    assert_eq!(mapped.kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(mapped.text.as_deref(), Some("hello"));
    assert_eq!(mapped.message, None);
    assert_eq!(mapped.data, None);
    assert!(!format!("{mapped:?}").contains("do-not-leak"));
}

#[test]
fn step_start_maps_to_status() {
    let event = opencode::OpencodeRunJsonEvent::StepStart {
        session_id: None,
        raw: json!({"type":"step_start","secret":"do-not-leak"}),
    };

    let mapped = map_run_json_event(event);
    assert_eq!(mapped.len(), 1);

    let mapped = &mapped[0];
    assert_eq!(mapped.kind, AgentWrapperEventKind::Status);
    assert_eq!(mapped.message.as_deref(), Some("step_start"));
    assert_eq!(mapped.data, None);
    assert!(!format!("{mapped:?}").contains("do-not-leak"));
}

#[test]
fn step_finish_maps_to_status() {
    let event = opencode::OpencodeRunJsonEvent::StepFinish {
        session_id: None,
        raw: json!({"type":"step_finish","secret":"do-not-leak"}),
    };

    let mapped = map_run_json_event(event);
    assert_eq!(mapped.len(), 1);

    let mapped = &mapped[0];
    assert_eq!(mapped.kind, AgentWrapperEventKind::Status);
    assert_eq!(mapped.message.as_deref(), Some("step_finish"));
    assert_eq!(mapped.data, None);
}

#[test]
fn unknown_maps_to_unknown_without_echoing_raw_line() {
    let event = opencode::OpencodeRunJsonEvent::Unknown {
        event_type: "tool_call".to_string(),
        session_id: None,
        raw: json!({"type":"tool_call","secret":"do-not-leak"}),
    };

    let mapped = map_run_json_event(event);
    assert_eq!(mapped.len(), 1);

    let mapped = &mapped[0];
    assert_eq!(mapped.kind, AgentWrapperEventKind::Unknown);
    assert_eq!(mapped.message, None);
    assert_eq!(mapped.data, None);
    assert!(!format!("{mapped:?}").contains("do-not-leak"));
}

#[test]
fn terminal_error_maps_to_public_error_without_echoing_raw_payload() {
    let event = opencode::OpencodeRunJsonEvent::TerminalError {
        message: "no session found".to_string(),
        raw: json!({"secret":"do-not-leak"}),
    };

    let mapped = map_run_json_event(event);
    assert_eq!(mapped.len(), 1);

    let mapped = &mapped[0];
    assert_eq!(mapped.kind, AgentWrapperEventKind::Error);
    assert_eq!(mapped.message.as_deref(), Some("no session found"));
    assert_eq!(mapped.data, None);
    assert!(!format!("{mapped:?}").contains("do-not-leak"));
}
