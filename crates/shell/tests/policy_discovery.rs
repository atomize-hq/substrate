#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
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
  host_visible: true
  fail_closed:
    routing: false
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
    )
}

fn policy_yaml_full_isolation_no_denies(read_allow: &[&str]) -> String {
    let allow_list_yaml = read_allow
        .iter()
        .map(|s| format!("      - \"{}\"\n", s.replace('"', "\\\"")))
        .collect::<String>();

    format!(
        r#"id: "full-isolation"
name: "full-isolation"

world_fs:
  host_visible: false
  fail_closed:
    routing: false
  write:
    enabled: true
  read:
    allow_list:
{allow_list_yaml}

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

fn policy_show_yaml(cmd: &mut Command) -> (String, YamlValue) {
    let output = cmd
        .arg("policy")
        .arg("show")
        .output()
        .expect("run substrate policy show");
    assert!(
        output.status.success(),
        "policy show should succeed: {output:?}"
    );

    let raw = String::from_utf8(output.stdout).expect("policy show should output UTF-8 YAML");
    let yaml: YamlValue = serde_yaml::from_str(&raw).expect("policy show should output YAML");
    (raw, yaml)
}

fn require_yaml_mapping<'a>(value: &'a YamlValue, label: &str) -> &'a serde_yaml::Mapping {
    value
        .as_mapping()
        .unwrap_or_else(|| panic!("{label} must be a YAML mapping, got: {value:?}"))
}

fn require_yaml_key<'a>(map: &'a serde_yaml::Mapping, key: &str) -> &'a YamlValue {
    map.get(&YamlValue::String(key.to_string()))
        .unwrap_or_else(|| panic!("missing YAML key: {key}"))
}

fn require_yaml_empty_list(value: &YamlValue, label: &str) {
    let seq = value
        .as_sequence()
        .unwrap_or_else(|| panic!("{label} must be a YAML list, got: {value:?}"));
    assert!(seq.is_empty(), "{label} must be empty, got: {seq:?}");
}

