use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd as _;
use clap as _;
use jsonschema as _;
use predicates as _;
use serde as _;
use serde_jcs as _;
use serde_json as _;
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
fn builtin_file_and_inline_profiles_compile_on_the_happy_path() {
    let compiler = pack::PackCompiler::new();

    let builtin = compiler
        .compile_profile(pack::builtin::profile_source("generic/default").expect("builtin"))
        .expect("builtin profile should compile");
    let file = compiler
        .compile_profile(pack::PackSource::File {
            path: crate_root().join("fixtures/pack/valid/profile_file_backed.toml"),
            format_hint: None,
        })
        .expect("file profile should compile");
    let inline = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "inline-happy".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: load_bytes("fixtures/pack/canonical/generic_default_order_b.toml"),
        })
        .expect("inline profile should compile");

    assert_eq!(builtin.header.id.as_str(), "generic/default");
    assert_eq!(file.header.id.as_str(), "acme/file-backed");
    assert_eq!(inline.header.id.as_str(), "generic/default");
    assert_eq!(file.header.origin.display(), file_fixture_display_path());
    assert!(matches!(
        file.topology.boundary_taxonomy,
        Some(pack::PackRef::File(_))
    ));
    assert!(matches!(file.score.model, Some(pack::PackRef::File(_))));
    assert_eq!(file.includes.rule_packs.len(), 2);
    assert!(file
        .apps
        .enabled
        .contains(&pack::AppName::parse("score").expect("app")));
}

#[test]
fn standalone_topology_packs_compile_on_the_happy_path() {
    let compiler = pack::PackCompiler::new();

    let builtin_boundary = compiler
        .compile_boundary_taxonomy(
            pack::builtin::boundary_taxonomy_source("generic/boundaries").expect("builtin"),
        )
        .expect("builtin boundary taxonomy should compile");
    let file_boundary = compiler
        .compile_boundary_taxonomy(pack::PackSource::File {
            path: crate_root().join("fixtures/pack/valid/topology/boundary-taxonomy.json"),
            format_hint: None,
        })
        .expect("file boundary taxonomy should compile");
    let inline_boundary = compiler
        .compile_boundary_taxonomy(pack::PackSource::Inline {
            logical_name: "inline-boundary".to_owned(),
            format: pack::PackFormat::Json,
            bytes: load_bytes("fixtures/pack/valid/generic_boundaries.json"),
        })
        .expect("inline boundary taxonomy should compile");

    assert_eq!(builtin_boundary.header.id.as_str(), "generic/boundaries");
    assert_eq!(file_boundary.header.id.as_str(), "acme/boundaries");
    assert_eq!(inline_boundary.header.id.as_str(), "generic/boundaries");
    assert_eq!(file_boundary.boundaries.len(), 2);
    assert!(file_boundary
        .boundaries
        .values()
        .next()
        .expect("boundary")
        .include_matcher
        .is_match(Path::new("services/runtime/main.rs")));

    let builtin_component = compiler
        .compile_component_map(
            pack::builtin::component_map_source("generic/components").expect("builtin"),
        )
        .expect("builtin component map should compile");
    let file_component = compiler
        .compile_component_map(pack::PackSource::File {
            path: crate_root().join("fixtures/pack/valid/topology/component-map.json"),
            format_hint: None,
        })
        .expect("file component map should compile");
    let inline_component = compiler
        .compile_component_map(pack::PackSource::Inline {
            logical_name: "inline-component".to_owned(),
            format: pack::PackFormat::Json,
            bytes: load_bytes("fixtures/pack/valid/generic_components.json"),
        })
        .expect("inline component map should compile");

    assert_eq!(builtin_component.header.id.as_str(), "generic/components");
    assert_eq!(file_component.header.id.as_str(), "acme/components");
    assert_eq!(inline_component.header.id.as_str(), "generic/components");
    assert!(file_component
        .components
        .values()
        .any(|component| component.tags.contains("public")));
}

#[test]
fn unsupported_format_and_missing_file_hard_fail() {
    let compiler = pack::PackCompiler::new();

    let unsupported = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "inline-json".to_owned(),
            format: pack::PackFormat::Json,
            bytes: br#"{}"#.to_vec(),
        })
        .expect_err("json input should fail in phase a");
    assert!(matches!(
        unsupported,
        pack::PackError::UnsupportedFormat { .. }
    ));

    let missing = compiler
        .compile_profile(pack::PackSource::File {
            path: crate_root().join("fixtures/pack/invalid/missing-profile.toml"),
            format_hint: None,
        })
        .expect_err("missing file should fail");
    assert!(matches!(missing, pack::PackError::Io { .. }));
}

