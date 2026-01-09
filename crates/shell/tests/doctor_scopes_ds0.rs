#![cfg(unix)]

mod support;

use serde_json::Value;
use support::{substrate_shell_driver, AgentSocket, ShellEnvFixture, SocketResponse};
use tempfile::Builder;

fn has_ds0_envelope(payload: &Value) -> bool {
    payload.get("schema_version").and_then(Value::as_u64) == Some(1)
        && payload.get("world_enabled").is_some()
        && payload.get("host").is_some()
}

fn parse_json(stdout: &[u8], label: &str) -> Value {
    serde_json::from_slice(stdout).unwrap_or_else(|err| {
        panic!(
            "{label} should emit valid JSON: {err}\nstdout={}",
            String::from_utf8_lossy(stdout)
        )
    })
}

fn assert_host_doctor_envelope_v1(payload: &Value) {
    assert_eq!(
        payload.get("schema_version").and_then(Value::as_u64),
        Some(1),
        "host doctor schema_version must be 1: {payload}"
    );
    assert!(
        matches!(
            payload.get("platform").and_then(Value::as_str),
            Some("linux" | "macos" | "windows")
        ),
        "host doctor platform must be linux|macos|windows: {payload}"
    );
    payload
        .get("world_enabled")
        .and_then(Value::as_bool)
        .expect("host doctor missing world_enabled bool");
    payload
        .get("ok")
        .and_then(Value::as_bool)
        .expect("host doctor missing ok bool");
    let host = payload.get("host").expect("host doctor missing host block");
    host.get("platform")
        .and_then(Value::as_str)
        .expect("host doctor host.platform missing");
    host.get("ok")
        .and_then(Value::as_bool)
        .expect("host doctor host.ok missing");
}

fn assert_world_doctor_envelope_v1(payload: &Value) {
    assert_eq!(
        payload.get("schema_version").and_then(Value::as_u64),
        Some(1),
        "world doctor schema_version must be 1: {payload}"
    );
    assert!(
        matches!(
            payload.get("platform").and_then(Value::as_str),
            Some("linux" | "macos" | "windows")
        ),
        "world doctor platform must be linux|macos|windows: {payload}"
    );
    payload
        .get("world_enabled")
        .and_then(Value::as_bool)
        .expect("world doctor missing world_enabled bool");
    payload
        .get("ok")
        .and_then(Value::as_bool)
        .expect("world doctor missing ok bool");

    let host = payload
        .get("host")
        .expect("world doctor missing host block");
    host.get("platform")
        .and_then(Value::as_str)
        .expect("world doctor host.platform missing");
    host.get("ok")
        .and_then(Value::as_bool)
        .expect("world doctor host.ok missing");

    let world = payload
        .get("world")
        .expect("world doctor missing world block");
    world
        .get("ok")
        .and_then(Value::as_bool)
        .expect("world doctor world.ok missing");
    world
        .get("status")
        .and_then(Value::as_str)
        .expect("world doctor world.status missing");
}

#[test]
fn host_doctor_help_wiring_is_present() {
    let mut cmd = substrate_shell_driver();
    let output = cmd
        .arg("host")
        .arg("--help")
        .output()
        .expect("substrate host --help");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unrecognized subcommand") || stderr.contains("unknown subcommand") {
            return;
        }
    }

    assert!(
        output.status.success(),
        "substrate host --help should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("doctor"),
        "expected host subcommand to include doctor: {stdout}"
    );
}

#[test]
fn host_doctor_json_matches_envelope_v1_when_available() {
    let fixture = ShellEnvFixture::new();

    let mut cmd = support::substrate_command_for_home(&fixture);
    let output = cmd
        .arg("host")
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("substrate host doctor --json");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unrecognized subcommand") || stderr.contains("unknown subcommand") {
            return;
        }
    }

    let payload = parse_json(&output.stdout, "host doctor --json");
    if !has_ds0_envelope(&payload) {
        return;
    }
    assert_host_doctor_envelope_v1(&payload);
}

#[test]
fn world_doctor_json_matches_envelope_v1_when_available() {
    let fixture = ShellEnvFixture::new();
    let socket_dir = Builder::new()
        .prefix("substrate-ds0-sock-")
        .tempdir_in("/tmp")
        .expect("create ds0 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let _socket = AgentSocket::start(&socket_path, SocketResponse::Capabilities);

    let mut cmd = support::substrate_command_for_home(&fixture);
    cmd.env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path);
    let output = cmd
        .arg("world")
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("substrate world doctor --json");
    assert!(
        output.status.success(),
        "world doctor --json should succeed"
    );

    let payload = parse_json(&output.stdout, "world doctor --json");
    if !has_ds0_envelope(&payload) {
        return;
    }
    assert_world_doctor_envelope_v1(&payload);
}
