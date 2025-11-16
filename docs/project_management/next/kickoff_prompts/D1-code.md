# Task D1-code – Code Agent Kickoff Prompt

Task ID: **D1-code** (Add Tier-2 manager entries)

Summary:
- Populate `config/manager_hooks.yaml` with the Tier-2 managers called out in the plan (mise/rtx, rbenv, sdkman, bun, volta, goenv, etc.), including detect/init/repair/guest sections that match the data map schema.
- Keep comments + ordering consistent with the Tier-1 entries so future overlays remain readable, and ensure every detect rule follows the expansion helpers (tilde/env) already documented.
- Update any supporting docs (plan, data map, file audit) if new managers introduce behaviors or guest install requirements that aren’t currently described.
- Leave testing to the paired D1-test task; focus on shipping a valid manifest + docs.

Focus files / context:
- `config/manager_hooks.yaml` (primary deliverable; keep version header and comments intact)
- Planning references: `docs/project_management/next/substrate_isolated_shell_plan.md` (§5.9), `.../substrate_isolated_shell_data_map.md`, `.../substrate_isolated_shell_file_audit.md`
- Existing Tier-1 manifest entries + schema helpers in `crates/common/src/manager_manifest.rs` for field names/constraints

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-common manager_manifest`

Reminders:
- Start from `AI_AGENT_START_HERE.md`, set this task to `in_progress` in `docs/project_management/next/tasks.json`, and add a START entry in `docs/project_management/next/session_log.md` before editing.
- Work inside git worktree `wt/d1-managers-code`; avoid touching coordination files there. Capture any schema surprises or open questions in the session log.
- When finished: run the commands above, commit/push changes in the worktree, switch back to `feat/isolated-shell-plan`, append an END entry + status update, and reference this kickoff prompt for the paired test agent.
