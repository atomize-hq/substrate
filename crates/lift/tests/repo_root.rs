use std::collections::BTreeSet;
use std::fs;

use assert_cmd as _;
use clap as _;
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

use repo_support::{write_file, TempDir};

#[test]
fn nearest_marker_ancestor_wins_when_starting_from_a_file() {
    let temp = TempDir::new("repo-root-file-start");
    let outer = temp.path().join("outer");
    let inner = outer.join("inner");
    fs::create_dir_all(outer.join(".git")).expect("outer marker dir");
    fs::create_dir_all(&inner).expect("inner dir");
    write_file(&inner.join(".git"), b"gitdir: ../.git/worktrees/inner\n");
    let start = inner.join("src/main.rs");
    write_file(&start, b"fn main() {}\n");

    let root = repo::root::detect_repo_root(&start, &repo::RepoRootDetectionOptions::git_default())
        .expect("root should be detected");

    assert_eq!(
        root.as_path(),
        fs::canonicalize(&inner).expect("inner path should canonicalize")
    );
}

#[test]
fn directory_start_detects_root_with_git_directory_marker() {
    let temp = TempDir::new("repo-root-dir-start");
    let repo_dir = temp.path().join("repo");
    fs::create_dir_all(repo_dir.join(".git")).expect("git dir");
    fs::create_dir_all(repo_dir.join("nested/deeper")).expect("nested dir");

    let root = repo::root::detect_repo_root(
        &repo_dir.join("nested/deeper"),
        &repo::RepoRootDetectionOptions::git_default(),
    )
    .expect("root should be detected");

    assert_eq!(
        root.as_path(),
        fs::canonicalize(&repo_dir).expect("repo path should canonicalize")
    );
}

#[test]
fn ceiling_dir_stops_ascent_before_higher_match() {
    let temp = TempDir::new("repo-root-ceiling");
    let outer = temp.path().join("outer");
    let inner = outer.join("inner");
    fs::create_dir_all(outer.join(".git")).expect("git dir");
    fs::create_dir_all(inner.join("nested")).expect("nested dir");

    let error = repo::root::detect_repo_root(
        &inner.join("nested"),
        &repo::RepoRootDetectionOptions {
            markers: BTreeSet::from([repo::RootMarker::parse(".git").expect("marker")]),
            ceiling_dir: Some(inner.clone()),
        },
    )
    .expect_err("ceiling dir should stop ascent");

    assert_eq!(
        error,
        repo::RepoError::RootNotFound {
            start_path: inner.join("nested").display().to_string(),
            markers: vec![".git".to_owned()],
        }
    );
}

#[test]
fn marker_order_does_not_change_selected_root() {
    let temp = TempDir::new("repo-root-marker-order");
    let repo_dir = temp.path().join("repo");
    fs::create_dir_all(repo_dir.join(".git")).expect("git dir");
    fs::create_dir_all(repo_dir.join("nested")).expect("nested dir");
    write_file(
        &repo_dir.join("Cargo.toml"),
        b"[package]\nname = \"fixture\"\n",
    );
    let start = repo_dir.join("nested/file.txt");
    write_file(&start, b"fixture");

    let first = repo::root::detect_repo_root(
        &start,
        &repo::RepoRootDetectionOptions {
            markers: BTreeSet::from([
                repo::RootMarker::parse(".git").expect("marker"),
                repo::RootMarker::parse("Cargo.toml").expect("marker"),
            ]),
            ceiling_dir: None,
        },
    )
    .expect("root should be detected");
    let second = repo::root::detect_repo_root(
        &start,
        &repo::RepoRootDetectionOptions {
            markers: BTreeSet::from([
                repo::RootMarker::parse("Cargo.toml").expect("marker"),
                repo::RootMarker::parse(".git").expect("marker"),
            ]),
            ceiling_dir: None,
        },
    )
    .expect("root should be detected");

    assert_eq!(first, second);
    assert_eq!(
        first.as_path(),
        fs::canonicalize(&repo_dir).expect("repo path should canonicalize")
    );
}

#[test]
fn missing_start_path_and_missing_root_emit_typed_failures() {
    let temp = TempDir::new("repo-root-errors");
    let missing = temp.path().join("missing");
    let missing_error =
        repo::root::detect_repo_root(&missing, &repo::RepoRootDetectionOptions::git_default())
            .expect_err("missing path should fail");
    assert_eq!(
        missing_error,
        repo::RepoError::StartPathNotFound {
            path: missing.display().to_string(),
        }
    );

    let existing = temp.path().join("plain");
    fs::create_dir_all(&existing).expect("plain dir");
    let root_error =
        repo::root::detect_repo_root(&existing, &repo::RepoRootDetectionOptions::git_default())
            .expect_err("marker-less dir should fail");
    assert_eq!(
        root_error,
        repo::RepoError::RootNotFound {
            start_path: existing.display().to_string(),
            markers: vec![".git".to_owned()],
        }
    );
}
