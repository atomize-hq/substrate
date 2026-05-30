# Spec: Agent Session Compactor v0.1

## Assumptions I'm Making

1. `agent-session-compactor` should be a library-first workspace crate with a thin standalone binary
   inside this repo.
2. v0.1 should analyze historical Codex artifacts from the host Codex home rather than live
   Substrate runtime streams.
3. `unified-agent-api-codex` and `unified-agent-api-wrapper-events` are the primary parser seams and
   should be reused before any bespoke parsing is added.
4. Exact-hash dedupe with stable keep-first semantics is the only required dedupe mode in v0.1.
5. The compactor should emit file-backed artifacts for later analyzer consumption and operator
   review, but should not infer drift or task frame itself.
6. Wrapper-style ingestion may exist in the crate surface, but rollout ingestion from
   `<codex-home>/sessions/**` is the required baseline.
7. If real-session pressure testing exposes parser-surface gaps in `unified-agent-api-*`, those
   should be evaluated as upstream changes before planning around them downstream.

## Objective

Build an implementation-grade v0.1 library and thin CLI that convert Codex session artifacts into a
deterministic, provenance-preserving transcript row corpus suitable for later drift analysis.

Primary user:

- the operator or downstream analyzer that needs a stable, reviewable session artifact instead of
  raw transcript files

Success means:

- historical Codex session files can be discovered and parsed reproducibly
- normalized rows preserve enough provenance to support later analysis
- exact duplicate content can be folded without destroying archival truth
- the crate produces a stable on-disk artifact bundle and does not mutate the source corpus

## Tech Stack

- Language: Rust 2021
- Product shape: library-first workspace crate plus thin binary
- Crate name: `agent-session-compactor`
- Primary parser dependencies:
  - `unified-agent-api-wrapper-events = { version = "=0.3.5", features = ["codex"] }`
  - `unified-agent-api-codex = "=0.3.5"`
- Supporting dependencies:
  - `serde`
  - `serde_json`
  - `thiserror`
  - `blake3`
  - `camino`
  - `walkdir`
  - `time`
  - `clap`

Dependency posture:

- no dependency on `crates/shell`
- no dependency on Substrate trace or runtime-state crates
- no Python, embedding stack, or semantic-dedupe dependency in v0.1

## Commands

Workspace validation:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Targeted crate commands:

```bash
cargo build -p agent-session-compactor
cargo test -p agent-session-compactor -- --nocapture
cargo run -p agent-session-compactor -- \
  --codex-home ~/.codex \
  --output-dir target/agent-session-compactor/latest
```

Targeted session example:

```bash
cargo run -p agent-session-compactor -- \
  --codex-home ~/.codex \
  --session-id <session-id> \
  --output-dir target/agent-session-compactor/session-<session-id>
```

## Project Structure

```text
Cargo.toml
  Add the workspace member.

crates/agent-session-compactor/Cargo.toml
  Crate manifest and parser dependencies.

crates/agent-session-compactor/src/main.rs
  Thin CLI entrypoint over library APIs.

crates/agent-session-compactor/src/lib.rs
  Public crate surface.

crates/agent-session-compactor/src/cli.rs
  CLI options and validation.

crates/agent-session-compactor/src/discovery.rs
  Codex-home resolution and session artifact discovery.

crates/agent-session-compactor/src/ingest/
  Rollout and optional wrapper ingestion adapters.

crates/agent-session-compactor/src/normalize/
  Event-to-row mapping and provenance assembly.

crates/agent-session-compactor/src/canonicalize/
  Text cleanup and stable hashing.

crates/agent-session-compactor/src/dedupe/
  Exact fold logic and dedupe audit groups.

crates/agent-session-compactor/src/export/
  Manifest, JSONL, and summary writers.

crates/agent-session-compactor/tests/
  Fixture-driven integration coverage.

docs/specs/agent-session-compactor-v0.1-spec.md
  This spec.
```

Deliberate non-touch paths for v0.1:

```text
crates/shell
crates/world*
crates/trace
crates/common
crates/replay
```

## Code Style

Use explicit typed conversions and provenance-preserving records. Avoid raw `serde_json::Value`
traversal as the default path when an owned parser surface already provides a typed shape.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactionRow {
    pub source_file: Utf8PathBuf,
    pub source_kind: SourceKind,
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub event_index: usize,
    pub line_number: usize,
    pub timestamp: Option<OffsetDateTime>,
    pub kind: CompactionKind,
    pub text: String,
    pub canonical_text: String,
    pub text_hash_hex: String,
}

pub fn fold_exact_duplicates(rows: &[CompactionRow]) -> Vec<DedupeGroup> {
    build_exact_hash_groups(rows)
}
```

Conventions:

- deterministic-first control flow
- provenance preserved separately from canonicalized text
- exact dedupe only in the archival baseline
- no silent fallback from parse failure to guessed semantics
- binary remains a thin wrapper around library-owned behavior

## Testing Strategy

Frameworks:

- unit tests beside implementation modules
- integration tests in `crates/agent-session-compactor/tests/`

Required test layers:

1. Discovery tests
   - Codex-home resolution order is stable
   - missing `sessions/` fails clearly
   - lexicographic file ordering is stable
2. Parser adapter tests
   - rollout JSONL fixtures parse through `unified-agent-api-codex`
   - wrapper fixtures parse through `unified-agent-api-wrapper-events` when enabled
   - unknown records remain auditable and explicit
3. Normalization tests
   - content-bearing session records map into stable `CompactionRow` shapes
   - row kinds preserve message, reasoning, tool, status, and error distinctions
4. Canonicalization tests
   - ANSI stripping and whitespace normalization are deterministic
   - oversized payload behavior is explicit and stable
5. Dedupe tests
   - exact duplicate rows fold by kind plus canonical hash
   - keep-first semantics are stable
   - archival rows remain intact
6. End-to-end bundle tests
   - a synthetic Codex home yields manifest, archival rows, compact rows, dedupe audit, and summary
   - source files remain untouched

## Boundaries

- Always:
  - read from a resolved Codex home or explicit input root
  - preserve provenance for every emitted row
  - keep source session files immutable
  - use owned parser crates before bespoke parsing
  - emit archival output separately from compacted output
- Ask first:
  - adding new source corpora beyond Codex session artifacts
  - widening dedupe beyond exact-hash fold
  - introducing new heavy dependencies
  - integrating the crate into live runtime flows
  - planning around parser limitations that could be solved cleanly in `unified-agent-api-*`
- Never:
  - rewrite or delete source transcript files
  - let compact output become the only exported view
  - silently drop oversized, malformed, or unknown records
  - mix drift inference into the compactor module
  - bury parser-seam problems in compactor-specific workarounds without first evaluating an
    upstream parser-crate fix

## Success Criteria

The spec is satisfied when:

1. The repo contains a standalone crate named `agent-session-compactor`.
2. The crate is library-first and exposes a thin binary for direct use and pressure testing.
3. The crate reads historical Codex session artifacts from a resolved host Codex home.
4. The ingestion layer reuses owned parser surfaces as the primary route.
5. The normalized row model preserves stable provenance and deterministic text hashing.
6. Exact duplicate content folds with stable keep-first semantics and separate audit output.
7. The crate emits a reviewable bundle:
   - `manifest.json`
   - `rows.archival.jsonl`
   - `rows.compact.jsonl`
   - `dedupe-audit.jsonl`
   - `summary.md`
8. The source Codex home remains unchanged.

## Open Questions

1. Should wrapper-style ingestion ship in the first cut or remain behind an explicit input flag?
2. What exact oversized-record representation best preserves deterministic hashing and auditability?
3. Should archived session directories outside `sessions/` be deferred entirely from v0.1?
4. Which parser-seam gaps, if any, justify planned changes in `unified-agent-api-*` rather than
   downstream workarounds?
