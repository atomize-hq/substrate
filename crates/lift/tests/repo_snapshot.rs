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

use repo_support::{
    copy_fixture_tree, default_snapshot_options, inventory_paths, write_file, TempDir,
};

#[test]
fn snapshot_materializes_inventory_and_blob_store_for_regular_files() {
    let fixture = copy_fixture_tree("fixtures/repo/trees/basic_worktree", "repo-snapshot-basic");
    let snapshot = repo_support::materialize(fixture.path(), default_snapshot_options());
    let paths = inventory_paths(&snapshot);

    assert_eq!(snapshot.inventory.len(), snapshot.blob_store.len());
    assert_eq!(snapshot.inventory.len() as u64, snapshot.stats.file_count);
    assert!(paths.contains(&"Cargo.toml".to_owned()));
    assert!(paths.contains(&"src/lib.rs".to_owned()));
    assert_eq!(
        snapshot
            .read_bytes(&crate::kernel::RepoPath::parse("src/lib.rs").expect("path"))
            .expect("blob bytes should exist"),
        b"pub fn fixture_value() -> &'static str {\n    \"fixture\"\n}\n"
    );
}

#[test]
fn missing_blob_lookup_returns_typed_failure() {
    let fixture = copy_fixture_tree(
        "fixtures/repo/trees/basic_worktree",
        "repo-snapshot-missing",
    );
    let snapshot = repo_support::materialize(fixture.path(), default_snapshot_options());
    let missing = crate::kernel::RepoPath::parse("missing.txt").expect("path");

    assert_eq!(
        snapshot
            .read_bytes(&missing)
            .expect_err("missing blob should fail"),
        repo::RepoError::MissingBlob { path: missing }
    );
}

#[test]
fn large_file_policy_behaves_as_configured() {
    let temp = TempDir::new("repo-snapshot-large-file");
    fs::create_dir_all(temp.path().join(".git")).expect("git dir");
    write_file(&temp.path().join("big.bin"), b"1234567890");

    let mut error_options = default_snapshot_options();
    error_options.max_file_bytes = Some(3);
    error_options.large_file_policy = repo::LargeFilePolicy::Error;
    assert_eq!(
        repo::materialize_snapshot(&repo_support::snapshot_request(temp.path(), error_options))
            .expect_err("oversized file should error"),
        repo::RepoError::FileTooLarge {
            display_path: fs::canonicalize(temp.path().join("big.bin"))
                .expect("large file path should canonicalize")
                .display()
                .to_string(),
            size_bytes: 10,
            max_file_bytes: 3,
        }
    );

    let mut skip_options = default_snapshot_options();
    skip_options.max_file_bytes = Some(3);
    skip_options.large_file_policy = repo::LargeFilePolicy::Skip;
    let snapshot = repo_support::materialize(temp.path(), skip_options);
    assert!(inventory_paths(&snapshot).is_empty());
    assert_eq!(snapshot.stats.skipped_large_files, 1);
    assert_eq!(
        snapshot.diagnostics[0].code.as_str(),
        "repo.snapshot.file_too_large"
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_path_policy_errors_or_skips_deterministically() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new("repo-snapshot-non-utf8");
    fs::create_dir_all(temp.path().join(".git")).expect("git dir");
    let bad_name = OsString::from_vec(vec![0xff, b'.', b't', b'x', b't']);
    let bad_path = temp.path().join(bad_name);
    if fs::write(&bad_path, b"bad").is_err() {
        return;
    }

    let error = repo::materialize_snapshot(&repo_support::snapshot_request(
        temp.path(),
        default_snapshot_options(),
    ))
    .expect_err("non-utf8 path should error by default");
    match error {
        repo::RepoError::NonUtf8Path { .. } => {}
        other => panic!("unexpected error: {other:?}"),
    }

    let mut skip_options = default_snapshot_options();
    skip_options.non_utf8_path_policy = repo::NonUtf8PathPolicy::Skip;
    let snapshot = repo_support::materialize(temp.path(), skip_options);
    assert_eq!(snapshot.stats.skipped_non_utf8_paths, 1);
    assert!(snapshot.inventory.is_empty());
    assert_eq!(
        snapshot.diagnostics[0].code.as_str(),
        "repo.snapshot.non_utf8_path"
    );
}

#[cfg(unix)]
#[test]
fn symlinks_are_skipped_and_never_enter_inventory() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new("repo-snapshot-symlink");
    fs::create_dir_all(temp.path().join(".git")).expect("git dir");
    write_file(&temp.path().join("target.txt"), b"target");
    symlink("target.txt", temp.path().join("link.txt")).expect("symlink should be creatable");

    let snapshot = repo_support::materialize(temp.path(), default_snapshot_options());
    assert!(!inventory_paths(&snapshot).contains(&"link.txt".to_owned()));
    assert_eq!(snapshot.stats.skipped_symlinks, 1);
    assert_eq!(
        snapshot.diagnostics[0].code.as_str(),
        "repo.snapshot.symlink_skipped"
    );
}

#[cfg(unix)]
#[test]
fn unsupported_file_kinds_are_skipped_with_diagnostics() {
    use std::os::unix::net::UnixListener;

    let temp = TempDir::new("repo-snapshot-unsupported");
    fs::create_dir_all(temp.path().join(".git")).expect("git dir");
    let socket_path = temp.path().join("s.sock");
    let _listener = UnixListener::bind(&socket_path).expect("socket should bind");

    let snapshot = repo_support::materialize(temp.path(), default_snapshot_options());
    assert!(inventory_paths(&snapshot).is_empty());
    assert_eq!(snapshot.stats.skipped_unsupported_file_kinds, 1);
    assert_eq!(
        snapshot.diagnostics[0].code.as_str(),
        "repo.snapshot.unsupported_file_kind"
    );
}
