//! Compiled query-pack output types.

use std::collections::BTreeMap;

use crate::kernel::QueryId;
use crate::pack::compiled::CompiledPackHeader;
use crate::pack::diagnostics::PackDiagnostic;
use crate::pack::names::LanguageId;

/// Supported compiled query engines for seam-1 query packs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum QueryEngineKind {
    TreeSitter,
}

impl QueryEngineKind {
    /// Returns the canonical engine identifier.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::TreeSitter => "tree_sitter",
        }
    }
}

/// Compiled query pack keyed by deterministic query ids.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledQueryPack {
    pub header: CompiledPackHeader,
    pub language: LanguageId,
    pub engine: QueryEngineKind,
    pub queries: BTreeMap<QueryId, CompiledQueryDef>,
    pub diagnostics: Vec<PackDiagnostic>,
}

/// Compiled query definition.
///
/// Query ids are derived from the identity lemma
/// `pack\0query_pack\0<pack-id>\0query\0<local-query-id>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledQueryDef {
    pub local_id: String,
    pub id: QueryId,
    pub summary: Option<String>,
    pub pattern: String,
    pub captures: Vec<CompiledQueryCapture>,
}

/// Compiled query capture metadata preserved for later engine compilation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledQueryCapture {
    pub name: String,
    pub required: bool,
}
