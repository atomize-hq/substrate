# Task I4-test (Landlock optional layer) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I4-spec.md`, and this prompt.
3. Set `I4-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I4-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i4-landlock-test
   git worktree add wt/ahih-i4-landlock-test ahih-i4-landlock-test
   cd wt/ahih-i4-landlock-test
   ```

## Spec (shared with I4-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I4-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add detection/enforcement tests; skip cleanly on hosts without Landlock.

## Suggested Commands
```
cargo fmt
cargo test -p world --tests -- --nocapture
```

## End Checklist
1. Confirm fmt + targeted tests are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I4-test`).
5. Remove worktree.

