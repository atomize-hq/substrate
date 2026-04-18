<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-lift-autoplan-restore-20260418-101411.md -->
# substrate-lift seam 3 spec — language platform (reviewed against landed seam 0 + seam 1 + seam 2 + landed seam 3 Phase A)

## 0. Ground truth from the landed crate

This spec is intentionally anchored to the crate as it exists today, not to the earlier idealized seam sketch.

Observed state in the landed crate:

- `src/kernel/**` is real, tested, schema-backed, and publicly re-exported from `lib.rs`.
- `src/pack/**` is real code and now compiles profiles, topology packs, score models, query packs, rule packs, and recipe packs into crate-private compiled artifacts.
- `src/repo/**` is real, tested, schema-backed, and crate-private. `RepoSnapshot`, `BlobStore`, `Inventory`, `RepoDiff`, `SnapshotRequest`, and repo diagnostics are already landed.
- `src/lang/**` is now real, tested, schema-backed, and still crate-private:
  - `src/lang/mod.rs` exports `adapter`, `driver`, `error`, `model`, `registry`, and `schema`;
  - `lang::LanguageId` is the expected re-export of `pack::LanguageId`;
  - `ParseDriver::parse_snapshot(&self, ...)` already normalizes requests, computes `request_fingerprint`,
    contains adapter panics, validates draft output, and emits deterministic `ParseSet` output;
  - `ParseStats` currently tracks parse outcomes only. It has **no** cache counters yet;
  - `lib.rs` still exposes `lang` only as `pub(crate) mod lang;`, so seam 3 is still not public API.
- `Cargo.toml` already contains feature flags for concrete language families:
  - `config-lang`
  - `rust-lang`
  - `python-lang`
  - `javascript-lang`
  - `typescript-lang`
- `Cargo.toml` does **not** yet include any parser runtime crates such as tree-sitter language grammars, syn, serde_yaml-specific AST layers, etc.
- seam 1 already owns a crate-private `pack::LanguageId` enum with canonical values:
  - `json`
  - `toml`
  - `yaml`
  - `rust`
  - `python`
  - `javascript`
  - `typescript`
- `CompiledAnalysisDefaults.languages`, `CompiledQueryPack.language`, and `CompiledRuleScope.languages` already use that `pack::LanguageId` type.
- seam 1 also already defines `QueryEngineKind::TreeSitter` in compiled query packs, but there is **no** query execution seam yet and no language runtime contract for query execution yet.
- `src/app/runtime.rs` currently stops at `ProfileBootstrap { bundle: CompiledPackSet }`; there is no lang-facing runtime orchestration yet.
- the compile-matrix test still asserts the crate builds with `--no-default-features`.
- the top-level `README.md` has already been updated to describe Phase A as landed.
- the real proof slice now exists in tests:
  - `tests/lang_consumer.rs`
  - `tests/support/lang_support.rs`
  - a bounded TOML adapter proof that demonstrates deterministic parse output and a consumer path
    deriving config-key inventory from `ParseSet` without rereading the filesystem.
- there is still **no** `src/lang/cache.rs`, no parse-cache contract, no in-memory cache implementation,
  and no built-in registry-construction helper for runtime bootstrapping.

That changes the planning posture.

Seam 3 is no longer the first landing question. Phase A is already in the crate. The remaining job
for this document is to lock the next increment cleanly:

> Phase B should add cache and runtime-readiness surfaces around the landed Phase-A contracts,
> without reopening the Phase-A boundary or smuggling concrete production adapters into seam 3.

The seam-1 `LanguageId` consequence still holds:

> seam 3 should **continue** to avoid a second competing `LanguageId` type in v1.

The crate already has one internal language identifier contract in `pack`, and back-migrating seam 1
just to move that enum is unnecessary churn. Seam 3 should keep consuming and re-exporting that
existing internal type under the `lang` module boundary.

---

## 1. Mission

Seam 3 owns the **language platform**.

It is responsible for:

- defining the adapter trait for language-specific parsers;
- defining deterministic adapter registration and lookup;
- defining the parse request and parse result contracts;
- defining the normalized parsed-unit contract used by later seams;
- defining normalized local symbol and local edge contracts;
- defining parse-time surface-marker contracts;
- validating adapter output into a canonical, deterministic representation;
- defining parse-cache keys and the cache trait;
- turning a `RepoSnapshot` into a deterministic `ParseSet` without rereading the live filesystem;
- surfacing typed language-platform errors and deterministic parse diagnostics;
- embedding fixture schemas for parse manifests.

It is **not** responsible for:

- concrete production parsing for Rust / Python / JSON / etc.;
- repo root detection or file walking;
- topology classification;
- repo-wide graph resolution;
- query execution;
- detector/fact execution;
- Lift score math;
- pack compilation;
- CLI rendering.

A useful rule:

> seam 3 ends at **validated, normalized per-file parse output**.
> It does not resolve repo-wide edges or emit app findings.

---

## 2. Boundary with existing code

### Existing seam-0 primitives seam 3 should reuse directly

Use directly from `kernel`:

- `RepoPath`
- `FileId`
- `SymbolId`
- `Fingerprint`
- `ByteSpan`
- `Locator`
- `Diagnostic`
- `DiagnosticCode`
- `Severity`
- `sha256_canonical_json`
- `sha256_bytes`

Unlike seam 1 and seam 2, seam 3 **can** use kernel `Locator` and kernel `Diagnostic` directly, because parse work is always anchored to repo-relative files and optional byte spans.

### Existing seam-2 contracts seam 3 should reuse directly

Use directly from `repo`:

- `RepoSnapshot`
- `Inventory`
- `InventoryEntry`
- `BlobStore`
- `BlobRecord`
- `SnapshotStats`

Seam 3 must consume bytes **only** from `RepoSnapshot` / `BlobStore`.
It must not reopen files from disk.

### Existing seam-1 contracts seam 3 should reuse carefully

Seam 3 should **not** define a duplicate language-id enum.

Instead, Phase A should do this:

```rust
// src/lang/mod.rs
pub(crate) use crate::pack::LanguageId;
```

That yields these rules:

- downstream seams should import `lang::LanguageId`, not `pack::LanguageId` directly;
- seam 3 may internally depend on `pack::LanguageId` in Phase A;
- seam 1 must not gain a dependency on `lang`;
- a later extraction of `LanguageId` into `kernel` or `lang` can happen behind the `lang::LanguageId` re-export without churning all downstream seams.

### Dependency direction

Seam 3 should depend only on:

- `kernel`
- `repo`
- `pack::LanguageId` (and only that narrow type alias surface)
- `std`
- `serde` for fixture/manifest contracts
- `jsonschema` only in tests / schema validation helpers

Seam 3 should **not** depend on:

- `pack::compiler`
- `pack::compiled::*` other than the re-exported `LanguageId`
- `topo`
- `graph`
- `facts`
- `derive`
- `query`
- `patch`
- `app`
- `cli`
- `anyhow`
- parser runtime crates in Phase A

---

## 3. Canonical phase map

Because the current crate only has a placeholder `src/lang/mod.rs`, seam 3 should land in three explicit phases.

This section is canonical. Later sections should reference these phases rather than inventing a competing rollout plan.

### Phase A — platform foundation only

Phase A lands:

- `src/lang/**` real modules instead of the placeholder, but in a reduced first slice;
- adapter name and adapter descriptor contracts;
- the `LanguageAdapter` trait;
- the `LanguageRegistry` and deterministic registration rules;
- the `ParseRequest`, `ParseSet`, `ParsedUnit`, `FailedParse`, and `SkippedParse` contracts;
- normalized local symbol, local edge, and surface-marker contracts folded into `model.rs`;
- deterministic output validation, canonical ordering, and platform-owned symbol ID assignment;
- the parse-driver boundary from `RepoSnapshot` to `ParseSet`;
- explicit Phase-A handling for requested languages with no registered adapter;
- an embedded parse-manifest schema for fixtures;
- test-only fake adapters proving extensibility;
- one bounded real-adapter spike or one narrow real adapter plus one real consuming workflow,
  specifically to stress the shared contracts before Phase A is considered complete;
- targeted unit/integration tests.

Phase A does **not** land:

- the full production adapter family for Rust / Python / JavaScript / TypeScript / config;
- tree-sitter integration;
- any cache contract or cache stats surface;
- on-disk persistent caching;
- query-engine execution hooks;
- public API promotion;
- adapter auto-registration from feature flags.

### Phase B — cache and runtime-readiness

Phase B lands:

- the cache contract and `cache.rs`;
- a no-op cache implementation plus an in-memory cache implementation;
- cache hit/miss accounting in `ParseStats`;
- adapter-version-aware cache invalidation;
- a small registry-construction helper for built-in adapters.

Phase B still does **not** land:

- production parsing implementations;
- persistent cache files;
- query execution;
- graph resolution.

### Phase C — platform integration hooks for later seams

Phase C lands:

- any additional adapter metadata needed by seam 4 and seam 7;
- optional adapter capability descriptors if required by real adapters;
- optional query-engine compatibility hooks if the concrete adapters now require them.

Phase C must still keep the rule that:

> concrete parsers belong to seam 4, not seam 3.

---

## 4. Exact module shape by phase

### Phase A `src/lang/`

```text
src/lang/
  mod.rs
  error.rs
  adapter.rs
  registry.rs
  driver.rs
  model.rs
  schema.rs
```

No production adapter modules live here in seam 3.
Those belong in seam 4.

For the reduced Phase-A landing, keep the data model boring:

- `LocalSymbolDraft`, `LocalSymbol`, `LocalEdgeDraft`, `LocalEdge`, `SurfaceMarkerDraft`, and
  `SurfaceMarker` live in `model.rs` instead of being split into extra files;
- only split those types back out later if seam 4 proves the file is actually too dense.

### Phase A `src/lang/mod.rs`

`src/lang/mod.rs` should re-export only the stable internal seam surface:

```rust
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
    EdgeEndpoint, EdgeEndpointDraft, FailedParse, LocalEdge, LocalEdgeDraft, LocalEdgeKind,
    LocalSymbol, LocalSymbolDraft, MissingRequestedLanguage, ParseRequest, ParseScope, ParseSet,
    ParseStats, ParsedUnit, ReferenceTarget, ReferenceTargetDraft, SkippedParse, SkippedReason,
    SurfaceMarker, SurfaceMarkerDraft, SurfaceMarkerKind, SymbolKind, SymbolVisibility,
};
pub(crate) use registry::{LanguageRegistry, LanguageRegistryBuilder};
pub(crate) use schema::{
    LANG_PARSE_MANIFEST_V1_SCHEMA_FILE, LANG_PARSE_MANIFEST_V1_SCHEMA_ID,
    LANG_PARSE_MANIFEST_V1_SCHEMA_JSON, LANG_PARSE_MANIFEST_V1_SCHEMA_VERSION,
};
```

