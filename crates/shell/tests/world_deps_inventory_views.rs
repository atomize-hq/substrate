#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, ShellEnvFixture};

use std::fs;
use std::path::{Path, PathBuf};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn ensure_workspace_root(root: &Path) {
    fs::create_dir_all(root.join(".substrate")).expect("create .substrate dir");
    write_file(&root.join(".substrate/workspace.yaml"), "{}\n");
}

fn global_deps_dir(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/deps")
}

fn workspace_deps_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".substrate/deps")
}

fn package_yaml(name: &str, description: &str, method: &str, entrypoints: &[&str]) -> String {
    let entrypoints_yaml = entrypoints
        .iter()
        .map(|e| format!("  - {e}"))
        .collect::<Vec<_>>()
        .join("\n");
    match method {
        "apt" => format!(
            "version: 1\nname: {name}\ndescription: {description}\nrunnable: true\nentrypoints:\n{entrypoints_yaml}\ninstall:\n  method: apt\n  apt:\n    - name: {name}\nprobe:\n  command: \"{name} --version\"\n"
        ),
        "script" => format!(
            "version: 1\nname: {name}\ndescription: {description}\nrunnable: true\nentrypoints:\n{entrypoints_yaml}\ninstall:\n  method: script\n  script_path: ../scripts/{name}.sh\nprobe:\n  command: \"{name} --version\"\n"
        ),
        _ => panic!("unsupported method"),
    }
}

fn pacman_package_yaml(name: &str, description: &str, packages: &[&str]) -> String {
    let packages_yaml = packages
        .iter()
        .map(|p| format!("    - {p}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "version: 1\nname: {name}\ndescription: {description}\nrunnable: false\ninstall:\n  method: pacman\n  pacman:\n{packages_yaml}\n"
    )
}

fn bundle_yaml(name: &str, description: &str, packages: &[&str]) -> String {
    let packages_yaml = packages
        .iter()
        .map(|p| format!("  - {p}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("version: 1\nname: {name}\ndescription: {description}\npackages:\n{packages_yaml}\n")
}

fn read_items(stdout: &[u8]) -> Vec<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_slice(stdout).expect("stdout JSON");
    value["items"].as_array().cloned().unwrap_or_default()
}

fn contains_item(items: &[serde_json::Value], kind: &str, name: &str) -> bool {
    items
        .iter()
        .any(|it| it["kind"] == kind && it["name"] == name)
}

#[test]
fn test_current_list_available_includes_builtins_global_and_workspace() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    let global_pkg_path = global_deps_dir(&fixture).join("packages/global-tool.yaml");
    write_file(
        &global_pkg_path,
        &package_yaml("global-tool", "global tool", "apt", &["global-tool"]),
    );

    let ws_pkg_path = workspace_deps_dir(&workspace_root).join("packages/ws-tool.yaml");
    write_file(
        &ws_pkg_path,
        &package_yaml("ws-tool", "ws tool", "apt", &["ws-tool"]),
    );

    let global_pacman_path = global_deps_dir(&fixture).join("packages/global-pacman.yaml");
    write_file(
        &global_pacman_path,
        &pacman_package_yaml("global-pacman", "global pacman tool", &["zstd", "pacman"]),
    );

    let ws_pacman_path = workspace_deps_dir(&workspace_root).join("packages/ws-pacman.yaml");
    write_file(
        &ws_pacman_path,
        &pacman_package_yaml("ws-pacman", "workspace pacman tool", &["pacman", "zstd"]),
    );

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "current", "list", "available", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let items = read_items(&output);
    assert!(contains_item(&items, "package", "bun"));
    assert!(contains_item(&items, "package", "node"));
    assert!(contains_item(&items, "package", "npm"));
    assert!(contains_item(&items, "bundle", "node-runtime"));
    assert!(contains_item(&items, "package", "global-tool"));
    assert!(contains_item(&items, "package", "ws-tool"));
    assert!(contains_item(&items, "package", "global-pacman"));
    assert!(contains_item(&items, "package", "ws-pacman"));
}

#[test]
fn test_inventory_mode_workspace_only_hides_builtins_and_global() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    write_file(
        &fixture.home().join(".substrate/config.yaml"),
        "world:\n  deps:\n    inventory_mode: workspace_only\n",
    );

    let global_pkg_path = global_deps_dir(&fixture).join("packages/global-tool.yaml");
    write_file(
        &global_pkg_path,
        &package_yaml("global-tool", "global tool", "apt", &["global-tool"]),
    );

    let ws_pkg_path = workspace_deps_dir(&workspace_root).join("packages/ws-tool.yaml");
    write_file(
        &ws_pkg_path,
        &package_yaml("ws-tool", "ws tool", "apt", &["ws-tool"]),
    );

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "current", "list", "available", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let items = read_items(&output);
    assert!(!contains_item(&items, "package", "bun"));
    assert!(!contains_item(&items, "package", "global-tool"));
    assert!(contains_item(&items, "package", "ws-tool"));
}

#[test]
fn test_builtins_disabled_hides_builtins() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    write_file(
        &fixture.home().join(".substrate/config.yaml"),
        "world:\n  deps:\n    builtins: disabled\n",
    );

    let global_pkg_path = global_deps_dir(&fixture).join("packages/global-tool.yaml");
    write_file(
        &global_pkg_path,
        &package_yaml("global-tool", "global tool", "apt", &["global-tool"]),
    );

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "current", "list", "available", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let items = read_items(&output);
    assert!(!contains_item(&items, "package", "bun"));
    assert!(contains_item(&items, "package", "global-tool"));
}

