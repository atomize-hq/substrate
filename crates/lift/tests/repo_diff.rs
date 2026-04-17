use assert_cmd as _;
use clap as _;
use globset as _;
use predicates as _;
use serde_jcs as _;
use sha2 as _;
use substrate_lift as _;
use thiserror as _;
use toml as _;
use walkdir as _;

use std::fs;

mod kernel {
    pub(crate) use substrate_lift::kernel::*;
}
#[path = "../src/repo/mod.rs"]
mod repo;
#[path = "support/repo_support.rs"]
mod repo_support;

use repo_support::{
    default_snapshot_options, materialize, materialize_basic_worktree_pair, write_file,
};

fn diff_paths(diff: &repo::RepoDiff) -> Vec<String> {
    diff.entries
        .iter()
        .map(|entry| entry.path.as_str().to_owned())
        .collect()
}

#[test]
fn base_exhausted_first_marks_remaining_entries_as_added() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) = materialize_basic_worktree_pair(
        "repo-diff-base-exhausted",
        |_| {},
        |head| {
            write_file(&head.join("zzz/added-a.txt"), b"added-a");
            write_file(&head.join("zzz/added-b.txt"), b"added-b");
        },
    );

    let diff = repo::build_diff(&base_snapshot, &head_snapshot);

    assert_eq!(
        diff_paths(&diff),
        vec!["zzz/added-a.txt", "zzz/added-b.txt"]
    );
    assert!(diff
        .entries
        .iter()
        .all(|entry| entry.kind == repo::DiffKind::Added));
    assert!(diff.entries.iter().all(|entry| entry.before.is_none()));
    assert!(diff.entries.iter().all(|entry| entry.after.is_some()));
}

#[test]
fn head_exhausted_first_marks_remaining_entries_as_removed() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) = materialize_basic_worktree_pair(
        "repo-diff-head-exhausted",
        |_| {},
        |head| {
            fs::remove_file(head.join("src/lib.rs")).expect("src/lib.rs should be removable");
            fs::remove_file(head.join("target/cache.txt"))
                .expect("target/cache.txt should be removable");
        },
    );

    let diff = repo::build_diff(&base_snapshot, &head_snapshot);

    assert_eq!(diff_paths(&diff), vec!["src/lib.rs", "target/cache.txt"]);
    assert!(diff
        .entries
        .iter()
        .all(|entry| entry.kind == repo::DiffKind::Removed));
    assert!(diff.entries.iter().all(|entry| entry.before.is_some()));
    assert!(diff.entries.iter().all(|entry| entry.after.is_none()));
}

#[test]
fn equal_path_and_equal_blob_entries_are_omitted() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) = materialize_basic_worktree_pair(
        "repo-diff-omit-equal",
        |_| {},
        |head| {
            write_file(
                &head.join("src/lib.rs"),
                b"pub fn fixture_value() -> &'static str {\n    \"changed\"\n}\n",
            );
        },
    );

    let diff = repo::build_diff(&base_snapshot, &head_snapshot);
    let paths = diff_paths(&diff);

    assert_eq!(paths, vec!["src/lib.rs"]);
    assert!(!paths.iter().any(|path| path == "Cargo.toml"));
    assert!(!paths.iter().any(|path| path == "build/output.txt"));
}

#[test]
fn equal_path_and_different_blob_produces_single_modified_entry() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) = materialize_basic_worktree_pair(
        "repo-diff-modified",
        |_| {},
        |head| {
            write_file(
                &head.join("src/lib.rs"),
                b"pub fn fixture_value() -> &'static str {\n    \"modified\"\n}\n",
            );
        },
    );

    let diff = repo::build_diff(&base_snapshot, &head_snapshot);

    assert_eq!(diff.entries.len(), 1);
    let entry = &diff.entries[0];
    assert_eq!(entry.path.as_str(), "src/lib.rs");
    assert_eq!(entry.kind, repo::DiffKind::Modified);
    assert!(entry.before.is_some());
    assert!(entry.after.is_some());

    let before = entry
        .before
        .as_ref()
        .expect("modified entry should have before");
    let after = entry
        .after
        .as_ref()
        .expect("modified entry should have after");
    assert_eq!(before.path, after.path);
    assert_ne!(before.blob_fingerprint, after.blob_fingerprint);
}

#[test]
fn identical_snapshots_produce_empty_diff() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) =
        materialize_basic_worktree_pair("repo-diff-identical", |_| {}, |_| {});

    let diff = repo::build_diff(&base_snapshot, &head_snapshot);

    assert!(diff.entries.is_empty());
    assert_eq!(diff.base_fingerprint, diff.head_fingerprint);
}

