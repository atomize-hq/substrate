# Task I1-test (Fail-closed semantics) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I1-spec.md`, and this prompt.
3. Set `I1-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I1-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i1-fail-closed-test
   git worktree add wt/ahih-i1-fail-closed-test ahih-i1-fail-closed-test
   cd wt/ahih-i1-fail-closed-test
   ```

## Spec (shared with I1-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I1-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add integration coverage proving required-world refuses host fallback.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell --tests -- --nocapture
```

## End Checklist
1. Confirm fmt + targeted tests are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I1-test`).
5. Remove worktree.