### Phase B `src/lang/`

Phase B should add:

```text
src/lang/
  cache.rs
```

Do **not** split `cache.rs` yet unless Phase-B implementation proves it is needed.

The current Phase-A seam is still small enough that the explicit choice is:

- keep `NoopParseCache` and `InMemoryParseCache` together in `cache.rs`;
- keep the built-in registry helper in `registry.rs`;
- keep `ParseDriver` responsible for cache lookup/store orchestration only, not cache internals.

### Phase C `src/lang/`

Phase C may add:

```text
src/lang/
  capabilities.rs
```

Only add this if real adapters now need explicit capability descriptors.
Do not pre-land it in Phase A.

---

## 5. Exact Rust contract shape

### 5.1 `error.rs`

```rust
pub(crate) type LangResult<T> = Result<T, LangError>;

#[derive(Debug, thiserror::Error, Clone, Eq, PartialEq)]
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
```

Rule:
- seam 3 exposes **typed** internal errors only;
- file-level parse failures and syntax failures are **not** `LangError`.
  They must be represented as `FailedParse` records with diagnostics.

### 5.2 `adapter.rs`

```rust
use crate::kernel::{ByteSpan, Diagnostic, FileId, Fingerprint, RepoPath};
use crate::lang::{LanguageId, LocalEdgeDraft, LocalSymbolDraft, SurfaceMarkerDraft};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct AdapterName(String);
// Canonical form:
// - lowercase dot-separated segments
// - first char of each segment alphabetic
// - later chars alnum or underscore
// Example: "builtin.fake_config"

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
```

Rules:

- `recognizes()` must be pure and deterministic.
- in Phase A, the driver selects the single registered adapter for a requested `LanguageId` first,
  then uses `recognizes()` only as a file-level inclusion filter inside that language, not as a
  second routing system.
- `parse()` must be pure and deterministic.
- neither method may perform filesystem I/O, env access, wall-clock reads, or random generation.
- `parse()` may emit diagnostics but must not panic on bad input.

### 5.3 `model.rs`

```rust
use crate::kernel::{ByteSpan, SymbolId};

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
```

Rules:

- `local_key` must be unique within a single parsed file.
- `path` must be deterministic and already normalized by the adapter.
- `path` may be empty for anonymous symbols, but `local_key` may not be empty.
- final `LocalSymbol` must not retain `local_key`.

```rust
use crate::kernel::{ByteSpan, RepoPath, SymbolId};

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
    LocalSymbol { local_key: String },
    QualifiedName { parts: Vec<String> },
    FilePath { path: RepoPath },
    ExternalPackage { package: String, symbol: Option<String> },
    Opaque { value: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum ReferenceTarget {
    LocalSymbol(SymbolId),
    QualifiedName { parts: Vec<String> },
    FilePath { path: RepoPath },
    ExternalPackage { package: String, symbol: Option<String> },
    Opaque { value: String },
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
```

Rules:

- draft edge references must resolve against the draft symbol table for the file;
- unresolved draft local keys are a file-level adapter-output failure, not a global platform crash;
- cross-file or external targets remain unresolved here by design.

```rust
use crate::kernel::{ByteSpan, SymbolId};

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
```

Rule:
- a draft marker may reference either a symbol or a span or both;
- if `symbol_local_key` is present, it must resolve.

```rust
use std::collections::BTreeSet;

use crate::kernel::{Diagnostic, FileId, Fingerprint, RepoPath};
use crate::lang::{AdapterName, LanguageId, LocalEdge, LocalSymbol, SurfaceMarker};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ParseScope {
    Snapshot,
    Paths(BTreeSet<RepoPath>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParseRequest {
    pub languages: BTreeSet<LanguageId>,
    pub scope: ParseScope,
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
```

Rules:

- `languages` in `ParseRequest` is sorted, unique, and empty means “all registered adapters”.
- `ParseScope::Paths` must be sorted and unique.
- `ParseScope::Paths(BTreeSet::new())` means a deterministic empty parse run with zero
  `considered_files`, zero units, zero failures, zero skips, and no implicit fallback to
  `ParseScope::Snapshot`.
- `ParseSet.units`, `failed`, and `skipped` must be sorted deterministically before serialization.
- `ParseSet.missing_languages` must be sorted by `language`.
- `ParseSet.diagnostics` is only for run-level diagnostics, for example malformed requests or
  missing requested-language coverage; file-level diagnostics live only on `ParsedUnit` or
  `FailedParse` and are not duplicated upward.
- `SkippedParse { reason: NoMatchingAdapter }` is materialized only for explicitly requested
  paths. Snapshot-wide files that do not match any selected adapter are counted in
  `ParseStats.skipped_no_adapter`, but are not emitted one-by-one into `ParseSet.skipped`.
- `request_fingerprint` must be computed from canonical JSON of `ParseRequest`.
- `unit_fingerprint` must be computed from canonical JSON of the final normalized `ParsedUnit` excluding the `unit_fingerprint` field itself.

### 5.4 `registry.rs`

```rust
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::lang::{AdapterDescriptor, AdapterName, LangError, LangResult, LanguageAdapter, LanguageId};

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
    pub(crate) fn new() -> Self;
    pub(crate) fn register<A: LanguageAdapter + 'static>(mut self, adapter: A) -> LangResult<Self>;
    pub(crate) fn build(self) -> LangResult<LanguageRegistry>;
}

impl LanguageRegistry {
    pub(crate) fn descriptors(&self) -> Vec<AdapterDescriptor>;
    pub(crate) fn adapter_for_language(&self, language: LanguageId) -> Option<&Arc<dyn LanguageAdapter>>;
}
```

Rules:

- Phase A allows **at most one registered adapter per `LanguageId`**.
- duplicate adapter names are rejected.
- duplicate language registration is rejected.
- registry iteration order must be deterministic and independent of registration order.

### 5.5 `driver.rs`

```rust
use crate::lang::{LangResult, LanguageRegistry, ParseRequest, ParseSet};
use crate::repo::RepoSnapshot;

pub(crate) struct ParseDriver {
    registry: LanguageRegistry,
}

impl ParseDriver {
    pub(crate) fn new(registry: LanguageRegistry) -> Self;
    pub(crate) fn with_cache<C: ParseCache + 'static>(registry: LanguageRegistry, cache: C) -> Self;
    pub(crate) fn parse_snapshot(
        &self,
        snapshot: &RepoSnapshot,
        request: &ParseRequest,
    ) -> LangResult<ParseSet>;
}
```

Rules:

- `ParseDriver` is the only seam-3 entrypoint that may iterate the snapshot.
- `ParseDriver::parse_snapshot` must walk the selected snapshot inventory only once per request,
  selecting the candidate adapter for each file during that pass and normalizing outputs
  afterward, rather than rescanning the full inventory per language.
- it must never reread the live filesystem.
- it must never resolve repo-wide graph edges.
- it must treat file-level parse failures as data, not as a seam-wide error.
- it must contain adapter panics and convert them into deterministic `FailedParse` records with a
  top-level file diagnostic, rather than allowing one hostile file to abort the whole run.
- it must keep diagnostic ownership non-overlapping: file-level diagnostics stay on units or failed
  records, while run-level diagnostics stay on `ParseSet`.
- if `ParseRequest.languages` names a language with no registered adapter, the parse run must emit a
  deterministic top-level diagnostic and a `MissingRequestedLanguage` record instead of silently
  ignoring the request.
- Phase A must define and test hostile-input containment rules before the first real adapter spike
  is accepted: panic containment, malformed-byte handling, and explicit output-budget behavior for
  symbol/edge/marker/diagnostic counts per file.

### 5.6 `cache.rs`

Phase B should cache **normalized per-file outcomes**, not raw adapter drafts and not whole-run
`ParseSet` blobs.

Exact contract shape:

```rust
use crate::kernel::{FileId, Fingerprint};
use crate::lang::{AdapterName, FailedParse, LanguageId, ParsedUnit};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub(crate) struct ParseCacheKey {
    pub file_id: FileId,
    pub blob_fingerprint: Fingerprint,
    pub language: LanguageId,
    pub adapter: AdapterName,
    pub adapter_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum CachedParseOutcome {
    Parsed(ParsedUnit),
    Failed(FailedParse),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CacheLookup {
    Disabled,
    Hit(CachedParseOutcome),
    Miss,
}

pub(crate) trait ParseCache: Send + Sync {
    fn get(&self, key: &ParseCacheKey) -> LangResult<CacheLookup>;
    fn put(&self, key: ParseCacheKey, value: CachedParseOutcome) -> LangResult<()>;
}

#[derive(Clone, Debug, Default)]
pub(crate) struct NoopParseCache;

#[derive(Clone, Debug, Default)]
pub(crate) struct InMemoryParseCache {
    // exact storage type intentionally not locked here, but it must be deterministic.
}
```

Rules:

- `ParseCacheKey` must use `file_id` and `blob_fingerprint` together.
  `file_id` is already path-sensitive in the repo seam, so Phase B does **not** need a duplicate
  raw path string in the cache key.
- the key must also include `language`, `adapter`, and `adapter_version`, so adapter upgrades or
  routing changes cannot return stale results.
- cache values must store the **final normalized outcome**:
  - `ParsedUnit` on success;
  - `FailedParse` on deterministic file-level parse failure.
- cache values must **not** store:
  - `SkippedParse` records;
  - `MissingRequestedLanguage` records;
  - whole-run `ParseSet` values.
- `NoopParseCache` must return `CacheLookup::Disabled` from `get()` and ignore `put()`.
- `InMemoryParseCache` must be deterministic for equivalent insertion sequences. A `BTreeMap` keyed
  by `ParseCacheKey` is preferred over `HashMap`.
- cache failures remain seam-level `LangError::CacheInvariant`, because a broken cache is a platform
  failure, not a file-level parse result.

### 5.7 Phase-B `ParseDriver` cache behavior

The driver should stay `&self` in Phase B.

That is still the cleanest contract because:

- callers do not need to learn a new mutability story just to opt into caching;
- `NoopParseCache` can remain the default behind `ParseDriver::new`;
- real cache implementations can use interior mutability without forcing the seam API to widen.

Exact behavior on each candidate file:

1. select the adapter exactly as Phase A already does;
2. build `ParseCacheKey` from the chosen adapter descriptor and the inventory entry;
3. query the cache;
4. on `Hit`, append the cached normalized outcome directly into the `ParseSet`;
5. on `Miss`, invoke the adapter, normalize the outcome, append it to `ParseSet`, then store it;
6. on `Disabled`, behave exactly like Phase A.

