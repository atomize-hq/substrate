
<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-lift-autoplan-restore-20260417-133748.md -->

# substrate-lift seam 2 spec — repo substrate (reviewed against landed seam 0 + seam 1 + seam 2 Phase A)

## 0. Ground truth from the landed crate

This spec is intentionally anchored to the crate as it exists today, not to the earlier idealized design.

Observed state in the landed crate:

- `src/kernel/**` is real, tested, schema-backed, and publicly re-exported from `lib.rs`.
- `src/pack/**` is real code and now compiles profiles, topology packs, score models, query packs, rule packs, recipe packs, and deterministic bundle resolution into `CompiledPackSet`.
- `src/repo/**` is real, tested, schema-backed, and crate-private. `src/repo/mod.rs` now re-exports the landed Phase-A seam surface.
- `lib.rs` still exposes `repo` only as `pub(crate) mod repo;`, so seam 2 is not public API yet.
- `src/app/runtime.rs` currently stops at `ProfileBootstrap { bundle: CompiledPackSet }`; there is no repo-facing runtime loop yet.
- `kernel::RepoPath`, `FileId`, `Fingerprint`, `DiagnosticCode`, and `Severity` already exist and should be reused rather than redefined.
- `RepoRoot`, `RepoRootDetectionOptions`, `SnapshotRequest`, `SnapshotOptions`, `RepoSnapshot`, `Inventory`, `BlobStore`, `RepoDiagnostic`, and `SnapshotStats` are now landed and covered by targeted repo tests.
- `pack::CompiledAnalysisDefaults` already exists and currently contains:
  - `languages: BTreeSet<LanguageId>`
  - `follow_symlinks: bool`
  - `max_scope_depth: u8`
- there is **no** landed pack-level repo config surface yet for ignore globs, non-UTF8 policy, large-file policy, VCS selection, or snapshot mode.
- `Cargo.toml` already contains both `globset` and `walkdir`; it does **not** yet contain a git backend crate.
- `schemas/repo/snapshot_manifest.v1.json` is landed and embedded through `src/repo/schema.rs`.
- `fixtures/repo/**` plus `tests/repo_root.rs`, `tests/repo_snapshot.rs`, `tests/repo_ignore.rs`, `tests/repo_fingerprints.rs`, `tests/repo_purity.rs`, and `tests/repo_schema.rs` are landed and passing.
- there is still no `src/repo/diff.rs`, no `schemas/repo/diff_manifest.v1.json`, and no `tests/repo_diff.rs`.
- the compile-matrix test already asserts the crate still builds with `--no-default-features`.

Validated on this branch before updating this spec:

- `cargo test -p substrate-lift --test repo_root --test repo_snapshot --test repo_ignore --test repo_fingerprints --test repo_purity --test repo_schema -- --nocapture`
- `cargo test -p substrate-lift --test compile_matrix -- --nocapture`

That means seam 2 is now an **internal, filesystem-first, immutable snapshot seam** with Phase A landed, and the next work should stay cleanly downstream of `kernel` and cleanly upstream of `lang`, `topo`, `graph`, and app orchestration.

The key consequence is this:

> seam 2 should **not** make `repo` public yet, and it should **not** depend on `pack`, `lang`, `graph`, app code, or CLI code.

A second consequence, based on the current seam-1 and seam-2 Phase-A reality:

> seam 2 phase B should **not** require retroactive changes to `profile.v1.toml` or `CompiledAnalysisDefaults`.
> It should diff already-materialized `RepoSnapshot` values directly, and later runtime/orchestration code can keep any pack-to-snapshot option mapping outside `repo`.

---

## 1. Mission

Seam 2 owns the **repo substrate**.

It is responsible for:

- detecting a repository root from a starting path;
- materializing an immutable repo snapshot from a selected source;
- enumerating a deterministic inventory of included files;
- deriving repo-relative `RepoPath` values for those files;
- reading and storing blob bytes for later seams;
- computing content digests and deterministic snapshot fingerprints;
- applying intrinsic and caller-supplied ignore rules;
- surfacing typed repo errors and deterministic repo diagnostics;
- computing pure path-based diffs between two already-materialized snapshots.

It is **not** responsible for:

- parsing source languages;
- interpreting config semantics;
- classifying components/boundaries/docs/tests;
- query matching;
- fact/detector execution;
- Lift scoring;
- pack/profile loading;
- CLI argument parsing.

A useful rule:

> seam 2 ends at **immutable repository materialization**.
> It does not interpret the contents.

---

## 2. Why seam 2 is a separate seam

The landed seam 1 established another important pattern beyond seam 0:

- raw on-disk inputs are compiled once;
- runtime consumers operate on immutable compiled artifacts;
- later seams do not keep reaching back into live sources.

Seam 2 should apply the same idea to filesystem and revision state.

Later seams should consume `RepoSnapshot`, not raw directories or live files.

That gives the engine:

- deterministic inventory ordering;
- deterministic content digests;
- purity after snapshot construction;
- no accidental live re-read drift while analysis is running.

The most important design constraint for seam 2 is therefore:

> `RepoSnapshot` must be **fully materialized and immutable**.
> Once created, later seams must not read the live filesystem again through repo APIs.

That requirement is stronger than “convenient” — it is necessary if determinism is going to survive parallelism, retries, or source-tree changes during a run.

---

## 3. Boundary with existing code

### Existing seam-0 primitives seam 2 should reuse

Use directly from `kernel`:

- `RepoPath`
- `FileId`
- `Fingerprint`
- `DiagnosticCode`
- `Severity`
- `sha256_bytes`
- `sha256_canonical_json`

### Existing seam-0 primitives seam 2 should **not** force-fit

Do **not** use `Locator` as the primary repo-local diagnostic location contract.

Reason:

- seam 2 must report root-detection and filesystem-entry issues before a stable `RepoPath` always exists;
- root-detection failures may involve absolute or relative host paths, not repo-relative logical paths;
- non-UTF8 entries may be diagnosable even when they cannot become a `RepoPath`.

Like seam 1’s `PackLocation`, seam 2 should define its own lightweight repo-local location type and only attach `RepoPath` when one exists.

### Dependency direction

Seam 2 should depend only on:

- `kernel`
- `std`
- a small traversal utility if needed
- `globset` for caller-supplied ignore compilation

Seam 2 should **not** depend on:

- `pack`
- `lang`
- `graph`
- `topo`
- `facts`
- `derive`
- `app`
- `cli`
- `anyhow`

A future runtime/orchestration seam may map `CompiledAnalysisDefaults` into `SnapshotOptions`, but that translation must live outside `repo`.

---

## 4. Canonical phase map

Because the current crate now has landed Phase A, but still has no diff module, no git backend crate, and no landed repo-facing runtime loop, seam 2 should continue in **three** phases.

This section is canonical. Later sections should reference these phases, not restate a competing phase model.

### Phase A — filesystem-first snapshot substrate

Phase A landed:

- repo root detection;
- worktree snapshot materialization;
- deterministic inventory;
- immutable in-memory blob store;
- intrinsic `.git` exclusion;
- caller-supplied exclude globs;
- repo diagnostics;
- snapshot fingerprints;
- fixture manifests and tests.

Phase A does **not** land:

- git revision materialization;
- path-based diffing;
- symlink following;
- well-known cache/build/vendor exclude presets;
- `.gitignore` interpretation;
- pack-driven repo config.

### Phase B — pure diff over already-materialized snapshots

Phase B lands:

- pure path-based `RepoDiff`;
- diff fixture schema;
- diff fixtures and tests.

Phase B does **not** land:

- git revision materialization;
- new filesystem walking behavior;
- symlink-following behavior;
- rename detection.

### Phase C — expanded materialization semantics

Phase C lands:

- `SnapshotSource::GitRev { rev }`;
- `SymlinkPolicy::Follow`;
- a typed well-known exclude policy for common cache/build/vendor directories;
- optional provider abstraction if and only if a second backend now exists.

Examples for the Phase C well-known exclude policy:

- `target/`
- `node_modules/`
- `.venv/`
- `venv/`
- `__pycache__/`
- `dist/`
- `build/`

This keeps Phase A honest, Phase B pure, and Phase C as the only place where snapshot-construction semantics expand beyond the minimal worktree substrate.

---

## 5. Exact module shape by phase

```text
src/repo/
  mod.rs
  error.rs
  diagnostics.rs
  root.rs
  ignore.rs
  inventory.rs
  blob.rs
  snapshot.rs
  schema.rs
```

Phase B adds:

```text
src/repo/
  diff.rs
```

Phase C may add:

```text
src/repo/
  provider.rs
```

### Phase A `src/repo/mod.rs`

`src/repo/mod.rs` should re-export only the stable internal **Phase A** seam surface:

```rust
//! Internal repo substrate seam.

pub(crate) mod blob;
pub(crate) mod diagnostics;
pub(crate) mod error;
pub(crate) mod ignore;
pub(crate) mod inventory;
pub(crate) mod root;
pub(crate) mod schema;
pub(crate) mod snapshot;

pub(crate) use blob::{BlobRecord, BlobStore};
pub(crate) use diagnostics::{RepoDiagnostic, RepoLocation, RepoRelatedLocation};
pub(crate) use error::{RepoError, RepoResult};
pub(crate) use ignore::{
    CompiledIgnoreSet, LargeFilePolicy, NonUtf8PathPolicy, SnapshotOptions, SymlinkPolicy,
};
pub(crate) use inventory::{Inventory, InventoryEntry};
pub(crate) use root::{RepoRoot, RepoRootDetectionOptions, RootMarker};
pub(crate) use snapshot::{RepoSnapshot, SnapshotRequest, SnapshotSource, SnapshotStats};
```

Phase B extends `mod.rs` with:

```rust
pub(crate) mod diff;
pub(crate) use diff::{build_diff, DiffEntry, DiffKind, RepoDiff};
```

Phase C may extend `mod.rs` with:

```rust
pub(crate) mod provider;
pub(crate) use provider::{FsRepoProvider, RepoProvider};
```

### Hard rules

- `repo` remains `pub(crate)` in seam 2.
- No new public crate API should be added yet.
- No other seam should shell out to git or walk the filesystem directly.
- Later seams should consume `RepoSnapshot`, not raw `PathBuf` trees.

---

## 6. Allowed new dependencies

