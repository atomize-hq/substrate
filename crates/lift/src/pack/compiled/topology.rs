//! Compiled topology contracts for boundary taxonomies and component maps.

use std::collections::{BTreeMap, BTreeSet};

use globset::GlobSet;

use crate::kernel::Fingerprint;
use crate::pack::compiled::CompiledPackHeader;
use crate::pack::diagnostics::PackDiagnostic;
use crate::pack::{BoundaryId, ComponentId};

/// Compiled counting mode for a boundary taxonomy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum BoundaryCountingMode {
    DistinctMinusOne,
}

/// Compiled counting mode for a component map.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum ComponentCountingMode {
    Distinct,
}

/// Compiled boundary taxonomy keyed by deterministic boundary ids.
#[derive(Clone, Debug)]
pub(crate) struct CompiledBoundaryTaxonomy {
    pub header: CompiledPackHeader,
    pub counting_mode: BoundaryCountingMode,
    pub boundaries: BTreeMap<BoundaryId, CompiledBoundary>,
    pub diagnostics: Vec<PackDiagnostic>,
}

/// Compiled boundary entry.
///
/// Boundary ids are derived from the identity lemma
/// `pack\0boundary_taxonomy\0<pack-id>\0boundary\0<local-boundary-id>`.
#[derive(Clone, Debug)]
pub(crate) struct CompiledBoundary {
    pub local_id: String,
    pub id: BoundaryId,
    pub label: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub include_matcher: GlobSet,
    pub exclude_matcher: GlobSet,
}

/// Compiled component map keyed by deterministic component ids.
#[derive(Clone, Debug)]
pub(crate) struct CompiledComponentMap {
    pub header: CompiledPackHeader,
    pub counting_mode: ComponentCountingMode,
    pub components: BTreeMap<ComponentId, CompiledComponent>,
    pub diagnostics: Vec<PackDiagnostic>,
}

/// Compiled component entry.
///
/// Component ids are derived from the identity lemma
/// `pack\0component_map\0<pack-id>\0component\0<local-component-id>`.
#[derive(Clone, Debug)]
pub(crate) struct CompiledComponent {
    pub local_id: String,
    pub id: ComponentId,
    pub label: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub tags: BTreeSet<String>,
    pub include_matcher: GlobSet,
    pub exclude_matcher: GlobSet,
}

/// Topology packs resolved for a compiled profile.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedProfileTopology {
    pub boundary_taxonomy: Option<CompiledBoundaryTaxonomy>,
    pub component_map: Option<CompiledComponentMap>,
    pub semantic_fingerprint: Fingerprint,
}

pub(crate) type CompiledGlobMatcher = GlobSet;
