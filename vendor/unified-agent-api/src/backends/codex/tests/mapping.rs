use super::support::*;
use serde_json::Value;

#[test]
fn thread_started_maps_to_status() {
    let mapped = map(r#"{"type":"thread.started","thread_id":"thread-1"}"#);
    assert_eq!(mapped.agent_kind.as_str(), "codex");
    assert_eq!(mapped.kind, AgentWrapperEventKind::Status);
    assert_eq!(mapped.text, None);
}

#[test]
fn turn_started_maps_to_status() {
    let mapped = map(r#"{"type":"turn.started","thread_id":"thread-1","turn_id":"turn-1"}"#);
    assert_eq!(mapped.kind, AgentWrapperEventKind::Status);
    assert_eq!(mapped.text, None);
}

#[test]
fn turn_completed_maps_to_status() {
    let mapped = map(r#"{"type":"turn.completed","thread_id":"thread-1","turn_id":"turn-1"}"#);
    assert_eq!(mapped.kind, AgentWrapperEventKind::Status);
    assert_eq!(mapped.text, None);
}

#[test]
fn turn_failed_maps_to_status() {
    let mapped = map(
        r#"{"type":"turn.failed","thread_id":"thread-1","turn_id":"turn-1","error":{"message":"boom"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::Status);
    assert_eq!(mapped.text, None);
}

#[test]
fn transport_error_maps_to_error_with_message() {
    let mapped = map(r#"{"type":"error","message":"transport failed"}"#);
    assert_eq!(mapped.kind, AgentWrapperEventKind::Error);
    assert_eq!(mapped.text, None);
    assert!(mapped.message.is_some());
}

#[test]
fn item_failed_without_item_type_maps_to_error_with_message() {
    let mapped = map(
        r#"{"type":"item.failed","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-1","error":{"message":"tool failed"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::Error);
    assert_eq!(mapped.text, None);
    assert!(mapped.message.is_some());
}

#[test]
fn item_failed_with_tool_item_type_maps_to_tool_result_failed() {
    let mapped = map(
        r#"{"type":"item.failed","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-1","item_type":"command_execution","error":{"message":"tool failed"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::ToolResult);
    assert_eq!(tool_schema(&mapped), Some(TOOLS_FACET_SCHEMA));
    assert_eq!(
        tool_field(&mapped, "phase").and_then(Value::as_str),
        Some("fail")
    );
    assert_eq!(
        tool_field(&mapped, "status").and_then(Value::as_str),
        Some("failed")
    );
    assert_eq!(
        tool_field(&mapped, "kind").and_then(Value::as_str),
        Some("command_execution")
    );
    assert_eq!(tool_field(&mapped, "exit_code"), Some(&Value::Null));
    let bytes = tool_field(&mapped, "bytes")
        .and_then(Value::as_object)
        .unwrap();
    assert_eq!(bytes.get("stdout"), Some(&Value::from(0)));
    assert_eq!(bytes.get("stderr"), Some(&Value::from(0)));
    assert_eq!(bytes.get("diff"), Some(&Value::from(0)));
    assert_eq!(bytes.get("result"), Some(&Value::from(0)));
}

#[test]
fn item_failed_with_non_tool_item_type_maps_to_error() {
    let mapped = map(
        r#"{"type":"item.failed","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-1","item_type":"agent_message","error":{"message":"not a tool failure"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::Error);
    assert!(mapped.message.is_some());
}

#[test]
fn agent_message_item_maps_to_text_output_and_uses_text_field() {
    let mapped = map(
        r#"{"type":"item.started","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-1","item_type":"agent_message","content":{"text":"hello"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(mapped.text.as_deref(), Some("hello"));
    assert_eq!(mapped.message, None);
}

#[test]
fn agent_message_delta_maps_to_text_output_and_uses_text_field() {
    let mapped = map(
        r#"{"type":"item.delta","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-1","item_type":"agent_message","delta":{"text_delta":"hi"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(mapped.text.as_deref(), Some("hi"));
    assert_eq!(mapped.message, None);
}

#[test]
fn reasoning_item_maps_to_text_output_and_uses_text_field() {
    let mapped = map(
        r#"{"type":"item.completed","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-2","item_type":"reasoning","content":{"text":"thinking"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(mapped.text.as_deref(), Some("thinking"));
    assert_eq!(mapped.message, None);
}

#[test]
fn command_execution_item_maps_to_tool_call() {
    let mapped = map(
        r#"{"type":"item.started","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-3","item_type":"command_execution","content":{"command":"echo hi"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::ToolCall);
    assert_eq!(tool_schema(&mapped), Some(TOOLS_FACET_SCHEMA));
    assert_eq!(
        tool_field(&mapped, "phase").and_then(Value::as_str),
        Some("start")
    );
}

#[test]
fn command_execution_item_completed_maps_to_tool_result() {
    let mapped = map(
        r#"{"type":"item.completed","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-3","item_type":"command_execution","content":{"command":"echo hi","stdout":"ok","stderr":"warn","exit_code":0}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::ToolResult);
    assert_eq!(tool_schema(&mapped), Some(TOOLS_FACET_SCHEMA));
    assert_eq!(
        tool_field(&mapped, "phase").and_then(Value::as_str),
        Some("complete")
    );
    assert_eq!(
        tool_field(&mapped, "status").and_then(Value::as_str),
        Some("completed")
    );
}

#[test]
fn todo_list_item_maps_to_status() {
    let mapped = map(
        r#"{"type":"item.started","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-4","item_type":"todo_list","content":{"items":[{"title":"one","completed":false}]}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::Status);
}

#[test]
fn item_payload_error_maps_to_error_with_message() {
    let mapped = map(
        r#"{"type":"item.completed","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-5","item_type":"error","content":{"message":"bad"}}"#,
    );
    assert_eq!(mapped.kind, AgentWrapperEventKind::Error);
    assert!(mapped.message.is_some());
}
