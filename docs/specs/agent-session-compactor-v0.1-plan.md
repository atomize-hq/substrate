# Plan: Agent Session Compactor v0.1

## Scope

This plan implements [agent-session-compactor-v0.1-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-session-compactor-v0.1-spec.md:1).

The goal is to deliver a library-first crate with a thin CLI that:

- discovers Codex session artifacts
- parses them through owned parser seams
- normalizes them into provenance-preserving rows
- canonicalizes and exact-dedupes content
- emits a reviewable artifact bundle

This module is the dependency root for the later analyzer and sentinel work.

## Implementation Strategy

Build the crate from the inside out:

1. Define the library-owned row and audit contracts first.
2. Land discovery and ingestion against real Codex session artifacts.
3. Add deterministic normalization and canonicalization.
4. Add exact dedupe and export bundle assembly.
5. Add the thin CLI only after the library pipeline is stable.

This ordering keeps the binary thin and prevents CLI concerns from distorting the library seam.

## Major Components

### 1. Artifact Contract

Deliver first:

- `CompactionRow`
- `SourceKind`
- `CompactionKind`
- `RowRef`
- `DedupeGroup`
- output bundle manifest types

Why first:

- every later module depends on these contracts
- the analyzer cannot be planned concretely unless the row surface is stable enough

### 2. Discovery Layer

Deliver second:

- Codex-home resolution order
- recursive session discovery under `<codex-home>/sessions`
- stable lexicographic ordering
- optional session filtering by session id

Why second:

- it establishes the stable input corpus before parser integration starts

### 3. Ingestion Adapters

Deliver third:

- rollout ingestion via `unified-agent-api-codex`
- optional wrapper ingestion seam, possibly feature-gated or flag-gated
- explicit unknown-record capture
- explicit oversized-record handling

Critical rule:

- if real-corpus testing exposes a parser-seam gap in `unified-agent-api-*`, stop and assess the
  upstream change before implementing downstream compensating logic

### 4. Normalization Pipeline

Deliver fourth:

- event-to-row mapping
- provenance assembly
- stable timestamps, event indexes, and line numbers
- kind/role separation for messages, reasoning, tools, status, and errors

Why here:

- dedupe and export should operate on row contracts, not parser-native event shapes

### 5. Canonicalization And Hashing

Deliver fifth:

- ANSI stripping
- line-ending normalization
- low-signal whitespace normalization
- canonical-text hashing
- optional raw-text hashing when available

Why here:

- dedupe policy depends on canonical text being explicit and test-covered

### 6. Dedupe And Audit

Deliver sixth:

- exact-hash grouping
- keep-first semantics
- dedupe audit groups
- archival rows kept separate from compacted rows

### 7. Export Bundle

Deliver seventh:

- `manifest.json`
- `rows.archival.jsonl`
- `rows.compact.jsonl`
- `dedupe-audit.jsonl`
- `summary.md`

### 8. Thin CLI

Deliver last:

- argument parsing
- input-root selection
- output-dir selection
- session-id filtering
- operator-facing exit errors

## Sequencing

Sequential work:

1. contract types
2. discovery
3. rollout ingestion
4. normalization
5. canonicalization
6. dedupe
7. export
8. CLI

Parallel-safe work after the row contract stabilizes:

- export implementation can proceed in parallel with some dedupe work
- CLI wiring can proceed in parallel with final integration tests
- wrapper-ingestion exploration can happen in parallel with rollout-path hardening if it remains
  clearly optional

## Risks And Mitigations

### Risk 1: Parser surface mismatch

Risk:

- real Codex artifacts may expose gaps or awkward seams in `unified-agent-api-*`

Mitigation:

- start with real-session pressure testing early
- add an explicit checkpoint after rollout ingestion
- pause for upstream parser planning rather than baking compactor-local workarounds

### Risk 2: Row schema too thin for analyzer needs

Risk:

- a minimal row surface may omit evidence later needed for task-frame inference or thrash detection

Mitigation:

- keep provenance, line ordering, turn identity, and content kinds explicit
- validate the row surface against analyzer requirements before freezing export format

### Risk 3: Canonicalization destroys meaning

Risk:

- aggressive whitespace or payload normalization could collapse semantically distinct records

Mitigation:

- keep raw and canonical fields separate
- test tool outputs, patch outputs, and command outputs independently

### Risk 4: The binary grows into the module

Risk:

- Codex-hook pressure testing could tempt the CLI into owning behavior that belongs in the library

Mitigation:

- library API first
- CLI added late
- keep success criteria anchored on the library contract

## Verification Checkpoints

### Checkpoint A: Contract Lock

Verify:

- row and audit types are stable enough for fixtures
- analyzer planning can reference the row surface without guessing

Evidence:

- unit tests for core types
- doc review against the analyzer spec

### Checkpoint B: Real-Corpus Ingestion

Verify:

- real Codex session files parse through owned parser seams
- any parser-surface gaps are identified explicitly

Evidence:

- targeted fixture tests
- at least one real-session dry run through the library pipeline

### Checkpoint C: Deterministic Compaction

Verify:

- normalization, canonicalization, and dedupe are stable across repeated runs

Evidence:

- repeated integration test runs produce identical JSONL output

### Checkpoint D: Bundle Complete

Verify:

- the artifact bundle is complete and reviewable
- the CLI is only a thin wrapper over library behavior

Evidence:

- crate tests
- end-to-end run using the documented `cargo run` command

## Handoff To Next Module

This plan is complete enough for `agent-drift-analyzer` only when all of the following are true:

- archival and compact row formats are stable enough for downstream consumption
- dedupe audit groups are available
- source ordering and provenance are explicit
- session identity is preserved
- any upstream parser change decision is resolved or explicitly deferred with rationale

## Exit Criteria

The plan is complete when:

1. the crate builds and tests cleanly
2. the artifact bundle is emitted from real or fixture session inputs
3. the library seam is stable enough for the analyzer to consume directly
4. no unresolved parser-surface workaround has been silently pushed downstream