#[test]
fn malformed_topology_json_reports_parse_failure() {
    let compiler = pack::PackCompiler::new();

    let boundary = compiler
        .compile_boundary_taxonomy(pack::PackSource::Inline {
            logical_name: "bad-boundary-json".to_owned(),
            format: pack::PackFormat::Json,
            bytes: br#"{"kind":"boundary_taxonomy""#.to_vec(),
        })
        .expect_err("malformed boundary taxonomy should fail");
    assert!(matches!(boundary, pack::PackError::ParseFailure { .. }));

    let component = compiler
        .compile_component_map(pack::PackSource::Inline {
            logical_name: "bad-component-json".to_owned(),
            format: pack::PackFormat::Json,
            bytes: br#"{"kind":"component_map""#.to_vec(),
        })
        .expect_err("malformed component map should fail");
    assert!(matches!(component, pack::PackError::ParseFailure { .. }));
}

#[test]
fn topology_schema_violations_are_sorted_deterministically() {
    let compiler = pack::PackCompiler::new();

    let boundary = compiler
        .compile_boundary_taxonomy(pack::PackSource::Inline {
            logical_name: "bad-boundary-schema".to_owned(),
            format: pack::PackFormat::Json,
            bytes: load_bytes("fixtures/pack/invalid/boundary_taxonomy_schema_violation.json"),
        })
        .expect_err("invalid boundary schema should fail");
    assert_schema_violation_codes(
        boundary,
        "inline:bad-boundary-schema",
        pack::PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_ID,
        &[
            "/boundaries/0/id",
            "/boundaries/0/include/0",
            "/boundaries/0/label",
            "/boundaries/0/label",
            "/counting/mode",
            "/name",
            "/version",
        ],
    );

    let component = compiler
        .compile_component_map(pack::PackSource::Inline {
            logical_name: "bad-component-schema".to_owned(),
            format: pack::PackFormat::Json,
            bytes: load_bytes("fixtures/pack/invalid/component_map_schema_violation.json"),
        })
        .expect_err("invalid component schema should fail");
    assert_schema_violation_codes(
        component,
        "inline:bad-component-schema",
        pack::PACK_COMPONENT_MAP_V1_SCHEMA_ID,
        &[
            "/components/0/id",
            "/components/0/include/0",
            "/components/0/label",
            "/components/0/label",
            "/components/0/tags/0",
            "/counting/mode",
            "/name",
            "/version",
        ],
    );
}

#[test]
fn duplicate_topology_entry_ids_fail_with_typed_errors() {
    let compiler = pack::PackCompiler::new();

    let boundary = compiler
        .compile_boundary_taxonomy(pack::PackSource::Inline {
            logical_name: "duplicate-boundary".to_owned(),
            format: pack::PackFormat::Json,
            bytes: br#"{
  "kind": "boundary_taxonomy",
  "version": 1,
  "id": "acme/boundaries",
  "name": "Dup boundaries",
  "counting": { "mode": "distinct_minus_one" },
  "boundaries": [
    { "id": "runtime", "label": "Runtime", "include": ["services/runtime/**"] },
    { "id": "runtime", "label": "Runtime two", "include": ["services/runtime/v2/**"] }
  ]
}"#
            .to_vec(),
        })
        .expect_err("duplicate boundary ids should fail");
    assert_duplicate_entry(
        boundary,
        pack::PackKind::BoundaryTaxonomy,
        "acme/boundaries",
        "boundary",
        "runtime",
    );

    let component = compiler
        .compile_component_map(pack::PackSource::Inline {
            logical_name: "duplicate-component".to_owned(),
            format: pack::PackFormat::Json,
            bytes: br#"{
  "kind": "component_map",
  "version": 1,
  "id": "acme/components",
  "name": "Dup components",
  "counting": { "mode": "distinct" },
  "components": [
    { "id": "api", "label": "API", "include": ["api/**"] },
    { "id": "api", "label": "API two", "include": ["api/v2/**"] }
  ]
}"#
            .to_vec(),
        })
        .expect_err("duplicate component ids should fail");
    assert_duplicate_entry(
        component,
        pack::PackKind::ComponentMap,
        "acme/components",
        "component",
        "api",
    );
}

