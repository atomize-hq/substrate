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
fn semantic_fingerprint_is_stable_across_toml_key_reordering() {
    let compiler = pack::PackCompiler::new();

    let first = compile_file(
        &compiler,
        "fixtures/pack/canonical/generic_default_order_a.toml",
    );
    let second = compile_inline(
        &compiler,
        "reordered",
        "fixtures/pack/canonical/generic_default_order_b.toml",
    );

    assert_eq!(
        first.header.semantic_fingerprint,
        second.header.semantic_fingerprint
    );
}

#[test]
fn omitted_optional_fields_normalize_to_the_same_semantic_output() {
    let compiler = pack::PackCompiler::new();

    let omitted = compile_file(&compiler, "fixtures/pack/canonical/minimal_omitted.toml");
    let explicit = compile_file(
        &compiler,
        "fixtures/pack/canonical/minimal_explicit_defaults.toml",
    );

    assert_eq!(
        omitted.header.semantic_fingerprint,
        explicit.header.semantic_fingerprint
    );
    assert_ne!(
        omitted.header.source_fingerprint,
        explicit.header.source_fingerprint
    );
    assert_eq!(
        omitted.apps.default,
        pack::AppName::parse("score").expect("default app")
    );
    assert_eq!(omitted.apps.enabled.len(), 1);
    assert!(omitted
        .analysis
        .languages
        .contains(&pack::LanguageId::parse("rust").expect("language")));
    assert!(!omitted.analysis.follow_symlinks);
    assert_eq!(omitted.analysis.max_scope_depth, 2);
}

#[test]
fn source_fingerprint_differs_while_semantic_fingerprint_matches() {
    let compiler = pack::PackCompiler::new();

    let file = compile_file(
        &compiler,
        "fixtures/pack/canonical/generic_default_order_a.toml",
    );
    let inline = compile_inline(
        &compiler,
        "same-semantics",
        "fixtures/pack/canonical/generic_default_order_b.toml",
    );

    assert_eq!(
        file.header.semantic_fingerprint,
        inline.header.semantic_fingerprint
    );
    assert_ne!(
        file.header.source_fingerprint,
        inline.header.source_fingerprint
    );
}

#[test]
fn builtin_file_and_inline_sources_are_semantically_equivalent() {
    let compiler = pack::PackCompiler::new();

    let builtin = compiler
        .compile_profile(pack::builtin::profile_source("generic/default").expect("builtin"))
        .expect("builtin should compile");
    let file = compile_file(
        &compiler,
        "fixtures/pack/canonical/generic_default_order_a.toml",
    );
    let inline = compile_inline(
        &compiler,
        "builtin-equivalent",
        "fixtures/pack/canonical/generic_default_order_b.toml",
    );

    assert_eq!(
        builtin.header.semantic_fingerprint,
        file.header.semantic_fingerprint
    );
    assert_eq!(
        builtin.header.semantic_fingerprint,
        inline.header.semantic_fingerprint
    );
    assert_ne!(
        builtin.header.source_fingerprint,
        file.header.source_fingerprint
    );
    assert_ne!(
        builtin.header.source_fingerprint,
        inline.header.source_fingerprint
    );
}

fn compile_file(compiler: &pack::PackCompiler, relative: &str) -> pack::CompiledProfile {
    compiler
        .compile_profile(pack::PackSource::File {
            path: crate_root().join(relative),
            format_hint: None,
        })
        .unwrap_or_else(|err| panic!("failed to compile {relative}: {err:?}"))
}

fn compile_inline(
    compiler: &pack::PackCompiler,
    logical_name: &str,
    relative: &str,
) -> pack::CompiledProfile {
    compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: logical_name.to_owned(),
            format: pack::PackFormat::Toml,
            bytes: load_bytes(relative),
        })
        .unwrap_or_else(|err| panic!("failed to compile {relative} inline: {err:?}"))
}

fn load_bytes(relative: &str) -> Vec<u8> {
    let path = crate_root().join(relative);
    fs::read(&path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", path.display());
    })
}

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
