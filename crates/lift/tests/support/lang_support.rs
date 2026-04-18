use std::collections::BTreeMap;

use toml::Value;

#[allow(dead_code)]
mod lang {
    pub(crate) use crate::pack::{LanguageId, QueryEngineKind};

    pub(crate) mod adapter {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/adapter.rs"));
    }
    pub(crate) mod cache {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/cache.rs"));
    }
    pub(crate) mod capabilities {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/lang/capabilities.rs"
        ));
    }
    pub(crate) mod driver {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/driver.rs"));
    }
    pub(crate) mod error {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/error.rs"));
    }
    pub(crate) mod model {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/model.rs"));
    }
    pub(crate) mod registry {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/registry.rs"));
    }

    pub(crate) use adapter::{
        AdapterDescriptor, AdapterName, AdapterParseOutput, AdapterParseResult, LanguageAdapter,
        ParseInput,
    };
    pub(crate) use cache::{
        CachedParseOutcome, NoopParseCache, ParseCache, ParseCacheKey, ParseCacheLookup,
    };
    pub(crate) use capabilities::AdapterCapabilities;
    pub(crate) use driver::ParseDriver;
    pub(crate) use error::{LangError, LangResult};
    pub(crate) use model::{
        compare_symbol_drafts, sort_local_edges, sort_local_symbols, sort_surface_markers,
        symbol_identity_lemma, EdgeEndpoint, EdgeEndpointDraft, FailedParse, LocalEdge,
        LocalEdgeDraft, LocalEdgeKind, LocalSymbol, LocalSymbolDraft, MissingRequestedLanguage,
        ParseRequest, ParseScope, ParseSet, ParseStats, ParsedUnit, ReferenceTarget,
        ReferenceTargetDraft, SkippedParse, SkippedReason, SurfaceMarker, SurfaceMarkerDraft,
        SymbolKind, SymbolVisibility,
    };
    pub(crate) use registry::{LanguageRegistry, LanguageRegistryBuilder};
}

#[derive(Clone, Debug)]
pub(crate) struct TomlTestAdapter {
    descriptor: lang::AdapterDescriptor,
}

impl TomlTestAdapter {
    pub(crate) fn new() -> Self {
        Self {
            descriptor: lang::AdapterDescriptor {
                name: lang::AdapterName::parse("test.toml_proof").expect("adapter name"),
                language: pack::LanguageId::Toml,
                version: "proof-v1".to_owned(),
            },
        }
    }
}

impl lang::LanguageAdapter for TomlTestAdapter {
    fn descriptor(&self) -> lang::AdapterDescriptor {
        self.descriptor.clone()
    }

    fn recognizes(&self, input: &lang::ParseInput<'_>) -> bool {
        input.path.as_str().ends_with(".toml")
    }

    fn parse(&self, input: &lang::ParseInput<'_>) -> lang::AdapterParseResult {
        let text = match std::str::from_utf8(input.bytes) {
            Ok(text) => text,
            Err(error) => {
                return lang::AdapterParseResult::Failed {
                    diagnostics: vec![file_diagnostic(
                        "lang.toml.invalid_utf8",
                        format!("file is not valid UTF-8: {error}"),
                        input.path,
                    )],
                };
            }
        };

        let document: Value = match toml::from_str(text) {
            Ok(document) => document,
            Err(error) => {
                return lang::AdapterParseResult::Failed {
                    diagnostics: vec![file_diagnostic(
                        "lang.toml.parse_failed",
                        format!("failed to parse TOML: {error}"),
                        input.path,
                    )],
                };
            }
        };

        let mut collector = TomlCollector::new();
        match document {
            Value::Table(table) => collector.walk_table(&[], &table),
            other => collector.walk_value(&[], &other),
        }

        let (symbols, edges) = collector.finish();

        lang::AdapterParseResult::Parsed(lang::AdapterParseOutput {
            symbols,
            edges,
            surface_markers: Vec::new(),
            diagnostics: Vec::new(),
        })
    }
}

