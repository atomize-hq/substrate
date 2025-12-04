#![cfg(unix)]

#[path = "common.rs"]
mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver};
use serde_json::{json, Map, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

const HOST_SCRIPT_TEMPLATE: &str = r#"#!/usr/bin/env bash
set -euo pipefail

log="${SUBSTRATE_WORLD_DEPS_HOST_LOG:?missing host log}"
echo "host-detect:{tool}" >>"$log"
marker="{marker}"
if [[ -f "$marker" ]]; then
  exit 0
fi
exit 1
"#;

const GUEST_SCRIPT_TEMPLATE: &str = r#"#!/usr/bin/env bash
set -euo pipefail

log="${SUBSTRATE_WORLD_DEPS_GUEST_LOG:?missing guest log}"
echo "guest-detect:{tool}" >>"$log"
marker="{marker}"
if [[ -f "$marker" ]]; then
  exit 0
fi
exit 1
"#;

const INSTALL_SCRIPT_TEMPLATE: &str = r#"#!/usr/bin/env bash
set -euo pipefail

log="${SUBSTRATE_WORLD_DEPS_EXECUTOR_LOG:?missing executor log}"
echo "install:{tool}:$*" >>"$log"
if [[ "${SUBSTRATE_WORLD_DEPS_FAIL_TOOL:-}" == "{tool}" ]]; then
  echo "simulated failure for {tool}" >&2
  exit 90
fi
marker_dir="${SUBSTRATE_WORLD_DEPS_MARKER_DIR:?missing marker dir}"
guest_bin="${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:-$marker_dir}"
mkdir -p "$marker_dir" "$guest_bin"
touch "$marker_dir/{marker}"
# also mark the guest tool path and guest marker to satisfy post-install checks
echo -e '#!/usr/bin/env bash\nexit 0' >"${guest_bin}/{tool}"
chmod +x "${guest_bin}/{tool}"
touch "$guest_bin/{marker}"
echo "install complete for {tool}"
"#;

struct WorldDepsFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    manifest_path: PathBuf,
    overlay_path: PathBuf,
    host_marker_dir: PathBuf,
    guest_marker_dir: PathBuf,
    scripts_dir: PathBuf,
    host_log_path: PathBuf,
    guest_log_path: PathBuf,
    executor_log_path: PathBuf,
    fake_socket_path: PathBuf,
    guest_bin_dir: PathBuf,
}

impl WorldDepsFixture {
    fn new() -> Self {
        // Use /tmp to ensure the world sandbox can write logs/markers.
        let temp = Builder::new()
            .prefix("substrate-world-deps-")
            .tempdir()
            .expect("world deps tempdir");
        let root = temp.path();
        let home = root.join("home");
        let substrate_home = home.join(".substrate");
        let manifest_path = root.join("manifests/world-deps.yaml");
        let overlay_path = substrate_home.join("world-deps.local.yaml");
        let host_marker_dir = root.join("markers/host");
        let guest_marker_dir = root.join("markers/guest");
        let scripts_dir = root.join("scripts");
        let logs_dir = root.join("logs");
        let host_log_path = logs_dir.join("host.log");
        let guest_log_path = logs_dir.join("guest.log");
        let executor_log_path = logs_dir.join("executor.log");
        let fake_socket_path = root.join("fake-world.sock");
        let guest_bin_dir = root.join("guest-bin");

        fs::create_dir_all(&home).expect("fixture home");
        fs::create_dir_all(&substrate_home).expect("fixture substrate home");
        fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("manifest dir");
        fs::create_dir_all(&host_marker_dir).expect("host marker dir");
        fs::create_dir_all(&guest_marker_dir).expect("guest marker dir");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&logs_dir).expect("logs dir");
        fs::create_dir_all(&guest_bin_dir).expect("guest bin dir");

