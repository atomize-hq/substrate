# Task C3-integ – Integration Agent Kickoff Prompt

Task ID: **C3-integ** (Integrate installer updates)

Summary / current state:
- Code worktree `wt/c3-installer-code` should update `scripts/substrate/install-substrate.sh`/`uninstall-substrate.sh` for the pass-through shell design: no host PATH edits, generate `manager_env.sh` that sources both `manager_init.sh` and the legacy `.substrate_bashenv`, persist `~/.substrate/config.json` with `{ "world_enabled": true/false }`, honor `--no-world`, and defer provisioning to `substrate world enable`.
- Test worktree `wt/c3-installer-test` adds `tests/installers/install_smoke.sh`, a bash harness that stubs `$HOME`, `$PATH`, and the system-level commands so the installer/uninstaller can run safely against fake releases. Scenarios currently fail because the installer still edits rc files, never writes manager metadata/config, and the dry-run output does not mention the new artifacts.
  - `--scenario default` runs a dry-run (no-world) to inspect the listed actions, then installs with world provisioning enabled and expects: rc files untouched, `~/.substrate/manager_env.sh` chaining `manager_init.sh` + `.substrate_bashenv`, `config.json` with `world_enabled: true`, and the `--sync-deps` reminder when the world backend provisioned.
  - `--scenario no-world` installs with `--no-world` and asserts `manager_env.sh` exports `SUBSTRATE_WORLD=disabled`/`SUBSTRATE_WORLD_ENABLED=0`, `config.json` reads `false`, the install log contains `substrate world enable` guidance, and there is no `substrate world deps sync` reminder.
  - `--scenario uninstall` seeds a no-world install, runs the uninstaller, and verifies the `~/.substrate` artifacts plus fake systemd/world-agent paths are removed without touching user rc files.

What you need to do:
1. Create/switch to `wt/c3-installer-integ` from `feat/isolated-shell-plan`, merge `wt/c3-installer-code` and `wt/c3-installer-test`, and resolve conflicts in the bash scripts + docs (`docs/INSTALLATION.md`, `docs/UNINSTALL.md`, `docs/CONFIGURATION.md`).
2. Run the harness from repo root:
   - `./tests/installers/install_smoke.sh --scenario default`
   - `./tests/installers/install_smoke.sh --scenario no-world`
   - `./tests/installers/install_smoke.sh --scenario uninstall`
   Use `--keep-temp` if you need to inspect the generated temp directories (the harness prints their locations on success). Each scenario should finish cleanly only when the installer keeps rc files untouched, writes the manager env/init pair + config JSON with the correct `world_enabled` flag, emits the `substrate world deps sync --all` reminder only after full provisioning, and ensures the uninstaller removes the new artifacts without editing user PATH/rc files.
3. Spot-check the real scripts if needed by pointing them at a local release archive or the dev installer to confirm `manager_env.sh` contains:
   ```bash
   export SUBSTRATE_WORLD=enabled
   export SUBSTRATE_WORLD_ENABLED=1
   # shellcheck disable=SC1090
   source "$HOME/.substrate/manager_init.sh"
   # shellcheck disable=SC1090
   source "$HOME/.substrate_bashenv"
   ```
   and that `config.json` flips between `true`/`false` when running the installer with and without `--no-world`.

Notes / assumptions:
- The harness builds a fake release tarball per run and stubs commands like `sudo`, `systemctl`, `fuse-overlayfs`, `nft`, `ip`, etc., so no host state is modified. Use the harness logs under `/tmp/substrate-installer-*/` for debugging when assertions fail.
- Default scenarios still call `substrate --shim-deploy` and `substrate world doctor --json`, so ensure the code branch keeps those behaviors working with the stub binary (`SUBSTRATE_ROOT` should be set automatically by the installer). Keep the `--sync-deps` flag wired to the new `world deps` CLI.

Finish checklist:
1. After successful runs, commit/push the merged integration worktree and return to `feat/isolated-shell-plan`.
2. Append START/END entries for C3-integ in `docs/project_management/next/session_log.md`, update `docs/project_management/next/tasks.json` (C3-code/C3-test/C3-integ statuses), and reference this prompt plus the harness commands in the log.
3. Include the harness output (or pointers to the preserved temp dirs if `--keep-temp` was used) in the integration notes/PR so downstream agents can reproduce the installer behavior.
