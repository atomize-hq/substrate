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

fn normalize_stdout(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw)
        .replace("\r\n", "\n")
        .trim()
        .to_string()
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected output to contain {needle:?}, output={haystack:?}"
    );
}

fn assert_contains_case_insensitive(haystack: &str, needle: &str) {
    let lower = haystack.to_lowercase();
    assert!(
        lower.contains(&needle.to_lowercase()),
        "expected output to contain {needle:?} (case-insensitive), output={haystack:?}"
    );
}

fn assert_before(haystack: &str, a: &str, b: &str) {
    let pos_a = haystack.find(a).unwrap_or_else(|| {
        panic!("expected output to contain {a:?}, output={haystack:?}");
    });
    let pos_b = haystack.find(b).unwrap_or_else(|| {
        panic!("expected output to contain {b:?}, output={haystack:?}");
    });
    assert!(
        pos_a < pos_b,
        "expected {a:?} to appear before {b:?}, output={haystack:?}"
    );
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

fn seed_global_inventory(fixture: &ShellEnvFixture) {
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
  command: "node --version"
"#,
    );

    write_file(
        &global_deps_dir(fixture).join("packages/npm.yaml"),
        r#"version: 1
name: npm
description: npm via apt
runnable: true
entrypoints: ["npm", "npx"]
install:
  method: apt
  apt:
    - name: npm
probe:
  command: "npm --version && npx --version"
"#,
    );

    write_file(
        &global_deps_dir(fixture).join("packages/bun.yaml"),
        r#"version: 1
name: bun
description: bun via script
runnable: true
entrypoints: ["bun"]
install:
  method: script
  script_path: ../scripts/bun.sh
probe:
  command: "bun --version"
"#,
    );

    write_file(
        &global_deps_dir(fixture).join("scripts/bun.sh"),
        r#"#!/usr/bin/env bash
set -euo pipefail
echo "bun stub"
"#,
    );

    write_file(
        &global_deps_dir(fixture).join("bundles/node-runtime.yaml"),
        r#"version: 1
name: node-runtime
description: node + npm
packages: ["node", "npm"]
"#,
    );
}

fn write_global_config_builtins_disabled(fixture: &ShellEnvFixture, extra_yaml: &str) {
    write_file(
        &global_config_path(fixture),
        &format!(
            "world:\n  deps:\n    builtins: disabled\n{extra_yaml}",
            extra_yaml = extra_yaml
        ),
    );
}

#[test]
fn test_current_install_dry_run_prints_deterministic_plan_and_orders_apt_before_scripts() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    seed_global_inventory(&fixture);
    write_global_config_builtins_disabled(&fixture, "    enabled: []\n");

    let missing_socket = ws_root.join("missing.sock");

    let mut cmd1 = substrate_command_for_home(&fixture);
    cmd1.current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args([
            "world",
            "deps",
            "current",
            "install",
            "node-runtime",
            "bun",
            "--dry-run",
        ]);
    let out1 = cmd1.output().expect("run substrate");
    assert!(
        out1.status.success(),
        "expected exit 0, status={:?}\nstdout={}\nstderr={}",
        out1.status.code(),
        normalize_stdout(&out1.stdout),
        normalize_stdout(&out1.stderr)
    );
    let stdout1 = normalize_stdout(&out1.stdout);

    assert_contains_case_insensitive(&stdout1, "apt");
    assert_contains_case_insensitive(&stdout1, "script");
    assert_contains(&stdout1, "nodejs");
    assert_contains(&stdout1, "npm");
    assert_contains(&stdout1, "bun");
    assert_before(&stdout1, "nodejs", "bun");

    let mut cmd2 = substrate_command_for_home(&fixture);
    cmd2.current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args([
            "world",
            "deps",
            "current",
            "install",
            "node-runtime",
            "bun",
            "--dry-run",
        ]);
    let out2 = cmd2.output().expect("run substrate (second time)");
    assert!(
        out2.status.success(),
        "expected exit 0 (second run), status={:?}\nstdout={}\nstderr={}",
        out2.status.code(),
        normalize_stdout(&out2.stdout),
        normalize_stdout(&out2.stderr)
    );
    let stdout2 = normalize_stdout(&out2.stdout);
    assert_eq!(
        stdout1, stdout2,
        "expected deterministic dry-run stdout; first={stdout1:?} second={stdout2:?}"
    );
}

