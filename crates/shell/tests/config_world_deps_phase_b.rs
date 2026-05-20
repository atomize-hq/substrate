#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::substrate_shell_driver;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

struct WorldDepsConfigFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl WorldDepsConfigFixture {
    fn new() -> Self {
        let temp = Builder::new()
            .prefix("substrate-config-world-deps-")
            .tempdir_in("/tmp")
            .expect("failed to allocate integration test temp dir");
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
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env_remove("SUBSTRATE_ANCHOR_MODE")
            .env_remove("SUBSTRATE_ANCHOR_PATH")
            .env_remove("SUBSTRATE_CAGED")
            .env_remove("SUBSTRATE_POLICY_MODE");
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

    fn workspace_disabled_path(&self) -> PathBuf {
        self.workspace_root
            .join(".substrate")
            .join("workspace.disabled")
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

    fn write_global_raw(&self, contents: &str) {
        fs::write(self.global_config_path(), contents).expect("write global config.yaml");
    }

    fn write_workspace_raw(&self, contents: &str) {
        let path = self.workspace_config_path();
        fs::create_dir_all(path.parent().expect("workspace config parent"))
            .expect("create workspace config dir");
        fs::write(path, contents).expect("write workspace.yaml");
    }

    fn read_global_raw(&self) -> Vec<u8> {
        fs::read(self.global_config_path()).expect("read global config.yaml")
    }

    fn read_workspace_raw(&self) -> Vec<u8> {
        fs::read(self.workspace_config_path()).expect("read workspace.yaml")
    }

    fn read_global_yaml(&self) -> YamlValue {
        let raw = fs::read_to_string(self.global_config_path()).expect("read global config.yaml");
        serde_yaml::from_str(&raw).expect("global config.yaml YAML parse")
    }

    fn read_workspace_yaml(&self) -> YamlValue {
        let raw = fs::read_to_string(self.workspace_config_path()).expect("read workspace.yaml");
        serde_yaml::from_str(&raw).expect("workspace.yaml YAML parse")
    }

    fn config_global_set_json(&self, cwd: &Path, updates: &[&str]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd)
            .arg("config")
            .arg("global")
            .arg("set")
            .arg("--json");
        for update in updates {
            cmd.arg(update);
        }
        cmd.output().expect("run config global set --json")
    }

    fn config_global_reset(&self, cwd: &Path, keys: &[&str]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd)
            .arg("config")
            .arg("global")
            .arg("reset");
        for key in keys {
            cmd.arg(key);
        }
        cmd.output().expect("run config global reset")
    }

    fn config_workspace_set_json(&self, cwd: &Path, updates: &[&str]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd)
            .arg("config")
            .arg("workspace")
            .arg("set")
            .arg("--json");
        for update in updates {
            cmd.arg(update);
        }
        cmd.output().expect("run config workspace set --json")
    }

    fn config_workspace_reset(&self, cwd: &Path, keys: &[&str]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd)
            .arg("config")
            .arg("workspace")
            .arg("reset");
        for key in keys {
            cmd.arg(key);
        }
        cmd.output().expect("run config workspace reset")
    }

    fn config_current_show_json(&self, cwd: &Path, explain: bool) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd)
            .arg("config")
            .arg("current")
            .arg("show")
            .arg("--json");
        if explain {
            cmd.arg("--explain");
        }
        cmd.output().expect("run config current show --json")
    }
}

fn yaml_mapping(value: &YamlValue) -> &serde_yaml::Mapping {
    value
        .as_mapping()
        .unwrap_or_else(|| panic!("expected YAML mapping, got {value:?}"))
}

fn yaml_get<'a>(root: &'a YamlValue, key: &str) -> Option<&'a YamlValue> {
    yaml_mapping(root).get(YamlValue::String(key.to_string()))
}

fn parse_explain(stderr: &[u8]) -> JsonValue {
    let text = String::from_utf8_lossy(stderr);
    let start = text
        .find('{')
        .unwrap_or_else(|| panic!("failed to locate JSON object in --explain stderr: {text}"));
    serde_json::from_str(&text[start..]).expect("explain JSON should parse from stderr JSON object")
}

