#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tempfile::TempDir;
use toml::value::Table as TomlTable;
use toml::Value as TomlValue;

struct ConfigSetFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace: PathBuf,
}

impl ConfigSetFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-config-set-");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("failed to create HOME fixture");
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).expect("failed to create workspace fixture");
        let substrate_home = temp.path().join("alt-substrate-home");
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME fixture");
        Self {
            _temp: temp,
            home,
            substrate_home,
            workspace,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home);
        cmd
    }

    fn config_path(&self) -> PathBuf {
        self.substrate_home.join("config.toml")
    }

    fn init_config(&self) {
        let mut cmd = self.command();
        cmd.arg("config").arg("init").assert().success();
    }

    fn read_config(&self) -> TomlValue {
        let data = fs::read_to_string(self.config_path()).expect("config contents");
        toml::from_str(&data).expect("config to parse as TOML")
    }

    fn raw_contents(&self) -> String {
        fs::read_to_string(self.config_path()).expect("config contents")
    }

    fn set_command(&self) -> Command {
        let mut cmd = self.command();
        cmd.arg("config").arg("set");
        cmd
    }

    fn workspace(&self) -> &Path {
        &self.workspace
    }

    fn substrate_home(&self) -> &Path {
        &self.substrate_home
    }
}

#[test]
fn config_set_updates_anchor_mode() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();

    fixture
        .set_command()
        .arg("world.anchor_mode=follow-cwd")
        .assert()
        .success();

    let config = fixture.read_config();
    let world = world_table(&config);
    assert_eq!(
        world.get("anchor_mode").and_then(|value| value.as_str()),
        Some("follow-cwd"),
        "world.anchor_mode should reflect config set change"
    );
}

#[test]
fn config_set_updates_anchor_path() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();

    let expected_path = "/tmp/config-set-anchor";
    fixture
        .set_command()
        .arg(format!("world.anchor_path={expected_path}"))
        .assert()
        .success();

    let config = fixture.read_config();
    let world = world_table(&config);
    assert_eq!(
        world.get("anchor_path").and_then(|value| value.as_str()),
        Some(expected_path),
        "world.anchor_path should update to requested path"
    );
}

#[test]
fn config_set_updates_world_caged_flag() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();

    fixture
        .set_command()
        .arg("world.caged=false")
        .assert()
        .success();

    let config = fixture.read_config();
    let world = world_table(&config);
    assert_eq!(
        world.get("caged").and_then(|value| value.as_bool()),
        Some(false),
        "world.caged should follow config set boolean"
    );
}

#[test]
fn config_set_updates_install_world_enabled_flag() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();

    fixture
        .set_command()
        .arg("install.world_enabled=false")
        .assert()
        .success();

    let config = fixture.read_config();
    let install = install_table(&config);
    assert_eq!(
        install
            .get("world_enabled")
            .and_then(|value| value.as_bool()),
        Some(false),
        "install.world_enabled should update to requested boolean"
    );
}

#[test]
fn config_set_updates_multiple_keys_atomically() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();

    fixture
        .set_command()
        .arg("world.anchor_mode=custom")
        .arg("world.anchor_path=/tmp/multi-anchor")
        .arg("install.world_enabled=false")
        .assert()
        .success();

    let config = fixture.read_config();
    let world = world_table(&config);
    assert_eq!(
        world.get("anchor_mode").and_then(|value| value.as_str()),
        Some("custom"),
        "world.anchor_mode should match multi-key update"
    );
    assert_eq!(
        world.get("anchor_path").and_then(|value| value.as_str()),
        Some("/tmp/multi-anchor"),
        "world.anchor_path should match multi-key update"
    );
    let install = install_table(&config);
    assert_eq!(
        install
            .get("world_enabled")
            .and_then(|value| value.as_bool()),
        Some(false),
        "install.world_enabled should reflect the combined run"
    );
}

#[test]
fn config_set_rejects_invalid_anchor_mode_without_mutation() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();
    let before = fixture.raw_contents();

    fixture
        .set_command()
        .arg("world.anchor_mode=invalid-mode")
        .assert()
        .failure();

    let after = fixture.raw_contents();
    assert_eq!(
        before, after,
        "config should not change when anchor_mode value is invalid"
    );
}

#[test]
fn config_set_rejects_non_boolean_values() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();
    let before = fixture.raw_contents();

    fixture
        .set_command()
        .arg("install.world_enabled=perhaps")
        .assert()
        .failure();

    let after = fixture.raw_contents();
    assert_eq!(
        before, after,
        "config should remain unchanged when boolean parsing fails"
    );
}

