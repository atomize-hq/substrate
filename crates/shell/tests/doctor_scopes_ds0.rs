#![cfg(unix)]

mod support;

use serde_json::json;
use serde_json::Value;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::Command;
use support::{substrate_shell_driver, AgentSocket, ShellEnvFixture, SocketResponse};
use tempfile::Builder;

#[cfg(target_os = "linux")]
const GUARD_MISSING_REASON: &str =
    "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules";

#[cfg(target_os = "linux")]
fn expected_host_context(prefix: &Path) -> (String, String) {
    let uid = unsafe { libc::geteuid() };
    let account_output = Command::new("/usr/bin/id")
        .args(["-nu", &uid.to_string()])
        .output()
        .expect("resolve current Unix account");
    assert!(account_output.status.success());
    let account = String::from_utf8(account_output.stdout)
        .expect("Unix account is UTF-8")
        .trim()
        .to_string();
    let context = transport_api_types::InstallBootstrapContextV1::new_unix(
        prefix.to_str().expect("selected host prefix is UTF-8"),
        &account,
        uid,
    )
    .expect("construct expected install context");
    let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(context)
        .expect("commit expected install context");
    (
        carrier.context.selected_host_prefix,
        carrier.host_context_commitment,
    )
}

#[cfg(target_os = "linux")]
fn passive_host_context(
    prefix: &Path,
) -> (
    transport_api_types::InstallBootstrapContextCarrierV1,
    String,
) {
    let uid = unsafe { libc::geteuid() };
    let account_output = Command::new("/usr/bin/id")
        .args(["-nu", &uid.to_string()])
        .output()
        .expect("resolve current Unix account");
    assert!(account_output.status.success());
    let account = String::from_utf8(account_output.stdout)
        .expect("Unix account is UTF-8")
        .trim()
        .to_string();
    let context = transport_api_types::InstallBootstrapContextV1::new_unix(
        prefix.to_str().expect("selected host prefix is UTF-8"),
        &account,
        uid,
    )
    .expect("construct passive install context");
    let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(context)
        .expect("commit passive install context");
    let encoded = carrier.encode().expect("encode passive install context");
    (carrier, encoded)
}

#[cfg(target_os = "linux")]
fn project_passive_host_context(
    command: &mut Command,
    carrier: &transport_api_types::InstallBootstrapContextCarrierV1,
    encoded: &str,
) {
    let transport_api_types::PlatformPrincipalV1::Unix { account, uid } =
        &carrier.context.intended_host_principal
    else {
        panic!("expected Unix passive principal");
    };
    command
        .env("SUBSTRATE_HOME", &carrier.context.host_substrate_home)
        .env("SUBSTRATE_ROOT", &carrier.context.host_substrate_root)
        .env(
            "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT",
            &carrier.host_context_commitment,
        )
        .env("SUBSTRATE_INSTALL_PRIMARY_USER", account)
        .env("SUBSTRATE_INSTALL_PRIMARY_UID", uid.to_string())
        .env("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1", encoded);
}

#[cfg(target_os = "linux")]
fn snapshot_tree(root: &Path) -> Vec<(PathBuf, u32, u32, u32, Vec<u8>)> {
    fn walk(root: &Path, path: &Path, entries: &mut Vec<(PathBuf, u32, u32, u32, Vec<u8>)>) {
        let metadata = std::fs::symlink_metadata(path).expect("snapshot tree metadata");
        let relative = path
            .strip_prefix(root)
            .expect("snapshot path beneath root")
            .to_path_buf();
        let payload = if metadata.file_type().is_symlink() {
            std::fs::read_link(path)
                .expect("snapshot symlink target")
                .as_os_str()
                .as_bytes()
                .to_vec()
        } else if metadata.is_file() {
            std::fs::read(path).expect("snapshot file contents")
        } else {
            Vec::new()
        };
        entries.push((
            relative,
            metadata.mode(),
            metadata.uid(),
            metadata.gid(),
            payload,
        ));
        if metadata.is_dir() {
            let mut children = std::fs::read_dir(path)
                .expect("snapshot directory")
                .map(|entry| entry.expect("snapshot directory entry").path())
                .collect::<Vec<_>>();
            children.sort();
            for child in children {
                walk(root, &child, entries);
            }
        }
    }

    let mut entries = Vec::new();
    walk(root, root, &mut entries);
    entries
}

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

