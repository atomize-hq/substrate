#![cfg(target_os = "macos")]

mod support;

use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use support::common::shared_tmpdir;
use support::{substrate_shell_driver, temp_dir};
use support::{AgentSocket, SocketResponse};
use tempfile::{Builder, TempDir};

const APT_PROBE_STDOUT: &str = "__SUBSTRATE_WDAP0__ os_release_readable=1\n__SUBSTRATE_WDAP0__ id=ubuntu\n__SUBSTRATE_WDAP0__ id_like=debian\n__SUBSTRATE_WDAP0__ pacman_present=0\n";
const PACMAN_PROBE_STDOUT: &str = "__SUBSTRATE_WDAP0__ os_release_readable=1\n__SUBSTRATE_WDAP0__ id=arch\n__SUBSTRATE_WDAP0__ id_like=archlinux\n__SUBSTRATE_WDAP0__ pacman_present=1\n";

const HELPER_SCRIPT: &str = r#"#!/usr/bin/env bash
set -euo pipefail

log="${SUBSTRATE_TEST_WORLD_LOG:?missing log path}"
mkdir -p "$(dirname "$log")"

echo "world-enable invoked: $*" >>"$log"

if [[ -n "${SUBSTRATE_TEST_WORLD_STDOUT:-}" ]]; then
  echo "${SUBSTRATE_TEST_WORLD_STDOUT}"
fi

if [[ -n "${SUBSTRATE_TEST_WORLD_STDERR:-}" ]]; then
  echo "${SUBSTRATE_TEST_WORLD_STDERR}" >&2
fi

exit_code="${SUBSTRATE_TEST_WORLD_EXIT:-0}"
if [[ "${SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET:-0}" != "1" ]]; then
python3 <<'PY'
import os
import socket

socket_path = os.environ.get("SUBSTRATE_WORLD_SOCKET")
if not socket_path:
    raise SystemExit("SUBSTRATE_WORLD_SOCKET unset")
os.makedirs(os.path.dirname(socket_path), exist_ok=True)
try:
    os.unlink(socket_path)
except FileNotFoundError:
    pass
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.bind(socket_path)
sock.listen(1)
sock.close()
PY
fi

exit "$exit_code"
"#;

struct WorldEnableProvisionFixture {
    _temp: TempDir,
    _socket_temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
    script_path: PathBuf,
    log_path: PathBuf,
    socket_path: PathBuf,
}

