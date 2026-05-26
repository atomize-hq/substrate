mod support;

use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::path::Path;

#[cfg(unix)]
use std::sync::{Arc, Mutex};

#[cfg(unix)]
use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};

#[cfg(unix)]
use tempfile::Builder;

#[cfg(unix)]
fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn read_repo_file(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative))
        .unwrap_or_else(|err| panic!("read {relative}: {err}"))
}

#[test]
fn wdap1_required_docs_reference_the_contract_and_provisioning_workflow() {
    let reference_readme = read_repo_file("docs/reference/world/deps/README.md");
    assert!(
        reference_readme.contains("## System-package runtime fail-early"),
        "expected docs/reference/world/deps/README.md to include the runtime heading"
    );
    assert!(
        reference_readme.contains("## Commands you will use"),
        "expected docs/reference/world/deps/README.md to include the commands heading"
    );
    assert!(
        reference_readme.contains(
            "docs/reference/world/deps/provisioning.md"
        ),
        "expected docs/reference/world/deps/README.md to link to the stable provisioning contract"
    );
    assert!(
        reference_readme.contains("substrate world enable --provision-deps"),
        "expected docs/reference/world/deps/README.md to mention the operator remediation"
    );

    let internals = read_repo_file("docs/internals/world/deps.md");
    assert!(
        internals.contains("## High-level flow"),
        "expected docs/internals/world/deps.md to include the high-level flow heading"
    );
    assert!(
        internals.contains("## System-package runtime fail-early"),
        "expected docs/internals/world/deps.md to include the runtime fail-early heading"
    );
    assert!(
        internals.contains(
            "docs/reference/world/deps/provisioning.md"
        ),
        "expected docs/internals/world/deps.md to link to the stable provisioning contract"
    );
    assert!(
        internals.contains("substrate world enable --provision-deps"),
        "expected docs/internals/world/deps.md to mention the provisioning workflow"
    );

    let world_doc = read_repo_file("docs/WORLD.md");
    assert!(
        world_doc.contains("## 5) Agent API (over UDS)"),
        "expected docs/WORLD.md to include the Agent API heading"
    );
    assert!(
        world_doc.contains("profile"),
        "expected docs/WORLD.md to mention the request profile field"
    );
    assert!(
        world_doc.contains(
            "docs/reference/world/deps/provisioning.md"
        ),
        "expected docs/WORLD.md to link to the stable provisioning contract"
    );

    let configuration = read_repo_file("docs/CONFIGURATION.md");
    assert!(
        configuration.contains("SUBSTRATE_WORLD_REQUEST_PROFILE"),
        "expected docs/CONFIGURATION.md to include SUBSTRATE_WORLD_REQUEST_PROFILE"
    );
    assert!(
        configuration.contains(
            "docs/reference/world/deps/provisioning.md"
        ),
        "expected docs/CONFIGURATION.md to link to the stable provisioning contract"
    );
    assert!(
        configuration.contains("substrate world enable --provision-deps"),
        "expected docs/CONFIGURATION.md to mention the operator workflow"
    );

    let commands = read_repo_file("docs/COMMANDS.md");
    assert!(
        commands.contains("### world Subcommand"),
        "expected docs/COMMANDS.md to include the world subcommand section"
    );
    assert!(
        commands.contains("--provision-deps"),
        "expected docs/COMMANDS.md to document the --provision-deps flag"
    );
    assert!(
        commands.contains(
            "docs/reference/world/deps/provisioning.md"
        ),
        "expected docs/COMMANDS.md to link to the stable provisioning contract"
    );

    let wdap1_contract = read_repo_file("docs/reference/world/deps/provisioning.md");
    assert!(
        wdap1_contract.contains("unsupported on Windows"),
        "expected the stable provisioning contract to preserve the Windows runtime guidance"
    );
    assert!(
        wdap1_contract.contains("Substrate will not mutate the host OS"),
        "expected the stable provisioning contract to preserve the Linux host-native runtime guidance"
    );
}

#[cfg(unix)]
fn normalize_output(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).replace("\r\n", "\n")
}

#[cfg(unix)]
fn workspace_root(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join("ws")
}

#[cfg(unix)]
fn global_config_path(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/config.yaml")
}

#[cfg(unix)]
fn workspace_config_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".substrate/workspace.yaml")
}

#[cfg(unix)]
fn global_deps_dir(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join(".substrate/deps")
}

