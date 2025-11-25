#![cfg(unix)]

#[path = "common.rs"]
mod common;

use common::doctor_fixture::DoctorFixture;
use serde_json::{json, Value};
use std::fs;

#[test]
fn shim_doctor_human_mode_reports_status_and_path_diagnostics() {
    let manifest = r#"version: 1
managers:
  - name: DetectedManager
    priority: 10
    detect:
      script: "exit 0"
    init:
      shell: |
        export DETECTED_MARKER=1
    repair_hint: |
      export DETECTED_MARKER=1
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
"#;
    let fixture = DoctorFixture::new(manifest);
    fixture.write_hint_event(
        "MissingManager",
        "source ~/.substrate_bashenv so MissingManager loads",
        "2025-11-16T00:00:00Z",
    );

    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-linux",
        "ok": false,
        "reason": "overlay missing"
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
                "name": "node",
                "host_detected": true,
                "provider": "apt",
                "guest": {"status": "missing"}
            }
        ]
    }));

    let host_bin = fixture.home().join("host-bin");
    fs::create_dir_all(&host_bin).unwrap();
    let shim_path = fixture.shim_dir().display().to_string();
    let path_value = format!("{}:{}", shim_path, host_bin.display());

    let output = fixture
        .command()
        .env("PATH", &path_value)
        .arg("shim")
        .arg("doctor")
        .output()
        .expect("failed to execute shim doctor");

    assert!(
        output.status.success(),
        "expected shim doctor to succeed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        stdout.contains("DetectedManager") && stdout.contains("MissingManager"),
        "doctor output should list both managers: {stdout}"
    );
    assert!(
        stdout.contains("Host PATH includes Substrate shims: yes"),
        "doctor output should report host PATH shims state: {stdout}"
    );
    assert!(
        stdout.contains(&shim_path),
        "doctor output should mention shim directory {shim_path}: {stdout}"
    );
    assert!(
        stdout.contains("MissingManager") && stdout.contains("source ~/.substrate_bashenv"),
        "doctor output should surface the latest repair hint: {stdout}"
    );
    assert!(
        stdout.contains("World backend") && stdout.contains("needs attention"),
        "doctor output should summarize world backend status: {stdout}"
    );
    assert!(
        stdout.contains("World deps"),
        "doctor output should include world deps header: {stdout}"
    );
}

