use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd as _;
use clap as _;
use jsonschema::{Retrieve, Uri, Validator};
use predicates as _;
use serde as _;
use serde_jcs as _;
use serde_json::Value;
use sha2 as _;
use substrate_lift as _;
use thiserror as _;
use toml as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::{
        sha256_bytes, sha256_canonical_json, DiagnosticCode, Fingerprint, JsonPointer, Severity,
    };
}
#[path = "../src/pack/mod.rs"]
mod pack;

#[test]
fn embedded_pack_schemas_match_disk() {
    assert_eq!(
        pack::PACK_COMMON_V1_SCHEMA_JSON,
        load_text("schemas/pack/common.v1.json")
    );
    assert_eq!(
        pack::PACK_PROFILE_V1_SCHEMA_JSON,
        load_text("schemas/pack/profile.v1.json")
    );
    assert_eq!(
        pack::PACK_COMMON_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/common.v1.json"
    );
    assert_eq!(
        pack::PACK_PROFILE_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/profile.v1.json"
    );
    assert_eq!(pack::PACK_COMMON_V1_SCHEMA_FILE, "common.v1.json");
    assert_eq!(pack::PACK_PROFILE_V1_SCHEMA_FILE, "profile.v1.json");
    assert_eq!(pack::PACK_COMMON_V1_SCHEMA_VERSION, 1);
    assert_eq!(pack::PACK_PROFILE_V1_SCHEMA_VERSION, 1);
}

#[test]
fn valid_pack_profile_schema_fixtures_validate_and_deserialize() {
    for fixture in [
        "fixtures/pack/valid/profile_minimal.json",
        "fixtures/pack/valid/profile_full.json",
    ] {
        let instance = load_json(fixture);
        assert_schema_valid(&instance);

        let raw: pack::raw::RawProfile =
            serde_json::from_value(instance).expect("fixture should deserialize");
        assert_eq!(raw.kind, pack::raw::PackKind::Profile);
    }
}

#[test]
fn invalid_pack_profile_schema_fixtures_fail_validation() {
    for fixture in [
        "fixtures/pack/invalid/profile_missing_name.json",
        "fixtures/pack/invalid/profile_empty_name.json",
        "fixtures/pack/invalid/profile_invalid_pack_ref.json",
        "fixtures/pack/invalid/profile_invalid_pack_ref_leading_dash.json",
        "fixtures/pack/invalid/profile_traversal_pack_ref.json",
        "fixtures/pack/invalid/profile_unknown_field.json",
    ] {
        let instance = load_json(fixture);
        assert_schema_invalid(&instance);
    }
}

fn assert_schema_valid(instance: &Value) {
    let validator = profile_validator();
    if let Err(error) = validator.validate(instance) {
        panic!("expected fixture to validate: {error}");
    }
}

fn assert_schema_invalid(instance: &Value) {
    let validator = profile_validator();
    assert!(
        validator.validate(instance).is_err(),
        "expected fixture to fail schema validation"
    );
}

fn profile_validator() -> Validator {
    let profile_schema = load_json("schemas/pack/profile.v1.json");
    let common_schema = load_json("schemas/pack/common.v1.json");
    let retriever = InMemoryRetriever {
        schemas: HashMap::from([
            (
                pack::PACK_COMMON_V1_SCHEMA_ID.to_owned(),
                common_schema.clone(),
            ),
            ("common.v1.json".to_owned(), common_schema),
        ]),
    };

    jsonschema::draft202012::options()
        .with_retriever(retriever)
        .build(&profile_schema)
        .expect("profile schema should compile")
}

#[derive(Clone, Debug)]
struct InMemoryRetriever {
    schemas: HashMap<String, Value>,
}

impl Retrieve for InMemoryRetriever {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.schemas
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| format!("schema not found: {uri}").into())
    }
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
