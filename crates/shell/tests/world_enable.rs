#![cfg(unix)]

#[path = "common.rs"]
mod common;

use assert_cmd::Command;
use common::{shared_tmpdir, substrate_shell_driver};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::os::unix::fs::{symlink, FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};
use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};

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

fn current_install_context(prefix: &Path) -> (String, String, String, String) {
    let uid = unsafe { libc::geteuid() };
    let account_output = std::process::Command::new("id")
        .args(["-nu", &uid.to_string()])
        .output()
        .expect("resolve current Unix account");
    assert!(account_output.status.success());
    let account = String::from_utf8(account_output.stdout)
        .expect("Unix account is UTF-8")
        .trim()
        .to_string();
    let context = InstallBootstrapContextV1::new_unix(
        prefix.to_str().expect("install prefix is UTF-8"),
        &account,
        uid,
    )
    .expect("construct install bootstrap context");
    let carrier = InstallBootstrapContextCarrierV1::from_context(context)
        .expect("commit install bootstrap context");
    (
        carrier.encode().expect("encode install bootstrap context"),
        carrier.host_context_commitment,
        account,
        uid.to_string(),
    )
}

fn project_install_context_environment(cmd: &mut Command, prefix: &Path) -> String {
    let (carrier, commitment, account, uid) = current_install_context(prefix);
    cmd.env("SUBSTRATE_HOME", prefix)
        .env("SUBSTRATE_ROOT", prefix)
        .env("SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT", commitment)
        .env("SUBSTRATE_INSTALL_PRIMARY_USER", account)
        .env("SUBSTRATE_INSTALL_PRIMARY_UID", uid)
        .env("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1", &carrier);
    carrier
}

fn add_internal_install_context(cmd: &mut Command, prefix: &Path) {
    let carrier = project_install_context_environment(cmd, prefix);
    cmd.arg("--install-bootstrap-context-v1").arg(carrier);
}

