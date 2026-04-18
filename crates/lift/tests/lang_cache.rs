#![allow(unused_crate_dependencies)]

use std::collections::BTreeSet;

use assert_cmd as _;
use gix as _;
use jsonschema as _;
use predicates as _;
use serde::Deserialize;
use serde_jcs as _;
use serde_json::json;
use sha2 as _;
use substrate_lift as _;
use toml as _;
use walkdir as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::*;
}
#[path = "../src/lang/mod.rs"]
mod lang;
#[path = "../src/pack/mod.rs"]
mod pack;
#[path = "../src/repo/mod.rs"]
mod repo;
#[path = "support/repo_support.rs"]
mod repo_support;

use kernel::RepoPath;
use lang::{
    AdapterDescriptor, AdapterName, AdapterParseOutput, AdapterParseResult, CachedParseOutcome,
    InMemoryParseCache, LangError, LangResult, LanguageAdapter, LanguageRegistryBuilder,
    ParseCache, ParseCacheKey, ParseCacheLookup, ParseDriver, ParseInput, ParseRequest, ParseScope,
    PLATFORM_CACHE_VERSION,
};

#[derive(Clone, Debug, Deserialize)]
struct FakeDocument {
    #[serde(default)]
    fail: bool,
    #[serde(default)]
    symbols: Vec<lang::LocalSymbolDraft>,
}

#[derive(Clone, Debug)]
struct FakeAdapter {
    descriptor: AdapterDescriptor,
    suffix: &'static str,
    panic_recognizes: BTreeSet<String>,
}

impl FakeAdapter {
    fn new(name: &str, language: &str, suffix: &'static str, version: &str) -> Self {
        Self {
            descriptor: AdapterDescriptor {
                name: AdapterName::parse(name).expect("adapter name"),
                language: pack::LanguageId::parse(language).expect("language"),
                version: version.to_owned(),
            },
            suffix,
            panic_recognizes: BTreeSet::new(),
        }
    }

    fn with_recognize_panic(mut self, path: &str) -> Self {
        self.panic_recognizes.insert(path.to_owned());
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
                diagnostics: Vec::new(),
            };
        }

        AdapterParseResult::Parsed(AdapterParseOutput {
            symbols: document.symbols,
            edges: Vec::new(),
            surface_markers: Vec::new(),
            diagnostics: Vec::new(),
        })
    }
}

struct FailingCache;

impl ParseCache for FailingCache {
    fn get(&self, _key: &ParseCacheKey) -> LangResult<ParseCacheLookup> {
        Err(LangError::CacheInvariant {
            reason: "simulated cache get failure".to_owned(),
        })
    }

    fn put(&self, _key: ParseCacheKey, _value: CachedParseOutcome) -> LangResult<()> {
        Ok(())
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
    let temp = repo_support::TempDir::new("lang-cache");
    write_repo_files(&temp, files);
    let snapshot = repo_support::materialize(temp.path(), repo_support::default_snapshot_options());
    (temp, snapshot)
}

fn parse_with_driver(
    driver: &ParseDriver,
    snapshot: &repo::RepoSnapshot,
    request: ParseRequest,
) -> lang::ParseSet {
    driver
        .parse_snapshot(snapshot, &request)
        .expect("parse snapshot")
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

fn assert_same_payload_except_stats(left: &lang::ParseSet, right: &lang::ParseSet) {
    assert_eq!(left.snapshot_fingerprint, right.snapshot_fingerprint);
    assert_eq!(left.request, right.request);
    assert_eq!(left.request_fingerprint, right.request_fingerprint);
    assert_eq!(left.units, right.units);
    assert_eq!(left.failed, right.failed);
    assert_eq!(left.skipped, right.skipped);
    assert_eq!(left.missing_languages, right.missing_languages);
    assert_eq!(left.diagnostics, right.diagnostics);
}

#[test]
fn in_memory_cache_hits_on_second_equivalent_run_except_stats() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/app.fake.json", &bytes)]);
    let cache = InMemoryParseCache::default();
    let driver = ParseDriver::with_cache(
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
            "1.0.0",
        )]),
        cache.clone(),
    );
    let request = ParseRequest {
        languages: language_set(&["json"]),
        scope: ParseScope::Snapshot,
    };

    let first = parse_with_driver(&driver, &snapshot, request.clone());
    let second = parse_with_driver(&driver, &snapshot, request);

    assert_same_payload_except_stats(&first, &second);
    assert_eq!(first.stats.cache_hits, 0);
    assert_eq!(first.stats.cache_misses, 1);
    assert_eq!(second.stats.cache_hits, 1);
    assert_eq!(second.stats.cache_misses, 0);
    assert_eq!(cache.len().expect("cache length"), 1);
}

