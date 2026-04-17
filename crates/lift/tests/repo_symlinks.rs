use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd as _;
use clap as _;
use gix as _;
use globset as _;
use jsonschema as _;
use predicates as _;
use serde as _;
use serde_jcs as _;
use serde_json as _;
use sha2 as _;
use substrate_lift as _;
use thiserror as _;
use toml as _;
use walkdir as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::*;
}
#[path = "../src/repo/mod.rs"]
mod repo;

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "substrate-lift-{prefix}-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temp dir should be creatable");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_file(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir should be creatable");
    }
    fs::write(path, contents).unwrap_or_else(|error| {
        panic!("failed to write {}: {error}", path.display());
    });
}

fn snapshot_request(
    root: &Path,
    source: repo::SnapshotSource,
    options: repo::SnapshotOptions,
) -> repo::SnapshotRequest {
    repo::SnapshotRequest {
        root: repo::RepoRoot::from_dir(root).expect("repo root should parse"),
        source,
        options,
    }
}

fn materialize(
    root: &Path,
    source: repo::SnapshotSource,
    options: repo::SnapshotOptions,
) -> repo::RepoSnapshot {
    repo::materialize_snapshot(&snapshot_request(root, source, options))
        .expect("snapshot should build")
}

fn run_git(repo_root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .expect("git command should run");
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn git_stdout(repo_root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .expect("git command should run");
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .expect("git output should be utf-8")
        .trim()
        .to_owned()
}

fn init_git_repo(prefix: &str) -> TempDir {
    let temp = TempDir::new(prefix);
    run_git(temp.path(), &["init", "--quiet"]);
    temp
}

fn commit_all(repo_root: &Path, message: &str) {
    run_git(repo_root, &["add", "-A"]);
    run_git(
        repo_root,
        &[
            "-c",
            "user.name=Substrate Lift",
            "-c",
            "user.email=lift@example.com",
            "commit",
            "--quiet",
            "-m",
            message,
        ],
    );
}

#[cfg(unix)]
#[test]
fn worktree_follow_materializes_target_bytes_at_the_link_path() {
    use std::os::unix::fs::symlink;

    let repo_root = TempDir::new("repo-symlink-worktree-follow");
    fs::create_dir_all(repo_root.path().join(".git")).expect("git dir should exist");
    write_file(&repo_root.path().join("dir/target.txt"), b"target-bytes");
    symlink("dir/target.txt", repo_root.path().join("link.txt")).expect("symlink should exist");

    let options = repo::SnapshotOptions {
        symlink_policy: repo::SymlinkPolicy::Follow,
        ..repo::SnapshotOptions::default()
    };
    let snapshot = materialize(repo_root.path(), repo::SnapshotSource::Worktree, options);
    let link = crate::kernel::RepoPath::parse("link.txt").expect("path should parse");
    let target = crate::kernel::RepoPath::parse("dir/target.txt").expect("path should parse");

    assert_eq!(
        snapshot.read_bytes(&link).expect("link bytes should exist"),
        b"target-bytes"
    );
    assert_eq!(
        snapshot
            .entry(&link)
            .expect("link entry should exist")
            .blob_fingerprint,
        snapshot
            .entry(&target)
            .expect("target entry should exist")
            .blob_fingerprint
    );
    assert_eq!(snapshot.stats.skipped_symlinks, 0);
    assert!(snapshot.diagnostics.is_empty());
}

#[cfg(unix)]
#[test]
fn worktree_follow_rejects_symlink_escapes_outside_the_repo_root() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new("repo-symlink-worktree-escape");
    let repo_root = temp.path().join("repo");
    fs::create_dir_all(repo_root.join(".git")).expect("git dir should exist");
    write_file(&temp.path().join("outside.txt"), b"outside");
    symlink("../outside.txt", repo_root.join("link.txt")).expect("symlink should exist");

    let options = repo::SnapshotOptions {
        symlink_policy: repo::SymlinkPolicy::Follow,
        ..repo::SnapshotOptions::default()
    };
    let error = repo::materialize_snapshot(&snapshot_request(
        &repo_root,
        repo::SnapshotSource::Worktree,
        options,
    ))
    .expect_err("escaping symlinks should fail");

    assert_eq!(
        error,
        repo::RepoError::SymlinkTargetEscape {
            path: crate::kernel::RepoPath::parse("link.txt").expect("path should parse"),
            target: "../outside.txt".to_owned(),
        }
    );
}

