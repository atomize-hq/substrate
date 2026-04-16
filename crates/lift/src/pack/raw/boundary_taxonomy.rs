//! Raw boundary taxonomy pack document shape.

use serde::{Deserialize, Serialize};

use crate::pack::raw::PackKind;

/// Raw counting configuration for boundary taxonomy packs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawBoundaryTaxonomyCounting {
    pub mode: RawBoundaryTaxonomyCountingMode,
}

/// Supported counting modes for raw boundary taxonomy packs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RawBoundaryTaxonomyCountingMode {
    DistinctMinusOne,
}

/// Raw boundary entry keyed by a pack-local boundary id.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawBoundaryEntry {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Raw seam-1 boundary taxonomy document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawBoundaryTaxonomy {
    pub kind: PackKind,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub counting: RawBoundaryTaxonomyCounting,
    #[serde(default)]
    pub boundaries: Vec<RawBoundaryEntry>,
}
