#![cfg(unix)]

mod support;

use serde_json::json;
use serde_json::Value;
use support::{substrate_shell_driver, AgentSocket, ShellEnvFixture, SocketResponse};
use tempfile::Builder;

#[cfg(target_os = "linux")]
const GUARD_MISSING_REASON: &str =
    "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules";

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

fn stdout_string(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn attribution_line(stdout: &str) -> Option<&str> {
    stdout
        .lines()
        .find(|line| line.contains("world isolation disabled by"))
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

fn assert_disable_attribution_fields(payload: &Value, expected_reason: &str, expected_layer: &str) {
    assert_eq!(
        payload.get("world_disable_reason").and_then(Value::as_str),
        Some(expected_reason),
        "unexpected world_disable_reason: {payload}"
    );
    let source = payload
        .get("world_disable_source")
        .expect("missing world_disable_source");
    assert_eq!(
        source.get("key").and_then(Value::as_str),
        Some("world.enabled")
    );
    assert_eq!(
        source.get("layer").and_then(Value::as_str),
        Some(expected_layer)
    );
    assert_eq!(
        source.get("value_display").and_then(Value::as_bool),
        Some(false)
    );
}

fn assert_disable_attribution_omitted(payload: &Value) {
    assert!(
        payload.get("world_disable_reason").is_none(),
        "enabled payload should omit world_disable_reason: {payload}"
    );
    assert!(
        payload.get("world_disable_source").is_none(),
        "enabled payload should omit world_disable_source: {payload}"
    );
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct ExpectedNetfilterStatus<'a> {
    requested: bool,
    enabled: bool,
    world_netfilter_enable_present: bool,
    last_failure_reason: Option<&'a str>,
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct WorldDoctorCase<'a> {
    name: &'a str,
    report_ok: bool,
    expected_world_status: &'a str,
    expected_netfilter_status: ExpectedNetfilterStatus<'a>,
}

#[cfg(target_os = "linux")]
fn build_world_doctor_report(case: &WorldDoctorCase<'_>) -> Value {
    json!({
        "schema_version": 2,
        "ok": case.report_ok,
        "collected_at_utc": "2026-01-08T00:00:00Z",
        "policy_snapshot_v1_supported": true,
        "policy_resolution_mode": "snapshot_v3",
        "netfilter_status": {
            "requested": case.expected_netfilter_status.requested,
            "enabled": case.expected_netfilter_status.enabled,
            "world_netfilter_enable_present": case.expected_netfilter_status.world_netfilter_enable_present,
            "last_failure_reason": case.expected_netfilter_status.last_failure_reason
        },
        "landlock": {
            "supported": true,
            "abi": 3,
            "reason": null
        },
        "world_fs_strategy": {
            "primary": "overlay",
            "fallback": "fuse",
            "probe": {
                "id": "enumeration_v1",
                "probe_file": ".substrate_enum_probe",
                "result": "pass",
                "failure_reason": null
            }
        }
    })
}

fn default_world_doctor_report() -> Value {
    json!({
        "schema_version": 2,
        "ok": true,
        "collected_at_utc": "2026-01-08T00:00:00Z",
        "policy_snapshot_v1_supported": true,
        "policy_resolution_mode": "snapshot_v3",
        "netfilter_status": {
            "requested": false,
            "enabled": false,
            "world_netfilter_enable_present": false,
            "last_failure_reason": null
        },
        "landlock": {
            "supported": true,
            "abi": 3,
            "reason": null
        },
        "world_fs_strategy": {
            "primary": "overlay",
            "fallback": "fuse",
            "probe": {
                "id": "enumeration_v1",
                "probe_file": ".substrate_enum_probe",
                "result": "pass",
                "failure_reason": null
            }
        }
    })
}

fn run_world_doctor_json(report: Value) -> Value {
    let fixture = ShellEnvFixture::new();
    let socket_dir = Builder::new()
        .prefix("substrate-ds0-sock-")
        .tempdir_in("/tmp")
        .expect("create ds0 socket tempdir");
    let socket_path = socket_dir.path().join("world-service.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndDoctorWorld { report },
    );

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
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unrecognized subcommand") || stderr.contains("unknown subcommand") {
            panic!("world doctor unavailable in DS0 conformance test: {stderr}");
        }
    }

    let payload = parse_json(&output.stdout, "world doctor --json");
    assert!(
        has_ds0_envelope(&payload),
        "world doctor should emit DS0 envelope: {payload}"
    );
    payload
}

