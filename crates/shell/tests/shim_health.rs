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

fn parity_manifest() -> &'static str {
    r#"version: 1
managers:
  - name: direnv
    priority: 10
    detect:
      files:
        - "/nonexistent/direnv"
    init:
      shell: |
        export DIRENV_MANAGER=1
  - name: asdf
    priority: 10
    detect:
      script: "exit 0"
    init:
      shell: |
        export ASDF_MANAGER=1
  - name: conda
    priority: 10
    detect:
      files:
        - "/nonexistent/conda"
    init:
      shell: |
        export CONDA_MANAGER=1
  - name: pyenv
    priority: 10
    detect:
      script: "exit 0"
    init:
      shell: |
        export PYENV_MANAGER=1
"#
}

fn parity_world_deps_report(fixture: &DoctorFixture) -> Value {
    json!({
        "manifest": {
            "base": fixture.home().join(".substrate/world-deps.yaml"),
            "overlay": null,
            "overlay_exists": false
        },
        "world_disabled_reason": null,
        "tools": [
            {
                "name": "direnv",
                "host_detected": false,
                "provider": "custom",
                "guest": {
                    "status": "missing",
                    "reason": "not installed in world"
                }
            },
            {
                "name": "asdf",
                "host_detected": true,
                "provider": "custom",
                "guest": {
                    "status": "missing",
                    "reason": "sync pending"
                }
            },
            {
                "name": "conda",
                "host_detected": false,
                "provider": "custom",
                "guest": {
                    "status": "present"
                }
            },
            {
                "name": "pyenv",
                "host_detected": true,
                "provider": "custom",
                "guest": {
                    "status": "present"
                }
            }
        ]
    })
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
    let failures = summary["failures"].as_array().expect("failures array");
    assert!(!failures.is_empty(), "expected at least one failure entry");
    let manager_states = summary["manager_states"]
        .as_array()
        .expect("manager state summaries missing");
    assert!(
        manager_states
            .iter()
            .any(|entry| entry["name"] == "HealthyManager" && entry["host_present"] == json!(true)),
        "host presence summary missing HealthyManager: {manager_states:?}"
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
        stdout.contains("world backend"),
        "world backend failure missing: {stdout}"
    );
}

#[test]
fn health_json_surfaces_world_fs_mode_details() {
    let fixture = DoctorFixture::new(sample_manifest());
    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-macos",
        "ok": true,
        "world_fs_mode": "read_only"
    }));

    let output = fixture
        .command()
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json");
    assert!(output.status.success(), "health --json should succeed");

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON payload");
    assert_eq!(
        payload["shim"]["world"]["details"]["world_fs_mode"],
        json!("read_only"),
        "health JSON should preserve world_fs_mode from shim/world doctor snapshot"
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
        !failures.iter().any(|value| value
            .as_str()
            .map(|line| line.contains("managers require world sync"))
            .unwrap_or(false)),
        "parity failures should not be emitted when world checks are disabled: {failures:?}"
    );
    assert!(
        summary.get("attention_required_managers").is_none(),
        "skip env should not mark managers for attention"
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
                "name": "HealthyManager",
                "host_detected": true,
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
    assert_eq!(summary["missing_guest_tools"], json!(["HealthyManager"]));
    assert_eq!(
        summary["attention_required_managers"],
        json!(["HealthyManager"])
    );
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
            .map(|line| line.contains("managers require world sync"))
            .unwrap_or(false)),
        "expected failure mentioning manager parity issues: {failures:?}"
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

