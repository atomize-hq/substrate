#![cfg(all(unix, target_os = "linux"))]

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use support::{temp_dir, AgentSocket, SocketResponse};
use tempfile::Builder;

fn manager_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/manager_hooks.yaml")
}

fn write_profile(project_dir: &Path, fail_closed_routing: bool) {
    let routing = if fail_closed_routing { "true" } else { "false" };
    let profile = format!(
        r#"id: test-policy
name: Test Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: {routing}
  write:
    enabled: true
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {{}}
"#
    );
    fs::write(project_dir.join(".substrate-profile"), profile).expect("write .substrate-profile");
}

fn base_cmd(project_dir: &Path, home_dir: &Path, trace_path: &Path) -> assert_cmd::Command {
    let mut cmd = support::get_substrate_binary();
    cmd.current_dir(project_dir)
        .env("HOME", home_dir)
        .env("USERPROFILE", home_dir)
        .env("SUBSTRATE_HOME", home_dir.join(".substrate"))
        .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
        .env("SHIM_TRACE_LOG", trace_path)
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged");
    cmd
}

#[test]
fn wfgadax1_fail_closed_routing_true_world_disabled_hard_errors_exit_2() {
    let temp = temp_dir("substrate-wfgadax1-world-disabled-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".substrate")).expect("create SUBSTRATE_HOME");
    fs::create_dir_all(&project).expect("create project");
    write_profile(&project, true);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let marker = "__wfgadax1_world_disabled__";
    let assert = base_cmd(&project, &home, &trace_path)
        .env("SUBSTRATE_OVERRIDE_WORLD", "disabled")
        .arg("-c")
        .arg(format!("printf {marker}"))
        .assert()
        .code(2);

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains(marker),
        "expected hard error to occur before execution, got stdout: {stdout}"
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.to_ascii_lowercase().contains("world")
            && stderr.to_ascii_lowercase().contains("disabled"),
        "expected stderr to mention world disabled, got: {stderr}"
    );
}

#[test]
fn wfgadax1_runtime_routing_fail_closed_missing_socket_maps_to_exit_3() {
    let temp = temp_dir("substrate-wfgadax1-world-missing-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".substrate")).expect("create SUBSTRATE_HOME");
    fs::create_dir_all(&project).expect("create project");
    write_profile(&project, true);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_path = temp.path().join("sockdir/missing.sock");
    fs::create_dir_all(socket_path.parent().expect("socket parent")).expect("create socket dir");

    let marker = "__wfgadax1_missing_socket__";
    let assert = base_cmd(&project, &home, &trace_path)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .arg("-c")
        .arg(format!("printf {marker}"))
        .assert()
        .code(3);

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains(marker),
        "expected fail-closed routing to prevent execution on host, got stdout: {stdout}"
    );
}

#[test]
fn wfgadax1_runtime_routing_fail_closed_strategy_unavailable_maps_to_exit_4() {
    let temp = temp_dir("substrate-wfgadax1-strategy-unavailable-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".substrate")).expect("create SUBSTRATE_HOME");
    fs::create_dir_all(&project).expect("create project");
    write_profile(&project, true);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_dir = Builder::new()
        .prefix("substrate-wfgadax1-sock-")
        .tempdir_in("/tmp")
        .expect("create socket tempdir");
    let socket_path: PathBuf = socket_dir.path().join("world-agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecuteStreamError {
            message: "WORLD_FS_STRATEGY_UNAVAILABLE fallback_reason=fallback_mount_failed"
                .to_string(),
        },
    );

    let marker = "__wfgadax1_strategy_unavailable__";
    let assert = base_cmd(&project, &home, &trace_path)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .arg("--world")
        .arg("-c")
        .arg(format!("printf {marker}"))
        .assert()
        .code(4);

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains(marker),
        "expected strategy-unavailable to fail closed before execution, got stdout: {stdout}"
    );
}

#[test]
fn wfgadax1_exports_fail_closed_routing_state_env_var_and_deletes_require_world() {
    let temp = temp_dir("substrate-wfgadax1-exported-env-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".substrate")).expect("create SUBSTRATE_HOME");
    fs::create_dir_all(&project).expect("create project");

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_dir = Builder::new()
        .prefix("substrate-wfgadax1-sock-")
        .tempdir_in("/tmp")
        .expect("create socket tempdir");
    let socket_path: PathBuf = socket_dir.path().join("world-agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    // Policy-derived state should override any input env value.
    write_profile(&project, true);

    let cmd = r#"if [ -z "${SUBSTRATE_WORLD_REQUIRE_WORLD+x}" ]; then req=unset; else req="set:${SUBSTRATE_WORLD_REQUIRE_WORLD}"; fi; printf "fc=%s req=%s\n" "${SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING:-MISSING}" "$req""#;

    let assert = base_cmd(&project, &home, &trace_path)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING", "0")
        .env("SUBSTRATE_WORLD_REQUIRE_WORLD", "1")
        .arg("--world")
        .arg("-c")
        .arg(cmd)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("fc=1 req=unset"),
        "expected env contract state (fc=1, req=unset), got stdout: {stdout}"
    );
}

#[test]
fn wfgadax1_exports_fail_closed_routing_state_false_is_0_and_output_only() {
    let temp = temp_dir("substrate-wfgadax1-exported-env-false-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".substrate")).expect("create SUBSTRATE_HOME");
    fs::create_dir_all(&project).expect("create project");

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_dir = Builder::new()
        .prefix("substrate-wfgadax1-sock-")
        .tempdir_in("/tmp")
        .expect("create socket tempdir");
    let socket_path: PathBuf = socket_dir.path().join("world-agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    write_profile(&project, false);

    let cmd = r#"if [ -z "${SUBSTRATE_WORLD_REQUIRE_WORLD+x}" ]; then req=unset; else req="set:${SUBSTRATE_WORLD_REQUIRE_WORLD}"; fi; printf "fc=%s req=%s\n" "${SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING:-MISSING}" "$req""#;

    let assert = base_cmd(&project, &home, &trace_path)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING", "1")
        .env("SUBSTRATE_WORLD_REQUIRE_WORLD", "1")
        .arg("--world")
        .arg("-c")
        .arg(cmd)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("fc=0 req=unset"),
        "expected env contract state (fc=0, req=unset), got stdout: {stdout}"
    );
}
