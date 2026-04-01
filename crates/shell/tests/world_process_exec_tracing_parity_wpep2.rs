#![cfg(all(unix, target_os = "linux"))]

mod support;

use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use support::{get_substrate_binary, temp_dir, AgentSocket, SocketResponse};
use tempfile::Builder;

fn setup_isolated_home(temp: &tempfile::TempDir) -> PathBuf {
    let home = temp.path().join("home");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims directory");
    fs::write(
        substrate_home.join("config.yaml"),
        "world:\n  enabled: true\n  anchor_mode: workspace\n  anchor_path: \"\"\n  caged: false\n\npolicy:\n  mode: observe\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    )
    .expect("write default config");
    home
}

fn read_trace(path: &std::path::Path) -> Vec<Value> {
    fs::read_to_string(path)
        .expect("read trace log")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse trace line"))
        .collect()
}

fn shell_command_complete(events: &[Value]) -> &Value {
    events
        .iter()
        .find(|event| {
            event.get("component").and_then(Value::as_str) == Some("shell")
                && event.get("event_type").and_then(Value::as_str) == Some("command_complete")
                && event.get("exit_code").is_some()
        })
        .expect("shell command_complete summary")
}

#[test]
fn world_command_complete_preserves_ptrace_not_permitted_diagnostics() {
    let temp = temp_dir("substrate-wpep2-ptrace-denied-");
    let home = setup_isolated_home(&temp);
    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_dir = Builder::new()
        .prefix("substrate-wpep2-ptrace-denied-sock-")
        .tempdir_in("/tmp")
        .expect("create socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecuteWithProcessEvents {
            stdout: "ok\n".to_string(),
            stderr: String::new(),
            exit: 0,
            scopes: vec![],
            process_events: vec![],
            process_events_status: "unavailable".to_string(),
            process_events_reason: Some("ptrace_not_permitted".to_string()),
            process_events_dropped: None,
        },
    );

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &trace_path)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .arg("-c")
        .arg("printf wpep2-ptrace-denied")
        .assert()
        .success();

    let events = read_trace(&trace_path);
    let complete = shell_command_complete(&events);
    assert_eq!(
        complete
            .get("process_events_status")
            .and_then(Value::as_str),
        Some("unavailable")
    );
    assert_eq!(
        complete
            .get("process_events_reason")
            .and_then(Value::as_str),
        Some("ptrace_not_permitted")
    );
    assert!(
        complete.get("process_events_dropped").is_none(),
        "unavailable ptrace degrade should not report dropped count: {complete:?}"
    );
}

#[test]
fn world_process_events_preserve_linux_capture_fields_and_explicit_argv_omission() {
    let temp = temp_dir("substrate-wpep2-linux-events-");
    let home = setup_isolated_home(&temp);
    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_dir = Builder::new()
        .prefix("substrate-wpep2-linux-events-sock-")
        .tempdir_in("/tmp")
        .expect("create socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecuteWithProcessEvents {
            stdout: "ok\n".to_string(),
            stderr: String::new(),
            exit: 0,
            scopes: vec![],
            process_events: vec![
                json!({
                    "ts": "2026-04-01T00:00:00Z",
                    "ts_unix_ns": 1_743_465_600_000_000_000u64,
                    "event_type": "world_process_start",
                    "component": "world-agent",
                    "session_id": "ses_wpep2",
                    "world_id": "wld_demo",
                    "pid": 101,
                    "ppid": 100,
                    "cwd": "/project",
                    "argv_omitted": true,
                    "parent_span": "spn_linux_parent",
                    "parent_cmd_id": "cmd_linux_parent"
                }),
                json!({
                    "ts": "2026-04-01T00:00:01Z",
                    "ts_unix_ns": 1_743_465_601_000_000_000u64,
                    "event_type": "world_process_exit",
                    "component": "world-agent",
                    "session_id": "ses_wpep2",
                    "world_id": "wld_demo",
                    "pid": 101,
                    "ppid": 100,
                    "cwd": "/project",
                    "argv_omitted": true,
                    "parent_span": "spn_linux_parent",
                    "parent_cmd_id": "cmd_linux_parent",
                    "exit_code": 0,
                    "duration_ms": 12
                }),
            ],
            process_events_status: "ok".to_string(),
            process_events_reason: None,
            process_events_dropped: None,
        },
    );

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &trace_path)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .arg("-c")
        .arg("printf wpep2-linux-events")
        .assert()
        .success();

    let events = read_trace(&trace_path);
    let process_events: Vec<&Value> = events
        .iter()
        .filter(|event| {
            matches!(
                event.get("event_type").and_then(Value::as_str),
                Some("world_process_start" | "world_process_exit")
            )
        })
        .collect();

    assert_eq!(
        process_events.len(),
        2,
        "expected start+exit events: {events:?}"
    );
    for event in &process_events {
        assert_eq!(
            event.get("component").and_then(Value::as_str),
            Some("world-agent")
        );
        assert_eq!(
            event.get("argv_omitted").and_then(Value::as_bool),
            Some(true),
            "argv omission must be explicit for every emitted process event: {event:?}"
        );
        assert!(
            event.get("argv").is_none(),
            "WPEP2 events should omit argv payloads when argv_omitted=true: {event:?}"
        );
        assert_eq!(
            event.get("parent_span").and_then(Value::as_str),
            Some("spn_linux_parent")
        );
        assert_eq!(
            event.get("parent_cmd_id").and_then(Value::as_str),
            Some("cmd_linux_parent")
        );
    }

    let complete = shell_command_complete(&events);
    assert_eq!(
        complete
            .get("process_events_status")
            .and_then(Value::as_str),
        Some("ok")
    );
    assert!(
        complete.get("process_events_reason").is_none(),
        "ok summaries should not report degrade reasons: {complete:?}"
    );
    assert!(
        complete.get("process_events_dropped").is_none(),
        "ok summaries should not report dropped counts: {complete:?}"
    );
}
