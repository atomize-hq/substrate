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
use walkdir as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::{
        sha256_bytes, sha256_canonical_json, DiagnosticCode, Fingerprint, JsonPointer, QueryId,
        RecipeId, RuleId, Severity,
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
        pack::PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_JSON,
        load_text("schemas/pack/boundary_taxonomy.v1.json")
    );
    assert_eq!(
        pack::PACK_COMPONENT_MAP_V1_SCHEMA_JSON,
        load_text("schemas/pack/component_map.v1.json")
    );
    assert_eq!(
        pack::PACK_SCORE_MODEL_V1_SCHEMA_JSON,
        load_text("schemas/pack/score_model.v1.json")
    );
    assert_eq!(
        pack::PACK_QUERY_PACK_V1_SCHEMA_JSON,
        load_text("schemas/pack/query_pack.v1.json")
    );
    assert_eq!(
        pack::PACK_RULE_PACK_V1_SCHEMA_JSON,
        load_text("schemas/pack/rule_pack.v1.json")
    );
    assert_eq!(
        pack::PACK_RECIPE_PACK_V1_SCHEMA_JSON,
        load_text("schemas/pack/recipe_pack.v1.json")
    );
    assert_eq!(
        pack::PACK_COMMON_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/common.v1.json"
    );
    assert_eq!(
        pack::PACK_PROFILE_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/profile.v1.json"
    );
    assert_eq!(
        pack::PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/boundary_taxonomy.v1.json"
    );
    assert_eq!(
        pack::PACK_COMPONENT_MAP_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/component_map.v1.json"
    );
    assert_eq!(
        pack::PACK_SCORE_MODEL_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/score_model.v1.json"
    );
    assert_eq!(
        pack::PACK_QUERY_PACK_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/query_pack.v1.json"
    );
    assert_eq!(
        pack::PACK_RULE_PACK_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/rule_pack.v1.json"
    );
    assert_eq!(
        pack::PACK_RECIPE_PACK_V1_SCHEMA_ID,
        "https://schemas.substrate.dev/lift/pack/recipe_pack.v1.json"
    );
    assert_eq!(pack::PACK_COMMON_V1_SCHEMA_FILE, "common.v1.json");
    assert_eq!(pack::PACK_PROFILE_V1_SCHEMA_FILE, "profile.v1.json");
    assert_eq!(
        pack::PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_FILE,
        "boundary_taxonomy.v1.json"
    );
    assert_eq!(
        pack::PACK_COMPONENT_MAP_V1_SCHEMA_FILE,
        "component_map.v1.json"
    );
    assert_eq!(pack::PACK_SCORE_MODEL_V1_SCHEMA_FILE, "score_model.v1.json");
    assert_eq!(pack::PACK_QUERY_PACK_V1_SCHEMA_FILE, "query_pack.v1.json");
    assert_eq!(pack::PACK_RULE_PACK_V1_SCHEMA_FILE, "rule_pack.v1.json");
    assert_eq!(pack::PACK_RECIPE_PACK_V1_SCHEMA_FILE, "recipe_pack.v1.json");
    assert_eq!(pack::PACK_COMMON_V1_SCHEMA_VERSION, 1);
    assert_eq!(pack::PACK_PROFILE_V1_SCHEMA_VERSION, 1);
    assert_eq!(pack::PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_VERSION, 1);
    assert_eq!(pack::PACK_COMPONENT_MAP_V1_SCHEMA_VERSION, 1);
    assert_eq!(pack::PACK_SCORE_MODEL_V1_SCHEMA_VERSION, 1);
    assert_eq!(pack::PACK_QUERY_PACK_V1_SCHEMA_VERSION, 1);
    assert_eq!(pack::PACK_RULE_PACK_V1_SCHEMA_VERSION, 1);
    assert_eq!(pack::PACK_RECIPE_PACK_V1_SCHEMA_VERSION, 1);
}

#[test]
fn valid_pack_profile_schema_fixtures_validate_and_deserialize() {
    for fixture in [
        "fixtures/pack/valid/profile_minimal.json",
        "fixtures/pack/valid/profile_full.json",
    ] {
        let instance = assert_schema_valid("profile", fixture);

        let raw: pack::raw::RawProfile =
            serde_json::from_value(instance).expect("fixture should deserialize");
        assert_eq!(raw.kind, pack::raw::PackKind::Profile);
    }
}

