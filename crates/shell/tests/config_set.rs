#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct ConfigSetFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl ConfigSetFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-config-set-");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("failed to create HOME fixture");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME fixture");
        let workspace_root = temp.path().join("workspace");
        fs::create_dir_all(&workspace_root).expect("failed to create workspace root");
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

    fn workspace_config_path(&self) -> PathBuf {
        self.workspace_root
            .join(".substrate")
            .join("workspace.yaml")
    }

    fn init_workspace(&self) {
        let output = self
            .command()
            .arg("workspace")
            .arg("init")
            .arg(&self.workspace_root)
            .output()
            .expect("failed to run workspace init");
        assert!(
            output.status.success(),
            "workspace init should succeed: {output:?}"
        );
        assert!(
            self.workspace_config_path().exists(),
            "workspace init should create workspace.yaml"
        );
    }

    fn read_workspace_yaml(&self) -> YamlValue {
        let raw = fs::read_to_string(self.workspace_config_path()).expect("read workspace.yaml");
        serde_yaml::from_str(&raw).expect("workspace.yaml YAML parse")
    }

    fn set_json(&self, cwd: &Path, updates: &[&str]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd);
        cmd.arg("config").arg("set").arg("--json");
        for update in updates {
            cmd.arg(update);
        }
        cmd.output().expect("failed to run config set --json")
    }
}