fn install_runtime_scripts_at(prefix: &Path) {
    let helper_path = prefix.join("scripts/substrate/world-enable.sh");
    fs::create_dir_all(helper_path.parent().expect("helper parent")).expect("create helper parent");
    fs::write(&helper_path, HELPER_SCRIPT).expect("write world-enable helper");
    let mut perms = fs::metadata(&helper_path)
        .expect("helper metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&helper_path, perms).expect("chmod world-enable helper");
}

struct WorldEnableFixture {
    _temp: TempDir,
    _socket_temp: TempDir,
    home: PathBuf,
    legacy_prefix: PathBuf,
    substrate_home: PathBuf,
    env_sh_path: PathBuf,
    script_path: PathBuf,
    log_path: PathBuf,
    socket_path: PathBuf,
}

impl WorldEnableFixture {
    fn new() -> Self {
        let fixture_root = fs::canonicalize(shared_tmpdir())
            .expect("canonicalize secured world-enable fixture root");
        let temp = Builder::new()
            .prefix("substrate-world-enable-")
            .tempdir_in(fixture_root)
            .expect("failed to allocate secured world-enable fixture");
        let home = temp.path().join("home");
        let legacy_prefix = temp.path().join("legacy-prefix");
        let substrate_home = temp.path().join("substrate-home");
        let env_sh_path = substrate_home.join("env.sh");
        let script_path = temp.path().join("scripts/world-enable.sh");
        let log_path = temp.path().join("logs/world-enable.log");
        let socket_temp = Builder::new()
            .prefix("substrate-world-enable-sock-")
            .tempdir_in("/tmp")
            .expect("failed to create socket tempdir");
        let socket_path = socket_temp.path().join("sock");

        fs::create_dir_all(&home).expect("failed to create fixture home");
        fs::create_dir_all(&legacy_prefix).expect("failed to create legacy prefix dir");
        fs::create_dir_all(&substrate_home).expect("failed to create substrate dir");
        fs::create_dir_all(script_path.parent().unwrap()).expect("failed to create script dir");
        fs::create_dir_all(log_path.parent().unwrap()).expect("failed to create log dir");
        fs::create_dir_all(socket_path.parent().unwrap()).expect("failed to create socket dir");

        let fixture = Self {
            _temp: temp,
            _socket_temp: socket_temp,
            home,
            legacy_prefix,
            substrate_home,
            env_sh_path,
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

    fn command_with_override(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.arg("world")
            .arg("enable")
            .arg("--home")
            .arg(&self.substrate_home)
            .env("TMPDIR", shared_tmpdir())
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_WORLD_ENABLE_SCRIPT", &self.script_path)
            .env("SUBSTRATE_WORLD_SOCKET", &self.socket_path)
            // Legacy env vars removed by ADR-0003 must not affect world enable.
            .env("SUBSTRATE_PREFIX", &self.legacy_prefix)
            .env("SUBSTRATE_TEST_WORLD_LOG", &self.log_path);
        cmd
    }

    fn command(&self) -> Command {
        self.command_with_override()
    }

    fn command_without_override(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.arg("world")
            .arg("enable")
            .arg("--home")
            .arg(&self.substrate_home)
            .env("TMPDIR", shared_tmpdir())
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_WORLD_SOCKET", &self.socket_path)
            .env("SUBSTRATE_PREFIX", &self.legacy_prefix)
            .env("SUBSTRATE_TEST_WORLD_LOG", &self.log_path);
        cmd
    }

    fn command_skip_doctor(&self) -> Command {
        let mut cmd = self.command();
        cmd.env("SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR", "1");
        cmd
    }

    fn command_skip_doctor_without_override(&self) -> Command {
        let mut cmd = self.command_without_override();
        cmd.env("SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR", "1");
        cmd
    }

    fn config_path(&self) -> PathBuf {
        self.substrate_home.join("config.yaml")
    }

    fn config_exists(&self) -> bool {
        self.config_path().exists()
    }

    fn env_sh_exists(&self) -> bool {
        self.env_sh_path.exists()
    }

    fn env_sh_contents(&self) -> String {
        fs::read_to_string(&self.env_sh_path).expect("env.sh contents")
    }

    fn write_config(&self, enabled: bool) {
        let flag = if enabled { "true" } else { "false" };
        let body = format!(
            "world:\n  enabled: {flag}\n  anchor_mode: workspace\n  anchor_path: \"\"\n  caged: true\n\npolicy:\n  mode: observe\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n"
        );
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
        let world = root
            .get(YamlValue::String("world".to_string()))
            .and_then(|value| value.as_mapping())
            .expect("world mapping missing");
        world
            .get(YamlValue::String("enabled".to_string()))
            .and_then(|value| value.as_bool())
            .expect("world.enabled missing")
    }

    fn log_contents(&self) -> Option<String> {
        fs::read_to_string(&self.log_path).ok()
    }

    fn assert_socket_exists(&self) {
        let metadata = fs::metadata(&self.socket_path).expect("socket metadata");
        assert!(
            metadata.file_type().is_socket(),
            "expected unix socket at {}",
            self.socket_path.display()
        );
    }

    fn install_prefix_runtime_bundle(&self) {
        self.install_prefix_runtime_scripts();
    }

    fn install_prefix_runtime_scripts(&self) {
        let helper_path = self
            .substrate_home
            .join("scripts/substrate/world-enable.sh");
        fs::create_dir_all(helper_path.parent().expect("helper parent"))
            .expect("create prefix helper dir");
        fs::write(&helper_path, HELPER_SCRIPT).expect("write prefix helper");
        let mut perms = fs::metadata(&helper_path)
            .expect("prefix helper metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&helper_path, perms).expect("chmod prefix helper");

        let install_helper = self
            .substrate_home
            .join("scripts/substrate/install-substrate.sh");
        fs::write(&install_helper, "#!/usr/bin/env bash\nexit 0\n").expect("write install helper");
        let mut perms = fs::metadata(&install_helper)
            .expect("install helper metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_helper, perms).expect("chmod install helper");
    }

    fn install_version_dir_binary(&self) -> PathBuf {
        let version_dir = self._temp.path().join("version-dir");
        let version_bin = version_dir.join("bin").join("substrate");
        fs::create_dir_all(version_bin.parent().expect("version bin parent"))
            .expect("create version dir bin");
        fs::write(&version_bin, "#!/usr/bin/env bash\nexit 0\n").expect("write version substrate");
        let mut perms = fs::metadata(&version_bin)
            .expect("version substrate metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&version_bin, perms).expect("chmod version substrate");

        let bin_dir = self.substrate_home.join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let substrate_bin = bin_dir.join("substrate");
        symlink(&version_bin, &substrate_bin).expect("symlink substrate binary");

        version_dir
    }

    fn install_accepted_staged_world_service(&self, version_dir: &Path, relative_path: &str) {
        let path = version_dir.join(relative_path);
        fs::create_dir_all(path.parent().expect("world-service parent"))
            .expect("create staged world-service dir");
        fs::write(&path, "#!/usr/bin/env bash\nexit 0\n").expect("write staged world-service");
        let mut perms = fs::metadata(&path)
            .expect("staged world-service metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("chmod staged world-service");
    }
}

#[test]
fn world_enable_provisions_and_sets_config_and_env_state() {
    let fixture = WorldEnableFixture::new();

    let mut cmd = fixture.command_skip_doctor();
    cmd.arg("--profile")
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
        "global config should mark world enabled"
    );

    assert!(
        fixture.env_sh_exists(),
        "env.sh should be written under --home"
    );
    let env_contents = fixture.env_sh_contents();
    assert!(
        env_contents.starts_with("#!/usr/bin/env bash\n"),
        "env.sh missing shebang: {env_contents}"
    );
    assert!(
        env_contents.contains("export SUBSTRATE_HOME="),
        "env.sh missing SUBSTRATE_HOME export: {env_contents}"
    );
    assert!(
        env_contents.contains(&fixture.substrate_home.display().to_string()),
        "env.sh should include substrate_home path: {env_contents}"
    );
    assert!(
        env_contents.contains("export SUBSTRATE_WORLD='enabled'"),
        "env.sh missing SUBSTRATE_WORLD enabled export: {env_contents}"
    );

    let log = fixture.log_contents().expect("helper log missing");
    assert!(log.contains("world-enable invoked"));

    assert!(
        !fixture.home.join(".substrate/config.yaml").exists(),
        "world enable should honor --home for state writes"
    );
    assert!(
        !fixture.legacy_prefix.join("config.yaml").exists(),
        "legacy SUBSTRATE_PREFIX must not affect state writes"
    );
}

#[test]
fn world_enable_rejects_prefix_flag() {
    let fixture = WorldEnableFixture::new();

    let mut cmd = substrate_shell_driver();
    cmd.arg("world")
        .arg("enable")
        .arg("--prefix")
        .arg(&fixture.legacy_prefix)
        .env("TMPDIR", shared_tmpdir())
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home);

    let output = cmd
        .output()
        .expect("failed to run substrate world enable --prefix");
    assert_eq!(
        output.status.code(),
        Some(2),
        "--prefix should be rejected by CLI: {output:?}"
    );
    assert!(
        !fixture.config_exists(),
        "world enable should not write config when args are invalid"
    );
    assert!(
        !fixture.env_sh_exists(),
        "world enable should not write env.sh when args are invalid"
    );
    assert!(
        fixture.log_contents().is_none(),
        "helper should not run when args are invalid"
    );
}

#[test]
fn world_enable_fails_when_helper_exits_non_zero() {
    let fixture = WorldEnableFixture::new();
    fixture.write_config(false);

    let mut cmd = fixture.command_skip_doctor();
    cmd.env("SUBSTRATE_TEST_WORLD_EXIT", "42");

    cmd.assert().failure();
    assert!(
        !fixture.install_world_enabled(),
        "global config should remain disabled when helper fails"
    );
    assert!(
        !fixture.env_sh_exists(),
        "env.sh should not be written when helper fails"
    );
}

#[test]
fn world_enable_fails_when_socket_missing() {
    let fixture = WorldEnableFixture::new();

    let mut cmd = fixture.command();
    cmd.arg("--profile")
        .arg("debug")
        .env("SUBSTRATE_TEST_SKIP_SOCKET", "1");

    cmd.assert().failure();
    assert!(!fixture.config_exists(), "config should not be created");
    assert!(!fixture.env_sh_exists(), "env.sh should not be created");
}

#[test]
fn world_enable_dry_run_skips_all_mutations() {
    let fixture = WorldEnableFixture::new();
    let initial_env_exists = fixture.env_sh_exists();

    fixture
        .command_skip_doctor()
        .arg("--dry-run")
        .assert()
        .success();

    assert!(!fixture.config_exists(), "dry run should not create config");
    assert_eq!(
        fixture.env_sh_exists(),
        initial_env_exists,
        "dry run should not create env.sh"
    );
    assert!(fixture.log_contents().is_none(), "helper should not run");
}

#[test]
fn world_enable_nested_home_selects_context_before_dispatch_under_ambient_b() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();

    let mut cmd = fixture.command_skip_doctor_without_override();
    project_install_context_environment(&mut cmd, &fixture.legacy_prefix);
    let assert = cmd.arg("--dry-run").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    assert!(
        stdout.contains(&fixture.substrate_home.display().to_string()),
        "dry-run should use nested --home as A: {stdout}"
    );
    assert!(
        !stdout.contains(&fixture.legacy_prefix.display().to_string()),
        "ambient B must not retarget nested --home A: {stdout}"
    );
    assert!(!fixture.legacy_prefix.join("deps").exists());
    assert!(!fixture.legacy_prefix.join("config.yaml").exists());
}

#[test]
fn world_enable_global_install_prefix_reaches_runner_without_nested_home() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();

    let mut cmd = substrate_shell_driver();
    cmd.arg("--install-prefix")
        .arg(&fixture.substrate_home)
        .arg("world")
        .arg("enable")
        .arg("--dry-run")
        .env("TMPDIR", shared_tmpdir())
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home)
        .env("SUBSTRATE_WORLD_SOCKET", &fixture.socket_path);

    let assert = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(&fixture.substrate_home.display().to_string()),
        "global --install-prefix should reach world enable: {stdout}"
    );
}

#[test]
fn world_enable_equal_global_and_nested_selectors_join_one_context() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();

    let mut cmd = fixture.command_skip_doctor_without_override();
    let assert = cmd
        .arg("--install-prefix")
        .arg(format!("{}/", fixture.substrate_home.display()))
        .arg("--dry-run")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains(&fixture.substrate_home.display().to_string()));
}

