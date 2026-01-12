#![cfg(unix)]

#[path = "common.rs"]
mod common;
#[path = "support/socket.rs"]
mod socket;

use assert_cmd::Command;
use common::substrate_shell_driver;
use serde_json::{json, Map, Value};
use socket::{AgentSocket, SocketResponse};
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
echo "guest-detect:{tool}:{tag}" >>"$log"
marker="{marker}"
if [[ -f "$marker" ]]; then
  exit 0
fi
exit 1
"#;

const INSTALL_SCRIPT_TEMPLATE: &str = r#"#!/usr/bin/env bash
set -euo pipefail

log="${SUBSTRATE_WORLD_DEPS_EXECUTOR_LOG:?missing executor log}"
echo "install:{tool}:{tag}:$*" >>"$log"

marker_dir="${SUBSTRATE_WORLD_DEPS_MARKER_DIR:?missing marker dir}"
guest_bin="${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:-$marker_dir}"
mkdir -p "$marker_dir" "$guest_bin"
touch "$marker_dir/{marker_name}"

# also mark the guest tool path and guest marker to satisfy post-install checks
echo -e '#!/usr/bin/env bash\nexit 0' >"${guest_bin}/{tool}"
chmod +x "${guest_bin}/{tool}"
touch "$guest_bin/{marker_name}"

echo "install complete for {tool} ({tag})"
"#;

struct LayeringFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    base_inventory_manifest: PathBuf,
    installed_overlay_manifest: PathBuf,
    user_overlay_manifest: PathBuf,
    host_marker_dir: PathBuf,
    guest_marker_dir: PathBuf,
    scripts_dir: PathBuf,
    host_log_path: PathBuf,
    guest_log_path: PathBuf,
    executor_log_path: PathBuf,
    fake_socket_path: PathBuf,
    guest_bin_dir: PathBuf,
}

impl LayeringFixture {
    fn new() -> Self {
        // Keep the Unix socket path short to avoid `SUN_LEN` failures.
        let temp = Builder::new()
            .prefix("substrate-world-deps-layering-")
            .tempdir_in("/tmp")
            .expect("world deps layering tempdir");
        let root = temp.path();
        let home = root.join("home");
        let substrate_home = home.join(".substrate");
        let manifests_dir = root.join("manifests");
        let base_inventory_manifest = manifests_dir.join("manager_hooks.yaml");
        let installed_overlay_manifest = manifests_dir.join("world-deps.yaml");
        let user_overlay_manifest = substrate_home.join("world-deps.local.yaml");
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
        fs::create_dir_all(&manifests_dir).expect("manifests dir");
        fs::create_dir_all(&host_marker_dir).expect("host marker dir");
        fs::create_dir_all(&guest_marker_dir).expect("guest marker dir");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&logs_dir).expect("logs dir");
        fs::create_dir_all(&guest_bin_dir).expect("guest bin dir");

