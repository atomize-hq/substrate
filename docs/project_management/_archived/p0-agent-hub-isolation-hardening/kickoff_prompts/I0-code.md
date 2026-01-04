# Task I0-code (Strict policy schema) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I0-spec.md`, and this prompt.
3. Set `I0-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I0-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i0-policy-schema-code
   git worktree add wt/ahih-i0-policy-schema-code ahih-i0-policy-schema-code
   cd wt/ahih-i0-policy-schema-code
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I0-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I0-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Implement strict `world_fs` schema + validation + broker outputs needed for enforcement.
- Breaking schema changes are acceptable (greenfield), but error messages must be actionable.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm fmt/clippy are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I0-code`).
5. Remove worktree.


Do not edit planning docs inside the worktree.
