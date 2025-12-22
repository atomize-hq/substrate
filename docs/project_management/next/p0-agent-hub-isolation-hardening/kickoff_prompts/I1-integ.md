# Task I1-integ (Fail-closed semantics) – INTEGRATION

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I1-spec.md`, and this prompt.
3. Set `I1-integ` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I1-integ`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i1-fail-closed-integ
   git worktree add wt/ahih-i1-fail-closed-integ ahih-i1-fail-closed-integ
   cd wt/ahih-i1-fail-closed-integ
   ```

## Duties
- Merge `ahih-i1-fail-closed-code` and `ahih-i1-fail-closed-test`.
- Reconcile any drift so behavior matches `I1-spec.md`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell --tests -- --nocapture
make preflight
```

## End Checklist
1. Commit integration changes.
2. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
3. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I1-integ`).
4. Remove worktree.

