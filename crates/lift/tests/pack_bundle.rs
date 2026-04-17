use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

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
fn bundle_resolution_pulls_transitive_query_closure() {
    let compiler = pack::PackCompiler::new();
    let profile = compile_profile(
        &compiler,
        "fixtures/pack/valid/bundle/profile_advanced_file_backed.toml",
    );

    let bundle = compiler
        .resolve_profile_pack_set(&profile)
        .expect("bundle should resolve");

    assert!(bundle.score_model.is_some());
    assert_eq!(bundle.query_packs.len(), 1);
    assert_eq!(bundle.rule_packs.len(), 1);
    assert_eq!(bundle.recipe_packs.len(), 1);
    assert!(bundle
        .query_packs
        .contains_key(&pack::PackName::parse("acme/rust-core").expect("name")));
}

#[test]
fn bundle_resolution_dedupes_same_source_references() {
    let compiler = pack::PackCompiler::new();
    let profile = compile_profile(
        &compiler,
        "fixtures/pack/valid/bundle/profile_advanced_dedupe.toml",
    );

    let bundle = compiler
        .resolve_profile_pack_set(&profile)
        .expect("bundle should resolve");

    assert_eq!(bundle.query_packs.len(), 1);
    assert_eq!(bundle.rule_packs.len(), 1);
    assert_eq!(bundle.recipe_packs.len(), 1);
}

