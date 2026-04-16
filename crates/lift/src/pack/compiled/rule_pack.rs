//! Compiled rule-pack contracts.

use std::collections::{BTreeMap, BTreeSet};

use crate::kernel::{RuleId, Severity};
use crate::pack::compiled::CompiledPackHeader;
use crate::pack::diagnostics::PackDiagnostic;
use crate::pack::names::LanguageId;
use crate::pack::CompiledQueryRef;

/// Reserved path-class vocabulary for compiled rule scopes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum ReservedPathClass {
    Test,
    Docs,
    Ci,
    Migration,
    Security,
    PublicApi,
    Generated,
    Vendor,
}

/// Compiled rule pack keyed by deterministic rule ids.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledRulePack {
    pub header: CompiledPackHeader,
    pub rules: BTreeMap<RuleId, CompiledRuleDef>,
    pub diagnostics: Vec<PackDiagnostic>,
}

/// Compiled rule definition.
///
/// Rule ids are derived from the identity lemma
/// `pack\0rule_pack\0<pack-id>\0rule\0<local-rule-id>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledRuleDef {
    pub local_id: String,
    pub id: RuleId,
    pub summary: Option<String>,
    pub severity: Severity,
    pub scope: Option<CompiledRuleScope>,
    pub query: CompiledQueryRef,
    pub emit: Vec<CompiledRuleEmit>,
}

/// Optional compiled scope for a rule definition.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CompiledRuleScope {
    pub languages: BTreeSet<LanguageId>,
    pub path_classes: BTreeSet<ReservedPathClass>,
}

/// Compiled emit actions preserved in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompiledRuleEmit {
    Finding { code: String, message: String },
}
