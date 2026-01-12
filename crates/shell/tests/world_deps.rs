#![cfg(unix)]

#[path = "common.rs"]
mod common;
#[cfg(not(target_os = "macos"))]
#[path = "support/socket.rs"]
mod socket;

use assert_cmd::Command;
use common::{
    binary_path, ensure_substrate_built, shared_tmpdir, substrate_shell_driver, temp_dir,
};
use serde_json::{json, Map, Value};
#[cfg(not(target_os = "macos"))]
use socket::{AgentSocket, SocketResponse};
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
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

const MANAGER_INIT_MARKER_ENV: &str = "SUBSTRATE_M5B_MANAGER_INIT_MARKER";
const MANAGER_INIT_MARKER_VALUE: &str = "manager-init-loaded";

const HOST_MANAGER_INIT_SCRIPT_TEMPLATE: &str = r#"#!/usr/bin/env bash
set -euo pipefail

log="${SUBSTRATE_WORLD_DEPS_HOST_LOG:?missing host log}"
echo "host-detect:{tool}:m5b-marker=${SUBSTRATE_M5B_MANAGER_INIT_MARKER:-}" >>"$log"
if [ "${SUBSTRATE_M5B_MANAGER_INIT_MARKER:-}" != "{marker}" ]; then
  exit 1
fi
command -v "{tool}" >/dev/null 2>&1
"#;

struct WorldDepsFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    manager_manifest_path: PathBuf,
    manifest_path: PathBuf,
    #[cfg(not(target_os = "macos"))]
    overlay_path: PathBuf,
    host_marker_dir: PathBuf,
    guest_marker_dir: PathBuf,
    scripts_dir: PathBuf,
    host_log_path: PathBuf,
    guest_log_path: PathBuf,
    executor_log_path: PathBuf,
    fake_socket_path: PathBuf,
    guest_bin_dir: PathBuf,
    manager_bin_dir: PathBuf,
}

impl WorldDepsFixture {
    fn new() -> Self {
        // Use /tmp to keep Unix socket paths short and ensure the world sandbox can write logs/markers.
        let temp = Builder::new()
            .prefix("substrate-world-deps-")
            .tempdir_in("/tmp")
            .expect("world deps tempdir");
        let root = temp.path();
        let home = root.join("home");
        let substrate_home = home.join(".substrate");
        let manager_manifest_path = root.join("manifests/manager_hooks.yaml");
        let manifest_path = root.join("manifests/world-deps.yaml");
        #[cfg(not(target_os = "macos"))]
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
        let manager_bin_dir = root.join("manager-bin");

        fs::create_dir_all(&home).expect("fixture home");
        fs::create_dir_all(&substrate_home).expect("fixture substrate home");
        fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("manifest dir");
        fs::create_dir_all(&host_marker_dir).expect("host marker dir");
        fs::create_dir_all(&guest_marker_dir).expect("guest marker dir");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&logs_dir).expect("logs dir");
        fs::create_dir_all(&guest_bin_dir).expect("guest bin dir");
        fs::create_dir_all(&manager_bin_dir).expect("manager bin dir");

        write_minimal_manifest(&manager_manifest_path);

