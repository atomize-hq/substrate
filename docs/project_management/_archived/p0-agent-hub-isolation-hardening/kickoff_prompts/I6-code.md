# Task I6-code (`substrate world verify`) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I6-spec.md`, and this prompt.
3. Set `I6-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I6-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i6-world-verify-code
   git worktree add wt/ahih-i6-world-verify-code ahih-i6-world-verify-code
   cd wt/ahih-i6-world-verify-code
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I6-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I6-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Implement `substrate world verify` per spec, including any required JSON output schema and platform guards.
- Keep output actionable; prefer stable JSON fields when adding `--json`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm required checks are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I6-code`).
5. Remove worktree.


Do not edit planning docs inside the worktree.