Cache lookup happens **after** adapter selection.

That means Phase B does **not** cache:

- `recognizes()` routing decisions;
- snapshot-wide “no matching adapter” counts;
- missing-requested-language records.

Those remain request-level or registry-level concerns.

### 5.8 `ParseStats` additions in Phase B

Phase B should extend `ParseStats` with cache accounting:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ParseStats {
    pub considered_files: u64,
    pub parsed_units: u64,
    pub failed_units: u64,
    pub skipped_no_adapter: u64,
    pub skipped_missing_paths: u64,
    pub missing_requested_languages: u64,
    pub diagnostic_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}
```

Rules:

- `cache_hits` increments only when `CacheLookup::Hit(_)` is returned.
- `cache_misses` increments only when `CacheLookup::Miss` is returned.
- `NoopParseCache` must leave both counters at zero by returning `Disabled`, not synthetic misses.
- for an enabled cache, `cache_hits + cache_misses` must equal the number of candidate files that
  actually reached parse-stage execution, which is `parsed_units + failed_units` for that run.

### 5.9 Built-in registry helper

Phase B should add one tiny registry-construction helper in `registry.rs`:

```rust
pub(crate) fn built_in_registry() -> LangResult<LanguageRegistry>;
```

Rules:

- this helper is the **only** place that should know about feature-gated built-in adapter
  registration order;
- it is allowed to return an empty registry today, because seam 4 still owns production adapter
  landings;
- when seam 4 starts adding feature-gated built-ins, they must be registered here in deterministic
  adapter-name order;
- `app/runtime.rs` and later seams should depend on this helper, not on concrete adapter types.

This gives the runtime one stable construction seam without forcing seam 3 to land production
adapters early.

---

## 6. Determinism and identity rules

These are non-negotiable.

### 6.1 LanguageId ownership rule

In Phase A:

- `lang::LanguageId` is a re-export of `pack::LanguageId`;
- seam 3 must not define a duplicate enum with the same values;
- downstream seams should depend on `lang::LanguageId` spelling only.

### 6.2 Adapter identity

`AdapterName` canonical string pattern:

```text
^[a-z][a-z0-9]*(\.[a-z][a-z0-9_]*)+$
```

Examples:

```text
builtin.fake_config
builtin.rust
builtin.typescript
```

Examples that must fail:

```text
Builtin.rust
builtin/
builtin-rust
builtin..rust
```

### 6.3 Parse request fingerprint

`ParseRequest` fingerprint must be:

```text
sha256_canonical_json(ParseRequest)
```

after request normalization:

- sorted unique languages;
- sorted unique paths when scope is `Paths`.

### 6.4 Symbol identity

Adapters return `LocalSymbolDraft { local_key, ... }`, but `local_key` is **not** the final `SymbolId`.

The platform computes final `SymbolId`s after canonical sorting using this identity lemma:

```text
lang\0symbol\0v1\0<language>\0<repo_path>\0<kind>\0<joined-symbol-path>\0<duplicate-ordinal>
```

Where:

- `joined-symbol-path` is the symbol `path` joined with `\0`;
- `duplicate-ordinal` is the zero-based ordinal among otherwise identical symbols after deterministic sorting by:
  1. `kind`
  2. `path`
  3. `name`
  4. `span.start_byte`
  5. `span.end_byte`
  6. `local_key`

Rule:
- `local_key` exists only so the platform can resolve draft-local references;
- final `SymbolId` generation remains platform-owned and deterministic.

### 6.5 Unit fingerprint

`unit_fingerprint` must be:

```text
sha256_canonical_json(ParsedUnit_without_unit_fingerprint)
```

after canonical sorting of:

- `symbols`
- `edges`
- `surface_markers`
- `diagnostics`

### 6.6 Parse ordering

`ParseSet` ordering must be deterministic:

- `units` sorted by `path`, then `adapter`, then `file_id`
- `failed` sorted by `path`, then `adapter`, then `file_id`
- `skipped` sorted by `path`, then `reason`
- `missing_languages` sorted by `language`
- `diagnostics` sorted by kernel diagnostic ordering

### 6.7 Requested language handling

If `ParseRequest.languages` contains a language with no registered adapter:

- seam 3 must not silently ignore it;
- the parse set must record a `MissingRequestedLanguage`;
- a deterministic top-level diagnostic must be emitted with code
  `lang.parse.missing_registered_adapter`.

### 6.8 Parse failure semantics

If an adapter cannot parse a file or emits invalid draft output:

- the file becomes a `FailedParse` record;
- diagnostics are preserved;
- the rest of the parse set continues;
- seam 3 does not return `LangError` for that file-level failure.

---

## 7. Schema inventory

Seam 3 should add one fixture-facing schema file in Phase A:

```text
schemas/lang/parse_manifest.v1.json
```

Seam 3 should add one schema constants module:

```text
src/lang/schema.rs
```

### `src/lang/schema.rs`

```rust
#![allow(dead_code)]

pub(crate) const LANG_PARSE_MANIFEST_V1_SCHEMA_ID: &str =
    "https://schemas.substrate.dev/lift/lang/parse_manifest.v1.json";

pub(crate) const LANG_PARSE_MANIFEST_V1_SCHEMA_VERSION: u32 = 1;

pub(crate) const LANG_PARSE_MANIFEST_V1_SCHEMA_FILE: &str =
    "parse_manifest.v1.json";

pub(crate) const LANG_PARSE_MANIFEST_V1_SCHEMA_JSON: &str =
    include_str!("../../schemas/lang/parse_manifest.v1.json");
```

### `schemas/lang/parse_manifest.v1.json`

This schema is for fixtures and manifest validation, not for a public API.

Top-level exact shape:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.substrate.dev/lift/lang/parse_manifest.v1.json",
  "title": "Lift Language Parse Manifest v1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "version",
    "case",
    "snapshot_fingerprint",
    "request_fingerprint",
    "request",
    "units",
    "failed",
    "skipped",
    "missing_languages",
    "diagnostics",
    "stats"
  ],
  "properties": {
    "version": { "const": 1 },
    "case": { "type": "string", "minLength": 1 },
    "snapshot_fingerprint": {
      "$ref": "../kernel/primitives.v1.json#/$defs/fingerprint"
    },
    "request_fingerprint": {
      "$ref": "../kernel/primitives.v1.json#/$defs/fingerprint"
    },
    "request": { "$ref": "#/$defs/request" },
    "units": {
      "type": "array",
      "items": { "$ref": "#/$defs/parsed_unit" }
    },
    "failed": {
      "type": "array",
      "items": { "$ref": "#/$defs/failed_parse" }
    },
    "skipped": {
      "type": "array",
      "items": { "$ref": "#/$defs/skipped_parse" }
    },
    "missing_languages": {
      "type": "array",
      "items": { "$ref": "#/$defs/missing_requested_language" }
    },
    "diagnostics": {
      "type": "array",
      "items": { "$ref": "../kernel/primitives.v1.json#/$defs/diagnostic" }
    },
    "stats": { "$ref": "#/$defs/stats" }
  }
}
```

Required defs to include:

- `language_id`
- `adapter_name`
- `symbol_kind`
- `symbol_visibility`
- `local_symbol`
- `edge_endpoint`
- `reference_target`
- `local_edge_kind`
- `local_edge`
- `surface_marker_kind`
- `surface_marker`
- `parsed_unit`
- `failed_parse`
- `skipped_reason`
- `skipped_parse`
- `missing_requested_language`
- `request`
- `scope`
- `stats`

### Runtime-only invariants not fully expressible in schema

These must be enforced in Rust validators:

- all `ByteSpan`s must satisfy `end_byte >= start_byte` and `end_byte <= bytes.len()` when attached to a parsed file;
- `local_key` values must be unique within one file;
- draft references must resolve to existing local keys;
- `units`, `failed`, and `skipped` must already be sorted deterministically before serialization;
- `missing_languages` must already be sorted deterministically before serialization;
- `unit_fingerprint` must match the final normalized unit;
- `request_fingerprint` must match the normalized request;
- `AdapterName` must be valid and canonical;
- no final `LocalSymbol` may retain a draft-only `local_key`.

---

## 8. Acceptance criteria

Seam 3 is done only when all of these are true.

### Contract completeness

- `src/lang/mod.rs` is real and no longer a placeholder.
- every public serializable seam-3 contract used in fixtures has a matching schema entry in `schemas/lang/parse_manifest.v1.json`.
- every seam-3 type used by later seams is re-exported from `src/lang/mod.rs`.
- there is no duplicate `LanguageId` type introduced in `lang`.
- the accepted CEO gate is reflected in the actual seam definition of done: Phase A is not complete
  until one bounded real-adapter spike or one narrow real adapter plus one real consuming workflow
  has exercised the shared contracts.

### Determinism

- registry iteration order is deterministic and independent of registration order.
- the same `RepoSnapshot` + same `ParseRequest` + same registered adapters yields byte-identical parse manifests.
- `request_fingerprint` is stable for the same normalized request.
- `unit_fingerprint` is stable for the same normalized parse output.
- no public seam-3 serialization relies on `HashMap` iteration order.

### Failure semantics

- one file’s parse failure does not abort parsing the rest of the snapshot.
- invalid adapter output becomes a deterministic `FailedParse` with diagnostics, not a panic.
- adapter panics become deterministic `FailedParse` records with diagnostics, not process-wide
  aborts.
- requested languages with no registered adapter become deterministic top-level records and
  diagnostics, not silent no-ops.
- requested paths missing from the snapshot become deterministic `SkippedParse { reason: PathNotInSnapshot }` records.
- empty explicit path scopes return a deterministic empty `ParseSet`, not a whole-snapshot parse.
- snapshot-wide non-matching files are counted, but not exploded into one `SkippedParse` record per
  file unless the caller explicitly requested those paths.

### Boundary cleanliness

- seam 3 remains `pub(crate)` only.
- seam 3 has zero dependencies on:
  - `graph`
  - `topo`
  - `facts`
  - `derive`
  - `query`
  - `patch`
  - `app`
  - `cli`
  - parser runtime crates in Phase A
- seam 3 reads bytes only from `RepoSnapshot` / `BlobStore`.
- seam 3 performs no filesystem I/O, env access, time access, or randomness.

### Extensibility proof

- there is at least one test-only fake adapter proving that a new adapter can be implemented without changing app code.
- there is at least one test proving duplicate adapter name rejection.
- there is at least one test proving duplicate language registration rejection.
- there is at least one test proving registration order does not affect output.
- there is at least one bounded real-adapter spike or one narrow real adapter plus one real
  consumer-path test proving the contracts survive contact with reality.

