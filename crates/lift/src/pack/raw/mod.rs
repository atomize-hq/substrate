//! Raw pack document shapes.

mod common;
mod profile;

pub(crate) use common::{PackKind, RawIncludeSection, RawProfileAnalysis, RawProfileApps};
pub(crate) use profile::{RawProfile, RawProfileScore, RawProfileTopology};