#[cfg(target_os = "linux")]
fn compile_install_projection_preload(root: &std::path::Path) -> std::path::PathBuf {
    let source = root.join("install_projection_preload.c");
    let library = root.join("install_projection_preload.so");
    std::fs::write(
        &source,
        r#"#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdlib.h>
#include <string.h>
typedef char *(*real_getenv_fn)(const char *);
char *getenv(const char *name) {
    static real_getenv_fn real_fn;
    static unsigned home_reads, root_reads, context_reads;
    if (real_fn == NULL) real_fn = (real_getenv_fn)dlsym(RTLD_NEXT, "getenv");
    const char *hide_after_first = real_fn("SUBSTRATE_R2_TEST_HIDE_PROJECTIONS_AFTER_FIRST_READ");
    if (hide_after_first != NULL && strcmp(hide_after_first, "1") == 0) {
        unsigned *reads = NULL;
        if (strcmp(name, "SUBSTRATE_HOME") == 0) reads = &home_reads;
        else if (strcmp(name, "SUBSTRATE_ROOT") == 0) reads = &root_reads;
        else if (strcmp(name, "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1") == 0) reads = &context_reads;
        if (reads != NULL && (*reads)++ > 0) return NULL;
    }
    return real_fn(name);
}
"#,
    )
    .expect("write install-projection preload source");
    let output = Command::new("/usr/bin/cc")
        .args(["-shared", "-fPIC", "-O2"])
        .arg(&source)
        .arg("-o")
        .arg(&library)
        .arg("-ldl")
        .output()
        .expect("compile install-projection preload fixture");
    assert!(
        output.status.success(),
        "install-projection preload compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    library
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
    #[cfg(target_os = "linux")]
    support::ensure_substrate_built();
    #[cfg(target_os = "linux")]
    let secure_parent = std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".cache"))
        })
        .expect("world doctor test requires XDG_RUNTIME_DIR or account HOME");
    #[cfg(target_os = "linux")]
    std::fs::create_dir_all(&secure_parent).expect("create secure world doctor test parent");
    #[cfg(target_os = "linux")]
    let fixture = Builder::new()
        .prefix("substrate-world-doctor-r2-")
        .tempdir_in(&secure_parent)
        .expect("allocate secure world doctor fixture");
    #[cfg(target_os = "linux")]
    let selected_prefix = fixture.path().join("selected-a");
    #[cfg(target_os = "linux")]
    std::fs::create_dir_all(&selected_prefix).expect("create selected world doctor prefix");
    #[cfg(target_os = "linux")]
    std::fs::set_permissions(&selected_prefix, std::fs::Permissions::from_mode(0o700))
        .expect("secure selected world doctor prefix");
    #[cfg(target_os = "linux")]
    let ambient_home = fixture.path().join("ambient-b");
    #[cfg(target_os = "linux")]
    let ambient_prefix = ambient_home.join(".substrate");
    #[cfg(target_os = "linux")]
    std::fs::create_dir_all(&ambient_prefix).expect("create conflicting world doctor prefix");
    #[cfg(not(target_os = "linux"))]
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

    #[cfg(target_os = "linux")]
    let mut cmd = Command::new(support::binary_path());
    #[cfg(not(target_os = "linux"))]
    let mut cmd = support::substrate_command_for_home(&fixture);
    #[cfg(target_os = "linux")]
    cmd.env("HOME", &ambient_home)
        .env("USERPROFILE", &ambient_home)
        .env("SUBSTRATE_HOME", &ambient_prefix)
        .env("SUBSTRATE_ROOT", &ambient_prefix)
        .env("PATH", "/usr/bin:/bin");
    cmd.env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path);
    #[cfg(target_os = "linux")]
    cmd.arg("--install-prefix").arg(&selected_prefix);
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

    assert!(
        !output.stdout.is_empty(),
        "world doctor --json emitted no JSON: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload = parse_json(&output.stdout, "world doctor --json");
    assert!(
        has_ds0_envelope(&payload),
        "world doctor should emit DS0 envelope: {payload}"
    );
    #[cfg(target_os = "linux")]
    {
        let (expected_prefix, expected_commitment) = expected_host_context(&selected_prefix);
        assert_eq!(
            payload
                .pointer("/host/selected_host_prefix")
                .and_then(Value::as_str),
            Some(expected_prefix.as_str())
        );
        assert_eq!(
            payload
                .pointer("/host/host_context_commitment")
                .and_then(Value::as_str),
            Some(expected_commitment.as_str())
        );
        assert_eq!(
            payload
                .pointer("/world/selected_host_prefix")
                .and_then(Value::as_str),
            Some(expected_prefix.as_str())
        );
        assert_eq!(
            payload
                .pointer("/world/host_context_commitment")
                .and_then(Value::as_str),
            Some(expected_commitment.as_str())
        );
    }
    payload
}

