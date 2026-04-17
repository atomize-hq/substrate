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

use repo_support::{copy_fixture_tree, default_snapshot_options};

#[test]
fn snapshot_survives_source_mutation_and_deletion() {
    let fixture = copy_fixture_tree("fixtures/repo/trees/basic_worktree", "repo-purity");
    let snapshot = repo_support::materialize(fixture.path(), default_snapshot_options());
    let path = crate::kernel::RepoPath::parse("src/lib.rs").expect("path");
    let entry_before = snapshot.entry(&path).expect("entry should exist").clone();
    let bytes_before = snapshot
        .read_bytes(&path)
        .expect("bytes should exist")
        .to_vec();
    let fingerprint_before = snapshot.fingerprint.clone();

    fs::write(fixture.path().join("src/lib.rs"), b"changed").expect("source should mutate");
    fs::remove_file(fixture.path().join("Cargo.toml")).expect("source should delete");

    assert_eq!(snapshot.entry(&path), Some(&entry_before));
    assert_eq!(
        snapshot.read_bytes(&path).expect("bytes should remain"),
        bytes_before
    );
    assert_eq!(snapshot.fingerprint, fingerprint_before);
}
