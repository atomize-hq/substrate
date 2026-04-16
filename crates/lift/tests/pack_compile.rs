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
        "not-a-ref",
    ] {
        assert!(
            pack::PackRef::parse(input).is_err(),
            "{input} should be rejected"
        );
    }
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
    assert_rejects_file_ref_origin(builtin_error, "builtin:bad-builtin");

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
    assert_rejects_file_ref_origin(inline_error, "inline:bad-inline");
}

fn assert_rejects_file_ref_origin(error: pack::PackError, expected_origin: &str) {
    match error {
        pack::PackError::SchemaViolation {
            origin,
            diagnostics,
            ..
        } => {
            assert_eq!(origin, expected_origin);
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(
                diagnostics[0].code.as_str(),
                "pack.refs.file_requires_file_origin"
            );
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
