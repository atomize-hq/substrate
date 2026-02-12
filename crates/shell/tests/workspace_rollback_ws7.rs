#![cfg(unix)]

mod support;

use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::thread;
use std::time::Duration;
use support::{substrate_shell_driver, temp_dir};
use tempfile::TempDir;

struct WorkspaceRollbackFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl WorkspaceRollbackFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-workspace-rollback-");
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

    fn substrate_stdout_line(&self, args: &[&str]) -> String {
        let out = self
            .command()
            .current_dir(&self.workspace_root)
            .args(args)
            .output()
            .expect("run substrate");
        assert!(
            out.status.success(),
            "command must succeed: status={:?} stdout={} stderr={}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(
            stdout.lines().count(),
            1,
            "stdout must be a single stable line: {stdout}"
        );
        stdout.trim().to_string()
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
fn workspace_rollback_by_explicit_checkpoint_id_restores_files_and_does_not_create_new_tags() {
    let fixture = WorkspaceRollbackFixture::new();
    fixture.init_workspace();

    let protected_substrate = fixture.workspace_root.join(".substrate").join("keep.txt");
    fs::write(&protected_substrate, "do not touch\n").expect("write protected .substrate file");
    let workspace_yaml_before =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml before");

    let tracked = fixture.workspace_root.join("tracked.txt");
    fs::write(&tracked, "v1\n").expect("write tracked file");
    let checkpoint_id = fixture.substrate_stdout_line(&["workspace", "checkpoint"]);
    assert_checkpoint_id_format(&checkpoint_id);

    let mut tag_commit_cmd = fixture.git();
    tag_commit_cmd.args(["rev-parse"]).arg(&checkpoint_id);
    let checkpoint_commit = git_stdout(&mut tag_commit_cmd);

    let mut commit_count_before_cmd = fixture.git();
    commit_count_before_cmd.args(["rev-list", "--all", "--count"]);
    let commit_count_before = git_stdout(&mut commit_count_before_cmd);

    let mut tags_before_cmd = fixture.git();
    tags_before_cmd.args(["tag", "-l", "cp/*"]);
    let tags_before = git_lines(&mut tags_before_cmd);

    fs::write(&tracked, "v2\n").expect("mutate tracked file");

    let out = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "rollback"])
        .arg(&checkpoint_id)
        .output()
        .expect("run workspace rollback");
    assert_eq!(
        out.status.code(),
        Some(0),
        "workspace rollback must succeed: {out:?}"
    );

    let tracked_after = fs::read_to_string(&tracked).expect("read tracked file after rollback");
    assert_eq!(tracked_after, "v1\n", "rollback must restore tracked file");

    let mut tags_after_cmd = fixture.git();
    tags_after_cmd.args(["tag", "-l", "cp/*"]);
    let tags_after = git_lines(&mut tags_after_cmd);
    assert_eq!(
        tags_before, tags_after,
        "rollback must not create new checkpoint tags"
    );

    let mut main_after_cmd = fixture.git();
    main_after_cmd.args(["rev-parse", "main"]);
    let main_after = git_stdout(&mut main_after_cmd);
    assert_eq!(
        main_after, checkpoint_commit,
        "rollback must restore internal main to the target commit"
    );

    let mut commit_count_after_cmd = fixture.git();
    commit_count_after_cmd.args(["rev-list", "--all", "--count"]);
    let commit_count_after = git_stdout(&mut commit_count_after_cmd);
    assert_eq!(
        commit_count_before, commit_count_after,
        "rollback must not create new internal commits"
    );

    let protected_substrate_after =
        fs::read_to_string(&protected_substrate).expect("read protected .substrate file after");
    assert_eq!(
        protected_substrate_after, "do not touch\n",
        "rollback must not mutate protected `.substrate/**` paths"
    );
    let workspace_yaml_after =
        fs::read_to_string(fixture.workspace_yaml_path()).expect("read workspace.yaml after");
    assert_eq!(
        workspace_yaml_before, workspace_yaml_after,
        "rollback must not mutate protected workspace.yaml"
    );
}

