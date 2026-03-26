#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod support;

use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use support::{binary_path, ensure_substrate_built, temp_dir, AgentSocket, SocketResponse};
use tempfile::TempDir;

fn manager_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/manager_hooks.yaml")
}

fn substrate_base_command(
    project: &Path,
    home: &Path,
    substrate_home: &Path,
    sock: &Path,
) -> Command {
    let mut cmd = Command::new(binary_path());
    cmd.current_dir(project)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("SUBSTRATE_HOME", substrate_home)
        .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged")
        .env("SHIM_TRACE_LOG", home.join(".substrate/trace.jsonl"))
        .env("SUBSTRATE_WORLD_SOCKET", sock)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env_remove("SHIM_ORIGINAL_PATH")
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .env_remove("SUBSTRATE_WORLD_ID")
        .env("SHELL", "/bin/bash")
        .arg("--shim-skip")
        .arg("--world");
    cmd
}

fn write_profile(project_dir: &Path) {
    let profile = r#"id: test-policy
name: Test Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: true
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
    fs::write(project_dir.join(".substrate-profile"), profile).expect("write .substrate-profile");
}

fn write_policy_with_net_allowed(home_substrate: &Path, net_allowed_yaml: &str) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let policy = format!(
        r#"id: test-global-policy
name: Test Global Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true
net_allowed: {net_allowed_yaml}
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
    fs::write(home_substrate.join("policy.yaml"), policy).expect("write policy.yaml");
}

fn write_config(home_substrate: &Path) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let config = r#"world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ""
  caged: false

policy:
  mode: observe
"#;
    fs::write(home_substrate.join("config.yaml"), config).expect("write config.yaml");
}

fn short_socket_dir(prefix: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("create short socket tempdir in /tmp")
}

#[test]
#[serial]
fn nonpty_world_request_carries_canonical_net_allowed_snapshot() {
    ensure_substrate_built();

    let temp = temp_dir("substrate-net-allowed-nonpty-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy_with_net_allowed(
        &substrate_home,
        "[\" Example.COM. \", \"example.com\", \"Api.Example.com.\"]",
    );
    write_config(&substrate_home);

    let sock_temp = short_socket_dir("sub-net-allowed-sock-");
    let sock = sock_temp.path().join("world.sock");
    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _sock = AgentSocket::start(
        &sock,
        SocketResponse::CapabilitiesAndExecuteRecord {
            stdout: "__NET_ALLOWED_NONPTY__\n".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
            records: records.clone(),
        },
    );

    let out = substrate_base_command(&project, &home, &substrate_home, &sock)
        .arg("-c")
        .arg("echo __NET_ALLOWED_NONPTY__")
        .output()
        .expect("run substrate -c");
    assert!(
        out.status.success(),
        "substrate -c failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let recorded = records.lock().expect("lock records");
    let request = recorded.last().expect("recorded execute request");
    let net_allowed = request
        .pointer("/policy_snapshot/net_allowed")
        .and_then(|value| value.as_array())
        .expect("policy_snapshot.net_allowed array");
    let net_allowed: Vec<&str> = net_allowed
        .iter()
        .map(|value| value.as_str().expect("net_allowed string"))
        .collect();
    assert_eq!(net_allowed, vec!["example.com", "api.example.com"]);
}
