use super::support::*;
use serde_json::{json, Value};

#[test]
fn fork_selector_is_extracted_into_policy_when_validate_and_extract_policy_is_called_directly() {
    let adapter = test_adapter();
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request
        .extensions
        .insert(EXT_SESSION_FORK_V1.to_string(), json!({"selector": "last"}));

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("expected policy extraction to succeed");
    assert_eq!(policy.fork, Some(SessionSelectorV1::Last));
}

#[test]
fn fork_key_is_supported_via_backend_harness_normalize_request() {
    let adapter = test_adapter();
    let defaults = BackendDefaults::default();
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request
        .extensions
        .insert(EXT_SESSION_FORK_V1.to_string(), json!({"selector": "last"}));

    let normalized = crate::backend_harness::normalize_request(&adapter, &defaults, request)
        .expect("expected fork extension key to be supported in S4b");
    assert_eq!(normalized.policy.fork, Some(SessionSelectorV1::Last));
}

#[test]
fn handle_facet_emitted_once_on_thread_started_and_attached_to_completion() {
    let adapter = test_adapter();

    let first = adapter.map_event(CodexBackendEvent::Thread(Box::new(parse_thread_event(
        r#"{"type":"thread.started","thread_id":"thread-1"}"#,
    ))));
    let second = adapter.map_event(CodexBackendEvent::Thread(Box::new(parse_thread_event(
        r#"{"type":"turn.started","thread_id":"thread-1","turn_id":"turn-1"}"#,
    ))));

    let seen: Vec<AgentWrapperEvent> = first.into_iter().chain(second).collect();
    let handle_events: Vec<&AgentWrapperEvent> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Status)
        .filter(|ev| handle_schema(ev) == Some(CAP_SESSION_HANDLE_V1))
        .collect();
    assert_eq!(handle_events.len(), 1);
    assert_eq!(handle_schema(&seen[0]), Some(CAP_SESSION_HANDLE_V1));
    assert_eq!(
        seen[0]
            .data
            .as_ref()
            .and_then(|data| data.get("session"))
            .and_then(|session| session.get("id"))
            .and_then(Value::as_str),
        Some("thread-1")
    );

    let completion = adapter
        .map_completion(CodexBackendCompletion {
            status: success_exit_status(),
            final_text: None,
            backend_error_message: None,
            selection_failure_message: None,
        })
        .expect("completion maps");
    assert_eq!(
        completion
            .data
            .as_ref()
            .and_then(|data| data.get("schema"))
            .and_then(Value::as_str),
        Some(CAP_SESSION_HANDLE_V1)
    );
    assert_eq!(
        completion
            .data
            .as_ref()
            .and_then(|data| data.get("session"))
            .and_then(|session| session.get("id"))
            .and_then(Value::as_str),
        Some("thread-1")
    );
}

#[test]
fn whitespace_thread_id_is_treated_as_unknown() {
    let adapter = test_adapter();

    let seen = adapter.map_event(CodexBackendEvent::Thread(Box::new(parse_thread_event(
        r#"{"type":"thread.started","thread_id":"   "}"#,
    ))));
    assert!(
        !seen
            .iter()
            .any(|ev| handle_schema(ev) == Some(CAP_SESSION_HANDLE_V1)),
        "expected no handle facet emission for whitespace-only thread ids"
    );

    let completion = adapter
        .map_completion(CodexBackendCompletion {
            status: success_exit_status(),
            final_text: None,
            backend_error_message: None,
            selection_failure_message: None,
        })
        .expect("completion maps");
    assert_eq!(completion.data, None);
}

#[test]
fn oversize_thread_id_is_omitted_and_warns_once() {
    let adapter = test_adapter();
    let oversize = "a".repeat(SESSION_HANDLE_ID_BOUND_BYTES + 1);
    let json = format!(r#"{{"type":"thread.started","thread_id":"{oversize}"}}"#);

    let first = adapter.map_event(CodexBackendEvent::Thread(Box::new(parse_thread_event(
        &json,
    ))));
    let second = adapter.map_event(CodexBackendEvent::Thread(Box::new(parse_thread_event(
        &json,
    ))));

    let seen: Vec<AgentWrapperEvent> = first.into_iter().chain(second).collect();
    assert!(
        !seen
            .iter()
            .any(|ev| handle_schema(ev) == Some(CAP_SESSION_HANDLE_V1)),
        "expected oversize ids to be treated as unknown (no facet emission)"
    );

    let warnings: Vec<&AgentWrapperEvent> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Status)
        .filter(|ev| {
            ev.message.as_deref().is_some_and(|message| {
                message.contains(SESSION_HANDLE_OVERSIZE_WARNING_MARKER)
                    && message.contains("len_bytes=1025")
                    && !message.contains(&oversize)
            })
        })
        .collect();
    assert_eq!(warnings.len(), 1);

    let completion = adapter
        .map_completion(CodexBackendCompletion {
            status: success_exit_status(),
            final_text: None,
            backend_error_message: None,
            selection_failure_message: None,
        })
        .expect("completion maps");
    assert_eq!(completion.data, None);
}

#[test]
fn synthetic_status_is_emitted_if_id_first_seen_on_non_status_event() {
    let adapter = test_adapter();

    let seen = adapter.map_event(CodexBackendEvent::Thread(Box::new(parse_thread_event(
        r#"{"type":"item.started","thread_id":"thread-1","turn_id":"turn-1","item_id":"item-1","item_type":"command_execution","content":{"command":"echo hi"}}"#,
    ))));

    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].kind, AgentWrapperEventKind::ToolCall);
    assert_eq!(tool_schema(&seen[0]), Some(TOOLS_FACET_SCHEMA));
    assert_eq!(seen[1].kind, AgentWrapperEventKind::Status);
    assert_eq!(handle_schema(&seen[1]), Some(CAP_SESSION_HANDLE_V1));
    assert_eq!(
        seen[1]
            .data
            .as_ref()
            .and_then(|data| data.get("session"))
            .and_then(|session| session.get("id"))
            .and_then(Value::as_str),
        Some("thread-1")
    );
}
