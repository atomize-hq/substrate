use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
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

fn next_temp_nonce() -> u64 {
    static NEXT_TEMP_NONCE: AtomicU64 = AtomicU64::new(0);
    NEXT_TEMP_NONCE.fetch_add(1, Ordering::Relaxed)
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let nonce = next_temp_nonce();
        let path = std::env::temp_dir().join(format!(
            "substrate-lift-{prefix}-{}-{suffix}-{nonce}",
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
fn worktree_snapshot_survives_source_mutation_and_deletion() {
    let temp = TempDir::new("repo-purity-worktree");
    fs::create_dir_all(temp.path().join(".git")).expect("git dir should exist");
    write_file(
        &temp.path().join("Cargo.toml"),
        b"[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
    );
    write_file(
        &temp.path().join("src/lib.rs"),
        b"pub fn fixture_value() -> &'static str {\n    \"fixture\"\n}\n",
    );

    let snapshot = materialize(
        temp.path(),
        repo::SnapshotSource::Worktree,
        repo::SnapshotOptions::default(),
    );
    let path = crate::kernel::RepoPath::parse("src/lib.rs").expect("path should parse");
    let entry_before = snapshot.entry(&path).expect("entry should exist").clone();
    let bytes_before = snapshot
        .read_bytes(&path)
        .expect("bytes should exist")
        .to_vec();
    let fingerprint_before = snapshot.fingerprint.clone();

    fs::write(temp.path().join("src/lib.rs"), b"changed").expect("source should mutate");
    fs::remove_file(temp.path().join("Cargo.toml")).expect("source should delete");

    assert_eq!(snapshot.entry(&path), Some(&entry_before));
    assert_eq!(
        snapshot.read_bytes(&path).expect("bytes should remain"),
        bytes_before
    );
    assert_eq!(snapshot.fingerprint, fingerprint_before);
}

#[test]
fn gitrev_snapshot_survives_head_advance_and_worktree_mutation() {
    let repo_root = init_git_repo("repo-purity-gitrev");
    write_file(
        &repo_root.path().join("src/lib.rs"),
        b"pub fn fixture_value() -> &'static str {\n    \"first\"\n}\n",
    );
    commit_all(repo_root.path(), "first");
    let first_rev = git_stdout(repo_root.path(), &["rev-parse", "HEAD"]);

    let snapshot = materialize(
        repo_root.path(),
        repo::SnapshotSource::GitRev {
            rev: first_rev.clone(),
        },
        repo::SnapshotOptions::default(),
    );
    let path = crate::kernel::RepoPath::parse("src/lib.rs").expect("path should parse");
    let entry_before = snapshot.entry(&path).expect("entry should exist").clone();
    let bytes_before = snapshot
        .read_bytes(&path)
        .expect("bytes should exist")
        .to_vec();
    let fingerprint_before = snapshot.fingerprint.clone();

    write_file(
        &repo_root.path().join("src/lib.rs"),
        b"pub fn fixture_value() -> &'static str {\n    \"second\"\n}\n",
    );
    write_file(&repo_root.path().join("notes.txt"), b"new file");
    commit_all(repo_root.path(), "second");

    assert_eq!(
        snapshot.source,
        repo::SnapshotSource::GitRev { rev: first_rev }
    );
    assert_eq!(snapshot.entry(&path), Some(&entry_before));
    assert_eq!(
        snapshot.read_bytes(&path).expect("bytes should remain"),
        bytes_before
    );
    assert_eq!(snapshot.fingerprint, fingerprint_before);
}
