# Task Y0-test (Settings stack TOML→YAML) – TEST

## Start Checklist (feat/yaml-settings-migration)
1. `git checkout feat/yaml-settings-migration && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `Y0-spec.md`, and this prompt.
3. Set `Y0-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start Y0-test`).
4. Create branch/worktree:
   ```
   git checkout -b ysm-y0-settings-test
   git worktree add wt/ysm-y0-settings-test ysm-y0-settings-test
   cd wt/ysm-y0-settings-test
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with Y0-code)
- `docs/project_management/next/yaml-settings-migration/Y0-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Update/add tests for YAML settings + config CLI behavior.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell --tests -- --nocapture
```

## End Checklist
1. Confirm fmt + targeted tests are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/yaml-settings-migration` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish Y0-test`).
5. Remove worktree.
