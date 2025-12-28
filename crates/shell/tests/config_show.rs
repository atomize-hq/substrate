#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct ConfigShowFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl ConfigShowFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-config-show-");
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

    fn global_config_path(&self) -> PathBuf {
        self.substrate_home.join("config.yaml")
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

    fn write_global_config(&self, contents: &str) {
        fs::write(self.global_config_path(), contents).expect("failed to write global config");
    }

    fn write_workspace_config(&self, contents: &str) {
        let workspace_config = self.workspace_config_path();
        let parent = workspace_config.parent().expect("workspace config parent");
        fs::create_dir_all(parent).expect("create workspace config dir");
        fs::write(workspace_config, contents).expect("failed to write workspace.yaml");
    }

    fn show_json(&self, cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd);
        for (k, v) in env {
            cmd.env(k, v);
        }
        for arg in args {
            cmd.arg(arg);
        }
        cmd.arg("config").arg("show").arg("--json");
        cmd.output().expect("failed to run config show --json")
    }

    fn show_yaml(&self, cwd: &Path, env: &[(&str, &str)]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd);
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.arg("config").arg("show");
        cmd.output().expect("failed to run config show")
    }
}

fn assert_json_str(json: &JsonValue, pointer: &str, expected: &str) {
    assert_eq!(
        json.pointer(pointer).and_then(|v| v.as_str()),
        Some(expected),
        "pointer {pointer} mismatch: {json}"
    );
}

fn assert_json_bool(json: &JsonValue, pointer: &str, expected: bool) {
    assert_eq!(
        json.pointer(pointer).and_then(|v| v.as_bool()),
        Some(expected),
        "pointer {pointer} mismatch: {json}"
    );
}

#[test]
fn config_show_requires_workspace() {
    let fixture = ConfigShowFixture::new();
    let cwd = fixture._temp.path().join("not-a-workspace");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = fixture.show_json(&cwd, &[], &[]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "missing workspace should exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not in a workspace") && stderr.contains("substrate workspace init"),
        "missing-workspace message should direct init\nstderr: {stderr}"
    );
}

#[test]
fn config_show_emits_yaml_by_default() {
    let fixture = ConfigShowFixture::new();
    fixture.init_workspace();

    let output = fixture.show_yaml(&fixture.workspace_root, &[("SUBSTRATE_WORLD", "")]);
    assert!(
        output.status.success(),
        "config show should succeed: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let yaml: YamlValue = serde_yaml::from_str(&stdout).expect("config show YAML parse");
    let root = yaml.as_mapping().expect("yaml root mapping");
    assert!(root.contains_key(&YamlValue::String("world".to_string())));
    assert!(root.contains_key(&YamlValue::String("policy".to_string())));
    assert!(root.contains_key(&YamlValue::String("sync".to_string())));
}

#[test]
fn config_show_resolves_effective_config_with_precedence() {
    let fixture = ConfigShowFixture::new();
    fixture.write_global_config(
        "world:\n  enabled: false\n  anchor_mode: follow-cwd\n  anchor_path: /global/anchor\n  caged: false\npolicy:\n  mode: disabled\nsync:\n  auto_sync: true\n  direction: both\n  conflict_policy: abort\n  exclude: [\"global-only\"]\n",
    );
    fixture.init_workspace();
    fixture.write_workspace_config(
        "world:\n  enabled: true\n  anchor_mode: custom\n  anchor_path: /workspace/anchor\n  caged: true\npolicy:\n  mode: observe\nsync:\n  auto_sync: false\n  direction: from_host\n  conflict_policy: prefer_world\n  exclude: [\"workspace-only\"]\n",
    );

    let cwd = fixture.workspace_root.join("nested").join("child");
    fs::create_dir_all(&cwd).expect("create nested cwd");

    let output = fixture.show_json(
        &cwd,
        &[
            "--world",
            "--anchor-mode",
            "follow-cwd",
            "--anchor-path",
            "/cli/anchor",
            "--uncaged",
        ],
        &[
            ("SUBSTRATE_WORLD", "disabled"),
            ("SUBSTRATE_ANCHOR_MODE", "custom"),
            ("SUBSTRATE_ANCHOR_PATH", "/env/anchor"),
            ("SUBSTRATE_CAGED", "1"),
            ("SUBSTRATE_POLICY_MODE", "enforce"),
            ("SUBSTRATE_SYNC_AUTO_SYNC", "1"),
            ("SUBSTRATE_SYNC_DIRECTION", "both"),
            ("SUBSTRATE_SYNC_CONFLICT_POLICY", "abort"),
            ("SUBSTRATE_SYNC_EXCLUDE", "env-a,env-b"),
        ],
    );
    assert!(
        output.status.success(),
        "config show should succeed: {output:?}"
    );
    let json: JsonValue = serde_json::from_slice(&output.stdout).expect("config show JSON parse");

    // CLI overrides env/config for the subset of world keys with CLI flags.
    assert_json_bool(&json, "/world/enabled", true);
    assert_json_str(&json, "/world/anchor_mode", "follow-cwd");
    assert_json_str(&json, "/world/anchor_path", "/cli/anchor");
    assert_json_bool(&json, "/world/caged", false);

    // Env overrides workspace/global for the remaining keys.
    assert_json_str(&json, "/policy/mode", "enforce");
    assert_json_bool(&json, "/sync/auto_sync", true);
    assert_json_str(&json, "/sync/direction", "both");
    assert_json_str(&json, "/sync/conflict_policy", "abort");

    let exclude = json
        .pointer("/sync/exclude")
        .and_then(|v| v.as_array())
        .expect("sync.exclude should be array");
    let items = exclude
        .iter()
        .map(|v| v.as_str())
        .collect::<Option<Vec<_>>>()
        .expect("sync.exclude should be string array");
    assert_eq!(
        &items[..3],
        [".git/**", ".substrate/**", ".substrate-git/**"],
        "protected excludes must be present and leading"
    );
    assert!(items.contains(&"env-a"));
    assert!(items.contains(&"env-b"));
    assert!(!items.contains(&"global-only"));
    assert!(!items.contains(&"workspace-only"));
}

#[test]
fn config_show_rejects_legacy_workspace_settings_yaml() {
    let fixture = ConfigShowFixture::new();
    fixture.init_workspace();

    let legacy = fixture
        .workspace_root
        .join(".substrate")
        .join("settings.yaml");
    fs::write(&legacy, "world:\n  enabled: true\n").expect("write legacy settings.yaml");

    let output = fixture.show_json(&fixture.workspace_root, &[], &[]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "legacy settings.yaml should exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported legacy workspace config detected"),
        "stderr should mention legacy workspace config\nstderr: {stderr}"
    );
}

#[test]
fn config_show_strictly_rejects_unknown_keys() {
    let fixture = ConfigShowFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_config(
        "world:\n  enabled: true\n  anchor_mode: workspace\n  anchor_path: \"\"\n  caged: true\npolicy:\n  mode: observe\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\nextra: 1\n",
    );

    let output = fixture.show_json(&fixture.workspace_root, &[], &[]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown keys should exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown field") || stderr.contains("invalid YAML"),
        "stderr should mention unknown key\nstderr: {stderr}"
    );
}

#[test]
fn config_show_strictly_rejects_type_mismatches() {
    let fixture = ConfigShowFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_config(
        "world:\n  enabled: \"nope\"\n  anchor_mode: workspace\n  anchor_path: \"\"\n  caged: true\npolicy:\n  mode: observe\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    let output = fixture.show_json(&fixture.workspace_root, &[], &[("SUBSTRATE_WORLD", "")]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "type mismatch should exit 2: {output:?}"
    );
}
