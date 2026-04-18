use std::collections::BTreeSet;

use assert_cmd as _;
use clap as _;
use gix as _;
use jsonschema as _;
use predicates as _;
use serde::Deserialize;
use serde_jcs as _;
use serde_json::json;
use sha2 as _;
use substrate_lift as _;
use thiserror as _;
use toml as _;
use walkdir as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::*;
}
#[path = "../src/pack/mod.rs"]
mod pack;
#[path = "../src/repo/mod.rs"]
mod repo;
#[path = "support/repo_support.rs"]
mod repo_support;

mod lang {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    use crate::kernel::{Diagnostic, FileId, Fingerprint, RepoPath};

    pub(crate) use crate::pack::LanguageId;

    pub(crate) type LangResult<T> = Result<T, LangError>;

    #[derive(Debug, Error, Clone, Eq, PartialEq)]
    pub(crate) enum LangError {
        #[error("duplicate adapter name")]
        DuplicateAdapterName { name: String },

        #[error("duplicate language adapter registration")]
        DuplicateLanguageAdapter {
            language: LanguageId,
            existing: String,
            duplicate: String,
        },

        #[error("invalid adapter name")]
        InvalidAdapterName { input: String },

        #[error("parse cache invariant failure")]
        CacheInvariant { reason: String },

        #[error("lang schema validation failure")]
        SchemaViolation {
            schema_id: &'static str,
            reason: String,
        },
    }

    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
    #[serde(try_from = "String", into = "String")]
    pub(crate) struct AdapterName(String);

    impl AdapterName {
        pub(crate) fn parse(input: &str) -> LangResult<Self> {
            if valid_adapter_name(input) {
                Ok(Self(input.to_owned()))
            } else {
                Err(LangError::InvalidAdapterName {
                    input: input.to_owned(),
                })
            }
        }

        pub(crate) fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl TryFrom<String> for AdapterName {
        type Error = LangError;

        fn try_from(value: String) -> LangResult<Self> {
            Self::parse(&value)
        }
    }

    impl From<AdapterName> for String {
        fn from(value: AdapterName) -> Self {
            value.0
        }
    }

    fn valid_adapter_name(input: &str) -> bool {
        let mut segments = input.split('.');
        let Some(first) = segments.next() else {
            return false;
        };
        if !valid_adapter_segment(first, false) {
            return false;
        }
        let mut saw_tail = false;
        for segment in segments {
            saw_tail = true;
            if !valid_adapter_segment(segment, true) {
                return false;
            }
        }
        saw_tail
    }