#[cfg(unix)]
fn fake_world_bin_dir(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join("fake-world-bin")
}

#[cfg(unix)]
fn write_global_config_builtins_disabled(fixture: &ShellEnvFixture, enabled_yaml: &str) {
    write_file(
        &global_config_path(fixture),
        &format!("world:\n  deps:\n    builtins: disabled\n    enabled: {enabled_yaml}\n"),
    );
}

#[cfg(unix)]
fn write_workspace_config(workspace_root: &Path, enabled_yaml: &str) {
    write_file(
        &workspace_config_path(workspace_root),
        &format!("world:\n  deps:\n    enabled: {enabled_yaml}\n"),
    );
}

#[cfg(unix)]
fn write_apt_package(fixture: &ShellEnvFixture, name: &str, apt_entries: &[(&str, Option<&str>)]) {
    let mut body = format!(
        "version: 1\nname: {name}\ndescription: {name} via apt\nrunnable: true\nentrypoints: [\"{name}\"]\ninstall:\n  method: apt\n  apt:\n"
    );
    for (pkg_name, version) in apt_entries {
        body.push_str(&format!("    - name: {pkg_name}\n"));
        if let Some(version) = version {
            body.push_str(&format!("      version: {version}\n"));
        }
    }
    body.push_str("probe:\n  command: \"true\"\n");
    write_file(
        &global_deps_dir(fixture).join(format!("packages/{name}.yaml")),
        &body,
    );
}

#[cfg(unix)]
fn write_script_package(fixture: &ShellEnvFixture, name: &str, token: &str) {
    write_file(
        &global_deps_dir(fixture).join(format!("packages/{name}.yaml")),
        &format!(
            r#"version: 1
name: {name}
description: {name} via script
runnable: true
entrypoints: ["{name}"]
install:
  method: script
  script_path: ../scripts/{name}.sh
probe:
  command: "command -v {name} >/dev/null 2>&1"
"#,
        ),
    );
    write_file(
        &global_deps_dir(fixture).join(format!("scripts/{name}.sh")),
        &format!(
            r#"#!/bin/sh
set -eu
echo "{token}"
mkdir -p "${{SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing world deps bin}}"
cat > "${{SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR}}/{name}" <<'EOF'
#!/bin/sh
exit 0
EOF
chmod 0755 "${{SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR}}/{name}"
"#,
        ),
    );
}

#[cfg(unix)]
fn install_fake_world_tools(fixture: &ShellEnvFixture) {
    let bin_dir = fake_world_bin_dir(fixture);
    fs::create_dir_all(&bin_dir).expect("create fake world bin");

    write_file(
        &bin_dir.join("dpkg-query"),
        r#"#!/bin/sh
set -eu
exit 1
"#,
    );
    write_file(
        &bin_dir.join("apt-get"),
        r#"#!/bin/sh
set -eu
exit 0
"#,
    );
    write_file(
        &bin_dir.join("sudo"),
        r#"#!/bin/sh
set -eu
if [ "${1:-}" = "-n" ]; then
  shift
fi
exec "$@"
"#,
    );
    write_file(
        &bin_dir.join("dpkg"),
        r#"#!/bin/sh
set -eu
exit 0
"#,
    );
    write_file(
        &bin_dir.join("mkdir"),
        r#"#!/bin/sh
set -eu
exit 0
"#,
    );
    write_file(
        &bin_dir.join("mktemp"),
        r#"#!/bin/sh
set -eu
template="${1:-${TMPDIR:-/tmp}/substrate-fake-mktemp.XXXXXX}"
template="${template#\"}"
template="${template%\"}"
dir="$(dirname "$template")"
path="$dir/.substrate-fake-mktemp"
: > "$path"
printf '%s\n' "$path"
"#,
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for name in ["dpkg-query", "apt-get", "sudo", "dpkg", "mkdir", "mktemp"] {
            let path = bin_dir.join(name);
            let mut perms = fs::metadata(&path).expect("metadata").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).expect("chmod");
        }
    }
}

#[cfg(unix)]
fn install_fake_pacman_probe_tools(fixture: &ShellEnvFixture) {
    let bin_dir = fake_world_bin_dir(fixture);
    fs::create_dir_all(&bin_dir).expect("create fake world bin");
    write_file(
        &bin_dir.join("pacman"),
        r#"#!/bin/sh
set -eu
exit 1
"#,
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = bin_dir.join("pacman");
        let mut perms = fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("chmod");
    }
}