### Schema parity

- manifest fixtures validate against `schemas/lang/parse_manifest.v1.json`.
- schema constants in `src/lang/schema.rs` match the actual file on disk.
- parity tests fail if a serialized field changes without a schema update.

### Feature-flag compatibility

- `cargo check -p substrate-lift --no-default-features` continues to work after seam 3 lands.
- seam 3 does not require any concrete adapter feature to compile.

### Value proof

- one representative real repo is parsed through the bounded spike with stable output across repeat
  runs on the same snapshot.
- one downstream consumer proves that the seam does not end at an orphan internal artifact.
- Phase A success metrics include at minimum: parsed files, failed files, skipped-no-adapter count,
  missing-language count, and stable output diff behavior across repeat runs.

---

## 9. Invariants

These should go into the seam doc as a must-not-break section.

1. `lang` remains crate-private in seam 3.
2. `lang::LanguageId` is a re-export, not a duplicate enum.
3. seam 3 consumes only immutable `RepoSnapshot` data.
4. seam 3 never rereads the live filesystem.
5. adapter registration order never changes parse results.
6. there is at most one registered adapter per `LanguageId` in Phase A.
7. adapter-local `local_key` values never leak into final serialized units.
8. final `SymbolId`s are platform-derived, not adapter-chosen.
9. file-level parse failures are data, not seam-wide errors.
10. requested languages with no registered adapter are surfaced explicitly, never dropped silently.
11. final parse outputs are sorted and deterministic.
12. seam 3 does not own graph resolution, topology, rule evaluation, score math, or query execution.
13. seam 3 does not pull parser-specific concrete types into shared contracts.
14. seam 3 does not create a second file-path or symbol-id contract outside `kernel`.

---

## 10. Falsification questions

These are the review questions that can invalidate the seam.

1. Can seam 3 reopen a file from disk after receiving a `RepoSnapshot`?
2. Can registration order change which adapter wins for a file?
3. Can two adapters for the same `LanguageId` be silently registered at once?
4. Can a requested language with no registered adapter disappear silently instead of being reported explicitly?
5. Can a single syntax error abort the whole parse run?
6. Can an adapter choose final `SymbolId`s directly?
7. Can `local_key` leak into a final `ParsedUnit`?
8. Can `unit_fingerprint` change because collection insertion order changed?
9. Can `ParseRequest` with the same logical languages/paths produce different `request_fingerprint`s because inputs were unsorted?
10. Can seam 3 introduce a second competing `LanguageId` enum?
11. Can seam 3 take a dependency on `pack::compiler`, `graph`, `query`, or app code?
12. Can file-level adapter output contract violations cause a panic instead of a deterministic failed record?
13. Can a span exceed the underlying blob length without being rejected or turned into a failed parse?
14. Can a requested path that is not in the snapshot disappear silently instead of being represented explicitly?
15. Can seam 3 require a real parser runtime crate before any concrete adapters exist?

If any answer is yes, seam 3 is not clean enough.

---

## 11. Risks and mitigations

### Risk 1: `LanguageId` ownership churn

The crate already has `pack::LanguageId`.
Creating a second enum in `lang` would create avoidable drift.

Mitigation:
- Phase A re-exports `pack::LanguageId` from `lang`.
- downstream code uses `lang::LanguageId` spelling.

### Risk 2: overdesigning around tree-sitter before adapters exist

The crate has `QueryEngineKind::TreeSitter` in seam 1, but there is no query execution seam yet.
If seam 3 tries to encode parser-engine specifics too early, it will likely overfit.

Mitigation:
- Phase A keeps parser-engine details out of the shared platform contracts.
- Phase C adds capability metadata only when real adapters need it.

### Risk 3: adapter output instability

If adapters choose their own final IDs or emit unstable orderings, later seams inherit nondeterminism.

Mitigation:
- adapters emit draft output;
- seam 3 validates, sorts, and canonicalizes into final `ParsedUnit`s.

### Risk 4: cache invalidation bugs

If cache keys omit adapter version or path-sensitive identity, parse results can go stale incorrectly.

Mitigation:
- include `file_id`, `blob_fingerprint`, `adapter`, and `adapter_version` in cache keys.
- keep the entire cache contract in Phase B so Phase A does not promise behavior it does not need yet.

### Risk 5: platform starts absorbing concrete adapter logic

There is a strong temptation to sneak extension tables or parser specifics into `lang`.
That would collapse seam 3 into seam 4.

Mitigation:
- seam 3 contains no production adapters.
- test-only fake adapters prove the platform without committing to real parser behavior.

### Risk 6: too many fatal errors

If parse failures bubble out as `LangError`, higher-level seams will have brittle behavior.

Mitigation:
- reserve `LangError` for registry/cache/schema/platform failures only.
- represent file-level parse failures as `FailedParse` data.

### Risk 7: hidden dependency on live repo state

If adapters read the filesystem or environment directly, determinism breaks.

Mitigation:
- `ParseInput` includes only repo-relative path, file id, blob fingerprint, and bytes.
- adapter trait docs should explicitly forbid live I/O.

---

## 12. In scope / out of scope

### In scope

- real `src/lang/**` module tree
- adapter trait and registry
- parse request / parse set contracts
- draft symbol / edge / surface-marker contracts
- final normalized parsed-unit contracts
- deterministic validation and sorting
- explicit missing-requested-language handling
- embedded parse-manifest schema
- test-only fake adapters
- targeted tests for registry, parsing flow, schema parity, and determinism

### Out of scope

- the broad production JSON/TOML/YAML adapter family
- the broad production Rust adapter
- the broad production Python adapter
- the broad production JavaScript/TypeScript adapter family
- tree-sitter runtime integration
- parser-specific AST storage in shared contracts
- any cache contract or cache stats surface
- persistent on-disk cache
- cross-file symbol resolution
- repo-wide graph construction
- query execution
- detector/fact execution
- topology/path-class classification
- public API promotion of `lang`

A deliberate rule:

> seam 3 must be useful before the full adapter family lands.
> It earns that claim through one bounded real-adapter spike or one narrow real adapter plus one
> real consuming workflow, not through fake adapters alone.

---

## 13. Specific implementation items

### A. Replace the placeholder module

Replace:

```rust
//! Reserved for the future language platform seam. No runtime logic in commit 1.

#[allow(dead_code)]
pub(crate) struct ReservedForFutureSeam;
```

with the real Phase-A module tree and re-exports.

### B. Add schema directory and constants

Add:

```text
schemas/lang/parse_manifest.v1.json
src/lang/schema.rs
```

### C. Add tests

Add at minimum:

```text
tests/lang_registry.rs
tests/lang_parse.rs
tests/lang_schema.rs
```

### D. Add fixtures

Add at minimum:

```text
fixtures/lang/README.md
fixtures/lang/valid/
fixtures/lang/invalid/
```

Use a test-only fake adapter and repo-snapshot fixtures from seam 2.

### E. Update top-level docs

Update `README.md` so seam 3 is no longer described purely as an abstract future seam.

### F. Keep feature flags clean

Do **not** wire `config-lang`, `rust-lang`, `python-lang`, `javascript-lang`, or `typescript-lang` to production adapter modules in seam 3.
That belongs to seam 4.

### G. Add one real proof slice

Add one of these before Phase A is treated as complete:

- a bounded real-adapter spike against one real language/config family plus a fixture-backed
  representative repo, or
- one narrow real adapter plus one real consumer path proving `ParseSet` feeds a later seam.

The goal is not broad language support. The goal is to force the contracts through one real value
path before they are treated as stable.

---

## 14. Fixture and test plan

### Required test-only adapter

Define at least one deterministic fake adapter under `#[cfg(test)]`, for example:

- `builtin.fake_config`
- language id: `json`
- recognizes only `*.fake.json`
- parse contract encoded by simple JSON fields or line-based conventions

Its purpose is not realism.
Its purpose is proving:

- registry registration works;
- `ParseDriver` delegates correctly;
- draft-to-final symbol resolution works;
- file-level failures are represented as data;
- deterministic ordering and hashing work.

This fake adapter is necessary but not sufficient. It proves seam mechanics, not real-language
fitness.

### Required real proof slice

Add one bounded real-adapter spike or one narrow real adapter plus one real consumer-path test.

Minimum proof expectations:

- one representative real repo fixture or repo-derived snapshot input,
- one hostile-input case proving panic containment / deterministic failure conversion,
- one consumer-path assertion proving the result is not dead infrastructure,
- repeat-run stability on the same snapshot.

### Minimum tests

1. `lang_registry_rejects_duplicate_adapter_names`
2. `lang_registry_rejects_duplicate_language_registration`
3. `parse_driver_parses_matching_files_from_snapshot`
4. `parse_driver_skips_missing_requested_paths`
5. `parse_driver_reports_missing_requested_languages`
6. `parse_driver_skips_files_when_recognizes_returns_false_within_selected_language`
7. `parse_driver_uses_all_registered_adapters_when_request_languages_is_empty`
8. `parse_driver_turns_duplicate_local_keys_into_failed_parse`
9. `parse_driver_turns_unresolved_edge_local_keys_into_failed_parse`
10. `parse_driver_turns_unresolved_surface_marker_local_keys_into_failed_parse`
11. `parse_driver_turns_invalid_spans_into_failed_parse`
12. `symbol_id_generation_is_stable_for_equivalent_adapter_output`
13. `unit_fingerprint_matches_normalized_unit_and_is_stable`
14. `parse_set_run_level_diagnostics_do_not_duplicate_file_level_diagnostics`
15. `parse_manifest_fixture_validates_against_schema`
16. `parse_output_is_independent_of_registration_order`
17. `equivalent_parse_requests_normalize_to_the_same_request_fingerprint`
18. `parse_driver_does_not_read_live_filesystem_after_snapshot`
19. `crate_compiles_without_default_features_after_lang_seam_lands`
20. `parse_driver_returns_empty_result_for_empty_explicit_path_scope`
21. `parse_driver_converts_adapter_panic_into_failed_parse`
22. `parse_driver_does_not_emit_unbounded_no_matching_adapter_records_for_snapshot_scope`
23. `bounded_real_adapter_spike_proves_contract_fit_on_one_real_repo_or_snapshot`
24. `consumer_path_proves_parse_set_is_load_bearing`

---

## 15. Locked review decisions

These decisions are now locked for seam 3 based on the engineering review:

1. Reduced Phase A scope: land only `mod.rs`, `error.rs`, `adapter.rs`, `registry.rs`,
   `driver.rs`, `model.rs`, and `schema.rs`; fold symbol/edge/surface contracts into `model.rs`.