impl WorldEnableProvisionFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-world-enable-provision-");
        let home = temp.path().join("home");
        let substrate_home = temp.path().join("substrate-home");
        let workspace_root = temp.path().join("ws");
        let script_path = temp.path().join("scripts/world-enable.sh");
        let log_path = temp.path().join("logs/world-enable.log");
        let socket_temp = Builder::new()
            .prefix("substrate-world-enable-provision-sock-")
            .tempdir_in("/tmp")
            .expect("failed to create socket tempdir");
        let socket_path = socket_temp.path().join("world.sock");

        fs::create_dir_all(&home).expect("create home");
        fs::create_dir_all(&substrate_home).expect("create substrate home");
        fs::create_dir_all(workspace_root.join(".substrate")).expect("create workspace config dir");
        fs::create_dir_all(script_path.parent().expect("script parent"))
            .expect("create script dir");
        fs::create_dir_all(log_path.parent().expect("log parent")).expect("create log dir");

        let fixture = Self {
            _temp: temp,
            _socket_temp: socket_temp,
            home,
            substrate_home,
            workspace_root,
            script_path,
            log_path,
            socket_path,
        };
        fixture.install_helper_script();
        fixture
    }

    fn install_helper_script(&self) {
        fs::write(&self.script_path, HELPER_SCRIPT).expect("write helper script");
        let mut perms = fs::metadata(&self.script_path)
            .expect("helper metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&self.script_path, perms).expect("chmod helper");
    }

    fn command(&self) -> assert_cmd::Command {
        let mut cmd = substrate_shell_driver();
        cmd.arg("world")
            .arg("enable")
            .arg("--home")
            .arg(&self.substrate_home)
            .current_dir(&self.workspace_root)
            .env("TMPDIR", shared_tmpdir())
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env("SUBSTRATE_WORLD_ENABLE_SCRIPT", &self.script_path)
            .env("SUBSTRATE_TEST_WORLD_LOG", &self.log_path)
            .env("SUBSTRATE_WORLD_SOCKET", &self.socket_path)
            .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
            .env("SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR", "1");
        cmd
    }

    fn global_config_path(&self) -> PathBuf {
        self.substrate_home.join("config.yaml")
    }

    fn workspace_config_path(&self) -> PathBuf {
        self.workspace_root.join(".substrate/workspace.yaml")
    }

    fn global_deps_dir(&self) -> PathBuf {
        self.substrate_home.join("deps")
    }

    fn log_contents(&self) -> Option<String> {
        fs::read_to_string(&self.log_path).ok()
    }

    fn write_global_config(&self, enabled: &[&str]) {
        write_file(
            &self.global_config_path(),
            &format!(
                "world:\n  deps:\n    builtins: disabled\n    enabled: {}\n",
                yaml_list(enabled)
            ),
        );
    }

    fn write_workspace_config(&self, enabled: &[&str]) {
        write_file(
            &self.workspace_config_path(),
            &format!("world:\n  deps:\n    enabled: {}\n", yaml_list(enabled)),
        );
    }

    fn write_apt_package(&self, name: &str, apt_entries: &[(&str, Option<&str>)]) {
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
            &self.global_deps_dir().join(format!("packages/{name}.yaml")),
            &body,
        );
    }

    fn write_pacman_package(&self, name: &str, pacman_entries: &[&str]) {
        let mut body = format!(
            "version: 1\nname: {name}\ndescription: {name} via pacman\nrunnable: false\ninstall:\n  method: pacman\n  pacman:\n"
        );
        for pkg_name in pacman_entries {
            body.push_str(&format!("    - {pkg_name}\n"));
        }
        write_file(
            &self.global_deps_dir().join(format!("packages/{name}.yaml")),
            &body,
        );
    }
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn yaml_list(items: &[&str]) -> String {
    let mut out = String::from("[");
    for (idx, item) in items.iter().enumerate() {
        if idx != 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(item);
        out.push('"');
    }
    out.push(']');
    out
}

fn assert_lines_in_order(haystack: &str, expected: &[&str]) {
    let mut cursor = 0usize;
    for needle in expected {
        let Some(relative) = haystack[cursor..].find(needle) else {
            panic!("expected to find '{needle}' after offset {cursor} in: {haystack}");
        };
        cursor += relative + needle.len();
    }
}

