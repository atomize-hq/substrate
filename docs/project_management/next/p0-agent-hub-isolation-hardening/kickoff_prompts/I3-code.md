# Task I3-code (Full cage PTY parity) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I3-spec.md`, and this prompt.
3. Set `I3-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I3-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i3-full-cage-pty-code
   git worktree add wt/ahih-i3-full-cage-pty-code ahih-i3-full-cage-pty-code
   cd wt/ahih-i3-full-cage-pty-code
   ```

## Spec (shared with I3-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I3-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Extend full cage to PTY stream paths with parity to non-PTY, preserving signals/resizing.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm fmt/clippy are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I3-code`).
5. Remove worktree.