#[test]
fn config_set_requires_workspace() {
    let fixture = ConfigSetFixture::new();
    let cwd = fixture._temp.path().join("not-a-workspace");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = fixture.set_json(&cwd, &["world.enabled=false"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "missing workspace should exit 2: {output:?}"
    );
}

#[test]
fn config_set_updates_workspace_and_prints_effective_config() {
    let fixture = ConfigSetFixture::new();
    fixture.init_workspace();

    let output = fixture.set_json(
        &fixture.workspace_root,
        &[
            "world.enabled=false",
            "world.anchor_mode=custom",
            "world.anchor_path=/tmp/custom-anchor",
            "policy.mode=enforce",
            "sync.exclude=[\"user\"]",
        ],
    );
    assert!(
        output.status.success(),
        "config set should succeed: {output:?}"
    );
    let json: JsonValue = serde_json::from_slice(&output.stdout).expect("effective JSON parse");

    assert_eq!(
        json.pointer("/world/enabled").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        json.pointer("/world/anchor_mode").and_then(|v| v.as_str()),
        Some("custom")
    );
    assert_eq!(
        json.pointer("/world/anchor_path").and_then(|v| v.as_str()),
        Some("/tmp/custom-anchor")
    );
    assert_eq!(
        json.pointer("/policy/mode").and_then(|v| v.as_str()),
        Some("enforce")
    );

    let exclude = json
        .pointer("/sync/exclude")
        .and_then(|v| v.as_array())
        .expect("sync.exclude array");
    let items = exclude
        .iter()
        .map(|v| v.as_str())
        .collect::<Option<Vec<_>>>()
        .expect("sync.exclude string array");
    assert_eq!(
        &items[..3],
        [".git/**", ".substrate/**", ".substrate-git/**"]
    );
    assert!(items.contains(&"user"));

    // The on-disk workspace.yaml stores only the user-provided excludes.
    let yaml = fixture.read_workspace_yaml();
    let root = yaml.as_mapping().expect("workspace.yaml root mapping");
    let sync = root
        .get(YamlValue::String("sync".to_string()))
        .and_then(|v| v.as_mapping())
        .expect("sync mapping");
    let stored = sync
        .get(YamlValue::String("exclude".to_string()))
        .and_then(|v| v.as_sequence())
        .expect("stored sync.exclude sequence");
    let stored_items = stored
        .iter()
        .map(|v| v.as_str())
        .collect::<Option<Vec<_>>>()
        .expect("stored exclude strings");
    assert_eq!(stored_items, vec!["user"]);
}

#[test]
fn config_set_supports_list_append_and_remove() {
    let fixture = ConfigSetFixture::new();
    fixture.init_workspace();
    let init = fixture.set_json(&fixture.workspace_root, &["sync.exclude=[\"a\",\"b\"]"]);
    assert!(init.status.success(), "precondition set should succeed");

    let output = fixture.set_json(
        &fixture.workspace_root,
        &["sync.exclude+=c", "sync.exclude-=a"],
    );
    assert!(output.status.success(), "append/remove should succeed");

    let yaml = fixture.read_workspace_yaml();
    let root = yaml.as_mapping().expect("workspace.yaml root mapping");
    let sync = root
        .get(YamlValue::String("sync".to_string()))
        .and_then(|v| v.as_mapping())
        .expect("sync mapping");
    let stored = sync
        .get(YamlValue::String("exclude".to_string()))
        .and_then(|v| v.as_sequence())
        .expect("stored sync.exclude sequence");
    let stored_items = stored
        .iter()
        .map(|v| v.as_str())
        .collect::<Option<Vec<_>>>()
        .expect("stored exclude strings");
    assert_eq!(stored_items, vec!["b", "c"]);
}

#[test]
fn config_set_rejects_unknown_key_without_mutation() {
    let fixture = ConfigSetFixture::new();
    fixture.init_workspace();
    let before = fs::read_to_string(fixture.workspace_config_path()).expect("read workspace.yaml");

    let output = fixture.set_json(&fixture.workspace_root, &["nope.key=true"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown key should exit 2: {output:?}"
    );

    let after = fs::read_to_string(fixture.workspace_config_path()).expect("read workspace.yaml");
    assert_eq!(before, after, "workspace.yaml must not change on error");
}

#[test]
fn config_set_accepts_boolean_synonyms() {
    let fixture = ConfigSetFixture::new();
    fixture.init_workspace();

    let output = fixture.set_json(&fixture.workspace_root, &["world.enabled=off"]);
    assert!(output.status.success(), "boolean synonym should succeed");

    let yaml = fixture.read_workspace_yaml();
    let root = yaml.as_mapping().expect("workspace.yaml root mapping");
    let world = root
        .get(YamlValue::String("world".to_string()))
        .and_then(|v| v.as_mapping())
        .expect("world mapping");
    assert_eq!(
        world
            .get(YamlValue::String("enabled".to_string()))
            .and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn config_set_rejects_invalid_list_literal_without_mutation() {
    let fixture = ConfigSetFixture::new();
    fixture.init_workspace();
    let before = fs::read_to_string(fixture.workspace_config_path()).expect("read workspace.yaml");

    let output = fixture.set_json(&fixture.workspace_root, &["sync.exclude=not-a-list"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid list literal should exit 2: {output:?}"
    );

    let after = fs::read_to_string(fixture.workspace_config_path()).expect("read workspace.yaml");
    assert_eq!(before, after, "workspace.yaml must not change on error");
}

#[test]
fn protected_excludes_are_always_present_in_effective_config() {
    let fixture = ConfigSetFixture::new();
    fixture.init_workspace();

    let output = fixture.set_json(
        &fixture.workspace_root,
        &[
            "sync.exclude=[\".git/**\",\"user\"]",
            "sync.exclude-=.git/**",
        ],
    );
    assert!(output.status.success(), "config set should succeed");

    let json: JsonValue = serde_json::from_slice(&output.stdout).expect("effective JSON parse");
    let exclude = json
        .pointer("/sync/exclude")
        .and_then(|v| v.as_array())
        .expect("sync.exclude array");
    let items = exclude
        .iter()
        .map(|v| v.as_str())
        .collect::<Option<Vec<_>>>()
        .expect("sync.exclude string array");
    assert_eq!(
        &items[..3],
        [".git/**", ".substrate/**", ".substrate-git/**"],
        "protected excludes must always be present"
    );
    assert!(items.contains(&"user"));
}
