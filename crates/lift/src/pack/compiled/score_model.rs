//! Compiled score-model contracts.

use std::collections::BTreeSet;

use crate::kernel::JsonPointer;
use crate::pack::compiled::CompiledPackHeader;
use crate::pack::diagnostics::PackDiagnostic;
use crate::pack::CompiledExpr;

/// Compiled seam-1 score model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledScoreModel {
    pub header: CompiledPackHeader,
    pub vector_version: u32,
    pub lift_score: CompiledExpr,
    pub estimated_slices: CompiledExpr,
    pub triggers: Vec<CompiledTriggerRule>,
    pub confidence: CompiledConfidenceModel,
    pub missing_input_rules: Vec<CompiledMissingInputRule>,
    pub diagnostics: Vec<PackDiagnostic>,
}

/// Compiled trigger rule, preserving source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledTriggerRule {
    pub id: String,
    pub when: CompiledExpr,
}

/// Compiled confidence model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledConfidenceModel {
    pub default_level: String,
    pub rules: Vec<CompiledConfidenceRule>,
}

/// Compiled confidence rule, preserving source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledConfidenceRule {
    pub id: String,
    pub when: CompiledExpr,
    pub set: String,
    pub causes: BTreeSet<String>,
}

/// Compiled missing-input rule, preserving source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledMissingInputRule {
    pub field: JsonPointer,
    pub when: CompiledExpr,
}