2. `lang` stays `pub(crate)` for seam 3.
3. seam 3 lands the platform first, but Phase A is not complete without one bounded real-adapter
   spike or one narrow real adapter plus one real consumer path.
4. `lang::LanguageId` is a re-export of `pack::LanguageId` in Phase A.
5. seam 3 reuses kernel `Diagnostic` and `Locator` directly.
6. adapters emit **draft** output; seam 3 computes final symbol IDs and canonical order.
7. file-level parse failures are data, not `LangError`.
8. Phase A keeps a single-adapter-per-language registry; ambiguous adapter arbitration is out of
   scope until a later seam proves it is needed.
9. requested languages with no registered adapter are handled explicitly in Phase A through
   deterministic top-level diagnostics and `MissingRequestedLanguage` records.
10. Phase A fixtures include both normalized `request` and `request_fingerprint`.
11. the entire cache contract and cache stats surface move to Phase B.
12. seam 3 adds one fixture schema file: `schemas/lang/parse_manifest.v1.json`.
13. seam 3 proves extensibility with test-only fake adapters before any production parser is added.
14. `ParseSet.diagnostics` is run-level only; file-level diagnostics are not duplicated upward.
15. explanatory `detail` fields use `Option<String>` consistently across non-happy-path records.
16. in Phase A, `recognizes()` is a file-level filter inside the selected language, not a second
    adapter-routing mechanism.
17. request normalization gets explicit coverage: logically equivalent language/path sets must
    produce the same normalized `ParseRequest` and the same `request_fingerprint`.
18. `ParseDriver::parse_snapshot` is specified as a single-pass inventory walk, not a per-language
    rescan loop.
19. `ParseDriver::parse_snapshot` takes `&self` in Phase A; cache-bearing mutability is deferred
    until Phase B introduces an explicit cache layer.
20. empty explicit path scopes produce deterministic empty results, not a whole-snapshot fallback.
21. snapshot-wide no-match files are counted, but not emitted as one skipped record per file unless
    the caller explicitly requested those paths.
22. panic containment and hostile-input behavior are part of Phase-A proof, not hand-waved adapter
    etiquette.

These decisions keep seam 3 narrow, deterministic, and compatible with the crate that actually
exists today.

## 16. Phase B detailed implementation plan

Phase B exists because the crate has now proven the seam shape. The next bottleneck is repeat work.

Today every `ParseDriver::parse_snapshot` run recomputes the same normalized per-file outcome even
when the repo snapshot, blob fingerprint, adapter, and adapter version are unchanged. That is not a
correctness bug. It is just waste. Phase B is the small, explicit fix.

### Mission

Phase B should make repeat parse runs cheaper without changing the Phase-A source of truth.

The key rule:

> Phase B caches the final normalized per-file parse result.
> It does not change routing semantics, request semantics, or cross-file ownership.

### Data flow

```text
RepoSnapshot
   |
   v
ParseDriver
   |
   +--> select adapter (unchanged Phase-A logic)
   |
   +--> build ParseCacheKey(file_id, blob_fingerprint, language, adapter, version)
   |
   +--> ParseCache::get
         |             |
         |             +--> Hit      -> append cached ParsedUnit/FailedParse
         |             |
         |             +--> Miss     -> parse -> normalize -> append -> cache put
         |             |
         |             +--> Disabled -> parse -> normalize -> append
   |
   v
ParseSet + ParseStats(cache_hits, cache_misses)
```

### Why this is the right layer

- The cache key belongs in `lang`, not `repo`, because only `lang` knows adapter identity and
  adapter version.
- The cache value belongs in `lang`, not `app/runtime`, because the thing being reused is the
  normalized parse outcome, not an app-specific derivation.
- The runtime helper belongs in `registry.rs`, not `app/runtime.rs`, because runtime should know
  “give me the built-in registry,” not “construct these five concrete adapter types in this order.”

### What Phase B must not do

- no on-disk cache files
- no cross-process cache sharing
- no caching of whole `ParseSet` values
- no caching of missing-language or no-match bookkeeping
- no new public API promotion
- no production adapter landings
- no query-engine hooks
- no runtime orchestration of parse runs in `app/runtime.rs`

### Sequence

1. Add `src/lang/cache.rs` with `ParseCacheKey`, `CachedParseOutcome`, `CacheLookup`,
   `ParseCache`, `NoopParseCache`, and `InMemoryParseCache`.
2. Extend `src/lang/mod.rs` to re-export the cache surface.
3. Update `ParseDriver` to own `Arc<dyn ParseCache>` internally and add `with_cache(...)`.
4. Wire cache hits and misses into `ParseStats`.
5. Add the tiny `built_in_registry()` helper in `registry.rs`.
6. Add deterministic tests for hit/miss behavior and invalidation.
7. Keep `app/runtime.rs` unchanged in this seam. The helper is enough runtime-readiness for now.

### Phase B acceptance criteria

- parsing the same `RepoSnapshot` twice through `InMemoryParseCache` yields byte-identical
  `ParseSet` output and non-zero `cache_hits` on the second run;
- changing only the blob bytes for one file causes a miss for that file and does not poison other
  cached results;
- changing only `adapter_version` causes a miss for that adapter even when bytes are unchanged;
- cached `FailedParse` outcomes are reused deterministically on repeat runs;
- `NoopParseCache` preserves Phase-A behavior exactly and leaves `cache_hits == 0` and
  `cache_misses == 0`;
- `built_in_registry()` returns deterministic output regardless of compile-time feature order;
- the crate still passes `cargo check -p substrate-lift --no-default-features`.

### Required Phase-B tests

1. `parse_driver_with_noop_cache_matches_phase_a_output`
2. `in_memory_cache_hits_on_second_equivalent_run`
3. `cache_miss_when_blob_fingerprint_changes`
4. `cache_miss_when_adapter_version_changes`
5. `cache_reuses_failed_parse_outcomes`
6. `cache_does_not_materialize_no_match_records`
7. `cache_counters_stay_zero_when_cache_is_disabled`
8. `built_in_registry_is_deterministic_for_enabled_features`
9. `phase_b_cache_preserves_no_default_features_compile_matrix`

### Failure modes specific to Phase B

| Codepath | Failure mode | Rescued? | Phase-B handling |
|---|---|---|---|
| cache key design | stale parse reused after adapter upgrade | yes | include `adapter_version` in `ParseCacheKey` |
| cache key design | rename/path identity mismatch | yes | rely on path-sensitive `file_id` plus `blob_fingerprint` |
| cache value design | raw draft output bypasses normalization fixes | yes | cache only `ParsedUnit` / `FailedParse` |
| disabled cache | misleading miss counters pollute stats | yes | `NoopParseCache` returns `Disabled`, not `Miss` |
| built-in registry helper | feature-order nondeterminism changes lookup behavior | yes | register in deterministic adapter-name order |

## 17. Phase B locked decisions

1. Cache normalized per-file outcomes, not raw adapter drafts.
2. Keep `ParseDriver::parse_snapshot(&self, ...)` in Phase B.
3. Use `NoopParseCache` as the default behind `ParseDriver::new(...)`.
4. Count cache hits and misses only when a cache is actually enabled.
5. Do not cache request-level bookkeeping like missing-language or no-match records.
6. Use `file_id + blob_fingerprint + language + adapter + adapter_version` as the cache key.
7. Keep all Phase-B cache implementation in `src/lang/cache.rs` unless the code proves it is too dense.
8. Add `built_in_registry()` now, even if it returns an empty registry until seam 4 lands adapters.

## 18. Phase B decision audit addendum

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 13 | Phase B | Cache normalized `ParsedUnit` and `FailedParse` outcomes only | Mechanical | P5 | The driver already owns normalization and failure semantics, so caching raw drafts would create a second truth source. | Caching adapter draft output |
| 14 | Phase B | Keep `ParseDriver::parse_snapshot(&self, ...)` and inject cache via trait | Mechanical | P5 | Runtime callers should not take on fake mutability just to turn caching on. | Switching the driver API to `&mut self` |
| 15 | Phase B | Key the cache by `file_id`, `blob_fingerprint`, `language`, `adapter`, and `adapter_version` | Mechanical | P1 + P5 | This captures path-sensitive file identity plus adapter invalidation without dragging request-scope noise into the key. | Keying only on path, only on blob, or on whole-request fingerprints |
| 16 | Phase B | Do not cache `SkippedParse`, missing-language, or whole-`ParseSet` results | Mechanical | P3 + P5 | Those are request-level bookkeeping artifacts, not reusable per-file parse outcomes. | Whole-run cache blobs or no-match negative caches |
| 17 | Phase B | Add `built_in_registry()` before real built-ins land | Mechanical | P3 | It gives later runtime code one stable seam without forcing production adapters into seam 3. | Wiring runtime code directly to concrete adapters later |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 2 | OPEN | mode: SELECTIVE_EXPANSION, 1 critical gap |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 5 | CLEAR | 11 issues, 0 critical gaps |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |

**UNRESOLVED:** 0 unresolved decisions across logged reviews.
**VERDICT:** ENG CLEARED — ready to implement. CEO review is optional and still has open scope-level concerns from the earlier autoplan pass.

---

## AUTOPLAN PHASE 1 — CEO REVIEW (2026-04-17)

### Working summary

This plan is grounded in the real crate state. `src/lang/mod.rs` is still a placeholder, `src/repo/**` and `src/pack/**` are already real seams, `pack::LanguageId` already exists, and `app/runtime.rs` still stops at `ProfileBootstrap`.

The strategic question is not whether a language seam is needed eventually. It is whether this seam should land first as pure platform contracts, or whether the first landing should prove one real end-to-end repo-intelligence outcome on one real language family.

### 0A. Premise challenge

| Premise | Assessment | Why it matters |
|---|---|---|
| Seam 3 should be the first real landing of `src/lang/**` | Valid | The current codebase has a real seam boundary to fill, and later seams need a parse artifact that is not a live-filesystem read. |
| Seam 3 should stay narrow and platform-only in Phase A | Medium confidence, challenged | This protects seam purity, but both outside voices converged that platform-only plus fake adapters risks shipping elegant plumbing with no proof of user value. |
| `lang::LanguageId` should re-export `pack::LanguageId` instead of creating a second enum | Valid | `pack::LanguageId` is already used by `CompiledAnalysisDefaults`, `CompiledQueryPack`, and `CompiledRuleScope`; duplicating it now would create churn without learning. |
| A fake adapter is enough to prove extensibility in Phase A | Rejected as sufficient proof | It proves internal coherence, not that the contracts survive one ugly real parser, one real repo, or one real consumer seam. |
| Single adapter per language is a safe Phase-A lock | Medium confidence, challenged | Reasonable as an initial simplifying rule, but not strong enough to lock as an architectural truth before one production adapter forces arbitration pressure. |
| Determinism is the dominant constraint | Valid, but incomplete | Determinism is core to this repo’s posture, but the plan does not yet connect that determinism to one user-visible win such as better replayability, lower false positives, or faster agent context capture. |