fn explain_key<'a>(explain: &'a JsonValue, key: &str) -> &'a JsonValue {
    explain
        .get("keys")
        .and_then(|v| v.as_object())
        .and_then(|m| m.get(key))
        .unwrap_or_else(|| panic!("missing explain key {key}: {explain}"))
}

fn explain_layers(explain_key: &JsonValue) -> Vec<&str> {
    explain_key
        .get("sources")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("missing explain sources: {explain_key}"))
        .iter()
        .map(|src| {
            src.get("layer")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("missing explain source layer: {src}"))
        })
        .collect()
}

#[test]
fn config_world_deps_enabled_merges_across_scopes_and_explains_multi_source_provenance() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    let global = fixture.config_global_set_json(
        &fixture.workspace_root,
        &["world.deps.enabled+=a", "world.deps.enabled+=b"],
    );
    assert!(
        global.status.success(),
        "global set should succeed: {global:?}"
    );

    let workspace = fixture.config_workspace_set_json(
        &fixture.workspace_root,
        &["world.deps.enabled+=b", "world.deps.enabled+=c"],
    );
    assert!(
        workspace.status.success(),
        "workspace set should succeed: {workspace:?}"
    );

    let current = fixture.config_current_show_json(&fixture.workspace_root, true);
    assert!(
        current.status.success(),
        "config current show should succeed: {current:?}"
    );
    let json: JsonValue = serde_json::from_slice(&current.stdout).expect("current JSON parse");
    let enabled = json
        .pointer("/world/deps/enabled")
        .and_then(|v| v.as_array())
        .expect("world.deps.enabled should be an array");
    let items = enabled
        .iter()
        .map(|v| v.as_str())
        .collect::<Option<Vec<_>>>()
        .expect("world.deps.enabled should be string array");
    assert_eq!(items, vec!["a", "b", "c"]);

    let explain = parse_explain(&current.stderr);
    let enabled_explain = explain_key(&explain, "world.deps.enabled");
    assert_eq!(
        enabled_explain
            .get("merge_strategy")
            .and_then(|v| v.as_str()),
        Some("concat_dedupe_ordered_set"),
        "world.deps.enabled merge strategy mismatch: {enabled_explain}"
    );
    assert_eq!(
        explain_layers(enabled_explain),
        vec!["global_patch", "workspace_patch"]
    );
}

#[test]
fn config_world_deps_enabled_supports_remove_and_persists_explicit_empty_list() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    let global =
        fixture.config_global_set_json(&fixture.workspace_root, &["world.deps.enabled+=a"]);
    assert!(global.status.success(), "global append should succeed");

    let global =
        fixture.config_global_set_json(&fixture.workspace_root, &["world.deps.enabled-=a"]);
    assert!(global.status.success(), "global remove should succeed");

    let yaml = fixture.read_global_yaml();
    let world = yaml_get(&yaml, "world").expect("world mapping should exist");
    let deps = yaml_get(world, "deps").expect("world.deps mapping should exist");
    let enabled = yaml_get(deps, "enabled").expect("world.deps.enabled should remain present");
    let seq = enabled
        .as_sequence()
        .expect("world.deps.enabled should be a YAML sequence");
    assert!(
        seq.is_empty(),
        "expected explicit empty list for world.deps.enabled, got {enabled:?}"
    );

    let workspace = fixture.config_workspace_set_json(
        &fixture.workspace_root,
        &["world.deps.enabled+=b", "world.deps.enabled-=b"],
    );
    assert!(
        workspace.status.success(),
        "workspace remove should succeed"
    );

    let yaml = fixture.read_workspace_yaml();
    let world = yaml_get(&yaml, "world").expect("world mapping should exist");
    let deps = yaml_get(world, "deps").expect("world.deps mapping should exist");
    let enabled = yaml_get(deps, "enabled").expect("world.deps.enabled should remain present");
    let seq = enabled
        .as_sequence()
        .expect("world.deps.enabled should be a YAML sequence");
    assert!(
        seq.is_empty(),
        "expected explicit empty list for world.deps.enabled, got {enabled:?}"
    );
}

