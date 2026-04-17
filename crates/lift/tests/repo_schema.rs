use assert_cmd as _;
use clap as _;
use globset as _;
use jsonschema::Validator;
use predicates as _;
use serde::{Deserialize, Serialize};
use serde_jcs as _;
use serde_json::Value;
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
    copy_fixture_tree, default_snapshot_options, load_json, load_text, manifest_from_snapshot,
};

#[derive(Debug, Deserialize, Serialize)]
struct FixtureManifest {
    version: u32,
    case: String,
    source_kind: String,
    options: FixtureOptions,
    files: Vec<FixtureFile>,
    snapshot_fingerprint: String,
    stats: FixtureStats,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixtureOptions {
    symlink_policy: String,
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
fn valid_fixture_manifests_validate_and_deserialize() {
    let validator = repo_support::snapshot_validator();
    let instance = load_json("fixtures/repo/valid/manifest_minimal.json");
    validator
        .validate(&instance)
        .expect("valid manifest should validate");
    let parsed: FixtureManifest =
        serde_json::from_value(instance).expect("fixture should deserialize");
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.source_kind, "worktree");
}

#[test]
fn invalid_fixture_manifests_fail_validation() {
    let validator = repo_support::snapshot_validator();
    for relative in [
        "fixtures/repo/invalid/manifest_bad_repo_path.json",
        "fixtures/repo/invalid/manifest_missing_stats.json",
    ] {
        let instance = load_json(relative);
        assert!(
            validator.validate(&instance).is_err(),
            "{relative} should fail validation"
        );
    }
}

#[test]
fn generated_snapshot_manifest_validates_and_preserves_runtime_invariants() {
    let validator: Validator = repo_support::snapshot_validator();
    let fixture = copy_fixture_tree("fixtures/repo/trees/basic_worktree", "repo-schema");
    let options = default_snapshot_options();
    let snapshot = repo_support::materialize(fixture.path(), options.clone());
    let manifest = manifest_from_snapshot("basic-worktree", &options, &snapshot);

    validator
        .validate(&manifest)
        .expect("generated manifest should validate");

    let parsed: FixtureManifest =
        serde_json::from_value(manifest.clone()).expect("generated manifest should deserialize");
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

    let manifest_value: Value = manifest;
    assert_eq!(
        manifest_value["snapshot_fingerprint"].as_str(),
        Some(snapshot.fingerprint.as_str())
    );
}
