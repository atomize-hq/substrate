#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tempfile::TempDir;

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
        self.substrate_home.join("config.yaml")
    }

    fn write_config(&self, contents: &str) {
        if let Some(parent) = self.config_path().parent() {
            fs::create_dir_all(parent).expect("failed to create config directory");
        }
        fs::write(self.config_path(), contents).expect("failed to seed config file");
    }

    fn legacy_config_path(&self) -> PathBuf {
        self.substrate_home.join("config.toml")
    }

    fn read_config_value(&self) -> YamlValue {
        let body = fs::read_to_string(self.config_path()).expect("config to exist");
        serde_yaml::from_str(&body).expect("config to parse as YAML")
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
fn config_show_prints_current_config_as_yaml() {
    if !ensure_config_show_available() {
        return;
    }

    let fixture = ConfigShowFixture::new();
    fixture.write_config(
        "install:\n  world_enabled: true\nworld:\n  anchor_mode: project\n  anchor_path: \"\"\n  root_mode: project\n  root_path: \"\"\n  caged: true\n",
    );

    let output = fixture.show_output(&[]);
    assert!(
        output.status.success(),
        "config show should succeed when config exists: {:?}",
        output
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cli_value: YamlValue =
        serde_yaml::from_str(&stdout).expect("config show output to parse as YAML");
    assert_eq!(
        cli_value,
        fixture.read_config_value(),
        "config show YAML should match file contents"
    );
}

#[test]
fn config_show_supports_json_output() {
    if !ensure_config_show_available() {
        return;
    }

    let fixture = ConfigShowFixture::new();
    fixture.write_config(
        "install:\n  world_enabled: false\nworld:\n  anchor_mode: follow-cwd\n  anchor_path: /tmp/example\n  root_mode: follow-cwd\n  root_path: /tmp/example\n  caged: false\n",
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
    let expected_json: JsonValue = serde_json::to_value(&expected).expect("yaml config to JSON");
    assert_eq!(
        cli_json, expected_json,
        "json payload should mirror the YAML data"
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
        "install:\n  world_enabled: true\n  api_token: install-secret\n  auth_token: install-another\nworld:\n  anchor_mode: project\n  root_mode: project\n  api_token: world-secret\n",
    );

    let output = fixture.show_output(&[]);
    assert!(
        output.status.success(),
        "config show should succeed for sensitive configs: {:?}",
        output
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cli_value: YamlValue =
        serde_yaml::from_str(&stdout).expect("config show YAML output should parse");

    let root = cli_value.as_mapping().expect("yaml root mapping");
    let install_table = root
        .get(YamlValue::String("install".to_string()))
        .and_then(|value| value.as_mapping())
        .expect("install mapping present");
    assert_eq!(
        install_table
            .get(YamlValue::String("api_token".to_string()))
            .and_then(|value| value.as_str()),
        Some(REDACTED_PLACEHOLDER),
        "install.api_token should be redacted"
    );
    assert_eq!(
        install_table
            .get(YamlValue::String("auth_token".to_string()))
            .and_then(|value| value.as_str()),
        Some(REDACTED_PLACEHOLDER),
        "install.auth_token should be redacted"
    );

    let world_table = root
        .get(YamlValue::String("world".to_string()))
        .and_then(|value| value.as_mapping())
        .expect("world mapping present");
    assert_eq!(
        world_table
            .get(YamlValue::String("api_token".to_string()))
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
    let stored_root = stored.as_mapping().expect("stored yaml root mapping");
    let stored_install = stored_root
        .get(YamlValue::String("install".to_string()))
        .and_then(|value| value.as_mapping())
        .expect("stored install mapping");
    assert_eq!(
        stored_install
            .get(YamlValue::String("api_token".to_string()))
            .and_then(|value| value.as_str()),
        Some("install-secret"),
        "redaction should not mutate the stored config"
    );
    assert_eq!(
        stored_install
            .get(YamlValue::String("auth_token".to_string()))
            .and_then(|value| value.as_str()),
        Some("install-another"),
        "redaction should not mutate install.auth_token on disk"
    );
    let stored_world = stored_root
        .get(YamlValue::String("world".to_string()))
        .and_then(|value| value.as_mapping())
        .expect("stored world mapping");
    assert_eq!(
        stored_world
            .get(YamlValue::String("api_token".to_string()))
            .and_then(|value| value.as_str()),
        Some("world-secret"),
        "redaction should not mutate world.api_token on disk"
    );
}

#[test]
fn config_show_refuses_legacy_toml() {
    if !ensure_config_show_available() {
        return;
    }

    let fixture = ConfigShowFixture::new();
    fixture.write_config("install:\n  world_enabled: true\n");
    fs::write(
        fixture.legacy_config_path(),
        "[install]\nworld_enabled = true\n",
    )
    .expect("write legacy config.toml");

    let output = fixture.show_output(&[]);
    assert!(
        !output.status.success(),
        "config show should fail when legacy toml exists: {:?}",
        output
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported legacy TOML config detected"),
        "stderr missing legacy TOML message: {stderr}"
    );
    assert!(
        stderr.contains(&fixture.legacy_config_path().display().to_string()),
        "stderr missing legacy path: {stderr}"
    );
    assert!(
        stderr.contains(&fixture.config_path().display().to_string()),
        "stderr missing yaml path: {stderr}"
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
