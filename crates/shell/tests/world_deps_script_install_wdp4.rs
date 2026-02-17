#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};

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

fn chmod_x(path: &Path) {
    let mut perms = fs::metadata(path)
        .unwrap_or_else(|err| panic!("stat {path:?}: {err}"))
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap_or_else(|err| panic!("chmod {path:?}: {err}"));
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

fn write_global_config_builtins_disabled(fixture: &ShellEnvFixture, enabled: &[&str]) {
    let mut enabled_yaml = String::new();
    enabled_yaml.push('[');
    for (idx, name) in enabled.iter().enumerate() {
        if idx != 0 {
            enabled_yaml.push_str(", ");
        }
        enabled_yaml.push('"');
        enabled_yaml.push_str(name);
        enabled_yaml.push('"');
    }
    enabled_yaml.push(']');

    write_file(
        &global_config_path(fixture),
        &format!("world:\n  deps:\n    builtins: disabled\n    enabled: {enabled_yaml}\n"),
    );
}

fn write_workspace_config(workspace_root: &Path, enabled: &[&str]) {
    let mut enabled_yaml = String::new();
    enabled_yaml.push('[');
    for (idx, name) in enabled.iter().enumerate() {
        if idx != 0 {
            enabled_yaml.push_str(", ");
        }
        enabled_yaml.push('"');
        enabled_yaml.push_str(name);
        enabled_yaml.push('"');
    }
    enabled_yaml.push(']');

    write_file(
        &workspace_config_path(workspace_root),
        &format!("world:\n  deps:\n    enabled: {enabled_yaml}\n"),
    );
}

fn start_host_world_socket() -> (tempfile::TempDir, PathBuf, AgentSocket) {
    // Keep the Unix socket path short to avoid `SUN_LEN` failures.
    let sock_tmp = Builder::new()
        .prefix("substrate-wdp4-")
        .tempdir_in("/tmp")
        .expect("socket tempdir");
    let socket_path = sock_tmp.path().join("world.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );
    (sock_tmp, socket_path, socket)
}

fn guest_bin_dir(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join("world-deps-bin")
}

fn base_path_for_host_exec(guest_bin: &Path) -> String {
    let base = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
    format!("{}:{base}", guest_bin.display())
}

fn script_package_yaml(name: &str, entrypoints: &[&str], probe_command: Option<&str>) -> String {
    let mut buf = String::new();
    buf.push_str("version: 1\n");
    buf.push_str(&format!("name: {name}\n"));
    buf.push_str(&format!("description: {name} via script\n"));
    buf.push_str("runnable: true\n");
    buf.push_str("entrypoints:\n");
    for ep in entrypoints {
        buf.push_str(&format!("  - {ep}\n"));
    }
    buf.push_str("install:\n");
    buf.push_str("  method: script\n");
    buf.push_str(&format!("  script_path: ../scripts/{name}.sh\n"));
    if let Some(cmd) = probe_command {
        buf.push_str("probe:\n");
        buf.push_str(&format!("  command: {cmd:?}\n"));
    }
    buf
}

#[test]
fn test_current_install_script_creates_entrypoint_and_probe_command_runs_under_sh() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_global_config_builtins_disabled(&fixture, &["hello"]);
    write_workspace_config(&ws_root, &[]);

    write_file(
        &global_deps_dir(&fixture).join("packages/hello.yaml"),
        &script_package_yaml("hello", &["hello"], Some("hello")),
    );
    write_file(
        &global_deps_dir(&fixture).join("scripts/hello.sh"),
        r#"#!/bin/sh
set -eu

bin="${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR}"
mkdir -p "$bin"
cat >"$bin/hello" <<'EOF'
#!/bin/sh
exit 0
EOF
chmod +x "$bin/hello"
"#,
    );
    chmod_x(&global_deps_dir(&fixture).join("scripts/hello.sh"));

    let (_sock_tmp, socket_path, _socket) = start_host_world_socket();

    let guest_bin = guest_bin_dir(&fixture);
    fs::create_dir_all(&guest_bin).expect("create guest bin");
    let path_for_world = base_path_for_host_exec(&guest_bin);

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("PATH", &path_for_world)
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &guest_bin)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "install", "hello"])
        .assert()
        .success();

    assert!(
        guest_bin.join("hello").exists(),
        "expected entrypoint installed under guest bin dir"
    );
}