#[test]
fn world_enable_conflicting_public_selectors_fail_before_mutation() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();
    install_runtime_scripts_at(&fixture.legacy_prefix);

    let mut cmd = fixture.command_skip_doctor_without_override();
    let output = cmd
        .arg("--install-prefix")
        .arg(&fixture.legacy_prefix)
        .arg("--dry-run")
        .output()
        .expect("run world enable with conflicting selectors");

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(!fixture.substrate_home.join("deps").exists());
    assert!(!fixture.legacy_prefix.join("deps").exists());
    assert!(!fixture.config_exists());
    assert!(!fixture.env_sh_exists());
    assert!(fixture.log_contents().is_none());
}

#[test]
fn world_enable_internal_carrier_and_matching_selector_reach_runner() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();

    let mut cmd = fixture.command_skip_doctor_without_override();
    add_internal_install_context(&mut cmd, &fixture.substrate_home);
    cmd.arg("--dry-run").assert().success();
}

#[test]
fn world_enable_internal_carrier_conflict_fails_before_mutation() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();
    install_runtime_scripts_at(&fixture.legacy_prefix);

    let mut cmd = substrate_shell_driver();
    cmd.arg("world")
        .arg("enable")
        .arg("--home")
        .arg(&fixture.legacy_prefix)
        .arg("--dry-run")
        .env("TMPDIR", shared_tmpdir())
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home);
    add_internal_install_context(&mut cmd, &fixture.substrate_home);

    let output = cmd
        .output()
        .expect("run world enable with conflicting internal selector");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(!fixture.substrate_home.join("deps").exists());
    assert!(!fixture.legacy_prefix.join("deps").exists());
    assert!(!fixture.config_exists());
    assert!(!fixture.env_sh_exists());
    assert!(fixture.log_contents().is_none());
}

