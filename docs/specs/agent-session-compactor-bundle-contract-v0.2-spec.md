# Spec: Agent Session Compactor Bundle Contract v0.2

## Assumptions I'm Making

1. `v0.2` is a follow-up artifact-contract cleanup for the existing `agent-session-compactor`
   and `agent-drift-analyzer` crates, not a new crate or a new analysis mode.
2. The immediate driver is repeated per-row provenance payload, especially `source_file`, rather
   than a new semantic compaction feature.
3. The compactor's in-memory normalization and dedupe types may remain richer than the exported
   on-disk bundle contract.
4. The current `canonical_text` export mistake is already corrected at the export layer, and
   `v0.2` should address the broader root cause: on-disk bundles should not be implicit
   serialization of internal structs.
5. `agent-drift-analyzer` is the first downstream consumer and can move directly to the `v0.2`
   bundle format because this seam is still greenfield.
6. Stable row identity is still required for dedupe audit, checkpoint evidence, and analyzer
   replay, but that identity does not need to embed a full absolute source path in every row.

## Objective

Define and land an explicit `v0.2` compactor bundle contract that removes repeated `source_file`
strings from every exported row, replaces them with compact stable file identifiers, and decouples
the on-disk artifact schema from the compactor's internal Rust structs.

Primary users:

- the operator inspecting compactor bundles
- `agent-drift-analyzer` as the first downstream consumer
- later drift/sentinel consumers that need stable evidence refs without inflated payloads

Success means:

- exported bundles no longer repeat the same absolute file path in every row
- stable row identity is preserved for dedupe audit and analyzer evidence
- analyzer input accepts the explicit `v0.2` bundle contract without fallback guessing
- the exported schema is explicit and versioned rather than accidental serialization of internal
  structs

## Tech Stack

- Language: Rust 2021
- Existing crates:
  - `crates/agent-session-compactor`
  - `crates/agent-drift-analyzer`
- Existing dependencies expected to remain sufficient:
  - `serde`
  - `serde_json`
  - `camino`
  - `time`
  - `thiserror`

Dependency posture:

- no new parser dependency
- no new storage backend
- no compression step or archive format change in `v0.2`
- no change to the source Codex session corpus format

## Commands

Workspace validation:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Targeted compactor/analyzer validation:

```bash
cargo test -p agent-session-compactor export_bundle -- --nocapture
cargo test -p agent-session-compactor end_to_end -- --nocapture
cargo test -p agent-drift-analyzer input_contract -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Bounded real-session smoke:

```bash
export SESSION_ID=<session-id>
export SMOKE_ROOT=target/hybrid-drift-smoke/$SESSION_ID
export COMPACTOR_OUT=$SMOKE_ROOT/compactor
export ANALYZER_OUT=$SMOKE_ROOT/analyzer

cargo run -p agent-session-compactor -- \
  --codex-home ~/.codex \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"
```

## Project Structure

```text
crates/agent-session-compactor/src/normalize/
  Internal row construction and provenance assembly.

crates/agent-session-compactor/src/dedupe/
  Internal dedupe logic and row-reference audit generation.

crates/agent-session-compactor/src/export/
  Explicit v0.2 bundle export DTOs and serializers.

crates/agent-drift-analyzer/src/input.rs
  `v0.2` bundle loading and row resolution.

crates/agent-session-compactor/tests/
  Export, compatibility, and end-to-end compactor coverage.

crates/agent-drift-analyzer/tests/
  Input contract and end-to-end analyzer coverage.

docs/specs/agent-session-compactor-bundle-contract-v0.2-spec.md
  This spec.
```

Deliberate non-goals for `v0.2`:

```text
crates/shell
crates/world*
crates/replay
source Codex session JSONL format
drift scoring logic beyond compatibility fallout
```

## Code Style

Do not serialize internal row structs directly to disk. Define export-facing DTOs that reflect the
artifact contract and are narrow by design.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleFileV0_2 {
    pub id: u32,
    pub path: Utf8PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportRowV0_2 {
    pub source_file_id: u32,
    pub source_kind: SourceKind,
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub event_index: usize,
    pub line_number: usize,
    pub row_ordinal: usize,
    pub timestamp: Option<OffsetDateTime>,
    pub kind: CompactionKind,
    pub user_message_role: Option<UserMessageRole>,
    pub dedupe_identity: Option<String>,
    pub text: String,
    pub text_hash_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowRefV0_2 {
    pub source_file_id: u32,
    pub line_number: usize,
    pub event_index: usize,
    pub row_ordinal: usize,
}
```

