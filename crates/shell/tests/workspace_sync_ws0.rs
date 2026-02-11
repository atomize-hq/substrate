#![cfg(unix)]

mod common;

use common::{substrate_shell_driver, temp_dir};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct WorkspaceSyncFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl WorkspaceSyncFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-workspace-sync-");
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

    fn command(&self) -> assert_cmd::Command {
        let mut cmd = substrate_shell_driver();
        cmd.env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env_remove("SUBSTRATE_WORLD")
            .env_remove("SUBSTRATE_WORLD_ENABLED")
            .env_remove("SUBSTRATE_WORLD_ID");
        cmd
    }

    fn init_workspace(&self) {
        let out = self
            .command()
            .arg("workspace")
            .arg("init")
            .arg(&self.workspace_root)
            .output()
            .expect("failed to run workspace init");
        assert!(out.status.success(), "workspace init must succeed: {out:?}");
        assert!(
            self.workspace_yaml_path().is_file(),
            "workspace init must create workspace.yaml"
        );
    }

    fn workspace_yaml_path(&self) -> PathBuf {
        self.workspace_root
            .join(".substrate")
            .join("workspace.yaml")
    }

    fn gitignore_path(&self) -> PathBuf {
        self.workspace_root.join(".gitignore")
    }

    fn write_workspace_yaml_patch(&self, body: &str) {
        fs::write(self.workspace_yaml_path(), body).expect("write workspace.yaml patch");
    }

    fn run_in(&self, cwd: &Path, args: &[&str]) -> std::process::Output {
        let mut cmd = self.command();
        cmd.current_dir(cwd);
        cmd.args(args);
        cmd.output().expect("failed to run substrate command")
    }
}

