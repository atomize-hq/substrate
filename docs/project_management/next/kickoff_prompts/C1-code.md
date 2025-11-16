# Task C1-code – Code Agent Kickoff Prompt

Task ID: **C1-code** (Implement `substrate world enable` command)

Summary:
- Add a `substrate world enable` CLI flow that upgrades installs created with `--no-world` by running the provisioning script, waiting for the world agent/socket to become healthy, and flipping the install metadata (`~/.substrate/config.json`) back to `{"world_enabled":true}` while exporting `SUBSTRATE_WORLD=enabled` for future shells.
- Reuse the existing provisioning logic in `scripts/substrate/install-substrate.sh` (or extract the relevant steps into a new helper such as `scripts/substrate/world-enable.sh`) so the CLI shares code with the installer instead of duplicating shell snippets. Pass along `--prefix`, `--profile`, and `--dry-run` style flags so operators can preview the actions.
- Update `crates/shell/src/lib.rs` / `crates/shell/src/commands` so `ShellConfig` knows how to locate `~/.substrate/config.json`, read/write the `world_enabled` flag, and wire new CLI help text + argument parsing (e.g., `substrate world enable [--dry-run] [--verbose] [--force]`).
- Ensure the command reports clear status messages, logs world doctor output (or socket verification) on success, and returns actionable errors when provisioning fails.

Focus files / context:
- `crates/shell/src/lib.rs`, `crates/shell/src/commands/mod.rs` – CLI plumbing, `ShellMode`, argument parsing, help text, `SUBSTRATE_WORLD` handling.
- New/updated module (e.g., `crates/shell/src/commands/world_enable.rs`) for the command implementation plus `~/.substrate/config.json` helpers.
- `scripts/substrate/install-substrate.sh` (plus the new `scripts/substrate/world-enable.sh` shim, if needed) for provisioning logic. Coordinate with `scripts/substrate/dev-install-substrate.sh` if you add shared helpers.
- Planning docs: `docs/project_management/next/substrate_isolated_shell_plan.md` (§5.3, §5.6), `.../substrate_isolated_shell_data_map.md` (config metadata + env vars), `.../substrate_isolated_shell_execution_plan.md` (Phase C details).

Commands to run:
1. `cargo fmt --all`
2. `cargo check -p substrate-shell`

Reminders:
- Begin on `feat/isolated-shell-plan`: read `AI_AGENT_START_HERE.md`, set this task to `in_progress` in `docs/project_management/next/tasks.json`, and append a START entry in `session_log.md` before switching into `wt/c1-world-enable-code`.
- Work exclusively inside `wt/c1-world-enable-code` for code/script changes; leave coordination artifacts untouched in that worktree. Capture notes for the upcoming Test Agent (what was implemented, any TODOs, mock behaviors they need) and record the prompt path when you wrap up.
- Cover edge cases: installs that already have `world_enabled=true`, missing config files, script failures, or sockets that never appear. Ensure the command prints next steps (`substrate world doctor`) and surfaces the provisioning log path when something goes wrong.
- Finish steps checklist: (1) commit/push `wt/c1-world-enable-code`, (2) return to `feat/isolated-shell-plan`, (3) append END log entry + set task status to `completed`, (4) add/link the Test Agent Kickoff Prompt, (5) commit/push coordination updates.
