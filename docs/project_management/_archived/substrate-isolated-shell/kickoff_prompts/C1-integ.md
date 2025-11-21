# Task C1-integ – Integration Agent Kickoff Prompt

Task ID: **C1-integ** (Integrate world enable command)

Summary / current state:
- Code worktree `wt/c1-world-enable-code` should add the new `substrate world enable` CLI that shells out to `scripts/substrate/world-enable.sh`, updates `~/.substrate/config.json`, and writes the env exports (`SUBSTRATE_WORLD`, `SUBSTRATE_WORLD_ENABLED`) into `manager_env.sh`. It must also accept the test overrides (`SUBSTRATE_WORLD_ENABLE_SCRIPT`, `SUBSTRATE_WORLD_SOCKET`, `SUBSTRATE_PREFIX`) so we can point the CLI at fixture directories.
- Test worktree `wt/c1-world-enable-test` adds `crates/shell/tests/world_enable.rs`, a temp HOME/PREFIX harness that injects a fake helper script and validates success, helper failures, socket-missing failures, dry-run/noop, `--force` idempotency, verbose streaming, and corrupt-config recovery. These tests currently fail because the CLI subcommand does not exist yet (`error: unrecognized subcommand 'enable'`). Once the code branch lands, they should pass as-is.

What you need to do:
1. Start from `feat/isolated-shell-plan`, create/switch into `wt/c1-world-enable-integ`, and merge the code + test branches (`wt/c1-world-enable-code` and `wt/c1-world-enable-test`). Resolve any conflicts across `crates/shell`, `scripts/substrate`, and docs if the code branch introduced additional wiring (config helpers, env exports, etc.).
2. After merging, run `cargo fmt --all` (should already be clean) and `cargo test -p substrate-shell world_enable`. The new test suite expects:
   - CLI honors `SUBSTRATE_WORLD_ENABLE_SCRIPT` and `SUBSTRATE_WORLD_SOCKET` overrides.
   - Successful runs create/repair `~/.substrate/config.json` with `{ "world_enabled": true }` and rewrite `manager_env.sh` with the two exports mentioned above.
   - Failures (helper exit non-zero or socket never appears) leave config/env untouched and bubble up the helper’s log path along with a `substrate world doctor` reminder.
3. Manually sanity check `substrate world enable --dry-run --prefix <tmp> --profile release --verbose` using the new CLI to confirm it prints the command it would execute, the helper log location, and the follow-up doctor guidance.
4. Once tests + manual check pass, return to `feat/isolated-shell-plan`, record the results in `docs/project_management/next/session_log.md`, flip the C1-task statuses in `tasks.json`, and ensure this prompt + any conflict resolutions are committed.

Notes/assumptions to keep in mind:
- The tests rely on `python3` inside the helper script to create a fake UNIX socket; no additional tooling should be required, but make sure the CLI does not hard-code `/run/substrate.sock` without honoring `SUBSTRATE_WORLD_SOCKET`.
- The helper script surfaces `SUBSTRATE_TEST_WORLD_STDOUT/STDERR` lines when `--verbose` is set; keep stdout/stderr streaming intact in the CLI so the tests can see them.
- Do not touch coordination artifacts (`tasks.json`, `session_log.md`, prompts) from the integration worktree—only update them once you return to `feat/isolated-shell-plan` per the standard workflow.

Finish checklist:
1. Commit/push the merged code in `wt/c1-world-enable-integ` once tests pass.
2. Switch back to `feat/isolated-shell-plan`, append START/END log entries for C1-integ, and mark `C1-code`/`C1-test`/`C1-integ` appropriately in `tasks.json`.
3. Reference this prompt path in the session log and include the exact commands you ran (`cargo fmt --all`, `cargo test -p substrate-shell world_enable`, manual `substrate world enable --dry-run ...`).
