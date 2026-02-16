#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};

use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::Builder;

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

fn package_yaml(name: &str) -> String {
    format!(
        "version: 1\nname: {name}\ndescription: {name}\nrunnable: true\nentrypoints: [{name}]\ninstall:\n  method: apt\n  apt:\n    - name: {name}\nprobe:\n  command: \"{name} --version\"\n"
    )
}

fn write_executable(dir: &Path, name: &str, contents: &str) {
    fs::create_dir_all(dir).expect("create bin dir");
    let path = dir.join(name);
    fs::write(&path, contents).expect("write executable");
    let mut perms = fs::metadata(&path).expect("stat executable").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod executable");
}

fn parse_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).expect("stdout should be valid JSON")
}

fn json_items_by_name(
    value: &serde_json::Value,
) -> std::collections::BTreeMap<String, serde_json::Value> {
    let items = value
        .get("items")
        .and_then(|v| v.as_array())
        .expect("output.items must be an array");
    let mut out = std::collections::BTreeMap::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .expect("each item has a string name")
            .to_string();
        out.insert(name, item.clone());
    }
    out
}

#[test]
fn test_current_list_applied_with_healthy_backend_exits_0_and_reports_world_status() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    // Determinism: remove builtins from the current inventory view so only test-owned items exist.
    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"a\"]\n",
    );
    write_file(
        &workspace_config_path(&ws_root),
        "world:\n  deps:\n    enabled: [\"b\"]\n",
    );

    for name in ["a", "b"] {
        write_file(
            &global_deps_dir(&fixture)
                .join("packages")
                .join(format!("{name}.yaml")),
            &package_yaml(name),
        );
    }

    let world_bin = fixture.home().join("world-bin");
    write_executable(&world_bin, "a", "#!/usr/bin/env bash\necho 1.0.0\n");

    // Keep the Unix socket path short to avoid `SUN_LEN` failures.
    let sock_tmp = Builder::new()
        .prefix("substrate-wdp2-")
        .tempdir_in("/tmp")
        .expect("socket tempdir");
    let socket_path = sock_tmp.path().join("world.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    // On macOS, the shell normalizes PATH for Linux guests unless it already looks Linux-ish.
    // Keep this test deterministic and compatible with that normalization by starting from a
    // stable guest PATH.
    const GUEST_BASE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
    let prefix = world_bin.display().to_string();
    let new_path = format!("{prefix}:{GUEST_BASE_PATH}");

    let output = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("PATH", new_path)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "list", "applied", "--json"])
        .output()
        .expect("run current list applied");

    assert_eq!(
        output.status.code().unwrap_or(-1),
        0,
        "expected exit 0, stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json(&output.stdout);
    let items = json_items_by_name(&json);
    let a = items.get("a").expect("items include a");
    let b = items.get("b").expect("items include b");

    assert_eq!(
        a.get("world").and_then(|v| v.as_str()),
        Some("present"),
        "expected a.world=present, got item={a}"
    );
    assert_eq!(
        b.get("world").and_then(|v| v.as_str()),
        Some("missing"),
        "expected b.world=missing, got item={b}"
    );
}

#[test]
fn test_current_list_applied_backend_unavailable_exits_3_with_actionable_remediation() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"a\"]\n",
    );
    write_file(
        &global_deps_dir(&fixture).join("packages/a.yaml"),
        &package_yaml("a"),
    );
    write_file(&workspace_config_path(&ws_root), "{}\n");

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", ws_root.join("missing.sock"))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "list", "applied"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("world backend unavailable"))
        .stderr(predicate::str::contains("substrate world doctor"));
}

#[test]
fn test_current_show_explain_unknown_item_exits_2_not_3_even_if_backend_unavailable() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: []\n",
    );
    write_file(&workspace_config_path(&ws_root), "{}\n");

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", ws_root.join("missing.sock"))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args([
            "world",
            "deps",
            "current",
            "show",
            "not-a-real-item",
            "--explain",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("not-a-real-item"));
}

#[test]
fn test_current_show_explain_backend_unavailable_exits_3_for_known_item() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"a\"]\n",
    );
    write_file(
        &global_deps_dir(&fixture).join("packages/a.yaml"),
        &package_yaml("a"),
    );
    write_file(&workspace_config_path(&ws_root), "{}\n");

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", ws_root.join("missing.sock"))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "show", "a", "--explain"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("world backend unavailable"));
}