#[cfg(unix)]
#[test]
fn worktree_follow_detects_symlink_loops() {
    use std::os::unix::fs::symlink;

    let repo_root = TempDir::new("repo-symlink-worktree-loop");
    fs::create_dir_all(repo_root.path().join(".git")).expect("git dir should exist");
    symlink("b.txt", repo_root.path().join("a.txt")).expect("symlink should exist");
    symlink("a.txt", repo_root.path().join("b.txt")).expect("symlink should exist");

    let options = repo::SnapshotOptions {
        symlink_policy: repo::SymlinkPolicy::Follow,
        ..repo::SnapshotOptions::default()
    };
    let error = repo::materialize_snapshot(&snapshot_request(
        repo_root.path(),
        repo::SnapshotSource::Worktree,
        options,
    ))
    .expect_err("symlink loops should fail");

    assert_eq!(
        error,
        repo::RepoError::SymlinkTargetLoop {
            path: crate::kernel::RepoPath::parse("a.txt").expect("path should parse"),
            target: "a.txt".to_owned(),
        }
    );
}

#[cfg(unix)]
#[test]
fn gitrev_follow_materializes_committed_symlink_targets() {
    use std::os::unix::fs::symlink;

    let repo_root = init_git_repo("repo-symlink-gitrev-follow");
    write_file(&repo_root.path().join("dir/target.txt"), b"git-target");
    symlink("dir/target.txt", repo_root.path().join("link.txt")).expect("symlink should exist");
    commit_all(repo_root.path(), "initial");
    let head = git_stdout(repo_root.path(), &["rev-parse", "HEAD"]);

    let options = repo::SnapshotOptions {
        symlink_policy: repo::SymlinkPolicy::Follow,
        ..repo::SnapshotOptions::default()
    };
    let snapshot = materialize(
        repo_root.path(),
        repo::SnapshotSource::GitRev { rev: head },
        options,
    );
    let link = crate::kernel::RepoPath::parse("link.txt").expect("path should parse");
    let target = crate::kernel::RepoPath::parse("dir/target.txt").expect("path should parse");

    assert_eq!(
        snapshot.read_bytes(&link).expect("link bytes should exist"),
        b"git-target"
    );
    assert_eq!(
        snapshot
            .entry(&link)
            .expect("link entry should exist")
            .blob_fingerprint,
        snapshot
            .entry(&target)
            .expect("target entry should exist")
            .blob_fingerprint
    );
    assert_eq!(snapshot.stats.skipped_symlinks, 0);
    assert!(snapshot.diagnostics.is_empty());
}

#[cfg(unix)]
#[test]
fn gitrev_follow_rejects_dangling_symlink_targets() {
    use std::os::unix::fs::symlink;

    let repo_root = init_git_repo("repo-symlink-gitrev-dangling");
    symlink("missing.txt", repo_root.path().join("link.txt")).expect("symlink should exist");
    commit_all(repo_root.path(), "initial");
    let head = git_stdout(repo_root.path(), &["rev-parse", "HEAD"]);

    let options = repo::SnapshotOptions {
        symlink_policy: repo::SymlinkPolicy::Follow,
        ..repo::SnapshotOptions::default()
    };
    let error = repo::materialize_snapshot(&snapshot_request(
        repo_root.path(),
        repo::SnapshotSource::GitRev { rev: head },
        options,
    ))
    .expect_err("dangling symlinks should fail");

    assert_eq!(
        error,
        repo::RepoError::SymlinkTargetDangling {
            path: crate::kernel::RepoPath::parse("link.txt").expect("path should parse"),
            target: "missing.txt".to_owned(),
        }
    );
}
