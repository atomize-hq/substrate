#![allow(unused_crate_dependencies)]

use std::collections::BTreeSet;

use assert_cmd as _;
use gix as _;
use globset as _;
use jsonschema as _;
use predicates as _;
use serde_json as _;
use sha2 as _;
use thiserror as _;
use toml as _;
use walkdir as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::*;
}

#[path = "../src/pack/mod.rs"]
mod pack;

#[path = "../src/lang/mod.rs"]
mod lang;

#[path = "../src/repo/mod.rs"]
mod repo;

use lang::{
    AdapterCapabilities, AdapterDescriptor, AdapterName, AdapterParseResult, LanguageAdapter,
    ParseInput, QueryEngineKind,
};

struct TestAdapter {
    descriptor: AdapterDescriptor,
}

impl TestAdapter {
    fn new() -> Self {
        Self {
            descriptor: AdapterDescriptor {
                name: AdapterName::parse("builtin.capabilities").expect("adapter name"),
                language: pack::LanguageId::Rust,
                version: "1.0.0".to_owned(),
            },
        }
    }
}

impl LanguageAdapter for TestAdapter {
    fn descriptor(&self) -> AdapterDescriptor {
        self.descriptor.clone()
    }

    fn recognizes(&self, _input: &ParseInput<'_>) -> bool {
        false
    }

    fn parse(&self, _input: &ParseInput<'_>) -> AdapterParseResult {
        AdapterParseResult::Failed {
            diagnostics: Vec::new(),
        }
    }
}

#[test]
fn default_adapter_capabilities_are_empty_and_backwards_compatible() {
    let adapter = TestAdapter::new();

    assert_eq!(adapter.capabilities(), AdapterCapabilities::default());
    assert_eq!(
        serde_json::from_str::<AdapterCapabilities>("{}").expect("deserialize empty object"),
        AdapterCapabilities::default()
    );
}

#[test]
fn adapter_capabilities_serde_round_trip_is_canonical() {
    let capabilities = AdapterCapabilities {
        emits_local_edges: true,
        emits_surface_markers: true,
        query_engines: BTreeSet::from([QueryEngineKind::TreeSitter]),
    };

    let canonical =
        kernel::canonical_json_string(&capabilities).expect("serialize canonical capabilities");
    assert_eq!(
        canonical,
        r#"{"emits_local_edges":true,"emits_surface_markers":true,"query_engines":["tree_sitter"]}"#
    );

    let reparsed: AdapterCapabilities =
        serde_json::from_str(&canonical).expect("deserialize canonical capabilities");
    assert_eq!(reparsed, capabilities);
}
