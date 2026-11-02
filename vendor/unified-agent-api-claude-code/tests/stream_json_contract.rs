mod support_paths;

use claude_code::{ClaudeStreamJsonErrorCode, ClaudeStreamJsonEvent, ClaudeStreamJsonParser};

fn read_fixture(name: &str) -> String {
    let path = support_paths::claude_code_stream_json_fixtures_dir().join(name);
    std::fs::read_to_string(path).expect("read fixture")
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

#[test]
fn parses_system_init_and_other() {
    assert!(matches!(
        parse_single_line("system_init.jsonl"),
        ClaudeStreamJsonEvent::SystemInit { .. }
    ));
    assert!(matches!(
        parse_single_line("system_other.jsonl"),
        ClaudeStreamJsonEvent::SystemOther { .. }
    ));
}

#[test]
fn result_discriminator_success_vs_error() {
    assert!(matches!(
        parse_single_line("result_success.jsonl"),
        ClaudeStreamJsonEvent::ResultSuccess { .. }
    ));
    assert!(matches!(
        parse_single_line("result_error.jsonl"),
        ClaudeStreamJsonEvent::ResultError { .. }
    ));
}

#[test]
fn normalize_is_emitted_only_for_result_inconsistency() {
    let mut parser = ClaudeStreamJsonParser::new();
    let line = read_fixture("result_inconsistent_is_error.jsonl");
    let err = parser.parse_line(line.trim()).unwrap_err();
    assert_eq!(err.code, ClaudeStreamJsonErrorCode::Normalize);
}

#[test]
fn unknown_outer_type_is_unknown_event_not_error() {
    assert!(matches!(
        parse_single_line("unknown_outer_type.jsonl"),
        ClaudeStreamJsonEvent::Unknown { .. }
    ));
}

#[test]
fn missing_required_path_for_known_type_is_typedparse() {
    let mut parser = ClaudeStreamJsonParser::new();
    let line = read_fixture("missing_required_path_typedparse.jsonl");
    let err = parser.parse_line(line.trim()).unwrap_err();
    assert_eq!(err.code, ClaudeStreamJsonErrorCode::TypedParse);
}

#[test]
fn stream_event_is_typed_and_preserves_inner_event_type() {
    let ev = parse_single_line("stream_event_text_delta.jsonl");
    match ev {
        ClaudeStreamJsonEvent::StreamEvent { stream, .. } => {
            assert_eq!(stream.event_type, "content_block_delta");
        }
        _ => panic!("expected StreamEvent"),
    }
}

#[test]
fn blank_lines_are_ignored_and_crlf_is_tolerated() {
    let mut parser = ClaudeStreamJsonParser::new();
    let text = read_fixture("blank_lines.jsonl");
    let mut count = 0usize;
    for line in text.lines() {
        if let Ok(Some(_)) = parser.parse_line(line) {
            count += 1;
        }
    }
    assert_eq!(count, 2);

    let mut parser = ClaudeStreamJsonParser::new();
    let text = read_fixture("crlf_lines.jsonl");
    for line in text.lines() {
        let with_crlf = format!("{line}\r");
        let out = parser.parse_line(&with_crlf).unwrap();
        assert!(out.is_some());
    }
}

#[test]
fn parse_json_matches_parse_line_taxonomy_for_typedparse_and_normalize() {
    let mut parser = ClaudeStreamJsonParser::new();

    let typedparse_value = serde_json::json!({"type":"user"});
    let err = parser.parse_json(&typedparse_value).unwrap_err();
    assert_eq!(err.code, ClaudeStreamJsonErrorCode::TypedParse);

    let normalize_value =
        serde_json::json!({"type":"result","subtype":"success","session_id":"s","is_error":true});
    let err = parser.parse_json(&normalize_value).unwrap_err();
    assert_eq!(err.code, ClaudeStreamJsonErrorCode::Normalize);
}
