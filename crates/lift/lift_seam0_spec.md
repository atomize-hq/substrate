# `lift` seam 0 spec

Yes, seam 0 should now be treated as a strict kernel contract seam, not a bag of helpers.

It should stay as an internal module under `crates/lift/src/kernel`, not as a separate workspace crate yet. The repo is already a multi-member Cargo workspace targeting Rust 1.89, and the current Lift system is already contract-first: the existing Lift Vector schema is Draft 2020-12 with `additionalProperties: false`, required root keys, and pinned v1 semantics; the current `--emit-json` contract also requires sorted unique arrays, additive evolution only, and deterministic `triggers` and `missing_inputs` for the same input and config. That makes seam 0 the right place to define the shared wire primitives and determinism rules that every later seam will inherit.

Two standards-level choices are locked here:

- internal field addressing uses JSON Pointer as the canonical path primitive, because RFC 6901 gives a standard, machine-precise way to identify values inside JSON-like documents
- canonical JSON bytes for hashing and fingerprints use RFC 8785 JCS, because it standardizes deterministic property ordering and canonical JSON serialization

The compat note is important: current Work Lift v1 emits dotted paths like `touch.crates_touched` in `missing_inputs`, so seam 0 must not make dotted field paths canonical. Keep JSON Pointer canonical in the kernel, and let `app::score::compat_v1` translate between `"/touch/crates_touched"` and `"touch.crates_touched"` at the boundary.

## 1. Mission

Seam 0 owns the portable primitive contracts used across the entire crate.

That means:

- deterministic IDs
- repo-relative paths
- byte spans and locators
- JSON field pointers
- fingerprints
- diagnostics
- canonical JSON serialization
- shared typed kernel errors
- versioning conventions for schemas and wire objects

It does not own:

- repo walking
- AST parsing
- graph edges
- facts
- derived facts
- Lift vectors
- scoring
- query matching
- patch plans
- CLI rendering

The kernel must stay useful even if score disappeared.

## 2. Directory and module shape

```text
crates/lift/
  src/kernel/
    mod.rs
    error.rs
    path.rs
    json_pointer.rs
    id.rs
    span.rs
    locator.rs
    fingerprint.rs
    canonical_json.rs
    diagnostic.rs
    schema.rs

  schemas/kernel/
    primitives.v1.json
```

`mod.rs` should re-export only the stable public surface.

```rust
pub mod canonical_json;
pub mod diagnostic;
pub mod error;
pub mod fingerprint;
pub mod id;
pub mod json_pointer;
pub mod locator;
pub mod path;
pub mod schema;
pub mod span;

pub use canonical_json::{canonical_json_bytes, canonical_json_string};
pub use diagnostic::{Diagnostic, DiagnosticCode, RelatedLocation, Severity};
pub use error::{KernelError, KernelResult};
pub use fingerprint::{sha256_bytes, sha256_canonical_json, Fingerprint};
pub use id::{
    BoundaryId, ComponentId, EvidenceId, FactId, FileId, MatchId, QueryId, RecipeId, RuleId,
    StableId, SymbolId,
};
pub use json_pointer::{FieldPath, JsonPointer};
pub use locator::Locator;
pub use path::RepoPath;
pub use span::ByteSpan;
```

Kernel-level crate rules:

```rust
#![forbid(unsafe_code)]
#![deny(missing_docs)]
```

## 3. Public Rust surface

This is the exact shape to lock first.

```rust
pub type ByteOffset = u64;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RepoPath(String);
// Canonical form:
// - UTF-8 only
// - repo-root-relative
// - POSIX separators only ('/')
// - no leading slash
// - no trailing slash
// - no empty segments
// - no '.' or '..' segments
// - no backslashes

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JsonPointer(String);
// RFC 6901 canonical pointer syntax.
// Empty string "" means root.
// Leading '/' required for non-root pointers.

pub type FieldPath = JsonPointer;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Fingerprint(String);
// Format: "sha256:<64 lower-hex chars>"

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableId(String);
// Format: "<kind>:sha256:<64 lower-hex chars>"

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SymbolId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BoundaryId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FactId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EvidenceId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuleId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueryId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecipeId(StableId);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MatchId(StableId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ByteSpan {
    pub start_byte: u64,
    pub end_byte: u64,
}
// Half-open interval [start_byte, end_byte)

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Locator {
    pub path: RepoPath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<ByteSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_pointer: Option<JsonPointer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiagnosticCode(String);
// Pattern: dot-separated, lowercase namespaced code
// Example: "kernel.repo_path.invalid_absolute"

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelatedLocation {
    pub locator: Locator,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Locator>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}
```