fn child_names(path: &Path) -> Vec<String> {
    let mut names = fs::read_dir(path)
        .expect("read_dir")
        .map(|entry| {
            entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn combined_output(output: &std::process::Output) -> String {
    let mut out = String::new();
    out.push_str(&String::from_utf8_lossy(&output.stdout));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&String::from_utf8_lossy(&output.stderr));
    out
}

#[test]
fn workspace_sync_requires_workspace_and_is_single_actionable_line() {
    let fixture = WorkspaceSyncFixture::new();
    let cwd = fixture._temp.path().join("not-a-workspace");
    fs::create_dir_all(&cwd).expect("create cwd");

    let before = child_names(&cwd);
    let output = fixture.run_in(&cwd, &["workspace", "sync", "--dry-run"]);
    let after = child_names(&cwd);

    assert_eq!(
        output.status.code(),
        Some(2),
        "outside a workspace, workspace sync must exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed = stderr.trim();
    assert!(
        trimmed.contains("not in a workspace") && trimmed.contains("substrate workspace init"),
        "stderr must be actionable and mention workspace init: {stderr}"
    );
    assert_eq!(
        trimmed.lines().count(),
        1,
        "stderr must be a single actionable line: {stderr}"
    );
    assert_eq!(before, after, "workspace sync must not mutate cwd");
    assert!(
        !cwd.join(".substrate").exists(),
        "workspace sync must not create .substrate outside a workspace"
    );
}

#[test]
fn workspace_checkpoint_requires_workspace_and_is_single_actionable_line() {
    let fixture = WorkspaceSyncFixture::new();
    let cwd = fixture._temp.path().join("not-a-workspace");
    fs::create_dir_all(&cwd).expect("create cwd");

    let before = child_names(&cwd);
    let output = fixture.run_in(&cwd, &["workspace", "checkpoint"]);
    let after = child_names(&cwd);

    assert_eq!(
        output.status.code(),
        Some(2),
        "outside a workspace, workspace checkpoint must exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed = stderr.trim();
    assert!(
        trimmed.contains("not in a workspace") && trimmed.contains("substrate workspace init"),
        "stderr must be actionable and mention workspace init: {stderr}"
    );
    assert_eq!(
        trimmed.lines().count(),
        1,
        "stderr must be a single actionable line: {stderr}"
    );
    assert_eq!(before, after, "workspace checkpoint must not mutate cwd");
    assert!(
        !cwd.join(".substrate").exists(),
        "workspace checkpoint must not create .substrate outside a workspace"
    );
}

#[test]
fn workspace_rollback_requires_workspace_and_is_single_actionable_line() {
    let fixture = WorkspaceSyncFixture::new();
    let cwd = fixture._temp.path().join("not-a-workspace");
    fs::create_dir_all(&cwd).expect("create cwd");

    let before = child_names(&cwd);
    let output = fixture.run_in(&cwd, &["workspace", "rollback"]);
    let after = child_names(&cwd);

    assert_eq!(
        output.status.code(),
        Some(2),
        "outside a workspace, workspace rollback must exit 2: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed = stderr.trim();
    assert!(
        trimmed.contains("not in a workspace") && trimmed.contains("substrate workspace init"),
        "stderr must be actionable and mention workspace init: {stderr}"
    );
    assert_eq!(
        trimmed.lines().count(),
        1,
        "stderr must be a single actionable line: {stderr}"
    );
    assert_eq!(before, after, "workspace rollback must not mutate cwd");
    assert!(
        !cwd.join(".substrate").exists(),
        "workspace rollback must not create .substrate outside a workspace"
    );
}

#[test]
fn workspace_sync_dry_run_prints_effective_preview_and_does_not_hit_world_backend() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();

    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: [\"workspace-only\"]\n",
    );

    let before_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml");
    let before_gitignore = fs::read_to_string(fixture.gitignore_path()).expect("read .gitignore");

    let missing_socket = fixture.workspace_root.join("missing.substrate.sock");
    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["workspace", "sync", "--dry-run"]);
    let output = cmd.output().expect("run workspace sync --dry-run");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --dry-run must exit 0: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("from_world"),
        "preview must include effective direction: {combined}"
    );
    assert!(
        combined.contains("prefer_host"),
        "preview must include effective conflict_policy: {combined}"
    );
    for exclude in [".git/**", ".substrate/**", "workspace-only"] {
        assert!(
            combined.contains(exclude),
            "preview must include exclude {exclude}: {combined}"
        );
    }
    assert!(
        combined.contains("pending diff") && combined.contains("WS1"),
        "preview must state pending diff discovery is not implemented until WS1: {combined}"
    );
    assert!(
        !combined.contains("world backend unavailable"),
        "dry-run must not attempt world backend access: {combined}"
    );

    let after_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml after");
    let after_gitignore =
        fs::read_to_string(fixture.gitignore_path()).expect("read .gitignore after");
    assert_eq!(
        before_workspace_yaml, after_workspace_yaml,
        "dry-run must not mutate workspace.yaml"
    );
    assert_eq!(
        before_gitignore, after_gitignore,
        "dry-run must not mutate .gitignore"
    );
}

#[test]
fn workspace_sync_dry_run_applies_cli_overrides_in_preview_only() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: [\"workspace-only\"]\n",
    );

    let before_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml");

    let missing_socket = fixture.workspace_root.join("missing.substrate.sock");
    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args([
            "workspace",
            "sync",
            "--dry-run",
            "--direction",
            "both",
            "--conflict-policy",
            "abort",
            "--exclude",
            "cli-extra",
        ]);
    let output = cmd
        .output()
        .expect("run workspace sync --dry-run with overrides");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --dry-run must exit 0: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    for needle in ["both", "abort", "workspace-only", "cli-extra"] {
        assert!(
            combined.contains(needle),
            "preview must include {needle}: {combined}"
        );
    }
    for exclude in [".git/**", ".substrate/**"] {
        assert!(
            combined.contains(exclude),
            "preview must include protected exclude {exclude}: {combined}"
        );
    }

    let after_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml after");
    assert_eq!(
        before_workspace_yaml, after_workspace_yaml,
        "dry-run overrides must not mutate workspace.yaml"
    );
}

#[test]
fn workspace_sync_without_dry_run_is_stubbed_in_ws0() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();

    let before_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml");

    let missing_socket = fixture.workspace_root.join("missing.substrate.sock");
    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["workspace", "sync"]);
    let output = cmd.output().expect("run workspace sync");

    assert_eq!(
        output.status.code(),
        Some(4),
        "workspace sync (no --dry-run) must be stubbed in WS0 (exit 4): {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("not implemented") && combined.contains("WS2"),
        "workspace sync stub must mention WS2: {combined}"
    );
    assert!(
        !combined.contains("world backend unavailable"),
        "stub must not attempt world backend access: {combined}"
    );

    let after_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml after");
    assert_eq!(
        before_workspace_yaml, after_workspace_yaml,
        "workspace sync stub must not mutate workspace.yaml"
    );
}

#[test]
fn workspace_checkpoint_is_stubbed_in_ws0() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();

    let before_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml");

    let missing_socket = fixture.workspace_root.join("missing.substrate.sock");
    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["workspace", "checkpoint"]);
    let output = cmd.output().expect("run workspace checkpoint");

    assert_eq!(
        output.status.code(),
        Some(4),
        "workspace checkpoint must be stubbed in WS0 (exit 4): {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("not implemented") && combined.contains("WS6"),
        "workspace checkpoint stub must mention WS6: {combined}"
    );

    let after_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml after");
    assert_eq!(
        before_workspace_yaml, after_workspace_yaml,
        "workspace checkpoint stub must not mutate workspace.yaml"
    );
}

#[test]
fn workspace_rollback_is_stubbed_in_ws0() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();

    let before_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml");

    let missing_socket = fixture.workspace_root.join("missing.substrate.sock");
    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["workspace", "rollback"]);
    let output = cmd.output().expect("run workspace rollback");

    assert_eq!(
        output.status.code(),
        Some(4),
        "workspace rollback must be stubbed in WS0 (exit 4): {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("not implemented") && combined.contains("WS7"),
        "workspace rollback stub must mention WS7: {combined}"
    );

    let after_workspace_yaml =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml after");
    assert_eq!(
        before_workspace_yaml, after_workspace_yaml,
        "workspace rollback stub must not mutate workspace.yaml"
    );
}
