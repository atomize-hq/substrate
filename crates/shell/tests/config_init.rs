#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

struct GlobalConfigFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
}

impl GlobalConfigFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-config-global-");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("failed to create HOME fixture");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME fixture");
        Self {
            _temp: temp,
            home,
            substrate_home,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home);
        cmd
    }

    fn config_path(&self) -> PathBuf {
        self.substrate_home.join("config.yaml")
    }

    fn write_raw_config(&self, contents: &str) {
        fs::write(self.config_path(), contents).expect("failed to seed config.yaml");
    }

    fn read_raw_config(&self) -> String {
        fs::read_to_string(self.config_path()).expect("read config.yaml")
    }

    fn read_yaml_config(&self) -> YamlValue {
        serde_yaml::from_str(&self.read_raw_config()).expect("config.yaml should parse as YAML")
    }

    fn global_init(&self, force: bool) -> std::process::Output {
        let mut cmd = self.command();
        cmd.arg("config").arg("global").arg("init");
        if force {
            cmd.arg("--force");
        }
        cmd.output().expect("failed to run config global init")
    }

    fn global_show_json(&self) -> JsonValue {
        let mut cmd = self.command();
        let output = cmd
            .arg("config")
            .arg("global")
            .arg("show")
            .arg("--json")
            .output()
            .expect("failed to run config global show --json");
        assert!(
            output.status.success(),
            "config global show should succeed: {output:?}"
        );
        serde_json::from_slice(&output.stdout).expect("global show JSON should parse")
    }
}

#[test]
fn config_global_show_prints_defaults_when_missing() {
    let fixture = GlobalConfigFixture::new();
    assert!(
        !fixture.config_path().exists(),
        "precondition: config missing"
    );

    let json = fixture.global_show_json();
    assert_eq!(
        json.pointer("/world/enabled").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        json.pointer("/world/anchor_mode").and_then(|v| v.as_str()),
        Some("workspace")
    );
    assert_eq!(
        json.pointer("/world/anchor_path").and_then(|v| v.as_str()),
        Some("")
    );
    assert_eq!(
        json.pointer("/world/caged").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        json.pointer("/policy/mode").and_then(|v| v.as_str()),
        Some("observe")
    );
    assert_eq!(
        json.pointer("/sync/auto_sync").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        json.pointer("/sync/direction").and_then(|v| v.as_str()),
        Some("from_world")
    );
    assert_eq!(
        json.pointer("/sync/conflict_policy")
            .and_then(|v| v.as_str()),
        Some("prefer_host")
    );
    assert_eq!(
        json.pointer("/sync/exclude")
            .and_then(|v| v.as_array())
            .map(|v| v.len()),
        Some(0)
    );
}

#[test]
fn config_global_init_writes_default_config_yaml() {
    let fixture = GlobalConfigFixture::new();
    let output = fixture.global_init(false);
    assert!(
        output.status.success(),
        "config global init should succeed: {output:?}"
    );
    assert!(
        fixture.config_path().exists(),
        "config global init should create {}",
        fixture.config_path().display()
    );

    let yaml = fixture.read_yaml_config();
    let root = yaml.as_mapping().expect("yaml root mapping");
    assert!(root.contains_key(&YamlValue::String("world".to_string())));
    assert!(root.contains_key(&YamlValue::String("policy".to_string())));
    assert!(root.contains_key(&YamlValue::String("sync".to_string())));
}

#[test]
fn config_global_init_does_not_overwrite_without_force() {
    let fixture = GlobalConfigFixture::new();
    fixture.write_raw_config(
        "world:\n  enabled: false\n  anchor_mode: follow-cwd\n  anchor_path: /tmp/example\n  caged: false\npolicy:\n  mode: disabled\nsync:\n  auto_sync: true\n  direction: both\n  conflict_policy: abort\n  exclude: [\"user\"]\n",
    );
    let before = fixture.read_raw_config();

    let output = fixture.global_init(false);
    assert!(
        output.status.success(),
        "config global init should succeed (no overwrite): {output:?}"
    );
    let after = fixture.read_raw_config();
    assert_eq!(before, after, "init without --force must not overwrite");
}

#[test]
fn config_global_set_creates_file_and_applies_updates() {
    let fixture = GlobalConfigFixture::new();
    assert!(
        !fixture.config_path().exists(),
        "precondition: config missing"
    );

    let mut cmd = fixture.command();
    let output = cmd
        .arg("config")
        .arg("global")
        .arg("set")
        .arg("--json")
        .arg("world.enabled=false")
        .arg("policy.mode=enforce")
        .arg("sync.exclude=[\"a\",\"b\"]")
        .output()
        .expect("failed to run config global set");

    assert!(
        output.status.success(),
        "config global set should succeed: {output:?}"
    );
    assert!(
        fixture.config_path().exists(),
        "global set should create config"
    );

    let json: JsonValue = serde_json::from_slice(&output.stdout).expect("set output JSON parse");
    assert_eq!(
        json.pointer("/world/enabled").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        json.pointer("/policy/mode").and_then(|v| v.as_str()),
        Some("enforce")
    );
    assert_eq!(
        json.pointer("/sync/exclude")
            .and_then(|v| v.as_array())
            .and_then(|v| v.iter().map(|x| x.as_str()).collect::<Option<Vec<_>>>()),
        Some(vec!["a", "b"])
    );
}

#[test]
fn config_global_set_rejects_unknown_keys_without_writing() {
    let fixture = GlobalConfigFixture::new();
    assert!(
        !fixture.config_path().exists(),
        "precondition: config missing"
    );

    let output = fixture
        .command()
        .arg("config")
        .arg("global")
        .arg("set")
        .arg("nope.key=true")
        .output()
        .expect("failed to run config global set");

    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown key must return exit 2: {output:?}"
    );
    assert!(
        !fixture.config_path().exists(),
        "global set must not create config.yaml on error"
    );
}

#[test]
fn config_global_init_force_overwrites_existing_config() {
    let fixture = GlobalConfigFixture::new();
    fixture.write_raw_config(
        "world:\n  enabled: false\n  anchor_mode: custom\n  anchor_path: /tmp/custom\n  caged: false\npolicy:\n  mode: disabled\nsync:\n  auto_sync: true\n  direction: both\n  conflict_policy: abort\n  exclude: [\"user\"]\n",
    );

    let output = fixture.global_init(true);
    assert!(
        output.status.success(),
        "config global init --force should succeed: {output:?}"
    );

    let json = fixture.global_show_json();
    assert_eq!(
        json.pointer("/world/enabled").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        json.pointer("/world/anchor_mode").and_then(|v| v.as_str()),
        Some("workspace")
    );
    assert_eq!(
        json.pointer("/policy/mode").and_then(|v| v.as_str()),
        Some("observe")
    );
}
