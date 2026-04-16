use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd as _;
use clap as _;
use jsonschema::Validator;
use predicates as _;
use serde as _;
use serde_jcs as _;
use serde_json::Value;
use sha2 as _;
use substrate_lift::kernel::{
    sha256_bytes, sha256_canonical_json, ByteSpan, Diagnostic, Fingerprint, JsonPointer, Locator,
    QueryId, RepoPath, StableId, PRIMITIVES_V1_SCHEMA_FILE, PRIMITIVES_V1_SCHEMA_ID,
    PRIMITIVES_V1_SCHEMA_JSON, PRIMITIVES_V1_SCHEMA_VERSION,
};
use thiserror as _;
use toml as _;

#[test]
fn stable_id_from_identity_is_deterministic() {
    let left = StableId::from_identity("file", "src/lib.rs");
    let right = StableId::from_identity("file", "src/lib.rs");

    assert_eq!(left, right);
    assert_eq!(
        left.as_str(),
        "file:sha256:13ec57d807bcff8c7ee2ffc89b3adfb999e5b183d93474f425ebbe2ce371c416"
    );
}

#[test]
fn typed_ids_reject_wrong_kinds() {
    let err = QueryId::parse(
        "file:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    )
    .expect_err("query ids must reject file ids");

    match err {
        substrate_lift::kernel::KernelError::InvalidStableId {
            expected_kind: Some(kind),
            ..
        } => assert_eq!(kind, QueryId::KIND),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn stable_id_and_fingerprint_round_trip_with_schema() {
    let stable_id = StableId::parse(
        "file:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    )
    .expect("fixture stable id should parse");
    let fingerprint = Fingerprint::parse(
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    )
    .expect("fixture fingerprint should parse");

    let stable_json = serde_json::to_value(&stable_id).expect("stable id should serialize");
    let fingerprint_json =
        serde_json::to_value(&fingerprint).expect("fingerprint should serialize");

    assert_schema_valid("stable_id", &stable_json);
    assert_schema_valid("fingerprint", &fingerprint_json);

    let stable_roundtrip: StableId =
        serde_json::from_value(stable_json).expect("stable id should deserialize");
    let fingerprint_roundtrip: Fingerprint =
        serde_json::from_value(fingerprint_json).expect("fingerprint should deserialize");

    assert_eq!(stable_roundtrip, stable_id);
    assert_eq!(fingerprint_roundtrip, fingerprint);
}

#[test]
fn canonical_json_is_deterministic_across_insertion_order() {
    let a = load_json("fixtures/kernel/canonical/object_a.json");
    let b = load_json("fixtures/kernel/canonical/object_b.json");
    let expected = load_text("fixtures/kernel/canonical/expected.json");
    let expected = expected.trim_end();

    let left = substrate_lift::kernel::canonical_json_string(&a).expect("canonical json");
    let right = substrate_lift::kernel::canonical_json_string(&b).expect("canonical json");

    assert_eq!(left, expected);
    assert_eq!(right, expected);
    assert_eq!(
        substrate_lift::kernel::canonical_json_bytes(&a).expect("canonical bytes"),
        expected.as_bytes()
    );
}

#[test]
fn canonical_json_hash_matches_fixture() {
    let value = load_json("fixtures/kernel/canonical/object_a.json");
    let expected_hash = load_text("fixtures/kernel/canonical/expected_sha256.txt");
    let expected_hash = expected_hash.trim_end();
    let expected_json = load_text("fixtures/kernel/canonical/expected.json");
    let expected_json = expected_json.trim_end();

    let fingerprint = sha256_canonical_json(&value).expect("canonical hash");
    assert_eq!(fingerprint.as_str(), format!("sha256:{expected_hash}"));
    assert_eq!(
        sha256_bytes(expected_json.as_bytes()).as_str(),
        format!("sha256:{expected_hash}")
    );
}

#[test]
fn valid_schema_fixtures_validate_and_round_trip() {
    for (fixture, definition) in valid_fixture_cases() {
        let instance = load_json(fixture);
        assert_schema_valid(definition, &instance);
    }

    let repo_path: RepoPath =
        serde_json::from_value(load_json("fixtures/kernel/valid/repo_path.json")).expect("path");
    assert_eq!(repo_path.as_str(), "src/lib.rs");

    let pointer: JsonPointer =
        serde_json::from_value(load_json("fixtures/kernel/valid/json_pointer_nested.json"))
            .expect("pointer");
    assert_eq!(pointer.as_str(), "/touch/crates_touched");

    let span: ByteSpan =
        serde_json::from_value(load_json("fixtures/kernel/valid/byte_span.json")).expect("span");
    assert_eq!(span.len(), 5);

    let locator: Locator =
        serde_json::from_value(load_json("fixtures/kernel/valid/locator_with_pointer.json"))
            .expect("locator");
    assert_eq!(
        locator
            .json_pointer
            .as_ref()
            .expect("pointer should exist")
            .as_str(),
        "/touch/crates_touched"
    );

    let diagnostic: Diagnostic =
        serde_json::from_value(load_json("fixtures/kernel/valid/diagnostic_full.json"))
            .expect("diagnostic");
    assert_eq!(diagnostic.code.as_str(), "kernel.schema.fixture_example");
    assert_eq!(diagnostic.related.len(), 1);
}

#[test]
fn invalid_schema_fixtures_fail_validation() {
    for (fixture, definition) in invalid_schema_fixture_cases() {
        let instance = load_json(fixture);
        assert_schema_invalid(definition, &instance);
    }
}

#[test]
fn runtime_only_invalid_fixtures_fail_deserialization() {
    assert!(serde_json::from_value::<JsonPointer>(load_json(
        "fixtures/kernel/invalid/json_pointer_malformed.json"
    ))
    .is_err());
    assert!(serde_json::from_value::<ByteSpan>(load_json(
        "fixtures/kernel/invalid/byte_span_reversed.json"
    ))
    .is_err());
}

#[test]
fn embedded_schema_matches_schema_fixture_on_disk() {
    let on_disk = load_text("schemas/kernel/primitives.v1.json");
    assert_eq!(PRIMITIVES_V1_SCHEMA_JSON, on_disk);
    assert_eq!(
        PRIMITIVES_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/kernel/primitives.v1.json"
    );
    assert_eq!(PRIMITIVES_V1_SCHEMA_FILE, "primitives.v1.json");
    assert_eq!(PRIMITIVES_V1_SCHEMA_VERSION, 1);
}

fn valid_fixture_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("fixtures/kernel/valid/repo_path.json", "repo_path"),
        (
            "fixtures/kernel/valid/json_pointer_root.json",
            "json_pointer",
        ),
        (
            "fixtures/kernel/valid/json_pointer_nested.json",
            "json_pointer",
        ),
        ("fixtures/kernel/valid/stable_id.json", "stable_id"),
        ("fixtures/kernel/valid/fingerprint.json", "fingerprint"),
        ("fixtures/kernel/valid/byte_span.json", "byte_span"),
        ("fixtures/kernel/valid/locator_path_only.json", "locator"),
        ("fixtures/kernel/valid/locator_with_span.json", "locator"),
        ("fixtures/kernel/valid/locator_with_pointer.json", "locator"),
        (
            "fixtures/kernel/valid/diagnostic_minimal.json",
            "diagnostic",
        ),
        ("fixtures/kernel/valid/diagnostic_full.json", "diagnostic"),
    ]
}

fn invalid_schema_fixture_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "fixtures/kernel/invalid/repo_path_absolute.json",
            "repo_path",
        ),
        (
            "fixtures/kernel/invalid/repo_path_backslash.json",
            "repo_path",
        ),
        (
            "fixtures/kernel/invalid/repo_path_dot_segment.json",
            "repo_path",
        ),
        (
            "fixtures/kernel/invalid/stable_id_bad_prefix.json",
            "stable_id",
        ),
        (
            "fixtures/kernel/invalid/stable_id_bad_hex.json",
            "stable_id",
        ),
        (
            "fixtures/kernel/invalid/fingerprint_bad_prefix.json",
            "fingerprint",
        ),
        (
            "fixtures/kernel/invalid/fingerprint_bad_hex.json",
            "fingerprint",
        ),
        (
            "fixtures/kernel/invalid/diagnostic_code_missing_namespace.json",
            "diagnostic",
        ),
        (
            "fixtures/kernel/invalid/related_location_empty_message.json",
            "related_location",
        ),
    ]
}

fn assert_schema_valid(definition: &str, instance: &Value) {
    let validator = validator_for(definition);
    if let Err(error) = validator.validate(instance) {
        panic!("expected fixture to validate against {definition}: {error}");
    }
}

fn assert_schema_invalid(definition: &str, instance: &Value) {
    let validator = validator_for(definition);
    assert!(
        validator.validate(instance).is_err(),
        "expected fixture to fail validation for {definition}"
    );
}

fn validator_for(definition: &str) -> Validator {
    let schema = load_json("schemas/kernel/primitives.v1.json");
    let wrapped = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$ref": format!("#/$defs/{definition}"),
        "$defs": schema["$defs"].clone(),
    });

    jsonschema::validator_for(&wrapped).expect("schema should compile")
}

fn load_json(relative: &str) -> Value {
    let path = crate_root().join(relative);
    let contents = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", path.display());
    });

    serde_json::from_str(&contents).unwrap_or_else(|err| {
        panic!("failed to parse {} as JSON: {err}", path.display());
    })
}

fn load_text(relative: &str) -> String {
    let path = crate_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", path.display());
    })
}

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