#[test]
fn invalid_topology_globs_fail_during_compile() {
    let compiler = pack::PackCompiler::new();

    let invalid_include = compiler
        .compile_boundary_taxonomy(pack::PackSource::Inline {
            logical_name: "invalid-include".to_owned(),
            format: pack::PackFormat::Json,
            bytes: br#"{
  "kind": "boundary_taxonomy",
  "version": 1,
  "id": "acme/boundaries",
  "name": "Bad include",
  "counting": { "mode": "distinct_minus_one" },
  "boundaries": [
    { "id": "runtime", "label": "Runtime", "include": ["["] }
  ]
}"#
            .to_vec(),
        })
        .expect_err("invalid include glob should fail");
    assert!(matches!(
        invalid_include,
        pack::PackError::GlobCompile { .. }
    ));

    let invalid_exclude = compiler
        .compile_component_map(pack::PackSource::Inline {
            logical_name: "invalid-exclude".to_owned(),
            format: pack::PackFormat::Json,
            bytes: br#"{
  "kind": "component_map",
  "version": 1,
  "id": "acme/components",
  "name": "Bad exclude",
  "counting": { "mode": "distinct" },
  "components": [
    { "id": "api", "label": "API", "include": ["api/**"], "exclude": ["["] }
  ]
}"#
            .to_vec(),
        })
        .expect_err("invalid exclude glob should fail");
    assert!(matches!(
        invalid_exclude,
        pack::PackError::GlobCompile { .. }
    ));
}

#[test]
fn invalid_toml_reports_typed_diagnostics() {
    let compiler = pack::PackCompiler::new();
    let error = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "invalid-toml".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: load_bytes("fixtures/pack/invalid/profile_invalid_toml.toml"),
        })
        .expect_err("invalid toml should fail");

    match error {
        pack::PackError::ParseFailure {
            origin,
            diagnostics,
        } => {
            assert_eq!(origin, "inline:invalid-toml");
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].code.as_str(), "pack.profile.invalid_toml");
            assert_eq!(
                diagnostics[0]
                    .subject
                    .as_ref()
                    .map(|subject| subject.origin.display()),
                Some(origin)
            );
            assert!(!diagnostics[0].message.is_empty());
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn schema_violations_are_sorted_deterministically() {
    let compiler = pack::PackCompiler::new();
    let error = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "schema-violation".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: load_bytes("fixtures/pack/invalid/profile_schema_violation.toml"),
        })
        .expect_err("schema violation should fail");

    match error {
        pack::PackError::SchemaViolation {
            origin,
            schema_id,
            diagnostics,
        } => {
            assert_eq!(origin, "inline:schema-violation");
            assert_eq!(schema_id, pack::PACK_PROFILE_V1_SCHEMA_ID);
            let actual: Vec<(String, String)> = diagnostics
                .iter()
                .map(|diagnostic| {
                    (
                        diagnostic
                            .subject
                            .as_ref()
                            .and_then(|subject| subject.path.as_ref())
                            .map(|path| path.as_str().to_owned())
                            .expect("schema diagnostics should include paths"),
                        diagnostic.code.as_str().to_owned(),
                    )
                })
                .collect();

            assert_eq!(
                actual,
                vec![
                    (
                        "/analysis/follow_symlinks".to_owned(),
                        "pack.schema.invalid_type".to_owned(),
                    ),
                    (
                        "/analysis/languages/1".to_owned(),
                        "pack.schema.invalid_language_id".to_owned(),
                    ),
                    (
                        "/analysis/max_scope_depth".to_owned(),
                        "pack.schema.invalid_type".to_owned(),
                    ),
                    (
                        "/apps/default".to_owned(),
                        "pack.schema.invalid_app_name".to_owned(),
                    ),
                    (
                        "/apps/enabled/1".to_owned(),
                        "pack.schema.invalid_app_name".to_owned(),
                    ),
                    ("/id".to_owned(), "pack.schema.invalid_pack_name".to_owned(),),
                    ("/kind".to_owned(), "pack.schema.invalid_kind".to_owned(),),
                    (
                        "/version".to_owned(),
                        "pack.schema.invalid_version".to_owned(),
                    ),
                ]
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn pack_refs_reject_absolute_and_traversal_inputs() {
    for input in [
        "file:/rules/security.toml",
        "file:./rules/security.toml",
        "file:../rules/security.toml",
        "file:rules//security.toml",
        "file:C:/rules/security.toml",
        "file:-bad.toml",
        "file:.bad.toml",
        "not-a-ref",
    ] {
        assert!(
            pack::PackRef::parse(input).is_err(),
            "{input} should be rejected"
        );
    }
}

#[test]
fn empty_profile_name_fails_schema_validation() {
    let compiler = pack::PackCompiler::new();
    let error = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "empty-name".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: br#"kind = "profile"
version = 1
id = "generic/default"
name = ""
"#
            .to_vec(),
        })
        .expect_err("empty name should fail");

    assert_schema_violation(
        error,
        "inline:empty-name",
        &[("/name", "pack.schema.invalid_name")],
    );
}

#[test]
fn malformed_file_ref_fails_schema_validation() {
    let compiler = pack::PackCompiler::new();
    let error = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "bad-ref".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: br#"kind = "profile"
version = 1
id = "generic/default"
name = "Invalid ref"

[rules]
packs = ["file:-bad.toml"]
"#
            .to_vec(),
        })
        .expect_err("invalid file ref should fail");

    assert_schema_violation(
        error,
        "inline:bad-ref",
        &[("/rules/packs/0", "pack.schema.invalid_pack_ref")],
    );
}