fn assert_subsequence(haystack: &str, expected: &[&str]) {
    let mut cursor = 0usize;
    for needle in expected {
        let Some(relative) = haystack[cursor..].find(needle) else {
            panic!("expected to find '{needle}' after offset {cursor} in: {haystack}");
        };
        cursor += relative + needle.len();
    }
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

fn recorded_requests(records: &Arc<Mutex<Vec<serde_json::Value>>>) -> Vec<serde_json::Value> {
    records.lock().expect("lock records").clone()
}

fn recorded_cmds(records: &Arc<Mutex<Vec<serde_json::Value>>>) -> Vec<String> {
    recorded_requests(records)
        .into_iter()
        .filter_map(|value| value.get("cmd")?.as_str().map(|s| s.to_string()))
        .collect()
}

fn first_apt_like_profile(records: &Arc<Mutex<Vec<serde_json::Value>>>) -> Option<String> {
    recorded_requests(records).into_iter().find_map(|value| {
        let cmd = value.get("cmd")?.as_str()?;
        if cmd.contains("apt-get") || cmd.contains("dpkg-query") {
            return value
                .get("profile")
                .and_then(|profile| profile.as_str())
                .map(|profile| profile.to_string());
        }
        None
    })
}

fn first_pacman_like_profile(records: &Arc<Mutex<Vec<serde_json::Value>>>) -> Option<String> {
    recorded_requests(records).into_iter().find_map(|value| {
        let cmd = value.get("cmd")?.as_str()?;
        if cmd.contains("pacman -Sy --noconfirm --needed") {
            return value
                .get("profile")
                .and_then(|profile| profile.as_str())
                .map(|profile| profile.to_string());
        }
        None
    })
}

fn first_probe_profile(records: &Arc<Mutex<Vec<serde_json::Value>>>) -> Option<String> {
    recorded_requests(records).into_iter().find_map(|value| {
        let cmd = value.get("cmd")?.as_str()?;
        if cmd.contains("os_release_readable")
            || cmd.contains("pacman_present")
            || cmd.contains("/etc/os-release")
        {
            return value
                .get("profile")
                .and_then(|profile| profile.as_str())
                .map(|profile| profile.to_string());
        }
        None
    })
}

fn assert_no_apt_like_requests(records: &Arc<Mutex<Vec<serde_json::Value>>>) {
    let cmds = recorded_cmds(records);
    assert!(
        cmds.iter()
            .all(|cmd| !cmd.contains("apt-get") && !cmd.contains("dpkg-query")),
        "expected no apt/dpkg-query requests; cmds={cmds:?}"
    );
}

#[test]
fn world_enable_provision_deps_dry_run_runs_probe_and_skips_apt_side_effects() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&["curl-unpinned", "nodejs-pinned", "nodejs-unpinned", "zlib"]);
    fixture.write_workspace_config(&[]);
    fixture.write_apt_package("curl-unpinned", &[("curl", None)]);
    fixture.write_apt_package("nodejs-pinned", &[("nodejs", Some("20.0.0-1"))]);
    fixture.write_apt_package("nodejs-unpinned", &[("nodejs", None)]);
    fixture.write_apt_package("zlib", &[("zlib1g", None)]);

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdap0-dry-run-", APT_PROBE_STDOUT, "", 0);

    let assert = fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps", "--dry-run", "--verbose"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("world-deps-provision"),
        "verbose dry-run must print the provisioning profile: {stdout}"
    );
    assert!(
        stdout.contains("curl\nnodejs=20.0.0-1\nzlib1g"),
        "dry-run must print normalized apt requirements in sorted order: {stdout}"
    );
    assert!(
        fixture.log_contents().is_none(),
        "dry-run must not run the helper or write helper logs"
    );
    assert_eq!(
        first_probe_profile(&records).as_deref(),
        Some("world-deps-probe"),
        "dry-run must execute the in-world manager probe; records={:?}",
        recorded_requests(&records)
    );
    assert_no_apt_like_requests(&records);
    assert!(
        stdout.contains("World-manager probe result: apt"),
        "dry-run should surface the probe classification: {stdout}"
    );
}

#[test]
fn world_enable_provision_deps_conflicts_fail_before_helper_or_world_agent() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&["nodejs-20", "nodejs-18"]);
    fixture.write_workspace_config(&[]);
    fixture.write_apt_package("nodejs-20", &[("nodejs", Some("20.0.0-1"))]);
    fixture.write_apt_package("nodejs-18", &[("nodejs", Some("18.0.0-1"))]);

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdap0-conflict-", APT_PROBE_STDOUT, "", 0);

    fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("nodejs"))
        .stderr(predicate::str::contains("20.0.0-1"))
        .stderr(predicate::str::contains("18.0.0-1"));

    assert!(
        fixture.log_contents().is_none(),
        "helper must not run when apt normalization conflicts"
    );
    assert!(
        recorded_requests(&records).is_empty(),
        "conflict path must not execute world-agent requests"
    );
}