#[cfg(unix)]
fn write_pacman_package(fixture: &ShellEnvFixture, name: &str, packages: &[&str]) {
    let mut body = format!(
        "version: 1\nname: {name}\ndescription: {name} via pacman\nrunnable: false\ninstall:\n  method: pacman\n  pacman:\n"
    );
    for pkg in packages {
        body.push_str(&format!("    - {pkg}\n"));
    }
    write_file(
        &global_deps_dir(fixture).join(format!("packages/{name}.yaml")),
        &body,
    );
}

#[cfg(unix)]
fn start_world_socket_host_execute_record(
    prefix: &str,
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
        SocketResponse::CapabilitiesAndHostExecuteRecord {
            scopes: vec![],
            records: records.clone(),
        },
    );

    (sock_tmp, socket_path, socket, records)
}

#[cfg(unix)]
fn recorded_cmds(records: &Arc<Mutex<Vec<serde_json::Value>>>) -> Vec<String> {
    records
        .lock()
        .expect("lock records")
        .iter()
        .filter_map(|value| value.get("cmd")?.as_str().map(|s| s.to_string()))
        .collect()
}

#[cfg(unix)]
fn assert_lines_in_order(haystack: &str, expected: &[&str]) {
    let mut offset = 0usize;
    for needle in expected {
        let next = haystack[offset..].find(needle).unwrap_or_else(|| {
            panic!(
                "expected output to contain {needle:?} after offset {offset}, output={haystack:?}"
            );
        });
        offset += next + needle.len();
    }
}

#[cfg(unix)]
fn assert_no_runtime_apt_mutation(cmds: &[String]) {
    assert!(
        cmds.iter().all(|cmd| {
            !cmd.contains("apt-get")
                && !cmd.contains("dpkg -i")
                && !cmd.contains("dpkg --install")
                && !cmd.contains("dpkg --unpack")
        }),
        "expected no runtime apt/dpkg mutation requests; cmds={cmds:?}"
    );
}

#[cfg(unix)]
fn world_deps_command(
    fixture: &ShellEnvFixture,
    ws_root: &Path,
    socket_path: &Path,
) -> assert_cmd::Command {
    let fake_world_bin = fake_world_bin_dir(fixture);
    let mut cmd = substrate_command_for_home(fixture);
    cmd.current_dir(ws_root)
        .env("SUBSTRATE_WORLD_SOCKET", socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &fake_world_bin);
    cmd
}

#[cfg(unix)]
#[test]
fn current_install_fails_early_for_unsatisfied_apt_and_never_mutates_or_runs_scripts() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    write_apt_package(&fixture, "node", &[("nodejs", None)]);
    write_script_package(&fixture, "hello", "SCRIPT_TOKEN_WDAP1_FAIL_EARLY");
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_host_execute_record("substrate-wdap1-install-fail-early-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args(["world", "deps", "current", "install", "node", "hello"])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(4),
        "expected exit 4 for unsatisfied apt requirements\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stderr.contains("substrate world enable --provision-deps"),
        "expected remediation command in stderr: {stderr}"
    );

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter().any(|cmd| cmd.contains("dpkg-query")),
        "expected a read-only dpkg-query probe request; cmds={cmds:?}"
    );
    assert_no_runtime_apt_mutation(&cmds);
    assert!(
        cmds.iter()
            .all(|cmd| !cmd.contains("SCRIPT_TOKEN_WDAP1_FAIL_EARLY")),
        "expected fail-early to suppress script installs; cmds={cmds:?}"
    );
}

#[cfg(unix)]
#[test]
fn current_install_scopes_apt_checks_to_explicit_items_only() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    write_apt_package(&fixture, "node", &[("nodejs", None)]);
    write_script_package(&fixture, "hello", "SCRIPT_TOKEN_WDAP1_EXPLICIT_ONLY");
    write_global_config_builtins_disabled(&fixture, "[\"node\"]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_host_execute_record("substrate-wdap1-install-explicit-only-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args(["world", "deps", "current", "install", "hello"])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert!(
        output.status.success(),
        "expected explicit non-APT install to succeed\nstdout={stdout}\nstderr={stderr}"
    );

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter()
            .all(|cmd| !cmd.contains("dpkg-query") && !cmd.contains("apt-get")),
        "expected no APT probe/install for explicit non-APT items; cmds={cmds:?}"
    );
    assert!(
        cmds.iter()
            .any(|cmd| cmd.contains("SCRIPT_TOKEN_WDAP1_EXPLICIT_ONLY")),
        "expected the explicit script package to run; cmds={cmds:?}"
    );
}

