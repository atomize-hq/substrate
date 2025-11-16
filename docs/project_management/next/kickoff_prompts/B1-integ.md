# Task B1-integ – Integration Agent Kickoff Prompt

Task ID: **B1-integ** (Integrate shim hinting)

Summary:
- Merge the shim implementation updates from `wt/b1-shim-code` with the new integration coverage in `wt/b1-shim-test` so hint logging + no-world bypass can land together.
- The fresh tests spin up temp shim/bin dirs, inject miniature manager manifests, and assert that failing commands emit structured `manager_hint` log entries while honoring the `SUBSTRATE_WORLD`/`SUBSTRATE_SHIM_HINTS` toggles. A second test confirms that setting `SUBSTRATE_WORLD_ENABLED=false` skips hint emission entirely (simulating the pass-through path).
- Expect the suite to fail until the code branch finishes wiring output capture + manifest-driven hint matching. Once merged, both tests should pass and provide regression coverage for manager doctor work.

Focus files / context:
- `crates/shim/tests/integration.rs` (new `manager_hint_logging_records_entry` + `manager_hint_skipped_when_world_disabled` cases)
- Shim runtime sources in `crates/shim/src/exec.rs`, `crates/shim/src/logger.rs`, and related manifest plumbing from B1-code
- Planning references: `docs/project_management/next/substrate_isolated_shell_plan.md` + `.../data_map.md` for expected env vars and logging schema

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shim`

Reminders:
- Start at `AI_AGENT_START_HERE.md`, update `tasks.json`/`session_log.md` while on `feat/isolated-shell-plan`, then integrate via worktree `wt/b1-shim-integ` (or equivalent merge tree) without touching coordination files there.
- Capture a sample trace line showing `manager_hint` after the merge succeeds; attach it in the session log so follow-up teams can reuse it for telemetry validation.
- Leave the tree ready for the next workstream (B2) by keeping shim-specific changes self-contained and ensuring the integration worktree is clean after tests pass.