### Constructors and helpers

These should exist in seam 0 and nowhere else:

```rust
impl RepoPath {
    pub fn parse(input: &str) -> KernelResult<Self>;
    pub fn as_str(&self) -> &str;
    pub fn join(&self, child: &RepoPath) -> KernelResult<Self>;
    pub fn parent(&self) -> Option<Self>;
}

impl JsonPointer {
    pub fn parse(input: &str) -> KernelResult<Self>;
    pub fn root() -> Self;
    pub fn as_str(&self) -> &str;
    pub fn push_token(&self, token: &str) -> Self;
}

impl StableId {
    pub fn parse(input: &str) -> KernelResult<Self>;
    pub fn from_identity(kind: &'static str, identity_lemma: &str) -> Self;
    pub fn kind(&self) -> &str;
    pub fn as_str(&self) -> &str;
}

impl Fingerprint {
    pub fn parse(input: &str) -> KernelResult<Self>;
    pub fn as_str(&self) -> &str;
}

impl ByteSpan {
    pub fn new(start_byte: u64, end_byte: u64) -> KernelResult<Self>;
    pub fn len(&self) -> u64;
    pub fn is_empty(&self) -> bool;
}

pub fn canonical_json_string<T: Serialize>(value: &T) -> KernelResult<String>;
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> KernelResult<Vec<u8>>;
pub fn sha256_bytes(bytes: &[u8]) -> Fingerprint;
pub fn sha256_canonical_json<T: Serialize>(value: &T) -> KernelResult<Fingerprint>;
```

## 4. Error surface

```rust
pub type KernelResult<T> = Result<T, KernelError>;

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("invalid repo path: {reason}")]
    InvalidRepoPath { input: String, reason: String },

    #[error("invalid JSON pointer")]
    InvalidJsonPointer { input: String },

    #[error("invalid stable id")]
    InvalidStableId { input: String, expected_kind: Option<&'static str> },

    #[error("invalid fingerprint")]
    InvalidFingerprint { input: String },

    #[error("invalid byte span: start {start_byte} > end {end_byte}")]
    InvalidByteSpan { start_byte: u64, end_byte: u64 },

    #[error("canonical JSON failure: {reason}")]
    CanonicalJson { reason: String },

    #[error("schema validation failure: {reason}")]
    SchemaViolation { reason: String },
}
```

Rule: seam 0 exposes typed errors, not `anyhow::Error`.

## 5. Schema inventory

The repo already uses Draft 2020-12 for Lift’s current JSON contracts, so seam 0 stays on the same dialect and keeps hand-authored schemas under `schemas/kernel/`.

Ship one source-of-truth schema file:

- `schemas/kernel/primitives.v1.json`

Later app schemas can `$ref` into its `$defs`.

