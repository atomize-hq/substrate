#![allow(unused_crate_dependencies)]

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd as _;
use gix as _;
use jsonschema::{Retrieve, Uri, Validator};
use predicates as _;
use serde::Deserialize;
use serde_jcs as _;
use serde_json::{Map, Value};
use sha2 as _;
use substrate_lift as _;
use toml as _;
use walkdir as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::*;
}

#[path = "../src/pack/mod.rs"]
mod pack;

#[path = "../src/repo/mod.rs"]
mod repo;

#[path = "../src/lang/mod.rs"]
mod lang;

#[path = "support/repo_support.rs"]
mod repo_support;

#[derive(Clone, Debug, Deserialize)]
struct ParseManifest {
    version: u32,
    case: String,
    #[serde(flatten)]
    parse_set: lang::ParseSet,
}

#[derive(Clone, Debug, Deserialize)]
struct FakeDocument {
    #[serde(default)]
    fail: bool,
    #[serde(default)]
    symbols: Vec<lang::LocalSymbolDraft>,
}

#[derive(Clone, Debug)]
struct FakeAdapter {
    descriptor: lang::AdapterDescriptor,
    capabilities: lang::AdapterCapabilities,
    suffix: &'static str,
}

impl FakeAdapter {
    fn new(name: &str, language: &str, suffix: &'static str, version: &str) -> Self {
        Self {
            descriptor: lang::AdapterDescriptor {
                name: lang::AdapterName::parse(name).expect("adapter name"),
                language: pack::LanguageId::parse(language).expect("language"),
                version: version.to_owned(),
            },
            capabilities: lang::AdapterCapabilities::default(),
            suffix,
        }
    }

    fn with_capabilities(mut self, capabilities: lang::AdapterCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }
}

impl lang::LanguageAdapter for FakeAdapter {
    fn descriptor(&self) -> lang::AdapterDescriptor {
        self.descriptor.clone()
    }

    fn capabilities(&self) -> lang::AdapterCapabilities {
        self.capabilities.clone()
    }

    fn recognizes(&self, input: &lang::ParseInput<'_>) -> bool {
        input.path.as_str().ends_with(self.suffix)
    }

    fn parse(&self, input: &lang::ParseInput<'_>) -> lang::AdapterParseResult {
        let document: FakeDocument = match serde_json::from_slice(input.bytes) {
            Ok(document) => document,
            Err(_) => {
                return lang::AdapterParseResult::Failed {
                    diagnostics: Vec::new(),
                };
            }
        };

        if document.fail {
            return lang::AdapterParseResult::Failed {
                diagnostics: Vec::new(),
            };
        }

        lang::AdapterParseResult::Parsed(lang::AdapterParseOutput {
            symbols: document.symbols,
            edges: Vec::new(),
            surface_markers: Vec::new(),
            diagnostics: Vec::new(),
        })
    }
}

#[test]
fn embedded_lang_schemas_match_disk() {
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

    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V2_SCHEMA_JSON,
        load_text("schemas/lang/parse_manifest.v2.json")
    );
    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V2_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/lang/parse_manifest.v2.json"
    );
    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V2_SCHEMA_FILE,
        "parse_manifest.v2.json"
    );
    assert_eq!(lang::LANG_PARSE_MANIFEST_V2_SCHEMA_VERSION, 2);
}

#[test]
fn lang_module_re_exports_v2_schema_constants() {
    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V2_SCHEMA_JSON,
        load_text("schemas/lang/parse_manifest.v2.json")
    );
    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V2_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/lang/parse_manifest.v2.json"
    );
    assert_eq!(
        lang::LANG_PARSE_MANIFEST_V2_SCHEMA_FILE,
        "parse_manifest.v2.json"
    );
    assert_eq!(lang::LANG_PARSE_MANIFEST_V2_SCHEMA_VERSION, 2);
}