### 0B. Existing code leverage

| Sub-problem | Existing code | Reuse decision |
|---|---|---|
| Stable IDs, spans, fingerprints, diagnostics | `src/kernel/**` | Reuse directly. The seam should not mint parallel IDs, paths, span types, or diagnostic contracts. |
| Immutable repo bytes and deterministic inventory | `src/repo/**`, especially `RepoSnapshot`, `Inventory`, `BlobStore`, `SnapshotStats` | Reuse directly. This is the strongest part of the current wedge because it already enforces snapshot-based determinism. |
| Language identity contract | `src/pack/names.rs` `LanguageId` | Re-export through `lang`, do not duplicate. |
| Existing runtime ingress | `src/app/runtime.rs` `ProfileBootstrap` | Reuse as the example of how a seam should prove a real consumer instead of stopping at internal contracts. |
| Feature-flag posture | `tests/compile_matrix.rs` | Reuse and extend. `--no-default-features` is already guarded and should stay part of seam acceptance. |

Existing leverage is good. The plan is not reinventing kernel or repo. The strategic gap is elsewhere: it is not reusing the repo’s existing pattern of “land the seam, then prove one consuming path” strongly enough.

### 0C. Dream state mapping

```text
CURRENT STATE                  THIS PLAN (as written)             12-MONTH IDEAL
placeholder lang seam          deterministic parse contracts      trusted repo intelligence
no parsed units                fake-adapter validation only       real adapters on real repos
real repo snapshots            no production parser yet           downstream seams consume ParseSet
real pack/runtime seams        no user-visible wedge yet          one secure/replayable analysis moat
```

### 0C-bis. Implementation alternatives

APPROACH A: Platform-First Phase A
  Summary: Land the seam exactly as written, with registry, parse contracts, fake adapters, schema parity, and no production adapter.
  Effort:  M
  Risk:    Medium
  Pros:
  - Cleanest seam boundary and lowest immediate blast radius
  - Preserves deterministic contracts before parser-specific pressure arrives
  - Reuses kernel/repo sharply
  Cons:
  - Ships no direct user-visible outcome
  - Fake adapters can validate the shape and still miss the real contract breaks
  - Risks six-month regret: beautiful substrate, no compounding leverage
  Reuses:
  - `kernel/**`, `repo/**`, `pack::LanguageId`, existing compile-matrix posture

APPROACH B: Vertical Slice Wedge
  Summary: Land the seam plus one production-grade adapter on one high-value family, likely Rust or config, and prove one downstream consumer consumes `ParseSet`.
  Effort:  L
  Risk:    Medium
  Pros:
  - Forces the contracts through real data immediately
  - Produces a measurable user outcome instead of internal correctness only
  - Validates whether single-adapter-per-language and current symbol contracts survive contact with reality
  Cons:
  - Larger first landing
  - Increases scope and parser-specific complexity now
  - Might require revising some Phase-A “locked” decisions quickly
  Reuses:
  - everything in Approach A plus current Rust-heavy repo reality and the existing runtime seam pattern

APPROACH C: Adapter Spike Then Contract Freeze
  Summary: Do a bounded spike for one real adapter first, use that evidence to tighten the contracts, then land the platform seam immediately afterward.
  Effort:  M
  Risk:    Low to Medium
  Pros:
  - Lowest risk of freezing the wrong contracts
  - Still keeps the formal seam landing narrow
  - Gives real evidence for marker taxonomy, symbol IDs, and arbitration rules
  Cons:
  - Slightly slower than jumping straight into the seam docs as implementation source of truth
  - Requires discipline to keep the spike bounded and evidence-oriented
  Reuses:
  - existing repo snapshots, deterministic fixtures, current Rust/config repo surfaces

**RECOMMENDATION:** Choose Approach C if the goal is to preserve seam quality without lying to yourself about product learning. Choose Approach B if the team explicitly wants a bolder wedge now. Approach A is the cleanest document and the weakest proof.

### 0D. Mode-specific analysis

Mode selected: `SELECTIVE_EXPANSION`

Complexity check:
- The written Phase-A landing touches at least `src/lang/mod.rs`, `error.rs`, `adapter.rs`, `registry.rs`, `driver.rs`, `model.rs`, `schema.rs`, `schemas/lang/parse_manifest.v1.json`, `tests/lang_registry.rs`, `tests/lang_parse.rs`, `tests/lang_schema.rs`, `fixtures/lang/**`, and `README.md`.
- That is comfortably over the “8 files” smell threshold even before any production adapter or consumer is added.

Minimum set of changes that still achieves the stated goal:
- real `src/lang/**` module tree
- `LanguageId` re-export
- registry and driver
- normalized parsed-unit contracts
- schema parity
- compile-matrix preservation

Expansion scan:
- One real adapter spike
- One downstream consumer of `ParseSet`
- Hostile-input parse budget rules
- Parse coverage / failure-rate metrics
- Explicit moat framing in the plan

Cherry-pick result under `/autoplan` rules:
- Auto-decided mechanical carry-forward: keep the seam internal-only for now, reuse `pack::LanguageId`, keep parse output deterministic, keep cache out of Phase A.
- Surfaced as **User Challenge**, not auto-decided: add one real adapter or one bounded real-adapter spike plus one real consuming workflow before treating the seam as strategically approved.

### 0E. Temporal interrogation

| Window | What the implementer needs resolved now |
|---|---|
| Hour 1 (human), ~5-10 min with CC | Is this a pure contract seam, or is one real adapter expected in the first landing? This changes every file count and acceptance gate. |
| Hour 2-3 (human), ~10-20 min with CC | What hostile-input guarantees must adapters honor? Time/space budget, panic containment, malformed UTF-8, and maximum blob/span expectations are not fully specified yet. |
| Hour 4-5 (human), ~10-20 min with CC | Which downstream consumer proves the seam is real? Without that, the implementer can finish the seam and still not know if the contracts are load-bearing. |
| Hour 6+ (human), ~15-30 min with CC | What metrics decide success on real repos: parse coverage, failure rate, runtime, and downstream utility? The current acceptance criteria mostly measure internal hygiene. |

### 0F. Mode selection confirmation

`SELECTIVE_EXPANSION` still fits. The baseline plan is useful, but the only scope addition worth fighting for is the one that proves the seam is not dead infrastructure.

### CODEX SAYS (CEO — strategy challenge)

- Critical: the document optimizes seam purity before proving a customer wedge.
- High: fake adapters are not enough to validate the contract.
- High: all acceptance criteria are internal-quality gates, not value gates.
- High: the plan excludes every real adapter and consumer, which makes “platform” sound bigger than what ships.
- High: the moat is implied, not stated. If the moat is secure, replayable, deterministic repo analysis inside Substrate constraints, the plan should say that plainly.

### CLAUDE SUBAGENT (CEO — strategic independence)

- Critical: the milestone is phrased as “language platform,” not as one real repo-intelligence outcome.
- High: several core premises are assumed rather than falsified with evidence.
- High: the six-month regret scenario is obvious: perfect substrate, still no shipped capability.
- High: viable alternatives were ruled out too early without a comparative decision record.
- High: competitive risk is understated because commodity parser stacks can ship “good enough” faster.

### CEO DUAL VOICES — CONSENSUS TABLE

```text
═══════════════════════════════════════════════════════════════
  Dimension                           Claude  Codex  Consensus
  ──────────────────────────────────── ──────  ─────  ─────────
  1. Premises valid?                  mixed    mixed  DISAGREE on some premises, both flag gaps
  2. Right problem to solve?          no       no     CONFIRMED
  3. Scope calibration correct?       no       no     CONFIRMED
  4. Alternatives sufficiently explored?
                                      no       no     CONFIRMED
  5. Competitive/market risks covered?
                                      no       no     CONFIRMED
  6. 6-month trajectory sound?        no       no     CONFIRMED
═══════════════════════════════════════════════════════════════
```

Confirmed = 5/6. Disagreements = 1/6, but even the disagreement is about which premises survive, not about whether the current plan is strategically under-proved.

### Section 1: Architecture review

Examined: the seam boundary in `lift_seam3_spec_reviewed.md`, the existing module layout in `README.md`, the actual placeholder `src/lang/mod.rs`, and the pattern already used by `app/runtime.rs`.

Finding: the architecture is internally coherent and reuse-heavy, but the seam currently stops one layer too early. The repo already has a healthier pattern in seam 1: a clean internal seam plus one real consumer path. This plan does not yet require that proof for seam 3.

### Section 2: Error & Rescue Map

The spec does a decent job distinguishing seam-wide `LangError` from file-level `FailedParse` data. That part is strong.

The gap is operational rescue, not type-level rescue. There is no explicit contract yet for hostile parser behavior on malicious or pathological source files, which means parse failures are modeled but parse resource exhaustion is not.

### Section 3: Security & Threat Model

This seam parses untrusted repository content. That is the actual trust boundary, and the current plan only partially acknowledges it.

Gap: the plan should explicitly state parser-hostile-input rules for future real adapters: no live I/O, no env/time/randomness, bounded memory posture, deterministic handling of malformed inputs, and no adapter panic leaking past the seam boundary.

### Section 4: Data Flow & Interaction Edge Cases

There is no UI scope here, so the relevant interaction surface is internal data flow:

```text
RepoSnapshot ──▶ ParseDriver ──▶ Adapter parse draft ──▶ Validation/canonicalization ──▶ ParseSet
     │                │                   │                         │                       │
     │                │                   │                         │                       ├── missing requested language
     │                │                   │                         ├── invalid span/local key
     │                │                   ├── malformed bytes/input
     │                ├── missing adapter
     └── missing path / empty scope
```

The main gap is not a missing happy-path edge case. It is that no downstream consumer is named in the same data-flow slice, so the flow ends at an internal artifact instead of a proven use.

### Section 5: Code Quality Review

The document is precise and mostly explicit. The quality risk is not sloppy wording. It is false certainty.

The most brittle pattern is the “Locked review decisions” block. Several items are worthy implementation defaults, but they are written with more finality than the evidence currently supports.

### Section 6: Test Review

The fixture and determinism matrix is thorough for contract correctness. It is not thorough for product proof.

The missing tests are not more fake-adapter tests. The missing tests are one real-adapter stress case and one downstream-consumer test that proves `ParseSet` makes later seams materially easier.

### Section 7: Performance Review