fn require_json_empty_list(value: &JsonValue, label: &str) {
    let array = value
        .as_array()
        .unwrap_or_else(|| panic!("{label} must be a JSON array, got: {value:?}"));
    assert!(array.is_empty(), "{label} must be empty, got: {array:?}");
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
        .arg("workspace")
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

#[test]
fn policy_show_yaml_is_v3_shaped_and_renders_empty_deny_lists_for_full_isolation() {
    let fixture = PolicyFixture::new();
    fixture.init_workspace();

    fs::write(
        fixture.workspace_policy_path(),
        policy_yaml_full_isolation_no_denies(&["./read-only"]),
    )
    .expect("write workspace policy");

    let child = fixture.workspace_root.join("a/b");
    fs::create_dir_all(&child).expect("create child dir");

    let mut cmd = fixture.command();
    cmd.current_dir(&child);
    let (_raw, yaml) = policy_show_yaml(&mut cmd);

    let root = require_yaml_mapping(&yaml, "policy root");
    let world_fs = require_yaml_key(root, "world_fs");
    let world_fs = require_yaml_mapping(world_fs, "world_fs");

    assert_eq!(
        world_fs
            .get(&YamlValue::String("host_visible".to_string()))
            .and_then(|v| v.as_bool()),
        Some(false),
        "expected world_fs.host_visible=false in effective output"
    );

    for legacy_key in ["mode", "isolation", "require_world", "enforcement"] {
        assert!(
            !world_fs.contains_key(&YamlValue::String(legacy_key.to_string())),
            "V2 key must not be rendered in operator-facing output: world_fs.{legacy_key}"
        );
    }
    for legacy_key in ["read_allowlist", "write_allowlist"] {
        assert!(
            !world_fs.contains_key(&YamlValue::String(legacy_key.to_string())),
            "V2 key must not be rendered in operator-facing output: world_fs.{legacy_key}"
        );
    }
    assert!(
        !root.contains_key(&YamlValue::String("world_fs_require_world".to_string())),
        "V2-shaped top-level key must not be rendered: world_fs_require_world"
    );

    let read = require_yaml_mapping(require_yaml_key(world_fs, "read"), "world_fs.read");
    let discover =
        require_yaml_mapping(require_yaml_key(world_fs, "discover"), "world_fs.discover");
    let write = require_yaml_mapping(require_yaml_key(world_fs, "write"), "world_fs.write");

    let read_allow = require_yaml_key(read, "allow_list")
        .as_sequence()
        .expect("world_fs.read.allow_list must be a list");
    let discover_allow = require_yaml_key(discover, "allow_list")
        .as_sequence()
        .expect("world_fs.discover.allow_list must be a list");
    assert_eq!(
        read_allow,
        &vec![YamlValue::String("read-only".to_string())],
        "read.allow_list should reflect the configured effective policy (normalized)"
    );
    assert_eq!(
        discover_allow, read_allow,
        "discover must be shown explicitly and default from read (same allow_list)"
    );

    require_yaml_key(write, "allow_list")
        .as_sequence()
        .expect("world_fs.write.allow_list must be a list");

    require_yaml_empty_list(
        require_yaml_key(read, "deny_list"),
        "world_fs.read.deny_list",
    );
    require_yaml_empty_list(
        require_yaml_key(discover, "deny_list"),
        "world_fs.discover.deny_list",
    );
    require_yaml_empty_list(
        require_yaml_key(write, "deny_list"),
        "world_fs.write.deny_list",
    );
}

#[test]
fn policy_show_json_is_v3_shaped_and_renders_empty_deny_lists_for_full_isolation() {
    let fixture = PolicyFixture::new();
    fixture.init_workspace();

    fs::write(
        fixture.workspace_policy_path(),
        policy_yaml_full_isolation_no_denies(&["./read-only"]),
    )
    .expect("write workspace policy");

    let child = fixture.workspace_root.join("a/b");
    fs::create_dir_all(&child).expect("create child dir");

    let mut cmd = fixture.command();
    cmd.current_dir(&child);
    let json = policy_show_json(&mut cmd);

    let world_fs = json
        .get("world_fs")
        .and_then(|v| v.as_object())
        .expect("world_fs must be an object");

    assert_eq!(
        world_fs.get("host_visible").and_then(|v| v.as_bool()),
        Some(false),
        "expected world_fs.host_visible=false in effective output"
    );

    for legacy_key in ["mode", "isolation", "require_world", "enforcement"] {
        assert!(
            !world_fs.contains_key(legacy_key),
            "V2 key must not be rendered in operator-facing output: world_fs.{legacy_key}"
        );
    }
    for legacy_key in ["read_allowlist", "write_allowlist"] {
        assert!(
            !world_fs.contains_key(legacy_key),
            "V2 key must not be rendered in operator-facing output: world_fs.{legacy_key}"
        );
    }
    assert!(
        !json.get("world_fs_require_world").is_some(),
        "V2-shaped top-level key must not be rendered: world_fs_require_world"
    );

    let read = world_fs
        .get("read")
        .and_then(|v| v.as_object())
        .expect("world_fs.read must be an object");
    let discover = world_fs
        .get("discover")
        .and_then(|v| v.as_object())
        .expect("world_fs.discover must be an object");
    let write = world_fs
        .get("write")
        .and_then(|v| v.as_object())
        .expect("world_fs.write must be an object");

    assert_eq!(
        read.get("allow_list")
            .and_then(|v| v.as_array())
            .expect("world_fs.read.allow_list must be an array"),
        &vec![JsonValue::String("read-only".to_string())],
        "read.allow_list should reflect the configured effective policy (normalized)"
    );
    assert_eq!(
        discover
            .get("allow_list")
            .and_then(|v| v.as_array())
            .expect("world_fs.discover.allow_list must be an array"),
        read.get("allow_list")
            .and_then(|v| v.as_array())
            .expect("world_fs.read.allow_list must be an array"),
        "discover must be shown explicitly and default from read (same allow_list)"
    );

    write
        .get("allow_list")
        .and_then(|v| v.as_array())
        .expect("world_fs.write.allow_list must be an array");

    require_json_empty_list(
        read.get("deny_list")
            .expect("world_fs.read.deny_list missing"),
        "world_fs.read.deny_list",
    );
    require_json_empty_list(
        discover
            .get("deny_list")
            .expect("world_fs.discover.deny_list missing"),
        "world_fs.discover.deny_list",
    );
    require_json_empty_list(
        write
            .get("deny_list")
            .expect("world_fs.write.deny_list missing"),
        "world_fs.write.deny_list",
    );
}
