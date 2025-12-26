# Task I8-test (I1 noise reduction) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I8-spec.md`, and this prompt.
3. Set `I8-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I8-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i8-i1-noise-test
   git worktree add wt/ahih-i8-i1-noise-test ahih-i8-i1-noise-test
   cd wt/ahih-i8-i1-noise-test
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I8-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I8-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add fixture-based tests that assert on warning/error counts for:
  - world unavailable + require_world=false (single warning, command runs)
  - world unavailable + require_world=true (single error, command does not run)

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell --tests -- --nocapture
```

## End Checklist
1. Confirm required checks are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I8-test`).
5. Remove worktree.
