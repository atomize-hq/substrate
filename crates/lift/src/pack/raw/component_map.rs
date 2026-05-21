//! Raw component map pack document shape.

use serde::{Deserialize, Serialize};

use crate::pack::raw::PackKind;

/// Raw counting configuration for component map packs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawComponentMapCounting {
    pub mode: RawComponentMapCountingMode,
}

/// Supported counting modes for raw component map packs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RawComponentMapCountingMode {
    Distinct,
}

/// Raw component entry keyed by a pack-local component id.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawComponentEntry {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Raw seam-1 component map document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawComponentMap {
    pub kind: PackKind,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub counting: RawComponentMapCounting,
    #[serde(default)]
    pub components: Vec<RawComponentEntry>,
}