#[test]
fn shim_doctor_json_reports_tier2_managers() {
    let manifest = r#"version: 1
managers:
  - name: Bun
    priority: 10
    detect:
      script: "exit 0"
    init:
      shell: |
        export BUN_MARKER=1
    repair_hint: |
      curl https://bun.sh/install | bash
  - name: Volta
    priority: 12
    detect:
      script: "exit 0"
    init:
      shell: |
        export VOLTA_MARKER=1
    repair_hint: |
      export VOLTA_HOME="$HOME/.volta"
"#;
    let fixture = DoctorFixture::new(manifest);
    fixture.write_hint_event(
        "Bun",
        "curl https://bun.sh/install | bash",
        "2025-11-17T00:00:00Z",
    );

    let output = fixture
        .command()
        .arg("shim")
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("failed to run shim doctor --json");
    assert!(
        output.status.success(),
        "shim doctor --json should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value = serde_json::from_slice(&output.stdout).expect("doctor output JSON");
    let states = report["states"]
        .as_array()
        .expect("states array missing from report");
    let bun_state = states
        .iter()
        .find(|state| state["name"] == "Bun")
        .expect("bun state missing");
    assert_eq!(bun_state["repair_available"], true);
    assert_eq!(
        bun_state["last_hint"]["hint"],
        "curl https://bun.sh/install | bash"
    );
    let volta_state = states
        .iter()
        .find(|state| state["name"] == "Volta")
        .expect("volta state missing");
    assert_eq!(volta_state["detected"], true);

    let hints = report["hints"]
        .as_array()
        .expect("hints array missing from report");
    assert!(
        hints.iter().any(|hint| hint["name"] == "Bun"),
        "expected Bun hint in report"
    );
    assert_eq!(report["world"]["platform"], json!("test-fixture"));
    assert_eq!(report["world"]["ok"], json!(true));
    let deps_report = report["world_deps"]["report"].clone();
    assert!(deps_report.is_object(), "world deps report missing");
    assert!(
        deps_report["tools"]
            .as_array()
            .expect("tools array missing")
            .is_empty(),
        "default world deps fixture should report zero tools"
    );
}

#[test]
fn shim_doctor_json_mode_surfaces_states_hints_and_path_details() {
    let missing_file = "/nonexistent/path/for/json-test";
    let manifest = format!(
        r#"version: 1
managers:
  - name: JsonDetected
    priority: 1
    detect:
      script: "exit 0"
    init:
      shell: |
        export JSON_DETECTED=1
    repair_hint: |
      export JSON_DETECTED=1
  - name: JsonMissing
    priority: 2
    detect:
      files:
        - "{missing_file}"
    init:
      shell: |
        export JSON_MISSING=1
    repair_hint: |
      export JSON_MISSING=1
"#
    );
    let fixture = DoctorFixture::new(&manifest);
    fixture.write_hint_event(
        "JsonMissing",
        "install JsonMissing locally",
        "2025-11-16T00:00:01Z",
    );
    fixture.write_hint_event(
        "JsonMissing",
        "install JsonMissing locally",
        "2025-11-17T00:00:01Z",
    );

    let host_path = fixture.home().join("bin");
    fs::create_dir_all(&host_path).unwrap();
    let output = fixture
        .command()
        .env("PATH", host_path.display().to_string())
        .arg("shim")
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("failed to execute shim doctor --json");

    assert!(
        output.status.success(),
        "expected shim doctor --json to succeed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value =
        serde_json::from_slice(&output.stdout).expect("doctor output should be valid JSON");
    let states = report
        .get("states")
        .and_then(Value::as_array)
        .expect("states array missing from report");
    let detected = states
        .iter()
        .find(|value| value.get("name").and_then(Value::as_str) == Some("JsonDetected"))
        .expect("JsonDetected state missing");
    assert_eq!(
        detected.get("detected").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        detected.get("reason").and_then(Value::as_str),
        Some("script")
    );
    assert!(
        detected
            .get("snippet")
            .and_then(Value::as_str)
            .map(|snippet| snippet.contains("JSON_DETECTED=1"))
            .unwrap_or(false),
        "detected snippet missing from JSON payload: {detected:?}"
    );
    let missing = states
        .iter()
        .find(|value| value.get("name").and_then(Value::as_str) == Some("JsonMissing"))
        .expect("JsonMissing state missing");
    assert_eq!(
        missing.get("detected").and_then(Value::as_bool),
        Some(false)
    );
    assert!(
        missing.get("reason").is_none() || missing.get("reason") == Some(&Value::Null),
        "missing manager should not have a detection reason: {missing:?}"
    );

    let hints = report
        .get("hints")
        .and_then(Value::as_array)
        .expect("hints array missing from report");
    assert_eq!(hints.len(), 1, "doctor should collapse duplicate hints");
    let hint = &hints[0];
    assert_eq!(
        hint.get("name").and_then(Value::as_str),
        Some("JsonMissing")
    );
    assert_eq!(
        hint.get("hint").and_then(Value::as_str),
        Some("install JsonMissing locally")
    );
    assert!(
        hint.get("last_seen")
            .and_then(Value::as_str)
            .map(|ts| ts.contains("2025-11-17"))
            .unwrap_or(false),
        "last_seen timestamp should reflect the newest log entry: {hint:?}"
    );

    let path = report
        .get("path")
        .and_then(Value::as_object)
        .expect("path diagnostics missing from report");
    assert_eq!(
        path.get("host_contains_shims").and_then(Value::as_bool),
        Some(false),
        "json path diagnostics should note that host PATH omits the shim dir"
    );
    let expected_shim_dir = fixture.shim_dir().display().to_string();
    assert_eq!(
        path.get("shim_dir").and_then(Value::as_str),
        Some(expected_shim_dir.as_str()),
        "json path diagnostics should point at the expected shim directory"
    );
    assert!(
        report.get("world").is_some(),
        "world snapshot missing from report"
    );
    assert!(
        report
            .get("world_deps")
            .and_then(|value| value.get("report"))
            .is_some(),
        "world deps section missing from report"
    );
}

#[test]
fn shim_doctor_reports_world_backend_and_deps_errors() {
    let manifest = r#"version: 1
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
"#;
    let fixture = DoctorFixture::new(manifest);
    fixture.write_world_doctor_fixture(json!({
        "platform": "fixture-linux",
        "ok": false,
        "stderr": "failed to connect to /run/substrate.sock"
    }));
    fixture.write_world_deps_fixture(json!({
        "manifest": {
            "base": fixture.home().join(".substrate/world-deps.yaml"),
            "overlay": null,
            "overlay_exists": false
        },
        "world_disabled_reason": null,
        "tools": "invalid"
    }));

    let json_output = fixture
        .command()
        .arg("shim")
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("failed to run shim doctor --json during error scenario");
    assert!(
        json_output.status.success(),
        "shim doctor --json should succeed even when fixtures contain errors"
    );
    let payload: Value = serde_json::from_slice(&json_output.stdout).expect("doctor output JSON");
    let world = payload
        .get("world")
        .expect("world section missing from doctor JSON");
    assert_eq!(world.get("ok"), Some(&Value::Bool(false)));
    let detail_stderr = world
        .get("details")
        .and_then(Value::as_object)
        .and_then(|details| details.get("stderr"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert!(
        detail_stderr.contains("substrate.sock"),
        "expected socket failure details in world snapshot, got: {detail_stderr}"
    );
    let deps = payload
        .get("world_deps")
        .expect("world deps section missing from doctor JSON");
    assert!(
        deps.get("report").is_none(),
        "world deps report should be omitted when fixture parsing fails: {deps:?}"
    );
    let deps_error = deps
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        deps_error.contains("invalid world deps fixture"),
        "world deps error should mention invalid fixture: {deps_error}"
    );

    let text_output = fixture
        .command()
        .arg("shim")
        .arg("doctor")
        .output()
        .expect("failed to run shim doctor (text) during error scenario");
    assert!(
        text_output.status.success(),
        "shim doctor text run should succeed even when fixtures fail:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&text_output.stdout),
        String::from_utf8_lossy(&text_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&text_output.stdout);
    assert!(
        stdout.contains("World backend:") && stdout.contains("needs attention"),
        "world backend summary missing during error scenario: {stdout}"
    );
    assert!(
        stdout.contains("World deps:"),
        "doctor output should include world deps header even on error: {stdout}"
    );
    assert!(
        stdout.contains("invalid world deps fixture"),
        "world deps error should be printed in human output: {stdout}"
    );
}

#[test]
fn shim_doctor_repair_appends_snippet_and_creates_backup_files() {
    let manifest = r#"version: 1
managers:
  - name: RepairManager
    priority: 1
    detect:
      script: "exit 0"
    init:
      shell: |
        export REPAIR_MANAGER=1
    repair_hint: |
      # Substrate repair for RepairManager
      export REPAIR_MANAGER=1
      [ -s "$HOME/.repair-manager/init.sh" ] && \\
        . "$HOME/.repair-manager/init.sh"
"#;
    let fixture = DoctorFixture::new(manifest);
    let bashenv_path = fixture.bashenv_path();
    fs::write(&bashenv_path, "export EXISTING_VAR=1\n").expect("failed to seed bashenv");

    let backup_path = fixture.home().join(".substrate_bashenv.bak");
    let run_repair = || {
        fixture
            .command()
            .arg("shim")
            .arg("repair")
            .arg("--manager")
            .arg("RepairManager")
            .arg("--yes")
            .output()
            .expect("failed to run shim repair")
    };

    let first = run_repair();
    assert!(
        first.status.success(),
        "expected shim repair to succeed on first run:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );

    let content = fs::read_to_string(&bashenv_path).expect("failed to read bashenv after repair");
    assert!(
        content.contains("export EXISTING_VAR=1"),
        "existing bashenv lines should be preserved: {content}"
    );
    assert!(
        content.contains("Substrate repair for RepairManager"),
        "repair snippet missing from bashenv: {content}"
    );
    assert!(
        content.matches("REPAIR_MANAGER=1").count() == 1,
        "repair snippet should only be appended once per run: {content}"
    );
    assert!(
        content.contains("SUBSTRATE_BASHENV_ACTIVE"),
        "bashenv guard missing after repair: {content}"
    );

    assert!(
        backup_path.exists(),
        "repair command must create a .substrate_bashenv.bak backup"
    );
    let backup_contents = fs::read_to_string(&backup_path).expect("failed to read bashenv backup");
    assert_eq!(
        backup_contents, "export EXISTING_VAR=1\n",
        "backup should capture the pre-repair bashenv contents"
    );

    let second = run_repair();
    assert!(
        second.status.success(),
        "expected shim repair to remain idempotent:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
    let second_contents =
        fs::read_to_string(&bashenv_path).expect("failed to read bashenv after second repair");
    assert_eq!(
        content, second_contents,
        "running repair twice should not duplicate the snippet"
    );
}

#[test]
fn shim_doctor_marks_world_deps_skipped_when_no_world_requested() {
    let manifest = r#"version: 1
managers:
  - name: SkipWorld
    priority: 1
    detect:
      script: "exit 0"
    init:
      shell: "echo skip"
    repair_hint: "skip"
"#;
    let fixture = DoctorFixture::new(manifest);
    let world_deps_fixture = fixture.health_dir().join("world_deps.json");
    fs::remove_file(&world_deps_fixture).expect("fixture world_deps.json should exist");

    let output = fixture
        .command()
        .arg("shim")
        .arg("doctor")
        .arg("--json")
        .arg("--no-world")
        .output()
        .expect("failed to run shim doctor --json --no-world");

    assert!(
        output.status.success(),
        "shim doctor --no-world should succeed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value =
        serde_json::from_slice(&output.stdout).expect("doctor --json output should be valid JSON");
    let deps = report["world_deps"]["report"]
        .as_object()
        .expect("world deps report missing");

    assert_eq!(
        deps.get("world_disabled_reason")
            .and_then(|value| value.as_str()),
        Some("--no-world flag is active")
    );
    assert_eq!(
        deps.get("tools")
            .and_then(|value| value.as_array())
            .map(|tools| tools.len()),
        Some(0)
    );
}