        Self {
            _temp: temp,
            home,
            substrate_home,
            manager_manifest_path,
            manifest_path,
            #[cfg(not(target_os = "macos"))]
            overlay_path,
            host_marker_dir,
            guest_marker_dir,
            scripts_dir,
            host_log_path,
            guest_log_path,
            executor_log_path,
            fake_socket_path,
            guest_bin_dir,
            manager_bin_dir,
        }
    }

    fn write_selection(&self, selected: &[&str]) {
        let path = self.substrate_home.join("world-deps.selection.yaml");
        let contents = if selected.is_empty() {
            "version: 1\nselected: []\n".to_string()
        } else {
            let mut buf = String::from("version: 1\nselected:\n");
            for tool in selected {
                buf.push_str(&format!("  - {tool}\n"));
            }
            buf
        };
        fs::write(&path, contents).expect("write selection file");
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
            .env_remove("SUBSTRATE_WORLD")
            .env_remove("SUBSTRATE_WORLD_ENABLED")
            .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
            .env("SUBSTRATE_WORLD_SOCKET", &self.fake_socket_path)
            .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
            .env("SUBSTRATE_MANAGER_MANIFEST", &self.manager_manifest_path)
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
                        "class": "user_space",
                        "custom": self.install_script(tool, tool)
                    }
                }),
            );
        }

        let manifest = Value::Object({
            let mut root = Map::new();
            root.insert("version".into(), json!(2));
            root.insert("managers".into(), Value::Object(managers));
            root
        });

        fs::write(
            &self.manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .expect("write manifest");
        self.write_selection(tools);
    }

    fn write_manifest_with_host_command(&self, tool: &str, host_command: &str) {
        let mut managers: Map<String, Value> = Map::new();
        managers.insert(
            tool.to_string(),
            json!({
                "detect": {
                    "commands": [host_command]
                },
                "guest_detect": {
                    "command": self.guest_script(tool)
                },
                "guest_install": {
                    "class": "user_space",
                    "custom": self.install_script(tool, tool)
                }
            }),
        );

        let manifest = Value::Object({
            let mut root = Map::new();
            root.insert("version".into(), json!(2));
            root.insert("managers".into(), Value::Object(managers));
            root
        });

        fs::write(
            &self.manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .expect("write manifest");
        self.write_selection(&[tool]);
    }

    fn write_manager_manifest_for_init(&self) {
        let contents = format!(
            "version: 2\nmanagers:\n  - name: manager-init-test\n    detect:\n      script: \"exit 0\"\n    init:\n      shell: |\n        export {marker_env}=\"{marker_value}\"\n        export PATH=\"{manager_bin}:$PATH\"\n",
            marker_env = MANAGER_INIT_MARKER_ENV,
            marker_value = MANAGER_INIT_MARKER_VALUE,
            manager_bin = self.manager_bin_dir.display()
        );
        fs::write(&self.manager_manifest_path, contents).expect("write manager manifest");
    }

    #[cfg(not(target_os = "macos"))]
    fn write_overlay_install_override(&self, tool: &str) {
        let marker_name = format!("overlay-{tool}");
        let script = self.install_script(tool, &marker_name);
        let guest_detect = self.guest_script_with_marker(tool, &marker_name, "-overlay");
        let overlay = Value::Object({
            let mut root = Map::new();
            root.insert("version".into(), json!(2));
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
                                "class": "user_space",
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

    fn host_script_requires_manager_init(&self, tool: &str) -> String {
        let path = self
            .scripts_dir
            .join(format!("host-manager-init-{tool}.sh"));
        let contents = HOST_MANAGER_INIT_SCRIPT_TEMPLATE
            .replace("{tool}", tool)
            .replace("{marker}", MANAGER_INIT_MARKER_VALUE);
        self.write_script(&path, &contents);
        path.to_string_lossy().into_owned()
    }

    fn guest_script(&self, tool: &str) -> String {
        self.guest_script_with_marker(tool, tool, "")
    }

    fn guest_script_with_marker(&self, tool: &str, marker_name: &str, suffix: &str) -> String {
        let marker = self.guest_marker_dir.join(marker_name);
        let path = self.scripts_dir.join(format!("guest-{tool}{suffix}.sh"));
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

    fn write_manager_tool(&self, tool: &str) {
        let path = self.manager_bin_dir.join(tool);
        self.write_script(&path, "#!/usr/bin/env bash\nexit 0\n");
    }

    fn mark_host_tool(&self, tool: &str) {
        fs::write(self.host_marker_path(tool), "host").expect("host marker write");
    }

    #[cfg(not(target_os = "macos"))]
    fn mark_guest_tool(&self, tool: &str) {
        fs::write(self.guest_marker_path(tool), "guest").expect("guest marker write");
    }

    fn host_marker_path(&self, tool: &str) -> PathBuf {
        self.host_marker_dir.join(tool)
    }

    fn guest_marker_path(&self, tool: &str) -> PathBuf {
        self.guest_marker_dir.join(tool)
    }

    #[cfg(not(target_os = "macos"))]
    fn overlay_marker_path(&self, tool: &str) -> PathBuf {
        self.guest_marker_dir.join(format!("overlay-{tool}"))
    }

    fn guest_marker_exists(&self, tool: &str) -> bool {
        self.guest_marker_path(tool).exists()
    }

    #[cfg(not(target_os = "macos"))]
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

fn write_minimal_manifest(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("manifest parent dir");
    }
    fs::write(path, "version: 2\nmanagers: {}\n").expect("write manifest");
}

fn canonicalize_or(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn parse_world_deps_status_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("parse world deps status JSON")
}

fn extract_inventory_base(report: &Value) -> PathBuf {
    report["manifest"]["inventory"]["base"]
        .as_str()
        .expect("manifest.inventory.base is string")
        .into()
}

fn extract_installed_overlay(report: &Value) -> PathBuf {
    report["manifest"]["overlays"]["installed"]
        .as_str()
        .expect("manifest.overlays.installed is string")
        .into()
}

fn extract_user_overlay(report: &Value) -> Option<PathBuf> {
    report["manifest"]["overlays"]["user"]
        .as_str()
        .map(PathBuf::from)
}

fn find_tool<'a>(report: &'a Value, name: &str) -> &'a Value {
    report["tools"]
        .as_array()
        .expect("tools array missing")
        .iter()
        .find(|entry| entry["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("tool {name} missing from report: {report}"))
}

struct InstalledLayoutFixture {
    _temp: TempDir,
    prefix: PathBuf,
    installed_bin: PathBuf,
    manager_manifest: PathBuf,
    base_manifest: PathBuf,
    home: PathBuf,
    cwd: PathBuf,
}

impl InstalledLayoutFixture {
    fn new(version_label: &str) -> Self {
        ensure_substrate_built();

        let temp = temp_dir("substrate-world-deps-installed-");
        let prefix = temp.path().join("prefix");
        let version_dir = prefix.join("versions").join(version_label);
        let version_bin_dir = version_dir.join("bin");
        let version_config_dir = version_dir.join("config");
        let manager_manifest = version_config_dir.join("manager_hooks.yaml");
        let base_manifest = version_config_dir.join("world-deps.yaml");
        let installed_bin = prefix.join("bin").join("substrate");
        let installed_real_bin = version_bin_dir.join("substrate");
        let home = temp.path().join("home");
        let cwd = temp.path().join("cwd");

        fs::create_dir_all(&home).expect("home dir");
        fs::create_dir_all(&cwd).expect("cwd dir");
        fs::create_dir_all(&version_bin_dir).expect("version bin dir");
        fs::create_dir_all(installed_bin.parent().expect("bin parent")).expect("bin dir");

        write_minimal_manifest(&manager_manifest);
        write_minimal_manifest(&base_manifest);

        fs::copy(PathBuf::from(binary_path()), &installed_real_bin)
            .expect("copy substrate into install");
        let mut perms = fs::metadata(&installed_real_bin)
            .expect("installed substrate metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&installed_real_bin, perms).expect("chmod installed substrate");

        symlink(&installed_real_bin, &installed_bin).expect("symlink prefix/bin/substrate");

        Self {
            _temp: temp,
            prefix,
            installed_bin,
            manager_manifest,
            base_manifest,
            home,
            cwd,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new(&self.installed_bin);
        cmd.current_dir(&self.cwd)
            .env("TMPDIR", shared_tmpdir())
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.prefix)
            .env("SUBSTRATE_WORLD", "disabled")
            .env("SUBSTRATE_WORLD_ENABLED", "0")
            .env_remove("SUBSTRATE_WORLD_DEPS_MANIFEST")
            .env_remove("SHIM_ORIGINAL_PATH");
        cmd
    }
}

#[test]
fn world_deps_status_warns_when_world_disabled_but_reports_host_info() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    let assert = fixture
        .command()
        .env("SUBSTRATE_OVERRIDE_WORLD", "disabled")
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
fn world_deps_status_detects_tools_from_manager_init_env() {
    let fixture = WorldDepsFixture::new();
    let tool = "m5b-manager-tool";
    fixture.write_manager_manifest_for_init();
    fixture.write_manager_tool(tool);
    let host_command = fixture.host_script_requires_manager_init(tool);
    fixture.write_manifest_with_host_command(tool, &host_command);

    let manager_bin = fixture.manager_bin_dir.display().to_string();
    let current_path = std::env::var("PATH").unwrap_or_default();
    let mut path_parts: Vec<&str> = current_path
        .split(':')
        .filter(|entry| *entry != manager_bin)
        .collect();
    if path_parts.is_empty() {
        path_parts = vec!["/usr/bin", "/bin"];
    }
    let sanitized_path = path_parts.join(":");
    let path = format!("{}:{}", fixture.guest_bin_dir.display(), sanitized_path);

    let assert = fixture
        .command()
        .env("PATH", path)
        .env_remove(MANAGER_INIT_MARKER_ENV)
        .arg("status")
        .arg("--json")
        .assert()
        .success();

    let report = parse_world_deps_status_json(&assert.get_output().stdout);
    let entry = find_tool(&report, tool);
    assert_eq!(
        entry.get("host_detected").and_then(Value::as_bool),
        Some(true),
        "expected host detection to use manager init env: {report}"
    );

    let host_log = fixture.host_log();
    assert!(
        host_log.contains("host-detect:m5b-manager-tool"),
        "host detection log missing tool entry: {host_log}"
    );
    assert!(
        host_log.contains(&format!("m5b-marker={MANAGER_INIT_MARKER_VALUE}")),
        "host detection log missing manager init marker: {host_log}"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn world_deps_status_marks_guest_unavailable_when_backend_unreachable_on_macos() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    let assert = fixture
        .command()
        .arg("status")
        .arg("--json")
        .assert()
        .success();

    let report = parse_world_deps_status_json(&assert.get_output().stdout);
    let entry = find_tool(&report, "git");
    assert_eq!(
        entry["guest"]["status"].as_str(),
        Some("unavailable"),
        "expected guest status unavailable when backend missing: {report}"
    );
    let reason = entry["guest"]["reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("backend unavailable"),
        "expected guest unavailable reason, got: {reason}"
    );
    assert!(
        fixture.guest_log().is_empty(),
        "guest detection should not fall back to host: {}",
        fixture.guest_log()
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn world_deps_install_executes_install_script_and_streams_output() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

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

#[cfg(not(target_os = "macos"))]
#[test]
fn world_deps_install_respects_dry_run() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

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
        .env("SUBSTRATE_OVERRIDE_WORLD", "disabled")
        .arg("install")
        .arg("git")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("world backend unavailable for world deps")
            && stderr.contains("substrate world doctor --json"),
        "stderr missing world backend unavailable guidance: {}",
        stderr
    );
    assert!(
        !fixture.guest_marker_exists("git"),
        "install should not mutate guest while disabled"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn world_deps_install_fails_when_backend_unavailable_on_macos() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    let assert = fixture
        .command()
        .arg("install")
        .arg("git")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("world backend unavailable for world deps on macOS"),
        "stderr missing macOS backend unavailable guidance: {}",
        stderr
    );
    assert!(
        stderr.contains("substrate world doctor --json"),
        "stderr missing world doctor guidance: {}",
        stderr
    );
    assert!(
        !fixture.guest_marker_exists("git"),
        "install should not mutate guest markers when backend is unavailable"
    );
    assert!(
        fixture.executor_log().is_empty(),
        "install should not execute guest recipes on the host: {}",
        fixture.executor_log()
    );
    assert!(
        fixture.guest_log().is_empty(),
        "guest detection should not fall back to host: {}",
        fixture.guest_log()
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn world_deps_sync_skips_missing_host_tools_without_all_flag() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git", "node"]);
    fixture.write_selection(&["node"]);
    fixture.mark_host_tool("node");
    fixture.mark_guest_tool("node"); // already present in guest
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    let assert = fixture
        .command()
        .arg("sync")
        .arg("--verbose")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("All scoped tools are already available inside the guest."),
        "sync should report scoped tools already present: {stdout}"
    );
    assert!(
        fixture.executor_log().is_empty(),
        "sync should not attempt installs when selected tools are already present: {}",
        fixture.executor_log()
    );
    assert!(
        !fixture.guest_marker_exists("git"),
        "sync should not install unselected tools"
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn world_deps_sync_installs_missing_tools_with_all_flag() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git", "node"]);
    fixture.mark_host_tool("git");
    fixture.mark_host_tool("node");
    fixture.mark_guest_tool("node"); // already present in guest
    fixture.write_selection(&["node"]);
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    let assert = fixture
        .command()
        .arg("sync")
        .arg("--all")
        .arg("--verbose")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("Selection ignored due to --all"),
        "stdout missing selection ignored banner: {stdout}"
    );

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

#[cfg(target_os = "macos")]
#[test]
fn world_deps_sync_fails_when_backend_unavailable_on_macos() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");

    let assert = fixture
        .command()
        .arg("sync")
        .arg("--all")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("world backend unavailable for world deps on macOS"),
        "stderr missing macOS backend unavailable guidance: {}",
        stderr
    );
    assert!(
        stderr.contains("substrate world doctor --json"),
        "stderr missing world doctor guidance: {}",
        stderr
    );
    assert!(
        !fixture.guest_marker_exists("git"),
        "sync should not mutate guest markers when backend is unavailable"
    );
    assert!(
        fixture.executor_log().is_empty(),
        "sync should not execute guest recipes on the host: {}",
        fixture.executor_log()
    );
    assert!(
        fixture.guest_log().is_empty(),
        "guest detection should not fall back to host: {}",
        fixture.guest_log()
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn world_deps_install_prefers_overlay_manifest_entries() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");
    fixture.write_overlay_install_override("git");
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

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

#[cfg(not(target_os = "macos"))]
#[test]
fn world_deps_install_surfaces_helper_failures() {
    let fixture = WorldDepsFixture::new();
    fixture.write_manifest(&["git"]);
    fixture.mark_host_tool("git");
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

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

#[test]
fn world_deps_uses_versioned_manifest_when_running_from_installed_layout() {
    let fixture = InstalledLayoutFixture::new("9.9.9-test");

    let assert = fixture
        .command()
        .args(["world", "deps", "status", "--json"])
        .assert()
        .success();

    let report = parse_world_deps_status_json(&assert.get_output().stdout);
    let inventory_base = extract_inventory_base(&report);
    assert_eq!(
        canonicalize_or(&inventory_base),
        canonicalize_or(&fixture.manager_manifest)
    );
    let installed = extract_installed_overlay(&report);
    assert_eq!(
        canonicalize_or(&installed),
        canonicalize_or(&fixture.base_manifest)
    );
    assert_eq!(
        extract_user_overlay(&report),
        Some(fixture.prefix.join("world-deps.local.yaml"))
    );
}

#[test]
fn world_deps_workspace_build_falls_back_to_repo_manifest_when_no_installed_layout_present() {
    let temp = temp_dir("substrate-world-deps-workspace-");
    let home = temp.path().join("home");
    let substrate_home = temp.path().join("substrate-home");
    let cwd = temp.path().join("cwd");
    fs::create_dir_all(&home).expect("home dir");
    fs::create_dir_all(&substrate_home).expect("substrate home dir");
    fs::create_dir_all(&cwd).expect("cwd dir");

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|dir| dir.parent())
        .expect("repo root")
        .to_path_buf();
    let expected_inventory = repo_root.join("config/manager_hooks.yaml");
    let expected_installed = repo_root.join("scripts/substrate/world-deps.yaml");

    let assert = substrate_shell_driver()
        .current_dir(&cwd)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .env_remove("SUBSTRATE_WORLD_DEPS_MANIFEST")
        .args(["world", "deps", "status", "--json"])
        .assert()
        .success();

    let report = parse_world_deps_status_json(&assert.get_output().stdout);
    let inventory_base = extract_inventory_base(&report);
    assert_eq!(
        canonicalize_or(&inventory_base),
        canonicalize_or(&expected_inventory)
    );
    let installed = extract_installed_overlay(&report);
    assert_eq!(
        canonicalize_or(&installed),
        canonicalize_or(&expected_installed)
    );
    assert_eq!(
        extract_user_overlay(&report),
        Some(substrate_home.join("world-deps.local.yaml"))
    );
}

#[test]
fn world_deps_manifest_env_override_takes_precedence_over_defaults() {
    let temp = temp_dir("substrate-world-deps-override-");
    let home = temp.path().join("home");
    let substrate_home = temp.path().join("substrate-home");
    let cwd = temp.path().join("cwd");
    let manifest = temp.path().join("override/world-deps.yaml");
    fs::create_dir_all(&home).expect("home dir");
    fs::create_dir_all(&substrate_home).expect("substrate home dir");
    fs::create_dir_all(&cwd).expect("cwd dir");
    write_minimal_manifest(&manifest);

    let assert = substrate_shell_driver()
        .current_dir(&cwd)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .env("SUBSTRATE_WORLD_DEPS_MANIFEST", &manifest)
        .args(["world", "deps", "status", "--json"])
        .assert()
        .success();

    let report = parse_world_deps_status_json(&assert.get_output().stdout);
    let installed = extract_installed_overlay(&report);
    assert_eq!(canonicalize_or(&installed), canonicalize_or(&manifest));
    assert_eq!(
        extract_user_overlay(&report),
        Some(substrate_home.join("world-deps.local.yaml"))
    );
}
