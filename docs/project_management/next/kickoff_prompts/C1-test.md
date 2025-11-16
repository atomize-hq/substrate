# Task C1-test – Test Agent Kickoff Prompt

Task ID: **C1-test** (Test `substrate world enable` command)

Summary:
- Add integration tests proving the new `substrate world enable` CLI upgrades no-world installs by invoking the provisioning helper, updating `~/.substrate/config.json`, and exporting the correct environment hints for subsequent shells.
- Exercise both success and failure paths: successful provisioning should mark `world_enabled=true`, surface a confirmation message, and optionally re-run `substrate world doctor`; failures (script exits non-zero, socket never appears) must bubble up actionable errors without mutating the config.
- Cover scenarios where the config already reports `world_enabled=true` (command should short-circuit with a friendly message) and where the config file is missing/corrupted but recoverable.

Focus files / context:
- `crates/shell/tests/` (create a dedicated `world_enable.rs` or extend `integration.rs` with new modules) using the existing temp HOME helpers from other CLI tests.
- Test fixtures that mimic the provisioning script invoked by `substrate world enable` (e.g., create a fake `scripts/substrate/world-enable.sh` inside a temp prefix and point the CLI at it via env vars/args exposed by the code task).
- Planning docs: `docs/project_management/next/substrate_isolated_shell_plan.md` (§5.3 / §5.6), `.../substrate_isolated_shell_data_map.md` (config schema + env toggles), `.../execution_plan.md` (Phase C expectations), plus the Code Agent prompt at `docs/project_management/next/kickoff_prompts/C1-code.md` for implementation details and any TODOs noted there.

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell world_enable`

Test expectations:
- Use temp directories for `HOME`, `SUBSTRATE_PREFIX`, and config paths so the tests never touch the developer’s real install.
- Mock the provisioning helper so the CLI writes a log file you can assert against (success path) and so you can force a failure exit status to validate error propagation; capture stdout/stderr snapshots to ensure they mention the log path and doctor follow-up instructions.
- Verify the config JSON now includes `{ "world_enabled": true }`, and that the CLI toggles `SUBSTRATE_WORLD_ENABLED` / `SUBSTRATE_WORLD` in the environment file as documented when the upgrade succeeds.
- Confirm idempotency: running the command twice should noop (and print a short message) once the config already signals an enabled world.

Reminders:
- Start on `feat/isolated-shell-plan`, set this task to `in_progress`, and append a START entry to `docs/project_management/next/session_log.md` before entering `wt/c1-world-enable-test`.
- Keep coordination files untouched inside the worktree. Coordinate closely with the code task’s artifacts (fixtures/exposed env vars) and record any assumptions or missing hooks in your END log entry.
- Before finishing, craft the Integration Agent Kickoff Prompt (C1-integ), store it under `docs/project_management/next/kickoff_prompts/C1-integ.md`, and reference it from the session log.
- Finish steps checklist: (1) commit/push `wt/c1-world-enable-test`, (2) switch to `feat/isolated-shell-plan`, (3) append END entry + update `tasks.json`, (4) add/link the Integration prompt, (5) commit/push coordination updates.
