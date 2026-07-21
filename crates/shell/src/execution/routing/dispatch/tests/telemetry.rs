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

fn test_frame_identity(frame_sequence: u64) -> transport_api_types::RuntimeFrameIdentityV1 {
    transport_api_types::RuntimeFrameIdentityV1 {
        schema_version: transport_api_types::RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
        stream_id: "rts_telemetry_fixture".to_string(),
        frame_sequence,
    }
}

fn test_event_identity(event_sequence: u64) -> transport_api_types::RuntimeEventIdentityV1 {
    transport_api_types::RuntimeEventIdentityV1 {
        event_id: format!("evt_telemetry_fixture_{event_sequence}"),
        event_sequence,
    }
}

// Telemetry stream handling
#[test]
#[serial_test::serial]
fn consume_agent_stream_buffer_without_context_suppresses_agent_events() {
    if crate::execution::run_in_bounded_test_subprocess(
        concat!(
            module_path!(),
            "::",
            stringify!(consume_agent_stream_buffer_without_context_suppresses_agent_events)
        ),
        "event_registry",
    ) {
        return;
    }
    let _guard = agent_events::acquire_event_test_guard();
    let rt = Runtime::new().expect("runtime");
    rt.block_on(async {
        let mut rx = init_event_channel();

        let frames = [
            ExecuteStreamFrame::Stdout {
                frame_identity: test_frame_identity(1),
                chunk_b64: BASE64.encode("hello"),
            },
            ExecuteStreamFrame::Stderr {
                frame_identity: test_frame_identity(2),
                chunk_b64: BASE64.encode("oops"),
            },
            ExecuteStreamFrame::Exit {
                frame_identity: test_frame_identity(3),
                event_identity: test_event_identity(1),
                terminal_identity: transport_api_types::RuntimeTerminalIdentityV1::from(
                    &test_event_identity(1),
                ),
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
#[serial_test::serial]
fn consume_agent_stream_buffer_preserves_runtime_event_identity_unchanged() {
    if crate::execution::run_in_bounded_test_subprocess(
        concat!(
            module_path!(),
            "::",
            stringify!(consume_agent_stream_buffer_preserves_runtime_event_identity_unchanged)
        ),
        "event_registry",
    ) {
        return;
    }
    let _guard = agent_events::acquire_event_test_guard();
    let mut rx = init_event_channel();
    let mut event = substrate_common::agent_events::AgentEvent::message(
        "agent",
        "session",
        "run",
        substrate_common::agent_events::MessageEventKind::Status,
        "identified",
    );
    let event_identity = test_event_identity(1);
    event.event_identity = Some(event_identity.clone());
    let frames = [
        ExecuteStreamFrame::Start {
            frame_identity: test_frame_identity(1),
            span_id: "spn_identity".to_string(),
        },
        ExecuteStreamFrame::Event {
            frame_identity: test_frame_identity(2),
            event,
        },
    ];
    let mut buffer = Vec::new();
    for frame in frames {
        buffer.extend(frame.canonical_ndjson_bytes().expect("canonical frame"));
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
    .expect("consume identified stream");

    let forwarded = rx.try_recv().expect("forwarded Event");
    assert_eq!(forwarded.event_identity.as_ref(), Some(&event_identity));
    assert!(exit_code.is_none());
    clear_agent_event_sender();
}

#[test]
fn consume_agent_stream_buffer_rejects_missing_event_identity_without_host_repair() {
    let event = substrate_common::agent_events::AgentEvent::message(
        "agent",
        "session",
        "run",
        substrate_common::agent_events::MessageEventKind::Status,
        "missing identity",
    );
    let frame = ExecuteStreamFrame::Event {
        frame_identity: test_frame_identity(1),
        event,
    };
    let mut buffer = serde_json::to_vec(&frame).expect("serialize malformed Event fixture");
    buffer.push(b'\n');
    let mut exit_code = None;
    let mut scopes_used = Vec::new();
    let mut fs_diff = None;

    let err = consume_agent_stream_buffer(
        "tester",
        &mut buffer,
        &mut exit_code,
        &mut scopes_used,
        &mut fs_diff,
    )
    .expect_err("missing runtime event identity must fail closed");
    assert!(format!("{err:#}").contains("requires event.event_identity"));
    assert!(exit_code.is_none());
}

#[test]
fn stream_exhaustion_and_transport_error_do_not_synthesize_terminal_completion() {
    let mut empty_buffer = Vec::new();
    let mut exit_code = None;
    let mut scopes_used = Vec::new();
    let mut fs_diff = None;
    consume_agent_stream_buffer(
        "tester",
        &mut empty_buffer,
        &mut exit_code,
        &mut scopes_used,
        &mut fs_diff,
    )
    .expect("stream exhaustion is only an observation");
    assert!(exit_code.is_none());

    let error = ExecuteStreamFrame::Error {
        frame_identity: test_frame_identity(1),
        message: "transport lost".to_string(),
    };
    let mut error_buffer = error.canonical_ndjson_bytes().expect("canonical Error");
    assert!(consume_agent_stream_buffer(
        "tester",
        &mut error_buffer,
        &mut exit_code,
        &mut scopes_used,
        &mut fs_diff,
    )
    .is_err());
    assert!(exit_code.is_none());
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
        frame_identity: test_frame_identity(1),
        event_identity: test_event_identity(1),
        terminal_identity: transport_api_types::RuntimeTerminalIdentityV1::from(
            &test_event_identity(1),
        ),
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