#[test]
fn test_current_sync_script_only_installs_entrypoints_detectable_via_probe_or_entrypoints() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_global_config_builtins_disabled(&fixture, &["hello", "ep_only"]);
    write_workspace_config(&ws_root, &[]);

    write_file(
        &global_deps_dir(&fixture).join("packages/hello.yaml"),
        &script_package_yaml("hello", &["hello"], Some("hello")),
    );
    write_file(
        &global_deps_dir(&fixture).join("scripts/hello.sh"),
        r#"#!/bin/sh
set -eu

bin="${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR}"
mkdir -p "$bin"
cat >"$bin/hello" <<'EOF'
#!/bin/sh
exit 0
EOF
chmod +x "$bin/hello"
"#,
    );
    chmod_x(&global_deps_dir(&fixture).join("scripts/hello.sh"));

    // No probe.command; "applied" must fall back to checking entrypoints[] via `command -v`.
    write_file(
        &global_deps_dir(&fixture).join("packages/ep_only.yaml"),
        &script_package_yaml("ep_only", &["ep_only"], None),
    );
    write_file(
        &global_deps_dir(&fixture).join("scripts/ep_only.sh"),
        r#"#!/bin/sh
set -eu

bin="${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR}"
mkdir -p "$bin"
cat >"$bin/ep_only" <<'EOF'
#!/bin/sh
exit 0
EOF
chmod +x "$bin/ep_only"
"#,
    );
    chmod_x(&global_deps_dir(&fixture).join("scripts/ep_only.sh"));

    let (_sock_tmp, socket_path, _socket) = start_host_world_socket();

    let guest_bin = guest_bin_dir(&fixture);
    fs::create_dir_all(&guest_bin).expect("create guest bin");
    let path_for_world = base_path_for_host_exec(&guest_bin);

    substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("PATH", &path_for_world)
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &guest_bin)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "sync"])
        .assert()
        .success();

    assert!(
        guest_bin.join("hello").exists(),
        "expected hello installed under guest bin dir"
    );
    assert!(
        guest_bin.join("ep_only").exists(),
        "expected ep_only installed under guest bin dir"
    );

    let out = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("PATH", &path_for_world)
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &guest_bin)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "list", "applied", "--json"])
        .output()
        .expect("run current list applied");

    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "expected exit 0, stdout={}, stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout should be valid JSON");
    let items = json
        .get("items")
        .and_then(|v| v.as_array())
        .expect("output.items must be an array");
    let mut world_by_name: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .expect("item.name must be string")
            .to_string();
        let world = item
            .get("world")
            .and_then(|v| v.as_str())
            .expect("item.world must be string")
            .to_string();
        world_by_name.insert(name, world);
    }

    assert_eq!(
        world_by_name.get("hello").map(String::as_str),
        Some("present"),
        "expected hello world=present, got map={world_by_name:?}"
    );
    assert_eq!(
        world_by_name.get("ep_only").map(String::as_str),
        Some("present"),
        "expected ep_only world=present, got map={world_by_name:?}"
    );
}

#[test]
fn test_current_sync_generates_sh_env_exec_wrapper_deterministic_and_invokable() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    write_global_config_builtins_disabled(&fixture, &["wrapped"]);
    write_workspace_config(&ws_root, &[]);

    write_file(
        &global_deps_dir(&fixture).join("packages/wrapped.yaml"),
        concat!(
            "version: 1\n",
            "name: wrapped\n",
            "description: wrapped via script\n",
            "runnable: true\n",
            "entrypoints:\n",
            "  - wrapped\n",
            "wrappers:\n",
            "  - name: wrapped\n",
            "    kind: sh_env_exec\n",
            "    exec: wrapped_real\n",
            "    env:\n",
            "      FOO: bar\n",
            "install:\n",
            "  method: script\n",
            "  script_path: ../scripts/wrapped.sh\n",
            "probe:\n",
            "  command: \"wrapped\"\n",
        ),
    );
    write_file(
        &global_deps_dir(&fixture).join("scripts/wrapped.sh"),
        r#"#!/bin/sh
set -eu

bin="${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR}"
mkdir -p "$bin"
cat >"$bin/wrapped_real" <<'EOF'
#!/bin/sh
if [ "${FOO:-}" != "bar" ]; then
  echo "missing FOO=bar" >&2
  exit 42
fi
exit 0
EOF
chmod +x "$bin/wrapped_real"
"#,
    );
    chmod_x(&global_deps_dir(&fixture).join("scripts/wrapped.sh"));

    let (_sock_tmp, socket_path, _socket) = start_host_world_socket();

    let guest_bin = guest_bin_dir(&fixture);
    fs::create_dir_all(&guest_bin).expect("create guest bin");
    let path_for_world = base_path_for_host_exec(&guest_bin);

    let run_sync = || {
        substrate_command_for_home(&fixture)
            .current_dir(&ws_root)
            .env("PATH", &path_for_world)
            .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &guest_bin)
            .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
            .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
            .args(["world", "deps", "current", "sync"])
            .output()
            .expect("run current sync")
    };

    let out1 = run_sync();
    assert!(
        out1.status.success(),
        "expected sync exit 0, stdout={}, stderr={}",
        String::from_utf8_lossy(&out1.stdout),
        String::from_utf8_lossy(&out1.stderr)
    );

    let wrapper_path = guest_bin.join("wrapped");
    assert!(
        wrapper_path.exists(),
        "expected wrapper generated at {wrapper_path:?}"
    );
    let wrapper_1 = fs::read_to_string(&wrapper_path).expect("read wrapper");
    assert!(
        wrapper_1.contains("export FOO=bar") || wrapper_1.contains("FOO=bar"),
        "expected wrapper to set env var, wrapper={wrapper_1:?}"
    );
    assert!(
        wrapper_1.contains("exec wrapped_real"),
        "expected wrapper to exec wrapped_real, wrapper={wrapper_1:?}"
    );

    let out_run = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("PATH", &path_for_world)
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &guest_bin)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .arg("-c")
        .arg("wrapped")
        .output()
        .expect("run wrapped via substrate -c");
    assert!(
        out_run.status.success(),
        "expected wrapped to succeed, stdout={}, stderr={}",
        String::from_utf8_lossy(&out_run.stdout),
        String::from_utf8_lossy(&out_run.stderr)
    );

    let out2 = run_sync();
    assert!(
        out2.status.success(),
        "expected sync exit 0 (second run), stdout={}, stderr={}",
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr)
    );
    let wrapper_2 = fs::read_to_string(&wrapper_path).expect("read wrapper (second run)");
    assert_eq!(
        wrapper_1, wrapper_2,
        "expected deterministic wrapper contents across sync runs"
    );
}

