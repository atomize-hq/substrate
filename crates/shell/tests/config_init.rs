#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use toml::Value as TomlValue;

struct ConfigInitFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
}

impl ConfigInitFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-config-init-");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("failed to create HOME fixture");
        let substrate_home = temp.path().join("alt-substrate-home");
        Self {
            _temp: temp,
            home,
            substrate_home,
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

    fn read_config(&self) -> TomlValue {
        let data = fs::read_to_string(self.config_path()).expect("config contents");
        toml::from_str(&data).expect("config to parse as TOML")
    }

    fn write_custom_config(&self, contents: &str) {
        if let Some(parent) = self.config_path().parent() {
            fs::create_dir_all(parent).expect("config parent");
        }
        fs::write(self.config_path(), contents).expect("write custom config");
    }

    fn raw_contents(&self) -> String {
        fs::read_to_string(self.config_path()).expect("config contents")
    }
}

fn assert_default_config(config: &TomlValue) {
    let install = config
        .get("install")
        .and_then(|value| value.as_table())
        .expect("install table present");
    assert_eq!(
        install
            .get("world_enabled")
            .and_then(|value| value.as_bool()),
        Some(true),
        "install.world_enabled should default to true"
    );

    let world = config
        .get("world")
        .and_then(|value| value.as_table())
        .expect("world table present");
    assert_eq!(
        world
            .get("anchor_mode")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        Some("project".to_string()),
        "world.anchor_mode default mismatch"
    );
    assert_eq!(
        world
            .get("anchor_path")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        Some(String::new()),
        "world.anchor_path default mismatch"
    );
    assert_eq!(
        world
            .get("root_mode")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        Some("project".to_string()),
        "world.root_mode should mirror anchor"
    );
    assert_eq!(
        world
            .get("root_path")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        Some(String::new()),
        "world.root_path default mismatch"
    );
    assert_eq!(
        world.get("caged").and_then(|value| value.as_bool()),
        Some(true),
        "world.caged default mismatch"
    );
}

#[test]
fn config_init_creates_default_tables_under_substrate_home() {
    let fixture = ConfigInitFixture::new();

    let mut cmd = fixture.command();
    cmd.arg("config").arg("init").assert().success();

    assert!(
        fixture.config_path().exists(),
        "config init should write {}",
        fixture.config_path().display()
    );

    let config = fixture.read_config();
    assert_default_config(&config);
}

#[test]
fn config_init_force_rewrites_existing_config() {
    let fixture = ConfigInitFixture::new();

    let mut initial = fixture.command();
    initial.arg("config").arg("init").assert().success();

    fixture.write_custom_config(
        r#"# user customizations that should be removed
[install]
world_enabled = false

[world]
anchor_mode = "custom"
anchor_path = "/tmp/custom"
root_mode = "custom"
root_path = "/tmp/custom"
caged = false
"#,
    );
    assert!(
        fixture.raw_contents().contains("user customizations"),
        "precondition: custom config should persist prior to --force"
    );

    let mut cmd = fixture.command();
    cmd.arg("config")
        .arg("init")
        .arg("--force")
        .assert()
        .success();

    let config = fixture.read_config();
    assert_default_config(&config);
    assert!(
        !fixture.raw_contents().contains("user customizations"),
        "force init should rewrite custom config contents"
    );
}

#[test]
fn shell_launch_without_config_prints_init_hint() {
    let fixture = ConfigInitFixture::new();

    let output = fixture
        .command()
        .arg("--no-world")
        .arg("-c")
        .arg("echo config-check")
        .output()
        .expect("failed to launch substrate shell");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let hint_found =
        stdout.contains("substrate config init") || stderr.contains("substrate config init");

    assert!(
        hint_found,
        "missing config hint not printed.\nstatus: {:#?}\nstdout: {}\nstderr: {}",
        output.status, stdout, stderr
    );
    assert!(
        !fixture.config_path().exists(),
        "config init hint should not silently create config.toml"
    );
}
