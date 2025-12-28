# Task I9-code (full cage robustness) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I9-spec.md`, and this prompt.
3. Set `I9-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I9-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i9-full-cage-verify-code
   git worktree add wt/ahih-i9-full-cage-verify-code ahih-i9-full-cage-verify-code
   cd wt/ahih-i9-full-cage-verify-code
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I9-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I9-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Fix the full-cage `/tmp`-rooted project failure and align `substrate world verify` full-cage behavior
  with the isolation model.
- Keep fail-closed semantics intact when `world_fs.require_world=true`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm required checks are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I9-code`).
5. Remove worktree.

