//! Internal pack compiler seam.

#![allow(dead_code)]
#![allow(unused_imports)]

pub(crate) mod builtin;
pub(crate) mod compiler;
pub(crate) mod diagnostics;
pub(crate) mod error;
pub(crate) mod expr;
pub(crate) mod names;
pub(crate) mod refs;
pub(crate) mod schema;
pub(crate) mod source;

pub(crate) mod compiled;
pub(crate) mod raw;

#[cfg(not(test))]
pub(crate) use crate::kernel::{BoundaryId, ComponentId};
pub(crate) use compiled::{
    BoundaryCountingMode, CompiledAnalysisDefaults, CompiledBoundary, CompiledBoundaryTaxonomy,
    CompiledComponent, CompiledComponentMap, CompiledConfidenceModel, CompiledConfidenceRule,
    CompiledGlobMatcher, CompiledMissingInputRule, CompiledPackHeader, CompiledPackSet,
    CompiledPathClasses, CompiledProfile, CompiledProfileApps, CompiledProfileIncludes,
    CompiledProfileScore, CompiledProfileTopology, CompiledQueryCapture, CompiledQueryDef,
    CompiledQueryPack, CompiledRecipeDef, CompiledRecipePack, CompiledRecipeTransform,
    CompiledRuleDef, CompiledRuleEmit, CompiledRulePack, CompiledRuleScope, CompiledScoreModel,
    CompiledTriggerRule, ComponentCountingMode, QueryEngineKind, ReservedPathClass,
    ResolvedProfileTopology,
};
pub(crate) use compiler::PackCompiler;
pub(crate) use diagnostics::{PackDiagnostic, PackLocation, PackRelatedLocation};
pub(crate) use error::{PackError, PackResult};
pub(crate) use expr::{compile_expr, compile_query_ref, CompiledExpr, CompiledQueryRef};
pub(crate) use names::{AppName, LanguageId, PackName};
pub(crate) use raw::PackKind;
pub(crate) use refs::{PackFileRef, PackRef};
pub(crate) use schema::{
    PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_FILE, PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_ID,
    PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_JSON, PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_VERSION,
    PACK_COMMON_V1_SCHEMA_FILE, PACK_COMMON_V1_SCHEMA_ID, PACK_COMMON_V1_SCHEMA_JSON,
    PACK_COMMON_V1_SCHEMA_VERSION, PACK_COMPONENT_MAP_V1_SCHEMA_FILE,
    PACK_COMPONENT_MAP_V1_SCHEMA_ID, PACK_COMPONENT_MAP_V1_SCHEMA_JSON,
    PACK_COMPONENT_MAP_V1_SCHEMA_VERSION, PACK_PROFILE_V1_SCHEMA_FILE, PACK_PROFILE_V1_SCHEMA_ID,
    PACK_PROFILE_V1_SCHEMA_JSON, PACK_PROFILE_V1_SCHEMA_VERSION, PACK_QUERY_PACK_V1_SCHEMA_FILE,
    PACK_QUERY_PACK_V1_SCHEMA_ID, PACK_QUERY_PACK_V1_SCHEMA_JSON,
    PACK_QUERY_PACK_V1_SCHEMA_VERSION, PACK_RECIPE_PACK_V1_SCHEMA_FILE,
    PACK_RECIPE_PACK_V1_SCHEMA_ID, PACK_RECIPE_PACK_V1_SCHEMA_JSON,
    PACK_RECIPE_PACK_V1_SCHEMA_VERSION, PACK_RULE_PACK_V1_SCHEMA_FILE, PACK_RULE_PACK_V1_SCHEMA_ID,
    PACK_RULE_PACK_V1_SCHEMA_JSON, PACK_RULE_PACK_V1_SCHEMA_VERSION,
    PACK_SCORE_MODEL_V1_SCHEMA_FILE, PACK_SCORE_MODEL_V1_SCHEMA_ID,
    PACK_SCORE_MODEL_V1_SCHEMA_JSON, PACK_SCORE_MODEL_V1_SCHEMA_VERSION,
};
pub(crate) use source::{PackFormat, PackOrigin, PackSource};
#[cfg(test)]
pub(crate) use substrate_lift::kernel::{BoundaryId, ComponentId};
