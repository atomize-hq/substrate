# Task C2-integ – Integration Agent Kickoff Prompt

Task ID: **C2-integ** (Integrate world deps CLI)

Summary / current state:
- Code worktree `wt/c2-world-deps-code` is expected to add the new `substrate world deps` command group (`status`, `install`, `sync`) wired into the manager/world manifest loader plus the world backend executor hooks. It must honor `SUBSTRATE_WORLD_DEPS_MANIFEST`, look for the overlay file at `~/.substrate/world-deps.local.yaml`, and surface warnings/errors when the world backend is disabled.
- Test worktree `wt/c2-world-deps-test` introduces `crates/shell/tests/world_deps.rs`. The harness builds a fake HOME/PREFIX, seeds base + overlay manifests (JSON that matches the manager-manifest schema), plants detection/install helper scripts, and validates:
  - `world deps status` prints host vs guest availability even if the world is disabled (but emits a warning) and labels entries as `host=present` / `guest=missing`.
  - `world deps install` streams helper output, respects `--dry-run`, surfaces helper failures, and errors when `SUBSTRATE_WORLD=disabled` or `--no-world` is set.
  - `world deps sync --all` installs only the host-present tools that are missing in the guest.
  - Overlay manifests in `~/.substrate/world-deps.local.yaml` override the base install recipe.
- `cargo test -p substrate-shell world_deps` currently fails because `substrate world` does not recognize the `deps` subcommand yet:
  ```
  error: Found argument 'deps' which wasn't expected, or isn't valid in this context
  
  Usage: substrate world <COMMAND>
  ```
  Once the CLI lands, the tests should cover the flows above using the stub scripts/logs.

What you need to do:
1. From `feat/isolated-shell-plan`, create/switch into `wt/c2-world-deps-integ` and merge `wt/c2-world-deps-code` + `wt/c2-world-deps-test`. Resolve conflicts in `crates/shell/src/commands`, manifest helpers, and any new helper scripts.
2. Run `cargo fmt --all` (should already be clean) and `cargo test -p substrate-shell world_deps`. These tests call the `substrate` binary directly, so ensure the new CLI honors:
   - `SUBSTRATE_WORLD_DEPS_MANIFEST` when pointing at the temp JSON manifest, and auto-loads `~/.substrate/world-deps.local.yaml` as the overlay.
   - Temp env vars the harness injects (`SUBSTRATE_WORLD_DEPS_HOST_LOG`, `_GUEST_LOG`, `_EXECUTOR_LOG`, `_MARKER_DIR`) so the fake scripts can log/check state.
   - `--dry-run`, `--verbose`, `--no-world`, and `SUBSTRATE_WORLD=disabled` toggles.
3. Optionally run a manual sanity check such as `SUBSTRATE_WORLD_DEPS_MANIFEST=/tmp/manifest.yaml target/debug/substrate world deps status --verbose` using a trimmed version of the fixture manifest to confirm the CLI output format before/after merge.
4. When everything passes, return to `feat/isolated-shell-plan`, document the END entry + task status updates, and commit coordination artifacts.

Notes / assumptions:
- The manifest format mirrors the existing manager manifest: `version` + `managers` map with `detect.commands`, `guest_detect.command`, and `guest_install.{apt,custom}`. The tests serialize JSON, but it is parsed via `serde_yaml`.
- Overlay support is required: `~/.substrate/world-deps.local.yaml` replaces fields per tool (the tests expect `guest_install.custom` to override the base recipe entirely).
- The helper scripts look at the env vars listed above to write log files and touch marker files. Preserve arbitrary env vars when invoking the guest executor so these values flow through.
- Tests assume the CLI still reports host info when the world backend is disabled but refuses to mutate state (`install`/`sync` should exit with actionable errors in that mode).

Finish checklist:
1. Commit/push the merged integration worktree after tests pass.
2. Switch back to `feat/isolated-shell-plan`, append START/END entries for C2-integ, and mark `C2-code`/`C2-test`/`C2-integ` as appropriate in `docs/project_management/next/tasks.json`.
3. Reference this prompt path plus the executed commands (`cargo fmt --all`, `cargo test -p substrate-shell world_deps`, optional manual status/install invocation) in the session log.