#[cfg(target_os = "linux")]
#[test]
fn authenticated_passive_world_doctor_is_exclusive_bounded_and_side_effect_free() {
    support::ensure_substrate_built();
    let secure_parent = std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".cache"))
        })
        .expect("passive world doctor test requires XDG_RUNTIME_DIR or account HOME");
    std::fs::create_dir_all(&secure_parent).expect("create secure passive test parent");
    let fixture = Builder::new()
        .prefix("substrate-passive-world-doctor-")
        .tempdir_in(&secure_parent)
        .expect("allocate passive world doctor fixture");
    let selected_a = fixture.path().join("selected-a");
    let ambient_b = fixture.path().join("ambient-b-marker");
    std::fs::create_dir_all(selected_a.join("health")).expect("create selected passive prefix");
    std::fs::create_dir_all(&ambient_b).expect("create ambient B prefix");
    std::fs::set_permissions(&selected_a, std::fs::Permissions::from_mode(0o700))
        .expect("secure selected passive prefix");
    std::fs::set_permissions(&ambient_b, std::fs::Permissions::from_mode(0o700))
        .expect("secure ambient B prefix");
    std::fs::write(
        selected_a.join("config.yaml"),
        "invalid: [config-policy-selection-must-not-run",
    )
    .expect("write poison config");
    std::fs::write(
        selected_a.join("health/world_doctor.json"),
        r#"{"ok":true,"fixture_secret":"fixture-secret-marker"}"#,
    )
    .expect("write forbidden product fixture");
    let poison_marker = fixture.path().join("service-process-started-marker");
    let poison_service = fixture.path().join("poison-world-service");
    std::fs::write(
        &poison_service,
        format!(
            "#!/bin/sh\nprintf invoked > '{}'\nexit 99\n",
            poison_marker.display()
        ),
    )
    .expect("write poison service");
    std::fs::set_permissions(&poison_service, std::fs::Permissions::from_mode(0o700))
        .expect("make poison service executable");
    let trace_marker = selected_a.join("passive-trace-must-not-exist.jsonl");
    let scaffold_marker = selected_a.join("deps/.bootstrap.lock");

    let socket_dir = Builder::new()
        .prefix("substrate-passive-sock-")
        .tempdir_in("/tmp")
        .expect("create passive socket tempdir");
    let socket_path = socket_dir.path().join("world-service.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndDoctorWorld {
            report: default_world_doctor_report(),
        },
    );
    let (carrier, encoded) = passive_host_context(&selected_a);
    let expected = json!({
        "schema_version": 1,
        "platform": std::env::consts::OS,
        "ok": false,
        "host": {
            "platform": std::env::consts::OS,
            "ok": false,
            "selected_host_prefix": carrier.context.selected_host_prefix.clone(),
            "host_context_commitment": carrier.host_context_commitment.clone()
        },
        "world": {
            "status": "unavailable",
            "ok": false,
            "selected_host_prefix": carrier.context.selected_host_prefix.clone(),
            "host_context_commitment": carrier.host_context_commitment.clone()
        }
    });
    let before = snapshot_tree(fixture.path());

    let mut command = Command::new(support::binary_path());
    project_passive_host_context(&mut command, &carrier, &encoded);
    let output = command
        .current_dir(&ambient_b)
        .env("HOME", &ambient_b)
        .env("USERPROFILE", &ambient_b)
        .env("XDG_CONFIG_HOME", &ambient_b)
        .env("SUBSTRATE_CONFIG", ambient_b.join("config-marker.yaml"))
        .env("SUBSTRATE_POLICY", ambient_b.join("policy-marker.yaml"))
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_AGENT_BIN", &poison_service)
        .env("SHIM_TRACE_LOG", &trace_marker)
        .env("PROVIDER_TOKEN", "provider-token-marker")
        .env("OPENAI_API_KEY", "api-key-marker")
        .env("PRIVATE_KEY", "private-key-marker")
        .env("PROMPT_MATERIAL", "prompt-request-marker")
        .env("REQUEST_BODY", "request-body-marker")
        .env("BOOTSTRAP_SHIM_CARRIER", "shim-carrier-marker")
        .env("COMMITMENT_PREIMAGE", "preimage-marker")
        .arg("--install-bootstrap-context-v1")
        .arg(&encoded)
        .args([
            "world",
            "doctor",
            "--json",
            "--internal-passive-world-doctor-v1",
        ])
        .output()
        .expect("run authenticated passive world doctor");

    assert_eq!(output.status.code(), Some(4));
    assert!(output.stderr.is_empty(), "passive stderr must be empty");
    assert_eq!(
        parse_json(&output.stdout, "authenticated passive world doctor"),
        expected
    );
    let rendered = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for marker in [
        "ambient-b-marker",
        "fixture-secret-marker",
        "provider-token-marker",
        "api-key-marker",
        "private-key-marker",
        "prompt-request-marker",
        "request-body-marker",
        "shim-carrier-marker",
        "preimage-marker",
        encoded.as_str(),
    ] {
        assert!(!rendered.contains(marker), "passive output leaked {marker}");
    }
    assert_eq!(socket.connection_count(), 0, "socket must not be connected");
    assert_eq!(
        socket.execute_request_count(),
        0,
        "execute endpoint must not be called"
    );
    assert!(!poison_marker.exists(), "service process must not start");
    assert!(
        !trace_marker.exists(),
        "passive mode must not initialize tracing"
    );
    assert!(
        !scaffold_marker.exists(),
        "passive mode must not create scaffold state"
    );
    assert_eq!(
        snapshot_tree(fixture.path()),
        before,
        "passive mode mutated fixture state"
    );

    let mut tampered = encoded.clone().into_bytes();
    let last = tampered.last_mut().expect("nonempty carrier");
    *last = if *last == b'A' { b'B' } else { b'A' };
    let tampered = String::from_utf8(tampered).expect("ASCII carrier");
    for (label, argv_carrier, include_carrier_arg, include_json, force_world, conflict_home) in [
        ("missing", "", false, true, false, false),
        (
            "environment-only",
            encoded.as_str(),
            false,
            true,
            false,
            false,
        ),
        ("malformed", "malformed-carrier", true, true, false, false),
        ("tampered", tampered.as_str(), true, true, false, false),
        ("missing-json", encoded.as_str(), true, false, false, false),
        ("nonexclusive", encoded.as_str(), true, true, true, false),
        ("conflicting-b", encoded.as_str(), true, true, false, true),
    ] {
        let mut rejected = Command::new(support::binary_path());
        project_passive_host_context(&mut rejected, &carrier, &encoded);
        rejected
            .current_dir(&ambient_b)
            .env("HOME", &ambient_b)
            .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
            .env("SUBSTRATE_WORLD_AGENT_BIN", &poison_service)
            .env("PROMPT_MATERIAL", "prompt-request-marker");
        if conflict_home {
            rejected.env("SUBSTRATE_HOME", &ambient_b);
        }
        if include_carrier_arg {
            rejected
                .arg("--install-bootstrap-context-v1")
                .arg(argv_carrier);
        }
        if force_world {
            rejected.arg("--world");
        }
        rejected.args(["world", "doctor"]);
        if include_json {
            rejected.arg("--json");
        }
        let rejected = rejected
            .arg("--internal-passive-world-doctor-v1")
            .output()
            .unwrap_or_else(|err| panic!("run rejected passive case {label}: {err}"));
        assert_eq!(rejected.status.code(), Some(2), "case {label}");
        assert!(rejected.stdout.is_empty(), "case {label} observed output");
        let stderr = String::from_utf8_lossy(&rejected.stderr);
        for marker in [
            "ambient-b-marker",
            "prompt-request-marker",
            encoded.as_str(),
            tampered.as_str(),
        ] {
            assert!(!stderr.contains(marker), "case {label} leaked {marker}");
        }
    }
    assert_eq!(
        socket.connection_count(),
        0,
        "rejections must not connect socket"
    );
    assert_eq!(
        socket.execute_request_count(),
        0,
        "rejections must not execute"
    );
    assert!(!poison_marker.exists(), "rejections must not start service");
    assert!(
        !trace_marker.exists(),
        "rejections must not initialize tracing"
    );
    assert_eq!(
        snapshot_tree(fixture.path()),
        before,
        "rejections mutated state"
    );
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

