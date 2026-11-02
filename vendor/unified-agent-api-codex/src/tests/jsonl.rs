use super::*;

#[tokio::test]
async fn json_stream_preserves_order_and_parses_tool_calls() {
    let lines = [
        r#"{"type":"thread.started","thread_id":"thread-1"}"#.to_string(),
        serde_json::to_string(&json!({
            "type": "item.started",
            "thread_id": "thread-1",
            "turn_id": "turn-1",
            "item_id": "item-1",
            "item_type": "mcp_tool_call",
            "content": {
                "server_name": "files",
                "tool_name": "list",
                "status": "running"
            }
        }))
        .unwrap(),
        serde_json::to_string(&json!({
            "type": "item.delta",
            "thread_id": "thread-1",
            "turn_id": "turn-1",
            "item_id": "item-1",
            "item_type": "mcp_tool_call",
            "delta": {
                "result": {"paths": ["foo.rs"]},
                "status": "completed"
            }
        }))
        .unwrap(),
    ];

    let (mut writer, reader) = tokio::io::duplex(4096);
    let (tx, rx) = mpsc::channel(8);
    let forward_handle = tokio::spawn(crate::jsonl::forward_json_events(reader, tx, false, None));

    for line in &lines {
        writer.write_all(line.as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
    }
    writer.shutdown().await.unwrap();

    let stream = crate::jsonl::EventChannelStream::new(rx, None);
    pin_mut!(stream);
    let events: Vec<_> = stream.collect().await;
    forward_handle.await.unwrap().unwrap();

    assert_eq!(events.len(), lines.len(), "events: {events:?}");

    match &events[0] {
        Ok(ThreadEvent::ThreadStarted(event)) => {
            assert_eq!(event.thread_id, "thread-1");
        }
        other => panic!("unexpected first event: {other:?}"),
    }

    match &events[1] {
        Ok(ThreadEvent::ItemStarted(envelope)) => {
            assert_eq!(envelope.thread_id, "thread-1");
            assert_eq!(envelope.turn_id, "turn-1");
            match &envelope.item.payload {
                ItemPayload::McpToolCall(state) => {
                    assert_eq!(state.server_name, "files");
                    assert_eq!(state.tool_name, "list");
                    assert_eq!(state.status, ToolCallStatus::Running);
                }
                other => panic!("unexpected payload: {other:?}"),
            }
        }
        other => panic!("unexpected second event: {other:?}"),
    }

    match &events[2] {
        Ok(ThreadEvent::ItemDelta(delta)) => {
            assert_eq!(delta.item_id, "item-1");
            match &delta.delta {
                ItemDeltaPayload::McpToolCall(call_delta) => {
                    assert_eq!(call_delta.status, ToolCallStatus::Completed);
                    let result = call_delta
                        .result
                        .as_ref()
                        .expect("tool call delta result is captured");
                    assert_eq!(result["paths"][0], "foo.rs");
                }
                other => panic!("unexpected delta payload: {other:?}"),
            }
        }
        other => panic!("unexpected third event: {other:?}"),
    }
}

#[tokio::test]
async fn json_stream_propagates_parse_errors() {
    let (mut writer, reader) = tokio::io::duplex(1024);
    let (tx, rx) = mpsc::channel(4);
    let forward_handle = tokio::spawn(crate::jsonl::forward_json_events(reader, tx, false, None));

    writer
        .write_all(br#"{"type":"thread.started","thread_id":"thread-err"}"#)
        .await
        .unwrap();
    writer.write_all(b"\nthis is not json\n").await.unwrap();
    writer.shutdown().await.unwrap();

    let stream = crate::jsonl::EventChannelStream::new(rx, None);
    pin_mut!(stream);
    let events: Vec<_> = stream.collect().await;
    forward_handle.await.unwrap().unwrap();

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        Ok(ThreadEvent::ThreadStarted(ThreadStarted { ref thread_id, .. }))
            if thread_id == "thread-err"
    ));
    match &events[1] {
        Err(ExecStreamError::Parse { line, .. }) => assert_eq!(line, "this is not json"),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[tokio::test]
async fn json_stream_tees_logs_before_forwarding() {
    let lines = [
        r#"{"type":"thread.started","thread_id":"tee-thread"}"#.to_string(),
        r#"{"type":"turn.started","thread_id":"tee-thread","turn_id":"turn-tee"}"#.to_string(),
    ];

    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("events.log");

    let (mut writer, reader) = tokio::io::duplex(2048);
    let (tx, rx) = mpsc::channel(4);
    let log_sink = crate::jsonl::JsonLogSink::new(log_path.clone())
        .await
        .unwrap();
    let forward_handle = tokio::spawn(crate::jsonl::forward_json_events(
        reader,
        tx,
        false,
        Some(log_sink),
    ));

    let stream = crate::jsonl::EventChannelStream::new(rx, None);
    pin_mut!(stream);

    writer.write_all(lines[0].as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();

    let first = stream.next().await.unwrap().unwrap();
    assert!(matches!(first, ThreadEvent::ThreadStarted(_)));

    let logged = fs::read_to_string(&log_path).await.unwrap();
    assert_eq!(logged, format!("{}\n", lines[0]));

    writer.write_all(lines[1].as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.shutdown().await.unwrap();

    let second = stream.next().await.unwrap().unwrap();
    assert!(matches!(second, ThreadEvent::TurnStarted(_)));
    assert!(stream.next().await.is_none());

    forward_handle.await.unwrap().unwrap();

    let final_log = fs::read_to_string(&log_path).await.unwrap();
    assert_eq!(final_log, format!("{}\n{}\n", lines[0], lines[1]));
}

#[tokio::test]
async fn json_event_log_captures_apply_diff_and_tool_payloads() {
    let diff = "@@ -1 +1 @@\n-fn foo() {}\n+fn bar() {}";
    let lines = vec![
        r#"{"type":"thread.started","thread_id":"log-thread"}"#.to_string(),
        serde_json::to_string(&json!({
            "type": "item.started",
            "thread_id": "log-thread",
            "turn_id": "turn-log",
            "item_id": "apply-1",
            "item_type": "file_change",
            "content": {
                "path": "src/main.rs",
                "change": "apply",
                "diff": diff,
                "stdout": "patched\n"
            }
        }))
        .unwrap(),
        serde_json::to_string(&json!({
            "type": "item.delta",
            "thread_id": "log-thread",
            "turn_id": "turn-log",
            "item_id": "apply-1",
            "item_type": "file_change",
            "delta": {
                "diff": diff,
                "stderr": "warning",
                "exit_code": 2
            }
        }))
        .unwrap(),
        serde_json::to_string(&json!({
            "type": "item.delta",
            "thread_id": "log-thread",
            "turn_id": "turn-log",
            "item_id": "tool-1",
            "item_type": "mcp_tool_call",
            "delta": {
                "result": {"paths": ["a.rs", "b.rs"]},
                "status": "completed"
            }
        }))
        .unwrap(),
    ];

    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("json.log");

    let (mut writer, reader) = tokio::io::duplex(4096);
    let (tx, rx) = mpsc::channel(8);
    let log_sink = crate::jsonl::JsonLogSink::new(log_path.clone())
        .await
        .unwrap();
    let forward_handle = tokio::spawn(crate::jsonl::forward_json_events(
        reader,
        tx,
        false,
        Some(log_sink),
    ));

    for line in &lines {
        writer.write_all(line.as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
    }
    writer.shutdown().await.unwrap();

    let stream = crate::jsonl::EventChannelStream::new(rx, None);
    pin_mut!(stream);
    let events: Vec<_> = stream.collect().await;
    forward_handle.await.unwrap().unwrap();

    assert_eq!(events.len(), lines.len());

    let log_contents = fs::read_to_string(&log_path).await.unwrap();
    assert_eq!(log_contents, lines.join("\n") + "\n");
}

#[tokio::test]
async fn event_channel_stream_times_out_when_idle() {
    let (_tx, rx) = mpsc::channel(1);
    let stream = crate::jsonl::EventChannelStream::new(rx, Some(Duration::from_millis(5)));
    pin_mut!(stream);

    let next = stream.next().await;
    match next {
        Some(Err(ExecStreamError::IdleTimeout { idle_for })) => {
            assert_eq!(idle_for, Duration::from_millis(5));
        }
        other => panic!("expected idle timeout, got {other:?}"),
    }
}

#[test]
fn normalize_stream_infers_missing_thread_and_turn() {
    let mut context = crate::jsonl::StreamContext::default();
    // thread.started establishes thread context
    let thread_line = r#"{"type":"thread.started","thread_id":"thread-1"}"#;
    let thread_event = crate::jsonl::normalize_thread_event(thread_line, &mut context).unwrap();
    match thread_event {
        ThreadEvent::ThreadStarted(t) => assert_eq!(t.thread_id, "thread-1"),
        other => panic!("unexpected event: {other:?}"),
    }
    // turn.started without thread_id should inherit
    let turn_line = r#"{"type":"turn.started","turn_id":"turn-1"}"#;
    let turn_event = crate::jsonl::normalize_thread_event(turn_line, &mut context).unwrap();
    match turn_event {
        ThreadEvent::TurnStarted(t) => {
            assert_eq!(t.thread_id, "thread-1");
            assert_eq!(t.turn_id, "turn-1");
        }
        other => panic!("unexpected event: {other:?}"),
    }
    // item.completed without ids should inherit both
    let item_line =
        r#"{"type":"item.completed","item":{"id":"msg-1","type":"agent_message","text":"hi"}}"#;
    let item_event = crate::jsonl::normalize_thread_event(item_line, &mut context).unwrap();
    match item_event {
        ThreadEvent::ItemCompleted(item) => {
            assert_eq!(item.turn_id, "turn-1");
            assert_eq!(item.thread_id, "thread-1");
            assert_eq!(item.item.item_id, "msg-1");
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn normalize_stream_errors_without_context() {
    let mut context = crate::jsonl::StreamContext::default();
    let line = r#"{"type":"turn.started"}"#;
    let err = crate::jsonl::normalize_thread_event(line, &mut context).unwrap_err();
    match err {
        ExecStreamError::Normalize { .. } => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
