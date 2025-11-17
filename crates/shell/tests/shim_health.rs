#![cfg(unix)]

#[path = "common.rs"]
mod common;

use common::doctor_fixture::DoctorFixture;
use serde_json::{json, Value};
use std::fs;

fn sample_manifest() -> &'static str {
    r#"version: 1
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
"#
}

#[test]
fn health_json_reports_summary_details() {
    let fixture = DoctorFixture::new(sample_manifest());
    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-linux",
        "ok": false,
        "error": "overlay missing"
    }));
    fixture.write_world_deps_fixture(json!({
        "manifest": {
            "base": fixture.home().join(".substrate/world-deps.yaml"),
            "overlay": null,
            "overlay_exists": false
        },
        "world_disabled_reason": null,
        "tools": [
            {
                "name": "bun",
                "host_detected": false,
                "provider": "custom",
                "guest": {"status": "missing"}
            }
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
    let summary = payload
        .get("summary")
        .expect("summary object missing")
        .clone();
    assert_eq!(summary["missing_managers"], json!(["MissingManager"]));
    assert_eq!(summary["missing_guest_tools"], json!(["bun"]));
    assert_eq!(summary["world_ok"], json!(false));
    assert_eq!(summary["ok"], json!(false));
    assert!(
        summary["failures"]
            .as_array()
            .expect("failures array")
            .len()
            >= 1
    );

    let shim = payload.get("shim").expect("shim report missing");
    assert!(
        shim.get("world").is_some(),
        "shim report should embed world data"
    );
    assert!(
        shim.get("world_deps").is_some(),
        "shim report should embed world deps data"
    );
}

#[test]
fn health_human_summary_highlights_failures() {
    let fixture = DoctorFixture::new(sample_manifest());
    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-linux",
        "ok": false,
        "error": "overlay missing"
    }));
    fixture.write_world_deps_fixture(json!({
        "manifest": {
            "base": fixture.home().join(".substrate/world-deps.yaml"),
            "overlay": null,
            "overlay_exists": false
        },
        "world_disabled_reason": "install metadata reports world disabled",
        "tools": [
            {
                "name": "bun",
                "host_detected": false,
                "provider": "custom",
                "guest": {"status": "missing"}
            }
        ]
    }));

    let output = fixture
        .command()
        .arg("health")
        .output()
        .expect("failed to run substrate health");
    assert!(output.status.success(), "health command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Managers detected"),
        "summary should include manager counts"
    );
    assert!(
        stdout.contains("World backend: needs attention"),
        "world summary missing"
    );
    assert!(
        stdout.contains("Guest tool sync: missing 1"),
        "guest sync summary missing: {stdout}"
    );
    assert!(
        stdout.contains("Overall status: attention required"),
        "overall status missing"
    );
    assert!(
        stdout.contains("guest missing tools"),
        "failure bullet missing: {stdout}"
    );
}

#[test]
fn health_json_marks_skip_manager_init_and_world_disabled_reason() {
    let fixture = DoctorFixture::new(sample_manifest());
    fixture.write_world_deps_fixture(json!({
        "manifest": {
            "base": fixture.home().join(".substrate/world-deps.yaml"),
            "overlay": fixture.home().join(".substrate/world-deps.local.yaml"),
            "overlay_exists": false
        },
        "world_disabled_reason": "install metadata reports world disabled",
        "tools": []
    }));

    let output = fixture
        .command()
        .env("SUBSTRATE_SKIP_MANAGER_INIT", "1")
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json with skip env");
    assert!(
        output.status.success(),
        "health --json should succeed when manager init skipped"
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON payload");
    assert_eq!(payload["shim"]["skip_all_requested"], json!(true));
    let summary = payload
        .get("summary")
        .expect("summary missing from health payload");
    assert_eq!(summary["skip_manager_init"], json!(true));
    assert_eq!(
        summary["world_disabled_reason"],
        json!("install metadata reports world disabled")
    );
    let failures = summary
        .get("failures")
        .and_then(Value::as_array)
        .expect("failures array missing");
    assert!(
        failures.iter().any(|value| value
            .as_str()
            .map(|line| line.contains("manager init skipped"))
            .unwrap_or(false)),
        "failures missing manager init skip message: {failures:?}"
    );
    assert!(
        failures.iter().any(|value| value
            .as_str()
            .map(|line| line.contains("managers missing detection"))
            .unwrap_or(false)),
        "failures missing missing-managers summary: {failures:?}"
    );
}

#[test]
fn health_json_reports_world_backend_error_and_guest_missing_tools() {
    let fixture = DoctorFixture::new(sample_manifest());
    fs::write(
        fixture.health_dir().join("world_doctor.json"),
        "{not valid json",
    )
    .expect("failed to corrupt world doctor fixture");
    fixture.write_world_deps_fixture(json!({
        "manifest": {
            "base": fixture.home().join(".substrate/world-deps.yaml"),
            "overlay": null,
            "overlay_exists": false
        },
        "world_disabled_reason": null,
        "tools": [
            {
                "name": "mise",
                "host_detected": false,
                "provider": "custom",
                "guest": {
                    "status": "missing",
                    "reason": "install pending"
                }
            }
        ]
    }));

    let output = fixture
        .command()
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json with world failure");
    assert!(output.status.success(), "health --json should succeed");

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON payload");
    let summary = payload
        .get("summary")
        .expect("summary missing from health payload");
    assert_eq!(summary["missing_guest_tools"], json!(["mise"]));
    let world_error = summary
        .get("world_error")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert!(
        world_error.contains("world doctor fixture"),
        "world error should propagate fixture failure details: {world_error}"
    );
    let failures = summary
        .get("failures")
        .and_then(Value::as_array)
        .expect("failures array missing");
    assert!(
        failures.iter().any(|value| value
            .as_str()
            .map(|line| line.contains("world backend error"))
            .unwrap_or(false)),
        "expected failure mentioning world backend error: {failures:?}"
    );
    assert!(
        failures.iter().any(|value| value
            .as_str()
            .map(|line| line.contains("guest missing tools"))
            .unwrap_or(false)),
        "expected failure mentioning guest sync issues: {failures:?}"
    );
}

#[test]
fn health_human_summary_reports_world_deps_error() {
    let fixture = DoctorFixture::new(sample_manifest());
    fs::write(
        fixture.health_dir().join("world_deps.json"),
        "{not valid json",
    )
    .expect("failed to write invalid world deps fixture");

    let output = fixture
        .command()
        .arg("health")
        .output()
        .expect("failed to run substrate health with invalid fixture");
    assert!(
        output.status.success(),
        "health command should succeed even when fixtures are invalid"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("failed to read world deps fixture")
            || stdout.contains("invalid world deps fixture"),
        "world deps error should be printed for invalid fixture: {stdout}"
    );
    assert!(
        stdout.contains("Overall status: attention required"),
        "overall summary should highlight failure when fixtures are invalid: {stdout}"
    );
}
