# Task I5-test (Docs + verification) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I5-spec.md`, and this prompt.
3. Set `I5-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I5-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i5-docs-verify-test
   git worktree add wt/ahih-i5-docs-verify-test ahih-i5-docs-verify-test
   cd wt/ahih-i5-docs-verify-test
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I5-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I5-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add minimal automated coverage for the surfaced schema/doctor fields and/or verification harness behavior.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell --tests -- --nocapture
```

## End Checklist
1. Confirm fmt + targeted tests are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I5-test`).
5. Remove worktree.


Do not edit planning docs inside the worktree.
