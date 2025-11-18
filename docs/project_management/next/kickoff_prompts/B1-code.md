# Task B1-code – Code Agent Kickoff Prompt

Task ID: **B1-code** (Implement shim hinting and no-world bypass)

Summary:
- Teach `substrate-shim` to parse the manager manifest (same schema from A1/A2) so it can emit structured `manager_hint` events when stderr patterns or scripts match while avoiding duplicate hints per process tree.
- Thread the per-session `--no-world` / `SUBSTRATE_WORLD=disabled` state into the shim so pass-through executions skip PATH shim + manager env injection entirely when requested.
- Preserve existing telemetry/log schema while adding any new fields documented in the isolated-shell plan.

Focus files / context:
- `crates/shim/src/exec.rs`, `crates/shim/src/context.rs`, `crates/shim/src/logger.rs` (or whichever modules handle process exec + logging)
- Shared manifest types in `crates/common/src/manager_manifest.rs`
- Docs in `docs/project_management/next/substrate_isolated_shell_plan.md`, `.../substrate_isolated_shell_data_map.md`

Commands to run:
1. `cargo fmt --all`
2. `cargo check -p substrate-shim`

Reminders:
- Start at `AI_AGENT_START_HERE.md`, update `docs/project_management/next/tasks.json` + `session_log.md` on `feat/isolated-shell-plan`, and record START/END entries.
- Work inside `wt/b1-shim-code`; keep coordination artifacts untouched in that worktree.
- Before finishing, craft the Test Agent Kickoff Prompt for `B1-test` so the next agent can validate shim hinting/skip behavior.