struct TomlCollector {
    symbols: BTreeMap<String, lang::LocalSymbolDraft>,
    edges: Vec<lang::LocalEdgeDraft>,
    edge_keys: BTreeSet<String>,
}

impl TomlCollector {
    fn new() -> Self {
        Self {
            symbols: BTreeMap::new(),
            edges: Vec::new(),
            edge_keys: BTreeSet::new(),
        }
    }

    fn walk_table(&mut self, prefix: &[String], table: &toml::map::Map<String, Value>) {
        let mut entries = table.iter().collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));

        for (key, value) in entries {
            let mut path = prefix.to_vec();
            path.push(key.clone());
            self.ensure_symbol(&path);
            self.push_contains(prefix, &path);
            self.walk_value(&path, value);
        }
    }

    fn walk_value(&mut self, current: &[String], value: &Value) {
        match value {
            Value::Table(table) => self.walk_table(current, table),
            Value::Array(items) => {
                for item in items {
                    if let Value::String(text) = item {
                        self.push_config_ref(current, text);
                    }
                }
            }
            Value::String(text) => self.push_config_ref(current, text),
            Value::Boolean(_)
            | Value::Datetime(_)
            | Value::Float(_)
            | Value::Integer(_) => {}
        }
    }

    fn ensure_symbol(&mut self, path: &[String]) -> String {
        let local_key = symbol_local_key(path);
        self.symbols
            .entry(local_key.clone())
            .or_insert_with(|| lang::LocalSymbolDraft {
                local_key: local_key.clone(),
                kind: lang::SymbolKind::ConfigKey,
                name: path.last().cloned(),
                path: path.to_vec(),
                span: crate::kernel::ByteSpan::new(0, 0).expect("zero span"),
                visibility: lang::SymbolVisibility::Unknown,
            });
        local_key
    }

    fn push_contains(&mut self, parent: &[String], child: &[String]) {
        let child_local_key = self.ensure_symbol(child);
        let source = if parent.is_empty() {
            lang::EdgeEndpointDraft::FileRoot
        } else {
            lang::EdgeEndpointDraft::Symbol {
                local_key: self.ensure_symbol(parent),
            }
        };
        let target = lang::ReferenceTargetDraft::LocalSymbol {
            local_key: child_local_key,
        };
        self.push_edge(lang::LocalEdgeKind::Contains, source, target);
    }

    fn push_config_ref(&mut self, source_path: &[String], value: &str) {
        let Some(parts) = parse_config_ref(value) else {
            return;
        };
        let source = lang::EdgeEndpointDraft::Symbol {
            local_key: self.ensure_symbol(source_path),
        };
        let target = lang::ReferenceTargetDraft::QualifiedName { parts };
        self.push_edge(lang::LocalEdgeKind::ConfigRef, source, target);
    }

    fn push_edge(
        &mut self,
        kind: lang::LocalEdgeKind,
        source: lang::EdgeEndpointDraft,
        target: lang::ReferenceTargetDraft,
    ) {
        let edge_key = format!(
            "{}|{}|{}",
            edge_kind_key(kind),
            endpoint_key(&source),
            target_key(&target)
        );
        if self.edge_keys.insert(edge_key) {
            self.edges.push(lang::LocalEdgeDraft {
                kind,
                source,
                target,
                span: None,
            });
        }
    }

    fn finish(self) -> (Vec<lang::LocalSymbolDraft>, Vec<lang::LocalEdgeDraft>) {
        (self.symbols.into_values().collect(), self.edges)
    }
}

fn symbol_local_key(path: &[String]) -> String {
    format!("config\0{}", path.join("\0"))
}

fn parse_config_ref(value: &str) -> Option<Vec<String>> {
    let tail = value.strip_prefix("config://")?;
    let parts = tail
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    (!parts.is_empty()).then_some(parts)
}

