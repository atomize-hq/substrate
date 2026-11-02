use claude_code::{parse_stream_json_lines, StreamJsonLineOutcome};

#[test]
fn parse_stream_json_lines_is_tolerant() {
    let input = r#"
{"type":"ok","n":1}
not json
{"type":"ok","n":2}
"#;

    let out = parse_stream_json_lines(input);
    assert_eq!(out.len(), 3);

    match &out[0] {
        StreamJsonLineOutcome::Ok { value, .. } => assert_eq!(value["n"], 1),
        _ => panic!("expected ok"),
    }
    match &out[1] {
        StreamJsonLineOutcome::Err { line, error } => {
            assert!(line.raw.contains("not json"));
            assert_eq!(error.line_number, line.line_number);
        }
        _ => panic!("expected err"),
    }
    match &out[2] {
        StreamJsonLineOutcome::Ok { value, .. } => assert_eq!(value["n"], 2),
        _ => panic!("expected ok"),
    }
}
