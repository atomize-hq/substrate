#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod support;

use serial_test::serial;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use support::{binary_path, ensure_substrate_built, temp_dir, AgentSocket, SocketResponse};
use tempfile::TempDir;

fn manager_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/manager_hooks.yaml")
}

fn write_profile(project_dir: &Path) {
    let profile = r#"id: test-policy
name: Test Policy
world_fs:
  mode: writable
  isolation: workspace
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist: []
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

fn write_policy(home_substrate: &Path) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let policy = r#"id: test-global-policy
name: Test Global Policy
world_fs:
  mode: writable
  isolation: workspace
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist: []
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
    fs::write(home_substrate.join("policy.yaml"), policy).expect("write policy.yaml");
}

fn write_config(home_substrate: &Path) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    // Keep this minimal but non-empty to avoid platforms auto-initializing global defaults during tests.
    // `SUBSTRATE_WORLD_SOCKET` provides the world backend override for these tests.
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
fn command_mode_world_consistency_v1_routes_both_c_and_pipe_via_world_socket() {
    ensure_substrate_built();

    let temp = temp_dir("substrate-c5-command-mode-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);
    write_config(&substrate_home);

    let sock_temp = short_socket_dir("sub-c5-sock-");
    let sock = sock_temp.path().join("world.sock");
    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _sock = AgentSocket::start(
        &sock,
        SocketResponse::CapabilitiesAndHostExecuteRecord {
            scopes: vec![],
            records: records.clone(),
        },
    );

    // `-c` mode should execute via the world socket.
    let out = Command::new(binary_path())
        .current_dir(&project)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged")
        .env("SHIM_TRACE_LOG", home.join(".substrate/trace.jsonl"))
        .env("SUBSTRATE_WORLD_SOCKET", &sock)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env_remove("SHIM_ORIGINAL_PATH")
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .env_remove("SUBSTRATE_WORLD_ID")
        .env("SHELL", "/bin/bash")
        .arg("--shim-skip")
        .arg("--world")
        .arg("-c")
        .arg("echo __C5_C_MODE__")
        .output()
        .expect("run substrate -c");
    assert!(
        out.status.success(),
        "substrate -c failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("__C5_C_MODE__"),
        "expected -c output marker; stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );

    // Pipe mode should also route via the world socket, not accidentally fall back to host-only.
    let mut child = Command::new(binary_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&project)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged")
        .env("SHIM_TRACE_LOG", home.join(".substrate/trace.jsonl"))
        .env("SUBSTRATE_WORLD_SOCKET", &sock)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env_remove("SHIM_ORIGINAL_PATH")
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .env_remove("SUBSTRATE_WORLD_ID")
        .env("SHELL", "/bin/bash")
        .arg("--shim-skip")
        .arg("--world")
        .spawn()
        .expect("spawn substrate pipe mode");

    {
        let mut stdin = child.stdin.take().expect("child stdin");
        writeln!(stdin, "echo __C5_PIPE_MODE__").expect("write pipe command");
        stdin.flush().expect("flush pipe command");
    }

    let output = child.wait_with_output().expect("pipe output");
    assert!(
        output.status.success(),
        "substrate pipe mode failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("__C5_PIPE_MODE__"),
        "expected pipe output marker; stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let recorded = records.lock().expect("lock records");
    assert!(
        recorded.len() >= 2,
        "expected both -c and pipe mode to issue world-agent execute requests; recorded={}",
        recorded.len()
    );
}