#[test]
fn cache_miss_when_blob_fingerprint_changes() {
    let original = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let updated = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "beta", 0, 4)]
    }))
    .expect("json");
    let (_left_temp, left_snapshot) = snapshot_with_files(&[("src/app.fake.json", &original)]);
    let (_right_temp, right_snapshot) = snapshot_with_files(&[("src/app.fake.json", &updated)]);
    let cache = InMemoryParseCache::default();
    let driver = ParseDriver::with_cache(
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
            "1.0.0",
        )]),
        cache.clone(),
    );
    let request = ParseRequest {
        languages: language_set(&["json"]),
        scope: ParseScope::Snapshot,
    };

    let first = parse_with_driver(&driver, &left_snapshot, request.clone());
    let second = parse_with_driver(&driver, &right_snapshot, request);

    assert_eq!(first.stats.cache_misses, 1);
    assert_eq!(second.stats.cache_hits, 0);
    assert_eq!(second.stats.cache_misses, 1);
    assert_ne!(
        first.units[0].blob_fingerprint,
        second.units[0].blob_fingerprint
    );
    assert_eq!(cache.len().expect("cache length"), 2);
}

#[test]
fn cache_miss_when_adapter_version_changes() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/app.fake.json", &bytes)]);
    let cache = InMemoryParseCache::default();
    let request = ParseRequest {
        languages: language_set(&["json"]),
        scope: ParseScope::Snapshot,
    };

    let first = parse_with_driver(
        &ParseDriver::with_cache(
            registry(vec![FakeAdapter::new(
                "builtin.fake_json",
                "json",
                ".fake.json",
                "1.0.0",
            )]),
            cache.clone(),
        ),
        &snapshot,
        request.clone(),
    );
    let second = parse_with_driver(
        &ParseDriver::with_cache(
            registry(vec![FakeAdapter::new(
                "builtin.fake_json",
                "json",
                ".fake.json",
                "2.0.0",
            )]),
            cache.clone(),
        ),
        &snapshot,
        request,
    );

    assert_eq!(first.stats.cache_misses, 1);
    assert_eq!(second.stats.cache_hits, 0);
    assert_eq!(second.stats.cache_misses, 1);
    assert_eq!(second.units[0].adapter_version, "2.0.0");
    assert_eq!(cache.len().expect("cache length"), 2);
}

#[test]
fn cache_miss_when_platform_cache_version_changes() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/app.fake.json", &bytes)]);
    let request = ParseRequest {
        languages: language_set(&["json"]),
        scope: ParseScope::Snapshot,
    };
    let seed_registry = registry(vec![FakeAdapter::new(
        "builtin.fake_json",
        "json",
        ".fake.json",
        "1.0.0",
    )]);
    let seed = parse_with_driver(&ParseDriver::new(seed_registry), &snapshot, request.clone());

    let cache = InMemoryParseCache::default();
    let entry = snapshot
        .entry(&RepoPath::parse("src/app.fake.json").expect("repo path"))
        .expect("snapshot entry should exist");
    let descriptor = registry(vec![FakeAdapter::new(
        "builtin.fake_json",
        "json",
        ".fake.json",
        "1.0.0",
    )])
    .adapter_for_language(pack::LanguageId::Json)
    .expect("adapter should exist")
    .descriptor();
    let bytes = snapshot
        .read_bytes(&entry.path)
        .expect("snapshot bytes should exist");
    let input = ParseInput {
        path: &entry.path,
        file_id: &entry.file_id,
        blob_fingerprint: &entry.blob_fingerprint,
        bytes,
    };
    cache
        .put(
            ParseCacheKey::with_platform_cache_version(&input, &descriptor, "stale-platform-v0"),
            CachedParseOutcome::Parsed(seed.units[0].clone()),
        )
        .expect("seed cache entry");

    let current = parse_with_driver(
        &ParseDriver::with_cache(
            registry(vec![FakeAdapter::new(
                "builtin.fake_json",
                "json",
                ".fake.json",
                "1.0.0",
            )]),
            cache.clone(),
        ),
        &snapshot,
        request,
    );

    assert_eq!(PLATFORM_CACHE_VERSION, "phase_b_v1");
    assert_eq!(current.stats.cache_hits, 0);
    assert_eq!(current.stats.cache_misses, 1);
    assert_eq!(cache.len().expect("cache length"), 2);
}

