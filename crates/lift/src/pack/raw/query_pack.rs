//! Raw query pack document shape.

use serde::{Deserialize, Serialize};

use crate::pack::raw::PackKind;

/// Raw query capture metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawQueryCapture {
    pub name: String,
    pub required: bool,
}

/// Raw query definition keyed by a pack-local query id.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawQueryDef {
    pub id: String,
    pub summary: Option<String>,
    pub pattern: String,
    pub captures: Vec<RawQueryCapture>,
}

/// Raw seam-1 query-pack document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawQueryPack {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub kind: PackKind,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub language: String,
    pub engine: String,
    pub queries: Vec<RawQueryDef>,
}