#[cfg(target_os = "linux")]
fn assert_world_doctor_netfilter_status(payload: &Value, case: &WorldDoctorCase<'_>) {
    assert_world_doctor_envelope_v1(payload);

    let world = payload
        .get("world")
        .expect("world doctor missing world block");
    assert_eq!(
        world.get("status").and_then(Value::as_str),
        Some(case.expected_world_status),
        "{}: unexpected world.status",
        case.name
    );

    let netfilter = world
        .get("netfilter_status")
        .expect("world doctor missing world.netfilter_status block");
    assert_eq!(
        netfilter.get("requested").and_then(Value::as_bool),
        Some(case.expected_netfilter_status.requested),
        "{}: unexpected requested flag",
        case.name
    );
    assert_eq!(
        netfilter.get("enabled").and_then(Value::as_bool),
        Some(case.expected_netfilter_status.enabled),
        "{}: unexpected enabled flag",
        case.name
    );
    assert_eq!(
        netfilter
            .get("world_netfilter_enable_present")
            .and_then(Value::as_bool),
        Some(
            case.expected_netfilter_status
                .world_netfilter_enable_present
        ),
        "{}: unexpected world_netfilter_enable_present flag",
        case.name
    );
    assert_eq!(
        netfilter.get("last_failure_reason").and_then(Value::as_str),
        case.expected_netfilter_status.last_failure_reason,
        "{}: unexpected last_failure_reason",
        case.name
    );
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
    assert_disable_attribution_omitted(&payload);
}

#[test]
fn world_doctor_json_matches_envelope_v1_when_available() {
    let payload = run_world_doctor_json(default_world_doctor_report());
    assert_world_doctor_envelope_v1(&payload);
    assert_disable_attribution_omitted(&payload);
}

#[test]
fn host_doctor_json_emits_disable_attribution_root_fields_when_disabled() {
    let fixture = ShellEnvFixture::new();

    let mut cmd = support::substrate_command_for_home(&fixture);
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD");
    let output = cmd
        .arg("host")
        .arg("doctor")
        .arg("--json")
        .arg("--no-world")
        .output()
        .expect("substrate host doctor --json --no-world");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unrecognized subcommand") || stderr.contains("unknown subcommand") {
            return;
        }
    }

    let payload = parse_json(&output.stdout, "host doctor --json --no-world");
    if !has_ds0_envelope(&payload) {
        return;
    }
    assert_host_doctor_envelope_v1(&payload);
    assert_disable_attribution_fields(
        &payload,
        "world isolation disabled by CLI flag --no-world",
        "cli_flag",
    );
}

#[test]
fn world_doctor_json_emits_disable_attribution_root_fields_when_disabled() {
    let fixture = ShellEnvFixture::new();

    let mut cmd = support::substrate_command_for_home(&fixture);
    cmd.env("SUBSTRATE_OVERRIDE_WORLD", "disabled");
    let output = cmd
        .arg("world")
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("substrate world doctor --json");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unrecognized subcommand") || stderr.contains("unknown subcommand") {
            return;
        }
    }

    let payload = parse_json(&output.stdout, "world doctor --json disabled");
    if !has_ds0_envelope(&payload) {
        return;
    }
    assert_world_doctor_envelope_v1(&payload);
    assert_disable_attribution_fields(
        &payload,
        "world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled",
        "override_env",
    );
}

fn write_workspace_config(home: &std::path::Path, body: &str) {
    let path = home.join(".substrate").join("workspace.yaml");
    std::fs::create_dir_all(path.parent().expect("workspace parent"))
        .expect("create workspace dir");
    std::fs::write(&path, body).expect("write workspace config");
}

fn write_global_config(home: &std::path::Path, body: &str) {
    let path = home.join(".substrate").join("config.yaml");
    std::fs::create_dir_all(path.parent().expect("global parent")).expect("create global dir");
    std::fs::write(&path, body).expect("write global config");
}

