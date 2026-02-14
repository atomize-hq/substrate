#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};

use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::Builder;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn workspace_root(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join("ws")
}

fn global_config_path(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/config.yaml")
}

fn workspace_config_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".substrate/workspace.yaml")
}

fn global_deps_dir(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/deps")
}

fn write_global_config_builtins_disabled(fixture: &ShellEnvFixture, enabled_yaml: &str) {
    write_file(
        &global_config_path(fixture),
        &format!("world:\n  deps:\n    builtins: disabled\n    enabled: {enabled_yaml}\n"),
    );
}

fn write_workspace_config(workspace_root: &Path, enabled_yaml: &str) {
    write_file(
        &workspace_config_path(workspace_root),
        &format!("world:\n  deps:\n    enabled: {enabled_yaml}\n"),
    );
}

fn seed_inventory_for_apt_and_script(fixture: &ShellEnvFixture, script_token: &str) {
    write_file(
        &global_deps_dir(fixture).join("packages/node.yaml"),
        r#"version: 1
name: node
description: node via apt
runnable: true
entrypoints: ["node"]
install:
  method: apt
  apt:
    - name: nodejs
probe:
  command: "true"
"#,
    );

    write_file(
        &global_deps_dir(fixture).join("packages/hello.yaml"),
        r#"version: 1
name: hello
description: hello via script
runnable: true
entrypoints: ["hello"]
install:
  method: script
  script_path: ../scripts/hello.sh
probe:
  command: "true"
"#,
    );

    write_file(
        &global_deps_dir(fixture).join("scripts/hello.sh"),
        &format!(
            r#"#!/bin/sh
set -eu
echo "{script_token}"
"#,
            script_token = script_token
        ),
    );
}

fn seed_manual_package(fixture: &ShellEnvFixture) {
    write_file(
        &global_deps_dir(fixture).join("packages/asdf-node.yaml"),
        r#"version: 1
name: asdf-node
description: manual node install
runnable: true
entrypoints: ["node", "npm", "npx"]
install:
  method: manual
  manual_instructions: |
    MANUAL_TOKEN: install node via asdf inside the world
probe:
  command: "command -v node >/dev/null 2>&1"
"#,
    );
}

fn start_world_socket_execute_record(
    prefix: &str,
    stdout: &str,
    stderr: &str,
    exit: i32,
) -> (
    tempfile::TempDir,
    PathBuf,
    AgentSocket,
    Arc<Mutex<Vec<serde_json::Value>>>,
) {
    let sock_tmp = Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("socket tempdir");
    let socket_path = sock_tmp.path().join("world.sock");

    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecuteRecord {
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit,
            scopes: vec![],
            records: records.clone(),
        },
    );

    (sock_tmp, socket_path, socket, records)
}

fn recorded_cmds(records: &Arc<Mutex<Vec<serde_json::Value>>>) -> Vec<String> {
    records
        .lock()
        .expect("lock records")
        .iter()
        .filter_map(|value| value.get("cmd")?.as_str().map(|s| s.to_string()))
        .collect()
}

fn first_index_containing(cmds: &[String], needle: &str) -> Option<usize> {
    cmds.iter().position(|cmd| cmd.contains(needle))
}

#[test]
fn test_current_install_manual_exits_4_and_does_not_call_world_agent() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    seed_manual_package(&fixture);
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdp5-manual-", "", "", 0);

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "install", "asdf-node"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("MANUAL (blocked):"))
        .stderr(predicate::str::contains("MANUAL_TOKEN"));

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.is_empty(),
        "expected no /v1/execute requests when manual items are blocked; cmds={cmds:?}"
    );
}