#[test]
fn config_workspace_reset_removes_world_deps_enabled_key_from_workspace_patch_mapping() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    let workspace =
        fixture.config_workspace_set_json(&fixture.workspace_root, &["world.deps.enabled+=a"]);
    assert!(workspace.status.success(), "workspace set should succeed");

    let before = fixture.read_workspace_yaml();
    let before_world = yaml_get(&before, "world").expect("world mapping should exist");
    let before_deps = yaml_get(before_world, "deps").expect("world.deps mapping should exist");
    assert!(
        yaml_get(before_deps, "enabled").is_some(),
        "precondition: expected world.deps.enabled present in workspace patch"
    );

    let reset = fixture.config_workspace_reset(&fixture.workspace_root, &["world.deps.enabled"]);
    assert!(
        reset.status.success(),
        "workspace reset should succeed: {reset:?}"
    );

    let after = fixture.read_workspace_yaml();
    let removed = match yaml_get(&after, "world") {
        None => true,
        Some(world) => match yaml_get(world, "deps") {
            None => true,
            Some(deps) => yaml_get(deps, "enabled").is_none(),
        },
    };
    assert!(
        removed,
        "expected world.deps.enabled removed from workspace patch, got: {after:?}"
    );
}

#[test]
fn config_world_deps_enum_keys_use_replace_precedence_with_single_source_provenance() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    let global = fixture.config_global_set_json(
        &fixture.workspace_root,
        &[
            "world.deps.inventory_mode=merged",
            "world.deps.builtins=enabled",
        ],
    );
    assert!(
        global.status.success(),
        "global set should succeed: {global:?}"
    );

    let workspace = fixture.config_workspace_set_json(
        &fixture.workspace_root,
        &[
            "world.deps.inventory_mode=workspace_only",
            "world.deps.builtins=disabled",
        ],
    );
    assert!(
        workspace.status.success(),
        "workspace set should succeed: {workspace:?}"
    );

    let current = fixture.config_current_show_json(&fixture.workspace_root, true);
    assert!(
        current.status.success(),
        "config current show should succeed: {current:?}"
    );
    let json: JsonValue = serde_json::from_slice(&current.stdout).expect("current JSON parse");
    assert_eq!(
        json.pointer("/world/deps/inventory_mode")
            .and_then(|v| v.as_str()),
        Some("workspace_only")
    );
    assert_eq!(
        json.pointer("/world/deps/builtins")
            .and_then(|v| v.as_str()),
        Some("disabled")
    );

    let explain = parse_explain(&current.stderr);
    let inv = explain_key(&explain, "world.deps.inventory_mode");
    assert_eq!(
        inv.get("merge_strategy").and_then(|v| v.as_str()),
        Some("replace")
    );
    assert_eq!(explain_layers(inv), vec!["workspace_patch"]);

    let builtins = explain_key(&explain, "world.deps.builtins");
    assert_eq!(
        builtins.get("merge_strategy").and_then(|v| v.as_str()),
        Some("replace")
    );
    assert_eq!(explain_layers(builtins), vec!["workspace_patch"]);

    let reset = fixture.config_workspace_reset(
        &fixture.workspace_root,
        &["world.deps.inventory_mode", "world.deps.builtins"],
    );
    assert!(
        reset.status.success(),
        "workspace reset should succeed: {reset:?}"
    );

    let current = fixture.config_current_show_json(&fixture.workspace_root, true);
    assert!(
        current.status.success(),
        "config current show should succeed after reset: {current:?}"
    );
    let json: JsonValue = serde_json::from_slice(&current.stdout).expect("current JSON parse");
    assert_eq!(
        json.pointer("/world/deps/inventory_mode")
            .and_then(|v| v.as_str()),
        Some("merged")
    );
    assert_eq!(
        json.pointer("/world/deps/builtins")
            .and_then(|v| v.as_str()),
        Some("enabled")
    );

    let explain = parse_explain(&current.stderr);
    assert_eq!(
        explain_layers(explain_key(&explain, "world.deps.inventory_mode")),
        vec!["global_patch"]
    );
    assert_eq!(
        explain_layers(explain_key(&explain, "world.deps.builtins")),
        vec!["global_patch"]
    );
}