#[test]
fn valid_lang_schema_v1_fixtures_validate_and_deserialize() {
    for fixture in [
        "fixtures/lang/valid/snapshot_parse_manifest.json",
        "fixtures/lang/valid/paths_scope_parse_manifest.json",
    ] {
        let instance = assert_schema_valid("schemas/lang/parse_manifest.v1.json", fixture);
        let manifest: ParseManifest =
            serde_json::from_value(instance.clone()).expect("fixture should deserialize");

        assert_eq!(manifest.version, 1);
        assert_manifest_invariants(&manifest, fixture);
        assert!(
            instance["stats"].get("cache_hits").is_none(),
            "unexpected cache_hits in {fixture}"
        );
        assert!(
            instance["stats"].get("cache_misses").is_none(),
            "unexpected cache_misses in {fixture}"
        );
    }
}

#[test]
fn valid_lang_schema_v2_fixtures_validate_and_deserialize() {
    for (fixture, expected_hits, expected_misses) in [
        (
            "fixtures/lang/valid/snapshot_parse_manifest_v2.json",
            1_u64,
            2_u64,
        ),
        (
            "fixtures/lang/valid/paths_scope_parse_manifest_v2.json",
            0_u64,
            0_u64,
        ),
    ] {
        let instance = assert_schema_valid("schemas/lang/parse_manifest.v2.json", fixture);
        let manifest: ParseManifest =
            serde_json::from_value(instance.clone()).expect("fixture should deserialize");

        assert_eq!(manifest.version, 2);
        assert_manifest_invariants(&manifest, fixture);
        assert_eq!(
            instance["stats"]["cache_hits"].as_u64(),
            Some(expected_hits)
        );
        assert_eq!(
            instance["stats"]["cache_misses"].as_u64(),
            Some(expected_misses)
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
        assert_schema_invalid("schemas/lang/parse_manifest.v1.json", &instance, fixture);
    }
}

#[test]
fn phase_c_preserves_parse_manifest_v2_shape() {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/app.fake.json", &bytes)]);
    let parse_set = parse_with_driver(
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
            "1.0.0",
        )
        .with_capabilities(lang::AdapterCapabilities {
            emits_local_edges: true,
            emits_surface_markers: true,
            query_engines: [lang::QueryEngineKind::TreeSitter].into_iter().collect(),
        })]),
        &snapshot,
        lang::ParseRequest {
            languages: language_set(&["json"]),
            scope: lang::ParseScope::Snapshot,
        },
    );

    let parse_set_json = serde_json::to_value(&parse_set).expect("parse set should serialize");
    assert_parse_set_shape(&parse_set_json);
    assert!(
        parse_set_json.get("capabilities").is_none(),
        "parse_set should not serialize adapter capabilities"
    );
    assert!(
        parse_set_json["request"].get("query_engines").is_none(),
        "request should not serialize adapter query engine metadata"
    );
    assert!(
        parse_set_json["units"][0].get("capabilities").is_none(),
        "parsed units should not serialize adapter capabilities"
    );
    assert!(
        parse_set_json["units"][0].get("query_engines").is_none(),
        "parsed units should not serialize adapter query engine metadata"
    );
    assert!(
        parse_set_json["units"][0]
            .get("emits_local_edges")
            .is_none(),
        "parsed units should not serialize edge capability flags"
    );
    assert!(
        parse_set_json["units"][0]
            .get("emits_surface_markers")
            .is_none(),
        "parsed units should not serialize surface marker capability flags"
    );
    assert!(
        parse_set_json["units"][0]["adapter"].is_string(),
        "adapter field should remain the manifest string identifier"
    );

    let manifest = manifest_v2("phase_c_preserves_parse_manifest_v2_shape", &parse_set);
    let validator = schema_validator("schemas/lang/parse_manifest.v2.json");
    if let Err(error) = validator.validate(&manifest) {
        panic!("expected phase-c manifest to validate: {error}");
    }
}

