//! Compiled pack output types.

use std::collections::BTreeMap;

use crate::kernel::Fingerprint;
use crate::pack::diagnostics::PackDiagnostic;
use crate::pack::names::PackName;

mod header;
mod profile;
mod query_pack;
mod recipe_pack;
mod rule_pack;
mod score_model;
mod topology;

pub(crate) use header::CompiledPackHeader;
pub(crate) use profile::{
    CompiledAnalysisDefaults, CompiledPathClasses, CompiledProfile, CompiledProfileApps,
    CompiledProfileIncludes, CompiledProfileScore, CompiledProfileTopology,
};
pub(crate) use query_pack::{
    CompiledQueryCapture, CompiledQueryDef, CompiledQueryPack, QueryEngineKind,
};
pub(crate) use recipe_pack::{CompiledRecipeDef, CompiledRecipePack, CompiledRecipeTransform};
pub(crate) use rule_pack::{
    CompiledRuleDef, CompiledRuleEmit, CompiledRulePack, CompiledRuleScope, ReservedPathClass,
};
pub(crate) use score_model::{
    CompiledConfidenceModel, CompiledConfidenceRule, CompiledMissingInputRule, CompiledScoreModel,
    CompiledTriggerRule,
};
pub(crate) use topology::{
    BoundaryCountingMode, CompiledBoundary, CompiledBoundaryTaxonomy, CompiledComponent,
    CompiledComponentMap, CompiledGlobMatcher, ComponentCountingMode, ResolvedProfileTopology,
};

/// Fully resolved pack bundle for one compiled profile.
///
/// Phase C bundle shape:
///   profile
///     + optional topology packs
///     + optional score model
///     + query packs
///     + rule packs
///     + recipe packs
#[derive(Clone, Debug)]
pub(crate) struct CompiledPackSet {
    pub profile: CompiledProfile,
    pub boundary_taxonomy: Option<CompiledBoundaryTaxonomy>,
    pub component_map: Option<CompiledComponentMap>,
    pub score_model: Option<CompiledScoreModel>,
    pub rule_packs: BTreeMap<PackName, CompiledRulePack>,
    pub query_packs: BTreeMap<PackName, CompiledQueryPack>,
    pub recipe_packs: BTreeMap<PackName, CompiledRecipePack>,
    pub diagnostics: Vec<PackDiagnostic>,
    pub semantic_fingerprint: Fingerprint,
}
