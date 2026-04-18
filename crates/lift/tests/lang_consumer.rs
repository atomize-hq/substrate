use std::collections::BTreeSet;

use assert_cmd as _;
use clap as _;
use gix as _;
use jsonschema as _;
use predicates as _;
use serde_jcs as _;
use sha2 as _;
use substrate_lift as _;
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
#[path = "support/repo_support.rs"]
mod repo_support;

include!("support/lang_support.rs");

fn config_key_inventory(parse_set: &lang::ParseSet) -> Vec<String> {
    let mut inventory = parse_set
        .units
        .iter()
        .flat_map(|unit| {
            unit.symbols
                .iter()
                .filter(|symbol| symbol.kind == lang::SymbolKind::ConfigKey)
                .map(|symbol| format!("{}::{}", unit.path.as_str(), symbol.path.join(".")))
        })
        .collect::<Vec<_>>();
    inventory.sort();
    inventory
}

fn edge_kinds(parse_set: &lang::ParseSet) -> BTreeSet<lang::LocalEdgeKind> {
    parse_set
        .units
        .iter()
        .flat_map(|unit| unit.edges.iter().map(|edge| edge.kind))
        .collect()
}

#[test]
fn toml_proof_slice_is_stable_across_repeat_runs() {
    let (_left_temp, _left_snapshot, left) = parse_fixture_case("repeat_run_repo");
    let (_right_temp, _right_snapshot, right) = parse_fixture_case("repeat_run_repo");

    assert_eq!(left, right);
    assert_eq!(left.stats.parsed_units, 2);
    assert!(left.failed.is_empty());

    let kinds = edge_kinds(&left);
    assert!(kinds.contains(&lang::LocalEdgeKind::Contains));
    assert!(kinds.contains(&lang::LocalEdgeKind::ConfigRef));
}

#[test]
fn hostile_malformed_toml_becomes_failed_parse_instead_of_crash() {
    let (_temp, _snapshot, parsed) = parse_fixture_case("malformed_repo");

    assert!(parsed.units.is_empty());
    assert_eq!(parsed.failed.len(), 1);
    assert_eq!(parsed.failed[0].path.as_str(), "configs/hostile.toml");
    assert_eq!(parsed.failed[0].language, pack::LanguageId::Toml);
    assert!(parsed.failed[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "lang.toml.parse_failed"));
    assert_eq!(parsed.stats.failed_units, 1);
}

#[test]
fn consumer_derives_deterministic_config_key_inventory_from_parse_set_without_rereading_raw_bytes()
{
    let (temp, _snapshot, parsed) = parse_fixture_case("valid_repo");

    let before = config_key_inventory(&parsed);

    repo_support::write_file(
        &temp.path().join("configs/app.toml"),
        br#"
[server]
port = 1
"#,
    );
    std::fs::remove_file(temp.path().join("configs/feature_flags.toml"))
        .expect("feature flag fixture should delete");

    let after = config_key_inventory(&parsed);

    assert_eq!(after, before);
    assert_eq!(
        after,
        vec![
            "configs/app.toml::name".to_owned(),
            "configs/app.toml::profiles".to_owned(),
            "configs/app.toml::profiles.primary".to_owned(),
            "configs/app.toml::profiles.primary.database_url".to_owned(),
            "configs/app.toml::profiles.primary.max_connections".to_owned(),
            "configs/app.toml::server".to_owned(),
            "configs/app.toml::server.port".to_owned(),
            "configs/app.toml::server.profile".to_owned(),
            "configs/feature_flags.toml::flags".to_owned(),
            "configs/feature_flags.toml::flags.enable_metrics".to_owned(),
            "configs/feature_flags.toml::flags.enable_search".to_owned(),
        ]
    );
}