The current crate already has `globset`, which seam 2 should reuse for caller-supplied exclude globs.

Phase A is allowed to add **at most one** new runtime dependency for deterministic directory walking if the implementation benefits from it:

```toml
walkdir = "2"
```

Phase A should **not** add:

- `ignore`
- `git2`
- `gix`
- `anyhow`

Reason:

- `.gitignore` semantics are intentionally out of scope in phase A;
- git revision materialization is deferred to phase B;
- the seam should keep typed error contracts.

If the implementation chooses not to add `walkdir`, a small internal recursive walker is acceptable, but traversal order must still be deterministic.

---

## 7. Exact internal Rust contracts

## 7.1 Repo root detection

```rust
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct RootMarker(String);
// Canonical form:
// - non-empty
// - exact entry name only
// - no path separators
// - UTF-8 only
//
// Examples:
//   ".git"
//   "work-lift.toml"
//   "Cargo.toml"

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoRootDetectionOptions {
    pub markers: std::collections::BTreeSet<RootMarker>,
    pub ceiling_dir: Option<std::path::PathBuf>,
}
// Default:
//   markers = { RootMarker::parse(".git")? }
//   ceiling_dir = None

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoRoot {
    absolute_path: std::path::PathBuf,
}
```

Required constructors/helpers:

```rust
impl RootMarker {
    pub(crate) fn parse(input: &str) -> RepoResult<Self>;
    pub(crate) fn as_str(&self) -> &str;
}

impl RepoRootDetectionOptions {
    pub(crate) fn git_default() -> Self;
}

impl RepoRoot {
    pub(crate) fn from_dir(path: &std::path::Path) -> RepoResult<Self>;
    pub(crate) fn as_path(&self) -> &std::path::Path;
    pub(crate) fn display(&self) -> String;
}
```

### Root-detection semantics

- input start path may be a file or directory;
- if file, detection starts from the parent directory;
- search ascends toward filesystem root until:
  - a marker match is found, or
  - the optional `ceiling_dir` is reached, or
  - the filesystem root is reached;
- the **nearest** matching ancestor wins;
- marker order must not affect the chosen root;
- a marker match is “an entry with the exact marker name exists in that directory”;
- `.git` must match as either a file or directory;
- detected root is stored as an absolute directory path;
- the absolute root path is for I/O only and must not enter snapshot fingerprints.

### Why `RootMarker` exists

Do not use bare strings directly across the seam.

A dedicated `RootMarker` type gives seam 2 a place to validate “exact entry name, not a glob, not a path fragment,” which avoids leaking caller sloppiness into root detection.

---