#[test]
fn cache_reuses_failed_parse_outcomes() {
    let bytes = serde_json::to_vec(&json!({
        "fail": true
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/app.fake.json", &bytes)]);
    let cache = InMemoryParseCache::default();
    let driver = ParseDriver::with_cache(
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
            "1.0.0",
        )]),
        cache.clone(),
    );
    let request = ParseRequest {
        languages: language_set(&["json"]),
        scope: ParseScope::Snapshot,
    };

    let first = parse_with_driver(&driver, &snapshot, request.clone());
    let second = parse_with_driver(&driver, &snapshot, request);

    assert!(first.units.is_empty());
    assert_eq!(first.failed.len(), 1);
    assert_same_payload_except_stats(&first, &second);
    assert_eq!(first.stats.cache_misses, 1);
    assert_eq!(second.stats.cache_hits, 1);
    assert_eq!(cache.len().expect("cache length"), 1);
}

#[test]
fn cache_does_not_cache_skipped_or_missing_language_records() {
    let matching = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let missing_cache = InMemoryParseCache::default();
    let (_missing_temp, missing_snapshot) =
        snapshot_with_files(&[("src/app.fake.json", &matching)]);
    let missing = parse_with_driver(
        &ParseDriver::with_cache(
            registry(vec![FakeAdapter::new(
                "builtin.fake_json",
                "json",
                ".fake.json",
                "1.0.0",
            )]),
            missing_cache.clone(),
        ),
        &missing_snapshot,
        ParseRequest {
            languages: language_set(&["rust"]),
            scope: ParseScope::Snapshot,
        },
    );

    assert_eq!(missing.stats.cache_hits, 0);
    assert_eq!(missing.stats.cache_misses, 0);
    assert_eq!(missing_cache.len().expect("cache length"), 0);
    assert_eq!(missing.missing_languages.len(), 1);

    let skip_cache = InMemoryParseCache::default();
    let (_skip_temp, skip_snapshot) = snapshot_with_files(&[
        ("src/plain.txt", b"plain text"),
        ("src/app.fake.json", &matching),
    ]);
    let skipped = parse_with_driver(
        &ParseDriver::with_cache(
            registry(vec![FakeAdapter::new(
                "builtin.fake_json",
                "json",
                ".fake.json",
                "1.0.0",
            )]),
            skip_cache.clone(),
        ),
        &skip_snapshot,
        ParseRequest {
            languages: language_set(&["json"]),
            scope: ParseScope::Paths(path_set(&["src/plain.txt"])),
        },
    );

    assert_eq!(skipped.stats.cache_hits, 0);
    assert_eq!(skipped.stats.cache_misses, 0);
    assert_eq!(skip_cache.len().expect("cache length"), 0);
    assert_eq!(skipped.skipped.len(), 1);
    assert_eq!(
        skipped.skipped[0].reason,
        lang::SkippedReason::NoMatchingAdapter
    );
}

#[test]
fn cache_backend_failures_surface_as_cache_invariant_errors() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/app.fake.json", &bytes)]);
    let driver = ParseDriver::with_cache(
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
            "1.0.0",
        )]),
        FailingCache,
    );

    let error = driver
        .parse_snapshot(
            &snapshot,
            &ParseRequest {
                languages: language_set(&["json"]),
                scope: ParseScope::Snapshot,
            },
        )
        .expect_err("cache get failure should surface");

    assert_eq!(
        error,
        LangError::CacheInvariant {
            reason: "simulated cache get failure".to_owned(),
        }
    );
}

#[test]
fn recognize_panic_behavior_remains_deterministic_and_uncached() {
    let bytes = serde_json::to_vec(&json!({
        "symbols": [fake_symbol("main", "alpha", 0, 5)]
    }))
    .expect("json");
    let (_temp, snapshot) = snapshot_with_files(&[("src/app.fake.json", &bytes)]);
    let cache = InMemoryParseCache::default();
    let driver = ParseDriver::with_cache(
        registry(vec![FakeAdapter::new(
            "builtin.fake_json",
            "json",
            ".fake.json",
            "1.0.0",
        )
        .with_recognize_panic("src/app.fake.json")]),
        cache.clone(),
    );
    let request = ParseRequest {
        languages: language_set(&["json"]),
        scope: ParseScope::Snapshot,
    };

    let first = parse_with_driver(&driver, &snapshot, request.clone());
    let second = parse_with_driver(&driver, &snapshot, request);

    assert_eq!(first, second);
    assert_eq!(first.stats.cache_hits, 0);
    assert_eq!(first.stats.cache_misses, 0);
    assert_eq!(second.stats.cache_hits, 0);
    assert_eq!(second.stats.cache_misses, 0);
    assert_eq!(cache.len().expect("cache length"), 0);
}
