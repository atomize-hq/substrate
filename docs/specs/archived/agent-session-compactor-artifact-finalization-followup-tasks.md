# Tasks: Agent Session Compactor Artifact Finalization Follow-up

This task list implements the post-`C11` compactor hardening packet required before `A1-A12`.

Primary sources:

- [agent-session-compactor-v0.1-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-session-compactor-v0.1-spec.md:1)
- [agent-session-compactor-v0.1-plan.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-session-compactor-v0.1-plan.md:1)
- [hybrid-drift-sentinel-implementation-order.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/hybrid-drift-sentinel-implementation-order.md:1)

## Task List

- [x] Task: Define the atomic bundle finalization contract
  - Acceptance: the compactor has an explicit contract for staging-path writes, final publish semantics, `manifest.json` ordering, and incomplete-run behavior.
  - Verify: contract reflected in code comments and/or adjacent docs, and exported behavior is enforced by tests rather than implied.
  - Files: `crates/agent-session-compactor/src/export/mod.rs`, `crates/agent-session-compactor/src/export/files.rs`, `docs/specs/hybrid-drift-sentinel-implementation-order.md`

- [x] Task: Implement staging-directory bundle export
  - Acceptance: bundle files are written into a staging directory instead of directly into the final output directory, and a failed run does not populate the final output location with partial files.
  - Verify: `cargo test -p agent-session-compactor export_bundle -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/files.rs`, `crates/agent-session-compactor/src/export/mod.rs`, `crates/agent-session-compactor/tests/export_bundle.rs`

- [x] Task: Publish the completed bundle atomically and write `manifest.json` last
  - Acceptance: the final output directory appears only after row files, dedupe audit, and summary are complete in staging, and `manifest.json` is written last before final publish.
  - Verify: `cargo test -p agent-session-compactor export_bundle -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/files.rs`, `crates/agent-session-compactor/tests/export_bundle.rs`

- [x] Task: Implement explicit incomplete-run and cleanup behavior
  - Acceptance: interrupted or failed runs either clean up staging state automatically or leave only clearly marked staging artifacts; they never leave a valid-looking final bundle behind.
  - Verify: `cargo test -p agent-session-compactor export_bundle -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/files.rs`, `crates/agent-session-compactor/tests/export_bundle.rs`

- [x] Task: Add interruption and partial-output regression coverage
  - Acceptance: tests cover the failure mode where export stops mid-run and verify that the final output directory is not published as complete.
  - Verify: `cargo test -p agent-session-compactor export_bundle -- --nocapture`
  - Files: `crates/agent-session-compactor/tests/export_bundle.rs`, `crates/agent-session-compactor/tests/end_to_end.rs`

- [x] Task: Re-run compactor validation on the hardened artifact seam
  - Acceptance: the hardened export path passes crate build/test validation, and at least one bounded real or fixture-backed run confirms that the bundle still emits successfully under the new publish model.
  - Verify: `cargo build -p agent-session-compactor && cargo test -p agent-session-compactor -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/files.rs`, `crates/agent-session-compactor/tests/export_bundle.rs`, `docs/specs/hybrid-drift-sentinel-implementation-order.md`