#[test]
fn test_current_list_available_ignores_legacy_world_deps_paths() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    seed_inventory_for_apt_and_script(&fixture, "SCRIPT_TOKEN_IGNORED");
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    // Seed legacy paths with intentionally invalid YAML: current inventory/enabled resolution
    // must not be influenced by these legacy files.
    write_file(
        &fixture.home().join(".substrate/manager_hooks.local.yaml"),
        "!!! not yaml\n",
    );
    write_file(
        &fixture.home().join(".substrate/world-deps.local.yaml"),
        "!!! not yaml\n",
    );
    write_file(
        &ws_root.join(".substrate/world-deps.selection.yaml"),
        "!!! not yaml\n",
    );
    write_file(&ws_root.join("config/manager_hooks.yaml"), "!!! not yaml\n");
    write_file(
        &ws_root.join("scripts/substrate/world-deps.yaml"),
        "!!! not yaml\n",
    );

    let out = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env(
            "SUBSTRATE_MANAGER_MANIFEST",
            ws_root.join("config/manager_hooks.yaml"),
        )
        .env(
            "SUBSTRATE_WORLD_DEPS_MANIFEST",
            ws_root.join("scripts/substrate/world-deps.yaml"),
        )
        .args(["world", "deps", "current", "list", "available", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: serde_json::Value = serde_json::from_slice(&out).expect("stdout JSON");
    let items = value["items"].as_array().cloned().unwrap_or_default();
    assert!(
        items
            .iter()
            .any(|it| it["kind"] == "package" && it["name"] == "node"),
        "expected current available to include seeded inventory package 'node'; items={items:?}"
    );
    assert!(
        items
            .iter()
            .any(|it| it["kind"] == "package" && it["name"] == "hello"),
        "expected current available to include seeded inventory package 'hello'; items={items:?}"
    );
}

#[test]
fn test_current_install_executes_apt_before_scripts_and_does_not_mutate_enabled_patches() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    let script_token = "SCRIPT_TOKEN_WDP5_INSTALL";
    seed_inventory_for_apt_and_script(&fixture, script_token);
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    let global_before = fs::read_to_string(global_config_path(&fixture)).expect("read config.yaml");
    let workspace_before =
        fs::read_to_string(workspace_config_path(&ws_root)).expect("read workspace.yaml");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdp5-install-", "", "", 0);

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "install", "node", "hello"])
        .assert()
        .success();

    let global_after = fs::read_to_string(global_config_path(&fixture)).expect("read config.yaml");
    let workspace_after =
        fs::read_to_string(workspace_config_path(&ws_root)).expect("read workspace.yaml");
    assert_eq!(
        global_before, global_after,
        "current install must not modify global enabled patch"
    );
    assert_eq!(
        workspace_before, workspace_after,
        "current install must not modify workspace enabled patch"
    );

    let cmds = recorded_cmds(&records);
    let apt_idx = first_index_containing(&cmds, "DEBIAN_FRONTEND=noninteractive")
        .or_else(|| first_index_containing(&cmds, "apt-get"));
    let script_idx = first_index_containing(&cmds, script_token);

    assert!(
        apt_idx.is_some(),
        "expected at least one apt install command to be sent to world-agent; cmds={cmds:?}"
    );
    assert!(
        script_idx.is_some(),
        "expected script install command to include token {script_token:?}; cmds={cmds:?}"
    );
    assert!(
        apt_idx.unwrap() < script_idx.unwrap(),
        "expected apt execution before script execution; cmds={cmds:?}"
    );
}

#[test]
fn test_current_sync_applies_effective_enabled_set_and_does_not_mutate_patches() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    let script_token = "SCRIPT_TOKEN_WDP5_SYNC";
    seed_inventory_for_apt_and_script(&fixture, script_token);
    write_global_config_builtins_disabled(&fixture, "[\"node\"]");
    write_workspace_config(&ws_root, "[\"hello\"]");

    let global_before = fs::read_to_string(global_config_path(&fixture)).expect("read config.yaml");
    let workspace_before =
        fs::read_to_string(workspace_config_path(&ws_root)).expect("read workspace.yaml");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdp5-sync-", "", "", 0);

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "sync"])
        .assert()
        .success();

    let global_after = fs::read_to_string(global_config_path(&fixture)).expect("read config.yaml");
    let workspace_after =
        fs::read_to_string(workspace_config_path(&ws_root)).expect("read workspace.yaml");
    assert_eq!(
        global_before, global_after,
        "current sync must not modify global enabled patch"
    );
    assert_eq!(
        workspace_before, workspace_after,
        "current sync must not modify workspace enabled patch"
    );

    let cmds = recorded_cmds(&records);
    assert!(
        first_index_containing(&cmds, "DEBIAN_FRONTEND=noninteractive")
            .or_else(|| first_index_containing(&cmds, "apt-get"))
            .is_some(),
        "expected apt execution during sync; cmds={cmds:?}"
    );
    assert!(
        first_index_containing(&cmds, script_token).is_some(),
        "expected script execution during sync; cmds={cmds:?}"
    );
}

#[test]
fn test_current_install_hardening_violation_exits_5_with_actionable_message() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    seed_inventory_for_apt_and_script(&fixture, "SCRIPT_TOKEN_UNUSED");
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, _records) = start_world_socket_execute_record(
        "substrate-wdp5-hardening-",
        "",
        "Permission denied: /var/lib/substrate/world-deps",
        13,
    );

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "install", "node"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains("blocked by hardening/cage"));
}