### `schemas/kernel/primitives.v1.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.substrate.dev/lift/kernel/primitives.v1.json",
  "title": "Lift Kernel Primitives v1",
  "$defs": {
    "repo_path": {
      "type": "string",
      "minLength": 1,
      "pattern": "^(?!/)(?!.*\\\\)(?!.*//)(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*/$).+$",
      "description": "UTF-8 repo-root-relative POSIX path."
    },
    "json_pointer": {
      "type": "string",
      "description": "RFC 6901 JSON Pointer. Empty string denotes root."
    },
    "stable_id": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9_]*:sha256:[0-9a-f]{64}$"
    },
    "fingerprint": {
      "type": "string",
      "pattern": "^sha256:[0-9a-f]{64}$"
    },
    "byte_span": {
      "type": "object",
      "additionalProperties": false,
      "required": ["start_byte", "end_byte"],
      "properties": {
        "start_byte": { "type": "integer", "minimum": 0 },
        "end_byte": { "type": "integer", "minimum": 0 }
      }
    },
    "locator": {
      "type": "object",
      "additionalProperties": false,
      "required": ["path"],
      "properties": {
        "path": { "$ref": "#/$defs/repo_path" },
        "span": { "$ref": "#/$defs/byte_span" },
        "json_pointer": { "$ref": "#/$defs/json_pointer" }
      }
    },
    "diagnostic_code": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9]*(\\.[a-z][a-z0-9_]*)+$"
    },
    "severity": {
      "type": "string",
      "enum": ["error", "warning", "info"]
    },
    "related_location": {
      "type": "object",
      "additionalProperties": false,
      "required": ["locator", "message"],
      "properties": {
        "locator": { "$ref": "#/$defs/locator" },
        "message": { "type": "string", "minLength": 1 }
      }
    },
    "diagnostic": {
      "type": "object",
      "additionalProperties": false,
      "required": ["code", "severity", "message"],
      "properties": {
        "code": { "$ref": "#/$defs/diagnostic_code" },
        "severity": { "$ref": "#/$defs/severity" },
        "message": { "type": "string", "minLength": 1 },
        "subject": { "$ref": "#/$defs/locator" },
        "related": {
          "type": "array",
          "items": { "$ref": "#/$defs/related_location" }
        },
        "help": { "type": "string", "minLength": 1 }
      }
    }
  }
}
```

### Runtime-only invariants not fully expressible in schema

These must be enforced in Rust validators:

- `ByteSpan.end_byte >= ByteSpan.start_byte`
- `Locator.json_pointer` must parse as RFC 6901
- `RepoPath` must be normalized, not just regex-valid
- `StableId.kind()` must match the typed newtype constructor
- arrays that are logical sets must be sorted ascending and deduped before serialization
- canonical JSON hashing must reject non-canonicalizable values

## 6. Determinism rules

These are non-negotiable.

### 6.1 Repo paths

`RepoPath` canonical form:

- UTF-8 only
- relative to repo root
- forward slashes only
- no leading slash
- no trailing slash
- no `.` segment
- no `..` segment
- no empty segment
- no backslash
- equality is exact byte equality of the normalized UTF-8 string

Example valid:

- `src/lib.rs`
- `crates/lift/src/kernel/path.rs`
- `docs/project_management/system/README.md`

Example invalid:

- `/src/lib.rs`
- `.\src\lib.rs`
- `src//lib.rs`
- `src/./lib.rs`
- `src/../lib.rs`
- `src/lib.rs/`

### 6.2 Field paths

Internal canonical field references use `JsonPointer` only.

Examples:

- `""`
- `/touch/crates_touched`
- `/contract/behavior_deltas`
- `/risk/unknowns_high`

Compat rule:

- `app::score::compat_v1` maps internal pointers to current dotted Lift field strings on output.

### 6.3 Stable IDs

Stable ID wire format is fixed:

`<kind>:sha256:<64-lower-hex>`

Examples:

- `file:sha256:7f4d...`
- `symbol:sha256:2c9a...`
- `component:sha256:8b18...`

Kernel rule:

- seam 0 defines the output format and digest algorithm
- each producing seam defines the identity lemma it hashes
- identity lemmas must be deterministic and documented in that seam’s spec

Recommended lemma prelude:

- `lift-kernel\0v1\0<kind>\0<lemma>`

### 6.4 Fingerprints

Fingerprint wire format is fixed:

- `sha256:<64-lower-hex>`

Allowed fingerprint inputs:

- raw bytes
- canonical JSON bytes

Canonical JSON must be RFC 8785 JCS over an I-JSON-compatible payload.

### 6.5 Diagnostics

Diagnostics are logical records, not free-form logs.

Sort order must be deterministic:

1. severity rank: error, warning, info
2. subject path
3. subject span start
4. subject span end
5. code
6. message

### 6.6 Maps and set-like arrays

- public serializable maps must use deterministic ordering (`BTreeMap` / `BTreeSet` or equivalent)
- arrays that are conceptually sets must be emitted sorted ascending and unique

This aligns with the current Lift v1 `emit-json` contract, which already requires sorted unique arrays for `triggers` and `missing_inputs`.

## 7. Versioning conventions

Seam 0 should define the rule, but not ship a generic `VersionTag` JSON object in v1.

Conventions:

- top-level wire objects use explicit fields like `api_version`, `schema_version`, `model_version`, `profile_version`
- these version fields are integers `>= 1`
- semver strings are used only for crate, adapter, or tool versions
- v1 evolution rule for kernel primitives:
  - additive only
  - never rename or remove existing fields in `primitives.v1.json`
  - incompatible changes require `primitives.v2.json`

This mirrors the existing Lift v1 evolution posture: additive contracts and frozen v1 semantics.

## 8. Acceptance criteria

Seam 0 is done only when all of these are true.

### Contract completeness

- every public serializable kernel type has a matching `$defs` entry in `schemas/kernel/primitives.v1.json`
- every `$defs` entry has at least one round-trip fixture
- public examples exist for:
  - valid repo path
  - invalid repo path
  - valid JSON pointer
  - valid stable ID
  - valid fingerprint
  - valid locator
  - valid diagnostic

### Determinism

- canonical JSON serialization is byte-identical across repeated runs for the same value
- map insertion order does not affect canonical JSON output
- stable IDs are identical across repeated runs for the same kind plus lemma
- diagnostics sort deterministically
- no public kernel serialization uses `HashMap` iteration order

### Validation behavior

- invalid repo paths are rejected at construction time
- invalid JSON Pointers are rejected at construction time
- invalid stable IDs are rejected at parse time
- invalid byte spans are rejected at construction time
- typed ID constructors reject wrong kinds

### Boundary cleanliness

- kernel has zero dependencies on:
  - repo walking
  - git
  - tree-sitter
  - query engine
  - Lift score types
  - CLI
- kernel exposes typed errors only
- kernel performs no I/O, no env access, no time access, no randomness

### Schema parity

- Rust serde round-trips match schema expectations
- fixture JSON validates against `primitives.v1.json`
- parity tests fail if a public Rust field changes without a schema update

### Compatibility readiness

- there is a tested helper in `app::score::compat_v1` that translates:
  - `JsonPointer` to dotted Lift field path strings
  - dotted Lift field path strings to `JsonPointer`
- seam 0 itself contains no Lift-specific dotted field path logic

## 9. Invariants

These are must-not-break constraints.

- `RepoPath` is always normalized.
- `RepoPath` is never absolute.
- `JsonPointer` is the only canonical field-path type inside the engine.
- `ByteSpan` is always half-open and valid.
- `StableId` and `Fingerprint` are lowercase only.
- stable IDs never encode host-specific absolute paths directly.
- kernel types are plain data; no hidden global state.
- kernel serialization is deterministic.
- kernel schemas are additive within a version.
- kernel never depends on any app seam.
- human-friendly rendering concerns do not enter kernel contracts.
- seam 0 does not know what a Lift Vector, Contract Delta, or Patch Plan is.

## 10. Falsification questions

These are the PR-review questions that can invalidate the seam.

- Can two equivalent repo paths produce different `RepoPath` values?
- Can a backslash path like `src\\lib.rs` enter the kernel without rejection?
- Can a `.` or `..` path segment survive normalization?
- Can the same logical JSON value hash differently because object keys were inserted in a different order?
- Can a stable ID depend on memory address, hash-map iteration order, temp directory, or wall clock?
- Can a dotted Lift field path appear anywhere in kernel types?
- Can a `Locator` represent a source location without a valid normalized `RepoPath`?
- Can `ByteSpan { start > end }` be constructed?
- Can a diagnostic be emitted without a stable machine code?
- Can a kernel public type reference a score-specific, policy-specific, or query-specific enum?
- Can an incompatible field change land in `primitives.v1.json` without a version bump?
- Can kernel serialization differ across Linux, macOS, and Windows for the same fixture payload?
- Can public kernel APIs require `anyhow` or stringly-typed errors to interpret failures?
- Can non-deterministic collection ordering leak into any public JSON object?

