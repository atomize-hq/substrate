//! Raw recipe-pack document shape.

use serde::{Deserialize, Serialize};

use crate::pack::raw::PackKind;

/// Raw query reference preserved structurally until bundle resolution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawRecipeQueryRef {
    pub pack: String,
    pub id: String,
}

/// Raw recipe transform preserved in source order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RawRecipeTransform {
    ReplaceCaptureText { capture: String, text: String },
}

/// Raw recipe definition keyed by a pack-local recipe id.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawRecipeDef {
    pub id: String,
    pub summary: Option<String>,
    pub query: RawRecipeQueryRef,
    pub transforms: Vec<RawRecipeTransform>,
}

/// Raw seam-1 recipe-pack document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawRecipePack {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub kind: PackKind,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub recipes: Vec<RawRecipeDef>,
}
