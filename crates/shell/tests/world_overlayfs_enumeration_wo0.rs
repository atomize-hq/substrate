#![cfg(all(unix, target_os = "linux"))]

mod support;

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use support::{get_substrate_binary, temp_dir};
use support::{AgentSocket, SocketResponse};
use tempfile::Builder;

fn setup_isolated_home(temp: &tempfile::TempDir) -> std::path::PathBuf {
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
    let log_content = fs::read_to_string(path).expect("read trace.jsonl");
    log_content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect()
}

fn command_complete_events(events: &[Value]) -> Vec<&Value> {
    events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("command_complete"))
        .collect()
}

#[test]
fn world_doctor_json_includes_strategy_keys_per_adr_0004() {
    let temp = temp_dir("substrate-wo0-world-doctor-");
    let home = setup_isolated_home(&temp);
    let socket_dir = Builder::new()
        .prefix("substrate-wo0-sock-")
        .tempdir_in("/tmp")
        .expect("create wo0 socket tempdir");
    let socket_path: PathBuf = socket_dir.path().join("world-agent.sock");
    let _socket = AgentSocket::start(&socket_path, SocketResponse::Capabilities);

    let output = get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .arg("world")
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("world doctor --json");

    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("world doctor should emit JSON on stdout");

    if payload.get("schema_version").is_some() {
        let strategy = payload
            .get("world")
            .and_then(|world| world.get("world_fs_strategy"))
            .expect("doctor JSON missing world.world_fs_strategy");
        assert_eq!(
            strategy.get("primary").and_then(Value::as_str),
            Some("overlay"),
            "doctor JSON missing world.world_fs_strategy.primary=overlay: {payload:?}"
        );
        assert_eq!(
            strategy.get("fallback").and_then(Value::as_str),
            Some("fuse"),
            "doctor JSON missing world.world_fs_strategy.fallback=fuse: {payload:?}"
        );

        let probe = strategy
            .get("probe")
            .expect("doctor JSON missing world.world_fs_strategy.probe object");
        assert_eq!(
            probe.get("id").and_then(Value::as_str),
            Some("enumeration_v1"),
            "doctor JSON missing probe id: {payload:?}"
        );
        assert_eq!(
            probe.get("probe_file").and_then(Value::as_str),
            Some(".substrate_enum_probe"),
            "doctor JSON missing probe_file: {payload:?}"
        );
        assert!(
            matches!(
                probe.get("result").and_then(Value::as_str),
                Some("pass" | "fail")
            ),
            "doctor JSON probe.result must be pass|fail: {payload:?}"
        );
        assert!(
            probe.get("failure_reason").is_some(),
            "doctor JSON missing probe.failure_reason (string or null): {payload:?}"
        );
    } else {
        assert_eq!(
            payload
                .get("world_fs_strategy_primary")
                .and_then(Value::as_str),
            Some("overlay"),
            "doctor JSON missing world_fs_strategy_primary=overlay: {payload:?}"
        );
        assert_eq!(
            payload
                .get("world_fs_strategy_fallback")
                .and_then(Value::as_str),
            Some("fuse"),
            "doctor JSON missing world_fs_strategy_fallback=fuse: {payload:?}"
        );

        let probe = payload
            .get("world_fs_strategy_probe")
            .expect("doctor JSON missing world_fs_strategy_probe object");
        assert_eq!(
            probe.get("id").and_then(Value::as_str),
            Some("enumeration_v1"),
            "doctor JSON missing probe id: {payload:?}"
        );
        assert_eq!(
            probe.get("probe_file").and_then(Value::as_str),
            Some(".substrate_enum_probe"),
            "doctor JSON missing probe_file: {payload:?}"
        );
        assert!(
            matches!(
                probe.get("result").and_then(Value::as_str),
                Some("pass" | "fail")
            ),
            "doctor JSON probe.result must be pass|fail: {payload:?}"
        );
        assert!(
            probe.get("failure_reason").is_some(),
            "doctor JSON missing probe.failure_reason (string or null): {payload:?}"
        );
    }
}

#[test]
fn trace_command_complete_includes_world_fs_strategy_fields_and_enums() {
    let temp = temp_dir("substrate-wo0-trace-strategy-");
    let home = setup_isolated_home(&temp);
    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &trace_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .arg("-c")
        .arg("echo wo0-trace-strategy")
        .assert()
        .success();

    let events = read_trace(&trace_path);
    let completes = command_complete_events(&events);
    assert!(
        !completes.is_empty(),
        "expected command_complete events in trace: path={}",
        trace_path.display()
    );

    let event = completes[0];
    let primary = event
        .get("world_fs_strategy_primary")
        .and_then(Value::as_str)
        .expect("trace missing world_fs_strategy_primary");
    assert!(
        matches!(primary, "overlay" | "fuse"),
        "world_fs_strategy_primary must be overlay|fuse, got {primary:?}"
    );

    let final_strategy = event
        .get("world_fs_strategy_final")
        .and_then(Value::as_str)
        .expect("trace missing world_fs_strategy_final");
    assert!(
        matches!(final_strategy, "overlay" | "fuse" | "host"),
        "world_fs_strategy_final must be overlay|fuse|host, got {final_strategy:?}"
    );

    let reason = event
        .get("world_fs_strategy_fallback_reason")
        .and_then(Value::as_str)
        .expect("trace missing world_fs_strategy_fallback_reason");
    assert!(
        matches!(
            reason,
            "none"
                | "primary_unavailable"
                | "primary_mount_failed"
                | "primary_probe_failed"
                | "fallback_unavailable"
                | "fallback_mount_failed"
                | "fallback_probe_failed"
                | "world_optional_fallback_to_host"
        ),
        "world_fs_strategy_fallback_reason must be one of the ADR-0004 enums, got {reason:?}"
    );
}
