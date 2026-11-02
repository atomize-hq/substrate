use super::support::*;

#[test]
fn claude_emits_handle_facet_once_when_first_event_is_status() {
    let adapter = new_adapter();

    let out1 = adapter.map_event(ClaudeBackendEvent::Stream(parse_stream_json_fixture(
        SYSTEM_INIT,
    )));
    let facet_events_1: Vec<_> = out1
        .iter()
        .filter(|event| handle_facet_schema(event) == Some(CAP_SESSION_HANDLE_V1))
        .collect();
    assert_eq!(facet_events_1.len(), 1);
    assert_eq!(facet_events_1[0].kind, AgentWrapperEventKind::Status);
    assert_eq!(
        facet_events_1[0]
            .data
            .as_ref()
            .and_then(|v| v.pointer("/session/id"))
            .and_then(|v| v.as_str()),
        Some("sess-1")
    );

    let out2 = adapter.map_event(ClaudeBackendEvent::Stream(parse_stream_json_fixture(
        SYSTEM_OTHER,
    )));
    assert!(
        out2.iter()
            .all(|event| handle_facet_schema(event) != Some(CAP_SESSION_HANDLE_V1)),
        "expected subsequent events to not re-emit the handle facet"
    );
}

#[test]
fn claude_emits_synthetic_status_handle_facet_when_first_event_is_not_status() {
    let adapter = new_adapter();

    let out = adapter.map_event(ClaudeBackendEvent::Stream(parse_stream_json_fixture(
        ASSISTANT_MESSAGE_TEXT,
    )));
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(out[0].text.as_deref(), Some("hello"));

    assert_eq!(out[1].kind, AgentWrapperEventKind::Status);
    assert_eq!(handle_facet_schema(&out[1]), Some(CAP_SESSION_HANDLE_V1));
    assert_eq!(
        out[1]
            .data
            .as_ref()
            .and_then(|v| v.pointer("/session/id"))
            .and_then(|v| v.as_str()),
        Some("sess-1")
    );
}

#[test]
fn claude_completion_attaches_handle_facet_when_id_is_known() {
    let adapter = new_adapter();

    let _events = adapter.map_event(ClaudeBackendEvent::Stream(parse_stream_json_fixture(
        SYSTEM_INIT,
    )));

    let completion = adapter
        .map_completion(super::super::harness::ClaudeBackendCompletion {
            status: success_exit_status(),
            final_text: None,
            backend_error_message: None,
        })
        .expect("completion maps");

    assert_eq!(
        completion
            .data
            .as_ref()
            .and_then(|v| v.get("schema"))
            .and_then(|v| v.as_str()),
        Some(CAP_SESSION_HANDLE_V1)
    );
    assert_eq!(
        completion
            .data
            .as_ref()
            .and_then(|v| v.pointer("/session/id"))
            .and_then(|v| v.as_str()),
        Some("sess-1")
    );
}

#[test]
fn claude_whitespace_session_id_is_treated_as_unknown() {
    let adapter = new_adapter();

    let out = adapter.map_event(ClaudeBackendEvent::Stream(parse_single_line(
        r#"{"type":"system","subtype":"init","session_id":"   "}"#,
    )));
    assert!(
        out.iter()
            .all(|event| handle_facet_schema(event) != Some(CAP_SESSION_HANDLE_V1)),
        "expected whitespace-only session_id to not produce a handle facet"
    );

    let completion = adapter
        .map_completion(super::super::harness::ClaudeBackendCompletion {
            status: success_exit_status(),
            final_text: None,
            backend_error_message: None,
        })
        .expect("completion maps");
    assert!(completion.data.is_none());
}

#[test]
fn claude_oversize_session_id_is_omitted_and_warns_once() {
    let adapter = new_adapter();

    let oversize = "a".repeat(SESSION_HANDLE_ID_BOUND_BYTES + 1);
    let line = format!(r#"{{"type":"system","subtype":"init","session_id":"{oversize}"}}"#);
    let out1 = adapter.map_event(ClaudeBackendEvent::Stream(parse_single_line(&line)));

    assert!(
        out1.iter()
            .all(|event| handle_facet_schema(event) != Some(CAP_SESSION_HANDLE_V1)),
        "expected oversize session_id to omit the handle facet"
    );

    let warnings_1: Vec<_> = out1
        .iter()
        .filter(|event| event.message.as_deref() == Some(SESSION_HANDLE_OVERSIZE_WARNING))
        .collect();
    assert_eq!(warnings_1.len(), 1);
    assert_eq!(warnings_1[0].kind, AgentWrapperEventKind::Status);
    assert!(warnings_1[0].data.is_none());

    let out2 = adapter.map_event(ClaudeBackendEvent::Stream(parse_single_line(&line)));
    assert!(
        out2.iter()
            .all(|event| event.message.as_deref() != Some(SESSION_HANDLE_OVERSIZE_WARNING)),
        "expected oversize warning to be emitted at most once"
    );

    let completion = adapter
        .map_completion(super::super::harness::ClaudeBackendCompletion {
            status: success_exit_status(),
            final_text: None,
            backend_error_message: None,
        })
        .expect("completion maps");
    assert!(completion.data.is_none());
}
