#![allow(unused_crate_dependencies)]

use std::fs;

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
#[path = "support/repo_support.rs"]
mod repo_support;

use repo_support::{copy_fixture_tree, default_snapshot_options, write_file, TempDir};

#[test]
fn identical_trees_under_different_absolute_roots_share_fingerprint() {
    let left = copy_fixture_tree(
        "fixtures/repo/trees/basic_worktree",
        "repo-fingerprint-left",
    );
    let right = copy_fixture_tree(
        "fixtures/repo/trees/basic_worktree",
        "repo-fingerprint-right",
    );

    let left_snapshot = repo_support::materialize(left.path(), default_snapshot_options());
    let right_snapshot = repo_support::materialize(right.path(), default_snapshot_options());

    assert_eq!(left_snapshot.fingerprint, right_snapshot.fingerprint);
}

#[test]
fn traversal_order_does_not_affect_inventory_order_or_fingerprint() {
    let left = TempDir::new("repo-fingerprint-order-left");
    fs::create_dir_all(left.path().join(".git")).expect("git dir");
    write_file(&left.path().join("b.txt"), b"b");
    write_file(&left.path().join("a.txt"), b"a");

    let right = TempDir::new("repo-fingerprint-order-right");
    fs::create_dir_all(right.path().join(".git")).expect("git dir");
    write_file(&right.path().join("a.txt"), b"a");
    write_file(&right.path().join("b.txt"), b"b");

    let left_snapshot = repo_support::materialize(left.path(), default_snapshot_options());
    let right_snapshot = repo_support::materialize(right.path(), default_snapshot_options());

    let left_paths = repo_support::inventory_paths(&left_snapshot);
    let right_paths = repo_support::inventory_paths(&right_snapshot);
    assert_eq!(left_paths, vec!["a.txt".to_owned(), "b.txt".to_owned()]);
    assert_eq!(left_paths, right_paths);
    assert_eq!(left_snapshot.fingerprint, right_snapshot.fingerprint);
}

#[test]
fn changing_one_file_changes_its_blob_and_snapshot_fingerprint_only() {
    let left = copy_fixture_tree(
        "fixtures/repo/trees/basic_worktree",
        "repo-fingerprint-before",
    );
    let right = copy_fixture_tree(
        "fixtures/repo/trees/basic_worktree",
        "repo-fingerprint-after",
    );
    write_file(
        &right.path().join("src/lib.rs"),
        b"pub fn fixture_value() -> &'static str {\n    \"changed\"\n}\n",
    );

    let left_snapshot = repo_support::materialize(left.path(), default_snapshot_options());
    let right_snapshot = repo_support::materialize(right.path(), default_snapshot_options());

    let changed = crate::kernel::RepoPath::parse("src/lib.rs").expect("path");
    let unchanged = crate::kernel::RepoPath::parse("Cargo.toml").expect("path");
    assert_ne!(
        left_snapshot
            .entry(&changed)
            .expect("entry")
            .blob_fingerprint,
        right_snapshot
            .entry(&changed)
            .expect("entry")
            .blob_fingerprint
    );
    assert_eq!(
        left_snapshot
            .entry(&unchanged)
            .expect("entry")
            .blob_fingerprint,
        right_snapshot
            .entry(&unchanged)
            .expect("entry")
            .blob_fingerprint
    );
    assert_ne!(left_snapshot.fingerprint, right_snapshot.fingerprint);
}

#[cfg(unix)]
#[test]
fn diagnostics_are_sorted_deterministically() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new("repo-fingerprint-diagnostics");
    fs::create_dir_all(temp.path().join(".git")).expect("git dir");
    write_file(&temp.path().join("b-large.txt"), b"1234567890");
    write_file(&temp.path().join("a-target.txt"), b"target");
    symlink("a-target.txt", temp.path().join("a-link.txt")).expect("symlink");

    let mut options = default_snapshot_options();
    options.max_file_bytes = Some(3);
    options.large_file_policy = repo::LargeFilePolicy::Skip;
    let snapshot = repo_support::materialize(temp.path(), options);

    let mut sorted = snapshot.diagnostics.clone();
    sorted.sort();
    assert_eq!(snapshot.diagnostics, sorted);
    let codes = snapshot
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        vec![
            "repo.snapshot.symlink_skipped".to_owned(),
            "repo.snapshot.file_too_large".to_owned(),
            "repo.snapshot.file_too_large".to_owned()
        ]
    );
}