#[cfg(unix)]
#[test]
fn current_install_scopes_pacman_checks_to_explicit_items_only() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    install_fake_pacman_probe_tools(&fixture);
    write_pacman_package(&fixture, "pacman-tool", &["curl"]);
    write_script_package(&fixture, "hello", "SCRIPT_TOKEN_WDAP1_PACMAN_EXPLICIT_ONLY");
    write_global_config_builtins_disabled(&fixture, "[\"pacman-tool\"]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_host_execute_record("substrate-wdap1-pacman-explicit-only-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args(["world", "deps", "current", "install", "hello"])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert!(
        output.status.success(),
        "expected explicit non-pacman install to succeed\nstdout={stdout}\nstderr={stderr}"
    );

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter().all(|cmd| !cmd.contains("pacman -Q")),
        "expected no pacman probe for explicit non-pacman items; cmds={cmds:?}"
    );
    assert!(
        cmds.iter()
            .any(|cmd| cmd.contains("SCRIPT_TOKEN_WDAP1_PACMAN_EXPLICIT_ONLY")),
        "expected the explicit script package to run; cmds={cmds:?}"
    );
}

#[cfg(unix)]
#[test]
fn current_sync_fails_early_for_unsatisfied_pacman_requirements() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    install_fake_pacman_probe_tools(&fixture);
    write_pacman_package(&fixture, "pacman-tool", &["curl"]);
    write_global_config_builtins_disabled(&fixture, "[\"pacman-tool\"]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_host_execute_record("substrate-wdap1-pacman-fail-early-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args(["world", "deps", "current", "sync"])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(4),
        "expected sync to fail early on unsatisfied pacman requirements\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stderr.contains("substrate world enable --provision-deps"),
        "expected remediation command in stderr: {stderr}"
    );
    assert!(
        stderr.contains("pacman"),
        "expected pacman-specific remediation text in stderr: {stderr}"
    );

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter().any(|cmd| cmd.contains("pacman -Q")),
        "expected a read-only pacman probe request; cmds={cmds:?}"
    );
    assert_no_runtime_apt_mutation(&cmds);
    assert!(
        cmds.iter()
            .all(|cmd| !cmd.contains("SCRIPT_TOKEN_WDAP1_PACMAN_FAIL_EARLY")),
        "expected fail-early to suppress script installs; cmds={cmds:?}"
    );
}

#[cfg(unix)]
#[test]
fn current_sync_all_applies_fail_early_to_visible_apt_items() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    write_apt_package(&fixture, "node", &[("nodejs", None)]);
    write_script_package(&fixture, "hello", "SCRIPT_TOKEN_WDAP1_SYNC_ALL");
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[\"hello\"]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_host_execute_record("substrate-wdap1-sync-all-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args(["world", "deps", "current", "sync", "--all"])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(4),
        "expected sync --all to fail early on visible APT items\nstdout={stdout}\nstderr={stderr}"
    );

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter().any(|cmd| cmd.contains("dpkg-query")),
        "expected a read-only dpkg-query probe request; cmds={cmds:?}"
    );
    assert_no_runtime_apt_mutation(&cmds);
    assert!(
        cmds.iter()
            .all(|cmd| !cmd.contains("SCRIPT_TOKEN_WDAP1_SYNC_ALL")),
        "expected fail-early to suppress non-APT installs during sync --all; cmds={cmds:?}"
    );
}

#[cfg(unix)]
#[test]
fn current_sync_dry_run_still_fails_early_and_prints_normalized_apt_requirements() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    write_apt_package(&fixture, "curl-unpinned", &[("curl", None)]);
    write_apt_package(&fixture, "nodejs-pinned", &[("nodejs", Some("20.0.0-1"))]);
    write_apt_package(&fixture, "zlib", &[("zlib1g", None)]);
    write_script_package(&fixture, "hello", "SCRIPT_TOKEN_WDAP1_DRY_RUN");
    write_global_config_builtins_disabled(
        &fixture,
        "[\"curl-unpinned\", \"hello\", \"nodejs-pinned\", \"zlib\"]",
    );
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_host_execute_record("substrate-wdap1-dry-run-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args(["world", "deps", "current", "sync", "--dry-run"])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(4),
        "expected dry-run to fail early when APT requirements are unsatisfied\nstdout={stdout}\nstderr={stderr}"
    );
    assert_lines_in_order(&stdout, &["curl", "nodejs=20.0.0-1", "zlib1g"]);

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter().any(|cmd| cmd.contains("dpkg-query")),
        "expected a read-only dpkg-query probe during dry-run; cmds={cmds:?}"
    );
    assert_no_runtime_apt_mutation(&cmds);
    assert!(
        cmds.iter()
            .all(|cmd| !cmd.contains("SCRIPT_TOKEN_WDAP1_DRY_RUN")),
        "expected dry-run fail-early to avoid script installs; cmds={cmds:?}"
    );
}

