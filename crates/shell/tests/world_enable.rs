#![cfg(unix)]

#[path = "common.rs"]
mod common;

use assert_cmd::Command;
use common::{shared_tmpdir, substrate_shell_driver, temp_dir};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::PathBuf;
use tempfile::{Builder, TempDir};

const HELPER_SCRIPT: &str = r#"#!/usr/bin/env bash
set -euo pipefail

log="${SUBSTRATE_TEST_WORLD_LOG:?missing log path}"
mkdir -p "$(dirname "$log")"

echo "world-enable invoked: $*" >>"$log"
if [[ -n "${SUBSTRATE_PREFIX:-}" ]]; then
  echo "prefix=${SUBSTRATE_PREFIX}" >>"$log"
fi

if [[ -n "${SUBSTRATE_TEST_WORLD_STDOUT:-}" ]]; then
  echo "${SUBSTRATE_TEST_WORLD_STDOUT}"
fi

if [[ -n "${SUBSTRATE_TEST_WORLD_STDERR:-}" ]]; then
  echo "${SUBSTRATE_TEST_WORLD_STDERR}" >&2
fi

exit_code="${SUBSTRATE_TEST_WORLD_EXIT:-0}"
if [[ "${SUBSTRATE_TEST_SKIP_SOCKET:-0}" != "1" ]]; then
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
else
python3 <<'PY'
import os
socket_path = os.environ.get("SUBSTRATE_WORLD_SOCKET")
if socket_path and os.path.exists(socket_path):
    os.unlink(socket_path)
PY
fi

exit "$exit_code"
"#;

struct WorldEnableFixture {
    _temp: TempDir,
    _socket_temp: TempDir,
    home: PathBuf,
    prefix: PathBuf,
    substrate_home: PathBuf,
    manager_env_path: PathBuf,
    script_path: PathBuf,
    log_path: PathBuf,
    socket_path: PathBuf,
}

impl WorldEnableFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-world-enable-");
        let home = temp.path().join("home");
        let prefix = temp.path().join("prefix");
        let substrate_home = home.join(".substrate");
        let manager_env_path = substrate_home.join("manager_env.sh");
        let script_path = temp.path().join("scripts/world-enable.sh");
        let log_path = temp.path().join("logs/world-enable.log");
        let socket_temp = Builder::new()
            .prefix("substrate-world-enable-sock-")
            .tempdir_in("/tmp")
            .expect("failed to create socket tempdir");
        let socket_path = socket_temp.path().join("sock");

        fs::create_dir_all(&home).expect("failed to create fixture home");
        fs::create_dir_all(&prefix).expect("failed to create fixture prefix");
        fs::create_dir_all(&substrate_home).expect("failed to create substrate dir");
        fs::create_dir_all(script_path.parent().unwrap()).expect("failed to create script dir");
        fs::create_dir_all(log_path.parent().unwrap()).expect("failed to create log dir");
        fs::create_dir_all(socket_path.parent().unwrap()).expect("failed to create socket dir");

        fs::write(
            &manager_env_path,
            "# world enable test fixture\nexport SUBSTRATE_WORLD=disabled\nexport SUBSTRATE_WORLD_ENABLED=0\n",
        )
        .expect("failed to seed manager env");

        let fixture = Self {
            _temp: temp,
            _socket_temp: socket_temp,
            home,
            prefix,
            substrate_home,
            manager_env_path,
            script_path,
            log_path,
            socket_path,
        };
        fixture.install_helper_script();
        fixture
    }

    fn install_helper_script(&self) {
        fs::write(&self.script_path, HELPER_SCRIPT).expect("failed to write helper script");
        let mut perms = fs::metadata(&self.script_path)
            .expect("helper metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&self.script_path, perms).expect("chmod helper");
    }

    fn command(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.arg("world")
            .arg("enable")
            .env("TMPDIR", shared_tmpdir())
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env("SUBSTRATE_MANAGER_ENV", &self.manager_env_path)
            .env("SUBSTRATE_WORLD_ENABLE_SCRIPT", &self.script_path)
            .env("SUBSTRATE_WORLD_SOCKET", &self.socket_path)
            .env("SUBSTRATE_PREFIX", &self.prefix)
            .env("SUBSTRATE_WORLD", "disabled")
            .env("SUBSTRATE_WORLD_ENABLED", "0")
            .env("SUBSTRATE_TEST_WORLD_LOG", &self.log_path);
        cmd
    }

    fn command_skip_doctor(&self) -> Command {
        let mut cmd = self.command();
        cmd.env("SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR", "1");
        cmd
    }

    fn manager_env_contents(&self) -> String {
        fs::read_to_string(&self.manager_env_path).expect("manager env contents")
    }

    fn config_path(&self) -> PathBuf {
        self.substrate_home.join("config.yaml")
    }

    fn config_exists(&self) -> bool {
        self.config_path().exists()
    }

    fn write_config(&self, enabled: bool) {
        let flag = if enabled { "true" } else { "false" };
        let body = format!("install:\n  world_enabled: {flag}\n");
        fs::write(self.config_path(), body).expect("write config yaml");
    }

    fn write_invalid_config(&self) {
        fs::write(
            self.config_path(),
            "install:\n  world_enabled: \"invalid\"\n",
        )
        .expect("write invalid config");
    }

    fn read_config(&self) -> YamlValue {
        let data = fs::read_to_string(self.config_path()).expect("read config");
        serde_yaml::from_str(&data).expect("parse config yaml")
    }

    fn install_world_enabled(&self) -> bool {
        let config = self.read_config();
        let root = config.as_mapping().expect("config root mapping");
        let install = root
            .get(YamlValue::String("install".to_string()))
            .and_then(|value| value.as_mapping())
            .expect("install mapping missing");
        install
            .get(YamlValue::String("world_enabled".to_string()))
            .and_then(|value| value.as_bool())
            .expect("install.world_enabled missing")
    }

    fn log_contents(&self) -> Option<String> {
        fs::read_to_string(&self.log_path).ok()
    }

    fn log_line_count(&self) -> usize {
        self.log_contents()
            .map(|contents| contents.lines().count())
            .unwrap_or(0)
    }

    fn assert_socket_exists(&self) {
        let metadata = fs::metadata(&self.socket_path).expect("socket metadata");
        assert!(
            metadata.file_type().is_socket(),
            "expected unix socket at {}",
            self.socket_path.display()
        );
    }
}

