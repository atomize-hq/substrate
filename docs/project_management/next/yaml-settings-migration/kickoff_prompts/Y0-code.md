# Task Y0-code (Settings stack TOML→YAML) – CODE

## Start Checklist (feat/yaml-settings-migration)
1. `git checkout feat/yaml-settings-migration && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `Y0-spec.md`, and this prompt.
3. Set `Y0-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start Y0-code`).
4. Create branch/worktree:
   ```
   git checkout -b ysm-y0-settings-code
   git worktree add wt/ysm-y0-settings-code ysm-y0-settings-code
   cd wt/ysm-y0-settings-code
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with Y0-test)
- `docs/project_management/next/yaml-settings-migration/Y0-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Convert the settings/config stack to YAML and update the config CLI.
- No dual-format support; prefer actionable errors when TOML is present.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm fmt/clippy are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/yaml-settings-migration` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish Y0-code`).
5. Remove worktree.
