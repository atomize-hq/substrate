# Task S0-code (Bundle manager manifest in releases) – CODE

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Read `settings_stack_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `S0-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start S0-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b ss-s0-manifest-code
   git worktree add wt/ss-s0-manifest-code ss-s0-manifest-code
   cd wt/ss-s0-manifest-code
   ```

## Scope
- Update release packaging so config/manifests are bundled:
  - `dist/scripts/collect-supporting-artifacts.sh`: copy `config/` contents.
  - `dist/scripts/assemble-release-bundles.sh`: ensure bundles include
    `config/manager_hooks.yaml` (and any other required manifests like
    `scripts/substrate/world-deps.yaml`).
- Update installers/uninstallers to deploy the new config directory:
  - `scripts/substrate/install-substrate.sh` + wrapper `install.sh`.
  - `scripts/substrate/uninstall.sh` (remove config dir when cleaning up).
- Refresh docs (`docs/INSTALLATION.md`, `docs/CONFIGURATION.md`, `docs/UNINSTALL.md`)
  to explain where the manifest lives post-install.
- No tests in this task; keep scope to production code/scripts/docs.

## Suggested Commands
```
# Keep scripts linted
cargo fmt
# Optional: run shellcheck on touched scripts if available
bash -n scripts/substrate/install-substrate.sh
./tests/installers/install_smoke.sh --scenario default --dry-run   # optional sanity
```
(Only run the smoke script if needed to validate behavior; document any runs in the END log entry.)

## End Checklist
1. Ensure fmt/lints succeed; capture any manual test outputs.
2. Commit worktree changes (`git commit -am "feat: bundle manager manifest"`).
3. Return to repo root, merge the worktree back:
   ```
   git checkout feat/settings-stack
   git merge --ff-only wt/ss-s0-manifest-code
   ```
4. Update `tasks.json` (status → `completed`) and append an END entry to the
   session log summarizing commands/results.
5. Reference the S0-test kickoff prompt path in the log and note any follow-up instructions.
6. Commit docs/log updates (`git commit -am "docs: finish S0-code"`).
7. Remove the worktree (`git worktree remove wt/ss-s0-manifest-code`).

## Deliverables
- Production changes staged on `feat/settings-stack` with manifest bundling enabled.
- Docs updated describing manifest locations.
- Session log entry containing command outputs and references to the kickoff prompt for S0-test.