#[test]
fn rename_shaped_change_is_removed_plus_added() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) = materialize_basic_worktree_pair(
        "repo-diff-rename-shaped",
        |_| {},
        |head| {
            fs::rename(head.join("src/lib.rs"), head.join("src/renamed.rs"))
                .expect("rename should succeed");
        },
    );

    let diff = repo::build_diff(&base_snapshot, &head_snapshot);

    assert_eq!(diff_paths(&diff), vec!["src/lib.rs", "src/renamed.rs"]);
    assert_eq!(diff.entries[0].kind, repo::DiffKind::Removed);
    assert_eq!(diff.entries[1].kind, repo::DiffKind::Added);

    let removed = diff.entries[0]
        .before
        .as_ref()
        .expect("removed entry should have before");
    let added = diff.entries[1]
        .after
        .as_ref()
        .expect("added entry should have after");
    assert_eq!(removed.blob_fingerprint, added.blob_fingerprint);
}

#[test]
fn diff_entry_path_order_matches_lexical_repo_path_order() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) = materialize_basic_worktree_pair(
        "repo-diff-lexical-order",
        |_| {},
        |head| {
            write_file(&head.join("docs/new.txt"), b"new");
            write_file(
                &head.join("src/lib.rs"),
                b"pub fn fixture_value() -> &'static str {\n    \"ordered\"\n}\n",
            );
            fs::remove_file(head.join("target/cache.txt"))
                .expect("target/cache.txt should be removable");
            write_file(&head.join("zeta/final.txt"), b"final");
        },
    );

    let diff = repo::build_diff(&base_snapshot, &head_snapshot);
    let paths = diff_paths(&diff);
    let mut sorted_paths = paths.clone();
    sorted_paths.sort();

    assert_eq!(
        paths,
        vec![
            "docs/new.txt",
            "src/lib.rs",
            "target/cache.txt",
            "zeta/final.txt",
        ]
    );
    assert_eq!(paths, sorted_paths);
}

#[test]
fn repeated_build_diff_calls_are_fingerprint_stable() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) = materialize_basic_worktree_pair(
        "repo-diff-repeatable",
        |_| {},
        |head| {
            write_file(&head.join("docs/new.txt"), b"new");
            write_file(
                &head.join("src/lib.rs"),
                b"pub fn fixture_value() -> &'static str {\n    \"repeatable\"\n}\n",
            );
        },
    );

    let first = repo::build_diff(&base_snapshot, &head_snapshot);
    let second = repo::build_diff(&base_snapshot, &head_snapshot);

    assert_eq!(first, second);
    assert_eq!(first.fingerprint, second.fingerprint);
}

#[test]
fn equivalent_trees_under_different_temp_roots_produce_the_same_semantic_diff() {
    let (_base_root_a, _head_root_a, base_snapshot_a, head_snapshot_a) =
        materialize_basic_worktree_pair(
            "repo-diff-semantic-a",
            |_| {},
            |head| {
                write_file(&head.join("docs/new.txt"), b"new");
                write_file(
                    &head.join("src/lib.rs"),
                    b"pub fn fixture_value() -> &'static str {\n    \"semantic\"\n}\n",
                );
            },
        );
    let (_base_root_b, _head_root_b, base_snapshot_b, head_snapshot_b) =
        materialize_basic_worktree_pair(
            "repo-diff-semantic-b",
            |_| {},
            |head| {
                write_file(&head.join("docs/new.txt"), b"new");
                write_file(
                    &head.join("src/lib.rs"),
                    b"pub fn fixture_value() -> &'static str {\n    \"semantic\"\n}\n",
                );
            },
        );

    let diff_a = repo::build_diff(&base_snapshot_a, &head_snapshot_a);
    let diff_b = repo::build_diff(&base_snapshot_b, &head_snapshot_b);

    assert_eq!(diff_a, diff_b);
}

#[test]
fn prebuilt_snapshots_remain_stable_after_later_live_tree_mutation() {
    let (base_root, head_root, base_snapshot, head_snapshot) =
        materialize_basic_worktree_pair("repo-diff-prebuilt-stable", |_| {}, |_| {});

    let original_diff = repo::build_diff(&base_snapshot, &head_snapshot);
    assert!(original_diff.entries.is_empty());

    write_file(
        &base_root.path().join("Cargo.toml"),
        b"[package]\nname = \"mutated-base\"\n",
    );
    write_file(&head_root.path().join("docs/new.txt"), b"mutated-head");

    let preserved_diff = repo::build_diff(&base_snapshot, &head_snapshot);
    assert_eq!(preserved_diff, original_diff);

    let fresh_base = materialize(base_root.path(), default_snapshot_options());
    let fresh_head = materialize(head_root.path(), default_snapshot_options());
    let fresh_diff = repo::build_diff(&fresh_base, &fresh_head);
    assert_ne!(fresh_diff, original_diff);
}
