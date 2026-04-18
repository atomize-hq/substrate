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

mod pack {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub(crate) enum LanguageId {
        Json,
        Toml,
        Yaml,
        Rust,
        Python,
        Javascript,
        Typescript,
    }

    impl LanguageId {
        pub(crate) fn as_str(self) -> &'static str {
            match self {
                Self::Json => "json",
                Self::Toml => "toml",
                Self::Yaml => "yaml",
                Self::Rust => "rust",
                Self::Python => "python",
                Self::Javascript => "javascript",
                Self::Typescript => "typescript",
            }
        }
    }
}

#[path = "../src/repo/mod.rs"]
mod repo;

#[path = "../src/lang/mod.rs"]
mod lang;

use lang::{
    AdapterDescriptor, AdapterName, AdapterParseResult, LangError, LanguageAdapter, LanguageId,
    LanguageRegistryBuilder, ParseInput,
};

struct TestAdapter {
    descriptor: AdapterDescriptor,
}

impl TestAdapter {
    fn new(name: &str, language: LanguageId, version: &str) -> Self {
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
fn duplicate_adapter_name_is_rejected() {
    let registration = LanguageRegistryBuilder::new()
        .register(TestAdapter::new("builtin.alpha", LanguageId::Rust, "1.0.0"))
        .expect("first adapter should register")
        .register(TestAdapter::new("builtin.alpha", LanguageId::Json, "1.0.0"));
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
        .register(TestAdapter::new("builtin.alpha", LanguageId::Rust, "1.0.0"))
        .expect("first adapter should register")
        .register(TestAdapter::new("builtin.beta", LanguageId::Rust, "1.0.0"));
    let error = match registration {
        Ok(_) => panic!("duplicate language should fail"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        LangError::DuplicateLanguageAdapter {
            language: LanguageId::Rust,
            existing: "builtin.alpha".to_owned(),
            duplicate: "builtin.beta".to_owned(),
        }
    );
}

#[test]
fn descriptors_are_sorted_independently_of_registration_order() {
    let forward = LanguageRegistryBuilder::new()
        .register(TestAdapter::new("builtin.zeta", LanguageId::Json, "1.0.0"))
        .expect("zeta should register")
        .register(TestAdapter::new("builtin.alpha", LanguageId::Rust, "1.1.0"))
        .expect("alpha should register")
        .register(TestAdapter::new(
            "builtin.gamma",
            LanguageId::Typescript,
            "1.2.0",
        ))
        .expect("gamma should register")
        .build()
        .expect("registry should build");

    let reverse = LanguageRegistryBuilder::new()
        .register(TestAdapter::new(
            "builtin.gamma",
            LanguageId::Typescript,
            "1.2.0",
        ))
        .expect("gamma should register")
        .register(TestAdapter::new("builtin.alpha", LanguageId::Rust, "1.1.0"))
        .expect("alpha should register")
        .register(TestAdapter::new("builtin.zeta", LanguageId::Json, "1.0.0"))
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
        .register(TestAdapter::new("builtin.alpha", LanguageId::Rust, "1.0.0"))
        .expect("rust adapter should register")
        .register(TestAdapter::new("builtin.beta", LanguageId::Json, "2.0.0"))
        .expect("json adapter should register")
        .build()
        .expect("registry should build");

    let rust_adapter = registry
        .adapter_for_language(LanguageId::Rust)
        .expect("rust adapter should exist")
        .descriptor();
    assert_eq!(rust_adapter.name.as_str(), "builtin.alpha");
    assert_eq!(rust_adapter.language, LanguageId::Rust);
    assert_eq!(rust_adapter.version, "1.0.0");

    assert!(registry
        .adapter_for_language(LanguageId::Typescript)
        .is_none());
}
