#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

struct PolicyFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl PolicyFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-policy-");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("create HOME fixture");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME fixture");
        let workspace_root = temp.path().join("workspace");
        fs::create_dir_all(&workspace_root).expect("create workspace root");
        Self {
            _temp: temp,
            home,
            substrate_home,
            workspace_root,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home);
        cmd
    }

    fn init_workspace(&self) {
        let output = self
            .command()
            .arg("workspace")
            .arg("init")
            .arg(&self.workspace_root)
            .output()
            .expect("workspace init");
        assert!(
            output.status.success(),
            "workspace init should succeed: {output:?}"
        );
    }

    fn workspace_policy_path(&self) -> PathBuf {
        self.workspace_root.join(".substrate").join("policy.yaml")
    }

    fn global_policy_path(&self) -> PathBuf {
        self.substrate_home.join("policy.yaml")
    }
}

fn policy_yaml_with_id(id: &str) -> String {
    format!(
        r#"id: "{id}"
name: "{id}"

world_fs:
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
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

metadata: {{}}
"#
    )
}

fn policy_show_json(cmd: &mut Command) -> JsonValue {
    let output = cmd
        .arg("policy")
        .arg("show")
        .arg("--json")
        .output()
        .expect("run substrate policy show --json");
    assert!(
        output.status.success(),
        "policy show --json should succeed: {output:?}"
    );
    serde_json::from_slice(&output.stdout).expect("policy show should output JSON")
}

#[test]
fn policy_discovery_prefers_workspace_policy_over_global() {
    let fixture = PolicyFixture::new();
    fixture.init_workspace();

    fs::write(
        fixture.global_policy_path(),
        policy_yaml_with_id("global-policy"),
    )
    .expect("write global policy");
    fs::write(
        fixture.workspace_policy_path(),
        policy_yaml_with_id("workspace-policy"),
    )
    .expect("write workspace policy");

    let child = fixture.workspace_root.join("a/b");
    fs::create_dir_all(&child).expect("create child dir");

    let mut cmd = fixture.command();
    cmd.current_dir(&child);
    let json = policy_show_json(&mut cmd);
    assert_eq!(
        json.get("id").and_then(|v| v.as_str()),
        Some("workspace-policy"),
        "workspace policy should win when present"
    );
}

#[test]
fn policy_discovery_falls_back_to_global_then_default() {
    let fixture = PolicyFixture::new();
    fixture.init_workspace();
    fs::remove_file(fixture.workspace_policy_path()).expect("remove workspace policy");

    fs::write(
        fixture.global_policy_path(),
        policy_yaml_with_id("global-policy"),
    )
    .expect("write global policy");

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root);
    let json = policy_show_json(&mut cmd);
    assert_eq!(
        json.get("id").and_then(|v| v.as_str()),
        Some("global-policy"),
        "global policy should be used when workspace policy missing"
    );

    fs::remove_file(fixture.global_policy_path()).expect("remove global policy");
    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root);
    let json = policy_show_json(&mut cmd);
    assert_eq!(
        json.get("id").and_then(|v| v.as_str()),
        Some("default"),
        "built-in default policy should be used when no policy files exist"
    );
}

#[test]
fn policy_workspace_scope_requires_workspace_root() {
    let fixture = PolicyFixture::new();

    let output = fixture
        .command()
        .current_dir(fixture._temp.path())
        .arg("policy")
        .arg("show")
        .output()
        .expect("run substrate policy show");

    assert_eq!(
        output.status.code(),
        Some(2),
        "workspace-scope policy commands should exit 2 outside a workspace: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("workspace init") || stderr.contains("workspace"),
        "expected actionable workspace error, got: {stderr}"
    );
}
