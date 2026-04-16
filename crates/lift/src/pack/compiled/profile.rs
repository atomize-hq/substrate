//! Compiled profile output types.

use std::collections::BTreeSet;

use crate::pack::compiled::CompiledPackHeader;
use crate::pack::diagnostics::PackDiagnostic;
use crate::pack::names::{AppName, LanguageId};
use crate::pack::refs::PackRef;

/// Compiled seam-1 profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledProfile {
    pub header: CompiledPackHeader,
    pub apps: CompiledProfileApps,
    pub analysis: CompiledAnalysisDefaults,
    pub topology: CompiledProfileTopology,
    pub score: CompiledProfileScore,
    pub includes: CompiledProfileIncludes,
    pub diagnostics: Vec<PackDiagnostic>,
}

/// Compiled application defaults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledProfileApps {
    pub enabled: BTreeSet<AppName>,
    pub default: AppName,
}

/// Compiled analysis defaults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledAnalysisDefaults {
    pub languages: BTreeSet<LanguageId>,
    pub follow_symlinks: bool,
    pub max_scope_depth: u8,
}

/// Deferred topology references selected by a profile.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CompiledProfileTopology {
    pub boundary_taxonomy: Option<PackRef>,
    pub component_map: Option<PackRef>,
    pub classes: CompiledPathClasses,
}

/// Deferred score model reference selected by a profile.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CompiledProfileScore {
    pub model: Option<PackRef>,
}

/// Deferred rule/query/recipe include lists selected by a profile.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CompiledProfileIncludes {
    pub rule_packs: BTreeSet<PackRef>,
    pub query_packs: BTreeSet<PackRef>,
    pub recipe_packs: BTreeSet<PackRef>,
}

/// Placeholder for later topology class compilation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CompiledPathClasses;
