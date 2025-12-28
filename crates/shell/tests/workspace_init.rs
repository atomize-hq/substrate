#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct WorkspaceFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
}

impl WorkspaceFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-workspace-");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("failed to create HOME fixture");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME fixture");
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

    fn init_workspace(&self, path: &Path, force: bool) -> std::process::Output {
        let mut cmd = self.command();
        cmd.arg("workspace").arg("init").arg(path);
        if force {
            cmd.arg("--force");
        }
        cmd.output().expect("failed to run workspace init")
    }
}

#[test]
fn workspace_init_creates_expected_inventory_and_gitignore() {
    let fixture = WorkspaceFixture::new();
    let root = fixture._temp.path().join("workspace");
    fs::create_dir_all(&root).expect("create workspace root");

    let output = fixture.init_workspace(&root, false);
    assert!(
        output.status.success(),
        "workspace init should succeed: {output:?}"
    );

    let substrate_dir = root.join(".substrate");
    let workspace_yaml = substrate_dir.join("workspace.yaml");
    let policy_yaml = substrate_dir.join("policy.yaml");
    let internal_git = root.join(".substrate-git").join("repo.git");
    assert!(substrate_dir.is_dir());
    assert!(workspace_yaml.is_file());
    assert!(policy_yaml.is_file());
    assert!(internal_git.is_dir());

    let gitignore = fs::read_to_string(root.join(".gitignore")).expect("read .gitignore");
    for rule in [
        ".substrate-git/",
        ".substrate/*",
        "!.substrate/workspace.yaml",
        "!.substrate/policy.yaml",
    ] {
        assert!(
            gitignore.lines().any(|line| line.trim_end() == rule),
            ".gitignore must include rule {rule}\n.gitignore:\n{gitignore}"
        );
    }

    let raw = fs::read_to_string(&workspace_yaml).expect("read workspace.yaml");
    let yaml: YamlValue = serde_yaml::from_str(&raw).expect("workspace.yaml should parse");
    let root = yaml.as_mapping().expect("workspace.yaml root mapping");
    assert!(root.contains_key(&YamlValue::String("world".to_string())));
    assert!(root.contains_key(&YamlValue::String("policy".to_string())));
    assert!(root.contains_key(&YamlValue::String("sync".to_string())));
}

#[test]
fn workspace_init_refuses_nested_workspaces_without_writes() {
    let fixture = WorkspaceFixture::new();
    let parent = fixture._temp.path().join("parent");
    let nested = parent.join("child");
    fs::create_dir_all(&nested).expect("create nested workspace dir");

    let parent_out = fixture.init_workspace(&parent, false);
    assert!(parent_out.status.success(), "parent init should succeed");

    let nested_out = fixture.init_workspace(&nested, false);
    assert_eq!(
        nested_out.status.code(),
        Some(2),
        "nested init should exit 2: {nested_out:?}"
    );

    assert!(
        !nested.join(".substrate").exists(),
        "nested init must not create .substrate"
    );
    assert!(
        !nested.join(".substrate-git").exists(),
        "nested init must not create .substrate-git"
    );
    assert!(
        !nested.join(".gitignore").exists(),
        "nested init must not touch .gitignore"
    );
}

#[test]
fn workspace_discovery_walks_up_to_marker() {
    let fixture = WorkspaceFixture::new();
    let root = fixture._temp.path().join("workspace");
    fs::create_dir_all(&root).expect("create workspace root");
    let out = fixture.init_workspace(&root, false);
    assert!(out.status.success(), "workspace init should succeed");

    let child = root.join("a").join("b");
    fs::create_dir_all(&child).expect("create child dir");

    let output = fixture
        .command()
        .current_dir(&child)
        .arg("config")
        .arg("show")
        .arg("--json")
        .output()
        .expect("run config show");

    assert!(
        output.status.success(),
        "config show from child should succeed: {output:?}"
    );
}

#[test]
fn workspace_init_force_repairs_missing_files_without_overwriting_existing() {
    let fixture = WorkspaceFixture::new();
    let root = fixture._temp.path().join("workspace");
    fs::create_dir_all(&root).expect("create workspace root");

    let output = fixture.init_workspace(&root, false);
    assert!(output.status.success(), "workspace init should succeed");

    let workspace_yaml = root.join(".substrate").join("workspace.yaml");
    let policy_yaml = root.join(".substrate").join("policy.yaml");

    fs::write(&workspace_yaml, "sentinel: 1\n").expect("write sentinel workspace.yaml");
    fs::remove_file(&policy_yaml).expect("remove policy.yaml");

    let output = fixture.init_workspace(&root, true);
    assert!(
        output.status.success(),
        "workspace init --force should succeed: {output:?}"
    );

    let after_workspace = fs::read_to_string(&workspace_yaml).expect("read workspace.yaml");
    assert_eq!(
        after_workspace, "sentinel: 1\n",
        "workspace init --force must not overwrite workspace.yaml"
    );
    assert!(
        policy_yaml.exists(),
        "workspace init --force should restore policy.yaml"
    );
}

#[test]
fn workspace_init_rejects_non_directory_paths() {
    let fixture = WorkspaceFixture::new();
    let root = fixture._temp.path().join("workspace");
    fs::create_dir_all(&root).expect("create workspace root");
    let file_path = root.join("not-a-dir");
    fs::write(&file_path, "x").expect("write file");

    let output = fixture.init_workspace(&file_path, false);
    assert_eq!(
        output.status.code(),
        Some(2),
        "workspace init should exit 2 on invalid PATH: {output:?}"
    );
    assert!(
        !root.join(".substrate").exists(),
        "workspace init should not write to sibling directories on invalid PATH"
    );
}