#[test]
fn world_enable_environment_only_context_cannot_replace_typed_authority() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();

    let mut cmd = substrate_shell_driver();
    cmd.arg("world")
        .arg("enable")
        .arg("--dry-run")
        .env("TMPDIR", shared_tmpdir())
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home);
    project_install_context_environment(&mut cmd, &fixture.substrate_home);

    let output = cmd
        .output()
        .expect("run repository binary with environment-only context");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(!fixture.substrate_home.join("deps").exists());
    assert!(!fixture.config_exists());
    assert!(!fixture.env_sh_exists());
}

#[test]
fn non_world_version_json_branch_remains_non_mutating() {
    let fixture = WorldEnableFixture::new();
    let untouched = fixture._temp.path().join("non-world-prefix");

    substrate_shell_driver()
        .arg("--install-prefix")
        .arg(&untouched)
        .arg("--version-json")
        .assert()
        .success();

    assert!(!untouched.exists());
}

#[test]
fn world_enable_prefers_prefix_runtime_bundle_without_override() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_bundle();

    let mut cmd = fixture.command_skip_doctor_without_override();
    cmd.arg("--dry-run");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(
            &fixture
                .substrate_home
                .join("scripts/substrate/world-enable.sh")
                .display()
                .to_string()
        ),
        "dry-run should resolve helper from prefix runtime bundle: {stdout}"
    );
    assert!(
        !stdout.contains("/target/scripts/substrate/world-enable.sh"),
        "dry-run should not resolve helper from target scripts: {stdout}"
    );
}