fn file_diagnostic(code: &str, message: String, path: &crate::kernel::RepoPath) -> crate::kernel::Diagnostic {
    crate::kernel::Diagnostic {
        code: crate::kernel::DiagnosticCode::parse(code).expect("diagnostic code"),
        severity: crate::kernel::Severity::Error,
        message,
        subject: Some(crate::kernel::Locator {
            path: path.clone(),
            span: None,
            json_pointer: None,
        }),
        related: Vec::new(),
        help: None,
    }
}

fn edge_kind_key(kind: lang::LocalEdgeKind) -> &'static str {
    match kind {
        lang::LocalEdgeKind::Contains => "contains",
        lang::LocalEdgeKind::Import => "import",
        lang::LocalEdgeKind::Export => "export",
        lang::LocalEdgeKind::Call => "call",
        lang::LocalEdgeKind::TypeRef => "type_ref",
        lang::LocalEdgeKind::Inherit => "inherit",
        lang::LocalEdgeKind::Implement => "implement",
        lang::LocalEdgeKind::TestRef => "test_ref",
        lang::LocalEdgeKind::ConfigRef => "config_ref",
        lang::LocalEdgeKind::SchemaRef => "schema_ref",
        lang::LocalEdgeKind::Unknown => "unknown",
    }
}

fn endpoint_key(endpoint: &lang::EdgeEndpointDraft) -> String {
    match endpoint {
        lang::EdgeEndpointDraft::FileRoot => "file_root".to_owned(),
        lang::EdgeEndpointDraft::Symbol { local_key } => format!("symbol:{local_key}"),
    }
}

fn target_key(target: &lang::ReferenceTargetDraft) -> String {
    match target {
        lang::ReferenceTargetDraft::LocalSymbol { local_key } => {
            format!("local_symbol:{local_key}")
        }
        lang::ReferenceTargetDraft::QualifiedName { parts } => {
            format!("qualified:{}", parts.join("."))
        }
        lang::ReferenceTargetDraft::FilePath { path } => format!("file:{}", path.as_str()),
        lang::ReferenceTargetDraft::ExternalPackage { package, symbol } => {
            format!("package:{package}:{}", symbol.clone().unwrap_or_default())
        }
        lang::ReferenceTargetDraft::Opaque { value } => format!("opaque:{value}"),
    }
}

pub(crate) fn fixture_case_path(case: &str) -> String {
    format!("fixtures/lang/proof/toml_consumer_repo/{case}")
}

pub(crate) fn materialize_fixture_case(case: &str) -> (repo_support::TempDir, repo::RepoSnapshot) {
    let temp = repo_support::copy_fixture_tree(
        &fixture_case_path(case),
        &format!("lang-proof-{case}"),
    );
    let snapshot = repo_support::materialize(temp.path(), repo_support::default_snapshot_options());
    (temp, snapshot)
}

pub(crate) fn parse_snapshot_with_toml(
    snapshot: &repo::RepoSnapshot,
    request: lang::ParseRequest,
) -> lang::ParseSet {
    let registry = lang::LanguageRegistryBuilder::new()
        .register(TomlTestAdapter::new())
        .expect("register TOML adapter")
        .build()
        .expect("build registry");

    lang::ParseDriver::new(registry)
        .parse_snapshot(snapshot, &request)
        .expect("parse snapshot")
}

pub(crate) fn parse_fixture_case(
    case: &str,
) -> (repo_support::TempDir, repo::RepoSnapshot, lang::ParseSet) {
    let (temp, snapshot) = materialize_fixture_case(case);
    let parse_set = parse_snapshot_with_toml(&snapshot, toml_snapshot_request());
    (temp, snapshot, parse_set)
}

pub(crate) fn toml_snapshot_request() -> lang::ParseRequest {
    let mut languages = BTreeSet::new();
    languages.insert(pack::LanguageId::Toml);
    lang::ParseRequest {
        languages,
        scope: lang::ParseScope::Snapshot,
    }
}
