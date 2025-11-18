# Task D1-test – Test Agent Kickoff Prompt

Task ID: **D1-test** (Test new manager entries)

Summary:
- Consume the refreshed Tier-2 manifest from D1-code and extend the unit/integration suites so every new manager (mise/rtx, rbenv, sdkman, bun, volta, goenv, etc.) is covered. Parser tests should assert detect/env/script expansion, repair hints, and guest install metadata round-trip through `ManagerManifest`.
- Augment shim/shell tests to prove the pass-through pipeline uses the new entries: confirm `manager_init` writes snippets for the added managers, `substrate shim doctor` surfaces the expected states/hints, and the `manager_env.sh` sourcing flow remains unchanged.
- Capture one failure snapshot (if code hasn’t landed) so the Integration agent has the exact stderr/logs, then update docs/test notes accordingly.
- Production manifest changes stay in D1-code; this task focuses on automated validation, fixtures, and any helper scripts needed to simulate the new managers.

Focus files / context:
- `crates/common/src/manager_manifest.rs` tests (schema + env expansion).
- `crates/shell/tests/manager_init.rs`, `crates/shim/tests/integration.rs`, and related fixtures to exercise hint logging + snippet sourcing.
- Planning docs: `docs/project_management/next/substrate_isolated_shell_plan.md` (§5.9), `docs/project_management/next/substrate_isolated_shell_data_map.md`, and the updated manifest from D1-code.

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-common manager_manifest`
3. `cargo test -p substrate-shell manager_init`
4. `cargo test -p substrate-shim`

Reminders:
- Start at `AI_AGENT_START_HERE.md`, set D1-test to `in_progress` in `docs/project_management/next/tasks.json`, and log START/END entries in `docs/project_management/next/session_log.md`.
- Work from `wt/d1-managers-test`, referencing the D1-code prompt + manifest commit to know the expected structure. If code isn’t merged yet, capture the failing command output once before building the new tests.
- Before wrapping up, craft the Integration Agent Kickoff Prompt for `D1-integ`, run the commands above, commit/push from the worktree, switch back to `feat/isolated-shell-plan`, and update the session log/tasks with the prompt location.