#[cfg(target_os = "linux")]
#[test]
fn host_doctor_uses_installed_witness_under_conflicting_ambient() {
    support::ensure_substrate_built();
    let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(format!("/run/user/{}", unsafe { libc::geteuid() }))
        });
    let fixture = Builder::new()
        .prefix("substrate-host-witness-r2-")
        .tempdir_in(safe_parent)
        .expect("allocate secure Host witness fixture");
    let projection_preload = compile_install_projection_preload(fixture.path());
    let selected_prefix = fixture.path().join("selected-a");
    let selected_bin = selected_prefix.join("bin");
    std::fs::create_dir_all(&selected_bin).expect("create installed witness bin");
    std::fs::set_permissions(&selected_prefix, std::fs::Permissions::from_mode(0o700))
        .expect("secure installed witness prefix");
    std::fs::set_permissions(&selected_bin, std::fs::Permissions::from_mode(0o755))
        .expect("secure installed witness bin");
    std::fs::write(
        selected_prefix.join("config.yaml"),
        "world:\n  enabled: false\n",
    )
    .expect("write selected Host config");
    let installed_witness = selected_bin.join("substrate");
    std::os::unix::fs::symlink(support::binary_path(), &installed_witness)
        .expect("create canonical installed-product witness");

    let ambient_home = fixture.path().join("ambient-b");
    let ambient_prefix = ambient_home.join(".substrate");
    std::fs::create_dir_all(&ambient_prefix).expect("create conflicting ambient prefix");
    let ambient_config = ambient_prefix.join("config.yaml");
    std::fs::write(&ambient_config, "world:\n  enabled: true\n")
        .expect("write conflicting ambient Host config");
    let ambient_before = snapshot_tree(&ambient_home);

    let mut installed = Command::new(&installed_witness);
    installed
        .env("HOME", &ambient_home)
        .env("USERPROFILE", &ambient_home)
        .env("SUBSTRATE_HOME", &ambient_prefix)
        .env("SUBSTRATE_ROOT", &ambient_prefix)
        .env("PATH", "/usr/bin:/bin")
        .env("LD_PRELOAD", &projection_preload)
        .env("SUBSTRATE_R2_TEST_HIDE_PROJECTIONS_AFTER_FIRST_READ", "1")
        .env_remove("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1")
        .env_remove("SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT")
        .env_remove("SUBSTRATE_INSTALL_PRIMARY_USER")
        .env_remove("SUBSTRATE_INSTALL_PRIMARY_UID")
        .env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .current_dir(&ambient_home)
        .arg("host")
        .arg("doctor")
        .arg("--json");
    let output = installed
        .output()
        .expect("run Host doctor through installed-product witness");
    assert_ne!(
        output.status.code(),
        Some(2),
        "installed Host witness must reach Host diagnostics rather than fail selector validation: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload = parse_json(&output.stdout, "installed-witness host doctor --json");
    assert_host_doctor_envelope_v1(&payload);
    let (expected_prefix, expected_commitment) = expected_host_context(&selected_prefix);
    assert_eq!(
        payload
            .pointer("/host/selected_host_prefix")
            .and_then(Value::as_str),
        Some(expected_prefix.as_str())
    );
    assert_eq!(
        payload
            .pointer("/host/host_context_commitment")
            .and_then(Value::as_str),
        Some(expected_commitment.as_str())
    );
    assert_eq!(
        payload.get("world_enabled").and_then(Value::as_bool),
        Some(false),
        "Host must resolve selected A's disabled config instead of ambient B: {payload}"
    );
    assert_eq!(
        payload
            .pointer("/world_disable_source/layer")
            .and_then(Value::as_str),
        Some("global_patch"),
        "Host must attribute the selected A config: {payload}"
    );
    assert_eq!(
        snapshot_tree(&ambient_home),
        ambient_before,
        "Host dispatch must preserve the complete conflicting ambient B tree"
    );
    for relative in [
        "deps/README.md",
        "deps/packages/example-manual.yaml",
        "deps/bundles/example-bundle.yaml",
        "deps/scripts/example-install.sh",
    ] {
        assert!(
            selected_prefix.join(relative).is_file(),
            "installed Host witness must scaffold A-derived dependency artifact {relative}"
        );
    }
    assert!(
        !ambient_prefix.join("deps").exists(),
        "installed Host witness must not scaffold dependency artifacts under B"
    );

    let mut direct = Command::new(support::binary_path());
    direct
        .env("HOME", &ambient_home)
        .env("USERPROFILE", &ambient_home)
        .env("SUBSTRATE_HOME", &ambient_prefix)
        .env("SUBSTRATE_ROOT", &ambient_prefix)
        .env("PATH", "/usr/bin:/bin")
        .env_remove("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1")
        .env_remove("SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT")
        .env_remove("SUBSTRATE_INSTALL_PRIMARY_USER")
        .env_remove("SUBSTRATE_INSTALL_PRIMARY_UID")
        .current_dir(&ambient_home)
        .arg("host")
        .arg("doctor")
        .arg("--json");
    let direct_output = direct
        .output()
        .expect("run direct repository Host invocation without witness");
    assert_eq!(
        direct_output.status.code(),
        Some(2),
        "direct repository invocation without selector or witness must fail closed: stdout={} stderr={}",
        String::from_utf8_lossy(&direct_output.stdout),
        String::from_utf8_lossy(&direct_output.stderr)
    );
    assert!(direct_output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&direct_output.stderr)
        .contains("installed-product invocation witness or --install-prefix is required"));
    assert_eq!(
        snapshot_tree(&ambient_home),
        ambient_before,
        "rejected direct invocation must preserve the complete B tree"
    );
    assert!(
        !ambient_prefix.join("shims").exists(),
        "rejected direct invocation must not scaffold ambient B"
    );
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
