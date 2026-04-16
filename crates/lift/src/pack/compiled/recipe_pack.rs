//! Compiled recipe-pack contracts.

use std::collections::BTreeMap;

use crate::kernel::RecipeId;
use crate::pack::compiled::CompiledPackHeader;
use crate::pack::diagnostics::PackDiagnostic;
use crate::pack::CompiledQueryRef;

/// Compiled recipe pack keyed by deterministic recipe ids.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledRecipePack {
    pub header: CompiledPackHeader,
    pub recipes: BTreeMap<RecipeId, CompiledRecipeDef>,
    pub diagnostics: Vec<PackDiagnostic>,
}

/// Compiled recipe definition.
///
/// Recipe ids are derived from the identity lemma
/// `pack\0recipe_pack\0<pack-id>\0recipe\0<local-recipe-id>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledRecipeDef {
    pub local_id: String,
    pub id: RecipeId,
    pub summary: Option<String>,
    pub query: CompiledQueryRef,
    pub transforms: Vec<CompiledRecipeTransform>,
}

/// Compiled recipe transforms preserved in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompiledRecipeTransform {
    ReplaceCaptureText { capture: String, text: String },
}
