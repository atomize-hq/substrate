#![cfg(unix)]
#![allow(dead_code, unused_imports)]

#[path = "../common.rs"]
pub mod common;

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub use common::{binary_path, ensure_substrate_built, substrate_shell_driver, temp_dir};
pub use substrate_common::dedupe_path;
mod socket;
pub use socket::{AgentSocket, SocketResponse};

pub const PAYLOAD_MARKER: &str = "__SUBSTRATE_PAYLOAD__";

pub fn get_substrate_binary() -> Command {
    substrate_shell_driver()
}

pub struct ShellEnvFixture {
    _temp: TempDir,
    home: PathBuf,
}

impl ShellEnvFixture {
    pub fn new() -> Self {
        let temp = temp_dir("substrate-test-");
        let home = temp.path().join("home");
        fs::create_dir_all(home.join(".substrate/shims"))
            .expect("failed to create shims directory");
        Self { _temp: temp, home }
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn shim_dir(&self) -> PathBuf {
        self.home.join(".substrate/shims")
    }

    pub fn manager_env_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_env.sh")
    }

    pub fn manager_init_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_init.sh")
    }

    pub fn preexec_path(&self) -> PathBuf {
        self.home.join(".substrate_preexec")
    }

    pub fn overlay_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_hooks.local.yaml")
    }

    pub fn write_manifest(&self, contents: &str) -> PathBuf {
        let path = self.home.join("manager_hooks.yaml");
        fs::write(&path, contents).expect("failed to write manager manifest");
        path
    }
}

pub fn substrate_command_for_home(fixture: &ShellEnvFixture) -> Command {
    let mut cmd = get_substrate_binary();
    cmd.env("HOME", fixture.home())
        .env("USERPROFILE", fixture.home())
        .current_dir(fixture.home())
        .env("SHELL", "/bin/bash")
        .env("SUBSTRATE_HOME", fixture.home().join(".substrate"))
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env_remove("SUBSTRATE_NO_SHIMS")
        // Ensure host-installed shims do not affect PATH injection in tests.
        .env_remove("SUBSTRATE_SHIM_PATH")
        .env_remove("SUBSTRATE_SHIM_ORIGINAL_PATH")
        .env_remove("SUBSTRATE_SHIM_DEPLOY_DIR")
        .env_remove("SHIM_ORIGINAL_PATH")
        .env_remove("PATH_BEFORE_SUBSTRATE_SHIM");
    cmd
}

pub fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn payload_lines(stdout: &[u8]) -> Vec<String> {
    let data = String::from_utf8_lossy(stdout);
    let mut marker_found = false;
    let mut lines = Vec::new();
    for line in data.lines() {
        if marker_found {
            let trimmed = line.trim_end();
            if trimmed.is_empty() || trimmed == PAYLOAD_MARKER {
                continue;
            }
            lines.push(trimmed.to_string());
        } else if line.trim() == PAYLOAD_MARKER {
            marker_found = true;
        }
    }
    assert!(
        marker_found,
        "payload marker `{}` not found in output: {}",
        PAYLOAD_MARKER, data
    );
    lines
}
