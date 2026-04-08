#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

struct AgentsValidateFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl AgentsValidateFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-agents-validate-");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("failed to create HOME fixture");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME fixture");
        let workspace_root = temp.path().join("workspace");
        fs::create_dir_all(&workspace_root).expect("failed to create workspace root");
        Self {
            _temp: temp,
            home,
            substrate_home,
            workspace_root,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home);
        cmd
    }

    fn init_workspace(&self) {
        let output = self
            .command()
            .arg("workspace")
            .arg("init")
            .arg(&self.workspace_root)
            .arg("--force")
            .output()
            .expect("failed to run workspace init");
        assert!(
            output.status.success(),
            "workspace init should succeed: {output:?}"
        );
    }

    fn write_global_policy_patch(&self, contents: &str) {
        fs::write(self.substrate_home.join("policy.yaml"), contents)
            .expect("failed to write policy.yaml");
    }

    fn write_agent_file(&self, file_name: &str, contents: &str) {
        let agents_dir = self.substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("failed to create agents directory");
        fs::write(agents_dir.join(file_name), contents).expect("failed to write agent file");
    }

    fn validate(&self) -> std::process::Output {
        self.command()
            .current_dir(&self.workspace_root)
            .arg("agents")
            .arg("validate")
            .output()
            .expect("failed to run agents validate")
    }
}

fn valid_cli_agent_file(agent_id: &str, policy_overlay: Option<&str>) -> String {
    let overlay = policy_overlay.unwrap_or("");
    if overlay.is_empty() {
        format!(
            "version: 1\nid: {agent_id}\nconfig:\n  kind: cli\n  enabled: true\n  execution:\n    scope: world\n  cli:\n    binary: codex\n    mode: persistent\n"
        )
    } else {
        format!(
            "version: 1\nid: {agent_id}\nconfig:\n  kind: cli\n  enabled: true\n  execution:\n    scope: world\n  cli:\n    binary: codex\n    mode: persistent\npolicy_overlay:\n{overlay}"
        )
    }
}

#[test]
fn agents_validate_accepts_valid_inventory_and_restrictive_world_fs_overlay() {
    let fixture = AgentsValidateFixture::new();
    fixture.init_workspace();
    fixture.write_global_policy_patch(
        r#"
world_fs:
  host_visible: false
"#,
    );
    fixture.write_agent_file(
        "codex.yaml",
        &valid_cli_agent_file(
            "codex",
            Some("  world_fs:\n    read:\n      allow_list:\n        - \".\"\n"),
        ),
    );

    let output = fixture.validate();
    assert!(
        output.status.success(),
        "valid inventory and restrictive world_fs overlay should succeed: {output:?}"
    );
}

#[test]
fn agents_validate_rejects_unknown_keys_with_exit_2() {
    let fixture = AgentsValidateFixture::new();
    fixture.init_workspace();
    fixture.write_agent_file(
        "bad_unknown.yaml",
        r#"version: 1
id: bad_unknown
unknown_key: true
config:
  kind: cli
  enabled: true
  execution:
    scope: world
  cli:
    binary: codex
    mode: persistent
"#,
    );

    let output = fixture.validate();
    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown key should exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("bad_unknown.yaml") && stderr.contains("unknown_key"),
        "stderr should mention the file path and unknown key\nstderr: {stderr}"
    );
}

#[test]
fn agents_validate_rejects_filename_id_mismatch_with_exit_2() {
    let fixture = AgentsValidateFixture::new();
    fixture.init_workspace();
    fixture.write_agent_file(
        "mismatch.yaml",
        r#"version: 1
id: other
config:
  kind: cli
  enabled: true
  execution:
    scope: world
  cli:
    binary: codex
    mode: persistent
"#,
    );

    let output = fixture.validate();
    assert_eq!(
        output.status.code(),
        Some(2),
        "id mismatch should exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("mismatch.yaml")
            && stderr.contains("id 'other' must match filename 'mismatch.yaml'"),
        "stderr should mention the file path and id mismatch\nstderr: {stderr}"
    );
}

#[test]
fn agents_validate_rejects_world_fs_broadening_with_exit_2() {
    let fixture = AgentsValidateFixture::new();
    fixture.init_workspace();
    fixture.write_global_policy_patch(
        r#"
world_fs:
  host_visible: false
"#,
    );
    fixture.write_agent_file(
        "broaden.yaml",
        &valid_cli_agent_file(
            "broaden",
            Some("  world_fs:\n    read:\n      allow_list:\n        - \"/tmp\"\n"),
        ),
    );

    let output = fixture.validate();
    assert_eq!(
        output.status.code(),
        Some(2),
        "world_fs broadening should exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("broaden.yaml")
            && stderr.contains("policy_overlay.world_fs.read.allow_list")
            && stderr.contains("broadens beyond the effective base policy"),
        "stderr should mention the file path and the world_fs broadening reason\nstderr: {stderr}"
    );
}
