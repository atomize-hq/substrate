#![cfg(target_os = "linux")]

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

fn write_policy(home_substrate: &Path) {
    write_policy_with_net_allowed(home_substrate, "[]");
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
metadata: {}
"#
    );
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

#[test]
#[serial]
fn command_mode_world_consistency_v1_nonpty_request_carries_canonical_net_allowed() {
    ensure_substrate_built();

    let temp = temp_dir("substrate-c5-net-allowed-nonpty-");
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

    let sock_temp = short_socket_dir("sub-c5-net-allowed-sock-");
    let sock = sock_temp.path().join("world.sock");
    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _sock = AgentSocket::start(
        &sock,
        SocketResponse::CapabilitiesAndExecuteRecord {
            stdout: "__C5_NET_ALLOWED_NONPTY__\n".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
            records: records.clone(),
        },
    );

    let out = substrate_base_command(&project, &home, &substrate_home, &sock)
        .arg("-c")
        .arg("echo __C5_NET_ALLOWED_NONPTY__")
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

#[test]
#[serial]
fn command_mode_world_consistency_v1_c_mode_does_not_host_canonicalize_cd_pwd() {
    ensure_substrate_built();

    let temp = temp_dir("substrate-c5-command-mode-cd-");
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

    let sock_temp = short_socket_dir("sub-c5-sock-cd-");
    let sock = sock_temp.path().join("world.sock");
    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _sock = AgentSocket::start(
        &sock,
        SocketResponse::CapabilitiesAndExecuteRecord {
            stdout: "__C5_C_MODE_CD_PWD__\n".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
            records: records.clone(),
        },
    );

    let dir_name = "c5_world_only_dir";
    let host_path = project.join(dir_name);
    assert!(
        !host_path.exists(),
        "expected host path to not exist before test: {}",
        host_path.display()
    );

    // Create the directory in the world overlay via a non-builtin command.
    let out = substrate_base_command(&project, &home, &substrate_home, &sock)
        .arg("-c")
        .arg(format!("mkdir -p {dir_name}"))
        .output()
        .expect("run substrate -c mkdir");
    assert!(
        out.status.success(),
        "substrate -c mkdir failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !host_path.exists(),
        "expected host path to still not exist after mkdir (world-only view): {}",
        host_path.display()
    );

    // Regression: host-side fs::canonicalize()-driven cd failures.
    // If `cd` is interpreted on the host, it will fail because the path does not exist there.
    let out = substrate_base_command(&project, &home, &substrate_home, &sock)
        .arg("-c")
        .arg(format!("cd {dir_name} && pwd -P"))
        .output()
        .expect("run substrate -c cd/pwd");
    assert!(
        out.status.success(),
        "substrate -c cd/pwd failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let recorded = records.lock().expect("lock records");
    let cmds: Vec<String> = recorded
        .iter()
        .filter_map(|value| value.get("cmd")?.as_str().map(|s| s.to_string()))
        .collect();
    assert!(
        cmds.iter()
            .any(|cmd| cmd.contains(&format!("mkdir -p {dir_name}"))),
        "expected mkdir command to be sent to world-agent; cmds={cmds:?}"
    );
    assert!(
        cmds.iter()
            .any(|cmd| cmd.contains(&format!("cd {dir_name}")) && cmd.contains("pwd -P")),
        "expected cd/pwd command text to be sent to world-agent (not host builtins); cmds={cmds:?}"
    );
}

#[test]
#[serial]
fn command_mode_world_consistency_v1_pipe_mode_does_not_host_canonicalize_cd_pwd() {
    ensure_substrate_built();

    let temp = temp_dir("substrate-c5-pipe-mode-cd-");
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

    let sock_temp = short_socket_dir("sub-c5-sock-pipe-");
    let sock = sock_temp.path().join("world.sock");
    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _sock = AgentSocket::start(
        &sock,
        SocketResponse::CapabilitiesAndExecuteRecord {
            stdout: "__C5_PIPE_MODE_CD_PWD__\n".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
            records: records.clone(),
        },
    );

    let dir_name = "c5_world_only_dir";
    let host_path = project.join(dir_name);
    assert!(
        !host_path.exists(),
        "expected host path to not exist before test: {}",
        host_path.display()
    );

    let mut child = substrate_base_command(&project, &home, &substrate_home, &sock)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn substrate pipe mode");

    {
        let mut stdin = child.stdin.take().expect("child stdin");
        writeln!(stdin, "mkdir -p {dir_name}").expect("write pipe mkdir");
        writeln!(stdin, "cd {dir_name} && pwd -P").expect("write pipe cd/pwd");
        stdin.flush().expect("flush pipe commands");
    }

    let output = child.wait_with_output().expect("pipe output");
    assert!(
        output.status.success(),
        "substrate pipe mode cd/pwd failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !host_path.exists(),
        "expected host path to still not exist after pipe mode (world-only view): {}",
        host_path.display()
    );

    let recorded = records.lock().expect("lock records");
    let cmds: Vec<String> = recorded
        .iter()
        .filter_map(|value| value.get("cmd")?.as_str().map(|s| s.to_string()))
        .collect();
    assert!(
        cmds.iter()
            .any(|cmd| cmd.contains(&format!("mkdir -p {dir_name}"))),
        "expected mkdir command to be sent to world-agent; cmds={cmds:?}"
    );
    assert!(
        cmds.iter()
            .any(|cmd| cmd.contains(&format!("cd {dir_name}")) && cmd.contains("pwd -P")),
        "expected cd/pwd command text to be sent to world-agent (not host builtins); cmds={cmds:?}"
    );
}

#[test]
#[serial]
fn command_mode_world_consistency_v1_colon_host_is_literal_in_c_mode_and_pipe_mode() {
    ensure_substrate_built();

    let temp = temp_dir("substrate-c5-colon-host-");
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

    let sock_temp = short_socket_dir("sub-c5-sock-host-");
    let sock = sock_temp.path().join("world.sock");
    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _sock = AgentSocket::start(
        &sock,
        SocketResponse::CapabilitiesAndExecuteRecord {
            stdout: "".to_string(),
            stderr: "command not found".to_string(),
            exit: 127,
            scopes: vec![],
            records: records.clone(),
        },
    );

    // `:host` must not be recognized in `-c/--command`; it should be treated as literal text.
    let out = substrate_base_command(&project, &home, &substrate_home, &sock)
        .arg("-c")
        .arg(":host echo __C5_COLON_HOST_C__")
        .output()
        .expect("run substrate -c :host");
    assert!(
        !out.status.success(),
        "expected -c ':host …' to fail as a literal command; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // Pipe mode must follow the same `:host` non-recognition rules.
    let mut child = substrate_base_command(&project, &home, &substrate_home, &sock)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn substrate pipe mode");

    {
        let mut stdin = child.stdin.take().expect("child stdin");
        writeln!(stdin, ":host echo __C5_COLON_HOST_PIPE__").expect("write pipe :host");
        stdin.flush().expect("flush pipe :host");
    }

    let output = child.wait_with_output().expect("pipe output");
    assert!(
        !output.status.success(),
        "expected pipe ':host …' to fail as a literal command; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let recorded = records.lock().expect("lock records");
    let cmds: Vec<String> = recorded
        .iter()
        .filter_map(|value| value.get("cmd")?.as_str().map(|s| s.to_string()))
        .collect();
    assert!(
        cmds.iter().any(|cmd| cmd.contains(":host")),
        "expected ':host' to be forwarded as literal command text; cmds={cmds:?}"
    );
}