#[test]
fn world_enable_uses_prefix_runtime_bundle_when_version_binary_is_missing() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();

    let output = fixture
        .command_skip_doctor_without_override()
        .arg("--dry-run")
        .output()
        .expect("failed to run substrate world enable without version binary");

    assert!(
        output.status.success(),
        "prefix helper should remain usable without an inferred version dir: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(
            &fixture
                .substrate_home
                .join("scripts/substrate/world-enable.sh")
                .display()
                .to_string()
        ),
        "dry-run should resolve helper from the prefix runtime bundle: {stdout}"
    );
}

#[test]
fn world_enable_exits_3_when_accepted_staged_world_service_missing() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();
    let _version_dir = fixture.install_version_dir_binary();

    let output = fixture
        .command_without_override()
        .output()
        .expect("failed to run substrate world enable with missing staged world-service");

    assert!(
        !output.status.success(),
        "missing accepted staged world-service should fail closed: {output:?}"
    );
    assert_eq!(
        output.status.code(),
        Some(3),
        "missing accepted staged world-service should exit 3: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        fixture.log_contents().is_none(),
        "preflight failure should not write a helper log"
    );
    assert!(
        !fixture.config_exists(),
        "preflight failure should not create config"
    );
    assert!(
        !fixture.env_sh_exists(),
        "preflight failure should not create env.sh"
    );
    assert!(
        stderr.contains("accepted staged world-service artifact missing"),
        "stderr should identify the missing staged artifact: {stderr}"
    );
    assert!(
        stderr.contains("bin/world-service"),
        "stderr should list the root staged path suffix: {stderr}"
    );
    assert!(
        stderr.contains("bin/linux/world-service"),
        "stderr should list the linux fallback staged path suffix: {stderr}"
    );
    assert!(
        stderr.contains("scripts/substrate/dev-install-substrate.sh --no-world"),
        "stderr should point operators at the staging remediation: {stderr}"
    );
    assert!(
        stderr.contains("cargo build -p world-service"),
        "stderr should point operators at rebuilding world-service: {stderr}"
    );
}

