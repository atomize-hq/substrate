//! Compiled pack header metadata.

use crate::kernel::Fingerprint;
use crate::pack::names::PackName;
use crate::pack::raw::PackKind;
use crate::pack::source::PackOrigin;

/// Stable header emitted for every compiled pack.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledPackHeader {
    pub kind: PackKind,
    pub id: PackName,
    pub version: u32,
    pub name: String,
    pub description: Option<String>,
    pub schema_id: &'static str,
    pub origin: PackOrigin,
    pub source_fingerprint: Fingerprint,
    pub semantic_fingerprint: Fingerprint,
}
