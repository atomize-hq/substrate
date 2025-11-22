# Task B3-integ – Integration Agent Kickoff Prompt

Task ID: **B3-integ** (Review documentation updates)

Summary:
- Merge the B3 docs worktree (`wt/b3-docs`) into `feat/isolated-shell-plan`, verify that the refreshed guides correctly describe the pass-through shim model, manager manifest/overlay configuration, shim doctor/repair usage, and world enable/deps concepts, and ensure coordination files remain untouched in the per-task tree.
- Re-run the documentation checks (`cargo fmt --all` for parity and the markdownlint sweep) to confirm nothing regressed; note that the existing docs/data-map still contain long historical lines, so MD013 warnings are expected until legacy sections are reformatted.
- Sanity check the CLI instructions by spot-reading README, `docs/INSTALLATION.md`, `docs/USAGE.md`, `docs/CONFIGURATION.md`, CHANGELOG, and the execution plan entry to ensure all new references align with the merged code work in B1/B2.

Focus files / context:
- `README.md`, `docs/INSTALLATION.md`, `docs/USAGE.md`, `docs/CONFIGURATION.md`
- `CHANGELOG.md`, `docs/project_management/next/substrate_isolated_shell_execution_plan.md`
- Coordination references in `docs/project_management/next/session_log.md` + `tasks.json` (already updated on `feat/isolated-shell-plan`)

Commands to run:
1. `cargo fmt --all`
2. `npx markdownlint-cli README.md docs/INSTALLATION.md docs/USAGE.md docs/CONFIGURATION.md docs/project_management/next/substrate_isolated_shell_plan.md docs/project_management/next/substrate_isolated_shell_data_map.md`

Reminders:
- Start at `AI_AGENT_START_HERE.md`, mark B3-integ as `in_progress` in `docs/project_management/next/tasks.json`, and add a START entry to `docs/project_management/next/session_log.md` before touching the integration worktree.
- Work inside `wt/b3-docs-integ`, merge the code worktree (`wt/b3-docs`) into it, and resolve any conflicts; avoid editing coordination files from inside the integration tree.
- Capture the markdownlint output: MD013 line-length warnings already exist across legacy docs (`docs/project_management/next/substrate_isolated_shell_data_map.md`, plan, README); document the known warnings rather than reflowing large sections unless instructed otherwise.
- After verifying docs and commands, merge the integration worktree back into `feat/isolated-shell-plan`, run the finish checklist (commit/push code, switch to coordination branch, append END log entry, update task status to `completed`, and record this prompt’s location for future reference).