#[test]
fn world_enable_dry_run_exits_3_when_accepted_staged_world_service_missing() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();
    let _version_dir = fixture.install_version_dir_binary();

    let output = fixture
        .command_without_override()
        .arg("--dry-run")
        .output()
        .expect("failed to run substrate world enable dry-run with missing staged world-service");

    assert!(
        !output.status.success(),
        "missing accepted staged world-service should fail closed in dry-run: {output:?}"
    );
    assert_eq!(
        output.status.code(),
        Some(3),
        "missing accepted staged world-service should exit 3 in dry-run: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("accepted staged world-service artifact missing"),
        "stderr should identify the missing staged artifact: {stderr}"
    );
    assert!(
        stderr.contains("bin/world-service"),
        "stderr should list the root staged path suffix: {stderr}"
    );
    assert!(
        stderr.contains("bin/linux/world-service"),
        "stderr should list the linux fallback staged path suffix: {stderr}"
    );
    assert!(
        stderr.contains("scripts/substrate/dev-install-substrate.sh --no-world"),
        "stderr should point operators at the staging remediation: {stderr}"
    );
    assert!(
        stderr.contains("cargo build -p world-service"),
        "stderr should point operators at rebuilding world-service: {stderr}"
    );
    assert!(
        fixture.log_contents().is_none(),
        "dry-run preflight failure should not write a helper log"
    );
    assert!(
        !fixture.config_exists(),
        "dry-run preflight failure should not create config"
    );
    assert!(
        !fixture.env_sh_exists(),
        "dry-run preflight failure should not create env.sh"
    );
}

#[test]
fn world_enable_dry_run_accepts_root_staged_world_service() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();
    let version_dir = fixture.install_version_dir_binary();
    fixture.install_accepted_staged_world_service(&version_dir, "bin/world-service");

    let output = fixture
        .command_without_override()
        .arg("--dry-run")
        .output()
        .expect("failed to run substrate world enable dry-run with root staged world-service");

    assert!(
        output.status.success(),
        "root staged world-service should satisfy dry-run preflight: {output:?}"
    );
    assert!(
        fixture.log_contents().is_none(),
        "dry-run should not create a helper log"
    );
    assert!(!fixture.config_exists(), "dry-run should not create config");
    assert!(!fixture.env_sh_exists(), "dry-run should not create env.sh");
}

#[test]
fn world_enable_dry_run_accepts_linux_fallback_staged_world_service() {
    let fixture = WorldEnableFixture::new();
    fixture.install_prefix_runtime_scripts();
    let version_dir = fixture.install_version_dir_binary();
    fixture.install_accepted_staged_world_service(&version_dir, "bin/linux/world-service");

    let output = fixture
        .command_without_override()
        .arg("--dry-run")
        .output()
        .expect("failed to run substrate world enable dry-run with linux staged world-service");

    assert!(
        output.status.success(),
        "linux fallback staged world-service should satisfy dry-run preflight: {output:?}"
    );
    assert!(
        fixture.log_contents().is_none(),
        "dry-run should not create a helper log"
    );
    assert!(!fixture.config_exists(), "dry-run should not create config");
    assert!(!fixture.env_sh_exists(), "dry-run should not create env.sh");
}

#[test]
fn world_enable_recovers_from_invalid_config_file() {
    let fixture = WorldEnableFixture::new();
    fixture.write_invalid_config();

    fixture.command_skip_doctor().assert().success();

    assert!(fixture.install_world_enabled());
    assert!(
        fixture.env_sh_exists(),
        "env.sh should be written after recovery"
    );
}
