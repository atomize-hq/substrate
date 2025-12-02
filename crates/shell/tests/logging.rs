#![cfg(unix)]

mod support;

use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use support::{get_substrate_binary, temp_dir, AgentSocket, SocketResponse};
use tempfile::Builder;

#[test]
fn test_command_start_finish_json_roundtrip() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("echo test")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    let lines: Vec<&str> = log_content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    let events: Vec<Value> = lines
        .iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let shell_starts: Vec<&Value> = events
        .iter()
        .filter(|e| e["event_type"] == "command_start" && e["component"] == "shell")
        .collect();
    let shell_completes: Vec<&Value> = events
        .iter()
        .filter(|e| e["event_type"] == "command_complete" && e["component"] == "shell")
        .collect();

    let (starts, completes, component_label) =
        if !shell_starts.is_empty() && !shell_completes.is_empty() {
            (shell_starts, shell_completes, "shell")
        } else {
            let shim_starts: Vec<&Value> = events
                .iter()
                .filter(|e| {
                    e["event_type"] == "command_start"
                        && e["component"] == "shim"
                        && e["command"] == "echo test"
                })
                .collect();
            let shim_completes: Vec<&Value> = events
                .iter()
                .filter(|e| {
                    e["event_type"] == "command_complete"
                        && e["component"] == "shim"
                        && e["command"] == "echo test"
                })
                .collect();
            (shim_starts, shim_completes, "shim")
        };

    assert_eq!(
        starts.len(),
        1,
        "Expected exactly one command_start event from {component_label}"
    );
    assert_eq!(
        completes.len(),
        1,
        "Expected exactly one command_complete event from {component_label}"
    );

    let start_event = starts[0];
    assert_eq!(start_event["command"], "echo test");
    assert!(start_event["session_id"].is_string());
    assert!(start_event["cmd_id"].is_string());

    let complete_event = completes[0];
    match component_label {
        "shell" => {
            assert_eq!(complete_event["exit_code"], 0);
            assert!(complete_event["duration_ms"].is_number());
        }
        "shim" => {
            assert_eq!(complete_event["exit"], 0);
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_log_directory_creation() {
    let temp = temp_dir("substrate-test-");
    let nested_log = temp.path().join("subdir").join("logs").join("trace.jsonl");

    assert!(!nested_log.parent().unwrap().exists());

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &nested_log)
        .arg("-c")
        .arg("true")
        .assert()
        .success();

    assert!(nested_log.exists());
    assert!(fs::read_to_string(&nested_log)
        .unwrap()
        .contains("command_start"));
}

#[test]
fn test_pipe_mode_detection() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .write_stdin("echo piped\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("piped"));

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("\"mode\":\"pipe\""));
}

#[test]
fn test_log_rotation() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    let large_content = "x".repeat(2 * 1024 * 1024);
    fs::write(&log_file, &large_content).unwrap();

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .env("TRACE_LOG_MAX_MB", "1")
        .arg("-c")
        .arg("echo test")
        .assert()
        .success();

    let rotated = log_file.with_extension("jsonl.1");
    assert!(rotated.exists());
    assert_eq!(
        fs::read_to_string(&rotated).unwrap().len(),
        large_content.len()
    );

    let new_content = fs::read_to_string(&log_file).unwrap();
    assert!(
        new_content.len() < 8192,
        "New log file should be much smaller than original. Size: {}",
        new_content.len()
    );
    assert!(new_content.contains("echo test"));
}

#[test]
fn test_pty_field_in_logs() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("echo test")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("\"pty\":false"));
}

#[test]
fn test_process_group_signal_handling() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("sleep 0.1 | cat")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("command_complete"));
}

#[test]
#[cfg(target_os = "linux")]
fn command_logs_include_socket_activation_flag() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");
    let socket_temp = Builder::new()
        .prefix("substrate-activation-")
        .tempdir_in("/tmp")
        .expect("failed to create socket tempdir");
    let socket_path = socket_temp.path().join("activation.sock");
    let _socket = AgentSocket::start(&socket_path, SocketResponse::Capabilities);

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .arg("-c")
        .arg("true")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    let events: Vec<Value> = log_content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let start = events
        .iter()
        .find(|event| event["event_type"] == "command_start" && event["component"] == "shell")
        .expect("missing shell command_start event");
    assert_eq!(
        start.get("socket_activation"),
        Some(&Value::Bool(true)),
        "expected socket_activation flag in telemetry event: {start:?}"
    );
}

#[test]
#[cfg(target_os = "linux")]
fn command_logs_mark_manual_mode_without_socket_activation() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("true")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    let events: Vec<Value> = log_content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let start = events
        .iter()
        .find(|event| event["event_type"] == "command_start" && event["component"] == "shell")
        .expect("missing shell command_start event");
    assert_eq!(
        start.get("socket_activation"),
        Some(&Value::Bool(false)),
        "manual runs should report socket_activation=false: {start:?}"
    );
}
