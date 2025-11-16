# Task D1-test – Test Agent Kickoff Prompt

Task ID: **D1-test** (Test new manager entries)

Summary:
- Extend parser/unit tests to cover every new Tier-2 manifest entry (mise/rtx, rbenv, sdkman, bun, volta, goenv, etc.), ensuring detect/env/file/script expansions round-trip through `ManagerManifest`.
- Add shim/shell integration coverage that exercises at least one new manager hint and validates that the generated snippets surface inside Substrate sessions without touching host files.
- Document current test expectations (especially for hint logging) so the integration agent can re-run the same suites after code + tests merge.
- Keep production manifest edits untouched in this task—focus on tests/fixtures.

Focus files / context:
- `crates/common/src/manager_manifest.rs` + its tests (schema + env expansion helpers)
- `crates/shim/tests/` and any new fixtures required for hint logging
- Planning references: `docs/project_management/next/substrate_isolated_shell_plan.md`, `.../substrate_isolated_shell_data_map.md`, and the new D1-code manifest changes

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-common manager_manifest`
3. `cargo test -p substrate-shim`

Reminders:
- Begin at `AI_AGENT_START_HERE.md`, update `docs/project_management/next/tasks.json` + `session_log.md` (START entry) on `feat/isolated-shell-plan` before touching tests.
- Work entirely inside git worktree `wt/d1-managers-test`; consume the Kickoff Prompt + manifest from D1-code to understand the required coverage.
- Capture failing test output (if D1-code hasn’t landed yet) once for the integration agent, then proceed with your implementation. Before finishing, craft the Integration Agent Kickoff Prompt for `D1-integ` and log where it lives.
- Finish steps: run the commands above, commit/push from the worktree, return to `feat/isolated-shell-plan`, append END entry/status updates, and ensure the new kickoff prompt references are recorded.