#[test]
fn config_set_reports_applied_changes_as_json() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();

    let mut cmd = fixture.set_command();
    let output = cmd
        .arg("--json")
        .arg("world.anchor_mode=custom")
        .arg("world.anchor_path=/tmp/json-anchor")
        .arg("install.world_enabled=false")
        .output()
        .expect("failed to execute substrate config set --json");
    assert!(
        output.status.success(),
        "config set --json should succeed: {:?}",
        output
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: JsonValue =
        serde_json::from_str(&stdout).expect("config set --json output to parse");
    let applied = payload
        .get("applied")
        .and_then(|value| value.as_object())
        .expect("applied map present in JSON payload");
    assert_eq!(
        applied
            .get("world.anchor_mode")
            .and_then(|value| value.as_str()),
        Some("custom"),
        "json payload should describe updated anchor_mode"
    );
    assert_eq!(
        applied
            .get("world.anchor_path")
            .and_then(|value| value.as_str()),
        Some("/tmp/json-anchor"),
        "json payload should describe updated anchor_path"
    );
    assert_eq!(
        applied
            .get("install.world_enabled")
            .and_then(|value| value.as_bool()),
        Some(false),
        "json payload should report boolean keys faithfully"
    );
}

#[test]
fn config_set_preserves_config_when_write_fails() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();
    let before = fixture.raw_contents();

    let mut perms = fs::metadata(fixture.substrate_home())
        .expect("substrate home metadata")
        .permissions();
    let original_mode = perms.mode();
    perms.set_mode(0o555);
    fs::set_permissions(fixture.substrate_home(), perms).expect("set read-only permissions");

    fixture
        .set_command()
        .arg("world.anchor_mode=custom")
        .arg("world.anchor_path=/tmp/failure")
        .assert()
        .failure();

    let restore = Permissions::from_mode(original_mode);
    fs::set_permissions(fixture.substrate_home(), restore).expect("restore permissions");

    let after = fixture.raw_contents();
    assert_eq!(
        before, after,
        "config should remain intact when atomic persist fails"
    );
}

#[test]
fn cli_flags_still_override_config_after_config_set() {
    if !ensure_config_set_available() {
        return;
    }

    let fixture = ConfigSetFixture::new();
    fixture.init_config();
    fixture
        .set_command()
        .arg("world.anchor_mode=custom")
        .arg("world.anchor_path=/tmp/from-config")
        .assert()
        .success();

    let project_dir = fixture.workspace().join("project");
    fs::create_dir_all(&project_dir).expect("failed to create project dir");

    let script = "printf '%s|%s' \"$SUBSTRATE_ANCHOR_MODE\" \"$SUBSTRATE_ANCHOR_PATH\"";
    let output = fixture
        .command()
        .current_dir(&project_dir)
        .arg("--anchor-mode")
        .arg("follow-cwd")
        .arg("--no-world")
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to launch substrate with CLI overrides");
    assert!(
        output.status.success(),
        "substrate run with CLI overrides should succeed: {:?}",
        output
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().split('|');
    assert_eq!(
        parts.next(),
        Some("follow-cwd"),
        "CLI --anchor-mode should override config file values"
    );
    assert_eq!(
        parts.next(),
        Some(project_dir.to_string_lossy().as_ref()),
        "follow-cwd should anchor to the current working directory"
    );
}

fn world_table<'a>(config: &'a TomlValue) -> &'a TomlTable {
    config
        .get("world")
        .and_then(|value| value.as_table())
        .expect("world table present")
}

fn install_table<'a>(config: &'a TomlValue) -> &'a TomlTable {
    config
        .get("install")
        .and_then(|value| value.as_table())
        .expect("install table present")
}

fn ensure_config_set_available() -> bool {
    if config_set_supported() {
        true
    } else {
        static WARNED: OnceLock<()> = OnceLock::new();
        WARNED.get_or_init(|| {
            eprintln!("skipping config set tests until the subcommand is implemented");
        });
        false
    }
}

fn config_set_supported() -> bool {
    static SUPPORTED: OnceLock<bool> = OnceLock::new();
    *SUPPORTED.get_or_init(|| {
        let mut cmd = substrate_shell_driver();
        match cmd.arg("config").arg("set").arg("--help").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    })
}