#[test]
fn world_enable_provision_deps_forces_helper_no_sync_and_provision_profile() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&["nodejs"]);
    fixture.write_workspace_config(&[]);
    fixture.write_apt_package("nodejs", &[("nodejs", None)]);

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdap0-profile-", APT_PROBE_STDOUT, "", 0);

    fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .env("SUBSTRATE_WORLD_REQUEST_PROFILE", "operator-override")
        .args(["--provision-deps"])
        .assert()
        .success();

    let log = fixture.log_contents().expect("helper log missing");
    assert!(
        log.contains("--no-sync-deps"),
        "helper must receive --no-sync-deps when provision-deps is enabled: {log}"
    );
    assert_eq!(
        first_apt_like_profile(&records).as_deref(),
        Some("world-deps-provision"),
        "first apt/dpkg world-agent request must force profile=world-deps-provision; records={:?}",
        recorded_requests(&records)
    );
}

#[test]
fn world_enable_provision_deps_empty_requirement_set_skips_apt_execution() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&[]);
    fixture.write_workspace_config(&[]);

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdap0-empty-", APT_PROBE_STDOUT, "", 0);

    fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps"])
        .assert()
        .success();

    let log = fixture.log_contents().expect("helper log missing");
    assert!(
        log.contains("--no-sync-deps"),
        "helper must receive --no-sync-deps when provision-deps is enabled: {log}"
    );
    assert_no_apt_like_requests(&records);
}

#[test]
fn world_enable_provision_deps_empty_dry_run_still_probes_and_surfaces_manager() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&[]);
    fixture.write_workspace_config(&[]);

    let (_sock_tmp, socket_path, _socket, records) = start_world_socket_execute_record(
        "substrate-wdap0-empty-dry-run-",
        APT_PROBE_STDOUT,
        "",
        0,
    );

    let assert = fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps", "--dry-run"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("World-manager probe result: apt"),
        "empty dry-run must still surface the detected manager: {stdout}"
    );
    assert!(
        fixture.log_contents().is_none(),
        "empty dry-run must remain a no-op"
    );
    assert_eq!(
        first_probe_profile(&records).as_deref(),
        Some("world-deps-probe"),
        "empty dry-run must still perform the in-world probe; records={:?}",
        recorded_requests(&records)
    );
    assert_no_apt_like_requests(&records);
}

#[test]
fn world_enable_provision_deps_installs_with_dns_preflight_and_remediation() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&["sl"]);
    fixture.write_workspace_config(&[]);
    fixture.write_apt_package("sl", &[("sl", None)]);

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdap0-dns-", APT_PROBE_STDOUT, "", 0);

    fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps"])
        .assert()
        .success();

    let cmds = recorded_cmds(&records);
    let install_cmd = cmds
        .iter()
        .find(|cmd| cmd.contains("apt-get install"))
        .unwrap_or_else(|| panic!("expected apt-get install request; cmds={cmds:?}"));
    assert!(
        install_cmd.contains("ports.ubuntu.com"),
        "expected DNS preflight to probe ports.ubuntu.com: {install_cmd}"
    );
    assert!(
        install_cmd.contains("systemd-resolved"),
        "expected DNS remediation to try enabling systemd-resolved: {install_cmd}"
    );
    assert!(
        install_cmd.contains("/run/systemd/resolve/resolv.conf"),
        "expected DNS remediation to link /etc/resolv.conf to /run/systemd/resolve/resolv.conf: {install_cmd}"
    );
}