fn assert_manifest_invariants(manifest: &ParseManifest, fixture: &str) {
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

fn assert_schema_valid(schema: &str, fixture: &str) -> Value {
    let instance = load_json(fixture);
    let validator = schema_validator(schema);
    if let Err(error) = validator.validate(&instance) {
        panic!("expected fixture to validate: {fixture}: {error}");
    }
    instance
}

fn assert_schema_invalid(schema: &str, instance: &Value, fixture: &str) {
    let validator = schema_validator(schema);
    assert!(
        validator.validate(instance).is_err(),
        "expected fixture to fail schema validation: {fixture}"
    );
}

fn schema_validator(schema: &str) -> Validator {
    let root_schema = load_json(schema);
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

fn registry(adapters: Vec<FakeAdapter>) -> lang::LanguageRegistry {
    let mut builder = lang::LanguageRegistryBuilder::new();
    for adapter in adapters {
        builder = builder.register(adapter).expect("register adapter");
    }
    builder.build().expect("build registry")
}

fn parse_with_driver(
    registry: lang::LanguageRegistry,
    snapshot: &repo::RepoSnapshot,
    request: lang::ParseRequest,
) -> lang::ParseSet {
    lang::ParseDriver::new(registry)
        .parse_snapshot(snapshot, &request)
        .expect("parse snapshot")
}

fn snapshot_with_files(files: &[(&str, &[u8])]) -> (repo_support::TempDir, repo::RepoSnapshot) {
    let temp = repo_support::TempDir::new("lang-schema");
    repo_support::write_file(&temp.path().join(".git/HEAD"), b"ref: refs/heads/main\n");
    for (path, bytes) in files {
        repo_support::write_file(&temp.path().join(path), bytes);
    }
    let snapshot = repo_support::materialize(temp.path(), repo_support::default_snapshot_options());
    (temp, snapshot)
}

fn language_set(languages: &[&str]) -> BTreeSet<pack::LanguageId> {
    languages
        .iter()
        .map(|language| pack::LanguageId::parse(language).expect("language"))
        .collect()
}

fn fake_symbol(local_key: &str, name: &str, start: u64, end: u64) -> serde_json::Value {
    serde_json::json!({
        "local_key": local_key,
        "kind": "function",
        "name": name,
        "path": [name],
        "span": {
            "start_byte": start,
            "end_byte": end
        },
        "visibility": "public"
    })
}

fn manifest_v2(case: &str, parse_set: &lang::ParseSet) -> Value {
    let Value::Object(parse_set_fields) =
        serde_json::to_value(parse_set).expect("parse set should serialize")
    else {
        panic!("parse set should serialize to an object");
    };
    let mut manifest = Map::from_iter([
        ("version".to_owned(), Value::from(2)),
        ("case".to_owned(), Value::from(case.to_owned())),
    ]);
    manifest.extend(parse_set_fields);
    Value::Object(manifest)
}

fn assert_parse_set_shape(parse_set: &Value) {
    let top_level_keys = parse_set
        .as_object()
        .expect("parse set should serialize to an object")
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        top_level_keys,
        BTreeSet::from([
            "diagnostics".to_owned(),
            "failed".to_owned(),
            "missing_languages".to_owned(),
            "request".to_owned(),
            "request_fingerprint".to_owned(),
            "skipped".to_owned(),
            "snapshot_fingerprint".to_owned(),
            "stats".to_owned(),
            "units".to_owned(),
        ])
    );

    let unit_keys = parse_set["units"][0]
        .as_object()
        .expect("parsed unit should serialize to an object")
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        unit_keys,
        BTreeSet::from([
            "adapter".to_owned(),
            "adapter_version".to_owned(),
            "blob_fingerprint".to_owned(),
            "diagnostics".to_owned(),
            "edges".to_owned(),
            "file_id".to_owned(),
            "language".to_owned(),
            "path".to_owned(),
            "surface_markers".to_owned(),
            "symbols".to_owned(),
            "unit_fingerprint".to_owned(),
        ])
    );
}
