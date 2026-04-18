#![allow(unused_crate_dependencies)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd as _;
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
            std::process::id(),
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

fn inventory_paths(snapshot: &repo::RepoSnapshot) -> Vec<String> {
    snapshot
        .inventory
        .iter()
        .map(|entry| entry.path.as_str().to_owned())
        .collect()
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

#[test]
fn gitrev_snapshot_reads_the_committed_tree_not_dirty_worktree_state() {
    let repo_root = init_git_repo("repo-gitrev-dirty-worktree");
    write_file(
        &repo_root.path().join("Cargo.toml"),
        b"[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
    );
    write_file(
        &repo_root.path().join("src/lib.rs"),
        b"pub fn fixture_value() -> &'static str {\n    \"committed\"\n}\n",
    );
    commit_all(repo_root.path(), "initial");
    let head = git_stdout(repo_root.path(), &["rev-parse", "HEAD"]);

    write_file(
        &repo_root.path().join("src/lib.rs"),
        b"pub fn fixture_value() -> &'static str {\n    \"dirty\"\n}\n",
    );
    write_file(&repo_root.path().join("untracked.txt"), b"worktree-only");

    let snapshot = materialize(
        repo_root.path(),
        repo::SnapshotSource::GitRev { rev: head.clone() },
        repo::SnapshotOptions::default(),
    );
    let paths = inventory_paths(&snapshot);

    assert_eq!(snapshot.source, repo::SnapshotSource::GitRev { rev: head });
    assert_eq!(paths, vec!["Cargo.toml", "src/lib.rs"]);
    assert!(
        !paths.iter().any(|path| path == "untracked.txt"),
        "git revision snapshots must ignore untracked worktree files"
    );
    assert_eq!(
        snapshot
            .read_bytes(&crate::kernel::RepoPath::parse("src/lib.rs").expect("path should parse"))
            .expect("blob bytes should exist"),
        b"pub fn fixture_value() -> &'static str {\n    \"committed\"\n}\n"
    );
}

#[test]
fn clean_worktree_and_gitrev_snapshots_have_matching_inventory_and_fingerprint() {
    let repo_root = init_git_repo("repo-gitrev-parity");
    write_file(
        &repo_root.path().join("Cargo.toml"),
        b"[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
    );
    write_file(
        &repo_root.path().join("src/lib.rs"),
        b"pub fn fixture_value() -> &'static str {\n    \"same\"\n}\n",
    );
    write_file(&repo_root.path().join("docs/guide.md"), b"# guide\n");
    commit_all(repo_root.path(), "initial");
    let head = git_stdout(repo_root.path(), &["rev-parse", "HEAD"]);

    let worktree = materialize(
        repo_root.path(),
        repo::SnapshotSource::Worktree,
        repo::SnapshotOptions::default(),
    );
    let gitrev = materialize(
        repo_root.path(),
        repo::SnapshotSource::GitRev { rev: head },
        repo::SnapshotOptions::default(),
    );

    assert_eq!(worktree.inventory, gitrev.inventory);
    assert_eq!(worktree.blob_store, gitrev.blob_store);
    assert_eq!(worktree.fingerprint, gitrev.fingerprint);
    assert_eq!(worktree.stats.file_count, gitrev.stats.file_count);
    assert_eq!(worktree.stats.total_bytes, gitrev.stats.total_bytes);
    assert_eq!(worktree.stats.skipped_by_ignore, 1);
    assert_eq!(gitrev.stats.skipped_by_ignore, 0);
    assert_eq!(
        worktree.stats.skipped_symlinks,
        gitrev.stats.skipped_symlinks
    );
    assert_eq!(
        worktree.stats.skipped_non_utf8_paths,
        gitrev.stats.skipped_non_utf8_paths
    );
    assert_eq!(
        worktree.stats.skipped_large_files,
        gitrev.stats.skipped_large_files
    );
    assert_eq!(
        worktree.stats.skipped_unsupported_file_kinds,
        gitrev.stats.skipped_unsupported_file_kinds
    );
    assert_eq!(worktree.diagnostics, gitrev.diagnostics);
}

#[test]
fn gitrev_snapshot_rejects_revisions_that_resolve_to_blobs() {
    let repo_root = init_git_repo("repo-gitrev-object-kind");
    write_file(
        &repo_root.path().join("src/lib.rs"),
        b"pub fn fixture_value() -> &'static str {\n    \"blob\"\n}\n",
    );
    commit_all(repo_root.path(), "initial");

    let error = repo::materialize_snapshot(&snapshot_request(
        repo_root.path(),
        repo::SnapshotSource::GitRev {
            rev: "HEAD:src/lib.rs".to_owned(),
        },
        repo::SnapshotOptions::default(),
    ))
    .expect_err("blob revisions should be rejected");

    assert_eq!(
        error,
        repo::RepoError::GitRevisionObjectKind {
            rev: "HEAD:src/lib.rs".to_owned(),
            actual_kind: "blob".to_owned(),
            expected_kind: "tree",
        }
    );
}

#[cfg(unix)]
#[test]
fn gitrev_followed_symlinks_still_compose_with_typed_excludes() {
    use std::os::unix::fs::symlink;

    let repo_root = init_git_repo("repo-gitrev-follow-ignore-typed");
    write_file(&repo_root.path().join("target/cache.txt"), b"cached");
    symlink("target/cache.txt", repo_root.path().join("link.txt")).expect("symlink should exist");
    commit_all(repo_root.path(), "initial");
    let head = git_stdout(repo_root.path(), &["rev-parse", "HEAD"]);

    let options = repo::SnapshotOptions {
        symlink_policy: repo::SymlinkPolicy::Follow,
        well_known_excludes: vec![repo::WellKnownExclude::RustTarget],
        ..repo::SnapshotOptions::default()
    };
    let snapshot = materialize(
        repo_root.path(),
        repo::SnapshotSource::GitRev { rev: head },
        options,
    );
    let link = crate::kernel::RepoPath::parse("link.txt").expect("path should parse");

    assert!(snapshot.entry(&link).is_none());
    assert_eq!(snapshot.stats.skipped_by_ignore, 2);
    assert_eq!(snapshot.stats.file_count, 0);
    assert!(snapshot.diagnostics.is_empty());
}

#[cfg(unix)]
#[test]
fn gitrev_followed_symlinks_compose_with_directory_glob_excludes() {
    use std::os::unix::fs::symlink;

    let repo_root = init_git_repo("repo-gitrev-follow-ignore-dir-glob");
    write_file(&repo_root.path().join("dist/bundle.js"), b"bundle");
    symlink("dist/bundle.js", repo_root.path().join("link.txt")).expect("symlink should exist");
    commit_all(repo_root.path(), "initial");
    let head = git_stdout(repo_root.path(), &["rev-parse", "HEAD"]);

    let options = repo::SnapshotOptions {
        symlink_policy: repo::SymlinkPolicy::Follow,
        exclude_globs: vec!["dist".to_owned()],
        ..repo::SnapshotOptions::default()
    };
    let snapshot = materialize(
        repo_root.path(),
        repo::SnapshotSource::GitRev { rev: head },
        options,
    );
    let link = crate::kernel::RepoPath::parse("link.txt").expect("path should parse");

    assert!(snapshot.entry(&link).is_none());
    assert!(inventory_paths(&snapshot)
        .into_iter()
        .all(|path| !path.starts_with("dist/")));
    assert_eq!(snapshot.stats.skipped_by_ignore, 2);
    assert_eq!(snapshot.stats.file_count, 0);
    assert!(snapshot.diagnostics.is_empty());
}

#[cfg(unix)]
#[test]
fn gitrev_followed_symlinks_compose_with_directory_slash_glob_excludes() {
    use std::os::unix::fs::symlink;

    let repo_root = init_git_repo("repo-gitrev-follow-ignore-dir-slash-glob");
    write_file(&repo_root.path().join("dist/bundle.js"), b"bundle");
    symlink("dist/bundle.js", repo_root.path().join("link.txt")).expect("symlink should exist");
    commit_all(repo_root.path(), "initial");
    let head = git_stdout(repo_root.path(), &["rev-parse", "HEAD"]);

    let options = repo::SnapshotOptions {
        symlink_policy: repo::SymlinkPolicy::Follow,
        exclude_globs: vec!["dist/".to_owned()],
        ..repo::SnapshotOptions::default()
    };
    let snapshot = materialize(
        repo_root.path(),
        repo::SnapshotSource::GitRev { rev: head },
        options,
    );
    let link = crate::kernel::RepoPath::parse("link.txt").expect("path should parse");

    assert!(snapshot.entry(&link).is_none());
    assert!(inventory_paths(&snapshot)
        .into_iter()
        .all(|path| !path.starts_with("dist/")));
    assert_eq!(snapshot.stats.skipped_by_ignore, 2);
    assert_eq!(snapshot.stats.file_count, 0);
    assert!(snapshot.diagnostics.is_empty());
}
