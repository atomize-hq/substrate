# Task A2-integ – Integration Agent Kickoff Prompt

Task ID: **A2-integ** (Integrate manager init worktrees)

Summary:
- Merge `wt/a2-manager-init-code` and `wt/a2-manager-init-test` so the new manager init module plus shell wiring tests land together.
- Ensure `crates/shell/src/manager_init.rs` covers skip env flags, detection helpers (files/commands/env/script), snippet/telemetry generation, and `write_snippet` atomic writes.
- Verify the shell helpers in `crates/shell/src/lib.rs` (`configure_manager_init`, `manager_manifest_base_path`, overlay path `manager_hooks.local.yaml`, and `SUBSTRATE_MANAGER_INIT` export) behave once both branches are merged.

Focus files:
- `crates/shell/src/manager_init.rs`
- `crates/shell/src/lib.rs`
- `docs/project_management/next/kickoff_prompts/A2-test.md` (context on test scope)

Commands to run:
1. `cargo fmt --all`
2. `cargo clippy -p substrate-shell -- -D warnings`
3. `cargo test -p substrate-shell manager_init`

Reminders:
- Start at `AI_AGENT_START_HERE.md`, update `tasks.json` + `session_log.md`, and capture START/END entries on `feat/isolated-shell-plan`.
- Work inside `wt/a2-manager-init-integ`, bring in both worktrees, resolve conflicts, and keep coordination artifacts off that worktree.
- Record command output + follow-ups in the END log entry and confirm the prompt reference for the next phase if needed.
