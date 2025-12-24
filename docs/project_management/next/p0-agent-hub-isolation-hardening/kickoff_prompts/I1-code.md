# Task I1-code (Fail-closed semantics) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I1-spec.md`, and this prompt.
3. Set `I1-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I1-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i1-fail-closed-code
   git worktree add wt/ahih-i1-fail-closed-code ahih-i1-fail-closed-code
   cd wt/ahih-i1-fail-closed-code
   ```

## Spec (shared with I1-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I1-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Implement required-world routing semantics (no host fallback) for both non-PTY and PTY paths.
- Keep `world_fs.require_world=false` behavior unchanged (warn once + host fallback).

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm fmt/clippy are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I1-code`).
5. Remove worktree.
