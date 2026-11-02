use codex::{find_rollout_file_by_id, rollout_jsonl_reader, RolloutEvent, RolloutJsonlParser};

#[test]
fn parses_rollout_lines_and_preserves_unknown_types() {
    let jsonl = r#"
{"timestamp":"2026-02-08T03:03:57.000Z","type":"session_meta","payload":{"id":"sess-1","cli_version":"0.99.0","cwd":"/tmp/demo","base_instructions":{"text":"hello"}}}
{"timestamp":"2026-02-08T03:03:57.001Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":123}}}}
{"timestamp":"2026-02-08T03:03:57.002Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"hi"}]}}
{"timestamp":"2026-02-08T03:03:57.003Z","type":"response_item","payload":{"type":"function_call","name":"exec_command","arguments":"{\"cmd\":\"echo hi\"}","call_id":"call_1"}}
{"timestamp":"2026-02-08T03:03:57.004Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_1","output":"ok"}}
{"timestamp":"2026-02-08T03:03:57.005Z","type":"mystery","payload":{"x":1}}
"#;

    let cursor = std::io::Cursor::new(jsonl);
    let records: Vec<_> = rollout_jsonl_reader(cursor).collect();
    assert_eq!(records.len(), 6);

    let mut ok = 0usize;
    for record in records {
        let event = record.outcome.unwrap();
        ok += 1;
        match event {
            RolloutEvent::SessionMeta(meta) => {
                assert_eq!(meta.payload.id.as_deref(), Some("sess-1"));
            }
            RolloutEvent::EventMsg(msg) => {
                assert_eq!(msg.payload.kind.as_deref(), Some("token_count"));
            }
            RolloutEvent::ResponseItem(item) => {
                assert!(item.payload.kind.is_some());
            }
            RolloutEvent::Unknown(unknown) => {
                assert_eq!(unknown.record_type, "mystery");
            }
        }
    }
    assert_eq!(ok, 6);
}

#[test]
fn parse_line_tolerates_trailing_crlf_carriage_return() {
    let mut parser = RolloutJsonlParser::new();
    let line =
        "{\"type\":\"event_msg\",\"payload\":{\"type\":\"agent_message\",\"message\":\"hi\"}}\r";
    let parsed = parser.parse_line(line).unwrap().unwrap();
    match parsed {
        RolloutEvent::EventMsg(msg) => {
            assert_eq!(msg.payload.kind.as_deref(), Some("agent_message"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn finds_rollout_file_by_session_meta_id_even_when_filename_does_not_match() {
    let root = tempfile::tempdir().expect("tempdir");
    let sessions = root
        .path()
        .join("sessions")
        .join("2026")
        .join("02")
        .join("08");
    std::fs::create_dir_all(&sessions).expect("create sessions");

    let target_id = "019c363e-14ca-7af0-9639-3a31d57ea5d3";
    let path = sessions.join("rollout-not-containing-id.jsonl");
    std::fs::write(
        &path,
        format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{target_id}\",\"cli_version\":\"0.98.0\"}}}}\n"
        ),
    )
    .expect("write");

    let found = find_rollout_file_by_id(root.path(), target_id).expect("find file");
    assert_eq!(found, path);
}
