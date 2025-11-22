# Task B1-test – Test Agent Kickoff Prompt

Task ID: **B1-test** (Test shim hinting)

Summary:
- Validate that `substrate-shim` loads the shared manager manifest, matches stderr patterns/scripts, and emits one `manager_hint` log per process tree when hints trigger.
- Exercise `--no-world` / `SUBSTRATE_WORLD=disabled` pass-through so shim execution skips PATH shimming + manager env injection and suppresses hint logging.
- Confirm hint suppression deduping and new fields maintain the documented log schema.

Focus & test targets:
- `crates/shim/tests/integration.rs` (add/update integration coverage near existing shim_logger tests).
- Manifest fixtures under `crates/shim/tests/fixtures` if needed.
- Logging utilities in `crates/shim/src/logger.rs` / `crates/shim/src/context.rs`.

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shim`

Notes & reminders:
- Start at `AI_AGENT_START_HERE.md`, update coordination files on `feat/isolated-shell-plan`, and record START/END entries in the session log.
- Work inside git worktree `wt/b1-shim-test`; keep `tasks.json` and `session_log.md` untouched from that worktree.
- Capture current `cargo test -p substrate-shim` output even if it fails until B1-code is merged; document results in the END log plus Integration Kickoff Prompt.
- Reference `docs/project_management/next/substrate_isolated_shell_plan.md` and the data map for schema/telemetry expectations.
