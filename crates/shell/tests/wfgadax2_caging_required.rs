#![cfg(unix)]

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use support::{temp_dir, AgentSocket, SocketResponse};
use tempfile::Builder;

fn manager_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/manager_hooks.yaml")
}

fn write_policy(substrate_home: &Path) {
    let profile = r#"id: test-policy
name: Test Policy
world_fs:
  host_visible: true
  caged_required: true
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
metadata: {}
"#;
    fs::create_dir_all(substrate_home).expect("create SUBSTRATE_HOME");
    fs::write(substrate_home.join("policy.yaml"), profile).expect("write policy.yaml");
}

fn write_config(substrate_home: &Path, anchor_mode: &str, caged: bool) {
    let config = format!(
        "world:\n  enabled: true\n  anchor_mode: {anchor_mode}\n  anchor_path: \"\"\n  caged: {caged}\n\npolicy:\n  mode: observe\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n"
    );
    fs::create_dir_all(substrate_home).expect("create SUBSTRATE_HOME");
    fs::write(substrate_home.join("config.yaml"), config).expect("write config.yaml");
}

fn base_env_cmd(
    project_dir: &Path,
    home_dir: &Path,
    socket_path: &Path,
    trace_path: &Path,
) -> assert_cmd::Command {
    let mut cmd = support::get_substrate_binary();
    let substrate_home = home_dir.join(".substrate");
    cmd.current_dir(project_dir)
        .env("HOME", home_dir)
        .env("USERPROFILE", home_dir)
        .env("SUBSTRATE_HOME", &substrate_home)
        .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
        .env("SHIM_TRACE_LOG", trace_path)
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged")
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", socket_path);
    cmd
}

#[test]
fn wfgadax2_control_caged_required_allows_caged_workspace_execution() {
    let temp = temp_dir("substrate-wfgadax2-control-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims directory");
    fs::create_dir_all(&project).expect("create project");
    write_policy(&substrate_home);
    write_config(&substrate_home, "workspace", true);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_dir = Builder::new()
        .prefix("substrate-wfgadax2-sock-")
        .tempdir_in("/tmp")
        .expect("create socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    let marker = "__wfgadax2_control__";
    let assert = base_env_cmd(&project, &home, &socket_path, &trace_path)
        .arg("-c")
        .arg(format!("printf {marker}"))
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(marker),
        "expected control command to execute, got stdout: {stdout}"
    );
    assert!(
        socket.execute_request_count() > 0,
        "expected control run to reach world-agent execution"
    );
}

#[test]
fn wfgadax2_caged_required_rejects_world_caged_false_exit_2() {
    let temp = temp_dir("substrate-wfgadax2-caged-false-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims directory");
    fs::create_dir_all(&project).expect("create project");
    write_policy(&substrate_home);
    write_config(&substrate_home, "workspace", false);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_dir = Builder::new()
        .prefix("substrate-wfgadax2-sock-")
        .tempdir_in("/tmp")
        .expect("create socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    let marker = "__wfgadax2_caged_false__";
    let assert = base_env_cmd(&project, &home, &socket_path, &trace_path)
        .arg("-c")
        .arg(format!("printf {marker}"))
        .assert()
        .code(2);

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains(marker),
        "expected hard error to occur before execution, got stdout: {stdout}"
    );
    assert_eq!(
        socket.execute_request_count(),
        0,
        "expected incompatibility to fail before any world-agent execution request"
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let stderr_lc = stderr.to_ascii_lowercase();
    assert!(
        stderr_lc.contains("caged")
            && (stderr_lc.contains("required") || stderr_lc.contains("require")),
        "expected stderr to mention caging requirement, got: {stderr}"
    );
}

#[test]
fn wfgadax2_caged_required_rejects_anchor_mode_follow_cwd_exit_2() {
    let temp = temp_dir("substrate-wfgadax2-follow-cwd-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims directory");
    fs::create_dir_all(&project).expect("create project");
    write_policy(&substrate_home);
    write_config(&substrate_home, "follow-cwd", true);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_dir = Builder::new()
        .prefix("substrate-wfgadax2-sock-")
        .tempdir_in("/tmp")
        .expect("create socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    let marker = "__wfgadax2_follow_cwd__";
    let assert = base_env_cmd(&project, &home, &socket_path, &trace_path)
        .arg("-c")
        .arg(format!("printf {marker}"))
        .assert()
        .code(2);

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains(marker),
        "expected hard error to occur before execution, got stdout: {stdout}"
    );
    assert_eq!(
        socket.execute_request_count(),
        0,
        "expected incompatibility to fail before any world-agent execution request"
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let stderr_lc = stderr.to_ascii_lowercase();
    assert!(
        stderr_lc.contains("anchor") && stderr_lc.contains("follow"),
        "expected stderr to mention anchor_mode=follow-cwd incompatibility, got: {stderr}"
    );
}
