//! Shared raw profile sections.

use serde::{Deserialize, Serialize};

/// Supported pack kinds in the seam-1 compiler domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PackKind {
    Profile,
    BoundaryTaxonomy,
    ComponentMap,
    ScoreModel,
    RulePack,
    QueryPack,
    RecipePack,
}

/// Raw apps section for a profile.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProfileApps {
    pub enabled: Option<Vec<String>>,
    pub default: Option<String>,
}

/// Raw analysis defaults section for a profile.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProfileAnalysis {
    pub languages: Option<Vec<String>>,
    pub follow_symlinks: Option<bool>,
    pub max_scope_depth: Option<u8>,
}

/// Raw list section for rule/query/recipe includes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawIncludeSection {
    #[serde(default)]
    pub packs: Vec<String>,
}
