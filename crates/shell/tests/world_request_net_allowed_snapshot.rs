#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod support;

use serde_json::Value;
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

fn write_config(home_substrate: &Path, world_net_filter: bool) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let config = format!(
        r#"world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ""
  caged: false
  net:
    filter: {}

policy:
  mode: observe
"#,
        if world_net_filter { "true" } else { "false" }
    );
    fs::write(home_substrate.join("config.yaml"), config).expect("write config.yaml");
}

fn short_socket_dir(prefix: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("create short socket tempdir in /tmp")
}

#[derive(Debug)]
struct RoutingCase<'a> {
    name: &'a str,
    net_allowed_yaml: &'a str,
    world_net_filter: bool,
    expected_net_allowed: &'a [&'a str],
    expected_isolate_network: bool,
    expected_allowed_domains: &'a [&'a str],
}

fn run_routing_case(case: &RoutingCase<'_>) {
    let temp = temp_dir("substrate-net-allowed-nonpty-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy_with_net_allowed(&substrate_home, case.net_allowed_yaml);
    write_config(&substrate_home, case.world_net_filter);

    let sock_temp = short_socket_dir("sub-net-allowed-sock-");
    let sock = sock_temp.path().join("world.sock");
    let records: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
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
        "{}: substrate -c failed: stdout={} stderr={}",
        case.name,
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
    assert_eq!(
        net_allowed, case.expected_net_allowed,
        "{}: unexpected canonical policy_snapshot.net_allowed",
        case.name
    );

    let isolate_network = request
        .pointer("/world_network/isolate_network")
        .and_then(|value| value.as_bool())
        .expect("world_network.isolate_network bool");
    assert!(
        isolate_network == case.expected_isolate_network,
        "{}: unexpected world_network.isolate_network",
        case.name
    );

    let allowed_domains = request
        .pointer("/world_network/allowed_domains")
        .and_then(|value| value.as_array())
        .expect("world_network.allowed_domains array");
    let allowed_domains: Vec<&str> = allowed_domains
        .iter()
        .map(|value| value.as_str().expect("allowed_domains string"))
        .collect();
    assert_eq!(
        allowed_domains, case.expected_allowed_domains,
        "{}: unexpected world_network.allowed_domains",
        case.name
    );
}

#[test]
#[serial]
fn nonpty_world_request_obeys_net_allowed_routing_matrix() {
    ensure_substrate_built();

    let cases = [
        RoutingCase {
            name: "gate off plus restrictive policy stays allow-all at routing layer",
            net_allowed_yaml: "[\" Example.COM. \", \"example.com\", \"Api.Example.com.\"]",
            world_net_filter: false,
            expected_net_allowed: &["example.com", "api.example.com"],
            expected_isolate_network: false,
            expected_allowed_domains: &[],
        },
        RoutingCase {
            name: "gate on plus allow-all singleton does not request isolation",
            net_allowed_yaml: "[\" * \"]",
            world_net_filter: true,
            expected_net_allowed: &["*"],
            expected_isolate_network: false,
            expected_allowed_domains: &[],
        },
        RoutingCase {
            name: "gate on plus deny-all requests isolation with empty allowlist",
            net_allowed_yaml: "[]",
            world_net_filter: true,
            expected_net_allowed: &[],
            expected_isolate_network: true,
            expected_allowed_domains: &[],
        },
        RoutingCase {
            name: "gate on plus restrictive allowlist requests isolation with canonical domains",
            net_allowed_yaml: "[\" Example.COM. \", \"example.com\", \"Api.Example.com.\"]",
            world_net_filter: true,
            expected_net_allowed: &["example.com", "api.example.com"],
            expected_isolate_network: true,
            expected_allowed_domains: &["example.com", "api.example.com"],
        },
    ];

    for case in &cases {
        run_routing_case(case);
    }
}