    fn valid_adapter_segment(segment: &str, allow_underscore: bool) -> bool {
        let mut chars = segment.chars();
        matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
            && chars.all(|ch| {
                ch.is_ascii_lowercase() || ch.is_ascii_digit() || (allow_underscore && ch == '_')
            })
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct AdapterDescriptor {
        pub name: AdapterName,
        pub language: LanguageId,
        pub version: String,
    }

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct ParseInput<'a> {
        pub path: &'a RepoPath,
        pub file_id: &'a FileId,
        pub blob_fingerprint: &'a Fingerprint,
        pub bytes: &'a [u8],
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct AdapterParseOutput {
        pub symbols: Vec<LocalSymbolDraft>,
        pub edges: Vec<LocalEdgeDraft>,
        pub surface_markers: Vec<SurfaceMarkerDraft>,
        pub diagnostics: Vec<Diagnostic>,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) enum AdapterParseResult {
        Parsed(AdapterParseOutput),
        Failed { diagnostics: Vec<Diagnostic> },
    }

    pub(crate) trait LanguageAdapter: Send + Sync {
        fn descriptor(&self) -> AdapterDescriptor;
        fn recognizes(&self, input: &ParseInput<'_>) -> bool;
        fn parse(&self, input: &ParseInput<'_>) -> AdapterParseResult;
    }

    pub(crate) mod model {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/model.rs"));
    }
    #[allow(unused_imports)]
    pub(crate) use model::{
        compare_symbol_drafts, sort_local_edges, sort_local_symbols, sort_surface_markers,
        symbol_identity_lemma, EdgeEndpoint, EdgeEndpointDraft, FailedParse, LocalEdge,
        LocalEdgeDraft, LocalEdgeKind, LocalSymbol, LocalSymbolDraft, MissingRequestedLanguage,
        ParseRequest, ParseScope, ParseSet, ParseStats, ParsedUnit, ReferenceTarget,
        ReferenceTargetDraft, SkippedParse, SkippedReason, SurfaceMarker, SurfaceMarkerDraft,
        SurfaceMarkerKind, SymbolKind, SymbolVisibility,
    };

    #[derive(Default)]
    pub(crate) struct LanguageRegistryBuilder {
        adapters: BTreeMap<AdapterName, Arc<dyn LanguageAdapter>>,
        languages: BTreeMap<LanguageId, AdapterName>,
    }

    pub(crate) struct LanguageRegistry {
        adapters: BTreeMap<AdapterName, Arc<dyn LanguageAdapter>>,
        languages: BTreeMap<LanguageId, AdapterName>,
    }

    impl LanguageRegistryBuilder {
        pub(crate) fn new() -> Self {
            Self::default()
        }

        pub(crate) fn register<A: LanguageAdapter + 'static>(
            mut self,
            adapter: A,
        ) -> LangResult<Self> {
            let adapter = Arc::new(adapter) as Arc<dyn LanguageAdapter>;
            let descriptor = adapter.descriptor();
            if self.adapters.contains_key(&descriptor.name) {
                return Err(LangError::DuplicateAdapterName {
                    name: descriptor.name.as_str().to_owned(),
                });
            }
            if let Some(existing) = self.languages.get(&descriptor.language) {
                return Err(LangError::DuplicateLanguageAdapter {
                    language: descriptor.language,
                    existing: existing.as_str().to_owned(),
                    duplicate: descriptor.name.as_str().to_owned(),
                });
            }
            self.languages
                .insert(descriptor.language, descriptor.name.clone());
            self.adapters.insert(descriptor.name, adapter);
            Ok(self)
        }

        pub(crate) fn build(self) -> LangResult<LanguageRegistry> {
            Ok(LanguageRegistry {
                adapters: self.adapters,
                languages: self.languages,
            })
        }
    }

    impl LanguageRegistry {
        pub(crate) fn descriptors(&self) -> Vec<AdapterDescriptor> {
            self.languages
                .iter()
                .filter_map(|(_, adapter_name)| self.adapters.get(adapter_name))
                .map(|adapter| adapter.descriptor())
                .collect()
        }

        pub(crate) fn adapter_for_language(
            &self,
            language: LanguageId,
        ) -> Option<&Arc<dyn LanguageAdapter>> {
            let adapter_name = self.languages.get(&language)?;
            self.adapters.get(adapter_name)
        }
    }

    pub(crate) mod driver {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lang/driver.rs"));
    }
    pub(crate) use driver::ParseDriver;
}

use kernel::{Diagnostic, RepoPath};
use lang::{
    AdapterDescriptor, AdapterName, AdapterParseOutput, AdapterParseResult, LanguageAdapter,
    LanguageRegistryBuilder, LocalEdgeDraft, LocalSymbolDraft, ParseDriver, ParseInput,
    ParseRequest, ParseScope, SurfaceMarkerDraft,
};

#[derive(Clone, Debug, Deserialize)]
struct FakeDocument {
    #[serde(default)]
    fail: bool,
    #[serde(default)]
    symbols: Vec<LocalSymbolDraft>,
    #[serde(default)]
    edges: Vec<LocalEdgeDraft>,
    #[serde(default)]
    surface_markers: Vec<SurfaceMarkerDraft>,
    #[serde(default)]
    diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug)]
struct FakeAdapter {
    descriptor: AdapterDescriptor,
    suffix: &'static str,
    panic_recognizes: BTreeSet<String>,
    panic_parses: BTreeSet<String>,
}

impl FakeAdapter {
    fn new(name: &str, language: &str, suffix: &'static str) -> Self {
        Self {
            descriptor: AdapterDescriptor {
                name: AdapterName::parse(name).expect("adapter name"),
                language: pack::LanguageId::parse(language).expect("language"),
                version: "1.0.0".to_owned(),
            },
            suffix,
            panic_recognizes: BTreeSet::new(),
            panic_parses: BTreeSet::new(),
        }
    }

    fn with_recognize_panic(mut self, path: &str) -> Self {
        self.panic_recognizes.insert(path.to_owned());
        self
    }

    fn with_parse_panic(mut self, path: &str) -> Self {
        self.panic_parses.insert(path.to_owned());
        self
    }
}

impl LanguageAdapter for FakeAdapter {
    fn descriptor(&self) -> AdapterDescriptor {
        self.descriptor.clone()
    }

