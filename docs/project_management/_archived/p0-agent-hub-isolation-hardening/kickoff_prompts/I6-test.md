# Task I6-test (`substrate world verify`) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I6-spec.md`, and this prompt.
3. Set `I6-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I6-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i6-world-verify-test
   git worktree add wt/ahih-i6-world-verify-test ahih-i6-world-verify-test
   cd wt/ahih-i6-world-verify-test
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I6-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I6-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add coverage for the new CLI wiring and JSON output stability.
- Prefer fixture-based tests (no dependence on a real provisioned world backend).

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell --tests -- --nocapture
```

## End Checklist
1. Confirm required checks are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I6-test`).
5. Remove worktree.


Do not edit planning docs inside the worktree.
