# Task C1-test – Test Agent Kickoff Prompt

Task ID: **C1-test** (Test `substrate world enable` command)

Summary / scope:
- Exercise the new `substrate world enable` CLI from `crates/shell` that shells out to `scripts/substrate/world-enable.sh`, passes along `--prefix/--profile/--dry-run/--verbose/--force`, watches for the world socket (or `substrate world doctor --json` output), and flips `~/.substrate/config.json` back to `{ "world_enabled": true }` plus exports `SUBSTRATE_WORLD=enabled` + `SUBSTRATE_WORLD_ENABLED=1` for future shells.
- Validate success, noop, and failure cases: (1) upgrade path when config is missing or `false`, (2) short-circuit messaging when already enabled, (3) provisioning helper exiting non-zero, (4) socket never appearing before timeout. Failures must not mutate config and should surface the captured log path as well as the suggested follow-up command (`substrate world doctor`).
- Ensure dry-run just prints the actions without touching disk, verbose mode streams the helper stdout/stderr (or log tail), and `--force` bypasses existing-state checks.

Where to focus:
- Tests live under `crates/shell/tests/` (recommended: a new `world_enable.rs` module alongside the shim doctor fixtures). Reuse the temp HOME prefix helper used in other CLI tests to isolate `~/.substrate` + config metadata.
- The helper script path is injected via env var `SUBSTRATE_WORLD_ENABLE_SCRIPT` (added by the code task) so tests can point to stub shell scripts inside the temp fixture directory. Provide both success/failure implementations.
- Inspect `crates/shell/src/commands/world_enable.rs`, `crates/shell/src/lib.rs`, and `scripts/substrate/world-enable.sh` to understand logging + env requirements; cross-reference the data map/config docs for JSON schema expectations.

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell world_enable`

Test design notes:
- Use `tempdir()` for HOME/PREFIX/LOG directories. Stub the helper script to append to a log file so you can assert CLI output references the file, and to create/remove a fake socket path under `${prefix}/run/substrate.sock` so the CLI’s health check passes.
- Assert CLI output strings: success should mention the prefix, the log path, `substrate world doctor`, and the config/env toggle summary. Failure should keep the old config JSON on disk and show the helper exit status.
- Verify `~/.substrate/config.json` is created with `{ "world_enabled": true }` on success, remains unchanged otherwise, and that rerunning without `--force` surfaces “already enabled” messaging.
- Cover dry-run (no file writes, but prints the command that would run) and verbose streaming (helper output forwarded). Use `Command::env_remove` to ensure `SUBSTRATE_WORLD` toggles propagate between invocations.

Reminders & finish steps:
- Start at `AI_AGENT_START_HERE.md`, update `tasks.json` + `session_log.md` (START/END) from `feat/isolated-shell-plan`, then switch to `wt/c1-world-enable-test` for code.
- Keep coordination artifacts untouched inside that worktree. Capture failing outputs if needed and document them in the END log entry.
- Before finishing, craft the Integration Agent Kickoff Prompt for `C1-integ`, store it at `docs/project_management/next/kickoff_prompts/C1-integ.md`, and note the path in the session log.
- Finish checklist: (1) commit/push `wt/c1-world-enable-test`, (2) return to `feat/isolated-shell-plan`, (3) append END entry + update `tasks.json`, (4) add/link the Integration prompt, (5) commit/push coordination updates.