#[test]
fn test_current_sync_dry_run_prints_plan_for_effective_enabled_set() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    seed_global_inventory(&fixture);
    write_global_config_builtins_disabled(
        &fixture,
        "    enabled: [\"node-runtime\", \"bun\", \"bun\"]\n",
    );
    write_file(
        &workspace_config_path(&ws_root),
        "world:\n  deps:\n    enabled: [\"node-runtime\", \"node\"]\n",
    );

    let global_before = fs::read_to_string(global_config_path(&fixture)).expect("read config.yaml");
    let workspace_before =
        fs::read_to_string(workspace_config_path(&ws_root)).expect("read workspace.yaml");

    let missing_socket = ws_root.join("missing.sock");

    let mut cmd1 = substrate_command_for_home(&fixture);
    cmd1.current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "sync", "--dry-run"]);
    let out1 = cmd1.output().expect("run substrate");
    assert!(
        out1.status.success(),
        "expected exit 0, status={:?}\nstdout={}\nstderr={}",
        out1.status.code(),
        normalize_stdout(&out1.stdout),
        normalize_stdout(&out1.stderr)
    );
    let stdout1 = normalize_stdout(&out1.stdout);
    assert_contains_case_insensitive(&stdout1, "apt");
    assert_contains_case_insensitive(&stdout1, "script");
    assert_contains(&stdout1, "nodejs");
    assert_contains(&stdout1, "npm");
    assert_contains(&stdout1, "bun");
    assert_before(&stdout1, "nodejs", "bun");

    let global_after = fs::read_to_string(global_config_path(&fixture)).expect("read config.yaml");
    let workspace_after =
        fs::read_to_string(workspace_config_path(&ws_root)).expect("read workspace.yaml");
    assert_eq!(
        global_before, global_after,
        "dry-run must not modify global enabled patch"
    );
    assert_eq!(
        workspace_before, workspace_after,
        "dry-run must not modify workspace enabled patch"
    );

    let mut cmd2 = substrate_command_for_home(&fixture);
    cmd2.current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "sync", "--dry-run"]);
    let out2 = cmd2.output().expect("run substrate (second time)");
    assert!(
        out2.status.success(),
        "expected exit 0 (second run), status={:?}\nstdout={}\nstderr={}",
        out2.status.code(),
        normalize_stdout(&out2.stdout),
        normalize_stdout(&out2.stderr)
    );
    let stdout2 = normalize_stdout(&out2.stdout);
    assert_eq!(
        stdout1, stdout2,
        "expected deterministic dry-run stdout; first={stdout1:?} second={stdout2:?}"
    );
}

#[test]
fn test_current_install_dry_run_surfaces_manual_items_as_blocked() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    seed_global_inventory(&fixture);
    write_file(
        &global_deps_dir(&fixture).join("packages/asdf-node.yaml"),
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
    write_global_config_builtins_disabled(&fixture, "    enabled: []\n");

    let missing_socket = ws_root.join("missing.sock");

    let mut cmd = substrate_command_for_home(&fixture);
    cmd.current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args([
            "world",
            "deps",
            "current",
            "install",
            "asdf-node",
            "--dry-run",
        ]);
    let out = cmd.output().expect("run substrate");
    assert!(
        out.status.success(),
        "expected exit 0, status={:?}\nstdout={}\nstderr={}",
        out.status.code(),
        normalize_stdout(&out.stdout),
        normalize_stdout(&out.stderr)
    );
    let stdout = normalize_stdout(&out.stdout);
    assert_contains(&stdout, "asdf-node");
    assert_contains_case_insensitive(&stdout, "manual");
    assert_contains(&stdout, "MANUAL_TOKEN");
    assert_contains_case_insensitive(&stdout, "blocked");
}
