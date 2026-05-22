#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, ShellEnvFixture};

use predicates::prelude::*;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn global_config_path(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/config.yaml")
}

fn global_deps_dir(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/deps")
}

fn workspace_root(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join("ws")
}

fn workspace_config_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".substrate/workspace.yaml")
}

fn workspace_deps_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".substrate/deps")
}

fn package_yaml(name: &str) -> String {
    format!(
        "version: 1\nname: {name}\ndescription: {name}\nrunnable: true\nentrypoints: [{name}]\ninstall:\n  method: apt\n  apt:\n    - name: {name}\nprobe:\n  command: \"{name} --version\"\n"
    )
}

fn yaml_mapping(value: &YamlValue) -> &serde_yaml::Mapping {
    value
        .as_mapping()
        .unwrap_or_else(|| panic!("expected YAML mapping, got {value:?}"))
}

fn yaml_get<'a>(root: &'a YamlValue, key: &str) -> Option<&'a YamlValue> {
    yaml_mapping(root).get(YamlValue::String(key.to_string()))
}

fn yaml_string_seq(value: &YamlValue) -> Vec<String> {
    value
        .as_sequence()
        .unwrap_or_else(|| panic!("expected YAML sequence, got {value:?}"))
        .iter()
        .map(|v| {
            v.as_str()
                .unwrap_or_else(|| panic!("expected YAML string, got {v:?}"))
                .to_string()
        })
        .collect()
}

fn extract_leading_comment_header(raw: &str) -> String {
    let mut header = String::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            header.push_str(line);
            header.push('\n');
            continue;
        }
        break;
    }
    header
}

fn enabled_list_from_patch_yaml(raw: &str) -> Option<Vec<String>> {
    let value: YamlValue = serde_yaml::from_str(raw).expect("patch YAML should parse");
    let world = yaml_get(&value, "world")?;
    let deps = yaml_get(world, "deps")?;
    let enabled = yaml_get(deps, "enabled")?;
    Some(yaml_string_seq(enabled))
}

fn find_name_pos(haystack: &str, name: &str) -> usize {
    let patterns = [
        format!("\n- {name}\n"),
        format!("\n- \"{name}\"\n"),
        format!("\n{name}\n"),
        format!("\n\"{name}\"\n"),
        format!(" {name}\n"),
        format!("\"{name}\""),
        format!("- {name}"),
        format!("- \"{name}\""),
    ];
    patterns
        .iter()
        .filter_map(|pat| haystack.find(pat))
        .min()
        .unwrap_or_else(|| {
            panic!("expected stdout to contain item name {name:?}, stdout={haystack:?}")
        })
}

#[test]
fn test_global_add_preserves_comment_header_and_only_updates_global_patch() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_deps_dir(&fixture).join("packages/a.yaml"),
        &package_yaml("a"),
    );
    write_file(
        &global_deps_dir(&fixture).join("packages/b.yaml"),
        &package_yaml("b"),
    );
    write_file(
        &workspace_deps_dir(&ws_root).join("packages/w.yaml"),
        &package_yaml("w"),
    );

    let ws_patch_path = workspace_config_path(&ws_root);
    write_file(
        &ws_patch_path,
        "# workspace header line 1\n# workspace header line 2\nworld:\n  deps:\n    enabled: [\"w\"]\n",
    );
    let ws_before = fs::read_to_string(&ws_patch_path).expect("read workspace.yaml");

    let global_patch_path = global_config_path(&fixture);
    write_file(
        &global_patch_path,
        "# global header line 1\n# global header line 2\nworld:\n  deps:\n    enabled: [\"a\"]\n",
    );
    let global_before = fs::read_to_string(&global_patch_path).expect("read config.yaml");
    let global_header = extract_leading_comment_header(&global_before);

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", ws_root.join("missing.sock"))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "global", "add", "b"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Enabled deps updated (global): added:"))
        .stdout(predicate::str::contains(
            "enabled deps changes apply to the world only after 'substrate world deps current sync'",
        ));

    let global_after = fs::read_to_string(&global_patch_path).expect("read config.yaml after");
    assert!(
        global_after.starts_with(&global_header),
        "expected global patch comment header preserved; before_header={global_header:?}, after={global_after:?}"
    );
    let enabled = enabled_list_from_patch_yaml(&global_after).expect("enabled list exists");
    assert!(
        enabled == vec!["a".to_string(), "b".to_string()],
        "expected enabled list to be [\"a\", \"b\"], got: {enabled:?}"
    );

    let ws_after = fs::read_to_string(&ws_patch_path).expect("read workspace.yaml after");
    assert_eq!(
        ws_after, ws_before,
        "global add must not mutate workspace patch"
    );
}