        Self {
            _temp: temp,
            home,
            substrate_home,
            manifest_path,
            overlay_path,
            host_marker_dir,
            guest_marker_dir,
            scripts_dir,
            host_log_path,
            guest_log_path,
            executor_log_path,
            fake_socket_path,
            guest_bin_dir,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.arg("world")
            .arg("deps")
            .env("TMPDIR", self._temp.path())
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            // Keep world enabled but avoid real host sockets; force manual mode and point to a temp socket.
            .env("SUBSTRATE_WORLD", "enabled")
            .env("SUBSTRATE_WORLD_ENABLED", "1")
            .env("SUBSTRATE_WORLD_SOCKET", &self.fake_socket_path)
            .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
            .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &self.manifest_path)
            .env("SUBSTRATE_WORLD_DEPS_MARKER_DIR", &self.guest_marker_dir)
            .env("SUBSTRATE_WORLD_DEPS_HOST_LOG", &self.host_log_path)
            .env("SUBSTRATE_WORLD_DEPS_GUEST_LOG", &self.guest_log_path)
            .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &self.guest_bin_dir)
            .env("SUBSTRATE_WORLD_DEPS_EXECUTOR_LOG", &self.executor_log_path);
        // Ensure the stub guest bin dir is on PATH so post-install checks succeed.
        let current_path = std::env::var("PATH").unwrap_or_default();
        let prefix = self.guest_bin_dir.display().to_string();
        let new_path = if current_path.is_empty() {
            prefix
        } else {
            format!("{prefix}:{current_path}")
        };
        cmd.env("PATH", new_path);
        cmd
    }

    fn write_manifest(&self, tools: &[&str]) {
        let mut managers: Map<String, Value> = Map::new();
        for tool in tools {
            managers.insert(
                (*tool).to_string(),
                json!({
                    "detect": {
                        "commands": [self.host_script(tool)]
                    },
                    "guest_detect": {
                        "command": self.guest_script(tool)
                    },
                    "guest_install": {
                        "custom": self.install_script(tool, tool)
                    }
                }),
            );
        }

        let manifest = Value::Object({
            let mut root = Map::new();
            root.insert("version".into(), json!(1));
            root.insert("managers".into(), Value::Object(managers));
            root
        });

        fs::write(
            &self.manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .expect("write manifest");
    }

    fn write_overlay_install_override(&self, tool: &str) {
        let marker_name = format!("overlay-{tool}");
        let script = self.install_script(tool, &marker_name);
        let guest_detect = self.guest_script_with_marker(tool, &marker_name, "-overlay");
        let overlay = Value::Object({
            let mut root = Map::new();
            root.insert("version".into(), json!(1));
            root.insert(
                "managers".into(),
                Value::Object({
                    let mut entries = Map::new();
                    entries.insert(
                        tool.to_string(),
                        json!({
                            "guest_detect": {
                                "command": guest_detect
                            },
                            "guest_install": {
                                "custom": script
                            }
                        }),
                    );
                    entries
                }),
            );
            root
        });

        if let Some(parent) = self.overlay_path.parent() {
            fs::create_dir_all(parent).expect("overlay dir");
        }
        fs::write(
            &self.overlay_path,
            serde_json::to_string_pretty(&overlay).unwrap(),
        )
        .expect("write overlay");
    }

    fn host_script(&self, tool: &str) -> String {
        let marker = self.host_marker_path(tool);
        let path = self.scripts_dir.join(format!("host-{tool}.sh"));
        let contents = HOST_SCRIPT_TEMPLATE
            .replace("{tool}", tool)
            .replace("{marker}", marker.to_string_lossy().as_ref());
        self.write_script(&path, &contents);
        path.to_string_lossy().into_owned()
    }

    fn guest_script(&self, tool: &str) -> String {
        self.guest_script_with_marker(tool, tool, "")
    }

    fn guest_script_with_marker(&self, tool: &str, marker_name: &str, suffix: &str) -> String {
        let marker = self.guest_marker_dir.join(marker_name);
        let path = self
            .scripts_dir
            .join(format!("guest-{tool}{suffix}.sh"));
        let contents = GUEST_SCRIPT_TEMPLATE
            .replace("{tool}", tool)
            .replace("{marker}", marker.to_string_lossy().as_ref());
        self.write_script(&path, &contents);
        path.to_string_lossy().into_owned()
    }

    fn install_script(&self, tool: &str, marker_name: &str) -> String {
        let path = self
            .scripts_dir
            .join(format!("install-{tool}-{marker_name}.sh"));
        let contents = INSTALL_SCRIPT_TEMPLATE
            .replace("{tool}", tool)
            .replace("{marker}", marker_name);
        self.write_script(&path, &contents);
        path.to_string_lossy().into_owned()
    }

    fn write_script(&self, path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("script dir");
        }
        fs::write(path, contents).expect("write script");
        let mut perms = fs::metadata(path).expect("script metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("chmod script");
    }

    fn mark_host_tool(&self, tool: &str) {
        fs::write(self.host_marker_path(tool), "host").expect("host marker write");
    }

    fn mark_guest_tool(&self, tool: &str) {
        fs::write(self.guest_marker_path(tool), "guest").expect("guest marker write");
    }

    fn host_marker_path(&self, tool: &str) -> PathBuf {
        self.host_marker_dir.join(tool)
    }

    fn guest_marker_path(&self, tool: &str) -> PathBuf {
        self.guest_marker_dir.join(tool)
    }

    fn overlay_marker_path(&self, tool: &str) -> PathBuf {
        self.guest_marker_dir.join(format!("overlay-{tool}"))
    }

    fn guest_marker_exists(&self, tool: &str) -> bool {
        self.guest_marker_path(tool).exists()
    }

    fn overlay_marker_exists(&self, tool: &str) -> bool {
        self.overlay_marker_path(tool).exists()
    }

    fn read_log(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    fn host_log(&self) -> String {
        Self::read_log(&self.host_log_path)
    }

    #[allow(dead_code)]
    fn guest_log(&self) -> String {
        Self::read_log(&self.guest_log_path)
    }

    fn executor_log(&self) -> String {
        Self::read_log(&self.executor_log_path)
    }
}

#[test]
fn world_deps_status_warns_when_world_disabled_but_reports_host_info() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    let assert = fixture
        .command()
        .env("SUBSTRATE_WORLD", "disabled")
        .env("SUBSTRATE_WORLD_ENABLED", "0")
        .arg("status")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("world backend disabled"),
        "expected world-disabled warning, got: {}",
        stdout
    );
    assert!(
        stdout.contains("git"),
        "stdout missing tool entry: {}",
        stdout
    );
    assert!(
        stdout.contains("host=present"),
        "stdout missing host=present: {}",
        stdout
    );
    assert!(
        stdout.contains("guest=missing"),
        "stdout missing guest=missing: {}",
        stdout
    );
    assert!(
        fixture.host_log().contains("host-detect:git"),
        "host detection should still run when world disabled"
    );
}