If any answer is yes, seam 0 is not clean enough.

## 11. In scope and out of scope

### In scope

- normalized repo paths
- JSON pointers
- stable ID envelope format
- fingerprint envelope format
- byte spans
- locators
- diagnostics
- canonical JSON
- typed kernel errors
- hand-authored JSON schema
- determinism rules
- schema parity tests
- compat hook for dotted Lift field paths outside kernel

### Out of scope

- non-UTF8 repo path support in v1
- line and column display ranges
- AST node shapes
- graph edges
- source excerpts
- query DSL
- patch hunks
- Lift score math
- policy findings
- app response envelopes
- repo walking
- git integration
- editor and LSP-specific UTF-16 position types
- binary serialization formats
- caching strategy

Non-UTF8 paths stay out of scope for v1. If the repo substrate later encounters them, it should surface a diagnostic and skip them until there is a deliberate v2 design.

## 12. Risks and mitigations

### Risk 1: seam 0 becomes a junk drawer

Mitigation: no app types, no repo I/O, no parsing, no scoring.

### Risk 2: too many abstractions too early

Mitigation: keep only primitive contracts here. No `EntityRef`, `FactSet`, or `MatchSet`.

### Risk 3: canonical JSON implementation drift

Mitigation: treat RFC 8785 as normative and add byte-for-byte golden tests.

### Risk 4: dotted Lift paths leak into the engine

Mitigation: make JSON Pointer canonical in kernel and quarantine dotted-path logic to `app::score::compat_v1`. The current Lift contract is the only place that should still speak dotted paths.

### Risk 5: path normalization behaves differently across platforms

Mitigation: `RepoPath` is a platform-independent logical path type, not an OS path wrapper.

### Risk 6: stable IDs churn too much

Mitigation: freeze the envelope format in seam 0, but let each producing seam define and test its own lemma.

### Risk 7: schema and Rust types drift apart

Mitigation: hand-authored schema plus parity tests in CI.

### Risk 8: temptation to move seam 0 into a separate crate too early

Mitigation: keep it inside `crates/lift/src/kernel` until at least two independent workspace crates need the same public API.

## 13. Specific implementation items

### A. Rust modules

- `path.rs`
- `json_pointer.rs`
- `id.rs`
- `span.rs`
- `locator.rs`
- `fingerprint.rs`
- `canonical_json.rs`
- `diagnostic.rs`
- `error.rs`
- `schema.rs`

### B. Schema

- add `schemas/kernel/primitives.v1.json`
- add fixture JSON samples under `fixtures/kernel/`

### C. Tests

- constructor validation tests
- serde round-trip tests
- schema validation tests
- canonical JSON golden tests
- stable ID determinism tests
- diagnostic ordering tests
- cross-platform path fixture tests

### D. Compat hook

- implement `JsonPointer` to dotted Lift path translation in `app::score::compat_v1`
- golden-test it against current v1 field names

### E. Docs

- seam 0 contract doc
- examples for every primitive
- how to add a new ID kind note
- how to bump kernel schema version note

## 14. Recommended decision set

These decisions are locked now:

- seam 0 stays internal to `crates/lift`
- JSON Pointer is the canonical internal field-path type
- canonical JSON hashing uses RFC 8785 JCS
- `RepoPath` is UTF-8-only in v1
- kernel owns byte spans only, not line and column display types
- public kernel JSON contracts live in one hand-authored Draft 2020-12 schema file
- dotted Lift field paths are compat-only, not engine-native
- stable ID envelope is fixed in seam 0; identity lemmas are defined by producing seams

That yields a seam 0 that is both small and durable.

The next useful artifact after this is seam 1 in the same format so the sessions implementing the crate can snap together cleanly.