#[test]
fn builtin_and_inline_sources_reject_file_refs_in_phase_a() {
    let compiler = pack::PackCompiler::new();

    let builtin_error = compiler
        .compile_profile(pack::PackSource::Builtin {
            logical_name: "bad-builtin",
            format: pack::PackFormat::Toml,
            bytes: br#"kind = "profile"
version = 1
id = "generic/default"
name = "Invalid builtin"

[score]
model = "file:score/model.toml"
"#,
        })
        .expect_err("builtin file refs should fail");
    assert_rejects_file_ref_origin(builtin_error, "builtin:bad-builtin", "/score/model");

    let inline_error = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "bad-inline".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: br#"kind = "profile"
version = 1
id = "generic/default"
name = "Invalid inline"

[rules]
packs = ["file:rules/security.toml"]
"#
            .to_vec(),
        })
        .expect_err("inline file refs should fail");
    assert_rejects_file_ref_origin(inline_error, "inline:bad-inline", "/rules/packs/0");
}

fn assert_rejects_file_ref_origin(
    error: pack::PackError,
    expected_origin: &str,
    expected_path: &str,
) {
    assert_schema_violation(
        error,
        expected_origin,
        &[(expected_path, "pack.refs.file_requires_file_origin")],
    );
}

fn assert_schema_violation(
    error: pack::PackError,
    expected_origin: &str,
    expected: &[(&str, &str)],
) {
    match error {
        pack::PackError::SchemaViolation {
            origin,
            diagnostics,
            ..
        } => {
            assert_eq!(origin, expected_origin);
            let actual: Vec<(String, String)> = diagnostics
                .iter()
                .map(|diagnostic| {
                    (
                        diagnostic
                            .subject
                            .as_ref()
                            .and_then(|subject| subject.path.as_ref())
                            .map(|path| path.as_str().to_owned())
                            .unwrap_or_default(),
                        diagnostic.code.as_str().to_owned(),
                    )
                })
                .collect();
            let expected: Vec<(String, String)> = expected
                .iter()
                .map(|(path, code)| ((*path).to_owned(), (*code).to_owned()))
                .collect();
            assert_eq!(actual, expected);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

fn assert_schema_violation_codes(
    error: pack::PackError,
    expected_origin: &str,
    expected_schema: &str,
    expected_paths: &[&str],
) {
    match error {
        pack::PackError::SchemaViolation {
            origin,
            schema_id,
            diagnostics,
        } => {
            assert_eq!(origin, expected_origin);
            assert_eq!(schema_id, expected_schema);
            let actual: Vec<String> = diagnostics
                .iter()
                .map(|diagnostic| {
                    diagnostic
                        .subject
                        .as_ref()
                        .and_then(|subject| subject.path.as_ref())
                        .map(|path| path.as_str().to_owned())
                        .expect("schema diagnostics should include paths")
                })
                .collect();
            assert_eq!(
                actual,
                expected_paths
                    .iter()
                    .map(|path| (*path).to_owned())
                    .collect::<Vec<_>>()
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

fn assert_duplicate_entry(
    error: pack::PackError,
    expected_kind: pack::PackKind,
    expected_pack_id: &str,
    expected_entry_kind: &'static str,
    expected_entry_id: &str,
) {
    match error {
        pack::PackError::DuplicateEntryId {
            pack_kind,
            pack_id,
            entry_kind,
            entry_id,
        } => {
            assert_eq!(pack_kind, expected_kind);
            assert_eq!(pack_id, expected_pack_id);
            assert_eq!(entry_kind, expected_entry_kind);
            assert_eq!(entry_id, expected_entry_id);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

fn load_bytes(relative: &str) -> Vec<u8> {
    let path = crate_root().join(relative);
    fs::read(&path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", path.display());
    })
}

fn file_fixture_display_path() -> String {
    crate_root()
        .join("fixtures/pack/valid/profile_file_backed.toml")
        .display()
        .to_string()
}

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
