# Task I0-test (Strict policy schema) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I0-spec.md`, and this prompt.
3. Set `I0-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I0-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i0-policy-schema-test
   git worktree add wt/ahih-i0-policy-schema-test ahih-i0-policy-schema-test
   cd wt/ahih-i0-policy-schema-test
   ```

## Spec (shared with I0-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I0-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add coverage for parsing + validation + error messaging for `world_fs`.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-broker -- --nocapture
```

## End Checklist
1. Confirm fmt + targeted tests are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I0-test`).
5. Remove worktree.

