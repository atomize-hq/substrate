# Spec: Agent Drift Analyzer v0.1

## Assumptions I'm Making

1. `agent-drift-analyzer` should consume compactor artifacts rather than raw Codex session files.
2. v0.1 analysis should be session-scoped, not cross-session by default.
3. The analyzer should be deterministic-first and should not require model adjudication to be useful.
4. `task frame` is the semantic center; plan artifacts are evidence when present, not prerequisites.
5. `dead_end_thrash` requires repetition-preserving evidence and therefore cannot rely on deduped-only
   rows.
6. The first consumer is the human operator evaluating drift, not an autonomous steering loop.
7. If compactor pressure testing exposes parser-surface gaps in `unified-agent-api-*`, those should
   be resolved upstream before the analyzer is planned around distorted artifacts.
8. The currently landed compactor contract is row-oriented and does not yet provide first-class
   `file_read`, `file_edit`, or `command_summary` event kinds, so analyzer v0.1 must begin from the
   actual row kinds and payload surfaces that now exist.

## Objective

Build an implementation-grade v0.1 crate that reads session-scoped compactor artifacts, infers a
current task frame from observable evidence, and emits deterministic checkpoints for three initial
drift classes:

- `wrong_plan_branch`
- `ignoring_repo_truth`
- `dead_end_thrash`

Primary user:

- the operator reviewing long runs and wanting evidence-backed drift visibility

Success means:

- task-frame inference works without a required plan file
- drift scores are explainable through stable evidence references
- checkpoint artifacts are reviewable, session-scoped, and deterministic

## Tech Stack

- Language: Rust 2021
- Product shape: library-first workspace crate plus thin binary
- Crate name: `agent-drift-analyzer`
- Primary input contract:
  - `rows.archival.jsonl`
  - `rows.compact.jsonl`
  - `dedupe-audit.jsonl`
  - `manifest.json`
  - atomically published final output directory whose `manifest.json` appears only after the other
    analyzer-facing files are complete
- Supporting dependencies:
  - `serde`
  - `serde_json`
  - `thiserror`
  - `camino`
  - `time`
  - `clap`

Dependency posture:

- no direct dependency on live Substrate runtime state
- no requirement to parse raw Codex transcript files
- no default model dependency in v0.1

## Commands

Workspace validation:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Targeted crate commands:

```bash
cargo build -p agent-drift-analyzer
cargo test -p agent-drift-analyzer -- --nocapture
cargo run -p agent-drift-analyzer -- \
  --input-dir target/agent-session-compactor/session-<session-id> \
  --output-dir target/agent-drift-analyzer/session-<session-id>
```

## Project Structure

```text
Cargo.toml
  Add the workspace member.

crates/agent-drift-analyzer/Cargo.toml
  Crate manifest.

crates/agent-drift-analyzer/src/main.rs
  Thin CLI entrypoint over library APIs.

crates/agent-drift-analyzer/src/lib.rs
  Public crate surface.

crates/agent-drift-analyzer/src/cli.rs
  CLI options and validation.

crates/agent-drift-analyzer/src/input.rs
  Compactor artifact loading and contract checks.

crates/agent-drift-analyzer/src/context/
  Objective extraction, truth-artifact ranking, working-set assembly.

crates/agent-drift-analyzer/src/inference/
  Task-frame hypothesis logic and confidence shaping.

crates/agent-drift-analyzer/src/scoring/
  Deterministic drift thresholds and evidence gathering.

crates/agent-drift-analyzer/src/checkpoint/
  Checkpoint schema, segmentation, and export.

crates/agent-drift-analyzer/tests/
  Fixture-driven session-analysis coverage.

docs/specs/agent-drift-analyzer-v0.1-spec.md
  This spec.
```

## Code Style