#[test]
fn host_doctor_text_omits_disable_line_when_enabled() {
    let fixture = ShellEnvFixture::new();
    let mut cmd = support::substrate_command_for_home(&fixture);
    let output = cmd
        .arg("host")
        .arg("doctor")
        .output()
        .expect("substrate host doctor");
    let stdout = stdout_string(&output);

    assert!(
        attribution_line(&stdout).is_none(),
        "enabled host doctor should omit disable attribution: {stdout}"
    );
}

#[test]
fn host_doctor_text_prints_cli_flag_attribution_once() {
    let fixture = ShellEnvFixture::new();
    let mut cmd = support::substrate_command_for_home(&fixture);
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD");
    let output = cmd
        .arg("host")
        .arg("doctor")
        .arg("--no-world")
        .output()
        .expect("substrate host doctor --no-world");
    let stdout = stdout_string(&output);
    let line = attribution_line(&stdout).expect("expected disable attribution line");
    assert_eq!(
        line,
        "FAIL  | world isolation disabled by CLI flag --no-world"
    );
    assert_eq!(stdout.matches("world isolation disabled by").count(), 1);
}

#[test]
fn world_doctor_text_prints_env_override_attribution_once() {
    let fixture = ShellEnvFixture::new();
    let mut cmd = support::substrate_command_for_home(&fixture);
    cmd.env("SUBSTRATE_OVERRIDE_WORLD", "disabled");
    let output = cmd
        .arg("world")
        .arg("doctor")
        .output()
        .expect("substrate world doctor");
    let stdout = stdout_string(&output);
    let line = attribution_line(&stdout).expect("expected disable attribution line");
    assert_eq!(
        line,
        "FAIL  | world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled"
    );
    assert_eq!(stdout.matches("world isolation disabled by").count(), 1);
}

#[test]
fn world_doctor_text_prefers_workspace_over_global_and_masks_paths() {
    let fixture = ShellEnvFixture::new();
    write_global_config(
        fixture.home(),
        r#"
world:
  enabled: true
"#,
    );
    write_workspace_config(
        fixture.home(),
        r#"
world:
  enabled: false
"#,
    );

    let mut cmd = support::substrate_command_for_home(&fixture);
    cmd.env("SUBSTRATE_OVERRIDE_WORLD", "disabled");
    let output = cmd
        .arg("world")
        .arg("doctor")
        .output()
        .expect("substrate world doctor");
    let stdout = stdout_string(&output);
    let line = attribution_line(&stdout).expect("expected disable attribution line");
    assert_eq!(
        line,
        "FAIL  | world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)"
    );

    let workspace_path = fixture.home().join(".substrate").join("workspace.yaml");
    let global_path = fixture.home().join(".substrate").join("config.yaml");
    let line_owned = line.to_string();
    assert!(
        !line_owned.contains(workspace_path.to_string_lossy().as_ref()),
        "workspace attribution line should not expose raw workspace path: {line_owned}"
    );
    assert!(
        !line_owned.contains(global_path.to_string_lossy().as_ref()),
        "workspace attribution line should not expose raw global path: {line_owned}"
    );
    assert_eq!(stdout.matches("world isolation disabled by").count(), 1);
}

#[cfg(target_os = "linux")]
#[test]
fn world_doctor_json_preserves_netfilter_status_permutations() {
    let cases = [
        WorldDoctorCase {
            name: "not requested defaults stay false/null",
            report_ok: true,
            expected_world_status: "ok",
            expected_netfilter_status: ExpectedNetfilterStatus {
                requested: false,
                enabled: false,
                world_netfilter_enable_present: false,
                last_failure_reason: None,
            },
        },
        WorldDoctorCase {
            name: "requested plus guard present reports enabled",
            report_ok: true,
            expected_world_status: "ok",
            expected_netfilter_status: ExpectedNetfilterStatus {
                requested: true,
                enabled: true,
                world_netfilter_enable_present: true,
                last_failure_reason: None,
            },
        },
        WorldDoctorCase {
            name: "requested plus missing guard reports failure details",
            report_ok: false,
            expected_world_status: "missing_prereqs",
            expected_netfilter_status: ExpectedNetfilterStatus {
                requested: true,
                enabled: false,
                world_netfilter_enable_present: false,
                last_failure_reason: Some(GUARD_MISSING_REASON),
            },
        },
    ];

    for case in &cases {
        let payload = run_world_doctor_json(build_world_doctor_report(case));
        assert_world_doctor_netfilter_status(&payload, case);
    }
}
