//! Raw profile document shape.

use serde::{Deserialize, Serialize};

use crate::pack::raw::{PackKind, RawIncludeSection, RawProfileAnalysis, RawProfileApps};

/// Raw topology references inside a profile.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProfileTopology {
    pub boundary_taxonomy: Option<String>,
    pub component_map: Option<String>,
}

/// Raw score configuration inside a profile.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProfileScore {
    pub model: Option<String>,
}

/// Raw seam-1 profile document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProfile {
    pub kind: PackKind,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub apps: Option<RawProfileApps>,
    pub analysis: Option<RawProfileAnalysis>,
    pub topology: Option<RawProfileTopology>,
    pub score: Option<RawProfileScore>,
    pub rules: Option<RawIncludeSection>,
    pub queries: Option<RawIncludeSection>,
    pub recipes: Option<RawIncludeSection>,
}