    fn recognizes(&self, input: &ParseInput<'_>) -> bool {
        if self.panic_recognizes.contains(input.path.as_str()) {
            panic!("recognize panic for {}", input.path.as_str());
        }
        input.path.as_str().ends_with(self.suffix)
    }

    fn parse(&self, input: &ParseInput<'_>) -> AdapterParseResult {
        if self.panic_parses.contains(input.path.as_str()) {
            panic!("parse panic for {}", input.path.as_str());
        }

        let document: FakeDocument = match serde_json::from_slice(input.bytes) {
            Ok(document) => document,
            Err(_) => {
                return AdapterParseResult::Failed {
                    diagnostics: Vec::new(),
                };
            }
        };

        if document.fail {
            return AdapterParseResult::Failed {
                diagnostics: document.diagnostics,
            };
        }

        AdapterParseResult::Parsed(AdapterParseOutput {
            symbols: document.symbols,
            edges: document.edges,
            surface_markers: document.surface_markers,
            diagnostics: document.diagnostics,
        })
    }
}

fn registry(adapters: Vec<FakeAdapter>) -> lang::LanguageRegistry {
    let mut builder = LanguageRegistryBuilder::new();
    for adapter in adapters {
        builder = builder.register(adapter).expect("register adapter");
    }
    builder.build().expect("build registry")
}

fn write_repo_files(temp: &repo_support::TempDir, files: &[(&str, &[u8])]) {
    repo_support::write_file(&temp.path().join(".git/HEAD"), b"ref: refs/heads/main\n");
    for (path, bytes) in files {
        repo_support::write_file(&temp.path().join(path), bytes);
    }
}

fn snapshot_with_files(files: &[(&str, &[u8])]) -> (repo_support::TempDir, repo::RepoSnapshot) {
    let temp = repo_support::TempDir::new("lang-parse");
    write_repo_files(&temp, files);
    let snapshot = repo_support::materialize(temp.path(), repo_support::default_snapshot_options());
    (temp, snapshot)
}

fn parse_snapshot(
    snapshot: &repo::RepoSnapshot,
    registry: lang::LanguageRegistry,
    request: ParseRequest,
) -> lang::ParseSet {
    ParseDriver::new(registry)
        .parse_snapshot(snapshot, &request)
        .expect("parse snapshot")
}

fn with_silenced_panic_hook<T>(work: impl FnOnce() -> T) -> T {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = work();
    std::panic::set_hook(hook);
    result
}

fn path_set(paths: &[&str]) -> BTreeSet<RepoPath> {
    paths
        .iter()
        .map(|path| RepoPath::parse(path).expect("repo path"))
        .collect()
}

fn language_set(languages: &[&str]) -> BTreeSet<pack::LanguageId> {
    languages
        .iter()
        .map(|language| pack::LanguageId::parse(language).expect("language"))
        .collect()
}

fn fake_symbol(local_key: &str, name: &str, start: u64, end: u64) -> serde_json::Value {
    json!({
        "local_key": local_key,
        "kind": "function",
        "name": name,
        "path": [name],
        "span": {
            "start_byte": start,
            "end_byte": end
        },
        "visibility": "public"
    })
}

fn fake_edge(source: &str, target: &str, start: u64, end: u64) -> serde_json::Value {
    json!({
        "kind": "call",
        "source": {
            "Symbol": {
                "local_key": source
            }
        },
        "target": {
            "LocalSymbol": {
                "local_key": target
            }
        },
        "span": {
            "start_byte": start,
            "end_byte": end
        }
    })
}

fn fake_marker(local_key: &str, start: u64, end: u64) -> serde_json::Value {
    json!({
        "kind": "entry_point",
        "symbol_local_key": local_key,
        "span": {
            "start_byte": start,
            "end_byte": end
        },
        "label": "entry"
    })
}

fn fake_diagnostic(code: &str, path: &str, start: u64, end: u64) -> serde_json::Value {
    json!({
        "code": code,
        "severity": "warning",
        "message": "adapter note",
        "subject": {
            "path": path,
            "span": {
                "start_byte": start,
                "end_byte": end
            }
        }
    })
}

#[test]
fn snapshot_parse_is_deterministic_and_uses_snapshot_bytes_only() {
    let original_json = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let original_yaml = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("cfg", "beta", 0, 4)]
    }))
    .expect("json");
    let (temp, snapshot) = snapshot_with_files(&[
        ("src/b.fake.json", &original_json),
        ("src/a.fake.yaml", &original_yaml),
    ]);

    repo_support::write_file(
        &temp.path().join("src/b.fake.json"),
        serde_json::to_string(&json!({
            "symbols": [fake_symbol("main", "mutated", 0, 7)]
        }))
        .expect("json")
        .as_bytes(),
    );

    let request = ParseRequest {
        languages: BTreeSet::new(),
        scope: ParseScope::Snapshot,
    };
    let left = parse_snapshot(
        &snapshot,
        registry(vec![
            FakeAdapter::new("builtin.fake_yaml", "yaml", ".fake.yaml"),
            FakeAdapter::new("builtin.fake_json", "json", ".fake.json"),
        ]),
        request.clone(),
    );
    let right = parse_snapshot(
        &snapshot,
        registry(vec![
            FakeAdapter::new("builtin.fake_json", "json", ".fake.json"),
            FakeAdapter::new("builtin.fake_yaml", "yaml", ".fake.yaml"),
        ]),
        request,
    );

    assert_eq!(left, right);
    assert_eq!(left.units.len(), 2);
    assert_eq!(left.units[0].path.as_str(), "src/a.fake.yaml");
    assert_eq!(left.units[1].path.as_str(), "src/b.fake.json");
    assert_eq!(left.units[1].symbols[0].name.as_deref(), Some("alpha"));
    assert_eq!(left.stats.parsed_units, 2);
    assert_eq!(left.stats.considered_files, 2);
    assert_eq!(left.stats.failed_units, 0);
}