#[test]
fn test_generated_bash_source_exec_wrapper_missing_source_prints_actionable_stderr() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(&ws_root).expect("create ws root");

    let guest_bin = guest_bin_dir(&fixture);
    fs::create_dir_all(&guest_bin).expect("create guest bin");
    let missing_source = guest_bin.join("missing.sh");
    let missing_source_str = missing_source.display().to_string();

    write_global_config_builtins_disabled(&fixture, &["badbash"]);
    write_workspace_config(&ws_root, &[]);

    write_file(
        &global_deps_dir(&fixture).join("packages/badbash.yaml"),
        format!(
            concat!(
                "version: 1\n",
                "name: badbash\n",
                "description: badbash wrapper failure\n",
                "runnable: true\n",
                "entrypoints:\n",
                "  - badbash\n",
                "wrappers:\n",
                "  - name: badbash\n",
                "    kind: bash_source_exec\n",
                "    bash_source: {bash_source:?}\n",
                "    exec: badbash_real\n",
                "install:\n",
                "  method: script\n",
                "  script_path: ../scripts/badbash.sh\n",
                "probe:\n",
                "  command: \"badbash\"\n",
            ),
            bash_source = missing_source_str
        )
        .as_str(),
    );
    write_file(
        &global_deps_dir(&fixture).join("scripts/badbash.sh"),
        r#"#!/bin/sh
set -eu

bin="${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR}"
mkdir -p "$bin"
cat >"$bin/badbash_real" <<'EOF'
#!/bin/sh
exit 0
EOF
chmod +x "$bin/badbash_real"
"#,
    );
    chmod_x(&global_deps_dir(&fixture).join("scripts/badbash.sh"));

    let (_sock_tmp, socket_path, _socket) = start_host_world_socket();
    let path_for_world = base_path_for_host_exec(&guest_bin);

    let out_sync = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("PATH", &path_for_world)
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &guest_bin)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .args(["world", "deps", "current", "sync"])
        .output()
        .expect("run current sync");
    assert!(
        out_sync.status.success(),
        "expected sync exit 0, stdout={}, stderr={}",
        String::from_utf8_lossy(&out_sync.stdout),
        String::from_utf8_lossy(&out_sync.stderr)
    );

    let out_run = substrate_command_for_home(&fixture)
        .current_dir(&ws_root)
        .env("PATH", &path_for_world)
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &guest_bin)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .arg("-c")
        .arg("badbash")
        .output()
        .expect("run badbash via substrate -c");

    assert!(
        !out_run.status.success(),
        "expected badbash to fail, stdout={}, stderr={}",
        String::from_utf8_lossy(&out_run.stdout),
        String::from_utf8_lossy(&out_run.stderr)
    );

    let stderr = String::from_utf8_lossy(&out_run.stderr);
    assert!(
        stderr.contains("bash_source_exec"),
        "expected stderr to mention wrapper kind, stderr={stderr:?}"
    );
    assert!(
        stderr.contains(&missing_source_str),
        "expected stderr to include bash_source path, stderr={stderr:?}"
    );
    assert!(
        stderr.to_lowercase().contains("bash"),
        "expected stderr to mention bash availability, stderr={stderr:?}"
    );
    assert!(
        stderr.contains("current show badbash") && stderr.contains("--explain"),
        "expected stderr to include an actionable next step, stderr={stderr:?}"
    );
}