#[test]
fn missing_bundle_ref_fails_with_unknown_pack_reference() {
    let compiler = pack::PackCompiler::new();
    let dir = unique_temp_dir("bundle-missing");
    write_text(
        &dir.join("profile.toml"),
        r#"kind = "profile"
version = 1
id = "acme/missing-bundle"
name = "Missing bundle"

[queries]
packs = ["file:queries/missing.json"]
"#,
    );

    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: dir.join("profile.toml"),
            format_hint: None,
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_pack_set(&profile)
        .expect_err("missing bundle ref should fail");

    match error {
        pack::PackError::UnknownPackReference {
            referring_pack,
            reference,
        } => {
            assert_eq!(referring_pack, "acme/missing-bundle");
            assert_eq!(reference, "file:queries/missing.json");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn wrong_kind_bundle_ref_fails_with_ref_kind_mismatch() {
    let compiler = pack::PackCompiler::new();
    let dir = unique_temp_dir("bundle-wrong-kind");
    write_text(
        &dir.join("profile.toml"),
        r#"kind = "profile"
version = 1
id = "acme/wrong-kind-bundle"
name = "Wrong kind bundle"

[queries]
packs = ["file:generic_policy.json"]
"#,
    );
    write_text(
        &dir.join("generic_policy.json"),
        &rule_pack_json("file:queries/rust_core.json"),
    );

    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: dir.join("profile.toml"),
            format_hint: None,
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_pack_set(&profile)
        .expect_err("wrong kind should fail");

    match error {
        pack::PackError::RefKindMismatch {
            reference,
            expected,
            actual,
        } => {
            assert_eq!(reference, "file:generic_policy.json");
            assert_eq!(expected, pack::PackKind::QueryPack);
            assert_eq!(actual, pack::PackKind::RulePack);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn duplicate_pack_ids_across_origins_fail_bundle_resolution() {
    let compiler = pack::PackCompiler::new();
    let dir = unique_temp_dir("bundle-duplicate");
    fs::create_dir_all(dir.join("queries")).expect("queries dir");
    fs::create_dir_all(dir.join("queries_dup")).expect("queries dup dir");
    write_text(
        &dir.join("profile.toml"),
        r#"kind = "profile"
version = 1
id = "acme/duplicate-bundle"
name = "Duplicate bundle"

[queries]
packs = ["file:queries/rust_core.json"]

[rules]
packs = ["file:generic_policy.json"]
"#,
    );
    write_text(
        &dir.join("queries/rust_core.json"),
        &query_pack_json("dup/core"),
    );
    write_text(
        &dir.join("queries_dup/rust_core.json"),
        &query_pack_json("dup/core"),
    );
    write_text(
        &dir.join("generic_policy.json"),
        &rule_pack_json("file:queries_dup/rust_core.json"),
    );

    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: dir.join("profile.toml"),
            format_hint: None,
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_pack_set(&profile)
        .expect_err("duplicate pack ids should fail");

    match error {
        pack::PackError::DuplicatePackId { kind, id } => {
            assert_eq!(kind, pack::PackKind::QueryPack);
            assert_eq!(id, "dup/core");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn duplicate_pack_id_against_profile_fails_bundle_resolution() {
    let compiler = pack::PackCompiler::new();
    let dir = unique_temp_dir("bundle-duplicate-profile-id");
    fs::create_dir_all(dir.join("queries")).expect("queries dir");
    write_text(
        &dir.join("profile.toml"),
        r#"kind = "profile"
version = 1
id = "dup/shared"
name = "Duplicate bundle"

[queries]
packs = ["file:queries/rust_core.json"]
"#,
    );
    write_text(
        &dir.join("queries/rust_core.json"),
        &query_pack_json("dup/shared"),
    );

    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: dir.join("profile.toml"),
            format_hint: None,
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_pack_set(&profile)
        .expect_err("duplicate profile id should fail");

    match error {
        pack::PackError::DuplicatePackId { kind, id } => {
            assert_eq!(kind, pack::PackKind::QueryPack);
            assert_eq!(id, "dup/shared");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn duplicate_pack_id_against_boundary_taxonomy_fails_bundle_resolution() {
    let compiler = pack::PackCompiler::new();
    let dir = unique_temp_dir("bundle-duplicate-boundary-id");
    fs::create_dir_all(dir.join("queries")).expect("queries dir");
    fs::create_dir_all(dir.join("topology")).expect("topology dir");
    write_text(
        &dir.join("profile.toml"),
        r#"kind = "profile"
version = 1
id = "acme/duplicate-boundary-bundle"
name = "Duplicate boundary bundle"

[topology]
boundary_taxonomy = "file:topology/boundary-taxonomy.json"

[queries]
packs = ["file:queries/rust_core.json"]
"#,
    );
    write_text(
        &dir.join("topology/boundary-taxonomy.json"),
        &boundary_taxonomy_json("dup/shared"),
    );
    write_text(
        &dir.join("queries/rust_core.json"),
        &query_pack_json("dup/shared"),
    );

    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: dir.join("profile.toml"),
            format_hint: None,
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_pack_set(&profile)
        .expect_err("duplicate boundary taxonomy id should fail");

    match error {
        pack::PackError::DuplicatePackId { kind, id } => {
            assert_eq!(kind, pack::PackKind::QueryPack);
            assert_eq!(id, "dup/shared");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn duplicate_pack_id_against_component_map_fails_bundle_resolution() {
    let compiler = pack::PackCompiler::new();
    let dir = unique_temp_dir("bundle-duplicate-component-id");
    fs::create_dir_all(dir.join("queries")).expect("queries dir");
    fs::create_dir_all(dir.join("topology")).expect("topology dir");
    write_text(
        &dir.join("profile.toml"),
        r#"kind = "profile"
version = 1
id = "acme/duplicate-component-bundle"
name = "Duplicate component bundle"

[topology]
component_map = "file:topology/component-map.json"

[queries]
packs = ["file:queries/rust_core.json"]
"#,
    );
    write_text(
        &dir.join("topology/component-map.json"),
        &component_map_json("dup/shared"),
    );
    write_text(
        &dir.join("queries/rust_core.json"),
        &query_pack_json("dup/shared"),
    );

    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: dir.join("profile.toml"),
            format_hint: None,
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_pack_set(&profile)
        .expect_err("duplicate component map id should fail");

    match error {
        pack::PackError::DuplicatePackId { kind, id } => {
            assert_eq!(kind, pack::PackKind::QueryPack);
            assert_eq!(id, "dup/shared");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

fn compile_profile(compiler: &pack::PackCompiler, relative: &str) -> pack::CompiledProfile {
    compiler
        .compile_profile(pack::PackSource::File {
            path: crate_root().join(relative),
            format_hint: None,
        })
        .unwrap_or_else(|err| panic!("failed to compile profile {relative}: {err:?}"))
}

fn query_pack_json(id: &str) -> String {
    format!(
        r#"{{
  "$schema": "https://schemas.substrate.dev/lift/pack/query_pack.v1.json",
  "kind": "query_pack",
  "version": 1,
  "id": "{id}",
  "name": "Query pack",
  "language": "rust",
  "engine": "tree_sitter",
  "queries": [
    {{
      "id": "use_statement",
      "pattern": "(use_declaration) @use",
      "captures": [{{ "name": "use", "required": true }}]
    }}
  ]
}}"#
    )
}

fn rule_pack_json(query_pack_ref: &str) -> String {
    format!(
        r#"{{
  "$schema": "https://schemas.substrate.dev/lift/pack/rule_pack.v1.json",
  "kind": "rule_pack",
  "version": 1,
  "id": "acme/policy",
  "name": "Policy rules",
  "rules": [
    {{
      "id": "architecture.cross_boundary_import",
      "severity": "warning",
      "query": {{
        "pack": "{query_pack_ref}",
        "id": "use_statement"
      }},
      "emit": [
        {{
          "kind": "finding",
          "code": "architecture.cross_boundary_import",
          "message": "Import crosses boundary"
        }}
      ]
    }}
  ]
}}"#
    )
}

fn boundary_taxonomy_json(id: &str) -> String {
    format!(
        r#"{{
  "kind": "boundary_taxonomy",
  "version": 1,
  "id": "{id}",
  "name": "Boundary taxonomy",
  "counting": {{
    "mode": "distinct_minus_one"
  }},
  "boundaries": [
    {{
      "id": "runtime",
      "label": "Runtime",
      "include": ["services/runtime/**"]
    }}
  ]
}}"#
    )
}

fn component_map_json(id: &str) -> String {
    format!(
        r#"{{
  "kind": "component_map",
  "version": 1,
  "id": "{id}",
  "name": "Component map",
  "counting": {{
    "mode": "distinct"
  }},
  "components": [
    {{
      "id": "runtime",
      "label": "Runtime",
      "include": ["services/runtime/**"],
      "tags": ["internal"]
    }}
  ]
}}"#
    )
}

fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).unwrap_or_else(|err| {
        panic!("failed to write {}: {err}", path.display());
    });
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("substrate-lift-{label}-{nonce}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn crate_root() -> PathBuf {
    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = CWD_LOCK.get_or_init(|| Mutex::new(()));
    let _guard: MutexGuard<'_, ()> = lock.lock().expect("lock");
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