#[test]
fn snapshot_scope_counts_no_matching_files_in_stats_only() {
    let matching = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[
        ("src/match.fake.json", &matching),
        ("README.txt", b"plain text"),
    ]);

    let parsed = parse_snapshot(
        &snapshot,
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
        )]),
        ParseRequest {
            languages: language_set(&["json"]),
            scope: ParseScope::Snapshot,
        },
    );

    assert_eq!(parsed.units.len(), 1);
    assert!(parsed.skipped.is_empty());
    assert_eq!(parsed.stats.considered_files, 2);
    assert_eq!(parsed.stats.skipped_no_adapter, 1);
}

#[test]
fn explicit_paths_emit_skips_for_missing_paths_and_no_matching_adapter() {
    let matching = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[
        ("src/match.fake.json", &matching),
        ("src/plain.txt", b"plain text"),
    ]);

    let parsed = parse_snapshot(
        &snapshot,
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
        )]),
        ParseRequest {
            languages: language_set(&["json"]),
            scope: ParseScope::Paths(path_set(&["src/plain.txt", "src/missing.txt"])),
        },
    );

    assert!(parsed.units.is_empty());
    assert_eq!(parsed.skipped.len(), 2);
    assert_eq!(parsed.skipped[0].path.as_str(), "src/missing.txt");
    assert_eq!(
        parsed.skipped[0].reason,
        lang::SkippedReason::PathNotInSnapshot
    );
    assert!(parsed.skipped[0].file_id.is_none());
    assert_eq!(parsed.skipped[1].path.as_str(), "src/plain.txt");
    assert_eq!(
        parsed.skipped[1].reason,
        lang::SkippedReason::NoMatchingAdapter
    );
    assert!(parsed.skipped[1].file_id.is_some());
    assert_eq!(parsed.stats.skipped_missing_paths, 1);
    assert_eq!(parsed.stats.skipped_no_adapter, 1);
    assert_eq!(parsed.stats.considered_files, 1);
}

#[test]
fn empty_explicit_scope_returns_a_deterministic_empty_parse_set() {
    let matching = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/match.fake.json", &matching)]);

    let parsed = parse_snapshot(
        &snapshot,
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
        )]),
        ParseRequest {
            languages: language_set(&["json"]),
            scope: ParseScope::Paths(BTreeSet::new()),
        },
    );

    assert!(parsed.units.is_empty());
    assert!(parsed.failed.is_empty());
    assert!(parsed.skipped.is_empty());
    assert_eq!(parsed.stats.considered_files, 0);
    assert_eq!(parsed.stats.parsed_units, 0);
    assert_eq!(parsed.stats.failed_units, 0);
    assert_eq!(parsed.stats.skipped_no_adapter, 0);
}

#[test]
fn missing_requested_languages_surface_run_level_records_and_diagnostics() {
    let matching = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/match.fake.json", &matching)]);

    let parsed = parse_snapshot(
        &snapshot,
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
        )]),
        ParseRequest {
            languages: language_set(&["json", "rust"]),
            scope: ParseScope::Snapshot,
        },
    );

    assert_eq!(parsed.units.len(), 1);
    assert_eq!(parsed.missing_languages.len(), 1);
    assert_eq!(parsed.missing_languages[0].language, pack::LanguageId::Rust);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(
        parsed.diagnostics[0].code.as_str(),
        "lang.parse.missing_registered_adapter"
    );
    assert_eq!(parsed.stats.missing_requested_languages, 1);
}

