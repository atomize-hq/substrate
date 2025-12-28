# Task I9-test (full cage robustness) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I9-spec.md`, and this prompt.
3. Set `I9-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I9-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i9-full-cage-verify-test
   git worktree add wt/ahih-i9-full-cage-verify-test ahih-i9-full-cage-verify-test
   cd wt/ahih-i9-full-cage-verify-test
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I9-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I9-spec.md`

## Scope & Guardrails
- Tests/fixtures only (no production code).
- Add/expand tests around full cage:
  - Regression for `/tmp`-rooted project/cwd execution in `world_fs.cage=full`.
  - Allowlist prefix patterns (e.g., `./writable/*`) behavior.
  - Outside-host read/write blocking.

## Required Commands
```
cargo fmt
# run targeted tests you add/touch (capture output for session log)
```

## End Checklist
1. Confirm required checks are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I9-test`).
5. Remove worktree.

