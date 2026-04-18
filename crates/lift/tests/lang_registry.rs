#![allow(unused_crate_dependencies)]

use std::collections::BTreeSet;

use assert_cmd as _;
use gix as _;
use globset as _;
use jsonschema as _;
use predicates as _;
use serde as _;
use serde_jcs as _;
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

#[path = "../src/repo/mod.rs"]
mod repo;

#[path = "../src/lang/mod.rs"]
mod lang;

use lang::{
    AdapterCapabilities, AdapterDescriptor, AdapterName, AdapterParseResult, LangError,
    LanguageAdapter, LanguageRegistryBuilder, ParseInput, QueryEngineKind,
};

struct TestAdapter {
    descriptor: AdapterDescriptor,
    capabilities: AdapterCapabilities,
}

impl TestAdapter {
    fn new(name: &str, language: pack::LanguageId, version: &str) -> Self {
        Self::with_capabilities(name, language, version, AdapterCapabilities::default())
    }

    fn with_capabilities(
        name: &str,
        language: pack::LanguageId,
        version: &str,
        capabilities: AdapterCapabilities,
    ) -> Self {
        Self {
            descriptor: AdapterDescriptor {
                name: AdapterName::parse(name).expect("adapter name should parse"),
                language,
                version: version.to_owned(),
            },
            capabilities,
        }
    }
}

impl LanguageAdapter for TestAdapter {
    fn descriptor(&self) -> AdapterDescriptor {
        self.descriptor.clone()
    }

    fn capabilities(&self) -> AdapterCapabilities {
        self.capabilities.clone()
    }

    fn recognizes(&self, _input: &ParseInput<'_>) -> bool {
        true
    }

    fn parse(&self, _input: &ParseInput<'_>) -> AdapterParseResult {
        AdapterParseResult::Failed {
            diagnostics: Vec::new(),
        }
    }
}

#[test]
fn built_in_registry_remains_deterministic_and_empty_before_seam4() {
    let first = lang::registry::built_in_registry().expect("built-in registry should build");
    let second = lang::registry::built_in_registry().expect("built-in registry should build");

    assert_eq!(first.descriptors(), second.descriptors());
    assert!(first.descriptors().is_empty());
    assert!(second.descriptors().is_empty());
    assert!(first.adapter_for_language(pack::LanguageId::Rust).is_none());
    assert!(second
        .adapter_for_language(pack::LanguageId::Rust)
        .is_none());
}

#[test]
fn registry_exposes_capabilities_in_language_scoped_lookup() {
    let rust_capabilities = AdapterCapabilities {
        emits_local_edges: true,
        emits_surface_markers: false,
        query_engines: BTreeSet::from([QueryEngineKind::TreeSitter]),
    };
    let registry = LanguageRegistryBuilder::new()
        .register(TestAdapter::with_capabilities(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.0.0",
            rust_capabilities.clone(),
        ))
        .expect("rust adapter should register")
        .register(TestAdapter::with_capabilities(
            "builtin.beta",
            pack::LanguageId::Json,
            "2.0.0",
            AdapterCapabilities {
                emits_local_edges: false,
                emits_surface_markers: true,
                query_engines: BTreeSet::new(),
            },
        ))
        .expect("json adapter should register")
        .build()
        .expect("registry should build");

    assert_eq!(
        registry.capabilities_for_language(pack::LanguageId::Rust),
        Some(rust_capabilities),
    );
    assert_eq!(
        registry.capabilities_for_language(pack::LanguageId::Typescript),
        None,
    );
}

#[test]
fn supported_query_engines_lookup_is_language_scoped() {
    let registry = LanguageRegistryBuilder::new()
        .register(TestAdapter::with_capabilities(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.0.0",
            AdapterCapabilities {
                emits_local_edges: true,
                emits_surface_markers: false,
                query_engines: BTreeSet::from([QueryEngineKind::TreeSitter]),
            },
        ))
        .expect("rust adapter should register")
        .register(TestAdapter::with_capabilities(
            "builtin.beta",
            pack::LanguageId::Json,
            "2.0.0",
            AdapterCapabilities::default(),
        ))
        .expect("json adapter should register")
        .build()
        .expect("registry should build");

    assert_eq!(
        registry.supported_query_engines_for_language(pack::LanguageId::Rust),
        BTreeSet::from([QueryEngineKind::TreeSitter]),
    );
    assert_eq!(
        registry.supported_query_engines_for_language(pack::LanguageId::Json),
        BTreeSet::new(),
    );
    assert_eq!(
        registry.supported_query_engines_for_language(pack::LanguageId::Typescript),
        BTreeSet::new(),
    );
}

