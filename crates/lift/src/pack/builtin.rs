//! Embedded built-in packs.

use crate::pack::source::{PackFormat, PackSource};

const GENERIC_DEFAULT_PROFILE: &str = r#"kind = "profile"
version = 1
id = "generic/default"
name = "Generic default profile"
description = "Default deterministic profile for generic repositories"

[apps]
enabled = ["score"]
default = "score"

[topology]
boundary_taxonomy = "builtin:generic/boundaries"
component_map = "builtin:generic/components"

[analysis]
languages = ["json", "toml", "yaml", "rust", "python", "javascript", "typescript"]
follow_symlinks = false
max_scope_depth = 2
"#;

const GENERIC_BOUNDARIES: &str = r#"{
  "kind": "boundary_taxonomy",
  "version": 1,
  "id": "generic/boundaries",
  "name": "Generic repository boundaries",
  "description": "Coarse boundary taxonomy for generic repositories.",
  "counting": {
    "mode": "distinct_minus_one"
  },
  "boundaries": [
    {
      "id": "application",
      "label": "Application surface",
      "include": ["apps/**", "services/**", "bin/**", "cmd/**"]
    },
    {
      "id": "library",
      "label": "Shared library surface",
      "include": ["crates/**", "packages/**", "libs/**", "src/**"]
    },
    {
      "id": "interface",
      "label": "Interface and contract surface",
      "include": ["api/**", "proto/**", "schemas/**", "contracts/**"]
    },
    {
      "id": "tests",
      "label": "Test surface",
      "include": ["tests/**", "test/**", "e2e/**", "integration-tests/**"]
    },
    {
      "id": "operations",
      "label": "Operations surface",
      "include": ["infra/**", "ops/**", "deploy/**", ".github/**", "scripts/**"]
    },
    {
      "id": "docs",
      "label": "Documentation surface",
      "include": ["docs/**"]
    }
  ]
}"#;

const GENERIC_COMPONENTS: &str = r#"{
  "kind": "component_map",
  "version": 1,
  "id": "generic/components",
  "name": "Generic repository components",
  "description": "Generic component map for common repository layouts.",
  "counting": {
    "mode": "distinct"
  },
  "components": [
    {
      "id": "frontend",
      "label": "Frontend",
      "include": ["web/**", "ui/**", "frontend/**", "apps/web/**", "packages/ui/**"],
      "tags": ["ui", "client"]
    },
    {
      "id": "backend",
      "label": "Backend",
      "include": ["api/**", "backend/**", "server/**", "services/**", "apps/api/**"],
      "tags": ["service", "server"]
    },
    {
      "id": "shared",
      "label": "Shared libraries",
      "include": ["crates/**", "packages/shared/**", "packages/config/**", "libs/**", "src/**"],
      "tags": ["shared", "library"]
    },
    {
      "id": "tooling",
      "label": "Tooling",
      "include": ["scripts/**", "tools/**", ".github/**", "infra/**"],
      "tags": ["tooling", "ops"]
    },
    {
      "id": "tests",
      "label": "Tests",
      "include": ["tests/**", "test/**", "e2e/**", "integration-tests/**"],
      "tags": ["test"]
    }
  ]
}"#;

/// Returns a built-in profile source by logical pack name.
pub(crate) fn profile_source(logical_name: &str) -> Option<PackSource> {
    match logical_name {
        "generic/default" => Some(PackSource::Builtin {
            logical_name: "generic/default",
            format: PackFormat::Toml,
            bytes: GENERIC_DEFAULT_PROFILE.as_bytes(),
        }),
        _ => None,
    }
}

/// Returns a built-in boundary taxonomy source by logical pack name.
pub(crate) fn boundary_taxonomy_source(logical_name: &str) -> Option<PackSource> {
    match logical_name {
        "generic/boundaries" => Some(PackSource::Builtin {
            logical_name: "generic/boundaries",
            format: PackFormat::Json,
            bytes: GENERIC_BOUNDARIES.as_bytes(),
        }),
        _ => None,
    }
}

/// Returns a built-in component map source by logical pack name.
pub(crate) fn component_map_source(logical_name: &str) -> Option<PackSource> {
    match logical_name {
        "generic/components" => Some(PackSource::Builtin {
            logical_name: "generic/components",
            format: PackFormat::Json,
            bytes: GENERIC_COMPONENTS.as_bytes(),
        }),
        _ => None,
    }
}

/// Returns a built-in score model source by logical pack name.
pub(crate) fn score_model_source(_logical_name: &str) -> Option<PackSource> {
    None
}

/// Returns a built-in query-pack source by logical pack name.
pub(crate) fn query_pack_source(_logical_name: &str) -> Option<PackSource> {
    None
}

/// Returns a built-in rule-pack source by logical pack name.
pub(crate) fn rule_pack_source(_logical_name: &str) -> Option<PackSource> {
    None
}

/// Returns a built-in recipe-pack source by logical pack name.
pub(crate) fn recipe_pack_source(_logical_name: &str) -> Option<PackSource> {
    None
}
