#![cfg(unix)]

mod support;

use assert_cmd::Command;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

struct ShellEnvFixture {
    _temp: TempDir,
    home: PathBuf,
}

impl ShellEnvFixture {
    fn new() -> Self {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::geteuid() })));
        let temp = Builder::new()
            .prefix("substrate-status-r2-")
            .tempdir_in(safe_parent)
            .expect("allocate secure shim status fixture root");
        let home = temp.path().join("home");
        let prefix = home.join(".substrate");
        fs::create_dir_all(prefix.join("shims")).expect("create shim status prefix");
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700))
            .expect("secure shim status home");
        fs::set_permissions(&prefix, fs::Permissions::from_mode(0o700))
            .expect("secure shim status prefix");
        Self { _temp: temp, home }
    }

    fn home(&self) -> &Path {
        &self.home
    }
}

fn seed_shims(home: &Path) {
    let shims_dir = home.join(".substrate").join("shims");
    fs::create_dir_all(&shims_dir).expect("create shims dir");
    let version_file = shims_dir.join(".version");
    let payload = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "deployed_at": {
            "secs_since_epoch": 1_700_000_000u64,
            "nanos_since_epoch": 0u32
        },
        "commands": ["git"]
    });
    fs::write(
        &version_file,
        serde_json::to_string_pretty(&payload).unwrap(),
    )
    .expect("write version file");
    let shim = shims_dir.join("git");
    fs::write(&shim, "#!/usr/bin/env bash\nexit 0\n").expect("write shim");
    let mut perms = fs::metadata(&shim).expect("shim metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&shim, perms).expect("set shim perms");
}

fn write_read_only_policy(home: &Path) {
    let yaml = r#"
world_fs:
  host_visible: true
  write:
    enabled: false
  fail_closed:
    routing: true
"#;
    let policy_path = home.join(".substrate").join("policy.yaml");
    fs::create_dir_all(policy_path.parent().unwrap()).expect("policy dir");
    fs::write(policy_path, yaml).expect("write policy");
}

fn base_command(fixture: &ShellEnvFixture) -> Command {
    let prefix = fixture.home().join(".substrate");
    let mut cmd = support::get_substrate_binary();
    cmd.env("HOME", fixture.home())
        .env("USERPROFILE", fixture.home())
        .env("SHELL", "/bin/bash")
        .env("SUBSTRATE_HOME", &prefix)
        .env("SUBSTRATE_ROOT", &prefix)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .env_remove("SUBSTRATE_NO_SHIMS")
        .env_remove("SUBSTRATE_SHIM_PATH")
        .env_remove("SUBSTRATE_SHIM_ORIGINAL_PATH")
        .env_remove("SUBSTRATE_SHIM_DEPLOY_DIR")
        .env_remove("SHIM_ORIGINAL_PATH")
        .env_remove("PATH_BEFORE_SUBSTRATE_SHIM")
        .current_dir(fixture.home())
        .arg("--install-prefix")
        .arg(prefix);
    cmd
}

#[test]
fn shim_status_json_reports_world_fs_mode() {
    let fixture = ShellEnvFixture::new();
    seed_shims(fixture.home());
    write_read_only_policy(fixture.home());

    let output = base_command(&fixture)
        .arg("--shim-status-json")
        .output()
        .expect("run --shim-status-json");

    let payload: Value = serde_json::from_slice(&output.stdout).expect("shim status json payload");
    assert_eq!(
        payload.get("world_fs_mode"),
        Some(&json!("read_only")),
        "shim status JSON should report world_fs_mode for parity"
    );
}

#[test]
fn shim_status_text_includes_world_fs_mode() {
    let fixture = ShellEnvFixture::new();
    seed_shims(fixture.home());
    write_read_only_policy(fixture.home());

    let output = base_command(&fixture)
        .arg("--shim-status")
        .output()
        .expect("run --shim-status");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("world_fs_mode")
            || stdout.contains("World fs mode")
            || stdout.contains("world fs mode"),
        "shim status text should include fs_mode label: {stdout}"
    );
    assert!(
        stdout.contains("read_only"),
        "shim status text should include fs_mode value: {stdout}"
    );
}
