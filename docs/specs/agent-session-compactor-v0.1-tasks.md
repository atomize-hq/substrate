# Tasks: Agent Session Compactor v0.1

This task list implements:

- [agent-session-compactor-v0.1-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-session-compactor-v0.1-spec.md:1)
- [agent-session-compactor-v0.1-plan.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-session-compactor-v0.1-plan.md:1)

## Task List

- [ ] Task: Scaffold the library-first crate and thin binary entrypoint
  - Acceptance: `agent-session-compactor` exists as a workspace member with `src/lib.rs` and a minimal `src/main.rs` that delegates to library-owned behavior.
  - Verify: `cargo build -p agent-session-compactor`
  - Files: `Cargo.toml`, `crates/agent-session-compactor/Cargo.toml`, `crates/agent-session-compactor/src/lib.rs`, `crates/agent-session-compactor/src/main.rs`

- [ ] Task: Define row, audit, and manifest core types
  - Acceptance: core types exist for `CompactionRow`, kind enums, `RowRef`, `DedupeGroup`, and manifest metadata, with enough fields to support downstream analyzer planning.
  - Verify: `cargo test -p agent-session-compactor core_types -- --nocapture`
  - Files: `crates/agent-session-compactor/src/lib.rs`, `crates/agent-session-compactor/src/normalize/mod.rs`, `crates/agent-session-compactor/src/dedupe/mod.rs`, `crates/agent-session-compactor/src/export/mod.rs`

- [ ] Task: Implement Codex-home resolution and session discovery
  - Acceptance: the library resolves `--codex-home`, `CODEX_HOME`, and `$HOME/.codex` in order, scans `<codex-home>/sessions`, sorts files lexicographically, and supports optional session-id filtering.
  - Verify: `cargo test -p agent-session-compactor discovery -- --nocapture`
  - Files: `crates/agent-session-compactor/src/discovery.rs`, `crates/agent-session-compactor/src/cli.rs`, `crates/agent-session-compactor/tests/discovery.rs`

- [ ] Task: Implement rollout ingestion via `unified-agent-api-codex`
  - Acceptance: rollout JSONL files under discovered session paths ingest through owned parser surfaces with explicit unknown-record capture and stable event ordering.
  - Verify: `cargo test -p agent-session-compactor rollout_ingest -- --nocapture`
  - Files: `crates/agent-session-compactor/src/ingest/mod.rs`, `crates/agent-session-compactor/src/ingest/codex_rollout.rs`, `crates/agent-session-compactor/tests/rollout_ingest.rs`

- [ ] Task: Gate real-session ingestion and decide if any upstream parser work is required
  - Acceptance: at least one real-session dry run is attempted, and any parser-surface gap is either resolved, explicitly deferred, or turned into an upstream `unified-agent-api-*` planning decision before downstream work continues.
  - Verify: `cargo run -p agent-session-compactor -- --codex-home ~/.codex --output-dir target/agent-session-compactor/latest`
  - Files: `docs/specs/agent-session-compactor-v0.1-plan.md`, `docs/specs/agent-session-compactor-v0.1-spec.md`

- [ ] Task: Implement normalization from parser events into `CompactionRow`
  - Acceptance: content-bearing session events map into stable row kinds with provenance fields, turn identity, line numbers, and event indexes preserved.
  - Verify: `cargo test -p agent-session-compactor normalization -- --nocapture`
  - Files: `crates/agent-session-compactor/src/normalize/mod.rs`, `crates/agent-session-compactor/src/normalize/row.rs`, `crates/agent-session-compactor/tests/normalization.rs`

- [ ] Task: Implement deterministic canonicalization and hashing
  - Acceptance: canonical text normalization, ANSI stripping, raw/canonical separation, and stable hash generation are implemented and fixture-covered.
  - Verify: `cargo test -p agent-session-compactor canonicalization -- --nocapture`
  - Files: `crates/agent-session-compactor/src/canonicalize/mod.rs`, `crates/agent-session-compactor/src/canonicalize/text.rs`, `crates/agent-session-compactor/tests/canonicalization.rs`

- [ ] Task: Implement exact dedupe and dedupe-audit emission
  - Acceptance: rows fold by stable keep-first semantics using kind plus canonical hash, while archival rows remain complete and dedupe groups capture representative/duplicate refs.
  - Verify: `cargo test -p agent-session-compactor dedupe -- --nocapture`
  - Files: `crates/agent-session-compactor/src/dedupe/mod.rs`, `crates/agent-session-compactor/src/dedupe/exact.rs`, `crates/agent-session-compactor/src/dedupe/audit.rs`, `crates/agent-session-compactor/tests/dedupe.rs`

- [ ] Task: Implement manifest and bundle export
  - Acceptance: the crate writes `manifest.json`, `rows.archival.jsonl`, `rows.compact.jsonl`, `dedupe-audit.jsonl`, and `summary.md` to the output directory.
  - Verify: `cargo test -p agent-session-compactor export_bundle -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/mod.rs`, `crates/agent-session-compactor/src/export/files.rs`, `crates/agent-session-compactor/tests/export_bundle.rs`

- [ ] Task: Wire the thin CLI to library behavior
  - Acceptance: the CLI accepts documented input/output options, delegates to library-owned behavior, and returns clear user-facing errors.
  - Verify: `cargo run -p agent-session-compactor -- --codex-home ~/.codex --output-dir target/agent-session-compactor/latest`
  - Files: `crates/agent-session-compactor/src/main.rs`, `crates/agent-session-compactor/src/cli.rs`, `crates/agent-session-compactor/src/lib.rs`

- [ ] Task: Gate end-to-end validation and freeze the downstream artifact contract
  - Acceptance: repeated test runs produce stable outputs, and the artifact contract is confirmed stable enough for `agent-drift-analyzer` consumption.
  - Verify: `cargo test -p agent-session-compactor -- --nocapture`
  - Files: `crates/agent-session-compactor/tests/end_to_end.rs`, `docs/specs/agent-session-compactor-v0.1-plan.md`, `docs/specs/agent-drift-analyzer-v0.1-spec.md`