#[test]
fn workspace_rollback_last_resolves_to_most_recent_checkpoint() {
    let fixture = WorkspaceRollbackFixture::new();
    fixture.init_workspace();

    let tracked = fixture.workspace_root.join("tracked.txt");
    fs::write(&tracked, "one\n").expect("write tracked file");
    let cp1 = fixture.substrate_stdout_line(&["workspace", "checkpoint"]);
    assert_checkpoint_id_format(&cp1);

    thread::sleep(Duration::from_secs(1));
    fs::write(&tracked, "two\n").expect("mutate tracked file for checkpoint 2");
    let cp2 = fixture.substrate_stdout_line(&["workspace", "checkpoint"]);
    assert_checkpoint_id_format(&cp2);
    assert_ne!(cp1, cp2, "checkpoint ids must be distinct");

    fs::write(&tracked, "three\n").expect("mutate tracked file before rollback");

    let out = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "rollback", "last"])
        .output()
        .expect("run workspace rollback last");
    assert_eq!(
        out.status.code(),
        Some(0),
        "workspace rollback last must succeed: {out:?}"
    );
    let tracked_after = fs::read_to_string(&tracked).expect("read tracked file after rollback");
    assert_eq!(
        tracked_after, "two\n",
        "rollback last must restore to the most recent checkpoint"
    );
}

#[test]
fn workspace_rollback_requires_force_for_noncheckpointed_paths_deletion_and_preserves_protected_paths(
) {
    let fixture = WorkspaceRollbackFixture::new();
    fixture.init_workspace();

    let protected_substrate = fixture.workspace_root.join(".substrate").join("keep.txt");
    fs::write(&protected_substrate, "protected\n").expect("write protected .substrate file");

    let tracked = fixture.workspace_root.join("tracked.txt");
    fs::write(&tracked, "v1\n").expect("write tracked file");
    let checkpoint_id = fixture.substrate_stdout_line(&["workspace", "checkpoint"]);
    assert_checkpoint_id_format(&checkpoint_id);

    fs::write(&tracked, "v2\n").expect("mutate tracked file before refusal test");
    let extra = fixture.workspace_root.join("extra.txt");
    fs::write(&extra, "not in snapshot\n").expect("write extra file");

    let out = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "rollback"])
        .arg(&checkpoint_id)
        .output()
        .expect("run workspace rollback without --force");
    assert_eq!(
        out.status.code(),
        Some(5),
        "rollback without --force must refuse when it would delete non-checkpointed paths: {out:?}"
    );
    assert_eq!(
        fs::read_to_string(&tracked).expect("read tracked after refusal"),
        "v2\n",
        "rollback refusal must not mutate tracked file"
    );
    assert!(
        extra.exists(),
        "rollback refusal must not delete extra file"
    );
    assert_eq!(
        fs::read_to_string(&protected_substrate).expect("read protected after refusal"),
        "protected\n",
        "rollback refusal must not mutate protected `.substrate/**` paths"
    );

    let out = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "rollback", "--force"])
        .arg(&checkpoint_id)
        .output()
        .expect("run workspace rollback --force");
    assert_eq!(
        out.status.code(),
        Some(0),
        "rollback --force must succeed: {out:?}"
    );
    assert_eq!(
        fs::read_to_string(&tracked).expect("read tracked after rollback --force"),
        "v1\n",
        "rollback --force must restore tracked file"
    );
    assert!(
        !extra.exists(),
        "rollback --force must delete non-checkpointed paths"
    );
    assert_eq!(
        fs::read_to_string(&protected_substrate).expect("read protected after rollback --force"),
        "protected\n",
        "rollback --force must preserve protected `.substrate/**` paths"
    );
}

