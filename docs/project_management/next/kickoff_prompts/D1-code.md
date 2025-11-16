# Task D1-code – Code Agent Kickoff Prompt

Task ID: **D1-code** (Add Tier-2 manager entries)

Summary:
- Expand `config/manager_hooks.yaml` with the Tier-2 managers identified in the Workstream D1 plan (mise/rtx, rbenv, sdkman, bun, volta, goenv, asdf-node, etc.). Each entry must provide complete `detect`, `init`, `repair_hint`, and `guest` blocks that align with the schema in `docs/project_management/next/substrate_isolated_shell_data_map.md`.
- Ensure detection heuristics take advantage of the existing helpers (tilde/env expansion, `files` + `commands` arrays, optional custom `script`) and avoid host mutations. Snippets and repair hints need to work under the pass-through model (`manager_env.sh` sources them before user rc files).
- Populate the `guest` install metadata so later Workstreams (C2/D2) can expose the same managers through `world deps` and doctor/health summaries. Document any new guest prerequisites or platform quirks in the plan/data map as needed.
- Keep comments/version headers consistent, and group Tier-2 entries logically (matching Tier-1 ordering) so overlays remain stable. Testing lives in D1-test; this task owns manifest + doc updates only.

Focus files / context:
- `config/manager_hooks.yaml` (primary deliverable; maintain version header, section comments, and manifest ordering).
- Planning references: `docs/project_management/next/substrate_isolated_shell_plan.md` (§5.9 “Tier-2 managers”), `docs/project_management/next/substrate_isolated_shell_data_map.md`, and `docs/project_management/next/substrate_isolated_shell_file_audit.md`.
- Schema helpers in `crates/common/src/manager_manifest.rs` plus recent manager-related docs (`docs/USAGE.md`, `docs/CONFIGURATION.md`) to keep terminology aligned.

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-common manager_manifest`
3. (Optional sanity check) `cargo test -p substrate-shell manager_init`

Reminders:
- Begin at `AI_AGENT_START_HERE.md`, set D1-code to `in_progress` in `docs/project_management/next/tasks.json`, and log a START entry in `docs/project_management/next/session_log.md` before changing files.
- Work entirely inside git worktree `wt/d1-managers-code`; capture any manifest/schema questions in the session log so the Test agent can consume them.
- When finished, run the commands above, commit/push from the worktree, switch back to `feat/isolated-shell-plan`, append an END entry + status update, and reference this kickoff prompt so D1-test knows where the manifest landed.