#[cfg(unix)]
#[test]
fn current_install_verbose_fail_early_prints_normalized_requirements_to_stderr() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    write_apt_package(&fixture, "curl-unpinned", &[("curl", None)]);
    write_apt_package(&fixture, "nodejs-pinned", &[("nodejs", Some("20.0.0-1"))]);
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_host_execute_record("substrate-wdap1-verbose-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args([
            "world",
            "deps",
            "current",
            "install",
            "curl-unpinned",
            "nodejs-pinned",
            "--verbose",
        ])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(4),
        "expected verbose install to fail early on unsatisfied APT requirements\nstdout={stdout}\nstderr={stderr}"
    );
    assert_lines_in_order(&stderr, &["curl", "nodejs=20.0.0-1"]);

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.iter().any(|cmd| cmd.contains("dpkg-query")),
        "expected a read-only dpkg-query probe request; cmds={cmds:?}"
    );
    assert_no_runtime_apt_mutation(&cmds);
}

#[cfg(target_os = "linux")]
#[test]
fn current_install_linux_remediation_mentions_host_os_non_mutation() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    write_apt_package(&fixture, "node", &[("nodejs", None)]);
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, _records) =
        start_world_socket_host_execute_record("substrate-wdap1-linux-guidance-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args(["world", "deps", "current", "install", "node"])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(4),
        "expected Linux runtime remediation to exit 4\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stderr.contains("Substrate will not mutate the host OS"),
        "expected Linux runtime remediation guidance in stderr: {stderr}"
    );
}

#[cfg(unix)]
#[test]
fn current_install_conflicting_apt_pins_exits_2_before_world_execution() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    install_fake_world_tools(&fixture);
    write_apt_package(&fixture, "nodejs-20", &[("nodejs", Some("20.0.0-1"))]);
    write_apt_package(&fixture, "nodejs-18", &[("nodejs", Some("18.0.0-1"))]);
    write_script_package(&fixture, "hello", "SCRIPT_TOKEN_WDAP1_CONFLICT");
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_host_execute_record("substrate-wdap1-conflict-");

    let output = world_deps_command(&fixture, &ws_root, &socket_path)
        .args([
            "world",
            "deps",
            "current",
            "install",
            "nodejs-20",
            "nodejs-18",
            "hello",
        ])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected conflicting APT pins to exit 2\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stderr.contains("nodejs") && stderr.contains("20.0.0-1") && stderr.contains("18.0.0-1"),
        "expected conflict details in stderr: {stderr}"
    );

    let cmds = recorded_cmds(&records);
    assert!(
        cmds.is_empty(),
        "expected conflicts to fail before any world-service execution; cmds={cmds:?}"
    );
}

#[cfg(unix)]
#[test]
fn current_install_backend_unavailable_for_probe_exits_3_with_actionable_stderr() {
    let fixture = ShellEnvFixture::new();
    let ws_root = workspace_root(&fixture);
    fs::create_dir_all(ws_root.join(".substrate")).expect("create workspace .substrate");

    write_apt_package(&fixture, "node", &[("nodejs", None)]);
    write_global_config_builtins_disabled(&fixture, "[]");
    write_workspace_config(&ws_root, "[]");

    let missing_socket = ws_root.join("missing-world.sock");
    let output = world_deps_command(&fixture, &ws_root, &missing_socket)
        .args(["world", "deps", "current", "install", "node"])
        .output()
        .expect("run substrate");
    let stdout = normalize_output(&output.stdout);
    let stderr = normalize_output(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(3),
        "expected missing world backend connectivity to exit 3\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stderr.contains("substrate world doctor --json"),
        "expected actionable world doctor guidance in stderr: {stderr}"
    );
}