#[test]
fn test_merge_workspace_overrides_global_definition() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    let global_pkg_path = global_deps_dir(&fixture).join("packages/dupe.yaml");
    write_file(
        &global_pkg_path,
        &package_yaml("dupe", "global dupe", "apt", &["a"]),
    );

    let ws_pkg_path = workspace_deps_dir(&workspace_root).join("packages/dupe.yaml");
    write_file(
        &ws_pkg_path,
        &package_yaml("dupe", "workspace dupe", "script", &["b"]),
    );

    let out = substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "current", "show", "dupe", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: serde_json::Value = serde_json::from_slice(&out).expect("stdout JSON");
    let item = &value["item"];
    assert_eq!(item["kind"], "package");
    assert_eq!(item["description"], "workspace dupe");
    assert_eq!(item["entrypoints"], serde_json::json!(["b"]));
    assert_eq!(item["install"]["method"], "script");
}

#[test]
fn test_package_bundle_name_collision_is_exit_2() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    let global_pkg_path = global_deps_dir(&fixture).join("packages/collide.yaml");
    write_file(
        &global_pkg_path,
        &package_yaml("collide", "global collide", "apt", &["collide"]),
    );

    let ws_bundle_path = workspace_deps_dir(&workspace_root).join("bundles/collide.yaml");
    write_file(
        &ws_bundle_path,
        &bundle_yaml("collide", "ws collide", &["collide"]),
    );

    substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "current", "list", "available"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("name collision"));
}

#[test]
fn test_invalid_yaml_exits_2_for_global_list_available() {
    let fixture = ShellEnvFixture::new();
    let path = global_deps_dir(&fixture).join("packages/bad.yaml");
    write_file(
        &path,
        "version: 1\nname: bad\nrunnable: true\nentrypoints: [bad\n",
    );

    substrate_command_for_home(&fixture)
        .args(["world", "deps", "global", "list", "available"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("invalid YAML"));
}

#[test]
fn test_invalid_yaml_exits_2_for_workspace_list_available() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    let path = workspace_deps_dir(&workspace_root).join("packages/bad.yaml");
    write_file(
        &path,
        "version: 1\nname: bad\nrunnable: true\nentrypoints: [bad\n",
    );

    substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "workspace", "list", "available"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("invalid YAML"));
}

#[test]
fn test_current_show_unknown_item_is_exit_2() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "current", "show", "does-not-exist"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("unknown deps item"));
}

#[test]
fn test_global_list_available_keeps_pacman_explicit() {
    let fixture = ShellEnvFixture::new();
    let path = global_deps_dir(&fixture).join("packages/global-pacman.yaml");
    write_file(
        &path,
        &pacman_package_yaml("global-pacman", "global pacman tool", &["zstd", "pacman"]),
    );

    let output = substrate_command_for_home(&fixture)
        .args(["world", "deps", "global", "list", "available", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let items = read_items(&output);
    let item = items
        .iter()
        .find(|item| item["name"] == "global-pacman")
        .expect("global pacman item");
    assert_eq!(item["method"], serde_json::json!("pacman"));
}

#[test]
fn test_workspace_list_available_keeps_pacman_explicit() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    let path = workspace_deps_dir(&workspace_root).join("packages/ws-pacman.yaml");
    write_file(
        &path,
        &pacman_package_yaml("ws-pacman", "workspace pacman tool", &["pacman", "zstd"]),
    );

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "workspace", "list", "available", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let items = read_items(&output);
    let item = items
        .iter()
        .find(|item| item["name"] == "ws-pacman")
        .expect("workspace pacman item");
    assert_eq!(item["method"], serde_json::json!("pacman"));
}

#[test]
fn test_current_show_json_preserves_pacman_order() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    let path = global_deps_dir(&fixture).join("packages/pacman-order-json.yaml");
    write_file(
        &path,
        &pacman_package_yaml(
            "pacman-order-json",
            "pacman order json",
            &["zstd", "pacman"],
        ),
    );

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args([
            "world",
            "deps",
            "current",
            "show",
            "pacman-order-json",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: serde_json::Value = serde_json::from_slice(&output).expect("stdout JSON");
    let item = &value["item"];
    assert_eq!(item["install"]["method"], serde_json::json!("pacman"));
    assert_eq!(
        item["install"]["pacman"],
        serde_json::json!(["zstd", "pacman"])
    );
}

#[test]
fn test_current_show_yaml_preserves_pacman_order() {
    let fixture = ShellEnvFixture::new();
    let workspace_root = fixture.home().join("ws");
    fs::create_dir_all(&workspace_root).expect("create ws");
    ensure_workspace_root(&workspace_root);

    let path = global_deps_dir(&fixture).join("packages/pacman-order-yaml.yaml");
    write_file(
        &path,
        &pacman_package_yaml(
            "pacman-order-yaml",
            "pacman order yaml",
            &["pacman", "zstd"],
        ),
    );

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace_root)
        .args(["world", "deps", "current", "show", "pacman-order-yaml"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: serde_yaml::Value = serde_yaml::from_slice(&output).expect("stdout YAML");
    assert_eq!(
        value["install"]["method"].as_str(),
        Some("pacman"),
        "expected pacman method in YAML output"
    );
    let pacman_values = value["install"]["pacman"]
        .as_sequence()
        .expect("pacman package list");
    let rendered = pacman_values
        .iter()
        .map(|value| value.as_str().expect("string"))
        .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["pacman", "zstd"]);
}