#[test]
fn config_current_show_ignores_workspace_patch_when_workspace_is_disabled() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    let global = fixture.config_global_set_json(
        &fixture.workspace_root,
        &[
            "world.deps.enabled+=a",
            "world.deps.inventory_mode=merged",
            "world.deps.builtins=enabled",
        ],
    );
    assert!(
        global.status.success(),
        "global set should succeed: {global:?}"
    );

    let workspace = fixture.config_workspace_set_json(
        &fixture.workspace_root,
        &[
            "world.deps.enabled+=b",
            "world.deps.inventory_mode=workspace_only",
            "world.deps.builtins=disabled",
        ],
    );
    assert!(
        workspace.status.success(),
        "workspace set should succeed: {workspace:?}"
    );

    fs::write(fixture.workspace_disabled_path(), "disabled\n").expect("write workspace.disabled");

    let current = fixture.config_current_show_json(&fixture.workspace_root, true);
    assert!(
        current.status.success(),
        "config current show should succeed: {current:?}"
    );
    let json: JsonValue = serde_json::from_slice(&current.stdout).expect("current JSON parse");
    let enabled = json
        .pointer("/world/deps/enabled")
        .and_then(|v| v.as_array())
        .expect("world.deps.enabled should be an array");
    let items = enabled
        .iter()
        .map(|v| v.as_str())
        .collect::<Option<Vec<_>>>()
        .expect("world.deps.enabled should be string array");
    assert_eq!(items, vec!["a"]);

    assert_eq!(
        json.pointer("/world/deps/inventory_mode")
            .and_then(|v| v.as_str()),
        Some("merged")
    );
    assert_eq!(
        json.pointer("/world/deps/builtins")
            .and_then(|v| v.as_str()),
        Some("enabled")
    );

    let explain = parse_explain(&current.stderr);
    assert_eq!(
        explain_layers(explain_key(&explain, "world.deps.enabled")),
        vec!["global_patch"]
    );
    assert_eq!(
        explain_layers(explain_key(&explain, "world.deps.inventory_mode")),
        vec!["global_patch"]
    );
    assert_eq!(
        explain_layers(explain_key(&explain, "world.deps.builtins")),
        vec!["global_patch"]
    );
}

#[test]
fn config_world_deps_invalid_enum_values_exit_2_and_do_not_mutate_patch_bytes() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    fixture.write_global_raw("# config header\nworld:\n  deps:\n    builtins: enabled\n");
    fixture
        .write_workspace_raw("# workspace header\nworld:\n  deps:\n    inventory_mode: merged\n");

    let global_before = fixture.read_global_raw();
    let output = fixture.config_global_set_json(
        &fixture.workspace_root,
        &["world.deps.inventory_mode=bogus"],
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid enum should exit 2: {output:?}"
    );
    let global_after = fixture.read_global_raw();
    assert_eq!(
        global_before, global_after,
        "global patch bytes must not change on invalid enum value"
    );

    let workspace_before = fixture.read_workspace_raw();
    let output =
        fixture.config_workspace_set_json(&fixture.workspace_root, &["world.deps.builtins=bogus"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid enum should exit 2: {output:?}"
    );
    let workspace_after = fixture.read_workspace_raw();
    assert_eq!(
        workspace_before, workspace_after,
        "workspace patch bytes must not change on invalid enum value"
    );
}

#[test]
fn config_global_reset_removes_world_deps_keys_from_global_patch_mapping() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    let set = fixture.config_global_set_json(
        &fixture.workspace_root,
        &[
            "world.deps.enabled+=a",
            "world.deps.inventory_mode=merged",
            "world.deps.builtins=enabled",
        ],
    );
    assert!(
        set.status.success(),
        "precondition global set should succeed"
    );

    let before = fixture.read_global_yaml();
    let before_world = yaml_get(&before, "world").expect("world mapping should exist");
    let before_deps = yaml_get(before_world, "deps").expect("world.deps mapping should exist");
    assert!(
        yaml_get(before_deps, "enabled").is_some()
            && yaml_get(before_deps, "inventory_mode").is_some()
            && yaml_get(before_deps, "builtins").is_some(),
        "precondition: expected world.deps keys present in global patch"
    );

    let reset = fixture.config_global_reset(
        &fixture.workspace_root,
        &[
            "world.deps.enabled",
            "world.deps.inventory_mode",
            "world.deps.builtins",
        ],
    );
    assert!(
        reset.status.success(),
        "config global reset should succeed: {reset:?}"
    );

    let after = fixture.read_global_yaml();
    let removed = match yaml_get(&after, "world") {
        None => true,
        Some(world) => match yaml_get(world, "deps") {
            None => true,
            Some(deps) => {
                yaml_get(deps, "enabled").is_none()
                    && yaml_get(deps, "inventory_mode").is_none()
                    && yaml_get(deps, "builtins").is_none()
            }
        },
    };
    assert!(
        removed,
        "expected world.deps keys removed from global patch, got: {after:?}"
    );
}

