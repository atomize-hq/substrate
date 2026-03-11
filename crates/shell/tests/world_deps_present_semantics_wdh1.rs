#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};

use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::Builder;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn write_executable(dir: &Path, name: &str, contents: &str) {
    fs::create_dir_all(dir).expect("create bin dir");
    let path = dir.join(name);
    fs::write(&path, contents).expect("write executable");
    let mut perms = fs::metadata(&path).expect("stat executable").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod executable");
}

fn global_config_path(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/config.yaml")
}

fn workspace_root(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join("ws")
}

fn workspace_config_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".substrate/workspace.yaml")
}

fn global_deps_dir(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/deps")
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

fn start_world_socket_host_execute(prefix: &str) -> (tempfile::TempDir, PathBuf, AgentSocket) {
    let sock_tmp = Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("socket tempdir");
    let socket_path = sock_tmp.path().join("world.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );
    (sock_tmp, socket_path, socket)
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

#[test]
fn test_current_list_applied_entrypoints_require_world_deps_wrapper_path_wdh1() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"unamewrap\"]\n",
    );
    write_file(&workspace_config_path(&ws_root), "{}\n");
    write_file(
        &global_deps_dir(&fixture).join("packages/unamewrap.yaml"),
        concat!(
            "version: 1\n",
            "name: unamewrap\n",
            "description: uname present semantics probe\n",
            "runnable: true\n",
            "entrypoints: [\"uname\"]\n",
            "install:\n",
            "  method: script\n",
            "  script: \"echo noop\"\n",
        ),
    );

    let world_bin = fixture.home().join("world-bin");

    let (_sock_tmp, socket_path, _socket) =
        start_world_socket_host_execute("substrate-wdh1-present-");

    let output_missing = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &world_bin)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "list", "applied", "--json"])
        .output()
        .expect("run current list applied");

    assert_eq!(
        output_missing.status.code().unwrap_or(-1),
        0,
        "expected exit 0, stdout={}, stderr={}",
        String::from_utf8_lossy(&output_missing.stdout),
        String::from_utf8_lossy(&output_missing.stderr)
    );

    let json_missing = parse_json(&output_missing.stdout);
    let items_missing = json_items_by_name(&json_missing);
    let item_missing = items_missing
        .get("unamewrap")
        .expect("items include unamewrap");
    assert_eq!(
        item_missing.get("world").and_then(|v| v.as_str()),
        Some("missing"),
        "expected unamewrap.world=missing when only /usr/bin/uname exists; got item={item_missing}"
    );

    write_executable(
        &world_bin,
        "uname",
        "#!/bin/sh\nexec /usr/bin/uname \"$@\"\n",
    );

    let output_present = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &world_bin)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "list", "applied", "--json"])
        .output()
        .expect("run current list applied (wrapper present)");

    assert_eq!(
        output_present.status.code().unwrap_or(-1),
        0,
        "expected exit 0, stdout={}, stderr={}",
        String::from_utf8_lossy(&output_present.stdout),
        String::from_utf8_lossy(&output_present.stderr)
    );

    let json_present = parse_json(&output_present.stdout);
    let items_present = json_items_by_name(&json_present);
    let item_present = items_present
        .get("unamewrap")
        .expect("items include unamewrap");
    assert_eq!(
        item_present.get("world").and_then(|v| v.as_str()),
        Some("present"),
        "expected unamewrap.world=present when wrapper exists under world-deps bin; got item={item_present}"
    );
}

#[test]
fn test_current_sync_fails_closed_on_entrypoint_collision_exit_5() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"aptdup\", \"scriptdup\"]\n",
    );
    write_file(&workspace_config_path(&ws_root), "{}\n");
    write_file(
        &global_deps_dir(&fixture).join("packages/aptdup.yaml"),
        concat!(
            "version: 1\n",
            "name: aptdup\n",
            "description: apt collision\n",
            "runnable: true\n",
            "entrypoints: [\"dup\"]\n",
            "install:\n",
            "  method: apt\n",
            "  apt:\n",
            "    - name: dup\n",
        ),
    );
    write_file(
        &global_deps_dir(&fixture).join("packages/scriptdup.yaml"),
        concat!(
            "version: 1\n",
            "name: scriptdup\n",
            "description: script collision\n",
            "runnable: true\n",
            "entrypoints: [\"dup\"]\n",
            "install:\n",
            "  method: script\n",
            "  script: \"echo noop\"\n",
        ),
    );

    let (_sock_tmp, socket_path, _socket, records) = start_world_socket_execute_record(
        "substrate-wdh1-collision-",
        "__SUBSTRATE_WDAP1__ dup 1 1.0.0\n",
        "",
        0,
    );

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "sync"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains("world deps wrapper collision"));

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter().any(|cmd| cmd.contains("dpkg-query")),
        "expected the read-only APT probe before collision detection; cmds={cmds:?}"
    );
    assert!(
        cmds.iter()
            .all(|cmd| !cmd.contains("apt-get") && !cmd.contains("echo noop")),
        "expected collision detection to stop before any mutating install commands; cmds={cmds:?}"
    );
}

#[test]
fn test_current_sync_creates_apt_entrypoint_wrappers_wdh1() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    write_file(
        &global_config_path(&fixture),
        "world:\n  deps:\n    builtins: disabled\n    enabled: [\"npm\"]\n",
    );
    write_file(&workspace_config_path(&ws_root), "{}\n");
    write_file(
        &global_deps_dir(&fixture).join("packages/npm.yaml"),
        concat!(
            "version: 1\n",
            "name: npm\n",
            "description: npm via apt (wrapper test)\n",
            "runnable: true\n",
            "entrypoints: [\"npm\", \"npx\"]\n",
            "install:\n",
            "  method: apt\n",
            "  apt:\n",
            "    - name: npm\n",
        ),
    );

    let (_sock_tmp, socket_path, _socket, records) = start_world_socket_execute_record(
        "substrate-wdh1-apt-wrappers-",
        "__SUBSTRATE_WDAP1__ npm 1 10.0.0-1\n",
        "",
        0,
    );

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "sync"])
        .assert()
        .success();

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter().any(|cmd| cmd.contains("dpkg-query")),
        "expected a read-only APT probe before wrapper creation; cmds={cmds:?}"
    );
    assert!(
        cmds.iter()
            .any(|cmd| cmd.contains("exec /usr/bin/npm \"$@\"") && cmd.contains("#!/bin/sh")),
        "expected an apt wrapper script that installs an /usr/bin/npm wrapper under world-deps bin; cmds={cmds:?}"
    );
    assert!(
        cmds.iter()
            .any(|cmd| cmd.contains("exec /usr/bin/npx \"$@\"") && cmd.contains("#!/bin/sh")),
        "expected an apt wrapper script that installs an /usr/bin/npx wrapper under world-deps bin; cmds={cmds:?}"
    );
}
