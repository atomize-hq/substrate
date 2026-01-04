# Task Y0-integ (Settings stack TOML→YAML) – INTEGRATION

## Start Checklist (feat/yaml-settings-migration)
1. `git checkout feat/yaml-settings-migration && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `Y0-spec.md`, and this prompt.
3. Set `Y0-integ` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start Y0-integ`).
4. Create branch/worktree:
   ```
   git checkout -b ysm-y0-settings-integ
   git worktree add wt/ysm-y0-settings-integ ysm-y0-settings-integ
   cd wt/ysm-y0-settings-integ
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Duties
- Merge `ysm-y0-settings-code` and `ysm-y0-settings-test`.
- Reconcile any drift so behavior matches `Y0-spec.md`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell --tests -- --nocapture
make integ-checks
```

## Smoke Scripts (required)
- Linux: `bash docs/project_management/next/yaml-settings-migration/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/next/yaml-settings-migration/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/next/yaml-settings-migration/smoke/windows-smoke.ps1`

## End Checklist
1. Commit integration changes.
2. Merge back to `feat/yaml-settings-migration` (ff-only).
3. Run the feature-local smoke script for your platform; capture output for the END entry.
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish Y0-integ`).
5. Remove worktree.


Do not edit planning docs inside the worktree.
