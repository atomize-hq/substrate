use super::*;
use crate::execution::agent_events::{self, clear_agent_event_sender, init_event_channel};
use crate::execution::routing::dispatch::world_ops::consume_agent_stream_buffer_with_meta;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::Value as JsonValue;
use substrate_common::FsDiff;
use tokio::runtime::Runtime;
use transport_api_types::{
    ExecuteStreamFrame, ProcessEvent, ProcessEventType, ProcessEventsStatus, ProcessTelemetry,
};

// Telemetry stream handling
#[test]
#[serial_test::serial]
fn consume_agent_stream_buffer_without_context_suppresses_agent_events() {
    let _guard = agent_events::acquire_event_test_guard();
    let rt = Runtime::new().expect("runtime");
    rt.block_on(async {
        let mut rx = init_event_channel();

        let frames = [
            ExecuteStreamFrame::Stdout {
                chunk_b64: BASE64.encode("hello"),
            },
            ExecuteStreamFrame::Stderr {
                chunk_b64: BASE64.encode("oops"),
            },
            ExecuteStreamFrame::Exit {
                exit: 0,
                span_id: "spn_test".into(),
                scopes_used: vec!["scope:a".into()],
                fs_diff: None,
                process_telemetry: ProcessTelemetry::default(),
            },
        ];

        let mut buffer = Vec::new();
        for frame in frames {
            let mut line = serde_json::to_vec(&frame).expect("serialize frame");
            line.push(b'\n');
            buffer.extend(line);
        }

        let mut exit_code = None;
        let mut scopes_used = Vec::new();
        let mut fs_diff = None;

        consume_agent_stream_buffer(
            "tester",
            &mut buffer,
            &mut exit_code,
            &mut scopes_used,
            &mut fs_diff,
        )
        .expect("consume stream");

        assert!(
            rx.try_recv().is_err(),
            "no-context stream parsing must not synthesize orchestration-scoped agent events"
        );

        assert_eq!(exit_code, Some(0));
        assert_eq!(scopes_used, vec!["scope:a".to_string()]);
        assert!(fs_diff.is_none());
    });
    clear_agent_event_sender();
}

#[test]
fn parse_fs_diff_from_agent_json() {
    let sample = r#"{
        "exit":0,
        "span_id":"spn_x",
        "stdout_b64":"",
        "stderr_b64":"",
        "scopes_used":["tcp:example.com:443"],
        "fs_diff":{
            "writes":["/tmp/t/a.txt"],
            "mods":[],
            "deletes":[],
            "truncated":false
        }
    }"#;
    let v: JsonValue = serde_json::from_str(sample).unwrap();
    let fd_val = v.get("fs_diff").cloned().unwrap();
    let diff: FsDiff = serde_json::from_value(fd_val).unwrap();
    assert_eq!(diff.writes.len(), 1);
    assert_eq!(diff.writes[0], std::path::PathBuf::from("/tmp/t/a.txt"));
    assert!(diff.mods.is_empty());
    assert!(diff.deletes.is_empty());
    assert!(!diff.truncated);
}

#[test]
fn consume_agent_stream_buffer_captures_process_event_summary() {
    let frame = ExecuteStreamFrame::Exit {
        exit: 0,
        span_id: "spn_proc".into(),
        scopes_used: vec![],
        fs_diff: None,
        process_telemetry: ProcessTelemetry {
            process_events: vec![ProcessEvent {
                event_type: ProcessEventType::WorldProcessExit,
                ts: "2026-04-01T00:00:01Z".into(),
                ts_unix_ns: 1_743_465_601_000_000_000,
                session_id: "ses_proc".into(),
                world_id: "wld_proc".into(),
                pid: 99,
                ppid: 10,
                cwd: "/project".into(),
                parent_span: "spn_parent".into(),
                parent_cmd_id: Some("cmd_proc".into()),
                argv: None,
                argv_omitted: Some(true),
                exe: None,
                exit_code: Some(0),
                signal: None,
                duration_ms: Some(12),
                env: None,
            }],
            process_events_status: ProcessEventsStatus::Truncated,
            process_events_reason: Some("capture_overflow".into()),
            process_events_dropped: Some(4),
            process_events_max: None,
            process_events_backend: None,
            process_events_error: None,
        },
    };

    let mut buffer = serde_json::to_vec(&frame).expect("serialize frame");
    buffer.push(b'\n');

    let mut exit_code = None;
    let mut scopes_used = Vec::new();
    let mut fs_diff = None;
    let mut fs_strategy = None;
    let mut process_telemetry = ProcessTelemetry::default();

    consume_agent_stream_buffer_with_meta(
        "tester",
        &mut buffer,
        &mut None,
        &mut exit_code,
        &mut scopes_used,
        &mut fs_diff,
        &mut fs_strategy,
        &mut process_telemetry,
    )
    .expect("consume stream");

    assert_eq!(exit_code, Some(0));
    assert_eq!(
        process_telemetry.process_events_status,
        ProcessEventsStatus::Truncated
    );
    assert_eq!(
        process_telemetry.process_events_reason.as_deref(),
        Some("capture_overflow")
    );
    assert_eq!(process_telemetry.process_events_dropped, Some(4));
    assert_eq!(process_telemetry.process_events.len(), 1);
}
