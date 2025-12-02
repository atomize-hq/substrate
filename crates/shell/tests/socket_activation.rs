#![cfg(all(unix, target_os = "linux"))]

mod support;

use assert_cmd::Command;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};
use tempfile::{Builder, TempDir};

struct ActivationFixture {
    shell: ShellEnvFixture,
    _socket_dir: TempDir,
    socket_path: PathBuf,
}

impl ActivationFixture {
    fn new() -> Self {
        let shell = ShellEnvFixture::new();
        let socket_dir = Builder::new()
            .prefix("substrate-activation-sock-")
            .tempdir_in("/tmp")
            .expect("failed to create socket tempdir");
        let socket_path = socket_dir.path().join("substrate.sock");
        Self {
            shell,
            _socket_dir: socket_dir,
            socket_path,
        }
    }

    fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    fn socket_path_str(&self) -> String {
        self.socket_path().to_string_lossy().into_owned()
    }

    fn base_command(&self) -> Command {
        let mut cmd = substrate_command_for_home(&self.shell);
        let substrate_home = self.shell.home().join(".substrate");
        cmd.env("SUBSTRATE_HOME", &substrate_home)
            .env("SUBSTRATE_WORLD_SOCKET", &self.socket_path)
            .env("SUBSTRATE_WORLD", "enabled")
            .env("SUBSTRATE_WORLD_ENABLED", "1");
        cmd
    }

    fn seed_shims(&self) {
        let shims_dir = self.shell.home().join(".substrate").join("shims");
        fs::create_dir_all(&shims_dir).expect("failed to create shims dir");
        let version_file = shims_dir.join(".version");
        let payload = json!({
            "version": env!("CARGO_PKG_VERSION"),
            "deployed_at": {
                "secs_since_epoch": 1_700_000_000u64,
                "nanos_since_epoch": 0u32
            },
            "commands": ["git", "cargo"]
        });
        fs::write(
            &version_file,
            serde_json::to_string_pretty(&payload).unwrap(),
        )
        .expect("failed to write version file");
    }
}

fn socket_field(payload: &Value) -> &Value {
    payload
        .get("world_socket")
        .or_else(|| payload.get("agent_socket"))
        .expect("socket activation field missing in payload")
}

#[test]
fn world_doctor_reports_socket_activation_in_json() {
    let fixture = ActivationFixture::new();
    let _socket = AgentSocket::start(fixture.socket_path(), SocketResponse::Capabilities);

    let assert = fixture
        .base_command()
        .arg("world")
        .arg("doctor")
        .arg("--json")
        .assert()
        .success();
    let payload: Value = serde_json::from_slice(&assert.get_output().stdout).expect("doctor json");
    let socket = socket_field(&payload);
    assert_eq!(
        socket.get("mode").and_then(Value::as_str),
        Some("socket_activation"),
        "doctor output missing socket activation mode: {payload:?}"
    );
    assert_eq!(
        socket.get("path").and_then(Value::as_str),
        Some(fixture.socket_path().to_string_lossy().as_ref()),
        "doctor output missing socket path"
    );
}

#[test]
fn world_doctor_fails_when_socket_activation_unresponsive() {
    let fixture = ActivationFixture::new();
    let _socket = AgentSocket::start(fixture.socket_path(), SocketResponse::Silent);

    let assert = fixture
        .base_command()
        .arg("world")
        .arg("doctor")
        .arg("--json")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("socket activation") || stderr.contains("world-agent readiness"),
        "expected socket activation error messaging in doctor stderr: {stderr}"
    );
}

#[test]
fn shim_status_text_includes_socket_activation_details() {
    let fixture = ActivationFixture::new();
    fixture.seed_shims();
    let _socket = AgentSocket::start(fixture.socket_path(), SocketResponse::Capabilities);

    let assert = fixture
        .base_command()
        .arg("--shim-status")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("World socket: socket activation"),
        "shim status text missing socket activation summary: {stdout}"
    );
    assert!(
        stdout.contains(&fixture.socket_path_str()),
        "shim status text missing socket path: {stdout}"
    );
}

#[test]
fn shim_status_json_reports_socket_activation_mode() {
    let fixture = ActivationFixture::new();
    fixture.seed_shims();
    let _socket = AgentSocket::start(fixture.socket_path(), SocketResponse::Capabilities);

    let assert = fixture
        .base_command()
        .arg("--shim-status-json")
        .assert()
        .success();
    let payload: Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("shim status json");
    let socket = socket_field(&payload);
    assert_eq!(
        socket.get("mode").and_then(Value::as_str),
        Some("socket_activation"),
        "shim status JSON missing activation mode"
    );
    assert_eq!(
        socket.get("path").and_then(Value::as_str),
        Some(fixture.socket_path().to_string_lossy().as_ref()),
        "shim status JSON missing socket path"
    );
}

#[test]
fn shim_status_json_marks_manual_mode_when_socket_absent() {
    let fixture = ActivationFixture::new();
    fixture.seed_shims();
    let assert = fixture
        .base_command()
        .arg("--shim-status-json")
        .assert()
        .success();
    let payload: Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("shim status json");
    let socket = socket_field(&payload);
    assert_eq!(
        socket.get("mode").and_then(Value::as_str),
        Some("manual"),
        "shim status JSON should report manual mode when no socket is present"
    );
}
