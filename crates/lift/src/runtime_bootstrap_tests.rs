use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::runtime::{bootstrap_profile, ProfileBootstrap};
use crate::pack::{builtin, PackCompiler, PackError, PackFormat, PackName, PackSource};

#[test]
fn bootstrap_builtin_profile_returns_profile_bootstrap() {
    let bootstrap = bootstrap_profile(
        builtin::profile_source("generic/default").expect("builtin profile source"),
    )
    .expect("builtin bootstrap should succeed");

    assert_eq!(
        bootstrap.bundle.profile.header.id.as_str(),
        "generic/default"
    );
    assert!(bootstrap.bundle.boundary_taxonomy.is_some());
    assert!(bootstrap.bundle.component_map.is_some());
    assert!(bootstrap.bundle.score_model.is_none());
    assert!(bootstrap.bundle.query_packs.is_empty());
    assert!(bootstrap.bundle.rule_packs.is_empty());
    assert!(bootstrap.bundle.recipe_packs.is_empty());
}

#[test]
fn bootstrap_file_backed_advanced_profile_returns_profile_bootstrap() {
    let bootstrap = bootstrap_profile(PackSource::File {
        path: crate_root().join("fixtures/pack/valid/bundle/profile_advanced_file_backed.toml"),
        format_hint: None,
    })
    .expect("file-backed bootstrap should succeed");

    assert_eq!(
        bootstrap.bundle.profile.header.id.as_str(),
        "acme/advanced-bundle"
    );
    assert_eq!(
        bootstrap
            .bundle
            .score_model
            .as_ref()
            .expect("score model")
            .header
            .id
            .as_str(),
        "acme/lift-v2"
    );
    assert!(bootstrap
        .bundle
        .query_packs
        .contains_key(&PackName::parse("acme/rust-core").expect("query pack name")));
    assert!(bootstrap
        .bundle
        .rule_packs
        .contains_key(&PackName::parse("acme/policy").expect("rule pack name")));
    assert!(bootstrap
        .bundle
        .recipe_packs
        .contains_key(&PackName::parse("acme/core-recipes").expect("recipe pack name")));
}

#[test]
fn bootstrap_inline_profile_returns_profile_bootstrap() {
    let bootstrap = bootstrap_profile(PackSource::Inline {
        logical_name: "inline-generic-default".to_owned(),
        format: PackFormat::Toml,
        bytes: load_bytes("fixtures/pack/canonical/generic_default_order_b.toml"),
    })
    .expect("inline bootstrap should succeed");

    assert_eq!(
        bootstrap.bundle.profile.header.id.as_str(),
        "generic/default"
    );
    assert!(bootstrap.bundle.boundary_taxonomy.is_some());
    assert!(bootstrap.bundle.component_map.is_some());
    assert!(bootstrap.bundle.score_model.is_none());
    assert!(bootstrap.bundle.query_packs.is_empty());
    assert!(bootstrap.bundle.rule_packs.is_empty());
    assert!(bootstrap.bundle.recipe_packs.is_empty());
}

#[test]
fn bootstrap_propagates_profile_compile_errors_without_translation() {
    let source = PackSource::Inline {
        logical_name: "schema-violation".to_owned(),
        format: PackFormat::Toml,
        bytes: load_bytes("fixtures/pack/invalid/profile_schema_violation.toml"),
    };

    let compiler = PackCompiler::new();
    let expected = compiler
        .compile_profile(source.clone())
        .expect_err("compile_profile should fail");
    let actual = bootstrap_profile(source).expect_err("bootstrap should fail");

    assert_eq!(actual, expected);
}