#[test]
fn world_enable_provisions_and_sets_config_and_env_state() {
    let fixture = WorldEnableFixture::new();

    let mut cmd = fixture.command_skip_doctor();
    cmd.arg("--prefix")
        .arg(&fixture.prefix)
        .arg("--profile")
        .arg("release")
        .arg("--verbose")
        .env("SUBSTRATE_TEST_WORLD_STDOUT", "helper stdout")
        .env("SUBSTRATE_TEST_WORLD_STDERR", "helper stderr");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("world doctor"),
        "stdout missing doctor hint: {}",
        stdout
    );
    assert!(
        stdout.contains("helper stdout"),
        "stdout missing helper output: {}",
        stdout
    );
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("helper stderr"),
        "stderr missing helper output: {}",
        stderr
    );

    fixture.assert_socket_exists();
    assert!(
        fixture.install_world_enabled(),
        "install config should mark world enabled"
    );

    let env_contents = fixture.manager_env_contents();
    assert!(
        env_contents.contains("SUBSTRATE_WORLD=enabled"),
        "manager env missing SUBSTRATE_WORLD export: {}",
        env_contents
    );
    assert!(
        env_contents.contains("SUBSTRATE_WORLD_ENABLED=1"),
        "manager env missing SUBSTRATE_WORLD_ENABLED export: {}",
        env_contents
    );

    let log = fixture.log_contents().expect("helper log missing");
    assert!(log.contains("world-enable invoked"));
    assert!(log.contains("prefix="));
}

#[test]
fn world_enable_fails_when_helper_exits_non_zero() {
    let fixture = WorldEnableFixture::new();
    fixture.write_config(false);

    let mut cmd = fixture.command_skip_doctor();
    cmd.arg("--prefix")
        .arg(&fixture.prefix)
        .env("SUBSTRATE_TEST_WORLD_EXIT", "42");

    cmd.assert().failure();
    assert!(
        !fixture.install_world_enabled(),
        "install config should remain disabled when helper fails"
    );
}

#[test]
fn world_enable_fails_when_socket_missing() {
    let fixture = WorldEnableFixture::new();

    let mut cmd = fixture.command();
    cmd.arg("--prefix")
        .arg(&fixture.prefix)
        .arg("--profile")
        .arg("debug")
        .env("SUBSTRATE_TEST_SKIP_SOCKET", "1");

    cmd.assert().failure();
    assert!(!fixture.config_exists(), "config should not be created");
}

#[test]
fn world_enable_short_circuits_when_already_enabled() {
    let fixture = WorldEnableFixture::new();

    // First run succeeds and toggles config.
    fixture
        .command_skip_doctor()
        .arg("--prefix")
        .arg(&fixture.prefix)
        .assert()
        .success();
    let first_log = fixture.log_contents().unwrap();

    // Second run should short-circuit and avoid running helper.
    let assert = fixture
        .command_skip_doctor()
        .arg("--prefix")
        .arg(&fixture.prefix)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("already enabled"), "stdout: {}", stdout);
    assert_eq!(fixture.log_contents().unwrap(), first_log);
}

#[test]
fn world_enable_force_reinvokes_even_when_enabled() {
    let fixture = WorldEnableFixture::new();

    fixture
        .command_skip_doctor()
        .arg("--prefix")
        .arg(&fixture.prefix)
        .assert()
        .success();
    let first_count = fixture.log_line_count();

    fixture
        .command_skip_doctor()
        .arg("--prefix")
        .arg(&fixture.prefix)
        .arg("--force")
        .assert()
        .success();
    assert!(
        fixture.log_line_count() > first_count,
        "expected helper log to grow when forced"
    );
}

#[test]
fn world_enable_dry_run_skips_all_mutations() {
    let fixture = WorldEnableFixture::new();
    let initial_env = fixture.manager_env_contents();

    fixture
        .command_skip_doctor()
        .arg("--prefix")
        .arg(&fixture.prefix)
        .arg("--dry-run")
        .assert()
        .success();

    assert!(!fixture.config_exists(), "dry run should not create config");
    assert!(fixture.log_contents().is_none(), "helper should not run");
    assert_eq!(fixture.manager_env_contents(), initial_env);
}

#[test]
fn world_enable_recovers_from_invalid_config_file() {
    let fixture = WorldEnableFixture::new();
    fixture.write_invalid_config();

    fixture
        .command_skip_doctor()
        .arg("--prefix")
        .arg(&fixture.prefix)
        .assert()
        .success();

    assert!(fixture.install_world_enabled());
}
