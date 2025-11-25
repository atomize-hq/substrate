#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tempfile::TempDir;
use toml::Value as TomlValue;

const REDACTED_PLACEHOLDER: &str = "*** redacted ***";

struct ConfigShowFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
}

impl ConfigShowFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-config-show-");
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

    fn write_config(&self, contents: &str) {
        if let Some(parent) = self.config_path().parent() {
            fs::create_dir_all(parent).expect("failed to create config directory");
        }
        fs::write(self.config_path(), contents).expect("failed to seed config file");
    }

    fn read_config_value(&self) -> TomlValue {
        let body = fs::read_to_string(self.config_path()).expect("config to exist");
        toml::from_str(&body).expect("config to parse as TOML")
    }

    fn show_output(&self, extra_args: &[&str]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.arg("config").arg("show");
        for arg in extra_args {
            cmd.arg(arg);
        }
        cmd.output()
            .expect("failed to execute substrate config show")
    }
}

#[test]
fn config_show_prints_current_config_as_toml() {
    if !ensure_config_show_available() {
        return;
    }

    let fixture = ConfigShowFixture::new();
    fixture.write_config(
        r#"[install]
world_enabled = true

[world]
anchor_mode = "project"
anchor_path = ""
root_mode = "project"
root_path = ""
caged = true
"#,
    );

    let output = fixture.show_output(&[]);
    assert!(
        output.status.success(),
        "config show should succeed when config exists: {:?}",
        output
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cli_value: TomlValue =
        toml::from_str(&stdout).expect("config show output to parse as TOML");
    assert_eq!(
        cli_value,
        fixture.read_config_value(),
        "config show TOML should match file contents"
    );
}

#[test]
fn config_show_supports_json_output() {
    if !ensure_config_show_available() {
        return;
    }

    let fixture = ConfigShowFixture::new();
    fixture.write_config(
        r#"[install]
world_enabled = false

[world]
anchor_mode = "follow-cwd"
anchor_path = "/tmp/example"
root_mode = "follow-cwd"
root_path = "/tmp/example"
caged = false
"#,
    );

    let expected = fixture.read_config_value();
    let output = fixture.show_output(&["--json"]);
    assert!(
        output.status.success(),
        "config show --json should succeed: {:?}",
        output
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cli_json: JsonValue =
        serde_json::from_str(&stdout).expect("config show output to parse as JSON");
    let expected_json: JsonValue = expected
        .try_into()
        .expect("written config to convert into JSON value");
    assert_eq!(
        cli_json, expected_json,
        "json payload should mirror the TOML data"
    );
}

#[test]
fn config_show_reports_missing_config_with_hint() {
    if !ensure_config_show_available() {
        return;
    }

    let fixture = ConfigShowFixture::new();
    let output = fixture.show_output(&[]);
    assert!(
        !output.status.success(),
        "config show should exit non-zero when config missing: {:?}",
        output
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("substrate config init") || stderr.contains("substrate config init"),
        "config show missing-config hint absent\nstdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn config_show_redacts_sensitive_paths_in_outputs() {
    if !ensure_config_show_available() {
        return;
    }

    let fixture = ConfigShowFixture::new();
    fixture.write_config(
        r#"[install]
world_enabled = true
api_token = "install-secret"
auth_token = "install-another"

[world]
anchor_mode = "project"
root_mode = "project"
api_token = "world-secret"
"#,
    );

    let output = fixture.show_output(&[]);
    assert!(
        output.status.success(),
        "config show should succeed for sensitive configs: {:?}",
        output
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cli_value: TomlValue =
        toml::from_str(&stdout).expect("config show TOML output should parse");

    let install_table = cli_value
        .get("install")
        .and_then(|value| value.as_table())
        .expect("install table present");
    assert_eq!(
        install_table
            .get("api_token")
            .and_then(|value| value.as_str()),
        Some(REDACTED_PLACEHOLDER),
        "install.api_token should be redacted"
    );
    assert_eq!(
        install_table
            .get("auth_token")
            .and_then(|value| value.as_str()),
        Some(REDACTED_PLACEHOLDER),
        "install.auth_token should be redacted"
    );

    let world_table = cli_value
        .get("world")
        .and_then(|value| value.as_table())
        .expect("world table present");
    assert_eq!(
        world_table
            .get("api_token")
            .and_then(|value| value.as_str()),
        Some(REDACTED_PLACEHOLDER),
        "world.api_token should be redacted"
    );

    let json_output = fixture.show_output(&["--json"]);
    assert!(
        json_output.status.success(),
        "config show --json should succeed for sensitive configs: {:?}",
        json_output
    );
    let json_body = String::from_utf8_lossy(&json_output.stdout);
    let cli_json: JsonValue =
        serde_json::from_str(&json_body).expect("config show JSON should parse");

    assert_eq!(
        cli_json
            .pointer("/install/api_token")
            .and_then(|value| value.as_str()),
        Some(REDACTED_PLACEHOLDER),
        "JSON config should redact install.api_token"
    );
    assert_eq!(
        cli_json
            .pointer("/install/auth_token")
            .and_then(|value| value.as_str()),
        Some(REDACTED_PLACEHOLDER),
        "JSON config should redact install.auth_token"
    );
    assert_eq!(
        cli_json
            .pointer("/world/api_token")
            .and_then(|value| value.as_str()),
        Some(REDACTED_PLACEHOLDER),
        "JSON config should redact world.api_token"
    );

    let stored = fixture.read_config_value();
    let stored_install = stored
        .get("install")
        .and_then(|value| value.as_table())
        .expect("stored install table");
    assert_eq!(
        stored_install
            .get("api_token")
            .and_then(|value| value.as_str()),
        Some("install-secret"),
        "redaction should not mutate the stored config"
    );
    assert_eq!(
        stored_install
            .get("auth_token")
            .and_then(|value| value.as_str()),
        Some("install-another"),
        "redaction should not mutate install.auth_token on disk"
    );
    let stored_world = stored
        .get("world")
        .and_then(|value| value.as_table())
        .expect("stored world table");
    assert_eq!(
        stored_world
            .get("api_token")
            .and_then(|value| value.as_str()),
        Some("world-secret"),
        "redaction should not mutate world.api_token on disk"
    );
}

fn ensure_config_show_available() -> bool {
    if config_show_supported() {
        true
    } else {
        eprintln!("skipping config show tests until the subcommand is implemented");
        false
    }
}

fn config_show_supported() -> bool {
    static SUPPORTED: OnceLock<bool> = OnceLock::new();
    *SUPPORTED.get_or_init(|| {
        let mut cmd = substrate_shell_driver();
        match cmd.arg("config").arg("--help").output() {
            Ok(output) => {
                if !output.status.success() {
                    return false;
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("\n  show")
            }
            Err(_) => false,
        }
    })
}
