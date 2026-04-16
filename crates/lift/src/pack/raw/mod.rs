//! Raw pack document shapes.

mod boundary_taxonomy;
mod common;
mod component_map;
mod profile;

pub(crate) use boundary_taxonomy::{
    RawBoundaryEntry, RawBoundaryTaxonomy, RawBoundaryTaxonomyCounting,
    RawBoundaryTaxonomyCountingMode,
};
pub(crate) use common::{PackKind, RawIncludeSection, RawProfileAnalysis, RawProfileApps};
pub(crate) use component_map::{
    RawComponentEntry, RawComponentMap, RawComponentMapCounting, RawComponentMapCountingMode,
};
pub(crate) use profile::{RawProfile, RawProfileScore, RawProfileTopology};
