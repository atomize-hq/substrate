#![cfg(all(unix, target_os = "linux"))]

#[path = "common.rs"]
mod common;

use common::{substrate_shell_driver, temp_dir};
use std::fs;
use std::path::{Path, PathBuf};

fn manager_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/manager_hooks.yaml")
}

fn write_profile(project_dir: &Path, require_world: bool) {
    let require_world = if require_world { "true" } else { "false" };
    let profile = format!(
        r#"id: test-policy
name: Test Policy
world_fs:
  mode: writable
  cage: project
  require_world: {require_world}
  read_allowlist:
    - "*"
  write_allowlist: []
"#
    );
    fs::write(project_dir.join(".substrate-profile"), profile).expect("write .substrate-profile");
}

fn base_env_cmd(
    project_dir: &Path,
    home_dir: &Path,
    socket_path: &Path,
    trace_path: &Path,
) -> assert_cmd::Command {
    let mut cmd = substrate_shell_driver();
    cmd.current_dir(project_dir)
        .env("HOME", home_dir)
        .env("USERPROFILE", home_dir)
        .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
        .env("SHIM_TRACE_LOG", trace_path)
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD_SOCKET", socket_path);
    cmd
}

#[test]
fn require_world_true_refuses_host_fallback_when_world_socket_unavailable() {
    let temp = temp_dir("substrate-i1-require-world-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    write_profile(&project, true);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_path = temp.path().join("sockdir/missing.sock");
    fs::create_dir_all(socket_path.parent().expect("socket parent")).expect("create socket dir");

    let assert = base_env_cmd(&project, &home, &socket_path, &trace_path)
        .arg("-c")
        .arg("true")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("world execution required"),
        "expected fail-closed error, got: {stderr}"
    );
    assert!(
        stderr.contains("SUBSTRATE_WORLD_SOCKET override")
            || stderr.contains(socket_path.to_string_lossy().as_ref()),
        "expected stderr to mention socket override or path ({}), got: {stderr}",
        socket_path.display()
    );
}

#[test]
fn require_world_false_warns_once_and_falls_back_to_host_when_world_socket_unavailable() {
    let temp = temp_dir("substrate-i1-optional-world-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    write_profile(&project, false);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_path = temp.path().join("sockdir/missing.sock");
    fs::create_dir_all(socket_path.parent().expect("socket parent")).expect("create socket dir");

    let assert = base_env_cmd(&project, &home, &socket_path, &trace_path)
        .arg("-c")
        .arg("printf fallback-ok")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("fallback-ok"),
        "expected host fallback command output, got: {stdout}"
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("substrate: warn:") && stderr.contains("running direct"),
        "expected fallback warning in stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("SUBSTRATE_WORLD_SOCKET override")
            || stderr.contains(socket_path.to_string_lossy().as_ref()),
        "expected warning to mention socket override or path ({}), got: {stderr}",
        socket_path.display()
    );
}
