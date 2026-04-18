use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd as _;
use clap as _;
use gix as _;
use jsonschema::{Retrieve, Uri, Validator};
use predicates as _;
use serde::Deserialize;
use serde_jcs as _;
use serde_json::Value;
use sha2 as _;
use substrate_lift as _;
use toml as _;
use walkdir as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::*;
}

#[path = "../src/pack/mod.rs"]
mod pack;

#[allow(dead_code)]
mod lang {
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    pub(crate) use crate::pack::LanguageId;

    #[derive(Debug, Error, Clone, Eq, PartialEq)]
    pub(crate) enum LangError {
        #[error("invalid adapter name")]
        InvalidAdapterName { input: String },
    }

    pub(crate) type LangResult<T> = Result<T, LangError>;

    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
    #[serde(try_from = "String", into = "String")]
    pub(crate) struct AdapterName(String);

    impl AdapterName {
        pub(crate) fn parse(input: &str) -> LangResult<Self> {
            if valid_adapter_name(input) {
                Ok(Self(input.to_owned()))
            } else {
                Err(LangError::InvalidAdapterName {
                    input: input.to_owned(),
                })
            }
        }
    }

    impl TryFrom<String> for AdapterName {
        type Error = LangError;

        fn try_from(value: String) -> LangResult<Self> {
            Self::parse(&value)
        }
    }

    impl From<AdapterName> for String {
        fn from(value: AdapterName) -> Self {
            value.0
        }
    }

    fn valid_adapter_name(input: &str) -> bool {
        let mut segments = input.split('.');
        let Some(first) = segments.next() else {
            return false;
        };
        if !valid_adapter_segment(first, false) {
            return false;
        }
        let mut saw_tail = false;
        for segment in segments {
            saw_tail = true;
            if !valid_adapter_segment(segment, true) {
                return false;
            }
        }
        saw_tail
    }

    fn valid_adapter_segment(segment: &str, allow_underscore: bool) -> bool {
        let mut chars = segment.chars();
        matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
            && chars.all(|ch| {
                ch.is_ascii_lowercase() || ch.is_ascii_digit() || (allow_underscore && ch == '_')
            })
    }

    pub(crate) mod model {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/model.rs"));
    }

    pub(crate) use model::ParseSet;

    pub(crate) mod schema {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/schema.rs"));
    }

    pub(crate) use schema::{
        LANG_PARSE_MANIFEST_V1_SCHEMA_FILE, LANG_PARSE_MANIFEST_V1_SCHEMA_ID,
        LANG_PARSE_MANIFEST_V1_SCHEMA_JSON, LANG_PARSE_MANIFEST_V1_SCHEMA_VERSION,
    };
}

#[derive(Clone, Debug, Deserialize)]
struct ParseManifest {
    version: u32,
    case: String,
    #[serde(flatten)]
    parse_set: lang::ParseSet,
}

#[test]
fn embedded_lang_schema_matches_disk() {
    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V1_SCHEMA_JSON,
        load_text("schemas/lang/parse_manifest.v1.json")
    );
    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/lang/parse_manifest.v1.json"
    );
    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V1_SCHEMA_FILE,
        "parse_manifest.v1.json"
    );
    assert_eq!(lang::LANG_PARSE_MANIFEST_V1_SCHEMA_VERSION, 1);
}

#[test]
fn valid_lang_schema_fixtures_validate_and_deserialize() {
    for fixture in [
        "fixtures/lang/valid/snapshot_parse_manifest.json",
        "fixtures/lang/valid/paths_scope_parse_manifest.json",
    ] {
        let instance = assert_schema_valid(fixture);
        let manifest: ParseManifest =
            serde_json::from_value(instance).expect("fixture should deserialize");

        assert_eq!(manifest.version, 1);
        assert!(
            !manifest.case.is_empty(),
            "fixture case should not be empty"
        );
        assert_eq!(
            manifest
                .parse_set
                .request
                .fingerprint()
                .expect("request fingerprint"),
            manifest.parse_set.request_fingerprint,
            "request_fingerprint mismatch in {fixture}"
        );

        for unit in &manifest.parse_set.units {
            assert_eq!(
                unit.fingerprint().expect("unit fingerprint"),
                unit.unit_fingerprint,
                "unit_fingerprint mismatch in {fixture} for {}",
                unit.path.as_str()
            );
        }

        assert_eq!(
            manifest.parse_set.stats.parsed_units,
            manifest.parse_set.units.len() as u64,
            "parsed_units mismatch in {fixture}"
        );
        assert_eq!(
            manifest.parse_set.stats.failed_units,
            manifest.parse_set.failed.len() as u64,
            "failed_units mismatch in {fixture}"
        );
        assert_eq!(
            manifest.parse_set.stats.missing_requested_languages,
            manifest.parse_set.missing_languages.len() as u64,
            "missing_requested_languages mismatch in {fixture}"
        );

        let expected_diagnostic_count = manifest.parse_set.diagnostics.len() as u64
            + manifest
                .parse_set
                .units
                .iter()
                .map(|unit| unit.diagnostics.len() as u64)
                .sum::<u64>()
            + manifest
                .parse_set
                .failed
                .iter()
                .map(|failed| failed.diagnostics.len() as u64)
                .sum::<u64>();
        assert_eq!(
            manifest.parse_set.stats.diagnostic_count, expected_diagnostic_count,
            "diagnostic_count mismatch in {fixture}"
        );
    }
}

#[test]
fn invalid_lang_schema_fixtures_fail_validation() {
    for fixture in [
        "fixtures/lang/invalid/adapter_name_invalid.json",
        "fixtures/lang/invalid/request_scope_missing_paths.json",
        "fixtures/lang/invalid/reference_target_invalid_shape.json",
        "fixtures/lang/invalid/top_level_unknown_field.json",
    ] {
        let instance = load_json(fixture);
        assert_schema_invalid(&instance, fixture);
    }
}

fn assert_schema_valid(fixture: &str) -> Value {
    let instance = load_json(fixture);
    let validator = schema_validator();
    if let Err(error) = validator.validate(&instance) {
        panic!("expected fixture to validate: {fixture}: {error}");
    }
    instance
}

fn assert_schema_invalid(instance: &Value, fixture: &str) {
    let validator = schema_validator();
    assert!(
        validator.validate(instance).is_err(),
        "expected fixture to fail schema validation: {fixture}"
    );
}

fn schema_validator() -> Validator {
    let root_schema = load_json("schemas/lang/parse_manifest.v1.json");
    let kernel_schema = load_json("schemas/kernel/primitives.v1.json");
    let retriever = InMemoryRetriever {
        schemas: HashMap::from([
            (
                kernel::PRIMITIVES_V1_SCHEMA_ID.to_owned(),
                kernel_schema.clone(),
            ),
            ("../kernel/primitives.v1.json".to_owned(), kernel_schema),
        ]),
    };

    jsonschema::draft202012::options()
        .with_retriever(retriever)
        .build(&root_schema)
        .expect("schema should compile")
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