#[test]
fn valid_topology_schema_fixtures_validate_and_deserialize() {
    let boundary: pack::raw::RawBoundaryTaxonomy = serde_json::from_value(assert_schema_valid(
        "boundary_taxonomy",
        "fixtures/pack/valid/generic_boundaries.json",
    ))
    .expect("boundary taxonomy should deserialize");
    assert_eq!(boundary.kind, pack::raw::PackKind::BoundaryTaxonomy);

    let components: pack::raw::RawComponentMap = serde_json::from_value(assert_schema_valid(
        "component_map",
        "fixtures/pack/valid/generic_components.json",
    ))
    .expect("component map should deserialize");
    assert_eq!(components.kind, pack::raw::PackKind::ComponentMap);
}

#[test]
fn valid_advanced_schema_fixtures_validate_and_deserialize() {
    let score: pack::raw::RawScoreModel = serde_json::from_value(assert_schema_valid(
        "score_model",
        "fixtures/pack/valid/score/generic_lift_v2.json",
    ))
    .expect("score model should deserialize");
    assert_eq!(score.kind, pack::raw::PackKind::ScoreModel);

    let query: pack::raw::RawQueryPack = serde_json::from_value(assert_schema_valid(
        "query_pack",
        "fixtures/pack/valid/queries/rust_core.json",
    ))
    .expect("query pack should deserialize");
    assert_eq!(query.kind, pack::raw::PackKind::QueryPack);

    let rule: pack::raw::RawRulePack = serde_json::from_value(assert_schema_valid(
        "rule_pack",
        "fixtures/pack/valid/rules/generic_policy.json",
    ))
    .expect("rule pack should deserialize");
    assert_eq!(rule.kind, pack::raw::PackKind::RulePack);

    let recipe: pack::raw::RawRecipePack = serde_json::from_value(assert_schema_valid(
        "recipe_pack",
        "fixtures/pack/valid/recipes/generic_core_recipes.json",
    ))
    .expect("recipe pack should deserialize");
    assert_eq!(recipe.kind, pack::raw::PackKind::RecipePack);
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
        assert_schema_invalid("profile", &instance);
    }
}

#[test]
fn invalid_topology_schema_fixtures_fail_validation() {
    assert_schema_invalid(
        "boundary_taxonomy",
        &load_json("fixtures/pack/invalid/boundary_taxonomy_schema_violation.json"),
    );
    assert_schema_invalid(
        "boundary_taxonomy",
        &load_json("fixtures/pack/invalid/boundary_taxonomy_unknown_field.json"),
    );
    assert_schema_invalid(
        "component_map",
        &load_json("fixtures/pack/invalid/component_map_schema_violation.json"),
    );
    assert_schema_invalid(
        "component_map",
        &load_json("fixtures/pack/invalid/component_map_unknown_field.json"),
    );
}

#[test]
fn invalid_advanced_schema_fixtures_fail_validation() {
    assert_schema_invalid(
        "score_model",
        &load_json("fixtures/pack/invalid/score_model_schema_violation.json"),
    );
    assert_schema_invalid(
        "query_pack",
        &load_json("fixtures/pack/invalid/query_pack_schema_violation.json"),
    );
    assert_schema_invalid(
        "rule_pack",
        &load_json("fixtures/pack/invalid/rule_pack_bad_query_ref.json"),
    );
    assert_schema_invalid(
        "rule_pack",
        &load_json("fixtures/pack/invalid/rule_pack_invalid_severity.json"),
    );
    assert_schema_invalid(
        "recipe_pack",
        &load_json("fixtures/pack/invalid/recipe_pack_bad_transform.json"),
    );
}

fn assert_schema_valid(kind: &str, fixture: &str) -> Value {
    let instance = load_json(fixture);
    let validator = schema_validator(kind);
    if let Err(error) = validator.validate(&instance) {
        panic!("expected fixture to validate: {error}");
    }
    instance
}

fn assert_schema_invalid(kind: &str, instance: &Value) {
    let validator = schema_validator(kind);
    assert!(
        validator.validate(instance).is_err(),
        "expected fixture to fail schema validation"
    );
}

fn schema_validator(kind: &str) -> Validator {
    let schema_path = match kind {
        "profile" => "schemas/pack/profile.v1.json",
        "boundary_taxonomy" => "schemas/pack/boundary_taxonomy.v1.json",
        "component_map" => "schemas/pack/component_map.v1.json",
        "score_model" => "schemas/pack/score_model.v1.json",
        "query_pack" => "schemas/pack/query_pack.v1.json",
        "rule_pack" => "schemas/pack/rule_pack.v1.json",
        "recipe_pack" => "schemas/pack/recipe_pack.v1.json",
        other => panic!("unsupported schema kind {other}"),
    };
    let root_schema = load_json(schema_path);
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