Conventions:

- keep internal compaction types separate from export DTOs
- version artifact contracts explicitly
- preserve deterministic ordering for file ids and row refs
- prefer explicit contract cutovers over long-lived dual-format support when the seam is still
  greenfield
- keep row identity compact, stable, and path-resolvable

## Testing Strategy

Frameworks:

- unit tests beside export/input modules
- integration tests in `crates/agent-session-compactor/tests/`
- integration tests in `crates/agent-drift-analyzer/tests/`

Required test layers:

1. Export DTO tests
   - `v0.2` row serialization omits repeated `source_file`
   - file-table ordering is deterministic
   - `RowRefV0_2` resolves to the expected file path
2. Input contract tests
   - analyzer input accepts `v0.2` bundles with `source_file_id`
   - malformed or partial `v0.2` bundles fail clearly rather than silently guessing
3. Dedupe audit tests
   - representative and duplicate refs remain stable after id compaction
   - dedupe audit still points at archival rows that exist
4. End-to-end compactor tests
   - repeated compactor runs emit deterministic `v0.2` bundles
   - archival and compact rows preserve prior semantic counts
5. End-to-end analyzer tests
   - analyzer output from a `v0.2` compactor bundle matches expected checkpoint behavior
   - no checkpoint/evidence regression from file-id-based row identity
6. Real-session smoke
   - a bounded real-session run demonstrates meaningful bundle size reduction without analyzer
     breakage

## Boundaries

- Always:
  - preserve deterministic row identity across export, dedupe audit, and analyzer evidence
  - keep bundle schema explicit and versioned
  - keep source Codex session files immutable
  - validate that bundle-level file tables and per-row file ids stay in sync
- Ask first:
  - adding compression or changing the file container format
  - changing checkpoint artifact schemas outside what is required for row-ref compatibility
  - replacing `manifest.json` semantics rather than extending them
- Never:
  - derive row identity from row order alone
  - silently remap unknown file ids to fallback paths
  - make analyzer correctness depend on absolute path duplication in every row
  - serialize internal compaction structs directly as the only definition of the on-disk contract

## Success Criteria

The spec is satisfied when:

1. The compactor emits an explicit `v0.2` artifact contract with versioned export DTOs.
2. `rows.archival.jsonl` and `rows.compact.jsonl` no longer repeat full `source_file` paths per
   row.
3. A bundle-level file table exists and row refs resolve through compact stable file ids.
4. Dedupe audit output preserves stable representative/duplicate references under the new contract.
5. `agent-drift-analyzer` can load the new `v0.2` bundles with `source_file_id`.
6. Existing analyzer checkpoint and evidence behavior remains semantically stable.
7. A bounded real-session smoke run confirms bundle size reduction and analyzer compatibility.
8. The source Codex home and source rollout files remain unchanged.

## Planning Resolutions

The following contract choices are locked for the `v0.2` plan/task slice:

1. The bundle-level file table lives in `manifest.json`; `v0.2` does not add a separate
   `files.json`.
2. `agent-session-compactor` keeps path-bearing internal row types for now; the explicit contract
   boundary is the export DTO layer rather than a full internal row-type split.
3. Analyzer checkpoint and evidence outputs continue to serialize resolved file paths for operator
   readability in this slice; the id-backed contract change stops at the compactor/analyzer input
   boundary.
4. File ids use `u32`.

## Gate Notes

- `C11` follow-up on `2026-05-31`: export-layer `canonical_text` removal proved that the bundle can
  safely diverge from internal row structs without analyzer breakage, which strengthens the case
  for an explicit `v0.2` export contract.
- `C12` bundle-size observation on `2026-05-31`: for real session
  `019e79dc-456c-7e92-bcbc-3b677d9e8b3f`, repeated `source_file` strings still account for roughly
  `35 KB` in archival rows and `27 KB` in compact rows. The size win is smaller than the
  `canonical_text` fix but still meaningful, and the bigger benefit is cleaner contract design.
