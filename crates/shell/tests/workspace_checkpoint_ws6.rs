#![cfg(unix)]

mod support;

use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use support::substrate_shell_driver;
use tempfile::{Builder, TempDir};

struct WorkspaceCheckpointFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl WorkspaceCheckpointFixture {
    fn new() -> Self {
        let temp = Builder::new()
            .prefix("substrate-workspace-checkpoint-")
            .tempdir_in("/tmp")
            .expect("failed to allocate ws6 temp dir");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("create HOME fixture");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME fixture");
        let workspace_root = temp.path().join("workspace");
        fs::create_dir_all(&workspace_root).expect("create workspace root");
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
            .expect("run workspace init");
        assert!(out.status.success(), "workspace init must succeed: {out:?}");
        assert!(
            self.workspace_yaml_path().is_file(),
            "workspace init must create workspace.yaml"
        );
        assert!(
            self.internal_git_repo_dir().is_dir(),
            "workspace init must create internal git dir"
        );
        assert!(
            !self.internal_git_repo_dir().join("HEAD").exists(),
            "workspace init must not initialize internal git (HEAD must not exist yet)"
        );
    }

    fn workspace_yaml_path(&self) -> PathBuf {
        self.workspace_root
            .join(".substrate")
            .join("workspace.yaml")
    }

    fn internal_git_repo_dir(&self) -> PathBuf {
        self.workspace_root
            .join(".substrate")
            .join("git")
            .join("repo.git")
    }

    fn git(&self) -> StdCommand {
        let mut cmd = StdCommand::new("git");
        cmd.arg("--git-dir")
            .arg(self.internal_git_repo_dir())
            .arg("--work-tree")
            .arg(&self.workspace_root);
        cmd
    }
}

fn assert_checkpoint_id_format(checkpoint_id: &str) {
    assert!(
        checkpoint_id.starts_with("cp/"),
        "checkpoint id must start with `cp/`, got: {checkpoint_id}"
    );
    assert_eq!(
        checkpoint_id.len(),
        "cp/".len() + "YYYYMMDDTHHMMSSZ".len(),
        "checkpoint id must match `cp/<YYYYMMDDTHHMMSSZ>`, got: {checkpoint_id}"
    );
    let ts = &checkpoint_id["cp/".len()..];
    let bytes = ts.as_bytes();
    assert_eq!(
        bytes[8] as char, 'T',
        "checkpoint timestamp missing `T`: {ts}"
    );
    assert_eq!(
        bytes[15] as char, 'Z',
        "checkpoint timestamp missing trailing `Z`: {ts}"
    );
    for (idx, ch) in ts.chars().enumerate() {
        if idx == 8 || idx == 15 {
            continue;
        }
        assert!(
            ch.is_ascii_digit(),
            "checkpoint timestamp must be digits except `T`/`Z`, got: {ts}"
        );
    }
}

fn git_stdout(cmd: &mut StdCommand) -> String {
    let out = cmd.output().expect("run git");
    assert!(
        out.status.success(),
        "git must succeed: status={:?} stdout={} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn git_lines(cmd: &mut StdCommand) -> Vec<String> {
    let out = cmd.output().expect("run git");
    assert!(
        out.status.success(),
        "git must succeed: status={:?} stdout={} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn workspace_checkpoint_initializes_internal_repo_creates_tag_and_prints_id() {
    let fixture = WorkspaceCheckpointFixture::new();
    fixture.init_workspace();

    fs::write(fixture.workspace_root.join("tracked.txt"), "hello\n").expect("write tracked file");
    fs::create_dir_all(fixture.workspace_root.join(".git")).expect("create .git");
    fs::write(
        fixture.workspace_root.join(".git").join("leak.txt"),
        "secret\n",
    )
    .expect("write protected .git file");
    fs::write(
        fixture.workspace_root.join(".substrate").join("leak.txt"),
        "secret\n",
    )
    .expect("write protected .substrate file");

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .args(["workspace", "checkpoint"]);
    let output = cmd.output().expect("run workspace checkpoint");

    assert!(
        output.status.success(),
        "workspace checkpoint must succeed: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let checkpoint_id = stdout.trim();
    assert_checkpoint_id_format(checkpoint_id);
    assert_eq!(
        stdout.lines().count(),
        1,
        "workspace checkpoint stdout must be a single stable line: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.trim().is_empty(),
        "workspace checkpoint must not write to stderr on success: {stderr}"
    );

    assert!(
        fixture.internal_git_repo_dir().join("HEAD").is_file(),
        "workspace checkpoint must initialize internal repo (HEAD must exist)"
    );

    let mut resolve_commit = fixture.git();
    resolve_commit.args(["rev-parse", checkpoint_id]);
    let commit = git_stdout(&mut resolve_commit);

    let mut list_tree = fixture.git();
    list_tree.args(["ls-tree", "-r", "--name-only", &commit]);
    let files = git_lines(&mut list_tree);
    assert!(
        files.iter().any(|p| p == "tracked.txt"),
        "checkpoint must include tracked file: {files:?}"
    );
    assert!(
        files.iter().all(|p| !p.starts_with(".git/")),
        "checkpoint must exclude `.git/**`: {files:?}"
    );
    assert!(
        files.iter().all(|p| !p.starts_with(".substrate/")),
        "checkpoint must exclude `.substrate/**`: {files:?}"
    );
}

#[test]
fn workspace_checkpoint_is_noop_when_no_changes_and_does_not_create_new_tag() {
    let fixture = WorkspaceCheckpointFixture::new();
    fixture.init_workspace();
    fs::write(fixture.workspace_root.join("file.txt"), "v1\n").expect("write file");

    let output = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "checkpoint"])
        .output()
        .expect("run workspace checkpoint");
    assert!(
        output.status.success(),
        "first workspace checkpoint must succeed: {output:?}"
    );
    let mut before_main_cmd = fixture.git();
    before_main_cmd.args(["rev-parse", "main"]);
    let before_main = git_stdout(&mut before_main_cmd);

    let mut before_tags_cmd = fixture.git();
    before_tags_cmd.args(["tag", "-l", "cp/*"]);
    let before_tags = git_lines(&mut before_tags_cmd);

    let output = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "checkpoint"])
        .output()
        .expect("run workspace checkpoint no-op");
    assert!(
        output.status.success(),
        "no-op workspace checkpoint must succeed: {output:?}"
    );
    let mut after_main_cmd = fixture.git();
    after_main_cmd.args(["rev-parse", "main"]);
    let after_main = git_stdout(&mut after_main_cmd);

    let mut after_tags_cmd = fixture.git();
    after_tags_cmd.args(["tag", "-l", "cp/*"]);
    let after_tags = git_lines(&mut after_tags_cmd);

    assert_eq!(
        before_main, after_main,
        "no-op checkpoint must not create a new commit"
    );
    assert_eq!(
        before_tags, after_tags,
        "no-op checkpoint must not create a new tag"
    );
}

