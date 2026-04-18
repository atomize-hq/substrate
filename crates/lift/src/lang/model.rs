use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::kernel::{
    sha256_canonical_json, ByteSpan, Diagnostic, FileId, Fingerprint, RepoPath, SymbolId,
};
use crate::lang::{AdapterName, LanguageId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SymbolKind {
    Module,
    Namespace,
    Package,
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    TypeAlias,
    Field,
    Constant,
    Variable,
    TestCase,
    TestSuite,
    ConfigKey,
    Unknown,
}

impl SymbolKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Namespace => "namespace",
            Self::Package => "package",
            Self::Function => "function",
            Self::Method => "method",
            Self::Class => "class",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Interface => "interface",
            Self::TypeAlias => "type_alias",
            Self::Field => "field",
            Self::Constant => "constant",
            Self::Variable => "variable",
            Self::TestCase => "test_case",
            Self::TestSuite => "test_suite",
            Self::ConfigKey => "config_key",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SymbolVisibility {
    Public,
    Private,
    Internal,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalSymbolDraft {
    pub local_key: String,
    pub kind: SymbolKind,
    pub name: Option<String>,
    pub path: Vec<String>,
    pub span: ByteSpan,
    pub visibility: SymbolVisibility,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalSymbol {
    pub id: SymbolId,
    pub kind: SymbolKind,
    pub name: Option<String>,
    pub path: Vec<String>,
    pub span: ByteSpan,
    pub visibility: SymbolVisibility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LocalEdgeKind {
    Contains,
    Import,
    Export,
    Call,
    TypeRef,
    Inherit,
    Implement,
    TestRef,
    ConfigRef,
    SchemaRef,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum EdgeEndpoint {
    FileRoot,
    Symbol(SymbolId),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum ReferenceTargetDraft {
    LocalSymbol {
        local_key: String,
    },
    QualifiedName {
        parts: Vec<String>,
    },
    FilePath {
        path: RepoPath,
    },
    ExternalPackage {
        package: String,
        symbol: Option<String>,
    },
    Opaque {
        value: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum ReferenceTarget {
    LocalSymbol(SymbolId),
    QualifiedName {
        parts: Vec<String>,
    },
    FilePath {
        path: RepoPath,
    },
    ExternalPackage {
        package: String,
        symbol: Option<String>,
    },
    Opaque {
        value: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalEdgeDraft {
    pub kind: LocalEdgeKind,
    pub source: EdgeEndpointDraft,
    pub target: ReferenceTargetDraft,
    pub span: Option<ByteSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum EdgeEndpointDraft {
    FileRoot,
    Symbol { local_key: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalEdge {
    pub kind: LocalEdgeKind,
    pub source: EdgeEndpoint,
    pub target: ReferenceTarget,
    pub span: Option<ByteSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SurfaceMarkerKind {
    PublicApi,
    Test,
    EntryPoint,
    Export,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SurfaceMarkerDraft {
    pub kind: SurfaceMarkerKind,
    pub symbol_local_key: Option<String>,
    pub span: Option<ByteSpan>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SurfaceMarker {
    pub kind: SurfaceMarkerKind,
    pub symbol: Option<SymbolId>,
    pub span: Option<ByteSpan>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "paths", rename_all = "snake_case")]
pub(crate) enum ParseScope {
    Snapshot,
    Paths(BTreeSet<RepoPath>),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ParseRequest {
    pub languages: BTreeSet<LanguageId>,
    pub scope: ParseScope,
}

impl ParseRequest {
    pub(crate) fn normalized(&self) -> Self {
        Self {
            languages: self.languages.clone(),
            scope: match &self.scope {
                ParseScope::Snapshot => ParseScope::Snapshot,
                ParseScope::Paths(paths) => ParseScope::Paths(paths.clone()),
            },
        }
    }

    pub(crate) fn fingerprint(&self) -> crate::kernel::KernelResult<Fingerprint> {
        sha256_canonical_json(&self.normalized())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct MissingRequestedLanguage {
    pub language: LanguageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ParsedUnit {
    pub path: RepoPath,
    pub file_id: FileId,
    pub blob_fingerprint: Fingerprint,
    pub language: LanguageId,
    pub adapter: AdapterName,
    pub adapter_version: String,
    pub unit_fingerprint: Fingerprint,
    pub symbols: Vec<LocalSymbol>,
    pub edges: Vec<LocalEdge>,
    pub surface_markers: Vec<SurfaceMarker>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ParsedUnit {
    pub(crate) fn fingerprint(&self) -> crate::kernel::KernelResult<Fingerprint> {
        #[derive(Serialize)]
        struct FingerprintDoc<'a> {
            path: &'a RepoPath,
            file_id: &'a FileId,
            blob_fingerprint: &'a Fingerprint,
            language: LanguageId,
            adapter: &'a AdapterName,
            adapter_version: &'a str,
            symbols: &'a [LocalSymbol],
            edges: &'a [LocalEdge],
            surface_markers: &'a [SurfaceMarker],
            diagnostics: &'a [Diagnostic],
        }

        sha256_canonical_json(&FingerprintDoc {
            path: &self.path,
            file_id: &self.file_id,
            blob_fingerprint: &self.blob_fingerprint,
            language: self.language,
            adapter: &self.adapter,
            adapter_version: &self.adapter_version,
            symbols: &self.symbols,
            edges: &self.edges,
            surface_markers: &self.surface_markers,
            diagnostics: &self.diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct FailedParse {
    pub path: RepoPath,
    pub file_id: FileId,
    pub blob_fingerprint: Fingerprint,
    pub language: LanguageId,
    pub adapter: AdapterName,
    pub adapter_version: String,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SkippedReason {
    NoMatchingAdapter,
    PathNotInSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SkippedParse {
    pub path: RepoPath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<FileId>,
    pub reason: SkippedReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ParseStats {
    pub considered_files: u64,
    pub parsed_units: u64,
    pub failed_units: u64,
    pub skipped_no_adapter: u64,
    pub skipped_missing_paths: u64,
    pub missing_requested_languages: u64,
    pub diagnostic_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ParseSet {
    pub snapshot_fingerprint: Fingerprint,
    pub request: ParseRequest,
    pub request_fingerprint: Fingerprint,
    pub units: Vec<ParsedUnit>,
    pub failed: Vec<FailedParse>,
    pub skipped: Vec<SkippedParse>,
    pub missing_languages: Vec<MissingRequestedLanguage>,
    pub diagnostics: Vec<Diagnostic>,
    pub stats: ParseStats,
}

impl ParseSet {
    pub(crate) fn sort_all(&mut self) {
        self.units.sort_by(compare_parsed_units);
        self.failed.sort_by(compare_failed_parses);
        self.skipped.sort_by(compare_skipped_parses);
        self.missing_languages.sort_by(compare_missing_languages);
        self.diagnostics.sort();
    }

    pub(crate) fn refresh_stats(&mut self) {
        self.stats.parsed_units = self.units.len() as u64;
        self.stats.failed_units = self.failed.len() as u64;
        self.stats.missing_requested_languages = self.missing_languages.len() as u64;
        self.stats.diagnostic_count = self.diagnostics.len() as u64
            + self
                .units
                .iter()
                .map(|unit| unit.diagnostics.len() as u64)
                .sum::<u64>()
            + self
                .failed
                .iter()
                .map(|unit| unit.diagnostics.len() as u64)
                .sum::<u64>();
    }
}

pub(crate) fn sort_local_symbols(symbols: &mut [LocalSymbol]) {
    symbols.sort_by(compare_local_symbols);
}

pub(crate) fn sort_local_edges(edges: &mut [LocalEdge]) {
    edges.sort_by(compare_local_edges);
}

pub(crate) fn sort_surface_markers(markers: &mut [SurfaceMarker]) {
    markers.sort_by(compare_surface_markers);
}

pub(crate) fn symbol_identity_lemma(
    language: LanguageId,
    path: &RepoPath,
    draft: &LocalSymbolDraft,
    duplicate_ordinal: usize,
) -> String {
    format!(
        "lang\0symbol\0v1\0{}\0{}\0{}\0{}\0{}",
        language.as_str(),
        path.as_str(),
        draft.kind.as_str(),
        draft.path.join("\0"),
        duplicate_ordinal
    )
}

pub(crate) fn compare_symbol_drafts(left: &LocalSymbolDraft, right: &LocalSymbolDraft) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.span.cmp(&right.span))
        .then_with(|| left.local_key.cmp(&right.local_key))
}

pub(crate) fn compare_parsed_units(left: &ParsedUnit, right: &ParsedUnit) -> Ordering {
    left.path
        .cmp(&right.path)
        .then_with(|| left.adapter.cmp(&right.adapter))
        .then_with(|| left.file_id.as_str().cmp(right.file_id.as_str()))
}

pub(crate) fn compare_failed_parses(left: &FailedParse, right: &FailedParse) -> Ordering {
    left.path
        .cmp(&right.path)
        .then_with(|| left.adapter.cmp(&right.adapter))
        .then_with(|| left.file_id.as_str().cmp(right.file_id.as_str()))
}

pub(crate) fn compare_skipped_parses(left: &SkippedParse, right: &SkippedParse) -> Ordering {
    left.path
        .cmp(&right.path)
        .then_with(|| left.reason.cmp(&right.reason))
        .then_with(|| {
            left.file_id
                .as_ref()
                .map(|id| id.as_str())
                .cmp(&right.file_id.as_ref().map(|id| id.as_str()))
        })
        .then_with(|| left.detail.cmp(&right.detail))
}

pub(crate) fn compare_missing_languages(
    left: &MissingRequestedLanguage,
    right: &MissingRequestedLanguage,
) -> Ordering {
    left.language
        .cmp(&right.language)
        .then_with(|| left.detail.cmp(&right.detail))
}

fn compare_local_symbols(left: &LocalSymbol, right: &LocalSymbol) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.span.cmp(&right.span))
        .then_with(|| left.id.as_str().cmp(right.id.as_str()))
}

fn compare_local_edges(left: &LocalEdge, right: &LocalEdge) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| compare_edge_endpoints(&left.source, &right.source))
        .then_with(|| compare_reference_targets(&left.target, &right.target))
        .then_with(|| left.span.cmp(&right.span))
}

fn compare_surface_markers(left: &SurfaceMarker, right: &SurfaceMarker) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| {
            left.symbol
                .as_ref()
                .map(|id| id.as_str())
                .cmp(&right.symbol.as_ref().map(|id| id.as_str()))
        })
        .then_with(|| left.span.cmp(&right.span))
        .then_with(|| left.label.cmp(&right.label))
}

fn compare_edge_endpoints(left: &EdgeEndpoint, right: &EdgeEndpoint) -> Ordering {
    match (left, right) {
        (EdgeEndpoint::FileRoot, EdgeEndpoint::FileRoot) => Ordering::Equal,
        (EdgeEndpoint::FileRoot, EdgeEndpoint::Symbol(_)) => Ordering::Less,
        (EdgeEndpoint::Symbol(_), EdgeEndpoint::FileRoot) => Ordering::Greater,
        (EdgeEndpoint::Symbol(left), EdgeEndpoint::Symbol(right)) => {
            left.as_str().cmp(right.as_str())
        }
    }
}

fn compare_reference_targets(left: &ReferenceTarget, right: &ReferenceTarget) -> Ordering {
    reference_target_rank(left)
        .cmp(&reference_target_rank(right))
        .then_with(|| match (left, right) {
            (ReferenceTarget::LocalSymbol(left), ReferenceTarget::LocalSymbol(right)) => {
                left.as_str().cmp(right.as_str())
            }
            (
                ReferenceTarget::QualifiedName { parts: left },
                ReferenceTarget::QualifiedName { parts: right },
            ) => left.cmp(right),
            (
                ReferenceTarget::FilePath { path: left },
                ReferenceTarget::FilePath { path: right },
            ) => left.cmp(right),
            (
                ReferenceTarget::ExternalPackage {
                    package: left_package,
                    symbol: left_symbol,
                },
                ReferenceTarget::ExternalPackage {
                    package: right_package,
                    symbol: right_symbol,
                },
            ) => left_package
                .cmp(right_package)
                .then_with(|| left_symbol.cmp(right_symbol)),
            (ReferenceTarget::Opaque { value: left }, ReferenceTarget::Opaque { value: right }) => {
                left.cmp(right)
            }
            _ => Ordering::Equal,
        })
}

fn reference_target_rank(target: &ReferenceTarget) -> u8 {
    match target {
        ReferenceTarget::LocalSymbol(_) => 0,
        ReferenceTarget::QualifiedName { .. } => 1,
        ReferenceTarget::FilePath { .. } => 2,
        ReferenceTarget::ExternalPackage { .. } => 3,
        ReferenceTarget::Opaque { .. } => 4,
    }
}