#[test]
fn duplicate_adapter_name_is_rejected() {
    let registration = LanguageRegistryBuilder::new()
        .register(TestAdapter::new(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.0.0",
        ))
        .expect("first adapter should register")
        .register(TestAdapter::new(
            "builtin.alpha",
            pack::LanguageId::Json,
            "1.0.0",
        ));
    let error = match registration {
        Ok(_) => panic!("duplicate name should fail"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        LangError::DuplicateAdapterName {
            name: "builtin.alpha".to_owned(),
        }
    );
}

#[test]
fn duplicate_language_registration_is_rejected() {
    let registration = LanguageRegistryBuilder::new()
        .register(TestAdapter::new(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.0.0",
        ))
        .expect("first adapter should register")
        .register(TestAdapter::new(
            "builtin.beta",
            pack::LanguageId::Rust,
            "1.0.0",
        ));
    let error = match registration {
        Ok(_) => panic!("duplicate language should fail"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        LangError::DuplicateLanguageAdapter {
            language: pack::LanguageId::Rust,
            existing: "builtin.alpha".to_owned(),
            duplicate: "builtin.beta".to_owned(),
        }
    );
}

#[test]
fn descriptor_order_is_unchanged_by_capability_metadata() {
    let forward = LanguageRegistryBuilder::new()
        .register(TestAdapter::with_capabilities(
            "builtin.zeta",
            pack::LanguageId::Json,
            "1.0.0",
            AdapterCapabilities {
                emits_local_edges: false,
                emits_surface_markers: true,
                query_engines: BTreeSet::new(),
            },
        ))
        .expect("zeta should register")
        .register(TestAdapter::with_capabilities(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.1.0",
            AdapterCapabilities {
                emits_local_edges: true,
                emits_surface_markers: false,
                query_engines: BTreeSet::from([QueryEngineKind::TreeSitter]),
            },
        ))
        .expect("alpha should register")
        .register(TestAdapter::with_capabilities(
            "builtin.gamma",
            pack::LanguageId::Typescript,
            "1.2.0",
            AdapterCapabilities {
                emits_local_edges: true,
                emits_surface_markers: true,
                query_engines: BTreeSet::from([QueryEngineKind::TreeSitter]),
            },
        ))
        .expect("gamma should register")
        .build()
        .expect("registry should build");

    let reverse = LanguageRegistryBuilder::new()
        .register(TestAdapter::with_capabilities(
            "builtin.gamma",
            pack::LanguageId::Typescript,
            "1.2.0",
            AdapterCapabilities {
                emits_local_edges: false,
                emits_surface_markers: false,
                query_engines: BTreeSet::new(),
            },
        ))
        .expect("gamma should register")
        .register(TestAdapter::with_capabilities(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.1.0",
            AdapterCapabilities::default(),
        ))
        .expect("alpha should register")
        .register(TestAdapter::with_capabilities(
            "builtin.zeta",
            pack::LanguageId::Json,
            "1.0.0",
            AdapterCapabilities {
                emits_local_edges: true,
                emits_surface_markers: true,
                query_engines: BTreeSet::from([QueryEngineKind::TreeSitter]),
            },
        ))
        .expect("zeta should register")
        .build()
        .expect("registry should build");

    let expected_names = vec![
        "builtin.alpha".to_owned(),
        "builtin.gamma".to_owned(),
        "builtin.zeta".to_owned(),
    ];

    assert_eq!(
        forward
            .descriptors()
            .iter()
            .map(|descriptor| descriptor.name.as_str().to_owned())
            .collect::<Vec<_>>(),
        expected_names,
    );
    assert_eq!(forward.descriptors(), reverse.descriptors());
}

#[test]
fn canonical_built_in_language_order_is_frozen() {
    assert_eq!(
        lang::registry::BUILT_IN_LANGUAGE_ORDER,
        &[
            pack::LanguageId::Json,
            pack::LanguageId::Toml,
            pack::LanguageId::Yaml,
            pack::LanguageId::Rust,
            pack::LanguageId::Python,
            pack::LanguageId::Javascript,
            pack::LanguageId::Typescript,
        ],
    );
}

#[test]
fn adapter_lookup_returns_registered_language_adapter() {
    let registry = LanguageRegistryBuilder::new()
        .register(TestAdapter::new(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.0.0",
        ))
        .expect("rust adapter should register")
        .register(TestAdapter::new(
            "builtin.beta",
            pack::LanguageId::Json,
            "2.0.0",
        ))
        .expect("json adapter should register")
        .build()
        .expect("registry should build");

    let rust_adapter = registry
        .adapter_for_language(pack::LanguageId::Rust)
        .expect("rust adapter should exist")
        .descriptor();
    assert_eq!(rust_adapter.name.as_str(), "builtin.alpha");
    assert_eq!(rust_adapter.language, pack::LanguageId::Rust);
    assert_eq!(rust_adapter.version, "1.0.0");

    assert!(registry
        .adapter_for_language(pack::LanguageId::Typescript)
        .is_none());
}
