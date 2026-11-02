mod support_paths;

use claude_code::{ClaudeStreamJsonEvent, ClaudeStreamJsonParser};

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(support_paths::claude_code_stream_json_fixtures_dir().join(name))
        .expect("read fixture")
}

fn parse_single_line(name: &str) -> ClaudeStreamJsonEvent {
    let mut parser = ClaudeStreamJsonParser::new();
    let text = read_fixture(name);
    let line = text
        .lines()
        .find(|l| !l.chars().all(|c| c.is_whitespace()))
        .unwrap();
    parser.parse_line(line).unwrap().unwrap()
}

fn assert_session_id_matches_field(ev: &ClaudeStreamJsonEvent, expected: Option<&str>) {
    let from_accessor = ev.session_id();
    assert_eq!(from_accessor, expected, "accessor mismatch for {ev:?}");

    let from_field: Option<&str> = match ev {
        ClaudeStreamJsonEvent::SystemInit { session_id, .. } => Some(session_id.as_str()),
        ClaudeStreamJsonEvent::SystemOther { session_id, .. } => Some(session_id.as_str()),
        ClaudeStreamJsonEvent::UserMessage { session_id, .. } => Some(session_id.as_str()),
        ClaudeStreamJsonEvent::AssistantMessage { session_id, .. } => Some(session_id.as_str()),
        ClaudeStreamJsonEvent::ResultSuccess { session_id, .. } => Some(session_id.as_str()),
        ClaudeStreamJsonEvent::ResultError { session_id, .. } => Some(session_id.as_str()),
        ClaudeStreamJsonEvent::StreamEvent { session_id, .. } => Some(session_id.as_str()),
        ClaudeStreamJsonEvent::Unknown { session_id, .. } => session_id.as_deref(),
    };
    assert_eq!(from_field, expected, "field mismatch for {ev:?}");

    if let Some(expected) = expected {
        let from_accessor = from_accessor.expect("expected Some");
        let from_field = from_field.expect("expected Some");
        assert_eq!(from_accessor, expected);
        assert_eq!(from_field, expected);
        assert_eq!(from_accessor.as_ptr(), from_field.as_ptr());
        assert_eq!(from_accessor.len(), from_field.len());
    }
}

#[test]
fn pinned_variants_return_session_id_and_are_borrowed() {
    let cases = [
        ("system_init.jsonl", "SystemInit"),
        ("system_other.jsonl", "SystemOther"),
        ("user_message.jsonl", "UserMessage"),
        ("assistant_message_text.jsonl", "AssistantMessage"),
        ("result_success.jsonl", "ResultSuccess"),
        ("result_error.jsonl", "ResultError"),
        ("stream_event_text_delta.jsonl", "StreamEvent"),
    ];

    for (fixture, label) in cases {
        let ev = parse_single_line(fixture);
        match (&ev, label) {
            (ClaudeStreamJsonEvent::SystemInit { .. }, "SystemInit")
            | (ClaudeStreamJsonEvent::SystemOther { .. }, "SystemOther")
            | (ClaudeStreamJsonEvent::UserMessage { .. }, "UserMessage")
            | (ClaudeStreamJsonEvent::AssistantMessage { .. }, "AssistantMessage")
            | (ClaudeStreamJsonEvent::ResultSuccess { .. }, "ResultSuccess")
            | (ClaudeStreamJsonEvent::ResultError { .. }, "ResultError")
            | (ClaudeStreamJsonEvent::StreamEvent { .. }, "StreamEvent") => {}
            _ => panic!("fixture {fixture} did not produce expected variant {label}: {ev:?}"),
        }
        assert_session_id_matches_field(&ev, Some("sess-1"));
    }
}

#[test]
fn unknown_with_session_id_fixture_returns_some_and_is_borrowed() {
    let ev = parse_single_line("unknown_outer_type.jsonl");
    assert!(matches!(
        ev,
        ClaudeStreamJsonEvent::Unknown {
            session_id: Some(_),
            ..
        }
    ));
    assert_session_id_matches_field(&ev, Some("sess-1"));
}

#[test]
fn unknown_without_session_id_returns_none() {
    let mut parser = ClaudeStreamJsonParser::new();
    let ev = parser
        .parse_line(r#"{"type":"weird","foo":1}"#)
        .unwrap()
        .unwrap();
    assert!(matches!(
        ev,
        ClaudeStreamJsonEvent::Unknown {
            session_id: None,
            ..
        }
    ));
    assert_session_id_matches_field(&ev, None);
}