#[test]
fn workspace_rollback_requires_force_when_user_repo_is_dirty() {
    let fixture = WorkspaceRollbackFixture::new();
    fixture.init_workspace();

    let tracked = fixture.workspace_root.join("tracked.txt");
    fs::write(&tracked, "v1\n").expect("write tracked file");
    let checkpoint_id = fixture.substrate_stdout_line(&["workspace", "checkpoint"]);
    assert_checkpoint_id_format(&checkpoint_id);

    let git_dir = fixture.workspace_root.join(".git");
    let keep = git_dir.join("keep.txt");

    let out = StdCommand::new("git")
        .arg("init")
        .current_dir(&fixture.workspace_root)
        .output()
        .expect("git init user repo");
    assert!(
        out.status.success(),
        "git init must succeed: status={:?} stdout={} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let out = StdCommand::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(&fixture.workspace_root)
        .output()
        .expect("git config user.name");
    assert!(
        out.status.success(),
        "git config user.name must succeed: status={:?} stdout={} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let out = StdCommand::new("git")
        .args(["config", "user.email", "tester@example.com"])
        .current_dir(&fixture.workspace_root)
        .output()
        .expect("git config user.email");
    assert!(
        out.status.success(),
        "git config user.email must succeed: status={:?} stdout={} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    fs::write(&keep, "do not touch\n").expect("write .git keep file");
    let user_file = fixture.workspace_root.join("user.txt");
    fs::write(&user_file, "committed\n").expect("write user file");
    let out = StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&fixture.workspace_root)
        .output()
        .expect("git add user repo");
    assert!(
        out.status.success(),
        "git add must succeed: status={:?} stdout={} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let out = StdCommand::new("git")
        .args(["commit", "-m", "seed"])
        .current_dir(&fixture.workspace_root)
        .output()
        .expect("git commit user repo");
    assert!(
        out.status.success(),
        "git commit must succeed: status={:?} stdout={} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    fs::write(&user_file, "dirty\n").expect("dirty user file");

    fs::write(&tracked, "v2\n").expect("mutate tracked before rollback");
    let out = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "rollback"])
        .arg(&checkpoint_id)
        .output()
        .expect("run workspace rollback without --force (dirty user repo)");
    assert_eq!(
        out.status.code(),
        Some(5),
        "rollback without --force must refuse on dirty workspace: {out:?}"
    );
    assert_eq!(
        fs::read_to_string(&tracked).expect("read tracked after refusal"),
        "v2\n",
        "rollback refusal must not mutate tracked file"
    );
    assert_eq!(
        fs::read_to_string(&keep).expect("read .git keep file after refusal"),
        "do not touch\n",
        "rollback refusal must not mutate protected `.git/**` paths"
    );

    let out = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "rollback", "--force"])
        .arg(&checkpoint_id)
        .output()
        .expect("run workspace rollback --force (dirty user repo)");
    assert_eq!(
        out.status.code(),
        Some(0),
        "rollback --force must succeed: {out:?}"
    );
    assert_eq!(
        fs::read_to_string(&tracked).expect("read tracked after rollback --force"),
        "v1\n",
        "rollback --force must restore tracked file"
    );
    assert_eq!(
        fs::read_to_string(&keep).expect("read .git keep file after rollback --force"),
        "do not touch\n",
        "rollback --force must not mutate protected `.git/**` paths"
    );
}

#[test]
fn workspace_rollback_invalid_target_exits_2() {
    let fixture = WorkspaceRollbackFixture::new();
    fixture.init_workspace();

    fs::write(fixture.workspace_root.join("tracked.txt"), "v1\n").expect("write tracked file");
    let checkpoint_id = fixture.substrate_stdout_line(&["workspace", "checkpoint"]);
    assert_checkpoint_id_format(&checkpoint_id);

    let out = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .args(["workspace", "rollback", "cp/19990101T000000Z"])
        .output()
        .expect("run workspace rollback invalid target");
    assert_eq!(
        out.status.code(),
        Some(2),
        "workspace rollback invalid target must exit 2: {out:?}"
    );
}

#[test]
fn workspace_rollback_exits_3_when_git_is_unavailable() {
    let fixture = WorkspaceRollbackFixture::new();
    fixture.init_workspace();

    fs::write(fixture.workspace_root.join("tracked.txt"), "v1\n").expect("write tracked file");
    let checkpoint_id = fixture.substrate_stdout_line(&["workspace", "checkpoint"]);
    assert_checkpoint_id_format(&checkpoint_id);

    let out = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .env("PATH", "/nonexistent")
        .args(["workspace", "rollback", "last"])
        .output()
        .expect("run workspace rollback with missing git in PATH");
    assert_eq!(
        out.status.code(),
        Some(3),
        "workspace rollback must exit 3 when git is unavailable: {out:?}"
    );
}
