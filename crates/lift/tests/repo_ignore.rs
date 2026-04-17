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

use repo_support::{copy_fixture_tree, default_snapshot_options, inventory_paths};

#[test]
fn invalid_caller_glob_hard_fails() {
    let error = repo::CompiledIgnoreSet::compile(&["[".to_owned()]).expect_err("glob should fail");
    match error {
        repo::RepoError::IgnoreGlobCompile { pattern, .. } => assert_eq!(pattern, "["),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn git_directory_is_intrinsically_excluded_without_caller_input() {
    let fixture = copy_fixture_tree("fixtures/repo/trees/basic_worktree", "repo-ignore-git");
    let snapshot = repo_support::materialize(fixture.path(), default_snapshot_options());

    assert!(inventory_paths(&snapshot)
        .into_iter()
        .all(|path| !path.starts_with(".git")));
}

#[test]
fn explicit_glob_exclusion_removes_matching_subtrees() {
    let fixture = copy_fixture_tree("fixtures/repo/trees/basic_worktree", "repo-ignore-explicit");
    let mut options = default_snapshot_options();
    options.exclude_globs = vec!["dist".to_owned(), "target".to_owned()];

    let snapshot = repo_support::materialize(fixture.path(), options);
    let paths = inventory_paths(&snapshot);

    assert!(!paths.iter().any(|path| path.starts_with("dist/")));
    assert!(!paths.iter().any(|path| path.starts_with("target/")));
}

#[test]
fn common_cache_and_vendor_dirs_are_not_implicitly_excluded() {
    let fixture = copy_fixture_tree("fixtures/repo/trees/basic_worktree", "repo-ignore-defaults");
    let snapshot = repo_support::materialize(fixture.path(), default_snapshot_options());
    let paths = inventory_paths(&snapshot);

    assert!(paths.contains(&".venv/bin/python".to_owned()));
    assert!(paths.contains(&"build/output.txt".to_owned()));
    assert!(paths.contains(&"dist/bundle.js".to_owned()));
    assert!(paths.contains(&"node_modules/pkg/index.js".to_owned()));
    assert!(paths.contains(&"target/cache.txt".to_owned()));
    assert!(paths.iter().all(|path| !path.contains('\\')));
}