#[test]
fn world_enable_provision_deps_dry_run_runs_pacman_probe_and_prints_normalized_packages() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&["pkg-a", "pkg-b"]);
    fixture.write_workspace_config(&[]);
    fixture.write_pacman_package("pkg-a", &["zlib", "curl"]);
    fixture.write_pacman_package("pkg-b", &["curl", "alpm"]);

    let (_sock_tmp, socket_path, _socket, records) = start_world_socket_execute_record(
        "substrate-wdap0-pacman-dry-run-",
        PACMAN_PROBE_STDOUT,
        "",
        0,
    );

    let assert = fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps", "--dry-run", "--verbose"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("world-deps-provision"),
        "verbose dry-run must print the provisioning profile: {stdout}"
    );
    assert_lines_in_order(&stdout, &["alpm", "curl", "zlib"]);
    assert_eq!(
        first_probe_profile(&records).as_deref(),
        Some("world-deps-probe"),
        "dry-run must execute the in-world manager probe; records={:?}",
        recorded_requests(&records)
    );
    assert_eq!(
        first_pacman_like_profile(&records).as_deref(),
        None,
        "dry-run must not execute pacman provisioning; records={:?}",
        recorded_requests(&records)
    );
}

#[test]
fn world_enable_provision_deps_runs_pacman_with_normalized_order_and_internal_profile() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&["pkg-a", "pkg-b"]);
    fixture.write_workspace_config(&[]);
    fixture.write_pacman_package("pkg-a", &["zlib", "curl"]);
    fixture.write_pacman_package("pkg-b", &["curl", "alpm"]);

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdap0-pacman-", PACMAN_PROBE_STDOUT, "", 0);

    fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps"])
        .assert()
        .success();

    let log = fixture.log_contents().expect("helper log missing");
    assert!(
        log.contains("--no-sync-deps"),
        "helper must receive --no-sync-deps when provision-deps is enabled: {log}"
    );

    let cmds = recorded_cmds(&records);
    let install_cmd = cmds
        .iter()
        .find(|cmd| cmd.contains("pacman -Sy --noconfirm --needed"))
        .unwrap_or_else(|| panic!("expected pacman install request; cmds={cmds:?}"));
    assert_subsequence(
        install_cmd,
        &["pacman -Sy --noconfirm --needed", "alpm", "curl", "zlib"],
    );
    assert_eq!(
        first_pacman_like_profile(&records).as_deref(),
        Some("world-deps-provision"),
        "pacman provisioning request must force profile=world-deps-provision; records={:?}",
        recorded_requests(&records)
    );
}

#[test]
fn world_enable_provision_deps_rejects_mixed_manager_sets_before_helper_or_world_agent() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&["apt-only", "pacman-only"]);
    fixture.write_workspace_config(&[]);
    fixture.write_apt_package("apt-only", &[("curl", None)]);
    fixture.write_pacman_package("pacman-only", &["curl"]);

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdap0-mixed-", APT_PROBE_STDOUT, "", 0);

    fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("mixed-manager"))
        .stderr(predicate::str::contains("apt"))
        .stderr(predicate::str::contains("pacman"));

    assert!(
        fixture.log_contents().is_none(),
        "helper must not run when mixed-manager provisioning is rejected"
    );
    assert!(
        recorded_requests(&records).is_empty(),
        "mixed-manager rejection must not execute world-agent requests"
    );
}

#[test]
fn world_enable_provision_deps_rejects_pacman_probe_before_apt_provisioning() {
    let fixture = WorldEnableProvisionFixture::new();
    fixture.write_global_config(&["curl"]);
    fixture.write_workspace_config(&[]);
    fixture.write_apt_package("curl", &[("curl", None)]);

    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdap0-pacman-", PACMAN_PROBE_STDOUT, "", 0);

    fixture
        .command()
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_TEST_HELPER_PRESERVE_SOCKET", "1")
        .args(["--provision-deps"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("pacman"))
        .stderr(predicate::str::contains("apt-backed packages only"));

    assert_no_apt_like_requests(&records);
    assert_eq!(
        first_probe_profile(&records).as_deref(),
        Some("world-deps-probe"),
        "pacman classification must still come from the in-world probe; records={:?}",
        recorded_requests(&records)
    );
}
