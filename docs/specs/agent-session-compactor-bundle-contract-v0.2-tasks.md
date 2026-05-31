# Tasks: Agent Session Compactor Bundle Contract v0.2

This task list implements:

- [agent-session-compactor-bundle-contract-v0.2-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-session-compactor-bundle-contract-v0.2-spec.md:1)
- [agent-session-compactor-bundle-contract-v0.2-plan.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-session-compactor-bundle-contract-v0.2-plan.md:1)

## Task List

- [x] Task: Lock the `v0.2` contract shape and implementation order in repo docs
  - Acceptance: the repo doc chain names one concrete cutover path for the file-table location, `u32` file ids, export-only DTOs, analyzer load-time path resolution, and placement before later sentinel follow-up work.
  - Verify: doc review across the spec, plan, task, and implementation-order docs
  - Files: `docs/specs/agent-session-compactor-bundle-contract-v0.2-spec.md`, `docs/specs/agent-session-compactor-bundle-contract-v0.2-plan.md`, `docs/specs/agent-session-compactor-bundle-contract-v0.2-tasks.md`, `docs/specs/hybrid-drift-sentinel-implementation-order.md`

- [ ] Task: Add `v0.2` export-facing manifest, file, row, and row-ref DTOs
  - Acceptance: `agent-session-compactor` defines explicit `v0.2` DTOs for the manifest-owned file registry, exported rows, and id-backed dedupe refs without changing the internal path-backed normalization and dedupe contracts.
  - Verify: `cargo test -p agent-session-compactor export_bundle -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/mod.rs`, `crates/agent-session-compactor/src/export/files.rs`, `crates/agent-session-compactor/tests/export_bundle.rs`

- [ ] Task: Emit archival and compact row JSONL with `source_file_id`
  - Acceptance: `rows.archival.jsonl` and `rows.compact.jsonl` serialize `source_file_id` instead of `source_file`, `manifest.json` reports `schema_version = "v0.2"` plus the deterministic file registry, and `summary.md` remains operator-readable.
  - Verify: `cargo test -p agent-session-compactor export_bundle -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/files.rs`, `crates/agent-session-compactor/src/export/mod.rs`, `crates/agent-session-compactor/tests/export_bundle.rs`

- [ ] Task: Migrate dedupe audit export to id-backed row refs and validate registry sync
  - Acceptance: exported dedupe representative and duplicate refs use `source_file_id`, every exported ref resolves through the manifest registry, and no dedupe audit ref points at an archival row that disappears under the new contract.
  - Verify: `cargo test -p agent-session-compactor dedupe -- --nocapture` and `cargo test -p agent-session-compactor export_bundle -- --nocapture`
  - Files: `crates/agent-session-compactor/src/dedupe/mod.rs`, `crates/agent-session-compactor/src/dedupe/audit.rs`, `crates/agent-session-compactor/src/export/files.rs`, `crates/agent-session-compactor/tests/dedupe.rs`, `crates/agent-session-compactor/tests/export_bundle.rs`

- [ ] Task: Cut analyzer input over to `v0.2` and resolve ids at the load boundary
  - Acceptance: `agent-drift-analyzer` loads the manifest file registry and `v0.2` row/audit DTOs, resolves ids into the existing internal path-backed row and ref types before sorting or validation, and fails clearly on unknown or duplicate file ids.
  - Verify: `cargo test -p agent-drift-analyzer input_contract -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/input.rs`, `crates/agent-drift-analyzer/tests/input_contract.rs`, `crates/agent-drift-analyzer/tests/support/mod.rs`

- [ ] Task: Preserve analyzer semantic behavior under the new bundle contract
  - Acceptance: checkpoint construction, evidence rendering, task-frame inference, and summary export remain semantically stable after the loader cutover, with no fallback guessing for mixed or partial bundle contracts.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/input.rs`, `crates/agent-drift-analyzer/tests/export_bundle.rs`, `crates/agent-drift-analyzer/tests/end_to_end.rs`

- [ ] Task: Gate the cutover on a bounded compactor-to-analyzer smoke run
  - Acceptance: a bounded real-session run emits a `v0.2` bundle, analyzer consumption still succeeds, and the repo docs capture the observed row-count stability plus bundle-size delta for the chosen smoke session.
  - Verify: `cargo run -p agent-session-compactor -- --codex-home ~/.codex --session-id "$SESSION_ID" --output-dir "$COMPACTOR_OUT"` and `cargo run -p agent-drift-analyzer -- --input-dir "$COMPACTOR_OUT" --output-dir "$ANALYZER_OUT"`
  - Files: `crates/agent-session-compactor/tests/end_to_end.rs`, `crates/agent-drift-analyzer/tests/end_to_end.rs`, `docs/specs/agent-session-compactor-bundle-contract-v0.2-plan.md`
