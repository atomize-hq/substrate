# Task C3-test – Test Agent Kickoff Prompt

Task ID: **C3-test** (Test installer changes)

Summary:
- Build an automated harness (bash or Python) under `tests/installers/` that exercises the updated installer/uninstaller scripts in temporary prefixes. Cover both default installs and the `--no-world` path to ensure PATH is untouched, `manager_env.sh`/`manager_init.sh` are generated, and `config.json` captures the correct `world_enabled` flag per `docs/project_management/next/substrate_isolated_shell_plan.md` (§5.3) and the data map.
- Validate that the installer’s `--sync-deps` reminder only appears when the world backend is provisioned, and that `--dry-run` output clearly lists skipped actions. Capture logs/artifacts so the Integration agent can rerun the same harness.
- Extend documentation/tests to assert that `scripts/substrate/uninstall-substrate.sh` removes the new artifacts without editing user PATH/rc files.

Focus files / context:
- New test harness directory (e.g., `tests/installers/install_smoke.sh`), plus any helper fixtures required to stub `$HOME`, `$SUBSTRATE_PREFIX`, or `SUBSTRATE_WORLD`.
- Existing scripts in `scripts/substrate/` so the harness invokes them exactly how users would.
- Reference docs: `docs/INSTALLATION.md`, `docs/UNINSTALL.md`, `docs/CONFIGURATION.md`, and Workstream C3 in `docs/project_management/next/substrate_isolated_shell_plan.md` + `subdata_map.md` for expected artifacts.

Commands to run:
1. `./tests/installers/install_smoke.sh --scenario default`
2. `./tests/installers/install_smoke.sh --scenario no-world`
3. `./tests/installers/install_smoke.sh --scenario uninstall`

(Feel free to adjust script names/flags, but document the exact commands you run in the session log and leave them runnable for Integration.)

Reminders:
- Work inside `wt/c3-installer-test`; set C3-test to `in_progress` in `docs/project_management/next/tasks.json` and log START/END entries before/after coding.
- Consume the notes left by C3-code (session log + prompt path) to understand how the installer signals success (files, config JSON, stdout). Add assertions for: host PATH unchanged, `manager_env.sh` chaining `manager_init.sh` + `.substrate_bashenv`, correct `world_enabled` flag, `substrate world enable` guidance when `--no-world` is used, and cleanup of artifacts.
- When done, craft the Integration Agent Kickoff Prompt (C3-integ) referencing the harness + required commands, commit/push the test worktree, and update coordination files per repo workflow.

Notes from C3-code:
- The installer now writes three runtime artifacts under the chosen prefix (default `~/.substrate`): `manager_init.sh` (placeholder that the CLI overwrites later), `manager_env.sh` (exports `SUBSTRATE_WORLD` + `SUBSTRATE_WORLD_ENABLED` and sources manager init + legacy `~/.substrate_bashenv`), and `config.json` (`{ "world_enabled": true/false }`). Dry runs print explicit `[substrate-install][dry-run] Write ...` lines without touching disk.
- Default runs provision the world backend, then print the paths to manager init/env/config plus guidance to add `<prefix>/bin` to PATH. `--no-world` skips provisioning/doctor/deps sync, writes the metadata with `world_enabled=false`, and logs the exact follow-up command (`<prefix>/bin/substrate world enable --prefix "<prefix>"`).
- No PATH/BASH_ENV snippets are ever written—verify `~/.zshrc`, `~/.bashrc`, etc. remain untouched and the dry-run log never mentions `.substrate_bashenv`.
- When running doctor/deps sync the script temporarily exports `PATH="<prefix>/bin:$ORIGINAL_PATH"`, `SHIM_ORIGINAL_PATH="$ORIGINAL_PATH"`, and `SUBSTRATE_ROOT="$PREFIX"`. The harness can stub these envs when invoking the script by setting `HOME`, `SUBSTRATE_PREFIX`, or passing `--prefix`.
- `manager_env.sh` contents should include the shebang, the two world exports, the three sourcing blocks (manager init, `SUBSTRATE_ORIGINAL_BASH_ENV`, and legacy `.substrate_bashenv`), and the “Managed by substrate-install” header with an ISO-8601 timestamp.
- Required env knobs for testing: `SUBSTRATE_PREFIX` (honored by downstream `substrate world enable` invocations), `SUBSTRATE_HOME`/`SUBSTRATE_MANAGER_ENV` (to redirect CLI reads), and `SUBSTRATE_WORLD_ENABLED` (read from the generated manager env). Dry runs should also show the installer referencing `SHIM_ORIGINAL_PATH`/`SUBSTRATE_ROOT` before calling `substrate world doctor`.