No production parser lands in this phase, so there is limited concrete performance analysis to do. Still, the plan is missing one important invariant: what happens on very large blobs, symbol-heavy files, or adversarial nested structures once a real adapter appears.

That should be called out now, because otherwise performance posture gets deferred until after the contracts are frozen.

### Section 8: Observability & Debuggability Review

The plan defines diagnostics, stats, and fingerprints, which is good seam-local observability. What it does not yet define is what success looks like on real repos.

Add explicit run-level metrics expectations for later implementation: parse coverage by language, failed file count, skipped reasons, and stable output diffs between runs on the same snapshot. Those are the signals an operator will actually care about.

### Section 9: Deployment & Rollout Review

This seam is internal-only and not a separately distributed artifact, so there is no external rollout surface to map. That part is fine.

The rollout gap is organizational: if the team implements this exactly as written, it can declare victory with no obligation to prove a downstream outcome. That is the rollout failure mode to prevent.

### Section 10: Long-term trajectory review

The long-term seam map in `README.md` is coherent. The risk is path dependency.

If seam 3 lands as pure contracts plus fake adapters, seam 4 will inherit the pressure to preserve those contracts even if the first real adapters reveal they were wrong. That is how temporary scaffolding turns into permanent architecture.

### Section 11: Design & UX review

Skipped. No UI scope detected in the seam-3 plan. The only plan-file match for UI-like terms was a false-positive phrase in a contract comment, not a user surface.

## NOT in scope

- Full platform-first approval with no real adapter pressure. Deferred pending premise confirmation because both outside voices challenged it.
- Public API promotion of `lang`. Still correctly out of scope for this phase.
- Tree-sitter or parser-runtime integration. Still deferred, but the plan should say what evidence will trigger it.
- Multi-adapter arbitration. Keep out of Phase A unless the real-adapter spike immediately proves it is needed.

## What already exists

- `kernel/**` already owns the cross-seam primitives this seam needs.
- `repo/**` already gives the snapshot determinism story this seam should build on.
- `pack::LanguageId` already exists and should be reused.
- `app/runtime.rs` already shows a better precedent for “seam plus one consuming path.”

## Dream state delta

As written, this plan moves the repo from “no language seam” to “well-specified language contracts.” That is real progress.

It does **not** yet move the repo all the way to “trusted repo intelligence that helps an agent or app do something materially better.” The delta to the 12-month ideal is therefore still missing one real adapter, one real consumer, and one explicit moat narrative.

## Error & Rescue Registry

| Method / Codepath | What can go wrong | Exception / Failure class | Rescued? | Rescue action | User / operator sees |
|---|---|---|---|---|---|
| `LanguageRegistryBuilder::register` | duplicate adapter name | `LangError::DuplicateAdapterName` | Y | reject registration deterministically | test failure / startup failure with typed error |
| `LanguageRegistryBuilder::register` | duplicate language registration | `LangError::DuplicateLanguageAdapter` | Y | reject registration deterministically | test failure / startup failure with typed error |
| `ParseDriver::parse_snapshot` | requested language has no registered adapter | run-level diagnostic + `MissingRequestedLanguage` | Y | continue parse, surface top-level record | operator sees explicit missing-language record |
| `ParseDriver::parse_snapshot` | requested path not in snapshot | `SkippedParse { PathNotInSnapshot }` | Y | continue parse, surface skipped record | operator sees skipped reason |
| adapter `parse` | syntax or semantic parse failure | `FailedParse` + diagnostics | Y | continue parse, preserve failure as data | operator sees file-level failure |
| adapter `parse` | pathological input causes excessive resource use | not yet specified | N | add explicit hostile-input/budget rules before implementation | currently ambiguous |

## Failure Modes Registry

| Codepath | Failure mode | Rescued? | Test? | User sees? | Logged? |
|---|---|---|---|---|---|
| parse request normalization | logically equivalent request fingerprints diverge | planned | planned | internal drift only | planned |
| adapter registration | two adapters claim one language | yes | planned | typed startup failure | planned |
| file parse | duplicate local keys or unresolved local refs | yes | planned | `FailedParse` record | planned |
| real adapter on hostile file | panic / blow-up / pathological runtime | no | no | ambiguous | no |
| seam-wide rollout | seam ships with no downstream consumer | no | no | silent strategic failure | no |

Rows 4 and 5 are the important gaps. Row 4 is an implementation-critical gap. Row 5 is the strategy gap both outside voices converged on.

## Completion Summary

```text
  +====================================================================+
  |            MEGA PLAN REVIEW — COMPLETION SUMMARY                   |
  +====================================================================+
  | Mode selected        | SELECTIVE EXPANSION                         |
  | System Audit         | strong seam reuse, weak value proof         |
  | Step 0               | 3 approaches compared, 1 user challenge     |
  | Section 1  (Arch)    | 1 major issue found                         |
  | Section 2  (Errors)  | 6 paths mapped, 1 GAP                       |
  | Section 3  (Security)| 1 issue found, 0 immediate critical         |
  | Section 4  (Data/UX) | 1 edge-case gap, 1 missing consumer         |
  | Section 5  (Quality) | 1 issue found                               |
  | Section 6  (Tests)   | reviewed, 1 major gap                       |
  | Section 7  (Perf)    | 1 issue found                               |
  | Section 8  (Observ)  | 1 gap found                                 |
  | Section 9  (Deploy)  | 1 rollout risk flagged                      |
  | Section 10 (Future)  | Reversibility: 3/5, debt items: 2           |
  | Section 11 (Design)  | SKIPPED (no UI scope)                       |
  +--------------------------------------------------------------------+
  | NOT in scope         | written (4 items)                           |
  | What already exists  | written                                     |
  | Dream state delta    | written                                     |
  | Error/rescue registry| 6 methods, 1 CRITICAL-ISH GAP               |
  | Failure modes        | 5 total, 2 critical strategic gaps          |
  | TODOS.md updates     | 0 proposed yet                              |
  | Scope proposals      | 1 surfaced, pending user decision           |
  | CEO plan             | existing prior artifact referenced          |
  | Outside voice        | ran (codex + subagent)                      |
  | Lake Score           | 3/3 mechanical choices favored completeness |
  | Diagrams produced    | 2 (dream-state, internal data flow)         |
  | Stale diagrams found | 0                                           |
  | Unresolved decisions | 1 (listed below)                            |
  +====================================================================+
```

## Unresolved decision

### User Challenge 1

Both outside voices recommend changing the stated direction.

- What the current plan says: land seam 3 as platform contracts first, with fake adapters and no production adapter or real consumer required in Phase A.
- What both models recommend: require either one bounded real-adapter spike or one real adapter plus one real consuming workflow before treating seam 3 as strategically approved.
- Why: this is the cleanest way to preserve determinism and seam quality without optimizing architecture before the repo proves value.
- What context we might be missing: you may intentionally want to de-risk the contract layer first because the internal seam map itself is the near-term goal.
- If we are wrong, the cost is: you take on more immediate scope and parser-specific churn than you wanted in the first landing.

### Premise gate outcome

User response: `A`

Accepted direction:
- seam 3 may still land a clean internal platform surface,
- but it is no longer considered strategically complete without one bounded real-adapter spike or one real adapter plus one real consuming workflow.

This resolves the CEO-phase user challenge in favor of proving one real value slice before the seam is treated as fully approved.

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | CEO | Use `SELECTIVE_EXPANSION` mode | Mechanical | P1 + P2 | The baseline seam is useful, but the only worthwhile expansion is the one that proves the seam is real. | HOLD SCOPE without surfacing the missing wedge |
| 2 | CEO | Treat UI scope as absent | Mechanical | P5 | The plan contains no real user interface surface, so a design phase would be noise. | Running design review from a false-positive keyword match |
| 3 | CEO | Keep `lang::LanguageId` as a re-export of `pack::LanguageId` | Mechanical | P4 + P5 | Existing compiled pack contracts already depend on this type, so duplication would create churn without learning. | Defining a second `LanguageId` enum in `lang` |
| 4 | CEO | Keep cache out of Phase A | Mechanical | P3 + P5 | The seam is already large; adding cache before a real adapter arrives grows complexity without proving value. | Shipping cache contracts in the first landing |
| 5 | CEO | Require user confirmation on adding one real adapter or adapter spike plus one consumer | User Challenge | P1 + P2 | Both outside voices converged that platform-only Phase A is strategically under-proved. | Auto-approving platform-only Phase A as sufficient proof |
| 6 | CEO | Accept gate option A: require one real adapter spike or one real adapter plus one consumer | User-approved challenge | P1 + P2 | The seam now has to prove value, not just internal correctness. | Keeping Phase A approved as fake-adapter-only infrastructure |
| 7 | Design | Skip design review | Mechanical | P5 | No UI scope exists in this seam plan, so a design pass would add noise instead of signal. | Running a UI review off a false-positive text match |
| 8 | Eng | Make the accepted proof slice part of the actual definition of done | Mechanical | P1 + P5 | The plan cannot have one review gate and a different implementation finish line. | Leaving the accepted CEO gate outside the seam acceptance criteria |
| 9 | Eng | Define empty explicit path scope as deterministic empty output | Mechanical | P5 | Empty inputs need explicit semantics or the first caller will accidentally trigger whole-repo work. | Implicit or ambiguous empty-scope behavior |
| 10 | Eng | Keep snapshot-scope no-match files in stats, not per-file skipped records | Mechanical | P3 + P5 | Prevents manifest-size blowups while preserving deterministic accounting. | Emitting one skipped record per non-matching file in snapshot scope |
| 11 | Eng | Use `ParseDriver::parse_snapshot(&self, ...)` in Phase A | Mechanical | P5 | There is no Phase-A cache/state yet, so mutability would only imply hidden coupling. | Smuggling future cache mutability into the first contract |
| 12 | Eng | Make panic containment and hostile-input behavior explicit Phase-A proof requirements | Mechanical | P1 + P5 | “Adapters must not panic” is etiquette, not a trust boundary. The seam has to own containment. | Leaving panic/resource-failure semantics implicit |

### Phase-transition summary

**Phase 1 complete.** Codex: 8 concerns. Claude subagent: 5 issues. Consensus: 5/6 confirmed, 1 disagreement surfaced as a premise-level caution. Passing to the premise gate before any later `/autoplan` phase continues.

## AUTOPLAN PHASE 2 — DESIGN REVIEW

Skipped. No UI scope was detected in this seam plan. The only earlier keyword hit was a false positive inside contract prose, not a user-visible screen, layout, or interaction surface.

**Phase 2 complete.** Skipped, no UI scope. Passing to Phase 3.

## AUTOPLAN PHASE 3 — ENG REVIEW (2026-04-17)

### Step 0: Scope challenge