#[test]
fn adapter_parse_failures_become_failed_parse_records_with_deterministic_diagnostics() {
    let (_temp, snapshot) = snapshot_with_files(&[("src/malformed.fake.json", b"\xff\xfe\xfd")]);

    let parsed = parse_snapshot(
        &snapshot,
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
        )]),
        ParseRequest {
            languages: language_set(&["json"]),
            scope: ParseScope::Snapshot,
        },
    );

    assert!(parsed.units.is_empty());
    assert_eq!(parsed.failed.len(), 1);
    assert_eq!(parsed.failed[0].diagnostics.len(), 1);
    assert_eq!(
        parsed.failed[0].diagnostics[0].code.as_str(),
        "lang.parse.failed"
    );
}

#[test]
fn duplicate_local_keys_become_failed_parse_records() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [
            fake_symbol("dup", "alpha", 0, 5),
            fake_symbol("dup", "beta", 6, 10)
        ]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/dup.fake.json", &bytes)]);

    let parsed = parse_snapshot(
        &snapshot,
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
        )]),
        ParseRequest {
            languages: language_set(&["json"]),
            scope: ParseScope::Snapshot,
        },
    );

    assert!(parsed.units.is_empty());
    assert_eq!(parsed.failed.len(), 1);
    assert!(parsed.failed[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "lang.parse.duplicate_local_key"));
}

#[test]
fn unresolved_local_and_marker_refs_become_failed_parse_records() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)],
        "edges": [fake_edge("missing_source", "main", 0, 1)],
        "surface_markers": [fake_marker("missing_marker", 0, 1)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/unresolved.fake.json", &bytes)]);

    let parsed = parse_snapshot(
        &snapshot,
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
        )]),
        ParseRequest {
            languages: language_set(&["json"]),
            scope: ParseScope::Snapshot,
        },
    );

    assert!(parsed.units.is_empty());
    assert_eq!(parsed.failed.len(), 1);
    assert!(parsed.failed[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "lang.parse.unresolved_local_ref"));
    assert!(parsed.failed[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "lang.parse.unresolved_marker_ref"));
}

#[test]
fn invalid_spans_become_failed_parse_records() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 999)],
        "diagnostics": [fake_diagnostic("lang.fake.note", "src/invalid.fake.json", 0, 999)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/invalid.fake.json", &bytes)]);

    let parsed = parse_snapshot(
        &snapshot,
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
        )]),
        ParseRequest {
            languages: language_set(&["json"]),
            scope: ParseScope::Snapshot,
        },
    );

    assert!(parsed.units.is_empty());
    assert_eq!(parsed.failed.len(), 1);
    assert!(parsed.failed[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "lang.parse.invalid_span"));
}

#[test]
fn adapter_panics_are_contained_as_failed_parse_records() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/panic.fake.json", &bytes)]);

    let parsed = with_silenced_panic_hook(|| {
        parse_snapshot(
            &snapshot,
            registry(vec![FakeAdapter::new(
                "builtin.fake_json",
                "json",
                ".fake.json",
            )
            .with_parse_panic("src/panic.fake.json")]),
            ParseRequest {
                languages: language_set(&["json"]),
                scope: ParseScope::Snapshot,
            },
        )
    });

    assert!(parsed.units.is_empty());
    assert_eq!(parsed.failed.len(), 1);
    assert_eq!(
        parsed.failed[0].diagnostics[0].code.as_str(),
        "lang.parse.adapter_panic"
    );
}

#[test]
fn recognize_panics_are_contained_as_failed_parse_records() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/recognize.fake.json", &bytes)]);

    let parsed = with_silenced_panic_hook(|| {
        parse_snapshot(
            &snapshot,
            registry(vec![FakeAdapter::new(
                "builtin.fake_json",
                "json",
                ".fake.json",
            )
            .with_recognize_panic("src/recognize.fake.json")]),
            ParseRequest {
                languages: language_set(&["json"]),
                scope: ParseScope::Snapshot,
            },
        )
    });

    assert!(parsed.units.is_empty());
    assert_eq!(parsed.failed.len(), 1);
    assert_eq!(
        parsed.failed[0].diagnostics[0].code.as_str(),
        "lang.parse.adapter_panic"
    );
}
