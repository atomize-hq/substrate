#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, ShellEnvFixture};

use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn write_manager_manifest(fixture: &ShellEnvFixture, tools: &[&str]) -> PathBuf {
    let mut contents = String::from("version: 1\n\nmanagers:\n");
    for tool in tools {
        contents.push_str(&format!(
            "  - name: {tool}\n    guest_detect:\n      command: \"true\"\n"
        ));
    }
    fixture.write_manifest(&contents)
}

fn write_empty_world_deps_overlay(fixture: &ShellEnvFixture) -> PathBuf {
    let path = fixture.home().join("world-deps.overlay.yaml");
    fs::write(&path, "version: 1\nmanagers: []\n").expect("write world deps overlay");
    path
}

fn write_selection_file(path: &Path, yaml: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create selection parent directory");
    }
    fs::write(path, yaml).expect("write selection file");
}

fn read_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).expect("stdout is valid JSON")
}

fn selection_path_global(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/world-deps.selection.yaml")
}

fn selection_path_workspace(workspace: &Path) -> PathBuf {
    workspace.join(".substrate/world-deps.selection.yaml")
}

fn canonicalize_for_assert(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

#[test]
fn test_world_deps_missing_selection_is_noop_for_status_and_all() {
    let fixture = ShellEnvFixture::new();
    let manifest = write_manager_manifest(
        &fixture,
        &["substrate-test-tool-a", "substrate-test-tool-b"],
    );
    let overlay = write_empty_world_deps_overlay(&fixture);
    let workspace = fixture.home().join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace dir");
    let missing_socket = workspace.join("missing.substrate.sock");

    for args in [["status"].as_slice(), ["status", "--all"].as_slice()] {
        substrate_command_for_home(&fixture)
            .current_dir(&workspace)
            .env("SUBSTRATE_MANAGER_MANIFEST", &manifest)
            .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &overlay)
            .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
            .args(["world", "deps"])
            .args(args)
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "substrate: world deps not configured (selection file missing)",
            ))
            .stderr(predicates::str::contains("world backend unavailable").not());
    }
}

#[test]
fn test_world_deps_missing_selection_is_noop_for_sync() {
    let fixture = ShellEnvFixture::new();
    let manifest = write_manager_manifest(&fixture, &["substrate-test-tool-a"]);
    let overlay = write_empty_world_deps_overlay(&fixture);
    let workspace = fixture.home().join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace dir");
    let missing_socket = workspace.join("missing.substrate.sock");

    substrate_command_for_home(&fixture)
        .current_dir(&workspace)
        .env("SUBSTRATE_MANAGER_MANIFEST", &manifest)
        .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &overlay)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["world", "deps", "sync"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "substrate: world deps not configured (selection file missing)",
        ))
        .stderr(predicates::str::contains("world backend unavailable").not());
}

#[test]
fn test_world_deps_missing_selection_is_noop_for_install() {
    let fixture = ShellEnvFixture::new();
    let manifest = write_manager_manifest(&fixture, &["substrate-test-tool-a"]);
    let overlay = write_empty_world_deps_overlay(&fixture);
    let workspace = fixture.home().join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace dir");
    let missing_socket = workspace.join("missing.substrate.sock");

    substrate_command_for_home(&fixture)
        .current_dir(&workspace)
        .env("SUBSTRATE_MANAGER_MANIFEST", &manifest)
        .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &overlay)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["world", "deps", "install", "substrate-test-tool-a"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "substrate: world deps not configured (selection file missing)",
        ))
        .stderr(predicates::str::contains("world backend unavailable").not());
}

