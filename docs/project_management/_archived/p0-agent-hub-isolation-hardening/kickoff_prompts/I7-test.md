# Task I7-test (Manual playbook alignment) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I7-spec.md`, and this prompt.
3. Set `I7-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I7-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i7-playbook-align-test
   git worktree add wt/ahih-i7-playbook-align-test ahih-i7-playbook-align-test
   cd wt/ahih-i7-playbook-align-test
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I7-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I7-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add a lightweight test that prevents obvious playbook drift (at minimum: `.substrate-profile` snippets
  must include `id` and `name`).
- Prefer a unit/integration test under an existing test crate; do not require a real world backend.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell --tests -- --nocapture
```

## End Checklist
1. Confirm required checks are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I7-test`).
5. Remove worktree.


Do not edit planning docs inside the worktree.
