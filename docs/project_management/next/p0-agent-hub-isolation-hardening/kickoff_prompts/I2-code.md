# Task I2-code (Full cage non-PTY pivot_root) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I2-spec.md`, and this prompt.
3. Set `I2-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I2-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i2-full-cage-nonpty-code
   git worktree add wt/ahih-i2-full-cage-nonpty-code ahih-i2-full-cage-nonpty-code
   cd wt/ahih-i2-full-cage-nonpty-code
   ```

## Spec (shared with I2-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I2-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Implement full cage for non-PTY execution on Linux (mount ns + pivot_root) with capability detection and fail-closed semantics.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm fmt/clippy are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I2-code`).
5. Remove worktree.