#[test]
fn test_world_deps_status_json_reports_workspace_selection_precedence() {
    let fixture = ShellEnvFixture::new();
    let manifest = write_manager_manifest(&fixture, &["substrate-test-tool-a"]);
    let overlay = write_empty_world_deps_overlay(&fixture);
    let workspace = fixture.home().join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace dir");
    let missing_socket = workspace.join("missing.substrate.sock");

    let workspace_selection = selection_path_workspace(&workspace);
    write_selection_file(&workspace_selection, "version: 1\nselected: []\n");

    let global_selection = selection_path_global(&fixture);
    write_selection_file(
        &global_selection,
        "version: 1\nselected:\n  - substrate-test-tool-a\n",
    );

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace)
        .env("SUBSTRATE_MANAGER_MANIFEST", &manifest)
        .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &overlay)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["world", "deps", "status", "--json"])
        .output()
        .expect("run world deps status --json");

    assert_eq!(
        output.status.code().unwrap_or(-1),
        0,
        "expected exit 0, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = read_json(&output.stdout);
    let selection = json
        .get("selection")
        .expect("status --json includes selection block");
    let workspace_selection_str = canonicalize_for_assert(&workspace_selection);
    let global_selection_str = canonicalize_for_assert(&global_selection);
    assert_eq!(
        selection.get("configured").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        selection.get("active_scope").and_then(|v| v.as_str()),
        Some("workspace")
    );
    assert_eq!(
        selection.get("active_path").and_then(|v| v.as_str()),
        Some(workspace_selection_str.as_str())
    );
    let shadowed = selection
        .get("shadowed_paths")
        .and_then(|v| v.as_array())
        .expect("selection.shadowed_paths is an array");
    assert!(
        shadowed
            .iter()
            .any(|value| value.as_str() == Some(global_selection_str.as_str())),
        "expected selection.shadowed_paths to include global selection path"
    );
    assert!(
        selection
            .get("selected")
            .and_then(|v| v.as_array())
            .is_some_and(|selected| selected.is_empty()),
        "expected selection.selected to be an empty array"
    );
    assert_eq!(
        selection
            .get("ignored_due_to_all")
            .and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn test_world_deps_status_all_ignores_selection_and_reports_unavailable_guests() {
    let fixture = ShellEnvFixture::new();
    let manifest = write_manager_manifest(
        &fixture,
        &["substrate-test-tool-a", "substrate-test-tool-b"],
    );
    let overlay = write_empty_world_deps_overlay(&fixture);
    let workspace = fixture.home().join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace dir");
    let missing_socket = workspace.join("missing.substrate.sock");

    let workspace_selection = selection_path_workspace(&workspace);
    write_selection_file(&workspace_selection, "version: 1\nselected: []\n");

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace)
        .env("SUBSTRATE_MANAGER_MANIFEST", &manifest)
        .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &overlay)
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["world", "deps", "status", "--json", "--all"])
        .output()
        .expect("run world deps status --json --all");

    assert_eq!(
        output.status.code().unwrap_or(-1),
        0,
        "expected exit 0, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = read_json(&output.stdout);
    let selection = json
        .get("selection")
        .expect("status --json includes selection block");
    assert_eq!(
        selection
            .get("ignored_due_to_all")
            .and_then(|v| v.as_bool()),
        Some(true)
    );

    let tools = json
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("status --json includes tools array");
    assert!(!tools.is_empty(), "expected at least one tool");
    let first_guest = tools[0]
        .get("guest")
        .expect("tools[0] includes guest block");
    assert_eq!(
        first_guest.get("status").and_then(|v| v.as_str()),
        Some("unavailable")
    );
    assert!(
        first_guest
            .get("reason")
            .and_then(|v| v.as_str())
            .is_some_and(|value| !value.trim().is_empty()),
        "expected guest.reason to be a non-empty string"
    );
}

#[test]
fn test_world_deps_invalid_selection_yaml_exits_2() {
    let fixture = ShellEnvFixture::new();
    let manifest = write_manager_manifest(&fixture, &["substrate-test-tool-a"]);
    let overlay = write_empty_world_deps_overlay(&fixture);
    let workspace = fixture.home().join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace dir");

    let workspace_selection = selection_path_workspace(&workspace);
    write_selection_file(&workspace_selection, "version: 1\nselected: [\n");

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace)
        .env("SUBSTRATE_MANAGER_MANIFEST", &manifest)
        .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &overlay)
        .args(["world", "deps", "status"])
        .output()
        .expect("run world deps status");

    assert_eq!(
        output.status.code().unwrap_or(-1),
        2,
        "expected exit 2, stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains(&canonicalize_for_assert(&workspace_selection)),
        "expected output to mention selection path"
    );
}

#[test]
fn test_world_deps_unknown_selected_tool_exits_2() {
    let fixture = ShellEnvFixture::new();
    let manifest = write_manager_manifest(&fixture, &["substrate-test-tool-a"]);
    let overlay = write_empty_world_deps_overlay(&fixture);
    let workspace = fixture.home().join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace dir");

    let workspace_selection = selection_path_workspace(&workspace);
    write_selection_file(
        &workspace_selection,
        "version: 1\nselected:\n  - not-a-real-tool\n",
    );

    let output = substrate_command_for_home(&fixture)
        .current_dir(&workspace)
        .env("SUBSTRATE_MANAGER_MANIFEST", &manifest)
        .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &overlay)
        .args(["world", "deps", "status"])
        .output()
        .expect("run world deps status");

    assert_eq!(
        output.status.code().unwrap_or(-1),
        2,
        "expected exit 2, stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
