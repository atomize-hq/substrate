#![allow(unused_crate_dependencies)]

use assert_cmd as _;
use gix as _;
use globset as _;
use jsonschema::Validator;
use predicates as _;
use serde::{Deserialize, Serialize};
use serde_jcs as _;
use serde_json::Value;
use sha2 as _;
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
    copy_fixture_tree, default_snapshot_options, load_json, load_text, manifest_from_diff,
    manifest_from_snapshot, manifest_from_snapshot_with_source,
};

#[derive(Debug, Deserialize, Serialize)]
struct FixtureManifest {
    version: u32,
    case: String,
    source_kind: String,
    source_rev: Option<String>,
    options: FixtureOptions,
    files: Vec<FixtureFile>,
    snapshot_fingerprint: String,
    stats: FixtureStats,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixtureOptions {
    symlink_policy: String,
    well_known_excludes: Vec<repo::WellKnownExclude>,
    exclude_globs: Vec<String>,
    non_utf8_path_policy: String,
    max_file_bytes: Option<u64>,
    large_file_policy: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixtureFile {
    path: String,
    file_id: String,
    blob_fingerprint: String,
    size_bytes: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixtureStats {
    file_count: u64,
    total_bytes: u64,
    skipped_by_ignore: u64,
    skipped_symlinks: u64,
    skipped_non_utf8_paths: u64,
    skipped_large_files: u64,
    skipped_unsupported_file_kinds: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiffFixtureManifest {
    version: u32,
    case: String,
    base_fingerprint: String,
    head_fingerprint: String,
    entries: Vec<DiffFixtureEntry>,
    diff_fingerprint: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiffFixtureEntry {
    path: String,
    kind: String,
    before_blob_fingerprint: Option<String>,
    after_blob_fingerprint: Option<String>,
}

fn assert_diff_entry_shape_invariant(entry: &DiffFixtureEntry, context: &str) {
    match entry.kind.as_str() {
        "added" => {
            assert!(
                entry.before_blob_fingerprint.is_none(),
                "{context} added entries must not include before_blob_fingerprint"
            );
            assert!(
                entry.after_blob_fingerprint.is_some(),
                "{context} added entries must include after_blob_fingerprint"
            );
        }
        "modified" => {
            assert!(
                entry.before_blob_fingerprint.is_some(),
                "{context} modified entries must include before_blob_fingerprint"
            );
            assert!(
                entry.after_blob_fingerprint.is_some(),
                "{context} modified entries must include after_blob_fingerprint"
            );
        }
        "removed" => {
            assert!(
                entry.before_blob_fingerprint.is_some(),
                "{context} removed entries must include before_blob_fingerprint"
            );
            assert!(
                entry.after_blob_fingerprint.is_none(),
                "{context} removed entries must not include after_blob_fingerprint"
            );
        }
        other => panic!("{context} should use lowercase diff kinds, got {other}"),
    }
}

fn assert_diff_manifest_shape_invariants(manifest: &DiffFixtureManifest, context: &str) {
    for entry in &manifest.entries {
        assert_diff_entry_shape_invariant(entry, context);
    }
}

fn assert_snapshot_manifest_shape_invariants(manifest: &FixtureManifest, context: &str) {
    match manifest.source_kind.as_str() {
        "worktree" => {
            assert!(
                manifest.source_rev.is_none(),
                "{context} worktree manifests must omit source_rev"
            );
        }
        "git_rev" => {
            let source_rev = manifest
                .source_rev
                .as_deref()
                .expect("git_rev manifests must include source_rev");
            assert!(
                !source_rev.is_empty(),
                "{context} git_rev manifests must use a non-empty source_rev"
            );
        }
        other => panic!("{context} should use a supported source_kind, got {other}"),
    }

    let unique_excludes = manifest
        .options
        .well_known_excludes
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        unique_excludes.len(),
        manifest.options.well_known_excludes.len(),
        "{context} should not repeat well_known_excludes"
    );
}

#[test]
fn embedded_snapshot_schema_matches_disk() {
    assert_eq!(
        repo::schema::SNAPSHOT_MANIFEST_V1_SCHEMA_JSON,
        load_text("schemas/repo/snapshot_manifest.v1.json")
    );
    assert_eq!(
        repo::schema::SNAPSHOT_MANIFEST_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/repo/snapshot_manifest.v1.json"
    );
    assert_eq!(
        repo::schema::SNAPSHOT_MANIFEST_V1_SCHEMA_FILE,
        "snapshot_manifest.v1.json"
    );
    assert_eq!(repo::schema::SNAPSHOT_MANIFEST_V1_SCHEMA_VERSION, 1);
}

#[test]
fn embedded_diff_schema_matches_disk() {
    assert_eq!(
        repo::schema::DIFF_MANIFEST_V1_SCHEMA_JSON,
        load_text("schemas/repo/diff_manifest.v1.json")
    );
    assert_eq!(
        repo::schema::DIFF_MANIFEST_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/repo/diff_manifest.v1.json"
    );
    assert_eq!(
        repo::schema::DIFF_MANIFEST_V1_SCHEMA_FILE,
        "diff_manifest.v1.json"
    );
    assert_eq!(repo::schema::DIFF_MANIFEST_V1_SCHEMA_VERSION, 1);
}

#[test]
fn valid_fixture_manifests_validate_and_deserialize() {
    let validator = repo_support::snapshot_validator();

    for relative in [
        "fixtures/repo/valid/manifest_follow_policy.json",
        "fixtures/repo/valid/manifest_git_rev.json",
        "fixtures/repo/valid/manifest_minimal.json",
    ] {
        let instance = load_json(relative);
        validator
            .validate(&instance)
            .unwrap_or_else(|_| panic!("{relative} should validate"));
        let parsed: FixtureManifest = serde_json::from_value(instance)
            .unwrap_or_else(|_| panic!("{relative} should deserialize"));
        assert_eq!(parsed.version, 1, "{relative} should be v1");
        assert_snapshot_manifest_shape_invariants(&parsed, relative);
    }
}

#[test]
fn valid_diff_fixture_manifests_validate_and_deserialize() {
    let validator = repo_support::diff_validator();

    for relative in [
        "fixtures/repo/diff/valid/added_file.json",
        "fixtures/repo/diff/valid/empty_diff.json",
        "fixtures/repo/diff/valid/modified_file.json",
        "fixtures/repo/diff/valid/removed_file.json",
        "fixtures/repo/diff/valid/rename_as_add_remove.json",
    ] {
        let instance = load_json(relative);
        validator
            .validate(&instance)
            .unwrap_or_else(|_| panic!("{relative} should validate"));
        let parsed: DiffFixtureManifest = serde_json::from_value(instance)
            .unwrap_or_else(|_| panic!("{relative} should deserialize"));
        assert_eq!(parsed.version, 1, "{relative} should be v1");
        assert_diff_manifest_shape_invariants(&parsed, relative);
    }
}

#[test]
fn invalid_fixture_manifests_fail_validation() {
    let validator = repo_support::snapshot_validator();
    for relative in [
        "fixtures/repo/invalid/manifest_bad_repo_path.json",
        "fixtures/repo/invalid/manifest_bad_well_known_exclude.json",
        "fixtures/repo/invalid/manifest_git_rev_missing_source_rev.json",
        "fixtures/repo/invalid/manifest_missing_stats.json",
        "fixtures/repo/invalid/manifest_worktree_with_source_rev.json",
    ] {
        let instance = load_json(relative);
        assert!(
            validator.validate(&instance).is_err(),
            "{relative} should fail validation"
        );
    }
}

#[test]
fn invalid_diff_fixture_manifests_fail_deterministically() {
    let validator = repo_support::diff_validator();

    for relative in [
        "fixtures/repo/diff/invalid/manifest_added_with_before_blob.json",
        "fixtures/repo/diff/invalid/manifest_bad_kind.json",
        "fixtures/repo/diff/invalid/manifest_bad_repo_path.json",
        "fixtures/repo/diff/invalid/manifest_before_after_shape_invalid.json",
        "fixtures/repo/diff/invalid/manifest_modified_missing_after_blob.json",
        "fixtures/repo/diff/invalid/manifest_modified_missing_before_blob.json",
        "fixtures/repo/diff/invalid/manifest_missing_diff_fingerprint.json",
        "fixtures/repo/diff/invalid/manifest_removed_with_after_blob.json",
    ] {
        let instance = load_json(relative);
        assert!(
            validator.validate(&instance).is_err(),
            "{relative} should fail validation"
        );
        assert!(
            validator.validate(&instance).is_err(),
            "{relative} should keep failing on repeated validation"
        );
    }
}

#[test]
fn generated_snapshot_manifest_validates_and_preserves_runtime_invariants() {
    let validator: Validator = repo_support::snapshot_validator();
    let fixture = copy_fixture_tree("fixtures/repo/trees/basic_worktree", "repo-schema");
    let mut options = default_snapshot_options();
    options.symlink_policy = repo::SymlinkPolicy::Follow;
    options.well_known_excludes = vec![
        repo::WellKnownExclude::RustTarget,
        repo::WellKnownExclude::WebDist,
    ];
    let snapshot = repo_support::materialize(fixture.path(), options.clone());
    let manifest = manifest_from_snapshot("basic-worktree", &options, &snapshot);

    validator
        .validate(&manifest)
        .expect("generated manifest should validate");

    let parsed: FixtureManifest =
        serde_json::from_value(manifest.clone()).expect("generated manifest should deserialize");
    assert_snapshot_manifest_shape_invariants(&parsed, "generated snapshot manifest");
    let manifest_paths = parsed
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let mut sorted_paths = manifest_paths.clone();
    sorted_paths.sort();
    assert_eq!(manifest_paths, sorted_paths);
    assert_eq!(parsed.stats.file_count as usize, parsed.files.len());
    assert_eq!(
        parsed.stats.total_bytes,
        parsed.files.iter().map(|file| file.size_bytes).sum::<u64>()
    );
    assert_eq!(
        parsed.snapshot_fingerprint,
        snapshot.fingerprint.as_str().to_owned()
    );
    assert_eq!(
        parsed.options.well_known_excludes,
        vec![
            repo::WellKnownExclude::RustTarget,
            repo::WellKnownExclude::WebDist,
        ]
    );

    let manifest_value: Value = manifest;
    assert_eq!(
        manifest_value["snapshot_fingerprint"].as_str(),
        Some(snapshot.fingerprint.as_str())
    );
    assert_eq!(manifest_value["source_kind"].as_str(), Some("worktree"));
    assert!(manifest_value.get("source_rev").is_none());
}

#[test]
fn generated_git_rev_manifest_shape_validates() {
    let validator: Validator = repo_support::snapshot_validator();
    let fixture = copy_fixture_tree("fixtures/repo/trees/basic_worktree", "repo-schema-git-rev");
    let mut options = default_snapshot_options();
    options.well_known_excludes = vec![repo::WellKnownExclude::NodeModules];
    let snapshot = repo_support::materialize(fixture.path(), options.clone());
    let source = repo::SnapshotSource::GitRev {
        rev: "HEAD~1".to_owned(),
    };
    let manifest =
        manifest_from_snapshot_with_source("basic-git-rev", &source, &options, &snapshot);

    validator
        .validate(&manifest)
        .expect("generated git_rev manifest should validate");

    let parsed: FixtureManifest =
        serde_json::from_value(manifest.clone()).expect("generated manifest should deserialize");
    assert_snapshot_manifest_shape_invariants(&parsed, "generated git_rev manifest");
    assert_eq!(parsed.source_kind, "git_rev");
    assert_eq!(parsed.source_rev.as_deref(), Some("HEAD~1"));
    assert_eq!(
        parsed.options.well_known_excludes,
        vec![repo::WellKnownExclude::NodeModules]
    );
    assert_eq!(manifest["source_kind"].as_str(), Some("git_rev"));
    assert_eq!(manifest["source_rev"].as_str(), Some("HEAD~1"));
}

#[test]
fn generated_diff_manifest_validates() {
    let validator: Validator = repo_support::diff_validator();
    let (_base_root, _head_root, base_snapshot, head_snapshot) =
        repo_support::materialize_basic_worktree_pair(
            "repo-diff-schema-generated",
            |_| {},
            |head| {
                repo_support::write_file(&head.join("docs/new.txt"), b"new");
                repo_support::write_file(
                    &head.join("src/lib.rs"),
                    b"pub fn fixture_value() -> &'static str {\n    \"schema\"\n}\n",
                );
            },
        );
    let diff = repo::build_diff(&base_snapshot, &head_snapshot);
    let manifest = manifest_from_diff("generated-diff", &diff);

    validator
        .validate(&manifest)
        .expect("generated diff manifest should validate");

    let parsed: DiffFixtureManifest =
        serde_json::from_value(manifest).expect("generated diff manifest should deserialize");
    assert_eq!(parsed.version, 1);
    assert_diff_manifest_shape_invariants(&parsed, "generated diff manifest");
}

#[test]
fn manifest_ordering_matches_runtime_ordering() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) =
        repo_support::materialize_basic_worktree_pair(
            "repo-diff-schema-ordering",
            |_| {},
            |head| {
                repo_support::write_file(&head.join("docs/new.txt"), b"new");
                repo_support::write_file(
                    &head.join("src/lib.rs"),
                    b"pub fn fixture_value() -> &'static str {\n    \"ordered\"\n}\n",
                );
                std::fs::remove_file(head.join("target/cache.txt"))
                    .expect("target/cache.txt should be removable");
            },
        );
    let diff = repo::build_diff(&base_snapshot, &head_snapshot);
    let manifest = manifest_from_diff("generated-ordering", &diff);

    let runtime_paths = diff
        .entries
        .iter()
        .map(|entry| entry.path.as_str().to_owned())
        .collect::<Vec<_>>();
    let manifest_paths = manifest["entries"]
        .as_array()
        .expect("entries should be an array")
        .iter()
        .map(|entry| {
            entry["path"]
                .as_str()
                .expect("path should be a string")
                .to_owned()
        })
        .collect::<Vec<_>>();

    assert_eq!(manifest_paths, runtime_paths);
}

#[test]
fn manifest_fingerprint_matches_runtime_fingerprint() {
    let (_base_root, _head_root, base_snapshot, head_snapshot) =
        repo_support::materialize_basic_worktree_pair(
            "repo-diff-schema-fingerprint",
            |_| {},
            |head| {
                repo_support::write_file(
                    &head.join("src/lib.rs"),
                    b"pub fn fixture_value() -> &'static str {\n    \"fingerprint\"\n}\n",
                );
            },
        );
    let diff = repo::build_diff(&base_snapshot, &head_snapshot);
    let manifest = manifest_from_diff("generated-fingerprint", &diff);

    assert_eq!(
        manifest["base_fingerprint"].as_str(),
        Some(diff.base_fingerprint.as_str())
    );
    assert_eq!(
        manifest["head_fingerprint"].as_str(),
        Some(diff.head_fingerprint.as_str())
    );
    assert_eq!(
        manifest["diff_fingerprint"].as_str(),
        Some(diff.fingerprint.as_str())
    );
}