#[test]
fn config_current_show_outputs_are_deterministic_without_changes() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    let global = fixture.config_global_set_json(
        &fixture.workspace_root,
        &[
            "world.deps.enabled+=a",
            "world.deps.enabled+=b",
            "world.deps.inventory_mode=merged",
            "world.deps.builtins=enabled",
        ],
    );
    assert!(
        global.status.success(),
        "global set should succeed: {global:?}"
    );

    let workspace = fixture.config_workspace_set_json(
        &fixture.workspace_root,
        &[
            "world.deps.enabled+=b",
            "world.deps.enabled+=c",
            "world.deps.inventory_mode=workspace_only",
            "world.deps.builtins=disabled",
        ],
    );
    assert!(
        workspace.status.success(),
        "workspace set should succeed: {workspace:?}"
    );

    let current_1 = fixture.config_current_show_json(&fixture.workspace_root, false);
    assert!(
        current_1.status.success(),
        "config current show should succeed: {current_1:?}"
    );
    let current_2 = fixture.config_current_show_json(&fixture.workspace_root, false);
    assert!(
        current_2.status.success(),
        "config current show should succeed: {current_2:?}"
    );
    assert_eq!(
        current_1.stdout, current_2.stdout,
        "config current show --json stdout must be deterministic for identical inputs"
    );
    assert_eq!(
        current_1.stderr, current_2.stderr,
        "config current show --json stderr must be deterministic for identical inputs"
    );

    let explain_1 = fixture.config_current_show_json(&fixture.workspace_root, true);
    assert!(
        explain_1.status.success(),
        "config current show --explain should succeed: {explain_1:?}"
    );
    let explain_2 = fixture.config_current_show_json(&fixture.workspace_root, true);
    assert!(
        explain_2.status.success(),
        "config current show --explain should succeed: {explain_2:?}"
    );
    assert_eq!(
        explain_1.stdout, explain_2.stdout,
        "config current show --json --explain stdout must be deterministic for identical inputs"
    );
    assert_eq!(
        explain_1.stderr, explain_2.stderr,
        "config current show --json --explain stderr must be deterministic for identical inputs"
    );
}

#[test]
fn config_world_deps_enabled_explain_reports_only_contributing_layers() {
    let fixture = WorldDepsConfigFixture::new();
    fixture.init_workspace();

    let global =
        fixture.config_global_set_json(&fixture.workspace_root, &["world.deps.enabled+=a"]);
    assert!(
        global.status.success(),
        "global set should succeed: {global:?}"
    );
    let current = fixture.config_current_show_json(&fixture.workspace_root, true);
    assert!(
        current.status.success(),
        "config current show should succeed: {current:?}"
    );
    let explain = parse_explain(&current.stderr);
    assert_eq!(
        explain_layers(explain_key(&explain, "world.deps.enabled")),
        vec!["global_patch"]
    );

    let reset = fixture.config_global_reset(&fixture.workspace_root, &["world.deps.enabled"]);
    assert!(
        reset.status.success(),
        "global reset should succeed: {reset:?}"
    );

    let workspace =
        fixture.config_workspace_set_json(&fixture.workspace_root, &["world.deps.enabled+=b"]);
    assert!(
        workspace.status.success(),
        "workspace set should succeed: {workspace:?}"
    );
    let current = fixture.config_current_show_json(&fixture.workspace_root, true);
    assert!(
        current.status.success(),
        "config current show should succeed: {current:?}"
    );
    let explain = parse_explain(&current.stderr);
    assert_eq!(
        explain_layers(explain_key(&explain, "world.deps.enabled")),
        vec!["workspace_patch"]
    );
}