## 7.2 Snapshot requests and options

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotSource {
    Worktree,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum SymlinkPolicy {
    Skip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum NonUtf8PathPolicy {
    Error,
    Skip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum LargeFilePolicy {
    Error,
    Skip,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotOptions {
    pub symlink_policy: SymlinkPolicy,
    pub exclude_globs: Vec<String>,
    pub non_utf8_path_policy: NonUtf8PathPolicy,
    pub max_file_bytes: Option<u64>,
    pub large_file_policy: LargeFilePolicy,
}
// Default:
//   symlink_policy = SymlinkPolicy::Skip
//   exclude_globs = Vec::new()
//   non_utf8_path_policy = NonUtf8PathPolicy::Error
//   max_file_bytes = None
//   large_file_policy = LargeFilePolicy::Error

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotRequest {
    pub root: RepoRoot,
    pub source: SnapshotSource,
    pub options: SnapshotOptions,
}
```

### Reality-based constraint

Phase A should **not** add new pack/profile schema fields just to support these options.

Instead:

- seam 2 accepts `SnapshotOptions` directly;
- later runtime/orchestration code may translate currently-landed `CompiledAnalysisDefaults.follow_symlinks` into:
  - `SymlinkPolicy::Skip` when `false`
  - a later Phase C follow policy when `true`

This avoids forcing seam 1 churn before seam 2 exists.

### Phase C extension shape

Phase C may extend these contracts with:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotSource {
    Worktree,
    GitRev { rev: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum SymlinkPolicy {
    Skip,
    Follow,
}
```

Phase C is also the home for a typed well-known exclude policy. The exact enum naming can be chosen in Phase C, but the semantics should cover the common cache/build/vendor directories listed in the canonical phase map.

---

## 7.3 Repo diagnostics

Seam 2 should follow the seam-1 diagnostics pattern: structured, sortable, and machine-readable.

```rust
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub(crate) struct RepoLocation {
    pub display_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_path: Option<crate::kernel::RepoPath>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub(crate) struct RepoRelatedLocation {
    pub location: RepoLocation,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct RepoDiagnostic {
    pub code: crate::kernel::DiagnosticCode,
    pub severity: crate::kernel::Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<RepoLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RepoRelatedLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}
```

Required ordering rules:

- diagnostics are sortable and emitted deterministically;
- sort key:
  1. severity rank: `error`, `warning`, `info`
  2. subject display path
  3. subject repo path
  4. diagnostic code
  5. message

Representative codes:

- `repo.root.start_path_missing`
- `repo.root.not_found`
- `repo.snapshot.non_utf8_path`
- `repo.snapshot.symlink_skipped`
- `repo.snapshot.unsupported_file_kind`
- `repo.snapshot.file_too_large`

### Important distinction from seam 0

`RepoLocation.display_path` is **not** stable across hosts and is **not** fingerprint input.

It exists for diagnostics only.

---

## 7.4 Inventory and blobs

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InventoryEntry {
    pub file_id: crate::kernel::FileId,
    pub path: crate::kernel::RepoPath,
    pub blob_fingerprint: crate::kernel::Fingerprint,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Inventory {
    pub entries: std::collections::BTreeMap<crate::kernel::RepoPath, InventoryEntry>,
    pub fingerprint: crate::kernel::Fingerprint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BlobRecord {
    pub file_id: crate::kernel::FileId,
    pub path: crate::kernel::RepoPath,
    pub blob_fingerprint: crate::kernel::Fingerprint,
    pub size_bytes: u64,
    bytes: std::sync::Arc<[u8]>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BlobStore {
    by_path: std::collections::BTreeMap<crate::kernel::RepoPath, BlobRecord>,
}
```

Required methods:

```rust
impl Inventory {
    pub(crate) fn get(&self, path: &crate::kernel::RepoPath) -> Option<&InventoryEntry>;
    pub(crate) fn len(&self) -> usize;
    pub(crate) fn is_empty(&self) -> bool;
    pub(crate) fn iter(&self) -> impl Iterator<Item = &InventoryEntry>;
}

impl BlobStore {
    pub(crate) fn contains(&self, path: &crate::kernel::RepoPath) -> bool;
    pub(crate) fn get(&self, path: &crate::kernel::RepoPath) -> Option<&BlobRecord>;
    pub(crate) fn read_bytes(&self, path: &crate::kernel::RepoPath) -> RepoResult<&[u8]>;
    pub(crate) fn len(&self) -> usize;
}
```

### Inventory membership rule

Inventory contains **only regular files** that survive all filtering/policy checks.

It does **not** contain:

- directories
- symlinks when `SymlinkPolicy::Skip`
- unsupported file kinds
- excluded `.git/**`
- files excluded by caller-supplied globs
- files rejected by non-UTF8 or large-file policy

### File identity lemma

Seam 0 required each producing seam to document stable-ID lemmas.

For seam 2:

```text
FileId lemma:
repo\0file\0v1\0<repo-path>
```

`file_id` must therefore be independent of:

- absolute root path
- blob digest
- mtime
- inode
- traversal order

That makes `FileId` stable for a logical path across snapshots, while `blob_fingerprint` captures content changes.

---

## 7.5 Repo snapshots

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotStats {
    pub file_count: u64,
    pub total_bytes: u64,
    pub skipped_by_ignore: u64,
    pub skipped_symlinks: u64,
    pub skipped_non_utf8_paths: u64,
    pub skipped_large_files: u64,
    pub skipped_unsupported_file_kinds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoSnapshot {
    pub source: SnapshotSource,
    pub inventory: Inventory,
    pub blob_store: BlobStore,
    pub fingerprint: crate::kernel::Fingerprint,
    pub diagnostics: Vec<RepoDiagnostic>,
    pub stats: SnapshotStats,

    root: RepoRoot,
}
```

Required methods:

```rust
impl RepoSnapshot {
    pub(crate) fn root(&self) -> &RepoRoot;
    pub(crate) fn entry(&self, path: &crate::kernel::RepoPath) -> Option<&InventoryEntry>;
    pub(crate) fn read_bytes(&self, path: &crate::kernel::RepoPath) -> RepoResult<&[u8]>;
}
```

### Critical snapshot purity invariant

Once `RepoSnapshot` is constructed:

- reading bytes must come from `blob_store`, never from the live filesystem;
- inventory iteration must not touch the filesystem;
- deleting or mutating the source tree after snapshot construction must not change:
  - `inventory`
  - `blob_store`
  - `fingerprint`
  - `stats`
  - `diagnostics`

This is the seam-2 equivalent of seam 1’s “compiled pack set stays pure after source cleanup.”

### Snapshot fingerprint semantics

In phase A:

```text
RepoSnapshot.fingerprint == Inventory.fingerprint
```

The fingerprint input document must be:

```json
{
  "version": 1,
  "files": [
    {
      "path": "src/lib.rs",
      "blob_fingerprint": "sha256:...",
      "size_bytes": 1234
    }
  ]
}
```

Rules:

- files are sorted ascending by `RepoPath`;
- absolute root path is excluded;
- diagnostics are excluded;
- stats are excluded;
- file IDs are excluded;
- timestamps, inodes, and permissions are excluded.

That keeps the snapshot fingerprint purely content-and-path based.

---

## 7.6 Diffs (Phase B)

Phase B adds a pure diff builder over already-materialized snapshots.

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum DiffKind {
    Added,
    Modified,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiffEntry {
    pub path: crate::kernel::RepoPath,
    pub kind: DiffKind,
    pub before: Option<InventoryEntry>,
    pub after: Option<InventoryEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoDiff {
    pub base_fingerprint: crate::kernel::Fingerprint,
    pub head_fingerprint: crate::kernel::Fingerprint,
    pub entries: Vec<DiffEntry>,
    pub fingerprint: crate::kernel::Fingerprint,
}
```

Required function:

```rust
pub(crate) fn build_diff(base: &RepoSnapshot, head: &RepoSnapshot) -> RepoDiff;
```

### Diff semantics

Diff is path-based only.

For each path in the union of base and head paths:

- only in head => `Added`
- only in base => `Removed`
- in both and `blob_fingerprint` differs => `Modified`
- in both and `blob_fingerprint` matches => omitted

### Important v1 rule

There is **no rename detection** in seam 2 v1.

A rename appears as:

- one `Removed` entry for the old path
- one `Added` entry for the new path

This is deliberate. Rename heuristics are notoriously unstable across backends and thresholds. Path-based add/remove/modify is simpler and deterministic.

### Diff fingerprint semantics

```json
{
  "version": 1,
  "base_fingerprint": "sha256:...",
  "head_fingerprint": "sha256:...",
  "entries": [
    {
      "path": "src/lib.rs",
      "kind": "modified",
      "before": "sha256:...",
      "after": "sha256:..."
    }
  ]
}
```

Entries must be sorted ascending by path.

---

## 7.7 Provider contract (Phase C)

The provider abstraction is deferred until Phase C.

Reason:

- Phase A has only one concrete backend, the filesystem worktree;
- Phase B is pure diffing over already-built snapshots;
- introducing the provider trait before a second backend exists is premature abstraction.

If Phase C lands a second backend, the seam may add:

```rust
pub(crate) trait RepoProvider {
    fn detect_root(
        &self,
        start: &std::path::Path,
        options: &RepoRootDetectionOptions,
    ) -> RepoResult<RepoRoot>;

    fn materialize(
        &self,
        request: &SnapshotRequest,
    ) -> RepoResult<RepoSnapshot>;
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FsRepoProvider;
```

Hard rule:

- do not add `RepoProvider` in Phase A just to reserve shape.

---

## 7.8 Ignore compilation

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledIgnoreSet {
    // internal compiled matcher
}
```

Required constructor:

```rust
impl CompiledIgnoreSet {
    pub(crate) fn compile(exclude_globs: &[String]) -> RepoResult<Self>;
    pub(crate) fn is_ignored(&self, repo_path: &crate::kernel::RepoPath, is_dir: bool) -> bool;
}
```

### Ignore semantics

Phase A ignore behavior is intentionally narrow:

- intrinsic exclude:
  - `.git`
  - anything under `.git/`
- caller-supplied exclude globs compiled with `globset`
- matching is evaluated against repo-relative POSIX `RepoPath` strings
- if a directory path matches an exclude, its subtree is skipped
- no automatic exclusion of:
  - `target`
  - `node_modules`
  - `.venv`
  - `dist`
- no `.gitignore`, `.ignore`, or global git-excludes interpretation

That last point is important:

> seam 2 phase A models “the materialized worktree under a repo root,” not “whatever git would currently treat as tracked.”

---

## 8. Exact error surface

```rust
pub(crate) type RepoResult<T> = Result<T, RepoError>;

#[derive(Debug, thiserror::Error, Clone, Eq, PartialEq)]
pub(crate) enum RepoError {
    #[error("repo start path does not exist")]
    StartPathNotFound { path: String },

    #[error("repo root could not be detected")]
    RootNotFound {
        start_path: String,
        markers: Vec<String>,
    },

    #[error("repo root is not a directory")]
    RootNotDirectory { path: String },

    #[error("repo I/O failure")]
    Io {
        op: &'static str,
        path: String,
        reason: String,
    },

    #[error("invalid root marker")]
    InvalidRootMarker { input: String },

    #[error("invalid repo-relative path derived from filesystem entry")]
    InvalidRepoPath {
        display_path: String,
        reason: String,
    },

    #[error("encountered non-utf8 filesystem path")]
    NonUtf8Path { display_path: String },

    #[error("ignore glob compile failure")]
    IgnoreGlobCompile { pattern: String, reason: String },

    #[error("snapshot source is not implemented")]
    UnsupportedSnapshotSource { source: &'static str },

    #[error("snapshot option is not implemented")]
    UnsupportedSnapshotOption { option: &'static str },

    #[error("file exceeds configured max_file_bytes")]
    FileTooLarge {
        display_path: String,
        size_bytes: u64,
        max_file_bytes: u64,
    },

    #[error("blob not present in snapshot")]
    MissingBlob { path: crate::kernel::RepoPath },
}
```

### Notes

- typed errors only; no `anyhow`;
- filesystem path strings in errors are display-only and may be host-specific;
- `RepoError` is for hard failure;
- `RepoDiagnostic` is for successful snapshot warnings/info.

---

## 9. JSON schema inventory

Unlike seam 1, seam 2 does **not** need user-authored runtime config schemas in its first landing.

The only JSON schemas seam 2 should add initially are **fixture manifest schemas** that lock deterministic test expectations.

### Phase A required schema

```text
schemas/repo/snapshot_manifest.v1.json
```

### Phase B reserved schema

```text
schemas/repo/diff_manifest.v1.json
```

`src/repo/schema.rs` should follow the same embed-and-access pattern already used by `src/kernel/schema.rs` and `src/pack/schema.rs`.

### Phase A manifest shape

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.substrate.dev/lift/repo/snapshot_manifest.v1.json",
  "title": "Lift Repo Snapshot Manifest v1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "version",
    "case",
    "source_kind",
    "options",
    "files",
    "snapshot_fingerprint",
    "stats"
  ],
  "properties": {
    "version": { "const": 1 },
    "case": { "type": "string", "minLength": 1 },
    "source_kind": { "type": "string", "enum": ["worktree"] },
    "options": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "symlink_policy",
        "exclude_globs",
        "non_utf8_path_policy",
        "max_file_bytes",
        "large_file_policy"
      ],
      "properties": {
        "symlink_policy": { "type": "string", "enum": ["skip", "follow"] },
        "exclude_globs": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 },
          "uniqueItems": true
        },
        "non_utf8_path_policy": { "type": "string", "enum": ["error", "skip"] },
        "max_file_bytes": { "type": ["integer", "null"], "minimum": 0 },
        "large_file_policy": { "type": "string", "enum": ["error", "skip"] }
      }
    },
    "files": {
      "type": "array",
      "items": { "$ref": "#/$defs/file_record" }
    },
    "snapshot_fingerprint": {
      "$ref": "../kernel/primitives.v1.json#/$defs/fingerprint"
    },
    "stats": { "$ref": "#/$defs/stats" }
  },
  "$defs": {
    "file_record": {
      "type": "object",
      "additionalProperties": false,
      "required": ["path", "file_id", "blob_fingerprint", "size_bytes"],
      "properties": {
        "path": {
          "$ref": "../kernel/primitives.v1.json#/$defs/repo_path"
        },
        "file_id": {
          "$ref": "../kernel/primitives.v1.json#/$defs/stable_id"
        },
        "blob_fingerprint": {
          "$ref": "../kernel/primitives.v1.json#/$defs/fingerprint"
        },
        "size_bytes": {
          "type": "integer",
          "minimum": 0
        }
      }
    },
    "stats": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "file_count",
        "total_bytes",
        "skipped_by_ignore",
        "skipped_symlinks",
        "skipped_non_utf8_paths",
        "skipped_large_files",
        "skipped_unsupported_file_kinds"
      ],
      "properties": {
        "file_count": { "type": "integer", "minimum": 0 },
        "total_bytes": { "type": "integer", "minimum": 0 },
        "skipped_by_ignore": { "type": "integer", "minimum": 0 },
        "skipped_symlinks": { "type": "integer", "minimum": 0 },
        "skipped_non_utf8_paths": { "type": "integer", "minimum": 0 },
        "skipped_large_files": { "type": "integer", "minimum": 0 },
        "skipped_unsupported_file_kinds": { "type": "integer", "minimum": 0 }
      }
    }
  }
}
```

### Runtime invariants not fully expressible in schema

- `files` must be sorted ascending by `path`
- `file_id` must be a `file:` stable ID, not just any stable ID
- `snapshot_fingerprint` must equal the canonical fingerprint of the sorted file list
- `stats.file_count == files.len()`
- `stats.total_bytes == sum(files[].size_bytes)`

### Phase B diff manifest shape

Phase B should add `schemas/repo/diff_manifest.v1.json` with:

- `version`
- `case`
- `base_fingerprint`
- `head_fingerprint`
- `entries`
- `diff_fingerprint`

where each entry includes:

- `path`
- `kind`
- `before_blob_fingerprint | null`
- `after_blob_fingerprint | null`

---

## 10. Determinism and invariants

These must be treated as normative.

### 10.1 Repo-root invariants

1. `RepoRoot` is always an existing absolute directory path.
2. Root-marker order never changes the chosen root.
3. The nearest matching ancestor always wins.
4. Root absolute path never enters snapshot fingerprints.

### 10.2 Snapshot invariants

1. `RepoSnapshot` is immutable after construction.
2. Snapshot inventory contains only regular files.
3. Snapshot inventory paths are valid kernel `RepoPath` values.
4. Inventory keys are unique and sorted.
5. Blob reads never hit the live filesystem after snapshot creation.
6. Snapshot fingerprint depends only on sorted `(path, blob_fingerprint, size_bytes)`.
7. Snapshot fingerprint does not depend on:
   - absolute root path
   - traversal order
   - mtime
   - inode
   - permissions
   - host path separators
8. `.git/**` is always excluded in phase A.
9. `target`, `node_modules`, `.venv`, and similar directories are **not** implicitly excluded by seam 2.
10. Ignore matching uses repo-relative POSIX paths only.
11. Non-UTF8 handling follows the configured policy and is never silently normalized into an invalid `RepoPath`.
12. `BlobStore.len() == Inventory.len()`.

### 10.3 Diff invariants

1. Diff is computed only from already-materialized snapshots.
2. Diff omits unchanged files.
3. Diff entries are sorted by path.
4. Diff kind is only one of:
   - `Added`
   - `Modified`
   - `Removed`
5. Rename detection does not exist in v1.

### 10.4 Boundary invariants

1. Seam 2 does not depend on `pack`.
2. Seam 2 does not depend on any language adapters.
3. Seam 2 does not know about components, boundaries, docs, tests, or CI classes.
4. Seam 2 does not know about Lift scoring or app result shapes.
5. Seam 2 contains all repo-walking logic; later seams must not re-walk the filesystem.

---

## 11. Acceptance criteria

Seam 2 is done only when all of these are true.

## 11.1 Phase A acceptance criteria

### Contract completeness

- all seam-2 runtime types above exist under `src/repo/**`
- `src/repo/mod.rs` re-exports the stable internal seam surface
- no new public crate API is added in `lib.rs`

### Root detection

- starting from a nested file under a fixture repo finds the nearest root marker
- starting from a nested directory under a fixture repo finds the nearest root marker
- `ceiling_dir` can stop ascent before a higher matching ancestor
- marker order does not change the chosen root
- `.git` marker works when `.git` is a directory
- `.git` marker works when `.git` is a file
- missing start path returns `RepoError::StartPathNotFound`
- no marker match returns `RepoError::RootNotFound`

### Snapshot materialization

- worktree snapshot materializes deterministic inventory and blob bytes
- `.git/**` is excluded even without caller globs
- files excluded by caller globs are absent from inventory and blob store
- directory-path glob matches skip the full subtree
- ignore matching uses repo-relative POSIX paths only
- `target`, `node_modules`, `.venv`, `venv`, `__pycache__`, `dist`, and `build` are **not** implicitly excluded in Phase A without caller globs
- `RepoPath` values in inventory are valid and normalized
- `Inventory.len() == BlobStore.len() == SnapshotStats.file_count`

### Purity

- after snapshot creation, mutating or deleting source files does not change:
  - `RepoSnapshot::entry`
  - `RepoSnapshot::read_bytes`
  - `RepoSnapshot::fingerprint`

### Determinism

- copying the same fixture tree to two different absolute temp roots yields identical snapshot fingerprints
- walk order in the underlying filesystem does not affect inventory ordering or fingerprint
- changing only one file changes only:
  - that file’s blob fingerprint
  - the snapshot fingerprint
- diagnostics sort deterministically

### Errors and policies

- non-UTF8 and large-file policies behave exactly as configured
- invalid ignore globs return `RepoError::IgnoreGlobCompile`
- skipped symlink / non-UTF8 / large-file / unsupported-kind diagnostics sort deterministically when emitted
- `BlobStore::read_bytes` for a missing path returns `RepoError::MissingBlob`

### Schema and fixtures

- `schemas/repo/snapshot_manifest.v1.json` exists and is embedded through `src/repo/schema.rs`
- fixture snapshot manifests validate against the embedded schema
- manifest-based tests verify snapshot fingerprint and inventory records
- manifest-based tests verify runtime invariants that JSON Schema cannot express:
  - file ordering
  - `stats.file_count == files.len()`
  - `stats.total_bytes == sum(files[].size_bytes)`
  - snapshot fingerprint matches the canonical sorted file list

### Build posture

- `cargo check -p substrate-lift --no-default-features` still passes
- seam 2 adds no dependency on CLI-only code

## 11.2 Phase B acceptance criteria

- `build_diff(base, head)` returns sorted add/remove/modify entries only
- unchanged files are omitted
- rename shows up as remove + add
- `schemas/repo/diff_manifest.v1.json` exists and validates diff fixtures
- diff fingerprints are identical across repeated runs for the same snapshot pair

## 11.3 Phase C acceptance criteria

- `SnapshotSource::GitRev { .. }` materializes deterministic snapshots for fixtures
- `SymlinkPolicy::Follow` is bounded, explicitly tested, and cannot escape repo constraints
- if `RepoProvider` is introduced, it lands only because more than one backend now exists
- the Phase C well-known exclude policy is typed, explicit, and tested against the canonical directory examples from the phase map
- Phase C snapshot materialization preserves the same purity and determinism guarantees as Phase A
- Phase C adds no silent fallback from `GitRev` materialization to worktree semantics

---

## 12. Falsification questions

These are the PR-review questions that can invalidate the seam.

1. Can the same fixture tree produce different snapshot fingerprints when copied to different absolute roots?
2. Can `RepoSnapshot::read_bytes` change after the source file changes on disk?
3. Can anything under `.git/` leak into inventory?
4. Can filesystem walk order change the emitted inventory order or fingerprint?
5. Can a file with the same repo path but different content keep the same blob fingerprint?
6. Can `FileId` depend on blob digest, absolute root path, mtime, or traversal order?
7. Can a later seam still read files directly from disk instead of from `RepoSnapshot`?
8. Can seam 2 silently honor `.gitignore` or global git ignores in phase A?
9. Can `target/` or `node_modules/` be excluded without an explicit caller glob or later policy layer?
10. Can a non-UTF8 path be converted into a bogus `RepoPath` instead of erroring or being skipped per policy?
11. Can a symlink escape the repo root in phase A without an explicit, tested policy?
12. Can `SnapshotSource::GitRev` appear to work in one environment but fall back to worktree semantics in another?
13. Can the diff builder emit rename semantics or heuristic similarity-based behavior?
14. Can seam 2 require `pack`, `lang`, or app types to compile?
15. Can the snapshot fingerprint depend on permission bits, inode numbers, or timestamps?
16. Can diagnostics or stats alter the snapshot fingerprint?
17. Can `BlobStore.len()` differ from `Inventory.len()` in a successful snapshot?
18. Can root-marker ordering change the detected root?

If any answer is “yes,” seam 2 is not clean enough.

---

## 13. In scope / out of scope

## 13.1 In scope for phase A

- root detection from a start path
- exact-name marker matching
- worktree snapshot materialization
- immutable blob store
- deterministic inventory ordering
- kernel `RepoPath` derivation
- content digests
- snapshot fingerprinting
- intrinsic `.git` exclusion
- caller-supplied exclude globs
- typed repo errors
- repo diagnostics
- fixture snapshot manifests and tests

## 13.2 In scope for phase B

- pure path-based diffing between snapshots
- diff fixture manifests and tests

## 13.3 In scope for phase C

- git revision materialization
- symlink-following semantics
- typed well-known exclude policy for common cache/build/vendor directories
- provider abstraction only if a second backend now exists

## 13.4 Out of scope for Phase A

- `.gitignore`, `.ignore`, or global git-exclude interpretation
- implicit policy-driven excludes like `target`, `node_modules`, `.venv`, `venv`, `__pycache__`, `dist`, `build`
- git revision materialization
- path-based diffing
- symlink following
- provider abstraction

## 13.5 Out of scope for seam 2 entirely in the current plan

- `.gitignore`, `.ignore`, or global git-exclude interpretation
- component or boundary classification
- docs/tests/CI classification
- language detection or parsing
- query execution
- Lift scoring
- rename detection
- permission-bit or executable-bit tracking
- line endings or text encoding inference
- mmap or on-disk blob caching
- public CLI/runtime integration
- pack schema changes for repo options
- sparse include-path snapshots
- live incremental FS watching

One explicit choice worth locking:

> seam 2 does **not** attempt to model “what git considers tracked.”
> It models “a deterministic materialized file tree under a detected repo root.”

---

## 14. Risks and mitigations

### Risk 1: memory growth from fully materialized snapshots

Because purity requires storing blob bytes, large repos can consume memory quickly.

Mitigation:

- phase A includes `max_file_bytes` and `LargeFilePolicy`;
- Phase A records `file_count` and `total_bytes` so whole-snapshot cost is visible from day one;
- hard limits on total snapshot bytes or file count are deferred until real Phase A usage proves they are needed;
- scope is explicit: immutable materialization first, optimization later;
- future work can add chunked or spilled blob storage without changing the snapshot contract.

### Risk 2: lack of `.gitignore` support makes worktree snapshots noisy

Build artifacts may appear unless explicitly excluded.

Mitigation:

- make the non-support explicit in the seam;
- keep `.git` as the only intrinsic exclude;
- allow caller-supplied exclude globs now;
- defer tracked-revision snapshots to phase C;
- defer well-known cache/build/vendor excludes to Phase C.

### Risk 3: symlink following is tricky and easy to make unsafe

Symlink loops or escapes can break boundedness.

Mitigation:

- phase A defaults to `SymlinkPolicy::Skip`;
- only add `Follow` in Phase C with explicit in-repo-bounds tests.

### Risk 4: non-UTF8 paths collide with seam-0 `RepoPath`

Mitigation:

- make non-UTF8 a first-class policy with `Error` / `Skip`;
- never silently coerce to invalid `RepoPath`.

### Risk 5: backend details leak upward

If later apps start depending on provider quirks, the seam becomes brittle.

Mitigation:

- keep downstream consumers on `RepoSnapshot` only;
- keep provider/backend details below the seam;
- do not introduce the provider abstraction until Phase C proves a second backend exists.

### Risk 6: git integration grows too early

Pulling in `git2`/`gix` before the phase-A contracts are stable will slow seam 2 down.

Mitigation:

- phase A is filesystem-first;
- Phase C adds git materialization only after snapshot contracts are stable.

### Risk 7: the seam is correct but does more work than needed

Snapshotting can stay functionally correct while still doing redundant sorting, hashing, or copying.

Mitigation:

- Phase A should materialize in one pass over the selected tree;
- inventory ordering should be produced with one deterministic sort boundary, not repeated resorting across layers;
- once the snapshot exists, no live re-read is allowed;
- benchmark thresholds are deferred until later hardening after the seam exists.

---

## 15. Specific implementation items

## 15.1 Rust modules

Phase A landed:

- `src/repo/error.rs`
- `src/repo/diagnostics.rs`
- `src/repo/root.rs`
- `src/repo/ignore.rs`
- `src/repo/inventory.rs`
- `src/repo/blob.rs`
- `src/repo/snapshot.rs`
- `src/repo/schema.rs`

`src/repo/mod.rs` is now the real seam re-export surface.

Phase B adds:

- `src/repo/diff.rs`

Phase C may add:

- `src/repo/provider.rs`

## 15.2 Schemas

Phase A landed:

- `schemas/repo/snapshot_manifest.v1.json`

Phase B adds:

- `schemas/repo/diff_manifest.v1.json`

`src/repo/schema.rs` should continue embedding schema constants in the same style already used by seam 0 and seam 1.

## 15.3 Fixtures

Phase A landed baseline:

```text
fixtures/repo/
  README.md
  valid/
    manifest_minimal.json
  invalid/
    manifest_bad_repo_path.json
    manifest_missing_stats.json
  trees/
    basic_worktree/
```

The landed Phase-A fixture plus temp-tree test helpers already exercise the important content cases:

- nearest `.git` marker wins
- file-start and dir-start root detection
- `ceiling_dir` stops ascent
- marker order does not change root selection
- `.git` marker as directory
- `.git` marker as file
- snapshot ignores `.git`
- explicit glob exclusion
- directory-match subtree skip semantics
- repo-relative POSIX path matching semantics
- no implicit exclusion of `target` / `node_modules` / `.venv` / `venv` / `__pycache__` / `dist` / `build`
- same tree under two different temp roots -> same fingerprint
- source tree deleted after snapshot -> snapshot still readable
- non-UTF8 path policy behavior (platform-guarded if necessary)
- large-file policy behavior

Phase B adds:

- `fixtures/repo/diff/**`

Phase C adds:

- `fixtures/repo/git_rev/**`
- `fixtures/repo/symlink_follow/**`
- `fixtures/repo/well_known_excludes/**`

## 15.4 Tests

Phase A landed tests analogous to the kernel/pack pattern:

- `tests/repo_root.rs`
- `tests/repo_snapshot.rs`
- `tests/repo_ignore.rs`
- `tests/repo_fingerprints.rs`
- `tests/repo_purity.rs`
- `tests/repo_schema.rs`

Phase B adds:

- `tests/repo_diff.rs`

Phase C adds:

- `tests/repo_git_rev.rs`
- `tests/repo_symlink_policy.rs`
- `tests/repo_materialization_policies.rs`

### Important test posture

Like the existing pack tests, these can include the internal seam directly:

```rust
#[path = "../src/repo/mod.rs"]
mod repo;
```

No public crate exports are required just to test seam 2.

## 15.5 README / housekeeping cleanup

With seam 2 Phase A landed and Phase B next, update:

- top-level `README.md` seam breakdown text to mark repo substrate as landed-initially
- `fixtures/README.md` to mention `fixtures/repo/`
- `schemas/README.md` to mention `schemas/repo/snapshot_manifest.v1.json`

---

## 15.6 Phase A execution plan, eng-review locked

This section is the implementation-grade companion to the earlier contract sections.

The earlier sections answer "what Phase A is." This section answers "how Phase A lands without inventing new decisions mid-implementation."

### 15.6.a Step 0, scope challenge

Phase A is the first real repo slice, so it needs the same scope discipline as the eng review, not just good contracts.

What already exists and must be reused:

| Sub-problem | Existing code | Phase A decision |
|---|---|---|
| crate-private seam boundary | `src/lib.rs`, current `pub(crate) mod repo;` | keep `repo` crate-private, do not promote public API yet |
| stable repo-relative path validation | `src/kernel/path.rs` | derive every inventory path through `RepoPath`, do not add a second path normalization layer |
| typed stable IDs | `src/kernel/id.rs` | derive `FileId` from the documented path lemma, do not add repo-local ad hoc IDs |
| content hashing and canonical fingerprinting | `src/kernel/fingerprint.rs` | reuse `sha256_bytes` and `sha256_canonical_json` directly |
| typed diagnostics and deterministic ordering pattern | `src/kernel/diagnostic.rs`, `src/pack/diagnostics.rs` | mirror the same sortable diagnostic style for repo failures |
| schema embedding pattern | `src/pack/schema.rs`, `tests/pack_schema.rs`, `tests/kernel_identity_schema.rs` | mirror the same embed-on-disk plus disk-vs-embedded validation pattern |
| no-default-features build posture | `tests/compile_matrix.rs` | Phase A must preserve `cargo check -p substrate-lift --no-default-features` |
| downstream runtime boundary | `src/app/runtime.rs` | keep runtime bootstrap out of Phase A, repo stops at immutable snapshot production |

Scope decision for the Phase A implementation PR:

- keep Phase A filesystem-first and worktree-only
- keep Phase A internal to `src/repo/**`
- take the already-allowed `walkdir = "2"` dependency instead of writing a custom recursive walker
- do not add `SnapshotSource::GitRev`, `RepoDiff`, `RepoProvider`, or runtime wiring
- do not add pack/profile schema changes just to thread repo options through seam 1
- do not add well-known cache/build/vendor excludes yet

Complexity check:

- Phase A touches more than 8 files, but that is expected and acceptable after reduction because most of the work is seam contracts, schema/fixture inventory, and deterministic tests
- the real reduction already happened in the phase map: git materialization, diffing, symlink following, well-known excludes, and provider abstraction all stay out

Distribution check:

- Phase A does not introduce a new binary, package, or published artifact
- no release workflow or install-path change is part of this PR

### 15.6.b Phase A architecture

Execution shape:

```text
start path
   |
   v
detect_repo_root(start_path, RepoRootDetectionOptions)
   |
   v
SnapshotRequest { root, source: Worktree, options }
   |
   +--> compile caller globs into CompiledIgnoreSet
   |
   v
walk root tree deterministically (`walkdir`)
   |
   +--> skip `.git/**` intrinsically
   +--> skip explicit-glob matches
   +--> reject or skip non-UTF8 paths by policy
   +--> reject or skip symlinks by policy
   +--> reject or skip large files by policy
   +--> reject unsupported file kinds with typed diagnostics
   |
   v
read regular-file bytes once
   |
   +--> blob_fingerprint = sha256_bytes(bytes)
   +--> file_id = FileId::from_identity("repo\\0file\\0v1\\0<repo-path>")
   |
   v
build BlobRecord + InventoryEntry
   |
   v
one deterministic sort boundary by RepoPath
   |
   v
snapshot fingerprint from sorted `(path, blob_fingerprint, size_bytes)`
   |
   v
RepoSnapshot { root, source, inventory, blobs, diagnostics, stats, fingerprint }
```

Module boundary for Phase A:

```text
src/lib.rs
  └── pub(crate) mod repo
        ├── mod.rs
        ├── error.rs
        ├── diagnostics.rs
        ├── root.rs
        ├── ignore.rs
        ├── inventory.rs
        ├── blob.rs
        ├── snapshot.rs
        └── schema.rs

tests/
  ├── repo_root.rs
  ├── repo_snapshot.rs
  ├── repo_ignore.rs
  ├── repo_fingerprints.rs
  ├── repo_purity.rs
  └── repo_schema.rs
```

Architectural rule:

> `snapshot.rs` owns the one-way assembly flow. Later seams read from `RepoSnapshot`. They do not reopen the filesystem or reinterpret the traversal.

### 15.6.c Exact implementation slices

| Slice | Files / modules | Deliverable | Done when |
|---|---|---|---|
| A1. seam surface + typed failures | `src/repo/mod.rs`, `error.rs`, `diagnostics.rs`, `root.rs` | stub replaced by a real crate-private seam surface with typed root-detection and repo-diagnostic contracts | `src/repo/mod.rs` is no longer a placeholder and root detection compiles with typed errors |
| A2. traversal + ignore policy | `src/repo/ignore.rs`, `Cargo.toml` | explicit Phase-A traversal policy, caller glob compilation, `.git` intrinsic exclusion, `walkdir` dependency choice landed | invalid globs fail deterministically and traversal policy types compile |
| A3. inventory + blob contracts | `src/repo/inventory.rs`, `blob.rs` | immutable inventory/blob-store contracts and lookup helpers exist | `Inventory` and `BlobStore` compile with the documented methods and invariants |
| A4. snapshot builder | `src/repo/snapshot.rs`, `root.rs`, `ignore.rs`, `inventory.rs`, `blob.rs` | deterministic worktree materialization pipeline exists end to end | root detect -> walk -> filter -> read -> hash -> sort -> fingerprint -> `RepoSnapshot` all work through one codepath |
| A5. schema embedding | `src/repo/schema.rs`, `schemas/repo/snapshot_manifest.v1.json` | embedded repo snapshot schema matches the manifest on disk | schema constants compile and disk-vs-embedded validation passes |
| A6. fixtures + integration tests | `fixtures/repo/**`, `tests/repo_*.rs`, `tests/compile_matrix.rs` | every required Phase-A branch and invariant is covered | all Phase-A acceptance paths have deterministic tests, including no-default-features posture |
| A7. docs sweep | `README.md`, `fixtures/README.md`, `schemas/README.md` | docs describe the landed repo seam honestly | no README still describes repo as only reserved future work |

Implementation order:

1. Land A1.
2. Land A2.
3. Land A3.
4. Land A4.
5. Land A5.
6. Launch A6 and A7 only after A4/A5 interfaces stop moving.

### 15.6.d Test review, required coverage for Phase A

CODE PATH COVERAGE
===========================
[+] Repo root detection
    ├── [REQUIRED] start path is a directory, nearest marker ancestor wins
    ├── [REQUIRED] start path is a file, detection begins at the parent directory
    ├── [REQUIRED] `ceiling_dir` stops ascent deterministically
    ├── [REQUIRED] marker order does not change the selected root
    ├── [REQUIRED] `.git` marker works as either file or directory
    └── [REQUIRED] missing start path / missing root emits typed failure

[+] Ignore compilation and path-policy handling
    ├── [REQUIRED] invalid caller glob hard-fails with typed error
    ├── [REQUIRED] `.git/**` is always excluded without caller input
    ├── [REQUIRED] explicit glob exclusion removes matching subtree
    ├── [REQUIRED] `target/`, `node_modules/`, `.venv/`, `dist/`, `build/` are not implicitly excluded
    ├── [REQUIRED] non-UTF8 path with `Error` fails deterministically
    ├── [REQUIRED] non-UTF8 path with `Skip` omits the path and records deterministic diagnostics
    ├── [REQUIRED] symlink with `Skip` never enters inventory
    └── [REQUIRED] large file with `Error` / `Skip` follows policy exactly

[+] Snapshot materialization
    ├── [REQUIRED] inventory contains only regular files that survive policy checks
    ├── [REQUIRED] each surviving file produces matching `InventoryEntry` + `BlobRecord`
    ├── [REQUIRED] `BlobStore.len()` equals `Inventory.len()` on success
    ├── [REQUIRED] identical trees under different temp roots yield the same fingerprint
    ├── [REQUIRED] traversal order does not affect inventory order or fingerprint
    ├── [REQUIRED] source tree deletion after snapshot does not change `read_bytes`
    └── [REQUIRED] missing blob/path lookup returns typed repo failure, never panic

[+] Schema and manifest coverage
    ├── [REQUIRED] embedded `snapshot_manifest.v1.json` matches the on-disk schema
    ├── [REQUIRED] valid fixture manifests validate and deserialize
    └── [REQUIRED] invalid fixture manifests fail validation deterministically

USER FLOW COVERAGE
===========================
[+] Internal engine flow
    ├── [REQUIRED] detect root -> build snapshot -> inspect stats/inventory/fingerprint
    ├── [REQUIRED] apply caller exclusion -> snapshot omits subtree and preserves determinism
    ├── [REQUIRED] policy violation -> typed diagnostics/error without partial corrupt snapshot
    └── [REQUIRED] snapshot created once -> later reads come only from `BlobStore`, not disk

─────────────────────────────────
COVERAGE GOAL: 28/28 required paths covered before Phase A promotion
  Code paths: 24/24
  Internal flows: 4/4
QUALITY BAR: no smoke-only tests for acceptance paths
GAPS ALLOWED AT PROMOTION: 0
─────────────────────────────────

Required test files:

- `tests/repo_root.rs` for root detection semantics and root-marker invariants
- `tests/repo_snapshot.rs` for worktree materialization happy path and policy behavior
- `tests/repo_ignore.rs` for caller glob compilation and intrinsic `.git` exclusion
- `tests/repo_fingerprints.rs` for cross-root determinism and traversal-order invariants
- `tests/repo_purity.rs` for "snapshot survives source mutation/deletion" behavior
- `tests/repo_schema.rs` for embedded-schema parity and fixture-manifest validation
- extend `tests/compile_matrix.rs` only as needed to keep `--no-default-features` coverage intact

Recommended validation commands:

```bash
cargo fmt --all
cargo clippy -p substrate-lift --all-targets -- -D warnings
cargo test -p substrate-lift --test repo_root -- --nocapture
cargo test -p substrate-lift --test repo_snapshot -- --nocapture
cargo test -p substrate-lift --test repo_ignore -- --nocapture
cargo test -p substrate-lift --test repo_fingerprints -- --nocapture
cargo test -p substrate-lift --test repo_purity -- --nocapture
cargo test -p substrate-lift --test repo_schema -- --nocapture
cargo test -p substrate-lift --test compile_matrix -- --nocapture
```

### 15.6.e Failure modes for Phase A

| Codepath | Failure mode | Test required? | Error handling required? | Consumer sees |
|---|---|---|---|---|
| root detection | wrong ancestor chosen because marker order leaks into selection | yes | yes, deterministic nearest-match semantics | typed root-detection failure or correct root |
| ignore compilation | invalid glob accepted and silently broadens or narrows scope | yes | yes, typed compile failure for the request | snapshot build rejected before traversal |
| path normalization | non-UTF8 host path coerced into bogus `RepoPath` | yes | yes, policy-driven error/skip only | typed failure or deterministic omission |
| symlink handling | symlink enters inventory in Phase A despite `Skip` policy | yes | yes, deterministic skip diagnostic | file absent from snapshot, never followed |
| snapshot purity | `read_bytes` reopens live disk after materialization | yes | test-enforced invariant | deterministic snapshot read, even if source tree changes |
| fingerprinting | traversal order or absolute temp root alters snapshot fingerprint | yes | test-enforced invariant | stable fingerprint across equivalent trees |
| blob/inventory consistency | inventory entry exists without backing blob bytes | yes | yes, constructor-level invariant | typed `MissingBlob` style failure, never panic |
| large-file policy | oversize file leaks through despite `Error` / `Skip` policy | yes | yes, typed policy failure or deterministic omission | clear error or clear omission, never silent inclusion |

Critical gap rule:

- Phase A does not promote if any failure mode is both untested and capable of causing silent snapshot drift

### 15.6.f NOT in scope for the Phase A implementation PR

- `SnapshotSource::GitRev { rev }`
- `RepoDiff`, rename detection, or any other Phase-B diff behavior
- `SymlinkPolicy::Follow`
- provider abstraction or git backend crates
- `.gitignore`, `.ignore`, or global git-exclude interpretation
- typed well-known excludes for cache/build/vendor directories
- runtime or CLI translation from `CompiledAnalysisDefaults` into `SnapshotOptions`
- public API promotion of `repo`

### 15.6.g Worktree parallelization strategy

Dependency table:

| Step | Modules touched | Depends on |
|---|---|---|
| A1 seam surface + root detection | `src/repo/`, `src/lib.rs` | — |
| A2 traversal + ignore policy | `src/repo/`, `Cargo.toml` | A1 |
| A3 inventory + blob contracts | `src/repo/` | A1 |
| A4 snapshot builder | `src/repo/` | A2, A3 |
| A5 schema embedding | `src/repo/`, `schemas/repo/` | A4 |
| A6 fixtures + tests | `fixtures/repo/`, `tests/` | A4, A5 |
| A7 docs sweep | `README.md`, `fixtures/README.md`, `schemas/README.md` | A5 |

Parallel lanes:

- Lane A: A1 -> A2/A3 -> A4 -> A5 (core lane; A2 and A3 are the only safe micro-split because both fold back into shared `src/repo/` before A4)
- Lane B: A6 (sidecar after A5, lives in `fixtures/repo/` and `tests/`)
- Lane C: A7 (sidecar after A5, lives in README surfaces only)

Execution order:

- Run Lane A first until the snapshot contract, schema ID, and typed errors stop moving.
- Once A5 lands, launch Lane B and Lane C in parallel worktrees.
- Merge B and C back into A before the Phase-A promotion gate.

Conflict flags:

- A1 through A5 all touch `src/repo/`, so the core implementation is one lane in practice
- Lane B must not reopen `src/repo/`; if missing test hooks require core changes, fold that work back into Lane A
- Lane C is safe only if doc updates stay in README surfaces and do not reopen contract decisions

### 15.6.h Phase A completion summary

- Step 0: scope accepted as filesystem-first immutable snapshot substrate
- Architecture: root-detect -> ignore-compile -> walk -> hash -> sort -> snapshot diagram written
- Test Review: 28 required paths named, 0 gaps allowed at promotion
- Failure modes: 0 silent-snapshot-drift gaps allowed
- NOT in scope: written
- What already exists: written
- Parallelization: 3 lanes, 2 sidecar lanes after 1 sequential core lane

Phase-A promotion gate:

1. `src/repo/mod.rs` is no longer a stub and remains crate-private.
2. root detection obeys nearest-match, file-start, ceiling-dir, and marker-order invariants.
3. worktree snapshot materialization produces immutable blobs plus deterministic inventory ordering.
4. `.git/**` is intrinsically excluded, caller globs behave deterministically, and no implicit cache/build/vendor excludes leak in.
5. embedded snapshot schema matches disk, and fixture manifests validate.
6. all required Phase-A tests above are present and passing.
7. `cargo check -p substrate-lift --no-default-features` still passes.

## 15.7 Phase B execution plan, eng-review locked

This section starts from the landed Phase-A repo substrate as it exists on `feat/lift` today.

The earlier sections answer "what Phase B is." This section answers "how Phase B lands without reopening Phase A or smuggling in Phase C."

### 15.7.a Step 0, scope challenge

Phase B must treat the shipped snapshot substrate as the baseline, not as fresh clay.

What already exists and must be reused:

| Sub-problem | Existing code | Phase B decision |
|---|---|---|
| crate-private seam boundary | `src/lib.rs`, `src/repo/mod.rs` | keep `repo` crate-private, do not promote public API just to expose diffing |
| immutable snapshot access | `src/repo/snapshot.rs`, `RepoSnapshot::entry`, `RepoSnapshot::read_bytes` | diff only already-materialized snapshots, never reopen filesystem walking or option semantics |
| stable ordered inventory | `src/repo/inventory.rs`, `Inventory::iter`, `Inventory::get`, `fingerprint_entries` | build Phase B on the already-sorted inventory surface, not a new unordered map layer |
| stable repo-relative path identity | `src/kernel/path.rs`, `src/repo/inventory.rs` | key the diff on `RepoPath` only, one diff row per repo-relative path |
| stable content and canonical fingerprinting | `src/kernel/fingerprint.rs`, `src/repo/inventory.rs` | fingerprint diff output with canonical JSON, never with absolute roots or traversal state |
| landed schema/test harness pattern | `src/repo/schema.rs`, `tests/support/repo_support.rs`, `tests/repo_schema.rs` | extend the same embedded-schema plus generated-manifest pattern for diff fixtures |
| no-default-features build posture | `tests/compile_matrix.rs` | Phase B must preserve `cargo check -p substrate-lift --no-default-features` |
| downstream runtime boundary | `src/app/runtime.rs` | keep runtime bootstrap out of Phase B, repo still ends at immutable artifact plus pure diff |

Scope decision for the Phase B implementation PR:

- add only pure path-based diffing over `RepoSnapshot`;
- add `src/repo/diff.rs`, `schemas/repo/diff_manifest.v1.json`, `fixtures/repo/diff/**`, `tests/repo_diff.rs`, and the minimal schema/test-harness extensions needed to support them;
- extend `src/repo/mod.rs` and `src/repo/schema.rs` only to expose the new Phase-B internal seam surface and embedded schema constants;
- keep `SnapshotSource`, `SnapshotOptions`, traversal behavior, root detection, ignore semantics, and snapshot diagnostics unchanged;
- do not add rename detection, blob-level patch hunks, diff-time filesystem reads, git revision materialization, runtime wiring, or pack-driven option translation.

Complexity check:

- Phase B is smaller than Phase A, but it still spans core seam code, schema, fixtures, and tests, so it should be treated as one focused repo slice rather than a "quick helper";
- the minimum credible change still touches more than 8 files once fixtures and tests are counted, but that is acceptable because the implementation stays inside one seam and adds no new runtime integration;
- if a proposed change needs to modify `src/app/runtime.rs`, `src/pack/**`, or snapshot-construction behavior in `src/repo/snapshot.rs`, it is probably Phase C or later work and should be deferred.

Search and distribution check:

- `[Layer 1]` use the built-in sorted `BTreeMap` inventory order already landed in `Inventory`; do not pull in a diff crate or invent a second ordering primitive;
- `[Layer 1]` reuse the existing `jsonschema` + embedded-schema test pattern from `tests/repo_schema.rs`; do not introduce a second fixture-validation path;
- Phase B does not introduce a new binary, package, feature flag, or published artifact, so no CLI surface, release workflow, or install-path change belongs in this PR.

Scope result:

- scope accepted as-is, because the plan is already the reduced version and the remaining work is the complete implementation of that reduced scope.

### 15.7.b Architecture review

Phase B execution shape:

```text
base: RepoSnapshot.inventory.iter()         head: RepoSnapshot.inventory.iter()
              |                                            |
              v                                            v
      peekable ordered stream                      peekable ordered stream
                     \                            /
                      \                          /
                       +---- merge-walk on RepoPath ----+
                                    |
                                    +--> base path only -> Removed { before: Some, after: None }
                                    +--> head path only -> Added   { before: None, after: Some }
                                    +--> same path, same blob  -> omit
                                    +--> same path, new blob   -> Modified { before: Some, after: Some }
                                    |
                                    v
                         ordered Vec<DiffEntry> in path order
                                    |
                                    v
               canonical diff fingerprint document:
               {
                 version,
                 base_fingerprint,
                 head_fingerprint,
                 entries[path, kind, before_blob, after_blob]
               }
                                    |
                                    v
               RepoDiff { base_fingerprint, head_fingerprint, entries, fingerprint }
```

Architectural rules:

- `build_diff(base, head)` compares `InventoryEntry` state only. It does not reopen the filesystem and does not need `BlobStore::read_bytes`.
- The algorithm should merge-walk the two already-sorted inventory iterators instead of first building a path union set and then sorting it again. That keeps the implementation explicit, deterministic, and linear in the number of inventory entries.
- `DiffEntry.before` and `DiffEntry.after` carry the already-materialized `InventoryEntry` values needed by downstream consumers. Blob bytes stay in the snapshots.
- There is at most one `DiffEntry` per `RepoPath`.
- The zero-diff case is first-class: identical snapshots produce `RepoDiff { entries: vec![] }` with a stable fingerprint.
- There is no meaningful new auth or external security surface here because Phase B is a crate-private pure transform. The risk is semantic drift, not permission bypass. The architecture therefore needs tests and invariants more than new runtime guards.

Realistic production failure scenarios this plan must account for:

- if the merge walk advances the wrong iterator on equal paths, a modified file can be emitted as `Added` plus `Removed` instead of `Modified`;
- if the fingerprint document is built from host-specific data or unstable entry order, identical semantic diffs hash differently across temp roots and test reruns;
- if the implementation consults `BlobStore` or the live filesystem during diffing, Phase B silently reintroduces non-determinism that Phase A was explicitly built to prevent.

Module boundary for Phase B only:

```text
src/repo/
  mod.rs
  diff.rs
  schema.rs

tests/
  repo_diff.rs
  repo_schema.rs
  support/repo_support.rs

fixtures/repo/
  diff/
```

### 15.7.c Code quality review

Phase B should stay boring on purpose.

| Area | Code quality rule | Why |
|---|---|---|
| `src/repo/diff.rs` | keep one small data-model layer plus a few private helpers, not a trait hierarchy or backend abstraction | there is one backend and one diff mode, so extra abstraction is fake flexibility |
| diff algorithm | prefer a merge-walk over sorted iterators, not `HashMap` / `HashSet` plus a cleanup sort | the inventories are already ordered, so a second ordering phase is pure churn |
| fingerprinting | fingerprint a plain canonical document built from diff entries, not ad hoc string concatenation | matches landed seam style and keeps the hash input reviewable |
| `src/repo/schema.rs` | add diff schema constants in the same pattern as snapshot schema constants | keeps schema embedding DRY and discoverable |
| `tests/support/repo_support.rs` | add `diff_validator`, `manifest_from_diff`, and paired-snapshot helpers by extending the current helper module, not by creating a second support module | prevents fixture logic from splitting across parallel helper stacks |
| `tests/repo_schema.rs` | keep schema parity and manifest-shape assertions here | this file already owns repo-schema validation |
| `tests/repo_diff.rs` | keep behavior assertions here, not inside schema tests | separates "what the diff means" from "what the manifest looks like" |

Hard quality rules:

1. Do not change `src/repo/snapshot.rs` unless an actual Phase-A defect is discovered. Phase B should consume snapshots, not renegotiate them.
2. Do not duplicate `InventoryEntry` or `RepoSnapshot` fields into a second test-only manifest type when existing runtime types can already supply the values.
3. Do not add a provider trait, a generic diff backend, or a rename-similarity helper "for later."
4. Keep docs last. If prose reveals a missing semantic decision, fix the semantics in code/tests first.

### 15.7.d Rules to lock now

Lock these rules:

1. `build_diff` is a pure function over `&RepoSnapshot` inputs.
2. `DiffKind` remains exactly `Added`, `Modified`, `Removed`.
3. Path comparison is by canonical repo-relative `RepoPath`, never absolute host path.
4. `Modified` is decided by `blob_fingerprint` inequality only.
5. Unchanged files are omitted even if file IDs or stats are inspected elsewhere.
6. Rename behavior is not inferred. A rename is one `Removed` plus one `Added`.
7. `DiffEntry.before` is present only for `Removed` and `Modified`.
8. `DiffEntry.after` is present only for `Added` and `Modified`.
9. `RepoDiff.fingerprint` excludes absolute roots, file IDs, stats, diagnostics, and blob bytes.
10. Phase B adds no new error surface beyond existing typed repo/schema failures needed to compile or validate fixtures.
11. The emitted `entries` vector is already in final order when constructed. No "sort at the end just to be safe" cleanup pass should be needed.
12. Schema/test helpers stay extensions of the landed repo test harness, not a second parallel fixture system.

### 15.7.e Phase B implementation slices

| Slice | Files / modules | Deliverable | Done when |
|---|---|---|---|
| B1. diff seam surface + merge-walk algorithm | `src/repo/{mod.rs,diff.rs}` | real Phase-B diff types and pure builder land behind the existing crate-private seam | identical, add, remove, modify, and empty-diff cases all compile through one deterministic ordered codepath |
| B2. diff fingerprint helper | `src/repo/diff.rs` | canonical diff fingerprint is computed from the ordered diff document and stored on `RepoDiff` | equivalent snapshot pairs produce the same fingerprint across repeated calls and different temp roots |
| B3. diff schema embedding | `src/repo/schema.rs`, `schemas/repo/diff_manifest.v1.json` | embedded diff schema matches the on-disk manifest and follows the existing repo-schema pattern | diff schema constants compile and disk-vs-embedded validation passes |
| B4. diff fixture/test harness | `tests/support/repo_support.rs`, `fixtures/repo/diff/**` | generated diff manifests can be built and validated the same way snapshot manifests are | helper code can materialize paired snapshots and emit diff-manifest JSON without custom one-off logic |
| B5. integration tests | `tests/repo_diff.rs`, `tests/repo_schema.rs`, `tests/compile_matrix.rs` if needed | every required Phase-B path and invariant is covered | add/remove/modify/rename-as-two-events/empty-diff/fingerprint determinism all have deterministic assertions |
| B6. docs sweep | `lift_seam2_spec_reviewed.md`, `README.md`, `fixtures/repo/README.md`, `schemas/README.md` | prose matches the landed Phase-B contract and no file still describes diffing as hand-wavy future work | README surfaces describe repo diffing as pure over snapshots, not as git-aware history analysis |

Implementation order:

1. Land B1 and B2 together. The data model and the fingerprint semantics should freeze at the same time.
2. Land B3 immediately after the diff contract stops moving.
3. Land B4 once the schema filename, manifest fields, and helper names are stable.
4. Land B5 after B1 through B4 are stable.
5. Land B6 last, once schema filename and fixture names are frozen.

### 15.7.f Test review, required coverage for Phase B

CODE PATH COVERAGE
===========================
[+] `src/repo/diff.rs` merge-walk
    |- [REQUIRED] base exhausted first -> remaining head paths emit `Added`
    |- [REQUIRED] head exhausted first -> remaining base paths emit `Removed`
    |- [REQUIRED] same path with equal blob fingerprint is omitted
    |- [REQUIRED] same path with different blob fingerprint emits exactly one `Modified`
    |- [REQUIRED] path ordering follows `RepoPath` lexical order without an extra final sort
    `- [REQUIRED] identical snapshots produce `entries.is_empty()`

[+] `DiffEntry` shape invariants
    |- [REQUIRED] `Added` has `before: None`, `after: Some`
    |- [REQUIRED] `Removed` has `before: Some`, `after: None`
    |- [REQUIRED] `Modified` has both `before` and `after`
    `- [REQUIRED] rename-shaped change appears as `Removed` + `Added`, never a fourth kind

[+] Fingerprinting
    |- [REQUIRED] repeated `build_diff(base, head)` calls produce the same `RepoDiff.fingerprint`
    |- [REQUIRED] changing entry order in fixture construction does not change the diff fingerprint
    |- [REQUIRED] changing the diff entry set changes the diff fingerprint
    |- [REQUIRED] equivalent trees under different absolute temp roots still diff to empty when snapshot fingerprints match
    `- [REQUIRED] diffing two prebuilt snapshots after later live-tree mutation still yields the original result

[+] Schema and manifest coverage
    |- [REQUIRED] embedded `diff_manifest.v1.json` matches the on-disk schema
    |- [REQUIRED] valid diff fixture manifests validate and deserialize
    |- [REQUIRED] invalid diff fixture manifests fail validation deterministically
    |- [REQUIRED] manifest ordering matches runtime ordering
    `- [REQUIRED] manifest fingerprints match runtime fingerprints

USER FLOW COVERAGE
===========================
[+] Internal engine flow
    |- [REQUIRED] materialize base snapshot -> materialize head snapshot -> build diff -> inspect ordered entries
    |- [REQUIRED] modified path carries both `before` and `after` inventory entries through the handoff
    |- [REQUIRED] empty diff still exposes stable base/head fingerprints plus a deterministic diff fingerprint
    `- [REQUIRED] diff fixture generation reuses the same runtime codepath as the behavior tests

---------------------------------
COVERAGE GOAL: 18/18 required Phase-B paths covered before promotion
  Code paths: 14/14
  Internal flows: 4/4
QUALITY BAR: no smoke-only tests for acceptance paths
GAPS ALLOWED AT PROMOTION: 0
---------------------------------

Required test files:

- add `tests/repo_diff.rs` for diff assembly, ordering, before/after-shape invariants, and rename-as-add-remove semantics;
- extend `tests/repo_schema.rs` for diff-schema embedding, valid diff manifests, invalid diff manifests, and runtime-vs-manifest parity assertions;
- extend `tests/support/repo_support.rs` with the minimal helpers needed to materialize paired snapshots, generate diff-manifest JSON, and validate diff manifests;
- extend `tests/compile_matrix.rs` only if the new schema constants or diff module alter existing crate-level compile assertions.

Required fixture inventory:

```text
fixtures/repo/diff/
  valid/
    empty_diff.json
    added_file.json
    removed_file.json
    modified_file.json
    rename_as_add_remove.json
  invalid/
    manifest_bad_repo_path.json
    manifest_bad_kind.json
    manifest_missing_diff_fingerprint.json
    manifest_before_after_shape_invalid.json
```

Test-plan artifact to write during review/implementation:

- write `~/.gstack/projects/$SLUG/{user}-{branch}-eng-review-test-plan-{timestamp}.md`;
- list the affected internal flow as `RepoSnapshot -> build_diff -> RepoDiff`;
- list the key interactions to verify as add/remove/modify/empty/rename-shaped changes plus schema parity;
- list the critical path as "materialize paired snapshots, build diff, validate manifest, rerun under a second temp root."

Recommended validation commands:

```bash
cargo fmt --all
cargo clippy -p substrate-lift --all-targets -- -D warnings
cargo test -p substrate-lift --test repo_diff -- --nocapture
cargo test -p substrate-lift --test repo_schema -- --nocapture
cargo test -p substrate-lift --test compile_matrix -- --nocapture
```

### 15.7.g Performance review

Phase B has no database or network path, but it still has real performance decisions to lock:

| Concern | Risk | Required decision |
|---|---|---|
| algorithmic shape | building a union set and sorting it later turns a simple ordered merge into avoidable extra allocation and CPU | use a merge-walk over the two ordered inventories |
| memory growth | cloning blob bytes or whole snapshots during diffing would scale with repository size, not with changed paths | clone only the `InventoryEntry` values needed for emitted `DiffEntry`s |
| fingerprint cost | fingerprinting the full snapshot again would repeat Phase-A work | fingerprint only the ordered diff document plus base/head snapshot fingerprints |
| hidden I/O | consulting `BlobStore::read_bytes` or host paths during diffing reintroduces live-tree coupling | Phase B stays metadata-only once the two snapshots already exist |

Performance rules to lock now:

- expected time complexity is `O(base_entries + head_entries + changed_entries_for_fingerprint)`;
- expected memory beyond the two snapshots is `O(changed_entries)`, not `O(all_paths)` plus blob bytes;
- do not add caches in Phase B. The data is already in memory, and a cache here would just be another invalidation problem;
- if an implementation reaches for `HashMap` / `HashSet` plus a cleanup sort, reject it unless a benchmark demonstrates a real regression in the ordered merge-walk.

### 15.7.h Failure modes for Phase B

| Codepath | Failure mode | Test required? | Error handling required? | Consumer sees |
|---|---|---|---|---|
| merge-walk advance logic | modified path is emitted as add+remove because the wrong iterator advances on equality | yes | yes, exact equal-path branch coverage | semantic drift in downstream change classification |
| base/head exhaustion | added or removed path is dropped because one tail of the iterator pair is never flushed | yes | yes, tail-handling tests | missing change never reaches downstream seams |
| modified detection | same-path content change is missed because comparison uses size or stale host reads instead of `blob_fingerprint` | yes | yes, compare the landed inventory fingerprints only | incorrect "unchanged" result |
| unchanged omission | unchanged paths leak into output and bloat every downstream consumer | yes | test-enforced invariant | noisy diff with unstable fanout |
| ordering | diff entries follow hash-map or traversal order instead of sorted `RepoPath` order | yes | test-enforced invariant | nondeterministic diff output and fingerprint drift |
| rename semantics | heuristic rename detection sneaks in through future convenience logic | yes | yes, hard rule to stay add/remove/modify only | backend-specific drift and unstable semantics |
| diff fingerprinting | absolute roots, file IDs, or output order leak into `RepoDiff.fingerprint` | yes | test-enforced invariant | same semantic diff hashes differently across runs |
| schema embedding | embedded diff schema drifts from the on-disk schema or fixture shape | yes | yes, schema parity tests must hard-fail | manifest validation mismatch in tests |

Critical gap rule:

- Phase B does not promote if any failure mode is both untested and capable of causing silent diff drift.

### 15.7.i NOT in scope for the Phase B implementation PR

- `SnapshotSource::GitRev { rev }`
- `SymlinkPolicy::Follow`
- any new filesystem walking or ignore semantics
- rename detection or similarity heuristics
- blob-level textual patches or hunk generation
- runtime, CLI, or pack integration work
- provider abstraction
- pack/profile schema changes to thread diff config into seam 2

### 15.7.j Worktree parallelization strategy

Dependency table:

| Step | Modules touched | Depends on |
|---|---|---|
| B1 core diff model + merge-walk | `src/repo/` | - |
| B2 diff schema embedding | `src/repo/`, `schemas/repo/` | B1 |
| B3 diff fixture/harness helpers | `tests/support/`, `fixtures/repo/diff/` | B1, B2 |
| B4 behavior + schema tests | `tests/` | B1, B2, B3 |
| B5 docs sweep | spec + README surfaces | B2, B3 |

Parallel lanes:

- Lane A: B1 -> B2 (sequential core lane, shared `src/repo/` contract surfaces)
- Lane B: B3 -> B4 (test lane after the core contract and schema stabilize)
- Lane C: B5 (docs-only sidecar after schema filename and fixture names freeze)

Execution order:

1. Run Lane A first until the diff contract, helper names, and schema ID stop moving.
2. Once B2 lands, launch Lane B and Lane C in parallel worktrees.
3. Merge Lane B before promotion, because the tests are part of the gate.
4. Merge Lane C last, once code and fixture names are frozen.

Conflict flags:

- Lane A is strictly sequential because `src/repo/mod.rs`, `diff.rs`, and `schema.rs` define the seam surface.
- Lane B must not reopen snapshot-construction behavior in `src/repo/snapshot.rs`; if tests discover that need, route it back to Lane A and re-evaluate scope.
- Lane C is safe only if it stays docs-only. If prose uncovers unresolved semantics, fix the semantics first in Lane A.
- Do not split B1 into two parallel worktrees. `mod.rs` and `diff.rs` are one shared ownership surface in practice.

### 15.7.k Phase B completion summary

- Step 0: scope accepted as pure diff over landed snapshots
- Architecture review: ordered merge-walk plus canonical fingerprint pipeline written
- Code quality review: no new abstraction layers, no snapshot renegotiation, helper reuse locked
- Test review: 18 required Phase-B paths named, 0 gaps allowed at promotion
- Performance review: linear merge-walk, no blob rereads, no cache layer, no extra union-sort pass
- Failure modes: 0 silent-diff-drift gaps allowed
- NOT in scope: written
- What already exists: written
- TODOS.md updates: none required beyond the explicit deferrals already captured here
- Outside voice: skipped for this document rewrite, because this section is locking implementation structure rather than running a fresh multi-model review
- Parallelization: 3 lanes, 1 sequential core lane plus 2 sidecars
- Lake score: complete implementation path selected, no intentional shortcuts

Phase-B promotion gate:

1. `src/repo/diff.rs` exists and remains a pure function over `&RepoSnapshot`.
2. `build_diff(base, head)` emits only ordered add/remove/modify entries and omits unchanged files.
3. the implementation uses the landed ordered inventory surface, not a second unordered path-collection layer.
4. rename-shaped changes appear only as `Removed` + `Added`.
5. `schemas/repo/diff_manifest.v1.json` is embedded through `src/repo/schema.rs` and validated in tests.
6. all required Phase-B tests above are present and passing.
7. `cargo check -p substrate-lift --no-default-features` still passes.

## 16. My recommended decision set

These are the decisions I would lock now.

1. `repo` stays `pub(crate)` for seam 2.
2. seam 2 phase A is **filesystem-first** and does not require pack schema changes.
3. `RepoSnapshot` is a **fully materialized immutable artifact**.
4. `FileId` is path-stable, not content-stable.
5. snapshot fingerprints are based only on sorted `(path, blob_fingerprint, size_bytes)`.
6. `.git/**` is the only intrinsic exclude in phase A.
7. `.gitignore` support is explicitly out of scope in phase A.
8. seam 2 has a canonical **A/B/C** phase map:
   - A = worktree snapshot substrate
   - B = pure diff over snapshots
   - C = expanded materialization semantics
9. `SymlinkPolicy::Follow` and `SnapshotSource::GitRev` belong to Phase C, not Phase A.
10. `RepoProvider` is deferred until Phase C and only lands if a second backend now exists.
11. diff semantics are path-based add/remove/modify only, with **no rename detection**.
12. seam 2 introduces fixture schemas, not end-user runtime config schemas.
13. well-known cache/build/vendor excludes are deferred to Phase C as an explicit materialization policy expansion.
14. whole-snapshot size/file-count limits are deferred, but Phase A must expose stats so memory pressure is visible.
15. Phase A performance guidance is lightweight: one materialization pass, one deterministic sort boundary, no live re-read after snapshot creation.
16. Phase A should take `walkdir = "2"` instead of growing a handwritten recursive walker.
17. Phase B should compare only already-materialized `InventoryEntry` state from two `RepoSnapshot`s.
18. Phase B should ship `diff_manifest.v1.json`, `tests/repo_diff.rs`, and deterministic diff fingerprints before any GitRev work.

That gives seam 2 a tight first landing that is honest about the current crate reality while still locking the right long-term shape for later seams.

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 2 | ISSUES OPEN | mode: SELECTIVE_EXPANSION, 1 critical gaps |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 3 | CLEAR | 12 issues, 0 critical gaps |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |

**UNRESOLVED:** 0
**VERDICT:** ENG CLEARED — Phase A is landed, and the next implementation target is the locked Phase-B pure-diff plan above. CEO review remains informational and has open issues.