        Self {
            _temp: temp,
            home,
            substrate_home,
            base_inventory_manifest,
            installed_overlay_manifest,
            user_overlay_manifest,
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

    fn selection_path(&self) -> PathBuf {
        self.substrate_home.join("world-deps.selection.yaml")
    }

    fn write_selection(&self, selected: &[&str]) {
        let path = self.selection_path();
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
            // Base inventory must align with shim doctor/health (manager manifest).
            .env("SUBSTRATE_MANAGER_MANIFEST", &self.base_inventory_manifest)
            // `world-deps.yaml` is treated as the installed overlay layer (M5a).
            .env(
                "SUBSTRATE_WORLD_DEPS_MANIFEST",
                &self.installed_overlay_manifest,
            )
            .env("SUBSTRATE_WORLD_DEPS_MARKER_DIR", &self.guest_marker_dir)
            .env("SUBSTRATE_WORLD_DEPS_HOST_LOG", &self.host_log_path)
            .env("SUBSTRATE_WORLD_DEPS_GUEST_LOG", &self.guest_log_path)
            .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &self.guest_bin_dir)
            .env("SUBSTRATE_WORLD_DEPS_EXECUTOR_LOG", &self.executor_log_path)
            // Force world execution through a test-owned socket stub (avoid relying on real world-agent).
            .env_remove("SUBSTRATE_WORLD")
            .env_remove("SUBSTRATE_WORLD_ENABLED")
            .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
            .env("SUBSTRATE_WORLD_SOCKET", &self.fake_socket_path)
            .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
            .env_remove("SHIM_ORIGINAL_PATH");

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

    #[cfg(not(target_os = "macos"))]
    fn mark_host_tool(&self, tool: &str) {
        fs::write(self.host_marker_dir.join(tool), "host").expect("host marker write");
    }

    #[cfg(not(target_os = "macos"))]
    fn marker_exists(&self, marker_name: &str) -> bool {
        self.guest_marker_dir.join(marker_name).exists()
    }

    fn write_base_inventory_manifest(&self, tools: &[(&str, &str)]) {
        self.write_manifest(&self.base_inventory_manifest, tools);
    }

    fn write_installed_overlay_manifest(&self, tools: &[(&str, &str)]) {
        self.write_manifest(&self.installed_overlay_manifest, tools);
    }

    #[cfg(not(target_os = "macos"))]
    fn write_user_overlay_manifest(&self, tools: &[(&str, &str)]) {
        self.write_manifest(&self.user_overlay_manifest, tools);
    }

    fn write_manifest(&self, path: &Path, tools: &[(&str, &str)]) {
        let mut managers: Map<String, Value> = Map::new();
        for (tool, marker_name) in tools {
            managers.insert(
                (*tool).to_string(),
                json!({
                    "detect": {
                        "commands": [self.host_script(tool)]
                    },
                    "guest_detect": {
                        "command": self.guest_script(tool, marker_name, path.file_name().unwrap_or_default().to_string_lossy().as_ref())
                    },
                    "guest_install": {
                        "class": "user_space",
                        "custom": self.install_script(tool, marker_name, path.file_name().unwrap_or_default().to_string_lossy().as_ref())
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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("manifest parent");
        }
        fs::write(path, serde_json::to_string_pretty(&manifest).unwrap()).expect("write manifest");
    }

    fn host_script(&self, tool: &str) -> String {
        let marker = self.host_marker_dir.join(tool);
        let path = self.scripts_dir.join(format!("host-{tool}.sh"));
        let contents = HOST_SCRIPT_TEMPLATE
            .replace("{tool}", tool)
            .replace("{marker}", marker.to_string_lossy().as_ref());
        self.write_script(&path, &contents);
        path.to_string_lossy().into_owned()
    }

    fn guest_script(&self, tool: &str, marker_name: &str, tag: &str) -> String {
        let marker = self.guest_marker_dir.join(marker_name);
        let safe_tag = tag.replace('/', "-");
        let path = self.scripts_dir.join(format!("guest-{tool}-{safe_tag}.sh"));
        let contents = GUEST_SCRIPT_TEMPLATE
            .replace("{tool}", tool)
            .replace("{tag}", tag)
            .replace("{marker}", marker.to_string_lossy().as_ref());
        self.write_script(&path, &contents);
        path.to_string_lossy().into_owned()
    }

    fn install_script(&self, tool: &str, marker_name: &str, tag: &str) -> String {
        let safe_tag = tag.replace('/', "-");
        let path = self
            .scripts_dir
            .join(format!("install-{tool}-{marker_name}-{safe_tag}.sh"));
        let contents = INSTALL_SCRIPT_TEMPLATE
            .replace("{tool}", tool)
            .replace("{tag}", tag)
            .replace("{marker_name}", marker_name);
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
}

fn canonicalize_or(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn parse_status(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("parse world deps status JSON")
}

fn tool_names(report: &Value) -> Vec<String> {
    report["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|entry| entry["name"].as_str().map(|value| value.to_string()))
        .collect()
}

fn assert_manifest_layers(
    report: &Value,
    expected_base: &Path,
    expected_installed: &Path,
    expected_user: &Path,
    user_exists: bool,
) {
    let manifest = report.get("manifest").expect("manifest section missing");

    let expected_base = canonicalize_or(expected_base);
    let expected_installed = canonicalize_or(expected_installed);
    let expected_user = canonicalize_or(expected_user);

    if let Some(layers) = manifest.get("layers").and_then(|value| value.as_array()) {
        let resolved: Vec<PathBuf> = layers
            .iter()
            .filter_map(|entry| entry.as_str())
            .map(PathBuf::from)
            .map(|path| canonicalize_or(&path))
            .collect();
        assert!(
            resolved.contains(&expected_base),
            "manifest.layers missing base inventory: {resolved:?}"
        );
        assert!(
            resolved.contains(&expected_installed),
            "manifest.layers missing installed overlay: {resolved:?}"
        );
        assert!(
            resolved.contains(&expected_user),
            "manifest.layers missing user overlay: {resolved:?}"
        );
        if let Some(exists) = manifest
            .get("user_overlay_exists")
            .and_then(|value| value.as_bool())
        {
            assert_eq!(
                exists, user_exists,
                "manifest.user_overlay_exists mismatch: {manifest}"
            );
        }
        return;
    }

    if let Some(overlays) = manifest.get("overlays").and_then(|value| value.as_array()) {
        let base = manifest
            .get("base")
            .and_then(|value| value.as_str())
            .expect("manifest.base missing");
        let resolved_base = canonicalize_or(Path::new(base));
        assert_eq!(
            resolved_base, expected_base,
            "manifest.base mismatch: {manifest}"
        );

        let resolved_overlays: Vec<PathBuf> = overlays
            .iter()
            .filter_map(|entry| entry.as_str())
            .map(PathBuf::from)
            .map(|path| canonicalize_or(&path))
            .collect();
        assert!(
            resolved_overlays.contains(&expected_installed),
            "manifest.overlays missing installed overlay: {resolved_overlays:?}"
        );
        assert!(
            resolved_overlays.contains(&expected_user),
            "manifest.overlays missing user overlay: {resolved_overlays:?}"
        );
        if let Some(exists) = manifest
            .get("user_overlay_exists")
            .and_then(|value| value.as_bool())
        {
            assert_eq!(
                exists, user_exists,
                "manifest.user_overlay_exists mismatch: {manifest}"
            );
        }
        return;
    }

    let base = manifest
        .get("base")
        .and_then(|value| value.as_str())
        .expect("manifest.base missing");
    let resolved_base = canonicalize_or(Path::new(base));
    assert_eq!(
        resolved_base, expected_base,
        "manifest.base mismatch: {manifest}"
    );

    let installed = manifest
        .get("installed_overlay")
        .and_then(|value| value.as_str())
        .expect("manifest.installed_overlay missing");
    let resolved_installed = canonicalize_or(Path::new(installed));
    assert_eq!(
        resolved_installed, expected_installed,
        "manifest.installed_overlay mismatch: {manifest}"
    );

    let user = manifest
        .get("user_overlay")
        .and_then(|value| value.as_str())
        .or_else(|| manifest.get("overlay").and_then(|value| value.as_str()))
        .expect("manifest user overlay missing");
    let resolved_user = canonicalize_or(Path::new(user));
    assert_eq!(
        resolved_user, expected_user,
        "manifest user overlay mismatch: {manifest}"
    );

    let exists = manifest
        .get("user_overlay_exists")
        .or_else(|| manifest.get("overlay_exists"))
        .and_then(|value| value.as_bool())
        .expect("manifest overlay exists missing");
    assert_eq!(
        exists, user_exists,
        "manifest overlay exists mismatch: {manifest}"
    );
}

#[test]
fn world_deps_inventory_includes_base_and_installed_overlay_tools() {
    let fixture = LayeringFixture::new();
    fixture.write_base_inventory_manifest(&[("baseonly", "baseonly")]);
    fixture.write_installed_overlay_manifest(&[("overlayonly", "overlayonly")]);
    fixture.write_selection(&[]);
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    let assert = fixture
        .command()
        .arg("status")
        .arg("--all")
        .arg("--json")
        .assert()
        .success();
    let report = parse_status(&assert.get_output().stdout);

    let names = tool_names(&report);
    assert!(
        names.contains(&"baseonly".to_string()),
        "base inventory tool missing: {names:?}"
    );
    assert!(
        names.contains(&"overlayonly".to_string()),
        "installed overlay tool missing: {names:?}"
    );

    assert_manifest_layers(
        &report,
        &fixture.base_inventory_manifest,
        &fixture.installed_overlay_manifest,
        &fixture.user_overlay_manifest,
        false,
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn world_deps_install_prefers_user_overlay_over_installed_and_base() {
    let fixture = LayeringFixture::new();
    fixture.write_base_inventory_manifest(&[("git", "base-git"), ("baseonly", "baseonly")]);
    fixture.write_installed_overlay_manifest(&[("git", "installed-git")]);
    fixture.write_user_overlay_manifest(&[("git", "user-git")]);
    fixture.mark_host_tool("git");
    fixture.write_selection(&["git"]);
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    let assert = fixture
        .command()
        .arg("status")
        .arg("--all")
        .arg("--json")
        .assert()
        .success();
    let report = parse_status(&assert.get_output().stdout);
    let names = tool_names(&report);
    assert!(
        names.contains(&"baseonly".to_string()),
        "expected base inventory tool to be present: {names:?}"
    );

    fixture
        .command()
        .arg("install")
        .arg("git")
        .arg("--verbose")
        .assert()
        .success();

    assert!(
        fixture.marker_exists("user-git"),
        "expected user overlay marker after install"
    );
    assert!(
        !fixture.marker_exists("installed-git"),
        "installed overlay marker should be overridden by user overlay"
    );
    assert!(
        !fixture.marker_exists("base-git"),
        "base inventory marker should be overridden by user overlay"
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn world_deps_install_prefers_installed_overlay_over_base_when_no_user_overlay() {
    let fixture = LayeringFixture::new();
    fixture.write_base_inventory_manifest(&[("git", "base-git"), ("baseonly", "baseonly")]);
    fixture.write_installed_overlay_manifest(&[("git", "installed-git")]);
    fixture.mark_host_tool("git");
    fixture.write_selection(&["git"]);
    let _socket = AgentSocket::start(
        &fixture.fake_socket_path,
        SocketResponse::CapabilitiesAndHostExecute { scopes: vec![] },
    );

    let assert = fixture
        .command()
        .arg("status")
        .arg("--all")
        .arg("--json")
        .assert()
        .success();
    let report = parse_status(&assert.get_output().stdout);
    let names = tool_names(&report);
    assert!(
        names.contains(&"baseonly".to_string()),
        "expected base inventory tool to be present: {names:?}"
    );

    fixture
        .command()
        .arg("install")
        .arg("git")
        .assert()
        .success();

    assert!(
        fixture.marker_exists("installed-git"),
        "expected installed overlay marker after install"
    );
    assert!(
        !fixture.marker_exists("base-git"),
        "base inventory marker should be overridden by installed overlay"
    );
}