#[test]
fn test_global_add_validates_against_global_inventory_only() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_deps_dir(&fixture).join("packages/a.yaml"),
        &package_yaml("a"),
    );
    write_file(
        &workspace_deps_dir(&ws_root).join("packages/w.yaml"),
        &package_yaml("w"),
    );
    write_file(
        &workspace_config_path(&ws_root),
        "world:\n  deps:\n    enabled: []\n",
    );
    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    enabled: []\n",
    );

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "global", "add", "w"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("w"))
        .stderr(predicate::str::contains("unknown").or(predicate::str::contains("Unknown")));
}

#[test]
fn test_workspace_add_respects_inventory_mode_workspace_only() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    inventory_mode: workspace_only\n",
    );

    write_file(
        &global_deps_dir(&fixture).join("packages/a.yaml"),
        &package_yaml("a"),
    );
    write_file(
        &workspace_deps_dir(&ws_root).join("packages/w.yaml"),
        &package_yaml("w"),
    );

    let ws_patch_path = workspace_config_path(&ws_root);
    write_file(
        &ws_patch_path,
        "# ws header\nworld:\n  deps:\n    enabled: []\n",
    );
    let ws_before = fs::read_to_string(&ws_patch_path).expect("read workspace.yaml");

    let global_patch_path = global_config_path(&fixture);
    let global_before = fs::read_to_string(&global_patch_path).expect("read config.yaml");

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "workspace", "add", "a"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("a"));

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "workspace", "add", "w"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Enabled deps updated (workspace): added:",
        ));

    let ws_after = fs::read_to_string(&ws_patch_path).expect("read workspace.yaml after");
    assert_ne!(
        ws_after, ws_before,
        "workspace add should update workspace patch"
    );
    let enabled = enabled_list_from_patch_yaml(&ws_after).expect("enabled list exists");
    assert!(
        enabled.contains(&"w".to_string()),
        "expected enabled list to contain 'w', got: {enabled:?}"
    );

    let global_after = fs::read_to_string(&global_patch_path).expect("read config.yaml after");
    assert_eq!(
        global_after, global_before,
        "workspace add must not mutate global patch"
    );
}

#[test]
fn test_scoped_list_enabled_is_patch_view_not_effective_merged() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_config_path(&fixture),
        "# global header\nworld:\n  deps:\n    enabled: [\"a\"]\n",
    );
    write_file(
        &workspace_config_path(&ws_root),
        "# ws header\nworld:\n  deps:\n    enabled: [\"w\"]\n",
    );

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "global", "list", "enabled"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\n    - a"))
        .stdout(predicate::str::contains("\n    - w").not());

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "workspace", "list", "enabled"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\n    - w"))
        .stdout(predicate::str::contains("\n    - a").not());
}

#[test]
fn test_current_list_enabled_merges_scopes_and_does_not_call_world_service() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    // Determinism: remove builtins from the current inventory view so only test-owned items exist.
    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"a\", \"b\"]\n",
    );
    write_file(
        &workspace_config_path(&ws_root),
        "world:\n  deps:\n    enabled: [\"b\", \"c\"]\n",
    );

    for name in ["a", "b", "c"] {
        write_file(
            &global_deps_dir(&fixture)
                .join("packages")
                .join(format!("{name}.yaml")),
            &package_yaml(name),
        );
    }

    let output = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", ws_root.join("missing.sock"))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "list", "enabled"])
        .output()
        .expect("run current list enabled");

    assert_eq!(
        output.status.code().unwrap_or(-1),
        0,
        "expected exit 0, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("showing current effective enabled deps list"),
        "expected note in stderr, got: {stderr}"
    );
    assert!(
        !stderr.contains("world backend unavailable"),
        "current list enabled must not call world-service: stderr={stderr}"
    );
    let a_pos = find_name_pos(&stdout, "a");
    let b_pos = find_name_pos(&stdout, "b");
    let c_pos = find_name_pos(&stdout, "c");
    assert!(
        a_pos < b_pos && b_pos < c_pos,
        "expected merged enabled order a,b,c; stdout={stdout:?}"
    );
}

