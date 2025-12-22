# Task I2-integ (Full cage non-PTY pivot_root) – INTEGRATION

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I2-spec.md`, and this prompt.
3. Set `I2-integ` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I2-integ`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i2-full-cage-nonpty-integ
   git worktree add wt/ahih-i2-full-cage-nonpty-integ ahih-i2-full-cage-nonpty-integ
   cd wt/ahih-i2-full-cage-nonpty-integ
   ```

## Duties
- Merge `ahih-i2-full-cage-nonpty-code` and `ahih-i2-full-cage-nonpty-test`.
- Reconcile any drift so behavior matches `I2-spec.md`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p world -p world-agent -- --nocapture
make preflight
```

## End Checklist
1. Commit integration changes.
2. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
3. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I2-integ`).
4. Remove worktree.

