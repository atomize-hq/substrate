# Task I2-test (Full cage non-PTY pivot_root) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I2-spec.md`, and this prompt.
3. Set `I2-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I2-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i2-full-cage-nonpty-test
   git worktree add wt/ahih-i2-full-cage-nonpty-test ahih-i2-full-cage-nonpty-test
   cd wt/ahih-i2-full-cage-nonpty-test
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I2-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I2-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add coverage for non-PTY full cage behavior; skip cleanly when privileges/features are unavailable.

## Suggested Commands
```
cargo fmt
cargo test -p world -p world-agent -- --nocapture
```

## End Checklist
1. Confirm fmt + targeted tests are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I2-test`).
5. Remove worktree.