#[test]
fn bootstrap_propagates_bundle_resolution_errors_without_translation() {
    let compiler = PackCompiler::new();

    let unknown_source = write_temp_profile(
        "runtime-bootstrap-unknown-ref",
        r#"kind = "profile"
version = 1
id = "acme/runtime-bootstrap-unknown-ref"
name = "Runtime bootstrap unknown ref"

[queries]
packs = ["file:queries/missing.json"]
"#,
    );
    let unknown_expected = compiler
        .resolve_profile_pack_set(
            &compiler
                .compile_profile(unknown_source.clone())
                .expect("unknown-ref profile should compile"),
        )
        .expect_err("bundle resolution should fail with missing ref");
    let unknown_actual = bootstrap_profile(unknown_source).expect_err("bootstrap should fail");
    assert_eq!(unknown_actual, unknown_expected);
    assert!(matches!(
        unknown_actual,
        PackError::UnknownPackReference { .. }
    ));

    let mismatch_dir = unique_temp_dir("runtime-bootstrap-ref-kind-mismatch");
    write_text(
        &mismatch_dir.join("profile.toml"),
        r#"kind = "profile"
version = 1
id = "acme/runtime-bootstrap-ref-kind-mismatch"
name = "Runtime bootstrap ref kind mismatch"

[queries]
packs = ["file:generic_policy.json"]
"#,
    );
    write_text(
        &mismatch_dir.join("generic_policy.json"),
        &rule_pack_json("file:queries/rust_core.json"),
    );
    let mismatch_source = PackSource::File {
        path: mismatch_dir.join("profile.toml"),
        format_hint: None,
    };
    let mismatch_expected = compiler
        .resolve_profile_pack_set(
            &compiler
                .compile_profile(mismatch_source.clone())
                .expect("mismatch profile should compile"),
        )
        .expect_err("bundle resolution should fail with ref kind mismatch");
    let mismatch_actual = bootstrap_profile(mismatch_source).expect_err("bootstrap should fail");
    assert_eq!(mismatch_actual, mismatch_expected);
    assert!(matches!(mismatch_actual, PackError::RefKindMismatch { .. }));
}

#[test]
fn profile_bootstrap_from_pack_set_is_pure_after_source_cleanup() {
    let compiler = PackCompiler::new();
    let source_root = unique_temp_dir("runtime-bootstrap-pure-source");
    copy_dir_recursive(
        &crate_root().join("fixtures/pack/valid/bundle"),
        &source_root,
    );

    let profile = compiler
        .compile_profile(PackSource::File {
            path: source_root.join("profile_advanced_file_backed.toml"),
            format_hint: None,
        })
        .expect("profile should compile");
    let bundle = compiler
        .resolve_profile_pack_set(&profile)
        .expect("bundle should resolve");

    fs::remove_dir_all(&source_root).expect("source tree should be removable");

    let bootstrap = ProfileBootstrap::from_pack_set(bundle);

    assert_eq!(
        bootstrap.bundle.profile.header.id.as_str(),
        "acme/advanced-bundle"
    );
    assert_eq!(
        bootstrap
            .bundle
            .score_model
            .as_ref()
            .expect("score model after cleanup")
            .header
            .id
            .as_str(),
        "acme/lift-v2"
    );
    assert_eq!(bootstrap.bundle.query_packs.len(), 1);
    assert_eq!(bootstrap.bundle.rule_packs.len(), 1);
    assert_eq!(bootstrap.bundle.recipe_packs.len(), 1);
    assert!(bootstrap
        .bundle
        .diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.message.is_empty()));
}

fn load_bytes(relative: &str) -> Vec<u8> {
    fs::read(crate_root().join(relative))
        .unwrap_or_else(|err| panic!("failed to read fixture {relative}: {err}"))
}

fn write_temp_profile(label: &str, contents: &str) -> PackSource {
    let dir = unique_temp_dir(label);
    let path = dir.join("profile.toml");
    write_text(&path, contents);
    PackSource::File {
        path,
        format_hint: None,
    }
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

fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

fn copy_dir_recursive(source: &Path, target: &Path) {
    fs::create_dir_all(target)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", target.display()));
    for entry in fs::read_dir(source)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read dir entry: {err}"));
        let entry_type = entry
            .file_type()
            .unwrap_or_else(|err| panic!("failed to stat {}: {err}", entry.path().display()));
        let destination = target.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_recursive(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), &destination).unwrap_or_else(|err| {
                panic!(
                    "failed to copy {} to {}: {err}",
                    entry.path().display(),
                    destination.display()
                )
            });
        }
    }
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
