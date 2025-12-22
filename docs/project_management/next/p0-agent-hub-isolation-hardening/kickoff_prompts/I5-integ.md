# Task I5-integ (Docs + verification) – INTEGRATION

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I5-spec.md`, and this prompt.
3. Set `I5-integ` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I5-integ`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i5-docs-verify-integ
   git worktree add wt/ahih-i5-docs-verify-integ ahih-i5-docs-verify-integ
   cd wt/ahih-i5-docs-verify-integ
   ```

## Duties
- Merge `ahih-i5-docs-verify-code` and `ahih-i5-docs-verify-test`.
- Reconcile any drift so behavior matches `I5-spec.md`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets -- --nocapture
make preflight
```

## End Checklist
1. Commit integration changes.
2. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
3. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I5-integ`).
4. Remove worktree.

