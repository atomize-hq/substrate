#![cfg(unix)]

#[path = "common.rs"]
mod common;

use common::doctor_fixture::DoctorFixture;
use serde_json::{json, Value};

#[test]
fn health_json_reports_world_deps_missing_and_world_backend_status() {
    let fixture = DoctorFixture::new(
        r#"version: 2
managers:
  - name: HealthyManager
    priority: 10
    detect:
      script: "exit 0"
    init:
      shell: |
        export HEALTHY_MARKER=1
    repair_hint: |
      export HEALTHY_MARKER=1
  - name: MissingManager
    priority: 5
    detect:
      files:
        - "/nonexistent/path"
    init:
      shell: |
        export MISSING_MARKER=1
    repair_hint: |
      export MISSING_MARKER=1
"#,
    );

    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-linux",
        "ok": false,
        "error": "overlay missing"
    }));

    fixture.write_world_deps_fixture(json!({
        "schema_version": 1,
        "cwd": fixture.home(),
        "inventory_packages": 1,
        "inventory_bundles": 0,
        "inventory_mode": "merged",
        "builtins": "enabled",
        "enabled": ["a"],
        "applied": [
            {"kind": "package", "name": "a", "enabled": true, "world": "missing"}
        ]
    }));

    let output = fixture
        .command()
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json");
    assert!(output.status.success(), "health --json should succeed");

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON payload");
    let summary = payload.get("summary").expect("summary object missing");

    assert_eq!(summary["missing_managers"], json!(["MissingManager"]));
    assert_eq!(summary["world_ok"], json!(false));
    assert_eq!(summary["world_deps_missing"], json!(["a"]));
    assert_eq!(summary["ok"], json!(false));
}

#[test]
fn health_human_summary_reports_world_deps_unavailable() {
    let fixture = DoctorFixture::new(
        r#"version: 2
managers:
  - name: SampleManager
    priority: 1
    detect:
      script: "exit 0"
    init:
      shell: |
        export SAMPLE_MANAGER=1
    repair_hint: |
      export SAMPLE_MANAGER=1
"#,
    );

    fixture.write_world_deps_fixture(json!({
        "schema_version": 1,
        "cwd": fixture.home(),
        "inventory_packages": 0,
        "inventory_bundles": 0,
        "inventory_mode": "merged",
        "builtins": "enabled",
        "enabled": [],
        "applied": [],
        "applied_error": "world backend unavailable: stubbed"
    }));

    let output = fixture
        .command()
        .arg("health")
        .output()
        .expect("failed to run substrate health");
    assert!(output.status.success(), "health should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("World deps: unavailable"),
        "expected world deps unavailable summary, got: {stdout}"
    );
    assert!(
        stdout.contains("Overall status: attention required"),
        "expected attention required summary, got: {stdout}"
    );
}

#[test]
fn health_json_is_valid_when_world_deps_fixture_is_invalid() {
    let fixture = DoctorFixture::new(
        r#"version: 2
managers:
  - name: SampleManager
    priority: 1
    detect:
      script: "exit 0"
    init:
      shell: |
        export SAMPLE_MANAGER=1
    repair_hint: |
      export SAMPLE_MANAGER=1
"#,
    );

    fixture.write_world_deps_fixture(json!({
        "schema_version": 1,
        "cwd": fixture.home(),
        "inventory_packages": 0,
        "inventory_bundles": 0,
        "inventory_mode": "merged",
        "builtins": "enabled",
        "enabled": "invalid",
        "applied": []
    }));

    let output = fixture
        .command()
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json");
    assert!(output.status.success(), "health --json should succeed");

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON payload");
    let err = payload["summary"]["world_deps_error"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        err.contains("invalid world deps fixture"),
        "expected world deps fixture error, got: {err}"
    );
}
