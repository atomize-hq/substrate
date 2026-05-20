#![allow(unused_crate_dependencies)]

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

use repo_support::{copy_fixture_tree, default_snapshot_options, inventory_paths, write_file};
#[cfg(unix)]
use repo_support::TempDir;

#[test]
fn invalid_caller_glob_hard_fails() {
    let mut options = default_snapshot_options();
    options.exclude_globs = vec!["[".to_owned()];
    let error = repo::CompiledIgnoreSet::compile(&options).expect_err("glob should fail");
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
fn explicit_glob_exclusion_still_composes_on_top_of_typed_policy() {
    let fixture = copy_fixture_tree(
        "fixtures/repo/trees/well_known_excludes",
        "repo-ignore-explicit",
    );
    write_file(&fixture.path().join("target/cache.txt"), b"cached");
    let mut options = default_snapshot_options();
    options.well_known_excludes = vec![repo::WellKnownExclude::RustTarget];
    options.exclude_globs = vec!["dist".to_owned()];

    let snapshot = repo_support::materialize(fixture.path(), options);
    let paths = inventory_paths(&snapshot);

    assert!(!paths.iter().any(|path| path.starts_with("dist/")));
    assert!(!paths.iter().any(|path| path.starts_with("target/")));
    assert!(paths.contains(&"build/output.txt".to_owned()));
    assert!(paths.contains(&"node_modules/pkg/index.js".to_owned()));
}

#[test]
fn common_cache_and_vendor_dirs_are_not_implicitly_excluded() {
    let fixture = copy_fixture_tree(
        "fixtures/repo/trees/well_known_excludes",
        "repo-ignore-defaults",
    );
    write_file(&fixture.path().join("__pycache__/module.pyc"), b"pyc");
    write_file(&fixture.path().join("target/cache.txt"), b"cached");
    let snapshot = repo_support::materialize(fixture.path(), default_snapshot_options());
    let paths = inventory_paths(&snapshot);

    assert!(paths.contains(&".venv/bin/python".to_owned()));
    assert!(paths.contains(&"__pycache__/module.pyc".to_owned()));
    assert!(paths.contains(&"build/output.txt".to_owned()));
    assert!(paths.contains(&"dist/bundle.js".to_owned()));
    assert!(paths.contains(&"node_modules/pkg/index.js".to_owned()));
    assert!(paths.contains(&"target/cache.txt".to_owned()));
    assert!(paths.contains(&"venv/bin/python".to_owned()));
    assert!(paths.iter().all(|path| !path.contains('\\')));
}

#[test]
fn typed_well_known_excludes_remove_only_the_selected_canonical_directories() {
    let fixture = copy_fixture_tree(
        "fixtures/repo/trees/well_known_excludes",
        "repo-ignore-well-known",
    );
    write_file(&fixture.path().join("__pycache__/module.pyc"), b"pyc");
    write_file(&fixture.path().join("target/cache.txt"), b"cached");
    let mut options = default_snapshot_options();
    options.well_known_excludes = vec![
        repo::WellKnownExclude::RustTarget,
        repo::WellKnownExclude::NodeModules,
        repo::WellKnownExclude::PythonHiddenVenv,
        repo::WellKnownExclude::PythonVenv,
        repo::WellKnownExclude::PythonPycache,
        repo::WellKnownExclude::WebDist,
        repo::WellKnownExclude::WebBuild,
    ];

    let snapshot = repo_support::materialize(fixture.path(), options);
    let paths = inventory_paths(&snapshot);

    assert!(!paths.iter().any(|path| path.starts_with(".venv/")));
    assert!(!paths.iter().any(|path| path.starts_with("__pycache__/")));
    assert!(!paths.iter().any(|path| path.starts_with("build/")));
    assert!(!paths.iter().any(|path| path.starts_with("dist/")));
    assert!(!paths.iter().any(|path| path.starts_with("node_modules/")));
    assert!(!paths.iter().any(|path| path.starts_with("target/")));
    assert!(!paths.iter().any(|path| path.starts_with("venv/")));
    assert!(paths.contains(&"src/lib.rs".to_owned()));
}

#[cfg(unix)]
#[test]
fn followed_symlinks_still_compose_with_caller_glob_excludes() {
    use std::os::unix::fs::symlink;

    let repo_root = TempDir::new("repo-ignore-follow-glob");
    std::fs::create_dir_all(repo_root.path().join(".git")).expect("git dir should exist");
    write_file(&repo_root.path().join("dist/bundle.js"), b"bundle");
    symlink("dist/bundle.js", repo_root.path().join("link.txt")).expect("symlink should exist");

    let mut options = default_snapshot_options();
    options.symlink_policy = repo::SymlinkPolicy::Follow;
    options.exclude_globs = vec!["dist/**".to_owned()];

    let snapshot = repo_support::materialize(repo_root.path(), options);
    let paths = inventory_paths(&snapshot);

    assert!(!paths.iter().any(|path| path == "link.txt"));
    assert!(!paths.iter().any(|path| path.starts_with("dist/")));
    assert_eq!(snapshot.stats.file_count, 0);
    assert!(snapshot.stats.skipped_by_ignore >= 2);
    assert!(snapshot.diagnostics.is_empty());
}

#[cfg(unix)]
#[test]
fn followed_symlinks_compose_with_directory_glob_excludes() {
    use std::os::unix::fs::symlink;

    let repo_root = TempDir::new("repo-ignore-follow-dir-glob");
    std::fs::create_dir_all(repo_root.path().join(".git")).expect("git dir should exist");
    write_file(&repo_root.path().join("dist/bundle.js"), b"bundle");
    symlink("dist/bundle.js", repo_root.path().join("link.txt")).expect("symlink should exist");

    let mut options = default_snapshot_options();
    options.symlink_policy = repo::SymlinkPolicy::Follow;
    options.exclude_globs = vec!["dist".to_owned()];

    let snapshot = repo_support::materialize(repo_root.path(), options);
    let paths = inventory_paths(&snapshot);

    assert!(!paths.iter().any(|path| path == "link.txt"));
    assert!(!paths.iter().any(|path| path.starts_with("dist/")));
    assert_eq!(snapshot.stats.file_count, 0);
    assert_eq!(snapshot.stats.skipped_by_ignore, 3);
    assert!(snapshot.diagnostics.is_empty());
}

#[cfg(unix)]
#[test]
fn followed_symlinks_compose_with_directory_slash_glob_excludes() {
    use std::os::unix::fs::symlink;

    let repo_root = TempDir::new("repo-ignore-follow-dir-slash-glob");
    std::fs::create_dir_all(repo_root.path().join(".git")).expect("git dir should exist");
    write_file(&repo_root.path().join("dist/bundle.js"), b"bundle");
    symlink("dist/bundle.js", repo_root.path().join("link.txt")).expect("symlink should exist");

    let mut options = default_snapshot_options();
    options.symlink_policy = repo::SymlinkPolicy::Follow;
    options.exclude_globs = vec!["dist/".to_owned()];

    let snapshot = repo_support::materialize(repo_root.path(), options);
    let paths = inventory_paths(&snapshot);

    assert!(!paths.iter().any(|path| path == "link.txt"));
    assert!(!paths.iter().any(|path| path.starts_with("dist/")));
    assert_eq!(snapshot.stats.file_count, 0);
    assert_eq!(snapshot.stats.skipped_by_ignore, 3);
    assert!(snapshot.diagnostics.is_empty());
}