#[test]
fn workspace_checkpoint_includes_gitignored_files_and_excludes_protected_paths() {
    let fixture = WorkspaceCheckpointFixture::new();
    fixture.init_workspace();

    let gitignore = fixture.workspace_root.join(".gitignore");
    let mut ignore = fs::read_to_string(&gitignore).unwrap_or_default();
    ignore.push_str("\nignored.txt\n");
    fs::write(&gitignore, ignore).expect("patch .gitignore");

    fs::write(fixture.workspace_root.join("ignored.txt"), "kept\n").expect("write ignored file");
    fs::write(fixture.workspace_root.join("kept.txt"), "kept\n").expect("write kept file");

    fs::create_dir_all(fixture.workspace_root.join(".git")).expect("create .git");
    fs::write(
        fixture.workspace_root.join(".git").join("config"),
        "should-not-snapshot\n",
    )
    .expect("write .git/config");

    fs::write(
        fixture.workspace_root.join(".substrate").join("note.txt"),
        "should-not-snapshot\n",
    )
    .expect("write .substrate/note.txt");

    let output = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "checkpoint"])
        .output()
        .expect("run workspace checkpoint");
    assert!(
        output.status.success(),
        "workspace checkpoint must succeed: {output:?}"
    );
    let checkpoint_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_checkpoint_id_format(&checkpoint_id);

    let mut resolve_commit = fixture.git();
    resolve_commit.args(["rev-parse", &checkpoint_id]);
    let commit = git_stdout(&mut resolve_commit);

    let mut list_tree = fixture.git();
    list_tree.args(["ls-tree", "-r", "--name-only", &commit]);
    let files = git_lines(&mut list_tree);

    assert!(
        files.iter().any(|p| p == "ignored.txt"),
        "checkpoint must include gitignored file: {files:?}"
    );
    assert!(
        files.iter().any(|p| p == "kept.txt"),
        "checkpoint must include normal file: {files:?}"
    );
    assert!(
        files.iter().all(|p| !p.starts_with(".git/")),
        "checkpoint must exclude `.git/**`: {files:?}"
    );
    assert!(
        files.iter().all(|p| !p.starts_with(".substrate/")),
        "checkpoint must exclude `.substrate/**`: {files:?}"
    );
}

#[test]
fn workspace_checkpoint_exits_3_when_git_missing() {
    let fixture = WorkspaceCheckpointFixture::new();
    fixture.init_workspace();
    fs::write(fixture.workspace_root.join("file.txt"), "v1\n").expect("write file");

    let empty_bin = fixture._temp.path().join("empty-bin");
    fs::create_dir_all(&empty_bin).expect("create empty PATH dir");

    let output = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .env("PATH", &empty_bin)
        .args(["workspace", "checkpoint"])
        .output()
        .expect("run workspace checkpoint");

    assert_eq!(
        output.status.code(),
        Some(3),
        "workspace checkpoint must exit 3 when git is unavailable: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed = stderr.trim();
    assert!(
        trimmed.contains("git"),
        "stderr must mention missing git: {stderr}"
    );
    assert_eq!(
        trimmed.lines().count(),
        1,
        "stderr must be a single actionable line: {stderr}"
    );
}
