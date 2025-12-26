#![cfg(unix)]

mod support;

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use support::temp_dir;
use tempfile::TempDir;

struct VerifyFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    fake_socket_path: PathBuf,
}

impl VerifyFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-world-verify-");
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        let fake_socket_path = temp.path().join("fake-world.sock");
        fs::create_dir_all(&substrate_home).expect("failed to create fixture SUBSTRATE_HOME");
        Self {
            _temp: temp,
            home,
            substrate_home,
            fake_socket_path,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = support::substrate_shell_driver();
        cmd.arg("world")
            .arg("verify")
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env("SUBSTRATE_WORLD", "enabled")
            .env("SUBSTRATE_WORLD_ENABLED", "1")
            .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
            .env("SUBSTRATE_WORLD_SOCKET", &self.fake_socket_path);
        cmd
    }
}

fn assert_verify_json_schema(payload: &Value) {
    let schema_version = payload
        .get("schema_version")
        .and_then(Value::as_u64)
        .expect("verify json missing schema_version");
    assert_eq!(schema_version, 1, "unexpected schema_version: {payload}");

    payload
        .get("ok")
        .and_then(Value::as_bool)
        .expect("verify json missing ok bool");

    let summary = payload
        .get("summary")
        .expect("verify json missing summary object");
    let total = summary
        .get("total")
        .and_then(Value::as_u64)
        .expect("summary.total missing");
    let passed = summary
        .get("passed")
        .and_then(Value::as_u64)
        .expect("summary.passed missing");
    let failed = summary
        .get("failed")
        .and_then(Value::as_u64)
        .expect("summary.failed missing");
    let skipped = summary
        .get("skipped")
        .and_then(Value::as_u64)
        .expect("summary.skipped missing");

    let checks = payload
        .get("checks")
        .and_then(Value::as_array)
        .expect("verify json missing checks array");
    assert_eq!(
        total as usize,
        checks.len(),
        "summary.total should equal checks length: {payload}"
    );
    assert_eq!(
        (passed + failed + skipped) as usize,
        checks.len(),
        "summary counts should add up to checks length: {payload}"
    );

    let mut ids = Vec::new();
    for check in checks {
        let id = check
            .get("id")
            .and_then(Value::as_str)
            .expect("check missing id string");
        let status = check
            .get("status")
            .and_then(Value::as_str)
            .expect("check missing status string");
        assert!(
            matches!(status, "pass" | "fail" | "skip"),
            "unexpected check status {status:?}: {check}"
        );
        ids.push(id.to_string());
    }

    for expected in ["read_only_project", "full_cage_host_paths"] {
        assert!(
            ids.iter().any(|id| id == expected),
            "verify json missing expected check id {expected:?}: {payload}"
        );
    }
}

#[test]
#[ignore = "requires I6-code: substrate world verify"]
fn world_verify_help_mentions_json_flag() {
    let fixture = VerifyFixture::new();
    let assert = fixture.command().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("--json"),
        "verify help should mention --json: {stdout}"
    );
}

#[test]
#[ignore = "requires I6-code: substrate world verify"]
fn world_verify_json_is_stable_when_world_backend_unavailable() {
    let fixture = VerifyFixture::new();
    let output = fixture
        .command()
        .arg("--json")
        .output()
        .expect("failed to run substrate world verify --json");

    assert_eq!(
        output.status.code(),
        Some(3),
        "expected dependency-unavailable exit code (3) when world backend is missing: status={:?} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout)
        .expect("world verify --json should write a JSON report even on failure");
    assert_verify_json_schema(&payload);
    assert_eq!(
        payload.get("ok").and_then(Value::as_bool),
        Some(false),
        "verify json ok should be false when world backend is missing: {payload}"
    );

    let checks = payload["checks"]
        .as_array()
        .expect("verify checks should be an array");
    assert!(
        checks.iter().all(|check| check["status"] == "skip"),
        "expected every check to be skipped when world backend is missing: {payload}"
    );
}
