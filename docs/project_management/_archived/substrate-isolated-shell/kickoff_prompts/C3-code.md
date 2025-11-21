# Task C3-code – Code Agent Kickoff Prompt

Task ID: **C3-code** (Update installer/uninstaller scripts)

Summary:
- Refresh the host installers so they match the new “pass-through shell” design described in `docs/project_management/next/substrate_isolated_shell_plan.md` (§5.3) and the execution plan’s Workstream C3. The bash installer must stop modifying the user’s PATH globally, generate the new `manager_env.sh`/`manager_init.sh` artifacts, honor the `--no-world` flag (skip provisioning, write `world_enabled=false`), and delegate follow-up provisioning to `substrate world enable`.
- Persist install metadata in `~/.substrate/config.json` (see `docs/project_management/next/substrate_isolated_shell_data_map.md`) so the shell/CLI can detect whether the world backend is enabled. The script should set `SUBSTRATE_WORLD_ENABLED=1/0` exports inside `manager_env.sh`, and the uninstall script should clean up these files without touching user shell rc files.
- Align documentation (`docs/INSTALLATION.md`, `docs/USAGE.md`, `docs/CONFIGURATION.md`, `docs/UNINSTALL.md`) with the new workflow: explain `--no-world`, reference `substrate world enable`, and clarify that PATH edits are no longer required.
- Ensure the Linux/macOS install paths both invoke the new helper: default runs keep world provisioning + optional `world deps sync`, while `--no-world` installs only copy binaries and print guidance. Keep `--dry-run` output informative for the Test/Integration agents.

Focus files / context:
- `scripts/substrate/install-substrate.sh`, `scripts/substrate/uninstall-substrate.sh`, and any platform-specific helpers these scripts call (mac/Linux sections, `world-enable.sh`).
- Docs mentioned above plus any release notes that cite installer flags.
- Existing metadata helpers (`manager_env.sh`, `config.json`) referenced in Workstreams A3/C1 so behavior stays consistent.

Commands to run:
1. `./scripts/substrate/install-substrate.sh --dry-run --prefix /tmp/substrate-c3 --sync-deps`
2. `./scripts/substrate/install-substrate.sh --dry-run --prefix /tmp/substrate-c3-no-world --no-world`
3. `./scripts/substrate/uninstall-substrate.sh --dry-run --prefix /tmp/substrate-c3`

Reminders:
- Work in `wt/c3-installer-code`; update `docs/project_management/next/tasks.json` (set C3-code to `in_progress`) and add START/END entries to `docs/project_management/next/session_log.md` before/after coding.
- Leave detailed notes for the Test agent: which env vars the installer inspects (`SUBSTRATE_PREFIX`, `SUBSTRATE_ROOT`, `SUBSTRATE_WORLD`, etc.), how to detect that PATH is untouched, what files should exist after `--no-world` vs default dry runs, and any tricky platform branches.
- When finished, commit/push from the code worktree, record this prompt path in the session log, and capture the exact dry-run commands (and any additional verification you performed) for the Integration agent.
