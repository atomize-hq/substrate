//! Internal language platform seam.

#![allow(dead_code)]
#![allow(unused_imports)]

pub(crate) mod adapter;
pub(crate) mod driver;
pub(crate) mod error;
pub(crate) mod model;
pub(crate) mod registry;
pub(crate) mod schema;

pub(crate) use crate::pack::LanguageId;

pub(crate) use adapter::{
    AdapterDescriptor, AdapterName, AdapterParseOutput, AdapterParseResult, LanguageAdapter,
    ParseInput,
};
pub(crate) use driver::ParseDriver;
pub(crate) use error::{LangError, LangResult};
pub(crate) use model::{
    compare_symbol_drafts, sort_local_edges, sort_local_symbols, sort_surface_markers,
    symbol_identity_lemma, EdgeEndpoint, EdgeEndpointDraft, FailedParse, LocalEdge, LocalEdgeDraft,
    LocalEdgeKind, LocalSymbol, LocalSymbolDraft, MissingRequestedLanguage, ParseRequest,
    ParseScope, ParseSet, ParseStats, ParsedUnit, ReferenceTarget, ReferenceTargetDraft,
    SkippedParse, SkippedReason, SurfaceMarker, SurfaceMarkerDraft, SurfaceMarkerKind, SymbolKind,
    SymbolVisibility,
};
pub(crate) use registry::{LanguageRegistry, LanguageRegistryBuilder};
pub(crate) use schema::{
    LANG_PARSE_MANIFEST_V1_SCHEMA_FILE, LANG_PARSE_MANIFEST_V1_SCHEMA_ID,
    LANG_PARSE_MANIFEST_V1_SCHEMA_JSON, LANG_PARSE_MANIFEST_V1_SCHEMA_VERSION,
};
