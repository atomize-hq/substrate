#![cfg(unix)]

#[path = "common.rs"]
mod common;

use common::doctor_fixture::DoctorFixture;
use serde_json::{json, Value};

fn write_invalid_workspace_fixture(root: &std::path::Path) {
    let substrate_dir = root.join(".substrate");
    std::fs::create_dir_all(&substrate_dir).expect("create workspace substrate dir");
    std::fs::write(
        substrate_dir.join("workspace.yaml"),
        "world:\n  enabled: [true\n",
    )
    .expect("write invalid workspace.yaml");
}

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
        .arg("--world")
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json");
    assert!(output.status.success(), "health --json should succeed");

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON payload");
    let summary = payload.get("summary").expect("summary object missing");

    assert!(
        payload.get("world_disable_reason").is_none(),
        "enabled health JSON should omit world_disable_reason: {payload:?}"
    );
    assert!(
        payload.get("world_disable_source").is_none(),
        "enabled health JSON should omit world_disable_source: {payload:?}"
    );
    assert_eq!(summary["missing_managers"], json!(["MissingManager"]));
    assert_eq!(summary["world_ok"], json!(false));
    assert_eq!(summary["world_deps_missing"], json!(["a"]));
    assert_eq!(summary["ok"], json!(false));
}

#[test]
fn health_json_uses_disabled_status_contracts_and_omits_disabled_failures() {
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

    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-linux",
        "ok": false,
        "stderr": "this fixture should be ignored when diagnostics are disabled"
    }));
    fixture.write_world_deps_fixture(json!({
        "schema_version": 1,
        "cwd": fixture.home(),
        "inventory_packages": 1,
        "inventory_bundles": 0,
        "inventory_mode": "merged",
        "builtins": "enabled",
        "enabled": ["ignored"],
        "applied": [
            {
                "kind": "package",
                "name": "ignored",
                "enabled": true,
                "world": "missing"
            }
        ]
    }));

    let output = fixture
        .command()
        .arg("--no-world")
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json with --no-world");
    assert!(
        output.status.success(),
        "health --json should succeed in disabled mode"
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON payload");
    let summary = payload.get("summary").expect("summary object missing");

    assert_eq!(
        payload["world_disable_reason"],
        json!("world isolation disabled by CLI flag --no-world")
    );
    assert_eq!(
        payload["world_disable_source"],
        json!({
            "key": "world.enabled",
            "layer": "cli_flag",
            "value_display": false,
            "flag": "--no-world"
        })
    );
    assert!(payload["world_disable_source"].get("env").is_none());
    assert!(payload["world_disable_source"]
        .get("path_display")
        .is_none());
    assert_eq!(summary["world_ok"], Value::Null);
    assert!(
        summary.get("world_error").is_none(),
        "disabled mode should omit world_error: {summary:?}"
    );
    assert!(
        summary.get("world_deps_error").is_none(),
        "disabled mode should omit world_deps_error: {summary:?}"
    );
    assert_eq!(summary["world_deps_missing"], json!([]));
    assert_eq!(summary["world_deps_blocked"], json!([]));
    assert_eq!(summary["ok"], json!(true));
    assert!(
        summary.get("failures").is_none(),
        "disabled mode should not emit failure entries: {summary:?}"
    );
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
        .arg("--world")
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
fn health_human_summary_uses_disabled_contract_lines_without_guidance() {
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

    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-linux",
        "ok": false,
        "stderr": "this fixture should be ignored when diagnostics are disabled"
    }));
    fixture.write_world_deps_fixture(json!({
        "schema_version": 1,
        "cwd": fixture.home(),
        "inventory_packages": 1,
        "inventory_bundles": 0,
        "inventory_mode": "merged",
        "builtins": "enabled",
        "enabled": ["ignored"],
        "applied": [
            {
                "kind": "package",
                "name": "ignored",
                "enabled": true,
                "world": "missing"
            }
        ]
    }));

    let output = fixture
        .command()
        .arg("--no-world")
        .arg("health")
        .output()
        .expect("failed to run substrate health in disabled mode");
    assert!(
        output.status.success(),
        "health should succeed in disabled mode"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let reason_index = stdout
        .find("world isolation disabled by CLI flag --no-world")
        .expect("missing disabled attribution line");
    let guidance_index = stdout
        .find("World backend: disabled")
        .expect("missing disabled world guidance");
    assert!(
        reason_index < guidance_index,
        "expected attribution line before disabled guidance: {stdout}"
    );
    assert!(
        stdout
            .contains("World backend: disabled\n  Next: run `substrate world enable` to provision"),
        "expected disabled world contract lines, got: {stdout}"
    );
    assert!(
        stdout.contains("World deps: skipped (world disabled)"),
        "expected disabled world deps contract lines, got: {stdout}"
    );
    assert!(
        !stdout.contains("substrate world deps current"),
        "disabled mode should not print enabled-world guidance: {stdout}"
    );
    assert!(
        stdout.contains("Overall status: healthy"),
        "disabled mode should not report attention required, got: {stdout}"
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
        .arg("--world")
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

#[test]
fn health_json_exits_2_before_output_on_invalid_workspace_yaml() {
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

    let workspace_root = fixture.home().join("workspace");
    std::fs::create_dir_all(&workspace_root).expect("create workspace root");
    write_invalid_workspace_fixture(&workspace_root);

    let output = fixture
        .command()
        .current_dir(&workspace_root)
        .arg("health")
        .arg("--json")
        .output()
        .expect("failed to run substrate health --json with invalid workspace yaml");

    assert_eq!(
        output.status.code(),
        Some(2),
        "health should exit 2 for invalid workspace yaml: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "health should not emit JSON on config error: stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid YAML") || stderr.contains("failed to read"),
        "stderr should report the config parse failure: {stderr}"
    );
}