What already exists:
- `src/kernel/**` already owns stable IDs, spans, diagnostics, paths, and fingerprints.
- `src/repo/**` already owns deterministic snapshot materialization, blob reads, stats, and schema-backed manifests.
- `src/pack/schema.rs` and `src/repo/schema.rs` already show the exact schema-constant pattern seam 3 should follow.
- `tests/compile_matrix.rs` already guards the `--no-default-features` posture.
- `README.md` already separates seam 3 “language platform” from seam 4 “concrete adapters”.

Minimum complete slice after the accepted gate:
- the reduced `src/lang/**` module tree,
- deterministic parse contracts and driver,
- schema parity,
- fake-adapter seam proof,
- one bounded real-adapter spike or one narrow real adapter plus one real consumer path,
- explicit hostile-input containment semantics.

Complexity check:
- This is still a >8-file seam even in the reduced cut.
- That is acceptable only because the seam already has strong reuse from `kernel`, `repo`, and the existing schema/test patterns.
- The dangerous shortcut would be pretending fake-adapter coverage is enough. That shortcut is no longer allowed after the accepted gate.

### CLAUDE SUBAGENT (eng — independent review)

- P1: contracts are being locked before one real adapter and one real consumer prove them.
- P1: “must not panic” is not a containment strategy for hostile input.
- P2: `ParseSet` can grow too large if skip semantics are materialized naively.
- P2: empty explicit path scope semantics were previously unspecified.
- P2: `lang` depending on `pack::LanguageId` is coupling pressure to watch.
- P3: `ParseDriver::parse_snapshot(&mut self, ...)` was unnecessary Phase-A mutability.

### CODEX SAYS (eng — architecture challenge)

- Critical: the accepted CEO gate was not yet reflected in the actual definition of done.
- High: failure isolation was aspirational, not enforceable.
- High: `pack::LanguageId` may be too coarse as a long-term runtime routing identity.
- Medium: `driver.rs` risked becoming a god-module with validation, normalization, and fingerprinting piled together.
- Medium: schema parity was at risk of ossifying the model before one real adapter validated it.

### ENG DUAL VOICES — CONSENSUS TABLE

```text
═══════════════════════════════════════════════════════════════
  Dimension                           Claude  Codex  Consensus
  ──────────────────────────────────── ──────  ─────  ─────────
  1. Architecture sound?              mixed    mixed  DISAGREE on details, both flag contract-proof gap
  2. Test coverage sufficient?        no       no     CONFIRMED
  3. Performance risks addressed?     no       no     CONFIRMED
  4. Security threats covered?        no       no     CONFIRMED
  5. Error paths handled?             no       no     CONFIRMED
  6. Deployment risk manageable?      mixed    mixed  CONFIRMED
═══════════════════════════════════════════════════════════════
```

Consensus confirmed: 5/6. The one mixed dimension is not “architecture is fine,” it is “the core shape is coherent, but only if the proof slice and containment rules become part of the source-of-truth plan.”

### 1. Architecture review

The seam boundary is still the right one. `kernel` owns primitives, `repo` owns bytes, `lang` should own per-file parsing contracts, and seam 4 should still own the broad adapter family.

The architecture problem was not the seam split. It was the lack of a proof slice and the risk that `driver.rs` silently becomes the home for too many responsibilities. The accepted gate and the new containment rules fix the first issue. The second remains a watch item for implementation.

Required ASCII architecture diagram:

```text
                    +-------------------+
                    |   kernel/**       |
                    | ids, spans, diag  |
                    +---------+---------+
                              |
                              v
 +-------------------+   +----+----+    +-------------------+
 |   repo/**         |-->|  lang    |--->| later seams       |
 | snapshot/blob     |   | driver   |    | graph/query/app   |
 | inventory/stats   |   | registry |    | consume ParseSet  |
 +-------------------+   | model    |    +-------------------+
                         | schema    |
                         +----+------+
                              |
                +-------------+-------------+
                |                           |
                v                           v
      fake adapter seam proof      bounded real proof slice
      (mechanics)                  (one real adapter spike or
                                   one consumer path)
```

Coupling assessment:
- Good coupling: `lang` depends on `kernel` and `repo`.
- Acceptable temporary coupling: `lang::LanguageId` re-exporting `pack::LanguageId`.
- Bad future coupling to avoid: `lang` depending on `pack::compiler`, app code, or graph/query seams.

### 2. Code quality review

The spec is explicit, but the old version was explicit in the wrong place. It locked decisions before the proof slice existed. That is the kind of precision that causes rework instead of preventing it.

Mechanical quality fixes now reflected in the plan:
- `ParseDriver::parse_snapshot` is `&self` in Phase A, not `&mut self`.
- empty explicit path scope is defined, not implied.
- snapshot-wide no-match files are counted, not emitted one-by-one as skipped records.
- proof-slice requirements are part of “done,” not just appended review text.

The remaining code-quality risk is implementation density in `driver.rs`. If normalization, local-ref resolution, span checks, sorting, ID derivation, and fingerprinting all collapse into one long module, the first real adapter will force an immediate refactor.

### 3. Test review

This seam is library-only, so the critical flows are codepath flows, not UI flows.

Required coverage diagram:

```text
CODE PATH COVERAGE
===========================
[+] LanguageRegistryBuilder
    ├── duplicate adapter name rejected
    ├── duplicate language rejected
    └── deterministic descriptor order

[+] ParseDriver::parse_snapshot
    ├── ParseScope::Snapshot
    ├── ParseScope::Paths(non-empty)
    ├── ParseScope::Paths(empty) [GAP before eng review, now required]
    ├── missing requested language
    ├── missing requested path
    ├── no-matching-adapter in snapshot scope (counter only)
    └── no-matching-adapter for explicitly requested path (skipped record)

[+] Adapter output normalization
    ├── duplicate local keys -> FailedParse
    ├── unresolved local refs -> FailedParse
    ├── invalid spans -> FailedParse
    ├── stable symbol ids
    └── stable unit fingerprint

[+] Hostile input containment
    ├── adapter panic -> FailedParse [NEW REQUIRED]
    ├── malformed bytes -> deterministic diagnostics [NEW REQUIRED]
    └── pathological real-adapter spike input [NEW REQUIRED]

[+] Real proof slice
    ├── bounded real-adapter spike on one real repo/snapshot [NEW REQUIRED]
    └── consumer-path test proving ParseSet is load-bearing [NEW REQUIRED]
```

Coverage judgment:
- The previous fake-adapter matrix was strong for seam mechanics.
- It was insufficient for production confidence.
- The new required tests are the ones that would matter at 2am Friday: empty scope, panic containment, skip-output size behavior, one real spike, one real consumer.

Test plan artifact written to:
- [spensermcconnell-feat-lift-eng-review-test-plan-20260417-223430.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-lift-eng-review-test-plan-20260417-223430.md)

### 4. Performance review

No broad production parser lands yet, so most performance analysis is future-facing. The non-future-facing issue was output size, not runtime.

The important correction is that `ParseSet.skipped` must not become a giant “we skipped everything” manifest on large snapshots. Counting snapshot-wide no-match files in stats and only materializing path-specific skip records is the right explicit Phase-A rule.

The second performance watch item is the real proof slice. That spike needs repeat-run stability and at least a basic boundedness readout on one representative repo or repo-derived snapshot.

### NOT in scope

- the broad production adapter family
- tree-sitter/runtime integration
- public API promotion of `lang`
- cache contracts and cache stats
- graph/query/facts/app integration beyond one narrow consumer proof path

### What already exists

- schema constant pattern in `src/pack/schema.rs` and `src/repo/schema.rs`
- fixture/test folder conventions under `tests/**` and `fixtures/**`
- compile-matrix enforcement in `tests/compile_matrix.rs`
- repo snapshot purity and determinism posture already enforced below this seam

### Failure modes

| Codepath | Failure mode | Test? | Error handling? | User / operator sees |
|---|---|---|---|---|
| `ParseDriver::parse_snapshot` | empty explicit path set accidentally parses whole snapshot | now required | now specified | deterministic empty result |
| adapter parse | panic on hostile input | now required | now specified | `FailedParse` with diagnostics |
| snapshot-scope no-match files | output-size blow-up from per-file skipped records | now required | now specified | bounded manifest + stats |
| real proof slice | contracts fail on first real language/repo | now required | partially by proof gate | seam not accepted as complete |
| consumer path | `ParseSet` remains orphan internal artifact | now required | proof gate | seam not accepted as complete |

### Worktree parallelization strategy

Sequential implementation is still the safest default because most of the Phase-A work clusters under `src/lang/**`.

There is only one safe side lane:
- Lane A: `src/lang/**` contracts, driver, and normalization
- Lane B: `schemas/lang/**`, `tests/lang_schema.rs`, fixture scaffolding, and README updates

Execution order:
- Launch Lane B once the core model names stabilize.
- Keep Lane A authoritative because it owns the actual seam contracts.

### TODOS.md updates

No new root `TODOS.md` update is proposed from this pass.

Reason:
- this repo does not currently maintain a root `TODOS.md`,
- and every material deferred item for seam 3 is already better tracked directly in this reviewed seam spec.

### Completion summary

```text
  Step 0: Scope Challenge — accepted proof-slice requirement carried into the spec
  Architecture Review: 2 major issues found, 1 fixed directly in the plan
  Code Quality Review: 2 issues found, 2 fixed directly in the plan
  Test Review: diagram produced, 5 critical gaps identified and converted into requirements
  Performance Review: 1 issue found, fixed via explicit skip-output semantics
  NOT in scope: written
  What already exists: written
  TODOS.md updates: 0
  Failure modes: 5 high-signal modes mapped
  Outside voice: ran (codex + subagent)
  Parallelization: 2 lanes, 1 primary + 1 side lane
  Lake Score: 5/5 recommendations favored complete over shortcut
```

### Cross-phase themes

**Theme: prove one real value slice**
- flagged in CEO phase and again in both engineering outside voices.
- high-confidence signal. This was the dominant cross-phase concern.

**Theme: fake adapters are necessary but not sufficient**
- flagged in CEO phase and eng phase independently.
- high-confidence signal. The seam needs one real contract stress case.

**Theme: determinism without containment is not enough**
- CEO phase surfaced the value side of determinism.
- Eng phase surfaced the hostile-input and panic-containment side.
- together they say the same thing: the seam must be trustworthy, not just tidy.

### Phase-transition summary

**Phase 3 complete.** Codex: 5 concerns. Claude subagent: 6 issues. Consensus: 5/6 confirmed, 1 mixed architecture dimension. The reviewed spec now includes the accepted proof-slice requirement, explicit empty-scope semantics, bounded no-match skip behavior, and Phase-A panic-containment expectations.