#[test]
fn health_summary_classifies_manager_parity_states() {
    let fixture = DoctorFixture::new(parity_manifest());
    fixture.write_world_deps_fixture(parity_world_deps_report(&fixture));

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
        .expect("summary missing from health payload");

    let mut missing_managers: Vec<String> = summary
        .get("missing_managers")
        .and_then(Value::as_array)
        .expect("missing_managers array missing")
        .iter()
        .map(|val| val.as_str().unwrap().to_string())
        .collect();
    missing_managers.sort();
    assert_eq!(missing_managers, vec!["conda", "direnv"]);

    let mut missing_guest_tools: Vec<String> = summary
        .get("missing_guest_tools")
        .and_then(Value::as_array)
        .expect("missing_guest_tools array missing")
        .iter()
        .map(|val| val.as_str().unwrap().to_string())
        .collect();
    missing_guest_tools.sort();
    assert_eq!(missing_guest_tools, vec!["asdf", "direnv"]);
    assert_eq!(summary["attention_required_managers"], json!(["asdf"]));
    assert_eq!(summary["world_only_managers"], json!(["conda"]));

    let states = summary["manager_states"]
        .as_array()
        .expect("manager states missing");
    let host_only = states
        .iter()
        .find(|entry| entry["name"] == "direnv")
        .expect("direnv entry missing");
    assert_eq!(host_only["host_present"], json!(false));
    assert_eq!(host_only["world"]["status"], json!("missing"));
    assert_eq!(host_only["attention_required"], json!(false));
    assert_eq!(host_only["world_only"], json!(false));

    let world_only = states
        .iter()
        .find(|entry| entry["name"] == "conda")
        .expect("conda entry missing");
    assert_eq!(world_only["host_present"], json!(false));
    assert_eq!(world_only["world"]["status"], json!("present"));
    assert_eq!(world_only["world_only"], json!(true));
    assert_eq!(world_only["attention_required"], json!(false));

    let matched = states
        .iter()
        .find(|entry| entry["name"] == "asdf")
        .expect("asdf entry missing");
    assert_eq!(matched["host_present"], json!(true));
    assert_eq!(matched["world"]["status"], json!("missing"));
    assert_eq!(matched["attention_required"], json!(true));
    assert_eq!(matched["world_only"], json!(false));

    let pyenv_state = states
        .iter()
        .find(|entry| entry["name"] == "pyenv")
        .expect("pyenv entry missing");
    assert_eq!(pyenv_state["host_present"], json!(true));
    assert_eq!(pyenv_state["world"]["status"], json!("present"));
    assert_eq!(pyenv_state["attention_required"], json!(false));
    assert_eq!(pyenv_state["world_only"], json!(false));

    assert_eq!(summary["ok"], json!(false));
    let failures = summary["failures"]
        .as_array()
        .expect("failures array missing");
    assert!(
        failures.iter().any(|value| value
            .as_str()
            .map(|line| line.contains("asdf"))
            .unwrap_or(false)),
        "failure summary should call out asdf attention requirements: {failures:?}"
    );
    assert!(
        failures.iter().all(|value| value
            .as_str()
            .map(|line| !line.contains("direnv"))
            .unwrap_or(true)),
        "direnv should not be flagged for attention when missing on host and world: {failures:?}"
    );
}

#[test]
fn health_human_summary_respects_manager_parity_states() {
    let fixture = DoctorFixture::new(parity_manifest());
    fixture.write_world_deps_fixture(parity_world_deps_report(&fixture));

    let output = fixture
        .command()
        .arg("health")
        .output()
        .expect("failed to run substrate health");
    assert!(
        output.status.success(),
        "health command should succeed for parity summary scenario"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Managers detected: 2/4"),
        "manager detection summary missing: {stdout}"
    );
    assert!(
        stdout.contains("Guest tool sync: missing 2 (direnv, asdf)"),
        "guest sync summary missing parity counts: {stdout}"
    );
    assert!(
        stdout.contains("managers require world sync: asdf"),
        "host-only manager should be flagged for world sync attention: {stdout}"
    );
    assert!(
        !stdout.contains("managers require world sync: direnv"),
        "both-missing manager should not be flagged for attention: {stdout}"
    );
    assert!(
        !stdout.contains("managers require world sync: conda"),
        "world-only manager should not be flagged for attention: {stdout}"
    );
    assert!(
        stdout.contains("Overall status: attention required"),
        "overall attention summary missing: {stdout}"
    );
}

#[test]
fn health_json_surfaces_world_fs_mode_details() {
    let fixture = DoctorFixture::new(sample_manifest());
    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-macos",
        "ok": true,
        "world_fs_mode": "read_only"
    }));

    let output = fixture
        .command()
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json");
    assert!(output.status.success(), "health --json should succeed");

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON payload");
    assert_eq!(
        payload["shim"]["world"]["details"]["world_fs_mode"],
        json!("read_only"),
        "health JSON should preserve world_fs_mode from shim/world doctor snapshot"
    );
}