Design the analyzer around evidence refs and explicit thresholds rather than prose-only summaries.
The initial implementation should consume the actual compactor row surface that now exists, including
stable `RowRef` identity by `source_file`, `line_number`, `event_index`, and `row_ordinal`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRef {
    pub row: RowRef,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriftScore {
    pub class: DriftClass,
    pub raw_score: u8,
    pub confidence: Confidence,
    pub flagged: bool,
    pub evidence: Vec<EvidenceRef>,
}
```

Conventions:

- session-scoped analysis first
- threshold logic explicit and test-covered
- evidence references preferred over freeform explanation strings
- no hidden model-like heuristics
- binary remains a thin wrapper around library-owned behavior
- infer files, tools, and command families from the landed row/payload surface unless the compactor
  contract is explicitly widened

## Testing Strategy

Frameworks:

- unit tests beside implementation modules
- integration tests in `crates/agent-drift-analyzer/tests/`

Required test layers:

1. Input contract tests
   - missing or malformed compactor artifacts fail clearly
   - session identity and ordering remain stable
2. Context assembly tests
   - objective extraction prefers literal user language
   - candidate truth artifacts rank explicit user mentions above inferred artifacts
   - working-set assembly distinguishes files, tools, and command families from the actual landed
     row and payload surface
3. Task-frame inference tests
   - the analyzer infers a usable task frame without any plan artifact
   - confidence drops when multiple plausible frames fit the evidence
4. Drift scoring tests
   - each of the three drift classes has positive, negative, and threshold-boundary coverage
   - `dead_end_thrash` reads repetition-preserving evidence rather than compacted-only rows
5. Checkpoint tests
   - checkpoint segmentation is deterministic
   - evidence refs point at stable row references
6. End-to-end tests
   - session-scoped compactor artifacts yield `checkpoints.jsonl` and a reviewable summary

## Boundaries

- Always:
  - analyze one session at a time by default
  - keep `task frame` as the semantic center
  - emit explicit evidence references for scored drift
  - preserve deterministic checkpoint ordering
  - use repetition-preserving evidence for thrash detection
  - treat the compactor's current row kinds and payloads as the starting analyzer contract
- Ask first:
  - cross-session aggregation or ranking
  - adding new drift classes beyond the initial three
  - making model adjudication default-on
  - treating inferred truth artifacts as authoritative without user pinning
  - planning around distorted or incomplete compactor artifacts if the real issue is an upstream
    parser surface in `unified-agent-api-*`
  - widening compactor output with first-class file or command summary event kinds if the current
    row surface is not sufficient
- Never:
  - parse raw Codex transcript files as the primary path
  - require a plan artifact
  - use deduped-only rows for all analyzer decisions
  - emit prose-only checkpoints with no stable evidence anchors

## Success Criteria

The spec is satisfied when:

1. The repo contains a standalone crate named `agent-drift-analyzer`.
2. The crate consumes compactor artifacts rather than raw session files.
3. The analyzer infers a session-scoped `task frame` without requiring a plan file.
4. The analyzer scores exactly three deterministic drift classes in v0.1.
5. The analyzer works from the landed compactor row contract, or explicitly stops at the artifact
   gate with evidence that the contract must be widened first.
6. Every flagged drift condition includes stable evidence references.
7. The analyzer emits a reviewable bundle containing:
   - `checkpoints.jsonl`
   - `summary.md`
8. The output is useful for human review before any live steering loop exists.

## Open Questions

1. What is the smallest checkpoint window that still yields stable task-frame inference?
2. Which evidence refs belong in the checkpoint contract versus a separate expanded audit artifact?
3. Should the analyzer emit only checkpoints, or also a machine-readable per-session evaluation
   report for calibration?
4. Does the landed compactor row surface support useful working-set inference as-is, or is a narrow
   contract widening needed before analyzer heuristics go deeper?

## Gate Notes

- `C11` compactor contract gate outcome on `2026-05-29`: the upstream compactor bundle is stable enough for analyzer planning with `manifest.json`, `rows.archival.jsonl`, `rows.compact.jsonl`, `dedupe-audit.jsonl`, and `summary.md` emitted deterministically from repeat runs against the same corpus.
- `A3` analyzer artifact gate outcome on `2026-05-30`: the landed compactor row contract is
  sufficient for useful working-set inference without analyzer-side guessing because directive text
  preserves literal file mentions, tool calls preserve raw argument JSON and tool names, archival
  rows preserve repetition for thrash scoring, and `RowRef` remains stable at
  `source_file`/`line_number`/`event_index`/`row_ordinal`.
- `A12` replay freeze outcome on `2026-05-30`: the analyzer checkpoint contract is stable enough
  for replay-mode sentinel consumption with session-scoped `checkpoints.jsonl` entries containing
  deterministic boundaries, task-frame evidence, explicit drift scores, and an `expected_next_step`
  field, plus operator-readable `summary.md`.
