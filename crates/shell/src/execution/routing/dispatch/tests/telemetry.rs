use super::*;
use crate::execution::agent_events::{self, clear_agent_event_sender, init_event_channel};
use agent_api_types::ExecuteStreamFrame;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::Value as JsonValue;
use substrate_common::agent_events::AgentEventKind;
use substrate_common::FsDiff;
use tokio::runtime::Runtime;

// Telemetry stream handling
#[test]
fn consume_agent_stream_buffer_emits_agent_events() {
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

        let stdout_event = rx.recv().await.expect("stdout event");
        assert_eq!(stdout_event.kind, AgentEventKind::PtyData);
        assert_eq!(stdout_event.data["chunk"], "hello");
        assert_eq!(stdout_event.data["stream"], "stdout");

        let stderr_event = rx.recv().await.expect("stderr event");
        assert_eq!(stderr_event.kind, AgentEventKind::PtyData);
        assert_eq!(stderr_event.data["chunk"], "oops");
        assert_eq!(stderr_event.data["stream"], "stderr");

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

