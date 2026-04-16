//! Raw rule-pack document shape.

use serde::{Deserialize, Serialize};

use crate::kernel::Severity;
use crate::pack::names::LanguageId;
use crate::pack::raw::PackKind;

/// Raw reserved path-class vocabulary for rule scopes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RawReservedPathClass {
    Test,
    Docs,
    Ci,
    Migration,
    Security,
    PublicApi,
    Generated,
    Vendor,
}

/// Raw query reference preserved structurally until bundle resolution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawRuleQueryRef {
    pub pack: String,
    pub id: String,
}

/// Optional structural scope for a rule definition.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawRuleScope {
    pub languages: Option<Vec<LanguageId>>,
    pub path_classes: Option<Vec<RawReservedPathClass>>,
}

/// Raw emit actions preserved in source order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RawRuleEmit {
    Finding { code: String, message: String },
}

/// Raw rule definition keyed by a pack-local rule id.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawRuleDef {
    pub id: String,
    pub summary: Option<String>,
    pub severity: Severity,
    pub scope: Option<RawRuleScope>,
    pub query: RawRuleQueryRef,
    pub emit: Vec<RawRuleEmit>,
}

/// Raw seam-1 rule-pack document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawRulePack {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub kind: PackKind,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<RawRuleDef>,
}
