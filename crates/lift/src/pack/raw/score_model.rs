//! Raw score-model pack document shape.

use serde::{Deserialize, Serialize};

use crate::pack::raw::PackKind;

/// Raw trigger rule keyed by a pack-local trigger id.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawScoreTriggerRule {
    pub id: String,
    pub when: serde_json::Value,
}

/// Raw confidence rule keyed by a pack-local confidence-rule id.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawScoreConfidenceRule {
    pub id: String,
    pub when: serde_json::Value,
    pub set: String,
    #[serde(default)]
    pub causes: Vec<String>,
}

/// Raw confidence model for a score-model pack.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawScoreConfidenceModel {
    #[serde(rename = "default")]
    pub default_level: String,
    #[serde(default)]
    pub rules: Vec<RawScoreConfidenceRule>,
}

/// Raw missing-input rule keyed by the field it guards.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawScoreMissingInputRule {
    pub field: String,
    pub when: serde_json::Value,
}

/// Raw seam-1 score-model document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawScoreModel {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub kind: PackKind,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub vector_version: u32,
    pub lift_score: serde_json::Value,
    pub estimated_slices: serde_json::Value,
    #[serde(default)]
    pub triggers: Vec<RawScoreTriggerRule>,
    pub confidence: RawScoreConfidenceModel,
    #[serde(default)]
    pub missing_input_rules: Vec<RawScoreMissingInputRule>,
}