#[test]
fn test_current_list_enabled_exits_2_when_enabled_not_in_inventory() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"missing\"]\n",
    );
    write_file(&workspace_config_path(&ws_root), "{}\n");

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", ws_root.join("missing.sock"))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "list", "enabled"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("missing"))
        .stderr(predicate::str::contains("world backend unavailable").not());
}

#[test]
fn test_global_and_workspace_reset_remove_enabled_key_and_preserve_comment_headers() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    let global_patch_path = global_config_path(&fixture);
    write_file(
        &global_patch_path,
        "# global header\nworld:\n  deps:\n    enabled: [\"a\"]\n    builtins: disabled\n",
    );
    let global_header = extract_leading_comment_header(
        &fs::read_to_string(&global_patch_path).expect("read global patch"),
    );

    let ws_patch_path = workspace_config_path(&ws_root);
    write_file(
        &ws_patch_path,
        "# ws header\nworld:\n  deps:\n    enabled: [\"w\"]\n",
    );
    let ws_header =
        extract_leading_comment_header(&fs::read_to_string(&ws_patch_path).expect("read ws patch"));

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "global", "reset"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Enabled deps reset (global)"));

    let global_after = fs::read_to_string(&global_patch_path).expect("read global patch after");
    assert!(
        global_after.starts_with(&global_header),
        "expected global comment header preserved after reset"
    );
    let enabled = enabled_list_from_patch_yaml(&global_after);
    assert!(
        enabled.is_none(),
        "expected global reset to remove world.deps.enabled key, got enabled={enabled:?} raw={global_after:?}"
    );

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "workspace", "reset"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Enabled deps reset (workspace)"));

    let ws_after = fs::read_to_string(&ws_patch_path).expect("read ws patch after");
    assert!(
        ws_after.starts_with(&ws_header),
        "expected workspace comment header preserved after reset"
    );
    let ws_enabled = enabled_list_from_patch_yaml(&ws_after);
    assert!(
        ws_enabled.is_none(),
        "expected workspace reset to remove world.deps.enabled key, got enabled={ws_enabled:?} raw={ws_after:?}"
    );
}

#[test]
fn test_global_remove_does_not_validate_inventory_preserves_header_and_warns_if_still_enabled_via_workspace(
) {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    let global_patch_path = global_config_path(&fixture);
    write_file(
        &global_patch_path,
        "# global header\nworld:\n  deps:\n    enabled: [\"unknown\"]\n",
    );
    let global_header = extract_leading_comment_header(
        &fs::read_to_string(&global_patch_path).expect("read global patch"),
    );

    let ws_patch_path = workspace_config_path(&ws_root);
    write_file(
        &ws_patch_path,
        "world:\n  deps:\n    enabled: [\"unknown\"]\n",
    );
    let ws_before = fs::read_to_string(&ws_patch_path).expect("read ws patch");

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "global", "remove", "unknown"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Enabled deps updated (global): removed:",
        ))
        .stdout(predicate::str::contains("still enabled via workspace"));

    let global_after = fs::read_to_string(&global_patch_path).expect("read global patch after");
    assert!(
        global_after.starts_with(&global_header),
        "expected global comment header preserved after remove"
    );
    let enabled = enabled_list_from_patch_yaml(&global_after).unwrap_or_default();
    assert!(
        enabled.is_empty(),
        "expected enabled list empty after remove, got {enabled:?}"
    );

    let ws_after = fs::read_to_string(&ws_patch_path).expect("read ws patch after");
    assert_eq!(
        ws_after, ws_before,
        "global remove must not mutate workspace patch"
    );
}

#[test]
fn test_workspace_remove_warns_if_still_enabled_via_global() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_deps_dir(&fixture).join("packages/a.yaml"),
        &package_yaml("a"),
    );
    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"a\"]\n",
    );
    write_file(
        &workspace_config_path(&ws_root),
        "world:\n  deps:\n    enabled: [\"a\"]\n",
    );

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .args(["world", "deps", "workspace", "remove", "a"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Enabled deps updated (workspace): removed:",
        ))
        .stdout(predicate::str::contains("still enabled via global"));
}
