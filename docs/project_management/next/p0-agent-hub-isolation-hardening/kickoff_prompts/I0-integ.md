# Task I0-integ (Strict policy schema) – INTEGRATION

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I0-spec.md`, and this prompt.
3. Set `I0-integ` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I0-integ`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i0-policy-schema-integ
   git worktree add wt/ahih-i0-policy-schema-integ ahih-i0-policy-schema-integ
   cd wt/ahih-i0-policy-schema-integ
   ```

## Duties
- Merge `ahih-i0-policy-schema-code` and `ahih-i0-policy-schema-test`.
- Reconcile any drift so behavior matches `I0-spec.md`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-broker -- --nocapture
make preflight
```

## End Checklist
1. Commit integration changes.
2. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
3. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I0-integ`).
4. Remove worktree.

