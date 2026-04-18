use assert_cmd as _;
use clap as _;
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
    AdapterDescriptor, AdapterName, AdapterParseResult, LangError, LanguageAdapter,
    LanguageRegistryBuilder, ParseInput,
};

struct TestAdapter {
    descriptor: AdapterDescriptor,
}

impl TestAdapter {
    fn new(name: &str, language: pack::LanguageId, version: &str) -> Self {
        Self {
            descriptor: AdapterDescriptor {
                name: AdapterName::parse(name).expect("adapter name should parse"),
                language,
                version: version.to_owned(),
            },
        }
    }
}

impl LanguageAdapter for TestAdapter {
    fn descriptor(&self) -> AdapterDescriptor {
        self.descriptor.clone()
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
fn built_in_registry_is_deterministic_for_active_feature_set() {
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
fn descriptors_are_sorted_independently_of_registration_order() {
    let forward = LanguageRegistryBuilder::new()
        .register(TestAdapter::new(
            "builtin.zeta",
            pack::LanguageId::Json,
            "1.0.0",
        ))
        .expect("zeta should register")
        .register(TestAdapter::new(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.1.0",
        ))
        .expect("alpha should register")
        .register(TestAdapter::new(
            "builtin.gamma",
            pack::LanguageId::Typescript,
            "1.2.0",
        ))
        .expect("gamma should register")
        .build()
        .expect("registry should build");

    let reverse = LanguageRegistryBuilder::new()
        .register(TestAdapter::new(
            "builtin.gamma",
            pack::LanguageId::Typescript,
            "1.2.0",
        ))
        .expect("gamma should register")
        .register(TestAdapter::new(
            "builtin.alpha",
            pack::LanguageId::Rust,
            "1.1.0",
        ))
        .expect("alpha should register")
        .register(TestAdapter::new(
            "builtin.zeta",
            pack::LanguageId::Json,
            "1.0.0",
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
