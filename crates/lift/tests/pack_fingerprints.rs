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

#[test]
fn boundary_taxonomy_semantic_fingerprint_is_stable_across_json_key_reordering() {
    let compiler = pack::PackCompiler::new();

    let first = compile_boundary_file(
        &compiler,
        "fixtures/pack/canonical/boundary_taxonomy_order_a.json",
    );
    let second = compile_boundary_inline(
        &compiler,
        "boundary-reordered",
        "fixtures/pack/canonical/boundary_taxonomy_order_b.json",
    );

    assert_eq!(
        first.header.semantic_fingerprint,
        second.header.semantic_fingerprint
    );
}

#[test]
fn boundary_taxonomy_source_fingerprint_differs_while_semantics_match() {
    let compiler = pack::PackCompiler::new();

    let file = compile_boundary_file(
        &compiler,
        "fixtures/pack/canonical/boundary_taxonomy_order_a.json",
    );
    let inline = compile_boundary_inline(
        &compiler,
        "boundary-same-semantics",
        "fixtures/pack/canonical/boundary_taxonomy_order_b.json",
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
fn builtin_file_and_inline_boundary_taxonomies_are_semantically_equivalent() {
    let compiler = pack::PackCompiler::new();

    let builtin = compiler
        .compile_boundary_taxonomy(
            pack::builtin::boundary_taxonomy_source("generic/boundaries").expect("builtin"),
        )
        .expect("builtin boundary taxonomy should compile");
    let file = compile_boundary_file(&compiler, "fixtures/pack/valid/generic_boundaries.json");
    let inline = compile_boundary_inline(
        &compiler,
        "boundary-builtin-equivalent",
        "fixtures/pack/valid/generic_boundaries.json",
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
}

#[test]
fn component_map_semantic_fingerprint_is_stable_across_json_key_reordering() {
    let compiler = pack::PackCompiler::new();

    let first = compile_component_file(
        &compiler,
        "fixtures/pack/canonical/component_map_order_a.json",
    );
    let second = compile_component_inline(
        &compiler,
        "component-reordered",
        "fixtures/pack/canonical/component_map_order_b.json",
    );

    assert_eq!(
        first.header.semantic_fingerprint,
        second.header.semantic_fingerprint
    );
}

#[test]
fn component_map_source_fingerprint_differs_while_semantics_match() {
    let compiler = pack::PackCompiler::new();

    let file = compile_component_file(
        &compiler,
        "fixtures/pack/canonical/component_map_order_a.json",
    );
    let inline = compile_component_inline(
        &compiler,
        "component-same-semantics",
        "fixtures/pack/canonical/component_map_order_b.json",
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
fn builtin_file_and_inline_component_maps_are_semantically_equivalent() {
    let compiler = pack::PackCompiler::new();

    let builtin = compiler
        .compile_component_map(
            pack::builtin::component_map_source("generic/components").expect("builtin"),
        )
        .expect("builtin component map should compile");
    let file = compile_component_file(&compiler, "fixtures/pack/valid/generic_components.json");
    let inline = compile_component_inline(
        &compiler,
        "component-builtin-equivalent",
        "fixtures/pack/valid/generic_components.json",
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

#[test]
fn resolved_topology_fingerprint_is_stable_for_equivalent_profile_inputs() {
    let compiler = pack::PackCompiler::new();

    let builtin = compiler
        .compile_profile(pack::builtin::profile_source("generic/default").expect("builtin"))
        .expect("builtin profile should compile");
    let inline = compile_inline(
        &compiler,
        "resolved-topology-inline",
        "fixtures/pack/canonical/generic_default_order_b.toml",
    );

    let builtin_resolved = compiler
        .resolve_profile_topology(&builtin)
        .expect("builtin topology should resolve");
    let inline_resolved = compiler
        .resolve_profile_topology(&inline)
        .expect("inline topology should resolve");

    assert_eq!(
        builtin_resolved.semantic_fingerprint,
        inline_resolved.semantic_fingerprint
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

fn compile_boundary_file(
    compiler: &pack::PackCompiler,
    relative: &str,
) -> pack::CompiledBoundaryTaxonomy {
    compiler
        .compile_boundary_taxonomy(pack::PackSource::File {
            path: crate_root().join(relative),
            format_hint: None,
        })
        .unwrap_or_else(|err| panic!("failed to compile boundary {relative}: {err:?}"))
}

fn compile_boundary_inline(
    compiler: &pack::PackCompiler,
    logical_name: &str,
    relative: &str,
) -> pack::CompiledBoundaryTaxonomy {
    compiler
        .compile_boundary_taxonomy(pack::PackSource::Inline {
            logical_name: logical_name.to_owned(),
            format: pack::PackFormat::Json,
            bytes: load_bytes(relative),
        })
        .unwrap_or_else(|err| panic!("failed to compile boundary {relative} inline: {err:?}"))
}

fn compile_component_file(
    compiler: &pack::PackCompiler,
    relative: &str,
) -> pack::CompiledComponentMap {
    compiler
        .compile_component_map(pack::PackSource::File {
            path: crate_root().join(relative),
            format_hint: None,
        })
        .unwrap_or_else(|err| panic!("failed to compile component map {relative}: {err:?}"))
}

fn compile_component_inline(
    compiler: &pack::PackCompiler,
    logical_name: &str,
    relative: &str,
) -> pack::CompiledComponentMap {
    compiler
        .compile_component_map(pack::PackSource::Inline {
            logical_name: logical_name.to_owned(),
            format: pack::PackFormat::Json,
            bytes: load_bytes(relative),
        })
        .unwrap_or_else(|err| panic!("failed to compile component map {relative} inline: {err:?}"))
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