#[test]
fn world_deps_install_executes_install_script_and_streams_output() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    let assert = fixture
        .command()
        .arg("install")
        .arg("git")
        .arg("--verbose")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("install complete for git"),
        "stdout missing script output: {}",
        stdout
    );
    assert!(
        fixture.guest_marker_exists("git"),
        "install should create guest marker"
    );
    assert!(
        fixture.executor_log().contains("install:git"),
        "expected executor log to capture install invocation"
    );
}

#[test]
fn world_deps_install_respects_dry_run() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    fixture
        .command()
        .arg("install")
        .arg("git")
        .arg("--dry-run")
        .assert()
        .success();

    assert!(
        !fixture.guest_marker_exists("git"),
        "dry run should avoid touching guest state"
    );
    assert!(
        fixture.executor_log().is_empty(),
        "executor log should stay empty during dry run"
    );
}

#[test]
fn world_deps_install_fails_when_world_disabled() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    let assert = fixture
        .command()
        .env("SUBSTRATE_WORLD", "disabled")
        .env("SUBSTRATE_WORLD_ENABLED", "0")
        .arg("install")
        .arg("git")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("world backend disabled"),
        "stderr missing guidance when world disabled: {}",
        stderr
    );
    assert!(
        !fixture.guest_marker_exists("git"),
        "install should not mutate guest while disabled"
    );
}

#[test]
fn world_deps_sync_installs_missing_tools_with_all_flag() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git", "node"]);
    fixture.mark_host_tool("git");
    fixture.mark_host_tool("node");
    fixture.mark_guest_tool("node"); // already present in guest

    fixture
        .command()
        .arg("sync")
        .arg("--all")
        .arg("--verbose")
        .assert()
        .success();

    assert!(
        fixture.guest_marker_exists("git"),
        "git should be installed"
    );
    assert!(
        fixture.guest_marker_exists("node"),
        "node marker should remain when already installed"
    );
    let log = fixture.executor_log();
    assert!(
        log.contains("install:git"),
        "git install missing from log: {}",
        log
    );
    assert!(
        !log.contains("install:node"),
        "sync should skip guest-complete tools: {}",
        log
    );
}

#[test]
fn world_deps_install_prefers_overlay_manifest_entries() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");
    fixture.write_overlay_install_override("git");

    fixture
        .command()
        .arg("install")
        .arg("git")
        .assert()
        .success();

    assert!(
        fixture.overlay_marker_exists("git"),
        "overlay install script should create overlay marker"
    );
    assert!(
        !fixture.guest_marker_exists("git"),
        "base install script should be overridden by overlay"
    );
}

#[test]
fn world_deps_install_surfaces_helper_failures() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    let assert = fixture
        .command()
        .env("SUBSTRATE_WORLD_DEPS_FAIL_TOOL", "git")
        .arg("install")
        .arg("git")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("simulated failure for git"),
        "stderr missing executor failure: {}",
        stderr
    );
    assert!(
        !fixture.guest_marker_exists("git"),
        "failed install should not report guest success"
    );
}
