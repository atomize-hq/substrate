# Task I4-integ (Landlock additive hardening) – INTEGRATION

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I4-spec.md`, and this prompt.
3. Set `I4-integ` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I4-integ`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i4-landlock-integ
   git worktree add wt/ahih-i4-landlock-integ ahih-i4-landlock-integ
   cd wt/ahih-i4-landlock-integ
   ```

## Duties
- Merge `ahih-i4-landlock-code` and `ahih-i4-landlock-test`.
- Reconcile any drift so behavior matches `I4-spec.md`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p world --tests -- --nocapture
make preflight
```

## End Checklist
1. Commit integration changes.
2. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
3. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I4-integ`).
4. Remove worktree.
