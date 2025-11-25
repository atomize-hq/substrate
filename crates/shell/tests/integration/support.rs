#![cfg(unix)]

use crate::common::{substrate_shell_driver, temp_dir};
use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Marker included in payload output lines for fixture parsing.
pub const PAYLOAD_MARKER: &str = "__SUBSTRATE_PAYLOAD__";

/// Helper function to get the substrate binary from workspace root.
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
        .env("SHELL", "/bin/bash");
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
            lines.push(line.trim_end().to_string());
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
