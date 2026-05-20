#![cfg(all(unix, target_os = "linux"))]

#[path = "common.rs"]
mod common;

use common::substrate_shell_driver;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::Builder;

fn manager_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/manager_hooks.yaml")
}

fn count_warning_lines(stderr: &str) -> usize {
    stderr
        .lines()
        .filter(|line| line.trim_start().starts_with("substrate: warn:"))
        .count()
}

fn count_error_lines(stderr: &str) -> usize {
    stderr
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("substrate: error:")
                || trimmed.starts_with("substrate: fatal:")
                || trimmed.starts_with("Error:")
        })
        .count()
}

fn write_policy(substrate_home: &Path, fail_closed_routing: bool) {
    let fail_closed_routing = if fail_closed_routing { "true" } else { "false" };
    let profile = format!(
        r#"id: test-policy
name: Test Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: {fail_closed_routing}
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
    fs::create_dir_all(substrate_home).expect("create SUBSTRATE_HOME");
    fs::write(substrate_home.join("policy.yaml"), profile).expect("write policy.yaml");
}

fn base_env_cmd(
    project_dir: &Path,
    home_dir: &Path,
    socket_path: &Path,
    trace_path: &Path,
) -> assert_cmd::Command {
    let mut cmd = substrate_shell_driver();
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
fn fail_closed_routing_true_exits_3_when_world_socket_unavailable() {
    let temp = Builder::new()
        .prefix("substrate-i1-fail-closed-routing-")
        .tempdir_in("/tmp")
        .expect("failed to allocate integration test temp dir");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    write_policy(&home.join(".substrate"), true);

    let trace_path = temp.path().join("trace.jsonl");
    fs::write(&trace_path, "").expect("seed trace log");

    let socket_path = temp.path().join("sockdir/missing.sock");
    fs::create_dir_all(socket_path.parent().expect("socket parent")).expect("create socket dir");

    let marker = "should-not-run";
    let assert = base_env_cmd(&project, &home, &socket_path, &trace_path)
        .arg("-c")
        .arg(format!("printf {marker}"))
        .assert()
        .code(3);

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains(marker),
        "expected fail-closed routing to prevent host fallback, got stdout: {stdout}"
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert_eq!(
        count_error_lines(&stderr),
        1,
        "expected exactly one error line for fail-closed routing, got: {stderr}"
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
    let temp = Builder::new()
        .prefix("substrate-i1-optional-world-")
        .tempdir_in("/tmp")
        .expect("failed to allocate integration test temp dir");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    write_policy(&home.join(".substrate"), false);

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
    assert_eq!(
        count_warning_lines(&stderr),
        1,
        "expected exactly one warning line, got: {stderr}"
    );
    assert_eq!(
        count_error_lines(&stderr),
        0,
        "expected no error lines for require_world=false, got: {stderr}"
    );
    assert!(
        stderr.contains("running direct")
            || stderr.contains("running on host")
            || stderr.contains("running on the host"),
        "expected fallback warning to mention host fallback, got: {stderr}"
    );
    assert!(
        stderr.contains("SUBSTRATE_WORLD_SOCKET override")
            || stderr.contains(socket_path.to_string_lossy().as_ref()),
        "expected warning to mention socket override or path ({}), got: {stderr}",
        socket_path.display()
    );
}
